//! Ice Monkey — 減速塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: deep_freeze (1s stun), icicle_impale (直線穿透 + 150 splash + 25 dmg)
//! - Stat: slow_factor_override (越小越強), slow_duration_bonus, splash_bonus,
//!   damage_bonus, range_bonus (透過 get_final_*)
//!
//! TODO:
//! - arctic_aura_20 / snowstorm / cryo_cannon: 需 aura tick（Task 14）
//! - embrittle_* : 需 damage_taken_bonus hook（Task 14）
//! - refreeze: 命中時 remove+add slow，簡化版暫未處理

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct IceTower;

const ATK: f32 = 3.0;
const ASD_INTERVAL: f32 = 1.5;
const RANGE: f32 = 180.0;
const BULLET_SPEED: f32 = 600.0;
const SPLASH_RADIUS: f32 = 90.0;
const SLOW_FACTOR: f32 = 0.5; // 減速至 50%
const SLOW_DURATION: f32 = 2.0;

impl UnitScript for IceTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("tower_ice")
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
            splash_radius: SPLASH_RADIUS,
            hit_radius: 0.0,
            slow_factor: SLOW_FACTOR,
            slow_duration: SLOW_DURATION,
            cost: 400,
            footprint: 40.0,
            hp: 1.0,
            turn_speed_deg: 360.0,
            label: RString::from("Ice Monkey"),
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
            RNone => return,
        };

        w.set_asd_count(e, asd_count - asd_interval);

        let atk = w.get_final_atk(e);

        // slow_factor_override：upgrade 寫入的目標 factor（越小越強，clamp 在 (0, 1) 才採用）
        let slow_override = w.get_stat_bonus(e, StatKey::SlowFactorOverride);
        let slow_factor = if slow_override > 0.0 && slow_override < 1.0 {
            slow_override
        } else {
            SLOW_FACTOR
        };

        let slow_dur_bonus = w.get_stat_bonus(e, StatKey::SlowDurationBonus);
        let slow_duration = SLOW_DURATION + slow_dur_bonus;

        let splash_bonus = w.get_stat_bonus(e, StatKey::SplashBonus);
        let splash_radius = SPLASH_RADIUS + splash_bonus;

        let stun = if w.has_tower_flag(e, RStr::from_str("deep_freeze")) {
            1.0
        } else {
            0.0
        };
        let icicle = w.has_tower_flag(e, RStr::from_str("icicle_impale"));

        let (path_spec, final_splash, final_damage, kind_tag) = if icicle {
            // 朝 target 直線穿透（至 1.5 倍 range）
            let t_pos = match w.get_pos(target) {
                RSome(p) => p,
                RNone => return,
            };
            let dx = t_pos.x - pos.x;
            let dy = t_pos.y - pos.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let nx = dx / len * range * 1.5;
            let ny = dy / len * range * 1.5;
            let end = Vec2f::new(pos.x + nx, pos.y + ny);
            (
                PathSpec::Straight { end_pos: end },
                150.0_f32,
                atk.max(25.0),
                "icicle",
            )
        } else {
            (
                PathSpec::Homing { target },
                splash_radius,
                atk,
                "ice",
            )
        };

        w.log_info(RStr::from_str("[tower_ice] fire!"));
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: path_spec,
            speed: BULLET_SPEED,
            damage: final_damage,
            hit_radius: 0.0,
            splash_radius: final_splash,
            slow_factor,
            slow_duration,
            stun_duration: stun,
            kind_tag: RString::from(kind_tag),
        });

        // TODO arctic_aura_20 / snowstorm / cryo_cannon: 需 aura tick + damage_taken_bonus hook (Task 14)
        // TODO embrittle_weak / embrittle_crit: 需 damage_taken_bonus (Task 14)
        // TODO refreeze: 命中時對 target remove_buff + add_stat_buff
    }
}
