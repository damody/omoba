//! Bomb Shooter — AoE 塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: bomb_stun (0.5s stun on splash), missile (彈速 ×1.5),
//!   frag_8 / frag_12 / frag_homing (命中後 8/12/16 碎片)
//! - Stat: splash_bonus, damage_bonus, range_bonus (透過 get_final_*)
//!
//! - moab_assassin: 超級導彈使用 tower internal cooldown，每 15s 最多一次。

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct BombTower;

// 數值唯一來源：scripts/lua_data/templates.lua → omoba_template_ids 編譯期生成
// `TOWER_BOMB_STATS`。runtime Lua content mode 會以 active lookup 覆蓋此 fallback。
const STATS: &TowerStats = &TOWER_BOMB_STATS;

impl UnitScript for BombTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_BOMB.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_BOMB, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_BOMB,
            STATS,
            &TOWER_BOMB_RENDER,
            TOWER_BOMB_ATTACK_TIMING,
        ))
    }

    fn on_tower_tick(
        &self,
        e: EntityHandle,
        dt: Fixed64,
        cooldowns: &mut TowerCooldownAccessDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed64::ZERO {
            return;
        }
        let stats = super::tower_stats(TOWER_BOMB, STATS);
        let timing = super::tower_attack_timing(TOWER_BOMB, TOWER_BOMB_ATTACK_TIMING);
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

        let atk = w.get_final_atk(e);

        // flash_bonus: 基礎 STATS.splash_radius + sum_add("splash_bonus")
        let splash_bonus = w.get_stat_bonus(e, StatKey::SplashBonus);
        let splash = stats.splash_radius + splash_bonus;

        let stun = if w.has_tower_flag(e, RStr::from_str("bomb_stun")) {
            Fixed64::from_raw(512) // 0.5
        } else {
            Fixed64::ZERO
        };
        let missile_tier2 = w.has_tower_flag(e, RStr::from_str("missile_speed_tier2"));
        let missile = missile_tier2 || w.has_tower_flag(e, RStr::from_str("missile"));
        let bullet_speed = if missile_tier2 {
            stats.bullet_speed * Fixed64::from_raw(2304) // 2.25 = 1.5 × 1.5
        } else if missile {
            stats.bullet_speed * Fixed64::from_raw(1536) // 1.5
        } else {
            stats.bullet_speed
        };

        w.log_info(RStr::from_str("[tower_bomb] fire!"));
        if let RSome(t_pos) = w.get_pos(target) {
            w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
        }
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: PathSpec::Homing { target },
            speed: bullet_speed,
            damage: atk,
            hit_radius: Fixed64::ZERO,
            splash_radius: splash,
            slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO,
            stun_duration: stun,
            kind_id: PROJECTILE_BOMB.0,
        });

        // moab_assassin：每 15s 額外發射高傷害超級導彈（atk × 10）
        if w.has_tower_flag(e, RStr::from_str("moab_assassin"))
            && cooldowns.get_tower_internal_cooldown(e) <= Fixed64::ZERO
        {
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: PathSpec::Homing { target },
                speed: Fixed64::from_i32(2000),
                damage: atk * Fixed64::from_i32(10),
                hit_radius: Fixed64::from_i32(60),
                splash_radius: splash,
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                stun_duration: stun,
                kind_id: PROJECTILE_BOMB.0,
            });
            cooldowns.start_tower_internal_cooldown(e, Fixed64::from_i32(15));
        }
    }

    fn on_projectile_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        context: ProjectileHitContext,
        query: &omb_script_abi::world::ProjectileQueryDyn<'_>,
        w: &mut GameWorldDyn<'_>,
    ) {
        let recursive = w.has_tower_flag(attacker, RStr::from_str("frag_recursive"));
        let frag_count: u32 = if context.kind_id == PROJECTILE_BOMB.0 && context.generation == 0 {
            let count = if w.has_tower_flag(attacker, RStr::from_str("frag_homing")) {
                16
            } else if w.has_tower_flag(attacker, RStr::from_str("frag_12")) {
                12
            } else if w.has_tower_flag(attacker, RStr::from_str("frag_8")) {
                8
            } else {
                return;
            };
            count
        } else if context.kind_id == PROJECTILE_BOMB_FRAG.0 && context.generation == 1 && recursive
        {
            4
        } else {
            return;
        };

        let frag_damage = if recursive {
            Fixed64::from_i32(45)
        } else if w.has_tower_flag(attacker, RStr::from_str("frag_12")) {
            Fixed64::from_i32(25)
        } else {
            Fixed64::from_i32(15)
        };

        let pos = match w.get_pos(victim) {
            RSome(p) => p,
            RNone => return,
        };
        let frag_range = Fixed64::from_i32(300);
        let frag_speed = Fixed64::from_i32(800);
        let frag_hit_radius = Fixed64::from_i32(40);
        let homing = w.has_tower_flag(attacker, RStr::from_str("frag_homing"));
        let mut homing_targets: Vec<EntityHandle> = if homing {
            query
                .enemy_candidates_bounded(
                    pos,
                    frag_range,
                    attacker,
                    RSome(victim),
                    frag_count as u16,
                )
                .iter()
                .copied()
                .collect()
        } else {
            Vec::new()
        };
        homing_targets.sort_by_key(|target| (target.id, target.gen));
        if homing && homing_targets.is_empty() {
            // A recursive homing fragment can legitimately be the only enemy left.
            // Reuse its still-valid impact target so the authored four-child wave exists.
            homing_targets.push(victim);
        }

        // 360° / frag_count，用 from_degrees_i32 維持決定性 LUT
        let step_deg: i32 = 360 / (frag_count as i32);

        for i in 0..frag_count {
            let angle = omoba_sim::trig::Angle::from_degrees_i32(step_deg * (i as i32));
            let end = Vec2 {
                x: pos.x + omoba_sim::trig::cos(angle) * frag_range,
                y: pos.y + omoba_sim::trig::sin(angle) * frag_range,
            };
            let path = if homing {
                PathSpec::Homing {
                    target: homing_targets[i as usize % homing_targets.len()],
                }
            } else {
                PathSpec::Straight { end_pos: end }
            };
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: attacker,
                path,
                speed: frag_speed,
                damage: frag_damage,
                hit_radius: frag_hit_radius,
                splash_radius: Fixed64::ZERO,
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: PROJECTILE_BOMB_FRAG.0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke, invoke_tick};
    use omoba_core::Outcome;
    use specs::WorldExt;

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
    fn frag_children_do_not_spawn_another_ordinary_frag_wave() {
        let fixture = fixture(
            &["frag_8", "frag_recursive"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        let tower = fixture.tower;
        let victim = fixture.enemies[0];

        let primary = invoke(
            &fixture.world,
            &BombTower,
            tower,
            victim,
            ProjectileHitContext {
                kind_id: PROJECTILE_BOMB.0,
                generation: 0,
            },
        );
        assert_eq!(projectile_generations(&primary), vec![1; 8]);

        let frag = invoke(
            &fixture.world,
            &BombTower,
            tower,
            victim,
            ProjectileHitContext {
                kind_id: PROJECTILE_BOMB_FRAG.0,
                generation: 1,
            },
        );
        assert_eq!(projectile_generations(&frag), vec![2; 4]);

        let recursive_child = invoke(
            &fixture.world,
            &BombTower,
            tower,
            victim,
            ProjectileHitContext {
                kind_id: PROJECTILE_BOMB_FRAG.0,
                generation: 2,
            },
        );
        assert!(projectile_generations(&recursive_child).is_empty());
    }

    #[test]
    fn homing_fragments_choose_targets_in_entity_order() {
        let fixture = fixture(
            &["frag_homing"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(30), Fixed64::ZERO),
            ],
        );
        let outcomes = invoke(
            &fixture.world,
            &BombTower,
            fixture.tower,
            fixture.enemies[0],
            ProjectileHitContext {
                kind_id: PROJECTILE_BOMB.0,
                generation: 0,
            },
        );
        let targets: Vec<_> = outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                Outcome::ScriptProjectile { target, .. } => *target,
                _ => None,
            })
            .collect();
        assert_eq!(targets.len(), 16);
        assert_eq!(targets[0], fixture.enemies[1]);
        assert_eq!(targets[1], fixture.enemies[2]);
        assert_eq!(targets[2], fixture.enemies[1]);
    }

    #[test]
    fn recursive_homing_frag_uses_single_target_fallback_for_four_children() {
        let fixture = fixture(
            &["frag_homing", "frag_recursive"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        let victim = fixture.enemies[0];
        let outcomes = invoke(
            &fixture.world,
            &BombTower,
            fixture.tower,
            victim,
            ProjectileHitContext {
                kind_id: PROJECTILE_BOMB_FRAG.0,
                generation: 1,
            },
        );
        let children: Vec<_> = outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                Outcome::ScriptProjectile {
                    target, generation, ..
                } => Some((*target, *generation)),
                _ => None,
            })
            .collect();
        assert_eq!(children, vec![(Some(victim), 2); 4]);
    }

    #[test]
    fn moab_assassin_respects_authored_fifteen_second_cooldown() {
        let mut ready = fixture(
            &["moab_assassin"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        ready
            .world
            .write_storage::<omoba_core::TAttack>()
            .get_mut(ready.tower)
            .unwrap()
            .asd_count = -Fixed64::from_raw(1);
        let ready_outcomes =
            invoke_tick(&ready.world, &BombTower, ready.tower, Fixed64::from_raw(1));
        assert!(ready_outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::ScriptSetTowerInternalCooldown { duration, .. }
                if *duration == Fixed64::from_i32(15)
        )));

        let mut fixture = fixture(
            &["moab_assassin"],
            &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)],
        );
        fixture
            .world
            .write_storage::<omoba_core::TAttack>()
            .get_mut(fixture.tower)
            .unwrap()
            .asd_count = -Fixed64::from_raw(1);
        fixture
            .world
            .write_storage::<omoba_core::Tower>()
            .get_mut(fixture.tower)
            .unwrap()
            .ultimate_cooldown = Fixed64::from_i32(5);

        let outcomes = invoke_tick(
            &fixture.world,
            &BombTower,
            fixture.tower,
            Fixed64::from_raw(1),
        );
        let projectiles = outcomes
            .iter()
            .filter(|outcome| matches!(outcome, Outcome::ScriptProjectile { .. }))
            .count();
        assert_eq!(
            projectiles, 1,
            "normal shot must remain, but assassin missile must wait"
        );
    }

    #[test]
    fn bomb_missile_speed_matches_both_authored_tiers() {
        fn fired_speed(flags: &[&str]) -> Fixed64 {
            let mut fixture = fixture(flags, &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)]);
            fixture
                .world
                .write_storage::<omoba_core::TAttack>()
                .get_mut(fixture.tower)
                .unwrap()
                .asd_count = -Fixed64::from_raw(1);
            invoke_tick(
                &fixture.world,
                &BombTower,
                fixture.tower,
                Fixed64::from_raw(1),
            )
            .into_iter()
            .find_map(|outcome| match outcome {
                Outcome::ScriptProjectile { msd, .. } => Some(msd),
                _ => None,
            })
            .unwrap()
        }

        assert_eq!(fired_speed(&["missile"]), Fixed64::from_i32(1350));
        assert_eq!(
            fired_speed(&["missile", "missile_speed_tier2"]),
            Fixed64::from_i32(2025)
        );
    }
}
