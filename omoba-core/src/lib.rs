//! OMOBA Core - Shared library for OMOBA frontends
//!
//! Provides MQTT communication, game state management, and player input handling.

pub mod config;
#[cfg(feature = "mqtt")]
pub mod mqtt;
pub mod state;
pub mod input;
#[cfg(feature = "grpc")]
pub mod grpc;

pub use config::{AppConfig, ServerConfig, BackendConfig, FrontendConfig};
pub use state::{GameState, Entity, EntityType, Viewport};
#[cfg(feature = "mqtt")]
pub use mqtt::{MqttClient, MqttEvent, MqttHandler};
pub use input::{PlayerSimulator, PlayerAction, MoveParams, CastAbilityParams, AttackParams};
#[cfg(feature = "grpc")]
pub use grpc::GrpcClient;
