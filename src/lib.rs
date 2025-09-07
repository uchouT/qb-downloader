pub mod app;
mod bencode;
mod config;
pub mod error;
mod qb;
mod request;
mod server;
mod task;
mod upload;

use crate::error::{Error, format_error_chain};
use crate::server::api::auth;
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// remove the trailing slash from a path or host.
/// # Example
/// ```
/// use qb_downloader_rust::remove_slash;
/// let path = "/home/user/torrents/";
/// assert_eq!(remove_slash(path), "/home/user/torrents");
/// let host = "http://localhost:8080/";
/// assert_eq!(remove_slash(host), "http://localhost:8080");
/// ```
fn remove_slash<T: AsRef<str>>(path_or_host: T) -> String {
    let s = path_or_host.as_ref();
    s.trim().strip_suffix('/').unwrap_or(s).to_string()
}
