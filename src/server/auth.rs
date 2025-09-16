use std::sync::{OnceLock, RwLock};

use crate::config::Account;
use md5::compute;
use serde::Serialize;
pub static TOKEN: OnceLock<RwLock<String>> = OnceLock::new();

/// auth the Authorization header.
/// Returns true if authorized, false otherwise.
pub fn authorize(auth_info: &str) -> bool {
    auth_info == TOKEN.get().unwrap().read().unwrap().as_str()
}

pub fn encode(data: &str) -> String {
    let digest = compute(data);
    format!("{digest:x}")
}

#[derive(Debug, Serialize)]
pub struct Login<'a> {
    pub account: &'a Account,
    pub key: &'a str,
}
