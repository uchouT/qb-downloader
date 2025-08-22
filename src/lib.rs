pub mod app;
pub mod config;
mod entity;
pub mod error;
pub mod qb;
pub mod request;
pub mod server;
pub mod task;
pub mod upload;
pub mod bencode;

pub use crate::server::api::auth;
pub use crate::error::{Error, format_error_chain};
pub use entity::Entity;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// remove the trailing slash from a path or host.
/// # Example
/// ```
/// use qb_downloader_rust::remove_slash;
/// let path = "/home/user/torrents/";
/// assert_eq!(remove_slash(path), "/home/user/torrents");
/// let host = "http://localhost:8080/";
/// assert_eq!(remove_slash(host), "http://localhost:8080");
/// ```
pub fn remove_slash<T: AsRef<str>>(path_or_host: T) -> String {
    let s = path_or_host.as_ref();
    s.trim().strip_suffix('/').unwrap_or(s).to_string()
}
