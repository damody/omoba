//! 火焰強襲（flame_assault）— 伊達政宗的 R 大招：範圍傷害 + 暈眩。

use abi_stable::std_types::{RNone, ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    buff_ids::BuffId,
    types::{DamageKind, DamageProfile, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, DamageType, EffectSpec, TargetSelector};
use omoba_template_ids::ABILITY_FLAME_ASSAULT;

use crate::ability_builder::{build_ability_ffi, extra_at_id, extra_at_id_f32};

pub struct FlameAssaultHandler;

impl AbilityScript for FlameAssaultHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_FLAME_ASSAULT.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData =
            serde_json::from_str(level_data_json.as_str()).unwrap_or_default();
        // JSON 額外內容仍然是 f32 → 邊界處的固定 64 轉換。
        // 第 2 階段：omoba_core::AbilityLevelData 仍為 f32；第二階段 KCP 標籤返工中的重新設計。
        let get_fx = |k: &str, dft: Fixed64| -> Fixed64 {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| Fixed64::from_raw((v * 1024.0) as i64))
                .unwrap_or(dft)
        };
        let damage = get_fx(
            "damage",
            extra_at_id(ABILITY_FLAME_ASSAULT, "damage", level),
        );
        let stun_duration = get_fx(
            "stun_duration",
            extra_at_id(ABILITY_FLAME_ASSAULT, "stun_duration", level),
        );
        let radius = get_fx(
            "radius",
            extra_at_id(ABILITY_FLAME_ASSAULT, "radius", level),
        );

        let center = match target {
            Target::Point(p) => p,
            _ => {
                world.log_warn(RStr::from_str(
                    "[flame_assault] missing target point — abort",
                ));
                return ROk(());
            }
        };

        world.emit_explosion(center, radius, Fixed64::from_raw(204) /* 0.2 */);
        let enemies = world.query_enemies_in_range(center, radius, caster);
        let stun_buff = BuffId::Stun.as_rstr();
        for victim in enemies.iter().copied() {
            world.deal_damage(
                victim,
                damage,
                DamageKind::Magical,
                DamageProfile::FIRE,
                RSome(caster),
            );
            world.add_buff(victim, stun_buff, stun_duration);
        }
        ROk(())
    }
}

pub fn flame_assault_ffi() -> AbilityDefFFI {
    let radius_lv1 = extra_at_id_f32(ABILITY_FLAME_ASSAULT, "radius", 1);
    let dmg_lv1 = extra_at_id_f32(ABILITY_FLAME_ASSAULT, "damage", 1);
    let effects_preview = vec![EffectSpec::AreaEffect {
        radius: radius_lv1,
        duration: 0.2,
        tick_interval: 0.0,
        effects: vec![EffectSpec::Damage {
            target: TargetSelector::AllEnemies,
            amount: dmg_lv1,
            damage_type: DamageType::Magical,
        }],
    }];
    build_ability_ffi(ABILITY_FLAME_ASSAULT, FlameAssaultHandler, effects_preview)
}
