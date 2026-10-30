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
    /// `value` is read against this script's snapshot of the mana: the host applies the change
    /// from that snapshot, so mana spent in the same tick (the cast's cost) is not undone.
    pub fn set_mana(&mut self, to: EntityHandle, value: Fixed64) -> Res {
        if !to.is_valid() { return Ok(()); }
        let seen = self.world.current_mana(to).max(Fixed64::ZERO);
        let spec = mana_burn::SetSpec::checked(value.max(Fixed64::ZERO)).and_then(|s| s.relative_to(seen))
            .ok_or("v3: set_mana out of range")?;
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

    /// v3 `knockback { who, from, distance, duration, stun }`: pushes `who` `distance` straight
    /// away from the point `from` over `duration` (host `forced_motion` Knockback: stops at walls,
    /// no move commands while it lasts; stunned for the duration unless `stun` is false). `who`
    /// is the caster itself or a hostile unit within `range` of it. For `from` = a unit, pass its
    /// position. A negative `distance` pulls `who` toward `from` instead (瀧川一益). Not supported:
    /// `stop_on_contact`, `on_end`.
    pub fn knockback(&mut self, who: EntityHandle, from: Vec2, distance: Fixed64, duration: Fixed64, stun: bool, range: Fixed64) -> Res {
        self.knockback_arc(who, from, distance, duration, stun, range, Fixed64::ZERO)
    }

    /// [`Cx::knockback`] with v3 `height`: the arc's peak height, for the presentation only (the
    /// simulation is flat). Zero is no arc.
    pub fn knockback_arc(&mut self, who: EntityHandle, from: Vec2, distance: Fixed64, duration: Fixed64, stun: bool, range: Fixed64,
        height: Fixed64) -> Res {
        use omoba_core::runtime::native::forced_motion as motion;
        if !who.is_valid() || (distance == Fixed64::ZERO && height <= Fixed64::ZERO) { return Ok(()); }
        // A pull toward `from` is a push away from its mirror image across `who`; a toss in place
        // (distance 0 with a height, 七本槍) travels one raw unit, so the host keeps its motion rules.
        let (from, distance) = if distance < Fixed64::ZERO {
            let at = self.pos(who)?;
            (at + (at - from), -distance)
        } else if distance == Fixed64::ZERO {
            let at = self.pos(who)?;
            (at - Vec2::new(Fixed64::ONE, Fixed64::ZERO), Fixed64::from_raw(1))
        } else { (from, distance) };
        if !motion::valid_duration(duration) { return Err("v3: invalid knockback duration"); }
        let spec = motion::Spec { source_id: self.caster.id, source_generation: self.caster.gen, mode: motion::Mode::Knockback,
            distance: distance.min(Fixed64::from_i32(motion::MAX_DISTANCE)), cast_range: range.max(Fixed64::ONE).min(Fixed64::from_i32(10_000)), to: None }
            .checked().ok_or("v3: invalid knockback")?;
        let height = (height > Fixed64::ZERO).then(|| height.min(Fixed64::from_i32(motion::MAX_HEIGHT)));
        let payload = spec.payload_with(motion::Options { origin: Some(from), stun, height });
        self.world.add_stat_buff(who, RStr::from_str(motion::BUFF), duration, RStr::from_str(&payload.to_string()));
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
        self.damage_flagged_at(ability, path, to, amount, kind, flags, extra, false)
    }

    /// [`Cx::damage_ex_at`] with the v3 flag `reflection`: the target's statuses see it as reflected
    /// (`event.damage.is_reflection`, `StatusEvent::is_reflection`).
    pub fn damage_reflect_at(&mut self, ability: &str, path: &str, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg,
        extra: DmgExtra) -> Res {
        self.damage_flagged_at(ability, path, to, amount, kind, flags, extra, true)
    }

    /// [`Cx::damage_reflect_at`] dealt by `source` (`damage.source`).
    pub fn damage_reflect_from_at(&mut self, ability: &str, path: &str, source: EntityHandle, to: EntityHandle, amount: Fixed64,
        kind: DamageKind, flags: Dmg, extra: DmgExtra) -> Res {
        if !source.is_valid() { return Ok(()); }
        let me = self.self_unit;
        self.self_unit = source;
        let r = self.damage_flagged_at(ability, path, to, amount, kind, flags, extra, true);
        self.self_unit = me;
        r
    }

    #[allow(clippy::too_many_arguments)]
    fn damage_flagged_at(&mut self, ability: &str, path: &str, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg,
        extra: DmgExtra, reflection: bool) -> Res {
        use omb_script_abi::types::DamageProfile;
        if !to.is_valid() || amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { return Ok(()); }
        let mut profile = DamageProfile::NORMAL;
        if reflection { profile = profile.union(DamageProfile::REFLECTION); }
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
    /// [`Cx::damage_ex_at`] dealt by `source` (`damage.source` with flags or credit).
    pub fn damage_ex_from_at(&mut self, ability: &str, path: &str, source: EntityHandle, to: EntityHandle, amount: Fixed64,
        kind: DamageKind, flags: Dmg, extra: DmgExtra) -> Res {
        if !source.is_valid() { return Ok(()); }
        let me = self.self_unit;
        self.self_unit = source;
        let r = self.damage_ex_at(ability, path, to, amount, kind, flags, extra);
        self.self_unit = me;
        r
    }

    /// v3 `has_tag(unit, "boss" | "dummy" | "fountain" | "general")`.
    pub fn has_unit_tag(&mut self, unit: EntityHandle, tag: &str) -> bool {
        self.unit_flag(unit, &format!("{}{tag}", omb_script_abi::ext::unit_attr::TAG_PREFIX))
    }

    /// v3 `kill { to, silent }`: `to` dies at once, credited to the caster (omoba-core
    /// script_kill; no damage packet, so invulnerability and shields do not stop it). `silent`
    /// removes a non-hero without bounty or death.
    pub fn kill(&mut self, to: EntityHandle, silent: bool) -> Res {
        use omoba_core::runtime::native::script_kill;
        if !to.is_valid() { return Ok(()); }
        let packet = script_kill::payload(self.caster.id, self.caster.gen, silent).to_string();
        self.world.add_stat_buff(to, RStr::from_str(script_kill::BUFF), Fixed64::ONE, RStr::from_str(&packet));
        Ok(())
    }

    /// v3 `has_status_tag(unit, tag)`: some status on `unit` carries catalog tag `tag`.
    pub fn has_status_tagged(&mut self, unit: EntityHandle, tag: &str) -> bool {
        self.unit_flag(unit, &format!("{}{tag}", omb_script_abi::ext::unit_attr::STATUS_TAG_PREFIX))
    }

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
    /// v3 `ability_of(unit, id).cooldown`: the ability's full cooldown at `unit`'s rank of it
    /// (the catalog's level data; rank 1 when not learned).
    pub fn ability_cooldown_total(&mut self, unit: EntityHandle, ability: &str) -> Fixed64 {
        let rank = u8::try_from((self.ability_level(unit, ability).raw() >> 10).clamp(1, 255)).unwrap_or(1);
        crate::metadata::ability_def(ability).and_then(|def| def.get_level_data(rank).map(|level| level.cooldown))
            .filter(|c| c.is_finite() && *c >= 0.0)
            .map_or(Fixed64::ZERO, |c| Fixed64::from_raw((f64::from(c) * 1024.0).round() as i64))
    }

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

    /// v3 `is_visible_to(x, u)`: whether `u`'s team can see `x` now (host ext IS_VISIBLE_TO).
    pub fn is_visible_to(&mut self, x: EntityHandle, u: EntityHandle) -> bool {
        matches!(self.read(op::IS_VISIBLE_TO, &[ExtArg::Unit(x), ExtArg::Unit(u)]), ExtValue::Bool(true))
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

    /// The units a v3 selector with `team = "ally"` (the caster's team, the caster included) or
    /// `"both"` walks within `radius` of `center`, filtered like [`Cx::enemies_in_circle`]: types,
    /// alive, invulnerable; magic immunity only excludes enemies. In entity id order.
    pub fn units_in_circle(&mut self, center: Vec2, radius: Fixed64, team: &str, select: crate::codegen::Select) -> Vec<EntityHandle> {
        use crate::codegen::UnitTypes;
        let caster = self.caster;
        let mut units: Vec<EntityHandle> = self.units_in_range(center, radius, caster, team).into_iter()
            .filter(|e| self.alive(*e))
            .filter(|e| match select.types {
                UnitTypes::All => true,
                UnitTypes::NonBuilding => !self.world.is_building(*e),
                UnitTypes::Heroes => !self.world.is_building(*e) && self.is_hero(*e),
            })
            .filter(|e| select.include_invulnerable || !self.world.has_buff(*e, RStr::from_str("invulnerable")))
            .filter(|e| select.include_hidden || !crate::codegen::hidden_excluded(*e))
            .collect();
        let caster_faction: Option<String> = self.world.faction_of(caster).into_option().map(|f| f.as_str().to_owned());
        units.retain(|e| {
            let enemy = match (caster_faction.as_deref(), self.world.faction_of(*e).into_option()) { (Some(a), Some(b)) => a != b.as_str(), _ => false };
            !enemy || select.include_magic_immune || !self.is_magic_immune(*e)
        });
        units.sort_by_key(|e| (e.id, e.gen));
        units.dedup();
        units
    }

    /// How many independent instances of `status` (`stacking = "independent"`) `unit` holds;
    /// `has_status` of such a status is `status_instances(..) > 0`.
    pub fn status_instances(&mut self, unit: EntityHandle, status: &crate::codegen::StatusDef) -> u32 {
        let id = self.runtime_id(status);
        match self.read(op::STATUS_INSTANCES, &[ExtArg::Unit(unit), ExtArg::Str(id.as_str().into())]) {
            ExtValue::Num(n) => u32::try_from(n.raw() >> 10).unwrap_or(0),
            _ => 0,
        }
    }

    /// v3 `remove_status` of an independent status with `limit = 1`: the earliest instance goes
    /// (with the shared states only it held).
    pub fn remove_status_instance(&mut self, from: EntityHandle, status: &crate::codegen::StatusDef) -> Res {
        use omoba_core::runtime::native::dispel::{remove_instance_payload, REMOVE_INSTANCE_BUFF};
        if !from.is_valid() { return Ok(()); }
        let id = self.runtime_id(status);
        self.world.add_stat_buff(from, RStr::from_str(REMOVE_INSTANCE_BUFF), Fixed64::ONE, RStr::from_str(&remove_instance_payload(&id).to_string()));
        Ok(())
    }

    /// v3 `reveal` with `ignore_terrain = true` (flying sight: trees and cliffs do not block it);
    /// otherwise the same as `Cx::reveal`.
    pub fn reveal_ex(&mut self, at: Vec2, radius: Fixed64, duration: Fixed64, all_teams: bool, follow: Option<EntityHandle>,
        ignore_terrain: bool) -> Res {
        use omoba_core::runtime::native::reveal_zone;
        let sight = reveal_zone::sight_payload(self.caster.id, self.caster.gen, at, radius, all_teams,
            follow.map(|u| (u.id, u.gen))).ok_or("v3: invalid reveal")?;
        let sight = if ignore_terrain { reveal_zone::with_ignore_terrain(sight) } else { sight };
        self.world.add_stat_buff(self.caster, RStr::from_str(reveal_zone::BUFF), duration, RStr::from_str(&sight.to_string()));
        Ok(())
    }

    /// v3 `status.absorb` of a status just applied on `to` (call after `apply_status`): absorbs
    /// up to `amount` of the damage `types` (`"physical"`, `"magical"`, `"pure"`; empty: all) after
    /// mitigation; used up, the status is removed. `duration` is the status's.
    pub fn absorb(&mut self, to: EntityHandle, status: &crate::codegen::StatusDef, amount: Fixed64, types: &[&str], duration: Fixed64) -> Res {
        use omoba_core::runtime::native::absorb::{Spec, Types, BUFF};
        if !to.is_valid() || amount <= Fixed64::ZERO { return Ok(()); }
        let types = Types::from_names(types).ok_or("v3: unknown absorb type")?;
        let spec = Spec { status: self.runtime_id(status), amount: amount.min(Fixed64::from_i32(1_000_000)), types };
        self.world.add_stat_buff(to, RStr::from_str(BUFF), duration, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `modify_attack { crit_multiplier }` in `on_attack_windup`: `attacker`'s attack in windup
    /// crits with at least this multiplier. Other modify_attack fields are not supported.
    pub fn modify_attack_crit(&mut self, attacker: EntityHandle, crit_multiplier: Fixed64) -> Res {
        use omoba_core::runtime::native::rules::attack::{modify_payload, MODIFY_BUFF};
        if !attacker.is_valid() || crit_multiplier <= Fixed64::ONE { return Ok(()); }
        let multiplier = crit_multiplier.min(Fixed64::from_i32(100));
        self.world.add_stat_buff(attacker, RStr::from_str(MODIFY_BUFF), Fixed64::ONE, RStr::from_str(&modify_payload(multiplier).to_string()));
        Ok(())
    }

    /// v3 `attack { attacker, target, instant, cannot_miss, fake, trigger_effects, use_orb,
    /// effect_multiplier }` (omoba-core perform_attack, 原作 PerformAttack): `attacker` hits `target`
    /// now with its ordinary attack, outside its own attack cycle. Only `instant` attacks at the
    /// ordinary effect multiplier (1) are supported; anything else is an error.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_attack(&mut self, attacker: EntityHandle, target: EntityHandle, instant: bool, cannot_miss: bool, fake: bool,
        trigger_effects: bool, use_orb: bool, effect_multiplier: Fixed64) -> Res {
        use omoba_core::runtime::native::perform_attack::{payload, Flags, BUFF};
        if !instant { return Err("v3: attack without instant (a projectile attack) is not supported"); }
        if effect_multiplier != Fixed64::ONE { return Err("v3: attack.effect_multiplier other than 1 is not supported"); }
        if !attacker.is_valid() || !target.is_valid() { return Ok(()); }
        let packet = payload(target.id, target.gen, Flags { cannot_miss, fake, trigger_effects, use_orb }).to_string();
        self.world.add_stat_buff(attacker, RStr::from_str(BUFF), Fixed64::ONE, RStr::from_str(&packet));
        Ok(())
    }

    /// v3 `create_unit.overrides` on a unit this cast just created (omoba-core unit_overrides):
    /// `rows` are `hp`, `attack_damage`, `attack_damage_max`, `move_speed`; they replace the
    /// template's values right after the spawn. Call it right after `create_unit`.
    pub fn unit_overrides(&mut self, unit: EntityHandle, rows: &[(&str, Fixed64)]) -> Res {
        use omoba_core::runtime::native::unit_overrides;
        if !unit.is_valid() { return Ok(()); }
        let packet = unit_overrides::payload(rows).ok_or("v3: invalid create_unit overrides")?.to_string();
        self.world.add_stat_buff(unit, RStr::from_str(unit_overrides::BUFF), Fixed64::ONE, RStr::from_str(&packet));
        Ok(())
    }

    /// v3 `create_unit` with `control`: `"owner"` and `"ai"` are [`Cx::create_unit`]'s summon
    /// (its owner may order it, its AI acts); `"none"` also refuses every order of its owner for
    /// its whole life (host `command_restricted`, summon_orders), its AI still acting.
    pub fn create_unit_controlled(&mut self, template: &str, at: Vec2, lifetime: Fixed64, control: &str) -> Result<EntityHandle, &'static str> {
        use omoba_core::runtime::native::command_restriction::BUFF;
        let unit = self.create_unit(template, at, lifetime)?;
        match control {
            "owner" | "ai" => {}
            "none" => if unit.is_valid() {
                let life = if lifetime > Fixed64::ZERO { lifetime } else { Self::FOREVER };
                self.world.add_buff(unit, RStr::from_str(BUFF), life);
            },
            _ => return Err("v3: unknown create_unit control"),
        }
        Ok(unit)
    }

    /// v3 `aura` of a passive or a status of `ability` on `holder` (host omoba-core `aura`): every
    /// `aura::PERIOD` the host calls `on_aura(holder, aura, level, receivers)` with the living units
    /// within `radius` on `team` (`"ally"`, `"enemy"`, `"both"`; the holder only with
    /// `include_self`), nearest first; the generated code filters them and applies the status with
    /// `linger + aura::PERIOD`. A passive aura (`passive`, duration FOREVER) is quiet while its
    /// holder is broken; a status aura names its status (`requires`) and goes with it.
    #[allow(clippy::too_many_arguments)]
    pub fn emit_aura(&mut self, holder: EntityHandle, ability: &str, aura: &str, radius: Fixed64, team: &str, include_self: bool,
        duration: Fixed64, requires: Option<&crate::codegen::StatusDef>, passive: bool) -> Res {
        use omoba_core::runtime::native::aura::{Spec, Team};
        if !holder.is_valid() || duration <= Fixed64::ZERO || radius <= Fixed64::ZERO { return Ok(()); }
        let requires = requires.map(|status| self.runtime_id(status));
        let spec = Spec { ability: ability.to_owned(), aura: aura.to_owned(), level: self.rank.max(1),
            radius: radius.min(Fixed64::from_i32(omoba_core::runtime::native::aura::MAX_RADIUS)),
            team: Team::parse(team).ok_or("v3: unknown aura team")?, include_self, requires, passive }
            .checked().ok_or("v3: invalid aura")?;
        self.world.add_stat_buff(holder, RStr::from_str(&spec.buff_id()), duration, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// v3 `game.is_night`.
    pub fn is_night(&mut self) -> bool {
        matches!(self.read(op::GAME_ATTR, &[ExtArg::Str(game_attr::IS_NIGHT.into())]), ExtValue::Bool(true))
    }

    /// v3 `game.time`: match seconds.
    pub fn game_time(&mut self) -> Fixed64 {
        match self.read(op::GAME_ATTR, &[ExtArg::Str(game_attr::TIME.into())]) { ExtValue::Num(n) => n, _ => Fixed64::ZERO }
    }

    /// v3 `game.dt`: the match time one tick advances.
    pub fn game_dt(&mut self) -> Fixed64 {
        match self.read(op::GAME_ATTR, &[ExtArg::Str(game_attr::DT.into())]) { ExtValue::Num(n) => n, _ => Fixed64::ZERO }
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
