//! 伊達政宗（Date Masamune）— B02 英雄技能組。
//!
//! 4 個技能：
//! - `flame_blade`（W, active）— 前方火焰刀瞬發傷害
//! - `fire_dash`（E, active）— 衝刺位移並對沿路敵人造傷害
//! - `flame_assault`（R, ultimate）— 巨型火焰範圍傷害 + 暈眩
//! - `matchlock_gun`（T, transform）— 變身火繩銃，增加射程/傷害、攻擊暈眩

pub mod No1_flame_blade;
pub mod No2_fire_dash;
pub mod No3_flame_assault;
pub mod No4_matchlock_gun;

pub use No1_flame_blade::flame_blade_ffi;
pub use No2_fire_dash::fire_dash_ffi;
pub use No3_flame_assault::flame_assault_ffi;
pub use No4_matchlock_gun::matchlock_gun_ffi;
