//! Presentation correlation of tick-local authority shop results. Never replayed.
use crate::game_proto::{player_input::Action, PlayerInput, TeamPublicEvent};
use crate::runtime::shop::{ShopCommand, ShopSettlement};
use crate::runtime::{CanonicalAcceptedInput, FactKind, ProjectionError};
use prost::Message;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopReceipt {
    pub player_id: u32,
    pub input_id: u64,
    pub tick: u64,
    pub action_kind: u32,
    pub catalog_id: u32,
    pub slot: u32,
    pub result_code: u32,
}

impl ShopReceipt {
    pub const PAYLOAD_LEN: usize = 40;
    fn valid(&self) -> bool {
        self.player_id != 0
            && self.input_id != 0
            && self.result_code <= 13
            && match self.action_kind {
                17 => {
                    self.slot == 0
                        && (self.catalog_id == 0 && self.result_code != 0
                            || omoba_template_ids::MOBA_ITEM_CATALOG
                                .iter()
                                .any(|item| u32::from(item.catalog_id) == self.catalog_id))
                }
                18 => self.catalog_id == 0 && (self.slot < 6 || self.result_code != 0),
                _ => false,
            }
    }
    pub fn encode(&self) -> Vec<u8> {
        [
            1u32.to_le_bytes().as_slice(),
            self.player_id.to_le_bytes().as_slice(),
            self.input_id.to_le_bytes().as_slice(),
            self.tick.to_le_bytes().as_slice(),
            self.action_kind.to_le_bytes().as_slice(),
            self.catalog_id.to_le_bytes().as_slice(),
            self.slot.to_le_bytes().as_slice(),
            self.result_code.to_le_bytes().as_slice(),
        ]
        .concat()
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }
        let u32_at = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let u64_at = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        if u32_at(0) != 1 {
            return None;
        }
        let receipt = Self {
            player_id: u32_at(4),
            input_id: u64_at(8),
            tick: u64_at(16),
            action_kind: u32_at(24),
            catalog_id: u32_at(28),
            slot: u32_at(32),
            result_code: u32_at(36),
        };
        receipt.valid().then_some(receipt)
    }
}

fn command(input: &CanonicalAcceptedInput) -> Option<ShopCommand> {
    match PlayerInput::decode(input.sanitized_payload.as_slice())
        .ok()?
        .action?
    {
        Action::ItemBuy(buy) if input.action_kind == 17 => Some(ShopCommand::Buy(buy.item_id)),
        Action::ItemSell(sell) if input.action_kind == 18 => {
            Some(ShopCommand::Sell(sell.item_slot as usize))
        }
        _ => None,
    }
}

/// Exact per-player queue alignment; no sorting by correlation before matching.
/// A missing result is never converted to success, and a mismatch fails closed.
pub fn project_shop_receipts(
    team: u32,
    tick: u64,
    accepted: &[CanonicalAcceptedInput],
    settlements: &[ShopSettlement],
) -> Result<Vec<TeamPublicEvent>, ProjectionError> {
    let mut queues: BTreeMap<u32, VecDeque<&ShopSettlement>> = BTreeMap::new();
    for result in settlements {
        queues
            .entry(result.player_id)
            .or_default()
            .push_back(result);
    }
    let mut events = Vec::new();
    for input in accepted
        .iter()
        .filter(|input| input.team_id == team && matches!(input.action_kind, 17 | 18))
    {
        let Some(result) = queues
            .get_mut(&input.player_id)
            .and_then(VecDeque::pop_front)
        else {
            continue; // Acceptance alone is not a transaction receipt.
        };
        if command(input).as_ref() != Some(&result.command) {
            return Err(ProjectionError::MalformedDisclosedState);
        }
        let (catalog_id, slot) = match &result.command {
            ShopCommand::Buy(id) => (
                omoba_template_ids::MOBA_ITEM_CATALOG
                    .iter()
                    .find(|item| item.id == id)
                    .map_or(0, |item| u32::from(item.catalog_id)),
                0,
            ),
            ShopCommand::Sell(slot) => (
                0,
                u32::try_from(*slot).map_err(|_| ProjectionError::MalformedDisclosedState)?,
            ),
        };
        let receipt = ShopReceipt {
            player_id: input.player_id,
            input_id: input.input_id,
            tick,
            action_kind: input.action_kind,
            catalog_id,
            slot,
            result_code: result
                .result
                .as_ref()
                .err()
                .map_or(0, |error| error.receipt_code()),
        };
        if !receipt.valid() {
            return Err(ProjectionError::MalformedDisclosedState);
        }
        events.push(TeamPublicEvent {
            event_kind: FactKind::ShopReceipt as u32,
            stable_sub_index: events.len() as u32,
            sanitized_payload: receipt.encode(),
            ..Default::default()
        });
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game_proto::ItemBuy, runtime::shop::ShopError};
    fn input(team: u32, player: u32, correlation: u64, id: &str) -> CanonicalAcceptedInput {
        CanonicalAcceptedInput::from_authoritative_acceptance(
            team,
            player,
            correlation,
            17,
            99,
            None,
            PlayerInput {
                action: Some(Action::ItemBuy(ItemBuy { item_id: id.into() })),
            }
            .encode_to_vec(),
        )
    }
    fn result(player: u32, id: &str, result: Result<(), ShopError>) -> ShopSettlement {
        ShopSettlement {
            player_id: player,
            command: ShopCommand::Buy(id.into()),
            result,
        }
    }
    #[test]
    fn ordered_results_correlate_exactly_without_cross_team_or_player_receipts() {
        let inputs = [
            input(1, 7, 92, "moba_sword"),
            input(2, 8, 4, "moba_armor"),
            input(1, 7, 13, "moba_sword"),
            input(1, 9, 6, "missing"),
        ];
        let results = [
            result(7, "moba_sword", Ok(())),
            result(8, "moba_armor", Ok(())),
            result(7, "moba_sword", Err(ShopError::InsufficientGold)),
            result(9, "missing", Err(ShopError::UnknownItem)),
        ];
        let events = project_shop_receipts(1, 42, &inputs, &results).unwrap();
        let receipts: Vec<_> = events
            .iter()
            .map(|event| ShopReceipt::decode(&event.sanitized_payload).unwrap())
            .collect();
        assert_eq!(
            receipts
                .iter()
                .map(|r| (r.player_id, r.input_id, r.result_code))
                .collect::<Vec<_>>(),
            [(7, 92, 0), (7, 13, 6), (9, 6, 1)]
        );
        assert!(events.iter().all(|event| event.subject.is_none()));
        assert!(project_shop_receipts(1, 42, &inputs, &[])
            .unwrap()
            .is_empty());
        assert!(project_shop_receipts(1, 42, &inputs, &[result(7, "moba_armor", Ok(()))]).is_err());
    }
    #[test]
    fn receipt_codec_refuses_unknown_versions_invalid_codes_and_fake_success() {
        let valid = ShopReceipt {
            player_id: 7,
            input_id: 42,
            tick: 55,
            action_kind: 17,
            catalog_id: 1,
            slot: 0,
            result_code: 0,
        };
        assert_eq!(ShopReceipt::decode(&valid.encode()), Some(valid.clone()));
        for changed in [
            ShopReceipt {
                result_code: 14,
                ..valid.clone()
            },
            ShopReceipt {
                catalog_id: 0,
                ..valid.clone()
            },
            ShopReceipt {
                catalog_id: 999,
                ..valid.clone()
            },
            ShopReceipt {
                input_id: 0,
                ..valid.clone()
            },
            ShopReceipt {
                action_kind: 18,
                catalog_id: 0,
                slot: 6,
                ..valid.clone()
            },
        ] {
            assert!(ShopReceipt::decode(&changed.encode()).is_none());
        }
        let mut bytes = valid.encode();
        bytes[0] = 2;
        assert!(ShopReceipt::decode(&bytes).is_none());
        assert!(ShopReceipt::decode(&bytes[..39]).is_none());
    }

    #[test]
    fn dead_actor_still_receives_rejection_without_a_fake_replay_actor() {
        use crate::runtime::{ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector};
        let mut accepted = input(1, 7, 42, "moba_sword");
        accepted.actor_canonical_id = 0;
        let frame = TeamViewProjector::new(1, TeamProjectorConfig::default())
            .build_frame_with_settlements(
                0,
                0,
                &Default::default(),
                vec![],
                &[],
                &ProjectionDependencyGraph::default(),
                vec![accepted],
                &[result(7, "moba_sword", Err(ShopError::HeroUnavailable))],
            )
            .unwrap();
        let step = frame.frame.step.unwrap();
        assert!(step.accepted_inputs.is_empty());
        assert_eq!(step.public_events.len(), 1);
        let receipt = ShopReceipt::decode(&step.public_events[0].sanitized_payload).unwrap();
        assert_eq!(
            (receipt.input_id, receipt.tick, receipt.result_code),
            (42, 0, 11)
        );
        assert!(step.public_events[0].subject.is_none());
    }

    #[test]
    fn persistent_economy_requires_matching_owner_team_audience() {
        use crate::runtime::{
            native::economy_projection::{CommittedEconomyState, OwnerEconomyState},
            FactAudience, FactOrderingKey, FactPhase, Gold, Inventory, ItemEffects, ObservableFact,
            OrderedFact, ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector,
        };
        for (team, audience, count) in [
            (1, FactAudience::Team(1), 1),
            (2, FactAudience::Team(1), 0),
            (1, FactAudience::AllPlayers, 0),
            (1, FactAudience::Team(2), 0),
        ] {
            let fact = OrderedFact {
                key: FactOrderingKey {
                    tick: 1,
                    phase: FactPhase::PostStep,
                    canonical_source_order: 0,
                    local_ordinal: 0,
                    fact_kind: FactKind::OwnerEconomy,
                },
                audience,
                fact: ObservableFact::OwnerEconomy {
                    team,
                    state: OwnerEconomyState {
                        player_id: 7,
                        economy: CommittedEconomyState::capture(
                            Gold(550),
                            &Inventory::default(),
                            ItemEffects::default(),
                        )
                        .unwrap(),
                        shop_available: false,
                    },
                },
            };
            let frame = TeamViewProjector::new(1, TeamProjectorConfig::default())
                .build_frame(
                    1,
                    1,
                    &Default::default(),
                    vec![],
                    &[fact],
                    &ProjectionDependencyGraph::default(),
                )
                .unwrap();
            assert_eq!(frame.frame.step.unwrap().public_events.len(), count);
        }
    }
}
