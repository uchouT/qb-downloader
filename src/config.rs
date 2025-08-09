use crate::{Entity, error::CommonError};
use directories_next::BaseDirs;
use log::{debug, info};
use md5::compute;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::OnceLock};
use tokio::sync::RwLock;
const CONFIG_FILE_NAME: &str = "config.toml";

pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigValue {
    pub qb_host: String,
    pub qb_username: String,
    pub qb_password: String,
    pub rclone_host: String,
    pub rclone_username: String,
    pub rclone_password: String,
    pub is_only_inner_ip: bool,
    pub verify_login_ip: bool,
    pub login: Login,
    pub default_save_path: String,
    pub default_upload_path: String,
    pub default_ratio_limit: f64,
    pub default_seeding_time_limit: i32,
}
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Login {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub key: String,
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
            verify_login_ip: true,
            login: Login::default(),
            default_save_path: String::new(),
            default_upload_path: String::new(),
            default_ratio_limit: -2.0,
            default_seeding_time_limit: -2,
        }
    }
}

impl Login {
    fn default() -> Self {
        let digest = compute("adminadmin");
        let password = format!("{digest:x}");
        Login {
            ip: String::from(""),
            username: String::from("admin"),
            password,
            key: String::from(""),
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
        Config::load(&mut config)?;
        debug!("Config loaded from: {}", &config.filepath.display());
        debug!("Config content: {:?}", &config.value);
        CONFIG
            .set(RwLock::new(config))
            .expect("Failed to set global config");
        info!("Config loaded.");
        Ok(())
    }
}
