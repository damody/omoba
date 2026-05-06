//! Ice Monkey — 減速塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: deep_freeze (1s stun), icicle_impale (直線穿透 + 150 splash + 25 dmg)
//! - Stat: slow_factor_override (越小越強), slow_duration_bonus, splash_bonus,
//!   damage_bonus, range_bonus (透過 get_final_*)
//!
//! 待辦事項：
//! - arctic_aura_20 / snowstorm / cryo_cannon: 需 aura tick（Task 14）
//! - embrittle_* : 需 damage_taken_bonus hook（Task 14）
//! - refreeze: 命中時 remove+add slow，簡化版暫未處理

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct IceTower;

// 數值唯一來源：scripts/lua_data/templates.lua → omoba_template_ids 編譯期生成
// `TOWER_ICE_STATS`。
const STATS: &TowerStats = &TOWER_ICE_STATS;

impl UnitScript for IceTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_ICE.as_str())
    }

    fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
        w.set_tower_atk(e, STATS.atk);
        w.set_tower_range(e, STATS.range);
        w.set_asd_interval(e, STATS.asd_interval);
    }

    fn tower_metadata(&self) -> ROption<TowerMetadata> {
        RSome(TowerMetadata {
            atk: STATS.atk,
            asd_interval: STATS.asd_interval,
            range: STATS.range,
            bullet_speed: STATS.bullet_speed,
            splash_radius: STATS.splash_radius,
            hit_radius: STATS.hit_radius,
            slow_factor: STATS.slow_factor,
            slow_duration: STATS.slow_duration,
            cost: STATS.cost,
            footprint: STATS.footprint,
            hp: STATS.hp,
            turn_speed_deg: STATS.turn_speed_deg,
            label: RString::from(tower_display(TOWER_ICE)),
        })
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed64::ZERO {
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
        let slow_factor = if slow_override > Fixed64::ZERO && slow_override < Fixed64::ONE {
            slow_override
        } else {
            STATS.slow_factor
        };

        let slow_dur_bonus = w.get_stat_bonus(e, StatKey::SlowDurationBonus);
        let slow_duration = STATS.slow_duration + slow_dur_bonus;

        let splash_bonus = w.get_stat_bonus(e, StatKey::SplashBonus);
        let splash_radius = STATS.splash_radius + splash_bonus;

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
            let len = if len_raw < Fixed64::ONE { Fixed64::ONE } else { len_raw };
            let scale = Fixed64::from_raw(1536); // 1.5
            let nx = dx / len * range * scale;
            let ny = dy / len * range * scale;
            let end = Vec2 { x: pos.x + nx, y: pos.y + ny };
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
            speed: STATS.bullet_speed,
            damage: final_damage,
            hit_radius: Fixed64::ZERO,
            splash_radius: final_splash,
            slow_factor,
            slow_duration,
            stun_duration: stun,
            kind_id,
        });

        // TODO arctic_aura_20 / snowstorm / cryo_cannon: 需 aura tick + damage_taken_bonus hook (Task 14)
        // TODO embrittle_weak / embrittle_crit: 需 damage_taken_bonus (Task 14)
        // TODO refreeze: 命中時對 target remove_buff + add_stat_buff
    }
}
