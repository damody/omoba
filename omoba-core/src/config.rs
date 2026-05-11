//! OMOBA 前端的設定管理

use anyhow::{Context, Result};
use omoba_template_ids::HERO_SAIKA_MAGOICHI;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 應用程式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub backend: BackendConfig,
    pub frontend: FrontendConfig,
}

/// 伺服器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub mqtt_host: String,
    pub mqtt_port: u16,
}

/// 後端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub executable_path: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// 前端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub player_name: String,
    pub hero_type: String,
    pub auto_start_backend: bool,
    pub backend_start_delay: u64,
    pub backend_shutdown_timeout: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                mqtt_host: "127.0.0.1".to_string(),
                mqtt_port: 1883,
            },
            backend: BackendConfig {
                executable_path: "../omb/target/debug/omobab".to_string(),
                args: vec![],
                working_directory: None,
                env: HashMap::new(),
            },
            frontend: FrontendConfig {
                player_name: "TestPlayer".to_string(),
                hero_type: HERO_SAIKA_MAGOICHI.as_str().to_string(),
                auto_start_backend: true,
                backend_start_delay: 1000,
                backend_shutdown_timeout: 5000,
            },
        }
    }
}

impl AppConfig {
    /// 從檔案載入配置
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read config file: {}", path))?;

        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("Cannot parse config file: {}", path))?;

        Ok(config)
    }

    /// 載入配置（首選文件，回退到預設值）
    pub fn load() -> Self {
        match Self::from_file("config.toml") {
            Ok(config) => {
                log::info!("Loaded config file: config.toml");
                config
            }
            Err(e) => {
                log::warn!("Cannot load config file, using defaults: {}", e);
                Self::default()
            }
        }
    }

    /// 將配置儲存到文件
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Cannot serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Cannot write config file: {}", path))?;

        Ok(())
    }

    /// 取得後端可執行檔的絕對路徑
    pub fn get_backend_executable_path(&self) -> Result<PathBuf> {
        let path = PathBuf::from(&self.backend.executable_path);

        let abs_path = if path.is_relative() {
            std::env::current_dir()?.join(path)
        } else {
            path
        };

        if !abs_path.exists() {
            anyhow::bail!("Backend executable not found: {:?}", abs_path);
        }

        Ok(abs_path)
    }
}

pub mod server_config {
    #[derive(Debug)]
    pub struct RuntimeServerConfig {
        pub PLAYER_NAME: String,
    }

    lazy_static::lazy_static! {
        pub static ref CONFIG: RuntimeServerConfig = RuntimeServerConfig {
            PLAYER_NAME: std::env::var("OMB_PLAYER_NAME").unwrap_or_else(|_| "player".to_string()),
        };
    }
}
