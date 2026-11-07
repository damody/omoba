//! Ground pickups (v3 `create_pickup`; C06T 豪奪的金袋, 原作 item_money). A script drops a
//! pickup by sending its own caster the request envelope `add_stat_buff(caster, BUFF, _, {KEY: ..})`.
//!
//! Flow:
//! 1. Request: settlement queues it on the authority ([`PendingPickups`], installed by the match).
//!    Replicas have no queue and drop it; the pickup reaches them as an ordinary disclosed body.
//! 2. Spawn: the next active match tick, before any movement, the pickup becomes a neutral body
//!    (identity [`IDENTITY`]) at its point: vision-disclosed like any unit (hidden in fog),
//!    attackable by a direct order of either team but never acquired automatically (a hostile
//!    neutral, like a camp), destroyed by one hit when `killable` (原作 ItemKillable),
//!    invulnerable otherwise.
//! 3. Pick up: each active tick, the nearest live hero allowed to take it (`who`) within
//!    [`PICKUP_RADIUS`] gains its gold (one pool, `script_gold::settle`; FC-129～131) and the body
//!    goes; ties go to the lowest canonical id. A destroyed body gives nothing.
//! 4. Expiry: an optional lifetime removes it unpicked. 原作 ground items never expire, so C06T
//!    passes none.
//!
//! Inferred: Dota picks an item up on an explicit order within about 150; here a hero within 150
//! takes it without an order (bots never path to pickups).
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Builder, Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::comp::{CProperty, Facing, Faction, FactionType, Gold, Pos, ReplicationScope, ReplicationScopeKind};

pub const KEY: &str = "__pickup_v1";
/// Request envelope id; never stored as a buff.
pub const BUFF: &str = "__create_pickup";
/// Presentation identity of the gold bag body (原作 item_money, models/props_gameplay/gold_bag).
pub const IDENTITY: &str = "item_money";
/// A hero within this distance takes the pickup (inferred, Dota item pickup reach).
pub const PICKUP_RADIUS: i32 = 150;
pub const MAX_GOLD: i64 = 100_000;
pub const MAX_LIFETIME_SECONDS: i32 = 3_600;
/// Live pickups per match; requests past it are dropped (C06T drops at most 10 per cast).
pub const MAX_LIVE: usize = 256;
/// Requests queued per tick.
const MAX_PENDING: usize = 64;

/// Who may take a pickup, relative to the dropping caster's team.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who { Any, Allies, Enemies }

impl Who {
    fn as_str(self) -> &'static str { match self { Who::Any => "any", Who::Allies => "allies", Who::Enemies => "enemies" } }
    fn parse(text: &str) -> Option<Self> {
        match text { "any" => Some(Who::Any), "allies" => Some(Who::Allies), "enemies" => Some(Who::Enemies), _ => None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub at: Vec2,
    pub gold: i64,
    pub who: Who,
    pub killable: bool,
    /// Zero: until picked up or destroyed.
    pub lifetime: Fixed64,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let max = omoba_sim::terrain::MAX_GEOMETRY_RAW;
        (self.at.x.raw().abs() <= max && self.at.y.raw().abs() <= max && (0..=MAX_GOLD).contains(&self.gold)
            && self.lifetime >= Fixed64::ZERO && self.lifetime <= Fixed64::from_i32(MAX_LIFETIME_SECONDS)).then_some(self)
    }
    pub fn payload(self) -> Value {
        json!({KEY: {"x": self.at.x.raw(), "y": self.at.y.raw(), "gold": self.gold, "who": self.who.as_str(),
            "killable": self.killable, "lifetime": self.lifetime.raw()}})
    }
    pub fn from_payload(payload: &Value) -> Option<Self> {
        let row = payload.get(KEY)?.as_object()?;
        let raw = |key: &str| row.get(key).and_then(Value::as_i64);
        Self { at: Vec2::new(Fixed64::from_raw(raw("x")?), Fixed64::from_raw(raw("y")?)), gold: raw("gold")?,
            who: Who::parse(row.get("who")?.as_str()?)?, killable: row.get("killable")?.as_bool()?,
            lifetime: Fixed64::from_raw(raw("lifetime")?) }.checked()
    }
}

/// Authority queue of requests settled since the last applied tick, with the dropper's team.
#[derive(Clone, Debug, Default)]
pub struct PendingPickups(pub Vec<(Spec, i32)>);

/// Settlement of the request envelope: queued on the authority, dropped elsewhere.
pub fn request(world: &World, caster: Entity, payload: &Value) {
    let Some(spec) = Spec::from_payload(payload) else { return };
    let Some(team) = world.read_storage::<Faction>().get(caster).map(|faction| faction.team_id) else { return };
    if let Some(mut pending) = world.try_fetch_mut::<PendingPickups>() {
        if pending.0.len() < MAX_PENDING { pending.0.push((spec, team)); }
    }
}

/// One live pickup body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pickup {
    pub entity: Entity,
    pub gold: i64,
    pub who: Who,
    pub team: i32,
    pub expire_at: Option<Fixed64>,
}

/// The match's live pickups by canonical id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pickups(pub BTreeMap<u64, Pickup>);

impl Pickups {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

fn live(world: &World, entity: Entity) -> bool {
    world.entities().is_alive(entity) && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
}

fn spawn(world: &mut World, spec: &Spec) -> Entity {
    use super::comp::{AiType, Unit, UnitType};
    let mut unit = Unit::new(IDENTITY.into(), IDENTITY.into(), UnitType::Neutral);
    unit.max_hp = 1;
    unit.current_hp = 1;
    unit.base_damage = 0;
    unit.move_speed = Fixed64::ZERO;
    unit.attack_range = Fixed64::ZERO;
    unit.ai_type = AiType::None;
    unit.gold_reward = 0;
    unit.exp_reward = 0;
    world.create_entity().with(unit)
        .with(Faction::new(FactionType::HostileNeutral, 0))
        .with(ReplicationScope { kind: ReplicationScopeKind::Vision, owner_team: None })
        .with(Pos(spec.at))
        .with(Facing(omoba_sim::Angle::ZERO))
        .with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
        .build()
}

/// Authority pre-step: spawn the queued pickups, retire destroyed and expired ones, and hand
/// each remaining one to the nearest hero allowed to take it.
pub(crate) fn tick(world: &mut World, state: &mut super::moba_match::MobaMatch) {
    let requests = world.try_fetch_mut::<PendingPickups>().map(|mut pending| std::mem::take(&mut pending.0)).unwrap_or_default();
    for (spec, team) in requests {
        if state.pickups.0.len() >= MAX_LIVE { break; }
        let entity = spawn(world, &spec);
        // A neutral body is never acquired automatically (combat_acquisition), only by a direct order.
        if !spec.killable { state.add_invulnerable_summon(&world.entities(), entity); }
        state.pickups.0.insert(crate::runtime::canonical_entity_id(entity), Pickup { entity, gold: spec.gold, who: spec.who, team,
            expire_at: (spec.lifetime > Fixed64::ZERO).then(|| state.elapsed + spec.lifetime) });
    }
    if state.pickups.is_empty() { return; }
    let heroes: Vec<(u64, Entity, i32, Vec2)> = state.heroes.iter().filter_map(|slot| slot.entity)
        .filter(|hero| live(world, *hero))
        .filter_map(|hero| Some((crate::runtime::canonical_entity_id(hero), hero,
            world.read_storage::<Faction>().get(hero)?.team_id, world.read_storage::<Pos>().get(hero)?.0)))
        .collect();
    let reach = Fixed64::from_i32(PICKUP_RADIUS);
    let mut gone = Vec::new();
    for (id, pickup) in &state.pickups.0 {
        if !live(world, pickup.entity) { gone.push((*id, None)); continue; }
        if pickup.expire_at.is_some_and(|at| state.elapsed >= at) { gone.push((*id, None)); continue; }
        let at = world.read_storage::<Pos>().get(pickup.entity).map_or(Vec2::ZERO, |p| p.0);
        let taker = heroes.iter().filter(|(_, _, team, _)| match pickup.who {
            Who::Any => true, Who::Allies => *team == pickup.team, Who::Enemies => *team != pickup.team })
            .map(|(hero_id, hero, _, pos)| ((*pos - at).length_squared(), *hero_id, *hero))
            .filter(|(d2, _, _)| *d2 <= reach * reach).min_by_key(|(d2, hero_id, _)| (d2.raw(), *hero_id));
        if let Some((_, _, hero)) = taker { gone.push((*id, Some((hero, pickup.gold)))); }
    }
    for (id, taker) in gone {
        let Some(pickup) = state.pickups.0.remove(&id) else { continue };
        state.invulnerable_summons.remove(&(pickup.entity.id(), pickup.entity.gen().id()));
        if let Some((hero, gold)) = taker {
            if let Some(pool) = world.write_storage::<Gold>().get_mut(hero) { pool.0 = super::script_gold::settle(pool.0, gold); }
        }
        if world.entities().is_alive(pickup.entity) { let _ = world.entities().delete(pickup.entity); }
    }
}

#[cfg(test)]
#[path = "pickup_tests.rs"]
mod tests;
