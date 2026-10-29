//! The original nobu map (E187, docs/plans/2026-10-28-history-nobu-full-port/nobu-map-comparison.md
//! §5.1-5.3, 5.6, 5.8, 5.9): per-team directed lanes, paired towers, two soldier camps per lane, the
//! general towers, the fort, fountain and hero spawns, the gridnav terrain, the fog extent and the
//! shops, all from templates/nobu_map.lua through moba_maps.lua; and (§5.4, §5.5) the 24 camps
//! and the 1,564 trees.
use super::*;
use crate::runtime::{SimulationDriver, SimulationTickProfile, StateInitializer};

fn nobu() -> &'static omoba_template_ids::MobaMapConst { omoba_template_ids::moba_map_by_name("nobu").expect("nobu map") }

fn config() -> SingleLaneConfig {
    SingleLaneConfig { map_id: Some("nobu".into()), lane_length: Fixed64::from_i32(nobu().lane_length),
        authored_minion_waves: true, ..SingleLaneConfig::default() }
}

fn fixture() -> World {
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    setup_single_lane_match(&mut world, config()).unwrap();
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    world
}

fn pos(world: &World, entity: Entity) -> Vec2 { world.read_storage::<Pos>().get(entity).unwrap().0 }

fn retire(world: &mut World, entity: Entity) {
    record_moba_death(world, entity);
    world.delete_entity(entity).unwrap();
    world.maintain();
}

fn lane(id: &str) -> usize { nobu().lane_index(id).unwrap() }

/// Standing towers of `team` in `lane`, layer by layer.
fn layers(world: &World, lane: usize, team: usize) -> Vec<Vec<Entity>> {
    let state = world.read_resource::<MobaMatch>();
    (0..state.lane_tower_layers[lane].len()).map(|layer| state.layer_towers(lane, layer, team).collect()).collect()
}

fn camps(world: &World, team: usize, lane: usize) -> Vec<Entity> {
    world.read_resource::<MobaMatch>().base_structures.iter()
        .filter(|n| n.team == team && n.lane == lane && n.role == LaneRole::Inhibitor).filter_map(|n| n.entity).collect()
}

fn generals(world: &World, team: usize) -> Vec<(Entity, Vec2)> {
    world.read_resource::<MobaMatch>().base_structures.iter()
        .filter(|n| n.team == team && n.role == LaneRole::NexusTower).filter_map(|n| n.entity.zip(n.position)).collect()
}

fn point((x, y): (i32, i32)) -> Vec2 { map_point((x, y)) }

#[test]
fn nobu_lays_out_every_structure_route_spawn_and_extent_from_the_template() {
    let world = fixture();
    let map = nobu();
    let state = world.read_resource::<MobaMatch>();
    // §5.1: each side walks its own authored route, never the other side's reversed.
    assert_eq!(state.routes.len(), 3);
    for (index, id) in ["top", "mid", "bottom"].into_iter().enumerate() {
        for side in 0..2 {
            let own = map.lanes.iter().find(|l| l.id == id && l.team == Some(side as u8)).unwrap();
            assert_eq!(state.routes[index][side], own.waypoints.iter().copied().map(point).collect::<Vec<_>>(), "{id} {side}");
            let other = map.lanes.iter().find(|l| l.id == id && l.team == Some(1 - side as u8)).unwrap();
            assert_ne!(state.routes[index][side], other.waypoints.iter().rev().copied().map(point).collect::<Vec<_>>());
        }
    }
    drop(state);
    // §5.2: 16 towers per side at the template positions, two per tier (side lanes 3, mid 2).
    for side in 0..2 {
        let mut placed = Vec::new();
        for (lane_index, id) in ["top", "mid", "bottom"].into_iter().enumerate() {
            let standing = layers(&world, lane_index, side);
            assert_eq!(standing.len(), if id == "mid" { 2 } else { 3 }, "{id}");
            for (layer, towers) in standing.iter().enumerate() {
                assert_eq!(towers.len(), 2, "two towers in {id} tier {}", layer + 1);
                for tower in towers {
                    placed.push((id, layer as u8 + 1, pos(&world, *tower)));
                }
            }
        }
        let mut want: Vec<_> = map.towers.iter().filter(|t| usize::from(t.team) == side).map(|t| (t.lane, t.tier, point(t.position))).collect();
        placed.sort_by_key(|row| (row.0, row.1, row.2.x.raw(), row.2.y.raw()));
        want.sort_by_key(|row| (row.0, row.1, row.2.x.raw(), row.2.y.raw()));
        assert_eq!(placed, want);
        assert_eq!(placed.len(), 16);
        // §5.3: two soldier camps per lane and five general towers at their positions.
        for id in ["top", "mid", "bottom"] {
            let mut got: Vec<_> = camps(&world, side, lane(id)).into_iter().map(|e| pos(&world, e)).collect();
            let mut want: Vec<_> = map.barracks.iter().filter(|b| usize::from(b.team) == side && b.lane == id).map(|b| point(b.position)).collect();
            got.sort_by_key(|p| (p.x.raw(), p.y.raw()));want.sort_by_key(|p| (p.x.raw(), p.y.raw()));
            assert_eq!(got, want);assert_eq!(got.len(), 2);
        }
        let got: Vec<_> = generals(&world, side).into_iter().map(|(e, at)| { assert_eq!(pos(&world, e), at); at }).collect();
        let want: Vec<_> = map.general_towers.iter().filter(|g| usize::from(g.team) == side).map(|g| point(g.position)).collect();
        assert_eq!(got, want);
        // The fort, and heroes at the authored spawns.
        let state = world.read_resource::<MobaMatch>();
        assert_eq!(pos(&world, state.bases[side].unwrap()), point(omoba_template_ids::moba_team_point(map.bases, side).unwrap()));
        let hero = state.heroes.iter().find(|slot| slot.side == side).unwrap().entity.unwrap();
        assert_eq!(pos(&world, hero), point(map.hero_spawns.iter().find(|p| usize::from(p.team) == side).unwrap().position));
    }
    // §5.6 / §5.5: the 305 gridnav rectangles and the 1,564 tree squares are the collision; the
    // rectangles and one VisionTree per tree are the vision occluders.
    let trees = map.trees();
    assert_eq!((trees.trees.len(), trees.radius), (1564, 64));
    assert_eq!(world.read_resource::<BlockedRegions>().0.len(), 305 + 1564);
    assert_eq!(world.read_resource::<crate::runtime::VisionOccluderSet>().0.len(), 305 + 1564);
    // §5.4: the 24 camps stand from the start (the bosses wait for their first appearance).
    let state = world.read_resource::<MobaMatch>();
    let standing: usize = state.jungle_camps.iter().filter(|c| c.definition.unit_respawn).map(|c| c.entities().count()).sum();
    assert_eq!(standing, 68);
    assert!(state.jungle_camps.iter().filter(|c| c.definition.has_tag(omoba_template_ids::MobaUnitTag::Boss)).all(|c| c.entity.is_none()));
    drop(state);
    // §5.8: fog covers the authored bounds; communication points are the bounds.
    let bounds = map.bounds.unwrap();
    let fog = world.read_resource::<crate::runtime::TeamVisibilityRuntime>().fog_geometry.unwrap();
    assert_eq!((fog.origin_x_raw, fog.origin_y_raw), (i64::from(bounds.min.0) * 1024, i64::from(bounds.min.1) * 1024));
    let span = i64::from((bounds.max.0 - bounds.min.0).max(bounds.max.1 - bounds.min.1)) * 1024;
    assert!(i64::from(fog.columns.max(fog.rows) - 1) * fog.cell_size_raw <= span && i64::from(fog.columns.max(fog.rows)) * fog.cell_size_raw > span);
    let state = world.read_resource::<MobaMatch>();
    assert!(state.communication_point_allowed(point(bounds.min)) && state.communication_point_allowed(point(bounds.max)));
    assert!(!state.communication_point_allowed(point((bounds.max.0 + 1, bounds.max.1))));
}

#[test]
fn existing_maps_keep_their_fog_geometry_and_have_no_explicit_layout() {
    // §5.8: maps without bounds keep the extent of their lanes, camps and terrain.
    for id in ["three_lane_training", "three_lane_layered_training", "three_lane_quick_training"] {
        let map = omoba_template_ids::moba_map_by_name(id).unwrap();
        assert!(map.bounds.is_none() && map.towers.is_empty() && map.barracks.is_empty() && map.shops.is_empty() && !map.has_team_lanes());
        let derived = map.derived_extent().unwrap();
        let fog = crate::runtime::fog_grid::FogGridGeometry::from_map(map).unwrap();
        assert_eq!((fog.origin_x_raw, fog.origin_y_raw), (i64::from(derived.min.0) * 1024, i64::from(derived.min.1) * 1024));
        // The standard maps span 0..4800: 63 x 63 cells of 76.2 units.
        assert_eq!((derived.min, derived.max, fog.columns, fog.rows, fog.cell_size_raw), ((0, 0), (4800, 4800), 63, 63, (4800 * 1024 + 62) / 63));
        // §5.4 / §5.5: no trees, no sided or per-unit camps; collision and vision are the terrain.
        assert_eq!(map.trees(), omoba_template_ids::MobaTreesConst::NONE);
        assert!(!map.has_sided_camps() && map.jungle_camps.iter().all(|c| !c.unit_respawn && c.leader_offset == (0, 0) && c.unit == c.id));
        assert_eq!(crate::runtime::moba_map_layout::compiled_blocked_regions(map).0.len(), map.terrain.len());
        assert_eq!(crate::runtime::moba_map_layout::compiled_vision_occluders(map).0.len(), map.terrain.len());
    }
}

#[test]
fn paired_towers_open_the_next_tier_only_when_both_of_the_outer_tier_fell() {
    let mut world = fixture();
    let top = lane("top");
    let standing = layers(&world, top, 1);
    let ([a, b], [c, d]) = ([standing[0][0], standing[0][1]], [standing[1][0], standing[1][1]]);
    assert!(moba_damage_allowed(&world, a) && moba_damage_allowed(&world, b), "both outer towers are open");
    assert!(!moba_damage_allowed(&world, c) && !moba_damage_allowed(&world, d), "the second tier waits");
    // Plating and building rewards see both outer towers as the outer layer.
    {let state = world.read_resource::<MobaMatch>();
        assert_eq!((state.tower_layer_of(top, 1, a), state.tower_layer_of(top, 1, b), state.tower_layer_of(top, 1, c)), (Some(0), Some(0), Some(1)));}
    let digest = single_lane_replay_digest(&world);
    retire(&mut world, a);
    assert_ne!(single_lane_replay_digest(&world), digest, "the partner slots are replay state");
    assert!(moba_damage_allowed(&world, b), "the partner stays open");
    assert!(!moba_damage_allowed(&world, c) && !moba_damage_allowed(&world, d), "one of the pair down is not enough");
    assert_eq!(world.read_resource::<MobaMatch>().lane_towers[top][1], Some(b));
    retire(&mut world, b);
    assert!(moba_damage_allowed(&world, c) && moba_damage_allowed(&world, d), "both down opens the next tier");
    // The camps of the lane open only when every tower of the lane fell.
    let lane_camps = camps(&world, 1, top);
    assert!(lane_camps.iter().all(|e| !moba_damage_allowed(&world, *e)));
    for tower in layers(&world, top, 1).concat() { retire(&mut world, tower); }
    assert!(lane_camps.iter().all(|e| moba_damage_allowed(&world, *e)));
    // The other side and the other lanes are untouched.
    assert_eq!(layers(&world, top, 0).concat().len(), 6);
    assert_eq!(layers(&world, lane("mid"), 1).concat().len(), 4);
}

/// (team, lane, id, max hp, armor) of a fresh wave `wave` spawned now.
fn wave(world: &mut World, wave: u32) -> Vec<(u8, usize, String, i32, Fixed64)> {
    let before: BTreeSet<Entity> = world.read_resource::<MobaMatch>().units.values().filter(|u| u.role == LaneRole::Creep).map(|u| u.entity).collect();
    let mut state = world.remove::<MobaMatch>().unwrap();
    state.waves = wave - 1;
    state.spawn_wave(world);
    world.insert(state);
    let state = world.read_resource::<MobaMatch>();
    state.units.values().filter(|u| u.role == LaneRole::Creep && !before.contains(&u.entity)).map(|u| {
        let unit = world.read_storage::<Unit>().get(u.entity).unwrap().clone();
        (u.team, u.lane, unit.id.clone(), unit.max_hp, world.read_storage::<CProperty>().get(u.entity).unwrap().def_physic)
    }).collect()
}

#[test]
fn the_first_soldier_camp_of_a_lane_gives_x1_3_and_the_second_the_full_reinforcement_waves_and_intensify_loss() {
    // chubing.lua team_broken == 1: every group x1.3; == 2: infantry / archers x1.3, gunners /
    // cavalry x2, the dummy.lua extra waves, and the general towers lose modifier_intensify.
    let mut world = fixture();
    let map = nobu();
    let group = |id: &str| *map.minion_wave.iter().find(|m| m.id == id).unwrap();
    let expect = |m: omoba_template_ids::MobaMinionConst, wave: u32, percent: u32| {
        let (hp, _) = m.grown(wave);
        ((i64::from(hp) * i64::from(percent) / 100) as i32, Fixed64::from_i32((i64::from(m.armor) * i64::from(percent) / 100) as i32))
    };
    let bottom = lane("bottom");
    for tower in layers(&world, bottom, 1).concat() { retire(&mut world, tower); }
    let [first, second] = camps(&world, 1, bottom)[..] else { panic!("two camps") };
    let intensified = |world: &World, team: usize| world.read_resource::<MobaMatch>().nexus_tower_intensified(team);
    retire(&mut world, first);
    {let state = world.read_resource::<MobaMatch>();
        assert_eq!(state.barracks_fallen(1, bottom), (1, 2));
        assert!(state.pending_fall_reinforcements().is_empty(), "no extra waves at the first camp");}
    assert!(intensified(&world, 1), "the general towers keep the intensify at the first camp");
    for (team, lane_index, id, hp, armor) in wave(&mut world, 6) {
        let percent = if team == 0 && lane_index == bottom { group(&id).first_barracks_percent } else { 100 };
        assert_eq!((hp, armor), expect(group(&id), 6, percent), "team {team} lane {lane_index} {id}");
    }
    assert!(map.minion_wave.iter().all(|m| m.first_barracks_percent == 130));
    retire(&mut world, second);
    assert_eq!(world.read_resource::<MobaMatch>().barracks_fallen(1, bottom), (2, 2));
    assert!(!intensified(&world, 1) && intensified(&world, 0), "the fallen side loses the intensify");
    let pending = world.read_resource::<MobaMatch>().pending_fall_reinforcements().to_vec();
    assert_eq!(pending.len(), map.base_structures.unwrap().fall_reinforcements.len());
    assert!(pending.iter().all(|row| row.team == 0 && row.lane == bottom), "the attackers' extra waves in the lane");
    for (team, lane_index, id, hp, armor) in wave(&mut world, 11) {
        let percent = if team == 0 && lane_index == bottom { group(&id).lane_down_percent } else { 100 };
        assert_eq!((hp, armor), expect(group(&id), 11, percent), "team {team} lane {lane_index} {id}");
    }
}

#[test]
fn the_fort_opens_with_a_fallen_camp_and_the_general_towers_beside_it_which_never_return() {
    // war_magic.lua:838-861: the fort is invulnerable while a com_general stands within 500.
    let mut world = fixture();
    let map = nobu();
    let fort = point(omoba_template_ids::moba_team_point(map.bases, 1).unwrap());
    let all = generals(&world, 1);
    let radius = Fixed64::from_i32(map.base_structures.unwrap().general_guard_radius);
    let (guards, far): (Vec<(Entity, Vec2)>, Vec<(Entity, Vec2)>) = all.iter().copied()
        .partition(|(_, at)| (*at - fort).length_squared() <= radius * radius);
    assert_eq!((guards.len(), far.len()), (2, 3), "the two com_general beside the fort guard it");
    let base = world.read_resource::<MobaMatch>().bases[1].unwrap();
    assert!(all.iter().all(|(e, _)| !moba_damage_allowed(&world, *e)), "general towers wait for a soldier camp");
    let mid = lane("mid");
    for tower in layers(&world, mid, 1).concat() { retire(&mut world, tower); }
    let camp = camps(&world, 1, mid)[0];
    retire(&mut world, camp);
    assert!(all.iter().all(|(e, _)| moba_damage_allowed(&world, *e)), "one fallen camp opens the general towers");
    assert!(!moba_damage_allowed(&world, base));
    retire(&mut world, guards[0].0);
    assert!(!moba_damage_allowed(&world, base), "one guard still stands");
    retire(&mut world, guards[1].0);
    assert!(moba_damage_allowed(&world, base), "the far general towers do not guard the fort");
    // Never rebuilt: soldier camps and general towers of the original stay down.
    let mut state = world.remove::<MobaMatch>().unwrap();
    assert!(state.base_structures.iter().filter(|n| n.entity.is_none()).all(|n| n.respawn_at().is_none()));
    state.elapsed = Fixed64::from_i32(7200);
    state.tick_base_structures(&mut world);
    assert_eq!(state.base_structures.iter().filter(|n| n.entity.is_none()).count(), 3);
    world.insert(state);
}

#[test]
fn shops_sell_within_the_purchase_radius_of_a_home_or_side_shop_the_side_may_use() {
    let mut world = fixture();
    let map = nobu();
    let radius = map.shop_radius;
    let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let at = |world: &mut World, (x, y): (i32, i32)| {
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = point((x, y));
        let state = world.remove::<MobaMatch>().unwrap();
        let open = super::super::shop::owner_shop_availability(world, &state, 1).0;
        world.insert(state);
        open
    };
    let fountain = omoba_template_ids::moba_team_point(map.fountains, 0).unwrap();
    let enemy_fountain = omoba_template_ids::moba_team_point(map.fountains, 1).unwrap();
    let fort = omoba_template_ids::moba_team_point(map.bases, 0).unwrap();
    assert!(at(&mut world, fountain), "the home shop at the own fountain");
    assert!(at(&mut world, (fountain.0 - radius, fountain.1)), "at the radius");
    assert!(!at(&mut world, (fountain.0 - radius - 1, fountain.1)), "beyond it");
    assert!(!at(&mut world, fort), "the fort is not a shop on this map");
    assert!(!at(&mut world, enemy_fountain), "the enemy's home shop is theirs");
    let open = map.shops.iter().find(|s| s.team.is_none() && s.kind == omoba_template_ids::MobaShopKind::Home).unwrap();
    assert!(at(&mut world, open.position), "a home shop without a team serves either side");
    // Secret shops sell only items marked secret; the catalog has none yet.
    for secret in map.shops.iter().filter(|s| s.kind == omoba_template_ids::MobaShopKind::Secret) {
        assert!(!at(&mut world, secret.position));
    }
    // The bots shop and recover at the same fountain with the same radius.
    let state = world.read_resource::<MobaMatch>();
    assert_eq!(state.bot_home(0, 0), point(fountain));
    assert_eq!(super::super::shop::shop_radius(&state), Fixed64::from_i32(radius));
}

#[test]
fn nobu_minions_follow_their_own_route_and_report_the_tick_cost() {
    // §5.1 acceptance: every minion stays on its own side's route while walking it. §5.6: the
    // tick cost with the 305 terrain rectangles is printed (the release headless batch measures it).
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    setup_single_lane_match(&mut world, SingleLaneConfig { warmup: Fixed64::ZERO, ..config() }).unwrap();
    world.insert(crate::runtime::ScriptRegistry::new());
    let mut driver = SimulationDriver::from_world(&mut world, SimulationTickProfile::Production60Hz).unwrap();
    let mut reached: BTreeMap<Entity, usize> = BTreeMap::new();
    let started = std::time::Instant::now();
    // 30 s of play: the first waves walk about 9,000 units before any lane fight.
    for _ in 0..1800 {
        driver.step(&mut world, []).unwrap();
        let state = world.read_resource::<MobaMatch>();
        for unit in state.units.values().filter(|u| u.role == LaneRole::Creep && u.target.is_none()) {
            let route = &state.routes[unit.lane][unit.team as usize];
            let at = pos(&world, unit.entity);
            // Distance to the own route polyline.
            let off = route.windows(2).map(|s| {
                let (a, b) = (s[0], s[1]);
                let ab = b - a;
                let t = ((at - a).dot(ab) / ab.length_squared()).max(Fixed64::ZERO).min(Fixed64::ONE);
                (a + ab * t - at).length()
            }).min().unwrap();
            assert!(off <= Fixed64::from_i32(150), "creep of side {} lane {} {} off its own route", unit.team, unit.lane, off.to_f32_for_render());
            let entry = reached.entry(unit.entity).or_default();
            *entry = (*entry).max(unit.next_waypoint);
        }
    }
    let per_tick = started.elapsed() / 1800;
    eprintln!("nobu: {:?} per 60 Hz tick (debug build, 30 s of play)", per_tick);
    assert!(reached.values().filter(|n| **n >= 2).count() >= 20, "minions passed their own waypoints: {reached:?}");
    let sides: BTreeSet<_> = world.read_resource::<MobaMatch>().units.values().filter(|u| u.role == LaneRole::Creep).map(|u| u.team).collect();
    assert_eq!(sides.len(), 2);
}

/// E187 §5.6 / §5.8 cost breakdown (run with --release --ignored --nocapture): ticks of the first
/// 40 s of play without bots, on the standard map and on nobu with and without its terrain
/// collision and vision geometry.
#[test]
#[ignore]
fn nobu_tick_cost_breakdown() {
    fn run(map: &str, collision: bool, vision: bool) -> std::time::Duration {
        let map_const = omoba_template_ids::moba_map_by_name(map).unwrap();
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(crate::runtime::TowerTemplateRegistry::default());
        setup_single_lane_match(&mut world, SingleLaneConfig { warmup: Fixed64::ZERO, map_id: Some(map.into()),
            lane_length: Fixed64::from_i32(map_const.lane_length), authored_minion_waves: true, ..SingleLaneConfig::default() }).unwrap();
        if !collision { *world.write_resource::<BlockedRegions>() = BlockedRegions::default(); }
        if !vision { *world.write_resource::<crate::runtime::VisionOccluderSet>() = Default::default(); }
        world.insert(crate::runtime::ScriptRegistry::new());
        let mut driver = SimulationDriver::from_world(&mut world, SimulationTickProfile::Production60Hz).unwrap();
        let started = std::time::Instant::now();
        for _ in 0..2400 { driver.step(&mut world, []).unwrap(); }
        started.elapsed() / 2400
    }
    for (map, collision, vision) in [("three_lane_training", true, true), ("nobu", true, true), ("nobu", false, true), ("nobu", true, false), ("nobu", false, false)] {
        eprintln!("{map} collision={collision} vision={vision}: {:?} per tick", run(map, collision, vision));
    }
}

/// E187 §5.5: a tree blocks movement as the square of half-extent 64 around its point (the 2x2
/// gridnav cells a Dota tree holds) and vision as a VisionTree circle of radius 64. Compared with
/// the terrain alone, the trees close the ground and the sight lines through their points.
#[test]
fn nobu_trees_block_movement_and_vision_where_they_stand() {
    let world = fixture();
    let trees = nobu().trees();
    let regions = world.read_resource::<BlockedRegions>().clone();
    let terrain_only = BlockedRegions(regions.0[..305].to_vec());
    let occluders = world.read_resource::<crate::runtime::VisionOccluderSet>().clone();
    let walls = crate::runtime::VisionOccluderSet(occluders.0.iter().filter(|o| matches!(o, crate::runtime::VisionOccluder::Terrain(_))).cloned().collect());
    assert_eq!(walls.0.len(), 305);
    let (committed, walls) = (occluders.committed(), walls.committed());
    let hits = |r: &BlockedRegions, a: Vec2, b: Vec2, radius: i32| crate::tick::hero_move_tick::path_hits_regions(a, b, Fixed64::from_i32(radius), r);
    let v = |x: i32, y: i32| Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
    let (mut movement, mut vision) = (0, 0);
    for (index, tree) in trees.trees.iter().enumerate() {
        let c = point(tree.position);
        // The compiled square and circle.
        let region = &regions.0[305 + index];
        assert_eq!(region.name, crate::runtime::moba_map_layout::tree_region_name(index));
        assert_eq!((region.points[0].x as i32, region.points[0].y as i32, region.points[2].x as i32, region.points[2].y as i32),
            (tree.position.0 - 64, tree.position.1 - 64, tree.position.0 + 64, tree.position.1 + 64));
        assert!(hits(&regions, c + v(63, 63), c + v(63, 63), 0) && hits(&regions, c - v(63, 63), c - v(63, 63), 0));
        // A hero body cannot stand on a tree point, and a line through it is closed.
        assert!(hits(&regions, c, c, 30));
        if !hits(&terrain_only, c - v(150, 0), c + v(150, 0), 30) { movement += 1; assert!(hits(&regions, c - v(150, 0), c + v(150, 0), 30)); }
        if walls.line_of_sight(c - v(150, 0), c + v(150, 0)) == crate::runtime::LosResult::Clear {
            vision += 1;
            assert_eq!(committed.line_of_sight(c - v(150, 0), c + v(150, 0)), crate::runtime::LosResult::Blocked);
        }
        // The circle: a line 63 off the point still crosses it.
        assert_eq!(committed.line_of_sight(c + v(-150, 63), c + v(150, 63)), crate::runtime::LosResult::Blocked);
    }
    assert!(movement > 1000 && vision > 1000, "trees close ground and sight the terrain leaves open: {movement} / {vision}");
    // The Wave B view (fog sample, unit disclosure) reads the same occluders.
    let view = crate::runtime::build_wave_b_read_view(&world, 1);
    assert_eq!(view.vision_occluders.len(), 305 + 1564);
    let lone = trees.trees.iter().map(|t| point(t.position)).find(|&c| walls.line_of_sight(c - v(300, 0), c + v(300, 0)) == crate::runtime::LosResult::Clear
        && walls.line_of_sight(c + v(-300, 70), c + v(300, 70)) == crate::runtime::LosResult::Clear
        && trees.trees.iter().filter(|o| (point(o.position) - c).length_squared() < Fixed64::from_i32(400 * 400)).count() == 1).expect("a lone tree");
    assert_eq!(view.vision_occluders.line_of_sight(lone - v(300, 0), lone + v(300, 0)), crate::runtime::LosResult::Blocked);
    assert_eq!(view.vision_occluders.line_of_sight(lone + v(-300, 70), lone + v(300, 70)), crate::runtime::LosResult::Clear);
}

/// E187 §5.5: with the trees most long lines across nobu are closed. Bot travel admits a
/// destination beyond the search span whose point is free, and a nearer one with a path in the
/// widened search box (taken leg by leg); with a path in the movement box the answer is the old one.
#[test]
fn bot_travel_reaches_every_own_camp_and_home_across_the_nobu_forest() {
    use crate::tick::hero_command_tick::{static_next_waypoint, static_travel_reachable};
    let regions = crate::runtime::moba_map_layout::compiled_blocked_regions(nobu());
    let radius = Fixed64::from_i32(30);
    let mut refused_before = 0;
    for side in 0..2u8 {
        let home = point(omoba_template_ids::moba_team_point(nobu().fountains, usize::from(side)).unwrap());
        for camp in nobu().jungle_camps.iter().filter(|c| c.side == Some(side)) {
            let center = point(camp.position);
            assert!(static_travel_reachable(home, center, radius, &regions), "{} from side {side}'s fountain", camp.id);
            assert!(static_travel_reachable(center, home, radius, &regions), "home from {}", camp.id);
            if static_next_waypoint(home, center, radius, &regions).is_none() { refused_before += 1; }
        }
    }
    assert!(refused_before > 0, "the old rule refused some trips the move orders can make");
    // A point inside a tree is no trip, near or beyond the span.
    let tree = point(nobu().trees().trees[0].position);
    for offset in [400, 7000] {
        let from = tree + Vec2::new(Fixed64::from_i32(offset), Fixed64::ZERO);
        assert!(!static_travel_reachable(from, tree, radius, &regions), "{offset}");
    }
    // The training maps are smaller than the span: travel is exactly the path rule.
    let training = crate::runtime::moba_map_layout::compiled_blocked_regions(omoba_template_ids::moba_map_by_name("three_lane_training").unwrap());
    for (a, b) in [((0, 0), (4800, 4800)), ((100, 2400), (2400, 100)), ((420, 1700), (900, 2100))] {
        assert_eq!(static_travel_reachable(point(a), point(b), radius, &training), static_next_waypoint(point(a), point(b), radius, &training).is_some());
    }
}

#[test]
fn bucketed_terrain_index_answers_exactly_like_a_full_scan_on_the_nobu_gridnav() {
    // E187 §5.6: the 305 rectangles go through a bucketed broad phase; every answer must equal the
    // exact swept-circle test over every polygon (segments short and long, radii 0..beyond bounds).
    use crate::tick::hero_command_tick::TerrainIndex;
    let regions = crate::runtime::moba_map_layout::compiled_blocked_regions(nobu());
    let polygons: Vec<Vec<Vec2>> = regions.0.iter().map(|r| r.points.iter().map(|p| Vec2::new(
        Fixed64::from_raw((f64::from(p.x) * 1024.0).round() as i64), Fixed64::from_raw((f64::from(p.y) * 1024.0).round() as i64))).collect()).collect();
    let full = |from, to, radius| polygons.iter().any(|polygon| omoba_sim::terrain::swept_circle_hits_polygon(from, to, radius, polygon));
    let index = TerrainIndex::new(&regions);
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = |range: i64| { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; (seed % range as u64) as i64 };
    let (mut hits, mut clear) = (0, 0);
    for case in 0..20_000 {
        let from = Vec2::new(Fixed64::from_i32(next(18_000) as i32 - 9_000), Fixed64::from_i32(next(18_000) as i32 - 9_000));
        let span = [64, 600, 6_000, 20_000][case % 4];
        let to = from + Vec2::new(Fixed64::from_i32((next(2 * span) - span) as i32), Fixed64::from_i32((next(2 * span) - span) as i32));
        let radius = Fixed64::from_i32([0, 20, 30, 100, 700][case % 5]);
        let want = full(from, to, radius);
        assert_eq!(index.hits(from, to, radius), want, "{from:?} -> {to:?} r {radius:?}");
        assert_eq!(crate::tick::hero_move_tick::path_hits_regions(from, to, radius, &regions), want);
        if want { hits += 1 } else { clear += 1 }
    }
    assert!(hits > 1000 && clear > 1000, "both answers exercised: {hits} / {clear}");
    // Out-of-range radius and points take the full scan.
    let far = Vec2::new(Fixed64::from_raw(1 << 30), Fixed64::ZERO);
    assert_eq!(index.hits(Vec2::ZERO, far, Fixed64::from_i32(20)), full(Vec2::ZERO, far, Fixed64::from_i32(20)));
    assert_eq!(index.hits(Vec2::ZERO, Vec2::ZERO, Fixed64::from_raw(-1)), full(Vec2::ZERO, Vec2::ZERO, Fixed64::from_raw(-1)));
    // The shared index follows a terrain change.
    let one = BlockedRegions(regions.0[..40].to_vec());
    let segment = (Vec2::new(Fixed64::from_i32(-9000), Fixed64::from_i32(-9000)), Vec2::new(Fixed64::from_i32(9000), Fixed64::from_i32(9000)));
    let want = |r: &BlockedRegions| TerrainIndex::new(r).hits(segment.0, segment.1, Fixed64::from_i32(20));
    assert_eq!(TerrainIndex::cached(&regions).hits(segment.0, segment.1, Fixed64::from_i32(20)), want(&regions));
    assert_eq!(TerrainIndex::cached(&one).hits(segment.0, segment.1, Fixed64::from_i32(20)), want(&one));
}
