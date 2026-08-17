//! 雨鐵炮（rain_iron_cannon）— 雜賀孫市的 R 位被動：普攻命中附加扇形 AoE 真實傷害。
//!
//! 學到後 `on_learn` 套永久可見 buff；`on_attack_hit` 以受擊點為中心、朝
//! attacker→victim 方向的扇形（半角 templates.lua 給）內所有敵人造成
//! `atk × true_damage_pct[level]` 真實傷害。被動不走 `execute`。

use abi_stable::std_types::{ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, DamageProfile, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_template_ids::ABILITY_RAIN_IRON_CANNON;

use crate::ability_builder::{build_ability_ffi, extra_at_id};

const BUFF_ID: &str = "rain_iron_cannon_passive";

pub struct RainIronCannonHandler;

impl AbilityScript for RainIronCannonHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_RAIN_IRON_CANNON.as_str())
    }

    fn execute(
        &self,
        _caster: EntityHandle,
        _target: Target,
        _level: u8,
        _level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        world.log_warn(RStr::from_str(
            "[rain_iron_cannon] passive ability cannot be cast actively",
        ));
        ROk(())
    }

    fn on_learn(&self, caster: EntityHandle, new_level: u8, world: &mut GameWorldDyn<'_>) {
        // 永久可見 buff — payload 標等級，前端可據此渲染 icon / tooltip。
        // 階段 1de.2：此有效負載不透過 BuffStore::sum_add 消耗（無
        // 匹配 StatKey);它是一個工具提示/僅視覺標記。保留f32
        // 前端工具提示路徑的發射。如果未來的系統讀取
        // `true_damage_pct` 數值，切換到 `.raw()`。
        let pct =
            extra_at_id(ABILITY_RAIN_IRON_CANNON, "true_damage_pct", new_level).to_f32_for_render();
        let modifiers = serde_json::json!({
            "visual_effect": "rain_iron_cannon_passive",
            "level": new_level,
            "true_damage_pct": pct,
        });
        let s = modifiers.to_string();
        // 永久增益 - 固定 64 持續時間使用接近 MAX 作為「無限期」哨兵
        // （BuffStore 約定；被動永遠不會透過刻度遞減清除）。
        world.add_stat_buff(
            caster,
            RStr::from_str(BUFF_ID),
            Fixed64::from_i32(i32::MAX / 1024),
            (&*s).into(),
        );
    }

    fn on_attack_hit(
        &self,
        _owner: EntityHandle,
        attacker: EntityHandle,
        victim: EntityHandle,
        level: u8,
        world: &mut GameWorldDyn<'_>,
    ) {
        let pct = extra_at_id(ABILITY_RAIN_IRON_CANNON, "true_damage_pct", level);
        if pct <= Fixed64::ZERO {
            return;
        }
        let aoe_radius = extra_at_id(ABILITY_RAIN_IRON_CANNON, "aoe_radius", level);
        let arc_half = extra_at_id(ABILITY_RAIN_IRON_CANNON, "arc_half_angle_rad", level);

        let attacker_pos = match world.get_pos(attacker) {
            RSome(p) => p,
            _ => return,
        };
        let victim_pos = match world.get_pos(victim) {
            RSome(p) => p,
            _ => return,
        };
        let dx = victim_pos.x - attacker_pos.x;
        let dy = victim_pos.y - attacker_pos.y;
        // 微小距離保護（f32 中為 0.0001；Fixed64 中原始 1 ~ 0.001）
        if dx * dx + dy * dy < Fixed64::from_raw(1) {
            return;
        }
        let base_angle = omoba_sim::trig::atan2(dy, dx);
        let atk = world.get_final_atk(attacker);
        let true_damage = atk * pct;
        if true_damage <= Fixed64::ZERO {
            return;
        }

        // arc_half 以弧度形式儲存在 templates.lua 中（舊版）。將弧度轉換為刻度：
        // 每弧度刻度數 = TAU_TICKS / TAU = 4096 / (2π) ≈ 651.9。
        // 乘以 652（原始，因此隱式除以 1024）—但更簡單：half_arc_ticks =
        // 原始弧度值 * (4096 / 2π)。我們透過刻度數學以相同的精度進行近似。
        // arc_half 的原始值為「弧度 * 1024」。刻度 = 弧度 * 4096 / (2π) =
        // 原 * (4096 / (2π * 1024)) ≈ 原 * 0.6366
        // 使用 i64 乘法：ticks ≈ raw * 4096 * 1000 / (6283 * 1024) — 保持確定性。
        // arc_half_ticks = arc_half.raw() * 4096 / (2π * 1024); 2π ≈ 6283/1000。
        let arc_half_ticks: i32 = ((arc_half.raw() as i64 * 4096 * 1000) / (6283 * 1024)) as i32;

        let enemies = world.query_enemies_in_range(victim_pos, aoe_radius, attacker);
        for enemy_h in enemies.iter().copied() {
            let epos = match world.get_pos(enemy_h) {
                RSome(p) => p,
                _ => continue,
            };
            let edx = epos.x - attacker_pos.x;
            let edy = epos.y - attacker_pos.y;
            if edx * edx + edy * edy < Fixed64::from_raw(1) {
                continue;
            }
            let enemy_angle = omoba_sim::trig::atan2(edy, edx);
            // 計算最短帶符號刻度差模 TAU (4096)。
            const TAU_TICKS: i32 = 4096;
            let mut diff = enemy_angle.ticks() - base_angle.ticks();
            // 標準化為 (-TAU/2, TAU/2]
            while diff > TAU_TICKS / 2 {
                diff -= TAU_TICKS;
            }
            while diff < -TAU_TICKS / 2 {
                diff += TAU_TICKS;
            }
            if diff.abs() <= arc_half_ticks {
                world.deal_damage(
                    enemy_h,
                    true_damage,
                    DamageKind::Pure,
                    DamageProfile::TRUE,
                    RSome(attacker),
                );
            }
        }
    }
}

pub fn rain_iron_cannon_ffi() -> AbilityDefFFI {
    build_ability_ffi(ABILITY_RAIN_IRON_CANNON, RainIronCannonHandler, Vec::new())
}
