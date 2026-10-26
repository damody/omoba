//! Generic ability projectiles (G-1): linear or homing flight, authored by Lua
//! effects or Rust scripts through the closed `add_stat_buff` envelope.
//!
//! Lifecycle:
//! 1. Launch: a cast emits `AddBuff { target: caster, buff_id: BUFF, payload: {KEY: ..} }`.
//!    Cast admission and serial settlement both run [`admitted`]; [`launch`]
//!    creates a position-only entity plus private state in [`AbilityProjectiles`].
//! 2. Flight: `tick::ability_projectile_tick` advances every projectile one fixed
//!    step (BTreeMap order) and emits `Outcome::AbilityProjectileStep`.
//! 3. Hits: [`settle_step`] runs at serial settlement against the current world,
//!    after earlier damage of the same tick. Candidates are ordered by first
//!    contact time, then canonical entity identity; each target is hit once.
//!
//! Allegiance and kill credit are captured at launch, so a projectile whose
//! caster dies keeps its original team unless `caster_death = expire`.
use omoba_content_model::ControlEffectKind;
use omoba_sim::{Fixed64, Vec2};
use omb_script_abi::types::{DamageKind, DamageProfile};
use serde_json::{json, Value};
use specs::{Builder, Entity, Join, World, WorldExt};
use std::collections::{BTreeMap, BTreeSet};

use super::comp::*;

pub const KEY: &str = "__ability_projectile_v1";
pub const BUFF: &str = "ability_projectile";
/// Live projectile caps. Launches beyond them are rejected at admission.
pub const MAX_LIVE: usize = 512;
pub const MAX_LIVE_PER_SOURCE: usize = 32;
pub const MAX_HITS: u16 = 64;
/// Disclosed-render kind for `DemoRenderState::kind` (0 other, 1 hero, 2 unit,
/// 3 neutral, 4 summon).
pub const RENDER_KIND: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// Flies the full `distance` toward the aim point, even if the point is nearer.
    Linear { aim: Vec2 },
    /// Tracks one exact entity life; only that entity can be hit.
    Homing { target_id: u32, target_generation: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallPolicy { Pass, Stop }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetLostPolicy {
    Expire,
    LastPosition,
    /// Homing only (`retarget_on_death`): when the locked target dies or is
    /// gone, lock the nearest live hostile non-building unit within this radius
    /// of the projectile (canonical id breaks ties); none left: expire.
    Retarget(Fixed64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CasterDeathPolicy { Continue, Expire }

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    /// Authored ability id; also the public presentation identity.
    pub ability: String,
    pub motion: Motion,
    pub speed: Fixed64,
    pub radius: Fixed64,
    /// Linear travel distance; zero for homing.
    pub distance: Fixed64,
    pub lifetime: Fixed64,
    /// 1 stops on the first hit; more pierces. Homing is always 1.
    pub max_hits: u16,
    pub wall: WallPolicy,
    pub target_lost: TargetLostPolicy,
    pub caster_death: CasterDeathPolicy,
    pub amount: Fixed64,
    pub kind: DamageKind,
    pub profile: u32,
    pub slow: Option<(Fixed64, Fixed64)>,
    pub control: Option<(ControlEffectKind, Fixed64)>,
    /// Damage over time each hit applies: (damage per second, duration), same
    /// kind as `amount`, through the generic periodic-damage buff.
    pub dot: Option<(Fixed64, Fixed64)>,
    /// Homing only: the locked target may be an enemy building.
    pub buildings: bool,
    /// Each hit displaces the target from (or toward) the owner:
    /// (mode, distance, duration), through the generic forced-motion buff.
    pub forced: Option<(super::forced_motion::Mode, Fixed64, Fixed64)>,
}

fn kind_name(kind: DamageKind) -> &'static str {
    match kind { DamageKind::Physical => "physical", DamageKind::Magical => "magical", DamageKind::Pure => "pure" }
}

fn supported(p: Vec2) -> bool {
    let max = omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
    p.x.raw().unsigned_abs() <= max && p.y.raw().unsigned_abs() <= max
}

impl Spec {
    pub fn valid(&self) -> bool {
        let ranged = |v: Fixed64, min: Fixed64, max: Fixed64| v >= min && v <= max;
        let motion = match self.motion {
            Motion::Linear { aim } => supported(aim)
                && ranged(self.distance, Fixed64::from_raw(1), Fixed64::from_i32(10_000))
                && (1..=MAX_HITS).contains(&self.max_hits) && !self.buildings,
            Motion::Homing { target_id, target_generation } => target_id != u32::MAX
                && target_generation > 0 && target_generation <= i32::MAX as u32
                && self.distance == Fixed64::ZERO && self.max_hits == 1 && self.wall == WallPolicy::Pass,
        };
        let retarget = match (self.target_lost, self.motion) {
            (TargetLostPolicy::Retarget(radius), Motion::Homing { .. }) => ranged(radius, Fixed64::ONE, Fixed64::from_i32(2_000)),
            (TargetLostPolicy::Retarget(_), Motion::Linear { .. }) => false,
            _ => true,
        };
        motion && retarget && self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && crate::runtime::encode_disclosed_presentation_identity(&self.ability).is_some()
            && ranged(self.speed, Fixed64::ONE, Fixed64::from_i32(20_000))
            && ranged(self.radius, Fixed64::ZERO, Fixed64::from_i32(1_000))
            && ranged(self.lifetime, Fixed64::from_raw(1), Fixed64::from_i32(30))
            && ranged(self.amount, Fixed64::ZERO, Fixed64::from_i32(1_000_000))
            && self.profile != 0 && DamageProfile::from_bits(self.profile).is_some()
            && self.slow.is_none_or(|(reduction, duration)| ranged(reduction, Fixed64::from_raw(1), Fixed64::ONE)
                && ranged(duration, Fixed64::from_raw(1), Fixed64::from_i32(60)))
            && self.control.is_none_or(|(_, duration)| ranged(duration, Fixed64::from_raw(1), Fixed64::from_i32(60)))
            && self.dot.is_none_or(|(rate, duration)| ranged(rate, Fixed64::from_raw(1), Fixed64::from_i32(1_000_000))
                && ranged(duration, Fixed64::from_raw(1), Fixed64::from_i32(60)))
            && self.forced.is_none_or(|(_, distance, duration)| ranged(distance, Fixed64::from_raw(1), Fixed64::from_i32(super::forced_motion::MAX_DISTANCE))
                && super::forced_motion::valid_duration(duration))
    }

    /// Strict JSON envelope. Every numeric field is a raw Fixed64 integer.
    pub fn payload(&self) -> Value {
        let mut row = json!({
            "source_id": self.source_id, "source_generation": self.source_generation,
            "ability": self.ability, "speed": self.speed.raw(), "radius": self.radius.raw(),
            "distance": self.distance.raw(), "lifetime": self.lifetime.raw(), "max_hits": self.max_hits,
            "wall": match self.wall { WallPolicy::Pass => "pass", WallPolicy::Stop => "stop" },
            "target_lost": match self.target_lost { TargetLostPolicy::Expire => "expire", TargetLostPolicy::LastPosition => "last_position",
                TargetLostPolicy::Retarget(_) => "retarget" },
            "caster_death": match self.caster_death { CasterDeathPolicy::Continue => "continue", CasterDeathPolicy::Expire => "expire" },
            "amount": self.amount.raw(), "kind": kind_name(self.kind), "profile": self.profile,
        });
        match self.motion {
            Motion::Linear { aim } => { row["linear"] = json!({"x": aim.x.raw(), "y": aim.y.raw()}); }
            Motion::Homing { target_id, target_generation } => {
                row["homing"] = json!({"target_id": target_id, "target_generation": target_generation});
            }
        }
        if let Some((reduction, duration)) = self.slow {
            row["slow"] = json!({"reduction": reduction.raw(), "duration": duration.raw()});
        }
        if let Some((control, duration)) = self.control {
            row["control"] = json!({"kind": control.as_str(), "duration": duration.raw()});
        }
        if let Some((rate, duration)) = self.dot {
            row["dot"] = json!({"rate": rate.raw(), "duration": duration.raw()});
        }
        // Canonical: present only when true.
        if let Some((mode, distance, duration)) = self.forced {
            row["forced"] = json!({"mode": mode.as_str(), "distance": distance.raw(), "duration": duration.raw()});
        }
        if self.buildings { row["buildings"] = json!(true); }
        if let TargetLostPolicy::Retarget(radius) = self.target_lost { row["retarget_radius"] = json!(radius.raw()); }
        json!({ (KEY): row })
    }

    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let optional = ["slow", "control", "dot", "buildings", "forced", "retarget_radius"].iter().filter(|key| row.contains_key(**key)).count();
        if row.len() != 15 + optional { return None; }
        let raw = |name: &str| row.get(name)?.as_i64().map(Fixed64::from_raw);
        let id = |value: &Value, name: &str| value.get(name)?.as_u64().and_then(|v| u32::try_from(v).ok());
        let text = |name: &str| row.get(name)?.as_str();
        let motion = match (row.get("linear"), row.get("homing")) {
            (Some(linear), None) if linear.as_object()?.len() == 2 => Motion::Linear {
                aim: Vec2::new(Fixed64::from_raw(linear.get("x")?.as_i64()?), Fixed64::from_raw(linear.get("y")?.as_i64()?)) },
            (None, Some(homing)) if homing.as_object()?.len() == 2 => Motion::Homing {
                target_id: id(homing, "target_id")?, target_generation: id(homing, "target_generation")? },
            _ => return None,
        };
        let slow = match row.get("slow") {
            None => None,
            Some(slow) if slow.as_object()?.len() == 2 => Some((Fixed64::from_raw(slow.get("reduction")?.as_i64()?),
                Fixed64::from_raw(slow.get("duration")?.as_i64()?))),
            _ => return None,
        };
        let control = match row.get("control") {
            None => None,
            Some(control) if control.as_object()?.len() == 2 => Some((match control.get("kind")?.as_str()? {
                "stun" => ControlEffectKind::Stun, "root" => ControlEffectKind::Root, "silence" => ControlEffectKind::Silence,
                _ => return None }, Fixed64::from_raw(control.get("duration")?.as_i64()?))),
            _ => return None,
        };
        let dot = match row.get("dot") {
            None => None,
            Some(dot) if dot.as_object()?.len() == 2 => Some((Fixed64::from_raw(dot.get("rate")?.as_i64()?),
                Fixed64::from_raw(dot.get("duration")?.as_i64()?))),
            _ => return None,
        };
        let buildings = match row.get("buildings") { None => false, Some(Value::Bool(true)) => true, _ => return None };
        let forced = match row.get("forced") {
            None => None,
            Some(forced) if forced.as_object()?.len() == 3 => Some((super::forced_motion::Mode::parse(forced.get("mode")?.as_str()?)?,
                Fixed64::from_raw(forced.get("distance")?.as_i64()?), Fixed64::from_raw(forced.get("duration")?.as_i64()?))),
            _ => return None,
        };
        let spec = Self {
            source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            ability: text("ability")?.to_owned(), motion,
            speed: raw("speed")?, radius: raw("radius")?, distance: raw("distance")?, lifetime: raw("lifetime")?,
            max_hits: u16::try_from(row.get("max_hits")?.as_u64()?).ok()?,
            wall: match text("wall")? { "pass" => WallPolicy::Pass, "stop" => WallPolicy::Stop, _ => return None },
            target_lost: match (text("target_lost")?, row.get("retarget_radius")) {
                ("expire", None) => TargetLostPolicy::Expire,
                ("last_position", None) => TargetLostPolicy::LastPosition,
                ("retarget", Some(radius)) => TargetLostPolicy::Retarget(Fixed64::from_raw(radius.as_i64()?)),
                _ => return None },
            caster_death: match text("caster_death")? { "continue" => CasterDeathPolicy::Continue,
                "expire" => CasterDeathPolicy::Expire, _ => return None },
            amount: raw("amount")?,
            kind: match text("kind")? { "physical" => DamageKind::Physical, "magical" => DamageKind::Magical,
                "pure" => DamageKind::Pure, _ => return None },
            profile: u32::try_from(row.get("profile")?.as_u64()?).ok()?,
            slow, control, dot, buildings, forced,
        };
        // The buff duration is the authored lifetime; a mismatch is ambiguous.
        (spec.valid() && duration == spec.lifetime).then_some(spec)
    }
}

/// Authority-private in-flight state. Never disclosed; see [`disclosed_state`].
#[derive(Clone, Debug)]
pub struct State {
    pub spec: Spec,
    pub owner: Entity,
    pub faction: Faction,
    pub team: u32,
    pub origin: ProjectileOrigin,
    /// Linear endpoint, or the last known homing target position.
    pub end: Vec2,
    pub target: Option<Entity>,
    pub target_lost: bool,
    pub age: Fixed64,
    pub hits: BTreeSet<(u32, u32)>,
    /// Last committed step delta; public motion direction for presentation.
    pub heading: Vec2,
}

#[derive(Clone, Debug, Default)]
pub struct AbilityProjectiles(pub BTreeMap<Entity, State>);

impl AbilityProjectiles {
    /// Canonical authority state for the lockstep desync hash.
    pub fn hash_value(&self) -> Value {
        Value::Array(self.0.iter().map(|(entity, state)| json!({
            "entity": crate::runtime::canonical_entity_id(*entity), "spec": state.spec.payload(),
            "owner": crate::runtime::canonical_entity_id(state.owner), "faction": state.faction, "team": state.team,
            "origin": state.origin, "end": [state.end.x.raw(), state.end.y.raw()],
            "target": state.target.map(crate::runtime::canonical_entity_id), "target_lost": state.target_lost,
            "age": state.age.raw(), "hits": state.hits, "heading": [state.heading.x.raw(), state.heading.y.raw()],
        })).collect())
    }
    fn per_source(&self, owner: Entity) -> usize { self.0.values().filter(|state| state.owner == owner).count() }
}

fn attackable(world: &World, target: Entity) -> bool {
    world.try_fetch::<crate::runtime::MobaMatch>().is_none_or(|state| state.current_match_target_attackable(target))
}

/// A live, hostile, non-building unit that ability projectiles may hit.
fn hittable(world: &World, faction: &Faction, target: Entity, removed: &[Entity], buildings: bool) -> bool {
    world.entities().is_alive(target) && !removed.contains(&target)
        && world.read_storage::<CProperty>().get(target).is_some_and(|p| p.hp > Fixed64::ZERO)
        && (buildings || world.read_storage::<IsBuilding>().get(target).is_none())
        && world.read_storage::<Pos>().get(target).is_some()
        && world.read_storage::<Faction>().get(target).is_some_and(|other| faction.is_hostile_to(other))
        && attackable(world, target)
}

/// Launch admission shared by cast settlement and the final serial launch.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value,
    expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    let entities = world.entities();
    let source = entities.entity(spec.source_id);
    if source.gen().id() as u32 != spec.source_generation || source != target || !entities.is_alive(source)
        || expected_source.is_some_and(|expected| expected != source) { return None; }
    if world.read_storage::<CProperty>().get(source).is_none_or(|p| p.hp <= Fixed64::ZERO) { return None; }
    let faction = world.read_storage::<Faction>().get(source).cloned()?;
    let origin = world.read_storage::<Pos>().get(source).map(|p| p.0).filter(|p| supported(*p))?;
    if let Some(live) = world.try_fetch::<AbilityProjectiles>() {
        if live.0.len() >= MAX_LIVE || live.per_source(source) >= MAX_LIVE_PER_SOURCE { return None; }
    }
    match spec.motion {
        Motion::Linear { aim } => (aim != origin).then_some(spec),
        Motion::Homing { target_id, target_generation } => {
            let victim = entities.entity(target_id);
            (victim.gen().id() as u32 == target_generation && victim != source
                && hittable(world, &faction, victim, &[], spec.buildings)).then_some(spec)
        }
    }
}

/// Point at exactly `distance` from `origin` toward `aim`, never clamped to `aim`.
fn extended_endpoint(origin: Vec2, aim: Vec2, distance: Fixed64) -> Option<Vec2> {
    if !supported(origin) || !supported(aim) || origin == aim || distance <= Fixed64::ZERO { return None; }
    let (dx, dy) = (i128::from((aim - origin).x.raw()), i128::from((aim - origin).y.raw()));
    let squared = (dx * dx + dy * dy) as u128;
    let floor = squared.isqrt();
    let length = (if floor * floor == squared { floor } else { floor + 1 }) as i128;
    let component = |raw: i128| Fixed64::from_raw((raw * i128::from(distance.raw()) / length) as i64);
    let end = origin + Vec2::new(component(dx), component(dy));
    supported(end).then_some(end)
}

/// Serial launch. Returns the new projectile entity.
/// Fog: a visible tracking flight toward an enemy the caster's team cannot see
/// would reveal its position. Such a launch is dropped silently at serial
/// settlement; the rest of an admitted multi-target cast still applies.
fn homing_target_disclosed(world: &World, faction: &Faction, motion: Motion) -> bool {
    let Motion::Homing { target_id, target_generation } = motion else { return true; };
    let visibility = world.try_fetch::<crate::runtime::TeamVisibilityRuntime>();
    super::combat_acquisition::DisclosureContext::resolve(
        world.try_fetch::<super::cast_windup::FormalCastRules>().is_some(),
        world.try_fetch::<crate::runtime::MobaMatch>().is_some(),
        visibility.as_deref())
        .allows(Some(faction), crate::runtime::selective::CanonicalEntityKey { id: target_id, generation: target_generation })
}

pub fn launch(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Entity> {
    let spec = admitted(world, target, buff_id, duration, payload, None)?;
    let source = target;
    if !homing_target_disclosed(world, world.read_storage::<Faction>().get(source)?, spec.motion) { return None; }
    let from = world.read_storage::<Pos>().get(source)?.0;
    let faction = world.read_storage::<Faction>().get(source).cloned()?;
    let mut origin = super::moba_match::capture_projectile_origin(world, source)?;
    origin.script = None;
    let (end, victim) = match spec.motion {
        Motion::Linear { aim } => (extended_endpoint(from, aim, spec.distance)?, None),
        Motion::Homing { target_id, .. } => {
            let victim = world.entities().entity(target_id);
            (world.read_storage::<Pos>().get(victim)?.0, Some(victim))
        }
    };
    let team = faction.team_id.max(0) as u32;
    let entity = world.create_entity()
        .with(Pos(from))
        .with(ReplicationScope { kind: ReplicationScopeKind::OwnerTeam, owner_team: Some(team) })
        .build();
    if world.try_fetch::<AbilityProjectiles>().is_none() { world.insert(AbilityProjectiles::default()); }
    world.write_resource::<AbilityProjectiles>().0.insert(entity, State {
        spec, owner: source, faction, team, origin, end, target: victim, target_lost: false,
        age: Fixed64::ZERO, hits: BTreeSet::new(), heading: Vec2::ZERO,
    });
    Some(entity)
}

/// Contact fraction (Q32) to a point on the segment.
fn point_at(from: Vec2, to: Vec2, q32: u64) -> Vec2 {
    let lerp = |a: Fixed64, b: Fixed64| Fixed64::from_raw(a.raw() + ((i128::from(b.raw() - a.raw()) * i128::from(q32)) >> 32) as i64);
    Vec2::new(lerp(from.x, to.x), lerp(from.y, to.y))
}

fn identity(entity: Entity) -> (u32, u32) { (entity.id(), entity.gen().id() as u32) }

fn emit_fact(world: &World, projectile: Entity, ordinal: u32, kind: crate::runtime::FactKind, fact: crate::runtime::ObservableFact) {
    let Some(tick) = world.try_fetch::<Tick>().map(|tick| tick.0) else { return; };
    let Some(facts) = world.try_fetch::<crate::runtime::ObservableFactBuffer>() else { return; };
    let _ = facts.emit(crate::runtime::OrderedFact {
        key: crate::runtime::FactOrderingKey {
            tick, phase: crate::runtime::FactPhase::PostStep,
            canonical_source_order: crate::runtime::canonical_entity_id(projectile),
            local_ordinal: ordinal, fact_kind: kind,
        },
        audience: crate::runtime::FactAudience::VisibilityPolicy(
            omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
        fact,
    });
}

/// Public launch fact, emitted after a successful [`launch`].
pub fn emit_launch(world: &World, projectile: Entity) {
    let Some(row) = world.try_fetch::<AbilityProjectiles>().and_then(|rows| rows.0.get(&projectile).map(|s| (s.owner, s.target))) else { return; };
    emit_fact(world, projectile, 0, crate::runtime::FactKind::Projectile, crate::runtime::ObservableFact::Projectile {
        source: crate::runtime::canonical_entity_id(row.0), target: row.1.map(crate::runtime::canonical_entity_id),
        effect_id: 0, active: true });
}

/// Serial hit resolution for one fixed step. Returns child outcomes in order:
/// per hit (damage, slow, control), then the projectile's own Death.
pub fn settle_step(world: &mut World, projectile: Entity, from: Vec2, to: Vec2, arrival: bool, terminal: bool,
    removed: &[Entity]) -> Vec<Outcome> {
    let Some(state) = world.try_fetch::<AbilityProjectiles>().and_then(|rows| rows.0.get(&projectile).cloned()) else {
        return Vec::new();
    };
    let remaining = usize::from(state.spec.max_hits).saturating_sub(state.hits.len());
    let mut hits: Vec<(u64, Entity)> = Vec::new();
    match (state.spec.motion, state.target) {
        (Motion::Homing { .. }, Some(target)) if arrival && !state.target_lost => {
            if hittable(world, &state.faction, target, removed, state.spec.buildings) && !state.hits.contains(&identity(target)) {
                hits.push((1 << 32, target));
            }
        }
        (Motion::Linear { .. }, _) if remaining > 0 => {
            let entities = world.entities();
            let positions = world.read_storage::<Pos>();
            let radii = world.read_storage::<CollisionRadius>();
            let mut rows: Vec<_> = (&entities, &positions, &world.read_storage::<Faction>()).join()
                .filter(|(target, _, other)| *target != state.owner && state.faction.is_hostile_to(other)
                    && !state.hits.contains(&identity(*target)))
                .filter_map(|(target, pos, _)| {
                    let reach = state.spec.radius + radii.get(target).map_or(Fixed64::ZERO, |r| r.0.max(Fixed64::ZERO));
                    omoba_sim::terrain::swept_circle_contact_q32(pos.0, from, to, reach).map(|contact| (contact, target))
                })
                .collect();
            drop((entities, positions, radii));
            rows.retain(|(_, target)| hittable(world, &state.faction, *target, removed, false));
            rows.sort_by_key(|(contact, target)| (*contact, identity(*target)));
            rows.truncate(remaining);
            hits = rows;
        }
        _ => {}
    }
    let zero = Fixed64::ZERO;
    let (phys, magi, real) = match state.spec.kind {
        DamageKind::Physical => (state.spec.amount, zero, zero),
        DamageKind::Magical => (zero, state.spec.amount, zero),
        DamageKind::Pure => (zero, zero, state.spec.amount),
    };
    let caster = omb_script_abi::types::EntityHandle { id: state.owner.id(), gen: state.owner.gen().id() as u32 };
    let mut outcomes = Vec::new();
    let mut last_contact = None;
    for (ordinal, (contact, target)) in hits.iter().enumerate() {
        let pos = world.read_storage::<Pos>().get(*target).map(|p| p.0).unwrap_or(to);
        outcomes.push(Outcome::RawDamage { pos, phys, magi, real, source: state.owner, target: *target,
            damage_profile: state.spec.profile, origin: Some(state.origin.clone()) });
        if let Some((reduction, duration)) = state.spec.slow {
            outcomes.push(Outcome::AddBuff { target: *target,
                buff_id: omb_script_abi::buff_ids::generic_slow_buff_id(&state.spec.ability, caster), duration,
                payload: json!({"move_speed_bonus": -reduction.raw(), "__aggregation_family": "generic_ability_slow"}) });
        }
        if let Some((control, duration)) = state.spec.control {
            outcomes.push(Outcome::AddBuff { target: *target, buff_id: control.as_str().to_owned(), duration,
                payload: Value::Null });
        }
        if let Some((rate, duration)) = state.spec.dot {
            // Same identity and payload as the declarative DoT, so stacking and
            // refresh rules are the existing periodic-damage ones.
            outcomes.push(Outcome::AddBuff { target: *target,
                buff_id: omb_script_abi::buff_ids::generic_dot_buff_id(&state.spec.ability, caster), duration,
                payload: crate::runtime::ability_runtime::periodic_damage::PeriodicDamageSpec::payload(rate, caster, state.spec.kind) });
        }
        if let Some((mode, distance, duration)) = state.spec.forced {
            // The hit already proved contact; range only bounds the owner link.
            let spec = super::forced_motion::Spec { source_id: state.owner.id(), source_generation: state.owner.gen().id() as u32,
                mode, distance, cast_range: Fixed64::from_i32(10_000) };
            outcomes.push(Outcome::AddBuff { target: *target, buff_id: super::forced_motion::BUFF.to_owned(), duration,
                payload: spec.payload() });
        }
        emit_fact(world, projectile, ordinal as u32 + 1, crate::runtime::FactKind::ProjectileImpact,
            crate::runtime::ObservableFact::ProjectileImpact { target: crate::runtime::canonical_entity_id(*target) });
        last_contact = Some(*contact);
    }
    let spent = state.hits.len() + hits.len() >= usize::from(state.spec.max_hits);
    let homing_resolved = matches!(state.spec.motion, Motion::Homing { .. }) && arrival;
    let end_pos = match (spent, last_contact) {
        (true, Some(contact)) if matches!(state.spec.motion, Motion::Linear { .. }) => point_at(from, to, contact),
        _ => to,
    };
    let done = terminal || spent || homing_resolved;
    {
        let mut rows = world.write_resource::<AbilityProjectiles>();
        if done {
            rows.0.remove(&projectile);
        } else if let Some(row) = rows.0.get_mut(&projectile) {
            row.hits.extend(hits.iter().map(|(_, target)| identity(*target)));
        }
    }
    if done {
        if let Some(pos) = world.write_storage::<Pos>().get_mut(projectile) { pos.0 = end_pos; }
        emit_fact(world, projectile, hits.len() as u32 + 1, crate::runtime::FactKind::Projectile,
            crate::runtime::ObservableFact::Projectile { source: crate::runtime::canonical_entity_id(projectile),
                target: None, effect_id: 0, active: false });
        outcomes.push(Outcome::Death { pos: end_pos, ent: projectile });
    }
    outcomes
}

/// Schema for the public flight component. Version 2, 18 bytes, big endian:
/// version u8 (=2), motion u8 (1 linear, 2 homing), speed raw i64, radius raw i64.
/// Immutable for the projectile's life: team replicas cannot predict
/// authority-private flight, so per-step motion reaches them only as public
/// `Movement` facts (position). Direction is derived from those positions.
/// Never carries the homing target. v1 (34 bytes, mutable heading) is rejected.
pub const DISCLOSED_SCHEMA_ID: u32 = 0x464f4718;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosedFlight { pub motion: u8, pub speed_raw: i64, pub radius_raw: i64 }

pub fn disclosed_state(state: &State) -> DisclosedFlight {
    DisclosedFlight {
        motion: match state.spec.motion { Motion::Linear { .. } => 1, Motion::Homing { .. } => 2 },
        speed_raw: state.spec.speed.raw(), radius_raw: state.spec.radius.raw(),
    }
}

pub fn encode_disclosed_flight(flight: DisclosedFlight) -> Vec<u8> {
    let mut bytes = vec![2, flight.motion];
    for value in [flight.speed_raw, flight.radius_raw] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}

pub fn decode_disclosed_flight(bytes: &[u8]) -> Option<DisclosedFlight> {
    if bytes.len() != 18 || bytes[0] != 2 || !(1..=2).contains(&bytes[1]) { return None; }
    let read = |index: usize| i64::from_be_bytes(bytes[2 + index * 8..10 + index * 8].try_into().expect("bounded"));
    let flight = DisclosedFlight { motion: bytes[1], speed_raw: read(0), radius_raw: read(1) };
    (flight.speed_raw > 0 && flight.radius_raw >= 0).then_some(flight)
}

#[cfg(test)]
#[path = "ability_projectile_tests.rs"]
mod tests;
