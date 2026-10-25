use omoba_core::game_proto::{
    player_input, renderer_input, AttackMove, AttackTarget, CastAbility, ItemUse, PlayerInput, RendererInput,
    TowerPlace, TowerSell, TowerUpgradeInput, Vec2I,
};

use crate::replica_host::ReplicaHost;

trait InputView {
    fn owned_summon_command(&self,_player_id:u32,_render_id:u64,_operation:u32,_point:Option<Vec2I>)->Option<PlayerInput>{None}
    fn combat_preferences_available(&self, player_id: u32) -> bool;
    fn input_scheme_for_owner(&self, player_id: u32) -> Option<omoba_core::runtime::input_scheme_transport::AdmittedOwnerInputScheme>;
    fn view_epoch(&self) -> u64;
    fn owns_hero(&self, player_id: u32) -> bool;
    fn owned_hero_reference(&self, player_id:u32) -> Option<omoba_core::game_proto::SecureReplicaTarget>;
    fn secure_reference(
        &self,
        render_id: u64,
    ) -> Option<omoba_core::game_proto::SecureReplicaTarget>;
    fn attack_relation_legal(&self, source_player_id: u32, render_id: u64) -> bool;
}

impl InputView for ReplicaHost {
    fn owned_summon_command(&self,player_id:u32,render_id:u64,operation:u32,point:Option<Vec2I>)->Option<PlayerInput> {
        self.owned_summon_command(player_id,render_id,operation,point)
    }
    fn owned_hero_reference(&self, player_id:u32) -> Option<omoba_core::game_proto::SecureReplicaTarget> {
        ReplicaHost::owned_hero_reference(self,player_id)
    }
    fn input_scheme_for_owner(&self, player_id: u32) -> Option<omoba_core::runtime::input_scheme_transport::AdmittedOwnerInputScheme> {
        ReplicaHost::input_scheme_for_owner(self,player_id)
    }
    fn combat_preferences_available(&self, player_id: u32) -> bool {
        self.combat_preferences_for_owner(player_id).is_some()
    }
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
    fn attack_relation_legal(&self, source_player_id: u32, render_id: u64) -> bool {
        self.attack_relation_legal(source_player_id, render_id)
    }
}

#[derive(Debug)]
pub enum InputDecision {
    RepeatedShop { request_id: u64, input_id: u32 },
    /// The same renderer request already has an authority id. Do not submit it again.
    RepeatedWard { request_id: u64, input_id: u32 },
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

/// Current owner fact for a view epoch. Built only from a decoded economy payload
/// and, when present, a decoded undo projection. Absence is not permission.
#[derive(Clone, Debug)]
pub struct OwnerShopDisclosure {
    pub epoch: u64,
    pub economy: omoba_core::runtime::native::economy_projection::OwnerEconomyState,
    pub undo: Option<omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection>,
}

#[derive(Default)]
pub struct InputBridge {
    next_input_id: u32,
    shop_enabled: bool,
    undo_enabled: bool,
    recall_enabled: bool,
    commands_enabled: bool,
    summoner_enabled: bool,
    ward_enabled: bool,
    disclosure: Option<OwnerShopDisclosure>,
    shop_requests: std::collections::BTreeMap<u64, (RendererInput, u32)>,
    upgrade_requests: std::collections::BTreeMap<u64, RendererInput>,
    ward_requests: std::collections::VecDeque<(RendererInput, u32)>,
    ward_tombstones: std::collections::VecDeque<u64>,
}

impl InputBridge {
    pub fn resume_after(last_seen_input_id: u32) -> Self {
        Self {
            next_input_id: last_seen_input_id,
            shop_enabled: false,
            undo_enabled: false,
            recall_enabled: false,
            commands_enabled: false,
            summoner_enabled: false,
            ward_enabled: false,
            disclosure: None,
            shop_requests: Default::default(),
            upgrade_requests: Default::default(),
            ward_requests: Default::default(),
            ward_tombstones: Default::default(),
        }
    }

    pub fn ward_authority_for(&self, request_id: u64) -> Option<u32> {
        self.ward_requests.iter().find(|(input, _)| input.request_id == request_id).map(|(_, id)| *id)
    }

    pub fn reserved_input_id(&self) -> u32 {
        self.next_input_id
    }

    pub fn set_shop_enabled(&mut self, enabled: bool) { self.shop_enabled = enabled; }
    pub fn set_shop_undo_enabled(&mut self, enabled: bool) { self.undo_enabled = enabled; }
    pub fn set_recall_enabled(&mut self, enabled: bool) { self.recall_enabled = enabled; }
    pub fn set_commands_enabled(&mut self, enabled: bool) { self.commands_enabled = enabled; }
    pub fn set_summoner_enabled(&mut self, enabled: bool) { self.summoner_enabled = enabled; }
    pub fn set_ward_enabled(&mut self, enabled: bool) { self.ward_enabled = enabled; }
    pub fn set_owner_shop_disclosure(&mut self, disclosure: Option<OwnerShopDisclosure>) {
        self.disclosure = disclosure;
    }

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
        if let Err(code) = validate_intent_shape(renderer_input.intent.as_ref()) {
            return reject(renderer_input.request_id, code);
        }
        let combat_preferences = matches!(renderer_input.intent, Some(renderer_input::Intent::CombatPreferences(_)));
        let input_scheme = matches!(renderer_input.intent, Some(renderer_input::Intent::InputScheme(_)));
        if input_scheme {
            if renderer_input.request_id==0 {return reject(0,"INVALID_INPUT_SCHEME_REQUEST");}
            let Some(state)=replica.input_scheme_for_owner(configured_player_id) else {return reject(renderer_input.request_id,"INPUT_SCHEME_UNAVAILABLE");};
            if let Some(renderer_input::Intent::InputScheme(value))=renderer_input.intent.as_ref() {
                if state.locked() && value.scheme!=state.scheme as u32 {return reject(renderer_input.request_id,"INPUT_SCHEME_LOCKED");}
            }
        }
        if combat_preferences {
            if renderer_input.request_id == 0 { return reject(0, "INVALID_COMBAT_PREFERENCES_REQUEST"); }
            if !replica.combat_preferences_available(configured_player_id) {
                return reject(renderer_input.request_id, "COMBAT_PREFERENCES_UNAVAILABLE");
            }
        }
        if !combat_preferences && !input_scheme && !replica.owns_hero(configured_player_id) {
            if let Err(code) = self.admit_retained_owner_shop(&renderer_input, configured_player_id, replica.view_epoch()) {
                return reject(renderer_input.request_id, code);
            }
        }
        let mut secure_target = None;
        if matches!(renderer_input.intent, Some(renderer_input::Intent::HoldPosition(_)) | Some(renderer_input::Intent::Stop(_)) | Some(renderer_input::Intent::SummonCommand(_)))
            && !self.commands_enabled { return reject(renderer_input.request_id, "COMMAND_PROTOCOL_UNAVAILABLE"); }
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
        if matches!(renderer_input.intent, Some(renderer_input::Intent::SummonerCast(_))) {
            if !self.summoner_enabled { return reject(renderer_input.request_id, "SUMMONER_PROTOCOL_UNAVAILABLE"); }
            if renderer_input.request_id == 0 { return reject(0, "INVALID_SUMMONER_REQUEST_ID"); }
        }
        let ward_record = if matches!(renderer_input.intent, Some(renderer_input::Intent::WardPlacement(_))) {
            if !self.ward_enabled { return reject(renderer_input.request_id, "WARD_PROTOCOL_UNAVAILABLE"); }
            if renderer_input.request_id == 0 { return reject(0, "INVALID_WARD_REQUEST_ID"); }
            if self.ward_tombstones.contains(&renderer_input.request_id) {
                return reject(renderer_input.request_id, "WARD_REQUEST_EVICTED");
            }
            if let Some((_, input_id)) = self.ward_requests.iter().find(|(input, _)| input.request_id == renderer_input.request_id) {
                let input_id = *input_id;
                return if self.ward_requests.iter().any(|(input, _)| input == &renderer_input) {
                    InputDecision::RepeatedWard { request_id: renderer_input.request_id, input_id }
                } else {
                    reject(renderer_input.request_id, "CONFLICTING_WARD_REQUEST_ID")
                };
            }
            Some(renderer_input.clone())
        } else {
            None
        };
        let is_shop = matches!(renderer_input.intent, Some(renderer_input::Intent::ItemBuy(_)) | Some(renderer_input::Intent::ItemSell(_)));
        let is_undo = matches!(renderer_input.intent, Some(renderer_input::Intent::ShopUndo(_)));
        if is_shop
            && !self.shop_enabled { return reject(renderer_input.request_id, "SHOP_PROTOCOL_UNAVAILABLE"); }
        if is_undo && !self.undo_enabled { return reject(renderer_input.request_id, "UNDO_PROTOCOL_UNAVAILABLE"); }
        let is_correlated = is_shop || is_undo;
        if is_correlated {
            if renderer_input.request_id == 0 { return reject(0, "INVALID_SHOP_REQUEST_ID"); }
            if let Some((original, input_id)) = self.shop_requests.get(&renderer_input.request_id) {
                return if original == &renderer_input { InputDecision::RepeatedShop { request_id: renderer_input.request_id, input_id: *input_id } }
                    else { reject(renderer_input.request_id, "CONFLICTING_SHOP_REQUEST_ID") };
            }
            if self.shop_requests.len() >= 1024 { return reject(renderer_input.request_id, "SHOP_REQUEST_HISTORY_CAPACITY"); }
        }
        let original_shop = is_correlated.then(|| renderer_input.clone());
        let action = match renderer_input.intent {
            Some(renderer_input::Intent::SummonCommand(value)) => {
                if renderer_input.request_id==0 {return reject(0,"INVALID_SUMMON_REQUEST");}
                let id=value.summon.as_ref().and_then(|r|r.replica_entity_id.as_ref()).map_or(0,|r|r.value);
                let Some(expected)=replica.owned_summon_command(configured_player_id,id,value.operation,value.point) else {
                    return reject(renderer_input.request_id,"SUMMON_SOURCE_UNAVAILABLE");
                };
                let supplied=PlayerInput{action:Some(player_input::Action::SummonCommand(value))};
                if supplied!=expected {return reject(renderer_input.request_id,"STALE_SUMMON_SOURCE");}
                expected.action.expect("constructed summon client action")
            }
            Some(renderer_input::Intent::AbilityUpgrade(value)) => player_input::Action::UpgradeAbility(
                omoba_core::game_proto::UpgradeAbility {ability_index:value.ability_index}),
            Some(renderer_input::Intent::Recall(_)) => player_input::Action::Recall(omoba_core::game_proto::Recall {}),
            Some(renderer_input::Intent::SummonerCast(intent)) => player_input::Action::SummonerCast(
                omoba_core::game_proto::SummonerCast { slot: intent.slot, x_raw: intent.x_raw, y_raw: intent.y_raw }),
            Some(renderer_input::Intent::WardPlacement(intent)) => {
                if let Err(code) = omoba_core::runtime::ward_transport::admit_ward_placement(
                    intent.slot, intent.x_raw, intent.y_raw,
                ) {
                    return reject(renderer_input.request_id, code);
                }
                player_input::Action::WardPlacement(omoba_core::game_proto::WardPlacement {
                    slot: intent.slot, x_raw: intent.x_raw, y_raw: intent.y_raw,
                })
            }
            Some(renderer_input::Intent::HoldPosition(value)) => player_input::Action::HoldPosition(value.clone()),
            Some(renderer_input::Intent::Stop(_)) => player_input::Action::Stop(omoba_core::game_proto::Stop {}),
            Some(renderer_input::Intent::CombatPreferences(value)) => player_input::Action::CombatPreferences(value),
            Some(renderer_input::Intent::InputScheme(value)) => player_input::Action::InputScheme(value),
            Some(renderer_input::Intent::Direction(value)) => {
                if renderer_input.request_id==0 {return reject(0,"INVALID_DIRECTION_REQUEST");}
                let Some(state)=replica.input_scheme_for_owner(configured_player_id) else {return reject(renderer_input.request_id,"DIRECTION_UNAVAILABLE");};
                if state.direction_protocol!=1 || state.scheme!=omoba_core::runtime::input_scheme::InputScheme::Keyboard {
                    return reject(renderer_input.request_id,"DIRECTION_UNAVAILABLE");
                }
                // End remains usable while paused. Never gate heartbeat/release
                // on a delayed active_press projection: authority owns retirement.
                if value.phase!=3 && state.direction_step.is_none() {return reject(renderer_input.request_id,"DIRECTION_INACTIVE");}
                let Some(source)=replica.owned_hero_reference(configured_player_id) else {return reject(renderer_input.request_id,"DIRECTION_SOURCE");};
                let client=match omoba_core::runtime::direction_transport::client_from_renderer(&value,source) {
                    Ok(client)=>client,Err(reason)=>return reject(renderer_input.request_id,reason),
                };
                player_input::Action::Direction(client)
            }
            Some(renderer_input::Intent::ItemBuy(intent)) => {
                let Some(id) = omoba_core::runtime::shop_transport::item_id_for_catalog(intent.catalog_id) else {
                    return reject(renderer_input.request_id, "INVALID_ITEM_CATALOG_ID");
                };
                player_input::Action::ItemBuy(omoba_core::game_proto::ItemBuy { item_id: id.into() })
            }
            Some(renderer_input::Intent::ItemSell(intent)) => player_input::Action::ItemSell(
                omoba_core::game_proto::ItemSell { item_slot: intent.item_slot }),
            Some(renderer_input::Intent::ShopUndo(intent)) => player_input::Action::ShopUndo(
                omoba_core::game_proto::ShopUndo {
                    mode: omoba_core::runtime::shop_undo_wire::SHOP_UNDO_MODE_SEQUENCE,
                    sequence: intent.sequence,
                }),
            Some(renderer_input::Intent::MoveTo(intent)) => {
                player_input::Action::MoveTo(omoba_core::game_proto::MoveTo {
                    target: fixed_vec(intent.x_raw, intent.y_raw),
                    queued: intent.queued,
                })
            }
            Some(renderer_input::Intent::AttackMove(intent)) => {
                player_input::Action::AttackMove(AttackMove {
                    target: fixed_vec(intent.x_raw, intent.y_raw),
                    queued: intent.queued,
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
                let (target_pos, point_explicit) = match item_point_from_intent(&intent) {
                    Ok(point) => point,
                    Err(code) => return reject(renderer_input.request_id, code),
                };
                player_input::Action::ItemUse(ItemUse {
                    item_slot: intent.item_slot,
                    target_pos,
                    target_entity,
                    point_explicit,
                    target_generation: None, owner_entity: None, owner_generation: None,
                })
            }
            Some(renderer_input::Intent::AttackTarget(intent)) => {
                let Ok(target_id) = u32::try_from(intent.target_render_id) else {
                    return reject(renderer_input.request_id, "INVALID_TARGET");
                };
                secure_target = replica.secure_reference(intent.target_render_id);
                if target_id == 0 || secure_target.is_none()
                    || !replica.attack_relation_legal(configured_player_id, intent.target_render_id) {
                    return reject(renderer_input.request_id, "INVALID_TARGET");
                }
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
            Some(renderer_input::Intent::Communication(_)) => return reject(renderer_input.request_id, "COMMUNICATION_IS_NOT_GAMEPLAY"),
            None => return reject(renderer_input.request_id, "MISSING_ACTION"),
        };
        let input_id = match self.allocate_input_id() {
            Ok(id) => id,
            Err(code) => return reject(renderer_input.request_id, code),
        };
        if let Some(original) = original_shop { self.shop_requests.insert(original.request_id, (original, input_id)); }
        if let Some(original) = original_upgrade { self.upgrade_requests.insert(original.request_id, original); }
        if let Some(original) = ward_record { self.remember_ward(original, input_id); }
        InputDecision::Accepted {
            input_id,
            input: PlayerInput {
                action: Some(action),
            },
            secure_target,
        }
    }

    fn remember_ward(&mut self, input: RendererInput, authority: u32) {
        const CAPACITY: usize = 8;
        if self.ward_requests.len() == CAPACITY {
            if let Some((old, _)) = self.ward_requests.pop_front() {
                if self.ward_tombstones.len() == CAPACITY {
                    self.ward_tombstones.pop_front();
                }
                self.ward_tombstones.push_back(old.request_id);
            }
        }
        self.ward_requests.push_back((input, authority));
    }

    /// Buy, sell, and undo only. A missing hero is not itself permission.
    fn admit_retained_owner_shop(
        &self,
        renderer_input: &RendererInput,
        configured_player_id: u32,
        view_epoch: u64,
    ) -> Result<(), &'static str> {
        let shop = matches!(renderer_input.intent, Some(renderer_input::Intent::ItemBuy(_)) | Some(renderer_input::Intent::ItemSell(_)));
        let undo = matches!(renderer_input.intent, Some(renderer_input::Intent::ShopUndo(_)));
        if !shop && !undo {
            return Err("OWN_HERO_NOT_DISCLOSED");
        }
        let Some(fact) = &self.disclosure else {
            return Err("OWN_HERO_NOT_DISCLOSED");
        };
        if fact.epoch != view_epoch {
            return Err("STALE_DISCLOSURE_EPOCH");
        }
        if fact.economy.player_id != configured_player_id {
            return Err("INVALID_OWNER");
        }
        if shop {
            if !self.shop_enabled {
                return Err("SHOP_PROTOCOL_UNAVAILABLE");
            }
            if !(fact.economy.shop_available
                && fact.economy.shop_reason == omoba_core::runtime::native::economy_projection::SHOP_REASON_AVAILABLE)
            {
                return Err("SHOP_AUTHORITY_CLOSED");
            }
            return Ok(());
        }
        if !self.undo_enabled {
            return Err("UNDO_PROTOCOL_UNAVAILABLE");
        }
        let Some(projection) = &fact.undo else {
            return Err("UNDO_AUTHORITY_CLOSED");
        };
        if projection.player_id != configured_player_id {
            return Err("INVALID_OWNER");
        }
        let sequence = match &renderer_input.intent {
            Some(renderer_input::Intent::ShopUndo(intent)) => intent.sequence,
            _ => return Err("UNDO_AUTHORITY_CLOSED"),
        };
        if projection.can_undo
            && projection.reason == omoba_core::runtime::shop::UNDO_REASON_CAN_UNDO
            && projection.newest_sequence == sequence
            && sequence != 0
        {
            return Ok(());
        }
        Err("UNDO_AUTHORITY_CLOSED")
    }
}

fn item_point_from_intent(intent: &omoba_core::game_proto::ItemUseIntent) -> Result<(Option<Vec2I>, Option<bool>), &'static str> {
    match (intent.explicit_x_raw, intent.explicit_y_raw) {
        (Some(x), Some(y)) => Ok((Some(bounded_vec(x, y)?), Some(true))),
        (None, None) => Ok((fixed_vec(intent.x_raw, intent.y_raw), None)),
        _ => Err("INVALID_POSITION"),
    }
}

fn bounded_vec(x_raw: i64, y_raw: i64) -> Result<Vec2I, &'static str> {
    Ok(Vec2I {
        x: i32::try_from(x_raw).map_err(|_| "INVALID_POSITION")?,
        y: i32::try_from(y_raw).map_err(|_| "INVALID_POSITION")?,
    })
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
        Some(renderer_input::Intent::SummonCommand(value)) => return
            omoba_core::runtime::summon_transport::validate_client(value),
        Some(renderer_input::Intent::CombatPreferences(value)) => return
            omoba_core::runtime::combat_transport::validate_update(value).map(|_| ()),
        Some(renderer_input::Intent::InputScheme(value)) => return
            omoba_core::runtime::input_scheme_transport::validate_update(value).map(|_| ()),
        Some(renderer_input::Intent::Direction(value)) => return
            omoba_core::runtime::direction_transport::validate_renderer(value).map(|_|()),
        Some(renderer_input::Intent::Communication(_)) => return Err("COMMUNICATION_IS_NOT_GAMEPLAY"),
        Some(renderer_input::Intent::AbilityUpgrade(value)) => return
            if value.ability_index<4 {Ok(())} else {Err("INVALID_ABILITY_SLOT")},
        Some(renderer_input::Intent::Recall(_)) => return Ok(()),
        Some(renderer_input::Intent::SummonerCast(value)) => {
            return omoba_core::runtime::summoner_transport::admit_summoner_cast(
                value.slot, value.x_raw, value.y_raw).map(|_| ());
        }
        Some(renderer_input::Intent::WardPlacement(value)) => {
            return omoba_core::runtime::ward_transport::admit_ward_placement(
                value.slot, value.x_raw, value.y_raw).map(|_| ());
        }
        Some(renderer_input::Intent::HoldPosition(_)) | Some(renderer_input::Intent::Stop(_)) => return Ok(()),
        Some(renderer_input::Intent::ItemBuy(value)) => return
            omoba_core::runtime::shop_transport::item_id_for_catalog(value.catalog_id)
                .map(|_| ()).ok_or("INVALID_ITEM_CATALOG_ID"),
        Some(renderer_input::Intent::ItemSell(value)) => return
            if value.item_slot < 6 { Ok(()) } else { Err("INVALID_ITEM_SLOT") },
        Some(renderer_input::Intent::ShopUndo(value)) => return
            if value.sequence != 0 { Ok(()) } else { Err("INVALID_UNDO_SEQUENCE") },
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
            match (value.explicit_x_raw, value.explicit_y_raw) {
                (Some(x), Some(y)) => (x, y),
                (None, None) => (value.x_raw, value.y_raw),
                _ => return Err("INVALID_POSITION"),
            }
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
    fn shared_renderer_codec_still_requires_disclosed_target_and_owner() {
        use omoba_core::renderer_protocol::player_input_to_renderer_intent;
        use prost::Message;
        let view = view(1);
        let id = *view.runtime.world().entities.keys().next().unwrap();
        let mut bridge = InputBridge::default();
        bridge.set_commands_enabled(true);
        for action in [
            player_input::Action::MoveTo(omoba_core::game_proto::MoveTo {
                target: Some(Vec2I { x: -123, y: 456 }), queued: true }),
            player_input::Action::AttackMove(AttackMove {
                target: Some(Vec2I { x: -123, y: 456 }), queued: true }),
            player_input::Action::HoldPosition(omoba_core::game_proto::HoldPosition { queued: true }),
        ] {
            let formal = PlayerInput { action: Some(action) };
            let encoded = request(&view, player_input_to_renderer_intent(&formal).unwrap());
            let encoded = RendererInput::decode(encoded.encode_to_vec().as_slice()).unwrap();
            let before = bridge.next_input_id;
            assert_rejected(bridge.validate_view(encoded.clone(), 8, &view), "INVALID_OWNER");
            assert_eq!(bridge.next_input_id, before);
            let InputDecision::Accepted { input, .. } = bridge.validate_view(encoded, 7, &view)
                else { panic!("shared encoded input rejected"); };
            assert_eq!(input, formal);
        }
        let self_attack = PlayerInput { action: Some(player_input::Action::AttackTarget(
            AttackTarget { target_id: id as u32, queued: true })) };
        let encoded = request(&view, player_input_to_renderer_intent(&self_attack).unwrap());
        let before = bridge.next_input_id;
        assert_rejected(bridge.validate_view(encoded, 7, &view), "INVALID_TARGET");
        assert_eq!(bridge.next_input_id, before);
        let hidden = PlayerInput { action: Some(player_input::Action::AttackTarget(
            AttackTarget { target_id: u32::MAX, queued: true })) };
        let encoded = request(&view, player_input_to_renderer_intent(&hidden).unwrap());
        let before = bridge.next_input_id;
        assert_rejected(bridge.validate_view(encoded, 7, &view), "INVALID_TARGET");
        assert_eq!(bridge.next_input_id, before);
    }

    #[test]
    fn point_queue_roundtrips_without_bypassing_owner_epoch_or_position_validation() {
        use prost::Message;
        let view = view(1);
        let mut bridge = InputBridge::default();
        for queued in [false, true] {
            for intent in [
                renderer_input::Intent::MoveTo(MoveToIntent { x_raw: -123, y_raw: 456, queued }),
                renderer_input::Intent::AttackMove(AttackMoveIntent { x_raw: -123, y_raw: 456, queued }),
            ] {
                let is_move = matches!(intent, renderer_input::Intent::MoveTo(_));
                let original = request(&view, intent);
                let original = RendererInput::decode(original.encode_to_vec().as_slice()).unwrap();
                let before = bridge.next_input_id;
                let mut wrong = original.clone(); wrong.player_id = 8;
                assert_rejected(bridge.validate_view(wrong, 7, &view), "INVALID_OWNER");
                let mut stale = original.clone(); stale.disclosure_epoch += 1;
                assert_rejected(bridge.validate_view(stale, 7, &view), "STALE_DISCLOSURE_EPOCH");
                assert_eq!(bridge.next_input_id, before);
                let InputDecision::Accepted { input, input_id, secure_target, .. } = bridge.validate_view(original, 7, &view)
                    else { panic!("valid point command rejected"); };
                assert_eq!(input_id, before + 1);
                assert!(secure_target.is_none());
                let (point, actual_queue) = match input.action.unwrap() {
                    player_input::Action::MoveTo(value) if is_move => (value.target, value.queued),
                    player_input::Action::AttackMove(value) if !is_move => (value.target, value.queued),
                    _ => panic!("point action changed"),
                };
                assert_eq!(point, Some(Vec2I { x: -123, y: 456 }));
                assert_eq!(actual_queue, queued);
            }
        }
    }

    #[test]
    fn combat_wire_renderer_gate_requires_owner_projection_even_while_dead() {
        use omoba_core::game_proto::{CombatPreferencesUpdate, CombatPreferencesProjection, CombatPreferencesRow};
        use omoba_core::runtime::{TeamViewProjector, TeamProjectorConfig, SelectiveReplicaRuntime};
        let update = CombatPreferencesUpdate { schema_version: 1, acquisition: 1, idle_auto_attack: false };
        let mut bridge = InputBridge::default();
        let mut view = view(1);
        assert_rejected(bridge.validate_view(request(&view, renderer_input::Intent::CombatPreferences(update)), 7, &view), "COMBAT_PREFERENCES_UNAVAILABLE");
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        projector.prepare_combat_preferences(Some(CombatPreferencesProjection { schema_version: 1, view_epoch: 1,
            rows: vec![CombatPreferencesRow { player_id: 7, acquisition: 0, idle_auto_attack: true, input_scheme: None }] })).unwrap();
        let start = projector.build_team_game_start(1, 60, 0);
        view.runtime = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, Default::default(), Default::default()).unwrap();
        assert!(!view.owns_hero(7), "owner roster admission must work without a living actor");
        let valid = request(&view, renderer_input::Intent::CombatPreferences(update));
        let before = bridge.next_input_id;
        assert_rejected(bridge.validate_view(valid.clone(), 8, &view), "INVALID_OWNER");
        let mut stale = valid.clone(); stale.disclosure_epoch += 1;
        assert_rejected(bridge.validate_view(stale, 7, &view), "STALE_DISCLOSURE_EPOCH");
        let mut invalid = valid.clone(); invalid.intent = Some(renderer_input::Intent::CombatPreferences(CombatPreferencesUpdate { acquisition: 256, ..update }));
        assert_rejected(bridge.validate_view(invalid, 7, &view), "COMBAT_PREFERENCES_ACQUISITION");
        assert_eq!(bridge.next_input_id, before);
        let InputDecision::Accepted { input, secure_target, .. } = bridge.validate_view(valid, 7, &view) else { panic!("valid combat update rejected"); };
        assert_eq!(input.action, Some(player_input::Action::CombatPreferences(update)));
        assert!(secure_target.is_none());
    }

    #[test]
    fn input_scheme_wire_renderer_gate_is_roster_owned_and_uses_authority_lock_before_allocating_ids() {
        use omoba_core::game_proto::{InputSchemeUpdate,InputSchemeOwnerState,CombatPreferencesProjection,CombatPreferencesRow};
        use omoba_core::runtime::{TeamViewProjector,TeamProjectorConfig,SelectiveReplicaRuntime};
        let update=InputSchemeUpdate {schema_version:1,scheme:1};let mut bridge=InputBridge::default();let mut view=view(1);
        assert_rejected(bridge.validate_view(request(&view,renderer_input::Intent::InputScheme(update)),7,&view),"INPUT_SCHEME_UNAVAILABLE");
        let make_view=|elapsed| {
            let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());
            projector.prepare_combat_preferences(Some(CombatPreferencesProjection {schema_version:1,view_epoch:1,rows:vec![CombatPreferencesRow {
                player_id:7,acquisition:0,idle_auto_attack:true,input_scheme:Some(InputSchemeOwnerState {
                    schema_version:1,scheme:0,hero_content_id:"training_luminary".into(),playing_elapsed_raw:elapsed,
                    lock_after_raw:600*1024,direction_lease_raw:307,last_press_id:0,active_press_id:0,
                    direction_step:None,direction_protocol:1,
                }),
            }]})).unwrap();
            SelectiveReplicaRuntime::bootstrap_from_team_game_start(&projector.build_team_game_start(1,60,0),Default::default(),Default::default()).unwrap()
        };
        view.runtime=make_view(600*1024-1);assert!(!view.owns_hero(7));
        let valid=request(&view,renderer_input::Intent::InputScheme(update));let before=bridge.next_input_id;
        assert_rejected(bridge.validate_view(valid.clone(),8,&view),"INVALID_OWNER");
        let mut invalid=valid.clone();invalid.intent=Some(renderer_input::Intent::InputScheme(InputSchemeUpdate {scheme:256,..update}));
        assert_rejected(bridge.validate_view(invalid,7,&view),"INPUT_SCHEME_VALUE");
        let mut stale=valid.clone();stale.disclosure_epoch+=1;
        assert_rejected(bridge.validate_view(stale,7,&view),"STALE_DISCLOSURE_EPOCH");
        view.runtime=make_view(600*1024);
        assert_rejected(bridge.validate_view(valid.clone(),7,&view),"INPUT_SCHEME_LOCKED");assert_eq!(bridge.next_input_id,before);
        view.runtime=make_view(600*1024-1);
        let InputDecision::Accepted {input,secure_target,..}=bridge.validate_view(valid,7,&view)else{panic!("valid dead-owner mode update rejected")};
        assert_eq!(input.action,Some(player_input::Action::InputScheme(update)));assert!(secure_target.is_none());
    }

    #[test]
    fn hold_position_preserves_queue_flag_and_rejects_wrong_owner_or_epoch() {
        use prost::Message;
        let view=view(1);let mut bridge=InputBridge::default();
        bridge.set_commands_enabled(true);
        let input=request(&view,renderer_input::Intent::HoldPosition(omoba_core::game_proto::HoldPosition {queued:true}));
        let input=RendererInput::decode(input.encode_to_vec().as_slice()).unwrap();
        assert_rejected(bridge.validate_view(input.clone(),8,&view),"INVALID_OWNER");
        let mut stale=input.clone();stale.disclosure_epoch+=1;
        assert_rejected(bridge.validate_view(stale,7,&view),"STALE_DISCLOSURE_EPOCH");
        assert!(matches!(bridge.validate_view(input,7,&view),InputDecision::Accepted {
            input,secure_target:None,..} if matches!(input.action,Some(player_input::Action::HoldPosition(value)) if value.queued)));
    }
    #[test]
    fn command_capability_is_required_before_allocating_hold_input_id() {
        let view = view(1);
        let hold = request(&view, renderer_input::Intent::HoldPosition(
            omoba_core::game_proto::HoldPosition { queued: false }));
        let mut bridge = InputBridge::resume_after(42);
        assert_rejected(bridge.validate_view(hold.clone(), 7, &view), "COMMAND_PROTOCOL_UNAVAILABLE");
        assert_eq!(bridge.next_input_id, 42);
        bridge.set_commands_enabled(true);
        assert!(matches!(bridge.validate_view(hold.clone(), 7, &view), InputDecision::Accepted { input_id: 43, .. }));
        bridge.set_commands_enabled(false);
        assert_rejected(bridge.validate_view(hold, 7, &view), "COMMAND_PROTOCOL_UNAVAILABLE");
        let movement = request(&view, renderer_input::Intent::MoveTo(
            omoba_core::game_proto::MoveToIntent { x_raw: 123, y_raw: 456, queued: false }));
        assert!(matches!(bridge.validate_view(movement, 7, &view), InputDecision::Accepted { input_id: 44, .. }));
    }

    #[test]
    fn render_input_bridge_requires_negotiated_summoner_capability() {
        let view = view(1);
        let cast = request(&view, renderer_input::Intent::SummonerCast(
            omoba_core::game_proto::SummonerCastIntent { slot: 1, x_raw: None, y_raw: None }));
        let mut bridge = InputBridge::resume_after(42);
        assert_rejected(bridge.validate_view(cast.clone(), 7, &view), "SUMMONER_PROTOCOL_UNAVAILABLE");
        assert_eq!(bridge.next_input_id, 42);
        bridge.set_summoner_enabled(true);
        assert!(matches!(bridge.validate_view(cast.clone(), 7, &view), InputDecision::Accepted {
            input_id: 43,
            input,
            ..
        } if matches!(input.action, Some(player_input::Action::SummonerCast(value)) if value.slot == 1 && value.x_raw.is_none())));
        bridge.set_summoner_enabled(false);
        assert_rejected(bridge.validate_view(cast, 7, &view), "SUMMONER_PROTOCOL_UNAVAILABLE");
        bridge.set_summoner_enabled(true);
        let mut one_sided = request(&view, renderer_input::Intent::SummonerCast(
            omoba_core::game_proto::SummonerCastIntent { slot: 0, x_raw: Some(1), y_raw: None }));
        assert_rejected(bridge.validate_view(one_sided.clone(), 7, &view), "MALFORMED_SUMMONER_POINT");
        one_sided.intent = Some(renderer_input::Intent::SummonerCast(
            omoba_core::game_proto::SummonerCastIntent { slot: 4, x_raw: None, y_raw: None }));
        assert_rejected(bridge.validate_view(one_sided, 7, &view), "INVALID_SUMMONER_SLOT");
        let qwer = request(&view, renderer_input::Intent::AbilityCast(
            omoba_core::game_proto::AbilityCastIntent { ability_index: 5, ..Default::default() }));
        assert_rejected(bridge.validate_view(qwer, 7, &view), "INVALID_ABILITY_SLOT");
        assert_eq!(bridge.next_input_id, 43);
    }

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
    fn stop_command_rejects_unavailable_stale_and_unauthorized_before_input_id() {
        let view = view(1);
        let stop = request(&view, renderer_input::Intent::Stop(omoba_core::game_proto::Stop {}));
        let mut bridge = InputBridge::resume_after(42);
        assert_rejected(bridge.validate_view(stop.clone(), 8, &view), "INVALID_OWNER");
        let mut stale = stop.clone();
        stale.disclosure_epoch += 1;
        assert_rejected(bridge.validate_view(stale, 7, &view), "STALE_DISCLOSURE_EPOCH");
        let mut hidden = stop.clone();
        hidden.player_id = 9;
        assert_rejected(bridge.validate_view(hidden, 9, &view), "OWN_HERO_NOT_DISCLOSED");
        assert_rejected(bridge.validate_view(stop, 7, &view), "COMMAND_PROTOCOL_UNAVAILABLE");
        assert_eq!(bridge.next_input_id, 42);
    }

    #[test]
    fn stop_command_routes_coordinate_free_and_preserves_repeated_shop_ids() {
        use omoba_core::game_proto::ItemBuyIntent;
        let view = view(1);
        let stop = request(&view, renderer_input::Intent::Stop(omoba_core::game_proto::Stop {}));
        let mut bridge = InputBridge::resume_after(42);
        bridge.set_commands_enabled(true);
        match bridge.validate_view(stop.clone(), 7, &view) {
            InputDecision::Accepted { input_id: 43, secure_target: None, input } => {
                assert_eq!(input.action, Some(player_input::Action::Stop(omoba_core::game_proto::Stop {})));
            }
            other => panic!("coordinate-free stop rejected: {other:?}"),
        }
        match bridge.validate_view(stop.clone(), 7, &view) {
            InputDecision::Accepted { input_id: 44, secure_target: None, .. } => {}
            other => panic!("second stop reused an id: {other:?}"),
        }
        bridge.set_commands_enabled(false);
        assert_rejected(bridge.validate_view(stop, 7, &view), "COMMAND_PROTOCOL_UNAVAILABLE");
        assert_eq!(bridge.next_input_id, 44);
        bridge.set_shop_enabled(true);
        let buy = request(&view, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 }));
        match bridge.validate_view(buy.clone(), 7, &view) {
            InputDecision::Accepted { input_id: 45, secure_target: None, .. } => {}
            other => panic!("shop after stop rejected: {other:?}"),
        }
        assert!(matches!(bridge.validate_view(buy, 7, &view), InputDecision::RepeatedShop { input_id: 45, .. }));
        assert_eq!(bridge.next_input_id, 45);
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
        let before = bridge.next_input_id;
        assert_rejected(bridge.validate_view(input.clone(), 7, &view), "INVALID_TARGET");
        assert_eq!(bridge.next_input_id, before, "a disclosed self attack allocates no input id");
        for target_render_id in [0, 9999, u64::from(u32::MAX) + 1] {
            let mut invalid = input.clone();
            invalid.intent = Some(renderer_input::Intent::AttackTarget(AttackTargetIntent { target_render_id, queued: false }));
            assert!(matches!(bridge.validate_view(invalid, 7, &view), InputDecision::Rejected { .. }));
        }
        assert!(matches!(bridge.validate_view(input, 8, &view), InputDecision::Rejected { .. }));
        assert_eq!(bridge.next_input_id, before);
    }
    #[test]
    fn attack_target_admits_only_current_disclosed_hostile_relation() {
        use omoba_core::game_proto::AttackTargetIntent;
        use omoba_core::runtime::{decode_disclosed_combat_relation, encode_disclosed_combat_relation, DisclosedCombatRelation, DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID};
        let property = |hp: i64| {
            let mut bytes = vec![0u8; 40];
            bytes[..8].copy_from_slice(&hp.to_be_bytes());
            bytes[8..16].copy_from_slice(&(100i64 * 1024).to_be_bytes());
            bytes
        };
        let combat = |kind: u8, team: i32, attackable: bool, alive: bool| {
            encode_disclosed_combat_relation(DisclosedCombatRelation { faction_kind: kind, team_id: team, attackable, alive }).unwrap()
        };
        let render = |team: u32, kind: u8, owner: u32| encode_demo_render_state(DemoRenderState {
            x_raw: 0, y_raw: 0, team_id: team, kind, owner_player_id: owner,
        });
        let mut rows = Vec::new();
        let push = |rows: &mut Vec<(u64, Vec<(u32, Vec<u8>)>)>, id, team, kind, owner, hp, relation: Option<Vec<u8>>| {
            let mut components = vec![
                (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(team, kind, owner)),
                (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property(hp)),
            ];
            if let Some(bytes) = relation { components.push((DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, bytes)); }
            rows.push((id, components));
        };
        push(&mut rows, 10, 1, 1, 7, 100 * 1024, Some(combat(1, 1, true, true)));
        push(&mut rows, 30, 1, 1, 9, 100 * 1024, Some(combat(1, 1, true, true)));
        push(&mut rows, 20, 2, 1, 4, 100 * 1024, Some(combat(1, 2, true, true)));
        push(&mut rows, 40, 2, 2, 0, 100 * 1024, Some(combat(2, 2, true, true)));
        push(&mut rows, 41, 1, 2, 0, 100 * 1024, Some(combat(4, 1, true, true)));
        push(&mut rows, 42, 0, 2, 0, 100 * 1024, Some(combat(5, 0, true, true)));
        push(&mut rows, 43, 0, 2, 0, 100 * 1024, Some(combat(3, 0, true, true)));
        push(&mut rows, 44, 2, 2, 0, 100 * 1024, Some(combat(2, 2, false, true)));
        push(&mut rows, 50, 2, 2, 0, 100 * 1024, None);
        push(&mut rows, 55, 2, 2, 0, 100 * 1024, Some(vec![1, 1]));
        push(&mut rows, 60, 2, 1, 4, 0, Some(combat(1, 2, false, false)));
        let view = relation_view(1, &rows);
        // Team identity allocates presentation render ids from 1. The canonical
        // ids above are not replica render ids and must not be attacked directly.
        fn locate(view: &DisclosedView, pred: impl Fn(DemoRenderState, Option<DisclosedCombatRelation>) -> bool) -> u64 {
            *view.runtime.world().entities.iter().find(|(_, entity)| {
                let Some(render) = entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)
                    .and_then(|bytes| decode_demo_render_state(bytes)) else { return false };
                let combat = entity.components.get(&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)
                    .and_then(|bytes| decode_disclosed_combat_relation(bytes));
                pred(render, combat)
            }).expect("disclosed replica row").0
        }
        let source = locate(&view, |render, combat| render.owner_player_id == 7 && render.kind == 1 && combat.is_some_and(|row| row.alive));
        let ally_hero = locate(&view, |render, _| render.owner_player_id == 9 && render.kind == 1);
        let enemy_hero = locate(&view, |render, combat| render.owner_player_id == 4 && render.kind == 1 && combat.is_some_and(|row| row.alive));
        let enemy_creep = locate(&view, |render, combat| render.owner_player_id == 0 && combat.is_some_and(|row| row.faction_kind == 2 && row.attackable));
        let ally_creep = locate(&view, |_, combat| combat.is_some_and(|row| row.faction_kind == 4));
        let hostile_neutral = locate(&view, |_, combat| combat.is_some_and(|row| row.faction_kind == 5));
        let neutral = locate(&view, |_, combat| combat.is_some_and(|row| row.faction_kind == 3));
        let locked = locate(&view, |_, combat| combat.is_some_and(|row| row.faction_kind == 2 && !row.attackable && row.alive));
        let missing = *view.runtime.world().entities.iter().find(|(_, entity)| {
            let Some(render) = entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes| decode_demo_render_state(bytes)) else { return false };
            render.owner_player_id == 0 && render.team_id == 2 && render.kind == 2
                && !entity.components.contains_key(&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)
        }).expect("missing combat stays disclosed").0;
        let malformed = *view.runtime.world().entities.iter().find(|(_, entity)| {
            entity.components.get(&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)
                .is_some_and(|bytes| decode_disclosed_combat_relation(bytes).is_none())
        }).expect("malformed combat stays disclosed").0;
        let dead_body = locate(&view, |render, combat| render.owner_player_id == 4 && combat.is_some_and(|row| !row.alive));
        assert_ne!(enemy_hero, 20, "presentation render id is not the canonical id");
        let mut bridge = InputBridge::default();
        let attack = |target, queued| request(&view, renderer_input::Intent::AttackTarget(AttackTargetIntent { target_render_id: target, queued }));
        let before = bridge.next_input_id;
        match bridge.validate_view(request(&view, renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 12, y_raw: 34, queued: false })), 7, &view) {
            InputDecision::Accepted { input_id: 1, secure_target: None, .. } => {}
            other => panic!("move must remain legal: {other:?}"),
        }
        match bridge.validate_view(attack(enemy_hero, true), 7, &view) {
            InputDecision::Accepted { input_id: 2, input, secure_target: Some(reference) } => {
                assert!(matches!(input.action, Some(player_input::Action::AttackTarget(value)) if value.target_id == u32::try_from(enemy_hero).unwrap() && value.queued));
                assert_eq!(reference.replica_entity_id.unwrap().value, enemy_hero);
            }
            other => panic!("enemy player owner is not a team id: {other:?}"),
        }
        for target in [0u64, source, ally_hero, ally_creep, neutral, locked, missing, malformed, dead_body, 9999] {
            assert_rejected(bridge.validate_view(attack(target, false), 7, &view), "INVALID_TARGET");
        }
        match bridge.validate_view(attack(enemy_creep, false), 7, &view) {
            InputDecision::Accepted { input_id: 3, input, .. } => {
                assert!(matches!(input.action, Some(player_input::Action::AttackTarget(value)) if value.target_id == u32::try_from(enemy_creep).unwrap() && !value.queued));
            }
            other => panic!("ownerless enemy creep refused: {other:?}"),
        }
        match bridge.validate_view(attack(hostile_neutral, false), 7, &view) {
            InputDecision::Accepted { input_id: 4, .. } => {}
            other => panic!("hostile neutral refused: {other:?}"),
        }
        let mut as_nine = attack(enemy_hero, false);
        as_nine.player_id = 9;
        match bridge.validate_view(as_nine, 9, &view) {
            InputDecision::Accepted { input_id: 5, input, .. } => {
                assert!(matches!(input.action, Some(player_input::Action::AttackTarget(value)) if value.target_id == u32::try_from(enemy_hero).unwrap()));
            }
            other => panic!("same-team owner 9 is hostile to the enemy: {other:?}"),
        }
        let mut nine_on_ally = attack(source, false);
        nine_on_ally.player_id = 9;
        assert_rejected(bridge.validate_view(nine_on_ally, 9, &view), "INVALID_TARGET");
        assert_eq!(bridge.next_input_id, 5);
        let mut dead_rows = rows;
        dead_rows[0].1[2].1 = combat(1, 1, false, false);
        dead_rows[0].1[1].1 = property(0);
        let dead = relation_view(1, &dead_rows);
        let dead_enemy = locate(&dead, |render, combat| render.owner_player_id == 4 && render.kind == 1 && combat.is_some_and(|row| row.alive));
        let mut dead_bridge = InputBridge::default();
        assert_rejected(dead_bridge.validate_view(request(&dead, renderer_input::Intent::AttackTarget(AttackTargetIntent { target_render_id: dead_enemy, queued: false })), 7, &dead), "INVALID_TARGET");
        assert_eq!(dead_bridge.next_input_id, before);
        match dead_bridge.validate_view(request(&dead, renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 1, y_raw: 2, queued: false })), 7, &dead) {
            InputDecision::Accepted { input_id: 1, .. } => {}
            other => panic!("a dead source can still move: {other:?}"),
        }
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
    fn shop_undo_pins_sequence_and_a_retry_keeps_the_original_input_id() {
        use omoba_core::game_proto::ShopUndoIntent;
        use omoba_core::runtime::shop_undo_wire::SHOP_UNDO_MODE_SEQUENCE;
        let view = view(1);
        let mut bridge = InputBridge::default();
        bridge.set_shop_enabled(true);
        let undo = |sequence| request(&view, renderer_input::Intent::ShopUndo(ShopUndoIntent { sequence }));
        assert_rejected(bridge.validate_view(undo(9), 7, &view), "UNDO_PROTOCOL_UNAVAILABLE");
        bridge.set_shop_undo_enabled(true);
        assert_rejected(bridge.validate_view(undo(0), 7, &view), "INVALID_UNDO_SEQUENCE");
        match bridge.validate_view(undo(9), 7, &view) {
            InputDecision::Accepted { input, input_id: 1, secure_target: None } => match input.action {
                Some(player_input::Action::ShopUndo(value)) => {
                    assert_eq!((value.mode, value.sequence), (SHOP_UNDO_MODE_SEQUENCE, 9));
                }
                other => panic!("undo became {other:?}"),
            },
            other => panic!("unexpected {other:?}"),
        }
        assert!(matches!(bridge.validate_view(undo(9), 7, &view), InputDecision::RepeatedShop { input_id: 1, .. }));
        let mut later = undo(10);
        later.request_id = 92;
        match bridge.validate_view(later, 7, &view) {
            InputDecision::Accepted { input_id: 2, input, .. } => {
                assert!(matches!(input.action, Some(player_input::Action::ShopUndo(value)) if value.sequence == 10));
            }
            other => panic!("later purchase undo rejected: {other:?}"),
        }
        assert!(matches!(bridge.validate_view(undo(9), 7, &view), InputDecision::RepeatedShop { input_id: 1, .. }));
        let absent = view_with_heroes(1, &[(10, 8)]);
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ShopUndo(ShopUndoIntent { sequence: 9 })), 7, &absent), "OWN_HERO_NOT_DISCLOSED");
    }
    #[test]
    fn retained_dead_snapshot_admits_only_authority_shop_and_undo() {
        use omoba_core::game_proto::{ItemBuyIntent, ItemSellIntent, ItemUseIntent, RecallIntent, ShopUndoIntent, TeamPublicEvent};
        use omoba_core::runtime::native::economy_projection::{
            CommittedEconomyState, OwnerEconomyState, SHOP_REASON_AVAILABLE, SHOP_REASON_DEAD_UNCAPTURED,
            SHOP_REASON_FOREIGN_OWNER, SHOP_REASON_HERO_ABSENT, SHOP_REASON_RETIRED, SHOP_REASON_STALE_IDENTITY,
        };
        use omoba_core::runtime::shop_undo_wire::OwnerShopUndoProjection;
        use omoba_core::runtime::{FactKind, FilteredRenderSnapshot};
        let absent = view_with_heroes(1, &[(10, 8)]);
        let economy = OwnerEconomyState {
            player_id: 7,
            shop_available: true,
            shop_reason: SHOP_REASON_AVAILABLE,
            shield_remaining_raw: 0,
            economy: CommittedEconomyState {
                gold: 550, item_ids: [0; 6], cooldown_bits: [0; 6], effect_bits: [0; 10], dirty: false,
            },
        };
        let undo = OwnerShopUndoProjection {
            phase: 1, player_id: 7, can_undo: true, reason: 0, newest_sequence: 9, commitment: [7; 32],
        };
        let mut snapshot = FilteredRenderSnapshot {
            team_id: 1, replica_tick: 4, entities: vec![],
            public_events: vec![
                TeamPublicEvent { event_kind: FactKind::OwnerEconomy as u32, sanitized_payload: economy.encode(), ..Default::default() },
                TeamPublicEvent { event_kind: FactKind::OwnerShopUndo as u32, sanitized_payload: undo.encode(), ..Default::default() },
            ],
            external_effects: vec![], memory_directives: vec![], remembered_presentations: Default::default(),
            hero_attributes: Default::default(), disclosed_summoners: Default::default(), authoritative_mode: None, combat_preferences: None,
        };
        let decoded = crate::shop_presentation::owner_economy_state(&snapshot, 7).expect("dead owner economy");
        let decoded_undo = crate::shop_presentation::owner_undo_projection(&snapshot, 7).expect("dead owner undo");
        assert!(decoded.shop_available && decoded.shop_reason == SHOP_REASON_AVAILABLE);
        assert!(snapshot.entities.is_empty());
        let mut bridge = InputBridge::default();
        bridge.set_shop_enabled(true);
        bridge.set_shop_undo_enabled(true);
        bridge.set_owner_shop_disclosure(Some(super::OwnerShopDisclosure {
            epoch: absent.view_epoch(), economy: decoded, undo: Some(decoded_undo),
        }));
        match bridge.validate_view(request(&absent, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })), 7, &absent) {
            InputDecision::Accepted { input_id: 1, secure_target: None, .. } => {}
            other => panic!("retained dead buy rejected: {other:?}"),
        }
        let mut undo_input = request(&absent, renderer_input::Intent::ShopUndo(ShopUndoIntent { sequence: 9 }));
        undo_input.request_id = 92;
        match bridge.validate_view(undo_input, 7, &absent) {
            InputDecision::Accepted { input_id: 2, input, secure_target: None } => {
                assert!(matches!(input.action, Some(player_input::Action::ShopUndo(value)) if value.sequence == 9));
            }
            other => panic!("retained dead undo rejected: {other:?}"),
        }
        let sell = {
            let mut input = request(&absent, renderer_input::Intent::ItemSell(ItemSellIntent { item_slot: 0 }));
            input.request_id = 93;
            input
        };
        match bridge.validate_view(sell, 7, &absent) {
            InputDecision::Accepted { input_id: 3, .. } => {}
            other => panic!("retained dead sell rejected: {other:?}"),
        }
        let mut reject_other = |intent| {
            assert_rejected(bridge.validate_view(request(&absent, intent), 7, &absent), "OWN_HERO_NOT_DISCLOSED");
        };
        reject_other(renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 1, y_raw: 2, queued: false }));
        reject_other(renderer_input::Intent::Recall(RecallIntent {}));
        reject_other(renderer_input::Intent::ItemUse(ItemUseIntent { item_slot: 0, target_render_id: 0, x_raw: 0, y_raw: 0, explicit_x_raw: None, explicit_y_raw: None }));
        let mut closed = |reason| {
            let mut state = economy.clone();
            state.shop_available = false;
            state.shop_reason = reason;
            let mut closed_snapshot = snapshot.clone();
            closed_snapshot.public_events[0].sanitized_payload = state.encode();
            assert!(crate::shop_presentation::owner_economy_state(&closed_snapshot, 7).unwrap().shop_reason == reason);
            bridge.set_owner_shop_disclosure(Some(super::OwnerShopDisclosure {
                epoch: absent.view_epoch(),
                economy: crate::shop_presentation::owner_economy_state(&closed_snapshot, 7).unwrap(),
                undo: Some(undo.clone()),
            }));
            assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })), 7, &absent), "SHOP_AUTHORITY_CLOSED");
        };
        for reason in [SHOP_REASON_HERO_ABSENT, SHOP_REASON_DEAD_UNCAPTURED, SHOP_REASON_STALE_IDENTITY, SHOP_REASON_FOREIGN_OWNER, SHOP_REASON_RETIRED] {
            closed(reason);
        }
        bridge.set_owner_shop_disclosure(None);
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })), 7, &absent), "OWN_HERO_NOT_DISCLOSED");
        let mut foreign = economy.clone();
        foreign.player_id = 8;
        snapshot.public_events[0].sanitized_payload = foreign.encode();
        assert!(crate::shop_presentation::owner_economy_state(&snapshot, 7).is_none());
        bridge.set_owner_shop_disclosure(Some(super::OwnerShopDisclosure {
            epoch: absent.view_epoch(), economy: foreign, undo: Some(undo.clone()),
        }));
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })), 7, &absent), "INVALID_OWNER");
        bridge.set_owner_shop_disclosure(Some(super::OwnerShopDisclosure {
            epoch: absent.view_epoch().wrapping_add(1), economy: economy.clone(), undo: Some(undo.clone()),
        }));
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })), 7, &absent), "STALE_DISCLOSURE_EPOCH");
        let mut wrong_undo = undo.clone();
        wrong_undo.can_undo = false;
        wrong_undo.reason = 14;
        wrong_undo.newest_sequence = 0;
        bridge.set_owner_shop_disclosure(Some(super::OwnerShopDisclosure {
            epoch: absent.view_epoch(), economy: OwnerEconomyState::decode(&economy.encode()).unwrap(), undo: Some(wrong_undo),
        }));
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ShopUndo(ShopUndoIntent { sequence: 9 })), 7, &absent), "UNDO_AUTHORITY_CLOSED");
        bridge.set_owner_shop_disclosure(Some(super::OwnerShopDisclosure {
            epoch: absent.view_epoch(), economy: OwnerEconomyState::decode(&economy.encode()).unwrap(), undo: Some(undo.clone()),
        }));
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ShopUndo(ShopUndoIntent { sequence: 10 })), 7, &absent), "UNDO_AUTHORITY_CLOSED");
        snapshot.public_events[0].sanitized_payload[4..8].copy_from_slice(&(-1i32).to_le_bytes());
        assert!(crate::shop_presentation::owner_economy_state(&snapshot, 7).is_none());
        bridge.set_owner_shop_disclosure(None);
        assert_rejected(bridge.validate_view(request(&absent, renderer_input::Intent::ItemBuy(ItemBuyIntent { catalog_id: 1 })), 7, &absent), "OWN_HERO_NOT_DISCLOSED");
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
                    renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 1, y_raw: 2, queued: false }),
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
            decode_demo_render_state, decode_disclosed_combat_relation, encode_component_baseline,
            encode_demo_render_state, secure_replica_component_allowlist, secure_replica_resource_allowlist,
            DemoRenderState, DisclosedCombatRelation, ProjectionDependencyGraph, SelectiveReplicaRuntime,
            TeamProjectorConfig, TeamViewProjector, VisibilityTransition, DEMO_RENDER_COMPONENT_SCHEMA_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
        },
    };
    struct DisclosedView {
        runtime: SelectiveReplicaRuntime,
        team: u32,
    }

    impl InputView for DisclosedView {
        fn owned_hero_reference(&self,player:u32) -> Option<SecureReplicaTarget> {
            let mut matches=self.runtime.world().entities.iter().filter_map(|(id,entity)| {
                let render=decode_demo_render_state(entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)?)?;
                (render.team_id==self.team && render.kind==1 && render.owner_player_id==player).then_some(*id)
            });
            let id=matches.next()?;if matches.next().is_some() {return None;}
            self.secure_reference(id)
        }
        fn input_scheme_for_owner(&self, player_id: u32) -> Option<omoba_core::runtime::input_scheme_transport::AdmittedOwnerInputScheme> {
            self.runtime.input_scheme_for_owner(player_id)
        }
        fn combat_preferences_available(&self, player_id: u32) -> bool {
            self.runtime.combat_preferences_for_owner(player_id).is_some()
        }
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
        fn attack_relation_legal(&self, source_player_id: u32, render_id: u64) -> bool {
            let rows: Vec<_> = self.runtime.world().entities.iter()
                .map(|(id, entity)| (*id, &entity.components))
                .collect();
            omoba_core::runtime::disclosed_attack_target_admitted(&rows, source_player_id, render_id)
        }
    }

    fn view(team: u32) -> DisclosedView {
        view_with_heroes(team, &[(10, 7)])
    }

    fn relation_view(team: u32, rows: &[(u64, Vec<(u32, Vec<u8>)>)]) -> DisclosedView {
        let mut projector = TeamViewProjector::new(team, TeamProjectorConfig::default());
        let transitions = rows.iter().map(|(id, components)| {
            let refs: Vec<(u32, &[u8])> = components.iter().map(|(schema, bytes)| (*schema, bytes.as_slice())).collect();
            VisibilityTransition::Reveal { canonical_id: *id, effective_tick: 0, baseline: encode_component_baseline(&refs) }
        }).collect();
        projector.build_frame(0, 0, &rows.iter().map(|(id, _)| *id).collect(), transitions, &[], &ProjectionDependencyGraph::default()).unwrap();
        let start = projector.build_team_game_start(1, 120, 42);
        DisclosedView {
            runtime: SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, secure_replica_component_allowlist(), secure_replica_resource_allowlist()).unwrap(),
            team,
        }
    }

    fn view_with_heroes(team: u32, heroes: &[(u64, u32)]) -> DisclosedView {
        view_with_heroes_and_scheme(team,heroes,None)
    }

    fn view_with_heroes_and_scheme(team:u32, heroes:&[(u64,u32)],scheme:Option<omoba_core::game_proto::InputSchemeOwnerState>) -> DisclosedView {
        let mut projector = TeamViewProjector::new(team, TeamProjectorConfig::default());
        if let Some(scheme)=scheme {
            projector.prepare_combat_preferences(Some(omoba_core::game_proto::CombatPreferencesProjection {
                schema_version:1,view_epoch:1,rows:vec![omoba_core::game_proto::CombatPreferencesRow {
                    player_id:7,acquisition:0,idle_auto_attack:true,input_scheme:Some(scheme),
                }],
            })).unwrap();
        }
        let transitions=heroes.iter().map(|&(canonical_id,owner_player_id)| {
            let render = encode_demo_render_state(DemoRenderState {
            x_raw: 0,
            y_raw: 0,
            team_id: team,
            kind: 1,
            owner_player_id,
            });
            VisibilityTransition::Reveal { canonical_id, effective_tick:0,
                baseline:encode_component_baseline(&[(DEMO_RENDER_COMPONENT_SCHEMA_ID,&render)]) }
        }).collect();
        projector
            .build_frame(
                0,
                0,
                &heroes.iter().map(|&(id,_)| id).collect(),
                transitions,
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

    fn direction_scheme() -> omoba_core::game_proto::InputSchemeOwnerState {
        omoba_core::game_proto::InputSchemeOwnerState {schema_version:1,scheme:1,hero_content_id:"training_luminary".into(),
            playing_elapsed_raw:34,lock_after_raw:600*1024,direction_lease_raw:307,last_press_id:0,active_press_id:0,direction_protocol:1,
            direction_step:Some(omoba_core::game_proto::InputSchemeDirectionStep {tick:0,scheme:1,direction_x_raw:0,
                direction_y_raw:0,press_id:0,navigation_reset:false,attack_reset:false})}
    }

    #[test]
    fn direction_renderer_binds_unique_current_source_and_preserves_high_latency_release() {
        use omoba_core::game_proto::RendererDirectionIntent;
        let view=view_with_heroes_and_scheme(1,&[(10,7)],Some(direction_scheme()));
        let source=view.owned_hero_reference(7).unwrap();
        let id=source.replica_entity_id.as_ref().unwrap().value;
        let mut bridge=InputBridge::default();
        // Authority has not yet projected the Begin: Update and End must reach
        // the authoritative state machine rather than relying on active_press.
        for (phase,x,y) in [(1,1,0),(2,-32767,32767),(3,0,0)] {
            let intent=RendererDirectionIntent {schema_version:1,source_render_id:id,press_id:99,phase,x,y};
            let before=bridge.next_input_id;
            let InputDecision::Accepted {input,input_id,secure_target}=bridge.validate_view(
                request(&view,renderer_input::Intent::Direction(intent.clone())),7,&view) else {panic!("direction rejected phase {phase}");};
            assert_eq!(input_id,before+1);assert!(secure_target.is_none());
            let Some(player_input::Action::Direction(value))=input.action else {panic!("wrong direction action");};
            assert_eq!(value.source.as_ref(),Some(&source));assert_eq!(value.sealed_source,0);
            assert_eq!((value.press_id,value.phase,value.x,value.y),(99,phase,x,y));
        }
        let mut paused=direction_scheme();paused.direction_step=None;
        let paused=view_with_heroes_and_scheme(1,&[(10,7)],Some(paused));
        let end=RendererDirectionIntent {schema_version:1,source_render_id:id,press_id:99,phase:3,x:0,y:0};
        assert!(matches!(bridge.validate_view(request(&paused,renderer_input::Intent::Direction(end)),7,&paused),InputDecision::Accepted {..}));
    }

    #[test]
    fn direction_renderer_rejections_do_not_allocate_or_choose_duplicate_owner() {
        use omoba_core::game_proto::RendererDirectionIntent;
        let good=view_with_heroes_and_scheme(1,&[(10,7),(20,8)],Some(direction_scheme()));
        let id=good.owned_hero_reference(7).unwrap().replica_entity_id.unwrap().value;
        let intent=RendererDirectionIntent {schema_version:1,source_render_id:id,press_id:2,phase:1,x:1,y:0};
        let other=*good.runtime.world().entities.keys().find(|&&key|key!=id).unwrap();
        let mut bridge=InputBridge::default();
        for kind in ["cap","point","inactive","duplicate","missing"] {
            let mut state=direction_scheme();
            match kind {"cap"=>state.direction_protocol=0,"point"=>state.scheme=0,"inactive"=>state.direction_step=None,_=>{}}
            let heroes=match kind {"duplicate"=>vec![(10,7),(20,7)],"missing"=>vec![(20,8)],_=>vec![(10,7)]};
            let view=view_with_heroes_and_scheme(1,&heroes,Some(state));
            let before=bridge.next_input_id;
            assert!(matches!(bridge.validate_view(request(&view,renderer_input::Intent::Direction(intent.clone())),7,&view),InputDecision::Rejected {..}),"{kind}");
            assert_eq!(bridge.next_input_id,before,"{kind}");
        }
        for bad in [RendererDirectionIntent {source_render_id:0,..intent.clone()},
            RendererDirectionIntent {source_render_id:other,..intent.clone()},
            RendererDirectionIntent {source_render_id:u64::MAX,..intent.clone()},
            RendererDirectionIntent {phase:257,..intent.clone()},RendererDirectionIntent {press_id:0,..intent.clone()},
            RendererDirectionIntent {press_id:u64::MAX,..intent.clone()},RendererDirectionIntent {schema_version:256,..intent.clone()},
            RendererDirectionIntent {x:32768,..intent.clone()},RendererDirectionIntent {x:0,..intent.clone()},
            RendererDirectionIntent {phase:3,..intent.clone()}] {
            let before=bridge.next_input_id;
            assert!(matches!(bridge.validate_view(request(&good,renderer_input::Intent::Direction(bad)),7,&good),InputDecision::Rejected {..}));
            assert_eq!(bridge.next_input_id,before);
        }
        for kind in ["owner","epoch","zero-request"] {
            let mut input=request(&good,renderer_input::Intent::Direction(intent.clone()));
            match kind {"owner"=>input.player_id=8,"epoch"=>input.disclosure_epoch+=1,_=>input.request_id=0}
            let before=bridge.next_input_id;
            assert!(matches!(bridge.validate_view(input,7,&good),InputDecision::Rejected {..}));assert_eq!(bridge.next_input_id,before);
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
            renderer_input::Intent::MoveTo(MoveToIntent { x_raw: 1, y_raw: 2, queued: false }),
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
    fn ally_cast_keeps_teammate_disclosure_identity_not_caster_identity() {
        let view=view_with_heroes(1,&[(10,7),(20,8)]);
        let ally_id=*view.runtime.world().entities.iter().find(|(_,entity)| {
            entity.components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes|decode_demo_render_state(bytes))
                .is_some_and(|render|render.owner_player_id==8)
        }).unwrap().0;
        let mut bridge=InputBridge::default();
        let cast=|target|request(&view,renderer_input::Intent::AbilityCast(AbilityCastIntent {
            ability_index:3,target_render_id:target,x_raw:0,y_raw:0,
        }));
        match bridge.validate_view(cast(ally_id),7,&view) {
            InputDecision::Accepted {input_id:1,input,secure_target:Some(reference)}=>{
                assert_eq!(Some(reference),view.secure_reference(ally_id));
                assert!(matches!(input.action,Some(player_input::Action::CastAbility(value))
                    if value.ability_index==3 && value.target_entity==Some(ally_id as u32)));
            },
            other=>panic!("disclosed ally cast rejected: {other:?}"),
        }
        assert_rejected(bridge.validate_view(cast(9999),7,&view),"INVALID_TARGET");
        let mut stale=cast(ally_id);stale.disclosure_epoch+=1;
        assert_rejected(bridge.validate_view(stale,7,&view),"STALE_DISCLOSURE_EPOCH");
        assert_eq!(bridge.next_input_id,1,"rejected casts must not allocate input IDs");
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
                    queued: true,
                }),
                "INVALID_POSITION",
            ),
            (
                renderer_input::Intent::AttackMove(AttackMoveIntent {
                    x_raw: 0,
                    y_raw: i64::MIN,
                    queued: true,
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

    #[test]
    fn item_use_explicit_origin_is_distinct_from_legacy_zero_at_admission() {
        let view = view(1);
        let mut bridge = InputBridge::default();
        let explicit = request(&view, renderer_input::Intent::ItemUse(ItemUseIntent {
            item_slot: 0, x_raw: 0, y_raw: 0, explicit_x_raw: Some(0), explicit_y_raw: Some(0),
            ..Default::default()
        }));
        let legacy = request(&view, renderer_input::Intent::ItemUse(ItemUseIntent {
            item_slot: 0, x_raw: 0, y_raw: 0, ..Default::default()
        }));
        let accepted_explicit = bridge.validate_view(explicit, 7, &view);
        let accepted_legacy = bridge.validate_view(legacy, 7, &view);
        match (accepted_explicit, accepted_legacy) {
            (InputDecision::Accepted { input: explicit_input, .. }, InputDecision::Accepted { input: legacy_input, .. }) => {
                match (explicit_input.action, legacy_input.action) {
                    (Some(player_input::Action::ItemUse(explicit_use)), Some(player_input::Action::ItemUse(legacy_use))) => {
                        assert_eq!(explicit_use.point_explicit, Some(true));
                        assert_eq!(explicit_use.target_pos, Some(Vec2I { x: 0, y: 0 }));
                        assert_eq!(explicit_use.target_generation, None);
                        assert_eq!(explicit_use.owner_entity, None);
                        assert_eq!(explicit_use.owner_generation, None);
                        assert_eq!(legacy_use.point_explicit, None);
                        assert_eq!(legacy_use.target_pos, Some(Vec2I { x: 0, y: 0 }));
                        assert_eq!(legacy_use.target_generation, None);
                        assert_eq!(legacy_use.owner_entity, None);
                        assert_eq!(legacy_use.owner_generation, None);
                        assert_ne!(explicit_use, legacy_use);
                    }
                    other => panic!("expected item uses, got {other:?}"),
                }
            }
            other => panic!("explicit origin admission failed: {other:?}"),
        }
        let half = request(&view, renderer_input::Intent::ItemUse(ItemUseIntent {
            item_slot: 0, explicit_x_raw: Some(0), ..Default::default()
        }));
        assert_rejected(bridge.validate_view(half, 7, &view), "INVALID_POSITION");
        assert_eq!(bridge.next_input_id, 2);
    }

    fn ward_input(view: &DisclosedView, request_id: u64, x_raw: i64) -> RendererInput {
        RendererInput {
            request_id,
            player_id: 7,
            disclosure_epoch: view.view_epoch(),
            intent: Some(renderer_input::Intent::WardPlacement(
                omoba_core::game_proto::WardPlacementIntent { slot: 0, x_raw: Some(x_raw), y_raw: Some(4) },
            )),
        }
    }

    #[test]
    fn renderer_request_1_maps_to_authority_901_and_evicted_requests_do_not_allocate() {
        let view = view(1);
        let mut bridge = InputBridge::resume_after(900);
        bridge.set_ward_enabled(true);
        let first = bridge.validate_view(ward_input(&view, 1, 3), 7, &view);
        match first {
            InputDecision::Accepted { input_id, .. } => assert_eq!(input_id, 901),
            other => panic!("expected authority 901, got {other:?}"),
        }
        assert_eq!(bridge.ward_authority_for(1), Some(901));
        assert_eq!(bridge.reserved_input_id(), 901);
        match bridge.validate_view(ward_input(&view, 1, 3), 7, &view) {
            InputDecision::RepeatedWard { request_id, input_id } => {
                assert_eq!(request_id, 1);
                assert_eq!(input_id, 901);
            }
            other => panic!("duplicate renderer request allocated again: {other:?}"),
        }
        assert_eq!(bridge.reserved_input_id(), 901, "a repeated ward request does not consume an authority id");
        match bridge.validate_view(ward_input(&view, 1, 9), 7, &view) {
            InputDecision::Rejected { code, .. } => assert_eq!(code, "CONFLICTING_WARD_REQUEST_ID"),
            other => panic!("conflicting payload was accepted: {other:?}"),
        }
        for request_id in 2..=8 {
            match bridge.validate_view(ward_input(&view, request_id, 3), 7, &view) {
                InputDecision::Accepted { input_id, .. } => assert_eq!(input_id, 900 + request_id as u32),
                other => panic!("request {request_id} was not accepted: {other:?}"),
            }
        }
        match bridge.validate_view(ward_input(&view, 9, 3), 7, &view) {
            InputDecision::Accepted { input_id, .. } => assert_eq!(input_id, 909),
            other => panic!("ninth request did not allocate: {other:?}"),
        }
        assert!(bridge.ward_authority_for(1).is_none());
        let floor = bridge.reserved_input_id();
        match bridge.validate_view(ward_input(&view, 1, 3), 7, &view) {
            InputDecision::Rejected { code, .. } => assert_eq!(code, "WARD_REQUEST_EVICTED"),
            other => panic!("evicted request was retried as a new placement: {other:?}"),
        }
        assert_eq!(bridge.reserved_input_id(), floor);
        let order: Vec<_> = bridge.ward_requests.iter().map(|(input, id)| (input.request_id, *id)).collect();
        assert_eq!(order, vec![(2, 902), (3, 903), (4, 904), (5, 905), (6, 906), (7, 907), (8, 908), (9, 909)]);
    }
}
