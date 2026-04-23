//! 雜賀鐵炮兵 — Saika Magoichi 的 `saika_reinforcements` 召喚物。
//!
//! 每 tick：
//! 1. 攻擊射程內有敵人 → 累計 asd、發射 homing projectile
//! 2. 攻擊射程外但 aggro 範圍內有敵人 → 朝目標方向位移逼近
//! 3. 完全沒有敵人 → 靜止
//!
//! 不走 host 端 hero_move_tick（僅處理 Hero）；位移在此 on_tick 內直接 `set_pos`。

use omb_script_abi::prelude::*;

pub struct SaikaGunner;

const BULLET_SPEED: f32 = 900.0;
/// 追擊搜索範圍 = 攻擊射程 × 此倍率（跟 Unit::create_saika_gunner 的 aggro 匹配）
const AGGRO_MULTIPLIER: f32 = 1.3;
/// 移動速度（單位/秒），跟 Unit 預設的 move_speed 匹配
const MOVE_SPEED: f32 = 280.0;

impl UnitScript for SaikaGunner {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("saika_gunner")
    }

    fn on_tick(&self, e: EntityHandle, dt: f32, w: &mut GameWorldDyn<'_>) {
        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let attack_range = w.get_tower_range(e);
        let aggro_range = attack_range * AGGRO_MULTIPLIER;

        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= 0.0 {
            return;
        }

        // 射程內敵人 → 開火
        if let RSome(target) = w.query_nearest_enemy(pos, attack_range, e) {
            let mut asd_count = w.get_asd_count(e);
            if asd_count < asd_interval {
                asd_count += dt;
                w.set_asd_count(e, asd_count);
            }
            if asd_count >= asd_interval {
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
                    stun_duration: 0.0,
                    kind_tag: RString::from("saika_shot"),
                });
            }
            return;
        }

        // 射程外但 aggro 內 → 逼近
        if let RSome(chase) = w.query_nearest_enemy(pos, aggro_range, e) {
            if let RSome(target_pos) = w.get_pos(chase) {
                let dx = target_pos.x - pos.x;
                let dy = target_pos.y - pos.y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 1.0 {
                    let step = MOVE_SPEED * dt;
                    let nx = pos.x + dx / len * step;
                    let ny = pos.y + dy / len * step;
                    w.set_pos(e, Vec2f::new(nx, ny));
                    // 朝目標方向 facing（radians, +X=0, CCW）
                    w.set_facing(e, dy.atan2(dx));
                }
            }
        }
        // aggro 外完全不動
    }
}
