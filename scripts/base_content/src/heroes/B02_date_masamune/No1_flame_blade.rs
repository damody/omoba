//! 火焰刀（flame_blade）— 伊達政宗的 W：對單體目標或目標點範圍造成火焰傷害。

use abi_stable::std_types::{RNone, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityLevelData, DamageType, EffectSpec, TargetSelector,
};
use omoba_template_ids::{ABILITY_FLAME_BLADE, ABILITY_FLAME_BLADE_CONST};

use crate::ability_builder::{build_ability_ffi, extra_at, extra_at_f32};

pub struct FlameBladeHandler;

impl AbilityScript for FlameBladeHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_FLAME_BLADE.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData = serde_json::from_str(level_data_json.as_str())
            .unwrap_or_default();
        // JSON extras 仍然是 f32 → 邊界處的固定 64 轉換。
        // 第 2 階段：omoba_core::AbilityLevelData 仍為 f32；第二階段 KCP 標籤返工中的重新設計。
        let damage = level_data
            .extra
            .get("damage")
            .and_then(|v| v.as_f64())
            .map(|v| Fixed64::from_raw((v * 1024.0) as i64))
            .unwrap_or_else(|| extra_at(&ABILITY_FLAME_BLADE_CONST, "damage", level));
        let swipe_radius = level_data
            .extra
            .get("swipe_radius")
            .and_then(|v| v.as_f64())
            .map(|v| Fixed64::from_raw((v * 1024.0) as i64))
            .unwrap_or_else(|| extra_at(&ABILITY_FLAME_BLADE_CONST, "swipe_radius", level));

        match target {
            Target::Entity(victim) => {
                world.deal_damage(victim, damage, DamageKind::Magical, RNone);
            }
            Target::Point(p) => {
                world.emit_explosion(p, swipe_radius, Fixed64::from_raw(204) /* 0.2 */);
                let enemies = world.query_enemies_in_range(p, swipe_radius, caster);
                for victim in enemies.iter().copied() {
                    world.deal_damage(victim, damage, DamageKind::Magical, RNone);
                }
            }
            Target::None => {
                world.log_warn(RStr::from_str("[flame_blade] no target — abort"));
            }
        }
        ROk(())
    }
}

pub fn flame_blade_ffi() -> AbilityDefFFI {
    let dmg_lv1 = extra_at_f32(&ABILITY_FLAME_BLADE_CONST, "damage", 1);
    let effects_preview = vec![EffectSpec::Damage {
        target: TargetSelector::Target,
        amount: dmg_lv1,
        damage_type: DamageType::Magical,
    }];
    build_ability_ffi(ABILITY_FLAME_BLADE, FlameBladeHandler, effects_preview)
}
