//! Generic summon AI for every summon template without a dedicated script (and not a dummy):
//! the template's stats are the whole behaviour, so content-defined summons (v3 unit_templates)
//! are pure data. Each tick:
//! 1. an enemy within attack range (the host query: an assigned target first, else the nearest
//!    disclosed attackable enemy): face it and attack every attack interval with an ordinary
//!    homing attack at the template's projectile speed;
//! 2. else, a mobile summon closes on an enemy within its acquisition range (原作
//!    AttackAcquisitionRange) through the host collision step;
//! 3. else nothing: the host follow rules (follow distances) and owner orders move it.
//! `ai = idle` never acquires a target by itself. Fixed emplacements (immobile) never move.
//! While it winds up a cast its owner ordered (D19, the host's `cast_preparation`) it neither
//! attacks nor moves, as a casting hero: a moved caster's windup ends.
use omb_script_abi::prelude::*;
use omoba_template_ids::SummonAi;

/// The host's status on a unit winding up a cast (omoba-core cast_windup).
const CAST_PREPARATION: &str = "cast_preparation";

#[derive(Clone)]
pub struct Generic {
    pub unit_id: RString,
    pub acquisition: Fixed64,
    pub ai: SummonAi,
    pub immobile: bool,
    pub projectile_speed: Fixed64,
    /// Its attack's damage type (E503).
    pub damage_type: omoba_template_ids::SummonDamageType,
    /// It never attacks (E511): no target of its own, no attack.
    pub no_attack: bool,
}

impl Generic {
    /// The behaviour of template `name`, from its stats; None for a dummy (it holds statuses only).
    pub fn of(name: &str) -> Option<Self> {
        let stats = omoba_template_ids::summon_by_name(name).and_then(omoba_template_ids::active_summon_stats)?;
        // A chase summon (E406) is flown by the host (omoba-core summon_chase).
        (!stats.dummy && stats.chase.is_none()).then(|| Generic { unit_id: name.into(), acquisition: stats.acquisition_range, ai: stats.ai,
            immobile: stats.immobile, projectile_speed: stats.projectile_speed, damage_type: stats.attack_damage_type,
            no_attack: stats.no_attack })
    }
}

impl UnitScript for Generic {
    fn unit_id(&self) -> RStr<'_> {
        self.unit_id.as_rstr()
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        if self.ai == SummonAi::Idle || self.no_attack { return; }
        if w.has_buff(e, CAST_PREPARATION.into()) { return; }
        let RSome(pos) = w.get_pos(e) else { return; };
        let interval = w.get_asd_interval(e);
        if interval <= Fixed64::ZERO { return; }
        let reach = w.get_tower_range(e);
        if let RSome(target) = w.query_nearest_enemy(pos, reach, e) {
            if let RSome(tpos) = w.get_pos(target) {
                let (dx, dy) = (tpos.x - pos.x, tpos.y - pos.y);
                if dx * dx + dy * dy > Fixed64::from_raw(1) { w.set_facing(e, omoba_sim::trig::atan2(dy, dx)); }
            }
            let mut count = w.get_asd_count(e);
            if count < interval { count += dt; w.set_asd_count(e, count); }
            if count < interval { return; }
            w.set_asd_count(e, count - interval);
            let backswing_ms = (interval.raw().max(0) as u64).saturating_mul(1000) / 1024;
            w.emit_attack_phase_fx(e, Target::Entity(target), 0, backswing_ms.min(u32::MAX as u64) as u32);
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos, owner: e, path: PathSpec::AttackHoming { target }, speed: self.projectile_speed,
                damage: w.get_tower_atk(e), damage_profile: match self.damage_type {
                    omoba_template_ids::SummonDamageType::Physical => DamageProfile::NORMAL,
                    omoba_template_ids::SummonDamageType::Magical => DamageProfile::ENERGY,
                    omoba_template_ids::SummonDamageType::Pure => DamageProfile::TRUE,
                },
                hit_radius: Fixed64::ZERO, splash_radius: Fixed64::ZERO, slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO, stun_duration: Fixed64::ZERO, kind_id: 0,
            });
            return;
        }
        if self.immobile { return; }
        let RSome(chase) = w.query_nearest_enemy(pos, self.acquisition.max(reach), e) else { return; };
        let RSome(tpos) = w.get_pos(chase) else { return; };
        let (dx, dy) = (tpos.x - pos.x, tpos.y - pos.y);
        if dx * dx + dy * dy <= Fixed64::ONE { return; }
        let step = w.get_final_move_speed(e) * dt;
        let next = w.advance_with_collision(e, tpos, step);
        w.set_pos(e, next);
        w.set_facing(e, omoba_sim::trig::atan2(dy, dx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi_stable::{sabi_trait::prelude::TD_Opaque, RMut};
    use omb_script_abi::world::GameWorld_TO;
    use omoba_core::{CProperty, Faction, FactionType, Outcome, Pos};
    use omoba_core::runtime::{StateInitializer, TowerTemplateRegistry,
        scripting::parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter}};
    use specs::{Builder, World, WorldExt};

    fn p(x: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO) }

    fn tick(world: &World, ai: &Generic, e: specs::Entity) -> Vec<Outcome> {
        let cache = ParallelAdapterCache::new(world, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, e);
        let mut w = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        ai.on_tick(ParallelWorldAdapter::entity_to_handle(e), Fixed64::ONE, &mut w);
        drop(w); adapter.finish()
    }

    /// Attacks in range, closes within acquisition, holds beyond it; idle never acts.
    #[test]
    fn a_generic_summon_attacks_in_range_closes_within_acquisition_and_idle_does_nothing() {
        let mut world = StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
        world.insert(TowerTemplateRegistry::default());
        let body = |world: &mut World, at: Vec2, team: i32| world.create_entity().with(Pos(at))
            .with(Faction::new(if team == 1 { FactionType::Player } else { FactionType::Enemy }, team))
            .with(CProperty { hp: Fixed64::from_i32(1000), mhp: Fixed64::from_i32(1000), msd: Fixed64::from_i32(300),
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(omoba_core::Facing(omoba_sim::Angle::ZERO)).build();
        let owner = body(&mut world, p(-500), 1);
        let summon = body(&mut world, p(0), 1);
        world.write_storage::<omoba_core::SummonedUnit>().insert(summon, omoba_core::SummonedUnit::new_fixed(owner, None)).unwrap();
        world.write_storage::<omoba_core::Unit>().insert(summon, omoba_core::Unit { id: "test_guard".into(), unit_type: omoba_core::UnitType::Summon, ..Default::default() }).unwrap();
        world.write_storage::<omoba_core::TAttack>().insert(summon, omoba_core::TAttack::new(
            Fixed64::from_i32(30), Fixed64::ONE, Fixed64::from_i32(200), Fixed64::from_i32(900))).unwrap();
        let enemy = body(&mut world, p(150), 2);
        let guard = Generic { unit_id: "test_guard".into(), acquisition: Fixed64::from_i32(600), ai: SummonAi::Guard,
            immobile: false, projectile_speed: Fixed64::from_i32(1500), damage_type: omoba_template_ids::SummonDamageType::Physical,
            no_attack: false };
        let rows = tick(&world, &guard, summon);
        assert!(rows.iter().any(|o| matches!(o, Outcome::ScriptProjectile { .. })), "in range: attacks {rows:?}");
        // E511: a summon without attack neither attacks nor closes on the enemy.
        world.write_storage::<omoba_core::TAttack>().get_mut(summon).unwrap().asd_count = Fixed64::from_i32(5);
        let unarmed = Generic { no_attack: true, ..guard.clone() };
        assert!(tick(&world, &unarmed, summon).is_empty(), "no attack");
        // E503: the attack's damage is of the template's type.
        for (kind, phys, magi, real) in [(omoba_template_ids::SummonDamageType::Physical, true, false, false),
            (omoba_template_ids::SummonDamageType::Magical, false, true, false), (omoba_template_ids::SummonDamageType::Pure, false, false, true)] {
            let typed = Generic { damage_type: kind, ..guard.clone() };
            world.write_storage::<omoba_core::TAttack>().get_mut(summon).unwrap().asd_count = Fixed64::from_i32(5);
            let rows = tick(&world, &typed, summon);
            assert!(rows.iter().any(|o| matches!(o, Outcome::ScriptProjectile { damage_phys, damage_magi, damage_real, .. }
                if (*damage_phys > Fixed64::ZERO, *damage_magi > Fixed64::ZERO, *damage_real > Fixed64::ZERO) == (phys, magi, real))),
                "{kind:?}: {rows:?}");
        }
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = p(500);
        let rows = tick(&world, &guard, summon);
        assert!(rows.iter().any(|o| matches!(o, Outcome::ScriptSetPos { .. })), "within acquisition: closes {rows:?}");
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = p(900);
        assert!(tick(&world, &guard, summon).is_empty(), "beyond acquisition: holds");
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = p(150);
        let idle = Generic { ai: SummonAi::Idle, ..guard };
        assert!(tick(&world, &idle, summon).is_empty(), "idle never acquires");
    }
}
