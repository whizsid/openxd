use std::{array::TryFromSliceError, fmt::Debug};

use app::external::{CreateProjectUsingExistingFileError, GetCurrentTabSnapshotError, ExportSnapshotError};
use hmac::digest::InvalidLength;
use hyper::header::ToStrError;


#[derive(thiserror::Error, Debug)]
pub enum Error <SE: Debug + std::error::Error + Send + Sync> {
    /// Errors related to database connection
    #[error("failed to read/write data to database")]
    Db(#[from] surrealdb::Error),
    /// Errors related to external storage (filesystem/ AWS S3)
    #[error("failed to download/upload some files to storage.")]
    Storage(SE),
    /// Errors related to IO
    #[error("failed to read/write data")]
    Io(#[from] std::io::Error),
    /// Errors occured when parsing multipart form data requests
    #[error("failed to parse multipart body data")]
    Multer(#[from] multer::Error),
    #[error(transparent)]
    WebSocketOpen(#[from] WebSocketOpenError),
    #[error(transparent)]
    CreateProject(#[from] CreateProjectError<SE>),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    CurrentSnapshot(#[from] GetCurrentTabSnapshotError),
    #[error(transparent)]
    SnapshotDownload(#[from] SnapshotDownloadError<SE>),
}

#[derive(thiserror::Error, Debug)]
pub enum WebSocketOpenError {
    #[error("ticket is expired {ticket_id}, ticket expired at {closed_at}.")]
    ExpiredTicket {
        ticket_id: String,
        closed_at: surrealdb::sql::Datetime
    },
    #[error("ticket is still pending {ticket_id}, ticket opened at {opened_at}.")]
    PendingTicket {
        ticket_id: String,
        opened_at: surrealdb::sql::Datetime
    },
    #[error("ticket id is not valid")]
    TryFromSlice(#[from] TryFromSliceError),
    #[error("ticket not found {ticket_id}.")]
    TicketNotFound {ticket_id: String},
    #[error("ticket not provided.")]
    TicketIdNotProvided
}

#[derive(thiserror::Error, Debug)]
pub enum CreateProjectError<SE: Debug + std::error::Error + Send + Sync> {
    #[error("boundary not found in multipart request body.")]
    BoundaryNotProvided,
    #[error("project name not provided in request")]
    ProjectNameNotProvided,
    #[error("file not provided in request")]
    FileNotProvided,
    #[error(transparent)]
    Inner(#[from] CreateProjectUsingExistingFileError<SE>)
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("authorization header not provided")]
    HeaderNotProvided,
    #[error("encoding failed for authorization header value")]
    HeaderToStr(#[from] ToStrError),
    #[error("invalid authorization method provided")]
    NotBearer,
    #[error("bearer token not provided")]
    BearerNotProvided,
    #[error("can not verify the token")]
    Jwt(#[from] jwt::Error),
    #[error("token didn't matched the requirements")]
    InvalidLength(#[from] InvalidLength)
}

#[derive(thiserror::Error, Debug)]
pub enum SnapshotDownloadError<SE: Debug + std::error::Error + Send + Sync> {
    #[error("invalid request")]
    Invalid { download_id: String },
    #[error("already downloaded the snapshot for given request")]
    AlreadyDownloaded {download_id: String},
    #[error(transparent)]
    ExportError (#[from] ExportSnapshotError<SE>)
}
