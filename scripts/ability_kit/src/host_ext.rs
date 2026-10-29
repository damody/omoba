//! Host components work line B adds for the generated Rust (M5 and later; `Cx` methods beyond
//! codegen.rs). Each method only emits a closed host packet; the host settles it in serial order,
//! so the DLL keeps no state. Signatures are listed in the shared notes (ability-kit-api.md).
use abi_stable::std_types::{RSlice, RStr};
use omoba_core::runtime::native::{mana_burn, script_gold};

use crate::codegen::{Cx, Dmg, Res};
use omb_script_abi::ext::{ability_attr, game_attr, op, ExtArg, ExtValue};

use omb_script_abi::types::{DamageKind, EntityHandle, Fixed64, Vec2};

/// Whole gold units of an amount (the fraction is dropped, as the original's integer gold).
fn whole_gold(amount: Fixed64) -> i64 {
    amount.raw() >> 10
}

impl Cx<'_, '_> {
    /// v3 `give_gold` (`reliable` is ignored: one gold pool, FC-129～131). Grants the whole-gold part
    /// of `amount` to the owner of `to`'s pool; zero or less does nothing.
    pub fn give_gold(&mut self, to: EntityHandle, amount: Fixed64) -> Res {
        self.gold(to, whole_gold(amount))
    }

    /// v3 `take_gold`: charges the whole-gold part of `amount`; the pool never goes below zero
    /// (a document that needs "only when they have enough" checks `gold` first, as C06T does).
    pub fn take_gold(&mut self, from: EntityHandle, amount: Fixed64) -> Res {
        self.gold(from, -whole_gold(amount))
    }

    fn gold(&mut self, unit: EntityHandle, delta: i64) -> Res {
        if !unit.is_valid() || delta == 0 { return Ok(()); }
        let spec = script_gold::Spec::checked(delta).ok_or("v3: gold amount out of range")?;
        self.world.add_stat_buff(unit, RStr::from_str(script_gold::BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `unit.gold`: the whole gold of the pool `u`'s player holds (unit attribute `gold`; one pool
    /// per player, FC-129～131). 0 when the world keeps no pool for it.
    pub fn gold_of(&mut self, u: EntityHandle) -> Fixed64 {
        self.unit_num(u, omb_script_abi::ext::unit_attr::GOLD)
    }

    /// v3 `create_pickup` with a gold `on_pickup` (C06T 豪奪的金袋, 原作 item_money): a body at
    /// `at` that the nearest hero `who` allows ("any", "allies", "enemies" of the caster's team)
    /// takes within 150, gaining the whole-gold part of `gold`; `killable` lets one hit destroy it;
    /// `lifetime` 0 keeps it until taken or destroyed (omoba-core pickup, E243).
    pub fn create_pickup(&mut self, at: Vec2, gold: Fixed64, who: &str, killable: bool, lifetime: Fixed64) -> Res {
        use omoba_core::runtime::native::pickup;
        let who = match who { "any" => pickup::Who::Any, "allies" => pickup::Who::Allies, "enemies" => pickup::Who::Enemies,
            _ => return Err("v3: invalid pickup who") };
        let spec = pickup::Spec { at, gold: whole_gold(gold).max(0), who, killable, lifetime }.checked().ok_or("v3: invalid pickup")?;
        self.world.add_stat_buff(self.caster, RStr::from_str(pickup::BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `burn_mana`: removes up to `amount` mana from `to` (a unit without mana loses nothing).
    /// It deals no damage; the document's own `damage` action does.
    /// v3 `burn_mana` with `source`: the original's ReduceMana credits no one, so the burn is the same
    /// as [`Cx::burn_mana`]; an invalid source burns nothing.
    pub fn burn_mana_from(&mut self, source: EntityHandle, to: EntityHandle, amount: Fixed64) -> Res {
        if !source.is_valid() { return Ok(()); }
        self.burn_mana(to, amount)
    }

    /// v3 `set_mana`: sets `to`'s mana to `value`, clamped to its pool (a unit without mana is
    /// unchanged). Works in every script context (status events, schedules), unlike the cast-only
    /// `restore_mana`; `min(self.mana + x, self.max_mana)` is the way to restore outside a cast.
    pub fn set_mana(&mut self, to: EntityHandle, value: Fixed64) -> Res {
        if !to.is_valid() { return Ok(()); }
        let spec = mana_burn::SetSpec::checked(value.max(Fixed64::ZERO)).ok_or("v3: set_mana out of range")?;
        self.world.add_stat_buff(to, RStr::from_str(mana_burn::SET_BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    pub fn burn_mana(&mut self, to: EntityHandle, amount: Fixed64) -> Res {
        if !to.is_valid() || amount <= Fixed64::ZERO { return Ok(()); }
        let spec = mana_burn::Spec::checked(amount).ok_or("v3: mana burn out of range")?;
        self.world.add_stat_buff(to, RStr::from_str(mana_burn::BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }
}

impl Cx<'_, '_> {
    /// v3 `dash` (to a point): the caster travels in a straight line to `to` over `duration`
    /// (host `forced_motion` Dash: rooted against commands, faces the travel, stops at walls).
    /// A dash of zero length does nothing.
    pub fn dash_to(&mut self, to: Vec2, duration: Fixed64) -> Res {
        use omoba_core::runtime::native::forced_motion as motion;
        let from = self.pos(self.caster)?;
        let length = (to - from).length();
        if length <= Fixed64::ZERO { return Ok(()); }
        if !motion::valid_duration(duration) { return Err("v3: invalid dash duration"); }
        let spec = motion::Spec { source_id: self.caster.id, source_generation: self.caster.gen, mode: motion::Mode::Dash,
            distance: length, cast_range: length + Fixed64::ONE, to: Some(to) }.checked().ok_or("v3: invalid dash")?;
        self.world.add_stat_buff(self.caster, RStr::from_str(motion::BUFF), duration, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `teleport` with `find_clear_space`: the caster lands at `preferred` if it can stand
    /// there, otherwise at the nearest walkable point the same distance from `anchor` (host
    /// `blink_landing`), and faces `anchor`. `range` bounds the caster–anchor distance.
    pub fn teleport_clear(&mut self, anchor: Vec2, preferred: Vec2, range: Fixed64) -> Res {
        use omoba_core::runtime::native::blink_landing as landing;
        let spec = landing::Spec { source_id: self.caster.id, source_generation: self.caster.gen, anchor, preferred,
            cast_range: range }.checked().ok_or("v3: invalid teleport")?;
        self.world.add_stat_buff(self.caster, RStr::from_str(landing::BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `dodge_projectiles`: retires every dodgeable homing ability projectile and every
    /// attack projectile tracking the caster, without settling them.
    pub fn dodge_projectiles(&mut self) -> Res {
        use omoba_core::runtime::native::ability_projectile as projectile;
        let payload = projectile::dodge_payload(self.caster.id, self.caster.gen);
        self.world.add_stat_buff(self.caster, RStr::from_str(projectile::DODGE_BUFF), Fixed64::ONE, RStr::from_str(&payload.to_string()));
        Ok(())
    }
}

impl Cx<'_, '_> {
    /// `damage` tagged with its ability and v3 effect path (e.g. `"events.on_cast[1].actions[2]"`),
    /// so a test world's damage trace (omoba-core `damage_trace`) can tell which action dealt it.
    /// Outside such a world the tag is not sent; the damage is exactly `damage`.
    /// Whether the world records a test damage trace (`DAMAGE_TRACE_ENABLED`).
    pub(crate) fn damage_trace_enabled(&mut self) -> bool {
        matches!(self.world.ext(op::DAMAGE_TRACE_ENABLED, RSlice::from_slice(&[])).into_result(), Ok(ExtValue::Bool(true)))
    }

    pub fn damage_at(&mut self, ability: &str, path: &str, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg) -> Res {
        // Only a packet `damage` really sends may carry a tag (a tag belongs to the next outcome).
        let sends = to.is_valid() && amount > Fixed64::ZERO && amount <= Fixed64::from_i32(1_000_000);
        if sends && matches!(self.world.ext(op::DAMAGE_TRACE_ENABLED, RSlice::from_slice(&[])).into_result(), Ok(ExtValue::Bool(true))) {
            let args = [ExtArg::Str(ability.into()), ExtArg::Str(path.into())];
            let _ = self.world.ext(op::DAMAGE_TAG, RSlice::from_slice(&args));
        }
        self.damage(to, amount, kind, flags)
    }

    /// `damage` with the flags `Dmg` does not carry (v3 `hp_removal`, `ignore_armor`) and an
    /// optional kill credit (`damage.credit`). `hp_removal` is never spell-amplified.
    pub fn damage_ex_at(&mut self, ability: &str, path: &str, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg,
        extra: DmgExtra) -> Res {
        use omb_script_abi::types::DamageProfile;
        if !to.is_valid() || amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { return Ok(()); }
        let mut profile = DamageProfile::NORMAL;
        if flags.require_source_alive { profile = profile.union(DamageProfile::REQUIRE_SOURCE_ALIVE); }
        if !flags.require_source_alive && !flags.no_amplify && !extra.hp_removal { profile = profile.union(DamageProfile::SPELL); }
        if extra.hp_removal { profile = profile.union(DamageProfile::HP_REMOVAL); }
        if extra.ignore_armor { profile = profile.union(DamageProfile::IGNORE_ARMOR); }
        // The credit, then the trace tag, then the packet (each belongs to what follows it).
        if let Some(credit) = extra.credit.filter(EntityHandle::is_valid) {
            let _ = self.world.ext(op::DAMAGE_CREDIT, RSlice::from_slice(&[ExtArg::Unit(credit)]));
        }
        if matches!(self.world.ext(op::DAMAGE_TRACE_ENABLED, RSlice::from_slice(&[])).into_result(), Ok(ExtValue::Bool(true))) {
            let args = [ExtArg::Str(ability.into()), ExtArg::Str(path.into())];
            let _ = self.world.ext(op::DAMAGE_TAG, RSlice::from_slice(&args));
        }
        self.world.deal_damage(to, amount, kind, profile, abi_stable::std_types::RSome(self.self_unit));
        Ok(())
    }

    /// `damage_from` (`damage.source`) with the same trace tag as [`Cx::damage_at`].
    pub fn damage_from_at(&mut self, ability: &str, path: &str, source: EntityHandle, to: EntityHandle, amount: Fixed64,
        kind: DamageKind, flags: Dmg) -> Res {
        if !source.is_valid() { return Ok(()); }
        let me = self.self_unit;
        self.self_unit = source;
        let r = self.damage_at(ability, path, to, amount, kind, flags);
        self.self_unit = me;
        r
    }

    /// v3 `ability_level(unit, ability)`: the unit's rank of that ability, 0 when not learned.
    pub fn ability_level(&mut self, unit: EntityHandle, ability: &str) -> Fixed64 {
        let args = [ExtArg::Unit(unit), ExtArg::Str(ability.into())];
        match self.world.ext(op::ABILITY_LEVEL, RSlice::from_slice(&args)).into_result() {
            Ok(ExtValue::Num(level)) => level,
            _ => Fixed64::ZERO,
        }
    }
}

impl Cx<'_, '_> {
    /// v3 state `taunt` (C06T 豪奪): `to` holds the status `taunt` for `duration`, is ordered to
    /// attack `by` at once (a hero) and refuses its owner's orders meanwhile (omoba-core taunt.rs).
    pub fn taunt(&mut self, to: EntityHandle, by: EntityHandle, duration: Fixed64) -> Res {
        use omoba_core::runtime::native::taunt;
        if !to.is_valid() || !by.is_valid() || duration <= Fixed64::ZERO { return Ok(()); }
        let payload = taunt::payload(by.id, by.gen as i32);
        self.world.add_stat_buff(to, RStr::from_str(taunt::BUFF), duration, RStr::from_str(&payload.to_string()));
        Ok(())
    }

    /// Removing a v3 status with the state `taunt` ends the taunt it holds (its attack order stays).
    pub fn end_taunt(&mut self, from: EntityHandle) -> Res {
        use omoba_core::runtime::native::taunt;
        if !from.is_valid() { return Ok(()); }
        self.world.remove_buff(from, RStr::from_str(taunt::BUFF));
        Ok(())
    }
}

impl Cx<'_, '_> {
    /// v3 `set_ability_active`: an inactive ability of `of` cannot be cast until activated again.
    pub fn set_ability_active(&mut self, of: EntityHandle, ability: &str, active: bool) -> Res {
        use omoba_core::runtime::native::ability_slots::{ActiveSpec, ACTIVE_BUFF};
        if !of.is_valid() { return Ok(()); }
        let spec = ActiveSpec { ability: ability.to_owned(), active };
        self.world.add_stat_buff(of, RStr::from_str(ACTIVE_BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `swap_abilities`: `to` takes `from`'s slot on `on` (with `from`'s rank when it has
    /// none), swapping back after `duration` when given. `reset` clears `to`'s cooldown;
    /// `continue` keeps each ability's own (`freeze` is not supported).
    pub fn swap_abilities(&mut self, on: EntityHandle, from: &str, to: &str, duration: Option<Fixed64>, cooldown_policy: &str) -> Res {
        use omoba_core::runtime::native::ability_slots::{CooldownPolicy, SwapSpec, SWAP_BUFF};
        if !on.is_valid() { return Ok(()); }
        let policy = match cooldown_policy {
            "continue" => CooldownPolicy::Continue, "reset" => CooldownPolicy::Reset,
            _ => return Err("v3: unsupported swap_abilities cooldown_policy"),
        };
        let duration = match duration {
            None => None,
            Some(d) if d > Fixed64::ZERO && d <= Fixed64::from_i32(3600) => Some(d),
            Some(_) => return Err("v3: swap_abilities duration out of range"),
        };
        let spec = SwapSpec { from: from.to_owned(), to: to.to_owned(), policy, duration };
        self.world.add_stat_buff(on, RStr::from_str(SWAP_BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }
}

/// v3 attribute reads the trait has no method for (`GameWorld::ext` UNIT_ATTR / UNIT_ABILITY_ATTR /
/// GAME_ATTR; the names are `omb_script_abi::ext::{unit_attr, ability_attr, game_attr}`). A dead or
/// unknown unit reads as zero / false / no unit. An unknown name is a generator bug: it panics in
/// debug builds and reads as zero in release.
impl Cx<'_, '_> {
    fn read(&mut self, op: u32, args: &[ExtArg]) -> ExtValue {
        match self.world.ext(op, RSlice::from_slice(args)).into_result() {
            Ok(value) => value,
            Err(e) => { debug_assert!(false, "{e}"); ExtValue::None }
        }
    }

    /// A numeric unit attribute, e.g. `cx.unit_num(u, unit_attr::INT)` for v3 `u.int`.
    pub fn unit_num(&mut self, unit: EntityHandle, attr: &str) -> Fixed64 {
        match self.read(op::UNIT_ATTR, &[ExtArg::Unit(unit), ExtArg::Str(attr.into())]) {
            ExtValue::Num(n) => n,
            _ => Fixed64::ZERO,
        }
    }

    /// A boolean unit attribute, e.g. `cx.unit_flag(u, unit_attr::IS_ILLUSION)`.
    pub fn unit_flag(&mut self, unit: EntityHandle, attr: &str) -> bool {
        matches!(self.read(op::UNIT_ATTR, &[ExtArg::Unit(unit), ExtArg::Str(attr.into())]), ExtValue::Bool(true))
    }

    /// A unit-valued attribute (`owner`, `attack_target`); an invalid handle when there is none.
    pub fn unit_ref(&mut self, unit: EntityHandle, attr: &str) -> EntityHandle {
        match self.read(op::UNIT_ATTR, &[ExtArg::Unit(unit), ExtArg::Str(attr.into())]) {
            ExtValue::Unit(u) => u,
            _ => EntityHandle::INVALID,
        }
    }

    /// v3 `ability.cooldown_remaining` of `unit`'s ability.
    pub fn ability_cooldown(&mut self, unit: EntityHandle, ability: &str) -> Fixed64 {
        let args = [ExtArg::Unit(unit), ExtArg::Str(ability.into()), ExtArg::Str(ability_attr::COOLDOWN_REMAINING.into())];
        match self.read(op::UNIT_ABILITY_ATTR, &args) { ExtValue::Num(n) => n, _ => Fixed64::ZERO }
    }

    /// v3 `ability.is_ready`: learned, off cooldown, holding a charge.
    pub fn ability_ready(&mut self, unit: EntityHandle, ability: &str) -> bool {
        let args = [ExtArg::Unit(unit), ExtArg::Str(ability.into()), ExtArg::Str(ability_attr::IS_READY.into())];
        matches!(self.read(op::UNIT_ABILITY_ATTR, &args), ExtValue::Bool(true))
    }

    /// Units within `radius` of `center` on `team` (`"ally"` / `"enemy"` / `"both"`) relative to
    /// `of`, nearest first; the caller's selector filters types, dead units and the rest.
    pub fn units_in_range(&mut self, center: Vec2, radius: Fixed64, of: EntityHandle, team: &str) -> Vec<EntityHandle> {
        let args = [ExtArg::Point(center), ExtArg::Num(radius), ExtArg::Unit(of), ExtArg::Str(team.into())];
        match self.read(op::UNITS_IN_RANGE, &args) { ExtValue::Units(units) => units.into_vec(), _ => Vec::new() }
    }

    /// v3 `game.is_night`.
    pub fn is_night(&mut self) -> bool {
        matches!(self.read(op::GAME_ATTR, &[ExtArg::Str(game_attr::IS_NIGHT.into())]), ExtValue::Bool(true))
    }

    /// v3 `game.time`: match seconds.
    pub fn game_time(&mut self) -> Fixed64 {
        match self.read(op::GAME_ATTR, &[ExtArg::Str(game_attr::TIME.into())]) { ExtValue::Num(n) => n, _ => Fixed64::ZERO }
    }
}

/// v3 damage flags beyond [`Dmg`], for [`Cx::damage_ex_at`].
#[derive(Clone, Copy, Debug, Default)]
pub struct DmgExtra {
    /// `flags = { "hp_removal" }`: life paid as a cost; shields do not absorb it.
    pub hp_removal: bool,
    /// `flags = { "ignore_armor" }`: physical damage ignores armor.
    pub ignore_armor: bool,
    /// `credit`: who gets the kill and assist (default: the source).
    pub credit: Option<EntityHandle>,
}

/// The filters of a v3 `remove_status` without a single `status` (vocab: `dispel`, `only`,
/// `team_of_source`, `except`, `states`, `limit`).
#[derive(Clone, Copy, Debug)]
pub struct StatusFilter<'s> {
    /// `"basic"`, `"strong"`, `"all"`, `"all_debuffs"`, `"all_buffs"`; `None` when the action names
    /// only `states` (then every class).
    pub dispel: Option<&'s str>,
    /// `"debuffs"` / `"buffs"` / `None`.
    pub only: Option<&'s str>,
    /// `"enemy"` / `"ally"` / `"any"`.
    pub team_of_source: &'s str,
    pub except: &'s [&'s str],
    pub states: &'s [&'s str],
    pub limit: Option<Fixed64>,
}

impl Cx<'_, '_> {
    /// v3 `remove_status` by filters: the host removes the statuses of `from` they name
    /// (omoba-core `dispel::purge`), shared controls following the statuses holding them.
    pub fn remove_statuses(&mut self, from: EntityHandle, filter: StatusFilter<'_>) -> Res {
        self.remove_statuses_tagged(from, filter, &[], &[])
    }

    /// [`Cx::remove_statuses`] with v3 `tags` (only statuses carrying one of them) and
    /// `exclude_tags` (statuses carrying one of them stay); the host reads statuses' tags from the
    /// compiled catalog.
    pub fn remove_statuses_tagged(&mut self, from: EntityHandle, filter: StatusFilter<'_>, tags: &[&str], exclude_tags: &[&str]) -> Res {
        use omoba_core::runtime::native::dispel::{Purge, PurgeMode, PurgeSpec, SourceTeam, PURGE_BUFF};
        if !from.is_valid() { return Ok(()); }
        let (mode, side) = match filter.dispel {
            None | Some("all") => (PurgeMode::All, None),
            Some("basic") => (PurgeMode::Basic, None),
            Some("strong") => (PurgeMode::Strong, None),
            Some("all_debuffs") => (PurgeMode::All, Some(Purge::Debuffs)),
            Some("all_buffs") => (PurgeMode::All, Some(Purge::Buffs)),
            Some(_) => return Err("v3: unknown dispel"),
        };
        let side = match filter.only {
            None => side,
            Some("debuffs") => Some(Purge::Debuffs),
            Some("buffs") => Some(Purge::Buffs),
            Some(_) => return Err("v3: unknown remove_status only"),
        };
        let team = match filter.team_of_source {
            "any" => SourceTeam::Any, "enemy" => SourceTeam::Enemy, "ally" => SourceTeam::Ally,
            _ => return Err("v3: unknown team_of_source"),
        };
        let limit = match filter.limit {
            None => None,
            Some(n) if n <= Fixed64::ZERO => return Ok(()),
            Some(n) => Some(u32::try_from(n.raw() >> 10).map_err(|_| "v3: remove_status limit")?.max(1)),
        };
        let spec = PurgeSpec { mode, side, team, except: filter.except.iter().map(|s| (*s).to_owned()).collect(),
            states: filter.states.iter().map(|s| (*s).to_owned()).collect(), limit,
            tags: tags.iter().map(|s| (*s).to_owned()).collect(), exclude_tags: exclude_tags.iter().map(|s| (*s).to_owned()).collect() };
        self.world.add_stat_buff(from, RStr::from_str(PURGE_BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gold_amounts_drop_the_fraction() {
        assert_eq!(whole_gold(Fixed64::from_i32(14)), 14);
        assert_eq!(whole_gold(Fixed64::from_raw(14 * 1024 + 900)), 14);
        assert_eq!(whole_gold(Fixed64::from_raw(900)), 0);
    }
}
