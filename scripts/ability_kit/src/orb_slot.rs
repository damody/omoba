//! v3 `passive.orb_slot` and `orb_slot_free` (E534, 原作 the shared orb slot `nobuorb1`: B15
//! 今川義元's flame arrows and the orb items take turns). A hero has one orb slot; its owner is the
//! ability or item that last claimed it, kept as the hero's hidden row `__orb_slot` (payload: the
//! owner id) in the host's `BuffStore` like every marker (the DLL keeps nothing). A claim
//! overwrites any owner; a release clears the slot only while the releaser still owns it; a
//! source's orb effect applies only while the slot is free or its own.
use crate::codegen::{Cx, Res};
use abi_stable::std_types::RStr;
use omb_script_abi::types::EntityHandle;

/// Buff id of a hero's orb slot row.
pub const BUFF: &str = "__orb_slot";
const KEY: &str = "__orb_slot_v1";

impl Cx<'_, '_> {
    /// The current owner of `unit`'s orb slot, if any.
    pub fn orb_slot_owner(&self, unit: EntityHandle) -> Option<String> {
        let payload = self.world.get_buff_payload(unit, RStr::from_str(BUFF)).into_option()?;
        let value: serde_json::Value = serde_json::from_str(payload.as_str()).ok()?;
        value.get(KEY)?.get("owner")?.as_str().map(str::to_owned)
    }

    /// `owner` (an ability or item id) takes `unit`'s orb slot (`orb_slot.claim`).
    pub fn orb_slot_claim(&mut self, unit: EntityHandle, owner: &str) -> Res {
        if !unit.is_valid() || owner.is_empty() || owner.len() > 64 { return Ok(()); }
        let payload = serde_json::json!({ (KEY): { "owner": owner } });
        self.world.add_stat_buff(unit, RStr::from_str(BUFF), Self::FOREVER, RStr::from_str(&payload.to_string()));
        Ok(())
    }

    /// `owner` gives `unit`'s orb slot back (`orb_slot.release`): only while it still owns it.
    pub fn orb_slot_release(&mut self, unit: EntityHandle, owner: &str) -> Res {
        if self.orb_slot_owner(unit).as_deref() == Some(owner) { self.world.remove_buff(unit, RStr::from_str(BUFF)); }
        Ok(())
    }

    /// v3 `orb_slot_free(unit, ability)`: the slot is empty or `owner`'s.
    pub fn orb_slot_free(&self, unit: EntityHandle, owner: &str) -> bool {
        self.orb_slot_owner(unit).is_none_or(|current| current == owner)
    }
}
