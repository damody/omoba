use abi_stable::std_types::{RNone, ROption, RSome, RStr, RVec};
use omb_script_abi::{
    stat_keys::StatKey,
    types::{Angle, DamageKind, EntityHandle, Fixed64, PathSpec, ProjectileSpec, Target, Vec2},
    world::GameWorld,
};
use specs::world::Generation;
use specs::{Entities, Entity, Join, Read, ReadStorage, World, WorldExt};
use std::collections::HashMap;

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
    invocation_entity: Entity,
    invocation_rng_op: u32,
    outcomes: Vec<Outcome>,
    overlay_pos: HashMap<Entity, Vec2>,
    overlay_facing: HashMap<Entity, Angle>,
    overlay_asd_count: HashMap<Entity, Fixed64>,
}

impl<'a> ParallelWorldAdapter<'a> {
    pub fn new(cache: &'a ParallelAdapterCache<'a>, invocation_entity: Entity) -> Self {
        Self {
            cache,
            invocation_entity,
            invocation_rng_op: 0,
            outcomes: Vec::new(),
            overlay_pos: HashMap::new(),
            overlay_facing: HashMap::new(),
            overlay_asd_count: HashMap::new(),
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
        let mut out = RVec::new();
        for (ent, pos, fac) in (&self.cache.entities, &self.cache.pos, &self.cache.faction).join() {
            if fac.team_id != my_team && pos.0.distance_squared(center) <= r2 {
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
        _kind: DamageKind,
        _source: ROption<EntityHandle>,
    ) {
        if let Some(ent) = Self::handle_to_entity(target) {
            self.outcomes.push(Outcome::ScriptDirectDamage {
                target: ent,
                amount,
            });
        }
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
            slow_factor: spec.slow_factor,
            slow_duration: spec.slow_duration,
            hit_radius: spec.hit_radius,
            stun_duration: spec.stun_duration,
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
        Self::handle_to_entity(e)
            .and_then(|ent| self.cache.tattack.get(ent).map(|t| t.asd.v))
            .unwrap_or(Fixed64::ZERO)
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
        let my_team = match self.cache.faction.get(of_ent) {
            Some(f) => f.team_id,
            None => return RNone,
        };
        let center_f = vek::Vec2::new(center.x.to_f32_for_render(), center.y.to_f32_for_render());
        let radius_f = radius.to_f32_for_render();
        for di in self.cache.searcher.creep.search_nn(center_f, radius_f, 16) {
            let Some(fac) = self.cache.faction.get(di.e) else {
                continue;
            };
            if fac.team_id != my_team && self.cache.creep.get(di.e).is_some() {
                return RSome(Self::entity_to_handle(di.e));
            }
        }
        RNone
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
        let op_kind = 1_000u32.wrapping_add(self.invocation_rng_op);
        self.invocation_rng_op = self.invocation_rng_op.wrapping_add(1);
        let mut rng = omoba_sim::SimRng::from_master_entity(
            self.cache.rng_seed,
            self.cache.tick.0 as u32,
            self.invocation_entity.id(),
            op_kind,
        );
        rng.gen_fixed64_unit()
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
            .and_then(|t| t.upgrade_levels.get(path as usize))
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
            self.deal_damage(*th, damage, kind, source);
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
    use omb_script_abi::world::GameWorld;
    use specs::Builder;

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
        world.insert(BuffStore::default());
        world.insert(Searcher::default());
        world.insert(BlockedRegions::default());
        world.insert(Tick(42));
        world
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
    fn parallel_adapter_rng_depends_on_entity_not_schedule_order() {
        let mut world = world_for_adapter_tests();
        let a = world.create_entity().build();
        let b = world.create_entity().build();
        let cache = ParallelAdapterCache::new(&world, 98765);

        let mut first_a = ParallelWorldAdapter::new(&cache, a);
        let a_rolls = (first_a.rand_unit(), first_a.rand_unit());
        let mut first_b = ParallelWorldAdapter::new(&cache, b);
        let b_rolls = (first_b.rand_unit(), first_b.rand_unit());

        let mut second_b = ParallelWorldAdapter::new(&cache, b);
        let b_rolls_again = (second_b.rand_unit(), second_b.rand_unit());
        let mut second_a = ParallelWorldAdapter::new(&cache, a);
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
                block_tower: None,
                status: CreepStatus::Walk,
            })
            .build();
        let cache = ParallelAdapterCache::new(&world, 123);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);

        adapter.deal_damage_splash(
            Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            Fixed64::from_i32(100),
            Fixed64::from_i32(7),
            DamageKind::Physical,
            RSome(ParallelWorldAdapter::entity_to_handle(tower)),
        );

        let outcomes = adapter.finish();
        assert!(
            matches!(outcomes[0], Outcome::ScriptTowerFireFx { entity, .. } if entity == tower)
        );
        assert!(
            matches!(outcomes[1], Outcome::ScriptDirectDamage { target, amount } if target == enemy && amount == Fixed64::from_i32(7))
        );
    }
}
