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
use surrealdb::{sql::Id, Connection, Surreal};
use tokio::io::{AsyncBufRead, AsyncReadExt};

use crate::{
    asset::{detect_asset_type_by_ext, ReplaceAsset},
    model::Project,
    oxd::OxdXml,
    storage::{Storage, StorageId},
};

use async_compression::tokio::bufread::XzDecoder;
use serde_xml_rs::de::from_str as xml_from_str;
use tokio_tar::Archive;

#[derive(Debug)]
pub enum CreateProjectUsingExistingFileError<SE: Debug> {
    Storage(SE),
    Io(std::io::Error),
    UnsupportedAsset(String),
    Serde(serde_xml_rs::Error),
    Utf8(Utf8Error),
    UnsupportedFile,
    Db(surrealdb::Error),
}

pub async fn create_project_using_existing_file<
    D: Connection,
    SE: Debug,
    SI: StorageId,
    S: Storage<SE, SI>,
    R: AsyncBufRead + Unpin + Send + Sync,
>(
    db: Arc<Surreal<D>>,
    storage: Arc<S>,
    content: R,
) -> Result<Project, CreateProjectUsingExistingFileError<SE>> {
    let project_id = Id::rand();
    let xz_decoder = XzDecoder::new(content);
    let mut tar = Archive::new(xz_decoder);
    let mut entries = tar
        .entries()
        .map_err(|e| CreateProjectUsingExistingFileError::Io(e))?;
    let mut oxd_content_opt: Option<OxdXml<PathBuf>> = None;
    let mut path_id_map: HashMap<PathBuf, SI> = HashMap::new();
    while let Some(entry) = entries.next().await {
        let mut entry = entry.map_err(CreateProjectUsingExistingFileError::Io)?;
        let path = entry
            .path()
            .map_err(CreateProjectUsingExistingFileError::Io)?;
        match path.extension() {
            Some(ext) => {
                if ext == "oxd" {
                    let mut xml_bytes: Vec<u8> = Vec::new();
                    entry
                        .read_to_end(&mut xml_bytes)
                        .await
                        .map_err(|e| CreateProjectUsingExistingFileError::Io(e))?;
                    let xml_str =
                        from_utf8(&xml_bytes).map_err(CreateProjectUsingExistingFileError::Utf8)?;
                    let xml: OxdXml<PathBuf> = xml_from_str(xml_str)
                        .map_err(CreateProjectUsingExistingFileError::Serde)?;
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
                            return Err(CreateProjectUsingExistingFileError::UnsupportedAsset(
                                path.to_str().unwrap().into(),
                            ));
                        }
                    }
                }
            }
            None => {
                return Err(CreateProjectUsingExistingFileError::UnsupportedAsset(
                    path.to_str().unwrap().into(),
                ));
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
                .content(replaced_oxd)
                .await
                .map_err(CreateProjectUsingExistingFileError::Db)?;

            // TODO:- Get the file name with file and replace this
            let project = Project::new(project_id, String::from("Untitled Project"));
            let created_project = db
                .create(Project::TABLE)
                .content(project)
                .await
                .map_err(CreateProjectUsingExistingFileError::Db)?;

            return Ok(created_project);
        }
        None => {
            return Err(CreateProjectUsingExistingFileError::UnsupportedFile);
        }
    }
}
