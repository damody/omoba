//! 雜賀孫市（Saika Magoichi）— B01 英雄技能組。
//!
//! 4 個技能：
//! - `sniper_mode`（Q, toggle）— 射程/傷害增益、攻速/移速懲罰
//! - `saika_reinforcements`（W, active）— 召喚雜賀眾
//! - `rain_iron_cannon`（E, passive）— 普攻命中點的前向扇形真實傷害
//! - `three_stage_technique`（R, active）— 變身增傷，連發僅一發判定傷害

pub mod No1_sniper_mode;
pub mod No2_saika_reinforcements;
pub mod No3_rain_iron_cannon;
pub mod No4_three_stage_technique;

pub use No1_sniper_mode::sniper_mode_ffi;
pub use No2_saika_reinforcements::saika_reinforcements_ffi;
pub use No3_rain_iron_cannon::rain_iron_cannon_ffi;
pub use No4_three_stage_technique::three_stage_ffi;
pub use No4_three_stage_technique::three_stage_ffi as three_stage_technique_ffi;
// M5 (2026-10-30): the b01_* abilities run generated Rust (scripts/heroes/saika_magoichi); their
// handlers are gone. The four above are the retained legacy loadout tests pin.
