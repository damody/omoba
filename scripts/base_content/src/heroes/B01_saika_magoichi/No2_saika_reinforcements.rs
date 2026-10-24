//! 雜賀眾（saika_reinforcements）— 雜賀孫市的 E 技能：召喚雜賀鐵炮兵。
//!
//! 數值（summon_count / duration / 陣形 spacing 等）由 templates.lua
//! `abilities[saika_reinforcements].extras` 提供，runtime 透過
//! active runtime content lookup 取得。

use abi_stable::std_types::{RErr, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{EntityHandle, Fixed64, Target, Vec2},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::EffectSpec;
use omoba_template_ids::{ABILITY_SAIKA_REINFORCEMENTS, SUMMON_SAIKA_GUNNER};

use crate::ability_builder::{build_ability_ffi, extra_at_id, extra_at_id_f32};

pub struct SaikaReinforcementsHandler;

impl AbilityScript for SaikaReinforcementsHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_SAIKA_REINFORCEMENTS.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let rows=omoba_template_ids::active_ability_const(ABILITY_SAIKA_REINFORCEMENTS)
            .map(|content|content.facing_formation_rows).unwrap_or(0);
        let spec = match crate::summon_formation::resolve(ABILITY_SAIKA_REINFORCEMENTS, level, level_data_json.as_str(), rows) {
            Ok(spec) => spec, Err(error) => return RErr(error.into()),
        };
        // Target compatibility is unchanged: this authored formation follows caster facing.
        let _ = target;
        if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp| hp > Fixed64::ZERO) {
            return RErr("formation requires living owner".into());
        }
        let Some(caster_pos) = world.get_pos(caster).into_option() else {return RErr("formation owner has no position".into());};
        let facing = world.get_facing(caster);
        let positions = match spec.positions(caster_pos, facing) {
            Ok(positions) => positions, Err(error) => return RErr(error.into()),
        };
        for position in positions {
            if world.spawn_summoned_unit(position, SUMMON_SAIKA_GUNNER.as_str().into(), caster, spec.duration) == EntityHandle::INVALID {
                // The host managed-cast transaction discards the complete request group.
                return RErr("summon creation rejected".into());
            }
        }
        let (rows,cols,duration)=(spec.rows,spec.count/spec.rows,spec.duration);

        world.log_info(
            RString::from(format!(
                "[saika_reinforcements] summoned {}x{} gunners in front of caster at ({:.1},{:.1}) facing_ticks={} dur={}",
                rows,
                cols,
                caster_pos.x.to_f32_for_render(),
                caster_pos.y.to_f32_for_render(),
                facing.ticks(),
                duration.to_f32_for_render(),
            ))
            .as_rstr(),
        );
        ROk(())
    }
}

pub fn saika_reinforcements_ffi() -> AbilityDefFFI {
    // Lv1 預覽：召喚 2 隻、duration 45s
    let count_lv1 =
        extra_at_id(ABILITY_SAIKA_REINFORCEMENTS, "summon_count", 1).to_f32_for_render() as u32;
    let duration_lv1 = extra_at_id_f32(ABILITY_SAIKA_REINFORCEMENTS, "duration", 1);
    let effects_preview = vec![EffectSpec::Summon {
        unit_type: SUMMON_SAIKA_GUNNER.as_str().into(),
        count: count_lv1,
        duration: Some(duration_lv1),
    }];
    build_ability_ffi(
        ABILITY_SAIKA_REINFORCEMENTS,
        SaikaReinforcementsHandler,
        effects_preview,
    )
}
