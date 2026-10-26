//! G-2 pulse channel: one per exact hero life, host-owned like
//! `recovery_channel`. Cast settlement charges cost and cooldown once; each
//! later pulse calls the ability's `AbilityScript::on_tick`, which may only
//! apply effects. Control, silence, movement, death, phase, recall, a new
//! command or a lost unit anchor end the channel before its next pulse.
//!
//! The owning team's replica admits the same cast from its accepted input and
//! runs this lifecycle as a predictor (no `MobaMatch` needed). The authority
//! still decides: its after-batch `owner_control` row sets the exact status and
//! ends the local row, and a rebase seeds it from the owner buff baseline.
use std::collections::BTreeMap;

use omb_script_abi::types::{ChannelPulse, Target};
use omoba_content_model::ChannelAnchor;
use omoba_sim::{Fixed64, Vec2};
use specs::{Entity, World, WorldExt};

use super::comp::{CProperty, DeltaTime, GamePause, Hero, PlayerOwner, Pos, Tick};
use super::{BuffStore, MobaMatch, MobaMatchPhase};

/// Public HUD status: remaining time plus `total_raw`, like the recovery channel.
pub(crate) const STATUS: &str = "ability_channel";
/// Bounded catch-up: a long frame fires at most this many pulses per channel.
pub(crate) const MAX_PULSES_PER_TICK: u16 = 8;
const MIN_INTERVAL_RAW: i64 = 100; // 0.1 s rounded to Fixed64
const MAX_PULSES: u16 = 200;
const MAX_TOTAL_RAW: i64 = 60 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Spec {
    pub anchor: ChannelAnchor,
    pub interval: Fixed64,
    pub pulses: u16,
    /// `None` only for a unit anchor: the pulse hits that unit alone.
    pub radius: Option<Fixed64>,
    /// Cast range for point and unit anchors.
    pub range: Fixed64,
}

impl Spec {
    pub(crate) fn valid(self) -> bool {
        self.interval.raw() >= MIN_INTERVAL_RAW && self.interval <= Fixed64::from_i32(10)
            && (1..=MAX_PULSES).contains(&self.pulses)
            && self.interval.raw().checked_mul(i64::from(self.pulses)).is_some_and(|total| total <= MAX_TOTAL_RAW)
            && self.radius.is_none_or(|radius| radius >= Fixed64::ZERO && radius <= Fixed64::from_i32(2_000))
            && (self.radius.is_some() || self.anchor == ChannelAnchor::Unit)
            && self.range >= Fixed64::ZERO && self.range <= Fixed64::from_i32(10_000)
    }

    pub(crate) fn total(self) -> Fixed64 {
        Fixed64::from_raw(self.interval.raw() * i64::from(self.pulses))
    }
}

/// `None` is not a pulse channel; malformed metadata fails closed.
pub(crate) fn ability_spec(def: &crate::ability_meta::AbilityDef, rank: &crate::ability_meta::AbilityLevelData) -> Result<Option<Spec>, ()> {
    use crate::ability_meta::{AbilityType, CastType, HostAbilityExecution, TargetType};
    let Some(HostAbilityExecution::PulseChannel { anchor, interval_key, pulses_key, radius_key }) = &def.host_execution else {
        return Ok(None);
    };
    let target = match anchor {
        ChannelAnchor::SelfOrigin => TargetType::None,
        ChannelAnchor::Point => TargetType::Point,
        ChannelAnchor::Unit => TargetType::Unit,
    };
    if def.cast_type != CastType::Channeled || def.target_type != target
        || !matches!(def.ability_type, AbilityType::Active | AbilityType::Ultimate)
        // A windup may precede the channel; the release stages it like a 0 s cast.
        || !rank.cast_time.is_finite() || !(0.0..=60.0).contains(&rank.cast_time) {
        return Err(());
    }
    let value = |key: &str| -> Result<f64, ()> {
        let value = rank.extra.get(key).and_then(|v| v.as_f64()).ok_or(())?;
        if key.is_empty() || !value.is_finite() { return Err(()); }
        Ok(value)
    };
    let fixed = |value: f64| Fixed64::from_raw((value * 1024.0).round() as i64);
    let pulses = value(pulses_key)?;
    if pulses.fract() != 0.0 || !(1.0..=f64::from(MAX_PULSES)).contains(&pulses) { return Err(()); }
    let radius = radius_key.as_deref().map(|key| value(key).map(fixed)).transpose()?;
    let range = if *anchor == ChannelAnchor::SelfOrigin { Fixed64::ZERO } else {
        if !rank.range.is_finite() || rank.range < 0.0 { return Err(()); }
        fixed(f64::from(rank.range))
    };
    let spec = Spec { anchor: *anchor, interval: fixed(value(interval_key)?), pulses: pulses as u16, radius, range };
    if spec.valid() { Ok(Some(spec)) } else { Err(()) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Anchor {
    Origin,
    Point(Vec2),
    Unit(Entity),
}

#[derive(Debug)]
struct Channel {
    owner: u32,
    skill_id: String,
    level: u8,
    anchor: Anchor,
    origin: Vec2,
    spec: Spec,
    fired: u16,
    clock: i64,
    last_tick: u64,
}

#[derive(Default, Debug)]
pub(crate) struct AbilityChannels(BTreeMap<Entity, Channel>);

pub(crate) fn any_active(world: &World) -> bool {
    world.try_fetch::<AbilityChannels>().is_some_and(|channels| !channels.0.is_empty())
}

pub(crate) fn diagnostic(world: &World) -> Option<String> {
    world.try_fetch::<AbilityChannels>().filter(|channels| !channels.0.is_empty()).map(|channels| format!("{:?}", channels.0))
}

/// Owner decision gate for bots and command routing.
pub(crate) fn is_active_for_owner(world: &World, entity: Entity, owner: u32) -> bool {
    world.try_fetch::<AbilityChannels>().is_some_and(|channels| channels.0.get(&entity)
        .is_some_and(|channel| channel.owner == owner && channel.fired < channel.spec.pulses))
        && live(world, entity, owner)
}

fn live(world: &World, entity: Entity, owner: u32) -> bool {
    owner > 0 && world.entities().is_alive(entity)
        && world.read_storage::<Hero>().get(entity).is_some()
        && world.read_storage::<PlayerOwner>().get(entity).is_some_and(|p| p.player_id == owner)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
        && world.try_fetch::<MobaMatch>().map_or(true, |s| s.phase == MobaMatchPhase::Playing && !s.is_recalling(entity)
            && s.heroes.iter().any(|h| h.player_id == owner && h.entity == Some(entity)))
        // `is_silenced` also covers stun.
        && world.try_fetch::<BuffStore>().is_some_and(|b| !b.is_silenced(entity))
}

fn in_geometry(point: Vec2) -> bool {
    point.x.raw().unsigned_abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
        && point.y.raw().unsigned_abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
}

fn unit_anchor_live(world: &World, caster: Entity, target: Entity) -> bool {
    target != caster && world.entities().is_alive(target)
        && world.read_storage::<CProperty>().get(target).is_some_and(|p| p.hp > Fixed64::ZERO)
        && world.read_storage::<Pos>().get(target).is_some()
        && match (world.read_storage::<super::comp::Faction>().get(caster), world.read_storage::<super::comp::Faction>().get(target)) {
            (Some(own), Some(other)) => own.is_hostile_to(other),
            _ => false,
        }
}

/// Start admission. Point and unit anchors must be inside cast range now.
pub(crate) fn can_start(world: &World, entity: Entity, owner: u32, spec: Spec, anchor: Anchor) -> bool {
    if !spec.valid() || world.read_resource::<GamePause>().is_paused || !live(world, entity, owner) {
        return false;
    }
    let Some(origin) = world.read_storage::<Pos>().get(entity).map(|p| p.0) else { return false };
    if !in_geometry(origin) { return false; }
    let within = |point: Vec2| (point - origin).length() <= spec.range;
    match (spec.anchor, anchor) {
        (ChannelAnchor::SelfOrigin, Anchor::Origin) => true,
        (ChannelAnchor::Point, Anchor::Point(point)) => in_geometry(point) && within(point),
        (ChannelAnchor::Unit, Anchor::Unit(target)) => unit_anchor_live(world, entity, target)
            && world.read_storage::<Pos>().get(target).is_some_and(|p| within(p.0)),
        _ => false,
    }
}

pub(crate) fn start(world: &mut World, entity: Entity, owner: u32, skill_id: String, level: u8, spec: Spec, anchor: Anchor) {
    assert!(can_start(world, entity, owner, spec, anchor));
    let origin = world.read_storage::<Pos>().get(entity).unwrap().0;
    let tick = world.read_resource::<Tick>().0;
    if world.try_fetch::<AbilityChannels>().is_none() { world.insert(AbilityChannels::default()); }
    log::info!("AbilityChannel start pid={} caster={:?} ability='{}' level={} pulses={} interval_raw={} anchor={:?} tick={}",
        owner, entity, skill_id, level, spec.pulses, spec.interval.raw(), anchor, tick);
    world.write_resource::<AbilityChannels>().0.insert(entity, Channel {
        owner, skill_id, level, anchor, origin, spec, fired: 0, clock: 0, last_tick: tick,
    });
    publish(world, entity, spec.total(), spec.total());
}

/// Absolute public status: `add` keeps the longer remaining time, so a
/// countdown replaces the entry whenever it differs (as `cast_preparation`).
fn publish(world: &World, entity: Entity, remaining: Fixed64, total: Fixed64) {
    let mut buffs = world.write_resource::<BuffStore>();
    let current = buffs.get(entity, STATUS).is_some_and(|entry| entry.remaining == remaining
        && entry.payload[STATUS]["total_raw"].as_i64() == Some(total.raw()));
    if !current {
        buffs.remove(entity, STATUS);
        buffs.add(entity, STATUS, remaining, serde_json::json!({STATUS: {"total_raw": total.raw()}}));
    }
}

pub(crate) fn cancel(world: &World, entity: Entity) {
    if let Some(mut channels) = world.try_fetch_mut::<AbilityChannels>() {
        if let Some(channel) = channels.0.remove(&entity) { log_end(entity, &channel, "cancelled"); }
    }
    if let Some(mut buffs) = world.try_fetch_mut::<BuffStore>() { buffs.remove(entity, STATUS); }
}

/// Replica only: the authority ended this channel, so the local predictor row
/// goes too (no log; the status was already set from the authority row).
pub(crate) fn forget(world: &World, entity: Entity) {
    if let Some(mut channels) = world.try_fetch_mut::<AbilityChannels>() { channels.0.remove(&entity); }
}

pub(crate) fn clear(world: &World) {
    if let Some(mut channels) = world.try_fetch_mut::<AbilityChannels>() {
        for (entity, channel) in std::mem::take(&mut channels.0) { log_end(entity, &channel, "reset"); }
    }
    if let Some(mut buffs) = world.try_fetch_mut::<BuffStore>() {
        let entities: Vec<_> = buffs.entities_with_key(STATUS).collect();
        for entity in entities { buffs.remove(entity, STATUS); }
    }
}

/// Why a live channel must end now, checked in a fixed order before any pulse.
fn end_reason(world: &World, caster: Entity, channel: &Channel) -> Option<&'static str> {
    if !world.entities().is_alive(caster)
        || world.read_storage::<CProperty>().get(caster).is_none_or(|p| p.hp <= Fixed64::ZERO) {
        return Some("dead");
    }
    if world.try_fetch::<BuffStore>().is_some_and(|b| b.is_stunned(caster)) { return Some("stunned"); }
    if world.try_fetch::<BuffStore>().is_some_and(|b| b.is_silenced(caster)) { return Some("silenced"); }
    if world.try_fetch::<MobaMatch>().is_some_and(|s| s.is_recalling(caster)) { return Some("recall"); }
    if !live(world, caster, channel.owner) { return Some("inactive"); }
    if world.read_storage::<Pos>().get(caster).is_none_or(|p| p.0 != channel.origin) { return Some("moved"); }
    if let Anchor::Unit(target) = channel.anchor {
        if !unit_anchor_live(world, caster, target) { return Some("anchor_lost"); }
    }
    None
}

fn log_end(caster: Entity, channel: &Channel, reason: &str) {
    log::info!("AbilityChannel end pid={} caster={:?} ability='{}' pulses={}/{} reason={}",
        channel.owner, caster, channel.skill_id, channel.fired, channel.spec.pulses, reason);
}

/// One due pulse, in deterministic (caster, index) order.
pub(crate) struct DuePulse {
    pub caster: Entity,
    pub skill_id: String,
    pub target: Target,
    pub elapsed: Fixed64,
    pub pulse: ChannelPulse,
}

/// Validates every channel, advances its clock once per tick and returns the
/// pulses due now. A channel that fails validation ends here, before any pulse.
pub(crate) fn advance(world: &mut World) -> Vec<DuePulse> {
    let Some(mut channels) = world.remove::<AbilityChannels>() else { return Vec::new() };
    let tick = world.read_resource::<Tick>().0;
    let dt = world.read_resource::<DeltaTime>().0;
    let paused = world.read_resource::<GamePause>().is_paused;
    let mut due = Vec::new();
    channels.0.retain(|entity, channel| {
        let caster = *entity;
        if let Some(reason) = end_reason(world, caster, channel) {
            log_end(caster, channel, reason);
            world.write_resource::<BuffStore>().remove(caster, STATUS);
            return false;
        }
        if !paused && dt > Fixed64::ZERO && channel.last_tick != tick {
            channel.last_tick = tick;
            channel.clock = channel.clock.saturating_add(dt.raw());
            let mut fired_now = 0;
            while channel.clock >= channel.spec.interval.raw() && channel.fired < channel.spec.pulses
                && fired_now < MAX_PULSES_PER_TICK {
                channel.clock -= channel.spec.interval.raw();
                channel.fired += 1;
                fired_now += 1;
                let anchor = match channel.anchor {
                    Anchor::Origin => channel.origin,
                    Anchor::Point(point) => point,
                    Anchor::Unit(target) => world.read_storage::<Pos>().get(target).map(|p| p.0).expect("live anchor"),
                };
                let target = match channel.anchor {
                    Anchor::Origin => Target::None,
                    Anchor::Point(point) => Target::Point(point),
                    Anchor::Unit(target) => Target::Entity(super::scripting::ParallelWorldAdapter::entity_to_handle(target)),
                };
                log::info!("AbilityChannel pulse pid={} caster={:?} ability='{}' pulse={}/{} tick={}",
                    channel.owner, caster, channel.skill_id, channel.fired, channel.spec.pulses, tick);
                due.push(DuePulse {
                    caster,
                    skill_id: channel.skill_id.clone(),
                    target,
                    elapsed: Fixed64::from_raw(channel.spec.interval.raw() * i64::from(channel.fired)),
                    pulse: ChannelPulse {
                        level: channel.level, index: channel.fired, total: channel.spec.pulses,
                        elapsed: Fixed64::from_raw(channel.spec.interval.raw() * i64::from(channel.fired)), anchor,
                    },
                });
            }
            // A capped catch-up drops the excess time instead of firing it later.
            if fired_now == MAX_PULSES_PER_TICK { channel.clock = channel.clock.min(channel.spec.interval.raw() - 1); }
        }
        if channel.fired >= channel.spec.pulses {
            log_end(caster, channel, "completed");
            world.write_resource::<BuffStore>().remove(caster, STATUS);
            return false;
        }
        let remaining = Fixed64::from_raw(channel.spec.interval.raw() * i64::from(channel.spec.pulses - channel.fired) - channel.clock);
        publish(world, caster, remaining.max(Fixed64::from_raw(1)), channel.spec.total());
        true
    });
    world.insert(channels);
    due
}

/// A pulse applies effects only. Cost, cooldown, a new channel or cast, mana
/// writes and spawns would be a second charge or an unbounded side effect.
pub(crate) fn pulse_outcome_allowed(outcome: &super::comp::Outcome) -> bool {
    use super::comp::Outcome as O;
    !matches!(outcome,
        O::ScriptCastTransaction {..} | O::ScriptCastCommitted {..} | O::ScriptManaCommit {..}
        | O::ScriptSetMana {..} | O::ScriptManaGained {..} | O::ScriptManaSpent {..}
        | O::ScriptStartCooldown {..} | O::ScriptRecoveryChannel {..} | O::ScriptAbilityChannel {..}
        | O::SpawnSummoned {..} | O::ScriptConsumeCharge {..})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(anchor: ChannelAnchor) -> Spec {
        Spec { anchor, interval: Fixed64::from_raw(102), pulses: 50, radius: Some(Fixed64::from_i32(340)), range: Fixed64::from_i32(700) }
    }

    fn fixture() -> (World, Entity) {
        let (mut world, hero) = super::super::moba_match::stop_command_world();
        world.insert(super::super::moba_match::stop_command_match(hero, MobaMatchPhase::Playing));
        world.insert(DeltaTime(Fixed64::ZERO));
        world.insert(BuffStore::default());
        (world, hero)
    }

    fn step(world: &mut World, dt: Fixed64) -> Vec<DuePulse> {
        world.write_resource::<Tick>().0 += 1;
        world.write_resource::<DeltaTime>().0 = dt;
        advance(world)
    }

    #[test]
    fn metadata_is_rank_resolved_bounded_and_fail_closed() {
        let mut def: crate::ability_meta::AbilityDef = serde_json::from_value(serde_json::json!({
            "id": "quake", "name": "Quake", "ability_type": "ultimate", "target_type": "point", "cast_type": "channeled",
            "host_execution": {"kind": "pulse_channel", "anchor": "point", "interval_key": "wave_interval", "pulses_key": "wave_count", "radius_key": "wave_radius"},
        })).unwrap();
        let rank: crate::ability_meta::AbilityLevelData = serde_json::from_value(serde_json::json!({
            "cooldown": 90, "mana_cost": 200, "range": 700, "wave_interval": 0.1, "wave_count": 60, "wave_radius": 340,
        })).unwrap();
        let resolved = ability_spec(&def, &rank).unwrap().unwrap();
        assert_eq!((resolved.interval.raw(), resolved.pulses, resolved.radius), (102, 60, Some(Fixed64::from_i32(340))));
        assert_eq!(resolved.range, Fixed64::from_i32(700));
        for (key, value) in [("wave_count", 0.0), ("wave_count", 60.5), ("wave_count", 201.0), ("wave_interval", 0.05), ("wave_interval", 2.0), ("wave_radius", 2001.0)] {
            let mut bad = rank.clone();
            bad.extra.insert(key.into(), serde_json::json!(value));
            assert!(ability_spec(&def, &bad).is_err(), "{key}={value}");
        }
        def.target_type = crate::ability_meta::TargetType::Unit;
        assert!(ability_spec(&def, &rank).is_err(), "anchor must match target type");
        def.host_execution = None;
        assert_eq!(ability_spec(&def, &rank), Ok(None));
    }

    #[test]
    fn pulses_follow_the_fixed_clock_bounded_per_tick_and_end_exactly() {
        for hz in [15, 30, 60] {
            let (mut world, hero) = fixture();
            let mut spec = spec(ChannelAnchor::SelfOrigin);
            spec.pulses = 10;
            start(&mut world, hero, 7, "quake".into(), 2, spec, Anchor::Origin);
            let mut fired = Vec::new();
            let mut elapsed = 0;
            for tick in 1..=hz * 2 {
                let next = tick * 1024 / hz;
                let due = step(&mut world, Fixed64::from_raw(next - elapsed));
                elapsed = next;
                assert!(due.len() <= usize::from(MAX_PULSES_PER_TICK));
                for pulse in &due {
                    assert_eq!(pulse.pulse.level, 2, "rank locked at cast");
                    assert_eq!(pulse.elapsed.raw(), 102 * i64::from(pulse.pulse.index));
                }
                fired.extend(due.iter().map(|pulse| pulse.pulse.index));
                // A second advance in the same tick never fires again.
                assert!(advance(&mut world).is_empty());
            }
            assert_eq!(fired, (1..=10).collect::<Vec<_>>(), "{hz} Hz");
            assert!(!world.read_resource::<BuffStore>().has(hero, STATUS));
            assert!(diagnostic(&world).is_none());
        }
        // One huge frame is capped and does not bank the excess.
        let (mut world, hero) = fixture();
        start(&mut world, hero, 7, "quake".into(), 1, spec(ChannelAnchor::SelfOrigin), Anchor::Origin);
        assert_eq!(step(&mut world, Fixed64::from_i32(3)).len(), usize::from(MAX_PULSES_PER_TICK));
        assert_eq!(step(&mut world, Fixed64::from_raw(102)).len(), 1);
    }

    #[test]
    fn control_silence_movement_death_phase_and_owner_end_without_a_ghost_pulse() {
        for case in 0..8 {
            let (mut world, hero) = fixture();
            start(&mut world, hero, 7, "quake".into(), 1, spec(ChannelAnchor::SelfOrigin), Anchor::Origin);
            assert_eq!(step(&mut world, Fixed64::from_raw(102)).len(), 1);
            match case {
                0 => world.write_resource::<BuffStore>().add(hero, "stun", Fixed64::ONE, serde_json::json!({})),
                1 => world.write_resource::<BuffStore>().add(hero, "silence", Fixed64::ONE, serde_json::json!({})),
                2 => world.write_storage::<Pos>().get_mut(hero).unwrap().0.x += Fixed64::ONE,
                3 => world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO,
                4 => world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Finished { winner: None, tick: 2 },
                5 => world.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = 8,
                6 => super::super::cast_windup::cancel(&world, hero),
                7 => { world.delete_entity(hero).unwrap(); world.maintain(); }
                _ => unreachable!(),
            }
            assert!(step(&mut world, Fixed64::from_i32(1)).is_empty(), "case {case}");
            assert!(!world.read_resource::<BuffStore>().has(hero, STATUS), "case {case}");
            assert!(diagnostic(&world).is_none(), "case {case}");
        }
    }

    #[test]
    fn end_reasons_name_the_interruption_for_live_logs() {
        let cases: [(&str, fn(&mut World, Entity)); 4] = [
            ("stunned", |w, e| w.write_resource::<BuffStore>().add(e, "stun", Fixed64::ONE, serde_json::json!({}))),
            ("silenced", |w, e| w.write_resource::<BuffStore>().add(e, "silence", Fixed64::ONE, serde_json::json!({}))),
            ("moved", |w, e| w.write_storage::<Pos>().get_mut(e).unwrap().0.x += Fixed64::ONE),
            ("dead", |w, e| w.write_storage::<CProperty>().get_mut(e).unwrap().hp = Fixed64::ZERO),
        ];
        for (expected, apply) in cases {
            let (mut world, hero) = fixture();
            start(&mut world, hero, 7, "quake".into(), 1, spec(ChannelAnchor::SelfOrigin), Anchor::Origin);
            {
                let channels = world.read_resource::<AbilityChannels>();
                assert_eq!(end_reason(&world, hero, &channels.0[&hero]), None);
            }
            apply(&mut world, hero);
            let channels = world.read_resource::<AbilityChannels>();
            assert_eq!(end_reason(&world, hero, &channels.0[&hero]), Some(expected));
        }
    }

    #[test]
    fn pause_freezes_the_clock_and_public_status_counts_down() {
        let (mut world, hero) = fixture();
        let mut spec = spec(ChannelAnchor::SelfOrigin);
        spec.pulses = 4;
        start(&mut world, hero, 7, "quake".into(), 1, spec, Anchor::Origin);
        let remaining = |world: &World| world.read_resource::<BuffStore>().get(hero, STATUS).unwrap().remaining.raw();
        assert_eq!(remaining(&world), 408);
        step(&mut world, Fixed64::from_raw(51));
        assert_eq!(remaining(&world), 357);
        world.write_resource::<GamePause>().is_paused = true;
        for _ in 0..5 { assert!(step(&mut world, Fixed64::from_raw(102)).is_empty()); }
        assert_eq!(remaining(&world), 357);
        world.write_resource::<GamePause>().is_paused = false;
        assert_eq!(step(&mut world, Fixed64::from_raw(51)).len(), 1);
        assert_eq!(remaining(&world), 306);
        let status = world.read_resource::<BuffStore>().get(hero, STATUS).unwrap().payload.clone();
        assert_eq!(status[STATUS]["total_raw"], 408);
    }

    /// Counts pulses; `execute` must never run for a host channel.
    #[derive(Clone)]
    struct Pulser(std::sync::Arc<std::sync::Mutex<Vec<(u8, u16)>>>, std::sync::Arc<std::sync::atomic::AtomicU32>);
    impl omb_script_abi::ability::AbilityScript for Pulser {
        fn ability_id(&self) -> abi_stable::std_types::RStr<'_> { "".into() }
        fn execute(&self, _: omb_script_abi::types::EntityHandle, _: Target, _: u8, _: abi_stable::std_types::RStr<'_>,
            _: &mut omb_script_abi::world::GameWorldDyn<'_>) -> abi_stable::std_types::RResult<(), abi_stable::std_types::RString> {
            self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            abi_stable::std_types::ROk(())
        }
        fn on_channel_pulse(&self, _: omb_script_abi::types::EntityHandle, _: Target, pulse: ChannelPulse,
            _: &mut omb_script_abi::world::GameWorldDyn<'_>) {
            self.0.lock().unwrap().push((pulse.level, pulse.index));
        }
    }

    fn driven_world(pulser: Pulser) -> (World, crate::runtime::SimulationDriver, Entity, String) {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
            warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
        }).unwrap();
        let hero = world.read_resource::<MobaMatch>().heroes.iter().find(|slot| slot.player_id == 1).and_then(|slot| slot.entity).unwrap();
        let id = world.read_storage::<Hero>().get(hero).unwrap().abilities[0].clone();
        let level = serde_json::json!({"cooldown": 20, "mana_cost": 0, "i": 0.1, "n": 5, "r": 300});
        let def: crate::ability_meta::AbilityDef = serde_json::from_value(serde_json::json!({
            "id": id, "name": id, "ability_type": "active", "target_type": "none", "cast_type": "channeled", "max_level": 4,
            "host_execution": {"kind": "pulse_channel", "anchor": "self", "interval_key": "i", "pulses_key": "n", "radius_key": "r"},
            "levels": {"1": level, "2": level, "3": level, "4": level},
        })).unwrap();
        let mut abilities = crate::runtime::ability_runtime::AbilityRegistry::new();
        abilities.register(def.clone());
        world.insert(abilities);
        let mut scripts = crate::runtime::ScriptRegistry::new();
        scripts.insert_ability_for_test(def, omb_script_abi::ability::AbilityScript_TO::from_value(pulser, abi_stable::sabi_trait::prelude::TD_Opaque));
        world.insert(scripts);
        let driver = crate::runtime::SimulationDriver::from_world(&mut world, crate::runtime::SimulationTickProfile::Production60Hz).unwrap();
        (world, driver, hero, id)
    }

    fn cast(world: &mut World, driver: &mut crate::runtime::SimulationDriver) {
        use crate::game_proto::{player_input::Action, CastAbility, PlayerInput};
        driver.step(world, [(1, PlayerInput { action: Some(Action::CastAbility(CastAbility { ability_index: 0, ..Default::default() })) })]).unwrap();
    }

    #[test]
    fn a_real_cast_starts_a_host_channel_and_every_pulse_reaches_on_channel_pulse_once() {
        let pulses = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let executes = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let (mut world, mut driver, hero, id) = driven_world(Pulser(pulses.clone(), executes.clone()));
        driver.step(&mut world, []).unwrap();
        let rank = world.read_storage::<Hero>().get(hero).unwrap().get_ability_level(&id) as u8;
        cast(&mut world, &mut driver);
        assert!(any_active(&world), "settled cast starts the channel");
        assert!(world.read_storage::<Hero>().get(hero).unwrap().is_on_cooldown(&id), "cooldown charged at start");
        assert!(world.read_resource::<BuffStore>().has(hero, STATUS), "public channel bar");
        for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
        assert_eq!(*pulses.lock().unwrap(), (1..=5).map(|index| (rank, index)).collect::<Vec<_>>());
        assert_eq!(executes.load(std::sync::atomic::Ordering::SeqCst), 0, "a host channel never runs execute");
        assert!(!any_active(&world));
        assert!(!world.read_resource::<BuffStore>().has(hero, STATUS));
    }

    #[test]
    fn a_stun_mid_channel_stops_pulses_with_no_refund_or_second_charge() {
        let pulses = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let (mut world, mut driver, hero, id) = driven_world(Pulser(pulses.clone(), Default::default()));
        driver.step(&mut world, []).unwrap();
        cast(&mut world, &mut driver);
        let cooldown = world.read_storage::<Hero>().get(hero).unwrap().get_cooldown(&id);
        while pulses.lock().unwrap().len() < 2 { driver.step(&mut world, []).unwrap(); }
        world.write_resource::<BuffStore>().add(hero, "stun", Fixed64::from_i32(1), serde_json::json!({}));
        for _ in 0..40 { driver.step(&mut world, []).unwrap(); }
        assert_eq!(pulses.lock().unwrap().len(), 2, "no pulse after the stun");
        assert!(!any_active(&world));
        let after = world.read_storage::<Hero>().get(hero).unwrap().get_cooldown(&id);
        assert!(after > Fixed64::ZERO && after < cooldown, "cooldown keeps running, never restarted or refunded");
    }

    #[test]
    fn pulse_outcomes_cannot_charge_again() {
        use super::super::comp::Outcome;
        let (_, hero) = fixture();
        assert!(pulse_outcome_allowed(&Outcome::ScriptHeal { target: hero, amount: Fixed64::ONE }));
        assert!(!pulse_outcome_allowed(&Outcome::ScriptStartCooldown { entity: hero, ability_id: "quake".into(), duration: Fixed64::ONE }));
        assert!(!pulse_outcome_allowed(&Outcome::ScriptManaSpent { entity: hero, amount: Fixed64::ONE, ability_id: "quake".into() }));
    }
}
