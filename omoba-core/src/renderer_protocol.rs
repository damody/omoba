//! Shared renderer wire contract, without a world, socket, or input allocator.
//! Encoding does not authorize a command. The client runtime still validates
//! owner/epoch/secure targets and the server remains the gameplay authority.

use crate::game_proto::{self as proto, player_input::Action, renderer_input::Intent, PlayerInput};

pub const PRESENTATION_MAGIC: u32 = 0x4f4d_5254;
/// v6 carries the summoner ABI rows. v5 requires the owner shield baseline; v4 also added queued point commands.
// v7 adds typed communication intents and local correlation, separate from gameplay.
pub const PRESENTATION_PROTOCOL_VERSION: u32 = 12;
pub const MAX_PRESENTATION_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// Convert an existing formal command into a renderer intent. Entity IDs must
/// already refer to the renderer's disclosed view; no hidden ID lookup occurs.
/// Unsupported actions and missing required positions fail closed. Keep this
/// match exhaustive so a new PlayerInput action cannot be silently forgotten.
pub fn player_input_to_renderer_intent(input: &PlayerInput) -> Option<Intent> {
    Some(match input.action.as_ref()? {
        Action::SummonCommand(value) => {
            crate::runtime::summon_transport::validate_client(value).ok()?;
            Intent::SummonCommand(value.clone())
        }
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
        Action::Stop(_) => Intent::Stop(proto::Stop {}),
        Action::InputScheme(value) => {
            crate::runtime::input_scheme_transport::validate_update(value).ok()?;
            Intent::InputScheme(*value)
        },
        Action::Direction(value) => {
            crate::runtime::direction_transport::validate_client(value).ok()?;
            Intent::Direction(proto::RendererDirectionIntent {
                schema_version:value.schema_version,
                source_render_id:value.source.as_ref()?.replica_entity_id.as_ref()?.value,
                press_id:value.press_id,phase:value.phase,x:value.x,y:value.y,
            })
        },
        Action::CombatPreferences(value) => {
            crate::runtime::combat_transport::validate_update(value).ok()?;
            Intent::CombatPreferences(*value)
        }
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
        Action::SummonerCast(value) => Intent::SummonerCast(proto::SummonerCastIntent {
            slot: value.slot, x_raw: value.x_raw, y_raw: value.y_raw,
        }),
        Action::WardPlacement(value) => Intent::WardPlacement(proto::WardPlacementIntent {
            slot: value.slot, x_raw: value.x_raw, y_raw: value.y_raw,
        }),
        Action::ItemUse(value) => {
            let explicit = value.point_explicit == Some(true);
            if explicit && value.target_pos.is_none() {
                return None;
            }
            let pos = value.target_pos.as_ref();
            // owner_entity and owner_generation stay on the authority command.
            Intent::ItemUse(proto::ItemUseIntent {
                item_slot: value.item_slot,
                target_render_id: u64::from(value.target_entity.unwrap_or(0)),
                x_raw: i64::from(pos.map_or(0, |point| point.x)),
                y_raw: i64::from(pos.map_or(0, |point| point.y)),
                explicit_x_raw: explicit.then(|| i64::from(pos.expect("explicit point checked").x)),
                explicit_y_raw: explicit.then(|| i64::from(pos.expect("explicit point checked").y)),
            })
        }
        Action::ItemBuy(value) => {
            let item = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.id == value.item_id)?;
            Intent::ItemBuy(proto::ItemBuyIntent { catalog_id: u32::from(item.catalog_id) })
        }
        Action::ItemSell(value) => Intent::ItemSell(proto::ItemSellIntent { item_slot: value.item_slot }),
        Action::ShopUndo(value) => {
            // Newest and a zero sequence must not become a renderer intent.
            // A later purchase cannot be selected by rewriting this field.
            if value.mode != crate::runtime::shop_undo_wire::SHOP_UNDO_MODE_SEQUENCE || value.sequence == 0 {
                return None;
            }
            Intent::ShopUndo(proto::ShopUndoIntent { sequence: value.sequence })
        }
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
        | Action::TowerAbilityCast(_) | Action::SetTowerTargetPriority(_)
        | Action::OwnedSummonOrder(_) => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn owned_summon_order_is_not_reinterpreted_as_hero_renderer_input() {
        for operation in [1, 2] {
            let order = proto::OwnedSummonOrder {
                owner_entity: 0, owner_generation: 1, summon_entity: 9,
                summon_generation: 2, operation,
                point: (operation == 1).then_some(proto::Vec2I { x: 120, y: 40 }),
            };
            assert!(player_input_to_renderer_intent(&PlayerInput {
                action: Some(Action::OwnedSummonOrder(order)),
            }).is_none());
        }
    }

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
    fn shared_renderer_direction_codec_keeps_only_opaque_source_and_integer_gesture() {
        let client=proto::HeroDirectionInput {schema_version:1,source:Some(proto::SecureReplicaTarget {
            replica_entity_id:Some(proto::ReplicaEntityId {value:9}),view_epoch:Some(proto::ViewEpoch {value:4}),
            disclosure_epoch:Some(proto::DisclosureEpoch {value:3}),
        }),press_id:u64::MAX-1,phase:2,x:-32767,y:32767,sealed_source:0};
        let Intent::Direction(intent)=roundtrip(Action::Direction(client.clone())) else {panic!("wrong renderer intent");};
        assert_eq!((intent.source_render_id,intent.press_id,intent.phase,intent.x,intent.y),(9,u64::MAX-1,2,-32767,32767));
        assert_eq!(crate::runtime::direction_transport::client_from_renderer(&intent,client.source.clone().unwrap()).unwrap(),client);
        assert!(player_input_to_renderer_intent(&PlayerInput {action:Some(Action::Direction(proto::HeroDirectionInput {
            source:None,sealed_source:1<<32,..client.clone()}))}).is_none());
        let mut wrong=client.source.unwrap();wrong.replica_entity_id.as_mut().unwrap().value=10;
        assert!(crate::runtime::direction_transport::client_from_renderer(&intent,wrong).is_err());
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
        assert_eq!(roundtrip(Action::SummonerCast(proto::SummonerCast { slot: 1, x_raw: None, y_raw: None })),
            Intent::SummonerCast(proto::SummonerCastIntent { slot: 1, x_raw: None, y_raw: None }));
        assert_eq!(roundtrip(Action::SummonerCast(proto::SummonerCast { slot: 0, x_raw: Some(-4), y_raw: Some(9) })),
            Intent::SummonerCast(proto::SummonerCastIntent { slot: 0, x_raw: Some(-4), y_raw: Some(9) }));
        assert_eq!(roundtrip(Action::ItemUse(proto::ItemUse { item_slot: 5, target_entity: Some(42), target_pos: point, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None })),
            Intent::ItemUse(proto::ItemUseIntent { item_slot: 5, target_render_id: 42, x_raw: -123, y_raw: 456, explicit_x_raw: None, explicit_y_raw: None }));
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
    fn item_use_explicit_origin_roundtrip_is_distinct_from_missing_point() {
        let origin = Some(proto::Vec2I { x: 0, y: 0 });
        let explicit = roundtrip(Action::ItemUse(proto::ItemUse {
            item_slot: 0, target_entity: None, target_pos: origin, point_explicit: Some(true), target_generation: Some(9),
            owner_entity: Some(3), owner_generation: Some(11),
        }));
        assert_eq!(explicit, Intent::ItemUse(proto::ItemUseIntent {
            item_slot: 0, target_render_id: 0, x_raw: 0, y_raw: 0,
            explicit_x_raw: Some(0), explicit_y_raw: Some(0),
        }));
        let missing = roundtrip(Action::ItemUse(proto::ItemUse {
            item_slot: 0, target_entity: None, target_pos: None, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None,
        }));
        let legacy_zero = roundtrip(Action::ItemUse(proto::ItemUse {
            item_slot: 0, target_entity: None, target_pos: origin, point_explicit: None, target_generation: None, owner_entity: None, owner_generation: None,
        }));
        assert_eq!(missing, legacy_zero);
        assert_ne!(explicit, missing);
        assert!(player_input_to_renderer_intent(&PlayerInput { action: Some(Action::ItemUse(proto::ItemUse {
            item_slot: 0, target_pos: None, target_entity: None, point_explicit: Some(true), target_generation: Some(9),
            owner_entity: Some(3), owner_generation: Some(11),
        })) }).is_none());
    }

    #[test]
    fn ability_index_four_and_five_stay_qwer_not_summoner() {
        for index in [4, 5] {
            let intent = roundtrip(Action::CastAbility(proto::CastAbility {
                ability_index: index, target_entity: None, target_pos: None,
            }));
            assert!(matches!(intent, Intent::AbilityCast(value) if value.ability_index == index));
        }
    }

    #[test]
    fn stop_command_renderer_roundtrip_has_no_coordinate() {
        let intent = roundtrip(Action::Stop(proto::Stop {}));
        assert_eq!(intent, Intent::Stop(proto::Stop {}));
        let encoded = PlayerInput { action: Some(Action::Stop(proto::Stop {})) }.encode_to_vec();
        let decoded = PlayerInput::decode(encoded.as_slice()).unwrap();
        assert_eq!(decoded.action, Some(Action::Stop(proto::Stop {})));
        assert!(matches!(player_input_to_renderer_intent(&decoded), Some(Intent::Stop(proto::Stop {}))));
    }

    #[test]
    fn shop_undo_sequence_roundtrips_and_newest_does_not_bind() {
        let pinned = roundtrip(Action::ShopUndo(proto::ShopUndo {
            mode: crate::runtime::shop_undo_wire::SHOP_UNDO_MODE_SEQUENCE,
            sequence: 9,
        }));
        assert_eq!(pinned, Intent::ShopUndo(proto::ShopUndoIntent { sequence: 9 }));
        for action in [
            Action::ShopUndo(proto::ShopUndo { mode: crate::runtime::shop_undo_wire::SHOP_UNDO_MODE_NEWEST, sequence: 0 }),
            Action::ShopUndo(proto::ShopUndo { mode: crate::runtime::shop_undo_wire::SHOP_UNDO_MODE_SEQUENCE, sequence: 0 }),
            Action::ShopUndo(proto::ShopUndo { mode: crate::runtime::shop_undo_wire::SHOP_UNDO_MODE_NEWEST, sequence: 9 }),
        ] {
            assert!(player_input_to_renderer_intent(&PlayerInput { action: Some(action) }).is_none());
        }
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
