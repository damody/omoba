//! Shared renderer wire contract, without a world, socket, or input allocator.
//! Encoding does not authorize a command. The client runtime still validates
//! owner/epoch/secure targets and the server remains the gameplay authority.

use crate::game_proto::{self as proto, player_input::Action, renderer_input::Intent, PlayerInput};

pub const PRESENTATION_MAGIC: u32 = 0x4f4d_5254;
/// Point-command queued semantics require v4; v3 silently discards the flag.
pub const PRESENTATION_PROTOCOL_VERSION: u32 = 4;
pub const MAX_PRESENTATION_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// Convert an existing formal command into a renderer intent. Entity IDs must
/// already refer to the renderer's disclosed view; no hidden ID lookup occurs.
/// Unsupported actions and missing required positions fail closed. Keep this
/// match exhaustive so a new PlayerInput action cannot be silently forgotten.
pub fn player_input_to_renderer_intent(input: &PlayerInput) -> Option<Intent> {
    Some(match input.action.as_ref()? {
        Action::MoveTo(value) => {
            let target = value.target?;
            Intent::MoveTo(proto::MoveToIntent {
                x_raw: i64::from(target.x), y_raw: i64::from(target.y), queued: value.queued,
            })
        }
        Action::AttackMove(value) => {
            let target = value.target?;
            Intent::AttackMove(proto::AttackMoveIntent {
                x_raw: i64::from(target.x), y_raw: i64::from(target.y), queued: value.queued,
            })
        }
        Action::AttackTarget(value) => Intent::AttackTarget(proto::AttackTargetIntent {
            target_render_id: u64::from(value.target_id), queued: value.queued,
        }),
        Action::HoldPosition(value) => Intent::HoldPosition(*value),
        Action::CastAbility(value) => Intent::AbilityCast(proto::AbilityCastIntent {
            ability_index: value.ability_index,
            target_render_id: u64::from(value.target_entity.unwrap_or(0)),
            x_raw: i64::from(value.target_pos.as_ref().map_or(0, |pos| pos.x)),
            y_raw: i64::from(value.target_pos.as_ref().map_or(0, |pos| pos.y)),
        }),
        Action::UpgradeAbility(value) => Intent::AbilityUpgrade(proto::AbilityUpgradeIntent {
            ability_index: value.ability_index,
        }),
        Action::Recall(_) => Intent::Recall(proto::RecallIntent {}),
        Action::ItemUse(value) => Intent::ItemUse(proto::ItemUseIntent {
            item_slot: value.item_slot,
            target_render_id: u64::from(value.target_entity.unwrap_or(0)),
            x_raw: i64::from(value.target_pos.as_ref().map_or(0, |pos| pos.x)),
            y_raw: i64::from(value.target_pos.as_ref().map_or(0, |pos| pos.y)),
        }),
        Action::ItemBuy(value) => {
            let item = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.id == value.item_id)?;
            Intent::ItemBuy(proto::ItemBuyIntent { catalog_id: u32::from(item.catalog_id) })
        }
        Action::ItemSell(value) => Intent::ItemSell(proto::ItemSellIntent { item_slot: value.item_slot }),
        Action::TowerPlace(value) => {
            let pos = value.pos?;
            Intent::TowerAction(proto::TowerActionIntent {
                action_kind: 1, tower_kind_id: value.tower_kind_id,
                x_raw: i64::from(pos.x), y_raw: i64::from(pos.y), ..Default::default()
            })
        }
        Action::TowerUpgrade(value) => Intent::TowerAction(proto::TowerActionIntent {
            action_kind: 2, tower_render_id: u64::from(value.tower_entity_id),
            path: value.path, level: value.level, ..Default::default()
        }),
        Action::TowerSell(value) => Intent::TowerAction(proto::TowerActionIntent {
            action_kind: 3, tower_render_id: u64::from(value.tower_entity_id), ..Default::default()
        }),
        Action::NoOp(_) | Action::StartRound(_) | Action::TogglePause(_)
        | Action::ToggleGameSpeed(_) | Action::DebugSpawnCreep(_)
        | Action::TowerAbilityCast(_) | Action::SetTowerTargetPriority(_) => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    fn roundtrip(action: Action) -> Intent {
        let input = PlayerInput { action: Some(action) };
        let input = PlayerInput::decode(input.encode_to_vec().as_slice()).unwrap();
        let wire = proto::RendererInput {
            request_id: 44, player_id: 7, disclosure_epoch: 3,
            intent: player_input_to_renderer_intent(&input),
        };
        let decoded = proto::RendererInput::decode(wire.encode_to_vec().as_slice()).unwrap();
        assert_eq!(decoded, wire);
        decoded.intent.unwrap()
    }

    #[test]
    fn shared_renderer_codec_preserves_point_target_and_queue_semantics() {
        for queued in [false, true] {
            let target = Some(proto::Vec2I { x: i32::MIN, y: i32::MAX });
            assert_eq!(roundtrip(Action::MoveTo(proto::MoveTo { target, queued })),
                Intent::MoveTo(proto::MoveToIntent { x_raw: i64::from(i32::MIN), y_raw: i64::from(i32::MAX), queued }));
            assert_eq!(roundtrip(Action::AttackMove(proto::AttackMove { target, queued })),
                Intent::AttackMove(proto::AttackMoveIntent { x_raw: i64::from(i32::MIN), y_raw: i64::from(i32::MAX), queued }));
            assert_eq!(roundtrip(Action::AttackTarget(proto::AttackTarget { target_id: 42, queued })),
                Intent::AttackTarget(proto::AttackTargetIntent { target_render_id: 42, queued }));
            assert_eq!(roundtrip(Action::HoldPosition(proto::HoldPosition { queued })),
                Intent::HoldPosition(proto::HoldPosition { queued }));
        }
    }

    #[test]
    fn shared_renderer_codec_preserves_slots_catalog_and_tower_fields() {
        let point = Some(proto::Vec2I { x: -123, y: 456 });
        assert_eq!(roundtrip(Action::CastAbility(proto::CastAbility { ability_index: 3, target_entity: Some(42), target_pos: point })),
            Intent::AbilityCast(proto::AbilityCastIntent { ability_index: 3, target_render_id: 42, x_raw: -123, y_raw: 456 }));
        assert_eq!(roundtrip(Action::CastAbility(proto::CastAbility { ability_index: 1, target_entity: None, target_pos: None })),
            Intent::AbilityCast(proto::AbilityCastIntent { ability_index: 1, ..Default::default() }));
        assert_eq!(roundtrip(Action::UpgradeAbility(proto::UpgradeAbility { ability_index: 2 })),
            Intent::AbilityUpgrade(proto::AbilityUpgradeIntent { ability_index: 2 }));
        assert_eq!(roundtrip(Action::Recall(proto::Recall {})), Intent::Recall(proto::RecallIntent {}));
        assert_eq!(roundtrip(Action::ItemUse(proto::ItemUse { item_slot: 5, target_entity: Some(42), target_pos: point })),
            Intent::ItemUse(proto::ItemUseIntent { item_slot: 5, target_render_id: 42, x_raw: -123, y_raw: 456 }));
        let item = &omoba_template_ids::MOBA_ITEM_CATALOG[0];
        assert_eq!(roundtrip(Action::ItemBuy(proto::ItemBuy { item_id: item.id.into() })),
            Intent::ItemBuy(proto::ItemBuyIntent { catalog_id: u32::from(item.catalog_id) }));
        assert_eq!(roundtrip(Action::ItemSell(proto::ItemSell { item_slot: 5 })),
            Intent::ItemSell(proto::ItemSellIntent { item_slot: 5 }));
        assert_eq!(roundtrip(Action::TowerPlace(proto::TowerPlace { tower_kind_id: 12, pos: point })),
            Intent::TowerAction(proto::TowerActionIntent { action_kind: 1, tower_kind_id: 12, x_raw: -123, y_raw: 456, ..Default::default() }));
        assert_eq!(roundtrip(Action::TowerUpgrade(proto::TowerUpgradeInput { tower_entity_id: 42, path: 2, level: 3 })),
            Intent::TowerAction(proto::TowerActionIntent { action_kind: 2, tower_render_id: 42, path: 2, level: 3, ..Default::default() }));
        assert_eq!(roundtrip(Action::TowerSell(proto::TowerSell { tower_entity_id: 42 })),
            Intent::TowerAction(proto::TowerActionIntent { action_kind: 3, tower_render_id: 42, ..Default::default() }));
    }

    #[test]
    fn shared_renderer_codec_rejects_unsupported_and_missing_required_data() {
        assert_eq!(player_input_to_renderer_intent(&PlayerInput::default()), None);
        for action in [
            Action::MoveTo(proto::MoveTo { target: None, queued: true }),
            Action::AttackMove(proto::AttackMove { target: None, queued: true }),
            Action::TowerPlace(proto::TowerPlace { pos: None, ..Default::default() }),
            Action::ItemBuy(proto::ItemBuy { item_id: "__unknown_item__".into() }),
            Action::NoOp(Default::default()), Action::StartRound(Default::default()),
            Action::TogglePause(Default::default()), Action::ToggleGameSpeed(Default::default()),
            Action::DebugSpawnCreep(Default::default()), Action::TowerAbilityCast(Default::default()),
            Action::SetTowerTargetPriority(Default::default()),
        ] { assert_eq!(player_input_to_renderer_intent(&PlayerInput { action: Some(action) }), None); }
    }
}
