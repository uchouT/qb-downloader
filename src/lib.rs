pub mod config;
pub mod error;
pub mod http;
pub mod qb;
pub mod task;
pub mod upload;

pub use crate::error::{Error, format_error_chain};
use directories_next::BaseDirs;
use std::{
    path::PathBuf,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// default workspace directory
pub fn get_base_dir() -> PathBuf {
    BaseDirs::new().unwrap().config_dir().join("qb-downloader")
}

/// remove the trailing slash from a path or host
/// e.g. "http://example.com/" -> "http://example.com"
pub fn remove_slash<T: AsRef<str>>(path_or_host: T) -> String {
    let s = path_or_host.as_ref();
    s.trim().strip_suffix('/').unwrap_or(s).to_string()
}

/// Trait for entities with RwLock
pub trait Entity {
    type LockedValue;
    type Target;

    fn get(
        locked: &'static RwLock<Self::LockedValue>,
    ) -> RwLockReadGuard<'static, Self::LockedValue>;
    fn read<T, F: Fn(&Self::Target) -> T>(f: F) -> T;
    fn get_mut(
        locked: &'static RwLock<Self::LockedValue>,
    ) -> RwLockWriteGuard<'static, Self::LockedValue>;
}

/// Trait for instances with RwLock
pub trait InstanceEntity {
    type LockedValue;
    fn get(&self) -> RwLockReadGuard<'_, Self::LockedValue>;
    fn get_mut(&mut self) -> RwLockWriteGuard<'_, Self::LockedValue>;
    fn write<T, F: FnOnce(&mut Self::LockedValue) -> T>(&mut self, f: F) -> T {
        let mut guard = self.get_mut();
        f(&mut *guard)
    }
    fn read<T, F: Fn(&Self::LockedValue) -> T>(&self, f: F) -> T {
        let read_guard = self.get();
        f(&*&read_guard)
    }
}
