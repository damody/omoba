use abi_stable::std_types::{RNone, ROption, RSome, RStr, RVec};
use omb_script_abi::{
    stat_keys::StatKey,
    types::{
        Angle, DamageKind, DamageProfile as AbiDamageProfile, EntityHandle, Fixed64, PathSpec,
        ProjectileSpec, Target, Vec2,
    },
    world::{GameWorld, ProjectileQuery, TowerActiveAbilityAccess, TowerCooldownAccess},
};
use specs::world::Generation;
use specs::{Entities, Entity, Join, Read, ReadStorage, World, WorldExt};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::comp::*;
use crate::runtime::ability_runtime::{BuffStore, UnitStats};

use super::tag::ScriptUnitTag;

pub struct ParallelAdapterCache<'a> {
    team_keys:HashMap<i32,String>,
    pub entities: Entities<'a>,
    pub tattack: ReadStorage<'a, TAttack>,
    pub pos: ReadStorage<'a, Pos>,
    move_targets: Option<ReadStorage<'a, MoveTarget>>,
    summoned: Option<ReadStorage<'a, SummonedUnit>>,
    commands: Option<ReadStorage<'a, HeroCommandQueue>>,
    summon_host_movement: Option<Read<'a, super::super::tick::summon_tick::HostMovement>>,
    pub facing: ReadStorage<'a, Facing>,
    pub cprop: ReadStorage<'a, CProperty>,
    pub unit: ReadStorage<'a, Unit>,
    pub hero: ReadStorage<'a, Hero>,
    pub faction: ReadStorage<'a, Faction>,
    pub creep: ReadStorage<'a, Creep>,
    pub tower: ReadStorage<'a, Tower>,
    pub is_building: ReadStorage<'a, IsBuilding>,
    pub collision: ReadStorage<'a, CollisionRadius>,
    pub tags: ReadStorage<'a, ScriptUnitTag>,
    pub player_owner: ReadStorage<'a, PlayerOwner>,
    pub buffs: Read<'a, BuffStore>,
    pub searcher: Read<'a, Searcher>,
    pub blocked: Read<'a, BlockedRegions>,
    pub tick: Read<'a, Tick>,
    pub rng_seed: u64,
    pub walls: Option<Read<'a, crate::runtime::VisionOccluderSet>>,
    visibility: Option<Read<'a, crate::runtime::TeamVisibilityRuntime>>,
    moba: Option<Read<'a, crate::runtime::MobaMatch>>,
    attack_gate: Option<Read<'a, crate::runtime::filtered_specs::DisclosedAttackabilityGate>>,
    filtered: bool,
    summon_placements: super::super::summon_navigation::PlacementReservations,
}

impl<'a> ParallelAdapterCache<'a> {
    pub fn new(world: &'a World, rng_seed: u64) -> Self {
        let summon_placements=world.try_fetch::<super::super::summon_navigation::PlacementReservations>()
            .map(|ledger|(*ledger).clone()).unwrap_or_default();
        summon_placements.0.lock().unwrap_or_else(|p|p.into_inner())
            .retain(|entity,_|world.entities().is_alive(*entity));
        let faction=world.read_storage::<Faction>();
        let mut team_keys=HashMap::new();
        for f in (&faction).join() {team_keys.entry(f.team_id).or_insert_with(||format!("team:{}",f.team_id));}
        Self {
            team_keys,
            summon_placements,
            entities: world.entities(),
            tattack: world.read_storage::<TAttack>(),
            pos: world.read_storage::<Pos>(),
            move_targets: world.has_value::<specs::storage::MaskedStorage<MoveTarget>>()
                .then(||world.read_storage::<MoveTarget>()),
            summoned: world.has_value::<specs::storage::MaskedStorage<SummonedUnit>>()
                .then(||world.read_storage::<SummonedUnit>()),
            commands: world.has_value::<specs::storage::MaskedStorage<HeroCommandQueue>>()
                .then(||world.read_storage::<HeroCommandQueue>()),
            summon_host_movement: world.try_fetch::<super::super::tick::summon_tick::HostMovement>().map(Into::into),
            facing: world.read_storage::<Facing>(),
            cprop: world.read_storage::<CProperty>(),
            unit: world.read_storage::<Unit>(),
            hero: world.read_storage::<Hero>(),
            faction,
            creep: world.read_storage::<Creep>(),
            tower: world.read_storage::<Tower>(),
            is_building: world.read_storage::<IsBuilding>(),
            collision: world.read_storage::<CollisionRadius>(),
            tags: world.read_storage::<ScriptUnitTag>(),
            player_owner: world.read_storage::<PlayerOwner>(),
            buffs: world.read_resource::<BuffStore>().into(),
            searcher: world.read_resource::<Searcher>().into(),
            blocked: world.read_resource::<BlockedRegions>().into(),
            tick: world.read_resource::<Tick>().into(),
            rng_seed,
            walls: world.try_fetch::<crate::runtime::VisionOccluderSet>().map(Into::into),
            visibility: world.try_fetch::<crate::runtime::TeamVisibilityRuntime>().map(Into::into),
            moba: world.try_fetch::<crate::runtime::MobaMatch>().map(Into::into),
            attack_gate: world.try_fetch::<crate::runtime::filtered_specs::DisclosedAttackabilityGate>().map(Into::into),
            filtered: world.try_fetch::<crate::runtime::filtered_specs::DisclosedMovementPriority>().is_some(),
        }
    }
}

pub struct ParallelWorldAdapter<'a> {
    pub cache: &'a ParallelAdapterCache<'a>,
    invocation_rng_op: u32,
    random_request_base: u64,
    outcomes: Vec<Outcome>,
    overlay_pos: HashMap<Entity, Vec2>,
    cast_succeeded: bool,
    /// Owner-visible code for a cast handler that did not succeed.
    cast_rejection: Option<&'static str>,
    hook_panicked: bool,
    mana_transaction: bool,
    mana_views: HashMap<Entity, crate::runtime::ability_runtime::ManaPool>,
    mana_dirty: Vec<Entity>,
    mana_originals: HashMap<Entity, crate::runtime::ability_runtime::ManaPool>,
    overlay_facing: HashMap<Entity, Angle>,
    overlay_asd_count: HashMap<Entity, Fixed64>,
    projectile_hit_generation: Option<u8>,
    projectile_origin: Option<ProjectileOrigin>,
    projectile_source_buffs: Option<BuffStore>,
    projectile_team_key: Option<String>,
    capture_cast_areas: bool,
    cast_areas: Vec<(Vec2, Fixed64, Fixed64)>,
    pub(crate) allocation_allowed: bool,
    pub(crate) allocation_requested: bool,
    reserved_entities: Vec<Entity>,
}

pub struct ParallelProjectileQuery<'a> {
    cache: &'a ParallelAdapterCache<'a>,
    origin: Option<&'a ProjectileOrigin>,
}

impl Drop for ParallelWorldAdapter<'_> {
    fn drop(&mut self) {
        for entity in self.reserved_entities.drain(..) {
            self.cache.summon_placements.release(entity);
            let _ = self.cache.entities.delete(entity);
        }
    }
}

pub struct ParallelTowerCooldownAccess<'a> {
    cache: &'a ParallelAdapterCache<'a>,
    outcomes: Vec<Outcome>,
    overlay: HashMap<Entity, Fixed64>,
}

pub struct ParallelTowerActiveAbilityAccess<'a> {
    cache: &'a ParallelAdapterCache<'a>,
    outcomes: Mutex<Vec<Outcome>>,
}

impl<'a> ParallelTowerActiveAbilityAccess<'a> {
    pub fn new(cache: &'a ParallelAdapterCache<'a>) -> Self {
        Self {
            cache,
            outcomes: Mutex::new(Vec::new()),
        }
    }

    pub fn finish(self) -> Vec<Outcome> {
        self.outcomes
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn matching_state(
        &self,
        e: EntityHandle,
        ability_id: RStr<'_>,
    ) -> Option<&TowerActiveAbilityState> {
        let entity = ParallelWorldAdapter::handle_to_entity(e)?;
        let state = self.cache.tower.get(entity)?.active_ability.as_ref()?;
        (state.ability_id == ability_id.as_str()).then_some(state)
    }
}

impl TowerActiveAbilityAccess for ParallelTowerActiveAbilityAccess<'_> {
    fn get_tower_ability_active_remaining(&self, e: EntityHandle, ability_id: RStr<'_>) -> Fixed64 {
        self.matching_state(e, ability_id)
            .map(|state| state.active_remaining)
            .unwrap_or(Fixed64::ZERO)
    }

    fn get_tower_ability_activation_serial(&self, e: EntityHandle, ability_id: RStr<'_>) -> u32 {
        self.matching_state(e, ability_id)
            .map(|state| state.activation_serial)
            .unwrap_or(0)
    }

    fn reset_attack_backswing(&self, e: EntityHandle) {
        let Some(entity) = ParallelWorldAdapter::handle_to_entity(e) else {
            return;
        };
        let Some(base_interval) = self.cache.tattack.get(entity).map(|attack| attack.asd.v) else {
            return;
        };
        let stats = UnitStats::from_refs(
            &self.cache.buffs,
            self.cache.is_building.get(entity).is_some(),
        );
        let interval =
            (stats.base_attack_interval(base_interval, entity) / stats.final_attack_speed_mult(entity)).max(Fixed64::from_raw(1));
        self.outcomes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(Outcome::ScriptSetAsdCount {
                entity,
                asd_count: interval,
            });
    }

    fn query_friendly_towers_in_range(
        &self,
        center: Vec2,
        radius: Fixed64,
        exclude: EntityHandle,
    ) -> RVec<EntityHandle> {
        let Some(excluded) = ParallelWorldAdapter::handle_to_entity(exclude) else {
            return RVec::new();
        };
        if radius < Fixed64::ZERO {
            return RVec::new();
        }
        let Some(origin_faction) = self.cache.faction.get(excluded) else {
            return RVec::new();
        };
        let origin_owner = self
            .cache
            .player_owner
            .get(excluded)
            .map(|owner| owner.player_id);
        let radius_squared = radius * radius;
        let mut matches = (
            &self.cache.entities,
            &self.cache.tower,
            &self.cache.pos,
            &self.cache.faction,
        )
            .join()
            .filter_map(|(entity, _, pos, faction)| {
                if entity == excluded || faction.team_id != origin_faction.team_id {
                    return None;
                }
                if let Some(owner) = origin_owner {
                    if self
                        .cache
                        .player_owner
                        .get(entity)
                        .map(|candidate| candidate.player_id)
                        != Some(owner)
                    {
                        return None;
                    }
                }
                let delta = pos.0 - center;
                (delta.x * delta.x + delta.y * delta.y <= radius_squared)
                    .then(|| ParallelWorldAdapter::entity_to_handle(entity))
            })
            .collect::<Vec<_>>();
        matches.sort_by_key(|handle| (handle.id, handle.gen));
        matches.into()
    }

    fn query_first_enemy_in_range(
        &self,
        center: Vec2,
        radius: Fixed64,
        of: EntityHandle,
    ) -> ROption<EntityHandle> {
        query_script_tower_enemy(self.cache, center, radius, of, TowerTargetPriority::First)
    }
}

impl<'a> ParallelTowerCooldownAccess<'a> {
    pub fn new(cache: &'a ParallelAdapterCache<'a>) -> Self {
        Self {
            cache,
            outcomes: Vec::new(),
            overlay: HashMap::new(),
        }
    }

    pub fn finish(self) -> Vec<Outcome> {
        self.outcomes
    }
}

impl TowerCooldownAccess for ParallelTowerCooldownAccess<'_> {
    fn get_tower_internal_cooldown(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = ParallelWorldAdapter::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        self.overlay
            .get(&ent)
            .copied()
            .or_else(|| {
                self.cache
                    .tower
                    .get(ent)
                    .map(|tower| tower.ultimate_cooldown)
            })
            .unwrap_or(Fixed64::ZERO)
    }

    fn start_tower_internal_cooldown(&mut self, e: EntityHandle, duration: Fixed64) {
        if let Some(ent) = ParallelWorldAdapter::handle_to_entity(e) {
            let duration = duration.max(Fixed64::ZERO);
            self.overlay.insert(ent, duration);
            self.outcomes.push(Outcome::ScriptSetTowerInternalCooldown {
                entity: ent,
                duration,
            });
        }
    }
}

impl<'a> ParallelProjectileQuery<'a> {
    pub fn new(cache: &'a ParallelAdapterCache<'a>) -> Self {
        Self { cache, origin: None }
    }

    pub fn with_origin(cache: &'a ParallelAdapterCache<'a>, origin: Option<&'a ProjectileOrigin>) -> Self {
        Self {cache, origin}
    }
}

impl ProjectileQuery for ParallelProjectileQuery<'_> {
    fn enemy_candidates_bounded(
        &self,
        center: Vec2,
        radius: Fixed64,
        of: EntityHandle,
        exclude: ROption<EntityHandle>,
        cap: u16,
    ) -> RVec<EntityHandle> {
        let Some(of_entity) = ParallelWorldAdapter::handle_to_entity(of) else {
            return RVec::new();
        };
        let Some(my_faction) = self.cache.faction.get(of_entity).or_else(||
            self.origin.filter(|origin|origin.source==of_entity && !self.cache.entities.is_alive(of_entity))
                .and_then(|origin|origin.script.as_ref()).and_then(|source|source.faction.as_ref())) else {
            return RVec::new();
        };
        let cap = usize::from(cap);
        if cap == 0 || radius <= Fixed64::ZERO {
            return RVec::new();
        }
        let excluded = match exclude {
            RSome(handle) => ParallelWorldAdapter::handle_to_entity(handle),
            RNone => None,
        };
        // Spatial retrieval stays explicitly bounded. Small overscan absorbs the one
        // exclusion plus allied/stale entries without falling back to a full ECS join.
        let search_cap = cap.saturating_mul(4).saturating_add(1).min(64);
        let spatial = self.cache.searcher.creep.search_nn_bounded(
            vek::Vec2::new(center.x.to_f32_for_render(), center.y.to_f32_for_render()),
            radius.to_f32_for_render(),
            search_cap,
        );
        let radius_squared = radius * radius;
        let mut candidates: Vec<(Fixed64, EntityHandle)> = spatial
            .into_iter()
            .filter_map(|candidate| {
                if Some(candidate.e) == excluded {
                    return None;
                }
                let faction = self.cache.faction.get(candidate.e)?;
                if faction.team_id == my_faction.team_id {
                    return None;
                }
                let pos = self.cache.pos.get(candidate.e)?.0;
                let distance_squared = pos.distance_squared(center);
                if distance_squared > radius_squared {
                    return None;
                }
                Some((
                    distance_squared,
                    ParallelWorldAdapter::entity_to_handle(candidate.e),
                ))
            })
            .collect();
        candidates.sort_by(|(distance_a, entity_a), (distance_b, entity_b)| {
            distance_a
                .partial_cmp(distance_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| (entity_a.id, entity_a.gen).cmp(&(entity_b.id, entity_b.gen)))
        });
        candidates.truncate(cap);
        candidates
            .into_iter()
            .map(|(_, entity)| entity)
            .collect::<Vec<_>>()
            .into()
    }
}

fn select_script_tower_target(
    priority: TowerTargetPriority,
    candidates: &[DisIndex],
    creeps: &ReadStorage<'_, Creep>,
    cprops: &ReadStorage<'_, CProperty>,
) -> Option<Entity> {
    candidates
        .iter()
        .min_by(|a, b| compare_script_tower_targets(priority, a, b, creeps, cprops))
        .map(|candidate| candidate.e)
}

fn query_script_tower_enemy(
    cache: &ParallelAdapterCache<'_>,
    center: Vec2,
    radius: Fixed64,
    of: EntityHandle,
    priority: TowerTargetPriority,
) -> ROption<EntityHandle> {
    query_script_tower_enemy_with_source(cache,center,radius,of,priority,None)
}

fn query_script_tower_enemy_with_source(
    cache:&ParallelAdapterCache<'_>,center:Vec2,radius:Fixed64,of:EntityHandle,
    priority:TowerTargetPriority,source:Option<&ProjectileScriptSource>,
) -> ROption<EntityHandle> {
    let Some(of_ent) = ParallelWorldAdapter::handle_to_entity(of) else {
        return RNone;
    };
    let my_team = match cache.faction.get(of_ent).or_else(||source.and_then(|source|source.faction.as_ref())) {
        Some(faction) => faction.team_id,
        None => return RNone,
    };
    let center_f = vek::Vec2::new(center.x.to_f32_for_render(), center.y.to_f32_for_render());
    let radius_f = radius.to_f32_for_render();
    let mut candidates = Vec::new();
    for candidate in
        cache
            .searcher
            .creep
            .search_nn(center_f, radius_f, cache.searcher.creep.count().max(1))
    {
        let Some(faction) = cache.faction.get(candidate.e) else {
            continue;
        };
        let target_creep = cache.creep.get(candidate.e);
        let camo_visible = match (cache.tower.get(of_ent).or_else(||source.and_then(|source|source.tower.as_ref())), target_creep) {
            (Some(tower), Some(creep)) => tower_can_target_creep(tower, creep),
            _ => true,
        };
        if faction.team_id != my_team && target_creep.is_some() && camo_visible {
            candidates.push(candidate);
        }
    }
    select_script_tower_target(priority, &candidates, &cache.creep, &cache.cprop)
        .map(ParallelWorldAdapter::entity_to_handle)
        .map_or(RNone, RSome)
}

fn compare_script_tower_targets(
    priority: TowerTargetPriority,
    a: &DisIndex,
    b: &DisIndex,
    creeps: &ReadStorage<'_, Creep>,
    cprops: &ReadStorage<'_, CProperty>,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let primary = match priority {
        TowerTargetPriority::First => {
            let ar = creeps
                .get(a.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            let br = creeps
                .get(b.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            ar.partial_cmp(&br).unwrap_or(Ordering::Equal)
        }
        TowerTargetPriority::Last => {
            let ar = creeps
                .get(a.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            let br = creeps
                .get(b.e)
                .map(|c| c.path_remaining_distance)
                .unwrap_or_else(|| Fixed64::from_i32(1_000_000));
            br.partial_cmp(&ar).unwrap_or(Ordering::Equal)
        }
        TowerTargetPriority::Nearest => a.dis.partial_cmp(&b.dis).unwrap_or(Ordering::Equal),
        TowerTargetPriority::Farthest => b.dis.partial_cmp(&a.dis).unwrap_or(Ordering::Equal),
        TowerTargetPriority::HighestHealth => {
            let ahp = cprops.get(a.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            let bhp = cprops.get(b.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            bhp.partial_cmp(&ahp).unwrap_or(Ordering::Equal)
        }
        TowerTargetPriority::LowestHealth => {
            let ahp = cprops.get(a.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            let bhp = cprops.get(b.e).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
            ahp.partial_cmp(&bhp).unwrap_or(Ordering::Equal)
        }
    };
    primary.then_with(|| a.e.id().cmp(&b.e.id()))
}

impl<'a> ParallelWorldAdapter<'a> {
    pub fn new(cache: &'a ParallelAdapterCache<'a>, invocation_entity: Entity) -> Self {
        Self::new_with_random_ordinal(cache, invocation_entity, 0)
    }

    pub fn new_with_random_ordinal(
        cache: &'a ParallelAdapterCache<'a>,
        _invocation_entity: Entity,
        stable_invocation_ordinal: u64,
    ) -> Self {
        Self {
            cache,
            invocation_rng_op: 0,
            random_request_base: stable_invocation_ordinal.saturating_mul(1024),
            outcomes: Vec::new(),
            overlay_pos: HashMap::new(),
            cast_succeeded: false,
            cast_rejection: None,
            hook_panicked: false,
            mana_transaction: false,
            mana_views: HashMap::new(),
            mana_dirty: Vec::new(),
            mana_originals: HashMap::new(),
            overlay_facing: HashMap::new(),
            overlay_asd_count: HashMap::new(),
            projectile_hit_generation: None,
            projectile_origin: None,
            projectile_source_buffs: None,
            projectile_team_key: None,
            capture_cast_areas: false,
            cast_areas: Vec::new(),
            allocation_allowed: true,
            allocation_requested: false,
            reserved_entities: Vec::new(),
        }
    }

    pub fn finish(self) -> Vec<Outcome> {
        let mut this = self;
        this.reserved_entities.clear(); // ownership transfers to outcomes
        std::mem::take(&mut this.outcomes)
    }

    pub fn mark_cast_succeeded(&mut self) { self.cast_succeeded = true; }

    /// How many outcomes this invocation has produced so far.
    pub(crate) fn outcome_count(&self) -> usize { self.outcomes.len() }

    /// Puts `outcomes` before everything produced since `at` (a rule that precedes a cast's own effects).
    pub(crate) fn insert_outcomes(&mut self, at: usize, outcomes: Vec<Outcome>) {
        let at = at.min(self.outcomes.len());
        self.outcomes.splice(at..at, outcomes);
    }

    pub(crate) fn note_cast_rejection(&mut self, code: &'static str) { self.cast_rejection = Some(code); }

    pub(crate) fn cast_rejection(&self) -> Option<&'static str> { self.cast_rejection }

    pub(crate) fn stage_recovery_channel(&mut self,caster:Entity,spec:super::super::recovery_channel::Spec)->bool {
        let Some(owner)=self.cache.player_owner.get(caster).map(|owner|owner.player_id).filter(|owner|*owner>0)else{return false;};
        let Some(state)=self.cache.moba.as_ref()else{return false;};
        if !spec.valid() || !self.cache.entities.is_alive(caster) || self.cache.hero.get(caster).is_none()
            || self.cache.cprop.get(caster).is_none_or(|props|props.hp<=Fixed64::ZERO)
            || self.cache.buffs.is_silenced(caster) || self.cache.buffs.is_stunned(caster)
            || state.phase!=crate::runtime::MobaMatchPhase::Playing || state.is_recalling(caster)
            || !state.heroes.iter().any(|hero|hero.player_id==owner&&hero.entity==Some(caster))
            || self.cache.pos.get(caster).is_none_or(|pos|pos.0.x.raw().unsigned_abs()>omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
                ||pos.0.y.raw().unsigned_abs()>omoba_sim::terrain::MAX_GEOMETRY_RAW as u64) {return false;}
        self.outcomes.push(Outcome::ScriptRecoveryChannel {caster,owner,amount:spec.amount,duration:spec.duration,interval:spec.interval});true
    }

    /// G-2 pulse channel start. Range and target are checked here so the owner
    /// sees OUT_OF_RANGE/INVALID_TARGET; settlement rechecks on live state.
    pub(crate) fn stage_ability_channel(&mut self,caster:Entity,skill_id:&str,level:u8,target:&Target,
        spec:super::super::ability_channel::Spec)->bool {
        use super::super::ability_channel::Anchor;
        use super::super::command_outcome as code;
        use omoba_content_model::ChannelAnchor;
        let Some(owner)=self.cache.player_owner.get(caster).map(|owner|owner.player_id).filter(|owner|*owner>0)else{return false;};
        // A replica has no MobaMatch: it admits from the same caster state, so
        // its own-team cooldown and channel stay deterministic (split16).
        if !spec.valid() || !self.cache.entities.is_alive(caster) || self.cache.hero.get(caster).is_none()
            || self.cache.cprop.get(caster).is_none_or(|props|props.hp<=Fixed64::ZERO)
            || self.cache.moba.as_ref().is_some_and(|state|state.phase!=crate::runtime::MobaMatchPhase::Playing || state.is_recalling(caster)
                || !state.heroes.iter().any(|hero|hero.player_id==owner&&hero.entity==Some(caster))) {return false;}
        if self.cache.buffs.is_silenced(caster) {self.note_cast_rejection(code::SILENCED);return false;}
        let Some(origin)=self.cache.pos.get(caster).map(|pos|pos.0)else{return false;};
        let (anchor,point)=match (spec.anchor,target) {
            (ChannelAnchor::SelfOrigin,_)=>(Anchor::Origin,origin),
            (ChannelAnchor::Point,Target::Point(point))=>(Anchor::Point(*point),*point),
            (ChannelAnchor::Unit,Target::Entity(handle))=>{
                let unit=Self::handle_to_entity(*handle).filter(|unit|*unit!=caster && self.cache.entities.is_alive(*unit)
                    && self.cache.cprop.get(*unit).is_some_and(|props|props.hp>Fixed64::ZERO)
                    && match (self.cache.faction.get(caster),self.cache.faction.get(*unit)) {(Some(own),Some(other))=>own.is_hostile_to(other),_=>false});
                let Some(unit)=unit else {self.note_cast_rejection(code::INVALID_TARGET);return false;};
                let Some(point)=self.cache.pos.get(unit).map(|pos|pos.0) else {self.note_cast_rejection(code::INVALID_TARGET);return false;};
                (Anchor::Unit(unit),point)
            }
            _=>{self.note_cast_rejection(code::INVALID_TARGET);return false;}
        };
        if (point-origin).length()>spec.range && anchor!=Anchor::Origin {self.note_cast_rejection(code::OUT_OF_RANGE);return false;}
        self.outcomes.push(Outcome::ScriptAbilityChannel {caster,owner,skill_id:skill_id.to_owned(),level,spec,anchor});true
    }

    pub fn set_projectile_origin(&mut self, origin: Option<ProjectileOrigin>) {
        self.projectile_team_key=origin.as_ref().and_then(|origin|origin.script.as_ref())
            .and_then(|source|source.faction.as_ref()).map(|faction|format!("team:{}",faction.team_id));
        self.projectile_source_buffs = origin.as_ref()
            .filter(|origin|!self.cache.entities.is_alive(origin.source))
            .and_then(|origin|origin.script.as_ref().map(|source| {
                let mut buffs=BuffStore::default();
                for (id,remaining,payload) in &source.buffs {buffs.add(origin.source,id,*remaining,payload.clone());}
                buffs
            }));
        self.projectile_origin=origin;
    }

    fn retired_source(&self, entity: Entity) -> Option<&ProjectileScriptSource> {
        self.projectile_origin.as_ref().filter(|origin|origin.source==entity && !self.cache.entities.is_alive(entity))
            .and_then(|origin|origin.script.as_deref())
    }
    pub(crate) fn projectile_script_id(&self, entity:Entity)->Option<String> {
        self.retired_source(entity).map(|source|source.unit_id.clone())
    }
    fn source_attack(&self, entity:Entity)->Option<&TAttack> {
        match self.retired_source(entity) {Some(source)=>source.attack.as_ref(),None=>self.cache.tattack.get(entity)}
    }
    fn source_property(&self, entity:Entity)->Option<&CProperty> {
        match self.retired_source(entity) {Some(source)=>source.property.as_ref(),None=>self.cache.cprop.get(entity)}
    }
    fn source_unit(&self, entity:Entity)->Option<&Unit> {
        match self.retired_source(entity) {Some(source)=>source.unit.as_ref(),None=>self.cache.unit.get(entity)}
    }
    pub(crate) fn source_ability_levels(&self,entity:Entity)->Vec<(String,u8)> {
        if let Some(source)=self.retired_source(entity) {return source.ability_levels.clone();}
        let Some(hero)=self.cache.hero.get(entity).filter(|h|h.ability_levels.len()<=64) else{return Vec::new();};
        let mut levels:Vec<_>=hero.ability_levels.iter().filter_map(|(id,rank)|
            u8::try_from(*rank).ok().filter(|rank|*rank>0).map(|rank|(id.clone(),rank))).collect();
        levels.sort();levels
    }
    fn source_faction(&self, entity:Entity)->Option<&Faction> {
        match self.retired_source(entity) {Some(source)=>source.faction.as_ref(),None=>self.cache.faction.get(entity)}
    }
    fn source_tower(&self, entity:Entity)->Option<&Tower> {
        match self.retired_source(entity) {Some(source)=>source.tower.as_ref(),None=>self.cache.tower.get(entity)}
    }
    fn source_is_building(&self, entity:Entity)->bool {
        match self.retired_source(entity) {Some(source)=>source.building,None=>self.cache.is_building.get(entity).is_some()}
    }
    fn source_buffs(&self, entity:Entity)->&BuffStore {
        if self.retired_source(entity).is_some() {self.projectile_source_buffs.as_ref().unwrap_or(&self.cache.buffs)}
        else {&self.cache.buffs}
    }
    fn attribute_projectile_outcome(&self, outcome:Outcome, source:Entity)->Outcome {
        match self.projectile_origin.as_ref().filter(|origin|origin.source==source) {
            Some(origin)=>Outcome::ProjectileContext {origin:origin.clone(),outcome:Box::new(outcome)},
            None=>outcome,
        }
    }
    fn is_summon(&self, entity:Entity)->bool {
        self.source_unit(entity).is_some_and(|unit|matches!(unit.unit_type,UnitType::Summon))
    }

    /// A preference from this summon owner's active command, never target admission.
    fn summon_focus_target(&self,source:Entity)->Option<Entity> {
        // An owner script's assignment (`summon_target` buff on the summon) wins
        // over the template priority while that exact life is alive.
        if let Some(target)=self.summon_assigned_target(source) {return Some(target);}
        let stats=omoba_template_ids::summon_by_name(&self.source_unit(source)?.id)
            .and_then(omoba_template_ids::active_summon_stats)?;
        if stats.target_priority!=omoba_template_ids::SummonTargetPriority::OwnerFocus {return None;}
        let summon=self.cache.summoned.as_ref()?.get(source)?;
        let owner=summon.summoner;
        if summon.is_expired() || !self.cache.entities.is_alive(owner)
            || self.cache.hero.get(owner).is_none()
            || !self.cache.cprop.get(owner).is_some_and(|p|p.hp>Fixed64::ZERO) {return None;}
        let player=self.cache.player_owner.get(owner)?.player_id;
        if player==0 || self.cache.player_owner.get(source)?.player_id!=player {return None;}
        let faction=self.source_faction(source)?;let owner_faction=self.cache.faction.get(owner)?;
        if faction.team_id<=0 || faction.team_id!=owner_faction.team_id || faction.faction_id!=owner_faction.faction_id {return None;}
        match self.cache.commands.as_ref()?.get(owner)?.active? {
            HeroCommand::AttackTarget {target,..}=>Some(target),
            _=>None,
        }
    }

    fn summon_assigned_target(&self,source:Entity)->Option<Entity> {
        let row=self.cache.buffs.get(source,omb_script_abi::buff_ids::SUMMON_TARGET_BUFF)?.payload
            .get(omb_script_abi::buff_ids::SUMMON_TARGET_BUFF)?.as_array()?;
        if row.len()!=2 {return None;}
        let id=u32::try_from(row[0].as_u64()?).ok()?;let generation=i32::try_from(row[1].as_i64()?).ok()?;
        let target=self.cache.entities.entity(id);
        (target.gen().id()==generation && self.cache.entities.is_alive(target)
            && self.cache.cprop.get(target).is_some_and(|p|p.hp>Fixed64::ZERO)).then_some(target)
    }

    fn summon_target_allowed(&self,source:Entity,target:Entity,center:Vec2,radius:Fixed64)->bool {
        use crate::runtime::native::combat_acquisition::DisclosureContext;
        // An already committed projectile is independent of its source's later
        // death/stun. Ordinary summon acquisition keeps all source admission.
        let committed=self.projectile_origin.as_ref().is_some_and(|origin|origin.source==source);
        if radius<=Fixed64::ZERO || source==target
            || (!committed && (!self.cache.entities.is_alive(source) || self.cache.buffs.is_stunned(source)
                || !self.cache.cprop.get(source).is_some_and(|h|h.hp>Fixed64::ZERO)))
            || !self.cache.entities.is_alive(target)
            || !self.cache.cprop.get(target).is_some_and(|h|h.hp>Fixed64::ZERO) {return false;}
        let Some((faction,other))=self.source_faction(source).zip(self.cache.faction.get(target)) else {return false;};
        let Some(position)=self.cache.pos.get(target).map(|p|p.0) else {return false;};
        let disclosure=DisclosureContext::resolve(self.cache.filtered,self.cache.moba.is_some(),self.cache.visibility.as_deref());
        if !faction.is_hostile_to(other) || position.distance_squared(center)>radius*radius
            || !disclosure.allows(Some(faction),crate::runtime::selective::CanonicalEntityKey {
                id:target.id(),generation:target.gen().id() as u32}) {return false;}
        let attackable=if let Some(state)=self.cache.moba.as_deref() {
            state.current_match_target_attackable(target) && state.ward_automatic_acquisition_allowed(target)
        }else {self.cache.attack_gate.as_deref().is_none_or(|gate|
            gate.allows(target) && !gate.ward_automatic_disabled.contains(&target))};
        attackable && self.cache.walls.as_ref().is_none_or(|walls|
            walls.line_of_sight(center,position)==crate::runtime::LosResult::Clear)
    }
    pub fn cast_succeeded(&self) -> bool { self.cast_succeeded && !self.hook_panicked }
    pub fn capture_formal_cast_areas(&mut self) { self.capture_cast_areas = true; }
    pub fn successful_cast_areas(&self) -> &[(Vec2, Fixed64, Fixed64)] {
        if self.cast_succeeded() { &self.cast_areas } else { &[] }
    }

    /// A relocation submitted by this successful invocation, not its requested
    /// target or a later world snapshot. Other entities' movement is excluded.
    pub fn successful_caster_relocation(&self, caster: Entity) -> Option<Vec2> {
        if !self.cast_succeeded() { return None; }
        let point = *self.overlay_pos.get(&caster)?;
        (self.cache.pos.get(caster).is_some_and(|origin| origin.0 != point)).then_some(point)
    }
    pub fn mark_hook_panicked(&mut self) { self.hook_panicked = true; }
    pub fn hook_panicked(&self) -> bool { self.hook_panicked }
    pub fn has_mana_changes(&self) -> bool { !self.mana_dirty.is_empty() }
    /// Only the serial dispatcher can enable read/modify/write resource admission.
    pub fn begin_mana_transaction(&mut self, ledger: &HashMap<Entity, Hero>) {
        self.mana_transaction = true;
        self.mana_views = ledger.iter().filter_map(|(entity,hero)|
            hero.mana_pool.clone().map(|pool|(*entity,pool))).collect();
        self.mana_originals = self.mana_views.clone();
    }
    pub fn stage_mana_pool(&mut self, entity: Entity, pool: crate::runtime::ability_runtime::ManaPool) {
        if !self.mana_originals.contains_key(&entity) {
            if let Some(original)=self.mana_views.get(&entity).cloned()
                .or_else(||self.cache.hero.get(entity).and_then(|hero|hero.mana_pool.clone())) {
                self.mana_originals.insert(entity,original);
            }
        }
        self.mana_views.insert(entity,pool);
        if !self.mana_dirty.contains(&entity) { self.mana_dirty.push(entity); }
    }
    pub fn stage_mana_spent(&mut self, entity:Entity, amount:Fixed64, ability_id:String) {
        if amount>Fixed64::ZERO {
            self.outcomes.push(Outcome::ScriptManaSpent {entity,amount,ability_id});
        }
    }
    /// Deterministic absolute commits; notifications are deferred until outcomes
    /// settle, and their callbacks run in a later dispatch (never recursively).
    pub fn commit_mana_transaction(&mut self) -> Vec<(Entity,crate::runtime::ability_runtime::ManaPool)> {
        self.mana_dirty.sort_by_key(|e|(e.id(),e.gen().id()));
        let dirty=std::mem::take(&mut self.mana_dirty);
        let mut changes=Vec::new();
        let result=dirty.into_iter().map(|entity| {
            let pool=self.mana_views[&entity].clone();
            let expected=self.mana_originals.insert(entity,pool.clone()).expect("managed mana baseline");
            changes.push(crate::comp::ManaSettlementChange {entity,expected,pool:pool.clone()});
            (entity,pool)
        }).collect::<Vec<_>>();
        if !changes.is_empty() {
            let mut notifications=Vec::new();
            let changed:std::collections::HashSet<_>=changes.iter().map(|row|row.entity).collect();
            self.outcomes.retain(|effect|match effect {
                Outcome::ScriptManaGained {entity,amount} if changed.contains(entity)=>{
                    notifications.push(crate::comp::ManaSettlementNotification::Gained {entity:*entity,amount:*amount});false
                },
                Outcome::ScriptManaSpent {entity,amount,ability_id} if changed.contains(entity)=>{
                    notifications.push(crate::comp::ManaSettlementNotification::Spent {entity:*entity,amount:*amount,ability_id:ability_id.clone()});false
                },
                _=>true,
            });
            self.outcomes.push(Outcome::ScriptManaCommit {changes,notifications});
        }
        result
    }
    fn managed_mana_entity(&self, e:EntityHandle) -> Option<Entity> {
        let entity=Self::handle_to_entity(e)?;
        if !self.cache.entities.is_alive(entity)
            || !self.cache.cprop.get(entity).is_some_and(|health|health.hp>Fixed64::ZERO)
            || self.cache.hero.get(entity)?.mana_pool.is_none() {return None;}
        Some(entity)
    }

    /// Each queued event owns a fresh adapter. Failed managed casts discard
    /// both unit pre-hooks and ability outcomes, including their local overlays.
    pub fn discard_cast(&mut self) {
        for entity in self.reserved_entities.drain(..) {
            self.cache.summon_placements.release(entity);
            let _ = self.cache.entities.delete(entity);
        }
        self.outcomes.clear();
        self.overlay_pos.clear();
        self.overlay_facing.clear();
        self.overlay_asd_count.clear();
        self.cast_succeeded = false;
        self.hook_panicked = false;
        self.mana_transaction = false;
        self.mana_views.clear();
        self.mana_dirty.clear();
        self.mana_originals.clear();
    }

    pub fn commit_mana(&mut self, entity: Entity, pool: crate::runtime::ability_runtime::ManaPool) {
        self.outcomes.push(Outcome::ScriptSetMana { entity, pool });
    }

    pub fn start_cooldown(&mut self, entity: Entity, ability_id: String, duration: Fixed64) {
        self.outcomes.push(Outcome::ScriptStartCooldown {
            entity,
            ability_id,
            duration,
        });
    }

    pub fn consume_charge(&mut self, entity: Entity, ability_id: String) {
        self.outcomes.push(Outcome::ScriptConsumeCharge { entity, ability_id });
    }

    pub fn set_projectile_hit_generation(&mut self, generation: u8) {
        self.projectile_hit_generation = Some(generation);
    }

    #[inline]
    pub fn entity_to_handle(e: Entity) -> EntityHandle {
        EntityHandle {
            id: e.id(),
            gen: e.gen().id() as u32,
        }
    }

    #[inline]
    pub fn handle_to_entity(h: EntityHandle) -> Option<Entity> {
        if !h.is_valid() {
            return None;
        }
        let gen_i = h.gen as i32;
        if gen_i == 0 {
            return None;
        }
        Some(Entity::new(h.id, Generation::new(gen_i)))
    }

    fn angle_to_rad_f32(angle: Angle) -> f32 {
        angle.ticks() as f32 / omoba_sim::trig::TAU_TICKS as f32 * std::f32::consts::TAU
    }

    fn push_tower_fire_fx(&mut self, owner: Entity, dir_rad: f32) {
        if self.cache.tower.get(owner).is_none() {
            return;
        }
        if self.outcomes.iter().any(|outcome| {
            matches!(
                outcome,
                Outcome::ScriptTowerFireFx { entity, .. } if *entity == owner
            )
        }) {
            return;
        }
        self.outcomes.push(Outcome::ScriptTowerFireFx {
            entity: owner,
            dir_rad,
        });
    }
}

impl<'a> GameWorld for ParallelWorldAdapter<'a> {
    fn get_pos(&self, e: EntityHandle) -> ROption<Vec2> {
        let Some(ent) = Self::handle_to_entity(e) else {
            return RNone;
        };
        if let Some(p) = self.overlay_pos.get(&ent) {
            return RSome(*p);
        }
        match self.retired_source(ent) {Some(source)=>source.pos,None=>self.cache.pos.get(ent).map(|p|p.0)}
            .map_or(RNone, RSome)
    }

    fn get_hp(&self, e: EntityHandle) -> ROption<Fixed64> {
        let Some(ent) = Self::handle_to_entity(e) else {
            return RNone;
        };
        if let Some(p) = self.source_property(ent) {
            return RSome(p.hp);
        }
        self.source_unit(ent)
            .map(|u| Fixed64::from_i32(u.current_hp))
            .map_or(RNone, RSome)
    }

    fn get_max_hp(&self, e: EntityHandle) -> ROption<Fixed64> {
        let Some(ent) = Self::handle_to_entity(e) else {
            return RNone;
        };
        if let Some(p) = self.source_property(ent) {
            return RSome(p.mhp);
        }
        self.source_unit(ent)
            .map(|u| Fixed64::from_i32(u.max_hp))
            .map_or(RNone, RSome)
    }

    fn is_alive(&self, e: EntityHandle) -> bool {
        Self::handle_to_entity(e)
            .map(|ent| self.cache.entities.is_alive(ent))
            .unwrap_or(false)
    }

    fn faction_of(&self, e: EntityHandle) -> ROption<RStr<'_>> {
        let Some(entity)=Self::handle_to_entity(e) else {return RNone;};
        if self.retired_source(entity).is_some() {
            return self.projectile_team_key.as_deref().map_or(RNone,|key|RSome(RStr::from_str(key)));
        }
        self.cache.faction.get(entity).and_then(|f|self.cache.team_keys.get(&f.team_id))
            .map_or(RNone,|key|RSome(RStr::from_str(key)))
    }

    fn unit_id_of(&self, e: EntityHandle) -> ROption<RStr<'_>> {
        let Some(entity)=Self::handle_to_entity(e) else {return RNone;};
        self.retired_source(entity).map(|source|source.unit_id.as_str())
            .or_else(||self.cache.tags.get(entity).map(|tag|tag.unit_id.as_str()))
            // MOBA heroes carry no script tag: report their hero template id so
            // scripts can identify heroes (e.g. per-target-class bonuses).
            .or_else(||self.cache.hero.get(entity).map(|hero|hero.id.as_str()))
            .map_or(RNone,|id|RSome(RStr::from_str(id)))
    }

    fn query_enemies_in_range(
        &self,
        center: Vec2,
        radius: Fixed64,
        of: EntityHandle,
    ) -> RVec<EntityHandle> {
        let Some(of_ent) = Self::handle_to_entity(of) else {
            return RVec::new();
        };
        if self.is_summon(of_ent) {
            let mut found:Vec<_>=(&self.cache.entities,&self.cache.pos).join()
                .filter(|(e,_)|self.summon_target_allowed(of_ent,*e,center,radius))
                .map(|(e,p)|(p.0.distance_squared(center).raw(),e)).collect();
            found.sort_by_key(|(d,e)|(*d,e.id(),e.gen().id()));
            return found.into_iter().map(|(_,e)|Self::entity_to_handle(e)).collect::<Vec<_>>().into();
        }
        let my_team = match self.source_faction(of_ent) {
            Some(f) => f.team_id,
            None => return RVec::new(),
        };
        let r2 = radius * radius;
        let source_tower = self.source_tower(of_ent);
        let mut out = RVec::new();
        for (ent, pos, fac) in (&self.cache.entities, &self.cache.pos, &self.cache.faction).join() {
            let camo_visible = match (source_tower, self.cache.creep.get(ent)) {
                (Some(tower), Some(creep)) => tower_can_target_creep(tower, creep),
                _ => true,
            };
            if fac.team_id != my_team && pos.0.distance_squared(center) <= r2 && camo_visible {
                out.push(Self::entity_to_handle(ent));
            }
        }
        out
    }

    fn set_pos(&mut self, e: EntityHandle, p: Vec2) {
        let Some(ent) = Self::handle_to_entity(e) else {
            return;
        };
        self.overlay_pos.insert(ent, p);
        self.outcomes.push(Outcome::ScriptSetPos {
            entity: ent,
            pos: p,
        });
    }

    fn advance_with_collision(&mut self, e: EntityHandle, target: Vec2, step: Fixed64) -> Vec2 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return target;
        };
        let pos = match self
            .overlay_pos
            .get(&ent)
            .copied()
            .or_else(|| self.cache.pos.get(ent).map(|p| p.0))
        {
            Some(p) => p,
            None => return target,
        };
        // Voluntary collision-aware movement obeys the same owner control
        // state as ordinary hero movement. Direct set_pos remains available
        // for explicit authority/forced relocation; do not silently drop it.
        if self.cache.buffs.is_rooted(ent) { return pos; }
        // Host movement already ran before scripts. An active destination owns
        // this summon's movement; autonomous pursuit must not add another step.
        if self.is_summon(ent) && (self.cache.move_targets.as_ref().is_some_and(|targets|targets.get(ent).is_some())
            || self.cache.summon_host_movement.as_ref().is_some_and(|record|record.owns(self.cache.tick.0,ent))) {
            return pos;
        }
        let radius = self
            .cache
            .collision
            .get(ent)
            .map(|r| r.0)
            .unwrap_or(Fixed64::from_i32(30));
        if self.is_summon(ent) {
            return crate::tick::hero_command_tick::static_step_toward(pos,target,step,radius,&self.cache.blocked);
        }
        let (new_pos, _) = crate::tick::hero_move_tick::advance_with_collision(
            pos,
            target,
            step,
            radius,
            &self.cache.searcher,
            &self.cache.collision,
            ent,
            &self.cache.blocked,
        );
        new_pos
    }

    fn deal_damage(
        &mut self,
        target: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
        profile: AbiDamageProfile,
        source: ROption<EntityHandle>,
    ) {
        let source = match source {
            ROption::RSome(source) => source,
            ROption::RNone => {
                log::error!(
                    "reject direct damage without source identity mask={:#x}",
                    profile.bits()
                );
                return;
            }
        };
        let Some(valid_profile) = AbiDamageProfile::from_bits(profile.bits()) else {
            log::error!(
                "reject direct damage from source={} with unknown damage profile mask={:#x}",
                source.id,
                profile.bits()
            );
            return;
        };
        let (Some(target), Some(source)) = (
            Self::handle_to_entity(target),
            Self::handle_to_entity(source),
        ) else {
            return;
        };
        let pos = self
            .cache
            .pos
            .get(target)
            .map(|pos| pos.0)
            .unwrap_or_default();
        let (phys, magi, real) = match kind {
            DamageKind::Physical => (amount, Fixed64::ZERO, Fixed64::ZERO),
            DamageKind::Magical => (Fixed64::ZERO, amount, Fixed64::ZERO),
            DamageKind::Pure => (Fixed64::ZERO, Fixed64::ZERO, amount),
        };
        let outcome = self.attribute_projectile_outcome(Outcome::RawDamage {
            pos,
            phys,
            magi,
            real,
            source,
            target,
            damage_profile: valid_profile.bits(),
            origin: None,
        },source);
        self.outcomes.push(outcome);
    }

    fn heal(&mut self, target: EntityHandle, amount: Fixed64) {
        if let Some(ent) = Self::handle_to_entity(target) {
            self.outcomes.push(Outcome::ScriptHeal {
                target: ent,
                amount,
            });
        }
    }

    fn add_buff(&mut self, target: EntityHandle, buff_id: RStr<'_>, duration: Fixed64) {
        if let Some(ent) = Self::handle_to_entity(target) {
            self.outcomes.push(Outcome::AddBuff {
                target: ent,
                buff_id: buff_id.as_str().to_string(),
                duration,
                payload: serde_json::Value::Null,
            });
        }
    }

    fn remove_buff(&mut self, target: EntityHandle, buff_id: RStr<'_>) {
        if let Some(ent) = Self::handle_to_entity(target) {
            self.outcomes.push(Outcome::ScriptRemoveBuff {
                target: ent,
                buff_id: buff_id.as_str().to_string(),
            });
        }
    }

    fn has_buff(&self, target: EntityHandle, buff_id: RStr<'_>) -> bool {
        Self::handle_to_entity(target)
            .map(|ent| self.source_buffs(ent).has(ent, buff_id.as_str()))
            .unwrap_or(false)
    }

    fn add_stat_buff(
        &mut self,
        target: EntityHandle,
        buff_id: RStr<'_>,
        duration: Fixed64,
        modifiers_json: RStr<'_>,
    ) {
        if let Some(ent) = Self::handle_to_entity(target) {
            let payload =
                serde_json::from_str(modifiers_json.as_str()).unwrap_or(serde_json::Value::Null);
            self.outcomes.push(Outcome::AddBuff {
                target: ent,
                buff_id: buff_id.as_str().to_string(),
                duration,
                payload,
            });
        }
    }

    fn spawn_summoned_unit(
        &mut self,
        pos: Vec2,
        unit_type: RStr<'_>,
        owner: EntityHandle,
        duration: Fixed64,
    ) -> EntityHandle {
        let Some(owner) = Self::handle_to_entity(owner).filter(|e|
            self.cache.entities.is_alive(*e)
                && self.cache.cprop.get(*e).is_some_and(|p| p.hp > Fixed64::ZERO)) else {
            return EntityHandle::INVALID;
        };
        let Some(faction) = self.cache.faction.get(owner).cloned() else {return EntityHandle::INVALID;};
        let Some(stats) = omoba_template_ids::summon_by_name(unit_type.as_str())
            .and_then(omoba_template_ids::active_summon_stats) else {return EntityHandle::INVALID;};
        if duration < Fixed64::ZERO || duration > Fixed64::from_i32(3600) || stats.hp <= Fixed64::ZERO
            || stats.attack_interval <= Fixed64::ZERO || stats.attack_range <= Fixed64::ZERO
            || stats.move_speed < Fixed64::ZERO || stats.collision_radius < Fixed64::ZERO
            || stats.vision_radius<Fixed64::ZERO || stats.vision_radius>Fixed64::from_i32(10_000)
            || self.reserved_entities.len() >= 128 {return EntityHandle::INVALID;}
        self.allocation_requested = true;
        if !self.allocation_allowed {return EntityHandle::INVALID;}
        let mut occupied:Vec<_>=(&self.cache.entities,&self.cache.pos,&self.cache.collision,&self.cache.faction)
            .join().filter(|(_,_,_,f)|f.team_id==faction.team_id).map(|(_,p,r,_)|(p.0,r.0)).collect();
        let mut reservations=self.cache.summon_placements.0.lock().unwrap_or_else(|p|p.into_inner());
        if reservations.len()>=4096 {return EntityHandle::INVALID;}
        occupied.extend(reservations.values().filter(|(team,_,_)|*team==faction.team_id)
            .map(|(_,pos,radius)|(*pos,*radius)));
        let anchor=self.overlay_pos.get(&owner).copied().or_else(||self.cache.pos.get(owner).map(|p|p.0));
        let Some(pos)=crate::runtime::native::summon_navigation::placement(pos,anchor,stats.collision_radius,
            &self.cache.blocked,&occupied) else {return EntityHandle::INVALID;};
        let entity = self.cache.entities.create();
        reservations.insert(entity,(faction.team_id,pos,stats.collision_radius));
        self.reserved_entities.push(entity);
        self.overlay_pos.insert(entity, pos);
        self.outcomes.push(Outcome::SpawnSummoned {entity, pos, unit_id:unit_type.as_str().into(),
            owner, faction, player_owner:self.cache.player_owner.get(owner).map(|p| p.player_id), duration});
        Self::entity_to_handle(entity)
    }

    fn spawn_projectile_ex(&mut self, spec: ProjectileSpec) -> EntityHandle {
        let Some(profile) = AbiDamageProfile::from_bits(spec.damage_profile.bits()) else {
            log::error!(
                "reject projectile from source={} with unknown damage profile mask={:#x}",
                spec.owner.id,
                spec.damage_profile.bits()
            );
            return EntityHandle::INVALID;
        };
        let Some(owner_ent) = Self::handle_to_entity(spec.owner) else {
            return EntityHandle::INVALID;
        };
        let basic_attack=matches!(spec.path,PathSpec::AttackHoming {..});
        if basic_attack && (self.projectile_hit_generation.is_some() || self.projectile_origin.is_some()
            || self.capture_cast_areas || !self.is_alive(spec.owner)
            || spec.splash_radius!=Fixed64::ZERO || spec.hit_radius!=Fixed64::ZERO
            || spec.speed<=Fixed64::ZERO || spec.damage<Fixed64::ZERO) {return EntityHandle::INVALID;}
        // Script attackers (summons, script towers) roll by entity id (rules::chance::roll_key).
        let roll_key=crate::runtime::native::rules::chance::roll_key(owner_ent,None);
        let missed=basic_attack && crate::runtime::native::attack_accuracy::source_misses(
            self.cache.rng_seed,self.cache.tick.0 as u32,roll_key,
            UnitStats::from_refs(&self.cache.buffs,false).miss_chance(owner_ent));
        let mut critical=false;
        let mut attack_damage=if missed {Fixed64::ZERO}else{spec.damage};
        if basic_attack&&!missed {
            // Each critical source rolls; the highest multiplier wins (rules::attack).
            let sources=crate::runtime::native::rules::attack::critical_sources(&self.cache.buffs,owner_ent,
                Fixed64::ZERO,Fixed64::from_i32(2));
            if let Some(multiplier)=crate::runtime::native::rules::attack::roll_critical(&sources,
                self.cache.rng_seed,self.cache.tick.0 as u32,roll_key,owner_ent,None) {
                critical=true;
                attack_damage=attack_damage*multiplier;
            }
        }
        let from = spec.from;
        let (target_opt, tpos) = match spec.path {
            PathSpec::Homing { target } | PathSpec::AttackHoming { target } => {
                let Some(target_ent) = Self::handle_to_entity(target) else {
                    return EntityHandle::INVALID;
                };
                if basic_attack && !self.summon_target_allowed(owner_ent,target_ent,from,self.get_tower_range(spec.owner)) {
                    return EntityHandle::INVALID;
                }
                let tp = self.cache.pos.get(target_ent).map(|p| p.0).unwrap_or(from);
                (Some(target_ent), tp)
            }
            PathSpec::Straight { end_pos } => (None, end_pos),
        };
        let fire_angle = omoba_sim::trig::atan2(tpos.y - from.y, tpos.x - from.x);
        self.overlay_facing.insert(owner_ent, fire_angle);
        self.outcomes.push(Outcome::ScriptSetFacing {
            entity: owner_ent,
            facing: fire_angle,
        });
        self.push_tower_fire_fx(owner_ent, Self::angle_to_rad_f32(fire_angle));
        let outcome = self.attribute_projectile_outcome(Outcome::ScriptProjectile {
            basic_attack:basic_attack&&!missed&&spec.damage>Fixed64::ZERO,
            source_missed:basic_attack&&missed,source_critical:critical,
            pos: from,
            owner: owner_ent,
            target: target_opt,
            tpos,
            radius: spec.splash_radius,
            msd: spec.speed,
            damage_phys: attack_damage,
            damage_magi: Fixed64::ZERO,
            damage_real: Fixed64::ZERO,
            damage_profile: profile.bits(),
            slow_factor: if missed {Fixed64::ZERO}else{spec.slow_factor},
            slow_duration: if missed {Fixed64::ZERO}else{spec.slow_duration},
            hit_radius: spec.hit_radius,
            stun_duration: if missed {Fixed64::ZERO}else{spec.stun_duration},
            kind_id: spec.kind_id,
            generation: self
                .projectile_hit_generation
                .map(|generation| generation.saturating_add(1))
                .unwrap_or(0),
        },owner_ent);
        self.outcomes.push(outcome);
        EntityHandle::INVALID
    }

    fn emit_explosion(&mut self, pos: Vec2, radius: Fixed64, duration: Fixed64) {
        if self.capture_cast_areas {
            // Formal casts use a caster-attributed, team-safe cue instead of
            // an anonymous legacy Explosion whose source is only an ordinal.
            if self.cast_areas.len() < 128 { self.cast_areas.push((pos, radius, duration)); }
            return;
        }
        self.outcomes.push(Outcome::Explosion {
            pos,
            radius,
            duration,
        });
    }

    fn despawn(&mut self, e: EntityHandle) {
        if let Some(ent) = Self::handle_to_entity(e) {
            self.outcomes.push(Outcome::EntityRemoved { entity: ent });
        }
    }

    fn get_tower_range(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent)=Self::handle_to_entity(e) else {return Fixed64::ZERO;};
        let base=self.source_attack(ent).map(|t|t.range.v).unwrap_or(Fixed64::ZERO);
        if self.is_summon(ent) {UnitStats::from_refs(self.source_buffs(ent),false).final_attack_range(base,ent)} else {base}
    }

    fn get_tower_atk(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent)=Self::handle_to_entity(e) else {return Fixed64::ZERO;};
        let base=self.source_attack(ent).map(|t|t.atk_physic.v).unwrap_or(Fixed64::ZERO);
        if self.is_summon(ent) {UnitStats::from_refs(self.source_buffs(ent),false).final_atk(base,ent)} else {base}
    }

    fn get_asd_interval(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let Some(base_interval) = self.source_attack(ent).map(|t| t.asd.v) else {
            return Fixed64::ZERO;
        };
        let stats =
            UnitStats::from_refs(self.source_buffs(ent), self.source_is_building(ent));
        let interval = stats.base_attack_interval(base_interval, ent) / stats.final_attack_speed_mult(ent);
        interval.max(Fixed64::from_raw(1))
    }

    fn get_asd_count(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        self.overlay_asd_count
            .get(&ent)
            .copied()
            .or_else(|| self.source_attack(ent).map(|t| t.asd_count))
            .unwrap_or(Fixed64::ZERO)
    }

    fn set_asd_count(&mut self, e: EntityHandle, v: Fixed64) {
        if let Some(ent) = Self::handle_to_entity(e) {
            self.overlay_asd_count.insert(ent, v);
            self.outcomes.push(Outcome::ScriptSetAsdCount {
                entity: ent,
                asd_count: v,
            });
        }
    }

    fn set_tower_atk(&mut self, e: EntityHandle, v: Fixed64) {
        if let Some(ent) = Self::handle_to_entity(e) {
            self.outcomes.push(Outcome::ScriptSetTowerAtk {
                entity: ent,
                value: v,
            });
        }
    }

    fn set_tower_range(&mut self, e: EntityHandle, v: Fixed64) {
        if let Some(ent) = Self::handle_to_entity(e) {
            self.outcomes.push(Outcome::ScriptSetTowerRange {
                entity: ent,
                value: v,
            });
        }
    }

    fn set_asd_interval(&mut self, e: EntityHandle, v: Fixed64) {
        if let Some(ent) = Self::handle_to_entity(e) {
            self.outcomes.push(Outcome::ScriptSetAsdInterval {
                entity: ent,
                value: v,
            });
        }
    }

    fn set_facing(&mut self, e: EntityHandle, angle: Angle) {
        if let Some(ent) = Self::handle_to_entity(e) {
            self.overlay_facing.insert(ent, angle);
            self.outcomes.push(Outcome::ScriptSetFacing {
                entity: ent,
                facing: angle,
            });
        }
    }

    fn get_facing(&self, e: EntityHandle) -> Angle {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Angle::ZERO;
        };
        self.overlay_facing
            .get(&ent)
            .copied()
            .or_else(|| self.cache.facing.get(ent).map(|f| f.0))
            .or_else(|| self.retired_source(ent).and_then(|source|source.facing.map(|f|f.0)))
            .unwrap_or(Angle::ZERO)
    }

    fn query_nearest_enemy(
        &self,
        center: Vec2,
        radius: Fixed64,
        of: EntityHandle,
    ) -> ROption<EntityHandle> {
        let Some(of_ent) = Self::handle_to_entity(of) else {
            return RNone;
        };
        if self.is_summon(of_ent) {
            let focus=self.summon_focus_target(of_ent);
            return (&self.cache.entities,&self.cache.pos,&self.cache.faction,&self.cache.cprop).join()
                .filter(|(e,_,_,_)|self.summon_target_allowed(of_ent,*e,center,radius))
                .min_by_key(|(e,p,_,_)|(Some(*e)!=focus,p.0.distance_squared(center).raw(),e.id(),e.gen().id()))
                .map(|(e,_,_,_)|RSome(Self::entity_to_handle(e))).unwrap_or(RNone);
        }
        let priority = self.source_tower(of_ent)
            .map(|tower| tower.target_priority)
            .unwrap_or(TowerTargetPriority::Nearest);
        query_script_tower_enemy_with_source(self.cache, center, radius, of, priority,self.retired_source(of_ent))
    }

    fn play_vfx(&mut self, id: RStr<'_>, at: Vec2) {
        log::debug!(
            "[scripting] play_vfx id={} at=({},{})",
            id.as_str(),
            at.x.to_f32_for_render(),
            at.y.to_f32_for_render()
        );
    }

    fn play_sfx(&mut self, id: RStr<'_>, at: Vec2) {
        log::debug!(
            "[scripting] play_sfx id={} at=({},{})",
            id.as_str(),
            at.x.to_f32_for_render(),
            at.y.to_f32_for_render()
        );
    }

    fn rand_unit(&mut self) -> Fixed64 {
        let request_ordinal = self
            .random_request_base
            .saturating_add(u64::from(self.invocation_rng_op));
        self.invocation_rng_op = self.invocation_rng_op.wrapping_add(1);
        Fixed64::from_raw(
            (crate::runtime::tick_random_u64(
                self.cache.rng_seed,
                self.cache.tick.0,
                request_ordinal,
            ) % 1024) as i64,
        )
    }

    fn log_info(&self, msg: RStr<'_>) {
        log::debug!("[script] {}", msg.as_str());
    }

    fn log_warn(&self, msg: RStr<'_>) {
        log::warn!("[script] {}", msg.as_str());
    }

    fn log_error(&self, msg: RStr<'_>) {
        log::error!("[script] {}", msg.as_str());
    }

    fn sum_stat(&self, e: EntityHandle, stat_key: StatKey) -> Fixed64 {
        Self::handle_to_entity(e)
            .map(|ent| self.source_buffs(ent).sum_add(ent, stat_key))
            .unwrap_or(Fixed64::ZERO)
    }

    fn product_stat(&self, e: EntityHandle, stat_key: StatKey) -> Fixed64 {
        Self::handle_to_entity(e)
            .map(|ent| self.source_buffs(ent).product_mult(ent, stat_key))
            .unwrap_or(Fixed64::ONE)
    }

    fn get_final_move_speed(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self.source_property(ent)
            .map(|p| p.msd)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .final_move_speed(base, ent)
    }

    fn get_final_atk(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self.source_attack(ent)
            .map(|t| t.atk_physic.v)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .final_atk(base, ent)
    }

    fn get_tower_upgrade(&self, e: EntityHandle, path: u8) -> u8 {
        Self::handle_to_entity(e)
            .and_then(|ent| self.source_tower(ent))
            .and_then(|tower| tower.upgrade_levels.get(path as usize))
            .copied()
            .unwrap_or(0)
    }

    fn has_tower_flag(&self, e: EntityHandle, flag: RStr<'_>) -> bool {
        Self::handle_to_entity(e)
            .and_then(|ent| self.source_tower(ent))
            .map(|t| t.upgrade_flags.iter().any(|f| f == flag.as_str()))
            .unwrap_or(false)
    }

    fn apply_tower_permanent_buff(
        &mut self,
        e: EntityHandle,
        buff_id: RStr<'_>,
        modifiers_json: RStr<'_>,
    ) {
        self.add_stat_buff(e, buff_id, Fixed64::from_raw(i64::MAX), modifiers_json);
    }

    fn get_final_attack_range(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self.source_attack(ent)
            .map(|t| t.range.v)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .final_attack_range(base, ent)
    }

    fn get_buff_remaining(&self, e: EntityHandle, buff_id: RStr<'_>) -> Fixed64 {
        Self::handle_to_entity(e)
            .and_then(|ent| {
                self.source_buffs(ent).get(ent, buff_id.as_str())
                    .map(|b| b.remaining)
            })
            .unwrap_or(Fixed64::ZERO)
    }

    fn current_mana(&self, e: EntityHandle) -> Fixed64 {
        Self::handle_to_entity(e)
            .and_then(|ent| {
                if let Some(pool)=self.mana_views.get(&ent) {return Some(pool.current());}
                self.cache.hero.get(ent).map(|hero| hero.mana_pool.as_ref()
                    .map(|pool| pool.current()).unwrap_or_else(|| hero.get_max_mana()))
            })
            .unwrap_or(Fixed64::ZERO)
    }

    fn spend_mana(&mut self, e: EntityHandle, amount: Fixed64, ability_id: RStr<'_>) -> bool {
        let Some(entity)=Self::handle_to_entity(e) else {return false;};
        if !self.cache.entities.is_alive(entity) {return false;}
        let Some(hero)=self.cache.hero.get(entity) else {return false;};
        if hero.mana_pool.is_none() {return amount>=Fixed64::ZERO;}
        if !self.mana_transaction || self.managed_mana_entity(e).is_none()
            || ability_id.len()>256 {return false;}
        let mut pool=self.mana_views.get(&entity).cloned().unwrap_or_else(||hero.mana_pool.clone().unwrap());
        if pool.spend(amount).is_err() {return false;}
        if amount>Fixed64::ZERO {
            self.stage_mana_pool(entity,pool);
            self.stage_mana_spent(entity,amount,ability_id.as_str().to_owned());
        }
        true
    }

    fn restore_mana(&mut self, e: EntityHandle, amount: Fixed64) {
        if !self.mana_transaction {return;}
        let Some(entity)=self.managed_mana_entity(e) else {return;};
        let mut pool=self.mana_views.get(&entity).cloned()
            .unwrap_or_else(||self.cache.hero.get(entity).unwrap().mana_pool.clone().unwrap());
        let Ok(restored)=pool.restore(amount) else {return;};
        if restored>Fixed64::ZERO {
            self.stage_mana_pool(entity,pool);
            self.outcomes.push(Outcome::ScriptManaGained {entity,amount:restored});
        }
    }

    fn trigger_state_changed(&mut self, _e: EntityHandle, _state_id: RStr<'_>, _active: bool) {}

    fn get_final_armor(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self.source_property(ent)
            .map(|c| c.def_physic)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .final_armor(base, ent)
    }

    fn get_final_magic_resist(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self.source_property(ent)
            .map(|c| c.def_magic)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .final_magic_resist(base, ent)
    }

    fn get_evasion_chance(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .evasion_chance(ent)
    }

    fn get_miss_chance(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .miss_chance(ent)
    }

    fn get_crit_chance(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .crit(ent)
        .0
    }

    fn get_crit_multiplier(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ONE;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .crit(ent)
        .1
    }

    fn get_cooldown_mult(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ONE;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .cooldown_mult(ent)
    }

    fn is_building(&self, e: EntityHandle) -> bool {
        Self::handle_to_entity(e)
            .map(|ent| self.source_is_building(ent))
            .unwrap_or(false)
    }

    fn get_max_hp_bonus(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .max_hp_bonus(ent)
    }

    fn get_hp_regen(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            self.source_buffs(ent),
            self.source_is_building(ent),
        )
        .hp_regen(Fixed64::ZERO, ent)
    }

    fn get_stat_bonus(&self, e: EntityHandle, key: StatKey) -> Fixed64 {
        Self::handle_to_entity(e)
            .map(|ent| self.source_buffs(ent).sum_add(ent, key))
            .unwrap_or(Fixed64::ZERO)
    }

    fn deal_damage_splash(
        &mut self,
        at: Vec2,
        radius: Fixed64,
        damage: Fixed64,
        kind: DamageKind,
        profile: AbiDamageProfile,
        source: ROption<EntityHandle>,
    ) {
        let of = match source {
            ROption::RSome(h) => h,
            ROption::RNone => return,
        };
        if let Some(source_ent) = Self::handle_to_entity(of) {
            let dir = Self::angle_to_rad_f32(self.get_facing(Self::entity_to_handle(source_ent)));
            self.push_tower_fire_fx(source_ent, dir);
        }
        let targets = self.query_enemies_in_range(at, radius, of);
        for th in targets.iter() {
            self.deal_damage(*th, damage, kind, profile, source);
        }
    }

    fn emit_attack_phase_fx(
        &mut self,
        entity: EntityHandle,
        target: Target,
        windup_ms: u32,
        backswing_ms: u32,
    ) {
        let Some(ent) = Self::handle_to_entity(entity) else {
            return;
        };
        let Some(pos) = self.cache.pos.get(ent).map(|p| p.0) else {
            return;
        };
        let (target_entity, target_pos) = match target {
            Target::Entity(handle) => {
                let target_ent = Self::handle_to_entity(handle);
                let tpos = target_ent.and_then(|te| self.cache.pos.get(te).map(|p| p.0));
                (target_ent, tpos)
            }
            Target::Point(point) => (None, Some(point)),
            Target::None => (None, None),
        };
        let dir_angle = if let Some(tpos) = target_pos {
            omoba_sim::trig::atan2(tpos.y - pos.y, tpos.x - pos.x)
        } else {
            self.get_facing(entity)
        };
        self.overlay_facing.insert(ent, dir_angle);
        self.outcomes.push(Outcome::ScriptSetFacing {
            entity: ent,
            facing: dir_angle,
        });
        self.outcomes.push(Outcome::ScriptAttackPhaseCue {
            entity: ent,
            target: target_entity,
            target_pos,
            windup_ms,
            backswing_ms,
            dir_rad: Self::angle_to_rad_f32(dir_angle),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi_stable::{sabi_trait::prelude::TD_Opaque, std_types::RStr};
    use omb_script_abi::{
        script::{UnitScript, UnitScript_TO},
        world::{GameWorld, GameWorldDyn, TowerActiveAbilityAccessDyn},
    };
    use serde_json::json;
    use specs::Builder;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::runtime::{
        comp::{
            PendingTowerAbilityActivation, PendingTowerAbilityActivationQueue,
            PendingTowerAbilityPulse, PendingTowerAbilityPulseQueue, PlayerOwner,
            TowerActiveAbilityState,
        },
        scripting::{dispatch::dispatch_tower_ability_callbacks, ScriptRegistry},
    };

    pub(super) fn world_for_adapter_tests() -> World {
        let mut world = World::new();
        world.register::<TAttack>();
        world.register::<Pos>();
        world.register::<Facing>();
        world.register::<CProperty>();
        world.register::<Unit>();
        world.register::<Hero>();
        world.register::<Faction>();
        world.register::<Creep>();
        world.register::<Tower>();
        world.register::<IsBuilding>();
        world.register::<CollisionRadius>();
        world.register::<ScriptUnitTag>();
        world.register::<PlayerOwner>();
        world.insert(BuffStore::default());
        world.insert(Searcher::default());
        world.insert(BlockedRegions::default());
        world.insert(Tick(42));
        world
    }

    #[test]
    fn summon_host_destination_blocks_script_chase_until_removed_without_blocking_other_units() {
        let mut world=world_for_adapter_tests();world.register::<MoveTarget>();
        let source=world.create_entity().with(Pos(Vec2::ZERO))
            .with(Unit::new("arbitrary_summon".into(),"arbitrary".into(),UnitType::Summon)).build();
        let destination=Vec2::new(Fixed64::ZERO,Fixed64::from_i32(100));
        world.write_storage::<MoveTarget>().insert(source,MoveTarget(destination)).unwrap();
        let aim=Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
        let advance=|world:&World| {
            let cache=ParallelAdapterCache::new(world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
            adapter.advance_with_collision(ParallelWorldAdapter::entity_to_handle(source),aim,Fixed64::from_i32(10))
        };
        assert_eq!(advance(&world),Vec2::ZERO);
        world.write_storage::<MoveTarget>().remove(source);
        assert_eq!(advance(&world),Vec2::new(Fixed64::from_i32(10),Fixed64::ZERO));
        world.write_resource::<BuffStore>().add(source,"root",Fixed64::ONE,json!({}));
        assert_eq!(advance(&world),Vec2::ZERO);world.write_resource::<BuffStore>().remove(source,"root");
        world.write_storage::<MoveTarget>().insert(source,MoveTarget(destination)).unwrap();
        world.write_storage::<Unit>().get_mut(source).unwrap().unit_type=UnitType::Hero;
        assert_eq!(advance(&world),Vec2::new(Fixed64::from_i32(10),Fixed64::ZERO));
    }

    #[test]
    fn summon_handles_are_reserved_only_serially_and_cancelled_on_failure_or_drop() {
        let mut world=world_for_adapter_tests();
        let owner=world.create_entity().with(Faction::new(FactionType::Player,7))
            .with(PlayerOwner::new(42)).with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,
                msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let cache=ParallelAdapterCache::new(&world,1);
        let mut probe=ParallelWorldAdapter::new(&cache,owner);probe.allocation_allowed=false;
        assert_eq!(probe.spawn_summoned_unit(Vec2::ZERO,"saika_gunner".into(),
            ParallelWorldAdapter::entity_to_handle(owner),Fixed64::ONE),EntityHandle::INVALID);
        assert!(probe.allocation_requested);assert!(probe.finish().is_empty());
        let mut adapter=ParallelWorldAdapter::new(&cache,owner);
        let handle=adapter.spawn_summoned_unit(Vec2::ZERO,"saika_gunner".into(),
            ParallelWorldAdapter::entity_to_handle(owner),Fixed64::ONE);
        assert_ne!(handle,EntityHandle::INVALID);
        let entity=ParallelWorldAdapter::handle_to_entity(handle).unwrap();
        assert!(cache.entities.is_alive(entity));
        adapter.discard_cast();assert!(adapter.outcomes.is_empty());
        let cancelled=entity;
        let handle=adapter.spawn_summoned_unit(Vec2::ZERO,"saika_gunner".into(),
            ParallelWorldAdapter::entity_to_handle(owner),Fixed64::ZERO);
        let entity=ParallelWorldAdapter::handle_to_entity(handle).unwrap();drop(adapter);
        drop(cache);world.maintain();
        assert!(!world.entities().is_alive(entity));assert!(!world.entities().is_alive(cancelled));
    }

    #[test]
    fn summon_rejects_unknown_template_stale_owner_and_negative_lifetime() {
        let mut world=world_for_adapter_tests();
        let owner=world.create_entity().with(Faction::new(FactionType::Player,1))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,owner);
        let handle=ParallelWorldAdapter::entity_to_handle(owner);
        for (id,who,duration) in [("unknown",handle,Fixed64::ONE),("saika_gunner",handle,-Fixed64::ONE),
            ("saika_gunner",EntityHandle::INVALID,Fixed64::ONE)] {
            assert_eq!(adapter.spawn_summoned_unit(Vec2::ZERO,id.into(),who,duration),EntityHandle::INVALID);
        }
        assert!(adapter.finish().is_empty());
    }

    fn summon_combat_fixture()->(World,Entity,Entity) {
        use crate::runtime::{StateInitializer,setup_single_lane_match,SingleLaneConfig,MobaMatch,MobaMatchPhase,VisionSource};
        let pool=StateInitializer::create_thread_pool();
        let mut world=StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::ZERO,..SingleLaneConfig::default()}).unwrap();
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        world.write_storage::<VisionSource>().clear();
        let mut spawn=|team,position,kind|world.create_entity().with(Pos(position))
            .with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
            .with(CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::from_i32(100),
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO})
            .with(Unit::new("query_fixture".into(),"query".into(),kind)).build();
        let source=spawn(1,Vec2::ZERO,UnitType::Summon);
        let target=spawn(2,Vec2::new(Fixed64::from_i32(20),Fixed64::ZERO),UnitType::Creep);
        world.write_storage::<VisionSource>().insert(source,VisionSource {team:1,radius:Fixed64::from_i32(100),detection_level:0}).unwrap();
        (world,source,target)
    }

    fn summon_query(world:&World,source:Entity)->Vec<EntityHandle> {
        let cache=ParallelAdapterCache::new(world,1);let adapter=ParallelWorldAdapter::new(&cache,source);
        let handle=ParallelWorldAdapter::entity_to_handle(source);
        let all=adapter.query_enemies_in_range(Vec2::ZERO,Fixed64::from_i32(100),handle).into_vec();
        assert_eq!(adapter.query_nearest_enemy(Vec2::ZERO,Fixed64::from_i32(100),handle).into_option(),all.first().copied());
        all
    }

    #[test]
    fn summon_owner_focus_preserves_default_visibility_range_owner_life_and_generation() {
        use crate::runtime::{VisionSource,visibility::run_committed_visibility_wave_b};
        let (mut world,source,near)=summon_combat_fixture();
        let far=world.create_entity().with(Pos(Vec2::new(Fixed64::from_i32(80),Fixed64::ZERO)))
            .with(Faction::new(FactionType::Enemy,2)).with(Unit::new("focus_target".into(),"focus".into(),UnitType::Creep))
            .with(CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let make_owner=|w:&mut World|w.create_entity().with(Pos(Vec2::ZERO))
            .with(Hero::new("owner".into(),"owner".into(),"owner".into())).with(PlayerOwner::new(7))
            .with(Faction::new(FactionType::Player,1))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO})
            .with(HeroCommandQueue {active:Some(HeroCommand::AttackTarget {target:far,chase_origin:None}),queued:Vec::new()}).build();
        let owner=make_owner(&mut world);
        world.write_storage::<SummonedUnit>().insert(source,SummonedUnit::new_fixed(owner,Some(Fixed64::from_i32(30)))).unwrap();
        world.write_storage::<PlayerOwner>().insert(source,PlayerOwner::new(7)).unwrap();
        let select=|w:&World| {let cache=ParallelAdapterCache::new(w,1);let adapter=ParallelWorldAdapter::new(&cache,source);
            adapter.query_nearest_enemy(Vec2::ZERO,Fixed64::from_i32(100),ParallelWorldAdapter::entity_to_handle(source)).into_option()};
        let handle=ParallelWorldAdapter::entity_to_handle;
        run_committed_visibility_wave_b(&mut world,1,0);
        assert_eq!(select(&world),Some(handle(near)),"unknown/default policy remains nearest");
        world.write_storage::<Unit>().get_mut(source).unwrap().id="saika_gunner".into();
        assert_eq!(select(&world),Some(handle(far)),"authored owner-focus policy assists the active command");
        world.write_storage::<VisionSource>().get_mut(source).unwrap().radius=Fixed64::from_i32(30);
        run_committed_visibility_wave_b(&mut world,2,0);
        assert_eq!(select(&world),Some(handle(near)),"last-known focused target is not current permission");
        world.write_storage::<VisionSource>().get_mut(source).unwrap().radius=Fixed64::from_i32(100);
        run_committed_visibility_wave_b(&mut world,3,0);assert_eq!(select(&world),Some(handle(far)));
        world.write_storage::<Pos>().get_mut(far).unwrap().0.x=Fixed64::from_i32(200);
        assert_eq!(select(&world),Some(handle(near)),"focus cannot extend the query range");
        world.write_storage::<Pos>().get_mut(far).unwrap().0.x=Fixed64::from_i32(80);
        world.write_storage::<CProperty>().get_mut(far).unwrap().hp=Fixed64::ZERO;
        assert_eq!(select(&world),Some(handle(near)));world.write_storage::<CProperty>().get_mut(far).unwrap().hp=Fixed64::from_i32(100);
        world.write_storage::<HeroCommandQueue>().get_mut(owner).unwrap().active=Some(HeroCommand::HoldPosition);
        assert_eq!(select(&world),Some(handle(near)));
        world.write_storage::<HeroCommandQueue>().get_mut(owner).unwrap().active=Some(HeroCommand::AttackTarget {target:far,chase_origin:None});
        world.write_storage::<PlayerOwner>().get_mut(owner).unwrap().player_id=8;
        assert_eq!(select(&world),Some(handle(near)));world.write_storage::<PlayerOwner>().get_mut(owner).unwrap().player_id=7;
        world.write_storage::<Faction>().get_mut(owner).unwrap().team_id=2;
        assert_eq!(select(&world),Some(handle(owner)),"foreign faction disables focus; that owner is now a legitimate nearest enemy");world.write_storage::<Faction>().get_mut(owner).unwrap().team_id=1;
        world.write_storage::<CProperty>().get_mut(owner).unwrap().hp=Fixed64::ZERO;
        assert_eq!(select(&world),Some(handle(near)));world.write_storage::<CProperty>().get_mut(owner).unwrap().hp=Fixed64::ONE;
        world.delete_entity(owner).unwrap();world.maintain();let fresh=make_owner(&mut world);
        assert_eq!(owner.id(),fresh.id());assert_ne!(owner,fresh);
        assert_eq!(select(&world),Some(handle(near)),"replacement life does not inherit old summon control");
    }

    #[test]
    fn summon_combat_uses_current_brush_disclosure_not_last_known_or_uncommitted_sight() {
        use crate::runtime::{VisionSource,visibility::run_committed_visibility_wave_b};
        use crate::runtime::brush_vision::{BrushVisionRegion,BrushVisionRegions};
        let (mut world,source,target)=summon_combat_fixture();
        assert!(summon_query(&world,source).is_empty(),"authority without committed view fails closed");
        world.insert(BrushVisionRegions::from(vec![BrushVisionRegion {
            min:Vec2::new(Fixed64::from_i32(10),Fixed64::from_i32(-5)),
            max:Vec2::new(Fixed64::from_i32(30),Fixed64::from_i32(5))}]));
        run_committed_visibility_wave_b(&mut world,1,0);
        assert!(summon_query(&world,source).is_empty(),"nearby concealed enemy stays hidden");
        let observer=world.create_entity().with(Pos(Vec2::new(Fixed64::from_i32(20),Fixed64::ZERO)))
            .with(VisionSource {team:1,radius:Fixed64::from_i32(100),detection_level:0}).build();
        run_committed_visibility_wave_b(&mut world,2,0);
        assert_eq!(summon_query(&world,source),vec![ParallelWorldAdapter::entity_to_handle(target)]);
        world.write_storage::<VisionSource>().remove(observer);
        run_committed_visibility_wave_b(&mut world,3,0);
        assert!(summon_query(&world,source).is_empty(),"last known body must not remain targetable");
        world.insert(crate::runtime::filtered_specs::DisclosedMovementPriority::default());
        assert_eq!(summon_query(&world,source),vec![ParallelWorldAdapter::entity_to_handle(target)],
            "admitted filtered worlds do not require an authority visibility stream");
    }

    #[test]
    fn summon_combat_respects_automatic_ward_gate_and_buffed_attack_stats() {
        let mut world=world_for_adapter_tests();
        let source=world.create_entity().with(Pos(Vec2::ZERO)).with(Unit::new("s".into(),"s".into(),UnitType::Summon))
            .with(Faction::new(FactionType::Player,1)).with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,
                msd:Fixed64::ONE,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO})
            .with(TAttack::new(Fixed64::from_i32(35),Fixed64::ONE,Fixed64::from_i32(100),Fixed64::ONE)).build();
        let target=world.create_entity().with(Pos(Vec2::new(Fixed64::ONE,Fixed64::ZERO)))
            .with(Faction::new(FactionType::Enemy,2)).with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,
                msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        assert_eq!(summon_query(&world,source).len(),1);
        let mut gate=crate::runtime::filtered_specs::DisclosedAttackabilityGate::default();
        gate.ward_automatic_disabled.insert(target);world.insert(gate);
        assert!(summon_query(&world,source).is_empty());
        world.insert(crate::runtime::filtered_specs::DisclosedAttackabilityGate::default());
        world.write_resource::<BuffStore>().add(source,"stun",Fixed64::from_i32(10),json!({}));
        assert!(summon_query(&world,source).is_empty());
        world.write_resource::<BuffStore>().remove(source,"stun");
        world.write_resource::<BuffStore>().add(source,"combat_bonus",Fixed64::from_i32(10),
            json!({"preattack_bonus_damage":10240,"attack_range_bonus":102400}));
        let cache=ParallelAdapterCache::new(&world,1);let adapter=ParallelWorldAdapter::new(&cache,source);
        let handle=ParallelWorldAdapter::entity_to_handle(source);
        assert_eq!(adapter.get_tower_atk(handle),Fixed64::from_i32(45));
        assert_eq!(adapter.get_tower_range(handle),Fixed64::from_i32(200));
    }

    #[test]
    fn summon_reservations_cross_caches_cancel_exactly_and_do_not_disclose_enemy_placements() {
        let mut world=world_for_adapter_tests();
        world.insert(super::super::super::summon_navigation::PlacementReservations::default());
        let mut owner=|team|world.create_entity().with(Faction::new(FactionType::Player,team))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let friendly=owner(1);let enemy=owner(2);
        fn spawn<'a>(cache:&'a ParallelAdapterCache<'a>,owner:Entity)->(EntityHandle,Vec2,ParallelWorldAdapter<'a>) {
            let mut adapter=ParallelWorldAdapter::new(cache,owner);
            let handle=adapter.spawn_summoned_unit(Vec2::ZERO,"saika_gunner".into(),
                ParallelWorldAdapter::entity_to_handle(owner),Fixed64::ONE);
            assert_ne!(handle,EntityHandle::INVALID);
            let position=adapter.get_pos(handle).into_option().unwrap();
            (handle,position,adapter)
        }
        let cache=ParallelAdapterCache::new(&world,1);
        let (first,position,adapter)=spawn(&cache,friendly);assert_eq!(adapter.finish().len(),1);drop(cache);
        let cache=ParallelAdapterCache::new(&world,1);
        let (_,second,mut adapter)=spawn(&cache,friendly);
        assert!((second-position).length()>=Fixed64::from_i32(40));
        adapter.discard_cast();drop(adapter);
        let (_,again,adapter)=spawn(&cache,friendly);assert_eq!(again,second);drop(adapter);
        let (_,enemy_position,adapter)=spawn(&cache,enemy);assert_eq!(enemy_position,position);
        drop(adapter);
        assert_eq!(cache.summon_placements.0.lock().unwrap().len(),1);
        let old=ParallelWorldAdapter::handle_to_entity(first).unwrap();
        cache.entities.delete(old).unwrap();drop(cache);world.maintain();
        let cache=ParallelAdapterCache::new(&world,1);
        assert!(cache.summon_placements.0.lock().unwrap().is_empty());
    }

    #[test]
    fn summon_reservation_budget_rejects_without_consuming_a_handle_or_outcome() {
        let mut world=world_for_adapter_tests();
        let owner=world.create_entity().with(Faction::new(FactionType::Player,1))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let cache=ParallelAdapterCache::new(&world,1);
        let mut ledger=cache.summon_placements.0.lock().unwrap();
        for _ in 0..4096 {ledger.insert(cache.entities.create(),(2,Vec2::ZERO,Fixed64::ONE));}
        drop(ledger);
        let mut adapter=ParallelWorldAdapter::new(&cache,owner);
        assert_eq!(adapter.spawn_summoned_unit(Vec2::ZERO,"saika_gunner".into(),
            ParallelWorldAdapter::entity_to_handle(owner),Fixed64::ONE),EntityHandle::INVALID);
        assert!(adapter.finish().is_empty());assert_eq!(cache.summon_placements.0.lock().unwrap().len(),4096);
    }

    struct SpawnOnTick;
    impl UnitScript for SpawnOnTick {
        fn unit_id(&self)->RStr<'_> {"spawn_on_tick_test".into()}
        fn on_tick(&self,e:EntityHandle,_dt:Fixed64,world:&mut GameWorldDyn<'_>) {
            let child=world.spawn_summoned_unit(Vec2::ZERO,"saika_gunner".into(),e,Fixed64::ONE);
            if child!=EntityHandle::INVALID {world.set_facing(child,Angle::from_ticks(123));}
        }
    }

    #[test]
    fn parallel_summon_hooks_allocate_in_canonical_order_and_keep_returned_handle_commands() {
        use crate::runtime::StateInitializer;
        use crate::runtime::scripting::{run_script_dispatch,ScriptEventQueue,ScriptVisualEventQueue};
        let pool=StateInitializer::create_thread_pool();
        let mut world=StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(ScriptEventQueue::default());world.insert(ScriptVisualEventQueue::default());
        world.insert(TickProfile::default());
        let owners:Vec<_>=(0..8).map(|_|world.create_entity()
            .with(Faction::new(FactionType::Player,1)).with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,
                msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO})
            .with(ScriptUnitTag {unit_id:"spawn_on_tick_test".into()}).build()).collect();
        let mut registry=ScriptRegistry::new();
        registry.insert_unit_for_test("spawn_on_tick_test",UnitScript_TO::from_value(SpawnOnTick,TD_Opaque));
        run_script_dispatch(&mut world,&registry,1,Fixed64::from_raw(17));
        let outcomes=world.read_resource::<Vec<Outcome>>();
        let spawned:Vec<_>=outcomes.iter().filter_map(|o|if let Outcome::SpawnSummoned {entity,owner,..}=o {
            Some((*entity,*owner))}else{None}).collect();
        assert_eq!(spawned.len(),owners.len());
        let placements:Vec<_>=outcomes.iter().filter_map(|o|if let Outcome::SpawnSummoned {entity,pos,..}=o {
            Some((*entity,*pos))}else{None}).collect();
        for (i,(_,position)) in placements.iter().enumerate() {
            for (_,other) in &placements[..i] {assert!((*position-*other).length()>=Fixed64::from_i32(40));}
        }
        for (index,(entity,owner)) in spawned.iter().enumerate() {
            assert_eq!(*owner,owners[index]);
            if index>0 {assert!(spawned[index-1].0.id()<entity.id());}
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::ScriptSetFacing {entity:e,facing}
                if e==entity && facing.ticks()==123)));
        }
        drop(outcomes);
        crate::runtime::process_outcomes(&mut world,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
        for (entity,position) in placements {assert_eq!(world.read_storage::<Pos>().get(entity).unwrap().0,position);}
        assert!(world.read_resource::<super::super::super::summon_navigation::PlacementReservations>().0.lock().unwrap().is_empty());

    }

    #[derive(Clone)]
    struct ExactTowerAbilityScript {
        activations: Arc<AtomicUsize>,
        pulses: Arc<AtomicUsize>,
    }

    impl UnitScript for ExactTowerAbilityScript {
        fn unit_id(&self) -> RStr<'_> {
            "tower_exact".into()
        }

        fn on_tower_ability_activate(
            &self,
            _tower: EntityHandle,
            _ability_id: RStr<'_>,
            _w: &mut GameWorldDyn<'_>,
        ) {
            self.activations.fetch_add(1, Ordering::SeqCst);
        }

        fn on_tower_ability_pulse(
            &self,
            _tower: EntityHandle,
            _ability_id: RStr<'_>,
            _pulse_index: u16,
            _w: &mut GameWorldDyn<'_>,
        ) -> bool {
            self.pulses.fetch_add(1, Ordering::SeqCst);
            true
        }
    }

    #[derive(Clone)]
    struct AccessTowerAbilityScript {
        remaining_raw: Arc<AtomicUsize>,
        serial: Arc<AtomicUsize>,
        friendly_count: Arc<AtomicUsize>,
    }

    impl UnitScript for AccessTowerAbilityScript {
        fn unit_id(&self) -> RStr<'_> {
            "tower_access".into()
        }

        fn on_tower_ability_activate_with_access(
            &self,
            tower: EntityHandle,
            ability_id: RStr<'_>,
            access: &TowerActiveAbilityAccessDyn<'_>,
            _w: &mut GameWorldDyn<'_>,
        ) {
            self.remaining_raw.store(
                access
                    .get_tower_ability_active_remaining(tower, ability_id)
                    .raw() as usize,
                Ordering::SeqCst,
            );
            self.serial.store(
                access.get_tower_ability_activation_serial(tower, ability_id) as usize,
                Ordering::SeqCst,
            );
            self.friendly_count.store(
                access
                    .query_friendly_towers_in_range(Vec2::ZERO, Fixed64::from_i32(100), tower)
                    .len(),
                Ordering::SeqCst,
            );
            access.reset_attack_backswing(tower);
        }
    }

    struct DefaultTowerAbilityScript;

    impl UnitScript for DefaultTowerAbilityScript {
        fn unit_id(&self) -> RStr<'_> {
            "tower_default".into()
        }
    }

    #[test]
    fn tower_ability_dispatch_exact_hooks_run_once_and_acknowledge_pulse() {
        let mut world = world_for_adapter_tests();
        world.insert(PendingTowerAbilityActivationQueue::default());
        world.insert(PendingTowerAbilityPulseQueue::default());
        let mut tower_data = Tower::new();
        let mut state = TowerActiveAbilityState::ready("test_active");
        state
            .activate(Fixed64::from_i32(10), Fixed64::from_i32(3), Fixed64::ONE, 1)
            .unwrap();
        assert!(state.advance(Fixed64::ONE).pulse_due);
        tower_data.active_ability = Some(state);
        let tower = world
            .create_entity()
            .with(tower_data)
            .with(ScriptUnitTag {
                unit_id: "tower_exact".to_string(),
            })
            .build();
        world
            .write_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .push(PendingTowerAbilityActivation {
                entity: tower,
                ability_id: "test_active".to_string(),
                activation_serial: 1,
            });
        world
            .write_resource::<PendingTowerAbilityPulseQueue>()
            .requests
            .push(PendingTowerAbilityPulse {
                entity: tower,
                ability_id: "test_active".to_string(),
                activation_serial: 1,
                pulse_index: 0,
            });

        let activations = Arc::new(AtomicUsize::new(0));
        let pulses = Arc::new(AtomicUsize::new(0));
        let mut registry = ScriptRegistry::new();
        registry.insert_unit_for_test(
            "tower_exact",
            UnitScript_TO::from_value(
                ExactTowerAbilityScript {
                    activations: Arc::clone(&activations),
                    pulses: Arc::clone(&pulses),
                },
                TD_Opaque,
            ),
        );

        dispatch_tower_ability_callbacks(&mut world, &registry, 7);
        dispatch_tower_ability_callbacks(&mut world, &registry, 7);

        assert_eq!(activations.load(Ordering::SeqCst), 1);
        assert_eq!(pulses.load(Ordering::SeqCst), 1);
        let towers = world.read_storage::<Tower>();
        let state = towers.get(tower).unwrap().active_ability.as_ref().unwrap();
        assert_eq!(state.pulses_remaining, 0);
        assert!(!state.opportunity_outstanding);
    }

    #[test]
    fn tower_ability_dispatch_extension_reads_state_and_resets_backswing() {
        let mut world = world_for_adapter_tests();
        world.insert(PendingTowerAbilityActivationQueue::default());
        world.insert(PendingTowerAbilityPulseQueue::default());
        let mut tower_data = Tower::new();
        let mut state = TowerActiveAbilityState::ready("test_active");
        state
            .activate(
                Fixed64::from_i32(10),
                Fixed64::from_i32(3),
                Fixed64::ZERO,
                0,
            )
            .unwrap();
        tower_data.active_ability = Some(state);
        let tower = world
            .create_entity()
            .with(Pos(Vec2::ZERO))
            .with(Faction::new(FactionType::Player, 1))
            .with(PlayerOwner::new(7))
            .with(TAttack::new(
                Fixed64::ONE,
                Fixed64::from_i32(2),
                Fixed64::from_i32(100),
                Fixed64::ONE,
            ))
            .with(tower_data)
            .with(ScriptUnitTag {
                unit_id: "tower_access".to_string(),
            })
            .build();
        let _friendly = world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO)))
            .with(Faction::new(FactionType::Player, 1))
            .with(PlayerOwner::new(7))
            .with(Tower::new())
            .build();
        let _other_owner = world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)))
            .with(Faction::new(FactionType::Player, 1))
            .with(PlayerOwner::new(8))
            .with(Tower::new())
            .build();
        world
            .write_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .push(PendingTowerAbilityActivation {
                entity: tower,
                ability_id: "test_active".to_string(),
                activation_serial: 1,
            });

        let remaining_raw = Arc::new(AtomicUsize::new(0));
        let serial = Arc::new(AtomicUsize::new(0));
        let friendly_count = Arc::new(AtomicUsize::new(0));
        let mut registry = ScriptRegistry::new();
        registry.insert_unit_for_test(
            "tower_access",
            UnitScript_TO::from_value(
                AccessTowerAbilityScript {
                    remaining_raw: Arc::clone(&remaining_raw),
                    serial: Arc::clone(&serial),
                    friendly_count: Arc::clone(&friendly_count),
                },
                TD_Opaque,
            ),
        );

        dispatch_tower_ability_callbacks(&mut world, &registry, 9);

        assert_eq!(
            remaining_raw.load(Ordering::SeqCst),
            Fixed64::from_i32(3).raw() as usize
        );
        assert_eq!(serial.load(Ordering::SeqCst), 1);
        assert_eq!(friendly_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            world
                .read_storage::<TAttack>()
                .get(tower)
                .unwrap()
                .asd_count,
            Fixed64::from_i32(2)
        );
    }

    #[test]
    fn tower_ability_dispatch_default_hooks_are_safe_and_consume_pulse() {
        let mut world = world_for_adapter_tests();
        world.insert(PendingTowerAbilityActivationQueue::default());
        world.insert(PendingTowerAbilityPulseQueue::default());
        let mut tower_data = Tower::new();
        let mut state = TowerActiveAbilityState::ready("test_default");
        state
            .activate(Fixed64::ONE, Fixed64::ONE, Fixed64::ONE, 1)
            .unwrap();
        assert!(state.advance(Fixed64::ONE).pulse_due);
        tower_data.active_ability = Some(state);
        let tower = world
            .create_entity()
            .with(tower_data)
            .with(ScriptUnitTag {
                unit_id: "tower_default".to_string(),
            })
            .build();
        world
            .write_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .push(PendingTowerAbilityActivation {
                entity: tower,
                ability_id: "test_default".to_string(),
                activation_serial: 1,
            });
        world
            .write_resource::<PendingTowerAbilityPulseQueue>()
            .requests
            .push(PendingTowerAbilityPulse {
                entity: tower,
                ability_id: "test_default".to_string(),
                activation_serial: 1,
                pulse_index: 0,
            });
        let mut registry = ScriptRegistry::new();
        registry.insert_unit_for_test(
            "tower_default",
            UnitScript_TO::from_value(DefaultTowerAbilityScript, TD_Opaque),
        );

        dispatch_tower_ability_callbacks(&mut world, &registry, 11);

        let towers = world.read_storage::<Tower>();
        let state = towers.get(tower).unwrap().active_ability.as_ref().unwrap();
        assert_eq!(state.pulses_remaining, 0);
        assert!(!state.opportunity_outstanding);
    }

    #[test]
    fn tower_ability_dispatch_missing_script_cancels_matching_state_once() {
        let mut world = world_for_adapter_tests();
        world.insert(PendingTowerAbilityActivationQueue::default());
        world.insert(PendingTowerAbilityPulseQueue::default());
        let mut tower_data = Tower::new();
        let mut state = TowerActiveAbilityState::ready("test_missing");
        state
            .activate(Fixed64::from_i32(10), Fixed64::from_i32(3), Fixed64::ONE, 1)
            .unwrap();
        assert!(state.advance(Fixed64::ONE).pulse_due);
        tower_data.active_ability = Some(state);
        let tower = world
            .create_entity()
            .with(tower_data)
            .with(ScriptUnitTag {
                unit_id: "not_registered".to_string(),
            })
            .build();
        world
            .write_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .push(PendingTowerAbilityActivation {
                entity: tower,
                ability_id: "test_missing".to_string(),
                activation_serial: 1,
            });
        world
            .write_resource::<PendingTowerAbilityPulseQueue>()
            .requests
            .push(PendingTowerAbilityPulse {
                entity: tower,
                ability_id: "test_missing".to_string(),
                activation_serial: 1,
                pulse_index: 0,
            });

        let registry = ScriptRegistry::new();
        dispatch_tower_ability_callbacks(&mut world, &registry, 13);
        dispatch_tower_ability_callbacks(&mut world, &registry, 13);

        let towers = world.read_storage::<Tower>();
        let state = towers.get(tower).unwrap().active_ability.as_ref().unwrap();
        assert_eq!(state.active_remaining, Fixed64::ZERO);
        assert_eq!(state.pulses_remaining, 0);
        assert_eq!(state.pending_due, 0);
        assert!(!state.opportunity_outstanding);
        assert!(world
            .read_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .is_empty());
        assert!(world
            .read_resource::<PendingTowerAbilityPulseQueue>()
            .requests
            .is_empty());
    }

    #[test]
    fn tower_ability_dispatch_missing_tower_logs_once_and_drains_stale_records() {
        let mut world = world_for_adapter_tests();
        world.insert(PendingTowerAbilityActivationQueue::default());
        world.insert(PendingTowerAbilityPulseQueue::default());
        let missing_tower = world.create_entity().build();
        world
            .write_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .push(PendingTowerAbilityActivation {
                entity: missing_tower,
                ability_id: "test_deleted".to_string(),
                activation_serial: 5,
            });
        world
            .write_resource::<PendingTowerAbilityPulseQueue>()
            .requests
            .push(PendingTowerAbilityPulse {
                entity: missing_tower,
                ability_id: "test_deleted".to_string(),
                activation_serial: 5,
                pulse_index: 0,
            });

        let registry = ScriptRegistry::new();
        let first = dispatch_tower_ability_callbacks(&mut world, &registry, 17);
        let second = dispatch_tower_ability_callbacks(&mut world, &registry, 17);

        assert_eq!(first.missing_tower_diagnostics, 1);
        assert_eq!(second.missing_tower_diagnostics, 0);
        assert!(world
            .read_resource::<PendingTowerAbilityActivationQueue>()
            .requests
            .is_empty());
        assert!(world
            .read_resource::<PendingTowerAbilityPulseQueue>()
            .requests
            .is_empty());
    }

    fn add_targetable_creep(
        world: &mut World,
        pos: Vec2,
        remaining: i32,
        hp: i32,
        team_id: i32,
    ) -> Entity {
        world
            .create_entity()
            .with(Pos(pos))
            .with(Faction {
                faction_id: FactionType::Enemy,
                team_id,
            })
            .with(Creep {
                name: "test_creep".to_string(),
                label: None,
                path: "test_path".to_string(),
                pidx: 0,
                path_remaining_distance: Fixed64::from_i32(remaining),
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: None,
            })
            .with(CProperty {
                hp: Fixed64::from_i32(hp),
                mhp: Fixed64::from_i32(hp),
                msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .build()
    }

    #[test]
    fn parallel_adapter_root_blocks_voluntary_advance_but_not_explicit_position() {
        let mut world=world_for_adapter_tests();
        let origin=Vec2::new(Fixed64::from_i32(1),Fixed64::from_i32(2));
        let target=Vec2::new(Fixed64::from_i32(20),Fixed64::from_i32(2));
        let entity=world.create_entity().with(Pos(origin)).build();
        world.write_resource::<BuffStore>().add(entity,"root",Fixed64::ONE,serde_json::json!({}));
        let handle=ParallelWorldAdapter::entity_to_handle(entity);
        let cache=ParallelAdapterCache::new(&world,123);
        let mut adapter=ParallelWorldAdapter::new(&cache,entity);
        assert_eq!(adapter.advance_with_collision(handle,target,Fixed64::from_i32(100)),origin);
        adapter.set_pos(handle,target);
        assert_eq!(adapter.get_pos(handle),RSome(target));
        assert_eq!(adapter.advance_with_collision(handle,origin,Fixed64::from_i32(100)),target,
            "blocked movement returns current overlay position, not stale cache");
        assert!(matches!(adapter.finish().as_slice(),[Outcome::ScriptSetPos {entity:actual,pos}]
            if *actual==entity && *pos==target));
    }

    #[test]
    fn parallel_adapter_buffers_mutations_and_reads_overlay() {
        let mut world = world_for_adapter_tests();
        let entity = world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(1), Fixed64::from_i32(2))))
            .with(Facing(Angle::ZERO))
            .with(Tower::new())
            .with(TAttack::new(
                Fixed64::from_i32(10),
                Fixed64::from_i32(1),
                Fixed64::from_i32(100),
                Fixed64::from_i32(900),
            ))
            .build();
        let handle = ParallelWorldAdapter::entity_to_handle(entity);
        let cache = ParallelAdapterCache::new(&world, 123);
        let mut adapter = ParallelWorldAdapter::new(&cache, entity);

        let pos = Vec2::new(Fixed64::from_i32(5), Fixed64::from_i32(6));
        let facing = Angle::from_degrees_i32(90);
        adapter.set_pos(handle, pos);
        adapter.set_facing(handle, facing);
        adapter.set_asd_count(handle, Fixed64::from_raw(321));

        assert_eq!(adapter.get_pos(handle), RSome(pos));
        assert_eq!(adapter.get_facing(handle), facing);
        assert_eq!(adapter.get_asd_count(handle), Fixed64::from_raw(321));

        let outcomes = adapter.finish();
        assert!(
            matches!(outcomes[0], Outcome::ScriptSetPos { entity: e, pos: p } if e == entity && p == pos)
        );
        assert!(
            matches!(outcomes[1], Outcome::ScriptSetFacing { entity: e, facing: f } if e == entity && f == facing)
        );
        assert!(
            matches!(outcomes[2], Outcome::ScriptSetAsdCount { entity: e, asd_count } if e == entity && asd_count == Fixed64::from_raw(321))
        );
    }

    #[test]
    fn mana_cast_discard_restores_local_views_and_drops_all_deferred_mutations() {
        let mut world = world_for_adapter_tests();
        let original = Vec2::new(Fixed64::from_i32(1), Fixed64::from_i32(2));
        let mut hero = Hero::default();
        hero.initialize_mana_pool().unwrap();
        let balance = hero.mana_pool.as_ref().unwrap().current();
        let entity = world.create_entity().with(Pos(original)).with(hero).build();
        let handle = ParallelWorldAdapter::entity_to_handle(entity);
        let cache = ParallelAdapterCache::new(&world, 123);
        let mut adapter = ParallelWorldAdapter::new(&cache, entity);
        adapter.begin_mana_transaction(&HashMap::new());
        adapter.stage_mana_pool(entity, crate::runtime::ability_runtime::ManaPool::full(Fixed64::from_i32(10)).unwrap());
        assert_eq!(adapter.current_mana(handle), Fixed64::from_i32(10));
        assert!(!adapter.spend_mana(handle, Fixed64::from_i32(1), "test".into()));
        adapter.set_pos(handle, Vec2::new(Fixed64::from_i32(9), Fixed64::ZERO));
        adapter.start_cooldown(entity, "test".into(), Fixed64::from_i32(5));
        adapter.commit_mana(entity, crate::runtime::ability_runtime::ManaPool::full(Fixed64::from_i32(10)).unwrap());
        adapter.mark_cast_succeeded();
        adapter.discard_cast();
        assert!(!adapter.cast_succeeded());
        assert_eq!(adapter.get_pos(handle), RSome(original));
        assert_eq!(adapter.current_mana(handle), balance);
        assert!(adapter.finish().is_empty());
    }

    #[test]
    fn mana_script_transaction_checks_amount_life_identity_capacity_and_rollback() {
        use crate::runtime::ability_runtime::ManaPool;
        let mut world=world_for_adapter_tests();
        let mut hero=Hero::default();
        hero.mana_pool=Some(ManaPool::new(Fixed64::from_i32(50),Fixed64::from_i32(100)).unwrap());
        let health=CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        let entity=world.create_entity().with(hero.clone()).with(health.clone()).build();
        let dead=world.create_entity().with(hero).with(CProperty {hp:Fixed64::ZERO,..health}).build();
        let handle=ParallelWorldAdapter::entity_to_handle(entity);
        let cache=ParallelAdapterCache::new(&world,123);
        let mut adapter=ParallelWorldAdapter::new(&cache,entity);
        assert!(!adapter.spend_mana(handle,Fixed64::ONE,"extra".into()),"parallel snapshot cannot admit a spend");
        adapter.begin_mana_transaction(&HashMap::new());
        assert!(!adapter.spend_mana(handle,Fixed64::from_raw(-1),"extra".into()));
        assert!(!adapter.spend_mana(EntityHandle {gen:handle.gen+1,..handle},Fixed64::ONE,"extra".into()));
        assert!(!adapter.spend_mana(ParallelWorldAdapter::entity_to_handle(dead),Fixed64::ONE,"extra".into()));
        adapter.restore_mana(ParallelWorldAdapter::entity_to_handle(dead),Fixed64::ONE);
        assert!(adapter.spend_mana(handle,Fixed64::from_i32(30),"extra".into()));
        assert!(!adapter.spend_mana(handle,Fixed64::from_i32(30),"extra".into()));
        assert_eq!(adapter.current_mana(handle),Fixed64::from_i32(20));
        adapter.restore_mana(handle,Fixed64::from_raw(-1));
        adapter.restore_mana(handle,Fixed64::from_raw(i64::MAX));
        adapter.restore_mana(handle,Fixed64::ONE);
        assert_eq!(adapter.current_mana(handle),Fixed64::from_i32(100));
        let changes=adapter.commit_mana_transaction();
        assert_eq!(changes.len(),1);
        assert_eq!(changes[0].1.raw_state(),(100*1024,100*1024,0));
        assert_eq!(cache.hero.get(entity).unwrap().mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(50));
        let outcomes=adapter.finish();
        assert_eq!(outcomes.len(),1);
        let Outcome::ScriptManaCommit {changes,notifications}=&outcomes[0] else {panic!("managed write must be guarded")};
        assert_eq!(changes.len(),1);assert_eq!(changes[0].expected.current(),Fixed64::from_i32(50));
        assert_eq!(changes[0].pool.current(),Fixed64::from_i32(100));
        assert_eq!(notifications.len(),2);
        assert!(matches!(&notifications[0],crate::comp::ManaSettlementNotification::Spent {amount,..} if *amount==Fixed64::from_i32(30)));
        assert!(matches!(&notifications[1],crate::comp::ManaSettlementNotification::Gained {amount,..} if *amount==Fixed64::from_i32(80)));
        let mut adapter=ParallelWorldAdapter::new(&cache,entity);
        adapter.begin_mana_transaction(&HashMap::new());
        adapter.restore_mana(handle,Fixed64::from_i32(10));
        assert!(adapter.spend_mana(handle,Fixed64::from_i32(60),"extra".into()));
        adapter.discard_cast();
        assert_eq!(adapter.current_mana(handle),Fixed64::from_i32(50));
        assert!(adapter.commit_mana_transaction().is_empty());
        assert!(adapter.finish().is_empty());
    }

    #[test]
    fn scripted_attack_interval_uses_final_attack_speed_multiplier() {
        let mut world = world_for_adapter_tests();
        let entity = world
            .create_entity()
            .with(TAttack::new(
                Fixed64::from_i32(10),
                Fixed64::from_i32(1),
                Fixed64::from_i32(100),
                Fixed64::from_i32(900),
            ))
            .build();
        world.write_resource::<BuffStore>().add(
            entity,
            "test_attack_speed",
            Fixed64::from_i32(10),
            json!({ StatKey::AttackSpeedMultiplier.as_str(): 1.2 }),
        );

        let cache = ParallelAdapterCache::new(&world, 123);
        let adapter = ParallelWorldAdapter::new(&cache, entity);
        let interval = adapter.get_asd_interval(ParallelWorldAdapter::entity_to_handle(entity));

        assert!(
            (interval.to_f32_for_render() - (1.0 / 1.2)).abs() < 0.002,
            "expected ~0.833 seconds, got {}",
            interval.to_f32_for_render()
        );
    }

    #[test]
    fn scripted_attack_interval_clamps_to_positive_minimum() {
        let mut world = world_for_adapter_tests();
        let entity = world
            .create_entity()
            .with(TAttack::new(
                Fixed64::from_i32(10),
                Fixed64::ZERO,
                Fixed64::from_i32(100),
                Fixed64::from_i32(900),
            ))
            .build();

        let cache = ParallelAdapterCache::new(&world, 123);
        let adapter = ParallelWorldAdapter::new(&cache, entity);

        assert_eq!(
            adapter.get_asd_interval(ParallelWorldAdapter::entity_to_handle(entity)),
            Fixed64::from_raw(1)
        );
    }

    #[test]
    fn parallel_adapter_rng_uses_stable_request_order_not_completion_order() {
        let mut world = world_for_adapter_tests();
        let a = world.create_entity().build();
        let b = world.create_entity().build();
        let cache = ParallelAdapterCache::new(&world, 98765);

        let mut first_a = ParallelWorldAdapter::new_with_random_ordinal(&cache, a, 0);
        let a_rolls = (first_a.rand_unit(), first_a.rand_unit());
        let mut first_b = ParallelWorldAdapter::new_with_random_ordinal(&cache, b, 1);
        let b_rolls = (first_b.rand_unit(), first_b.rand_unit());

        let mut second_b = ParallelWorldAdapter::new_with_random_ordinal(&cache, b, 1);
        let b_rolls_again = (second_b.rand_unit(), second_b.rand_unit());
        let mut second_a = ParallelWorldAdapter::new_with_random_ordinal(&cache, a, 0);
        let a_rolls_again = (second_a.rand_unit(), second_a.rand_unit());

        assert_eq!(a_rolls, a_rolls_again);
        assert_eq!(b_rolls, b_rolls_again);
        assert_ne!(a_rolls, b_rolls);
    }

    #[test]
    fn parallel_adapter_damage_splash_buffers_deterministic_outcomes() {
        let mut world = world_for_adapter_tests();
        let tower = world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::ZERO, Fixed64::ZERO)))
            .with(Facing(Angle::ZERO))
            .with(Faction {
                faction_id: FactionType::Player,
                team_id: 1,
            })
            .with(Tower::new())
            .build();
        let enemy = world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)))
            .with(Faction {
                faction_id: FactionType::Enemy,
                team_id: 2,
            })
            .with(Creep {
                name: "test_creep".to_string(),
                label: None,
                path: "test_path".to_string(),
                pidx: 0,
                path_remaining_distance: Fixed64::from_i32(1_000_000),
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: None,
            })
            .build();
        let cache = ParallelAdapterCache::new(&world, 123);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);

        adapter.deal_damage_splash(
            Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            Fixed64::from_i32(100),
            Fixed64::from_i32(7),
            DamageKind::Physical,
            AbiDamageProfile::NORMAL,
            RSome(ParallelWorldAdapter::entity_to_handle(tower)),
        );

        let outcomes = adapter.finish();
        assert!(
            matches!(outcomes[0], Outcome::ScriptTowerFireFx { entity, .. } if entity == tower)
        );
        assert!(
            matches!(outcomes[1], Outcome::RawDamage { target, phys, damage_profile, .. }
                if target == enemy
                    && phys == Fixed64::from_i32(7)
                    && damage_profile == AbiDamageProfile::NORMAL.bits())
        );
    }

    #[test]
    fn query_nearest_enemy_uses_selected_tower_priority_for_scripted_towers() {
        let mut world = world_for_adapter_tests();
        let tower = world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::ZERO, Fixed64::ZERO)))
            .with(Faction {
                faction_id: FactionType::Player,
                team_id: 1,
            })
            .with(Tower {
                target_priority: TowerTargetPriority::First,
                ..Tower::new()
            })
            .build();
        let nearest_to_tower = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            500,
            10,
            2,
        );
        let closest_to_exit = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(90), Fixed64::ZERO),
            20,
            10,
            2,
        );
        {
            let mut searcher = world.write_resource::<Searcher>();
            searcher.creep.rebuild_from([
                (nearest_to_tower, vek::Vec2::new(10.0, 0.0)),
                (closest_to_exit, vek::Vec2::new(90.0, 0.0)),
            ]);
        }

        let cache = ParallelAdapterCache::new(&world, 123);
        let adapter = ParallelWorldAdapter::new(&cache, tower);
        let target = adapter.query_nearest_enemy(
            Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            Fixed64::from_i32(100),
            ParallelWorldAdapter::entity_to_handle(tower),
        );

        assert_eq!(
            target,
            RSome(ParallelWorldAdapter::entity_to_handle(closest_to_exit)),
            "First priority should choose the in-range creep nearest the path endpoint, not the closest creep to the tower"
        );
    }

    #[test]
    fn bounded_candidate_query_excludes_caps_and_uses_stable_ties() {
        let mut world = world_for_adapter_tests();
        let tower = world
            .create_entity()
            .with(Pos(Vec2::ZERO))
            .with(Faction::new(FactionType::Player, 1))
            .build();
        let excluded = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(5), Fixed64::ZERO),
            0,
            10,
            2,
        );
        let first_tie = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            0,
            10,
            2,
        );
        let second_tie = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            0,
            10,
            2,
        );
        let farther = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
            0,
            10,
            2,
        );
        let fixed_out_of_range = add_targetable_creep(
            &mut world,
            Vec2::new(Fixed64::from_i32(101), Fixed64::ZERO),
            0,
            10,
            2,
        );
        let decoys: Vec<_> = (0..70)
            .map(|_| {
                add_targetable_creep(
                    &mut world,
                    Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                    0,
                    10,
                    2,
                )
            })
            .collect();
        let mut spatial = decoys
            .iter()
            .rev()
            .map(|entity| (*entity, vek::Vec2::new(10.0, 0.0)))
            .collect::<Vec<_>>();
        spatial.extend([
            (farther, vek::Vec2::new(20.0, 0.0)),
            (second_tie, vek::Vec2::new(10.0, 0.0)),
            (fixed_out_of_range, vek::Vec2::new(1.0, 0.0)),
            (first_tie, vek::Vec2::new(10.0, 0.0)),
            (excluded, vek::Vec2::new(5.0, 0.0)),
        ]);
        world
            .write_resource::<Searcher>()
            .creep
            .rebuild_from(spatial);

        let cache = ParallelAdapterCache::new(&world, 123);
        let query = ParallelProjectileQuery::new(&cache);
        let candidates = query.enemy_candidates_bounded(
            Vec2::ZERO,
            Fixed64::from_i32(100),
            ParallelWorldAdapter::entity_to_handle(tower),
            RSome(ParallelWorldAdapter::entity_to_handle(excluded)),
            2,
        );

        assert_eq!(candidates.len(), 2);
        assert_ne!(
            candidates[0],
            ParallelWorldAdapter::entity_to_handle(excluded)
        );
        assert_ne!(
            candidates[1],
            ParallelWorldAdapter::entity_to_handle(excluded)
        );
        assert!(candidates[0].id < candidates[1].id);
        assert_eq!(
            query.enemy_candidates_bounded(
                Vec2::ZERO,
                Fixed64::from_i32(100),
                ParallelWorldAdapter::entity_to_handle(tower),
                RSome(ParallelWorldAdapter::entity_to_handle(excluded)),
                2,
            ),
            candidates
        );
    }
}

/// Cast-time disclosure query (see `omb_script_abi::world::DisclosureQuery`).
/// Same disclosure rule as summon acquisition, so authority and team replicas
/// select identical candidates before any handler-side cap.
pub(crate) struct CastDisclosureQuery<'a> {
    pub cache: &'a ParallelAdapterCache<'a>,
}

impl omb_script_abi::world::DisclosureQuery for CastDisclosureQuery<'_> {
    fn disclosed_enemies_in_range(&self, center: Vec2, radius: Fixed64, of: EntityHandle, cap: u16) -> RVec<EntityHandle> {
        use crate::runtime::native::combat_acquisition::DisclosureContext;
        let Some(source) = ParallelWorldAdapter::handle_to_entity(of).filter(|e| self.cache.entities.is_alive(*e)) else { return RVec::new(); };
        let Some(faction) = self.cache.faction.get(source) else { return RVec::new(); };
        if radius < Fixed64::ZERO || cap == 0 { return RVec::new(); }
        let disclosure = DisclosureContext::resolve(self.cache.filtered, self.cache.moba.is_some(), self.cache.visibility.as_deref());
        let mut rows: Vec<Entity> = (&self.cache.entities, &self.cache.pos, &self.cache.faction, &self.cache.cprop).join()
            .filter(|(entity, pos, other, property)| *entity != source && property.hp > Fixed64::ZERO
                && faction.is_hostile_to(other)
                && omoba_sim::terrain::point_in_swept_circle(pos.0, center, center, radius)
                && disclosure.allows(Some(faction), crate::runtime::selective::CanonicalEntityKey {
                    id: entity.id(), generation: entity.gen().id() as u32 }))
            .map(|(entity, ..)| entity).collect();
        rows.sort_by_key(|entity| (entity.id(), entity.gen().id()));
        rows.truncate(usize::from(cap.min(256)));
        rows.into_iter().map(ParallelWorldAdapter::entity_to_handle).collect()
    }
}

#[cfg(test)]
mod disclosure_query_tests {
    use super::*;
    use omb_script_abi::world::DisclosureQuery;
    use specs::Builder;
    fn unit(world: &mut World, x: i32, team: i32, hp: i32) -> Entity {
        world.create_entity().with(Pos(Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO)))
            .with(Faction::new(if team == 1 { FactionType::Player } else { FactionType::Enemy }, team))
            .with(CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build()
    }
    #[test]
    fn disclosed_query_is_live_hostile_bounded_sorted_and_capped() {
        let mut world = super::tests::world_for_adapter_tests();
        let source = unit(&mut world, 0, 1, 100);
        let far = unit(&mut world, 300, 2, 100);
        let ally = unit(&mut world, 50, 1, 100);
        let dead = unit(&mut world, 60, 2, 0);
        let near = unit(&mut world, 100, 2, 100);
        let edge = unit(&mut world, 200, 2, 100);
        let cache = ParallelAdapterCache::new(&world, 1);
        let query = CastDisclosureQuery { cache: &cache };
        let handle = ParallelWorldAdapter::entity_to_handle;
        let got = query.disclosed_enemies_in_range(Vec2::ZERO, Fixed64::from_i32(200), handle(source), 8);
        // Closed disk, hostile, hp > 0, (id, gen) order; ally, dead and out-of-range rows excluded.
        assert_eq!(got.to_vec(), vec![handle(near), handle(edge)]);
        assert!(!got.contains(&handle(ally)) && !got.contains(&handle(dead)) && !got.contains(&handle(far)));
        assert_eq!(query.disclosed_enemies_in_range(Vec2::ZERO, Fixed64::from_i32(400), handle(source), 1).to_vec(),
            vec![handle(far)], "cap after canonical (id, gen) order");
        assert!(query.disclosed_enemies_in_range(Vec2::ZERO, Fixed64::from_i32(400), EntityHandle::INVALID, 8).is_empty());
        assert!(query.disclosed_enemies_in_range(Vec2::ZERO, Fixed64::from_raw(-1), handle(source), 8).is_empty());
    }
}
