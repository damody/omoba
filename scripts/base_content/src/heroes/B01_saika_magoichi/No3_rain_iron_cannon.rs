//! 雨鐵炮（rain_iron_cannon）— 雜賀孫市的 R 位被動：普攻命中附加扇形 AoE 真實傷害。
//!
//! 學到後 `on_learn` 套永久可見 buff；`on_attack_hit` 以受擊點為中心、朝
//! attacker→victim 方向的扇形（半角 templates.json 給）內所有敵人造成
//! `atk × true_damage_pct[level]` 真實傷害。被動不走 `execute`。

use abi_stable::std_types::{ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_template_ids::{ABILITY_RAIN_IRON_CANNON, ABILITY_RAIN_IRON_CANNON_CONST};
use std::f32::consts::PI;

use crate::ability_builder::{build_ability_ffi, extra_at};

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

    fn on_learn(
        &self,
        caster: EntityHandle,
        new_level: u8,
        world: &mut GameWorldDyn<'_>,
    ) {
        // 永久可見 buff — payload 標等級，前端可據此渲染 icon / tooltip。
        let pct = extra_at(&ABILITY_RAIN_IRON_CANNON_CONST, "true_damage_pct", new_level);
        let modifiers = serde_json::json!({
            "visual_effect": "rain_iron_cannon_passive",
            "level": new_level,
            "true_damage_pct": pct,
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
        let pct = extra_at(&ABILITY_RAIN_IRON_CANNON_CONST, "true_damage_pct", level);
        if pct <= 0.0 {
            return;
        }
        let aoe_radius = extra_at(&ABILITY_RAIN_IRON_CANNON_CONST, "aoe_radius", level);
        let arc_half = extra_at(&ABILITY_RAIN_IRON_CANNON_CONST, "arc_half_angle_rad", level);

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
            return;
        }
        let base_angle = dy.atan2(dx);
        let atk = world.get_final_atk(attacker);
        let true_damage = atk * pct;
        if true_damage <= 0.0 {
            return;
        }

        let enemies = world.query_enemies_in_range(victim_pos, aoe_radius, attacker);
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
            if diff.abs() <= arc_half {
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

pub fn rain_iron_cannon_ffi() -> AbilityDefFFI {
    build_ability_ffi(ABILITY_RAIN_IRON_CANNON, RainIronCannonHandler, Vec::new())
}
