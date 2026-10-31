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
    /// `(Unit, Str runtime_id) -> Num`: how many independent instances (`<runtime_id>#<n>`) of a
    /// status the unit holds (0 when none).
    pub const STATUS_INSTANCES: u32 = 10;
    /// `()` -> `Bool`: the cast running now was released from a windup; its target and range
    /// were judged once, when the windup began, and must not be judged again (a team replica's
    /// view of a moving target may differ from the authority's by then).
    pub const CAST_RELEASED: u32 = 11;
    /// `(Unit x, Unit u)` -> `Bool`: v3 `is_visible_to(x, u)`, whether `u`'s team can see `x` now
    /// (its own units always; an own unit from the enemy's side through the disclosed enemy sight).
    pub const IS_VISIBLE_TO: u32 = 12;
    /// `(Str attr)` -> value: the ability an `on_ability_executed` status event is for (see
    /// [`event_ability_attr`]); `None` in any other call.
    pub const EVENT_ABILITY_ATTR: u32 = 13;
    /// `(Unit u)` -> `None`: v3 `end_channel { interrupted = true }`: `u` (the hook's own unit;
    /// v3 names no other) ends the ability it is channelling now, as an interrupted channel (its
    /// on_channel_end runs with interrupted). Nothing happens when it channels nothing.
    pub const END_CHANNEL: u32 = 14;
    /// `(Point p)` -> `Bool`: v3 `tree_alive(tree)`: a standing tree's collision square holds `p`
    /// (a tree target is that tree's centre; E403).
    pub const TREE_ALIVE: u32 = 15;
    /// `(Unit u, Str status, Str state…)` -> `Bool`: a live status on `u` is immune to v3 status
    /// `status` holding these host state buffs (catalog `immune_to`: `tag:<tag>`, a status id or
    /// a v3 state; E445), so applying it does nothing.
    pub const STATUS_IMMUNE: u32 = 16;
    /// `(Str "cooldown")` -> `None`: v3 `refund { cooldown = true }` in a cast: the cast's own
    /// cooldown does not start (原作 EndCooldown right after the cast; E455). Its mana is spent.
    pub const REFUND: u32 = 17;
    /// `(Unit, Num slot)` -> `Str` | `None`: v3 `ability_in_slot(unit, slot)` (E497): the id of
    /// the ability at index `slot` (0, 1, 2, ...) of the hero's ability list; `None` for a unit
    /// that is not a hero, or an empty slot.
    pub const ABILITY_IN_SLOT: u32 = 18;
    /// `(Unit holder, Str ability, Str hook, Str message)` -> `None`: a generated hook's `Err`
    /// (E517), which the hook's dispatch would otherwise drop. The host logs it and records it
    /// in its script error log when it keeps one (conformance fails the scenario).
    pub const SCRIPT_ERROR: u32 = 19;
    /// `(Point center, Num radius, Unit of, Str team, Num within)` -> `Units`: combat units removed
    /// (dead) within the last `within` seconds within `radius` of `center` on `team` (`"ally"` /
    /// `"enemy"` / `"both"`) relative to `of`, nearest first (E532, v3 selector flag
    /// `include_dead`: 原作 DOTA_UNIT_TARGET_FLAG_DEAD finds corpses). Their last position is
    /// `get_pos`.
    pub const RETIRED_IN_RANGE: u32 = 20;
    /// `(Num team, Str kind)` -> `Unit`: the match landmark of `team` (E534, v3 `landmark`):
    /// `"base"` / `"fort"` its base structure; `None` when the match has none (destroyed, or no
    /// match) or for a kind the map does not keep as a unit.
    pub const LANDMARK: u32 = 21;
}

/// Attribute names of [`op::EVENT_ABILITY_ATTR`].
pub mod event_ability_attr {
    /// `Str`: the ability id; an item used is `item:<item id>`.
    pub const ID: &str = "id";
    /// `Num`: its cast point in seconds at the rank the caster holds (0 for an item).
    pub const CAST_POINT: &str = "cast_point";
    /// `Bool`: a toggle ability.
    pub const IS_TOGGLE: &str = "is_toggle";
    /// `Point`: where it was cast: a point-target cast's point (the cursor), a unit-target
    /// cast's target position when it executed, otherwise (no target, an item) the caster's.
    pub const POINT: &str = "point";
    /// `Num`: its cast range at the rank the caster holds (0 for an item); E465.
    pub const CAST_RANGE: &str = "cast_range";
    /// `Num`: in a damage status event (on_take_damage / on_deal_damage / on_kill), the hit's total
    /// before the target's mitigation (v3 `event.damage.raw`); 0 elsewhere. E478.
    pub const RAW_DAMAGE: &str = "raw_damage";
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
    /// `Bool`: on no team: a jungle camp or a non-combatant neutral (faction `HostileNeutral` /
    /// `Neutral`); v3 `is_enemy` counts the jungle as an enemy, this tells it apart (E447).
    pub const IS_NEUTRAL: &str = "is_neutral";
    /// `Bool`: attacks at range (states `melee_attack` / `ranged_attack` override).
    pub const IS_RANGED: &str = "is_ranged";
    /// `Unit`: the summoner of a summon; `None` otherwise.
    pub const OWNER: &str = "owner";
    /// `Str`: the hero a unit shows: a hero's own id, an illusion's copied unit (omoba-core
    /// illusion `look`); `None` for other units. Hero selections that include illusions (Dota's
    /// HERO type, v3 `include_illusions`) read it.
    pub const LOOK: &str = "look";
    /// `Unit`: the unit it is attacking; `None` when idle.
    pub const ATTACK_TARGET: &str = "attack_target";
    /// `Num`: final attack speed multiplier (1 = base).
    pub const ATTACK_SPEED_MULT: &str = "attack_speed_mult";
    /// `Num`: attack projectile speed.
    pub const PROJECTILE_SPEED: &str = "projectile_speed";
    /// `Num`: v3 `avg_attack_against(u, target)`: the physical damage of `u`'s next ordinary attack
    /// as it leaves, base and bonuses (the next-attack bonus included), without a critical. The
    /// target changes nothing: armor and the building modifiers apply at the hit (omoba-core
    /// rules::attack / rules::damage). 0 without an attack.
    pub const AVG_ATTACK: &str = "avg_attack";
    /// `Num`: armor above the base (statuses, attributes and items; negative when reduced).
    pub const BONUS_ARMOR: &str = "bonus_armor";
    /// `Num`: base magic resistance (without bonuses).
    pub const BASE_MAGIC_RESIST: &str = "base_magic_resist";
    /// `Num`: base attack range (without bonuses).
    pub const BASE_ATTACK_RANGE: &str = "base_attack_range";
    /// `Num`: cast range bonus the unit's statuses grant.
    pub const CAST_RANGE_BONUS: &str = "cast_range_bonus";
    /// `Num`: the unit's collision radius (0 without one).
    pub const COLLISION_RADIUS: &str = "collision_radius";
    /// `Num`: whole gold in the pool of the unit's player (one pool per player, FC-129～131: no
    /// reliable / unreliable split); 0 when the world keeps none for it (an enemy on a team replica).
    pub const GOLD: &str = "gold";
    /// `Bool`: v3 `has_tag(unit, "boss" | "dummy" | "summon" | "neutral" | "fountain" | "general")`,
    /// the name after `tag:` (e.g. `"tag:boss"`). A map boss carries the boss tag; a training dummy
    /// is a dummy; a summon is a unit some unit summoned (an illusion included) or a summon
    /// template; neutral is a jungle unit (faction HostileNeutral). This game has no fountain or
    /// general units, so those are always false.
    pub const TAG_PREFIX: &str = "tag:";
    /// `Num`: v3 `event.attack.damage` in an attack trigger: the physical damage of the unit's
    /// current (or last) attack as it left, with its critical multiplier; 0 before it leaves or
    /// without one (omoba-core rules::attack AttackRecords).
    pub const ATTACK_DAMAGE: &str = "attack_damage";
    /// `Num`: v3 `event.attack.effect_multiplier` of that attack: 1 for an ordinary attack; 0
    /// without one.
    pub const ATTACK_EFFECT_MULTIPLIER: &str = "attack_effect_multiplier";
    /// `Bool`: v3 `has_status_tag(unit, tag)`, the tag after `status_tag:`: some status on the
    /// unit carries that catalog tag.
    pub const STATUS_TAG_PREFIX: &str = "status_tag:";
}

/// Attribute names of [`op::UNIT_ABILITY_ATTR`].
pub mod ability_attr {
    /// `Num`: seconds left on the ability's cooldown (0 when ready or unknown).
    pub const COOLDOWN_REMAINING: &str = "cooldown_remaining";
    /// `Bool`: learned, off cooldown and (a charge ability) holding a charge.
    pub const IS_READY: &str = "is_ready";
    /// `Num`: its full cooldown in seconds at the level the unit holds (0 when not learned; E497).
    pub const COOLDOWN: &str = "cooldown";
    /// `Bool`: a passive ability (catalog `ability_type`).
    pub const IS_PASSIVE: &str = "is_passive";
    /// `Bool`: a toggle ability (catalog `ability_type`).
    pub const IS_TOGGLE: &str = "is_toggle";
}

/// Attribute names of [`op::GAME_ATTR`].
pub mod game_attr {
    /// `Bool`: night (only `temporary_night` makes night; there is no day cycle).
    pub const IS_NIGHT: &str = "is_night";
    /// `Num`: match time in seconds.
    pub const TIME: &str = "time";
    /// `Num`: seconds of match time one tick advances.
    pub const DT: &str = "dt";
}
