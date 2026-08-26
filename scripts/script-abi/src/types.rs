//! 跨越主機/DLL 邊界的穩定 ABI 值類型。

use abi_stable::{
    std_types::{ROption, RString, RVec},
    StableAbi,
};

/// 從 omoba-sim 重新匯出 Fix64 / Vec2 / Angle，以便 script-abi 消費者（base_content、omb）
/// 不需要 omoba-sim 上的單獨部門。類型透過 abi_stable::StableAbi 攜帶
/// omoba-sim 的「abi-stable」功能。
pub use omoba_sim::{Angle, Fixed64, Vec2};

/// Append-only identifier carried across the script DLL boundary. The host
/// owns interpretation and rejects unknown/missing IDs before a secure match.
#[repr(C)]
#[derive(StableAbi, Clone, Debug, PartialEq, Eq)]
pub struct ProjectionPolicyId {
    pub abi_version: u16,
    pub value: RString,
}

impl ProjectionPolicyId {
    pub const ABI_VERSION: u16 = 1;

    pub fn new(value: impl Into<RString>) -> Self {
        Self { abi_version: Self::ABI_VERSION, value: value.into() }
    }
}

pub mod projection_policy_ids {
    pub const MOVEMENT: &str = "movement.v1";
    pub const SPAWN: &str = "spawn.v1";
    pub const DEATH: &str = "death.v1";
    pub const OWNERSHIP: &str = "ownership.v1";
    pub const DIRECT_COMBAT: &str = "direct-combat.v1";
    pub const PROJECTILE: &str = "projectile.v1";
    pub const AOE: &str = "aoe.v1";
    pub const BUFF_DEBUFF: &str = "buff-debuff.v1";
    pub const HERO_ABILITY: &str = "hero-ability.v1";
    pub const TOWER: &str = "tower.v1";
    pub const ITEM: &str = "item.v1";
}

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

pub const DAMAGE_PROFILE_ABI_VERSION: u16 = 1;

/// Stable TD damage compatibility mask. Bit assignments are append-only ABI.
#[repr(transparent)]
#[derive(StableAbi, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DamageProfile(pub u32);

impl DamageProfile {
    pub const SHARP: Self = Self(1 << 0);
    pub const EXPLOSIVE: Self = Self(1 << 1);
    pub const ENERGY: Self = Self(1 << 2);
    pub const FIRE: Self = Self(1 << 3);
    pub const COLD: Self = Self(1 << 4);
    pub const NORMAL: Self = Self(1 << 5);
    pub const CRUSHING: Self = Self(1 << 6);
    pub const TRUE: Self = Self(1 << 7);
    pub const KNOWN_BITS: u32 = (1 << 8) - 1;

    pub const fn from_bits(bits: u32) -> Option<Self> {
        if bits != 0 && bits & !Self::KNOWN_BITS == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

/// 作為“&mut”傳遞給“on_damage_taken”——腳本可以修改“amount”
/// （例如護盾、傷害減免、反射）。
#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct DamageInfo {
    pub attacker: ROption<EntityHandle>,
    pub amount: Fixed64,
    pub kind: DamageKind,
    pub profile: DamageProfile,
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

/// Provenance attached to a projectile impact before it crosses into script code.
#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectileHitContext {
    /// Script-visible projectile kind.
    pub kind_id: u16,
    /// Zero for primary shots; child shots increment with saturation.
    pub generation: u8,
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
    pub visual_size: Fixed64,
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
            visual_size: Fixed64::ZERO,
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
    /// runtime collision/template footprint；建塔放置檢查使用 `placement_radius`。
    pub footprint: Fixed64,
    /// 建塔放置檢查半徑（backend 單位）；不等同 runtime CollisionRadius。
    pub placement_radius: Fixed64,
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
    /// Explicit TD compatibility tags; host rejects zero or unknown bits.
    pub damage_profile: DamageProfile,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_profile_bits_are_stable_and_round_trip() {
        assert_eq!(DAMAGE_PROFILE_ABI_VERSION, 1);
        assert_eq!(DamageProfile::SHARP.bits(), 1);
        assert_eq!(DamageProfile::EXPLOSIVE.bits(), 2);
        assert_eq!(DamageProfile::ENERGY.bits(), 4);
        assert_eq!(DamageProfile::FIRE.bits(), 8);
        assert_eq!(DamageProfile::COLD.bits(), 16);
        assert_eq!(DamageProfile::NORMAL.bits(), 32);
        assert_eq!(DamageProfile::CRUSHING.bits(), 64);
        assert_eq!(DamageProfile::TRUE.bits(), 128);

        let combined = DamageProfile::FIRE.union(DamageProfile::EXPLOSIVE);
        assert_eq!(DamageProfile::from_bits(combined.bits()), Some(combined));
        assert!(combined.intersects(DamageProfile::FIRE));
        assert!(combined.intersects(DamageProfile::EXPLOSIVE));
    }

    #[test]
    fn damage_profile_rejects_zero_and_unknown_bits() {
        assert_eq!(DamageProfile::from_bits(0), None);
        assert_eq!(DamageProfile::from_bits(1 << 8), None);
        assert_eq!(
            DamageProfile::from_bits(DamageProfile::KNOWN_BITS | (1 << 31)),
            None
        );
    }
}
