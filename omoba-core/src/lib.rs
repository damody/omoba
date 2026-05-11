//! OMOBA Core - OMOBA 前端的共享庫
//!
//! 提供 MQTT 通訊、遊戲狀態管理和玩家輸入處理。

pub mod ability_meta;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
pub mod lockstep_timing;
pub mod quant;
#[cfg(not(target_arch = "wasm32"))]
pub mod tower_meta;
// template_ids 已刪除 (2026-04-25)：遷移到 omoba-template-ids 箱。
// 下游代碼：`use omoba_template_ids::*;` — 源自連續的 u16 新類型
// 來自 script/lua_data/templates.lua 而不是手動維護的 FNV-1a 雜湊表。
#[cfg(all(feature = "grpc", not(target_arch = "wasm32")))]
pub mod grpc;
#[cfg(not(target_arch = "wasm32"))]
pub mod input;
#[cfg(feature = "game-proto")]
pub mod game_proto {
    include!(concat!(env!("OUT_DIR"), "/game.rs"));
}
#[cfg(all(feature = "kcp", not(target_arch = "wasm32")))]
pub mod kcp;
#[cfg(all(feature = "mqtt", not(target_arch = "wasm32")))]
pub mod mqtt;
#[cfg(not(target_arch = "wasm32"))]
pub mod state;

pub use ability_meta::{AbilityDef, AbilityLevelData, AbilityType, CastType, TargetType};
#[cfg(not(target_arch = "wasm32"))]
pub use config::{AppConfig, BackendConfig, FrontendConfig, ServerConfig};
#[cfg(all(feature = "grpc", not(target_arch = "wasm32")))]
pub use grpc::client::GameEventData;
#[cfg(all(feature = "grpc", not(target_arch = "wasm32")))]
pub use grpc::GrpcClient;
#[cfg(not(target_arch = "wasm32"))]
pub use input::{AttackParams, CastAbilityParams, MoveParams, PlayerAction, PlayerSimulator};
#[cfg(all(feature = "kcp", not(target_arch = "wasm32")))]
pub use kcp::client::GameEventData;
#[cfg(all(feature = "kcp", not(target_arch = "wasm32")))]
pub use kcp::KcpClient;
#[cfg(all(feature = "mqtt", not(target_arch = "wasm32")))]
pub use mqtt::{MqttClient, MqttEvent, MqttHandler};
#[cfg(not(target_arch = "wasm32"))]
pub use state::{Entity, EntityType, GameState, Viewport};
