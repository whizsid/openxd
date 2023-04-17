use dotenv_codegen::dotenv;

pub const WS_HOST:&str = dotenv!("WS_HOST");
pub const WS_PORT:&str = dotenv!("WS_PORT");
pub const WS_PATH:&str = dotenv!("WS_PATH");
