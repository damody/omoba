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

pub fn economy(
    snapshot: &FilteredRenderSnapshot,
    player_id: u32,
) -> Option<OwnerEconomyPresentation> {
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
    let state = OwnerEconomyState::decode(bytes)?;
    Some(OwnerEconomyPresentation {
        schema_version: 2,
        player_id,
        gold: state.economy.gold,
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
        .filter(|input| input.player_id == player && !matches!(input.action_kind, 17 | 18))
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
    results
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
    pub(crate) evicted: u64,
}
impl ShopReceiptHistory {
    /// Called on every applied frame, including discarded catch-up snapshots.
    pub(crate) fn capture(
        &mut self,
        epoch: u64,
        frame_tick: u64,
        player: u32,
        events: &[TeamPublicEvent],
    ) {
        if self.epoch != Some(epoch) {
            self.entries.clear();
            self.epoch = Some(epoch);
        }
        for event in events.iter().filter(|event| {
            event.event_kind == FactKind::ShopReceipt as u32 && event.subject.is_none()
        }) {
            let Some(receipt) = ShopReceipt::decode(&event.sanitized_payload) else {
                continue;
            };
            if receipt.player_id != player || receipt.tick != frame_tick {
                continue;
            }
            let receipt = ShopTransactionReceipt {
                schema_version: 1,
                player_id: receipt.player_id,
                input_id: receipt.input_id,
                settled_tick: receipt.tick,
                action_kind: receipt.action_kind,
                catalog_id: receipt.catalog_id,
                item_slot: receipt.slot,
                result_code: receipt.result_code,
            };
            if let Some(previous) = self
                .entries
                .iter()
                .find(|old| old.input_id == receipt.input_id)
            {
                if previous != &receipt {
                    log::error!("conflicting shop receipt discarded");
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
        }
    }
    pub(crate) fn project(&self, epoch: u64) -> Vec<ShopTransactionReceipt> {
        if self.epoch == Some(epoch) {
            self.entries.iter().cloned().collect()
        } else {
            vec![]
        }
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
    fn dead_owner_economy_does_not_require_a_live_entity_or_invent_slots() {
        let state = OwnerEconomyState {
            player_id: 7,
            shop_available: false,
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
        };
        let value = economy(&snapshot, 7).unwrap();
        assert_eq!(value.gold, 550);
        assert_eq!(value.slots.len(), 6);
        assert_eq!(value.slots[1].catalog_id, 3);
        assert!(!value.shop_available);
        assert_eq!(value.schema_version, 2);
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
}
