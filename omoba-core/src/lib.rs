//! OMOBA Core - OMOBA 前端的共享庫
//!
//! 提供 MQTT 通訊、遊戲狀態管理和玩家輸入處理。

pub mod ability_meta;
pub mod config;
pub mod lockstep_timing;
pub mod quant;
pub mod tower_meta;
// template_ids 已刪除 (2026-04-25)：遷移到 omoba-template-ids 箱。
// 下游代碼：`use omoba_template_ids::*;` — 源自連續的 u16 新類型
// 來自 script/lua_data/templates.lua 而不是手動維護的 FNV-1a 雜湊表。
#[cfg(feature = "grpc")]
pub mod grpc;
pub mod input;
#[cfg(feature = "kcp")]
pub mod kcp;
#[cfg(feature = "mqtt")]
pub mod mqtt;
pub mod state;

pub use ability_meta::{AbilityDef, AbilityLevelData, AbilityType, CastType, TargetType};
pub use config::{AppConfig, BackendConfig, FrontendConfig, ServerConfig};
#[cfg(feature = "grpc")]
pub use grpc::client::GameEventData;
#[cfg(feature = "grpc")]
pub use grpc::GrpcClient;
pub use input::{AttackParams, CastAbilityParams, MoveParams, PlayerAction, PlayerSimulator};
#[cfg(feature = "kcp")]
pub use kcp::client::GameEventData;
#[cfg(feature = "kcp")]
pub use kcp::KcpClient;
#[cfg(feature = "mqtt")]
pub use mqtt::{MqttClient, MqttEvent, MqttHandler};
pub use state::{Entity, EntityType, GameState, Viewport};
