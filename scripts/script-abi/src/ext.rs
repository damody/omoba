//! `GameWorld::ext`: one general extension point (M3', work line B). `GameWorld` is a
//! `#[sabi_trait]` and holds at most 64 methods (E235); new host reads and requests go through
//! `ext(op, args)` instead of new trait methods. The host dispatches on `op`; an unknown op or
//! wrong arguments return `RErr`, so a DLL built against a newer op set fails loudly, never
//! silently. Ops only read the world or emit closed outcomes; none keeps state in the DLL.
use crate::types::{EntityHandle, Fixed64, Vec2};
use abi_stable::{std_types::{RString, RVec}, StableAbi};

#[repr(C)]
#[derive(StableAbi, Clone, Debug, PartialEq)]
pub enum ExtArg {
    Unit(EntityHandle),
    Num(Fixed64),
    Point(Vec2),
    Str(RString),
    Bool(bool),
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug, PartialEq)]
pub enum ExtValue {
    None,
    Unit(EntityHandle),
    Num(Fixed64),
    Point(Vec2),
    Str(RString),
    Bool(bool),
    /// A unit list in a deterministic order (op docs say which).
    Units(RVec<EntityHandle>),
}

/// Operation ids. Never renumber; append.
pub mod op {
    /// `() -> Bool`: whether the world records a damage trace (test worlds with `DamageTrace`).
    pub const DAMAGE_TRACE_ENABLED: u32 = 1;
    /// `(Str ability, Str path) -> None`: tags the next damage this script deals (trace only).
    pub const DAMAGE_TAG: u32 = 2;
    /// `(Unit, Str ability) -> Num`: the unit's rank of that ability (0 when not learned).
    pub const ABILITY_LEVEL: u32 = 3;
    /// `(Unit, Str attr) -> value`: a v3 unit attribute the trait has no method for (names in
    /// [`super::unit_attr`]). A dead or unknown unit reads as the attribute's zero value.
    pub const UNIT_ATTR: u32 = 4;
    /// `(Unit, Str ability, Str attr) -> value`: a v3 attribute of one of the unit's abilities
    /// (names in [`super::ability_attr`]).
    pub const UNIT_ABILITY_ATTR: u32 = 5;
    /// `(Str attr) -> value`: a v3 `game.*` fact (names in [`super::game_attr`]).
    pub const GAME_ATTR: u32 = 6;
    /// `(Unit credit) -> None`: the next damage this script deals credits kills and assists to
    /// `credit` (v3 `damage.credit`) instead of its source.
    pub const DAMAGE_CREDIT: u32 = 7;
    // 8: ORDER_ATTACK (work line A).
    /// `(Point center, Num radius, Unit relative_to, Str team) -> Units`: every unit whose position
    /// is within `radius` of `center`, `team` = `"ally"` (relative_to's team, itself included),
    /// `"enemy"` or `"both"`; nearest first, ties by entity id. No other filter: the selector
    /// (types, dead, invulnerable, magic immune…) filters what it reads.
    pub const UNITS_IN_RANGE: u32 = 9;
}

/// Attribute names of [`op::UNIT_ATTR`] and what they return.
pub mod unit_attr {
    /// `Num`: hero level; 1 for other units.
    pub const LEVEL: &str = "level";
    /// `Num`: strength / agility / intelligence after level growth and bonuses (0 for non-heroes).
    pub const STR: &str = "str";
    pub const AGI: &str = "agi";
    pub const INT: &str = "int";
    /// `Num`: mana capacity (0 for a unit without mana).
    pub const MAX_MANA: &str = "max_mana";
    /// `Bool`: an illusion (state `illusion`).
    pub const IS_ILLUSION: &str = "is_illusion";
    /// `Bool`: channelling an ability.
    pub const IS_CHANNELING: &str = "is_channeling";
    /// `Bool`: state `invulnerable`.
    pub const IS_INVULNERABLE: &str = "is_invulnerable";
    /// `Bool`: holds an invisibility level above zero.
    pub const IS_INVISIBLE: &str = "is_invisible";
    /// `Bool`: a lane or neutral creep.
    pub const IS_CREEP: &str = "is_creep";
    /// `Bool`: attacks at range (states `melee_attack` / `ranged_attack` override).
    pub const IS_RANGED: &str = "is_ranged";
    /// `Unit`: the summoner of a summon; `None` otherwise.
    pub const OWNER: &str = "owner";
    /// `Unit`: the unit it is attacking; `None` when idle.
    pub const ATTACK_TARGET: &str = "attack_target";
    /// `Num`: final attack speed multiplier (1 = base).
    pub const ATTACK_SPEED_MULT: &str = "attack_speed_mult";
    /// `Num`: attack projectile speed.
    pub const PROJECTILE_SPEED: &str = "projectile_speed";
    /// `Num`: base magic resistance (without bonuses).
    pub const BASE_MAGIC_RESIST: &str = "base_magic_resist";
    /// `Num`: base attack range (without bonuses).
    pub const BASE_ATTACK_RANGE: &str = "base_attack_range";
    /// `Num`: cast range bonus the unit's statuses grant.
    pub const CAST_RANGE_BONUS: &str = "cast_range_bonus";
    /// `Num`: whole gold in the pool of the unit's player (one pool per player, FC-129～131: no
    /// reliable / unreliable split); 0 when the world keeps none for it (an enemy on a team replica).
    pub const GOLD: &str = "gold";
}

/// Attribute names of [`op::UNIT_ABILITY_ATTR`].
pub mod ability_attr {
    /// `Num`: seconds left on the ability's cooldown (0 when ready or unknown).
    pub const COOLDOWN_REMAINING: &str = "cooldown_remaining";
    /// `Bool`: learned, off cooldown and (a charge ability) holding a charge.
    pub const IS_READY: &str = "is_ready";
}

/// Attribute names of [`op::GAME_ATTR`].
pub mod game_attr {
    /// `Bool`: night (only `temporary_night` makes night; there is no day cycle).
    pub const IS_NIGHT: &str = "is_night";
    /// `Num`: match time in seconds.
    pub const TIME: &str = "time";
}
