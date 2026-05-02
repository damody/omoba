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

const BULLET_SPEED: Fixed32 = Fixed32::from_i32(900);
/// 追擊搜索範圍 = 攻擊射程 × 此倍率（跟 Unit::create_saika_gunner 的 aggro 匹配）
const AGGRO_MULTIPLIER: Fixed32 = Fixed32::from_raw(1331); // 1.3 * 1024 = 1331.2 ≈ 1331
/// 移動速度（單位/秒），跟 Unit 預設的 move_speed 匹配
const MOVE_SPEED: Fixed32 = Fixed32::from_i32(280);

impl UnitScript for SaikaGunner {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str(SUMMON_SAIKA_GUNNER.as_str())
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed32, w: &mut GameWorldDyn<'_>) {
        let pos = match w.get_pos(e) {
            RSome(p) => p,
            RNone => return,
        };
        let attack_range = w.get_tower_range(e);
        let aggro_range = attack_range * AGGRO_MULTIPLIER;

        let asd_interval = w.get_asd_interval(e);
        if asd_interval <= Fixed32::ZERO {
            return;
        }
        // 玩家命令移動由 host 端 `summon_move_tick` 處理 MoveTarget；script 專注攻擊 AI。
        // summon_move_tick 在本 tick 前執行，抵達後移除 MoveTarget。

        // 射程內敵人 → 轉向目標並開火
        if let RSome(target) = w.query_nearest_enemy(pos, attack_range, e) {
            // 先轉向目標：即使還沒 ready to fire 也讓模型面朝敵人，避免子彈視覺
            // 上「從身體外射出」（前端以 facing 決定槍口模型方向）。
            if let RSome(tpos) = w.get_pos(target) {
                let dx = tpos.x - pos.x;
                let dy = tpos.y - pos.y;
                if dx * dx + dy * dy > Fixed32::from_raw(1) {
                    w.set_facing(e, omoba_sim::trig::atan2(dy, dx));
                }
            }
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
                    hit_radius: Fixed32::ZERO,
                    splash_radius: Fixed32::ZERO,
                    slow_factor: Fixed32::ZERO,
                    slow_duration: Fixed32::ZERO,
                    stun_duration: Fixed32::ZERO,
                    kind_id: PROJECTILE_SAIKA_SHOT.0,
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
                if len > Fixed32::ONE {
                    let step = MOVE_SPEED * dt;
                    // 透過 host 碰撞檢測 —— 會自動避開其他 CollisionRadius 實體與
                    // BlockedRegion blocker；若被完全擋住會留在原地。
                    let new_pos = w.advance_with_collision(e, target_pos, step);
                    w.set_pos(e, new_pos);
                    w.set_facing(e, omoba_sim::trig::atan2(dy, dx));
                }
            }
        }
        // aggro 外完全不動
    }
}
