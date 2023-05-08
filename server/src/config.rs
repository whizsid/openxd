use dotenv_codegen::dotenv;

pub const WS_HOST:&str = dotenv!("WS_HOST");
pub const WS_PORT:&str = dotenv!("WS_PORT");
pub const WS_PATH:&str = dotenv!("WS_PATH");

pub const DB_URL: &str = dotenv!("DB_URL");

#[cfg(feature="storage-fs")]
pub const STORAGE_FS_ROOT: &str = dotenv!("STORAGE_FS_ROOT");
