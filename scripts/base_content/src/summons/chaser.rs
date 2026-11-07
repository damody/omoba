//! Generic flying chase AI (summon_flying_chase) for mobile summons that hunt
//! one target, like 原作 C15T_dummy (DOTA_UNIT_CAP_MOVE_FLY, melee, acquisition
//! range): every tick it takes the host-selected enemy within `acquisition`
//! (an assigned `summon_target` first, else the nearest disclosed enemy), flies
//! straight at it at its final move speed ignoring terrain and unit collision,
//! and once the centre distance is within its attack range it attacks
//! with an ordinary attack (`AttackHoming`) every attack interval. The first
//! swing is immediate (Dota attacks are ready on arrival); later swings wait
//! the interval. Without a target it holds position.
use omb_script_abi::prelude::*;

pub struct Chaser {
    pub unit_id: RString,
    /// Auto-acquisition radius (original `AttackAcquisitionRange`).
    pub acquisition: Fixed64,
}

/// Marker: this summon life has swung at least once.
pub const SWUNG: &str = "summon_chaser_swung";
const ATTACK_SPEED: Fixed64 = Fixed64::from_i32(10_000);

impl UnitScript for Chaser {
    fn unit_id(&self) -> RStr<'_> {
        self.unit_id.as_rstr()
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let RSome(pos) = w.get_pos(e) else { return; };
        let interval = w.get_asd_interval(e);
        if interval <= Fixed64::ZERO || dt <= Fixed64::ZERO { return; }
        let RSome(target) = w.query_nearest_enemy(pos, self.acquisition, e) else { return; };
        let RSome(tpos) = w.get_pos(target) else { return; };
        let delta = tpos - pos;
        let distance = delta.length();
        if distance > Fixed64::from_raw(1) { w.set_facing(e, omoba_sim::trig::atan2(delta.y, delta.x)); }
        let swung = w.has_buff(e, SWUNG.into());
        let mut count = w.get_asd_count(e);
        if swung && count < interval { count += dt; w.set_asd_count(e, count); }
        // The host admits a basic attack only within the attack range (centre distance).
        let reach = w.get_tower_range(e);
        if distance > reach {
            // Flying: a straight step, never blocked by terrain or units.
            let step = (w.get_final_move_speed(e) * dt).min(distance - reach);
            if step > Fixed64::ZERO && distance > Fixed64::ZERO {
                w.set_pos(e, pos + Vec2::new(delta.x * step / distance, delta.y * step / distance));
            }
            return;
        }
        if swung && count < interval { return; }
        w.set_asd_count(e, if swung { count - interval } else { Fixed64::ZERO });
        if !swung { w.add_stat_buff(e, SWUNG.into(), Fixed64::from_i32(3600), "{}".into()); }
        let backswing_ms = (interval.raw().max(0) as u64).saturating_mul(1000) / 1024;
        w.emit_attack_phase_fx(e, Target::Entity(target), 0, backswing_ms.min(u32::MAX as u64) as u32);
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos, owner: e, path: PathSpec::AttackHoming { target }, speed: ATTACK_SPEED,
            damage: w.get_tower_atk(e), damage_profile: DamageProfile::NORMAL,
            hit_radius: Fixed64::ZERO, splash_radius: Fixed64::ZERO, slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO, stun_duration: Fixed64::ZERO, kind_id: 0,
        });
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

    fn p(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

    fn tick(world: &World, chaser: &Chaser, e: specs::Entity) -> Vec<Outcome> {
        let cache = ParallelAdapterCache::new(world, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, e);
        let mut w = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        chaser.on_tick(ParallelWorldAdapter::entity_to_handle(e), Fixed64::from_raw(17), &mut w);
        drop(w); adapter.finish()
    }

    /// Flies straight at the target, swings at once on contact, holds without a target.
    #[test]
    fn flies_straight_and_attacks_on_contact() {
        let mut world = StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
        world.insert(TowerTemplateRegistry::default());
        let body = |world: &mut World, at: Vec2, team: i32| world.create_entity().with(Pos(at))
            .with(Faction::new(if team == 1 { FactionType::Player } else { FactionType::Enemy }, team))
            .with(CProperty { hp: Fixed64::from_i32(1000), mhp: Fixed64::from_i32(1000), msd: Fixed64::from_i32(900),
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(omoba_core::Facing(omoba_sim::Angle::ZERO)).build();
        let owner = body(&mut world, p(-500, 0), 1);
        let arrow = body(&mut world, p(0, 0), 1);
        world.write_storage::<omoba_core::SummonedUnit>().insert(arrow, omoba_core::SummonedUnit::new_fixed(owner, Some(Fixed64::from_i32(10)))).unwrap();
        world.write_storage::<omoba_core::Unit>().insert(arrow, omoba_core::Unit { id: "test_arrow".into(), unit_type: omoba_core::UnitType::Summon, ..Default::default() }).unwrap();
        world.write_storage::<omoba_core::TAttack>().insert(arrow, omoba_core::TAttack::new(
            Fixed64::from_i32(50), Fixed64::ONE, Fixed64::from_i32(58), Fixed64::from_i32(900))).unwrap();
        let enemy = body(&mut world, p(300, 0), 2);
        let chaser = Chaser { unit_id: "test_arrow".into(), acquisition: Fixed64::from_i32(900) };
        let rows = tick(&world, &chaser, arrow);
        let moved = rows.iter().find_map(|o| match o { Outcome::ScriptSetPos { entity, pos } if *entity == arrow => Some(*pos), _ => None })
            .expect("flies");
        assert_eq!(moved.y, Fixed64::ZERO);
        assert!(moved.x > Fixed64::from_i32(14) && moved.x < Fixed64::from_i32(16), "900/s for one 17/1024 s tick: {moved:?}");
        assert!(!rows.iter().any(|o| matches!(o, Outcome::ScriptProjectile { .. })));
        // Within the attack range 58: the first swing is immediate.
        world.write_storage::<Pos>().get_mut(arrow).unwrap().0 = p(250, 0);
        let rows = tick(&world, &chaser, arrow);
        assert!(rows.iter().any(|o| matches!(o, Outcome::ScriptProjectile { .. })), "first swing on arrival");
        assert!(!rows.iter().any(|o| matches!(o, Outcome::ScriptSetPos { .. })), "no step inside reach");
        // Out of acquisition range: holds.
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = p(1300, 0);
        assert!(tick(&world, &chaser, arrow).is_empty());
        let _ = enemy;
    }
}
