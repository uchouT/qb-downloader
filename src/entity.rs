mod sealed {
    use crate::{
        config::{CONFIG, Config, ConfigValue},
        error::CommonError,
        task::{TASK_LIST, Task, TaskItem, TaskMap, TaskValue},
    };
    use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

    use tokio::{
        fs,
        sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    };
    /// Internal trait for get lock and get value
    pub trait EntityInternal {
        type LockedValue: 'static;
        type Target;

        fn get() -> impl Future<Output = RwLockReadGuard<'static, Self::LockedValue>> + Send;
        fn get_mut() -> impl Future<Output = RwLockWriteGuard<'static, Self::LockedValue>> + Send;
        fn value(locked: &Self::LockedValue) -> &Self::Target;
        fn mut_value(locked: &mut Self::LockedValue) -> &mut Self::Target;
        fn filepath(locked: &Self::LockedValue) -> &PathBuf;
        fn serialize(
            entity: &Self::Target,
        ) -> impl Future<Output = Result<String, CommonError>> + Send;
        fn deserialize(data: &str) -> Result<Self::Target, CommonError>;
        fn save_entity(
            entity: &Self::LockedValue,
        ) -> impl Future<Output = Result<(), CommonError>> {
            async move {
                let content = Self::serialize(Self::value(entity)).await?;
                let file_path = Self::filepath(entity);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                fs::write(file_path, content).await?;
                Ok(())
            }
        }
    }

    impl EntityInternal for Task {
        type LockedValue = Task;
        type Target = TaskMap;
        async fn get() -> RwLockReadGuard<'static, Self::LockedValue> {
            TASK_LIST
                .get()
                .expect("task list not initialized")
                .read()
                .await
        }
        async fn get_mut() -> RwLockWriteGuard<'static, Self::LockedValue> {
            TASK_LIST
                .get()
                .expect("task list not initialized")
                .write()
                .await
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

        async fn serialize(entity: &Self::Target) -> Result<String, CommonError> {
            let mut content = BTreeMap::new();
            for (k, v) in entity {
                content.insert(k.as_str(), v.0.read().await.clone());
            }
            let serialized = serde_json::to_string(&content)?;
            Ok(serialized)
        }

        fn deserialize(data: &str) -> Result<Self::Target, CommonError> {
            let content: BTreeMap<String, TaskValue> = serde_json::from_str(data)?;
            let mut deserialized = BTreeMap::new();
            for (k, v) in content {
                deserialized.insert(k, Arc::new(TaskItem(RwLock::new(v))));
            }
            Ok(deserialized)
        }
    }

    impl EntityInternal for Config {
        type LockedValue = Config;
        type Target = ConfigValue;

        async fn get() -> RwLockReadGuard<'static, Self::LockedValue> {
            CONFIG.get().expect("Config not initialized").read().await
        }
        fn value(locked: &Self::LockedValue) -> &Self::Target {
            &locked.value
        }
        fn mut_value(locked: &mut Self::LockedValue) -> &mut Self::Target {
            &mut locked.value
        }
        async fn get_mut() -> RwLockWriteGuard<'static, Self::LockedValue> {
            CONFIG.get().expect("Config not initialized").write().await
        }
        fn filepath(locked: &Self::LockedValue) -> &PathBuf {
            &locked.filepath
        }

        async fn serialize(entity: &Self::Target) -> Result<String, CommonError> {
            let serialized = toml::to_string(entity)?;
            Ok(serialized)
        }

        fn deserialize(data: &str) -> Result<Self::Target, CommonError> {
            let deserialized: Self::Target = toml::from_str(data)?;
            Ok(deserialized)
        }
    }
}

use crate::error::CommonError;
use sealed::EntityInternal;
use std::{fs, path::PathBuf};

/// Trait for entities with RwLock
/// # Type Parameters
/// - `LockedValue`: The type of the locked value.
/// - `Target`: The type of the target value required for read and write operations.
pub trait Entity: EntityInternal {
    /// Create a new instance of the entity.
    fn new(path: Option<PathBuf>) -> Self::LockedValue;

    /// Init the value
    fn init(path: Option<PathBuf>) -> Result<(), CommonError>;

    /// Save the current entity to file, whose path is defined in filepath field.
    fn save() -> impl Future<Output = Result<(), CommonError>> {
        async move {
            let entity = Self::get().await;
            Self::save_entity(&*entity).await?;
            Ok(())
        }
    }

    /// Load entity from file
    fn load(entity: &mut Self::LockedValue) -> Result<(), CommonError> {
        let filepath = Self::filepath(entity);
        if !filepath.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(filepath)?;
        let target: Self::Target = Self::deserialize(&content)?;
        let value = Self::mut_value(entity);
        *value = target;
        Ok(())
    }
    fn read<T, F: FnOnce(&Self::Target) -> T>(f: F) -> impl Future<Output = T> {
        async move {
            let lock = Self::get().await;
            let target = Self::value(&lock);
            f(target)
        }
    }
    fn write<T, F: FnOnce(&mut Self::Target) -> T>(f: F) -> impl Future<Output = T> {
        async move {
            let mut lock = Self::get_mut().await;
            let target = Self::mut_value(&mut lock);
            f(target)
        }
    }
}
