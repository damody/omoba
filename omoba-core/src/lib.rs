//! OMOBA Core - Shared library for OMOBA frontends
//!
//! Provides MQTT communication, game state management, and player input handling.

pub mod ability_meta;
pub mod tower_meta;
pub mod config;
pub mod quant;
// template_ids removed (2026-04-25): migrated to omoba-template-ids crate.
// Downstream code: `use omoba_template_ids::*;` — sequential u16 newtypes sourced
// from scripts/lua_data/templates.lua instead of hand-maintained FNV-1a hash tables.
#[cfg(feature = "mqtt")]
pub mod mqtt;
pub mod state;
pub mod input;
#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "kcp")]
pub mod kcp;

pub use ability_meta::{AbilityDef, AbilityLevelData, AbilityType, TargetType, CastType};
pub use config::{AppConfig, ServerConfig, BackendConfig, FrontendConfig};
pub use state::{GameState, Entity, EntityType, Viewport};
#[cfg(feature = "mqtt")]
pub use mqtt::{MqttClient, MqttEvent, MqttHandler};
pub use input::{PlayerSimulator, PlayerAction, MoveParams, CastAbilityParams, AttackParams};
#[cfg(feature = "grpc")]
pub use grpc::GrpcClient;
#[cfg(feature = "grpc")]
pub use grpc::client::GameEventData;
#[cfg(feature = "kcp")]
pub use kcp::KcpClient;
#[cfg(feature = "kcp")]
pub use kcp::client::GameEventData;
