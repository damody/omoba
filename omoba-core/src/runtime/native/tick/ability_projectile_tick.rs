//! Fixed-step flight for generic ability projectiles. Motion only: hits are
//! resolved at serial settlement (`ability_projectile::settle_step`).
use crate::comp::*;
use crate::runtime::native::ability_projectile::{AbilityProjectiles, CasterDeathPolicy, Motion, TargetLostPolicy, WallPolicy};
use omoba_sim::{Fixed64, Vec2 as SimVec2};
use specs::{shred, Entities, Read, ReadStorage, SystemData, Write, WriteStorage};

#[derive(SystemData)]
pub struct AbilityProjectileData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    tick: Read<'a, Tick>,
    facts: Option<Read<'a, crate::runtime::ObservableFactBuffer>>,
    projectiles: Option<Write<'a, AbilityProjectiles>>,
    regions: Option<Read<'a, BlockedRegions>>,
    pos: WriteStorage<'a, Pos>,
    props: ReadStorage<'a, CProperty>,
    factions: ReadStorage<'a, Faction>,
    buildings: ReadStorage<'a, IsBuilding>,
    moba: Option<Read<'a, crate::runtime::MobaMatch>>,
    outcomes: Write<'a, Vec<Outcome>>,
}

#[derive(Default)]
pub struct Sys;

/// First blocked prefix of `from -> to`, as a point; binary search on Q16 time.
fn wall_contact(from: SimVec2, to: SimVec2, regions: &BlockedRegions) -> Option<SimVec2> {
    let blocked = |end: SimVec2| super::hero_move_tick::path_hits_regions(from, end, Fixed64::ZERO, regions);
    if !blocked(to) { return None; }
    let at = |t: i64| SimVec2::new(
        Fixed64::from_raw(from.x.raw() + (i128::from(to.x.raw() - from.x.raw()) * i128::from(t) >> 16) as i64),
        Fixed64::from_raw(from.y.raw() + (i128::from(to.y.raw() - from.y.raw()) * i128::from(t) >> 16) as i64));
    let (mut low, mut high) = (0_i64, 1_i64 << 16);
    while low < high { let mid = low + (high - low) / 2; if blocked(at(mid)) { high = mid; } else { low = mid + 1; } }
    Some(at(low))
}

impl<'a> System<'a> for Sys {
    type SystemData = AbilityProjectileData<'a>;

    const NAME: &'static str = "ability_projectile";

    fn run(_job: &mut Job<Self>, mut data: Self::SystemData) {
        let Some(projectiles) = data.projectiles.as_mut() else { return; };
        let dt = data.dt.0;
        // Rows whose entity retired by any other path are pruned, never moved.
        let entities = &data.entities;
        let positions = &data.pos;
        projectiles.0.retain(|entity, _| entities.is_alive(*entity) && positions.get(*entity).is_some());
        if dt <= Fixed64::ZERO { return; }
        let alive = |entity: specs::Entity| data.entities.is_alive(entity)
            && data.props.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO);
        // BTreeMap order: canonical (index, generation), independent of hashing.
        for (&entity, state) in projectiles.0.iter_mut() {
            let from = data.pos.get(entity).expect("retained projectile position").0;
            let step_token = |to: SimVec2, arrival: bool, terminal: bool| Outcome::AbilityProjectileStep {
                projectile: entity, from, to, arrival, terminal };
            if state.spec.caster_death == CasterDeathPolicy::Expire && !alive(state.owner) {
                data.outcomes.push(step_token(from, false, true));
                continue;
            }
            if let (Motion::Homing { .. }, Some(target), false) = (state.spec.motion, state.target, state.target_lost) {
                match data.pos.get(target).filter(|_| alive(target)) {
                    Some(pos) => state.end = pos.0,
                    None if state.spec.target_lost == TargetLostPolicy::Expire => {
                        data.outcomes.push(step_token(from, false, true));
                        continue;
                    }
                    None if matches!(state.spec.target_lost, TargetLostPolicy::Retarget(_)) => {
                        let TargetLostPolicy::Retarget(radius) = state.spec.target_lost else { unreachable!() };
                        // Nearest live hostile non-building unit; canonical id breaks ties.
                        use specs::Join;
                        let mut best: Option<(i128, (u32, i32), specs::Entity, SimVec2)> = None;
                        for (candidate, pos, prop, faction) in (&data.entities, &data.pos, &data.props, &data.factions).join() {
                            if candidate == target || prop.hp <= Fixed64::ZERO || data.buildings.get(candidate).is_some()
                                || !state.faction.is_hostile_to(faction)
                                || data.moba.as_ref().is_some_and(|m| !m.current_match_target_attackable(candidate)) { continue; }
                            let delta = pos.0 - from;
                            let distance = i128::from(delta.x.raw()).pow(2) + i128::from(delta.y.raw()).pow(2);
                            if distance > i128::from(radius.raw()).pow(2) { continue; }
                            let key = (distance, (candidate.id(), candidate.gen().id()));
                            if best.as_ref().is_none_or(|(d, id, _, _)| key < (*d, *id)) { best = Some((key.0, key.1, candidate, pos.0)); }
                        }
                        match best {
                            Some((_, _, next, at)) => { state.target = Some(next); state.end = at; }
                            None => { data.outcomes.push(step_token(from, false, true)); continue; }
                        }
                    }
                    None => state.target_lost = true,
                }
            }
            let active = dt.min(state.spec.lifetime - state.age).max(Fixed64::ZERO);
            let budget = state.spec.speed * active;
            state.age = state.age + active;
            let expired = state.age >= state.spec.lifetime;
            let homing = matches!(state.spec.motion, Motion::Homing { .. }) && !state.target_lost;
            // Homing contact: within this step's travel plus the projectile radius.
            let reach = budget + state.spec.radius;
            let arrival = homing && (state.end - from).length_squared() <= reach * reach;
            let Some(mut to) = (if arrival { Some(state.end) } else {
                omoba_sim::terrain::checked_travel_endpoint(from, state.end, budget) }) else {
                data.outcomes.push(step_token(from, false, true));
                continue;
            };
            let mut wall = false;
            if state.spec.wall == WallPolicy::Stop {
                if let Some(contact) = data.regions.as_ref().and_then(|regions| wall_contact(from, to, regions)) {
                    to = contact;
                    wall = true;
                }
            }
            let reached_end = !homing && to == state.end;
            state.heading = to - from;
            data.pos.get_mut(entity).expect("retained projectile position").0 = to;
            // Authority-only motion reaches team replicas as a public Movement fact
            // (projection policy decides who sees it), exactly like legacy projectiles.
            if let Some(facts) = data.facts.as_ref() {
                let source = crate::runtime::canonical_entity_id(entity);
                let _ = facts.emit(crate::runtime::OrderedFact {
                    key: crate::runtime::FactOrderingKey { tick: data.tick.0, phase: crate::runtime::FactPhase::Step,
                        canonical_source_order: source, local_ordinal: 0, fact_kind: crate::runtime::FactKind::Movement },
                    audience: crate::runtime::FactAudience::VisibilityPolicy(
                        omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
                    fact: crate::runtime::ObservableFact::Movement { source, x_mm: to.x.raw(), y_mm: to.y.raw(), facing_ticks: None },
                });
            }
            data.outcomes.push(step_token(to, arrival, wall || reached_end || expired || arrival));
        }
    }
}
