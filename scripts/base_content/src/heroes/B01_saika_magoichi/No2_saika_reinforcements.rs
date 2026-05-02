//! 雜賀眾（saika_reinforcements）— 雜賀孫市的 E 技能：召喚雜賀鐵炮兵。
//!
//! 數值（summon_count / duration / 陣形 spacing 等）由 templates.json
//! `abilities[saika_reinforcements].extras` 提供，runtime 透過
//! `ABILITY_SAIKA_REINFORCEMENTS_CONST` 取得。

use abi_stable::std_types::{ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{EntityHandle, Target, Vec2f},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, EffectSpec};
use omoba_template_ids::{
    ABILITY_SAIKA_REINFORCEMENTS, ABILITY_SAIKA_REINFORCEMENTS_CONST, SUMMON_SAIKA_GUNNER,
};

use crate::ability_builder::{build_ability_ffi, extra_at};

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
        let get_f = |k: &str, dft: f32| {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(dft)
        };
        let count = get_f("summon_count", 2.0).max(2.0) as u64;
        let duration = get_f("duration", 45.0);
        let row_spacing = get_f("row_spacing", 60.0);
        let col_spacing = get_f("col_spacing", 60.0);
        let front_row_distance = get_f("front_row_distance", 120.0);

        // forward = 永遠用 caster 的 Facing（忽略 target，因前端每次按鍵都送滑鼠
        // 位置做 target_pos；這個 W 設計上是「以自身朝向往前方兩排召喚」）。
        let _ = target;
        let caster_pos = world
            .get_pos(caster)
            .into_option()
            .unwrap_or(Vec2f::new(0.0, 0.0));
        let facing_rad = world.get_facing(caster);

        let fwd_x = facing_rad.cos();
        let fwd_y = facing_rad.sin();
        let perp_x = -fwd_y;
        let perp_y = fwd_x;

        // 2 排 × cols 欄：count 必為偶數（保底 max(2)）；cols = count/2。
        let rows = 2u64;
        let cols = count / rows;
        let col_center = (cols as f32 - 1.0) * 0.5;

        let unit_type = RStr::from_str(SUMMON_SAIKA_GUNNER.as_str());
        for r in 0..rows {
            let row_dist = front_row_distance + (r as f32) * row_spacing;
            let base_x = caster_pos.x + fwd_x * row_dist;
            let base_y = caster_pos.y + fwd_y * row_dist;
            for c in 0..cols {
                let off = (c as f32 - col_center) * col_spacing;
                let p = Vec2f::new(base_x + perp_x * off, base_y + perp_y * off);
                world.spawn_summoned_unit(p, unit_type, caster, duration);
            }
        }

        world.log_info(
            RString::from(format!(
                "[saika_reinforcements] summoned {}x{} gunners in front of caster at ({:.1},{:.1}) facing={:.2}rad dur={}",
                rows, cols, caster_pos.x, caster_pos.y, facing_rad, duration
            ))
            .as_rstr(),
        );
        ROk(())
    }
}

pub fn saika_reinforcements_ffi() -> AbilityDefFFI {
    // Lv1 預覽：召喚 2 隻、duration 45s
    let count_lv1 = extra_at(&ABILITY_SAIKA_REINFORCEMENTS_CONST, "summon_count", 1) as u32;
    let duration_lv1 = extra_at(&ABILITY_SAIKA_REINFORCEMENTS_CONST, "duration", 1);
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
