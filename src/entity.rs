use serde::{Deserialize, Serialize};

use crate::{
    Error,
    config::{CONFIG, Config, ConfigValue},
    task::{TASK_LIST, Task, TaskItem},
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

/// Trait for entities with RwLock
/// # Type Parameters
/// - `LockedValue`: The type of the locked value.
/// - `Target`: The type of the target value required for read and write operations.

pub trait Entity: EntityInternal {
    /// Create a new instance of the entity.
    fn new(path: Option<PathBuf>) -> Self::LockedValue;

    /// Init the value
    fn init(path: Option<PathBuf>) -> Result<(), Error>;

    /// Save the current entity to file, whose path is defined in filepath field.
    fn save() -> Result<(), Error> {
        let entity = Self::get();
        Self::save_entity(&*entity)?;
        Ok(())
    }

    /// Load entity from file
    fn load(entity: &mut Self::LockedValue) -> Result<(), Error> {
        let filepath = Self::filepath(entity);
        if !filepath.exists() {
            return Self::save_entity(entity);
        }
        let content = fs::read_to_string(filepath)?;
        let target: Self::Target = toml::from_str(&content)?;
        let value = Self::mut_value(entity);
        *value = target;
        Ok(())
    }
    fn read<T, F: FnOnce(&Self::Target) -> T>(f: F) -> T {
        let lock = Self::get();
        let target = Self::value(&lock);
        f(target)
    }
    fn write<T, F: FnOnce(&mut Self::Target) -> T>(f: F) -> T {
        let mut lock = Self::get_mut();
        let target = Self::mut_value(&mut lock);
        f(target)
    }
}

/// Internal trait for get lock and get value
pub trait EntityInternal {
    type LockedValue: 'static;
    type Target: Serialize + for<'de> Deserialize<'de>;

    fn get() -> RwLockReadGuard<'static, Self::LockedValue>;
    fn get_mut() -> RwLockWriteGuard<'static, Self::LockedValue>;
    fn value(locked: &Self::LockedValue) -> &Self::Target;
    fn mut_value(locked: &mut Self::LockedValue) -> &mut Self::Target;
    fn filepath(locked: &Self::LockedValue) -> &PathBuf;
    fn save_entity(entity: &Self::LockedValue) -> Result<(), Error> {
        let content = toml::to_string(Self::value(entity))?;
        let file_path = Self::filepath(entity);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Ok(())
    }
}

impl EntityInternal for Task {
    type LockedValue = Task;
    type Target = HashMap<String, TaskItem>;
    fn get() -> std::sync::RwLockReadGuard<'static, Self::LockedValue> {
        TASK_LIST
            .get()
            .expect("task list not initialized")
            .read()
            .expect("Failed to acquire read lock on Task")
    }
    fn get_mut() -> std::sync::RwLockWriteGuard<'static, Self::LockedValue> {
        TASK_LIST
            .get()
            .expect("task list not initialized")
            .write()
            .expect("Failed to acquire write lock on Task")
    }
    fn value(locked: &Self::LockedValue) -> &Self::Target {
        &locked.value
    }
    fn mut_value(locked: &mut Self::LockedValue) -> &mut Self::Target {
        &mut locked.value
    }
    fn filepath(locked: &Self::LockedValue) -> &PathBuf {
        &locked.filepath
    }
}

impl EntityInternal for Config {
    type LockedValue = Config;
    type Target = ConfigValue;

    fn get() -> RwLockReadGuard<'static, Self::LockedValue> {
        CONFIG
            .get()
            .expect("Config not initialized")
            .read()
            .expect("Failed to acquire read lock on Config")
    }
    fn value(locked: &Self::LockedValue) -> &Self::Target {
        &locked.value
    }
    fn mut_value(locked: &mut Self::LockedValue) -> &mut Self::Target {
        &mut locked.value
    }
    fn get_mut() -> RwLockWriteGuard<'static, Self::LockedValue> {
        CONFIG
            .get()
            .expect("Config not initialized")
            .write()
            .expect("Failed to acquire write lock on Config")
    }
    fn filepath(locked: &Self::LockedValue) -> &PathBuf {
        &locked.filepath
    }
}
