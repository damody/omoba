use crate::comp::*;
use crate::tick::hero_move_tick::hits_any;
use omoba_core::runtime::ability_runtime::BuffStore;
use omoba_sim::{fixed::SCALE, Fixed64, Vec2 as SimVec2};
use specs::{shred, Entities, Join, Read, ReadStorage, SystemData, WriteStorage};
use std::collections::{BTreeMap, VecDeque};

const PATH_GRID_WORLD_UNITS: i64 = 64;
const PATH_GRID_RAW: i64 = PATH_GRID_WORLD_UNITS * SCALE;
const PATH_GRID_MARGIN: i64 = 8;
const MAX_PATH_GRID_SPAN: i64 = 96;

#[derive(SystemData)]
pub struct HeroCommandData<'a> {
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
}

#[derive(Clone, Debug)]
pub(crate) struct HeroCommandDecision {
    entity: specs::Entity,
    queue: HeroCommandQueue,
    next_waypoint: Option<SimVec2>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = HeroCommandData<'a>;

    const NAME: &'static str = "hero_command";

    fn run(_job: &mut Job<Self>, mut data: Self::SystemData) {
        let arrive_eps = Fixed64::from_raw(768);
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

        let mut decisions: Vec<_> = (&data.entities, &data.heroes, &data.pos, &data.queues)
            .join()
            .map(|(entity, _hero, pos, queue)| {
                decide_hero_command(&read, entity, pos.0, queue.clone(), arrive_eps)
            })
            .collect();
        decisions.sort_by_key(|decision| decision.entity.id());

        for decision in decisions {
            if let Some(queue) = data.queues.get_mut(decision.entity) {
                *queue = decision.queue;
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

fn decide_hero_command(
    data: &HeroCommandRead<'_, '_>,
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
                if attack_move_should_hold(data, entity, pos) {
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
                let Some(step) =
                    resolve_attack_target_command(data, entity, pos, target, chase_origin)
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

fn attack_move_should_hold(
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    pos: SimVec2,
) -> bool {
    if matches!(
        data.attacks.get(entity).map(|a| a.attack_phase),
        Some(AttackSequencePhase::Windup | AttackSequencePhase::Backswing)
    ) {
        return true;
    }
    let Some(range) = effective_attack_range(data, entity) else {
        return false;
    };
    find_hostile_in_range(data, entity, pos, range).is_some()
}

fn resolve_attack_target_command(
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    pos: SimVec2,
    target: specs::Entity,
    chase_origin: Option<SimVec2>,
) -> Option<AttackTargetStep> {
    let target_pos = data.pos.get(target)?.0;
    let source_faction = data.factions.get(entity)?;
    let target_faction = data.factions.get(target)?;
    if !source_faction.is_hostile_to(target_faction) {
        return None;
    }
    if matches!(
        data.attacks.get(entity).map(|a| a.attack_phase),
        Some(AttackSequencePhase::Windup | AttackSequencePhase::Backswing)
    ) {
        return Some(AttackTargetStep::Hold);
    }
    let attack_range = effective_attack_range(data, entity)?;
    let distance = (target_pos - pos).length();
    if distance <= attack_range {
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

fn find_hostile_in_range(
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    pos: SimVec2,
    attack_range: Fixed64,
) -> Option<specs::Entity> {
    let source_faction = data.factions.get(entity)?;
    let pos_vek = vek::Vec2::new(pos.x.to_f32_for_render(), pos.y.to_f32_for_render());
    let attack_range_f = attack_range.to_f32_for_render();
    let search_range_f = attack_range_f + 50.0;
    let (mut creep_targets, _) =
        data.searcher
            .creep
            .search_nn_two_radii(pos_vek, attack_range_f, search_range_f, 10);
    let (tower_targets, _) =
        data.searcher
            .tower
            .search_nn_two_radii(pos_vek, attack_range_f, search_range_f, 10);
    creep_targets.extend(tower_targets);
    creep_targets.sort_by(|a, b| {
        a.dis
            .partial_cmp(&b.dis)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.e.id().cmp(&b.e.id()))
    });
    let range_sq = attack_range_f * attack_range_f;
    creep_targets
        .into_iter()
        .filter(|target| target.dis <= range_sq)
        .find(|target| {
            data.factions
                .get(target.e)
                .map(|f| source_faction.is_hostile_to(f))
                .unwrap_or(false)
        })
        .map(|target| target.e)
}

fn plan_next_waypoint(
    data: &HeroCommandRead<'_, '_>,
    entity: specs::Entity,
    from: SimVec2,
    target: SimVec2,
) -> Option<SimVec2> {
    let start = grid_key(from);
    let goal = grid_key(target);
    if start == goal {
        return (!position_blocked(data, entity, target)).then_some(target);
    }

    let dx = (goal.x - start.x).abs();
    let dy = (goal.y - start.y).abs();
    if dx > MAX_PATH_GRID_SPAN || dy > MAX_PATH_GRID_SPAN {
        return Some(target);
    }

    let min_x = start.x.min(goal.x) - PATH_GRID_MARGIN;
    let max_x = start.x.max(goal.x) + PATH_GRID_MARGIN;
    let min_y = start.y.min(goal.y) - PATH_GRID_MARGIN;
    let max_y = start.y.max(goal.y) + PATH_GRID_MARGIN;

    let mut open = VecDeque::new();
    let mut parent: BTreeMap<GridKey, GridKey> = BTreeMap::new();
    let mut best = start;
    let mut best_score = key_target_distance_sq(start, target);
    parent.insert(start, start);
    open.push_back(start);

    while let Some(cur) = open.pop_front() {
        if cur == goal {
            best = cur;
            break;
        }
        for next in neighbors(cur) {
            if next.x < min_x || next.x > max_x || next.y < min_y || next.y > max_y {
                continue;
            }
            if parent.contains_key(&next) {
                continue;
            }
            if next != goal && grid_key_blocked(data, entity, next) {
                continue;
            }
            if next == goal && position_blocked(data, entity, target) {
                continue;
            }
            parent.insert(next, cur);
            let score = key_target_distance_sq(next, target);
            if score < best_score || (score == best_score && next < best) {
                best = next;
                best_score = score;
            }
            open.push_back(next);
        }
    }

    if best == start {
        return None;
    }

    let mut cur = best;
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

fn position_blocked(data: &HeroCommandRead<'_, '_>, entity: specs::Entity, pos: SimVec2) -> bool {
    let radius = data
        .radii
        .get(entity)
        .map(|r| r.0)
        .unwrap_or_else(|| Fixed64::from_i32(20));
    hits_any(pos, radius, data.searcher, data.radii, entity, data.regions)
}

fn grid_key_blocked(data: &HeroCommandRead<'_, '_>, entity: specs::Entity, key: GridKey) -> bool {
    position_blocked(data, entity, grid_pos(key))
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

fn key_target_distance_sq(key: GridKey, target: SimVec2) -> i128 {
    let x = i128::from(key.x) * i128::from(PATH_GRID_RAW);
    let y = i128::from(key.y) * i128::from(PATH_GRID_RAW);
    let dx = x - i128::from(target.x.raw());
    let dy = y - i128::from(target.y.raw());
    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
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

    fn add_region_blocker(world: &mut World, x: i32, y: i32) -> specs::Entity {
        world
            .create_entity()
            .with(Pos(SimVec2::new(
                Fixed64::from_i32(x),
                Fixed64::from_i32(y),
            )))
            .with(CollisionRadius(Fixed64::from_i32(40)))
            .with(RegionBlocker)
            .build()
    }

    fn rebuild_region_index(world: &mut World, blockers: &[specs::Entity]) {
        let items: Vec<_> = {
            let positions = world.read_storage::<Pos>();
            blockers
                .iter()
                .filter_map(|entity| {
                    positions.get(*entity).map(|pos| {
                        (
                            *entity,
                            vek::Vec2::new(
                                pos.0.x.to_f32_for_render(),
                                pos.0.y.to_f32_for_render(),
                            ),
                        )
                    })
                })
                .collect()
        };
        world
            .write_resource::<Searcher>()
            .region
            .rebuild_from(items);
    }

    #[test]
    fn path_planner_routes_around_blocked_grid_cell() {
        let (mut world, hero) = planner_world();
        let blocker = add_region_blocker(&mut world, 64, 0);
        rebuild_region_index(&mut world, &[blocker]);

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
        let blockers = [
            add_region_blocker(&mut world, 64, 0),
            add_region_blocker(&mut world, 0, 64),
            add_region_blocker(&mut world, -64, 0),
            add_region_blocker(&mut world, 0, -64),
            add_region_blocker(&mut world, 64, 64),
            add_region_blocker(&mut world, -64, 64),
            add_region_blocker(&mut world, -64, -64),
            add_region_blocker(&mut world, 64, -64),
        ];
        rebuild_region_index(&mut world, &blockers);

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
