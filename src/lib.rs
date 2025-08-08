pub mod config;
mod entity;
pub mod error;
pub mod http;
pub mod qb;
pub mod task;
pub mod upload;

use directories_next::BaseDirs;
use std::path::PathBuf;

pub use crate::error::{Error, format_error_chain};
pub use entity::Entity;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// default workspace directory
pub fn get_base_dir() -> PathBuf {
    BaseDirs::new().unwrap().config_dir().join("qb-downloader")
}

/// remove the trailing slash from a path or host.
/// e.g. "http://example.com/" -> "http://example.com"
pub fn remove_slash<T: AsRef<str>>(path_or_host: T) -> String {
    let s = path_or_host.as_ref();
    s.trim().strip_suffix('/').unwrap_or(s).to_string()
}
