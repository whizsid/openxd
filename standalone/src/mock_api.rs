use std::{fmt::Debug, sync::Arc, path::Path};

use app::external::{
    create_project_using_existing_file, get_current_tab_snapshot_id,
    CreateProjectUsingExistingFileError, GetCurrentTabSnapshotError, export_snapshot, ExportSnapshotError,
};
use async_trait::async_trait;
use rfd::{AsyncFileDialog, FileHandle};
use surrealdb::{engine::local::Db, Surreal};
use tokio::fs::OpenOptions;
use ui::external::External;

use crate::fs::{FileSystemStorage, StorageError};

pub struct MockApi {
    db: Arc<Surreal<Db>>,
    storage: Arc<FileSystemStorage>,
}

#[derive(Debug)]
pub enum MockApiError<SE: Debug + std::error::Error + Send + Sync> {
    CreateProjectUsingExistingFile(CreateProjectUsingExistingFileError<SE>),
    GetCurrentTabSnapshot(GetCurrentTabSnapshotError),
    ExportSnapshot(ExportSnapshotError<SE>),
    Io(tokio::io::Error),
}

impl<SE: Debug + std::error::Error + Send + Sync> From<GetCurrentTabSnapshotError>
    for MockApiError<SE>
{
    fn from(value: GetCurrentTabSnapshotError) -> Self {
        MockApiError::GetCurrentTabSnapshot(value)
    }
}

impl<SE: Debug + std::error::Error + Send + Sync> From<tokio::io::Error>
    for MockApiError<SE>
{
    fn from(value: tokio::io::Error) -> Self {
        MockApiError::Io(value)
    }
}

impl<SE: Debug + std::error::Error + Send + Sync> From<ExportSnapshotError<SE>>
    for MockApiError<SE>
{
    fn from(value: ExportSnapshotError<SE>) -> Self {
        MockApiError::ExportSnapshot(value)
    }
}

impl MockApi {
    pub fn new(db: Arc<Surreal<Db>>, storage: Arc<FileSystemStorage>) -> MockApi {
        MockApi { db, storage }
    }
}

#[async_trait]
impl External for MockApi {
    type Error = MockApiError<StorageError>;

    async fn create_project_using_existing_file(
        self: Arc<Self>,
        buf: Vec<u8>,
        file_name: String,
    ) -> Result<String, Self::Error> {
        let mut buf_reader: &[u8] = buf.as_slice();
        let userid = String::from("currentuser");
        let project = create_project_using_existing_file(
            self.db.clone(),
            self.storage.clone(),
            &mut buf_reader,
            file_name,
            userid,
        )
        .await
        .map_err(MockApiError::CreateProjectUsingExistingFile)?;
        Ok(project.id.to_string())
    }

    async fn save_current_snapshot(self: Arc<Self>) -> Result<(), Self::Error> {
        let userid = String::from("currentuser");
        let current_snapshot_id = get_current_tab_snapshot_id(self.db.clone(), userid).await?;
        let file_dialog = AsyncFileDialog::new();
        let choosed_file: Option<FileHandle> = file_dialog
            .add_filter("OpenXD", &["oxd"])
            .set_file_name("Untitled Project")
            .save_file()
            .await;

        if let Some(file_handle) = choosed_file {
            let path: &Path = file_handle.path();
            let file = OpenOptions::new().write(true).read(true).create(true).truncate(true).open(path).await?;
            export_snapshot(self.db.clone(), self.storage.clone(), file, current_snapshot_id).await?;
        }

        Ok(())
    }
}
