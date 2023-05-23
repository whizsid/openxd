use dotenv_codegen::dotenv;

pub const WS_HOST:&str = dotenv!("WS_HOST");
pub const WS_PORT:&str = dotenv!("WS_PORT");
pub const WS_PATH:&str = dotenv!("WS_PATH");

pub const DB_URL: &str = dotenv!("DB_URL");
pub const DB_NAME: &str = dotenv!("DB_NAME");
pub const DB_NAMESPACE: &str = dotenv!("DB_NAMESPACE");
pub const DB_USER: &str = dotenv!("DB_USER");
pub const DB_PASSWORD: &str = dotenv!("DB_PASSWORD");

#[cfg(feature="storage-fs")]
pub const STORAGE_FS_ROOT: &str = dotenv!("STORAGE_FS_ROOT");

pub const JWT_SECRET: &str = dotenv!("JWT_SECRET");
