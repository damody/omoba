//! 伊達政宗（Date Masamune）— B02 英雄技能組。
//!
//! 4 個技能：
//! - `flame_blade`（slot 0 / Q, active）— 指定地點圓形瞬發火焰傷害
//! - `fire_dash`（slot 1 / W, active）— 衝刺位移並對沿路敵人造傷害
//! - `flame_assault`（slot 2 / E, ultimate）— 指定地點圓形火焰傷害 + 暈眩
//! - `matchlock_gun`（slot 3 / R, transform）— 變身火繩銃，增加射程/傷害、攻擊暈眩

pub mod No1_flame_blade;
pub mod No2_fire_dash;
pub mod No3_flame_assault;
pub mod No4_matchlock_gun;

pub use No1_flame_blade::flame_blade_ffi;
pub use No2_fire_dash::fire_dash_ffi;
pub use No3_flame_assault::flame_assault_ffi;
pub use No4_matchlock_gun::matchlock_gun_ffi;
