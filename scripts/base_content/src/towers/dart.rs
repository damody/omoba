//! Dart Monkey — 單體 homing 快射塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: triple_shot (3 發 15° 扇形), fan_club (5 發 30° 扇形 + 彈速 ×2),
//!   spike_o_pult (40dmg splash 100 巨釘, hit radius 50)
//! - Path2: always_crit, mega_crit (crit 時 60dmg splash 60)
//! - Stat: crit_chance, crit_bonus, damage_bonus, range_bonus (透過 get_final_*)
//!
use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct DartTower;

// 數值唯一來源：scripts/lua_data/templates.lua → omoba_template_ids 編譯期生成
// `TOWER_DART_STATS`。
const STATS: &TowerStats = &TOWER_DART_STATS;

// 升級加成（不來自 templates.lua — 是腳本邏輯參數）
// 0.25 * 1024 = 256
const BONUS_PROC_CHANCE: Fixed64 = Fixed64::from_raw(256);
// 30.0 * 1024 = 30720
const BONUS_DAMAGE: Fixed64 = Fixed64::from_raw(30720);
const HEAVY_BURST_ABILITY_ID: &str = "dart_heavy_burst";
const HEAVY_BURST_ACTIVE_BUFF: &str = "dart_heavy_burst_active";

impl UnitScript for DartTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_DART.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        let stats = super::tower_stats(TOWER_DART, STATS);
        w.set_tower_atk(e, stats.atk);
        w.set_tower_range(e, stats.range);
        w.set_asd_interval(e, stats.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(super::tower_metadata_from_consts(
            TOWER_DART,
            STATS,
            &TOWER_DART_RENDER,
            TOWER_DART_ATTACK_TIMING,
        ))
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed64::ZERO {
            return;
        }
        let stats = super::tower_stats(TOWER_DART, STATS);
        let timing = super::tower_attack_timing(TOWER_DART, TOWER_DART_ATTACK_TIMING);
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
            RNone => return, // 沒目標，保留 asd_count（下次有敵人立即開火）
        };
        if matches!(phase, super::AttackPhaseStep::Ready) {
            if let RSome(t_pos) = w.get_pos(target) {
                w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
            }
            super::start_attack_windup(e, asd_interval, timing, Target::Entity(target), w);
            return;
        }

        let atk = w.get_final_atk(e);

        let fan_club = w.has_tower_flag(e, RStr::from_str("fan_club"));
        let triple = w.has_tower_flag(e, RStr::from_str("triple_shot"));
        let spike = w.has_tower_flag(e, RStr::from_str("spike_o_pult"));

        // 決定發射模式 — spread 用整數度數，給 Angle::from_degrees_i32 直接用
        let (count, spread_deg): (u32, i32) = if fan_club {
            (5, 30)
        } else if triple {
            (3, 15)
        } else {
            (1, 0)
        };

        // Spike-o-pult 覆蓋：巨釘、splash、直徑 100 的沿路判定（優先於 sharp_pierce）
        let (bullet_speed, damage, hit_radius, splash_radius) = if spike {
            let forty = Fixed64::from_i32(40);
            let dmg = if atk > forty { atk } else { forty };
            (
                stats.bullet_speed,
                dmg,
                Fixed64::from_i32(50),
                Fixed64::from_i32(100),
            )
        } else {
            let speed_mul = if fan_club {
                Fixed64::from_i32(2)
            } else {
                Fixed64::ONE
            };
            // sharp_pierce：hit_radius 擴大至 90 模擬穿透效果（spike_o_pult 優先，已在上方處理）
            let pierce_radius = if w.has_tower_flag(e, RStr::from_str("sharp_pierce")) {
                Fixed64::from_i32(90)
            } else {
                Fixed64::ZERO
            };
            (
                stats.bullet_speed * speed_mul,
                atk,
                pierce_radius,
                Fixed64::ZERO,
            )
        };

        w.log_info(RStr::from_str("[tower_dart] fire!"));

        let t_pos = match w.get_pos(target) {
            RSome(p) => p,
            RNone => return,
        };
        let dx = t_pos.x - pos.x;
        let dy = t_pos.y - pos.y;
        let base_angle = omoba_sim::trig::atan2(dy, dx);
        w.set_facing(e, base_angle);
        let range_x_1_5 = range * Fixed64::from_raw(1536); // 1.5

        for i in 0..count {
            let angle = if count == 1 {
                base_angle
            } else {
                // step_deg 計算式：(2 * spread_deg) / (count - 1)
                let denom = (count as i32) - 1;
                // i 個彈 step：from -spread_deg to +spread_deg；
                // 總刻度 = base_angle.ticks() + (-spread_deg + step_deg*i) （以度為單位）
                let offset_deg = -spread_deg + (2 * spread_deg * (i as i32)) / denom;
                let offset_ticks = omoba_sim::trig::Angle::from_degrees_i32(offset_deg).ticks();
                omoba_sim::trig::Angle::from_ticks(base_angle.ticks() + offset_ticks)
            };

            // Spike 或多發：直線；單發 homing
            let path_spec = if spike || count > 1 {
                let end = Vec2 {
                    x: pos.x + omoba_sim::trig::cos(angle) * range_x_1_5,
                    y: pos.y + omoba_sim::trig::sin(angle) * range_x_1_5,
                };
                PathSpec::Straight { end_pos: end }
            } else {
                PathSpec::Homing { target }
            };

            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: e,
                path: path_spec,
                speed: bullet_speed,
                damage,
                hit_radius,
                splash_radius,
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: if spike {
                    PROJECTILE_SPIKE_OPULT.0
                } else {
                    PROJECTILE_DART.0
                },
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
        if ability_id.as_str() != HEAVY_BURST_ABILITY_ID {
            return;
        }
        let remaining = access.get_tower_ability_active_remaining(tower, ability_id);
        if remaining > Fixed64::ZERO {
            w.add_buff(tower, RStr::from_str(HEAVY_BURST_ACTIVE_BUFF), remaining);
        }
    }

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        // always_crit：必爆
        let always = w.has_tower_flag(attacker, RStr::from_str("always_crit"));

        // crit_chance override：upgrade buff 寫入 crit_chance 就用那個；否則回 BONUS_PROC_CHANCE (0.25)
        let crit_chance_bonus = w.get_stat_bonus(attacker, StatKey::CritChance);
        let effective_chance = if crit_chance_bonus > Fixed64::ZERO {
            crit_chance_bonus
        } else {
            BONUS_PROC_CHANCE
        };

        let roll = w.rand_unit();
        if !always && roll >= effective_chance {
            return;
        }

        // 暴擊獎勵覆蓋
        let crit_bonus_extra = w.get_stat_bonus(attacker, StatKey::CritBonus);
        let bonus_damage = if crit_bonus_extra > Fixed64::ZERO {
            crit_bonus_extra
        } else {
            BONUS_DAMAGE
        };

        w.log_info(RStr::from_str("[tower_dart] crit!"));
        w.deal_damage(victim, bonus_damage, DamageKind::Physical, RSome(attacker));
        if let RSome(at) = w.get_pos(victim) {
            w.play_vfx(RStr::from_str("vfx_dart_crit"), at);
        }

        // mega_crit：crit 時額外 AoE 爆炸
        if w.has_tower_flag(attacker, RStr::from_str("mega_crit")) {
            if let RSome(at) = w.get_pos(victim) {
                let heavy_burst = w
                    .get_buff_remaining(attacker, RStr::from_str(HEAVY_BURST_ACTIVE_BUFF))
                    > Fixed64::ZERO;
                let (radius, damage) = if heavy_burst {
                    (Fixed64::from_i32(120), Fixed64::from_i32(120))
                } else {
                    (Fixed64::from_i32(60), Fixed64::from_i32(60))
                };
                w.play_vfx(RStr::from_str("vfx_explosion"), at);
                w.deal_damage_splash(at, radius, damage, DamageKind::Physical, RSome(attacker));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towers::projectile_test_support::{fixture, invoke_activation, invoke_attack_hit};
    use omoba_core::runtime::BuffStore;
    use omoba_core::{Outcome, Tower, TowerActiveAbilityState};
    use specs::WorldExt;

    #[test]
    fn heavy_burst_marks_five_second_window() {
        let mut fixture = fixture(&["always_crit", "mega_crit"], &[]);
        fixture
            .world
            .write_storage::<Tower>()
            .get_mut(fixture.tower)
            .unwrap()
            .active_ability = Some(TowerActiveAbilityState {
            ability_id: "dart_heavy_burst".to_string(),
            active_remaining: Fixed64::from_i32(5),
            activation_serial: 1,
            ..Default::default()
        });

        let outcomes = invoke_activation(
            &fixture.world,
            &DartTower,
            fixture.tower,
            "dart_heavy_burst",
        );

        assert!(outcomes.iter().any(|outcome| matches!(outcome,
            Outcome::AddBuff { buff_id, duration, .. }
                if buff_id == "dart_heavy_burst_active" && *duration == Fixed64::from_i32(5)
        )));
    }

    #[test]
    fn heavy_burst_doubles_mega_crit_damage_and_radius() {
        let mut fixture = fixture(
            &["always_crit", "mega_crit"],
            &[
                Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                Vec2::new(Fixed64::from_i32(90), Fixed64::ZERO),
            ],
        );
        fixture.world.write_resource::<BuffStore>().add(
            fixture.tower,
            "dart_heavy_burst_active",
            Fixed64::from_i32(5),
            serde_json::Value::Null,
        );

        let outcomes = invoke_attack_hit(
            &fixture.world,
            &DartTower,
            fixture.tower,
            fixture.enemies[0],
        );
        let active_splash_hits = outcomes
            .iter()
            .filter(|outcome| {
                matches!(outcome,
                    Outcome::ScriptDirectDamage { amount, .. }
                        if *amount == Fixed64::from_i32(120)
                )
            })
            .count();

        assert_eq!(active_splash_hits, 2);

        fixture
            .world
            .write_resource::<BuffStore>()
            .remove(fixture.tower, "dart_heavy_burst_active");
        let inactive = invoke_attack_hit(
            &fixture.world,
            &DartTower,
            fixture.tower,
            fixture.enemies[0],
        );
        let normal_splash_hits = inactive
            .iter()
            .filter(|outcome| {
                matches!(outcome,
                    Outcome::ScriptDirectDamage { amount, .. }
                        if *amount == Fixed64::from_i32(60)
                )
            })
            .count();
        assert_eq!(normal_splash_hits, 1);
    }

    #[test]
    fn heavy_burst_ignores_unknown_ability_id() {
        let fixture = fixture(&["always_crit", "mega_crit"], &[]);
        assert!(invoke_activation(&fixture.world, &DartTower, fixture.tower, "wrong").is_empty());
    }
}
