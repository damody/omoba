//! Approach to cast (E210). A unit-target ability ordered beyond its cast range walks into
//! range first and is cast there, as in the source (Dota orders, v3 `target.range` for unit
//! targets): the host owns it, never a handler. Point abilities keep their own source
//! behavior for now (a dash or line clamps to its range, others refuse): content does not
//! tell a position from a direction yet, and v3 states the approach rule for unit targets.
//!
//! - Every decision reads positions as they were at the start of the tick
//!   (`TickStartPositions`): the authority after its pre-step NPC movement, a team replica
//!   after the same pre-step facts. Both hold those positions exactly; mid-tick positions
//!   of enemy units differ (the replica receives them after the step).
//! - The fresh order is judged on both sides alike: in range, the cast is admitted as
//!   before; out of range, the hero takes a move order toward the target and the request is
//!   kept as an approach.
//! - Only the authority ends an approach. In range at a tick start, it admits the kept
//!   request and announces it to the team replica as an accepted cast input; when the target
//!   died, was removed or is no longer disclosed to the caster's team, it stops the hero and
//!   announces a Stop. The replica only steers; it never ends an approach on its own.
//! - Any new order of the hero (move, attack, stop, cast) clears the approach
//!   (`clear_hero_command_queue`). Control effects pause the walk and keep the order.
use std::collections::BTreeMap;
use omoba_sim::{Fixed64, Vec2};
use specs::{Entity, Join, World, WorldExt};

use super::comp::*;

/// Positions at the start of the tick, before hero movement (see the module notes).
#[derive(Default)]
pub struct TickStartPositions {
    tick: Option<u64>,
    rows: BTreeMap<Entity, Vec2>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApproachTarget { Unit(Entity), Point(Vec2) }

#[derive(Clone, Debug)]
pub struct Approach {
    pub request: PendingAbilityCast,
    pub target: ApproachTarget,
    pub range: Fixed64,
}

/// Approaches in hero order.
#[derive(Default)]
pub struct CastApproaches(pub BTreeMap<Entity, Approach>);

/// Synthetic accepted inputs announced by the authority carry this bit in their stable order.
pub const ANNOUNCED_ORDER_BIT: u64 = 1 << 62;

fn is_replica(world: &World) -> bool { world.has_value::<crate::runtime::filtered_specs::ReplicaPhaseTrace>() }

fn applies(world: &World) -> bool { world.try_fetch::<crate::runtime::MobaMatch>().is_some() || is_replica(world) }

/// Called once per tick before hero movement (authority: end of `begin_moba_match_tick`;
/// replica: after the pre-step facts).
pub fn capture_tick_start(world: &mut World) {
    if !applies(world) { return; }
    let tick = world.try_fetch::<Tick>().map(|tick| tick.0);
    let rows = (&world.entities(), &world.read_storage::<Pos>()).join().map(|(entity, pos)| (entity, pos.0)).collect();
    world.insert(TickStartPositions { tick, rows });
}

fn start_pos(world: &World, entity: Entity) -> Option<Vec2> {
    let tick = world.try_fetch::<Tick>().map(|tick| tick.0);
    if let Some(start) = world.try_fetch::<TickStartPositions>().filter(|start| start.tick == tick) {
        if let Some(pos) = start.rows.get(&entity) { return Some(*pos); }
    }
    world.read_storage::<Pos>().get(entity).map(|pos| pos.0)
}

fn target_pos(world: &World, target: ApproachTarget) -> Option<Vec2> {
    match target { ApproachTarget::Unit(entity) => start_pos(world, entity), ApproachTarget::Point(point) => Some(point) }
}

fn in_range(world: &World, hero: Entity, target: ApproachTarget, range: Fixed64) -> Option<bool> {
    let (from, to) = (start_pos(world, hero)?, target_pos(world, target)?);
    Some((to - from).length_squared() <= range * range)
}

/// The approach a fresh request needs: a learned unit-target ability off cooldown whose
/// target is beyond the rank's cast range. Anything else goes to admission unchanged.
fn plan(world: &World, req: &PendingAbilityCast) -> Option<(Entity, ApproachTarget, Fixed64)> {
    let hero = super::game_processor::player_hero_entity(world, "AbilityCast", req.owner_pid).ok()?;
    if super::command_restriction::restricted(world, hero) { return None; }
    let (id, rank) = {
        let heroes = world.read_storage::<Hero>();
        let h = heroes.get(hero)?;
        let id = h.abilities.get(req.ability_index as usize).filter(|id| !id.is_empty())?.clone();
        if !h.can_use_ability(&id) || h.is_on_cooldown(&id) { return None; }
        let rank = h.get_ability_level(&id);
        (id, rank)
    };
    let definition = omoba_template_ids::ability_by_name(&id).and_then(omoba_template_ids::active_ability_const)?;
    let range = definition.levels.get(usize::try_from(rank).ok()?.checked_sub(1)?)?.range;
    if range <= Fixed64::ZERO { return None; }
    let kind = world.try_fetch::<crate::runtime::ability_runtime::AbilityRegistry>()
        .and_then(|registry| registry.get(&id).map(|def| def.target_type.clone()))?;
    let target = match kind {
        crate::ability_meta::TargetType::Unit => ApproachTarget::Unit(super::game_processor::entity_by_id(world, req.target_entity?)?),
        _ => return None,
    };
    if let ApproachTarget::Unit(entity) = target { if entity == hero { return None; } }
    (!in_range(world, hero, target, range)?).then_some((hero, target, range))
}

/// Moves the hero toward the target's tick-start position.
fn steer(world: &mut World, hero: Entity, target: ApproachTarget) {
    let Some(to) = target_pos(world, target) else { return };
    let Some(pos) = super::game_processor::resolve_move_destination(world, hero, to) else { return };
    let mut queues = world.write_storage::<HeroCommandQueue>();
    if queues.get(hero).is_none() { let _ = queues.insert(hero, HeroCommandQueue::default()); }
    if let Some(queue) = queues.get_mut(hero) { queue.replace(HeroCommand::MoveTo { pos }); }
}

/// A fresh request beyond its range becomes an approach (true); the caller then skips admission.
pub(crate) fn intercept(world: &mut World, req: &PendingAbilityCast) -> bool {
    if !applies(world) { return false; }
    let Some((hero, target, range)) = plan(world, req) else { return false };
    // The order replaces whatever the hero did, like any accepted command.
    super::game_processor::clear_hero_command_queue(world, hero);
    super::cast_windup::cancel(world, hero);
    super::game_processor::interrupt_attack_for_accepted_command(world, hero);
    if let Some(mut state) = world.try_fetch_mut::<crate::runtime::MobaMatch>() { state.neutralize_direction(req.owner_pid, hero); }
    steer(world, hero, target);
    world.entry::<CastApproaches>().or_insert_with(Default::default).0.insert(hero, Approach { request: req.clone(), target, range });
    log::info!("AbilityCast approach pid={} slot={} hero={:?}", req.owner_pid, req.ability_index, hero);
    true
}

/// The hero takes another order: the approach ends with it.
pub(crate) fn clear(world: &World, hero: Entity) {
    if let Some(mut approaches) = world.try_fetch_mut::<CastApproaches>() { approaches.0.remove(&hero); }
}

fn alive(world: &World, entity: Entity) -> bool {
    world.entities().is_alive(entity) && world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
}

/// The target is still a live unit disclosed to the caster's team (authority only).
fn still_valid(world: &World, hero: Entity, target: ApproachTarget) -> bool {
    if !alive(world, hero) { return false; }
    let ApproachTarget::Unit(entity) = target else { return true };
    if !alive(world, entity) { return false; }
    let team = world.read_storage::<Faction>().get(hero).map(|faction| faction.team_id.max(0) as u32);
    match (team, world.try_fetch::<crate::runtime::TeamVisibilityRuntime>()) {
        (Some(team), Some(visibility)) => visibility.teams.get(&team)
            .is_none_or(|state| state.index.current.contains(&crate::runtime::canonical_entity_id(entity))),
        _ => true,
    }
}

/// Announces an input the authority decided to the caster's team replica, like an accepted one.
fn announce(world: &World, hero: Entity, approach: &Approach, input: crate::game_proto::PlayerInput, kind: u32, target: Option<Entity>) {
    use prost::Message as _;
    let Some(mut projection) = world.try_fetch_mut::<crate::runtime::TeamProjectionRuntime>() else { return };
    let Some(team) = world.read_storage::<Faction>().get(hero).map(|faction| faction.team_id.max(0) as u32) else { return };
    let tick = world.try_fetch::<Tick>().map_or(0, |tick| tick.0);
    let order = ANNOUNCED_ORDER_BIT | (tick << 16) | u64::from(approach.request.owner_pid & 0xffff);
    projection.pending_accepted_inputs.push(crate::runtime::CanonicalAcceptedInput::from_authoritative_acceptance(
        team, approach.request.owner_pid, order, kind, crate::runtime::canonical_entity_id(hero),
        target.map(crate::runtime::canonical_entity_id), input.encode_to_vec()));
}

/// Before this tick's new requests (AbilityCasts). Returns the kept requests whose hero
/// reached the range (authority), to be admitted now. INCOMING: this tick's new requests;
/// their heroes follow the new order instead.
pub(crate) fn advance(world: &mut World, incoming: &[PendingAbilityCast]) -> Vec<PendingAbilityCast> {
    use crate::game_proto::player_input::Action;
    if !applies(world) { return Vec::new(); }
    let approaches: Vec<(Entity, Approach)> = match world.try_fetch::<CastApproaches>() {
        Some(approaches) => approaches.0.iter().map(|(hero, approach)| (*hero, approach.clone())).collect(),
        None => return Vec::new(),
    };
    let replica = is_replica(world);
    let mut cast_now = Vec::new();
    for (hero, approach) in approaches {
        if incoming.iter().any(|req| req.owner_pid == approach.request.owner_pid) { continue; }
        if replica { steer(world, hero, approach.target); continue; }
        if !still_valid(world, hero, approach.target) {
            clear(world, hero);
            super::game_processor::clear_hero_command_queue(world, hero);
            announce(world, hero, &approach, crate::game_proto::PlayerInput {
                action: Some(Action::Stop(crate::game_proto::Stop {})) }, 21, None);
            log::info!("AbilityCast approach stopped pid={} hero={:?} reason=target_lost", approach.request.owner_pid, hero);
            continue;
        }
        if in_range(world, hero, approach.target, approach.range) == Some(true) {
            clear(world, hero);
            let (target_entity, target_pos) = match approach.target {
                ApproachTarget::Unit(entity) => (Some(entity), None),
                ApproachTarget::Point(point) => (None, Some(point)),
            };
            announce(world, hero, &approach, crate::game_proto::PlayerInput {
                action: Some(Action::CastAbility(crate::game_proto::CastAbility {
                    ability_index: approach.request.ability_index, target_entity: None,
                    target_pos: target_pos.map(|p| crate::game_proto::Vec2I { x: p.x.raw() as i32, y: p.y.raw() as i32 }),
                })) }, 4, target_entity);
            cast_now.push(approach.request.clone());
            continue;
        }
        steer(world, hero, approach.target);
    }
    cast_now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announced_inputs_use_their_own_order_space() {
        assert_eq!(ANNOUNCED_ORDER_BIT & ((60u64 << 16) | 7), 0);
    }
}
