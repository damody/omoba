use crate::comp::*;
use crate::runtime::combat_controls::{within_raw_range, AcquisitionIntent};
use crate::runtime::native::combat_acquisition::{
    acquire, assess_direct_target, AcquisitionRequest, CombatAcquisitionView, DirectStatus,
    DisclosureContext,
};
use crate::tick::hero_move_tick::path_hits_regions;
use omoba_core::runtime::ability_runtime::BuffStore;
use omoba_sim::{fixed::SCALE, Fixed64, Vec2 as SimVec2};
use specs::{shred, Entities, Join, Read, ReadStorage, SystemData, Write, WriteStorage};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const PATH_GRID_WORLD_UNITS: i64 = 64;
const PATH_GRID_RAW: i64 = PATH_GRID_WORLD_UNITS * SCALE;
const PATH_GRID_MARGIN: i64 = 8;
const MAX_PATH_GRID_SPAN: i64 = 96;

#[derive(SystemData)]
pub struct HeroCommandData<'a> {
    dt: Read<'a, DeltaTime>,
    tick: Read<'a, Tick>,
    directional_steps: Option<Write<'a, crate::runtime::input_scheme::DirectionalStepPlan>>,
    admitted_directions: Option<Read<'a, crate::runtime::input_scheme::AdmittedDirectionalSteps>>,
    entities: Entities<'a>,
    heroes: ReadStorage<'a, Hero>,
    pos: ReadStorage<'a, Pos>,
    factions: ReadStorage<'a, Faction>,
    attacks: ReadStorage<'a, TAttack>,
    radii: ReadStorage<'a, CollisionRadius>,
    searcher: Read<'a, Searcher>,
    regions: Read<'a, BlockedRegions>,
    buff_store: Read<'a, omoba_core::runtime::ability_runtime::BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
    queues: WriteStorage<'a, HeroCommandQueue>,
    move_targets: WriteStorage<'a, MoveTarget>,
    properties: ReadStorage<'a, CProperty>,
    owners: ReadStorage<'a, PlayerOwner>,
    visibility: Option<Read<'a, crate::runtime::TeamVisibilityRuntime>>,
    moba: Option<Read<'a, crate::runtime::MobaMatch>>,
    casts: Option<Read<'a, super::super::cast_windup::AbilityWindups>>,
    disclosed_priority: Option<Read<'a, crate::runtime::DisclosedMovementPriority>>,
    admitted: Option<Read<'a, crate::runtime::native::combat_acquisition::AdmittedCombatPreferences>>,
    attack_gate: Option<Read<'a, crate::runtime::filtered_specs::DisclosedAttackabilityGate>>,
    disclosed_wards: Option<Read<'a, crate::runtime::filtered_specs::DisclosedWardCandidates>>,
    replica_identities: ReadStorage<'a, crate::runtime::ReplicaIdentity>,
    ward_bodies: ReadStorage<'a, crate::runtime::DisclosedWardBody>,
}

#[derive(Clone, Debug)]
pub(crate) struct HeroCommandDecision {
    entity: specs::Entity,
    queue: HeroCommandQueue,
    next_waypoint: Option<SimVec2>,
    direction: Option<SimVec2>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = HeroCommandData<'a>;

    const NAME: &'static str = "hero_command";

    fn run(_job: &mut Job<Self>, mut data: Self::SystemData) {
        let arrive_eps = Fixed64::from_raw(768);
        let hero_ids: BTreeSet<specs::Entity> = (&data.entities, &data.heroes)
            .join()
            .map(|(entity, _hero)| entity)
            .collect();
        let mut decisions: Vec<_> = {
            let disclosure = DisclosureContext::resolve(
                data.disclosed_priority.is_some(),
                data.moba.is_some(),
                data.visibility.as_deref(),
            );
            let acquisition = CombatAcquisitionView {
                entities: &data.entities,
                positions: &data.pos,
                health: &data.properties,
                factions: &data.factions,
                owners: &data.owners,
                heroes: &hero_ids,
                searcher: &*data.searcher,
                disclosure,
                admitted: data.admitted.as_deref(),
                attack_gate: data.attack_gate.as_deref(),
                match_state: data.moba.as_deref(),
                disclosed_wards: data.disclosed_wards.as_deref(),
                replica_identities: &data.replica_identities,
                ward_bodies: &data.ward_bodies,
            };
            let read = HeroCommandRead {
                pos: &data.pos,
                factions: &data.factions,
                attacks: &data.attacks,
                radii: &data.radii,
                searcher: &*data.searcher,
                regions: &*data.regions,
                buff_store: &*data.buff_store,
                is_buildings: &data.is_buildings,
            };
            (&data.entities, &data.heroes, &data.pos)
                .join()
                .filter_map(|(entity, _hero, pos)| {
                    let queue = data.queues.get(entity);
                    let direction = data.owners.get(entity)
                        .and_then(|owner| data.moba.as_ref().map(|state| state.direction_for_source(owner.player_id, entity)))
                        .or_else(|| data.admitted_directions.as_ref().filter(|steps| steps.tick == data.tick.0)
                            .and_then(|steps| steps.rows.get(&entity)).map(|step| step.direction))
                        .unwrap_or(SimVec2::ZERO);
                    if direction==SimVec2::ZERO && data.casts.as_ref().is_some_and(|casts|casts.0.contains_key(&entity)) {
                        return Some(HeroCommandDecision {entity,queue:queue.cloned().unwrap_or_default(),
                            next_waypoint:None,direction:None});
                    }
                    if direction != SimVec2::ZERO && data.properties.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
                        && !data.moba.as_ref().is_some_and(|state| state.is_recalling(entity))
                    {
                        let stats = crate::runtime::ability_runtime::UnitStats::from_refs(&*data.buff_store, data.is_buildings.get(entity).is_some());
                        let speed = stats.final_move_speed(data.properties.get(entity).expect("checked hero property").msd, entity);
                        return Some(decide_keyboard_direction(&read, &acquisition, entity, pos.0, queue.cloned().unwrap_or_default(),
                            direction, speed * data.dt.0, data.dt.0, arrive_eps));
                    }
                    queue.map(|queue| decide_hero_command(
                        &read,
                        &acquisition,
                        entity,
                        pos.0,
                        queue.clone(),
                        arrive_eps,
                    ))
                })
                .collect()
        };
        decisions.sort_by_key(|decision| decision.entity.id());

        if let Some(plan)=data.directional_steps.as_mut() {
            plan.tick=data.tick.0;plan.directions.clear();
            for decision in &decisions {
                if let Some(direction)=decision.direction {plan.directions.insert(decision.entity,direction);}
            }
        }

        for decision in decisions {
            if let Some(queue) = data.queues.get_mut(decision.entity) {
                *queue = decision.queue;
            } else {
                // Keep an empty queue after the first held direction so that
                // neutral/expired leases retire their previous MoveTarget.
                let _ = data.queues.insert(decision.entity, decision.queue);
            }
            match decision.next_waypoint {
                Some(target) => {
                    let _ = data
                        .move_targets
                        .insert(decision.entity, MoveTarget(target));
                }
                None => {
                    data.move_targets.remove(decision.entity);
                }
            }
        }
    }
}

/// A held direction never invokes path finding or invents a remote MoveTo.
/// Explicit attacks retain their legal target; windup/impact pause movement,
/// while cooldown/backswing permit movement. Point-click behavior is unchanged.
fn decide_keyboard_direction(
    data: &HeroCommandRead<'_, '_>, acquisition: &CombatAcquisitionView<'_, '_>,
    entity: specs::Entity, pos: SimVec2, mut queue: HeroCommandQueue,
    direction: SimVec2, step: Fixed64, dt: Fixed64, arrive_eps: Fixed64,
) -> HeroCommandDecision {
    if let Some(HeroCommand::AttackTarget {target,..})=queue.active {
        if !matches!(assess_direct_target(acquisition,entity,target),Some(DirectStatus::Legal{..})) {
            queue.advance();
        }
    }
    // Explicit point commands and Hold keep their existing arbitration. A new
    // directional press retires these through the authenticated input router.
    if queue.active.is_some() && !matches!(queue.active,Some(HeroCommand::AttackTarget{..})) {
        return decide_hero_command(data,acquisition,entity,pos,queue,arrive_eps);
    }
    let mut hold = false;
    if let Some(attack)=data.attacks.get(entity) {
        hold=attack.asd_count < Fixed64::ZERO;
        if let Some(HeroCommand::AttackTarget{target,..})=queue.active {
            let stats=crate::runtime::ability_runtime::UnitStats::from_refs(&*data.buff_store,data.is_buildings.get(entity).is_some());
            let multiplier=stats.final_attack_speed_mult(entity).max(Fixed64::from_raw(10));
            let mut clock=attack.asd_count;
            let ready=crate::tick::attack_phase::advance_attack_phase(&mut clock,dt,attack.asd.v/multiplier)
                == crate::tick::attack_phase::AttackPhaseStep::Ready;
            if ready {
                hold |= acquire(acquisition,AcquisitionRequest{source:entity,intent:AcquisitionIntent::AttackTarget,
                    command_point:None,range:stats.final_attack_range(attack.range.v,entity),explicit_target:Some(target)}).is_some();
            }
        }
    }
    let next_waypoint=(!hold && step>Fixed64::ZERO).then(||pos+direction*(step*Fixed64::from_i32(2)+Fixed64::ONE));
    HeroCommandDecision{entity,queue,direction:next_waypoint.map(|_|direction),next_waypoint}
}

fn decide_hero_command(
    data: &HeroCommandRead<'_, '_>,
    acquisition: &CombatAcquisitionView<'_, '_>,
    entity: specs::Entity,
    pos: SimVec2,
    mut queue: HeroCommandQueue,
    arrive_eps: Fixed64,
) -> HeroCommandDecision {
    let mut next_waypoint = None;

    for _ in 0..=HeroCommandQueue::LIMIT {
        let Some(command) = queue.active else {
            break;
        };
        match command {
            HeroCommand::HoldPosition => break,
            HeroCommand::MoveTo { pos: target } => {
                if (target - pos).length() <= arrive_eps {
                    queue.advance();
                    continue;
                }
                next_waypoint = plan_next_waypoint(data, entity, pos, target);
                if next_waypoint.is_none() {
                    log::warn!(
                        "hero_command: rejecting unreachable MoveTo for entity {:?} target=({:.1},{:.1})",
                        entity,
                        target.x.to_f32_for_render(),
                        target.y.to_f32_for_render()
                    );
                    queue.advance();
                    continue;
                }
                break;
            }
            HeroCommand::AttackMove { pos: target } => {
                if (target - pos).length() <= arrive_eps {
                    queue.advance();
                    continue;
                }
                if attack_move_should_hold(acquisition, data, entity, target) {
                    next_waypoint = None;
                    break;
                }
                next_waypoint = plan_next_waypoint(data, entity, pos, target);
                if next_waypoint.is_none() {
                    log::warn!(
                        "hero_command: rejecting unreachable AttackMove for entity {:?} target=({:.1},{:.1})",
                        entity,
                        target.x.to_f32_for_render(),
                        target.y.to_f32_for_render()
                    );
                    queue.advance();
                    continue;
                }
                break;
            }
            HeroCommand::AttackTarget {
                target,
                chase_origin,
            } => {
                let Some(step) = resolve_attack_target_command(
                    acquisition,
                    data,
                    entity,
                    pos,
                    target,
                    chase_origin,
                )
                else {
                    queue.advance();
                    continue;
                };
                match step {
                    AttackTargetStep::Hold => {
                        next_waypoint = None;
                        break;
                    }
                    AttackTargetStep::Chase { target_pos, origin } => {
                        queue.active = Some(HeroCommand::AttackTarget {
                            target,
                            chase_origin: Some(origin),
                        });
                        next_waypoint = plan_next_waypoint(data, entity, pos, target_pos);
                        if next_waypoint.is_none() {
                            log::warn!(
                                "hero_command: rejecting unreachable AttackTarget chase for entity {:?} target {:?}",
                                entity,
                                target
                            );
                            queue.advance();
                            continue;
                        }
                        break;
                    }
                }
            }
        }
    }

    if queue.total_len() > HeroCommandQueue::LIMIT {
        log::warn!("hero_command: queue advance guard tripped for {:?}", entity);
        queue.clear_all();
        next_waypoint = None;
    }

    HeroCommandDecision {
        entity,
        queue,
        next_waypoint,
        direction: None,
    }
}

enum AttackTargetStep {
    Hold,
    Chase {
        target_pos: SimVec2,
        origin: SimVec2,
    },
}

struct HeroCommandRead<'a, 'b> {
    pos: &'b ReadStorage<'a, Pos>,
    factions: &'b ReadStorage<'a, Faction>,
    attacks: &'b ReadStorage<'a, TAttack>,
    radii: &'b ReadStorage<'a, CollisionRadius>,
    searcher: &'b Searcher,
    regions: &'b BlockedRegions,
    buff_store: &'b BuffStore,
    is_buildings: &'b ReadStorage<'a, IsBuilding>,
}

fn attack_phase_locks_animation(data: &HeroCommandRead<'_, '_>, entity: specs::Entity) -> bool {
    matches!(
        data.attacks.get(entity).map(|attack| attack.attack_phase),
        Some(AttackSequencePhase::Windup | AttackSequencePhase::Backswing)
    )
}

fn attack_move_should_hold(
    acquisition: &CombatAcquisitionView<'_, '_>,
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    command_point: SimVec2,
) -> bool {
    if attack_phase_locks_animation(data, entity) {
        return true;
    }
    let Some(range) = effective_attack_range(data, entity) else {
        return false;
    };
    acquire(
        acquisition,
        AcquisitionRequest {
            source: entity,
            intent: AcquisitionIntent::AttackMove,
            command_point: Some(command_point),
            range,
            explicit_target: None,
        },
    )
    .is_some()
}

fn resolve_attack_target_command(
    acquisition: &CombatAcquisitionView<'_, '_>,
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    pos: SimVec2,
    target: specs::Entity,
    chase_origin: Option<SimVec2>,
) -> Option<AttackTargetStep> {
    let status = assess_direct_target(acquisition, entity, target)?;
    let DirectStatus::Legal { position: target_pos } = status else {
        return None;
    };
    if attack_phase_locks_animation(data, entity) {
        return Some(AttackTargetStep::Hold);
    }
    let attack_range = effective_attack_range(data, entity)?;
    if within_raw_range(pos, target_pos, attack_range) {
        return Some(AttackTargetStep::Hold);
    }
    let origin = chase_origin.unwrap_or(pos);
    let leash = attack_range * Fixed64::from_raw(512);
    if (pos - origin).length() > leash {
        return None;
    }
    Some(AttackTargetStep::Chase { target_pos, origin })
}

fn effective_attack_range(
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
) -> Option<Fixed64> {
    let atk = data.attacks.get(entity)?;
    let stats = omoba_core::runtime::ability_runtime::UnitStats::from_refs(
        &*data.buff_store,
        data.is_buildings.get(entity).is_some(),
    );
    Some(stats.final_attack_range(atk.range.v, entity))
}

fn plan_next_waypoint(
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    from: SimVec2,
    target: SimVec2,
) -> Option<SimVec2> {
    let radius = data.radii.get(entity).map(|r| r.0).unwrap_or_else(|| Fixed64::from_i32(20));
    static_next_waypoint(from, target, radius, data.regions)
}

/// Shared bounded, deterministically ordered search over public terrain only.
/// Heroes preserve their existing grid behavior; NPCs try a clear direct path first.
pub(crate) fn static_next_waypoint(
    from: SimVec2,
    target: SimVec2,
    radius: Fixed64,
    regions: &BlockedRegions,
) -> Option<SimVec2> {
    let start = grid_key(from);
    let goal = grid_key(target);
    if path_hits_regions(target, target, radius, regions) { return None; }
    let same_cell = start == goal;
    if same_cell {
        if !path_hits_regions(from, target, radius, regions) { return Some(target); }
        // The actual endpoints can lie on opposite sides of a thin obstacle
        // inside one cell. Search outside the cell instead of rejecting them.
    }

    let dx = (goal.x - start.x).abs();
    let dy = (goal.y - start.y).abs();
    if dx > MAX_PATH_GRID_SPAN || dy > MAX_PATH_GRID_SPAN {
        return (!path_hits_regions(from, target, radius, regions)).then_some(target);
    }

    let min_x = start.x.min(goal.x) - PATH_GRID_MARGIN;
    let max_x = start.x.max(goal.x) + PATH_GRID_MARGIN;
    let min_y = start.y.min(goal.y) - PATH_GRID_MARGIN;
    let max_y = start.y.max(goal.y) + PATH_GRID_MARGIN;

    let mut open = VecDeque::new();
    let mut parent: BTreeMap<GridKey, GridKey> = BTreeMap::new();
    let mut reached = None;
    parent.insert(start, start);
    open.push_back(start);

    while let Some(cur) = open.pop_front() {
        if !same_cell && cur == goal {
            reached = Some(cur);
            break;
        }
        // A same-cell goal cannot reuse the already-visited start key. Treat
        // the precise target as a terminal edge, checked like every other edge.
        if same_cell && cur != start
            && !path_hits_regions(grid_pos(cur), target, radius, regions) {
            reached = Some(cur);
            break;
        }
        for next in neighbors(cur) {
            if next.x < min_x || next.x > max_x || next.y < min_y || next.y > max_y {
                continue;
            }
            if parent.contains_key(&next) {
                continue;
            }
            if next != goal && path_hits_regions(grid_pos(next), grid_pos(next), radius, regions) {
                continue;
            }
            let edge_from = if cur == start { from } else { grid_pos(cur) };
            let edge_to = if next == goal { target } else { grid_pos(next) };
            if path_hits_regions(edge_from, edge_to, radius, regions) { continue; }
            parent.insert(next, cur);
            open.push_back(next);
        }
    }

    // A closest reachable cell is not proof that the requested destination is
    // reachable. Never turn an exhausted bounded search into a partial route.
    let mut cur = reached?;
    while let Some(prev) = parent.get(&cur).copied() {
        if prev == start {
            return Some(if cur == goal { target } else { grid_pos(cur) });
        }
        if prev == cur {
            break;
        }
        cur = prev;
    }
    None
}

/// Never teleport through a blocked edge, including a near-target snap.
pub(crate) fn static_step_toward(from: SimVec2, target: SimVec2, budget: Fixed64,
    radius: Fixed64, regions: &BlockedRegions) -> SimVec2 {
    if budget <= Fixed64::ZERO { return from; }
    let waypoint = if !path_hits_regions(from, target, radius, regions) { Some(target) }
        else { static_next_waypoint(from, target, radius, regions) };
    let Some(waypoint) = waypoint else { return from; };
    let next = omoba_sim::navigation::step_toward(from, waypoint, budget);
    if path_hits_regions(from, next, radius, regions) { from } else { next }
}

pub(crate) fn static_advance_route(route: &[SimVec2], cursor: &mut usize,
    from: SimVec2, budget: Fixed64, radius: Fixed64, regions: &BlockedRegions) -> SimVec2 {
    if budget <= Fixed64::ZERO { return from; }
    let Some(&target) = route.get(*cursor).or_else(|| route.last()) else { return from; };
    // Looking only at this tick's tiny proposed step would alternate between
    // detouring and walking back toward the wall. Keep the full waypoint query.
    if path_hits_regions(from, target, radius, regions) {
        let next = static_step_toward(from, target, budget, radius, regions);
        if next == target && *cursor < route.len() { *cursor += 1; }
        return next;
    }
    let mut proposed_cursor = *cursor;
    let proposed = omoba_sim::navigation::advance_route(route, &mut proposed_cursor, from, budget);
    if !path_hits_regions(from, proposed, radius, regions) {
        *cursor = proposed_cursor;
        return proposed;
    }
    let next = static_step_toward(from, target, budget, radius, regions);
    if next == target && *cursor < route.len() { *cursor += 1; }
    next
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct GridKey {
    x: i64,
    y: i64,
}

fn grid_key(pos: SimVec2) -> GridKey {
    GridKey {
        x: round_fixed_to_grid(pos.x.raw()),
        y: round_fixed_to_grid(pos.y.raw()),
    }
}

fn grid_pos(key: GridKey) -> SimVec2 {
    SimVec2::new(
        Fixed64::from_raw(key.x.saturating_mul(PATH_GRID_RAW)),
        Fixed64::from_raw(key.y.saturating_mul(PATH_GRID_RAW)),
    )
}

fn round_fixed_to_grid(raw: i64) -> i64 {
    let q = raw.div_euclid(PATH_GRID_RAW);
    let r = raw.rem_euclid(PATH_GRID_RAW);
    if r.saturating_mul(2) >= PATH_GRID_RAW {
        q + 1
    } else {
        q
    }
}

fn neighbors(key: GridKey) -> [GridKey; 8] {
    [
        GridKey {
            x: key.x + 1,
            y: key.y,
        },
        GridKey {
            x: key.x,
            y: key.y + 1,
        },
        GridKey {
            x: key.x - 1,
            y: key.y,
        },
        GridKey {
            x: key.x,
            y: key.y - 1,
        },
        GridKey {
            x: key.x + 1,
            y: key.y + 1,
        },
        GridKey {
            x: key.x - 1,
            y: key.y + 1,
        },
        GridKey {
            x: key.x - 1,
            y: key.y - 1,
        },
        GridKey {
            x: key.x + 1,
            y: key.y - 1,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn npc_static_navigation_detours_and_returns_at_60hz() {
        let from = SimVec2::new(Fixed64::ZERO, Fixed64::ZERO);
        let target = SimVec2::new(Fixed64::from_i32(192), Fixed64::ZERO);
        let regions = BlockedRegions(vec![BlockedRegion { name: "thin-npc-wall".into(),
            points: vec![vek::Vec2::new(80.0,-30.0),vek::Vec2::new(81.0,-30.0),
                vek::Vec2::new(81.0,30.0),vek::Vec2::new(80.0,30.0)] }]);
        let radius = Fixed64::from_i32(20);
        let budget = Fixed64::from_i32(240)*Fixed64::from_raw(17);
        let trace = || {
            let mut current = from;
            let mut trace = Vec::new();
            let mut detoured = false;
            for destination in [target,from] {
                for _ in 0..180 {
                    let next = static_step_toward(current,destination,budget,radius,&regions);
                    assert!(!path_hits_regions(current,next,radius,&regions));
                    detoured |= next.y != Fixed64::ZERO;
                    trace.push(next);
                    current = next;
                }
                assert_eq!(current,destination);
            }
            assert!(detoured);
            trace
        };
        assert_eq!(trace(),trace());
        let mut cursor = 1;
        let mut current = from;
        for _ in 0..180 { current = static_advance_route(&[from,target],&mut cursor,current,budget,radius,&regions); }
        assert_eq!(current,target);
        assert_eq!(cursor,2);
        assert_eq!(static_step_toward(from,target,Fixed64::ZERO,radius,&regions),from);
        assert_eq!(static_step_toward(from,target,-budget,radius,&regions),from);
        // An enclosed/blocked endpoint is never snapped into, even with excess budget.
        let inside = SimVec2::new(Fixed64::from_i32(80),Fixed64::ZERO);
        let next = static_step_toward(from,inside,Fixed64::from_i32(1000),radius,&regions);
        assert!(!path_hits_regions(from,next,radius,&regions));
        assert_ne!(next,inside);
    }

    #[test]
    fn grid_rounding_is_deterministic_for_negative_values() {
        assert_eq!(round_fixed_to_grid(0), 0);
        assert_eq!(round_fixed_to_grid(PATH_GRID_RAW / 2 - 1), 0);
        assert_eq!(round_fixed_to_grid(PATH_GRID_RAW / 2), 1);
        assert_eq!(round_fixed_to_grid(-PATH_GRID_RAW / 2), 0);
        assert_eq!(round_fixed_to_grid(-PATH_GRID_RAW / 2 - 1), -1);
    }

    #[test]
    fn neighbors_have_stable_tie_break_order() {
        let ns = neighbors(GridKey { x: 0, y: 0 });
        assert_eq!(
            ns,
            [
                GridKey { x: 1, y: 0 },
                GridKey { x: 0, y: 1 },
                GridKey { x: -1, y: 0 },
                GridKey { x: 0, y: -1 },
                GridKey { x: 1, y: 1 },
                GridKey { x: -1, y: 1 },
                GridKey { x: -1, y: -1 },
                GridKey { x: 1, y: -1 },
            ]
        );
    }

    fn planner_world() -> (World, specs::Entity) {
        let mut world = World::new();
        world.register::<Pos>();
        world.register::<CollisionRadius>();
        world.register::<Faction>();
        world.register::<TAttack>();
        world.register::<IsBuilding>();
        world.register::<RegionBlocker>();
        world.insert(Searcher::default());
        world.insert(BlockedRegions::default());
        world.insert(BuffStore::default());
        let hero = world
            .create_entity()
            .with(Pos(SimVec2::new(Fixed64::ZERO, Fixed64::ZERO)))
            .with(CollisionRadius(Fixed64::from_i32(20)))
            .with(Faction::new(FactionType::Player, 0))
            .build();
        (world, hero)
    }

    fn add_public_blocked_square(world: &mut World, x: i32, y: i32) {
        let half = 30.0;
        world
            .write_resource::<BlockedRegions>()
            .0
            .push(BlockedRegion {
                name: format!("test-{x}-{y}"),
                points: vec![
                    vek::Vec2::new(x as f32 - half, y as f32 - half),
                    vek::Vec2::new(x as f32 + half, y as f32 - half),
                    vek::Vec2::new(x as f32 + half, y as f32 + half),
                    vek::Vec2::new(x as f32 - half, y as f32 + half),
                ],
            });
    }

    fn plan_in_world(world: &World, hero: specs::Entity, from: SimVec2, target: SimVec2) -> Option<SimVec2> {
        let pos=world.read_storage::<Pos>();
        let factions=world.read_storage::<Faction>();
        let attacks=world.read_storage::<TAttack>();
        let radii=world.read_storage::<CollisionRadius>();
        let searcher=world.read_resource::<Searcher>();
        let regions=world.read_resource::<BlockedRegions>();
        let buff_store=world.read_resource::<BuffStore>();
        let is_buildings=world.read_storage::<IsBuilding>();
        plan_next_waypoint(&HeroCommandRead { pos:&pos,factions:&factions,attacks:&attacks,
            radii:&radii,searcher:&searcher,regions:&regions,buff_store:&buff_store,is_buildings:&is_buildings },hero,from,target)
    }

    #[test]
    fn planner_checks_edges_not_only_grid_destinations() {
        let (world,hero)=planner_world();
        world.write_storage::<CollisionRadius>().insert(hero,CollisionRadius(Fixed64::ONE)).unwrap();
        world.write_resource::<BlockedRegions>().0.push(BlockedRegion {
            name:"between-grid-cells".into(),
            points:vec![vek::Vec2::new(20.0,-5.0),vek::Vec2::new(21.0,-5.0),
                vek::Vec2::new(21.0,5.0),vek::Vec2::new(20.0,5.0)],
        });
        let target=SimVec2::new(Fixed64::from_i32(64),Fixed64::ZERO);
        let waypoint=plan_in_world(&world,hero,SimVec2::ZERO,target).expect("detour");
        assert_ne!(waypoint,target);
        assert!(waypoint.y!=Fixed64::ZERO);
        assert!(!path_hits_regions(SimVec2::ZERO,waypoint,Fixed64::ONE,&world.read_resource::<BlockedRegions>()));
        // Same-cell targets may detour, but never bypass the swept terrain.
        let close_target=SimVec2::new(Fixed64::from_i32(30),Fixed64::ZERO);
        let waypoint=plan_in_world(&world,hero,SimVec2::ZERO,close_target).expect("same-cell detour");
        assert_ne!(waypoint,close_target);
        assert!(!path_hits_regions(SimVec2::ZERO,waypoint,Fixed64::ONE,&world.read_resource::<BlockedRegions>()));
        assert!(plan_in_world(&world,hero,SimVec2::ZERO,SimVec2::new(Fixed64::from_i32(20),Fixed64::ZERO)).is_none(),
            "an occupied precise target remains unreachable");
        // Out-of-grid-budget fallback remains fail-closed.
        assert!(plan_in_world(&world,hero,SimVec2::ZERO,SimVec2::new(Fixed64::from_i32(10000),Fixed64::ZERO)).is_none());
    }

    #[test]
    fn path_planner_rejects_partial_route_to_enclosed_or_occupied_target() {
        let (world,hero)=planner_world();
        let target=SimVec2::new(Fixed64::from_i32(128),Fixed64::ZERO);
        world.write_storage::<CollisionRadius>().insert(hero,CollisionRadius(Fixed64::ONE)).unwrap();
        for (x0,y0,x1,y1) in [(100,-41,101,41),(160,-41,161,41),
            (100,-41,161,-40),(100,40,161,41)] {
            world.write_resource::<BlockedRegions>().0.push(BlockedRegion {
                name:"enclosed-target".into(),points:vec![
                    vek::Vec2::new(x0 as f32,y0 as f32),vek::Vec2::new(x1 as f32,y0 as f32),
                    vek::Vec2::new(x1 as f32,y1 as f32),vek::Vec2::new(x0 as f32,y1 as f32)],
            });
        }
        assert!(!path_hits_regions(target,target,Fixed64::ONE,&world.read_resource::<BlockedRegions>()));
        for _ in 0..2 {
            assert_eq!(plan_in_world(&world,hero,SimVec2::ZERO,target),None,
                "the reachable x64 cell cannot stand in for the enclosed goal");
        }
        let occupied=SimVec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
        assert_eq!(plan_in_world(&world,hero,SimVec2::ZERO,occupied),None);
        world.write_resource::<BlockedRegions>().0.clear();
        assert!(plan_in_world(&world,hero,SimVec2::ZERO,target).is_some());
    }

    #[test]
    fn path_planner_routes_around_blocked_grid_cell() {
        let (mut world, hero) = planner_world();
        add_public_blocked_square(&mut world, 64, 0);

        let pos = world.read_storage::<Pos>();
        let factions = world.read_storage::<Faction>();
        let attacks = world.read_storage::<TAttack>();
        let radii = world.read_storage::<CollisionRadius>();
        let searcher = world.read_resource::<Searcher>();
        let regions = world.read_resource::<BlockedRegions>();
        let buff_store = world.read_resource::<BuffStore>();
        let is_buildings = world.read_storage::<IsBuilding>();
        let read = HeroCommandRead {
            pos: &pos,
            factions: &factions,
            attacks: &attacks,
            radii: &radii,
            searcher: &*searcher,
            regions: &*regions,
            buff_store: &*buff_store,
            is_buildings: &is_buildings,
        };

        let waypoint = plan_next_waypoint(
            &read,
            hero,
            SimVec2::new(Fixed64::ZERO, Fixed64::ZERO),
            SimVec2::new(Fixed64::from_i32(128), Fixed64::ZERO),
        )
        .expect("route around blocker");

        assert_ne!(waypoint, SimVec2::new(Fixed64::from_i32(64), Fixed64::ZERO));
        assert!(waypoint.y.raw() != 0);
    }

    #[test]
    fn path_planner_rejects_when_start_is_fully_surrounded() {
        let (mut world, hero) = planner_world();
        for (x, y) in [
            (64, 0),
            (0, 64),
            (-64, 0),
            (0, -64),
            (64, 64),
            (-64, 64),
            (-64, -64),
            (64, -64),
        ] {
            add_public_blocked_square(&mut world, x, y);
        }

        let pos = world.read_storage::<Pos>();
        let factions = world.read_storage::<Faction>();
        let attacks = world.read_storage::<TAttack>();
        let radii = world.read_storage::<CollisionRadius>();
        let searcher = world.read_resource::<Searcher>();
        let regions = world.read_resource::<BlockedRegions>();
        let buff_store = world.read_resource::<BuffStore>();
        let is_buildings = world.read_storage::<IsBuilding>();
        let read = HeroCommandRead {
            pos: &pos,
            factions: &factions,
            attacks: &attacks,
            radii: &radii,
            searcher: &*searcher,
            regions: &*regions,
            buff_store: &*buff_store,
            is_buildings: &is_buildings,
        };

        assert!(plan_next_waypoint(
            &read,
            hero,
            SimVec2::new(Fixed64::ZERO, Fixed64::ZERO),
            SimVec2::new(Fixed64::from_i32(128), Fixed64::ZERO),
        )
        .is_none());
    }
}

#[cfg(test)]
mod acquisition_consumer {
    use crate::comp::HeroCommand;
    use crate::runtime::native::combat_acquisition::test_support::{
        away_from, held, CombatFixture, IndexedBody,
    };

    #[test]
    fn attack_move_hold_reaches_the_enemy_beyond_ten_nearer_allies() {
        let mut fix = CombatFixture::legacy();
        for index in 1..=12 {
            fix.spawn_units(IndexedBody::Creep, index, 1, 100);
        }
        let enemy = fix.spawn_units(IndexedBody::Creep, 24, 2, 100);
        fix.command(HeroCommand::AttackMove { pos: away_from(0) });
        assert_eq!(fix.attack_targets(), vec![enemy]);
        fix.ready();
        let observation = fix.run_command();
        assert!(held(&observation));
        assert!(matches!(
            observation.active,
            Some(HeroCommand::AttackMove { .. })
        ));
    }
}
