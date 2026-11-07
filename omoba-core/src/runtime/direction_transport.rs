//! Distinct trust boundaries for source-only directional commands.
//! Client references, authority seals and redacted acknowledgements cannot be
//! substituted for each other, even when their gesture payload is identical.
use crate::game_proto::HeroDirectionInput;
use super::input_scheme::{normalized_axes, DirectionGesture};

pub const SCHEMA_VERSION: u32 = 1;
pub const ACTION_KIND: u32 = 28;

pub fn gesture(value: &HeroDirectionInput) -> Result<DirectionGesture, &'static str> {
    if value.schema_version != SCHEMA_VERSION || value.press_id == 0 || value.press_id == u64::MAX {
        return Err("DIRECTION_SHAPE");
    }
    let direction = normalized_axes(value.x, value.y).map_err(|_| "DIRECTION_SHAPE")?;
    match value.phase {
        1 if direction != omoba_sim::Vec2::ZERO => Ok(DirectionGesture::Begin(value.press_id)),
        2 if direction != omoba_sim::Vec2::ZERO => Ok(DirectionGesture::Update(value.press_id)),
        3 if direction == omoba_sim::Vec2::ZERO => Ok(DirectionGesture::End(value.press_id)),
        _ => Err("DIRECTION_SHAPE"),
    }
}

pub fn validate_client(value: &HeroDirectionInput) -> Result<DirectionGesture, &'static str> {
    let result = gesture(value)?;
    let source = value.source.as_ref().ok_or("DIRECTION_SOURCE")?;
    if value.sealed_source != 0 || !source.replica_entity_id.as_ref().is_some_and(|v|v.value!=0)
        || !source.view_epoch.as_ref().is_some_and(|v|v.value!=0)
        || !source.disclosure_epoch.as_ref().is_some_and(|v|v.value!=0) {
        return Err("DIRECTION_SOURCE");
    }
    Ok(result)
}

pub fn validate_sealed(value: &HeroDirectionInput) -> Result<DirectionGesture, &'static str> {
    let result = gesture(value)?;
    let generation = value.sealed_source >> 32;
    if value.source.is_some() || generation == 0 || generation > i32::MAX as u64 {
        return Err("DIRECTION_SOURCE");
    }
    Ok(result)
}

/// Mutate only after all session/source checks succeed. The caller supplies
/// authenticated bindings, never fields copied from the client's payload.
pub fn seal_client(value:&mut HeroDirectionInput, snapshot:&super::SecureInputValidationSnapshot,
    team:u32, view_epoch:u64, tick:u64) -> Result<(), &'static str> {
    validate_client(value)?;
    let source=value.source.as_ref().ok_or("DIRECTION_SOURCE")?;
    let actor=super::SecureTargetReference {
        replica_id:source.replica_entity_id.as_ref().ok_or("DIRECTION_SOURCE")?.value,
        view_epoch:source.view_epoch.as_ref().ok_or("DIRECTION_SOURCE")?.value,
        disclosure_epoch:source.disclosure_epoch.as_ref().ok_or("DIRECTION_SOURCE")?.value,
    };
    if actor.view_epoch!=view_epoch {return Err("DIRECTION_SOURCE");}
    let canonical=snapshot.validate_actor_and_resolve(team,tick,actor).map_err(|_|"DIRECTION_SOURCE")?;
    value.source=None;value.sealed_source=canonical;Ok(())
}

pub fn validate_ack(value: &HeroDirectionInput) -> Result<DirectionGesture, &'static str> {
    let result = gesture(value)?;
    if value.source.is_some() || value.sealed_source != 0 { return Err("DIRECTION_SOURCE"); }
    Ok(result)
}

pub fn available(projection: Option<&crate::game_proto::CombatPreferencesProjection>, epoch:u64, owner:u32) -> bool {
    super::input_scheme_transport::for_owner(projection,epoch,owner)
        .is_some_and(|row|row.direction_protocol==1)
}

pub fn validate_renderer(value:&crate::game_proto::RendererDirectionIntent) -> Result<DirectionGesture, &'static str> {
    if value.source_render_id==0 {return Err("DIRECTION_SOURCE");}
    gesture(&HeroDirectionInput {schema_version:value.schema_version,source:None,sealed_source:0,
        press_id:value.press_id,phase:value.phase,x:value.x,y:value.y})
}

pub fn client_from_renderer(value:&crate::game_proto::RendererDirectionIntent, source:crate::game_proto::SecureReplicaTarget)
    -> Result<HeroDirectionInput, &'static str> {
    validate_renderer(value)?;
    if source.replica_entity_id.as_ref().map(|id|id.value)!=Some(value.source_render_id) {return Err("DIRECTION_SOURCE");}
    let client=HeroDirectionInput {schema_version:value.schema_version,source:Some(source),sealed_source:0,
        press_id:value.press_id,phase:value.phase,x:value.x,y:value.y};
    validate_client(&client)?;Ok(client)
}

pub fn validate_projected_ack(value:&crate::game_proto::TeamAcceptedInput) -> Result<(), &'static str> {
    use prost::Message;
    if value.action_kind!=ACTION_KIND || value.player_id==0 || value.input_id==0
        || !value.actor.as_ref().is_some_and(|actor|actor.value!=0) || value.target.is_some() {
        return Err("DIRECTION_ACK");
    }
    let input=crate::game_proto::PlayerInput::decode(value.sanitized_payload.as_slice()).map_err(|_|"DIRECTION_ACK")?;
    let Some(crate::game_proto::player_input::Action::Direction(direction))=input.action else {return Err("DIRECTION_ACK");};
    validate_ack(&direction)?;Ok(())
}

pub fn reject_kind_alias(value:&crate::game_proto::TeamAcceptedInput) -> Result<(), &'static str> {
    use prost::Message;
    if value.action_kind!=ACTION_KIND && crate::game_proto::PlayerInput::decode(value.sanitized_payload.as_slice())
        .is_ok_and(|input|matches!(input.action,Some(crate::game_proto::player_input::Action::Direction(_)))) {
        return Err("DIRECTION_ACK");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_proto::{SecureReplicaTarget,ReplicaEntityId,ViewEpoch,DisclosureEpoch};

    fn client() -> HeroDirectionInput {
        HeroDirectionInput {schema_version:1, source:Some(SecureReplicaTarget {
            replica_entity_id:Some(ReplicaEntityId {value:9}),
            view_epoch:Some(ViewEpoch {value:4}),disclosure_epoch:Some(DisclosureEpoch {value:3}),
        }), press_id:2, phase:1, x:1, y:0, sealed_source:0}
    }

    #[test]
    fn direction_client_sealing_is_atomic_and_does_not_accept_a_previous_seal() {
        let canonical=(2_u64<<32)|7;
        let mut view=super::super::TeamInputValidationView {view_epoch:4,..Default::default()};
        view.replicas.insert(9,super::super::ReplicaValidationRecord {canonical_id:canonical,disclosure_epoch:3,owner_team:Some(1)});
        view.visible_by_tick.insert(10,std::sync::Arc::new([canonical].into_iter().collect()));
        let mut snapshot=super::super::SecureInputValidationSnapshot::default();snapshot.teams.insert(1,view);
        for (team,epoch,tick) in [(2,4,10),(1,5,10),(1,4,9)] {
            let mut value=client();let before=value.clone();
            assert!(seal_client(&mut value,&snapshot,team,epoch,tick).is_err());assert_eq!(value,before);
        }
        let mut value=client();seal_client(&mut value,&snapshot,1,4,10).unwrap();
        assert_eq!(value.sealed_source,canonical);assert!(value.source.is_none());assert!(validate_sealed(&value).is_ok());
        let before=value.clone();assert!(seal_client(&mut value,&snapshot,1,4,10).is_err());assert_eq!(value,before);
    }

    #[test]
    fn direction_projected_ack_rejects_identity_leaks_target_and_kind_aliases() {
        use prost::Message;
        use crate::game_proto::{TeamAcceptedInput,PlayerInput,ReplicaEntityId,player_input::Action};
        let direction=HeroDirectionInput {source:None,..client()};
        let ack=TeamAcceptedInput {player_id:7,input_id:91,action_kind:ACTION_KIND,
            actor:Some(ReplicaEntityId {value:12}),target:None,
            sanitized_payload:PlayerInput {action:Some(Action::Direction(direction.clone()))}.encode_to_vec(),stable_sub_index:0};
        validate_projected_ack(&ack).unwrap();
        for bad in [TeamAcceptedInput {player_id:0,..ack.clone()},TeamAcceptedInput {input_id:0,..ack.clone()},
            TeamAcceptedInput {actor:None,..ack.clone()},TeamAcceptedInput {actor:Some(ReplicaEntityId {value:0}),..ack.clone()},
            TeamAcceptedInput {target:Some(ReplicaEntityId {value:12}),..ack.clone()},TeamAcceptedInput {action_kind:27,..ack.clone()},
            TeamAcceptedInput {sanitized_payload:vec![],..ack.clone()},
            TeamAcceptedInput {sanitized_payload:PlayerInput {action:Some(Action::Direction(client()))}.encode_to_vec(),..ack.clone()},
            TeamAcceptedInput {sanitized_payload:PlayerInput {action:Some(Action::Direction(HeroDirectionInput {sealed_source:1<<32,..direction}))}.encode_to_vec(),..ack.clone()}] {
            assert!(validate_projected_ack(&bad).is_err());
        }
    }

    #[test]
    fn direction_trust_boundaries_are_disjoint_and_identity_is_redacted() {
        let value=client();
        assert!(validate_client(&value).is_ok());
        assert!(validate_sealed(&value).is_err());assert!(validate_ack(&value).is_err());
        let sealed=HeroDirectionInput {source:None,sealed_source:(1<<32)|8,..value.clone()};
        assert!(validate_sealed(&sealed).is_ok());
        assert!(validate_client(&sealed).is_err());assert!(validate_ack(&sealed).is_err());
        let ack=HeroDirectionInput {source:None,sealed_source:0,..value};
        assert!(validate_ack(&ack).is_ok());
        assert!(validate_client(&ack).is_err());assert!(validate_sealed(&ack).is_err());
        for sealed_source in [0,8,1_u64<<63,u64::MAX] {
            assert!(validate_sealed(&HeroDirectionInput {sealed_source,..sealed.clone()}).is_err());
        }
    }

    #[test]
    fn direction_wide_aliases_and_invalid_gestures_are_rejected() {
        let value=client();
        for schema_version in [0,2,256,u32::MAX] {
            assert!(validate_client(&HeroDirectionInput {schema_version,..value.clone()}).is_err());
        }
        for phase in [0,4,257,u32::MAX] {
            assert!(validate_client(&HeroDirectionInput {phase,..value.clone()}).is_err());
        }
        for press_id in [0,u64::MAX] {
            assert!(validate_client(&HeroDirectionInput {press_id,..value.clone()}).is_err());
        }
        for axis in [i32::MIN,-32768,32768,i32::MAX] {
            assert!(validate_client(&HeroDirectionInput {x:axis,..value.clone()}).is_err());
            assert!(validate_client(&HeroDirectionInput {y:axis,..value.clone()}).is_err());
        }
        assert!(validate_client(&HeroDirectionInput {x:0,..value.clone()}).is_err());
        assert!(validate_client(&HeroDirectionInput {phase:3,..value.clone()}).is_err());
        assert_eq!(validate_client(&HeroDirectionInput {phase:3,x:0,..value.clone()}),Ok(DirectionGesture::End(2)));
        assert_eq!(validate_client(&HeroDirectionInput {phase:2,x:-32767,y:32767,..value.clone()}),Ok(DirectionGesture::Update(2)));
        for field in 0..3 {
            let mut bad=value.clone();let source=bad.source.as_mut().unwrap();
            match field {0=>source.replica_entity_id=None,1=>source.view_epoch=None,_=>source.disclosure_epoch=None}
            assert!(validate_client(&bad).is_err());
        }
        for field in 0..3 {
            let mut bad=value.clone();let source=bad.source.as_mut().unwrap();
            match field {0=>source.replica_entity_id.as_mut().unwrap().value=0,
                1=>source.view_epoch.as_mut().unwrap().value=0,
                _=>source.disclosure_epoch.as_mut().unwrap().value=0}
            assert!(validate_client(&bad).is_err());
        }
        assert!(validate_client(&HeroDirectionInput {sealed_source:1<<32,..value}).is_err());
    }
}
