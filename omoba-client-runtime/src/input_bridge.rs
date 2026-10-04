use omoba_core::game_proto::{
    player_input, renderer_input, AttackMove, AttackTarget, CastAbility, ItemUse, PlayerInput, RendererInput,
    TowerPlace, TowerSell, TowerUpgradeInput, Vec2I,
};

use crate::replica_host::ReplicaHost;

trait InputView {
    fn view_epoch(&self) -> u64;
    fn owns_hero(&self, player_id: u32) -> bool;
    fn secure_reference(
        &self,
        render_id: u64,
    ) -> Option<omoba_core::game_proto::SecureReplicaTarget>;
}

impl InputView for ReplicaHost {
    fn view_epoch(&self) -> u64 {
        self.view_epoch()
    }
    fn owns_hero(&self, player_id: u32) -> bool {
        self.owns_hero(player_id)
    }
    fn secure_reference(
        &self,
        render_id: u64,
    ) -> Option<omoba_core::game_proto::SecureReplicaTarget> {
        self.secure_reference(render_id)
    }
}

#[derive(Debug)]
pub enum InputDecision {
    RepeatedShop { request_id: u64, input_id: u32 },
    Accepted {
        input_id: u32,
        input: PlayerInput,
        secure_target: Option<omoba_core::game_proto::SecureReplicaTarget>,
    },
    Rejected {
        request_id: u64,
        code: &'static str,
    },
}

#[derive(Default)]
pub struct InputBridge {
    next_input_id: u32,
    shop_enabled: bool,
    recall_enabled: bool,
    shop_requests: std::collections::BTreeMap<u64, (RendererInput, u32)>,
    upgrade_requests: std::collections::BTreeMap<u64, RendererInput>,
}

impl InputBridge {
    pub fn resume_after(last_seen_input_id: u32) -> Self {
        Self {
            next_input_id: last_seen_input_id,
            shop_enabled: false,
            recall_enabled: false,
            shop_requests: Default::default(),
            upgrade_requests: Default::default(),
        }
    }

    pub fn set_shop_enabled(&mut self, enabled: bool) { self.shop_enabled = enabled; }
    pub fn set_recall_enabled(&mut self, enabled: bool) { self.recall_enabled = enabled; }

    pub fn allocate_input_id(&mut self) -> Result<u32, &'static str> {
        self.next_input_id = self
            .next_input_id
            .checked_add(1)
            .ok_or("INPUT_ID_EXHAUSTED")?;
        Ok(self.next_input_id)
    }

    pub fn validate(
        &mut self,
        renderer_input: RendererInput,
        configured_player_id: u32,
        replica: &ReplicaHost,
    ) -> InputDecision {
        self.validate_view(renderer_input, configured_player_id, replica)
    }

    fn validate_view(
        &mut self,
        renderer_input: RendererInput,
        configured_player_id: u32,
        replica: &impl InputView,
    ) -> InputDecision {
        if renderer_input.player_id != configured_player_id {
            return reject(renderer_input.request_id, "INVALID_OWNER");
        }
        if renderer_input.disclosure_epoch != replica.view_epoch() {
            return reject(renderer_input.request_id, "STALE_DISCLOSURE_EPOCH");
        }
        if !replica.owns_hero(configured_player_id) {
            return reject(renderer_input.request_id, "OWN_HERO_NOT_DISCLOSED");
        }
        if let Err(code) = validate_intent_shape(renderer_input.intent.as_ref()) {
            return reject(renderer_input.request_id, code);
        }
        let mut secure_target = None;
        let is_upgrade = matches!(renderer_input.intent, Some(renderer_input::Intent::AbilityUpgrade(_)));
        if is_upgrade {
            if renderer_input.request_id == 0 { return reject(0, "INVALID_UPGRADE_REQUEST_ID"); }
            if let Some(original) = self.upgrade_requests.get(&renderer_input.request_id) {
                return reject(renderer_input.request_id, if original == &renderer_input {
                    "REPEATED_UPGRADE_REQUEST_ID"
                } else { "CONFLICTING_UPGRADE_REQUEST_ID" });
            }
            if self.upgrade_requests.len() >= 1024 { return reject(renderer_input.request_id, "UPGRADE_REQUEST_HISTORY_CAPACITY"); }
        }
        let original_upgrade = is_upgrade.then(|| renderer_input.clone());
        if matches!(renderer_input.intent, Some(renderer_input::Intent::Recall(_))) {
            if !self.recall_enabled { return reject(renderer_input.request_id, "RECALL_PROTOCOL_UNAVAILABLE"); }
            if renderer_input.request_id == 0 { return reject(0, "INVALID_RECALL_REQUEST_ID"); }
        }
        let is_shop = matches!(renderer_input.intent, Some(renderer_input::Intent::ItemBuy(_)) | Some(renderer_input::Intent::ItemSell(_)));
        if is_shop
            && !self.shop_enabled { return reject(renderer_input.request_id, "SHOP_PROTOCOL_UNAVAILABLE"); }
        if is_shop {
            if renderer_input.request_id == 0 { return reject(0, "INVALID_SHOP_REQUEST_ID"); }
            if let Some((original, input_id)) = self.shop_requests.get(&renderer_input.request_id) {
                return if original == &renderer_input { InputDecision::RepeatedShop { request_id: renderer_input.request_id, input_id: *input_id } }
                    else { reject(renderer_input.request_id, "CONFLICTING_SHOP_REQUEST_ID") };
            }
            if self.shop_requests.len() >= 1024 { return reject(renderer_input.request_id, "SHOP_REQUEST_HISTORY_CAPACITY"); }
        }
        let original_shop = is_shop.then(|| renderer_input.clone());
        let action = match renderer_input.intent {
            Some(renderer_input::Intent::AbilityUpgrade(value)) => player_input::Action::UpgradeAbility(
                omoba_core::game_proto::UpgradeAbility {ability_index:value.ability_index}),
            Some(renderer_input::Intent::Recall(_)) => player_input::Action::Recall(omoba_core::game_proto::Recall {}),
            Some(renderer_input::Intent::ItemBuy(intent)) => {
                let Some(id) = omoba_core::runtime::shop_transport::item_id_for_catalog(intent.catalog_id) else {
                    return reject(renderer_input.request_id, "INVALID_ITEM_CATALOG_ID");
                };
                player_input::Action::ItemBuy(omoba_core::game_proto::ItemBuy { item_id: id.into() })
            }
            Some(renderer_input::Intent::ItemSell(intent)) => player_input::Action::ItemSell(
                omoba_core::game_proto::ItemSell { item_slot: intent.item_slot }),
            Some(renderer_input::Intent::MoveTo(intent)) => {
                player_input::Action::MoveTo(omoba_core::game_proto::MoveTo {
                    target: fixed_vec(intent.x_raw, intent.y_raw),
                    queued: false,
                })
            }
            Some(renderer_input::Intent::AttackMove(intent)) => {
                player_input::Action::AttackMove(AttackMove {
                    target: fixed_vec(intent.x_raw, intent.y_raw),
                    queued: false,
                })
            }
            Some(renderer_input::Intent::AbilityCast(intent)) => {
                let target_entity = match optional_target(intent.target_render_id, replica) {
                    Ok(value) => value,
                    Err(code) => return reject(renderer_input.request_id, code),
                };
                secure_target = (intent.target_render_id != 0)
                    .then(|| replica.secure_reference(intent.target_render_id))
                    .flatten();
                player_input::Action::CastAbility(CastAbility {
                    ability_index: intent.ability_index,
                    target_pos: fixed_vec(intent.x_raw, intent.y_raw),
                    target_entity,
                })
            }
            Some(renderer_input::Intent::ItemUse(intent)) => {
                let target_entity = match optional_target(intent.target_render_id, replica) {
                    Ok(value) => value,
                    Err(code) => return reject(renderer_input.request_id, code),
                };
                secure_target = (intent.target_render_id != 0)
                    .then(|| replica.secure_reference(intent.target_render_id))
                    .flatten();
                player_input::Action::ItemUse(ItemUse {
                    item_slot: intent.item_slot,
                    target_pos: fixed_vec(intent.x_raw, intent.y_raw),
                    target_entity,
                })
            }
            Some(renderer_input::Intent::AttackTarget(intent)) => {
                let Ok(target_id) = u32::try_from(intent.target_render_id) else {
                    return reject(renderer_input.request_id, "INVALID_TARGET");
                };
                secure_target = replica.secure_reference(intent.target_render_id);
                if target_id == 0 || secure_target.is_none() { return reject(renderer_input.request_id, "INVALID_TARGET"); }
                player_input::Action::AttackTarget(AttackTarget { target_id, queued: intent.queued })
            }
            Some(renderer_input::Intent::TowerAction(intent)) => match intent.action_kind {
                1 => player_input::Action::TowerPlace(TowerPlace {
                    tower_kind_id: intent.tower_kind_id,
                    pos: fixed_vec(intent.x_raw, intent.y_raw),
                }),
                2 => {
                    let Ok(tower_entity_id) = u32::try_from(intent.tower_render_id) else {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    };
                    if replica.secure_reference(intent.tower_render_id).is_none() {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    }
                    secure_target = replica.secure_reference(intent.tower_render_id);
                    player_input::Action::TowerUpgrade(TowerUpgradeInput {
                        tower_entity_id,
                        path: intent.path,
                        level: intent.level,
                    })
                }
                3 => {
                    let Ok(tower_entity_id) = u32::try_from(intent.tower_render_id) else {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    };
                    if replica.secure_reference(intent.tower_render_id).is_none() {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    }
                    secure_target = replica.secure_reference(intent.tower_render_id);
                    player_input::Action::TowerSell(TowerSell { tower_entity_id })
                }
                _ => return reject(renderer_input.request_id, "INVALID_ACTION"),
            },
            None => return reject(renderer_input.request_id, "MISSING_ACTION"),
        };
        let input_id = match self.allocate_input_id() {
            Ok(id) => id,
            Err(code) => return reject(renderer_input.request_id, code),
        };
        if let Some(original) = original_shop { self.shop_requests.insert(original.request_id, (original, input_id)); }
        if let Some(original) = original_upgrade { self.upgrade_requests.insert(original.request_id, original); }
        InputDecision::Accepted {
            input_id,
            input: PlayerInput {
                action: Some(action),
            },
            secure_target,
        }
    }
}

fn fixed_vec(x_raw: i64, y_raw: i64) -> Option<Vec2I> {
    // Shape validation rejects overflow before reaching this conversion.
    Some(Vec2I {
        x: i32::try_from(x_raw).expect("validated x coordinate"),
        y: i32::try_from(y_raw).expect("validated y coordinate"),
    })
}

fn optional_target(render_id: u64, replica: &impl InputView) -> Result<Option<u32>, &'static str> {
    if render_id == 0 {
        return Ok(None);
    }
    if replica.secure_reference(render_id).is_none() {
        return Err("INVALID_TARGET");
    }
    u32::try_from(render_id)
        .map(Some)
        .map_err(|_| "INVALID_TARGET")
}

fn validate_intent_shape(intent: Option<&renderer_input::Intent>) -> Result<(), &'static str> {
    let (x, y) = match intent {
        Some(renderer_input::Intent::AbilityUpgrade(value)) => return
            if value.ability_index<4 {Ok(())} else {Err("INVALID_ABILITY_SLOT")},
        Some(renderer_input::Intent::Recall(_)) => return Ok(()),
        Some(renderer_input::Intent::ItemBuy(value)) => return
            omoba_core::runtime::shop_transport::item_id_for_catalog(value.catalog_id)
                .map(|_| ()).ok_or("INVALID_ITEM_CATALOG_ID"),
        Some(renderer_input::Intent::ItemSell(value)) => return
            if value.item_slot < 6 { Ok(()) } else { Err("INVALID_ITEM_SLOT") },
        Some(renderer_input::Intent::MoveTo(value)) => (value.x_raw, value.y_raw),
        Some(renderer_input::Intent::AttackMove(value)) => (value.x_raw, value.y_raw),
        Some(renderer_input::Intent::AbilityCast(value)) => {
            if value.ability_index >= 4 {
                return Err("INVALID_ABILITY_SLOT");
            }
            (value.x_raw, value.y_raw)
        }
        Some(renderer_input::Intent::ItemUse(value)) => {
            if value.item_slot >= 6 {
                return Err("INVALID_ITEM_SLOT");
            }
            (value.x_raw, value.y_raw)
        }
        Some(renderer_input::Intent::AttackTarget(value)) => {
            return if value.target_render_id > 0 && value.target_render_id <= u64::from(u32::MAX) { Ok(()) } else { Err("INVALID_TARGET") };
        }
        Some(renderer_input::Intent::TowerAction(value)) => {
            if value.action_kind != 1 {
                return Ok(());
            }
            (value.x_raw, value.y_raw)
        }
        None => return Err("MISSING_ACTION"),
    };
    if i32::try_from(x).is_err() || i32::try_from(y).is_err() {
        return Err("INVALID_POSITION");
    }
    Ok(())
}

fn reject(request_id: u64, code: &'static str) -> InputDecision {
    InputDecision::Rejected { request_id, code }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upgrade_roundtrips_and_rejects_invalid_owner_epoch_slot_and_id() {
        use prost::Message;
        use omoba_core::game_proto::AbilityUpgradeIntent;
        let view = view(1);
        let mut bridge = InputBridge::default();
        let input = request(&view, renderer_input::Intent::AbilityUpgrade(AbilityUpgradeIntent { ability_index: 0 }));
        let input = RendererInput::decode(input.encode_to_vec().as_slice()).unwrap();
        assert_rejected(bridge.validate_view(input.clone(), 8, &view), "INVALID_OWNER");
        let mut stale = input.clone(); stale.disclosure_epoch += 1;
        assert_rejected(bridge.validate_view(stale, 7, &view), "STALE_DISCLOSURE_EPOCH");
        let mut invalid = input.clone(); invalid.intent = Some(renderer_input::Intent::AbilityUpgrade(AbilityUpgradeIntent { ability_index: 4 }));
        assert_rejected(bridge.validate_view(invalid, 7, &view), "INVALID_ABILITY_SLOT");
        let mut zero = input.clone(); zero.request_id = 0;
        assert!(matches!(bridge.validate_view(zero, 7, &view), InputDecision::Rejected { request_id: 0, code: "INVALID_UPGRADE_REQUEST_ID" }));
        for slot in 0..4 {
            let mut valid = input.clone();
            valid.request_id += u64::from(slot);
            valid.intent = Some(renderer_input::Intent::AbilityUpgrade(AbilityUpgradeIntent { ability_index: slot }));
            match bridge.validate_view(valid, 7, &view) {
                InputDecision::Accepted { input_id, input, secure_target: None } => {
                    assert_eq!(input_id, slot + 1);
                    assert!(matches!(input.action, Some(player_input::Action::UpgradeAbility(value)) if value.ability_index == slot));
                }
                other => panic!("unexpected upgrade decision: {other:?}"),
            }
        }
        assert_rejected(bridge.validate_view(input.clone(), 7, &view), "REPEATED_UPGRADE_REQUEST_ID");
        let mut conflict = input;
        conflict.intent = Some(renderer_input::Intent::AbilityUpgrade(AbilityUpgradeIntent { ability_index: 1 }));
        assert_rejected(bridge.validate_view(conflict, 7, &view), "CONFLICTING_UPGRADE_REQUEST_ID");
        assert_eq!(bridge.allocate_input_id(), Ok(5));
    }
    #[test]
    fn recall_is_independent_and_preserves_owner_epoch_and_request_id() {
        let view = view(1);
        let mut bridge = InputBridge::default();
        bridge.set_shop_enabled(true);
        let input = request(&view, renderer_input::Intent::Recall(omoba_core::game_proto::RecallIntent {}));
        assert_rejected(bridge.validate_view(input.clone(), 7, &view), "RECALL_PROTOCOL_UNAVAILABLE");
        bridge.set_recall_enabled(true);
        assert_rejected(bridge.validate_view(input.clone(), 8, &view), "INVALID_OWNER");
        let mut stale = input.clone(); stale.disclosure_epoch += 1;
        assert_rejected(bridge.validate_view(stale, 7, &view), "STALE_DISCLOSURE_EPOCH");
        let mut zero = input.clone(); zero.request_id = 0;
        assert!(matches!(bridge.validate_view(zero, 7, &view), InputDecision::Rejected { request_id: 0, code: "INVALID_RECALL_REQUEST_ID" }));
        assert!(matches!(bridge.validate_view(input, 7, &view), InputDecision::Accepted {
            input_id: 1, input: PlayerInput { action: Some(player_input::Action::Recall(_)) }, secure_target: None
        }));
    }
    #[test]
    fn attack_target_requires_disclosed_reference_and_preserves_queue() {
        use omoba_core::game_proto::AttackTargetIntent;
        let view = view(1);
        let id = *view.runtime.world().entities.keys().next().unwrap();
        let mut bridge = InputBridge::default();
        let input = request(&view, renderer_input::Intent::AttackTarget(AttackTargetIntent { target_render_id: id, queued: true }));
        match bridge.validate_view(input.clone(), 7, &view) {
            InputDecision::Accepted { input, secure_target: Some(reference), .. } => {
                let Some(player_input::Action::AttackTarget(target)) = input.action else { panic!("wrong action"); };
                assert_eq!(u64::from(target.target_id), id);
                assert!(target.queued);
                assert_eq!(reference.replica_entity_id.unwrap().value, id);
            }
            other => panic!("expected bound target: {other:?}"),
        }
        for target_render_id in [0, 9999, u64::from(u32::MAX) + 1] {
            let mut invalid = input.clone();
            invalid.intent = Some(renderer_input::Intent::AttackTarget(AttackTargetIntent { target_render_id, queued: false }));
            assert!(matches!(bridge.validate_view(invalid, 7, &view), InputDecision::Rejected { .. }));
        }
        assert!(matches!(bridge.validate_view(input, 8, &view), InputDecision::Rejected { .. }));
    }
    #[test]
    fn shop_intents_require_capability_catalog_slot_owner_and_epoch() {
        use omoba_core::game_proto::{ItemBuyIntent, ItemSellIntent};
        let view = view(1);
        let mut bridge = InputBridge::default();
        let buy = || request(&view, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 }));
        assert_rejected(bridge.validate_view(buy(), 7, &view), "SHOP_PROTOCOL_UNAVAILABLE");
        bridge.set_shop_enabled(true);
        match bridge.validate_view(buy(), 7, &view) {
            InputDecision::Accepted { input, input_id: 1, secure_target: None } =>
                assert!(matches!(input.action, Some(player_input::Action::ItemBuy(value)) if value.item_id == "moba_sword")),
            other => panic!("unexpected {other:?}"),
        }
        assert!(matches!(bridge.validate_view(buy(), 7, &view), InputDecision::RepeatedShop { input_id: 1, .. }));
        let mut changed = buy(); changed.intent = Some(renderer_input::Intent::ItemSell(ItemSellIntent { item_slot: 0 }));
        assert_rejected(bridge.validate_view(changed, 7, &view), "CONFLICTING_SHOP_REQUEST_ID");
        assert_rejected(bridge.validate_view(request(&view, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: u32::MAX })), 7, &view), "INVALID_ITEM_CATALOG_ID");
        assert_rejected(bridge.validate_view(request(&view, renderer_input::Intent::ItemSell(ItemSellIntent { item_slot: 6 })), 7, &view), "INVALID_ITEM_SLOT");
        assert_rejected(bridge.validate_view(buy(), 8, &view), "INVALID_OWNER");
        let mut stale = buy(); stale.disclosure_epoch += 1;
        assert_rejected(bridge.validate_view(stale, 7, &view), "STALE_DISCLOSURE_EPOCH");
        assert_eq!(bridge.allocate_input_id(), Ok(2), "invalid intents must not consume IDs");
    }
    #[test]
    fn resumed_allocator_never_wraps_or_reuses_the_server_floor() {
        let mut bridge = InputBridge::resume_after(42);
        assert_eq!(bridge.allocate_input_id(), Ok(43));
        let mut bridge = InputBridge::resume_after(u32::MAX - 1);
        assert_eq!(bridge.allocate_input_id(), Ok(u32::MAX));
        assert_eq!(bridge.allocate_input_id(), Err("INPUT_ID_EXHAUSTED"));
        assert_eq!(bridge.allocate_input_id(), Err("INPUT_ID_EXHAUSTED"));
        let view = view(1);
        assert_rejected(
            bridge.validate_view(
                request(
                    &view,
                    renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 1, y_raw: 2 }),
                ),
                7,
                &view,
            ),
            "INPUT_ID_EXHAUSTED",
        );
        assert_eq!(bridge.next_input_id, u32::MAX);
    }
    use omoba_core::{
        game_proto::{
            AbilityCastIntent, AttackMoveIntent, DisclosureEpoch, ItemUseIntent, MoveToIntent,
            ReplicaEntityId, SecureReplicaTarget, ViewEpoch,
        },
        runtime::{
            decode_demo_render_state, encode_component_baseline, encode_demo_render_state,
            secure_replica_component_allowlist, secure_replica_resource_allowlist, DemoRenderState,
            ProjectionDependencyGraph, SelectiveReplicaRuntime, TeamProjectorConfig,
            TeamViewProjector, VisibilityTransition, DEMO_RENDER_COMPONENT_SCHEMA_ID,
        },
    };
    use std::collections::BTreeSet;

    struct DisclosedView {
        runtime: SelectiveReplicaRuntime,
        team: u32,
    }

    impl InputView for DisclosedView {
        fn view_epoch(&self) -> u64 {
            self.runtime.view_epoch()
        }
        fn owns_hero(&self, player: u32) -> bool {
            self.runtime.world().entities.values().any(|entity| {
                entity
                    .components
                    .get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)
                    .and_then(|bytes| decode_demo_render_state(bytes))
                    .is_some_and(|render| {
                        render.team_id == self.team
                            && render.kind == 1
                            && render.owner_player_id == player
                    })
            })
        }
        fn secure_reference(&self, id: u64) -> Option<SecureReplicaTarget> {
            let entity = self.runtime.world().entities.get(&id)?;
            Some(SecureReplicaTarget {
                replica_entity_id: Some(ReplicaEntityId { value: id }),
                view_epoch: Some(ViewEpoch {
                    value: self.view_epoch(),
                }),
                disclosure_epoch: Some(DisclosureEpoch {
                    value: entity.disclosure_epoch,
                }),
            })
        }
    }

    fn view(team: u32) -> DisclosedView {
        let mut projector = TeamViewProjector::new(team, TeamProjectorConfig::default());
        let render = encode_demo_render_state(DemoRenderState {
            x_raw: 0,
            y_raw: 0,
            team_id: team,
            kind: 1,
            owner_player_id: 7,
        });
        projector
            .build_frame(
                0,
                0,
                &BTreeSet::from([10]),
                vec![VisibilityTransition::Reveal {
                    canonical_id: 10,
                    effective_tick: 0,
                    baseline: encode_component_baseline(&[(
                        DEMO_RENDER_COMPONENT_SCHEMA_ID,
                        &render,
                    )]),
                }],
                &[],
                &ProjectionDependencyGraph::default(),
            )
            .unwrap();
        let start = projector.build_team_game_start(1, 120, 42);
        DisclosedView {
            runtime: SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                &start,
                secure_replica_component_allowlist(),
                secure_replica_resource_allowlist(),
            )
            .unwrap(),
            team,
        }
    }

    fn request(view: &DisclosedView, intent: renderer_input::Intent) -> RendererInput {
        RendererInput {
            request_id: 91,
            player_id: 7,
            disclosure_epoch: view.view_epoch(),
            intent: Some(intent),
        }
    }

    fn assert_rejected(decision: InputDecision, expected: &str) {
        match decision {
            InputDecision::Rejected { request_id, code } => {
                assert_eq!(request_id, 91);
                assert_eq!(code, expected);
            }
            other => panic!("expected {expected}, got {other:?}"),
        }
    }

    #[test]
    fn rejects_wrong_owner_stale_view_and_missing_owned_hero() {
        let view = view(1);
        let mut bridge = InputBridge::default();
        let input = request(
            &view,
            renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 1, y_raw: 2 }),
        );
        let mut wrong = input.clone();
        wrong.player_id = 8;
        assert_rejected(bridge.validate_view(wrong, 7, &view), "INVALID_OWNER");
        let mut stale = input.clone();
        stale.disclosure_epoch += 1;
        assert_rejected(
            bridge.validate_view(stale, 7, &view),
            "STALE_DISCLOSURE_EPOCH",
        );
        let mut no_hero = input;
        no_hero.player_id = 8;
        assert_rejected(
            bridge.validate_view(no_hero, 8, &view),
            "OWN_HERO_NOT_DISCLOSED",
        );
        assert_eq!(bridge.next_input_id, 0);
    }

    #[test]
    fn casts_only_disclosed_targets_with_secure_identity() {
        let view = view(1);
        let id = *view.runtime.world().entities.keys().next().unwrap();
        let mut bridge = InputBridge::default();
        let cast = |target| {
            request(
                &view,
                renderer_input::Intent::AbilityCast(AbilityCastIntent {
                    ability_index: 3,
                    target_render_id: target,
                    x_raw: 100,
                    y_raw: -200,
                }),
            )
        };
        assert_rejected(
            bridge.validate_view(cast(u64::MAX), 7, &view),
            "INVALID_TARGET",
        );
        match bridge.validate_view(cast(id), 7, &view) {
            InputDecision::Accepted {
                input_id,
                input,
                secure_target,
            } => {
                assert_eq!(input_id, 1);
                assert_eq!(secure_target, view.secure_reference(id));
                assert!(
                    matches!(input.action, Some(player_input::Action::CastAbility(CastAbility { ability_index: 3, target_entity: Some(target), .. })) if u64::from(target) == id)
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            bridge.validate_view(cast(0), 7, &view),
            InputDecision::Accepted {
                secure_target: None,
                ..
            }
        ));
    }

    #[test]
    fn rejects_slot_and_position_overflow_without_consuming_input_ids() {
        let view = view(1);
        let mut bridge = InputBridge::default();
        for (intent, code) in [
            (
                renderer_input::Intent::AbilityCast(AbilityCastIntent {
                    ability_index: 4,
                    ..Default::default()
                }),
                "INVALID_ABILITY_SLOT",
            ),
            (
                renderer_input::Intent::ItemUse(ItemUseIntent {
                    item_slot: 6,
                    ..Default::default()
                }),
                "INVALID_ITEM_SLOT",
            ),
            (
                renderer_input::Intent::MoveTo(MoveToIntent {
                    x_raw: i64::MAX,
                    y_raw: 0,
                }),
                "INVALID_POSITION",
            ),
            (
                renderer_input::Intent::AttackMove(AttackMoveIntent {
                    x_raw: 0,
                    y_raw: i64::MIN,
                }),
                "INVALID_POSITION",
            ),
        ] {
            assert_rejected(bridge.validate_view(request(&view, intent), 7, &view), code);
        }
        assert_eq!(bridge.next_input_id, 0);
        let input = request(
            &view,
            renderer_input::Intent::ItemUse(ItemUseIntent {
                item_slot: 5,
                x_raw: i64::from(i32::MAX),
                y_raw: i64::from(i32::MIN),
                ..Default::default()
            }),
        );
        assert!(matches!(
            bridge.validate_view(input, 7, &view),
            InputDecision::Accepted { input_id: 1, .. }
        ));
    }
}
