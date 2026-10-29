//! a13_chain_sickle（A13E 忍法．勾鎖鐮）：手寫的 hook（D7，m3-rust-codegen-design.md §2.4）。
//!
//! 為什麼不能共用：全遊戲只有這招是「沿路徑飛、命中第一個單位後沿原路拉回」的鉤子；投射物
//! 元件沒有路徑與拉回（hook-candidates.md 複核）。行為與預期結果寫在 v3 `hooks.a13_chain_sickle_hook`。
//!
//! 實作：用排程逐步推進，狀態全在排程參數裡，DLL 不保存任何東西。
//! - 飛行（step 1）：每步沿施法者到指定點的方向前進 `hook_step`（原作每次 think 前進 10），
//!   過了 `arm_distance` 才判定；鉤頭 `hook_width` 範圍內第一個敵人（依 id）受到 `hook_damage` 物理傷害、
//!   帶 `a13e_hook_back`，轉入拉回；飛滿 `hook_distance` 或施法者死亡就結束。
//! - 拉回（step 2）：每步把目標往擲出點移動 `hook_step`；到點或 `a13e_hook_back` 結束時加
//!   `a13e_phased`、移除 `a13e_hook_back`。
//! 尚未做到（等工作線 B 的讀取延伸點）：鉤到友方（kit 目前只能選敵人）、雙點 vector 指定
//! （host 只有單點，轉折點取施法方向的延長線）。
use super::*;

pub struct Impl;

const FLY: u16 = 1;
const PULL: u16 = 2;

fn val(r: &ability_kit::Ranked<4>, cx: &Cx<'_, '_>) -> Fixed64 { r.at(cx.rank) }

impl Hook for Impl {
    fn on_cast(cx: &mut Cx<'_, '_>) -> Res {
        use super::super::values::a13_chain_sickle as v;
        let origin = cx.pos(cx.caster)?;
        let toward = cx.target_point()?;
        let way = ability_kit::direction(origin, toward);
        if way == Vec2::ZERO { return Err("v3: a13_chain_sickle needs a direction"); }
        let caster = cx.caster;
        cx.apply_status(caster, &super::super::ids::status::A13E_FOLLOWTHROUGH, val(&v::FOLLOWTHROUGH, cx), &[])?;
        let args = [ability_kit::arg_point(origin), ability_kit::arg_point(way), ability_kit::arg_num(Fixed64::ZERO)];
        cx.schedule(caster, ID, FLY, val(&v::HOOK_THINK, cx), &args)?;
        Ok(())
    }

    fn on_scheduled(cx: &mut Cx<'_, '_>, at: Scheduled, args: &[ScheduleArg]) -> Res {
        use super::super::values::a13_chain_sickle as v;
        match at.step {
            FLY => {
                let caster = cx.caster;
                if !cx.alive(caster) { return Ok(()); }
                let origin = ability_kit::arg_as_point(args, 0);
                let way = ability_kit::arg_as_point(args, 1);
                let flown = ability_kit::arg_as_num(args, 2) + val(&v::HOOK_STEP, cx);
                let head = ability_kit::offset(origin, way, flown);
                if flown > val(&v::ARM_DISTANCE, cx) {
                    let select = Select { types: UnitTypes::NonBuilding, include_magic_immune: true, include_invulnerable: false };
                    // v3 `radius = $hook_width`
                    if let Some(hit) = cx.enemies_in_circle(head, val(&v::HOOK_WIDTH, cx), select).into_iter().find(|u| *u != caster) {
                        cx.damage(hit, val(&v::HOOK_DAMAGE, cx), DamageKind::Physical,
                            Dmg { require_source_alive: false, no_amplify: false })?;
                        cx.apply_status(hit, &super::super::ids::status::A13E_HOOK_BACK, val(&v::PULL_MAX_TIME, cx), &[])?;
                        let pull = [ability_kit::arg_unit(hit), ability_kit::arg_point(origin)];
                        cx.schedule(caster, ID, PULL, val(&v::HOOK_THINK, cx), &pull)?;
                        return Ok(());
                    }
                }
                if flown < val(&v::HOOK_DISTANCE, cx) {
                    let next = [ability_kit::arg_point(origin), ability_kit::arg_point(way), ability_kit::arg_num(flown)];
                    cx.schedule(caster, ID, FLY, val(&v::HOOK_THINK, cx), &next)?;
                }
                Ok(())
            }
            PULL => {
                let hit = ability_kit::arg_as_unit(args, 0);
                let origin = ability_kit::arg_as_point(args, 1);
                if !cx.alive(hit) { return Ok(()); }
                let at = cx.pos(hit)?;
                let left = ability_kit::distance(at, origin);
                let held = cx.has_status(hit, &super::super::ids::status::A13E_HOOK_BACK);
                let step = val(&v::HOOK_STEP, cx);
                if !held || left <= step {
                    if held { cx.teleport(hit, origin, None)?; }
                    cx.apply_status(hit, &super::super::ids::status::A13E_PHASED, val(&v::PHASE_TIME, cx), &[])?;
                    cx.remove_status(hit, &super::super::ids::status::A13E_HOOK_BACK)?;
                    return Ok(());
                }
                cx.teleport(hit, ability_kit::offset(at, ability_kit::direction(at, origin), step), None)?;
                let caster = cx.caster;
                cx.schedule(caster, ID, PULL, val(&v::HOOK_THINK, cx), args)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
