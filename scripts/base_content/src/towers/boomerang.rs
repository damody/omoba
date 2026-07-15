//! Boomerang Monkey — 回力鏢穿透塔，BTD6 同名塔的復刻。
//!
//! 機制：
//! - 基礎：投擲有寬度（hit_radius 60）的回力鏢，可同時打到前排多個敵人
//! - Path1: glaive_ricochet（命中後彈向下一個敵人）、glaive_lord（2 顆）、moab_press（減速 + 雙傷）
//! - Path2: faster_rangs（彈速 ×1.5）、bionic_burst（一次 3 顆連射）、turbo_charge（攻速倍增）
//! - Path3: shuriken（升為手裡劍：hit_radius 90 + 傷害提升）、double_shuriken（2 顆）、
//!          storm_shuriken（3 顆 + 彈射）
//!
//! 注意：由於 PathSpec 目前只有 Homing / Straight，回力鏢以
//! homing + hit_radius 模擬穿透效果，不實現拋物線弧度。

use omb_script_abi::prelude::*;

pub struct BoomerangTower;

const STATS: &TowerStats = &TOWER_BOOMERANG_STATS;

// 彈射時的搜尋半徑（從命中點往外找下一個目標）250 * 1024 = 256000
const RICOCHET_RADIUS: Fixed64 = Fixed64::from_raw(256000);
// moab_press 減速 0.5 * 1024 = 512
const MOAB_PRESS_SLOW_FACTOR: Fixed64 = Fixed64::from_raw(512);
const MOAB_PRESS_SLOW_DUR: Fixed64 = Fixed64::from_raw(512);
const TURBO_INTERVAL_MULTIPLIER: Fixed64 = Fixed64::from_raw(358);
const TURBO_ACTIVE_BUFF: &str = "boomerang_turbo_charge_active";
const TURBO_ABILITY_ID: &str = "boomerang_turbo_charge";
const SHURIKEN_STORM_ABILITY_ID: &str = "boomerang_shuriken_storm";
const SHURIKEN_STORM_PULSES: u16 = 3;
const SHURIKEN_STORM_PROJECTILES: u32 = 12;
const SHURIKEN_STORM_PHASE_DEGREES: i32 = 10;

fn turbo_interval(base_final_interval: Fixed64) -> Fixed64 {
    base_final_interval * TURBO_INTERVAL_MULTIPLIER
}

fn boomerang_projectile_count(permanent_count: u32, turbo_active: bool) -> u32 {
    permanent_count + if turbo_active { 2 } else { 0 }
}

impl UnitScript for BoomerangTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_BOOMERANG.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_BOOMERANG, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_BOOMERANG,
            STATS,
            &TOWER_BOOMERANG_RENDER,
            TOWER_BOOMERANG_ATTACK_TIMING,
        ))
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let turbo_active =
            w.get_buff_remaining(e, RStr::from_str(TURBO_ACTIVE_BUFF)) > Fixed64::ZERO;
        let base_final_interval = w.get_asd_interval(e);
        let asd_interval = if turbo_active {
            turbo_interval(base_final_interval)
        } else {
            base_final_interval
        };
        if asd_interval <= Fixed64::ZERO {
            return;
        }
        let stats = super::tower_stats(TOWER_BOOMERANG, STATS);
        let timing = super::tower_attack_timing(TOWER_BOOMERANG, TOWER_BOOMERANG_ATTACK_TIMING);
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

        // ── Impact：決定發射參數 ──────────────────────────────────────
        let atk = w.get_final_atk(e);

        let shuriken = w.has_tower_flag(e, RStr::from_str("shuriken"));
        let storm = w.has_tower_flag(e, RStr::from_str("storm_shuriken"));

        // hit_radius：手裡劍系列更寬
        let hit_radius = if shuriken || storm {
            Fixed64::from_i32(90)
        } else {
            stats.hit_radius // 60
        };

        // 彈速：faster_rangs 升級讓彈速 ×1.5
        let bullet_speed = if w.has_tower_flag(e, RStr::from_str("faster_rangs")) {
            stats.bullet_speed * Fixed64::from_raw(1536) // ×1.5
        } else {
            stats.bullet_speed
        };
        // moab_press：附加減速
        let (slow_factor, slow_duration) = if w.has_tower_flag(e, RStr::from_str("moab_press")) {
            (MOAB_PRESS_SLOW_FACTOR, MOAB_PRESS_SLOW_DUR)
        } else {
            (Fixed64::ZERO, Fixed64::ZERO)
        };

        // 決定發射數量：
        //   storm_shuriken → 3（手裡劍系）
        //   bionic_burst   → 3（仿生系）
        //   double_shuriken / glaive_lord → 2
        //   其他 → 1
        let permanent_count: u32 = if storm || w.has_tower_flag(e, RStr::from_str("bionic_burst")) {
            3
        } else if w.has_tower_flag(e, RStr::from_str("double_shuriken"))
            || w.has_tower_flag(e, RStr::from_str("glaive_lord"))
        {
            2
        } else {
            1
        };
        let count = boomerang_projectile_count(permanent_count, turbo_active);

        let kind_id = if shuriken || storm {
            PROJECTILE_SHURIKEN.0
        } else {
            PROJECTILE_BOOMERANG.0
        };

        if let RSome(t_pos) = w.get_pos(target) {
            w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
        }

        w.log_info(RStr::from_str("[tower_boomerang] fire!"));

        // 多顆時向目標方向稍微扇開（±10°），單顆直接 homing
        let t_pos_opt = w.get_pos(target);
        let base_angle = if let RSome(t_pos) = t_pos_opt {
            omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x)
        } else {
            w.get_facing(e)
        };

        for i in 0..count {
            let path_spec = if count == 1 {
                PathSpec::Homing { target }
            } else {
                // 扇形展開：中央 0°，±10° for 2 顆；-10°/0°/+10° for 3 顆
                let offset_deg: i32 = if count == 2 {
                    (i as i32) * 20 - 10 // -10, +10
                } else if count == 3 {
                    (i as i32) * 10 - 10 // -10, 0, +10
                } else {
                    (2 * i as i32 - (count as i32 - 1)) * 5
                };
                let offset_ticks = omoba_sim::trig::Angle::from_degrees_i32(offset_deg).ticks();
                let angle = omoba_sim::trig::Angle::from_ticks(base_angle.ticks() + offset_ticks);
                let end = Vec2 {
                    x: pos.x + omoba_sim::trig::cos(angle) * range,
                    y: pos.y + omoba_sim::trig::sin(angle) * range,
                };
                PathSpec::Straight { end_pos: end }
            };

            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: path_spec,
                speed: bullet_speed,
                damage: atk,
                hit_radius,
                splash_radius: Fixed64::ZERO,
                slow_factor,
                slow_duration,
                stun_duration: Fixed64::ZERO,
                kind_id,
            });
        }
    }

    fn on_tower_ability_activate_with_access(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        access: &TowerActiveAbilityAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        if ability_id.as_str() != TURBO_ABILITY_ID {
            return;
        }
        let remaining = access.get_tower_ability_active_remaining(tower, ability_id);
        if remaining <= Fixed64::ZERO {
            return;
        }
        w.add_buff(tower, RStr::from_str(TURBO_ACTIVE_BUFF), remaining);
        access.reset_attack_backswing(tower);
    }

    fn on_tower_ability_pulse_with_access(
        &self,
        tower: EntityHandle,
        ability_id: RStr<'_>,
        pulse_index: u16,
        _access: &TowerActiveAbilityAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) -> bool {
        if ability_id.as_str() != SHURIKEN_STORM_ABILITY_ID || pulse_index >= SHURIKEN_STORM_PULSES
        {
            return false;
        }
        let pos = match w.get_pos(tower) {
            RSome(pos) => pos,
            RNone => return false,
        };
        let range = w.get_final_attack_range(tower);
        let speed = if w.has_tower_flag(tower, RStr::from_str("faster_rangs")) {
            STATS.bullet_speed * Fixed64::from_raw(1536)
        } else {
            STATS.bullet_speed
        };
        let damage = w.get_final_atk(tower);
        for i in 0..SHURIKEN_STORM_PROJECTILES {
            let degrees = i as i32 * 30 + pulse_index as i32 * SHURIKEN_STORM_PHASE_DEGREES;
            let angle = omoba_sim::trig::Angle::from_degrees_i32(degrees);
            let end = Vec2 {
                x: pos.x + omoba_sim::trig::cos(angle) * range,
                y: pos.y + omoba_sim::trig::sin(angle) * range,
            };
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: tower,
                path: PathSpec::Straight { end_pos: end },
                speed,
                damage,
                hit_radius: Fixed64::from_i32(90),
                splash_radius: Fixed64::ZERO,
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: PROJECTILE_SHURIKEN.0,
            });
        }
        true
    }

    fn on_projectile_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        context: ProjectileHitContext,
        query: &omb_script_abi::world::ProjectileQueryDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        // glaive_ricochet / storm_shuriken：命中後從受擊位置彈向下一個目標
        let has_ricochet = w.has_tower_flag(attacker, RStr::from_str("glaive_ricochet"));
        let has_storm = w.has_tower_flag(attacker, RStr::from_str("storm_shuriken"));
        if !has_ricochet && !has_storm {
            return;
        }
        if context.kind_id != PROJECTILE_BOOMERANG.0 && context.kind_id != PROJECTILE_SHURIKEN.0 {
            return;
        }
        let max_generation = if has_storm { 2 } else { 1 };
        if context.generation >= max_generation {
            return;
        }

        let victim_pos = match w.get_pos(victim) {
            RSome(p) => p,
            RNone => return,
        };

        // 在受擊點附近找下一個敵人（排除原目標自身）
        let mut nearby: Vec<EntityHandle> = query
            .enemy_candidates_bounded(victim_pos, RICOCHET_RADIUS, attacker, RSome(victim), 1)
            .iter()
            .copied()
            .collect();
        nearby.sort_by_key(|target| (target.id, target.gen));
        let bounce_target = nearby.first().copied();
        let bounce_target = match bounce_target {
            Some(t) => t,
            None => return,
        };

        let atk = w.get_final_atk(attacker);
        let shuriken = w.has_tower_flag(attacker, RStr::from_str("shuriken")) || has_storm;
        let hit_radius = if shuriken {
            Fixed64::from_i32(90)
        } else {
            Fixed64::from_i32(60)
        };
        let kind_id = if shuriken {
            PROJECTILE_SHURIKEN.0
        } else {
            PROJECTILE_BOOMERANG.0
        };

        w.spawn_projectile_ex(ProjectileSpec {
            from: victim_pos,
            owner: attacker,
            path: PathSpec::Homing {
                target: bounce_target,
            },
            speed: Fixed64::from_i32(1500),
            damage: atk,
            hit_radius,
            splash_radius: Fixed64::ZERO,
            slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO,
            stun_duration: Fixed64::ZERO,
            kind_id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke, invoke_pulse};
    use abi_stable::{sabi_trait::prelude::TD_Opaque, RMut, RRef};
    use omb_script_abi::world::{GameWorld_TO, TowerActiveAbilityAccess_TO};
    use omoba_core::scripting::parallel_world_adapter::{
        ParallelAdapterCache, ParallelTowerActiveAbilityAccess, ParallelWorldAdapter,
    };
    use omoba_core::Outcome;
    use specs::WorldExt;

    fn invoke_activation(fixture: &specs::World, tower: specs::Entity) -> Vec<Outcome> {
        let cache = ParallelAdapterCache::new(fixture, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, tower);
        let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
        let mut world_dyn = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        let access_dyn =
            TowerActiveAbilityAccess_TO::from_ptr(RRef::new(&access_adapter), TD_Opaque);
        BoomerangTower.on_tower_ability_activate_with_access(
            EntityHandle {
                id: tower.id(),
                gen: tower.gen().id() as u32,
            },
            RStr::from_str("boomerang_turbo_charge"),
            &access_dyn,
            &mut world_dyn,
        );
        drop(access_dyn);
        drop(world_dyn);
        let mut outcomes = adapter.finish();
        outcomes.extend(access_adapter.finish());
        outcomes
    }

    #[test]
    fn turbo_adds_two_projectiles_and_multiplies_interval() {
        assert_eq!(boomerang_projectile_count(1, true), 3);
        assert_eq!(turbo_interval(Fixed64::from_i32(1)), Fixed64::from_raw(358));
    }

    #[test]
    fn turbo_activation_marks_five_second_window_and_resets_backswing() {
        let fixture = fixture(&["turbo_charge"], &[]);
        fixture
            .world
            .write_storage::<omoba_core::Tower>()
            .get_mut(fixture.tower)
            .unwrap()
            .active_ability = Some(omoba_core::TowerActiveAbilityState {
            ability_id: "boomerang_turbo_charge".to_string(),
            active_remaining: Fixed64::from_i32(5),
            activation_serial: 1,
            ..Default::default()
        });

        let outcomes = invoke_activation(&fixture.world, fixture.tower);

        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { buff_id, duration, .. }
                if buff_id == "boomerang_turbo_charge_active" && *duration == Fixed64::from_i32(5)
        )));
        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::ScriptSetAsdCount { asd_count, .. } if *asd_count == Fixed64::ONE
        )));
    }

    #[test]
    fn turbo_window_adds_two_projectiles_after_permanent_count_upgrades() {
        let fixture = fixture(
            &["bionic_burst", "turbo_charge"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        fixture
            .world
            .write_resource::<omoba_core::runtime::BuffStore>()
            .add(
                fixture.tower,
                "boomerang_turbo_charge_active",
                Fixed64::from_i32(5),
                serde_json::Value::Null,
            );
        fixture
            .world
            .write_storage::<omoba_core::TAttack>()
            .get_mut(fixture.tower)
            .unwrap()
            .asd_count = -Fixed64::from_raw(1);

        let outcomes = crate::towers::projectile_test_support::invoke_tick(
            &fixture.world,
            &BoomerangTower,
            fixture.tower,
            Fixed64::from_raw(1),
        );

        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, Outcome::ScriptProjectile { .. }))
                .count(),
            5
        );
    }

    fn projectile_generations(outcomes: &[Outcome]) -> Vec<u8> {
        outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                Outcome::ScriptProjectile { generation, .. } => Some(*generation),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn two_target_ricochet_stops_after_one_bounce() {
        let fixture = fixture(
            &["glaive_ricochet"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
            ],
        );
        let tower = fixture.tower;
        let first = fixture.enemies[0];
        let second = fixture.enemies[1];

        let primary = invoke(
            &fixture.world,
            &BoomerangTower,
            tower,
            first,
            ProjectileHitContext {
                kind_id: PROJECTILE_BOOMERANG.0,
                generation: 0,
            },
        );
        assert_eq!(projectile_generations(&primary), vec![1]);

        let bounce = invoke(
            &fixture.world,
            &BoomerangTower,
            tower,
            second,
            ProjectileHitContext {
                kind_id: PROJECTILE_BOOMERANG.0,
                generation: 1,
            },
        );
        assert!(projectile_generations(&bounce).is_empty());
    }

    #[test]
    fn storm_ricochet_stops_after_two_bounces() {
        let fixture = fixture(
            &["storm_shuriken"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
            ],
        );
        let tower = fixture.tower;
        let first = fixture.enemies[0];
        let second = fixture.enemies[1];
        for (generation, victim, expected) in [
            (0, first, vec![1]),
            (1, second, vec![2]),
            (2, first, Vec::new()),
        ] {
            let outcomes = invoke(
                &fixture.world,
                &BoomerangTower,
                tower,
                victim,
                ProjectileHitContext {
                    kind_id: PROJECTILE_SHURIKEN.0,
                    generation,
                },
            );
            assert_eq!(projectile_generations(&outcomes), expected);
        }
    }

    #[test]
    fn shuriken_storm_emits_three_rotated_rings_of_twelve_shuriken() {
        let fixture = fixture(&["storm_shuriken"], &[]);
        let final_range = Fixed64::from_i32(623);
        let final_damage = Fixed64::from_i32(17);
        fixture
            .world
            .write_resource::<omoba_core::runtime::BuffStore>()
            .add(
                fixture.tower,
                "shuriken_storm_final_stats_test",
                Fixed64::from_i32(5),
                serde_json::json!({
                    "attack_range_bonus": Fixed64::from_i32(123).raw(),
                    "preattack_bonus_damage": Fixed64::from_i32(7).raw(),
                }),
            );

        for pulse_index in 0..3 {
            let (consumed, outcomes) = invoke_pulse(
                &fixture.world,
                &BoomerangTower,
                fixture.tower,
                "boomerang_shuriken_storm",
                pulse_index,
            );
            assert!(consumed);
            let shots: Vec<_> = outcomes
                .iter()
                .filter_map(|outcome| match outcome {
                    Outcome::ScriptProjectile {
                        tpos,
                        msd,
                        damage_phys,
                        hit_radius,
                        kind_id,
                        generation,
                        ..
                    } if *kind_id == PROJECTILE_SHURIKEN.0 => {
                        assert_eq!(*msd, Fixed64::from_i32(1500));
                        assert_eq!(*damage_phys, final_damage);
                        assert_eq!(*hit_radius, Fixed64::from_i32(90));
                        assert_eq!(*generation, 0);
                        Some(*tpos)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(shots.len(), 12);
            for (i, endpoint) in shots.iter().enumerate() {
                let degrees = i as i32 * 30 + pulse_index as i32 * 10;
                let angle = omoba_sim::trig::Angle::from_degrees_i32(degrees);
                let expected = Vec2 {
                    x: omoba_sim::trig::cos(angle) * final_range,
                    y: omoba_sim::trig::sin(angle) * final_range,
                };
                assert_eq!(
                    *endpoint, expected,
                    "pulse {pulse_index}, projectile {i}, angle {degrees} degrees"
                );
            }
        }
    }

    #[test]
    fn shuriken_storm_rejects_unknown_id_and_out_of_range_pulse() {
        let fixture = fixture(&["storm_shuriken"], &[]);
        for (ability_id, pulse_index) in [("wrong", 0), ("boomerang_shuriken_storm", 3)] {
            let (consumed, outcomes) = invoke_pulse(
                &fixture.world,
                &BoomerangTower,
                fixture.tower,
                ability_id,
                pulse_index,
            );
            assert!(!consumed);
            assert!(outcomes.is_empty());
        }
    }

    #[test]
    fn shuriken_storm_inherits_faster_rangs_and_existing_ricochet_bound() {
        let fixture = fixture(
            &["storm_shuriken", "faster_rangs"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
            ],
        );
        let (_, outcomes) = invoke_pulse(
            &fixture.world,
            &BoomerangTower,
            fixture.tower,
            "boomerang_shuriken_storm",
            0,
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome,
                    Outcome::ScriptProjectile { msd, kind_id, .. }
                        if *kind_id == PROJECTILE_SHURIKEN.0
                            && *msd == Fixed64::from_i32(2250)
                ))
                .count(),
            12
        );

        let generation_two = invoke(
            &fixture.world,
            &BoomerangTower,
            fixture.tower,
            fixture.enemies[0],
            ProjectileHitContext {
                kind_id: PROJECTILE_SHURIKEN.0,
                generation: 2,
            },
        );
        assert!(generation_two.is_empty());
    }
}
