//! Host components work line B adds for the generated Rust (M5 and later; `Cx` methods beyond
//! codegen.rs). Each method only emits a closed host packet; the host settles it in serial order,
//! so the DLL keeps no state. Signatures are listed in the shared notes (ability-kit-api.md).
use abi_stable::std_types::{RSlice, RStr};
use omoba_core::runtime::native::{mana_burn, script_gold};

use crate::codegen::{Cx, Dmg, Res};
use omb_script_abi::ext::{op, ExtArg, ExtValue};
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

    /// v3 `burn_mana`: removes up to `amount` mana from `to` (a unit without mana loses nothing).
    /// It deals no damage; the document's own `damage` action does.
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
    pub fn damage_at(&mut self, ability: &str, path: &str, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg) -> Res {
        // Only a packet `damage` really sends may carry a tag (a tag belongs to the next outcome).
        let sends = to.is_valid() && amount > Fixed64::ZERO && amount <= Fixed64::from_i32(1_000_000);
        if sends && matches!(self.world.ext(op::DAMAGE_TRACE_ENABLED, RSlice::from_slice(&[])).into_result(), Ok(ExtValue::Bool(true))) {
            let args = [ExtArg::Str(ability.into()), ExtArg::Str(path.into())];
            let _ = self.world.ext(op::DAMAGE_TAG, RSlice::from_slice(&args));
        }
        self.damage(to, amount, kind, flags)
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
            states: filter.states.iter().map(|s| (*s).to_owned()).collect(), limit };
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
