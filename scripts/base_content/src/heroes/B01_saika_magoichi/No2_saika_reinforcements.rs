//! 雜賀眾（saika_reinforcements）— 雜賀孫市的 E 技能：召喚雜賀鐵炮兵。
//!
//! 數值（summon_count / duration / 陣形 spacing 等）由 templates.json
//! `abilities[saika_reinforcements].extras` 提供，runtime 透過
//! `ABILITY_SAIKA_REINFORCEMENTS_CONST` 取得。

use abi_stable::std_types::{ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{EntityHandle, Fixed32, Target, Vec2},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, EffectSpec};
use omoba_template_ids::{
    ABILITY_SAIKA_REINFORCEMENTS, ABILITY_SAIKA_REINFORCEMENTS_CONST, SUMMON_SAIKA_GUNNER,
};

use crate::ability_builder::{build_ability_ffi, extra_at, extra_at_f32};

pub struct SaikaReinforcementsHandler;

impl AbilityScript for SaikaReinforcementsHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_SAIKA_REINFORCEMENTS.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        _level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData = match serde_json::from_str(level_data_json.as_str()) {
            Ok(d) => d,
            Err(_) => AbilityLevelData::default(),
        };
        // JSON extras still carry f32 (omoba_core::AbilityLevelData not yet migrated).
        // Read as f64 then convert to Fixed32 at the boundary.
        // TODO Phase 1[d]: omoba_core::ability_meta migrated → drop the f64 hop.
        let get_fx = |k: &str, dft: Fixed32| -> Fixed32 {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| Fixed32::from_raw((v * 1024.0) as i32))
                .unwrap_or(dft)
        };
        let get_count = |k: &str, dft: u64| -> u64 {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| v as u64)
                .unwrap_or(dft)
        };
        let count = get_count("summon_count", 2).max(2);
        let duration = get_fx("duration", Fixed32::from_i32(45));
        let row_spacing = get_fx("row_spacing", Fixed32::from_i32(60));
        let col_spacing = get_fx("col_spacing", Fixed32::from_i32(60));
        let front_row_distance = get_fx("front_row_distance", Fixed32::from_i32(120));

        // forward = 永遠用 caster 的 Facing（忽略 target，因前端每次按鍵都送滑鼠
        // 位置做 target_pos；這個 W 設計上是「以自身朝向往前方兩排召喚」）。
        let _ = target;
        let caster_pos = world
            .get_pos(caster)
            .into_option()
            .unwrap_or(Vec2::ZERO);
        let facing = world.get_facing(caster);

        let fwd_x = omoba_sim::trig::cos(facing);
        let fwd_y = omoba_sim::trig::sin(facing);
        let perp_x = -fwd_y;
        let perp_y = fwd_x;

        // 2 排 × cols 欄：count 必為偶數（保底 max(2)）；cols = count/2。
        let rows: u64 = 2;
        let cols = count / rows;
        // col_center = (cols - 1) / 2 — keep in Fixed32 for downstream offset math.
        let half = Fixed32::from_raw(512); // 0.5
        let col_center = Fixed32::from_i32((cols as i32) - 1) * half;

        let unit_type = RStr::from_str(SUMMON_SAIKA_GUNNER.as_str());
        for r in 0..rows {
            let row_dist = front_row_distance + Fixed32::from_i32(r as i32) * row_spacing;
            let base_x = caster_pos.x + fwd_x * row_dist;
            let base_y = caster_pos.y + fwd_y * row_dist;
            for c in 0..cols {
                let off = (Fixed32::from_i32(c as i32) - col_center) * col_spacing;
                let p = Vec2 {
                    x: base_x + perp_x * off,
                    y: base_y + perp_y * off,
                };
                world.spawn_summoned_unit(p, unit_type, caster, duration);
            }
        }

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
    let count_lv1 = extra_at(&ABILITY_SAIKA_REINFORCEMENTS_CONST, "summon_count", 1)
        .to_f32_for_render() as u32;
    let duration_lv1 = extra_at_f32(&ABILITY_SAIKA_REINFORCEMENTS_CONST, "duration", 1);
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
