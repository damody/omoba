//! 踏火無痕（fire_dash）— 伊達政宗的 E：往目標點衝刺，沿路對敵人造成持續傷害。

use abi_stable::std_types::{RNone, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, DamageType, EffectSpec, TargetSelector};
use omoba_template_ids::{ABILITY_FIRE_DASH, ABILITY_FIRE_DASH_CONST};

use crate::ability_builder::{build_ability_ffi, extra_at, extra_at_f32};

pub struct FireDashHandler;

impl AbilityScript for FireDashHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_FIRE_DASH.as_str())
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
        let damage_per_tick = get_fx(
            "damage_per_tick",
            extra_at(&ABILITY_FIRE_DASH_CONST, "damage_per_tick", level),
        );
        let dash_duration = get_fx(
            "dash_duration",
            extra_at(&ABILITY_FIRE_DASH_CONST, "dash_duration", level),
        );
        let dash_width = get_fx(
            "dash_width",
            extra_at(&ABILITY_FIRE_DASH_CONST, "dash_width", level),
        );
        let tick_interval = Fixed64::from_raw(102); // 0.1 (≈ 102/1024)
                                                    // 總傷害=每次tick傷害*（dash_duration/tick_interval）
        let total_damage = damage_per_tick * (dash_duration / tick_interval);

        let dest = match target {
            Target::Point(p) => p,
            _ => {
                world.log_warn(RStr::from_str("[fire_dash] missing target point — abort"));
                return ROk(());
            }
        };

        world.set_pos(caster, dest);
        let half_width = dash_width / Fixed64::from_i32(2);
        let enemies = world.query_enemies_in_range(dest, half_width, caster);
        for victim in enemies.iter().copied() {
            world.deal_damage(victim, total_damage, DamageKind::Magical, RNone);
        }
        ROk(())
    }
}

pub fn fire_dash_ffi() -> AbilityDefFFI {
    // EffectSpec 預覽值保持 f32（omoba_core 未遷移）。
    let dpt = extra_at_f32(&ABILITY_FIRE_DASH_CONST, "damage_per_tick", 1);
    let dur = extra_at_f32(&ABILITY_FIRE_DASH_CONST, "dash_duration", 1);
    let dmg_lv1 = dpt * (dur / 0.1);
    let radius = extra_at_f32(&ABILITY_FIRE_DASH_CONST, "dash_width", 1) / 2.0;
    let effects_preview = vec![EffectSpec::Damage {
        target: TargetSelector::InRadius {
            center: vek::Vec2::new(0.0, 0.0),
            radius,
            enemy: true,
        },
        amount: dmg_lv1,
        damage_type: DamageType::Magical,
    }];
    build_ability_ffi(ABILITY_FIRE_DASH, FireDashHandler, effects_preview)
}
