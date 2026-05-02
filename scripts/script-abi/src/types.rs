//! Stable-ABI value types that cross the host/DLL boundary.

use abi_stable::{StableAbi, std_types::{ROption, RString}};

/// Re-export Fixed64 / Vec2 / Angle from omoba-sim so script-abi consumers (base_content, omb)
/// don't need a separate dep on omoba-sim. Types carry abi_stable::StableAbi via
/// omoba-sim's `abi-stable` feature.
pub use omoba_sim::{Angle, Fixed64, Vec2};

/// Opaque handle to a game entity. Host converts to/from `specs::Entity`.
#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityHandle {
    pub id: u32,
    pub gen: u32,
}

impl EntityHandle {
    pub const INVALID: Self = Self { id: u32::MAX, gen: 0 };
    pub fn is_valid(&self) -> bool { self.id != u32::MAX }
}

#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DamageKind {
    Physical,
    Magical,
    Pure,
}

/// Passed to `on_damage_taken` as `&mut` — scripts may modify `amount`
/// (e.g. shield, damage reduction, reflect).
#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct DamageInfo {
    pub attacker: ROption<EntityHandle>,
    pub amount: Fixed64,
    pub kind: DamageKind,
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub enum Target {
    Entity(EntityHandle),
    Point(Vec2),
    None,
}

/// 子彈飛行路徑規格：
/// - `Homing` 鎖定 `target` 實體並 per-tick 跟進位置
/// - `Straight` 從發射位置直線飛到 `end_pos`
#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub enum PathSpec {
    Homing { target: EntityHandle },
    Straight { end_pos: Vec2 },
}

/// TD 塔的完整 metadata（由腳本回報；host 和前端共用）。
#[repr(C)]
#[derive(StableAbi, Clone, Debug, Default)]
pub struct TowerMetadata {
    /// 基礎攻擊力（物理）
    pub atk: Fixed64,
    /// 攻擊間隔秒數
    pub asd_interval: Fixed64,
    /// 射程（backend 單位）
    pub range: Fixed64,
    /// 子彈飛行速度（backend 單位/秒）
    pub bullet_speed: Fixed64,
    /// 命中後 AoE 半徑（0 = 單體）
    pub splash_radius: Fixed64,
    /// 沿路命中半徑（Tack 針用；0 = 只在 end_pos 觸發）
    pub hit_radius: Fixed64,
    /// 減速乘數（0 = 不減速）
    pub slow_factor: Fixed64,
    /// 減速持續秒數
    pub slow_duration: Fixed64,

    /// 建造金幣
    pub cost: i32,
    /// 放置碰撞半徑
    pub footprint: Fixed64,
    /// 塔 HP
    pub hp: Fixed64,
    /// 塔轉向速度（度/秒）
    pub turn_speed_deg: Fixed64,
    /// UI 顯示名稱
    pub label: RString,
}

/// 發射子彈的完整規格。`spawn_projectile_ex` 接這個。
#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct ProjectileSpec {
    /// 起始位置（世界座標）
    pub from: Vec2,
    /// 發射者 entity（傷害歸屬與 faction filter）
    pub owner: EntityHandle,
    /// 路徑規格
    pub path: PathSpec,
    /// 子彈飛行速度（單位/秒）
    pub speed: Fixed64,
    /// 基礎傷害（物理）
    pub damage: Fixed64,
    /// 沿路 hit-test 半徑
    pub hit_radius: Fixed64,
    /// 命中後 AoE 半徑
    pub splash_radius: Fixed64,
    /// 減速乘數
    pub slow_factor: Fixed64,
    /// 減速持續秒數
    pub slow_duration: Fixed64,
    /// 命中後 stun 秒數
    pub stun_duration: Fixed64,
    /// 前端渲染 kind id
    pub kind_id: u16,
}
