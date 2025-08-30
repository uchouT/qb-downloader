use crate::{
    Entity,
    auth::{TOKEN, encode},
    error::CommonError,
    remove_slash,
};
use directories_next::BaseDirs;
use log::{debug, info};
use serde::{Deserialize, Deserializer, Serialize};
use std::{path::PathBuf, sync::OnceLock};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

const CONFIG_FILE_NAME: &str = "config.toml";

pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigValue {
    #[serde(deserialize_with = "strip_slash")]
    pub qb_host: String,
    pub qb_username: String,
    pub qb_password: String,
    #[serde(deserialize_with = "strip_slash")]
    pub rclone_host: String,
    pub rclone_username: String,
    pub rclone_password: String,
    pub is_only_inner_ip: bool,
    pub multi_login: bool,
    pub account: Account,
    #[serde(deserialize_with = "strip_slash")]
    pub default_save_path: String,
    #[serde(deserialize_with = "strip_slash")]
    pub default_upload_path: String,
    pub default_ratio_limit: Option<f64>,
    pub default_seeding_time_limit: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub username: String,
    pub password: String,
}

impl Default for Account {
    fn default() -> Self {
        let password = encode("adminadmin");
        Self {
            username: String::from("admin"),
            password,
        }
    }
}
impl Default for ConfigValue {
    fn default() -> Self {
        ConfigValue {
            qb_host: String::from("http://localhost:8080"),
            qb_username: String::from("admin"),
            qb_password: String::from("adminadmin"),
            rclone_host: String::from("http://localhost:5572"),
            rclone_username: String::from("admin"),
            rclone_password: String::from("password"),
            is_only_inner_ip: false,
            multi_login: true,
            account: Account::default(),
            default_save_path: String::new(),
            default_upload_path: String::new(),
            default_ratio_limit: Some(-2.0),
            default_seeding_time_limit: Some(-2),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub value: ConfigValue,
    pub filepath: PathBuf,
}

impl Entity for Config {
    fn new(filepath: Option<PathBuf>) -> Self {
        let filepath = if let Some(path) = filepath {
            path
        } else {
            BaseDirs::new()
                .expect("Failed to get config dir.")
                .config_dir()
                .join("qb-downloader")
                .join(CONFIG_FILE_NAME)
        };

        Config {
            value: ConfigValue::default(),
            filepath,
        }
    }

    fn init(path: Option<PathBuf>) -> Result<(), CommonError> {
        let mut config = Config::new(path);
        TOKEN
            .set(std::sync::RwLock::new(String::new()))
            .expect("Failed to set global token");
        Config::load(&mut config)?;
        debug!("Config loaded from: {}", &config.filepath.display());
        debug!("Config content: {:?}", &config.value);
        CONFIG
            .set(RwLock::new(config))
            .expect("Failed to set global config");
        info!("Config loaded.");
        Ok(())
    }

    async fn get() -> RwLockReadGuard<'static, Self::LockedValue> {
        CONFIG.get().expect("Config not initialized").read().await
    }
    async fn get_mut() -> RwLockWriteGuard<'static, Self::LockedValue> {
        CONFIG.get().expect("Config not initialized").write().await
    }
}

/// nomalize the path and host field
pub fn strip_slash<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(remove_slash(&s))
}
