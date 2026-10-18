//! Player-scoped persistent economy and a bounded recent receipt history.
use omoba_core::{
    game_proto::{
        InventorySlotPresentation, OwnerEconomyPresentation, ShopTransactionReceipt,
        TeamPublicEvent,
    },
    runtime::{
        native::economy_projection::OwnerEconomyState, shop_receipt::ShopReceipt, FactKind,
        FilteredRenderSnapshot,
    },
};
use std::collections::VecDeque;

pub fn owner_economy_state(snapshot: &FilteredRenderSnapshot, player_id: u32) -> Option<OwnerEconomyState> {
    if player_id == 0 || snapshot.team_id == 0 {
        return None;
    }
    let bytes = &snapshot
        .public_events
        .iter()
        .rev()
        .find(|event| {
            event.event_kind == FactKind::OwnerEconomy as u32
                && event.subject.is_none()
                && event.sanitized_payload.get(..4) == Some(player_id.to_le_bytes().as_slice())
        })?
        .sanitized_payload;
    OwnerEconomyState::decode(bytes).filter(|state| state.player_id == player_id)
}

pub fn economy(
    snapshot: &FilteredRenderSnapshot,
    player_id: u32,
) -> Option<OwnerEconomyPresentation> {
    let state = owner_economy_state(snapshot, player_id)?;
    Some(OwnerEconomyPresentation {
        schema_version: 3,
        player_id,
        gold: state.economy.gold,
        shop_reason: u32::from(state.shop_reason),
        slots: (0..6)
            .map(|slot| InventorySlotPresentation {
                slot: slot as u32,
                catalog_id: u32::from(state.economy.item_ids[slot]),
                cooldown_seconds: f32::from_bits(state.economy.cooldown_bits[slot]),
            })
            .collect(),
        shop_available: state.shop_available,
        shop_protocol_enabled: false,
        shield_remaining_raw: state.shield_remaining_raw,
    })
}

pub const MAX_RECENT_SHOP_RECEIPTS: usize = 64;
/// Admitted to transport. Not purchase success and not a wallet change.
pub const SHOP_TXN_PHASE_ADMITTED: u32 = 1;
/// The original request is still the one that may settle. Do not allocate another purchase.
pub const SHOP_TXN_PHASE_UNCERTAIN: u32 = 2;

/// Result of binding one renderer request to one authority input.
/// `Completed` does not consume pending capacity. `Conflict` leaves every row unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopRequestNote {
    Pending,
    Completed,
    Conflict,
    Capacity,
}
impl ShopRequestNote {
    pub fn keeps_pair(self) -> bool {
        matches!(self, Self::Pending | Self::Completed)
    }
}

#[derive(Clone, Copy)]
struct ShopTxnPending {
    epoch: u64,
    owner: u32,
    authority_input_id: u32,
    request_id: u64,
    action_kind: u32,
    phase: u32,
}

pub fn settled_inputs(
    frame: &omoba_core::game_proto::TeamTickFrame,
    player: u32,
) -> Vec<(u32, bool, &'static str)> {
    let Some(step) = &frame.step else {
        return vec![];
    };
    let mut results: Vec<_> = step
        .accepted_inputs
        .iter()
        .filter(|input| input.player_id == player && !matches!(input.action_kind, 17 | 18 | omoba_core::runtime::shop_undo_wire::SHOP_UNDO_ACTION_KIND))
        .filter_map(|input| {
            Some((
                u32::try_from(input.input_id).ok()?,
                true,
                "APPLIED_TO_PRESENTATION",
            ))
        })
        .collect();
    for event in &step.public_events {
        if event.event_kind != FactKind::ShopReceipt as u32 || event.subject.is_some() {
            continue;
        }
        let Some(receipt) = ShopReceipt::decode(&event.sanitized_payload) else {
            continue;
        };
        if receipt.player_id != player || receipt.tick != frame.replica_tick {
            continue;
        }
        let Ok(correlation) = u32::try_from(receipt.input_id) else {
            continue;
        };
        let name = receipt_result_name(receipt.result_code);
        results.push((correlation, receipt.result_code == 0, name));
    }
    for event in &step.public_events {
        if event.event_kind != FactKind::ShopUndoReceipt as u32 || event.subject.is_some() {
            continue;
        }
        let Some(receipt) = omoba_core::runtime::shop_undo_wire::ShopUndoReceipt::decode(&event.sanitized_payload) else {
            continue;
        };
        if receipt.player_id != player || receipt.tick != frame.replica_tick {
            continue;
        }
        let Ok(correlation) = u32::try_from(receipt.input_id) else { continue };
        results.push((correlation, receipt.result_code == 0, undo_result_name(receipt.result_code)));
    }
    results
}

pub fn undo_result_name(code: u32) -> &'static str {
    match code {
        0 => "UNDO_SETTLED",
        14 => "UNDO_UNAVAILABLE",
        15 => "UNDO_DUPLICATE",
        other => receipt_result_name(other),
    }
}

pub fn receipt_result_name(code: u32) -> &'static str {
    match code {
            0 => "SHOP_SETTLED",
            1 => "SHOP_UNKNOWN_ITEM",
            2 => "SHOP_INVALID_CATALOG",
            3 => "SHOP_INVALID_BALANCE",
            4 => "SHOP_MISSING_COMPONENT",
            5 => "SHOP_INVENTORY_FULL",
            6 => "SHOP_INSUFFICIENT_GOLD",
            7 => "SHOP_INVALID_SLOT",
            8 => "SHOP_BALANCE_OVERFLOW",
            9 => "SHOP_MATCH_UNAVAILABLE",
            10 => "SHOP_UNKNOWN_PLAYER",
            11 => "SHOP_HERO_UNAVAILABLE",
            12 => "SHOP_OUTSIDE_SHOP",
            13 => "SHOP_MISSING_COMPONENTS",
        _ => "SHOP_INVALID_RECEIPT",
    }
}
#[derive(Default)]
pub(crate) struct ShopReceiptHistory {
    epoch: Option<u64>,
    entries: VecDeque<ShopTransactionReceipt>,
    pending: VecDeque<ShopTxnPending>,
    pub(crate) evicted: u64,
    pub(crate) pending_refused: u64,
}
impl ShopReceiptHistory {
    fn adopt_epoch(&mut self, epoch: u64) {
        if self.epoch != Some(epoch) {
            self.entries.clear();
            self.pending.clear();
            self.epoch = Some(epoch);
        }
    }

    fn retire_completed(&mut self, owner: u32, authority_input_id: u32, action_kind: u32) {
        self.pending.retain(|item| {
            item.owner != owner || item.authority_input_id != authority_input_id || item.action_kind != action_kind
        });
    }

    /// Binds one renderer request to one authority input for this owner and view.
    /// Pending rows and retained receipts are both read before any write.
    /// An exact repeat is idempotent. A free unbound receipt binds immediately.
    /// A request or authority already owned by a different pair changes nothing.
    pub(crate) fn note_request(
        &mut self,
        epoch: u64,
        owner: u32,
        authority_input_id: u32,
        request_id: u64,
        action_kind: u32,
    ) -> ShopRequestNote {
        if owner == 0 || authority_input_id == 0 || request_id == 0 || !matches!(action_kind, 17 | 18) {
            return ShopRequestNote::Conflict;
        }
        self.adopt_epoch(epoch);
        let authority = u64::from(authority_input_id);
        let pending_authority = self.pending.iter().find(|item| {
            item.owner == owner && item.authority_input_id == authority_input_id
        }).copied();
        let pending_request = self.pending.iter().find(|item| {
            item.owner == owner && item.request_id == request_id
        }).copied();
        let receipt_authority = self.entries.iter().position(|row| {
            row.player_id == owner && row.input_id == authority
        });
        let receipt_request = self.entries.iter().position(|row| {
            row.player_id == owner && row.request_id == request_id
        });
        let authority_pending_conflict = match pending_authority {
            Some(item) => item.request_id != request_id || item.action_kind != action_kind,
            None => false,
        };
        let authority_receipt_conflict = match receipt_authority {
            Some(index) => {
                let row = &self.entries[index];
                row.action_kind != action_kind || (row.request_id != 0 && row.request_id != request_id)
            }
            None => false,
        };
        let other_pending_has_request = match pending_request {
            Some(item) => item.authority_input_id != authority_input_id,
            None => false,
        };
        let other_receipt_has_request = match receipt_request {
            Some(index) => self.entries[index].input_id != authority,
            None => false,
        };
        if authority_pending_conflict || authority_receipt_conflict || other_pending_has_request || other_receipt_has_request {
            return ShopRequestNote::Conflict;
        }
        if let Some(index) = receipt_authority {
            if self.entries[index].request_id == 0 {
                self.entries[index].request_id = request_id;
                self.entries[index].view_epoch = epoch;
            }
            self.retire_completed(owner, authority_input_id, action_kind);
            return ShopRequestNote::Completed;
        }
        if pending_authority.is_some() {
            return ShopRequestNote::Pending;
        }
        if self.pending.len() == MAX_RECENT_SHOP_RECEIPTS {
            self.pending_refused = self.pending_refused.saturating_add(1);
            log::warn!("shop transaction correlation is full; active pending was kept");
            return ShopRequestNote::Capacity;
        }
        self.pending.push_back(ShopTxnPending {
            epoch,
            owner,
            authority_input_id,
            request_id,
            action_kind,
            phase: SHOP_TXN_PHASE_ADMITTED,
        });
        ShopRequestNote::Pending
    }

    /// Marks the original request uncertain. It does not allocate a replacement purchase.
    pub(crate) fn note_uncertain(&mut self, epoch: u64, owner: u32, request_id: u64) -> bool {
        if owner == 0 || request_id == 0 || self.epoch != Some(epoch) {
            return false;
        }
        let Some(item) = self.pending.iter_mut().find(|item| item.owner == owner && item.request_id == request_id) else {
            return false;
        };
        item.phase = SHOP_TXN_PHASE_UNCERTAIN;
        true
    }

    pub(crate) fn note_uncertain_request(&mut self, request_id: u64) -> bool {
        if request_id == 0 {
            return false;
        }
        let Some(item) = self.pending.iter_mut().find(|item| item.request_id == request_id) else {
            return false;
        };
        item.phase = SHOP_TXN_PHASE_UNCERTAIN;
        true
    }

    /// Called on every applied frame, including discarded catch-up snapshots.
    pub(crate) fn capture(
        &mut self,
        epoch: u64,
        frame_tick: u64,
        player: u32,
        events: &[TeamPublicEvent],
    ) {
        self.adopt_epoch(epoch);
        for event in events.iter().filter(|event| {
            event.event_kind == FactKind::ShopReceipt as u32 && event.subject.is_none()
        }) {
            let Some(receipt) = ShopReceipt::decode(&event.sanitized_payload) else {
                continue;
            };
            if receipt.player_id != player || receipt.tick != frame_tick {
                continue;
            }
            let authority = u32::try_from(receipt.input_id).ok();
            let pending = authority.and_then(|id| {
                self.pending.iter().find(|item| item.owner == player && item.authority_input_id == id).copied()
            });
            if let Some(item) = pending {
                if item.action_kind != receipt.action_kind {
                    log::error!("conflicting shop receipt discarded");
                    continue;
                }
            }
            let request_id = pending.map(|item| item.request_id).unwrap_or(0);
            let receipt = ShopTransactionReceipt {
                schema_version: 1,
                player_id: receipt.player_id,
                input_id: receipt.input_id,
                settled_tick: receipt.tick,
                action_kind: receipt.action_kind,
                catalog_id: receipt.catalog_id,
                item_slot: receipt.slot,
                result_code: receipt.result_code,
                request_id,
                view_epoch: if request_id == 0 { 0 } else { epoch },
            };
            if let Some(index) = self.entries.iter().position(|old| old.input_id == receipt.input_id) {
                let previous_request = self.entries[index].request_id;
                let same = self.entries[index].schema_version == receipt.schema_version
                    && self.entries[index].player_id == receipt.player_id
                    && self.entries[index].settled_tick == receipt.settled_tick
                    && self.entries[index].action_kind == receipt.action_kind
                    && self.entries[index].catalog_id == receipt.catalog_id
                    && self.entries[index].item_slot == receipt.item_slot
                    && self.entries[index].result_code == receipt.result_code;
                let conflicts = !same
                    || (receipt.request_id != 0 && previous_request != 0 && receipt.request_id != previous_request);
                if conflicts {
                    log::error!("conflicting shop receipt discarded");
                    continue;
                }
                if previous_request == 0 && receipt.request_id != 0 {
                    self.entries[index].request_id = receipt.request_id;
                    self.entries[index].view_epoch = receipt.view_epoch;
                }
                if let Some(id) = authority {
                    self.retire_completed(player, id, receipt.action_kind);
                }
                continue;
            }
            if self.entries.len() == MAX_RECENT_SHOP_RECEIPTS {
                self.entries.pop_front();
                self.evicted = self.evicted.saturating_add(1);
                if self.evicted == 1 {
                    log::warn!("shop receipt history full; oldest result evicted (64)");
                }
            }
            self.entries.push_back(receipt);
            if let Some(id) = authority {
                if request_id != 0 {
                    self.pending.retain(|item| item.owner != player || item.authority_input_id != id);
                }
            }
        }
    }
    pub(crate) fn project(&self, epoch: u64) -> Vec<ShopTransactionReceipt> {
        if self.epoch == Some(epoch) {
            self.entries.iter().cloned().collect()
        } else {
            vec![]
        }
    }
    pub(crate) fn project_pending(&self, epoch: u64) -> Vec<omoba_core::game_proto::ShopTransactionCorrelation> {
        if self.epoch != Some(epoch) {
            return vec![];
        }
        self.pending
            .iter()
            .filter(|item| item.epoch == epoch)
            .map(|item| omoba_core::game_proto::ShopTransactionCorrelation {
                player_id: item.owner,
                view_epoch: item.epoch,
                request_id: item.request_id,
                authority_input: item.authority_input_id,
                action_kind: item.action_kind,
                phase: item.phase,
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerShopUndoView {
    pub player_id: u32,
    pub can_undo: bool,
    pub undo_reason: u8,
    pub newest_sequence: u64,
    pub commitment: [u8; 32],
    /// Session negotiated undo. This is not the authoritative can-undo bit.
    pub undo_enabled: bool,
}

pub fn owner_undo_projection(
    snapshot: &FilteredRenderSnapshot,
    player_id: u32,
) -> Option<omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection> {
    if player_id == 0 || snapshot.team_id == 0 {
        return None;
    }
    let bytes = &snapshot.public_events.iter().rev().find(|event| {
        event.event_kind == FactKind::OwnerShopUndo as u32
            && event.subject.is_none()
            && event.sanitized_payload.get(2..6) == Some(player_id.to_le_bytes().as_slice())
    })?.sanitized_payload;
    let state = omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection::decode(bytes)?;
    (state.player_id == player_id).then_some(state)
}

pub fn owner_undo(snapshot: &FilteredRenderSnapshot, player_id: u32, undo_enabled: bool) -> Option<OwnerShopUndoView> {
    if !undo_enabled {
        return None;
    }
    let state = owner_undo_projection(snapshot, player_id)?;
    Some(OwnerShopUndoView {
        player_id,
        can_undo: state.can_undo,
        undo_reason: state.reason,
        newest_sequence: state.newest_sequence,
        commitment: state.commitment,
        undo_enabled: true,
    })
}

pub fn owner_undo_presentation(
    view: Option<&OwnerShopUndoView>,
    undo_enabled: bool,
    player_id: u32,
) -> Option<omoba_core::game_proto::OwnerShopUndoPresentation> {
    if player_id == 0 || (!undo_enabled && view.is_none()) {
        return None;
    }
    let Some(view) = view else {
        return Some(omoba_core::game_proto::OwnerShopUndoPresentation {
            schema_version: 1,
            player_id,
            projected: false,
            undo_supported: undo_enabled,
            ..Default::default()
        });
    };
    Some(omoba_core::game_proto::OwnerShopUndoPresentation {
        schema_version: 1,
        player_id,
        projected: true,
        undo_supported: undo_enabled,
        can_undo: view.can_undo,
        reason: u32::from(view.undo_reason),
        newest_sequence: view.newest_sequence,
        ledger_commitment: view.commitment.to_vec(),
    })
}

/// Epoch retirement for projected undo eligibility. Not written into schema-2 economy.
#[derive(Default)]
pub struct OwnerUndoCache {
    epoch: Option<u64>,
    enabled: bool,
    view: Option<OwnerShopUndoView>,
    projection: Option<omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection>,
    pending: crate::shop_recovery::PendingShopQueries,
}

impl OwnerUndoCache {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.view = None;
            self.projection = None;
            self.pending = crate::shop_recovery::PendingShopQueries::default();
        }
    }
    pub fn observe(&mut self, epoch: u64, snapshot: &FilteredRenderSnapshot, player_id: u32) {
        if self.epoch != Some(epoch) {
            self.view = None;
            self.projection = None;
            self.pending = crate::shop_recovery::PendingShopQueries::default();
            self.epoch = Some(epoch);
        }
        if !self.enabled {
            self.view = None;
            self.projection = None;
            return;
        }
        self.projection = owner_undo_projection(snapshot, player_id);
        self.view = owner_undo(snapshot, player_id, true);
        for event in &snapshot.public_events {
            if event.event_kind != FactKind::ShopUndoReceipt as u32 || event.subject.is_some() { continue; }
            let Some(receipt) = omoba_core::runtime::shop_undo_wire::ShopUndoReceipt::decode(&event.sanitized_payload) else { continue };
            if receipt.player_id == player_id && receipt.tick == snapshot.replica_tick {
                if let Ok(id) = u32::try_from(receipt.input_id) { self.pending.finish(id); }
            }
        }
    }
    pub fn view(&self) -> Option<&OwnerShopUndoView> { self.view.as_ref() }
    pub fn projection(&self, epoch: u64) -> Option<&omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection> {
        if self.epoch == Some(epoch) { self.projection.as_ref() } else { None }
    }
    pub fn enabled(&self) -> bool { self.enabled }
    pub fn can_track(&self) -> bool { self.enabled && self.pending.can_track() }
    pub fn track(&mut self, id: u32, now_ms: u64) { if self.enabled { self.pending.track(id, now_ms); } }
    pub fn finish(&mut self, id: u32) { self.pending.finish(id); }
    pub fn due(&mut self, now_ms: u64) -> Vec<u32> {
        if self.enabled { self.pending.due(now_ms) } else { Vec::new() }
    }
    pub fn has_pending(&self, id: u32) -> bool { self.pending.contains(id) }
}

/// One accepted undo submission. The authority input id and the renderer request
/// id stay distinct. Sequence is the pinned purchase, not a lookup key by itself.
#[derive(Clone, Copy)]
struct UndoCorrelation {
    epoch: u64,
    owner: u32,
    authority_input_id: u32,
    request_id: u64,
    sequence: u64,
}

/// Bounded undo settlement identities. Epoch scoped. Not the 92-byte wallet and not v1 history.
#[derive(Default)]
pub(crate) struct ShopUndoReceiptHistory {
    epoch: Option<u64>,
    entries: VecDeque<omoba_core::game_proto::ShopUndoTransactionReceipt>,
    pending: VecDeque<UndoCorrelation>,
}
impl ShopUndoReceiptHistory {
    fn adopt_epoch(&mut self, epoch: u64) {
        if self.epoch != Some(epoch) {
            self.entries.clear();
            self.pending.clear();
            self.epoch = Some(epoch);
        }
    }

    /// Records the original renderer request for one authority input. A retry keeps
    /// that pair. The pending table is capped at 64 and is not a per-frame scan.
    pub(crate) fn note_request(&mut self, epoch: u64, owner: u32, authority_input_id: u32, request_id: u64, sequence: u64) {
        if owner == 0 || authority_input_id == 0 || request_id == 0 || sequence == 0 { return; }
        self.adopt_epoch(epoch);
        let authority = u64::from(authority_input_id);
        if self.entries.iter().any(|entry| entry.player_id == owner && entry.input_id == authority && entry.request_id != 0) {
            return;
        }
        if self.pending.iter().any(|item| item.owner == owner && item.authority_input_id == authority_input_id) {
            return;
        }
        if self.pending.len() == MAX_RECENT_SHOP_RECEIPTS {
            self.pending.pop_front();
        }
        self.pending.push_back(UndoCorrelation { epoch, owner, authority_input_id, request_id, sequence });
        self.bind_open_entries();
    }

    pub(crate) fn capture(&mut self, epoch: u64, frame_tick: u64, player: u32, events: &[TeamPublicEvent]) {
        self.adopt_epoch(epoch);
        for event in events.iter().filter(|event| {
            event.event_kind == FactKind::ShopUndoReceipt as u32 && event.subject.is_none()
        }) {
            let Some(receipt) = omoba_core::runtime::shop_undo_wire::ShopUndoReceipt::decode(&event.sanitized_payload) else {
                continue;
            };
            if receipt.player_id != player || receipt.tick != frame_tick { continue; }
            if self.correlation_rejects_sequence(player, receipt.input_id, receipt.sequence) { continue; }
            self.push(omoba_core::game_proto::ShopUndoTransactionReceipt {
                schema_version: 1,
                player_id: receipt.player_id,
                input_id: receipt.input_id,
                settled_tick: receipt.tick,
                sequence: receipt.sequence,
                result_code: receipt.result_code,
                gold: receipt.gold,
                request_id: 0,
                view_epoch: epoch,
            });
        }
        self.bind_open_entries();
    }

    fn correlation_rejects_sequence(&self, owner: u32, authority_input_id: u64, sequence: u64) -> bool {
        let Ok(authority) = u32::try_from(authority_input_id) else { return false };
        self.pending.iter().any(|item| {
            item.owner == owner && item.authority_input_id == authority && item.sequence != sequence
        })
    }

    fn bind_open_entries(&mut self) {
        let epoch = self.epoch;
        let mut stamps = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.request_id != 0 { continue; }
            let Some(epoch) = epoch else { continue };
            let Ok(authority) = u32::try_from(entry.input_id) else { continue };
            let Some(item) = self.pending.iter().find(|item| {
                item.epoch == epoch && item.owner == entry.player_id && item.authority_input_id == authority && item.sequence == entry.sequence
            }) else { continue };
            stamps.push((index, item.request_id, entry.player_id, authority));
        }
        for (index, request_id, owner, authority) in stamps {
            self.entries[index].request_id = request_id;
            self.pending.retain(|item| item.owner != owner || item.authority_input_id != authority);
        }
    }

    pub(crate) fn push(&mut self, receipt: omoba_core::game_proto::ShopUndoTransactionReceipt) {
        if let Some(index) = self.entries.iter().position(|old| old.player_id == receipt.player_id && old.input_id == receipt.input_id) {
            let previous = &self.entries[index];
            let same_settlement = previous.schema_version == receipt.schema_version
                && previous.settled_tick == receipt.settled_tick
                && previous.sequence == receipt.sequence
                && previous.result_code == receipt.result_code
                && previous.gold == receipt.gold
                && previous.view_epoch == receipt.view_epoch;
            if same_settlement && previous.request_id == 0 && receipt.request_id != 0 {
                self.entries[index].request_id = receipt.request_id;
            } else if !same_settlement || (receipt.request_id != 0 && previous.request_id != 0 && receipt.request_id != previous.request_id) {
                log::error!("conflicting shop undo receipt discarded");
            }
            return;
        }
        if self.entries.len() == MAX_RECENT_SHOP_RECEIPTS {
            self.entries.pop_front();
        }
        self.entries.push_back(receipt);
    }
    pub(crate) fn project(&self, epoch: u64) -> Vec<omoba_core::game_proto::ShopUndoTransactionReceipt> {
        if self.epoch == Some(epoch) { self.entries.iter().cloned().collect() } else { vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_core::runtime::native::economy_projection::CommittedEconomyState;
    fn event(player: u32, correlation: u64, tick: u64) -> TeamPublicEvent {
        TeamPublicEvent {
            event_kind: FactKind::ShopReceipt as u32,
            sanitized_payload: ShopReceipt {
                player_id: player,
                input_id: correlation,
                tick,
                action_kind: 17,
                catalog_id: 1,
                slot: 0,
                result_code: 6,
            }
            .encode(),
            ..Default::default()
        }
    }
    #[test]
    fn history_is_owner_only_bounded_idempotent_and_epoch_scoped() {
        let mut history = ShopReceiptHistory::default();
        let events = [event(7, 1, 10), event(8, 2, 10), event(7, 3, 11)];
        history.capture(3, 10, 7, &events);
        history.capture(3, 10, 7, &events);
        assert_eq!(history.project(3).len(), 1);
        assert_eq!(history.project(3)[0].result_code, 6);
        for id in 2..=65 {
            history.capture(3, 10, 7, &[event(7, id, 10)]);
        }
        assert_eq!(history.project(3).len(), 64);
        assert_eq!(history.project(3)[0].input_id, 2);
        assert_eq!(history.evicted, 1);
        assert!(history.project(4).is_empty());
        history.capture(4, 12, 7, &[]);
        assert!(history.project(4).is_empty());
    }

    #[test]
    fn buy_sell_correlation_is_bounded_and_only_a_matching_receipt_is_terminal() {
        let mut history = ShopReceiptHistory::default();
        assert!(history.note_request(4, 7, 801, 41, 17).keeps_pair());
        assert!(!history.note_request(4, 7, 801, 99, 17).keeps_pair(), "a different request cannot replace the original");
        assert_eq!(history.project_pending(4)[0].request_id, 41);
        assert_eq!(history.project_pending(4)[0].phase, SHOP_TXN_PHASE_ADMITTED);
        assert!(history.note_uncertain(4, 7, 41));
        assert_eq!(history.project_pending(4)[0].phase, SHOP_TXN_PHASE_UNCERTAIN);
        assert!(history.project(4).is_empty(), "admission and uncertainty are not receipts");
        history.capture(4, 10, 7, &[event(8, 801, 10)]);
        assert_eq!(history.project_pending(4).len(), 1, "another owner cannot finish this request");
        assert!(history.project(4).is_empty(), "another owner is not retained in this history");
        history.capture(4, 11, 7, &[event(7, 999, 11)]);
        assert_eq!(history.project(4).len(), 1, "an unbound receipt stays in history");
        assert_eq!(history.project(4)[0].request_id, 0);
        assert_eq!(history.project_pending(4).len(), 1, "a different authority input does not replace the pending buy");
        let settled = TeamPublicEvent {
            event_kind: FactKind::ShopReceipt as u32,
            sanitized_payload: ShopReceipt {
                player_id: 7, input_id: 801, tick: 12, action_kind: 17, catalog_id: 4, slot: 0, result_code: 0,
            }.encode(),
            ..Default::default()
        };
        history.capture(4, 12, 7, &[settled.clone()]);
        history.capture(4, 12, 7, &[settled]);
        let rows = history.project(4);
        assert_eq!(rows.iter().filter(|row| row.input_id == 801).count(), 1);
        let bound = rows.iter().find(|row| row.input_id == 801).unwrap();
        assert_eq!((bound.request_id, bound.view_epoch, bound.result_code), (41, 4, 0));
        assert!(history.project_pending(4).is_empty());
        assert!(history.note_request(4, 7, 802, 42, 18).keeps_pair());
        let rejected = TeamPublicEvent {
            event_kind: FactKind::ShopReceipt as u32,
            sanitized_payload: ShopReceipt {
                player_id: 7, input_id: 802, tick: 13, action_kind: 18, catalog_id: 0, slot: 0, result_code: 6,
            }.encode(),
            ..Default::default()
        };
        history.capture(4, 13, 7, &[rejected]);
        let sell = history.project(4).into_iter().find(|row| row.input_id == 802).unwrap();
        assert_eq!(sell.result_code, 6);
        assert!(history.project_pending(4).is_empty());
        history.capture(3, 14, 7, &[]);
        assert!(history.project(4).is_empty() && history.project_pending(4).is_empty(), "an old view cannot keep the new request");
        for id in 1..=64 {
            assert!(history.note_request(9, 7, id, u64::from(id) + 100, 17).keeps_pair());
        }
        assert_eq!(history.note_request(9, 7, 80, 900, 17), ShopRequestNote::Capacity);
        assert_eq!(history.project_pending(9).len(), 64);
        assert_eq!(history.pending_refused, 1);
        assert_eq!(history.project_pending(9)[0].request_id, 101);
    }
    #[test]
    fn shop_correlation_repeated_settlement_does_not_reopen_pending() {
        let mut history = ShopReceiptHistory::default();
        assert!(history.note_request(4, 7, 801, 41, 17).keeps_pair());
        history.capture(4, 10, 7, &[event(7, 801, 10)]);
        assert!(history.project_pending(4).is_empty());
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Completed);
        assert!(history.project_pending(4).is_empty(), "querying an already settled request must not consume active capacity");
        history.capture(4, 10, 7, &[event(7, 801, 10)]);
        assert!(history.project_pending(4).is_empty(), "a repeated canonical receipt cannot leave a phantom pending request");
        assert_eq!(history.project(4).len(), 1);
        assert_eq!(history.project(4)[0].request_id, 41);
    }
    #[test]
    fn shop_correlation_late_request_binds_existing_receipt_without_replay() {
        let mut history = ShopReceiptHistory::default();
        history.capture(4, 10, 7, &[event(7, 801, 10)]);
        assert_eq!(history.project(4)[0].request_id, 0);
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Completed);
        let rows = history.project(4);
        assert_eq!((rows[0].request_id, rows[0].view_epoch), (41, 4), "late correlation must bind the retained canonical receipt without another network replay");
        assert!(history.project_pending(4).is_empty());
    }
    #[test]
    fn shop_correlation_conflict_keeps_the_original_tuple() {
        let mut history = ShopReceiptHistory::default();
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Pending);
        assert_eq!(history.note_request(4, 7, 801, 41, 18), ShopRequestNote::Conflict);
        assert_eq!(history.project_pending(4)[0].action_kind, 17);
        assert_eq!(history.note_request(4, 7, 802, 41, 17), ShopRequestNote::Conflict);
        assert_eq!(history.project_pending(4).len(), 1);
        assert_eq!(history.project_pending(4)[0].request_id, 41);
        let settled = TeamPublicEvent {
            event_kind: FactKind::ShopReceipt as u32,
            sanitized_payload: ShopReceipt {
                player_id: 7, input_id: 801, tick: 12, action_kind: 17, catalog_id: 4, slot: 0, result_code: 0,
            }.encode(),
            ..Default::default()
        };
        history.capture(4, 12, 7, &[settled]);
        assert_eq!(history.note_request(4, 7, 801, 77, 17), ShopRequestNote::Conflict);
        assert_eq!(history.note_request(4, 7, 801, 41, 18), ShopRequestNote::Conflict);
        let bound = &history.project(4)[0];
        assert_eq!((bound.request_id, bound.view_epoch, bound.action_kind, bound.result_code), (41, 4, 17, 0));
        assert!(history.project_pending(4).is_empty());
        assert_eq!(history.pending_refused, 0);
        assert_eq!(history.note_request(4, 8, 900, 50, 18), ShopRequestNote::Pending);
        assert_eq!(history.project(4)[0].player_id, 7);
        assert_eq!(history.project_pending(4)[0].player_id, 8);
        assert_eq!(history.note_request(5, 7, 801, 41, 17), ShopRequestNote::Pending);
        assert!(history.project(4).is_empty(), "another view does not keep the previous owner receipt");
    }
    #[test]
    fn shop_correlation_late_binding_rejects_request_used_by_pending_input() {
        let mut history = ShopReceiptHistory::default();
        assert_eq!(history.note_request(4, 7, 802, 41, 17), ShopRequestNote::Pending);
        history.capture(4, 10, 7, &[event(7, 801, 10)]);
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Conflict,
            "late canonical binding must not steal the request from another active authority input");
        assert_eq!(history.project(4)[0].request_id, 0);
        let pending = history.project_pending(4);
        assert_eq!(pending.len(), 1);
        assert_eq!((pending[0].authority_input, pending[0].request_id), (802, 41));
    }
    #[test]
    fn shop_correlation_completed_request_cannot_bind_another_authority_input() {
        let mut history = ShopReceiptHistory::default();
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Pending);
        history.capture(4, 10, 7, &[event(7, 801, 10)]);
        assert_eq!(history.note_request(4, 7, 802, 41, 17), ShopRequestNote::Conflict,
            "a retained completed renderer request cannot be reused for another authority input");
        assert!(history.project_pending(4).is_empty());
        assert_eq!(history.project(4)[0].request_id, 41);
    }
    #[test]
    fn shop_correlation_crossed_pairs_keep_both_owners() {
        let mut history = ShopReceiptHistory::default();
        assert_eq!(history.note_request(4, 7, 802, 42, 17), ShopRequestNote::Pending);
        history.capture(4, 10, 7, &[event(7, 801, 10)]);
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Completed);
        assert_eq!(history.note_request(4, 7, 801, 42, 17), ShopRequestNote::Conflict);
        assert_eq!(history.note_request(4, 7, 802, 41, 17), ShopRequestNote::Conflict);
        assert_eq!(history.project(4)[0].request_id, 41);
        assert_eq!(history.project(4)[0].action_kind, 17);
        let pending = history.project_pending(4);
        assert_eq!(pending.len(), 1);
        assert_eq!((pending[0].authority_input, pending[0].request_id, pending[0].action_kind), (802, 42, 17));
        assert_eq!(history.note_request(4, 7, 801, 41, 17), ShopRequestNote::Completed);
        assert_eq!(history.note_request(4, 7, 802, 42, 17), ShopRequestNote::Pending);
        assert_eq!(history.project_pending(4).len(), 1);
        assert_eq!(history.note_request(4, 8, 801, 41, 18), ShopRequestNote::Pending);
        assert_eq!(history.project(4)[0].player_id, 7);
        assert_eq!(history.project(4)[0].request_id, 41);
        assert_eq!(history.project_pending(4).iter().filter(|row| row.player_id == 8).count(), 1);
    }
    #[test]
    fn dead_owner_economy_does_not_require_a_live_entity_or_invent_slots() {
        let state = OwnerEconomyState {
            player_id: 7,
            shop_available: false,
            shop_reason: omoba_core::runtime::native::economy_projection::SHOP_REASON_HERO_ABSENT,
            shield_remaining_raw: 0,
            economy: CommittedEconomyState {
                gold: 550,
                item_ids: [0, 3, 0, 0, 0, 0],
                cooldown_bits: [0; 6],
                effect_bits: [0; 10],
                dirty: false,
            },
        };
        let mut snapshot = FilteredRenderSnapshot {
            team_id: 2,
            replica_tick: 11,
            entities: vec![],
            public_events: vec![TeamPublicEvent {
                event_kind: FactKind::OwnerEconomy as u32,
                sanitized_payload: state.encode(),
                ..Default::default()
            }],
            external_effects: vec![],
            memory_directives: vec![],
            remembered_presentations: Default::default(),
            hero_attributes: Default::default(), disclosed_summoners: Default::default(), authoritative_mode: None, combat_preferences: None,
        };
        let value = economy(&snapshot, 7).unwrap();
        assert_eq!(value.gold, 550);
        assert_eq!(value.slots.len(), 6);
        assert_eq!(value.slots[1].catalog_id, 3);
        assert!(!value.shop_available);
        assert_eq!(value.schema_version, 3);
        assert_eq!(value.shop_reason, u32::from(omoba_core::runtime::native::economy_projection::SHOP_REASON_HERO_ABSENT));
        assert_eq!(value.shield_remaining_raw, 0);
        let mut shield = state.clone();
        shield.shield_remaining_raw = 60 * 1024;
        snapshot.public_events[0].sanitized_payload = shield.encode();
        assert_eq!(economy(&snapshot, 7).unwrap().shield_remaining_raw, 60 * 1024);
        assert!(economy(&snapshot, 8).is_none());
        snapshot.public_events[0].sanitized_payload[4..8].copy_from_slice(&(-1i32).to_le_bytes());
        assert!(economy(&snapshot, 7).is_none());
    }

    #[test]
    fn accepted_shop_input_never_becomes_a_success_ack_without_real_settlement() {
        use omoba_core::game_proto::{Step, TeamAcceptedInput, TeamTickFrame};
        let mut frame = TeamTickFrame {
            replica_tick: 10,
            step: Some(Step {
                accepted_inputs: vec![
                    TeamAcceptedInput {
                        player_id: 7,
                        input_id: 1,
                        action_kind: 17,
                        ..Default::default()
                    },
                    TeamAcceptedInput {
                        player_id: 7,
                        input_id: 2,
                        action_kind: 2,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            settled_inputs(&frame, 7),
            [(2, true, "APPLIED_TO_PRESENTATION")]
        );
        frame.step.as_mut().unwrap().public_events =
            vec![event(7, 1, 10), event(8, 3, 10), event(7, 4, 11)];
        assert_eq!(
            settled_inputs(&frame, 7),
            [
                (2, true, "APPLIED_TO_PRESENTATION"),
                (1, false, "SHOP_INSUFFICIENT_GOLD")
            ]
        );
        let mut success = event(7, 1, 10);
        success.sanitized_payload[36..40].copy_from_slice(&0u32.to_le_bytes());
        frame.step.as_mut().unwrap().public_events = vec![success];
        assert_eq!(settled_inputs(&frame, 7)[1], (1, true, "SHOP_SETTLED"));
    }

    #[test]
    fn shop_undo_protocol_settlement_names_ignore_v1_history_and_epoch_clears_pending() {
        use omoba_core::game_proto::{Step, TeamAcceptedInput, TeamTickFrame};
        use omoba_core::runtime::shop_undo_wire::{OwnerShopUndoProjection, ShopUndoReceipt};
        let settled = ShopUndoReceipt {
            player_id: 7, input_id: 5, tick: 10, sequence: 1, result_code: 0, gold: 400,
            catalogs: [0; 6], cooldowns: [0; 6],
        };
        let duplicate = ShopUndoReceipt { input_id: 4, result_code: 15, ..settled.clone() };
        let undo_events = vec![
            TeamPublicEvent { event_kind: FactKind::ShopUndoReceipt as u32, sanitized_payload: settled.encode(), ..Default::default() },
            TeamPublicEvent { event_kind: FactKind::ShopUndoReceipt as u32, sanitized_payload: duplicate.encode(), ..Default::default() },
        ];
        let frame = TeamTickFrame {
            replica_tick: 10,
            step: Some(Step {
                accepted_inputs: vec![
                    TeamAcceptedInput { player_id: 7, input_id: 5, action_kind: 24, ..Default::default() },
                    TeamAcceptedInput { player_id: 7, input_id: 6, action_kind: 17, ..Default::default() },
                    TeamAcceptedInput { player_id: 7, input_id: 7, action_kind: 2, ..Default::default() },
                ],
                public_events: undo_events.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            settled_inputs(&frame, 7),
            [(7, true, "APPLIED_TO_PRESENTATION"), (5, true, "UNDO_SETTLED"), (4, false, "UNDO_DUPLICATE")]
        );
        let mut history = ShopReceiptHistory::default();
        history.capture(1, 10, 7, &undo_events);
        assert!(history.project(1).is_empty());
        history.capture(1, 10, 7, &[event(7, 9, 10)]);
        assert_eq!(history.project(1).len(), 1);
        assert_eq!(history.project(1)[0].schema_version, 1);
        let projection = OwnerShopUndoProjection {
            phase: 0, player_id: 7, can_undo: true, reason: 2, newest_sequence: 1, commitment: [0; 32],
        };
        assert!(OwnerShopUndoProjection::decode(&projection.encode()).is_none());
        let projection = OwnerShopUndoProjection {
            can_undo: false, reason: omoba_core::runtime::shop::UNDO_REASON_OUTSIDE_SHOP, newest_sequence: 4, ..projection
        };
        let snapshot = FilteredRenderSnapshot {
            team_id: 1,
            replica_tick: 10,
            entities: vec![],
            public_events: vec![TeamPublicEvent {
                event_kind: FactKind::OwnerShopUndo as u32,
                sanitized_payload: projection.encode(),
                ..Default::default()
            }],
            external_effects: vec![],
            memory_directives: vec![],
            remembered_presentations: Default::default(),
            hero_attributes: Default::default(), disclosed_summoners: Default::default(),
            authoritative_mode: None, combat_preferences: None,
        };
        let mut cache = OwnerUndoCache::default();
        cache.set_enabled(true);
        cache.observe(1, &snapshot, 7);
        assert_eq!(cache.view().unwrap().undo_reason, omoba_core::runtime::shop::UNDO_REASON_OUTSIDE_SHOP);
        assert!(!cache.view().unwrap().can_undo);
        assert!(cache.view().unwrap().undo_enabled);
        assert_eq!(cache.view().unwrap().newest_sequence, 4);
        cache.track(4, 10);
        assert!(cache.has_pending(4));
        let mut finished = snapshot.clone();
        finished.public_events.push(undo_events[1].clone());
        cache.observe(1, &finished, 7);
        assert!(!cache.has_pending(4));
        cache.track(8, 20);
        cache.observe(2, &snapshot, 7);
        assert!(!cache.has_pending(8));
        assert!(cache.view().is_some());
        cache.set_enabled(false);
        assert!(cache.view().is_none());
        cache.track(8, 30);
        assert!(!cache.has_pending(8));
    }

    fn undo_event(player: u32, input_id: u64, tick: u64, sequence: u64, result: u32, gold: i32) -> TeamPublicEvent {
        use omoba_core::runtime::shop_undo_wire::ShopUndoReceipt;
        TeamPublicEvent {
            event_kind: FactKind::ShopUndoReceipt as u32,
            sanitized_payload: ShopUndoReceipt {
                player_id: player, input_id, tick, sequence, result_code: result, gold,
                catalogs: [1, 0, 0, 0, 0, 0], cooldowns: [0; 6],
            }.encode(),
            ..Default::default()
        }
    }

    #[test]
    fn distinct_undo_counters_settle_only_the_original_request() {
        let mut history = ShopUndoReceiptHistory::default();
        let mut buys = ShopReceiptHistory::default();
        let buy = event(7, 41, 10);
        buys.capture(3, 10, 7, &[buy.clone()]);
        assert_eq!(buys.project(3)[0].input_id, 41);
        history.capture(3, 10, 7, &[buy]);
        assert!(history.project(3).is_empty(), "a v1 buy receipt is not an undo correlation");

        // Receipt observed before the renderer correlation exists.
        history.capture(3, 10, 7, &[
            undo_event(7, 901, 10, 9, 0, 400),
            undo_event(8, 901, 10, 9, 0, 400),
        ]);
        assert_eq!(history.project(3).len(), 1);
        assert_eq!(history.project(3)[0].request_id, 0);
        assert_eq!(history.project(3)[0].input_id, 901);

        history.note_request(3, 7, 901, 41, 9);
        history.note_request(3, 7, 901, 77, 9);
        history.note_request(3, 7, 41, 901, 9);
        let bound = history.project(3);
        assert_eq!(bound.len(), 1);
        assert_eq!(bound[0].request_id, 41, "late correlation keeps the original renderer request");
        assert_eq!(bound[0].input_id, 901);
        assert_eq!(bound[0].sequence, 9);
        assert_eq!(bound[0].view_epoch, 3);
        assert_eq!(bound[0].gold, 400);
        assert!(history.pending.iter().all(|item| item.authority_input_id != 901), "a bound receipt leaves the pending table");
        assert!(history.pending.iter().any(|item| item.authority_input_id == 41 && item.request_id == 901));

        history.note_request(3, 7, 901, 77, 44);
        history.capture(3, 10, 7, &[undo_event(7, 901, 10, 9, 0, 400)]);
        assert_eq!(history.project(3)[0].request_id, 41, "replay keeps the accepted correlation");

        history.capture(3, 10, 7, &[
            undo_event(7, 901, 10, 9, 0, 400),
            undo_event(7, 880, 10, 9, 0, 900),
            undo_event(7, 41, 10, 9, 0, 50),
            undo_event(7, 901, 10, 44, 0, 900),
        ]);
        let batch = history.project(3);
        assert_eq!(batch.len(), 3);
        assert_eq!((batch[0].input_id, batch[0].request_id, batch[0].sequence, batch[0].gold), (901, 41, 9, 400));
        assert_eq!((batch[1].input_id, batch[1].request_id, batch[1].gold), (880, 0, 900));
        assert_eq!((batch[2].input_id, batch[2].request_id), (41, 901));
        assert_ne!(batch[2].request_id, 41, "an authority id equal to another UI request does not take that request");
        assert_eq!(buys.project(3).len(), 1);
        assert_eq!(buys.project(3)[0].input_id, 41);

        let mut fresh = ShopUndoReceiptHistory::default();
        fresh.note_request(3, 7, 901, 41, 9);
        fresh.capture(3, 10, 7, &[undo_event(7, 901, 10, 44, 0, 900)]);
        assert!(fresh.project(3).is_empty(), "sequence alone does not remap the request");
        fresh.capture(3, 10, 7, &[undo_event(7, 901, 10, 9, 0, 400)]);
        assert_eq!(fresh.project(3)[0].request_id, 41);

        fresh.capture(4, 11, 7, &[undo_event(7, 901, 11, 9, 0, 400)]);
        assert_eq!(fresh.project(4)[0].request_id, 0, "an old epoch correlation does not settle the new epoch");
        assert!(fresh.project(3).is_empty());
        fresh.note_request(4, 8, 901, 41, 9);
        fresh.capture(4, 11, 7, &[undo_event(7, 901, 11, 9, 0, 400)]);
        assert_eq!(fresh.project(4)[0].request_id, 0, "a foreign owner does not bind this player's receipt");
    }

    #[test]
    fn undo_correlation_capacity_reset_and_ordered_replay() {
        let mut history = ShopUndoReceiptHistory::default();
        for id in 1..=65u32 {
            history.note_request(3, 7, id, 1_000 + u64::from(id), 9);
        }
        assert_eq!(history.pending.len(), 64);
        assert!(history.pending.front().is_some_and(|item| item.authority_input_id == 2));
        history.capture(3, 10, 7, &[undo_event(7, 1, 10, 9, 0, 400), undo_event(7, 65, 10, 9, 0, 400)]);
        let projected = history.project(3);
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].request_id, 0, "the evicted correlation cannot settle");
        assert_eq!(projected[1].request_id, 1_065);

        let mut replay = ShopUndoReceiptHistory::default();
        replay.capture(3, 10, 7, &[undo_event(7, 901, 10, 9, 0, 400)]);
        assert_eq!(replay.project(3)[0].request_id, 0);
        replay.note_request(3, 7, 901, 41, 9);
        assert_eq!(replay.project(3)[0].request_id, 41);
        replay.capture(3, 10, 7, &[undo_event(7, 901, 10, 9, 0, 400)]);
        assert_eq!(replay.project(3)[0].request_id, 41);
        assert!(replay.pending.is_empty());
        replay.note_request(5, 7, 901, 41, 9);
        assert!(replay.project(3).is_empty());
        assert_eq!(replay.pending.len(), 1);
        replay.capture(5, 12, 7, &[]);
        assert!(replay.project(5).is_empty());
        assert_eq!(replay.pending.len(), 1, "an empty frame keeps an unbound correlation");
        replay.capture(6, 13, 7, &[]);
        assert!(replay.pending.is_empty() && replay.project(6).is_empty(), "a new epoch drops the previous correlation");
    }
}
