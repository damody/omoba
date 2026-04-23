//! 雜賀鐵炮兵 — Saika Magoichi 的 `saika_reinforcements` 召喚物。
//!
//! 每 tick 累計 asd、在攻擊射程內找最近敵人 → 發射 homing projectile。
//! 初版不位移（被召喚後原地站樁射擊），未來可擴 on_tick 加移動追敵。

use omb_script_abi::prelude::*;

pub struct SaikaGunner;

const BULLET_SPEED: f32 = 900.0;

impl UnitScript for SaikaGunner {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("saika_gunner")
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

        // 射程內最近敵人（範圍外的敵人本版不處理；未來可走 move 邏輯）
        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let range = w.get_tower_range(e);
        let target = match w.query_nearest_enemy(pos, range, e) {
            RSome(t) => t,
            RNone => return, // 保留 asd_count，下次有敵人立即開火
        };

        // 消耗一次 asd、發射
        w.set_asd_count(e, asd_count - asd_interval);
        let atk = w.get_tower_atk(e);
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
            kind_tag: RString::from("saika_shot"),
        });
    }
}
