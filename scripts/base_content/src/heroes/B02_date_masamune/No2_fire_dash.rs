//! 踏火無痕（fire_dash）— 伊達政宗的 E：往目標點衝刺，沿路對敵人造成持續傷害。

use abi_stable::std_types::{RNone, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityLevelData, DamageType, EffectSpec, TargetSelector,
};
use omoba_template_ids::{ABILITY_FIRE_DASH, ABILITY_FIRE_DASH_CONST};

use crate::ability_builder::{build_ability_ffi, extra_at};

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
        let level_data: AbilityLevelData = serde_json::from_str(level_data_json.as_str())
            .unwrap_or_default();
        let get_f = |k: &str, dft: f32| {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(dft)
        };
        let damage_per_tick = get_f("damage_per_tick", extra_at(&ABILITY_FIRE_DASH_CONST, "damage_per_tick", level));
        let dash_duration = get_f("dash_duration", extra_at(&ABILITY_FIRE_DASH_CONST, "dash_duration", level));
        let dash_width = get_f("dash_width", extra_at(&ABILITY_FIRE_DASH_CONST, "dash_width", level));
        let tick_interval = 0.1f32;
        let total_damage = damage_per_tick * (dash_duration / tick_interval);

        let dest = match target {
            Target::Point(p) => p,
            _ => {
                world.log_warn(RStr::from_str("[fire_dash] missing target point — abort"));
                return ROk(());
            }
        };

        world.set_pos(caster, dest);
        let enemies = world.query_enemies_in_range(dest, dash_width / 2.0, caster);
        for victim in enemies.iter().copied() {
            world.deal_damage(victim, total_damage, DamageKind::Magical, RNone);
        }
        ROk(())
    }
}

pub fn fire_dash_ffi() -> AbilityDefFFI {
    let dmg_lv1 = extra_at(&ABILITY_FIRE_DASH_CONST, "damage_per_tick", 1)
        * (extra_at(&ABILITY_FIRE_DASH_CONST, "dash_duration", 1) / 0.1);
    let radius = extra_at(&ABILITY_FIRE_DASH_CONST, "dash_width", 1) / 2.0;
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
