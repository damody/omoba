//! Ice Monkey — 減速塔，支援全部 9 個升級 flag。
//!
//! 支援:
//! - Path1: deep_freeze (1s stun), absolute_zero (全局凍結), icicle_impale (直線穿透 + 150 splash + 25 dmg)
//! - Path2: arctic_aura_20 (光環 20% 減速), snowstorm (光環 50% 減速), cryo_cannon (AoE 冰凍砲)
//! - Path3: embrittle_15 (目標受傷 +15%), embrittle_25 (目標受傷 +25%), refreeze (命中後重置凍結)
//! - Stat: slow_factor_override (越小越強), slow_duration_bonus, splash_bonus,
//!   damage_bonus, range_bonus (透過 get_final_*)
//!
//! 注意：
//! - absolute_zero：每 15s 凍結全場 2s
//! - cryo_cannon：每 10s 以 deal_damage_splash + query_enemies_in_range 發射 AoE 冰凍砲

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct IceTower;

// 數值唯一來源：scripts/lua_data/templates.lua → omoba_template_ids 編譯期生成
// `TOWER_ICE_STATS`。
const STATS: &TowerStats = &TOWER_ICE_STATS;

// 光環 buff 持續時間：略長於單 tick，確保每幀刷新不斷開（~0.2s = 205/1024）
const AURA_BUFF_DUR: Fixed64 = Fixed64::from_raw(205);
// arctic_aura_20: 移速降低 20%
const AURA_20_JSON: &str = r#"{"movespeed_bonus_percentage":-0.2}"#;
// snowstorm: 移速降低 35%
const AURA_35_JSON: &str = r#"{"movespeed_bonus_percentage":-0.35}"#;
// cryo_cannon: 移速降低 40%
const AURA_40_JSON: &str = r#"{"movespeed_bonus_percentage":-0.4}"#;
// cryo_cannon: AoE 凍結半徑（200 units = 204800/1024）
const CRYO_CANNON_RADIUS: Fixed64 = Fixed64::from_raw(204800);
// cryo_cannon: 凍結持續時間（1s）
const CRYO_CANNON_FREEZE_DUR: Fixed64 = Fixed64::ONE;
// embrittle: debuff 持續時間（3s = 3072/1024）
const EMBRITTLE_DUR: Fixed64 = Fixed64::from_raw(3072);
// embrittle_15: 目標受傷增幅 +15%
const EMBRITTLE_15_JSON: &str = r#"{"incoming_damage_percentage":0.15}"#;
// embrittle_25: 目標受傷增幅 +25%
const EMBRITTLE_25_JSON: &str = r#"{"incoming_damage_percentage":0.25}"#;
// absolute_zero: 超大半徑模擬全場（2000 units = 2048000/1024）
const GLOBAL_RADIUS: Fixed64 = Fixed64::from_raw(2048000);
// refreeze: 重置凍結 duration（1s）
const REFREEZE_DUR: Fixed64 = Fixed64::ONE;

impl UnitScript for IceTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_ICE.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_ICE, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_ICE,
            STATS,
            &TOWER_ICE_RENDER,
            TOWER_ICE_ATTACK_TIMING,
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

        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let range = w.get_final_attack_range(e);

        // ── Path2 光環：arctic_aura_20 / snowstorm（每 tick 刷新，不受攻速影響）──
        let has_cryo = w.has_tower_flag(e, RStr::from_str("cryo_cannon"));
        let has_snowstorm = w.has_tower_flag(e, RStr::from_str("snowstorm"));
        let has_aura_20 = w.has_tower_flag(e, RStr::from_str("arctic_aura_20"));
        if has_aura_20 || has_snowstorm || has_cryo {
            let json = if has_cryo {
                AURA_40_JSON
            } else if has_snowstorm {
                AURA_35_JSON
            } else {
                AURA_20_JSON
            };
            let aura_targets = w.query_enemies_in_range(pos, range, e);
            for victim in aura_targets.iter().copied() {
                w.add_stat_buff(
                    victim,
                    RStr::from_str("ice_aura_slow"),
                    AURA_BUFF_DUR,
                    RStr::from_str(json),
                );
            }
        }

        let atk = w.get_final_atk(e);
        let absolute_zero = w.has_tower_flag(e, RStr::from_str("absolute_zero"));
        if cooldowns.get_tower_internal_cooldown(e) <= Fixed64::ZERO {
            if absolute_zero {
                let all_enemies = w.query_enemies_in_range(pos, GLOBAL_RADIUS, e);
                if !all_enemies.is_empty() {
                    for victim in all_enemies.iter().copied() {
                        w.add_buff(victim, RStr::from_str("stun"), Fixed64::from_i32(2));
                    }
                    cooldowns.start_tower_internal_cooldown(e, Fixed64::from_i32(15));
                }
            } else if has_cryo {
                if let RSome(cryo_target) = w.query_nearest_enemy(pos, range, e) {
                    if let RSome(t_pos) = w.get_pos(cryo_target) {
                        w.deal_damage_splash(
                            t_pos,
                            CRYO_CANNON_RADIUS,
                            atk,
                            DamageKind::Magical,
                            RSome(e),
                        );
                        let cryo_targets = w.query_enemies_in_range(t_pos, CRYO_CANNON_RADIUS, e);
                        for victim in cryo_targets.iter().copied() {
                            w.add_buff(victim, RStr::from_str("stun"), CRYO_CANNON_FREEZE_DUR);
                        }
                        cooldowns.start_tower_internal_cooldown(e, Fixed64::from_i32(10));
                    }
                }
            }
        }

        let stats = super::tower_stats(TOWER_ICE, STATS);
        let timing = super::tower_attack_timing(TOWER_ICE, TOWER_ICE_ATTACK_TIMING);
        let phase = super::advance_attack_phase(e, dt, asd_interval, timing, w);
        if matches!(phase, super::AttackPhaseStep::Charging) {
            return;
        }

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

        if let RSome(t_pos) = w.get_pos(target) {
            w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
        }

        // slow_factor_override：upgrade 寫入的目標 factor（越小越強，clamp 在 (0, 1) 才採用）
        let slow_override = w.get_stat_bonus(e, StatKey::SlowFactorOverride);
        let slow_factor = if slow_override > Fixed64::ZERO && slow_override < Fixed64::ONE {
            slow_override
        } else {
            stats.slow_factor
        };

        let slow_dur_bonus = w.get_stat_bonus(e, StatKey::SlowDurationBonus);
        let slow_duration = stats.slow_duration + slow_dur_bonus;

        let splash_bonus = w.get_stat_bonus(e, StatKey::SplashBonus);
        let splash_radius = stats.splash_radius + splash_bonus;

        let stun = if w.has_tower_flag(e, RStr::from_str("deep_freeze")) {
            Fixed64::ONE
        } else {
            Fixed64::ZERO
        };
        let icicle = w.has_tower_flag(e, RStr::from_str("icicle_impale"));
        let (path_spec, final_splash, final_damage, kind_id) = if icicle {
            // 朝 target 直線穿透（至 1.5 倍 range）
            let t_pos = match w.get_pos(target) {
                RSome(p) => p,
                RNone => return,
            };
            let dx = t_pos.x - pos.x;
            let dy = t_pos.y - pos.y;
            let len_raw = (dx * dx + dy * dy).sqrt();
            let len = if len_raw < Fixed64::ONE {
                Fixed64::ONE
            } else {
                len_raw
            };
            let scale = Fixed64::from_raw(1536); // 1.5
            let nx = dx / len * range * scale;
            let ny = dy / len * range * scale;
            let end = Vec2 {
                x: pos.x + nx,
                y: pos.y + ny,
            };
            let twenty_five = Fixed64::from_i32(25);
            let dmg = if atk > twenty_five { atk } else { twenty_five };
            (
                PathSpec::Straight { end_pos: end },
                Fixed64::from_i32(150),
                dmg,
                PROJECTILE_ICICLE.0,
            )
        } else {
            (
                PathSpec::Homing { target },
                splash_radius,
                atk,
                PROJECTILE_ICE.0,
            )
        };

        w.log_info(RStr::from_str("[tower_ice] fire!"));
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: path_spec,
            speed: stats.bullet_speed,
            damage: final_damage,
            hit_radius: Fixed64::ZERO,
            splash_radius: final_splash,
            slow_factor,
            slow_duration,
            stun_duration: stun,
            kind_id,
        });
    }

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        // ── Path3：embrittle — 命中時對目標施加受傷增幅 debuff ─────────
        // embrittle_25 優先（升級覆蓋 embrittle_15）
        let has_embrittle_25 = w.has_tower_flag(attacker, RStr::from_str("embrittle_25"));
        let has_embrittle_15 = w.has_tower_flag(attacker, RStr::from_str("embrittle_15"));
        if has_embrittle_25 {
            w.add_stat_buff(
                victim,
                RStr::from_str("ice_embrittle"),
                EMBRITTLE_DUR,
                RStr::from_str(EMBRITTLE_25_JSON),
            );
        } else if has_embrittle_15 {
            w.add_stat_buff(
                victim,
                RStr::from_str("ice_embrittle"),
                EMBRITTLE_DUR,
                RStr::from_str(EMBRITTLE_15_JSON),
            );
        }

        // ── Path3：refreeze — 命中時重置目標的凍結（stun）buff ──────────
        if w.has_tower_flag(attacker, RStr::from_str("refreeze")) {
            w.remove_buff(victim, RStr::from_str("stun"));
            w.add_buff(victim, RStr::from_str("stun"), REFREEZE_DUR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke_tick};
    use omoba_core::Outcome;
    use specs::WorldExt;

    fn impact_outcomes(flags: &[&str], cooldown: Fixed64) -> Vec<Outcome> {
        let mut fixture = fixture(flags, &[Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO)]);
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
            .ultimate_cooldown = cooldown;
        invoke_tick(
            &fixture.world,
            &IceTower,
            fixture.tower,
            Fixed64::from_raw(1),
        )
    }

    #[test]
    fn absolute_zero_freezes_for_two_seconds_and_waits_fifteen_seconds() {
        let ready = impact_outcomes(&["absolute_zero"], Fixed64::ZERO);
        assert!(ready.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { buff_id, duration, .. } if buff_id == "stun" && *duration == Fixed64::from_i32(2)
        )));
        assert!(ready.iter().any(|outcome| matches!(outcome,
            Outcome::ScriptSetTowerInternalCooldown { duration, .. }
                if *duration == Fixed64::from_i32(15)
        )));

        let cooling_down = impact_outcomes(&["absolute_zero"], Fixed64::from_i32(5));
        assert!(!cooling_down.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { buff_id, .. } if buff_id == "stun"
        )));
    }

    #[test]
    fn ice_aura_percentages_match_twenty_thirty_five_and_forty_percent() {
        for (flags, expected) in [
            (vec!["arctic_aura_20"], -0.2),
            (vec!["snowstorm"], -0.35),
            (vec!["cryo_cannon"], -0.4),
        ] {
            let outcomes = impact_outcomes(&flags, Fixed64::from_i32(5));
            assert!(outcomes.iter().any(|outcome| matches!(outcome,
                Outcome::AddBuff { buff_id, payload, .. }
                    if buff_id == "ice_aura_slow" && payload["movespeed_bonus_percentage"] == expected
            )), "missing authored aura for {flags:?}");
        }
    }

    #[test]
    fn cryo_cannon_fires_only_on_its_ten_second_cadence() {
        let ready = impact_outcomes(&["cryo_cannon"], Fixed64::ZERO);
        assert_eq!(
            ready
                .iter()
                .filter(|outcome| matches!(outcome, Outcome::ScriptDirectDamage { .. }))
                .count(),
            1
        );
        assert!(ready.iter().any(|outcome| matches!(outcome,
            Outcome::ScriptSetTowerInternalCooldown { duration, .. }
                if *duration == Fixed64::from_i32(10)
        )));

        let cooling_down = impact_outcomes(&["cryo_cannon"], Fixed64::from_i32(5));
        assert_eq!(
            cooling_down
                .iter()
                .filter(|outcome| matches!(outcome, Outcome::ScriptDirectDamage { .. }))
                .count(),
            0
        );
    }
}
