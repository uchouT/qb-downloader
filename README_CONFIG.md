# qb-downloader 全局配置管理

这个项目实现了完整的全局配置管理系统，使用 TOML 格式存储配置文件。

## 功能特性

- 🔧 **全局配置管理**: 使用 `OnceLock` 实现线程安全的全局配置
- 💾 **自动持久化**: 配置修改后自动保存到文件
- 📂 **标准路径**: 配置文件存储在 `~/.config/qb-downloader/config.toml`
- ✅ **配置验证**: 内置配置验证功能
- 📤 **导入导出**: 支持配置的导入和导出
- 🔒 **线程安全**: 使用 `RwLock` 支持并发读写

## 使用方法

### 1. 初始化配置

```rust
use qb_downloader_rust::{init_config, get_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用默认配置路径
    init_config(None).await?;
    
    // 或使用自定义配置路径
    // init_config(Some(PathBuf::from("./my_config.toml"))).await?;
    
    let config = get_config();
    Ok(())
}
```

### 2. 读取配置

```rust
let config = get_config();

// 读取配置值
let cfg = config.get().await;
println!("qBittorrent Host: {}", cfg.qb_host);
```

### 3. 更新配置

```rust
// 直接更新
config.update(|cfg| {
    cfg.qb_host = "http://new-host:8080".to_string();
    cfg.default_save_path = "/new/path".to_string();
}).await?;

// 使用配置管理器
use qb_downloader_rust::config_manager::ConfigManager;

ConfigManager::set_qb_config(
    config,
    "http://localhost:8080".to_string(),
    "admin".to_string(),
    "password".to_string(),
).await?;
```

### 4. 配置管理器方法

`ConfigManager` 提供了许多便利方法：

```rust
// 设置各种配置
ConfigManager::set_qb_config(config, host, user, pass).await?;
ConfigManager::set_rclone_config(config, host, user, pass).await?;
ConfigManager::set_paths(config, save_path, upload_path).await?;
ConfigManager::set_limits(config, ratio, time).await?;
ConfigManager::set_security_config(config, inner_ip, verify_ip).await?;

// 获取配置
let (host, user, pass) = ConfigManager::get_qb_config(config).await;

// 验证配置
let errors = ConfigManager::validate_config(config).await;

// 导入导出
ConfigManager::export_config(config, export_path).await?;
ConfigManager::import_config(config, import_path).await?;

// 重置为默认值
ConfigManager::reset_to_default(config).await?;
```

## 配置结构

```toml
# qBittorrent 配置
qb_host = "http://localhost:8080"
qb_username = "admin"
qb_password = "adminadmin"

# rclone 配置
rclone_host = "http://localhost:5572"
rclone_username = "admin"
rclone_password = "password"

# 安全配置
is_only_inner_ip = false
verify_login_ip = true

# 默认路径
default_save_path = ""
default_upload_path = ""

# 默认限制
default_ratio_limit = -2.0      # -2 表示无限制
default_seeding_time_limit = -2 # -2 表示无限制

# 登录信息
[login]
ip = ""
username = "admin"
password = "hash"
key = ""
```

## 最佳实践

1. **程序启动时初始化**: 在 `main` 函数开始时调用 `init_config()`
2. **使用配置管理器**: 优先使用 `ConfigManager` 的方法而不是直接操作配置
3. **错误处理**: 适当处理配置加载和保存的错误
4. **配置验证**: 定期验证配置的有效性
5. **备份配置**: 重要配置修改前可以导出备份

## 示例

运行完整示例：

```bash
cargo run --example config_example
```

这将演示所有配置管理功能的使用方法。
