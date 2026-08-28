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
    pub entities: Entities<'a>,
    pub tattack: ReadStorage<'a, TAttack>,
    pub pos: ReadStorage<'a, Pos>,
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
}

impl<'a> ParallelAdapterCache<'a> {
    pub fn new(world: &'a World, rng_seed: u64) -> Self {
        Self {
            entities: world.entities(),
            tattack: world.read_storage::<TAttack>(),
            pos: world.read_storage::<Pos>(),
            facing: world.read_storage::<Facing>(),
            cprop: world.read_storage::<CProperty>(),
            unit: world.read_storage::<Unit>(),
            hero: world.read_storage::<Hero>(),
            faction: world.read_storage::<Faction>(),
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
        }
    }
}

pub struct ParallelWorldAdapter<'a> {
    pub cache: &'a ParallelAdapterCache<'a>,
    invocation_rng_op: u32,
    random_request_base: u64,
    outcomes: Vec<Outcome>,
    overlay_pos: HashMap<Entity, Vec2>,
    overlay_facing: HashMap<Entity, Angle>,
    overlay_asd_count: HashMap<Entity, Fixed64>,
    projectile_hit_generation: Option<u8>,
}

pub struct ParallelProjectileQuery<'a> {
    cache: &'a ParallelAdapterCache<'a>,
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
            (base_interval / stats.final_attack_speed_mult(entity)).max(Fixed64::from_raw(1));
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
        Self { cache }
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
        let Some(my_faction) = self.cache.faction.get(of_entity) else {
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
    let Some(of_ent) = ParallelWorldAdapter::handle_to_entity(of) else {
        return RNone;
    };
    let my_team = match cache.faction.get(of_ent) {
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
        let camo_visible = match (cache.tower.get(of_ent), target_creep) {
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
            overlay_facing: HashMap::new(),
            overlay_asd_count: HashMap::new(),
            projectile_hit_generation: None,
        }
    }

    pub fn finish(self) -> Vec<Outcome> {
        self.outcomes
    }

    pub fn start_cooldown(&mut self, entity: Entity, ability_id: String, duration: Fixed64) {
        self.outcomes.push(Outcome::ScriptStartCooldown {
            entity,
            ability_id,
            duration,
        });
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
        self.cache.pos.get(ent).map(|p| p.0).map_or(RNone, RSome)
    }

    fn get_hp(&self, e: EntityHandle) -> ROption<Fixed64> {
        let Some(ent) = Self::handle_to_entity(e) else {
            return RNone;
        };
        if let Some(p) = self.cache.cprop.get(ent) {
            return RSome(p.hp);
        }
        self.cache
            .unit
            .get(ent)
            .map(|u| Fixed64::from_i32(u.current_hp))
            .map_or(RNone, RSome)
    }

    fn get_max_hp(&self, e: EntityHandle) -> ROption<Fixed64> {
        let Some(ent) = Self::handle_to_entity(e) else {
            return RNone;
        };
        if let Some(p) = self.cache.cprop.get(ent) {
            return RSome(p.mhp);
        }
        self.cache
            .unit
            .get(ent)
            .map(|u| Fixed64::from_i32(u.max_hp))
            .map_or(RNone, RSome)
    }

    fn is_alive(&self, e: EntityHandle) -> bool {
        Self::handle_to_entity(e)
            .map(|ent| self.cache.entities.is_alive(ent))
            .unwrap_or(false)
    }

    fn faction_of(&self, _e: EntityHandle) -> ROption<RStr<'_>> {
        RNone
    }

    fn unit_id_of(&self, _e: EntityHandle) -> ROption<RStr<'_>> {
        RNone
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
        let my_team = match self.cache.faction.get(of_ent) {
            Some(f) => f.team_id,
            None => return RVec::new(),
        };
        let r2 = radius * radius;
        let source_tower = self.cache.tower.get(of_ent);
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
        let radius = self
            .cache
            .collision
            .get(ent)
            .map(|r| r.0)
            .unwrap_or(Fixed64::from_i32(30));
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
        self.outcomes.push(Outcome::Damage {
            pos,
            phys,
            magi,
            real,
            source,
            target,
            damage_profile: valid_profile.bits(),
            predeclared: false,
        });
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
            .map(|ent| self.cache.buffs.has(ent, buff_id.as_str()))
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
        _pos: Vec2,
        _unit_type: RStr<'_>,
        _owner: EntityHandle,
        _duration: Fixed64,
    ) -> EntityHandle {
        EntityHandle::INVALID
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
        let from = spec.from;
        let (target_opt, tpos) = match spec.path {
            PathSpec::Homing { target } => {
                let Some(target_ent) = Self::handle_to_entity(target) else {
                    return EntityHandle::INVALID;
                };
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
        self.outcomes.push(Outcome::ScriptProjectile {
            pos: from,
            owner: owner_ent,
            target: target_opt,
            tpos,
            radius: spec.splash_radius,
            msd: spec.speed,
            damage_phys: spec.damage,
            damage_magi: Fixed64::ZERO,
            damage_real: Fixed64::ZERO,
            damage_profile: profile.bits(),
            slow_factor: spec.slow_factor,
            slow_duration: spec.slow_duration,
            hit_radius: spec.hit_radius,
            stun_duration: spec.stun_duration,
            kind_id: spec.kind_id,
            generation: self
                .projectile_hit_generation
                .map(|generation| generation.saturating_add(1))
                .unwrap_or(0),
        });
        EntityHandle::INVALID
    }

    fn emit_explosion(&mut self, pos: Vec2, radius: Fixed64, duration: Fixed64) {
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
        Self::handle_to_entity(e)
            .and_then(|ent| self.cache.tattack.get(ent).map(|t| t.range.v))
            .unwrap_or(Fixed64::ZERO)
    }

    fn get_tower_atk(&self, e: EntityHandle) -> Fixed64 {
        Self::handle_to_entity(e)
            .and_then(|ent| self.cache.tattack.get(ent).map(|t| t.atk_physic.v))
            .unwrap_or(Fixed64::ZERO)
    }

    fn get_asd_interval(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let Some(base_interval) = self.cache.tattack.get(ent).map(|t| t.asd.v) else {
            return Fixed64::ZERO;
        };
        let stats =
            UnitStats::from_refs(&self.cache.buffs, self.cache.is_building.get(ent).is_some());
        let interval = base_interval / stats.final_attack_speed_mult(ent);
        interval.max(Fixed64::from_raw(1))
    }

    fn get_asd_count(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        self.overlay_asd_count
            .get(&ent)
            .copied()
            .or_else(|| self.cache.tattack.get(ent).map(|t| t.asd_count))
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
        let priority = self
            .cache
            .tower
            .get(of_ent)
            .map(|tower| tower.target_priority)
            .unwrap_or(TowerTargetPriority::Nearest);
        query_script_tower_enemy(self.cache, center, radius, of, priority)
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
            .map(|ent| self.cache.buffs.sum_add(ent, stat_key))
            .unwrap_or(Fixed64::ZERO)
    }

    fn product_stat(&self, e: EntityHandle, stat_key: StatKey) -> Fixed64 {
        Self::handle_to_entity(e)
            .map(|ent| self.cache.buffs.product_mult(ent, stat_key))
            .unwrap_or(Fixed64::ONE)
    }

    fn get_final_move_speed(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self
            .cache
            .cprop
            .get(ent)
            .map(|p| p.msd)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .final_move_speed(base, ent)
    }

    fn get_final_atk(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self
            .cache
            .tattack
            .get(ent)
            .map(|t| t.atk_physic.v)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .final_atk(base, ent)
    }

    fn get_tower_upgrade(&self, e: EntityHandle, path: u8) -> u8 {
        Self::handle_to_entity(e)
            .and_then(|ent| self.cache.tower.get(ent))
            .and_then(|tower| tower.upgrade_levels.get(path as usize))
            .copied()
            .unwrap_or(0)
    }

    fn has_tower_flag(&self, e: EntityHandle, flag: RStr<'_>) -> bool {
        Self::handle_to_entity(e)
            .and_then(|ent| self.cache.tower.get(ent))
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
        let base = self
            .cache
            .tattack
            .get(ent)
            .map(|t| t.range.v)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .final_attack_range(base, ent)
    }

    fn get_buff_remaining(&self, e: EntityHandle, buff_id: RStr<'_>) -> Fixed64 {
        Self::handle_to_entity(e)
            .and_then(|ent| {
                self.cache
                    .buffs
                    .get(ent, buff_id.as_str())
                    .map(|b| b.remaining)
            })
            .unwrap_or(Fixed64::ZERO)
    }

    fn current_mana(&self, e: EntityHandle) -> Fixed64 {
        Self::handle_to_entity(e)
            .and_then(|ent| self.cache.hero.get(ent).map(|h| h.get_max_mana()))
            .unwrap_or(Fixed64::ZERO)
    }

    fn spend_mana(&mut self, _e: EntityHandle, _amount: Fixed64, _ability_id: RStr<'_>) -> bool {
        true
    }

    fn restore_mana(&mut self, _e: EntityHandle, _amount: Fixed64) {}

    fn trigger_state_changed(&mut self, _e: EntityHandle, _state_id: RStr<'_>, _active: bool) {}

    fn get_final_armor(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self
            .cache
            .cprop
            .get(ent)
            .map(|c| c.def_physic)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .final_armor(base, ent)
    }

    fn get_final_magic_resist(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        let base = self
            .cache
            .cprop
            .get(ent)
            .map(|c| c.def_magic)
            .unwrap_or(Fixed64::ZERO);
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .final_magic_resist(base, ent)
    }

    fn get_evasion_chance(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .evasion_chance(ent)
    }

    fn get_miss_chance(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .miss_chance(ent)
    }

    fn get_crit_chance(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .crit(ent)
        .0
    }

    fn get_crit_multiplier(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ONE;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .crit(ent)
        .1
    }

    fn get_cooldown_mult(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ONE;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .cooldown_mult(ent)
    }

    fn is_building(&self, e: EntityHandle) -> bool {
        Self::handle_to_entity(e)
            .map(|ent| self.cache.is_building.get(ent).is_some())
            .unwrap_or(false)
    }

    fn get_max_hp_bonus(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .max_hp_bonus(ent)
    }

    fn get_hp_regen(&self, e: EntityHandle) -> Fixed64 {
        let Some(ent) = Self::handle_to_entity(e) else {
            return Fixed64::ZERO;
        };
        UnitStats::from_refs(
            &*self.cache.buffs,
            self.cache.is_building.get(ent).is_some(),
        )
        .hp_regen(Fixed64::ZERO, ent)
    }

    fn get_stat_bonus(&self, e: EntityHandle, key: StatKey) -> Fixed64 {
        Self::handle_to_entity(e)
            .map(|ent| self.cache.buffs.sum_add(ent, key))
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

    fn world_for_adapter_tests() -> World {
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
            matches!(outcomes[1], Outcome::Damage { target, phys, damage_profile, .. }
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
