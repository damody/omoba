//! v3 `ai = "chase_target"` (E406): A29T 兀鷹, B10W 亡靈 and C16R 行空魂 share the source's
//! Exorcism program (vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:126-404). The authority moves
//! each chase summon every tick from its template's `chase` table (omoba-template-ids
//! SummonChaseConst); every replica adopts the committed summon pose (moba_match's summon
//! Movement facts), so replicas run none of this.
//!
//! One summon:
//! - **Seek**: a visible enemy (hero, unit or building, magic immune included; never an
//!   invulnerable or unselectable one) within `acquisition_range` of the anchor (its owner, or its
//!   spawn point). With `retarget = owner_target` it takes its owner's last attack target, except
//!   that each search picks a random candidate with `random_retarget_per_mille`. No candidate: it
//!   wanders to a random point in the ring of radius `acquisition_range / 2 ..= acquisition_range`
//!   around the anchor.
//! - **Chase**: it turns toward the target `turn_degrees` every `turn_every_frames` source frames
//!   (0.03 s), straight at it within `snap_degrees`, and flies forward at `speed`. Within
//!   `reach_radius` it reached the target: the reach ability's status hook hears CHASE_REACH
//!   (status_event, holder = the summon, status = its template id, other = the target), and it
//!   returns to its anchor. A target gone, or the summon farther than `max_distance` from its
//!   anchor, sends it back too.
//! - **Return**: to its anchor; within `reach_radius` it seeks again.
//! - **Wander**: to its wander point; there it returns with `wander_return_per_mille`, else seeks.
//! - Anywhere: farther than `teleport_back_distance` from where it is heading, it is put back on
//!   its anchor and seeks (原作 max_distance).
//! Random choices use the match's deterministic stream (SimRng per summon and tick).
use omoba_sim::trig::{self, Angle, TAU_TICKS};
use omoba_sim::{Fixed64, SimRng, Vec2};
use specs::{Component, DenseVecStorage, Entity, Join, World, WorldExt};
use std::collections::BTreeMap;

use super::ability_runtime::BuffStore;
use super::comp::{CProperty, Facing, Faction, Pos, SummonedUnit};
use crate::scripting::ScriptUnitTag;

/// The source's physics frame the turn data counts in.
const SOURCE_FRAME_MS: i64 = 30;
/// SimRng op kind of chase choices.
const RNG_OP: u32 = 0xC4A5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChaseMode { Seek, Chase(Entity), Return, Wander(Vec2) }

/// A chase summon's state (authority only).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[storage(DenseVecStorage)]
pub struct ChaseState {
    pub mode: ChaseMode,
    pub spawn: Vec2,
    /// The target of its owner's last attack start (retarget = owner_target).
    pub owner_target: Option<Entity>,
}

impl ChaseState {
    pub fn new(spawn: Vec2) -> Self { ChaseState { mode: ChaseMode::Seek, spawn, owner_target: None } }
}

/// A chase summon reached its target this tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChaseReach { pub summon: Entity, pub target: Entity, pub owner: Entity, pub ability: &'static str, pub template: String }

type Lookup<'a> = &'a dyn Fn(&str) -> Option<(&'static omoba_template_ids::SummonChaseConst, Fixed64)>;

/// A template's chase table and acquisition range.
fn template_chase(id: &str) -> Option<(&'static omoba_template_ids::SummonChaseConst, Fixed64)> {
    let stats = omoba_template_ids::summon_by_name(id).and_then(omoba_template_ids::active_summon_stats)?;
    Some((stats.chase?, stats.acquisition_range))
}

/// Moves every chase summon one tick (authority only) and returns the targets reached, in entity order.
pub fn advance(world: &World, dt: Fixed64, seed: u64) -> Vec<ChaseReach> {
    advance_with(world, dt, seed, &template_chase)
}

fn advance_with(world: &World, dt: Fixed64, seed: u64, lookup: Lookup<'_>) -> Vec<ChaseReach> {
    if !world.has_value::<specs::storage::MaskedStorage<ChaseState>>()
        || world.try_fetch::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>().is_some()
        || world.try_fetch::<crate::comp::GamePause>().is_some_and(|pause| pause.is_paused) || dt <= Fixed64::ZERO {
        return Vec::new();
    }
    let tick = world.read_resource::<crate::comp::Tick>().0;
    // The owners' attack starts this tick (原作 OnAttackStart records the owner's target).
    let mut started: BTreeMap<Entity, Entity> = BTreeMap::new();
    if let Some(events) = world.try_fetch::<super::rules::attack::AttackEvents>().filter(|events| events.tick == tick) {
        for event in &events.events {
            if let super::rules::attack::AttackEvent::Windup { attacker, target: Some(target), .. } = event { started.insert(*attacker, *target); }
        }
    }
    let rows: Vec<(Entity, &'static omoba_template_ids::SummonChaseConst, Fixed64, String, Entity)> = {
        let entities = world.entities();
        let tags = world.read_storage::<ScriptUnitTag>();
        let summons = world.read_storage::<SummonedUnit>();
        let states = world.read_storage::<ChaseState>();
        let properties = world.read_storage::<CProperty>();
        (&entities, &states, &summons).join()
            .filter(|(entity, _, summon)| !summon.is_expired() && properties.get(*entity).is_some_and(|p| p.hp > Fixed64::ZERO))
            .filter_map(|(entity, _, summon)| {
                let id = tags.get(entity)?.unit_id.clone();
                lookup(&id).map(|(chase, range)| (entity, chase, range, id, summon.summoner))
            })
            .collect()
    };
    let mut reaches = Vec::new();
    for (entity, chase, range, template, owner) in rows {
        let Some(pos) = world.read_storage::<Pos>().get(entity).map(|p| p.0) else { continue };
        let Some(mut state) = world.read_storage::<ChaseState>().get(entity).copied() else { continue };
        if let Some(target) = started.get(&owner) { state.owner_target = Some(*target); }
        let anchor = if chase.anchor_owner {
            match world.read_storage::<Pos>().get(owner) { Some(p) if world.entities().is_alive(owner) => p.0, _ => continue }
        } else { state.spawn };
        let mut rng = SimRng::from_master_entity(seed, tick as u32, entity.id(), RNG_OP);
        let mut facing = world.read_storage::<Facing>().get(entity).map_or(Angle::ZERO, |f| f.0);
        let mut at = pos;
        // Settle the mode, then fly one step toward its goal.
        if state.mode == ChaseMode::Seek { state.mode = seek(world, entity, chase, range, anchor, state.owner_target, &mut rng); }
        if let ChaseMode::Chase(target) = state.mode {
            let alive = world.entities().is_alive(target) && world.read_storage::<CProperty>().get(target).is_some_and(|p| p.hp > Fixed64::ZERO);
            if !alive || (at - anchor).length() > chase.max_distance { state.mode = ChaseMode::Return; }
        }
        let goal = match state.mode {
            ChaseMode::Chase(target) => world.read_storage::<Pos>().get(target).map_or(anchor, |p| p.0),
            ChaseMode::Wander(point) => point,
            ChaseMode::Return | ChaseMode::Seek => anchor,
        };
        if (goal - at).length() > chase.teleport_back_distance {
            at = anchor;
            state.mode = ChaseMode::Seek;
        } else {
            let (next, heading) = fly(at, facing, goal, chase, dt);
            at = next;
            facing = heading;
            let reached = (goal - at).length() < chase.reach_radius;
            match state.mode {
                ChaseMode::Chase(target) if reached => {
                    reaches.push(ChaseReach { summon: entity, target, owner, ability: chase.reach_ability, template: template.clone() });
                    state.mode = ChaseMode::Return;
                }
                ChaseMode::Return if reached => state.mode = ChaseMode::Seek,
                ChaseMode::Wander(_) if reached => {
                    state.mode = if (rng.range(0, 1000) as u16) < chase.wander_return_per_mille { ChaseMode::Return } else { ChaseMode::Seek };
                }
                _ => {}
            }
        }
        world.write_storage::<Pos>().get_mut(entity).expect("chase summon position").0 = at;
        if let Some(f) = world.write_storage::<Facing>().get_mut(entity) { f.0 = facing; }
        *world.write_storage::<ChaseState>().get_mut(entity).expect("chase state") = state;
    }
    reaches
}

/// A target for a seeking summon, else a wander point.
fn seek(world: &World, entity: Entity, chase: &omoba_template_ids::SummonChaseConst, range: Fixed64, anchor: Vec2,
    owner_target: Option<Entity>, rng: &mut SimRng) -> ChaseMode {
    let team = world.read_storage::<Faction>().get(entity).map_or(0, |f| f.team_id.max(0) as u32);
    let candidates: Vec<Entity> = {
        let entities = world.entities();
        let positions = world.read_storage::<Pos>();
        let properties = world.read_storage::<CProperty>();
        let factions = world.read_storage::<Faction>();
        let buffs = world.read_resource::<BuffStore>();
        let mut found: Vec<Entity> = (&entities, &positions, &properties, &factions).join()
            .filter(|(other, pos, property, faction)| *other != entity && property.hp > Fixed64::ZERO
                && faction.team_id.max(0) as u32 != team && (pos.0 - anchor).length() <= range
                && !buffs.has(*other, super::rules::vocab::state::INVULNERABLE) && !buffs.has(*other, "unselectable"))
            .map(|(other, ..)| other).collect();
        found.sort_by_key(|other| (other.id(), other.gen().id()));
        found
    };
    let visible: Vec<Entity> = candidates.into_iter()
        .filter(|other| super::game_processor::item_entity_visible_to_team(world, *other, team)).collect();
    if visible.is_empty() {
        // The ring radius/2 ..= radius around the anchor.
        let angle = Angle::from_ticks(rng.range(0, TAU_TICKS));
        let half = range / Fixed64::from_i32(2);
        let distance = half + Fixed64::from_raw(rng.range(0, (half.raw().clamp(0, i32::MAX as i64)) as i32 + 1) as i64);
        return ChaseMode::Wander(anchor + Vec2::new(trig::cos(angle), trig::sin(angle)) * distance);
    }
    let random = (rng.range(0, 1000) as u16) < chase.random_retarget_per_mille;
    let preferred = owner_target.filter(|target| chase.retarget_owner_target && visible.contains(target));
    match preferred {
        Some(target) if !random => ChaseMode::Chase(target),
        _ => ChaseMode::Chase(visible[rng.range(0, visible.len() as i32) as usize]),
    }
}

/// One step: turn toward `goal` (straight at it within the snap), then forward at the speed.
fn fly(at: Vec2, facing: Angle, goal: Vec2, chase: &omoba_template_ids::SummonChaseConst, dt: Fixed64) -> (Vec2, Angle) {
    let offset = goal - at;
    if offset.length_squared() <= Fixed64::ZERO { return (at, facing); }
    let desired = trig::atan2(offset.y, offset.x);
    let degree_ticks = |degrees: u16| i64::from(degrees) * i64::from(TAU_TICKS) / 360;
    let diff = (desired.ticks() - facing.ticks()).rem_euclid(TAU_TICKS);
    let diff = i64::from(if diff > TAU_TICKS / 2 { diff - TAU_TICKS } else { diff });
    let heading = if diff.abs() <= degree_ticks(chase.snap_degrees) { desired } else {
        let dt_ms = dt.raw() * 1000 / 1024;
        let step = (degree_ticks(chase.turn_degrees) * dt_ms / (SOURCE_FRAME_MS * i64::from(chase.turn_every_frames.max(1)))).max(1);
        trig::angle_rotate_toward(facing, desired, step as i32)
    };
    let step = chase.speed * dt;
    let next = if heading == desired && offset.length() <= step { goal }
        else { at + Vec2::new(trig::cos(heading), trig::sin(heading)) * step };
    (next, heading)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::comp::{FactionType, ReplicationScope, ReplicationScopeKind};
    use crate::runtime::StateInitializer;
    use specs::Builder;

    const CHASE: omoba_template_ids::SummonChaseConst = omoba_template_ids::SummonChaseConst {
        speed: Fixed64::from_raw(600 * 1024), max_distance: Fixed64::from_raw(1500 * 1024),
        teleport_back_distance: Fixed64::from_raw(3000 * 1024), reach_radius: Fixed64::from_raw(50 * 1024),
        anchor_owner: false, retarget_owner_target: false, turn_degrees: 180, turn_every_frames: 1, snap_degrees: 180,
        random_retarget_per_mille: 0, wander_return_per_mille: 1000, reach_ability: "test_reach" };

    fn v(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }
    fn alive() -> CProperty { CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO } }

    fn fixture() -> (World, Entity, Entity) {
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        if !world.has_value::<specs::storage::MaskedStorage<ChaseState>>() { world.register::<ChaseState>(); }
        let owner = world.create_entity().with(Faction { faction_id: FactionType::Player, team_id: 0 }).build();
        let summon = world.create_entity().with(Pos(v(0, 0))).with(Facing(Angle::ZERO)).with(alive())
            .with(Faction { faction_id: FactionType::Player, team_id: 0 }).with(ScriptUnitTag { unit_id: "test_chaser".into() })
            .with(SummonedUnit::new_fixed(owner, None)).with(ChaseState::new(v(0, 0))).build();
        let enemy = world.create_entity().with(Pos(v(0, 300))).with(alive()).with(Faction { faction_id: FactionType::Enemy, team_id: 1 })
            .with(ReplicationScope { kind: ReplicationScopeKind::Public, owner_team: None }).build();
        (world, summon, enemy)
    }

    fn step(world: &World) -> Vec<ChaseReach> {
        world.write_resource::<crate::comp::Tick>().0 += 1;
        advance_with(world, Fixed64::ONE / Fixed64::from_i32(30), 7, &|id| (id == "test_chaser").then_some((&CHASE, Fixed64::from_i32(1000))))
    }
    fn pos(world: &World, entity: Entity) -> Vec2 { world.read_storage::<Pos>().get(entity).unwrap().0 }
    fn mode(world: &World, entity: Entity) -> ChaseMode { world.read_storage::<ChaseState>().get(entity).unwrap().mode }

    #[test]
    fn a_chase_summon_flies_to_a_visible_enemy_reaches_it_once_and_returns_to_its_anchor() {
        let (world, summon, enemy) = fixture();
        let mut reaches = Vec::new();
        for _ in 0..30 {
            reaches.extend(step(&world));
            if !reaches.is_empty() { break; }
            assert_eq!(mode(&world, summon), ChaseMode::Chase(enemy));
        }
        assert_eq!(reaches.len(), 1);
        assert_eq!((reaches[0].summon, reaches[0].target, reaches[0].ability, reaches[0].template.as_str()), (summon, enemy, "test_reach", "test_chaser"));
        assert!((pos(&world, enemy) - pos(&world, summon)).length() < CHASE.reach_radius);
        assert_eq!(mode(&world, summon), ChaseMode::Return);
        // Home again, it seeks and takes the same enemy.
        for _ in 0..30 { assert!(step(&world).is_empty()); if mode(&world, summon) != ChaseMode::Return { break; } }
        assert!(pos(&world, summon).length() < CHASE.reach_radius);
        step(&world);
        assert_eq!(mode(&world, summon), ChaseMode::Chase(enemy));
    }

    #[test]
    fn with_no_enemy_it_wanders_in_the_ring_and_far_away_it_is_put_back_on_its_anchor() {
        let (world, summon, enemy) = fixture();
        world.write_storage::<CProperty>().get_mut(enemy).unwrap().hp = Fixed64::ZERO;
        step(&world);
        let ChaseMode::Wander(point) = mode(&world, summon) else { panic!("wanders") };
        let distance = point.length();
        assert!(distance >= Fixed64::from_i32(500) && distance <= Fixed64::from_i32(1001), "{distance:?}");
        world.write_storage::<Pos>().get_mut(summon).unwrap().0 = v(4000, 0);
        world.write_storage::<ChaseState>().get_mut(summon).unwrap().mode = ChaseMode::Return;
        step(&world);
        assert_eq!(pos(&world, summon), v(0, 0));
        assert_eq!(mode(&world, summon), ChaseMode::Seek);
    }

    #[test]
    fn replicas_never_move_a_chase_summon() {
        let (mut world, summon, _) = fixture();
        world.insert(crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets::default());
        assert!(step(&world).is_empty());
        assert_eq!(pos(&world, summon), v(0, 0));
        assert_eq!(mode(&world, summon), ChaseMode::Seek);
    }
}
