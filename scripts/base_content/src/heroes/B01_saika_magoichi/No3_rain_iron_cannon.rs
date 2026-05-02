//! 雨鐵炮（rain_iron_cannon）— 雜賀孫市的 R 位被動：普攻命中附加扇形 AoE 真實傷害。
//!
//! 學到後 `on_learn` 套永久可見 buff；`on_attack_hit` 以受擊點為中心、朝
//! attacker→victim 方向的扇形（半角 templates.json 給）內所有敵人造成
//! `atk × true_damage_pct[level]` 真實傷害。被動不走 `execute`。

use abi_stable::std_types::{ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_template_ids::{ABILITY_RAIN_IRON_CANNON, ABILITY_RAIN_IRON_CANNON_CONST};

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
        // Phase 1de.2: this payload is NOT consumed via BuffStore::sum_add (no
        // matching StatKey); it's a tooltip / visual-only marker. Keep f32
        // emission for the frontend tooltip path. If a future system reads
        // `true_damage_pct` numerically, switch to `.raw()`.
        let pct = extra_at(&ABILITY_RAIN_IRON_CANNON_CONST, "true_damage_pct", new_level)
            .to_f32_for_render();
        let modifiers = serde_json::json!({
            "visual_effect": "rain_iron_cannon_passive",
            "level": new_level,
            "true_damage_pct": pct,
        });
        let s = modifiers.to_string();
        // Permanent buff — Fixed64 duration uses near-MAX as "indefinite" sentinel
        // (BuffStore convention; passive never cleared via tick decrement).
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
        let pct = extra_at(&ABILITY_RAIN_IRON_CANNON_CONST, "true_damage_pct", level);
        if pct <= Fixed64::ZERO {
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
        // tiny-distance guard (was 0.0001 in f32; raw 1 ~ 0.001 in Fixed64)
        if dx * dx + dy * dy < Fixed64::from_raw(1) {
            return;
        }
        let base_angle = omoba_sim::trig::atan2(dy, dx);
        let atk = world.get_final_atk(attacker);
        let true_damage = atk * pct;
        if true_damage <= Fixed64::ZERO {
            return;
        }

        // arc_half is stored in templates.json as radians (legacy). Convert radians to ticks:
        // ticks_per_radian = TAU_TICKS / TAU = 4096 / (2π) ≈ 651.9.
        // Multiply by 652 (raw, so divide by 1024 implicitly) — but easier: half_arc_ticks =
        // raw_radian_value * (4096 / 2π). We approximate via tick math at the same precision.
        // raw value of arc_half is `radians * 1024`. ticks = radians * 4096 / (2π) =
        //                    raw * (4096 / (2π * 1024))  ≈ raw * 0.6366
        // Use i64 multiply: ticks ≈ raw * 4096 * 1000 / (6283 * 1024) — keep deterministic.
        // arc_half_ticks = arc_half.raw() * 4096 / (2π * 1024); 2π ≈ 6283/1000.
        let arc_half_ticks: i32 =
            ((arc_half.raw() as i64 * 4096 * 1000) / (6283 * 1024)) as i32;

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
            // Compute shortest signed tick difference modulo TAU (4096).
            const TAU_TICKS: i32 = 4096;
            let mut diff = enemy_angle.ticks() - base_angle.ticks();
            // Normalize to (-TAU/2, TAU/2]
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
                    RSome(attacker),
                );
            }
        }
    }
}

pub fn rain_iron_cannon_ffi() -> AbilityDefFFI {
    build_ability_ffi(ABILITY_RAIN_IRON_CANNON, RainIronCannonHandler, Vec::new())
}
