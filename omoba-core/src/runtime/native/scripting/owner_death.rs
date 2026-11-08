//! v3 `on_owner_death` (an item active's event) and an item passive's `on_death`: the dying unit's
//! statuses that listen to [`ON_OWNER_DEATH`] (status_event mask; an item's marker) hear the death
//! while the unit is still whole, before the death settles.
//!
//! The death outcome settles in the same batch that deletes the body, and a hero's death takes
//! its hero variables, inventory and gold for the next life right then (moba_match
//! record_moba_death), so a status event queued for the next dispatch would find nothing. Here,
//! when a `Death` outcome comes up, the listening rows of the dying unit (in buff id order; a
//! broken unit's learned passives do not hear it) call `AbilityScript::on_status_event` with kind
//! [`OWNER_DEATH`] at once; their effects settle ahead of the death itself, so a variable kept
//! across death, an item removed or a status taken off is what the death records. `event.other`
//! is invalid (the killer is not tracked here). Only effects are kept, like any status event's:
//! a hook that charges, casts or spawns has its batch dropped.
use abi_stable::sabi_trait::prelude::TD_Opaque;
use abi_stable::std_types::RStr;
use abi_stable::RMut;
use omb_script_abi::types::EntityHandle;
use omb_script_abi::world::{GameWorldDyn, GameWorld_TO};
use omoba_sim::Fixed64;
use specs::{Entity, World};
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
use super::registry::ScriptRegistry;
use crate::comp::Outcome;
use crate::runtime::native::status_event::{Spec, ON_OWNER_DEATH, OWNER_DEATH};
use crate::runtime::BuffStore;

/// The effects of `dead`'s statuses hearing its death, in call order; empty when none listens or
/// the world has no scripts.
pub fn hear(world: &World, dead: Entity) -> Vec<Outcome> {
    let Some(registry) = world.try_fetch::<ScriptRegistry>() else { return Vec::new() };
    let rows: Vec<(String, Spec)> = {
        let Some(buffs) = world.try_fetch::<BuffStore>() else { return Vec::new() };
        let mut rows: Vec<(String, Spec)> = buffs.stat_rows(dead)
            .filter_map(|(id, b)| Spec::from_payload(&b.payload).filter(|s| s.mask & ON_OWNER_DEATH != 0).map(|s| (id.to_owned(), s)))
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    };
    if rows.is_empty() { return Vec::new(); }
    let seed = world.try_fetch::<crate::comp::MasterSeed>().map_or(0, |s| s.0);
    let tick = world.try_fetch::<crate::comp::Tick>().map_or(0, |t| t.0);
    let handle = ParallelWorldAdapter::entity_to_handle(dead);
    let cache = ParallelAdapterCache::new(world, seed);
    let mut accepted = Vec::new();
    for (ordinal, (row, spec)) in rows.into_iter().enumerate() {
        let Some((_, script)) = registry.get_ability(&spec.ability) else { continue };
        let event = spec.event(&row, OWNER_DEATH, EntityHandle::INVALID, Fixed64::ZERO, 0);
        let random_ordinal = (tick << 20) ^ (u64::from(dead.id()) << 8) ^ (0x80 | ordinal as u64);
        let mut adapter = ParallelWorldAdapter::new_with_random_ordinal(&cache, dead, random_ordinal);
        let mut world_dyn: GameWorldDyn<'_> = GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
        let heard = catch_unwind(AssertUnwindSafe(|| script.on_status_event(handle, RStr::from_str(&spec.status), event, &mut world_dyn)));
        drop(world_dyn);
        if heard.is_err() {
            log::error!("[scripting] panic in AbilityScript::on_status_event (owner death) of {}", spec.ability);
            adapter.discard_cast();
            continue;
        }
        let outcomes = adapter.finish();
        if outcomes.iter().all(crate::runtime::native::ability_channel::pulse_outcome_allowed) { accepted.extend(outcomes); }
        else { log::warn!("[scripting] owner death hook of '{}' ({}) tried to charge or spawn; dropped", spec.status, spec.ability); }
    }
    accepted
}
