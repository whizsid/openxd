//! Extra operations that should be communicated using external method
//! due to large data size
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
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWrite};

use crate::{
    asset::{detect_asset_type_by_ext, GetAssets, ReplaceAsset},
    helpers::remove_symbols_and_extra_spaces,
    model::{Branch, Commit, Project, Session, Tab, thing, User, Snapshot},
    oxd::OxdXml,
    storage::{Storage, StorageId},
    DEFAULT_BRANCH,
};

#[cfg(feature="compression-zstd")]
use async_compression::tokio::{bufread::ZstdDecoder as Decoder, write::ZstdEncoder as Encoder};
#[cfg(feature="compression-xz")]
use async_compression::tokio::{bufread::XzDecoder as Decoder, write::XzEncoder as Encoder};
#[cfg(feature="compression-zlib")]
use async_compression::tokio::{bufread::ZlibDecoder as Decoder, write::ZlibEncoder as Encoder};
#[cfg(feature="compression-lzma")]
use async_compression::tokio::{bufread::LzmaDecoder as Decoder, write::LzmaEncoder as Encoder};
#[cfg(feature="compression-gzip")]
use async_compression::tokio::{bufread::GzipDecoder as Decoder, write::GzipEncoder as Encoder};
#[cfg(feature="compression-deflate")]
use async_compression::tokio::{bufread::DeflateDecoder as Decoder, write::DeflateEncoder as Encoder};
#[cfg(feature="compression-brotli")]
use async_compression::tokio::{bufread::BrotliDecoder as Decoder, write::BrotliEncoder as Encoder};
#[cfg(feature="compression-bzip2")]
use async_compression::tokio::{bufread::BzDecoder as Decoder, write::BzEncoder as Encoder};

use serde_xml_rs::{de::from_str as xml_from_str, ser::to_string as xml_to_str};
use tokio_tar::{Archive, Builder, Header};

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectUsingExistingFileError<SE: Debug + std::error::Error + Send + Sync> {
    #[error("error occured while reading/writing the data")]
    Io(#[from] std::io::Error),
    #[error("unsupported asset format {path}")]
    UnsupportedAsset { path: String },
    #[error("data not properly formatted.")]
    Serde(#[from] serde_xml_rs::Error),
    #[error("failed to encode the provided text")]
    Utf8(#[from] Utf8Error),
    #[error("unsupported file.")]
    UnsupportedFile,
    #[error("could not read/write the data from database")]
    Db(#[from] surrealdb::Error),
    #[error("asset upload/download error")]
    Storage(SE),
}

/// Create a new project using an existing oxd file
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
    let decoder = Decoder::new(content);
    let mut tar = Archive::new(decoder);
    let mut entries = tar.entries()?;
    let mut oxd_content_opt: Option<OxdXml<PathBuf>> = None;
    let mut path_id_map: HashMap<PathBuf, SI> = HashMap::new();
    while let Some(entry) = entries.next().await {
        let mut entry = entry?;
        let path = entry.path()?;
        match path.extension() {
            Some(ext) => {
                if ext == "xml" {
                    let mut xml_bytes: Vec<u8> = Vec::new();
                    entry.read_to_end(&mut xml_bytes).await?;
                    let xml_str = from_utf8(&xml_bytes)?;
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
            let replaced_oxd: OxdXml<SI> = oxd_content.replace_asset(&mut path_id_map);
            for (_, v) in path_id_map {
                storage
                    .delete(v)
                    .await
                    .map_err(|e| CreateProjectUsingExistingFileError::Storage(e))?;
            }
            let replaced_snapshot = Snapshot::new(replaced_oxd);
            let created_snapshot: Snapshot<SI> = db
                .create(Snapshot::<SI>::TABLE)
                .content(replaced_snapshot.clone())
                .await?;

            let branch = Branch::new::<SI>(String::from(DEFAULT_BRANCH), None);
            let created_branch: Branch = db.create(Branch::TABLE).content(branch).await?;

            let commit = Commit::new::<SI>(
                String::from("Initial Commit"),
                created_branch.id.clone().unwrap(),
                thing(User::TABLE, user_id.clone()),
                None,
                created_snapshot.id.unwrap(),
            );
            let _created_commit: Commit = db.create(Commit::TABLE).content(commit).await?;

            let file_name_without_sym_spc = remove_symbols_and_extra_spaces(project_name.clone());
            let slug = file_name_without_sym_spc.to_lowercase().replace("", "-");

            let project = Project::new(
                thing(Project::TABLE, project_id),
                String::from(file_name_without_sym_spc),
                slug,
                created_branch.id.unwrap(),
                thing(User::TABLE, user_id),
            );
            let created_project = db.create(Project::TABLE).content(project).await?;

            return Ok(created_project);
        }
        None => {
            return Err(CreateProjectUsingExistingFileError::UnsupportedFile);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetCurrentTabSnapshotError {
    #[error("could not read/write the data from database")]
    Db(#[from] surrealdb::Error),
    #[error("no any active session belongs to user")]
    NoActiveSession,
    #[error("no tab opened by the user")]
    NoTabOpened,
}

pub async fn get_current_tab_snapshot_id<D: Connection> (db: Arc<Surreal<D>>, user_id: String) -> Result<String, GetCurrentTabSnapshotError> {
    let mut sessions = db.query("SELECT * FROM type::table($table) WHERE user=type::thing($user_id) AND closed_at IS none ORDER BY last_activity DESC LIMIT 1")
        .bind(("table", Session::TABLE))
        .bind(("user_id", thing(User::TABLE, user_id.clone())))
        .await?;
    let session: Option<Session> = sessions.take(0)?;

    if let Some(session) = session {
        if let Some(current_tab) = session.current_tab {
            let tab: Option<Tab> = db.select(current_tab).await?;
            let tab = tab.unwrap();

            Ok(tab.snapshot.id.to_string())
        } else {
            Err(GetCurrentTabSnapshotError::NoTabOpened)
        }
    } else {
        Err(GetCurrentTabSnapshotError::NoActiveSession)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportSnapshotError<SE: std::error::Error + Debug + Send + Sync> {
    #[error("could not read/write the data from database")]
    Db(#[from] surrealdb::Error),
    #[error("asset upload/download error")]
    Storage(SE),
    #[error("error occured while reading/writing the data")]
    Io(#[from] std::io::Error),
    #[error("data not properly formatted.")]
    Serde(#[from] serde_xml_rs::Error),
    #[error("not found a snapshot for the id")]
    NotFound,
}
/// Exporting a snapshot to a oxd file
pub async fn export_snapshot<
    D: Connection,
    SE: Debug + std::error::Error + Send + Sync,
    SI: StorageId,
    S: Storage<SE, SI>,
    W: AsyncWrite + Send + Unpin + 'static,
>(
    db: Arc<Surreal<D>>,
    storage: Arc<S>,
    body: W,
    snapshot_id: String,
) -> Result<(), ExportSnapshotError<SE>> {

    let snapshot: Option<Snapshot<SI>> =
        db.select((Snapshot::<SI>::TABLE, &snapshot_id)).await?;

    if snapshot.is_none() {
        return Err(ExportSnapshotError::NotFound);
    }

    let snapshot = snapshot.unwrap();

    let oxd = snapshot.oxd;

    let storage_objs = oxd.get_assets();

    let encoder = Encoder::with_quality(body, async_compression::Level::Best);
    let mut tar_builder = Builder::new(encoder);

    let mut replace: HashMap<SI, PathBuf> = HashMap::new();
    for (i, id) in storage_objs.iter().enumerate() {
        let obj = storage
            .get(id.clone())
            .await
            .map_err(ExportSnapshotError::Storage)?;
        let info = storage
            .info(id.clone())
            .await
            .map_err(ExportSnapshotError::Storage)?;

        let mut header = Header::new_gnu();
        header.set_size(info.size);
        let mut path = PathBuf::from("./");
        if let Some(ext) = info.ext {
            path = path.join(format!("{}.{}", i, &ext));
        }
        tar_builder
            .append_data(&mut header, path.clone(), obj)
            .await?;
        replace.insert(id.clone(), path);
    }

    let replaced_oxd = oxd.replace_asset(&mut replace);
    let xml = xml_to_str(&replaced_oxd)?;
    let xml_bytes = xml.into_bytes();
    let mut xml_bytes_slice = xml_bytes.as_slice();

    let mut header = Header::new_gnu();
    header.set_size(xml_bytes_slice.len() as u64);
    tar_builder
        .append_data(
            &mut header,
            PathBuf::from("./oxd.xml"),
            &mut xml_bytes_slice,
        )
        .await?;
    tar_builder.finish().await?;

    Ok(())
}
