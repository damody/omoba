//! OMFX-specific configuration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// OMFX display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmfxConfig {
    /// Window settings
    pub window: WindowConfig,
    /// Camera settings
    pub camera: CameraConfig,
    /// Debug settings
    pub debug: DebugConfig,
    /// Render settings
    pub render: RenderConfig,
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
        match Self::from_file("omfx_config.toml") {
            Ok(config) => {
                log::info!("Loaded OMFX config from omfx_config.toml");
                config
            }
            Err(e) => {
                log::warn!("Cannot load OMFX config, using defaults: {}", e);
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
}
