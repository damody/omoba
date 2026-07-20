//! Arty Tower（繽紛吉拿棒迫擊砲）— 超長程 AoE 砲，命中時可附加暈眩。
//!
//! 支援:
//! - Path1: damage_bonus、splash_bonus、range_bonus（透過 get_final_*）
//! - Path2: arty_stun（1.0s）, arty_stun_2（2.0s）, arty_stun_3（3.0s）, arty_slow_50（5.0s 50% 減速）
//! - Path3: attack_speed_multiplier，最終解鎖 arty_fire_at_will
//! - 高傷害 / 慢攻速 / 超長射程，適合控制型打法

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct ArtyTower;

const STATS: &TowerStats = &TOWER_ARTY_STATS;
const CONTROL_SLOW_FACTOR: Fixed64 = Fixed64::from_raw(512);
const CONTROL_SLOW_DURATION: Fixed64 = Fixed64::from_i32(5);
const ARTY_FIRE_AT_WILL_PULSES: u16 = 6;
const ARTY_FIRE_AT_WILL_INTERVAL: Fixed64 = Fixed64::from_raw(512);
const ARTY_FIRE_AT_WILL_ID: &str = "arty_fire_at_will";

fn fire_shell(e: EntityHandle, target: EntityHandle, w: &mut GameWorldDyn<'_>) -> bool {
    let pos = match w.get_pos(e) {
        RSome(pos) => pos,
        RNone => return false,
    };
    let stats = super::tower_stats(TOWER_ARTY, STATS);
    let splash = stats.splash_radius + w.get_stat_bonus(e, StatKey::SplashBonus);
    let stun = if w.has_tower_flag(e, RStr::from_str("arty_stun_3")) {
        Fixed64::from_i32(3)
    } else if w.has_tower_flag(e, RStr::from_str("arty_stun_2")) {
        Fixed64::from_i32(2)
    } else if w.has_tower_flag(e, RStr::from_str("arty_stun")) {
        Fixed64::from_i32(1)
    } else {
        Fixed64::ZERO
    };
    let (slow_factor, slow_duration) = if w.has_tower_flag(e, RStr::from_str("arty_slow_50")) {
        (CONTROL_SLOW_FACTOR, CONTROL_SLOW_DURATION)
    } else {
        (Fixed64::ZERO, Fixed64::ZERO)
    };
    if let RSome(t_pos) = w.get_pos(target) {
        w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
    }
    w.spawn_projectile_ex(ProjectileSpec {
        from: pos,
        owner: e,
        path: PathSpec::Homing { target },
        speed: stats.bullet_speed,
        damage: w.get_final_atk(e),
        hit_radius: Fixed64::ZERO,
        splash_radius: splash,
        slow_factor,
        slow_duration,
        stun_duration: stun,
        kind_id: PROJECTILE_BOMB.0,
    });
    true
}

impl UnitScript for ArtyTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_ARTY.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_ARTY, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_ARTY,
            STATS,
            &TOWER_ARTY_RENDER,
            TOWER_ARTY_ATTACK_TIMING,
        ))
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed64::ZERO {
            return;
        }
        let timing = super::tower_attack_timing(TOWER_ARTY, TOWER_ARTY_ATTACK_TIMING);
        let phase = super::advance_attack_phase(e, dt, asd_interval, timing, w);
        if matches!(phase, super::AttackPhaseStep::Charging) {
            return;
        }

        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let range = w.get_final_attack_range(e);
        let target = match w.query_nearest_enemy(pos, range, e) {
            RSome(t) => t,
            RNone => return,
        };
        if matches!(phase, super::AttackPhaseStep::Ready) {
            if let RSome(t_pos) = w.get_pos(target) {
                w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
            }
            super::start_attack_windup(e, asd_interval, timing, Target::Entity(target), w);
            return;
        }

        w.log_info(RStr::from_str("[tower_arty] fire!"));
        fire_shell(e, target, w);
    }

    fn on_tower_ability_pulse_with_access(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        pulse_index: u16,
        access: &TowerActiveAbilityAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) -> bool {
        if ability_id.as_str() != ARTY_FIRE_AT_WILL_ID || pulse_index >= ARTY_FIRE_AT_WILL_PULSES {
            return false;
        }
        debug_assert_eq!(ARTY_FIRE_AT_WILL_INTERVAL, Fixed64::from_raw(512));
        let pos = match w.get_pos(tower) {
            RSome(pos) => pos,
            RNone => return false,
        };
        let range = w.get_final_attack_range(tower);
        let target = match access.query_first_enemy_in_range(pos, range, tower) {
            RSome(target) => target,
            RNone => return false,
        };
        fire_shell(tower, target, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke_tick};
    use abi_stable::{sabi_trait::prelude::TD_Opaque, RMut, RRef};
    use omb_script_abi::world::{GameWorld_TO, TowerActiveAbilityAccess_TO};
    use omoba_core::scripting::parallel_world_adapter::{
        ParallelAdapterCache, ParallelTowerActiveAbilityAccess, ParallelWorldAdapter,
    };
    use omoba_core::Outcome;
    use specs::WorldExt;

    fn invoke_pulse(fixture: &specs::World, tower: specs::Entity) -> (bool, Vec<Outcome>) {
        let cache = ParallelAdapterCache::new(fixture, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);
        let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
        let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        let access_dyn =
            TowerActiveAbilityAccess_TO::from_ptr(RRef::new(&access_adapter), TD_Opaque);
        let consumed = ArtyTower.on_tower_ability_pulse_with_access(
            EntityHandle {
                id: tower.id(),
                gen: tower.gen().id() as u32,
            },
            RStr::from_str("arty_fire_at_will"),
            0,
            &access_dyn,
            &mut world_dyn,
        );
        drop(access_dyn);
        drop(world_dyn);
        (consumed, adapter.finish())
    }

    #[test]
    fn fire_at_will_has_six_pulses() {
        assert_eq!(ARTY_FIRE_AT_WILL_PULSES, 6);
        assert_eq!(ARTY_FIRE_AT_WILL_INTERVAL, Fixed64::from_raw(512));
    }

    #[test]
    fn fire_at_will_preserves_charge_when_no_target_exists() {
        let fixture = fixture(&[], &[]);

        let (consumed, outcomes) = invoke_pulse(&fixture.world, fixture.tower);

        assert!(!consumed);
        assert!(!outcomes
            .iter()
            .any(|outcome| matches!(outcome, Outcome::ScriptProjectile { .. })));
    }

    #[test]
    fn fire_at_will_uses_first_priority_and_current_control_shell() {
        let fixture = fixture(
            &["arty_stun_3", "arty_slow_50"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
            ],
        );
        fixture
            .world
            .write_storage::<omoba_core::Tower>()
            .get_mut(fixture.tower)
            .unwrap()
            .target_priority = omoba_core::TowerTargetPriority::Nearest;
        {
            let mut creeps = fixture.world.write_storage::<omoba_core::Creep>();
            creeps
                .get_mut(fixture.enemies[0])
                .unwrap()
                .path_remaining_distance = Fixed64::from_i32(100);
            creeps
                .get_mut(fixture.enemies[1])
                .unwrap()
                .path_remaining_distance = Fixed64::from_i32(10);
        }

        let (consumed, outcomes) = invoke_pulse(&fixture.world, fixture.tower);

        assert!(consumed);
        assert!(
            outcomes.iter().any(|outcome| matches!(outcome,
                Outcome::ScriptProjectile {
                    target: Some(target),
                    damage_phys,
                    slow_factor,
                    slow_duration,
                    stun_duration,
                    ..
                } if *target == fixture.enemies[1]
                    && *damage_phys == Fixed64::from_i32(10)
                    && *slow_factor == Fixed64::from_raw(512)
                    && *slow_duration == Fixed64::from_i32(5)
                    && *stun_duration == Fixed64::from_i32(3)
            )),
            "missing authored Fire at Will projectile: {outcomes:?}"
        );
    }

    #[test]
    fn level_four_control_shell_has_three_second_stun_and_finite_half_speed_slow() {
        let fixture = fixture(
            &["arty_stun", "arty_stun_2", "arty_stun_3", "arty_slow_50"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        fixture
            .world
            .write_storage::<omoba_core::TAttack>()
            .get_mut(fixture.tower)
            .unwrap()
            .asd_count = -Fixed64::from_raw(1);

        let outcomes = invoke_tick(
            &fixture.world,
            &ArtyTower,
            fixture.tower,
            Fixed64::from_raw(1),
        );

        assert!(
            outcomes.iter().any(|outcome| matches!(outcome,
                Outcome::ScriptProjectile {
                    stun_duration,
                    slow_factor,
                    slow_duration,
                    ..
                } if *stun_duration == Fixed64::from_i32(3)
                    && *slow_factor == Fixed64::from_raw(512)
                    && *slow_duration == Fixed64::from_i32(5)
            )),
            "missing authored Arty level-4 control projectile: {outcomes:?}"
        );
    }
}
