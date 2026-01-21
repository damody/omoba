//! OMOBA Core - Shared library for OMOBA frontends
//!
//! Provides MQTT communication, game state management, and player input handling.

pub mod config;
pub mod mqtt;
pub mod state;
pub mod input;

pub use config::{AppConfig, ServerConfig, BackendConfig, FrontendConfig};
pub use state::{GameState, Entity, EntityType, Viewport};
pub use mqtt::{MqttClient, MqttHandler};
pub use input::{PlayerSimulator, PlayerAction};
