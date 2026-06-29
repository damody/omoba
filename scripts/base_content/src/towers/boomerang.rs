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
        let asd_interval = w.get_asd_interval(e);
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
        // turbo_charge：攻速 ×2 + 傷害 +30% 由 Lua stat_mod（AttackSpeedMultiplier × 0.5、
        // BaseDamageOutgoingPercentage +0.3）自動套用至 get_asd_interval / get_final_atk，
        // 無需額外的 has_tower_flag 檢查。behavior_flag 保留供未來渲染特效使用。

        // moab_press：附加減速
        let (slow_factor, slow_duration) =
            if w.has_tower_flag(e, RStr::from_str("moab_press")) {
                (MOAB_PRESS_SLOW_FACTOR, MOAB_PRESS_SLOW_DUR)
            } else {
                (Fixed64::ZERO, Fixed64::ZERO)
            };

        // 決定發射數量：
        //   storm_shuriken → 3（手裡劍系）
        //   bionic_burst   → 3（仿生系）
        //   double_shuriken / glaive_lord → 2
        //   其他 → 1
        let count: u32 = if storm || w.has_tower_flag(e, RStr::from_str("bionic_burst")) {
            3
        } else if w.has_tower_flag(e, RStr::from_str("double_shuriken"))
            || w.has_tower_flag(e, RStr::from_str("glaive_lord"))
        {
            2
        } else {
            1
        };

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
                } else {
                    (i as i32) * 10 - 10 // -10, 0, +10
                };
                let offset_ticks =
                    omoba_sim::trig::Angle::from_degrees_i32(offset_deg).ticks();
                let angle =
                    omoba_sim::trig::Angle::from_ticks(base_angle.ticks() + offset_ticks);
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

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        // glaive_ricochet / storm_shuriken：命中後從受擊位置彈向下一個目標
        let has_ricochet = w.has_tower_flag(attacker, RStr::from_str("glaive_ricochet"));
        let has_storm = w.has_tower_flag(attacker, RStr::from_str("storm_shuriken"));
        if !has_ricochet && !has_storm {
            return;
        }

        let victim_pos = match w.get_pos(victim) {
            RSome(p) => p,
            RNone => return,
        };

        // 在受擊點附近找下一個敵人（排除原目標自身）
        let nearby = w.query_enemies_in_range(victim_pos, RICOCHET_RADIUS, attacker);
        let bounce_target = nearby.iter().copied().find(|&e| e != victim);
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
            path: PathSpec::Homing { target: bounce_target },
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
