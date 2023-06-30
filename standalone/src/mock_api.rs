use std::{fmt::Debug, path::Path, sync::Arc};

use app::external::{
    create_project_using_existing_file, export_snapshot, get_current_tab,
    CreateProjectUsingExistingFileError, ExportSnapshotError, GetCurrentTabSnapshotError,
};
use async_trait::async_trait;
use rfd::{AsyncFileDialog, FileHandle};
use surrealdb::{engine::local::Db, Surreal};
use tokio::fs::OpenOptions;
use ui::external::External;

use crate::{
    fs::{FileSystemStorage, StorageError},
    USER_ID,
};

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

impl<SE: Debug + std::error::Error + Send + Sync> From<MockApiError<SE>> for String {
    fn from(value: MockApiError<SE>) -> Self {
        format!("{:?}", value)
    }
}

impl<SE: Debug + std::error::Error + Send + Sync> From<GetCurrentTabSnapshotError>
    for MockApiError<SE>
{
    fn from(value: GetCurrentTabSnapshotError) -> Self {
        MockApiError::GetCurrentTabSnapshot(value)
    }
}

impl<SE: Debug + std::error::Error + Send + Sync> From<tokio::io::Error> for MockApiError<SE> {
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
    async fn create_project_using_existing_file(
        &self,
        buf: Vec<u8>,
        file_name: String,
    ) -> Result<String, String> {
        let mut buf_reader: &[u8] = buf.as_slice();
        let userid = String::from(USER_ID);
        let project = create_project_using_existing_file(
            self.db.clone(),
            self.storage.clone(),
            &mut buf_reader,
            file_name,
            userid,
        )
        .await
        .map_err(MockApiError::CreateProjectUsingExistingFile)?;
        Ok(project.id.unwrap().id.to_string())
    }

    async fn save_current_snapshot(&self) -> Result<(), String> {
        let userid = String::from(USER_ID);
        let current_tab = get_current_tab(self.db.clone(), userid)
            .await
            .map_err(MockApiError::<StorageError>::from)?;
        let file_dialog = AsyncFileDialog::new();
        let choosed_file: Option<FileHandle> = file_dialog
            .add_filter("OpenXD", &["oxd"])
            .set_file_name(&current_tab.name)
            .save_file()
            .await;

        if let Some(file_handle) = choosed_file {
            let path: &Path = file_handle.path();

            let extension = path.extension();

            let mut append_extension = false;
            match extension {
                Some(ext_str) => match ext_str.to_str() {
                    Some(ext) if ext == "oxd" => {}
                    _ => {
                        append_extension = true;
                    }
                },
                None => {
                    append_extension = true;
                }
            }

            let new_path = if append_extension {
                path.with_extension("oxd")
            } else {
                path.to_path_buf()
            };

            let file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .open(new_path)
                .await
                .map_err(MockApiError::<StorageError>::from)?;

            export_snapshot(
                self.db.clone(),
                self.storage.clone(),
                file,
                current_tab.snapshot.id.to_string(),
            )
            .await
            .map_err(MockApiError::<StorageError>::from)?;
        }

        Ok(())
    }
}
