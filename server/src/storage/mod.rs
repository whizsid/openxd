#[cfg(feature = "storage-fs")]
pub mod fs;
#[cfg(feature = "storage-s3")]
pub mod s3;

#[cfg(feature = "storage-fs")]
pub use fs::FileSystemStorage as StorageImpl;
#[cfg(feature = "storage-fs")]
pub use fs::StorageError;
#[cfg(feature = "storage-s3")]
pub use s3::S3Storage as StorageImpl;
#[cfg(feature = "storage-s3")]
pub use s3::StorageError;
