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
use std::collections::{BTreeSet, VecDeque};

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
    /// Static terrain memo kept across ticks (wall clock only, never state).
    nav_memo: Option<Write<'a, NavMemo>>,
    buff_store: Read<'a, omoba_core::runtime::ability_runtime::BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
    queues: WriteStorage<'a, HeroCommandQueue>,
    move_targets: WriteStorage<'a, MoveTarget>,
    properties: ReadStorage<'a, CProperty>,
    owners: ReadStorage<'a, PlayerOwner>,
    visibility: Option<Read<'a, crate::runtime::TeamVisibilityRuntime>>,
    moba: Option<Read<'a, crate::runtime::MobaMatch>>,
    disclosed_recalls: Option<Read<'a, super::super::cast_windup::DisclosedRecallingOwners>>,
    casts: Option<Read<'a, super::super::cast_windup::AbilityWindups>>,
    disclosed_priority: Option<Read<'a, crate::runtime::DisclosedMovementPriority>>,
    admitted: Option<Read<'a, crate::runtime::native::combat_acquisition::AdmittedCombatPreferences>>,
    attack_gate: Option<Read<'a, crate::runtime::filtered_specs::DisclosedAttackabilityGate>>,
    disclosed_wards: Option<Read<'a, crate::runtime::filtered_specs::DisclosedWardCandidates>>,
    retaliation: Option<Read<'a, super::super::retaliation::HeroRetaliation>>,
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
        // Built once per distinct terrain, not every tick (E187 §5.6).
        let terrain = TerrainIndex::cached(&data.regions);
        let terrain = &*terrain;
        let nav = std::cell::RefCell::new(data.nav_memo.as_deref_mut().map(std::mem::take).unwrap_or_default());
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
                terrain: &terrain,
                nav: &nav,
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
                        && !super::super::cast_windup::hero_recalling(data.moba.as_deref(),data.disclosed_recalls.as_deref(),&data.owners,entity)
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
                        auto_engage(&data, &acquisition, entity, pos.0, queue.clone()),
                        arrive_eps,
                    ))
                })
                .collect()
        };
        decisions.sort_by_key(|decision| decision.entity.id());
        if let Some(memo) = data.nav_memo.as_deref_mut() { *memo = nav.into_inner(); }

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

/// The original game's acquisition (Dota AttackAcquisitionRange, templates/game_rules.lua): an
/// idle hero with no order goes after a recent attacker within its acquisition range (a camp
/// included, retaliation.rs), else the nearest enemy within it (camps never pulled). A target
/// already in attack range is left to hero_tick, which attacks without moving. Authority only:
/// the chase is committed to replicas through movement and attack facts.
fn auto_engage(
    data: &HeroCommandData<'_>,
    acquisition: &CombatAcquisitionView<'_, '_>,
    entity: specs::Entity,
    pos: SimVec2,
    mut queue: HeroCommandQueue,
) -> HeroCommandQueue {
    let Some(state) = data.moba.as_deref() else { return queue };
    let Some(rules) = state.config.game_rules else { return queue };
    if queue.active.is_some() || !queue.queued.is_empty()
        || !data.properties.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO)
        || data.casts.as_ref().is_some_and(|casts| casts.0.contains_key(&entity))
        || super::super::cast_windup::hero_recalling(data.moba.as_deref(), data.disclosed_recalls.as_deref(), &data.owners, entity) {
        return queue;
    }
    let Some(attack) = data.attacks.get(entity) else { return queue };
    let stats = crate::runtime::ability_runtime::UnitStats::from_refs(&*data.buff_store, data.is_buildings.get(entity).is_some());
    let attack_range = stats.final_attack_range(attack.range.v, entity);
    // The hero's own AttackAcquisitionRange (source unit), else the game rules' hero default.
    let own = data.heroes.get(entity).and_then(|hero| omoba_template_ids::hero_by_name(&hero.id))
        .and_then(omoba_template_ids::active_hero_stats).and_then(|stats| stats.acquisition_range);
    let acquisition_range = own.unwrap_or(Fixed64::from_i32(rules.hero_acquisition_range.min(i32::MAX as u32) as i32))
        .max(attack_range);
    let attacker = data.retaliation.as_deref()
        .and_then(|hits| super::super::retaliation::attacker(hits, entity, state.elapsed))
        .filter(|attacker| matches!(assess_direct_target(acquisition, entity, *attacker),
            Some(DirectStatus::Legal { position }) if within_raw_range(pos, position, acquisition_range)));
    let target = attacker.or_else(|| acquire(acquisition, AcquisitionRequest { source: entity, intent: AcquisitionIntent::Idle,
        command_point: None, range: acquisition_range, explicit_target: None }));
    if let Some(target) = target {
        let in_range = data.pos.get(target).is_some_and(|at| within_raw_range(pos, at.0, attack_range));
        if !in_range { queue.active = Some(HeroCommand::AttackTarget { target, chase_origin: None }); }
    }
    queue
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
    terrain: &'b TerrainIndex,
    nav: &'b std::cell::RefCell<NavMemo>,
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
    static_next_waypoint_with(from, target, radius, data.terrain, &mut data.nav.borrow_mut())
}

/// Public terrain, quantized once. `hits` returns exactly what
/// `path_hits_regions` returns for the same regions (any non-finite point
/// blocks everything); it only skips the per-call quantization and allocation.
pub(crate) struct TerrainIndex {
    polygons: Vec<Vec<SimVec2>>,
    nonfinite: bool,
    fingerprint: u64,
    /// Broad phase (E187 §5.6: the original nobu map has 305 gridnav rectangles): polygon indices
    /// by the TERRAIN_BUCKET_RAW cells their bounds touch, and each polygon's inclusive raw
    /// bounds. Polygons with a coordinate beyond the geometry bound are in `always`, because the
    /// exact test reports them as hit wherever they are.
    buckets: std::collections::HashMap<(i64, i64), Vec<u32>>,
    bounds: Vec<[i64; 4]>,
    always: Vec<u32>,
}

/// Broad phase bucket edge: 512 world units.
const TERRAIN_BUCKET_RAW: i64 = 512 * SCALE;
/// A query covering more buckets than this scans every polygon instead.
const TERRAIN_BUCKET_QUERY_LIMIT: i64 = 1024;

impl TerrainIndex {
    /// Bounds and buckets of the quantized polygons; the exact test stays
    /// `swept_circle_hits_polygon`, so `hits` answers exactly as a full scan would.
    fn build_broad_phase(&mut self) {
        let max = omoba_sim::terrain::MAX_GEOMETRY_RAW;
        for (index, polygon) in self.polygons.iter().enumerate() {
            let bound = [polygon.iter().map(|p| p.x.raw()).min().unwrap_or(0), polygon.iter().map(|p| p.y.raw()).min().unwrap_or(0),
                polygon.iter().map(|p| p.x.raw()).max().unwrap_or(0), polygon.iter().map(|p| p.y.raw()).max().unwrap_or(0)];
            self.bounds.push(bound);
            // Fewer than three points never hit.
            if polygon.len() < 3 { continue; }
            if polygon.iter().any(|p| p.x.raw().unsigned_abs() > max as u64 || p.y.raw().unsigned_abs() > max as u64) {
                self.always.push(index as u32);
                continue;
            }
            for cx in bound[0].div_euclid(TERRAIN_BUCKET_RAW)..=bound[2].div_euclid(TERRAIN_BUCKET_RAW) {
                for cy in bound[1].div_euclid(TERRAIN_BUCKET_RAW)..=bound[3].div_euclid(TERRAIN_BUCKET_RAW) {
                    self.buckets.entry((cx, cy)).or_default().push(index as u32);
                }
            }
        }
    }

    pub(crate) fn new(regions: &BlockedRegions) -> Self {
        let mut fingerprint: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |value: i64| {
            for byte in value.to_le_bytes() {
                fingerprint = (fingerprint ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let mut nonfinite = false;
        let mut polygons = Vec::with_capacity(regions.0.len());
        mix(regions.0.len() as i64);
        for region in &regions.0 {
            if region.points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
                nonfinite = true;
                mix(-1);
                continue;
            }
            let polygon: Vec<SimVec2> = region.points.iter().map(|p| SimVec2::new(
                Fixed64::from_raw((f64::from(p.x) * 1024.0).round() as i64),
                Fixed64::from_raw((f64::from(p.y) * 1024.0).round() as i64))).collect();
            mix(polygon.len() as i64);
            for point in &polygon { mix(point.x.raw()); mix(point.y.raw()); }
            polygons.push(polygon);
        }
        let mut index = Self { polygons, nonfinite, fingerprint, buckets: Default::default(), bounds: Vec::new(), always: Vec::new() };
        index.build_broad_phase();
        index
    }

    pub(crate) fn hits(&self, from: SimVec2, to: SimVec2, radius: Fixed64) -> bool {
        if self.nonfinite { return true; }
        let max = omoba_sim::terrain::MAX_GEOMETRY_RAW;
        let bounded = |v: SimVec2| v.x.raw().unsigned_abs() <= max as u64 && v.y.raw().unsigned_abs() <= max as u64;
        let full_scan = || self.polygons.iter()
            .any(|polygon| omoba_sim::terrain::swept_circle_hits_polygon(from, to, radius, polygon));
        // Out-of-range inputs: the exact test decides for every polygon.
        if radius.raw() < 0 || radius.raw() > max || !bounded(from) || !bounded(to) { return full_scan(); }
        let r = radius.raw();
        let query = [from.x.raw().min(to.x.raw()) - r, from.y.raw().min(to.y.raw()) - r,
            from.x.raw().max(to.x.raw()) + r, from.y.raw().max(to.y.raw()) + r];
        let cells = [query[0].div_euclid(TERRAIN_BUCKET_RAW), query[1].div_euclid(TERRAIN_BUCKET_RAW),
            query[2].div_euclid(TERRAIN_BUCKET_RAW), query[3].div_euclid(TERRAIN_BUCKET_RAW)];
        if (cells[2] - cells[0] + 1) * (cells[3] - cells[1] + 1) > TERRAIN_BUCKET_QUERY_LIMIT { return full_scan(); }
        let test = |index: u32| omoba_sim::terrain::swept_circle_hits_polygon(from, to, radius, &self.polygons[index as usize]);
        if self.always.iter().any(|index| test(*index)) { return true; }
        // A polygon outside every bucket the query box touches has bounds disjoint from it, which
        // the exact test's own broad phase rejects; inside, the same inclusive bounds come first.
        for cx in cells[0]..=cells[2] {
            for cy in cells[1]..=cells[3] {
                let Some(bucket) = self.buckets.get(&(cx, cy)) else { continue };
                for &index in bucket {
                    let b = self.bounds[index as usize];
                    if query[2] < b[0] || query[0] > b[2] || query[3] < b[1] || query[1] > b[3] { continue; }
                    if test(index) { return true; }
                }
            }
        }
        false
    }

    /// The match's terrain index, built once per distinct terrain (E187 §5.6): callers that only
    /// hold `BlockedRegions` share it instead of rebuilding it on every query.
    pub(crate) fn cached(regions: &BlockedRegions) -> std::rc::Rc<TerrainIndex> {
        type Shape = Vec<Vec<(u32, u32)>>;
        thread_local! {
            static CACHE: std::cell::RefCell<Option<(Shape, std::rc::Rc<TerrainIndex>)>> = const { std::cell::RefCell::new(None) };
        }
        // Reused only for bit-identical points (the exact geometry the index was built from).
        let same = |shape: &Shape| shape.len() == regions.0.len() && shape.iter().zip(&regions.0).all(|(cached, region)|
            cached.len() == region.points.len() && cached.iter().zip(&region.points).all(|(c, p)| *c == (p.x.to_bits(), p.y.to_bits())));
        CACHE.with(|cache| {
            if let Some((shape, index)) = cache.borrow().as_ref() {
                if same(shape) { return index.clone(); }
            }
            let shape = regions.0.iter().map(|region| region.points.iter().map(|p| (p.x.to_bits(), p.y.to_bits())).collect()).collect();
            let index = std::rc::Rc::new(TerrainIndex::new(regions));
            *cache.borrow_mut() = Some((shape, index.clone()));
            index
        })
    }
}

/// Memo of the static grid tests (a cell centre blocked, or the edge between
/// two cell centres blocked) per radius. They depend only on public terrain,
/// so a memo hit is the same bool the geometry would return; it is cleared
/// whenever the terrain fingerprint changes and bounded in size.
#[derive(Default)]
pub struct NavMemo {
    fingerprint: Option<u64>,
    cells: std::collections::HashMap<(i64, GridKey), bool>,
    edges: std::collections::HashMap<(i64, GridKey, GridKey), bool>,
    parent: Vec<u32>,
}

impl NavMemo {
    const MAX_ENTRIES: usize = 2_000_000;

    fn sync(&mut self, terrain: &TerrainIndex) {
        if self.fingerprint != Some(terrain.fingerprint) || self.cells.len() + self.edges.len() > Self::MAX_ENTRIES {
            self.cells.clear();
            self.edges.clear();
            self.fingerprint = Some(terrain.fingerprint);
        }
    }

    fn cell_blocked(&mut self, terrain: &TerrainIndex, radius: Fixed64, key: GridKey) -> bool {
        *self.cells.entry((radius.raw(), key))
            .or_insert_with(|| terrain.hits(grid_pos(key), grid_pos(key), radius))
    }

    fn edge_blocked(&mut self, terrain: &TerrainIndex, radius: Fixed64, from: GridKey, to: GridKey) -> bool {
        *self.edges.entry((radius.raw(), from, to))
            .or_insert_with(|| terrain.hits(grid_pos(from), grid_pos(to), radius))
    }
}

/// Shared bounded, deterministically ordered search over public terrain only.
/// Heroes preserve their existing grid behavior; NPCs try a clear direct path first.
pub(crate) fn static_next_waypoint(
    from: SimVec2,
    target: SimVec2,
    radius: Fixed64,
    regions: &BlockedRegions,
) -> Option<SimVec2> {
    // The shared index and memo (E187 §5.6): both only cache public terrain answers.
    thread_local! { static MEMO: std::cell::RefCell<NavMemo> = std::cell::RefCell::new(NavMemo::default()); }
    let terrain = TerrainIndex::cached(regions);
    MEMO.with(|memo| match memo.try_borrow_mut() {
        Ok(mut memo) => static_next_waypoint_with(from, target, radius, &terrain, &mut memo),
        Err(_) => static_next_waypoint_with(from, target, radius, &terrain, &mut NavMemo::default()),
    })
}

/// Same search and tie-breaking as the original (FIFO, fixed neighbour
/// order). Static cell and cell-to-cell edge tests come from `memo`; parents
/// live in a dense array over the bounded search box instead of a tree map.
pub(crate) fn static_next_waypoint_with(
    from: SimVec2,
    target: SimVec2,
    radius: Fixed64,
    terrain: &TerrainIndex,
    memo: &mut NavMemo,
) -> Option<SimVec2> {
    memo.sync(terrain);
    let start = grid_key(from);
    let goal = grid_key(target);
    if terrain.hits(target, target, radius) { return None; }
    let same_cell = start == goal;
    if same_cell {
        if !terrain.hits(from, target, radius) { return Some(target); }
        // The actual endpoints can lie on opposite sides of a thin obstacle
        // inside one cell. Search outside the cell instead of rejecting them.
    }

    let dx = (goal.x - start.x).abs();
    let dy = (goal.y - start.y).abs();
    if dx > MAX_PATH_GRID_SPAN || dy > MAX_PATH_GRID_SPAN {
        return (!terrain.hits(from, target, radius)).then_some(target);
    }

    let min_x = start.x.min(goal.x) - PATH_GRID_MARGIN;
    let max_x = start.x.max(goal.x) + PATH_GRID_MARGIN;
    let min_y = start.y.min(goal.y) - PATH_GRID_MARGIN;
    let max_y = start.y.max(goal.y) + PATH_GRID_MARGIN;
    let width = (max_x - min_x + 1) as usize;
    let index = |key: GridKey| (key.y - min_y) as usize * width + (key.x - min_x) as usize;
    let key_of = |slot: usize| GridKey { x: min_x + (slot % width) as i64, y: min_y + (slot / width) as i64 };
    let unvisited = u32::MAX;
    let mut parent = std::mem::take(&mut memo.parent);
    parent.clear();
    parent.resize(width * (max_y - min_y + 1) as usize, unvisited);

    let mut open = VecDeque::new();
    let mut reached = None;
    parent[index(start)] = index(start) as u32;
    open.push_back(start);

    while let Some(cur) = open.pop_front() {
        if !same_cell && cur == goal {
            reached = Some(cur);
            break;
        }
        // A same-cell goal cannot reuse the already-visited start key. Treat
        // the precise target as a terminal edge, checked like every other edge.
        if same_cell && cur != start && !terrain.hits(grid_pos(cur), target, radius) {
            reached = Some(cur);
            break;
        }
        for next in neighbors(cur) {
            if next.x < min_x || next.x > max_x || next.y < min_y || next.y > max_y {
                continue;
            }
            if parent[index(next)] != unvisited {
                continue;
            }
            if next != goal && memo.cell_blocked(terrain, radius, next) {
                continue;
            }
            // Edges touching the exact start or target position stay live.
            let blocked = if cur == start || next == goal {
                let edge_from = if cur == start { from } else { grid_pos(cur) };
                let edge_to = if next == goal { target } else { grid_pos(next) };
                terrain.hits(edge_from, edge_to, radius)
            } else {
                memo.edge_blocked(terrain, radius, cur, next)
            };
            if blocked { continue; }
            parent[index(next)] = index(cur) as u32;
            open.push_back(next);
        }
    }

    // A closest reachable cell is not proof that the requested destination is
    // reachable. Never turn an exhausted bounded search into a partial route.
    let mut result = None;
    if let Some(mut cur) = reached {
        loop {
            let prev_slot = parent[index(cur)];
            if prev_slot == unvisited { break; }
            let prev = key_of(prev_slot as usize);
            if prev == start {
                result = Some(if cur == goal { target } else { grid_pos(cur) });
                break;
            }
            if prev == cur { break; }
            cur = prev;
        }
    }
    memo.parent = parent;
    result
}

/// MOBA move-order rule for a destination the hero cannot reach (inside a
/// wall, enclosed, or beyond the searchable span with a blocked line): head
/// for the reachable grid cell closest to it. A far destination is first
/// pulled back along the line to the searchable span. The search is the same
/// bounded FIFO search with the same neighbour order, run to exhaustion; ties
/// keep the earliest cell in search order, so authority and replicas agree.
/// `Some(target)` when the destination itself is reachable; `None` only when
/// no reachable cell is closer than the hero's own cell (it cannot move closer).
pub(crate) fn static_nearest_reachable(
    from: SimVec2,
    target: SimVec2,
    radius: Fixed64,
    terrain: &TerrainIndex,
    memo: &mut NavMemo,
) -> Option<SimVec2> {
    if static_next_waypoint_with(from, target, radius, terrain, memo).is_some() {
        return Some(target);
    }
    memo.sync(terrain);
    let start = grid_key(from);
    let mut goal = grid_key(target);
    let span = MAX_PATH_GRID_SPAN - 1;
    if (goal.x - start.x).abs() > span || (goal.y - start.y).abs() > span {
        let dx = goal.x - start.x;
        let dy = goal.y - start.y;
        let scale = dx.abs().max(dy.abs());
        goal = GridKey { x: start.x + dx * span / scale, y: start.y + dy * span / scale };
    }
    let aim = grid_pos(goal);
    let min_x = start.x.min(goal.x) - PATH_GRID_MARGIN;
    let max_x = start.x.max(goal.x) + PATH_GRID_MARGIN;
    let min_y = start.y.min(goal.y) - PATH_GRID_MARGIN;
    let max_y = start.y.max(goal.y) + PATH_GRID_MARGIN;
    let width = (max_x - min_x + 1) as usize;
    let index = |key: GridKey| (key.y - min_y) as usize * width + (key.x - min_x) as usize;
    let unvisited = u32::MAX;
    let mut parent = std::mem::take(&mut memo.parent);
    parent.clear();
    parent.resize(width * (max_y - min_y + 1) as usize, unvisited);
    let distance = |key: GridKey| {
        let delta = grid_pos(key) - aim;
        i128::from(delta.x.raw()) * i128::from(delta.x.raw()) + i128::from(delta.y.raw()) * i128::from(delta.y.raw())
    };
    let mut best = (distance(start), start);
    let mut open = VecDeque::new();
    parent[index(start)] = index(start) as u32;
    open.push_back(start);
    while let Some(cur) = open.pop_front() {
        let here = distance(cur);
        if here < best.0 { best = (here, cur); }
        for next in neighbors(cur) {
            if next.x < min_x || next.x > max_x || next.y < min_y || next.y > max_y { continue; }
            if parent[index(next)] != unvisited { continue; }
            if memo.cell_blocked(terrain, radius, next) { continue; }
            let blocked = if cur == start { terrain.hits(from, grid_pos(next), radius) }
                else { memo.edge_blocked(terrain, radius, cur, next) };
            if blocked { continue; }
            parent[index(next)] = index(cur) as u32;
            open.push_back(next);
        }
    }
    memo.parent = parent;
    (best.1 != start).then(|| grid_pos(best.1))
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    use std::collections::BTreeMap;

    #[test]
    fn nearest_reachable_heads_for_the_closest_cell_outside_a_wall_and_refuses_when_enclosed() {
        let p = |x: i32, y: i32| SimVec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
        let rect = |name: &str, x0: f32, y0: f32, x1: f32, y1: f32| BlockedRegion { name: name.into(), points: vec![
            vek::Vec2::new(x0, y0), vek::Vec2::new(x1, y0), vek::Vec2::new(x1, y1), vek::Vec2::new(x0, y1)] };
        let radius = Fixed64::from_i32(20);
        // split38: a lane route point inside a jungle wall.
        let wall = BlockedRegions(vec![rect("jungle_wall", 400.0, 3300.0, 1000.0, 3420.0)]);
        let terrain = TerrainIndex::new(&wall);
        let mut memo = NavMemo::default();
        let (from, inside) = (p(760, 2560), p(845, 3351));
        assert_eq!(static_next_waypoint_with(from, inside, radius, &terrain, &mut memo), None, "the click itself is unreachable");
        let nearest = static_nearest_reachable(from, inside, radius, &terrain, &mut memo).expect("a closer reachable point");
        assert!(!terrain.hits(nearest, nearest, radius) && static_next_waypoint_with(from, nearest, radius, &terrain, &mut memo).is_some());
        assert!((nearest - inside).length() < (from - inside).length(), "it moves the hero closer");
        assert!(nearest.y < Fixed64::from_i32(3300), "it stops on the hero's side of the wall: {nearest:?}");
        assert_eq!(static_nearest_reachable(from, inside, radius, &terrain, &mut NavMemo::default()), Some(nearest), "deterministic");
        // A reachable destination is kept exactly.
        assert_eq!(static_nearest_reachable(from, p(900, 3000), radius, &terrain, &mut memo), Some(p(900, 3000)));
        // Far destinations are pulled back along the line before searching.
        let far = static_nearest_reachable(from, p(845, 3351 + 30_000), radius, &terrain, &mut memo).expect("far target");
        assert!(far.y > from.y);
        // A pen too small for any other grid cell: the hero cannot get closer, so no
        // order is produced and the caller rejects it with UNREACHABLE.
        let pen = BlockedRegions(vec![rect("n", 0.0, 30.0, 60.0, 70.0), rect("s", 0.0, -70.0, 60.0, -30.0),
            rect("w", -40.0, -70.0, 0.0, 70.0), rect("e", 60.0, -70.0, 100.0, 70.0)]);
        let pen_terrain = TerrainIndex::new(&pen);
        assert_eq!(static_nearest_reachable(p(30, 0), p(30, 1000), radius, &pen_terrain, &mut NavMemo::default()), None);
    }

    /// Deterministic queries over every compiled map: the memoized dense planner
    /// returns exactly the reference planner's waypoint, with one memo reused
    /// across all queries (as in the live system) and with a fresh memo.
    #[test]
    fn memoized_planner_matches_reference_on_every_compiled_map() {
        let mut seed: u64 = 0x5eed_1234_abcd_0001;
        let mut next = |bound: i64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as i64).rem_euclid(bound.max(1))
        };
        let mut compared = 0usize;
        for map in omoba_template_ids::MOBA_MAPS {
            let regions = crate::runtime::moba_map_layout::compiled_blocked_regions(map);
            if regions.0.is_empty() { continue; }
            let (mut lo_x, mut lo_y, mut hi_x, mut hi_y) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
            for region in map.terrain.iter() {
                lo_x = lo_x.min(i64::from(region.min.0)); lo_y = lo_y.min(i64::from(region.min.1));
                hi_x = hi_x.max(i64::from(region.max.0)); hi_y = hi_y.max(i64::from(region.max.1));
            }
            let (lo_x, lo_y, hi_x, hi_y) = (lo_x - 600, lo_y - 600, hi_x + 600, hi_y + 600);
            let terrain = TerrainIndex::new(&regions);
            let mut shared = NavMemo::default();
            for query in 0..600 {
                let point = |x: i64, y: i64| SimVec2::new(Fixed64::from_raw(x * 1024 + (x % 7) * 37), Fixed64::from_raw(y * 1024 - (y % 5) * 41));
                let from = point(lo_x + next(hi_x - lo_x), lo_y + next(hi_y - lo_y));
                // Mostly nearby targets (the searched case), some far ones (direct case).
                let reach = if query % 5 == 0 { 9000 } else { 2400 };
                let target = SimVec2::new(from.x + Fixed64::from_raw((next(2 * reach) - reach) * 1024),
                    from.y + Fixed64::from_raw((next(2 * reach) - reach) * 1024));
                for radius in [Fixed64::from_i32(20), Fixed64::from_raw(35 * 1024 + 300), Fixed64::from_i32(64)] {
                    let expected = static_next_waypoint_reference(from, target, radius, &regions);
                    assert_eq!(static_next_waypoint_with(from, target, radius, &terrain, &mut shared), expected,
                        "map {} from {:?} target {:?} radius {:?} (shared memo)", map.id, from, target, radius);
                    assert_eq!(static_next_waypoint(from, target, radius, &regions), expected, "fresh memo");
                    compared += 1;
                }
            }
        }
        assert!(compared > 1000, "too few comparisons: {compared}");
    }

    #[test]
    fn terrain_index_matches_path_hits_regions_and_memo_follows_terrain_changes() {
        let wall = |x: f32| BlockedRegions(vec![BlockedRegion { name: "wall".into(), points: vec![
            vek::Vec2::new(x, -500.0), vek::Vec2::new(x + 40.0, -500.0), vek::Vec2::new(x + 40.0, 500.0), vek::Vec2::new(x, 500.0)] }]);
        let radius = Fixed64::from_i32(20);
        let p = |x: i32, y: i32| SimVec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
        for regions in [wall(100.0), wall(300.0), BlockedRegions(vec![BlockedRegion { name: "nan".into(), points: vec![vek::Vec2::new(f32::NAN, 0.0); 4] }])] {
            let index = TerrainIndex::new(&regions);
            for (a, b) in [(p(0, 0), p(400, 0)), (p(0, 0), p(0, 300)), (p(200, 0), p(200, 0)), (p(-50, 600), p(450, 600))] {
                assert_eq!(index.hits(a, b, radius), path_hits_regions(a, b, radius, &regions));
            }
        }
        // One memo across a terrain change must not reuse the old wall's answers.
        let mut memo = NavMemo::default();
        let (from, target) = (p(0, 0), p(400, 0));
        for regions in [wall(100.0), wall(250.0), wall(100.0)] {
            assert_eq!(static_next_waypoint_with(from, target, radius, &TerrainIndex::new(&regions), &mut memo),
                static_next_waypoint_reference(from, target, radius, &regions));
        }
    }

    /// Release-only timing comparison (ignored by default):
    /// `cargo test --release -p omoba-core --lib -- planner_speedup --ignored --nocapture`
    #[test]
    #[ignore]
    fn planner_speedup_on_compiled_maps() {
        let map = omoba_template_ids::MOBA_MAPS.iter().find(|map| map.id == "three_lane_quick_training")
            .or_else(|| omoba_template_ids::MOBA_MAPS.first()).unwrap();
        let regions = crate::runtime::moba_map_layout::compiled_blocked_regions(map);
        let radius = Fixed64::from_i32(35);
        let queries: Vec<(SimVec2, SimVec2)> = (0..200).map(|i| {
            let x = (i * 137 % 4000) as i32 - 2000; let y = (i * 251 % 4000) as i32 - 2000;
            (SimVec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)), SimVec2::new(Fixed64::from_i32(x + 1500), Fixed64::from_i32(y - 900)))
        }).collect();
        let started = std::time::Instant::now();
        let reference: Vec<_> = queries.iter().map(|(a, b)| static_next_waypoint_reference(*a, *b, radius, &regions)).collect();
        let reference_ns = started.elapsed().as_nanos();
        let terrain = TerrainIndex::new(&regions);
        let mut memo = NavMemo::default();
        let _warm: Vec<_> = queries.iter().map(|(a, b)| static_next_waypoint_with(*a, *b, radius, &terrain, &mut memo)).collect();
        let started = std::time::Instant::now();
        let memoized: Vec<_> = queries.iter().map(|(a, b)| static_next_waypoint_with(*a, *b, radius, &terrain, &mut memo)).collect();
        let memo_ns = started.elapsed().as_nanos();
        assert_eq!(reference, memoized);
        println!("planner on {}: reference {:.3} ms/query, memoized (warm) {:.3} ms/query, speedup {:.1}x",
            map.id, reference_ns as f64 / 200.0 / 1e6, memo_ns as f64 / 200.0 / 1e6, reference_ns as f64 / memo_ns.max(1) as f64);
    }

    // The pre-memo planner, kept verbatim as the parity reference.
    /// Shared bounded, deterministically ordered search over public terrain only.
    /// Heroes preserve their existing grid behavior; NPCs try a clear direct path first.
    fn static_next_waypoint_reference(
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
            radii:&radii,searcher:&searcher,regions:&regions,terrain:&TerrainIndex::new(&regions),nav:&std::cell::RefCell::new(NavMemo::default()),buff_store:&buff_store,is_buildings:&is_buildings },hero,from,target)
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
            terrain: &TerrainIndex::new(&regions),
            nav: &std::cell::RefCell::new(NavMemo::default()),
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
            terrain: &TerrainIndex::new(&regions),
            nav: &std::cell::RefCell::new(NavMemo::default()),
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
