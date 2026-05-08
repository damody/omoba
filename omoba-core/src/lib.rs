//! OMOBA Core - OMOBA 前端的共享庫
//!
//! 提供 MQTT 通訊、遊戲狀態管理和玩家輸入處理。

pub mod ability_meta;
pub mod tower_meta;
pub mod config;
pub mod lockstep_timing;
pub mod quant;
// template_ids 已刪除 (2026-04-25)：遷移到 omoba-template-ids 箱。
// 下游代碼：`use omoba_template_ids::*;` — 源自連續的 u16 新類型
// 來自 script/lua_data/templates.lua 而不是手動維護的 FNV-1a 雜湊表。
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
