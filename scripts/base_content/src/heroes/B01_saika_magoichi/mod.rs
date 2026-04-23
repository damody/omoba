//! 雜賀孫市（Saika Magoichi）— B01 英雄技能組。
//!
//! 4 個技能：
//! - `sniper_mode`（W, toggle）— 射程/傷害增益、攻速/移速懲罰
//! - `saika_reinforcements`（E, active）— 召喚雜賀眾（summon API 未完成，stub）
//! - `rain_iron_cannon`（R, ultimate）— 雨鐵炮 AoE 傷害
//! - `three_stage_technique`（T, active）— 三段擊單體多次傷害

pub mod No1_sniper_mode;
pub mod No2_saika_reinforcements;
pub mod No3_rain_iron_cannon;
pub mod No4_three_stage_technique;

pub use No1_sniper_mode::sniper_mode_ffi;
pub use No2_saika_reinforcements::saika_reinforcements_ffi;
pub use No3_rain_iron_cannon::rain_iron_cannon_ffi;
pub use No4_three_stage_technique::three_stage_ffi;
