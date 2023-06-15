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

use futures::AsyncReadExt;
use surrealdb::{sql::Id, Connection, Surreal};
use tokio::io::{AsyncBufRead, AsyncWrite, AsyncWriteExt};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::{
    asset::{detect_asset_type_by_ext, GetAssets, ReplaceAsset},
    helpers::remove_symbols_and_extra_spaces,
    model::{thing, Branch, Commit, Project, Session, Snapshot, Tab, User},
    oxd::OxdXml,
    storage::{Storage, StorageId},
    DEFAULT_BRANCH,
};

use serde_xml_rs::{de::from_str as xml_from_str, ser::to_string as xml_to_str};

use async_zip::{
    base::{read::stream::ZipFileReader, write::ZipFileWriter},
    Compression, ZipEntryBuilder,
};

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectUsingExistingFileError<SE: Debug + std::error::Error + Send + Sync> {
    #[error("error occured while reading/writing the data")]
    Io(#[from] std::io::Error),
    #[error("could not decompress the oxd")]
    Zip(#[from] async_zip::error::ZipError),
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
    let mut zip_entry_reader = ZipFileReader::with_tokio(content);
    let mut oxd_content_opt: Option<OxdXml<PathBuf>> = None;
    let mut path_id_map: HashMap<PathBuf, SI> = HashMap::new();
    while let Some(mut entry) = zip_entry_reader.next_with_entry().await? {
        let path = entry.reader().entry().filename().as_str()?;
        let path = PathBuf::from(path);
        match path.clone().extension() {
            Some(ext) => {
                if ext == "xml" {
                    let reader_mut = entry.reader_mut();
                    let mut xml_bytes: Vec<u8> = Vec::new();
                    reader_mut.read_to_end(&mut xml_bytes).await?;
                    let xml_str = from_utf8(&xml_bytes)?;
                    let xml: OxdXml<PathBuf> = xml_from_str(xml_str)?;
                    oxd_content_opt = Some(xml);
                } else {
                    match detect_asset_type_by_ext(ext.to_str().unwrap()) {
                        Some(_) => {
                            let ext = ext.to_str().unwrap();
                            let ext_cloned = String::from(ext);
                            let mut reader = entry.reader_mut().compat();
                            let id = storage
                                .put(
                                    &mut reader,
                                    format!("session/{}/assets", project_id),
                                    ext_cloned,
                                )
                                .await
                                .map_err(CreateProjectUsingExistingFileError::Storage)?;
                            path_id_map.insert(path, id);
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

        zip_entry_reader = entry.skip().await?;
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

pub async fn get_current_tab<D: Connection>(
    db: Arc<Surreal<D>>,
    user_id: String,
) -> Result<Tab, GetCurrentTabSnapshotError> {
    let mut sessions = db.query("SELECT * FROM type::table($table) WHERE user=type::thing($user_id) AND closed_at IS none ORDER BY last_activity DESC LIMIT 1")
        .bind(("table", Session::TABLE))
        .bind(("user_id", thing(User::TABLE, user_id.clone())))
        .await?;
    let session: Option<Session> = sessions.take(0)?;

    if let Some(session) = session {
        if let Some(current_tab) = session.current_tab {
            let tab: Option<Tab> = db.select(current_tab).await?;
            let tab = tab.unwrap();

            Ok(tab)
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
    #[error("could not compress the oxd")]
    Zip(#[from] async_zip::error::ZipError),
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
    let snapshot: Option<Snapshot<SI>> = db.select((Snapshot::<SI>::TABLE, &snapshot_id)).await?;

    if snapshot.is_none() {
        return Err(ExportSnapshotError::NotFound);
    }

    let snapshot = snapshot.unwrap();

    let oxd = snapshot.oxd;

    let storage_objs = oxd.get_assets();

    let mut zip_writer = ZipFileWriter::with_tokio(body);

    let mut replace: HashMap<SI, PathBuf> = HashMap::new();
    for (i, id) in storage_objs.iter().enumerate() {
        let mut obj = storage
            .get(id.clone())
            .await
            .map_err(ExportSnapshotError::Storage)?;
        let info = storage
            .info(id.clone())
            .await
            .map_err(ExportSnapshotError::Storage)?;

        let path = if let Some(ext) = info.ext {
            format!("{}.{}", i, &ext)
        } else {
            i.to_string()
        };
        let opts = ZipEntryBuilder::new(path.clone().into(), Compression::Xz);
        let entry_writer = zip_writer.write_entry_stream(opts).await?;
        let mut compat_entry_writer = entry_writer.compat_write();
        tokio::io::copy(&mut obj, &mut compat_entry_writer).await?;
        compat_entry_writer.flush().await?;
        replace.insert(id.clone(), PathBuf::from(path));
        compat_entry_writer.shutdown().await?;
    }

    let replaced_oxd = oxd.replace_asset(&mut replace);
    let xml = xml_to_str(&replaced_oxd)?;
    let xml_bytes = xml.into_bytes();

    let opts = ZipEntryBuilder::new(format!("oxd.xml").into(), Compression::Xz);
    zip_writer.write_entry_whole(opts, &xml_bytes).await?;
    zip_writer.close().await?;

    Ok(())
}
