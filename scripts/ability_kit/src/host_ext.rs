//! Host components work line B adds for the generated Rust (M5 and later; `Cx` methods beyond
//! codegen.rs). Each method only emits a closed host packet; the host settles it in serial order,
//! so the DLL keeps no state. Signatures are listed in the shared notes (ability-kit-api.md).
use abi_stable::std_types::{RSlice, RStr};
use omoba_core::runtime::native::{mana_burn, script_gold};

use crate::codegen::{Cx, Dmg, Res};
use omb_script_abi::ext::{ability_attr, game_attr, op, ExtArg, ExtValue};

use omb_script_abi::types::{DamageKind, EntityHandle, Fixed64, Target, Vec2};

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
        self.knockback_ended(who, from, distance, duration, stun, range, height, None, None)
    }

    /// [`Cx::knockback_arc`] with v3 `stop_on_contact` + `contact_radius` (E494: it also stops
    /// when another living unit that is not a building, of any team, is ahead of `who` within
    /// that edge distance) and `on_end` (`(ability, step, captures)`: `on_scheduled(step)` of
    /// `ability` once on the caster when the motion ends for any reason, with `who`'s last
    /// position appended to the captures).
    #[allow(clippy::too_many_arguments)]
    pub fn knockback_ended(&mut self, who: EntityHandle, from: Vec2, distance: Fixed64, duration: Fixed64, stun: bool, range: Fixed64,
        height: Fixed64, contact_radius: Option<Fixed64>, on_end: Option<(&str, u16, &[omb_script_abi::types::ScheduleArg])>) -> Res {
        use omoba_core::runtime::native::{forced_motion as motion, scheduled_effect};
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
        let contact = contact_radius.map(|radius| radius.max(Fixed64::ZERO).min(Fixed64::from_i32(motion::MAX_CONTACT)));
        let mut payload = spec.payload_with(motion::Options { origin: Some(from), stun, height, contact });
        if let Some((ability, step, args)) = on_end {
            let instance = (0..3).fold(0u32, |acc, _| (acc << 10) ^ (self.world.rand_unit().raw() as u32 & 0x3ff));
            let template = scheduled_effect::Spec { ability: ability.to_owned(), level: self.rank.max(1), step, instance, args: args.to_vec() }
                .checked().filter(|spec| spec.args.len() < scheduled_effect::MAX_ARGS).ok_or("v3: invalid knockback on_end")?;
            payload = motion::with_on_end(payload, &template);
        }
        self.world.add_stat_buff(who, RStr::from_str(motion::BUFF), duration, RStr::from_str(&payload.to_string()));
        Ok(())
    }

    /// v3 `modify_base_stat { to, stat, amount }` (E523, 原作 ModifyAgility): `to`'s base (white)
    /// attribute changes by `amount`, rounded to whole points; a unit that is not a hero is unchanged.
    pub fn modify_base_stat(&mut self, to: EntityHandle, stat: crate::Attr, amount: Fixed64) -> Res {
        use omoba_core::runtime::native::{attributes, comp::AttributeType};
        let stat = match stat {
            crate::Attr::Strength => AttributeType::Strength, crate::Attr::Agility => AttributeType::Agility,
            crate::Attr::Intelligence => AttributeType::Intelligence, crate::Attr::Universal => return Err("v3: modify_base_stat of universal"),
        };
        if !to.is_valid() || amount == Fixed64::ZERO { return Ok(()); }
        let payload = attributes::base_stat_payload(stat, amount);
        attributes::base_stat_spec(attributes::BASE_STAT_BUFF, &payload).ok_or("v3: invalid modify_base_stat amount")?;
        self.world.add_stat_buff(to, RStr::from_str(attributes::BASE_STAT_BUFF), Fixed64::ONE, RStr::from_str(&payload.to_string()));
        Ok(())
    }

    /// v3 `impulse { who, direction, speed, decay, duration }` (E514, omoba-core `impulse`): adds a
    /// velocity of `speed` units per physics frame (1/30 s) along `direction` to `who`, kept
    /// `decay` of each frame (0.95: 原作 Physics friction); several add up, and `who` still moves by
    /// itself. A zero direction adds nothing.
    pub fn impulse(&mut self, who: EntityHandle, direction: Vec2, speed: Fixed64, decay: Fixed64, duration: Option<Fixed64>) -> Res {
        use omoba_core::runtime::native::impulse;
        let length = direction.length();
        if !who.is_valid() || length <= Fixed64::ZERO || speed == Fixed64::ZERO { return Ok(()); }
        let speed = speed.max(Fixed64::from_i32(-impulse::MAX_SPEED)).min(Fixed64::from_i32(impulse::MAX_SPEED));
        let velocity = Vec2::new(direction.x * speed / length, direction.y * speed / length);
        let spec = impulse::Spec { velocity, decay, duration }.checked().ok_or("v3: invalid impulse")?;
        self.world.add_stat_buff(who, RStr::from_str(impulse::BUFF), Fixed64::ONE, RStr::from_str(&spec.payload().to_string()));
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
        let me = self.caster;
        self.caster = source;
        let r = self.damage_flagged_at(ability, path, to, amount, kind, flags, extra, true);
        self.caster = me;
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
        if !flags.require_source_alive && !flags.no_amplify && !extra.hp_removal && !extra.no_source_bonuses { profile = profile.union(DamageProfile::SPELL); }
        if extra.hp_removal { profile = profile.union(DamageProfile::HP_REMOVAL); }
        if extra.no_source_bonuses { profile = profile.union(DamageProfile::NO_SOURCE_BONUSES); }
        if extra.ignore_armor { profile = profile.union(DamageProfile::IGNORE_ARMOR); }
        // The credit, then the trace tag, then the packet (each belongs to what follows it).
        if let Some(credit) = extra.credit.filter(EntityHandle::is_valid) {
            let _ = self.world.ext(op::DAMAGE_CREDIT, RSlice::from_slice(&[ExtArg::Unit(credit)]));
        }
        if matches!(self.world.ext(op::DAMAGE_TRACE_ENABLED, RSlice::from_slice(&[])).into_result(), Ok(ExtValue::Bool(true))) {
            let args = [ExtArg::Str(ability.into()), ExtArg::Str(path.into())];
            let _ = self.world.ext(op::DAMAGE_TAG, RSlice::from_slice(&args));
        }
        // the attacker is cx.caster (Cx::damage, E450)
        self.world.deal_damage(to, amount, kind, profile, abi_stable::std_types::RSome(self.caster));
        Ok(())
    }

    /// `damage_from` (`damage.source`) with the same trace tag as [`Cx::damage_at`].
    /// [`Cx::damage_ex_at`] dealt by `source` (`damage.source` with flags or credit).
    pub fn damage_ex_from_at(&mut self, ability: &str, path: &str, source: EntityHandle, to: EntityHandle, amount: Fixed64,
        kind: DamageKind, flags: Dmg, extra: DmgExtra) -> Res {
        if !source.is_valid() { return Ok(()); }
        let me = self.caster;
        self.caster = source;
        let r = self.damage_ex_at(ability, path, to, amount, kind, flags, extra);
        self.caster = me;
        r
    }

    /// v3 `has_tag(unit, "boss" | "dummy" | "summon" | "neutral" | "fountain" | "general")`.
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
        let me = self.caster;
        self.caster = source;
        let r = self.damage_at(ability, path, to, amount, kind, flags);
        self.caster = me;
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

    /// The rank a value of another ability is read at: `unit`'s rank of it, or this program's
    /// rank when `unit` has not learned it (a summon running its owner's routine).
    pub fn rank_of(&mut self, unit: EntityHandle, ability: &str) -> u8 {
        let level = self.ability_level(unit, ability).raw() >> 10;
        if level > 0 { u8::try_from(level.min(255)).unwrap_or(u8::MAX) } else { self.rank }
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

    /// v3 `event.ability.cast_point` in an `on_ability_executed` status trigger (host ext
    /// EVENT_ABILITY_ATTR): its cast point at the caster's rank; 0 for an item or outside one.
    pub fn event_ability_cast_point(&mut self) -> Fixed64 {
        match self.read(op::EVENT_ABILITY_ATTR, &[ExtArg::Str(omb_script_abi::ext::event_ability_attr::CAST_POINT.into())]) {
            ExtValue::Num(seconds) => seconds, _ => Fixed64::ZERO,
        }
    }

    /// v3 `event.damage.raw` in a damage trigger (host ext EVENT_ABILITY_ATTR `raw_damage`,
    /// E478): the hit's total before the target's mitigation; 0 outside one.
    pub fn event_damage_raw(&mut self) -> Fixed64 {
        match self.read(op::EVENT_ABILITY_ATTR, &[ExtArg::Str(omb_script_abi::ext::event_ability_attr::RAW_DAMAGE.into())]) {
            ExtValue::Num(amount) => amount, _ => Fixed64::ZERO,
        }
    }

    /// v3 `event.order.ability.cast_range` in an `on_order_targeted` trigger (host ext
    /// EVENT_ABILITY_ATTR `cast_range`, E465): the ordered ability's cast range at the orderer's
    /// rank; 0 for an attack order or outside one.
    pub fn event_ability_cast_range(&mut self) -> Fixed64 {
        match self.read(op::EVENT_ABILITY_ATTR, &[ExtArg::Str(omb_script_abi::ext::event_ability_attr::CAST_RANGE.into())]) {
            ExtValue::Num(range) => range, _ => Fixed64::ZERO,
        }
    }

    /// v3 `event.point` in an `on_ability_executed` status trigger (host ext EVENT_ABILITY_ATTR
    /// `point`): a point cast's point (the cursor), a unit cast's target position, otherwise the
    /// caster's position; None outside one.
    pub fn event_ability_point(&mut self) -> Option<omb_script_abi::types::Vec2> {
        match self.read(op::EVENT_ABILITY_ATTR, &[ExtArg::Str(omb_script_abi::ext::event_ability_attr::POINT.into())]) {
            ExtValue::Point(point) => Some(point), _ => None,
        }
    }

    /// v3 `event.point` of an `on_ability_executed` trigger: the host's (a status trigger), else
    /// the cast's own target (an owner's passive: a point, a unit's position, else the caster's).
    pub fn event_cast_point(&mut self) -> Result<Vec2, &'static str> {
        if let Some(point) = self.event_ability_point() { return Ok(point); }
        match self.target {
            Target::None => self.pos(self.self_unit),
            _ => self.target_point(),
        }
    }

    /// Whether `u` is of the v3 unit class `class` (`has_tag(u, <class>)`): hero, creep,
    /// building, summon or neutral (host unit_attr `tag:summon` / `tag:neutral`).
    pub fn of_class(&mut self, u: EntityHandle, class: &str) -> bool {
        match class {
            "hero" => self.is_hero(u),
            "building" => self.is_building(u),
            "creep" => self.unit_flag(u, omb_script_abi::ext::unit_attr::IS_CREEP),
            "summon" | "neutral" => self.has_unit_tag(u, class),
            _ => false,
        }
    }

    /// v3 `count_in_radius(center, radius, team, class)`: the living units of `class` within
    /// `radius` of `center` on `team` (`"ally"` / `"enemy"`) relative to `of`, as a selector
    /// without flags takes them (invulnerable left out, magic-immune enemies left out).
    pub fn count_in_radius(&mut self, center: Vec2, radius: Fixed64, of: EntityHandle, team: &str, class: &str) -> Fixed64 {
        let units = self.units_in_range(center, radius, of, team);
        let mut n = 0;
        for u in units {
            if !self.alive(u) || !self.of_class(u, class) { continue; }
            if self.world.has_buff(u, RStr::from_str("invulnerable")) { continue; }
            if self.is_enemy(of, u) && self.is_magic_immune(u) { continue; }
            n += 1;
        }
        Fixed64::from_i32(n)
    }

    /// v3 `end_channel { interrupted = true }`: `unit` (the hook's own) ends its channel now; its
    /// on_channel_end runs with interrupted (host ext END_CHANNEL).
    pub fn end_channel(&mut self, unit: EntityHandle) {
        if unit.is_valid() { let _ = self.read(op::END_CHANNEL, &[ExtArg::Unit(unit)]); }
    }

    /// v3 `refund { cooldown = true }` in a cast: its own cooldown does not start (host ext
    /// REFUND, E455). No other refund exists.
    pub fn refund_cooldown(&mut self) { let _ = self.read(op::REFUND, &[ExtArg::Str("cooldown".into())]); }

    /// v3 `immune_to` (E445): a status on `to` refuses `status` (host ext STATUS_IMMUNE).
    pub fn status_immune(&mut self, to: EntityHandle, status: &crate::codegen::StatusDef) -> bool {
        let mut args = vec![ExtArg::Unit(to), ExtArg::Str(status.id.into())];
        args.extend(status.state_buffs.iter().map(|state| ExtArg::Str((*state).into())));
        matches!(self.read(op::STATUS_IMMUNE, &args), ExtValue::Bool(true))
    }

    /// v3 `tree_alive(tree)`: the tree at `at` (a tree target, its centre) still stands (E403).
    pub fn tree_alive(&mut self, at: omb_script_abi::types::Vec2) -> bool {
        matches!(self.read(op::TREE_ALIVE, &[ExtArg::Point(at)]), ExtValue::Bool(true))
    }

    /// The skill of an `on_ability_executed` status trigger (`item:<id>` for an item); None
    /// outside one.
    pub fn event_ability_id(&mut self) -> Option<String> {
        match self.read(op::EVENT_ABILITY_ATTR, &[ExtArg::Str(omb_script_abi::ext::event_ability_attr::ID.into())]) {
            ExtValue::Str(id) => Some(id.into()), _ => None,
        }
    }

    /// v3 `event.ability.is_toggle` in an `on_ability_executed` status trigger.
    pub fn event_ability_is_toggle(&mut self) -> bool {
        matches!(self.read(op::EVENT_ABILITY_ATTR, &[ExtArg::Str(omb_script_abi::ext::event_ability_attr::IS_TOGGLE.into())]), ExtValue::Bool(true))
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

    /// v3 `ability_in_slot(unit, slot)` (E497): the id of the ability at index `slot` (0, 1, 2, ...)
    /// of the hero `unit`'s ability list; None for a unit that is not a hero or an empty slot.
    /// A generated hook's `Err` (E517): the host logs it and records it for conformance instead of
    /// the dispatch dropping it. The caster is the holder.
    pub fn report_error(&mut self, ability: &str, hook: &str, error: &str) {
        let args = [ExtArg::Unit(self.caster), ExtArg::Str(ability.into()), ExtArg::Str(hook.into()), ExtArg::Str(error.into())];
        let _ = self.world.ext(op::SCRIPT_ERROR, RSlice::from_slice(&args));
    }

    pub fn ability_in_slot(&mut self, unit: EntityHandle, slot: u8) -> Option<String> {
        match self.read(op::ABILITY_IN_SLOT, &[ExtArg::Unit(unit), ExtArg::Num(Fixed64::from_i32(i32::from(slot)))]) {
            ExtValue::Str(id) => Some(id.into()), _ => None,
        }
    }

    fn ability_num(&mut self, unit: EntityHandle, ability: &str, attr: &str) -> Fixed64 {
        let args = [ExtArg::Unit(unit), ExtArg::Str(ability.into()), ExtArg::Str(attr.into())];
        match self.read(op::UNIT_ABILITY_ATTR, &args) { ExtValue::Num(n) => n, _ => Fixed64::ZERO }
    }

    fn ability_flag(&mut self, unit: EntityHandle, ability: &str, attr: &str) -> bool {
        let args = [ExtArg::Unit(unit), ExtArg::Str(ability.into()), ExtArg::Str(attr.into())];
        matches!(self.read(op::UNIT_ABILITY_ATTR, &args), ExtValue::Bool(true))
    }

    /// v3 `ability.is_passive` (E497).
    pub fn ability_is_passive(&mut self, unit: EntityHandle, ability: &str) -> bool { self.ability_flag(unit, ability, ability_attr::IS_PASSIVE) }
    /// v3 `ability.is_toggle` (E497).
    pub fn ability_is_toggle(&mut self, unit: EntityHandle, ability: &str) -> bool { self.ability_flag(unit, ability, ability_attr::IS_TOGGLE) }

    /// v3 `modify_cooldown { ability = <an ability of another unit>, mode = "start" }` without an
    /// amount (E497): `unit`'s `ability` starts its own full cooldown at the level it holds (原作
    /// StartCooldown(GetCooldown)); nothing when it is not learned.
    pub fn cooldown_start_own(&mut self, unit: EntityHandle, ability: &str) -> Res {
        let seconds = self.ability_num(unit, ability, ability_attr::COOLDOWN);
        if seconds <= Fixed64::ZERO { return Ok(()); }
        self.cooldown_start(unit, ability, seconds)
    }

    /// Units within `radius` of `center` on `team` (`"ally"` / `"enemy"` / `"both"`) relative to
    /// `of`, nearest first; the caller's selector filters types, dead units and the rest.
    pub fn units_in_range(&mut self, center: Vec2, radius: Fixed64, of: EntityHandle, team: &str) -> Vec<EntityHandle> {
        let args = [ExtArg::Point(center), ExtArg::Num(radius), ExtArg::Unit(of), ExtArg::Str(team.into())];
        match self.read(op::UNITS_IN_RANGE, &args) { ExtValue::Units(units) => units.into_vec(), _ => Vec::new() }
    }

    /// v3 `landmark(team, kind)` (E534): the team's base for `base` / `fort`;
    /// `EntityHandle::INVALID` when there is none.
    /// The team number of `unit` (`unit_attr::TEAM`, E538): what [`Cx::landmark`] takes.
    pub fn team_id(&mut self, unit: EntityHandle) -> Fixed64 { self.unit_num(unit, omb_script_abi::ext::unit_attr::TEAM) }

    /// [`Cx::landmark`] of `unit`'s team (v3 `landmark(team_of(u), kind)`).
    pub fn landmark_of(&mut self, unit: EntityHandle, kind: &str) -> EntityHandle {
        let team = self.team_id(unit);
        self.landmark(team, kind)
    }

    pub fn landmark(&mut self, team: Fixed64, kind: &str) -> EntityHandle {
        match self.read(op::LANDMARK, &[ExtArg::Num(team), ExtArg::Str(kind.into())]) {
            ExtValue::Unit(u) => u,
            _ => EntityHandle::INVALID,
        }
    }

    /// v3 selector flag `include_dead` (E532): the dead units within `radius` of `center` on
    /// `team` relative to the caster: heroes waiting to respawn and combat units removed within
    /// the last `within` seconds (their bodies), nearest first. Types and `where` are the
    /// caller's; a removed unit's type is no longer known, only its team.
    pub fn dead_in_circle(&mut self, center: Vec2, radius: Fixed64, team: &str, within: Fixed64) -> Vec<EntityHandle> {
        let caster = self.caster;
        let mut units: Vec<EntityHandle> = self.units_in_range(center, radius, caster, team).into_iter()
            .filter(|e| !self.alive(*e)).collect();
        let args = [ExtArg::Point(center), ExtArg::Num(radius), ExtArg::Unit(caster), ExtArg::Str(team.into()), ExtArg::Num(within)];
        if let ExtValue::Units(gone) = self.read(op::RETIRED_IN_RANGE, &args) { units.extend(gone.into_vec()); }
        let mut seen = std::collections::BTreeSet::new();
        units.retain(|u| seen.insert((u.id, u.gen)));
        units
    }

    /// The units a v3 selector with `team = "ally"` (the caster's team, the caster included) or
    /// `"both"` walks within `radius` of `center`, filtered like [`Cx::enemies_in_circle`]: types,
    /// alive, invulnerable; magic immunity only excludes enemies. In entity id order.
    pub fn units_in_circle(&mut self, center: Vec2, radius: Fixed64, team: &str, select: crate::codegen::Select) -> Vec<EntityHandle> {
        let caster = self.caster;
        let mut units: Vec<EntityHandle> = self.units_in_range(center, radius, caster, team).into_iter()
            .filter(|e| self.alive(*e))
            .filter(|e| self.of_select(*e, select))
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

    /// v3 `remove_status` of its own independent status inside one of its triggers (E504): only
    /// the instance whose trigger runs (`event_instance`, row `<runtime id>#<n>`) goes, with its
    /// own slow and control rows (dispel::remove_status); the other instances keep running.
    /// Nothing outside an instance's trigger (`event_instance` 0). Its vars are the generator's
    /// keys `__sv:<status>#<n>:<var>`, which no later instance reuses (n only grows).
    pub fn remove_this_instance(&mut self, from: EntityHandle, status: &crate::codegen::StatusDef) -> Res {
        let instance = self.event_instance;
        if !from.is_valid() || instance == 0 { return Ok(()); }
        let id = format!("{}#{instance}", self.runtime_id(status));
        self.world.remove_buff(from, RStr::from_str(&id));
        Ok(())
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
    /// v3 unit_template `abilities` (E529): non-hero `unit` holds `ability` at `level` (its owner
    /// casts it through a summon cast order; omoba-core unit_abilities). A hero is left alone: its
    /// abilities are its slots.
    pub fn grant_unit_ability(&mut self, unit: EntityHandle, ability: &str, level: Fixed64) -> Res {
        use omoba_core::runtime::native::unit_abilities;
        if !unit.is_valid() || self.is_hero(unit) { return Ok(()); }
        let level = u8::try_from((crate::floor(level).raw() >> 10).clamp(1, i64::from(unit_abilities::MAX_LEVEL))).unwrap_or(1);
        let payload = unit_abilities::payload(ability, level).ok_or("v3: invalid unit ability")?;
        self.world.add_stat_buff(unit, RStr::from_str(&unit_abilities::row_id(ability)), Self::FOREVER,
            RStr::from_str(&payload.to_string()));
        Ok(())
    }

    /// [`Cx::reveal_ex`] for v3 `reveal.teams` (E527): `"own"`, `"all"` or `"enemy"` (every team but
    /// the caster's: the caster shown to its enemies).
    pub fn reveal_teams(&mut self, at: Vec2, radius: Fixed64, duration: Fixed64, teams: &str, follow: Option<EntityHandle>,
        ignore_terrain: bool) -> Res {
        use omoba_core::runtime::native::reveal_zone;
        if teams != "enemy" { return self.reveal_ex(at, radius, duration, teams == "all", follow, ignore_terrain); }
        let sight = reveal_zone::sight_payload(self.caster.id, self.caster.gen, at, radius, false,
            follow.map(|u| (u.id, u.gen))).ok_or("v3: invalid reveal")?;
        let sight = reveal_zone::with_enemy_teams(sight);
        let sight = if ignore_terrain { reveal_zone::with_ignore_terrain(sight) } else { sight };
        self.world.add_stat_buff(self.caster, RStr::from_str(reveal_zone::BUFF), duration, RStr::from_str(&sight.to_string()));
        Ok(())
    }

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
    /// crits with at least this multiplier.
    pub fn modify_attack_crit(&mut self, attacker: EntityHandle, crit_multiplier: Fixed64) -> Res {
        self.modify_attack(attacker, Some(crit_multiplier), false, false)
    }

    /// v3 `modify_attack { crit_multiplier?, cannot_miss, is_orb }` in `on_attack_windup` (E488):
    /// `attacker`'s attack in windup crits with at least the multiplier (None or 1 or less: as
    /// rolled), cannot miss (no miss at launch, no evasion on arrival), is an orb attack.
    pub fn modify_attack(&mut self, attacker: EntityHandle, crit_multiplier: Option<Fixed64>, cannot_miss: bool, is_orb: bool) -> Res {
        use omoba_core::runtime::native::rules::attack::{modify_payload_with, Modify, MODIFY_BUFF};
        let crit_multiplier = crit_multiplier.filter(|multiplier| *multiplier > Fixed64::ONE).map(|multiplier| multiplier.min(Fixed64::from_i32(100)));
        if !attacker.is_valid() || (crit_multiplier.is_none() && !cannot_miss && !is_orb) { return Ok(()); }
        let packet = modify_payload_with(Modify { crit_multiplier, cannot_miss, is_orb }).to_string();
        self.world.add_stat_buff(attacker, RStr::from_str(MODIFY_BUFF), Fixed64::ONE, RStr::from_str(&packet));
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
        if !instant && !trigger_effects { return Err("v3: attack without instant needs trigger_effects (a projectile attack is an attack)"); }
        if effect_multiplier != Fixed64::ONE { return Err("v3: attack.effect_multiplier other than 1 is not supported"); }
        if !attacker.is_valid() || !target.is_valid() { return Ok(()); }
        let packet = payload(target.id, target.gen, Flags { instant, cannot_miss, fake, trigger_effects, use_orb }).to_string();
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
        let caster = self.caster;
        self.create_unit_controlled_for(caster, template, at, lifetime, control)
    }

    /// [`Cx::create_unit_controlled`] owned by `owner` ([`Cx::create_unit_for`], E500).
    pub fn create_unit_controlled_for(&mut self, owner: EntityHandle, template: &str, at: Vec2, lifetime: Fixed64, control: &str)
        -> Result<EntityHandle, &'static str> {
        use omoba_core::runtime::native::command_restriction::BUFF;
        let unit = self.create_unit_for(owner, template, at, lifetime)?;
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

    /// v3 `create_illusion` (omoba-core illusion.rs): an `illusion` summon at `at` for `lifetime`
    /// (`control` as [`Cx::create_unit_controlled`]) that becomes a copy of `of` right after its
    /// spawn: its speed, defenses, attack and the stat rows of its level and items, and what `copy`
    /// names: its life (`hp`, None = `of`'s current), `mana`, `abilities` (ranks and learned
    /// passives, less the excepted) and `statuses` (by tag, except, original duration, the
    /// illusion as their applier). `copy.outgoing` / `copy.incoming` are the damage it deals and
    /// takes per type (physical, magical, pure) as fractions: -0.65 deals 35%, 2.0 takes 300%.
    pub fn create_illusion(&mut self, of: EntityHandle, at: Vec2, lifetime: Fixed64, control: &str,
        copy: &omoba_core::runtime::native::illusion::Copy) -> Result<EntityHandle, &'static str> {
        use omoba_core::runtime::native::illusion::{payload, BUFF, TEMPLATE};
        if copy.hp.is_some_and(|hp| hp <= Fixed64::ZERO) { return Err("v3: create_illusion hp must be positive"); }
        if copy.mana.is_some_and(|mana| mana < Fixed64::ZERO) { return Err("v3: create_illusion mana must not be negative"); }
        if !of.is_valid() { return Ok(EntityHandle::INVALID); }
        let unit = self.create_unit_controlled(TEMPLATE, at, lifetime, control)?;
        if !unit.is_valid() { return Ok(unit); }
        let packet = payload(of.id, of.gen, copy).to_string();
        self.world.add_stat_buff(unit, RStr::from_str(BUFF), Fixed64::ONE, RStr::from_str(&packet));
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
    /// `flags = { "no_source_bonuses" }`: dealt as a dummy would, without the source's bonuses
    /// (DamageProfile::NO_SOURCE_BONUSES, line B fa3e6b9f); the kill still credits the source.
    pub no_source_bonuses: bool,
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
