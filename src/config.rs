use log::info;
use md5::compute;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{CONFIG_FILE_NAME, get_base_dir};

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
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

impl ConfigValue {
    pub fn default() -> Self {
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
        let password = format!("{:x}", digest);
        Login {
            ip: String::from(""),
            username: String::from("admin"),
            password,
            key: String::from(""),
        }
    }
}

pub struct Config {
    value: ConfigValue,
    filepath: PathBuf,
}

impl Config {
    pub fn new(filepath: Option<PathBuf>) -> Self {
        let filepath = if let Some(path) = filepath {
            path
        } else {
            get_base_dir().join(CONFIG_FILE_NAME)
        };

        Config {
            value: ConfigValue::default(),
            filepath,
        }
    }
}

/// initialize the global configuration
pub fn init(custom_file_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut config = Config::new(custom_file_path);
    load(&mut config)?;
    CONFIG.set(RwLock::new(config)).map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Config already initialized",
        ))
    })?;
    info!("Config loaded.");
    Ok(())
}

/// load config from file
fn load(config: &mut Config) -> Result<(), Box<dyn Error>> {
    if !config.filepath.exists() {
        // If the file does not exist, create it with default values
        return save_config(config);
    }
    let content = fs::read_to_string(&config.filepath)?;
    let config_value: ConfigValue = toml::from_str(&content)?;
    config.value = config_value;
    Ok(())
}

/// save the current configuration to file
pub fn save() -> Result<(), Box<dyn Error>> {
    let config = get();
    save_config(&*config)?;
    Ok(())
}

fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string_pretty(&config.value)?;
    let config_filepath = &config.filepath;

    if let Some(parent) = config_filepath.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_filepath, content)?;
    Ok(())
}

/// Get the global configuration read lock
pub fn get() -> RwLockReadGuard<'static, Config> {
    CONFIG
        .get()
        .expect("Config not initialized")
        .read()
        .expect("Config lock poisoned")
}

/// Get the global configuration write lock
pub fn get_mut() -> RwLockWriteGuard<'static, Config> {
    CONFIG
        .get()
        .expect("Config not initialized")
        .write()
        .expect("Config lock poisoned")
}
