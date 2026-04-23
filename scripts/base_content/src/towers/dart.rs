//! Dart Monkey — 單體 homing 快射塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: triple_shot (3 發 15° 扇形), fan_club (5 發 30° 扇形 + 彈速 ×2),
//!   spike_o_pult (40dmg splash 100 巨釘, 彈速 ×0.5)
//! - Path2: always_crit, mega_crit (crit 時 60dmg splash 60)
//! - Stat: crit_chance, crit_bonus, damage_bonus, range_bonus (透過 get_final_*)
//!
//! TODO: attack_speed_multiplier 目前不會影響腳本層 ASD（見 Task 14/15 plan）。

use omb_script_abi::prelude::*;

pub struct DartTower;

// 數值 source of truth — 改完重 build DLL 就生效，不用動 host
const ATK: f32 = 10.0;
const ASD_INTERVAL: f32 = 0.8;
const RANGE: f32 = 350.0;
const BULLET_SPEED: f32 = 1200.0;

const BONUS_PROC_CHANCE: f32 = 0.25;
const BONUS_DAMAGE: f32 = 30.0;

impl UnitScript for DartTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("tower_dart")
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        w.set_tower_atk(e, ATK);
        w.set_tower_range(e, RANGE);
        w.set_asd_interval(e, ASD_INTERVAL);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(TowerMetadata {
            atk: ATK,
            asd_interval: ASD_INTERVAL,
            range: RANGE,
            bullet_speed: BULLET_SPEED,
            splash_radius: 0.0,
            hit_radius: 0.0,
            slow_factor: 0.0,
            slow_duration: 0.0,
            cost: 200,
            footprint: 40.0,
            hp: 1.0,
            turn_speed_deg: 360.0,
            label: RString::from("Dart Monkey"),
        })
    }

    fn on_tick(&self, e: EntityHandle, dt: f32, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= 0.0 {
            return;
        }
        let mut asd_count = w.get_asd_count(e);
        if asd_count < asd_interval {
            asd_count += dt;
            w.set_asd_count(e, asd_count);
        }
        if asd_count < asd_interval {
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

        w.set_asd_count(e, asd_count - asd_interval);

        let atk = w.get_final_atk(e);

        let fan_club = w.has_tower_flag(e, RStr::from_str("fan_club"));
        let triple = w.has_tower_flag(e, RStr::from_str("triple_shot"));
        let spike = w.has_tower_flag(e, RStr::from_str("spike_o_pult"));

        // 決定發射模式
        let (count, spread_deg) = if fan_club {
            (5u32, 30.0_f32)
        } else if triple {
            (3u32, 15.0_f32)
        } else {
            (1u32, 0.0_f32)
        };

        // Spike-o-pult 覆蓋：巨釘、splash、半速彈
        let (bullet_speed, damage, splash_radius) = if spike {
            (BULLET_SPEED * 0.5, 40.0_f32.max(atk), 100.0)
        } else {
            let speed_mul = if fan_club { 2.0 } else { 1.0 };
            (BULLET_SPEED * speed_mul, atk, 0.0)
        };

        w.log_info(RStr::from_str("[tower_dart] fire!"));

        let t_pos = match w.get_pos(target) {
            RSome(p) => p,
            RNone => return,
        };
        let dx = t_pos.x - pos.x;
        let dy = t_pos.y - pos.y;
        let base_angle = dy.atan2(dx);

        for i in 0..count {
            let angle = if count == 1 {
                base_angle
            } else {
                let offset = spread_deg * core::f32::consts::PI / 180.0;
                let step = (2.0 * offset) / (count as f32 - 1.0);
                base_angle - offset + step * (i as f32)
            };

            // Spike 或多發：直線；單發 homing
            let path_spec = if spike || count > 1 {
                let end = Vec2f::new(
                    pos.x + angle.cos() * range * 1.5,
                    pos.y + angle.sin() * range * 1.5,
                );
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
                hit_radius: 0.0,
                splash_radius,
                slow_factor: 0.0,
                slow_duration: 0.0,
                stun_duration: 0.0,
                kind_tag: RString::from(if spike { "spike_opult" } else { "dart" }),
            });
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
        let crit_chance_bonus = w.get_stat_bonus(attacker, RStr::from_str("crit_chance"));
        let effective_chance = if crit_chance_bonus > 0.0 {
            crit_chance_bonus
        } else {
            BONUS_PROC_CHANCE
        };

        let roll = w.rand_f32();
        if !always && roll >= effective_chance {
            return;
        }

        // crit_bonus override
        let crit_bonus_extra = w.get_stat_bonus(attacker, RStr::from_str("crit_bonus"));
        let bonus_damage = if crit_bonus_extra > 0.0 {
            crit_bonus_extra
        } else {
            BONUS_DAMAGE
        };

        w.log_info(RStr::from_str("[tower_dart] crit!"));
        w.deal_damage(
            victim,
            bonus_damage,
            DamageKind::Physical,
            RSome(attacker),
        );
        if let RSome(at) = w.get_pos(victim) {
            w.play_vfx(RStr::from_str("vfx_dart_crit"), at);
        }

        // mega_crit：crit 時額外 AoE 爆炸
        if w.has_tower_flag(attacker, RStr::from_str("mega_crit")) {
            if let RSome(at) = w.get_pos(victim) {
                w.play_vfx(RStr::from_str("vfx_explosion"), at);
                w.deal_damage_splash(
                    at,
                    60.0,
                    60.0,
                    DamageKind::Physical,
                    RSome(attacker),
                );
            }
        }
    }
}
