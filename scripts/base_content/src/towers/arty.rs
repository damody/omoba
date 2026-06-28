//! Arty Tower（繽紛吉拿棒迫擊砲）— 超長程 AoE 砲，命中時可附加暈眩。
//!
//! 支援:
//! - Path1: arty_stun（1.0s 暈眩）, arty_stun_2（2.0s）, arty_stun_3（3.0s）
//! - Stat: splash_bonus, damage_bonus, range_bonus（透過 get_final_*）
//! - 高傷害 / 慢攻速 / 超長射程，適合控制型打法

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct ArtyTower;

const STATS: &TowerStats = &TOWER_ARTY_STATS;

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
        let stats = super::tower_stats(TOWER_ARTY, STATS);
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

        let atk = w.get_final_atk(e);
        let splash_bonus = w.get_stat_bonus(e, StatKey::SplashBonus);
        let splash = stats.splash_radius + splash_bonus;

        // 暈眩等級：arty_stun_3 > arty_stun_2 > arty_stun（依升級路徑疊加）
        let stun = if w.has_tower_flag(e, RStr::from_str("arty_stun_3")) {
            Fixed64::from_i32(3)
        } else if w.has_tower_flag(e, RStr::from_str("arty_stun_2")) {
            Fixed64::from_i32(2)
        } else if w.has_tower_flag(e, RStr::from_str("arty_stun")) {
            Fixed64::from_i32(1)
        } else {
            Fixed64::ZERO
        };

        w.log_info(RStr::from_str("[tower_arty] fire!"));
        if let RSome(t_pos) = w.get_pos(target) {
            w.set_facing(e, omoba_sim::trig::atan2(t_pos.y - pos.y, t_pos.x - pos.x));
        }
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: PathSpec::Homing { target },
            speed: stats.bullet_speed,
            damage: atk,
            hit_radius: Fixed64::ZERO,
            splash_radius: splash,
            slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO,
            stun_duration: stun,
            kind_id: PROJECTILE_BOMB.0,
        });
    }
}
