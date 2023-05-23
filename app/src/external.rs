//! Extra operations that should be communicated using external method
//!
//! For web:- It should be REST APIs
//! For Desktop:- Direct executions

use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    str::{from_utf8, Utf8Error},
    sync::Arc,
};

use futures::StreamExt;
use regex::Regex;
use surrealdb::{sql::Id, Connection, Surreal};
use tokio::io::{AsyncBufRead, AsyncReadExt};

use crate::{
    asset::{detect_asset_type_by_ext, ReplaceAsset},
    model::{Project, Branch, Commit},
    oxd::OxdXml,
    storage::{Storage, StorageId}, DEFAULT_BRANCH,
};

use async_compression::tokio::bufread::XzDecoder;
use serde_xml_rs::de::from_str as xml_from_str;
use tokio_tar::Archive;

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectUsingExistingFileError<SE: Debug + std::error::Error + Send + Sync> {
    #[error("error occured while reading/writing the data")]
    Io(#[from] std::io::Error),
    #[error("unsupported asset format {path}")]
    UnsupportedAsset{ path: String },
    #[error("data not properly formatted.")]
    Serde(#[from] serde_xml_rs::Error),
    #[error("failed to encode the provided text")]
    Utf8(#[from] Utf8Error),
    #[error("unsupported file.")]
    UnsupportedFile,
    #[error("could not read/write the data from database")]
    Db(#[from] surrealdb::Error),
    #[error("file upload/download error")]
    Storage(SE),
}

pub async fn create_project_using_existing_file<
    D: Connection,
    SE: Debug + std::error::Error + Send + Sync,
    SI: StorageId,
    S: Storage<SE, SI>,
    R: AsyncBufRead + Unpin + Send + Sync,
>(
    db: Arc<Surreal<D>>,
    storage: Arc<S>,
    content: R,
    project_name: String,
    user_id: String,
) -> Result<Project, CreateProjectUsingExistingFileError<SE>> {
    let project_id = Id::rand();
    let xz_decoder = XzDecoder::new(content);
    let mut tar = Archive::new(xz_decoder);
    let mut entries = tar
        .entries()?;
    let mut oxd_content_opt: Option<OxdXml<PathBuf>> = None;
    let mut path_id_map: HashMap<PathBuf, SI> = HashMap::new();
    while let Some(entry) = entries.next().await {
        let mut entry = entry?;
        let path = entry
            .path()?;
        match path.extension() {
            Some(ext) => {
                if ext == "oxd" {
                    let mut xml_bytes: Vec<u8> = Vec::new();
                    entry
                        .read_to_end(&mut xml_bytes)
                        .await?;
                    let xml_str =
                        from_utf8(&xml_bytes)?;
                    let xml: OxdXml<PathBuf> = xml_from_str(xml_str)?;
                    oxd_content_opt = Some(xml);
                } else {
                    match detect_asset_type_by_ext(ext.to_str().unwrap()) {
                        Some(_) => {
                            let ext = ext.to_str().unwrap();
                            let ext_cloned = String::from(ext);
                            let path_new = entry.path().unwrap();
                            let path_buf = path_new.to_path_buf();
                            let id = storage
                                .put(
                                    &mut entry,
                                    format!("session/{}/assets", project_id),
                                    ext_cloned,
                                )
                                .await
                                .map_err(CreateProjectUsingExistingFileError::Storage)?;
                            path_id_map.insert(path_buf, id);
                        }
                        None => {
                            return Err(CreateProjectUsingExistingFileError::UnsupportedAsset {
                                path: path.to_str().unwrap().into(),
                            });
                        }
                    }
                }
            }
            None => {
                return Err(CreateProjectUsingExistingFileError::UnsupportedAsset {
                    path: path.to_str().unwrap().into(),
                });
            }
        }
    }

    match oxd_content_opt.take() {
        Some(oxd_content) => {
            let mut replaced_oxd: OxdXml<SI> = oxd_content.replace_asset(&mut path_id_map);

            for (_, v) in path_id_map {
                storage
                    .delete(v)
                    .await
                    .map_err(|e| CreateProjectUsingExistingFileError::Storage(e))?;
            }
            replaced_oxd.clear_id();
            let created_oxd: OxdXml<SI> = db
                .create(OxdXml::<SI>::TABLE)
                .content(replaced_oxd.clone())
                .await?;


            let branch = Branch::new::<SI>(String::from(DEFAULT_BRANCH), None);
            let created_branch: Branch = db.create(Branch::TABLE)
                .content(branch)
                .await?;

            let commit = Commit::new::<SI>(String::from("Initial Commit"), created_branch.id.clone(), Id::String(user_id.clone()), None, created_oxd.id);
            let _created_commit: Commit = db.create(Commit::TABLE).content(commit).await?;

            let symbol_rgx = Regex::new("^[a-zA-Z0-9\\s]").unwrap();
            let project_name_cloned = project_name.clone();
            let file_name_without_symbols = symbol_rgx.replace(&project_name_cloned, "");
            let multi_space_rgx= Regex::new("\\s+").unwrap();
            let file_name_without_sym_spc = multi_space_rgx.replace(&file_name_without_symbols, " ");
            let slug = file_name_without_sym_spc.to_lowercase().replace("", "-");

            // TODO:- Get the file name with file and replace this
            let project = Project::new(project_id, String::from(file_name_without_sym_spc), slug, created_branch.id, Id::String(user_id));
            let created_project = db
                .create(Project::TABLE)
                .content(project)
                .await?;

            return Ok(created_project);
        }
        None => {
            return Err(CreateProjectUsingExistingFileError::UnsupportedFile);
        }
    }
}
