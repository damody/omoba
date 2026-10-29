//! status_damage_pre：v3 `on_deal_damage_pre`／`on_take_damage_pre`——傷害落地前，攻擊方與受傷方
//! 身上的狀態可以改傷害量或取消（rules::damage 的減免前事件，接在法術增幅後面）。
//!
//! A status the generated code applies may listen to [`ON_DEAL_DAMAGE_PRE`] /
//! [`ON_TAKE_DAMAGE_PRE`] (status_event mask). For every packet, the source's listening rows and
//! then the target's, each in buff id order, call the owning ability's
//! `AbilityScript::on_damage_pre(holder, status, event)` synchronously; each returned
//! [`DamageEdit`] applies to the packet before the next call sees it. Effects the hook applies
//! (a heal, a state write) settle with the next outcome batch, like a delayed effect's (no
//! charge, cast or spawn). The script registry is reachable here only while the outcome batch
//! settles ([`with_scripts`]); without it (a world with no scripts) nothing listens.
use abi_stable::sabi_trait::prelude::TD_Opaque;
use abi_stable::std_types::RStr;
use abi_stable::RMut;
use omb_script_abi::types::DamageEdit;
use omb_script_abi::world::{GameWorldDyn, GameWorld_TO};
use omoba_sim::Fixed64;
use specs::{World, WorldExt};
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
use super::registry::ScriptRegistry;
use crate::runtime::native::rules::damage::Packet;
use crate::runtime::native::status_event::{self as se, Spec};
use crate::runtime::BuffStore;

pub use crate::runtime::native::status_event::{ON_DEAL_DAMAGE_PRE, ON_TAKE_DAMAGE_PRE};

/// Run `f` (an outcome batch) with `scripts` in the world, so the damage pipeline can reach the
/// abilities' `on_damage_pre`; the registry goes back to the caller afterwards.
pub fn with_scripts<T>(world: &mut World, scripts: &mut ScriptRegistry, f: impl FnOnce(&mut World) -> T) -> T {
    world.insert(std::mem::take(scripts));
    let out = f(world);
    *scripts = world.remove::<ScriptRegistry>().unwrap_or_default();
    out
}

/// The listening rows of `holder` for `bit`, in buff id order.
fn rows(buffs: &BuffStore, holder: specs::Entity, bit: u32) -> Vec<Spec> {
    if !buffs.entity_has_key(holder, se::KEY) { return Vec::new(); }
    let mut rows: Vec<(String, Spec)> = buffs.iter_for(holder)
        .filter_map(|(id, b)| Spec::from_payload(&b.payload).filter(|s| s.mask & bit != 0).map(|s| (id.to_owned(), s)))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter().map(|(_, s)| s).collect()
}

fn total(packet: &Packet) -> Fixed64 { packet.physical + packet.magical + packet.pure }

/// `DamageEdit` on the packet: `mode` 1 multiplies every kind by `value`, 2 scales them so the
/// total is `value` (never more than it was), 3 cancels; anything else keeps it.
pub fn apply_edit(packet: &mut Packet, edit: DamageEdit) {
    let scale = |packet: &mut Packet, factor: Fixed64| {
        packet.physical = packet.physical * factor;
        packet.magical = packet.magical * factor;
        packet.pure = packet.pure * factor;
    };
    match edit.mode {
        DamageEdit::MULTIPLY => scale(packet, edit.value.max(Fixed64::ZERO)),
        DamageEdit::SET => {
            let before = total(packet);
            let target = edit.value.max(Fixed64::ZERO);
            if before > Fixed64::ZERO && target < before {
                let (p, m) = (packet.physical, packet.magical);
                packet.physical = Fixed64::from_raw((i128::from(p.raw()) * i128::from(target.raw()) / i128::from(before.raw())) as i64);
                packet.magical = Fixed64::from_raw((i128::from(m.raw()) * i128::from(target.raw()) / i128::from(before.raw())) as i64);
                packet.pure = (target - packet.physical - packet.magical).max(Fixed64::ZERO);
            }
        }
        DamageEdit::CANCEL => scale(packet, Fixed64::ZERO),
        _ => {}
    }
}

/// The rules::damage stage (before mitigation, after spell amplification).
pub fn status_damage_pre(world: &World, packet: &mut Packet) {
    let Some(registry) = world.try_fetch::<ScriptRegistry>() else { return };
    let calls: Vec<(specs::Entity, Spec, u8)> = {
        let buffs = world.read_resource::<BuffStore>();
        let mut calls: Vec<_> = rows(&buffs, packet.source, ON_DEAL_DAMAGE_PRE).into_iter().map(|s| (packet.source, s, se::DEAL_DAMAGE_PRE)).collect();
        calls.extend(rows(&buffs, packet.target, ON_TAKE_DAMAGE_PRE).into_iter().map(|s| (packet.target, s, se::TAKE_DAMAGE_PRE)));
        calls
    };
    if calls.is_empty() || total(packet) <= Fixed64::ZERO { return; }
    let seed = world.try_fetch::<crate::comp::MasterSeed>().map_or(0, |s| s.0);
    let tick = world.try_fetch::<crate::comp::Tick>().map_or(0, |t| t.0);
    let handle = |e: specs::Entity| ParallelWorldAdapter::entity_to_handle(e);
    let mut accepted = Vec::new();
    {
        let cache = ParallelAdapterCache::new(world, seed);
        for (ordinal, (holder, spec, kind)) in calls.into_iter().enumerate() {
            if total(packet) <= Fixed64::ZERO { break; }
            let Some((_, script)) = registry.get_ability(&spec.ability) else { continue };
            let other = if kind == se::DEAL_DAMAGE_PRE { packet.target } else { packet.source };
            let damage_kind = if packet.physical > Fixed64::ZERO { 0 } else if packet.magical > Fixed64::ZERO { 1 } else { 2 };
            let event = spec.event(kind, handle(other), total(packet), damage_kind);
            let random_ordinal = (tick << 20) ^ (u64::from(packet.target.id()) << 8) ^ ordinal as u64;
            let mut adapter = ParallelWorldAdapter::new_with_random_ordinal(&cache, holder, random_ordinal);
            let mut world_dyn: GameWorldDyn<'_> = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
            let edit = catch_unwind(AssertUnwindSafe(|| script.on_damage_pre(handle(holder), RStr::from_str(&spec.status), event, &mut world_dyn)));
            drop(world_dyn);
            let Ok(edit) = edit else {
                log::error!("[scripting] panic in AbilityScript::on_damage_pre of {}", spec.ability);
                adapter.discard_cast();
                continue;
            };
            let outcomes = adapter.finish();
            if outcomes.iter().all(crate::runtime::native::ability_channel::pulse_outcome_allowed) {
                accepted.extend(outcomes);
            } else {
                log::warn!("[scripting] damage hook of '{}' ({}) tried to charge or spawn; its effects dropped", spec.status, spec.ability);
            }
            apply_edit(packet, edit);
        }
    }
    if !accepted.is_empty() {
        world.write_resource::<Vec<crate::comp::Outcome>>().extend(accepted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn packet(p: i32, m: i32, r: i32) -> Packet {
        let mut world = World::new();
        let e = world.create_entity().build();
        Packet::new(e, e, Fixed64::from_i32(p), Fixed64::from_i32(m), Fixed64::from_i32(r), 1)
    }

    #[test]
    fn edits_multiply_set_or_cancel_every_kind() {
        let mut p = packet(100, 50, 10);
        apply_edit(&mut p, DamageEdit { mode: DamageEdit::MULTIPLY, value: Fixed64::from_raw(512) });
        assert_eq!((p.physical, p.magical, p.pure), (Fixed64::from_i32(50), Fixed64::from_i32(25), Fixed64::from_i32(5)));
        let mut p = packet(100, 50, 10);
        apply_edit(&mut p, DamageEdit { mode: DamageEdit::SET, value: Fixed64::from_i32(80) });
        assert_eq!(total(&p), Fixed64::from_i32(80), "the total becomes the value");
        assert_eq!(p.physical, Fixed64::from_i32(50), "split in proportion");
        let mut p = packet(100, 0, 0);
        apply_edit(&mut p, DamageEdit { mode: DamageEdit::SET, value: Fixed64::from_i32(150) });
        assert_eq!(total(&p), Fixed64::from_i32(100), "set never raises the damage");
        let mut p = packet(100, 50, 10);
        apply_edit(&mut p, DamageEdit { mode: DamageEdit::CANCEL, value: Fixed64::ZERO });
        assert_eq!(total(&p), Fixed64::ZERO);
        let mut p = packet(100, 50, 10);
        apply_edit(&mut p, DamageEdit::KEEP);
        assert_eq!(total(&p), Fixed64::from_i32(160));
    }
}
