pub mod config;
pub mod http;

use directories_next::BaseDirs;
use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// default workspace directory
pub fn get_base_dir() -> PathBuf {
    BaseDirs::new().unwrap().config_dir().join("qb-downloader")
}
