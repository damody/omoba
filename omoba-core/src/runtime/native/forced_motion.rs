//! Forced displacement (G-4 `knockback` / `pull_hook`). A script moves an
//! enemy by a timed buff on the victim whose payload is `{KEY: ..}`:
//! - `knockback` pushes it `distance` straight away from the source;
//! - `pull` drags it straight toward the source, stopping at body contact;
//! - `dash` moves the source itself to a fixed endpoint (a continuous dash, H02):
//!   the packet is on its own caster, carries no stun (the motion buff alone roots
//!   it against commands) and turns it to face the travel direction.
//!
//! The direction and travel are fixed when the serial settlement applies the
//! buff, from the positions at that moment; the victim then moves at a constant
//! speed (travel ÷ duration) over the buff's duration. Terrain stops it at the
//! wall (other units never block it). The victim is stunned for the duration and
//! cannot move by command even if that stun is cleansed (`BuffStore::is_rooted`).
//! The motion ends early on death, on a wall, or when the buff is removed.
//!
//! v3 `knockback` options (`Options`, extra fields of the same packet row):
//! - `origin`: push away from this point instead of from the source (v3 `from` = a point such
//!   as `offset(caster, caster.forward, -1)`); the source then may be the victim itself (a
//!   self push: 茶茶、伊達政宗), otherwise the victim is a hostile unit in range as before;
//! - `no_stun`: no stun with the motion (v3 `stun = false`); the motion buff still owns the
//!   position (no move commands) but the victim can attack and cast;
//! - `height_raw`: the arc's peak height (v3 `height`, 直江兼續、柳生宗嚴). The simulation is flat:
//!   it only travels with the status buff for the presentation to draw the arc.
//! - `contact_raw` (v3 `stop_on_contact` + `contact_radius`, 前田利家, E494): the motion also ends
//!   when another living unit that is not a building, of any team, is ahead of the victim (in its
//!   direction of travel) within this distance of their bodies' edges;
//! - `on_end` (v3 `knockback.on_end`, E494): a scheduled_effect template due once on the source
//!   when the motion ends for any reason (its travel done, a wall, a contact, the victim's death,
//!   the buff's removal; a newer motion replaces it without its end), the victim's last position
//!   appended to its captures. Only the authority steps motions, so only it schedules the end.
//!
//! Only the authority integrates motion (`step`, from the MobaMatch pre-step);
//! every world sees the result as ordinary `PreStepMovement` facts, so team
//! replicas never predict a hidden unit's path.
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::comp::{BlockedRegions, CProperty, CollisionRadius, DeltaTime, IsBuilding, Pos, Tick};
use super::ability_runtime::BuffStore;

pub const KEY: &str = "__forced_motion_v1";
/// The visible status buff; its payload is the spec.
pub const BUFF: &str = "forced_motion";
pub const MAX_DISTANCE: i32 = 3_000;
pub const MAX_DURATION: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Knockback, Pull, Dash }

impl Mode {
    pub const fn as_str(self) -> &'static str { match self { Self::Knockback => "knockback", Self::Pull => "pull", Self::Dash => "dash" } }
    pub fn parse(text: &str) -> Option<Self> {
        match text { "knockback" => Some(Self::Knockback), "pull" => Some(Self::Pull), "dash" => Some(Self::Dash), _ => None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub mode: Mode,
    /// Knockback travel, or the pull's longest travel.
    pub distance: Fixed64,
    /// Admission range from the live source to the victim (a dash: to its endpoint).
    pub cast_range: Fixed64,
    /// A dash's endpoint (`Some` exactly for `Mode::Dash`).
    pub to: Option<Vec2>,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && self.distance > Fixed64::ZERO && self.distance <= Fixed64::from_i32(MAX_DISTANCE)
            && self.cast_range > Fixed64::ZERO && self.cast_range <= Fixed64::from_i32(10_000)
            && (self.mode == Mode::Dash) == self.to.is_some()
            && self.to.is_none_or(|to| to.x.raw().abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW
                && to.y.raw().abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        let mut row = json!({ "source_id": self.source_id, "source_generation": self.source_generation,
            "mode": self.mode.as_str(), "distance_raw": self.distance.raw(), "range_raw": self.cast_range.raw() });
        if let Some(to) = self.to { row["to"] = json!([to.x.raw(), to.y.raw()]); }
        json!({ (KEY): row })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || !valid_duration(duration) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let to = match row.get("to") {
            Some(point) => {
                let xy = point.as_array()?;
                if xy.len() != 2 { return None; }
                Some(Vec2::new(Fixed64::from_raw(xy[0].as_i64()?), Fixed64::from_raw(xy[1].as_i64()?)))
            }
            None => None,
        };
        let options = Options::from_row(row)?;
        let ends = row.contains_key("on_end");
        if ends && on_end_of(payload).is_none() { return None; }
        if (options.origin.is_some() || !options.stun || options.height.is_some() || options.contact.is_some() || ends)
            && Mode::parse(row.get("mode")?.as_str()?)? != Mode::Knockback { return None; }
        if row.len() != 5 + usize::from(to.is_some()) + options.fields() + usize::from(ends) { return None; }
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            mode: Mode::parse(row.get("mode")?.as_str()?)?,
            distance: Fixed64::from_raw(row.get("distance_raw")?.as_i64()?),
            cast_range: Fixed64::from_raw(row.get("range_raw")?.as_i64()?), to }.checked()
    }
    /// The packet with knockback options (see the module docs); default options give `payload()`.
    pub fn payload_with(&self, options: Options) -> Value {
        let mut payload = self.payload();
        if let Some(origin) = options.origin { payload[KEY]["origin"] = json!([origin.x.raw(), origin.y.raw()]); }
        if !options.stun { payload[KEY]["no_stun"] = json!(true); }
        if let Some(height) = options.height { payload[KEY]["height_raw"] = json!(height.raw()); }
        if let Some(contact) = options.contact { payload[KEY]["contact_raw"] = json!(contact.raw()); }
        payload
    }
    pub fn source(&self) -> Entity { self.source_entity() }
    fn source_entity(&self) -> Entity { Entity::new(self.source_id, specs::world::Generation::new(self.source_generation as i32)) }
}

/// Knockback options carried in the packet row (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub origin: Option<Vec2>,
    pub stun: bool,
    /// Peak height of the arc (presentation only), above zero.
    pub height: Option<Fixed64>,
    /// Stops on a unit ahead within this edge distance (zero or more).
    pub contact: Option<Fixed64>,
}

pub const MAX_HEIGHT: i32 = 2_000;
pub const MAX_CONTACT: i32 = 1_000;

impl Default for Options {
    fn default() -> Self { Self { origin: None, stun: true, height: None, contact: None } }
}

impl Options {
    fn from_row(row: &serde_json::Map<String, Value>) -> Option<Self> {
        let origin = match row.get("origin") {
            Some(point) => {
                let xy = point.as_array()?;
                if xy.len() != 2 { return None; }
                let (x, y) = (xy[0].as_i64()?, xy[1].as_i64()?);
                let bound = omoba_sim::terrain::MAX_GEOMETRY_RAW;
                if x.abs() > bound || y.abs() > bound { return None; }
                Some(Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y)))
            }
            None => None,
        };
        let stun = match row.get("no_stun") { Some(Value::Bool(true)) => false, None => true, _ => return None };
        let height = match row.get("height_raw") {
            Some(raw) => {
                let height = Fixed64::from_raw(raw.as_i64()?);
                if height <= Fixed64::ZERO || height > Fixed64::from_i32(MAX_HEIGHT) { return None; }
                Some(height)
            }
            None => None,
        };
        let contact = match row.get("contact_raw") {
            Some(raw) => {
                let contact = Fixed64::from_raw(raw.as_i64()?);
                if contact < Fixed64::ZERO || contact > Fixed64::from_i32(MAX_CONTACT) { return None; }
                Some(contact)
            }
            None => None,
        };
        Some(Self { origin, stun, height, contact })
    }
    fn fields(self) -> usize {
        usize::from(self.origin.is_some()) + usize::from(!self.stun) + usize::from(self.height.is_some()) + usize::from(self.contact.is_some())
    }
    pub fn of(payload: &Value) -> Self {
        payload.get(KEY).and_then(Value::as_object).and_then(Self::from_row).unwrap_or_default()
    }
}

/// A knockback packet whose end schedules `template` once on the source (module docs, `on_end`);
/// at most `MAX_ARGS - 1` authored captures.
pub fn with_on_end(mut payload: Value, template: &super::scheduled_effect::Spec) -> Value {
    payload[KEY]["on_end"] = template.payload()[super::scheduled_effect::KEY].clone();
    payload
}

/// The on-end schedule template of a packet, if valid.
pub fn on_end_of(payload: &Value) -> Option<super::scheduled_effect::Spec> {
    use super::scheduled_effect as se;
    let row = payload.get(KEY)?.as_object()?.get("on_end")?.clone();
    let probe = row.as_object()?;
    let id = format!("{}{}:{}:{}", se::PREFIX, probe.get("ability")?.as_str()?, probe.get("instance")?.as_u64()?, probe.get("step")?.as_u64()?);
    let spec = se::Spec::from_buff(&id, &json!({ (se::KEY): row }))?;
    (spec.args.len() < se::MAX_ARGS).then_some(spec)
}

pub fn valid_duration(duration: Fixed64) -> bool {
    duration >= Fixed64::from_raw(1) && duration <= Fixed64::from_i32(MAX_DURATION)
}

/// One victim's motion, fixed at settlement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Motion {
    pub source: Entity,
    pub mode: Mode,
    /// Unit direction (Fixed64 components, length ≈ 1).
    pub direction: Vec2,
    pub remaining: Fixed64,
    pub time_left: Fixed64,
    /// A dash's exact endpoint: its last step lands here, free of direction rounding.
    pub to: Option<Vec2>,
    /// Ends on a unit ahead within this edge distance ([`Options::contact`]).
    pub contact: Option<Fixed64>,
    /// Due once on the source when the motion ends ([`with_on_end`]).
    pub on_end: Option<super::scheduled_effect::Spec>,
}

#[derive(Clone, Debug, Default)]
pub struct ForcedMotions(pub BTreeMap<Entity, Motion>);

impl ForcedMotions {
    /// Canonical authority state for the lockstep desync hash.
    pub fn hash_value(&self) -> Value {
        Value::Array(self.0.iter().map(|(entity, motion)| {
            let mut row = json!({
                "entity": crate::runtime::canonical_entity_id(*entity),
                "source": crate::runtime::canonical_entity_id(motion.source), "mode": motion.mode.as_str(),
                "direction": [motion.direction.x.raw(), motion.direction.y.raw()],
                "remaining": motion.remaining.raw(), "time_left": motion.time_left.raw(),
                "to": motion.to.map(|to| [to.x.raw(), to.y.raw()]),
            });
            if let Some(contact) = motion.contact { row["contact"] = json!(contact.raw()); }
            if let Some(on_end) = &motion.on_end { row["on_end"] = on_end.payload(); }
            row
        }).collect())
    }
}

fn movable(world: &World, entity: Entity) -> bool {
    world.entities().is_alive(entity)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
        && world.read_storage::<IsBuilding>().get(entity).is_none()
        && world.read_storage::<Pos>().get(entity).is_some()
}

/// Live hostile target in range of the exact live source, not a building; a dash
/// is on its own live source with its endpoint in range.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value,
    expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    if let Some(to) = spec.to {
        let source = spec.source_entity();
        let at = world.read_storage::<Pos>().get(target).map(|pos| pos.0)?;
        return (target == source && world.entities().entity(spec.source_id) == source
            && expected_source.is_none_or(|expected| expected == source) && movable(world, target)
            && (to - at).length_squared() <= spec.cast_range * spec.cast_range).then_some(spec);
    }
    let source = spec.source_entity();
    if Options::of(payload).origin.is_some() && target == source {
        return (world.entities().entity(spec.source_id) == source && expected_source.is_none_or(|expected| expected == source)
            && movable(world, target)).then_some(spec);
    }
    (super::targeted_effect::admitted(world, target, spec.source_id, spec.source_generation, spec.cast_range, false, expected_source)
        && movable(world, target)).then_some(spec)
}

fn radius(world: &World, entity: Entity) -> Fixed64 {
    world.read_storage::<CollisionRadius>().get(entity).map_or(Fixed64::ZERO, |r| r.0.max(Fixed64::ZERO))
}

/// (unit direction, travel) for `spec` from the current positions, or None
/// when there is nowhere to go (coincident bodies, a pull already in contact).
pub fn plan(world: &World, target: Entity, spec: &Spec) -> Option<(Vec2, Fixed64)> {
    plan_from(world, target, spec, None)
}

/// [`plan`] with a knockback origin point in place of the source's position.
pub fn plan_from(world: &World, target: Entity, spec: &Spec, origin: Option<Vec2>) -> Option<(Vec2, Fixed64)> {
    let positions = world.read_storage::<Pos>();
    let at = positions.get(target)?.0;
    let from = match origin.filter(|_| spec.mode == Mode::Knockback) {
        Some(origin) => origin,
        None => positions.get(spec.source_entity())?.0,
    };
    let delta = match (spec.mode, spec.to) {
        (Mode::Dash, Some(to)) => to - at,
        (Mode::Knockback, _) => at - from,
        _ => from - at,
    };
    let length = delta.length();
    if length <= Fixed64::ZERO { return None; }
    let direction = Vec2::new(delta.x / length, delta.y / length);
    let travel = match spec.mode {
        Mode::Knockback => spec.distance,
        Mode::Pull => spec.distance.min(length - radius(world, target) - radius(world, spec.source_entity())),
        Mode::Dash => spec.distance.min(length),
    };
    (travel > Fixed64::ZERO).then_some((direction, travel))
}

/// Serial settlement of the buff: starts (or replaces) the victim's motion and
/// stores the status buff plus a stun of the same duration. False drops it.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    let Some(spec) = admitted(world, target, buff_id, duration, payload, None) else { return false; };
    let options = Options::of(payload);
    let Some((direction, remaining)) = plan_from(world, target, &spec, options.origin) else { return false; };
    if world.try_fetch::<ForcedMotions>().is_none() { world.insert(ForcedMotions::default()); }
    {
        let buffs = world.read_resource::<BuffStore>();
        // A world that never steps (a team replica) keeps no stale rows.
        world.write_resource::<ForcedMotions>().0.retain(|entity, _| buffs.has(*entity, BUFF));
    }
    world.write_resource::<ForcedMotions>().0.insert(target, Motion {
        source: spec.source_entity(), mode: spec.mode, direction, remaining, time_left: duration,
        to: spec.to.filter(|_| spec.mode == Mode::Dash), contact: options.contact, on_end: on_end_of(payload) });

    if spec.mode == Mode::Dash {
        if let Some(face) = world.write_storage::<super::comp::Facing>().get_mut(target) {
            face.0 = omoba_sim::trig::atan2(direction.y, direction.x);
        }
    }
    let mut buffs = world.write_resource::<BuffStore>();
    buffs.add(target, BUFF, duration, payload.clone());
    // A dash is the caster's own motion: rooted by the motion buff, not stunned.
    // A knockback with `no_stun` is rooted the same way and can still attack and cast.
    if spec.mode != Mode::Dash && options.stun { buffs.add(target, omb_script_abi::buff_ids::BuffId::Stun.as_str(), duration, Value::Null); }
    true
}

/// First blocked point of `from -> to` for a body of `radius`; Q16 bisection.
pub(crate) fn wall_contact(from: Vec2, to: Vec2, radius: Fixed64, regions: &BlockedRegions) -> Option<Vec2> {
    let blocked = |end: Vec2| super::tick::hero_move_tick::path_hits_regions(from, end, radius, regions);
    if !blocked(to) { return None; }
    let at = |t: i64| Vec2::new(
        Fixed64::from_raw(from.x.raw() + (i128::from(to.x.raw() - from.x.raw()) * i128::from(t) >> 16) as i64),
        Fixed64::from_raw(from.y.raw() + (i128::from(to.y.raw() - from.y.raw()) * i128::from(t) >> 16) as i64));
    let (mut low, mut high) = (0_i64, 1_i64 << 16);
    // Largest unblocked prefix.
    while low < high { let mid = low + (high - low + 1) / 2; if blocked(at(mid)) { high = mid - 1; } else { low = mid; } }
    Some(at(low))
}

/// The motion's on_end, due on its source on the next dispatch with `at` appended.
fn schedule_end(world: &mut World, motion: &Motion, at: Vec2) {
    use super::scheduled_effect as se;
    let Some(template) = &motion.on_end else { return };
    let mut args = template.args.clone();
    args.push(omb_script_abi::types::ScheduleArg { kind: se::ARG_POINT, num: Fixed64::ZERO,
        unit: omb_script_abi::types::EntityHandle::INVALID, point: at });
    let spec = se::Spec { args, ..template.clone() };
    if world.try_fetch::<Vec<crate::comp::Outcome>>().is_none() { world.insert(Vec::<crate::comp::Outcome>::new()); }
    world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::AddBuff {
        target: motion.source, buff_id: spec.buff_id(), duration: se::GRACE, payload: spec.payload() });
}

/// Another living unit that is not a building ahead of `victim` at `at` (moving along
/// `direction`) within `contact` of their bodies' edges.
fn touches(world: &World, victim: Entity, at: Vec2, direction: Vec2, contact: Fixed64) -> bool {
    use specs::Join;
    let reach = radius(world, victim) + contact;
    let (entities, positions, properties, buildings) =
        (world.entities(), world.read_storage::<Pos>(), world.read_storage::<CProperty>(), world.read_storage::<IsBuilding>());
    (&entities, &positions, &properties, !&buildings).join().any(|(other, pos, property, ())| {
        if other == victim || property.hp <= Fixed64::ZERO { return false; }
        let gap = pos.0 - at;
        gap.dot(direction) > Fixed64::ZERO && gap.length() <= reach + radius(world, other)
    })
}

/// Advances every motion by this tick's `DeltaTime`, in canonical entity order,
/// and publishes each new position as a `PreStepMovement` fact (facing kept).
pub fn step(world: &mut World) {
    let dt = world.read_resource::<DeltaTime>().0;
    let Some(mut rows) = world.try_fetch_mut::<ForcedMotions>().map(|mut rows| std::mem::take(&mut rows.0)) else { return; };
    if dt <= Fixed64::ZERO { world.write_resource::<ForcedMotions>().0 = rows; return; }
    let mut moved = Vec::new();
    let mut ended = Vec::new();
    rows.retain(|entity, motion| {
        let keep = advance(world, *entity, motion, dt, &mut moved);
        if !keep && motion.on_end.is_some() {
            let at = world.read_storage::<Pos>().get(*entity).map_or(Vec2::ZERO, |pos| pos.0);
            ended.push((motion.clone(), at));
        }
        keep
    });
    world.write_resource::<ForcedMotions>().0 = rows;
    for (motion, at) in ended { schedule_end(world, &motion, at); }
    publish(world, moved);
}

/// One motion's step; false when it ends.
fn advance(world: &World, entity: Entity, motion: &mut Motion, dt: Fixed64, moved: &mut Vec<(Entity, Vec2)>) -> bool {
    {
        let entity = &entity;
        if !movable(world, *entity) || !world.read_resource::<BuffStore>().has(*entity, BUFF) { return false; }
        let from = world.read_storage::<Pos>().get(*entity).expect("movable position").0;
        let last = motion.time_left <= dt;
        let travel = if last { motion.remaining } else { motion.remaining.min(motion.remaining * dt / motion.time_left) };
        let to = match motion.to {
            Some(end) if last || travel >= motion.remaining => end,
            _ => from + Vec2::new(motion.direction.x * travel, motion.direction.y * travel),
        };
        let wall = world.try_fetch::<BlockedRegions>().and_then(|regions| wall_contact(from, to, radius(world, *entity), &regions));
        let end = wall.unwrap_or(to);
        if end != from {
            world.write_storage::<Pos>().get_mut(*entity).expect("movable position").0 = end;
            moved.push((*entity, end));
        }
        motion.remaining = motion.remaining - travel;
        motion.time_left = motion.time_left - dt;
        let contact = motion.contact.is_some_and(|contact| touches(world, *entity, end, motion.direction, contact));
        wall.is_none() && !contact && !last && motion.remaining > Fixed64::ZERO
    }
}

/// Each new position as a `PreStepMovement` fact (facing kept).
pub(crate) fn publish(world: &World, moved: Vec<(Entity, Vec2)>) {
    let tick = world.read_resource::<Tick>().0;
    for (entity, pos) in moved {
        let source = crate::runtime::canonical_entity_id(entity);
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
            key: crate::runtime::FactOrderingKey { tick, phase: crate::runtime::FactPhase::PreStep,
                canonical_source_order: source, local_ordinal: 1, fact_kind: crate::runtime::FactKind::PreStepMovement },
            audience: crate::runtime::FactAudience::VisibilityPolicy(
                omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned()),
            fact: crate::runtime::ObservableFact::Movement { source, x_mm: pos.x.raw(), y_mm: pos.y.raw(), facing_ticks: None },
        }).expect("valid forced movement fact");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn p(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

    #[test]
    fn a_dash_moves_its_own_caster_continuously_rooted_not_stunned_and_stops_at_a_wall() {
        for wall in [false, true] {
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
            world.insert(crate::runtime::ObservableFactBuffer::default());
            world.insert(DeltaTime(Fixed64::from_raw(256)));
            if wall {
                world.insert(BlockedRegions(vec![super::super::comp::BlockedRegion { name: "wall".into(), points: vec![
                    vek::Vec2::new(500.0, -200.0), vek::Vec2::new(600.0, -200.0), vek::Vec2::new(600.0, 200.0), vek::Vec2::new(500.0, 200.0)] }]));
            }
            let caster = world.create_entity().with(Pos(p(0, 0))).with(super::super::comp::Facing(omoba_sim::Angle::from_degrees_i32(90)))
                .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                    def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
                .with(CollisionRadius(Fixed64::from_i32(20))).build();
            let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, mode: Mode::Dash,
                distance: Fixed64::from_i32(1_000), cast_range: Fixed64::from_i32(1_001), to: Some(p(1_000, 0)) }.checked().unwrap();
            // One second for 1000 units: four 0.25 s steps.
            assert!(apply(&mut world, caster, BUFF, Fixed64::ONE, &spec.payload()), "wall={wall}");
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, p(0, 0), "not an instant teleport");
            assert_eq!(world.read_storage::<super::super::comp::Facing>().get(caster).unwrap().0, omoba_sim::Angle::ZERO, "faces the dash");
            {
                let buffs = world.read_resource::<BuffStore>();
                assert!(buffs.is_rooted(caster) && !buffs.has(caster, omb_script_abi::buff_ids::BuffId::Stun.as_str()), "rooted, not stunned");
            }
            step(&mut world);
            assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, p(250, 0), "a quarter of the way after a quarter second");
            for _ in 0..3 { step(&mut world); }
            let end = world.read_storage::<Pos>().get(caster).unwrap().0;
            if wall {
                assert!(end.x < Fixed64::from_i32(481) && end.x > Fixed64::from_i32(470), "stops at the wall: {end:?}");
            } else {
                assert_eq!(end, p(1_000, 0));
            }
            assert!(world.read_resource::<ForcedMotions>().0.is_empty(), "the motion ends");
        }
    }

    #[test]
    fn a_knockback_from_a_point_can_push_its_own_source_without_a_stun() {
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        world.insert(crate::runtime::ObservableFactBuffer::default());
        world.insert(DeltaTime(Fixed64::from_raw(256)));
        let caster = world.create_entity().with(Pos(p(0, 0)))
            .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
        let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, mode: Mode::Knockback,
            distance: Fixed64::from_i32(400), cast_range: Fixed64::from_i32(10), to: None }.checked().unwrap();
        // 茶茶：pushed forward (+x) from a point one unit behind.
        let options = Options { origin: Some(p(-1, 0)), stun: false, height: None, contact: None };
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &spec.payload(), Some(caster)).is_none(), "no origin: never on its source");
        assert!(apply(&mut world, caster, BUFF, Fixed64::ONE, &spec.payload_with(options)));
        {
            let buffs = world.read_resource::<BuffStore>();
            assert!(buffs.is_rooted(caster) && !buffs.has(caster, omb_script_abi::buff_ids::BuffId::Stun.as_str()), "rooted, not stunned");
        }
        for _ in 0..4 { step(&mut world); }
        assert_eq!(world.read_storage::<Pos>().get(caster).unwrap().0, p(400, 0));
    }

    /// E494: a knockback with a contact radius stops at the first unit ahead of it (not one
    /// behind), and its on_end is due once on the source with the victim's last position.
    #[test]
    fn a_knockback_stops_on_contact_and_schedules_its_end_on_the_source() {
        use super::super::scheduled_effect as se;
        for blocker in [true, false] {
            let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
            world.insert(crate::runtime::ObservableFactBuffer::default());
            world.insert(DeltaTime(Fixed64::from_raw(256)));
            let body = |world: &mut World, at: Vec2| world.create_entity().with(Pos(at)).with(CollisionRadius(Fixed64::from_i32(20)))
                .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                    def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
            // A self push from a point behind (the admission of a hostile victim is not under test).
            let victim = body(&mut world, p(0, 0));
            let caster = victim;
            body(&mut world, p(-50, 0));
            if blocker { body(&mut world, p(300, 0)); }
            let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, mode: Mode::Knockback,
                distance: Fixed64::from_i32(800), cast_range: Fixed64::from_i32(600), to: None }.checked().unwrap();
            let template = se::Spec { ability: "a14_impact_spear".into(), level: 2, step: 3, instance: 9, args: Vec::new() };
            let options = Options { origin: Some(p(-1, 0)), stun: false, height: None, contact: Some(Fixed64::from_i32(100)) };
            let payload = with_on_end(spec.payload_with(options), &template);
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &payload), Some(spec), "round trips");
            assert_eq!(on_end_of(&payload), Some(template.clone()));
            assert!(apply(&mut world, victim, BUFF, Fixed64::ONE, &payload));
            for _ in 0..4 { step(&mut world); }
            let at = world.read_storage::<Pos>().get(victim).unwrap().0;
            if blocker {
                // 200 units a step: stops at x = 200, within 100 of the blocker's edge (300 - 20 - 20 - 100 = 160).
                assert_eq!(at, p(200, 0), "stopped by the unit ahead");
            } else {
                assert_eq!(at, p(800, 0), "the unit behind it does not stop it");
            }
            assert!(world.read_resource::<ForcedMotions>().0.is_empty());
            let outcomes = world.read_resource::<Vec<crate::comp::Outcome>>();
            let due: Vec<_> = outcomes.iter().filter_map(|o| match o {
                crate::comp::Outcome::AddBuff { target, buff_id, payload, .. } if *target == caster => se::Spec::from_buff(buff_id, payload),
                _ => None }).collect();
            assert_eq!(due.len(), 1, "once: {due:?}");
            assert_eq!((due[0].step, due[0].args.len()), (3, 1));
            assert_eq!((due[0].args[0].kind, due[0].args[0].point), (se::ARG_POINT, at), "the end point appended");
        }
    }

    #[test]
    fn a_dash_packet_is_only_admitted_on_its_own_caster() {
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        let body = |world: &mut World, at: Vec2| world.create_entity().with(Pos(at))
            .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
        let (caster, other) = (body(&mut world, p(0, 0)), body(&mut world, p(10, 0)));
        let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, mode: Mode::Dash,
            distance: Fixed64::from_i32(500), cast_range: Fixed64::from_i32(501), to: Some(p(500, 0)) }.checked().unwrap();
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &spec.payload(), Some(caster)).is_some());
        assert!(admitted(&world, other, BUFF, Fixed64::ONE, &spec.payload(), None).is_none(), "never moves another unit");
        let far = Spec { cast_range: Fixed64::from_i32(400), ..spec };
        assert!(admitted(&world, caster, BUFF, Fixed64::ONE, &far.payload(), None).is_none(), "endpoint beyond range");
    }
    fn spec(mode: Mode) -> Spec {
        Spec { source_id: 3, source_generation: 1, mode, distance: Fixed64::from_i32(400), cast_range: Fixed64::from_i32(600),
            to: (mode == Mode::Dash).then(|| Vec2::new(Fixed64::from_i32(400), Fixed64::ZERO)) }
    }
    #[test]
    fn payload_round_trips_strictly() {
        for mode in [Mode::Knockback, Mode::Pull, Mode::Dash] {
            let s = spec(mode);
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &s.payload()), Some(s));
        }
        assert_eq!(Spec { to: None, ..spec(Mode::Dash) }.checked(), None, "a dash names its endpoint");
        assert_eq!(Spec { to: spec(Mode::Dash).to, ..spec(Mode::Pull) }.checked(), None, "only a dash has an endpoint");
        let s = spec(Mode::Knockback);
        assert_eq!(Spec::from_payload("stun", Fixed64::ONE, &s.payload()), None, "only the status buff carries it");
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ZERO, &s.payload()), None);
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(MAX_DURATION + 1), &s.payload()), None);
        let mut extra = s.payload(); extra[KEY]["bonus"] = 1.into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &extra), None);
        let options = Options { origin: Some(Vec2::new(Fixed64::from_i32(-1), Fixed64::ZERO)), stun: false, height: Some(Fixed64::from_i32(200)),
            contact: Some(Fixed64::from_i32(100)) };
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &s.payload_with(options)), Some(s));
        let mut low = s.payload_with(options); low[KEY]["height_raw"] = 0.into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &low), None, "a height is above zero");
        assert_eq!(Options::of(&s.payload_with(options)), options);
        assert_eq!(Options::of(&s.payload()), Options::default());
        assert_eq!(s.payload_with(Options::default()), s.payload());
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &spec(Mode::Pull).payload_with(options)), None, "options are knockback only");
        let mut stun = s.payload(); stun[KEY]["no_stun"] = false.into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &stun), None, "no_stun is only ever true");
        let mut mode = s.payload(); mode[KEY]["mode"] = "throw".into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &mode), None);
        for (field, value) in [("distance_raw", 0), ("distance_raw", Fixed64::from_i32(MAX_DISTANCE + 1).raw()), ("range_raw", 0)] {
            let mut bad = s.payload(); bad[KEY][field] = value.into();
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None, "{field}={value}");
        }
    }
}

/// v3 `redirect_motion { who, toward, speed }` (E586, B24W's bounce toward its stele): a one-shot
/// packet on `who` that turns its running knockback or pull toward `toward` at `speed` for the
/// time it has left, ending on `toward` at the latest. Nothing happens without such a motion.
pub const REDIRECT_KEY: &str = "__redirect_motion_v1";
pub const REDIRECT_BUFF: &str = "__redirect_motion";

pub fn redirect_payload(toward_id: u32, toward_generation: u32, speed: Fixed64) -> Value {
    json!({ (REDIRECT_KEY): { "toward": [toward_id, toward_generation], "speed_raw": speed.raw() } })
}

fn redirect_of(payload: &Value) -> Option<(Entity, Fixed64)> {
    if payload.as_object()?.len() != 1 { return None; }
    let row = payload.get(REDIRECT_KEY)?.as_object().filter(|row| row.len() == 2)?;
    let toward = row.get("toward")?.as_array().filter(|unit| unit.len() == 2)?;
    let (id, gen) = (u32::try_from(toward[0].as_u64()?).ok()?, i32::try_from(toward[1].as_u64()?).ok().filter(|gen| *gen > 0)?);
    let speed = Fixed64::from_raw(row.get("speed_raw")?.as_i64()?);
    (speed > Fixed64::ZERO && speed <= Fixed64::from_i32(10_000))
        .then(|| (Entity::new(id, specs::world::Generation::new(gen)), speed))
}

/// Settles a redirect packet on `target`; false when it changes nothing.
pub fn redirect(world: &mut World, target: Entity, payload: &Value) -> bool {
    let Some((toward, speed)) = redirect_of(payload) else { return false };
    if !world.entities().is_alive(toward) { return false; }
    let (Some(from), Some(goal)) = ({ let p = world.read_storage::<Pos>(); (p.get(target).map(|p| p.0), p.get(toward).map(|p| p.0)) })
        else { return false };
    let gap = goal - from;
    let length = gap.length();
    if length <= Fixed64::ZERO { return false; }
    let Some(mut motions) = world.try_fetch_mut::<ForcedMotions>() else { return false };
    let Some(motion) = motions.0.get_mut(&target).filter(|motion| motion.mode != Mode::Dash && motion.time_left > Fixed64::ZERO) else { return false };
    motion.direction = gap.normalized();
    motion.remaining = (speed * motion.time_left).min(length);
    motion.to = None;
    true
}

#[cfg(test)]
mod redirect_tests {
    use super::*;
    use specs::Builder;

    /// E586: a running knockback turns toward the unit at the new speed for its time left, never
    /// past it; without a motion, a malformed packet or a dash, nothing changes.
    #[test]
    fn a_redirect_turns_a_running_knockback_toward_the_unit() {
        let mut world = World::new();
        world.register::<Pos>();
        let at = |x: i32, y: i32| Pos(Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)));
        let victim = world.create_entity().with(at(0, 0)).build();
        let stele = world.create_entity().with(at(0, 300)).build();
        let motion = Motion { source: stele, mode: Mode::Knockback, direction: Vec2::new(Fixed64::ONE, Fixed64::ZERO),
            remaining: Fixed64::from_i32(200), time_left: Fixed64::from_raw(512), to: None, contact: None, on_end: None };
        assert!(!redirect(&mut world, victim, &redirect_payload(stele.id(), stele.gen().id() as u32, Fixed64::from_i32(1000))), "no motion");
        world.insert(ForcedMotions([(victim, motion.clone())].into_iter().collect()));
        assert!(redirect(&mut world, victim, &redirect_payload(stele.id(), stele.gen().id() as u32, Fixed64::from_i32(400))));
        let now = world.read_resource::<ForcedMotions>().0[&victim].clone();
        assert!(now.direction.x == Fixed64::ZERO && now.direction.y > Fixed64::from_raw(1000), "toward the stele");
        assert_eq!(now.remaining, Fixed64::from_i32(200), "400 for half a second");
        assert!(redirect(&mut world, victim, &redirect_payload(stele.id(), stele.gen().id() as u32, Fixed64::from_i32(1000))));
        assert_eq!(world.read_resource::<ForcedMotions>().0[&victim].remaining, Fixed64::from_i32(300), "never past the unit");
        assert!(!redirect(&mut world, victim, &redirect_payload(stele.id(), stele.gen().id() as u32, Fixed64::ZERO)), "no speed");
        world.write_resource::<ForcedMotions>().0.get_mut(&victim).unwrap().mode = Mode::Dash;
        assert!(!redirect(&mut world, victim, &redirect_payload(stele.id(), stele.gen().id() as u32, Fixed64::from_i32(400))), "a dash is the unit's own");
    }
}
