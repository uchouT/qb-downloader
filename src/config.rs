use crate::{
    auth::{TOKEN, encode},
    errors::{CommonError, ResultExt},
    remove_slash,
};
use arc_swap::{ArcSwap, Guard};
use directories_next::BaseDirs;

use log::{debug, info};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

const CONFIG_FILE_NAME: &str = "config.toml";

pub static CONFIG: OnceLock<Config> = OnceLock::new();

/// get config value
pub fn value() -> Guard<Arc<ConfigValue>> {
    CONFIG.get().unwrap().value.load()
}

/// set config value, frontend send the whole config value,
/// and set the global config value to it
pub fn set_value(config_value: Arc<ConfigValue>) {
    CONFIG.get().unwrap().value.store(config_value);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QbConfig {
    #[serde(deserialize_with = "strip_slash")]
    pub qb_host: String,
    pub qb_username: String,
    pub qb_password: String,
    #[serde(deserialize_with = "strip_slash")]
    pub default_save_path: String,
    pub default_ratio_limit: Option<f64>,
    pub default_seeding_time_limit: Option<i32>,
}

impl Default for QbConfig {
    fn default() -> Self {
        Self {
            qb_host: String::from("http://localhost:8080"),
            qb_username: String::from("admin"),
            qb_password: String::from("adminadmin"),
            default_ratio_limit: Some(-2.0),
            default_seeding_time_limit: Some(-2),
            default_save_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RcloneConfig {
    #[serde(deserialize_with = "strip_slash")]
    pub rclone_host: String,
    pub rclone_username: String,
    pub rclone_password: String,
}

impl Default for RcloneConfig {
    fn default() -> Self {
        Self {
            rclone_host: String::from("http://localhost:5572"),
            rclone_username: String::from("admin"),
            rclone_password: String::from("password"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub is_only_inner_ip: bool,
    pub multi_login: bool,
    pub account: Account,
    #[serde(deserialize_with = "strip_slash")]
    pub default_upload_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub username: String,
    pub password: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            is_only_inner_ip: false,
            multi_login: true,
            account: Account::default(),
            default_upload_path: String::new(),
        }
    }
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigValue {
    pub qb: QbConfig,
    pub rclone: RcloneConfig,
    pub general: GeneralConfig,
}

#[derive(Debug)]
pub struct Config {
    pub value: ArcSwap<ConfigValue>,
    pub filepath: PathBuf,
}

impl Config {
    fn new(filepath: PathBuf) -> Self {
        Self {
            value: ArcSwap::from_pointee(ConfigValue::default()),
            filepath,
        }
    }

    /// load config from the given file
    /// # Error
    /// If the file cannot be read or parsed, it will return a [`CommonError`].
    fn load(filepath: PathBuf) -> Result<Self, CommonError> {
        let content = std::fs::read_to_string(&filepath)
            .add_context(format!("Failed to read config file {}", filepath.display()))?;
        let config_value: ConfigValue =
            toml::from_str(&content).add_context("Failed to parse config file")?;
        Ok(Self {
            filepath,
            value: ArcSwap::from_pointee(config_value),
        })
    }
}

/// save the config to target file, which is stored in filepath field
pub async fn save() -> Result<(), CommonError> {
    let value = value();
    let content =
        toml::to_string_pretty(value.as_ref()).add_context("Failed to serialize config file")?;
    let filepath = &CONFIG.get().unwrap().filepath;
    if let Some(parent) = filepath.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .add_context("Failed to create config directory")?;
    }
    tokio::fs::write(filepath, content)
        .await
        .add_context("Failed to write config file")?;
    Ok(())
}

pub fn init(path: Option<PathBuf>) -> Result<(), CommonError> {
    let config = {
        let path = match path {
            Some(p) => p,
            None => BaseDirs::new()
                .expect("Failed to get config dir.")
                .config_dir()
                .join("qb-downloader")
                .join(CONFIG_FILE_NAME),
        };

        if path.exists() {
            Config::load(path)?
        } else {
            Config::new(path)
        }
    };
    TOKEN
        .set(std::sync::RwLock::new(String::new()))
        .expect("Failed to set global token");

    info!("Config loading from: {}", &config.filepath.display());
    debug!("Config content: {:?}", &config.value);
    CONFIG.set(config).expect("Failed to set global config");
    Ok(())
}

/// nomalize the path and host field
pub fn strip_slash<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(remove_slash(&s))
}
