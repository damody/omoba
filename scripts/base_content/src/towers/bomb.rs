//! Bomb Shooter — AoE 塔，MVP 支援 12 升級 flag / stat。
//!
//! 支援:
//! - Path1: bomb_stun (0.5s stun on splash), missile (彈速 ×1.5),
//!   frag_8 / frag_12 / frag_homing (命中後 8/12/16 碎片)
//! - Stat: splash_bonus, damage_bonus, range_bonus (透過 get_final_*)
//!
//! TODO:
//! - moab_assassin: 需 ultimate_cooldown FFI (Task 14+)
//! - frag_recursive: 碎片再產生碎片的深度遞迴，暫只調高碎片傷害 (45 vs 25)

use omb_script_abi::prelude::*;
use omb_script_abi::stat_keys::StatKey;

pub struct BombTower;

// 數值唯一來源：omb/Story/templates.json → omoba_template_ids 編譯期生成
// `TOWER_BOMB_STATS`。改數值編 omoba-template-ids → scripts 重 build 即生效。
const STATS: &TowerStats = &TOWER_BOMB_STATS;

impl UnitScript for BombTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(TOWER_BOMB.as_str())
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
            label: RString::from(tower_display(TOWER_BOMB)),
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

        // splash_bonus: base STATS.splash_radius + sum_add("splash_bonus")
        let splash_bonus = w.get_stat_bonus(e, StatKey::SplashBonus);
        let splash = STATS.splash_radius + splash_bonus;

        let stun = if w.has_tower_flag(e, RStr::from_str("bomb_stun")) {
            0.5
        } else {
            0.0
        };
        let missile = w.has_tower_flag(e, RStr::from_str("missile"));
        let bullet_speed = if missile { STATS.bullet_speed * 1.5 } else { STATS.bullet_speed };

        w.log_info(RStr::from_str("[tower_bomb] fire!"));
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: PathSpec::Homing { target },
            speed: bullet_speed,
            damage: atk,
            hit_radius: 0.0,
            splash_radius: splash,
            slow_factor: 0.0,
            slow_duration: 0.0,
            stun_duration: stun,
            kind_id: PROJECTILE_BOMB.0,
        });
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
            45.0
        } else if w.has_tower_flag(attacker, RStr::from_str("frag_12")) {
            25.0
        } else {
            15.0
        };

        let pos = match w.get_pos(victim) {
            RSome(p) => p,
            RNone => return,
        };
        let step = core::f32::consts::TAU / (frag_count as f32);
        let frag_range = 300.0;

        for i in 0..frag_count {
            let angle = step * (i as f32);
            let end = Vec2f::new(
                pos.x + angle.cos() * frag_range,
                pos.y + angle.sin() * frag_range,
            );
            w.spawn_projectile_ex(ProjectileSpec {
                from: pos,
                owner: attacker,
                path: PathSpec::Straight { end_pos: end },
                speed: 800.0,
                damage: frag_damage,
                hit_radius: 40.0,
                splash_radius: 0.0,
                slow_factor: 0.0,
                slow_duration: 0.0,
                stun_duration: 0.0,
                kind_id: PROJECTILE_BOMB_FRAG.0,
            });
        }
    }
}
