//! OMFX configuration - unified config for frontend, server, and backend

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OMFX unified configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmfxConfig {
    /// Server/MQTT settings
    pub server: ServerConfig,
    /// Backend settings
    pub backend: BackendConfig,
    /// Frontend/player settings
    pub frontend: FrontendConfig,
    /// Window settings
    pub window: WindowConfig,
    /// Camera settings
    pub camera: CameraConfig,
    /// Debug settings
    pub debug: DebugConfig,
    /// Render settings
    pub render: RenderConfig,
}

/// Server/MQTT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub mqtt_host: String,
    pub mqtt_port: u16,
}

/// Backend process configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Path to backend executable
    pub executable_path: String,
    /// Command line arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory (optional)
    pub working_directory: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Auto-start backend when omfx starts
    pub auto_start: bool,
    /// Delay in milliseconds before starting backend
    pub start_delay_ms: u64,
    /// Timeout in milliseconds for graceful shutdown
    pub shutdown_timeout_ms: u64,
    /// Require backend health check (MQTT connection) before starting
    #[serde(default = "default_require_health_check")]
    pub require_health_check: bool,
    /// Timeout in milliseconds for health check
    #[serde(default = "default_health_check_timeout")]
    pub health_check_timeout_ms: u64,
}

fn default_require_health_check() -> bool {
    true
}

fn default_health_check_timeout() -> u64 {
    10000  // 10 seconds
}

/// Frontend/player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub player_name: String,
    pub hero_type: String,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fullscreen: bool,
    pub vsync: bool,
}

/// Camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub edge_scroll_speed: f32,
    pub edge_scroll_zone: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub default_zoom: f32,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub show_fps: bool,
    pub show_entity_count: bool,
    pub default_overlays: Vec<String>,
    pub log_level: String,
}

/// Render configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub health_bar_width: f32,
    pub health_bar_height: f32,
    pub fog_tile_size: f32,
    pub trail_duration_ms: u32,
}

impl Default for OmfxConfig {
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
                auto_start: true,
                start_delay_ms: 1000,
                shutdown_timeout_ms: 5000,
                require_health_check: true,
                health_check_timeout_ms: 10000,
            },
            frontend: FrontendConfig {
                player_name: "TestPlayer".to_string(),
                hero_type: "saika_magoichi".to_string(),
            },
            window: WindowConfig {
                width: 1920,
                height: 1080,
                title: "OMFX - OMOBA Debug Frontend".to_string(),
                fullscreen: false,
                vsync: true,
            },
            camera: CameraConfig {
                edge_scroll_speed: 800.0,
                edge_scroll_zone: 20.0,
                zoom_speed: 0.1,
                min_zoom: 0.5,
                max_zoom: 3.0,
                default_zoom: 1.0,
            },
            debug: DebugConfig {
                show_fps: true,
                show_entity_count: true,
                default_overlays: vec![],
                log_level: "info".to_string(),
            },
            render: RenderConfig {
                health_bar_width: 40.0,
                health_bar_height: 6.0,
                fog_tile_size: 32.0,
                trail_duration_ms: 500,
            },
        }
    }
}

impl OmfxConfig {
    /// Load from file
    pub fn from_file(path: &str) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Cannot read config file: {}", path))?;

        let config: OmfxConfig =
            toml::from_str(&content).with_context(|| format!("Cannot parse config file: {}", path))?;

        Ok(config)
    }

    /// Load config (prefer file, fallback to default)
    pub fn load() -> Self {
        Self::load_from("config.toml")
    }

    /// Load config from specific path
    pub fn load_from(path: &str) -> Self {
        match Self::from_file(path) {
            Ok(config) => {
                log::info!("Loaded config from {}", path);
                config
            }
            Err(e) => {
                log::warn!("Cannot load config from {}, using defaults: {}", path, e);
                Self::default()
            }
        }
    }

    /// Save to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Cannot serialize config")?;

        std::fs::write(path, content).with_context(|| format!("Cannot write config file: {}", path))?;

        Ok(())
    }

    /// Convert server config to omoba-core ServerConfig
    pub fn to_core_server_config(&self) -> omoba_core::ServerConfig {
        omoba_core::ServerConfig {
            mqtt_host: self.server.mqtt_host.clone(),
            mqtt_port: self.server.mqtt_port,
        }
    }
}
