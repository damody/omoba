//! Dart Monkey — 單體 homing 快射塔。
//!
//! 由 `on_tick` 主動驅動：累計 asd → 找最近敵人 → 發射 homing projectile。
//! `on_attack_hit` 保留原本的 25% proc 額外傷害（之後可挪到升級路線）。

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

    fn on_tick(&self, e: EntityHandle, dt: f32, w: &mut GameWorldDyn<'_>) {
        // 攻速計時：尚未到下一發間隔 → 累積
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

        // 位置 + 射程內最近敵人
        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let range = w.get_tower_range(e);
        let target = match w.query_nearest_enemy(pos, range, e) {
            RSome(t) => t,
            RNone => return, // 沒目標，保留 asd_count（下次有敵人立即開火）
        };

        // 消耗一次 asd、朝目標開槍
        // 轉向不是 script 的責任 — host `tower_tick` 會依 TurnSpeed 平滑朝敵人旋轉
        w.set_asd_count(e, asd_count - asd_interval);

        let atk = w.get_tower_atk(e);
        w.log_info(RStr::from_str("[tower_dart] fire!"));
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos,
            owner: e,
            path: PathSpec::Homing { target },
            speed: BULLET_SPEED,
            damage: atk,
            hit_radius: 0.0,
            splash_radius: 0.0,
            slow_factor: 0.0,
            slow_duration: 0.0,
            kind_tag: RString::from("dart"),
        });
    }

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        // 25% 機率額外造成 BONUS_DAMAGE
        let roll = w.rand_f32();
        if roll < BONUS_PROC_CHANCE {
            w.log_info(RStr::from_str("[tower_dart] bonus shot proc!"));
            w.deal_damage(
                victim,
                BONUS_DAMAGE,
                DamageKind::Physical,
                RSome(attacker),
            );
            if let RSome(at) = w.get_pos(victim) {
                w.play_vfx(RStr::from_str("vfx_dart_crit"), at);
            }
        }
    }
}
