//! 雨鐵炮（rain_iron_cannon）— 雜賀孫市的 R 位被動：普攻命中附加扇形 AoE 真實傷害。
//!
//! 學到後 `on_learn` 套永久可見 buff；`on_attack_hit` 以受擊點為中心、朝 attacker→victim
//! 方向 45° 半角扇形 150 半徑，對範圍內所有敵人造成 `atk × (15/25/35/45)%` 真實傷害。
//! 不吃 cooldown / mana；被動不走 `execute`（被 host CD/passive gate 擋）。

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, ROk, RResult, RSome, RStr, RString},
};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    types::{DamageKind, EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, TargetType,
};
use std::collections::HashMap;
use std::f32::consts::PI;

pub const ABILITY_ID: &str = "rain_iron_cannon";
const BUFF_ID: &str = "rain_iron_cannon_passive";

/// 各級真實傷害倍率（相對於攻擊者 final_atk）
const TRUE_DMG_PCT: [f32; 4] = [0.15, 0.25, 0.35, 0.45];
/// AoE 半徑（圓形）
const AOE_RADIUS: f32 = 150.0;
/// 扇形半角：45° → ±PI/4 → 全扇形 90°
const ARC_HALF_ANGLE_RAD: f32 = PI / 4.0;

pub struct RainIronCannonHandler;

impl AbilityScript for RainIronCannonHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_ID)
    }

    fn execute(
        &self,
        _caster: EntityHandle,
        _target: Target,
        _level: u8,
        _level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        // 被動技 — 不走主動施放；dispatch 的 Passive gate 會在 host 端直接擋下，
        // 但保留此 no-op 實作以防被錯誤呼叫。
        world.log_warn(RStr::from_str(
            "[rain_iron_cannon] passive ability cannot be cast actively",
        ));
        ROk(())
    }

    fn on_learn(
        &self,
        caster: EntityHandle,
        new_level: u8,
        world: &mut GameWorldDyn<'_>,
    ) {
        // 套一個永久可見 buff — payload 標等級，前端可據此渲染 icon / tooltip。
        // duration = f32::MAX 等同永久；BuffStore 端會以 infinite 顯示 remaining = -1。
        let modifiers = serde_json::json!({
            "visual_effect": "rain_iron_cannon_passive",
            "level": new_level,
            "true_damage_pct": TRUE_DMG_PCT[(new_level.saturating_sub(1) as usize).min(3)],
        });
        let s = modifiers.to_string();
        world.add_stat_buff(
            caster,
            RStr::from_str(BUFF_ID),
            f32::INFINITY,
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
        let lvl_idx = (level.saturating_sub(1) as usize).min(3);
        let pct = TRUE_DMG_PCT[lvl_idx];
        if pct <= 0.0 {
            return;
        }

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
        if dx * dx + dy * dy < 0.0001 {
            return; // 重疊無法決定方向
        }
        let base_angle = dy.atan2(dx);
        let atk = world.get_final_atk(attacker);
        let true_damage = atk * pct;
        if true_damage <= 0.0 {
            return;
        }

        // 用 victim 位置為扇形中心點做圓形查詢，再 per-unit 過濾角度
        let enemies = world.query_enemies_in_range(victim_pos, AOE_RADIUS, attacker);
        for enemy_h in enemies.iter().copied() {
            let epos = match world.get_pos(enemy_h) {
                RSome(p) => p,
                _ => continue,
            };
            let edx = epos.x - attacker_pos.x;
            let edy = epos.y - attacker_pos.y;
            if edx * edx + edy * edy < 0.0001 {
                continue;
            }
            let enemy_angle = edy.atan2(edx);
            let mut diff = enemy_angle - base_angle;
            while diff > PI {
                diff -= 2.0 * PI;
            }
            while diff < -PI {
                diff += 2.0 * PI;
            }
            if diff.abs() <= ARC_HALF_ANGLE_RAD {
                world.deal_damage(
                    enemy_h,
                    true_damage,
                    DamageKind::Pure,
                    RSome(attacker),
                );
            }
        }
    }
}

pub fn rain_iron_cannon_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "true_damage_pct".into(),
            serde_json::json!(TRUE_DMG_PCT[(lvl - 1) as usize]),
        );
        extra.insert("aoe_radius".into(), serde_json::json!(AOE_RADIUS));
        extra.insert("arc_degrees".into(), serde_json::json!(90.0)); // 半角 45° → 全角 90°
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 0.0,
                mana_cost: 0.0,
                cast_time: 0.0,
                range: 0.0,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "雨鐵炮".into(),
        description: "被動：普攻命中時以受擊點為中心、朝攻擊方向 90° 扇形 150 半徑對範圍內所有單位造成 15/25/35/45% 攻擊力的真實傷害。".into(),
        ability_type: AbilityType::Passive,
        target_type: TargetType::None,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: Vec::new(),
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn rain_iron_cannon_ffi() -> AbilityDefFFI {
    let def = rain_iron_cannon_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(RainIronCannonHandler, TD_Opaque),
    }
}
