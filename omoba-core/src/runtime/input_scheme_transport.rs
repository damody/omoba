//! Validated authority roster identity for mode selection and hero overrides.
//! Owner rows share the existing complete, epoch-bound preferences projection.
//! This capability does not assert that a directional input was acknowledged.

use omoba_sim::Fixed64;
use crate::game_proto::InputSchemeOwnerState;
use super::input_scheme::{InputScheme, InputSchemeRules};

pub const SCHEMA_VERSION: u32 = 1;
pub const ACTION_KIND: u32 = 27;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmittedDirectionStep {
    pub tick: u64,
    pub scheme: InputScheme,
    pub direction: omoba_sim::Vec2,
    pub press_id: u64,
    pub navigation_reset: bool,
    pub attack_reset: bool,
}
pub fn decode_direction_step(value: &crate::game_proto::InputSchemeDirectionStep)
    -> Result<AdmittedDirectionStep, &'static str> {
    let scheme = InputScheme::from_raw(value.scheme).map_err(|_| "INPUT_SCHEME_STEP")?;
    let (x, y) = (value.direction_x_raw, value.direction_y_raw);
    if !(-1024..=1024).contains(&x) || !(-1024..=1024).contains(&y) || value.press_id == u64::MAX {
        return Err("INPUT_SCHEME_STEP");
    }
    let squared = x*x + y*y;
    if squared != 0 && (scheme != InputScheme::Keyboard || value.press_id == 0
        || !(1022*1022..=1024*1024).contains(&squared)) {
        return Err("INPUT_SCHEME_STEP");
    }
    if (scheme == InputScheme::PointAndClick && value.press_id != 0)
        || (value.attack_reset && !value.navigation_reset) { return Err("INPUT_SCHEME_STEP"); }
    Ok(AdmittedDirectionStep {tick: value.tick, scheme,
        direction: omoba_sim::Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y)),
        press_id: value.press_id, navigation_reset: value.navigation_reset, attack_reset:value.attack_reset})
}
pub fn decode_direction_steps(projection: Option<&crate::game_proto::CombatPreferencesProjection>, epoch: u64, tick: u64)
    -> Result<std::collections::BTreeMap<u32, AdmittedDirectionStep>, &'static str> {
    super::combat_transport::decode_projection(projection, epoch)?;
    let mut rows = std::collections::BTreeMap::new();
    if let Some(projection) = projection {
        for row in &projection.rows {
            if let Some(step) = row.input_scheme.as_ref().and_then(|state| state.direction_step.as_ref()) {
                let step = decode_direction_step(step)?;
                if step.tick != tick { return Err("INPUT_SCHEME_STEP_TICK"); }
                rows.insert(row.player_id, step);
            }
        }
    }
    Ok(rows)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedOwnerInputScheme {
    pub scheme: InputScheme,
    pub hero_content_id: String,
    pub playing_elapsed: Fixed64,
    pub rules: InputSchemeRules,
    pub last_press_id: u64,
    pub active_press_id: u64,
    pub direction_step: Option<AdmittedDirectionStep>,
    pub direction_protocol: u32,
}
impl AdmittedOwnerInputScheme {
    pub fn locked(&self) -> bool { self.playing_elapsed >= self.rules.lock_after() }
}

pub fn valid_content_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b,b'_'|b'-'|b'.'|b'/'))
        && !id.starts_with('/') && !id.ends_with('/') && !id.contains("..") && !id.contains("//")
}

pub fn decode(value: &InputSchemeOwnerState) -> Result<AdmittedOwnerInputScheme, &'static str> {
    if value.schema_version!=SCHEMA_VERSION || value.direction_protocol>1 || !valid_content_id(&value.hero_content_id)
        || value.playing_elapsed_raw<0 || value.last_press_id==u64::MAX
        || (value.active_press_id!=0 && value.active_press_id!=value.last_press_id)
        || (value.scheme!=1 && value.active_press_id!=0) {
        return Err("INPUT_SCHEME_OWNER_STATE");
    }
    let scheme=InputScheme::from_raw(value.scheme).map_err(|_| "INPUT_SCHEME_OWNER_STATE")?;
    let rules=InputSchemeRules::new(Fixed64::from_raw(value.lock_after_raw),Fixed64::from_raw(value.direction_lease_raw))
        .map_err(|_| "INPUT_SCHEME_OWNER_STATE")?;
    Ok(AdmittedOwnerInputScheme {scheme,hero_content_id:value.hero_content_id.clone(),
        playing_elapsed:Fixed64::from_raw(value.playing_elapsed_raw),rules,last_press_id:value.last_press_id,
        active_press_id:value.active_press_id,direction_protocol:value.direction_protocol,
        direction_step:value.direction_step.as_ref().map(decode_direction_step).transpose()?})
}

pub fn project(state: &super::MobaMatch, slot: &super::native::moba_match::MobaHeroSlot) -> Option<InputSchemeOwnerState> {
    let last_press_id=slot.entity.map_or(0,|entity| slot.input_scheme().last_press(super::selective::CanonicalEntityKey {
        id:entity.id(),generation:entity.gen().id() as u32,
    }));
    let value=InputSchemeOwnerState {schema_version:SCHEMA_VERSION,scheme:slot.input_scheme().scheme() as u32,
        hero_content_id:slot.hero_content_id().to_string(),playing_elapsed_raw:state.input_scheme_elapsed().raw(),
        lock_after_raw:state.config.input_scheme_rules.lock_after().raw(),
        direction_lease_raw:state.config.input_scheme_rules.direction_lease().raw(),last_press_id,direction_protocol:1,
        active_press_id:slot.entity.map_or(0,|entity|slot.input_scheme().active_press(super::selective::CanonicalEntityKey {
            id:entity.id(),generation:entity.gen().id() as u32,
        })),
        direction_step:state.input_step.as_ref().and_then(|snapshot| {
            let row=snapshot.rows.get(&slot.player_id)?;
            if slot.entity!=Some(row.source) {return None;}
            Some(crate::game_proto::InputSchemeDirectionStep {tick:snapshot.tick,scheme:row.scheme as u32,
                direction_x_raw:row.direction.x.raw(),direction_y_raw:row.direction.y.raw(),
                press_id:row.press_id,navigation_reset:row.navigation_reset,attack_reset:row.attack_reset})
        })};
    decode(&value).ok()?;Some(value)
}

/// Reject a malformed complete owner vector before exposing any nested mode.
pub fn for_owner(projection: Option<&crate::game_proto::CombatPreferencesProjection>, epoch: u64, owner: u32)
    -> Option<AdmittedOwnerInputScheme> {
    super::combat_transport::decode_projection(projection,epoch).ok()?;
    let value=projection?.rows.iter().find(|row| row.player_id==owner)?.input_scheme.as_ref()?;
    decode(value).ok()
}

pub fn validate_update(value: &crate::game_proto::InputSchemeUpdate) -> Result<InputScheme,&'static str> {
    if value.schema_version!=SCHEMA_VERSION {return Err("INPUT_SCHEME_SCHEMA");}
    InputScheme::from_raw(value.scheme).map_err(|_|"INPUT_SCHEME_VALUE")
}
pub fn is_owner_setting_action(kind: u32) -> bool {
    kind==ACTION_KIND || kind==super::combat_transport::ACTION_KIND
}
pub fn validate_owner_setting_ack(value: &crate::game_proto::TeamAcceptedInput) -> Result<(),&'static str> {
    use prost::Message;
    if value.action_kind==super::combat_transport::ACTION_KIND {return super::combat_transport::validate_owner_ack(value);}
    if value.action_kind!=ACTION_KIND || value.player_id==0 || value.input_id==0
        || value.actor.is_some() || value.target.is_some() {return Err("INPUT_SCHEME_ACK");}
    let payload=crate::game_proto::PlayerInput::decode(value.sanitized_payload.as_slice()).map_err(|_|"INPUT_SCHEME_ACK")?;
    match payload.action {
        Some(crate::game_proto::player_input::Action::InputScheme(value))=>validate_update(&value).map(|_|()),
        _=>Err("INPUT_SCHEME_ACK"),
    }
}
pub fn owner_setting_available(projection: Option<&crate::game_proto::CombatPreferencesProjection>,epoch:u64,kind:u32,owner:u32) -> bool {
    if kind==ACTION_KIND {return for_owner(projection,epoch,owner).is_some();}
    kind==super::combat_transport::ACTION_KIND && super::combat_transport::decode_projection(projection,epoch)
        .is_ok_and(|rows|rows.contains_key(&owner))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_proto::{CombatPreferencesProjection, CombatPreferencesRow};
    use crate::runtime::{TeamProjectorConfig,TeamViewProjector,SelectiveReplicaRuntime,NoopDisclosedWorldStepper,ProjectionDependencyGraph};
    use std::collections::BTreeSet;
    fn owner_state() -> InputSchemeOwnerState {
        InputSchemeOwnerState {schema_version:1,scheme:1,hero_content_id:"training_luminary".into(),
            playing_elapsed_raw:600*1024,lock_after_raw:600*1024,direction_lease_raw:307,last_press_id:9,active_press_id:0,direction_step:None,direction_protocol:1}
    }
    fn projection(value: InputSchemeOwnerState) -> CombatPreferencesProjection {
        CombatPreferencesProjection {schema_version:1,view_epoch:1,rows:vec![CombatPreferencesRow {
            player_id:7,acquisition:0,idle_auto_attack:true,input_scheme:Some(value),
        }]}
    }

    #[test]
    fn input_scheme_direction_step_has_independent_tick_and_post_lease_validation() {
        let step = crate::game_proto::InputSchemeDirectionStep {tick:1, scheme:1,
            direction_x_raw:1024, direction_y_raw:0, press_id:9, navigation_reset:true,attack_reset:false};
        let mut state = owner_state();
        state.direction_step = Some(step);
        // A Stop drained after movement retires the post-step lease, but not
        // the already consumed pre-dispatch controls.
        assert_eq!(state.active_press_id, 0);
        let valid = projection(state.clone());
        assert_eq!(decode_direction_steps(Some(&valid),1,1).unwrap()[&7].direction.x.raw(),1024);
        assert!(decode_direction_steps(Some(&valid),1,2).is_err());
        let invalids = [
            crate::game_proto::InputSchemeDirectionStep {scheme:256,..step},
            crate::game_proto::InputSchemeDirectionStep {direction_x_raw:i64::MAX,..step},
            crate::game_proto::InputSchemeDirectionStep {direction_y_raw:1024,..step},
            crate::game_proto::InputSchemeDirectionStep {direction_x_raw:1,..step},
            crate::game_proto::InputSchemeDirectionStep {press_id:0,..step},
            crate::game_proto::InputSchemeDirectionStep {press_id:u64::MAX,..step},
            crate::game_proto::InputSchemeDirectionStep {scheme:0,direction_x_raw:0,..step},
            crate::game_proto::InputSchemeDirectionStep {navigation_reset:false,attack_reset:true,..step},
        ];
        for invalid in invalids {
            let mut bad = state.clone(); bad.direction_step=Some(invalid);
            assert!(super::super::combat_transport::decode_projection(Some(&projection(bad)),1).is_err());
        }
        let mut projector = TeamViewProjector::new(1,TeamProjectorConfig {hash_checkpoint_interval_ticks:1,..Default::default()});
        projector.prepare_combat_preferences(Some(valid)).unwrap();
        let start = projector.build_team_game_start(1,60,0);
        let mut replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,BTreeSet::new(),BTreeSet::new()).unwrap();
        let frame = projector.build_frame_with_inputs(1,1,&BTreeSet::new(),vec![],&[],&ProjectionDependencyGraph::default(),vec![]).unwrap().frame;
        let mut invalid = frame.clone();
        invalid.combat_preferences.as_mut().unwrap().rows[0].input_scheme.as_mut().unwrap().direction_step.as_mut().unwrap().tick=2;
        let before = replica.canonical_team_hash();
        assert!(replica.apply_frame(invalid,&mut NoopDisclosedWorldStepper).is_err());
        assert_eq!(replica.canonical_team_hash(), before);
        replica.apply_frame(frame.clone(),&mut NoopDisclosedWorldStepper).unwrap();
        assert_eq!(replica.canonical_team_hash().as_slice(),frame.post_step.unwrap().hash_checkpoint.unwrap().canonical_team_hash.as_slice());
    }
    #[test]
    fn input_scheme_owner_wire_is_atomic_strict_and_independently_capability_gated() {
        let value=owner_state();let accepted=decode(&value).unwrap();
        assert!(accepted.locked());assert_eq!(accepted.scheme,InputScheme::Keyboard);
        assert_eq!(accepted.last_press_id,9);
        let mut before=value.clone();before.playing_elapsed_raw-=1;assert!(!decode(&before).unwrap().locked());
        let mut invalids=Vec::new();
        for scheme in [2,256,u32::MAX] { invalids.push(InputSchemeOwnerState {scheme,..value.clone()}); }
        for schema_version in [0,2,u32::MAX] { invalids.push(InputSchemeOwnerState {schema_version,..value.clone()}); }
        for direction_protocol in [2,256,u32::MAX] {invalids.push(InputSchemeOwnerState {direction_protocol,..value.clone()});}
        for hero_content_id in ["","../hero","/hero","hero/","hero//a","hero\\a","a\nhero","角色"] {
            invalids.push(InputSchemeOwnerState {hero_content_id:hero_content_id.into(),..value.clone()});
        }
        invalids.push(InputSchemeOwnerState {hero_content_id:"a".repeat(129),..value.clone()});
        invalids.push(InputSchemeOwnerState {playing_elapsed_raw:-1,..value.clone()});
        invalids.push(InputSchemeOwnerState {lock_after_raw:-1,..value.clone()});
        invalids.push(InputSchemeOwnerState {direction_lease_raw:101,..value.clone()});
        invalids.push(InputSchemeOwnerState {last_press_id:u64::MAX,..value.clone()});
        invalids.push(InputSchemeOwnerState {active_press_id:10,..value.clone()});
        invalids.push(InputSchemeOwnerState {active_press_id:8,..value.clone()});
        invalids.push(InputSchemeOwnerState {scheme:0,active_press_id:9,..value.clone()});
        for invalid in invalids {
            assert!(decode(&invalid).is_err());
            let poisoned=projection(invalid);
            assert!(crate::runtime::combat_transport::decode_projection(Some(&poisoned),1).is_err());
            assert!(for_owner(Some(&poisoned),1,7).is_none());
        }
        let mut legacy=projection(value);legacy.rows[0].input_scheme=None;
        assert!(crate::runtime::combat_transport::decode_projection(Some(&legacy),1).is_ok());
        assert!(for_owner(Some(&legacy),1,7).is_none());
    }
    #[test]
    fn input_scheme_owner_wire_survives_bootstrap_frames_rebase_and_binds_hash() {
        let controls=projection(owner_state());
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig {hash_checkpoint_interval_ticks:1,..Default::default()});
        projector.prepare_combat_preferences(Some(controls.clone())).unwrap();
        let start=projector.build_team_game_start(1,60,0);
        let mut runtime=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,BTreeSet::new(),BTreeSet::new()).unwrap();
        assert_eq!(runtime.input_scheme_for_owner(7).unwrap().hero_content_id,"training_luminary");
        assert!(runtime.input_scheme_for_owner(8).is_none());
        let graph=ProjectionDependencyGraph::default();
        let payload=crate::game_proto::PlayerInput {action:Some(crate::game_proto::player_input::Action::InputScheme(
            crate::game_proto::InputSchemeUpdate {schema_version:1,scheme:1}))};
        let accepted=crate::runtime::CanonicalAcceptedInput::from_authoritative_acceptance(1,7,91,ACTION_KIND,0,None,prost::Message::encode_to_vec(&payload));
        let mut unknown=accepted.clone();unknown.player_id=8;
        let mut malformed=accepted.clone();malformed.sanitized_payload.clear();
        let frame=projector.build_frame_with_inputs(1,1,&BTreeSet::new(),vec![],&[],&graph,vec![accepted,unknown,malformed]).unwrap().frame;
        let acks=&frame.step.as_ref().unwrap().accepted_inputs;
        assert_eq!(acks.len(),1);validate_owner_setting_ack(&acks[0]).unwrap();assert!(acks[0].actor.is_none());
        runtime.apply_frame(frame.clone(),&mut NoopDisclosedWorldStepper).unwrap();
        assert_eq!(runtime.canonical_team_hash().as_slice(),frame.post_step.unwrap().hash_checkpoint.unwrap().canonical_team_hash.as_slice());
        let continued=projector.build_team_game_start(2,60,0);
        let mut changed=continued.clone();changed.combat_preferences.as_mut().unwrap().rows[0].input_scheme.as_mut().unwrap().hero_content_id="another_hero".into();
        let different=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&changed,BTreeSet::new(),BTreeSet::new()).unwrap();
        assert_ne!(runtime.canonical_team_hash(),different.canonical_team_hash());
        let bundle=projector.build_filtered_rebase(2,2,2).unwrap();
        let manifest=bundle.manifest;
        let snapshot=crate::game_proto::FilteredTeamSnapshot {
            snapshot_schema_version:1,snapshot_id:manifest.snapshot_id.clone(),team_id:1,
            view_epoch:manifest.view_epoch.clone(),authoritative_tick:2,disclosed_world:vec![],
            filtered_snapshot_hash:manifest.filtered_snapshot_hash.clone(),..Default::default()
        };
        runtime.apply_verified_rebase(&snapshot,&manifest,&[]).unwrap();
        assert_eq!(runtime.input_scheme_for_owner(7).unwrap().hero_content_id,"training_luminary");
        let mut invalid=manifest.clone();invalid.combat_preferences.as_mut().unwrap().rows[0].input_scheme.as_mut().unwrap().scheme=256;
        let before=runtime.canonical_team_hash();assert!(runtime.apply_verified_rebase(&snapshot,&invalid,&[]).is_err());
        assert_eq!(runtime.canonical_team_hash(),before);
    }
    #[test]
    fn input_scheme_mode_wire_ack_is_typed_actorless_and_cannot_alias_other_actions() {
        use crate::game_proto::{InputSchemeUpdate,PlayerInput,player_input::Action,TeamAcceptedInput,ReplicaEntityId};
        for scheme in [0,1] {
            let update=InputSchemeUpdate {schema_version:1,scheme};assert!(validate_update(&update).is_ok());
            let ack=TeamAcceptedInput {player_id:7,input_id:91,action_kind:ACTION_KIND,
                sanitized_payload:prost::Message::encode_to_vec(&PlayerInput {action:Some(Action::InputScheme(update))}),..Default::default()};
            validate_owner_setting_ack(&ack).unwrap();
            for invalid in [TeamAcceptedInput {player_id:0,..ack.clone()},TeamAcceptedInput {input_id:0,..ack.clone()},
                TeamAcceptedInput {actor:Some(ReplicaEntityId{value:1}),..ack.clone()},
                TeamAcceptedInput {target:Some(ReplicaEntityId{value:1}),..ack.clone()},
                TeamAcceptedInput {action_kind:super::super::combat_transport::ACTION_KIND,..ack.clone()},
                TeamAcceptedInput {sanitized_payload:vec![],..ack.clone()}] {
                assert!(validate_owner_setting_ack(&invalid).is_err());
            }
        }
        for (schema_version,scheme) in [(0,0),(2,1),(1,2),(1,256),(1,u32::MAX)] {
            assert!(validate_update(&InputSchemeUpdate {schema_version,scheme}).is_err());
        }
    }
}
