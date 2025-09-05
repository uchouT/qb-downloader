use crate::{
    auth::{TOKEN, encode},
    error::CommonError,
    remove_slash,
};
use directories_next::BaseDirs;
use futures_util::future::join3;
use log::{debug, error, info};
use serde::{Deserialize, Deserializer, Serialize};
use std::{path::PathBuf, sync::OnceLock};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
const CONFIG_FILE_NAME: &str = "config.toml";

static CONFIG: OnceLock<Config> = OnceLock::new();

/// get config value
pub fn value() -> &'static ConfigValue {
    &CONFIG.get().unwrap().value
}

/// set config value, frontend send the whole config value,
/// and set the global config value to it
pub async fn set_value(config_value: ConfigValueTemplate) {
    let cfg = value();
    let (mut qb_config, mut rclone_config, mut general_config) =
        join3(cfg.qb_mut(), cfg.rclone_mut(), cfg.general_mut()).await;
    *qb_config = config_value.qb;
    *rclone_config = config_value.rclone;
    *general_config = config_value.general;
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

#[derive(Debug, Default)]
pub struct ConfigValue {
    qb: RwLock<QbConfig>,
    rclone: RwLock<RcloneConfig>,
    general: RwLock<GeneralConfig>,
}

impl ConfigValue {
    pub async fn qb(&self) -> RwLockReadGuard<'_, QbConfig> {
        self.qb.read().await
    }

    pub async fn qb_mut(&self) -> RwLockWriteGuard<'_, QbConfig> {
        self.qb.write().await
    }

    pub async fn rclone(&self) -> RwLockReadGuard<'_, RcloneConfig> {
        self.rclone.read().await
    }

    pub async fn rclone_mut(&self) -> RwLockWriteGuard<'_, RcloneConfig> {
        self.rclone.write().await
    }

    pub async fn general(&self) -> RwLockReadGuard<'_, GeneralConfig> {
        self.general.read().await
    }

    pub async fn general_mut(&self) -> RwLockWriteGuard<'_, GeneralConfig> {
        self.general.write().await
    }

    pub async fn to_template(&self) -> ConfigValueTemplate {
        let (qb, rclone, general) = join3(self.qb(), self.rclone(), self.general()).await;
        ConfigValueTemplate {
            qb: qb.clone(),
            rclone: rclone.clone(),
            general: general.clone(),
        }
    }
}

// intermediate struct for deserializing config file
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigValueTemplate {
    pub qb: QbConfig,
    pub rclone: RcloneConfig,
    pub general: GeneralConfig,
}

#[derive(Debug)]
pub struct Config {
    pub value: ConfigValue,
    pub filepath: PathBuf,
}

impl Config {
    fn new(filepath: PathBuf) -> Self {
        Self {
            value: ConfigValue::default(),
            filepath,
        }
    }

    /// load config from the given file
    /// # Error
    /// If the file cannot be read or parsed, it will return a [`CommonError`].
    fn load(filepath: PathBuf) -> Result<Self, CommonError> {
        let content = std::fs::read_to_string(&filepath).inspect_err(|_| {
            error!("Failed to read config file {}", filepath.display());
        })?;
        let config_value: ConfigValueTemplate = toml::from_str(&content)?;
        Ok(Self {
            filepath: filepath,
            value: ConfigValue {
                qb: RwLock::new(config_value.qb),
                rclone: RwLock::new(config_value.rclone),
                general: RwLock::new(config_value.general),
            },
        })
    }
}

/// save the config to target file, which is stored in filepath field
pub async fn save() -> Result<(), CommonError> {
    let value = value().to_template().await;
    let content = toml::to_string_pretty(&value)?;
    let filepath = &CONFIG.get().unwrap().filepath;
    if let Some(parent) = filepath.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(filepath, content).await?;
    Ok(())
}

pub fn init(path: Option<PathBuf>) -> Result<(), CommonError> {
    // 有 path 或者 为 None 时，尝试 load，否则创建默认配置
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

    debug!("Config loaded from: {}", &config.filepath.display());
    debug!("Config content: {:?}", &config.value);
    CONFIG.set(config).expect("Failed to set global config");
    info!("Config loaded.");
    Ok(())
}

/// nomalize the path and host field
pub fn strip_slash<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(remove_slash(&s))
}
