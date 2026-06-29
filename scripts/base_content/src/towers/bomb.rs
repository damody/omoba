//! Bomb Shooter — AoE 塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: bomb_stun (0.5s stun on splash), missile (彈速 ×1.5),
//!   frag_8 / frag_12 / frag_homing (命中後 8/12/16 碎片)
//! - Stat: splash_bonus, damage_bonus, range_bonus (透過 get_final_*)
//!
//! 待辦事項：
//! - moab_assassin: 超級導彈目前無 15s 冷卻計時（簡化版）。
//!   待 ultimate_cooldown FFI 就緒後，限制觸發頻率為每 15s 一次。
//! - frag_recursive: 碎片再產生碎片的深度遞迴，暫只調高碎片傷害 (45 vs 25)

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

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
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
        let missile = w.has_tower_flag(e, RStr::from_str("missile"));
        let bullet_speed = if missile {
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

        // moab_assassin：額外發射高傷害超級導彈（atk × 10）
        // TODO: 目前無 15s 冷卻計時（簡化版），待 ultimate_cooldown FFI 就緒後補充冷卻限制
        if w.has_tower_flag(e, RStr::from_str("moab_assassin")) {
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
        }
    }

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        // Cluster bomb 碎片：命中時於 victim 位置向 N 方向發射子彈
        let frag_count: u32 = if w.has_tower_flag(attacker, RStr::from_str("frag_homing")) {
            16
        } else if w.has_tower_flag(attacker, RStr::from_str("frag_12")) {
            12
        } else if w.has_tower_flag(attacker, RStr::from_str("frag_8")) {
            8
        } else {
            return;
        };

        // frag_recursive 目前只拉高單片傷害（真正遞迴留 TODO）
        let frag_damage = if w.has_tower_flag(attacker, RStr::from_str("frag_recursive")) {
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

        // 360° / frag_count，用 from_degrees_i32 維持決定性 LUT
        let step_deg: i32 = 360 / (frag_count as i32);

        for i in 0..frag_count {
            let angle = omoba_sim::trig::Angle::from_degrees_i32(step_deg * (i as i32));
            let end = Vec2 {
                x: pos.x + omoba_sim::trig::cos(angle) * frag_range,
                y: pos.y + omoba_sim::trig::sin(angle) * frag_range,
            };
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: attacker,
                path: PathSpec::Straight { end_pos: end },
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
