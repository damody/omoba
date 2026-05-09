//! 跨越主機/DLL 邊界的穩定 ABI 值類型。

use abi_stable::{
    std_types::{ROption, RString, RVec},
    StableAbi,
};

/// 從 omoba-sim 重新匯出 Fix64 / Vec2 / Angle，以便 script-abi 消費者（base_content、omb）
/// 不需要 omoba-sim 上的單獨部門。類型透過 abi_stable::StableAbi 攜帶
/// omoba-sim 的「abi-stable」功能。
pub use omoba_sim::{Angle, Fixed64, Vec2};

/// 遊戲實體的不透明句柄。主機與“specs::Entity”之間進行轉換。
#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityHandle {
    pub id: u32,
    pub gen: u32,
}

impl EntityHandle {
    pub const INVALID: Self = Self {
        id: u32::MAX,
        gen: 0,
    };
    pub fn is_valid(&self) -> bool {
        self.id != u32::MAX
    }
}

#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DamageKind {
    Physical,
    Magical,
    Pure,
}

/// 作為“&mut”傳遞給“on_damage_taken”——腳本可以修改“amount”
/// （例如護盾、傷害減免、反射）。
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

#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, Default)]
pub struct TowerRenderPoint {
    pub x: Fixed64,
    pub y: Fixed64,
}

#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug)]
pub struct TowerRenderAnimation {
    pub fps: Fixed64,
    pub loop_animation: bool,
    pub fire_fps: Fixed64,
    pub fire_once: bool,
}

impl Default for TowerRenderAnimation {
    fn default() -> Self {
        Self {
            fps: Fixed64::from_i32(10),
            loop_animation: true,
            fire_fps: Fixed64::from_i32(18),
            fire_once: true,
        }
    }
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug, Default)]
pub struct TowerBarrelVariant {
    pub min_path: u8,
    pub min_level: u8,
    pub count: u16,
    pub image: RString,
    pub frames: RVec<RString>,
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct TowerRecoil {
    pub mode: RString,
    pub distance: Fixed64,
    pub scale: Fixed64,
    pub duration_ms: u32,
    pub return_ms: u32,
}

impl Default for TowerRecoil {
    fn default() -> Self {
        Self {
            mode: RString::from("directional"),
            distance: Fixed64::from_i32(7),
            scale: Fixed64::from_raw(963),
            duration_ms: 70,
            return_ms: 110,
        }
    }
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct TowerRenderMetadata {
    pub render_mode: RString,
    pub base: RString,
    pub barrel: RString,
    pub barrel_frames: RVec<RString>,
    pub body_frames: RVec<RString>,
    pub barrel_animation: TowerRenderAnimation,
    pub body_animation: TowerRenderAnimation,
    pub rotation_mode: RString,
    pub barrel_layout: RString,
    pub barrel_variants: RVec<TowerBarrelVariant>,
    pub barrel_offset: TowerRenderPoint,
    pub barrel_pivot: TowerRenderPoint,
    pub muzzle_offset: TowerRenderPoint,
    pub default_angle_deg: Fixed64,
    pub recoil: TowerRecoil,
}

impl Default for TowerRenderMetadata {
    fn default() -> Self {
        Self {
            render_mode: RString::from("base_barrel"),
            base: RString::new(),
            barrel: RString::new(),
            barrel_frames: RVec::new(),
            body_frames: RVec::new(),
            barrel_animation: TowerRenderAnimation::default(),
            body_animation: TowerRenderAnimation::default(),
            rotation_mode: RString::from("targeted"),
            barrel_layout: RString::from("single"),
            barrel_variants: RVec::new(),
            barrel_offset: TowerRenderPoint::default(),
            barrel_pivot: TowerRenderPoint {
                x: Fixed64::from_raw(512),
                y: Fixed64::from_raw(666),
            },
            muzzle_offset: TowerRenderPoint::default(),
            default_angle_deg: Fixed64::ZERO,
            recoil: TowerRecoil::default(),
        }
    }
}

#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, Default)]
pub struct AttackTimingMetadata {
    pub windup: u16,
    pub backswing: u16,
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
    /// 戰鬥畫面 base/barrel/body-frame render metadata。
    pub render: TowerRenderMetadata,
    /// 普攻 windup / backswing 整數權重；總和必須為 1000。
    pub attack_timing: AttackTimingMetadata,
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
