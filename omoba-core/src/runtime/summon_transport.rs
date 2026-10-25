//! Opaque client references, authority lives and redacted ACKs are disjoint.
use crate::game_proto::{OwnedSummonOrder,SecureReplicaTarget,SummonCommandInput,TeamAcceptedInput,Vec2I};
pub const ACTION_KIND:u32=29;
fn shape(operation:u32,point:Option<&Vec2I>)->Result<(),&'static str> {
    match (operation,point) {
        (1,Some(p)) if i64::from(p.x).unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
            && i64::from(p.y).unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64=>Ok(()),
        (2,None)=>Ok(()),_=>Err("SUMMON_SHAPE"),
    }
}
fn reference(value:Option<&SecureReplicaTarget>)->Result<super::SecureTargetReference,&'static str> {
    let value=value.ok_or("SUMMON_REFERENCE")?;
    let result=super::SecureTargetReference {replica_id:value.replica_entity_id.as_ref().ok_or("SUMMON_REFERENCE")?.value,
        view_epoch:value.view_epoch.as_ref().ok_or("SUMMON_REFERENCE")?.value,
        disclosure_epoch:value.disclosure_epoch.as_ref().ok_or("SUMMON_REFERENCE")?.value};
    if result.replica_id==0 || result.view_epoch==0 || result.disclosure_epoch==0 {return Err("SUMMON_REFERENCE");}Ok(result)
}
pub fn validate_client(value:&SummonCommandInput)->Result<(),&'static str> {
    shape(value.operation,value.point.as_ref())?;
    let owner=reference(value.owner.as_ref())?;let summon=reference(value.summon.as_ref())?;
    if owner.replica_id==summon.replica_id || owner.view_epoch!=summon.view_epoch {return Err("SUMMON_REFERENCE");}Ok(())
}
pub fn client_from_references(owner:SecureReplicaTarget,summon:SecureReplicaTarget,operation:u32,point:Option<Vec2I>)->Result<SummonCommandInput,&'static str> {
    let value=SummonCommandInput {owner:Some(owner),summon:Some(summon),operation,point};validate_client(&value)?;Ok(value)
}
pub fn validate_authority(value:&OwnedSummonOrder)->Result<(u64,u64),&'static str> {
    shape(value.operation,value.point.as_ref())?;
    if value.owner_generation==0 || value.owner_generation>i32::MAX as u32
        || value.summon_generation==0 || value.summon_generation>i32::MAX as u32 {return Err("SUMMON_LIFE");}
    let owner=(u64::from(value.owner_generation)<<32)|u64::from(value.owner_entity);
    let summon=(u64::from(value.summon_generation)<<32)|u64::from(value.summon_entity);
    if owner==summon {return Err("SUMMON_LIFE");}Ok((owner,summon))
}
/// Resolves both references only in the authenticated current view. Exact
/// player/summoner ownership is rechecked by the existing formal authority.
pub fn seal_client(value:&SummonCommandInput,snapshot:&super::SecureInputValidationSnapshot,team:u32,epoch:u64,tick:u64)->Result<OwnedSummonOrder,&'static str> {
    validate_client(value)?;let owner=reference(value.owner.as_ref())?;let summon=reference(value.summon.as_ref())?;
    if epoch==0 || owner.view_epoch!=epoch {return Err("SUMMON_REFERENCE");}
    let owner=snapshot.validate_actor_and_resolve(team,tick,owner).map_err(|_|"SUMMON_REFERENCE")?;
    let summon=snapshot.validate_actor_and_resolve(team,tick,summon).map_err(|_|"SUMMON_REFERENCE")?;
    let result=OwnedSummonOrder {owner_entity:owner as u32,owner_generation:(owner>>32)as u32,
        summon_entity:summon as u32,summon_generation:(summon>>32)as u32,operation:value.operation,point:value.point};
    validate_authority(&result)?;Ok(result)
}
pub fn redact(value:&mut OwnedSummonOrder)->Result<(),&'static str> {
    validate_authority(value)?;value.owner_entity=0;value.owner_generation=0;value.summon_entity=0;value.summon_generation=0;Ok(())
}
pub fn validate_projected_ack(value:&TeamAcceptedInput)->Result<(),&'static str> {
    use prost::Message;
    if value.action_kind!=ACTION_KIND || value.player_id==0 || value.input_id==0 {return Err("SUMMON_ACK");}
    let owner=value.actor.as_ref().ok_or("SUMMON_ACK")?.value;let summon=value.target.as_ref().ok_or("SUMMON_ACK")?.value;
    if owner==0 || summon==0 || owner==summon {return Err("SUMMON_ACK");}
    let input=crate::game_proto::PlayerInput::decode(value.sanitized_payload.as_slice()).map_err(|_|"SUMMON_ACK")?;
    let Some(crate::game_proto::player_input::Action::OwnedSummonOrder(order))=input.action else {return Err("SUMMON_ACK");};
    shape(order.operation,order.point.as_ref())?;
    if order.owner_entity!=0 || order.owner_generation!=0 || order.summon_entity!=0 || order.summon_generation!=0 {return Err("SUMMON_ACK");}Ok(())
}
pub fn reject_kind_alias(value:&TeamAcceptedInput)->Result<(),&'static str> {
    use prost::Message;use crate::game_proto::player_input::Action;
    let Ok(input)=crate::game_proto::PlayerInput::decode(value.sanitized_payload.as_slice()) else {return Ok(());};
    if matches!(input.action,Some(Action::SummonCommand(_)))
        || (value.action_kind!=ACTION_KIND && matches!(input.action,Some(Action::OwnedSummonOrder(_)))) {return Err("SUMMON_ACK");}Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;use super::super::*;use prost::Message;
    use crate::game_proto::{ReplicaEntityId,ViewEpoch,DisclosureEpoch,PlayerInput,player_input::Action};
    fn reference(id:u64)->SecureReplicaTarget {SecureReplicaTarget {replica_entity_id:Some(ReplicaEntityId{value:id}),
        view_epoch:Some(ViewEpoch{value:4}),disclosure_epoch:Some(DisclosureEpoch{value:3})}}
    fn client()->SummonCommandInput {client_from_references(reference(9),reference(12),1,Some(Vec2I{x:100,y:0})).unwrap()}
    fn snapshot()->SecureInputValidationSnapshot {
        let mut view=TeamInputValidationView {view_epoch:4,..Default::default()};
        for (id,canonical) in [(9,(2_u64<<32)|7),(12,(3_u64<<32)|8)] {view.replicas.insert(id,ReplicaValidationRecord {canonical_id:canonical,disclosure_epoch:3,owner_team:Some(1)});}
        view.visible_by_tick.insert(10,view.replicas.values().map(|r|r.canonical_id).collect());
        let mut s=SecureInputValidationSnapshot::default();s.teams.insert(1,view);s
    }
    #[test]
    fn summon_wire_shape_is_closed_and_client_has_no_canonical_life() {
        let good=client();let wire=PlayerInput {action:Some(Action::SummonCommand(good.clone()))};
        assert_eq!(PlayerInput::decode(wire.encode_to_vec().as_slice()).unwrap(),wire);
        for bad in [SummonCommandInput{operation:0,..good.clone()},SummonCommandInput{operation:2,..good.clone()},
            SummonCommandInput{point:None,..good.clone()},SummonCommandInput{owner:None,..good.clone()},
            SummonCommandInput{summon:Some(reference(9)),..good.clone()},
            SummonCommandInput{point:Some(Vec2I{x:i32::MAX,y:0}),..good.clone()}] {assert!(validate_client(&bad).is_err());}
        validate_client(&SummonCommandInput{operation:2,point:None,..good}).unwrap();
    }
    #[test]
    fn summon_ui_renderer_contract_keeps_both_opaque_lives_and_rejects_bad_shape() {
        let value=client();let input=PlayerInput{action:Some(Action::SummonCommand(value.clone()))};
        let intent=crate::renderer_protocol::player_input_to_renderer_intent(&input).unwrap();
        assert_eq!(intent,crate::game_proto::renderer_input::Intent::SummonCommand(value.clone()));
        let wire=crate::game_proto::RendererInput{request_id:7,player_id:9,disclosure_epoch:4,intent:Some(intent)};
        assert_eq!(crate::game_proto::RendererInput::decode(wire.encode_to_vec().as_slice()).unwrap(),wire);
        let mut bad=value;bad.point=None;
        assert!(crate::renderer_protocol::player_input_to_renderer_intent(&PlayerInput{action:Some(Action::SummonCommand(bad))}).is_none());
        assert_eq!(crate::renderer_protocol::PRESENTATION_PROTOCOL_VERSION,12);
    }
    #[test]
    fn summon_wire_seals_both_current_owned_references_atomically() {
        let value=client();let original=value.clone();let mut s=snapshot();
        for (team,epoch,tick) in [(2,4,10),(1,5,10),(1,4,9)] {assert!(seal_client(&value,&s,team,epoch,tick).is_err());assert_eq!(value,original);}
        let sealed=seal_client(&value,&s,1,4,10).unwrap();assert_eq!(validate_authority(&sealed),Ok(((2_u64<<32)|7,(3_u64<<32)|8)));
        for field in 0..4 {
            let mut bad=value.clone();let target=bad.summon.as_mut().unwrap();
            match field {0=>target.replica_entity_id.as_mut().unwrap().value=999,1=>target.view_epoch.as_mut().unwrap().value=5,
                2=>target.disclosure_epoch.as_mut().unwrap().value=2,_=>target.replica_entity_id=None}
            assert!(seal_client(&bad,&s,1,4,10).is_err());
        }
        s.teams.get_mut(&1).unwrap().replicas.get_mut(&12).unwrap().owner_team=Some(2);
        assert!(seal_client(&value,&s,1,4,10).is_err());
        s.teams.get_mut(&1).unwrap().replicas.get_mut(&12).unwrap().owner_team=Some(1);
        s.teams.get_mut(&1).unwrap().replicas.get_mut(&12).unwrap().canonical_id=8;
        assert!(seal_client(&value,&s,1,4,10).is_err());
        assert_eq!(value,original);
    }
    #[test]
    fn summon_wire_ack_redacts_lives_rejects_leaks_and_kind_aliases() {
        let mut order=seal_client(&client(),&snapshot(),1,4,10).unwrap();let sealed=order.clone();
        redact(&mut order).unwrap();assert!(validate_authority(&order).is_err());
        let ack=TeamAcceptedInput {player_id:7,input_id:91,action_kind:ACTION_KIND,actor:Some(ReplicaEntityId{value:9}),target:Some(ReplicaEntityId{value:12}),
            sanitized_payload:PlayerInput{action:Some(Action::OwnedSummonOrder(order))}.encode_to_vec(),stable_sub_index:0};
        validate_projected_ack(&ack).unwrap();
        for bad in [TeamAcceptedInput{player_id:0,..ack.clone()},TeamAcceptedInput{input_id:0,..ack.clone()},
            TeamAcceptedInput{target:None,..ack.clone()},TeamAcceptedInput{target:ack.actor,..ack.clone()},TeamAcceptedInput{sanitized_payload:vec![255],..ack.clone()},
            TeamAcceptedInput{sanitized_payload:PlayerInput{action:Some(Action::OwnedSummonOrder(sealed))}.encode_to_vec(),..ack.clone()}] {assert!(validate_projected_ack(&bad).is_err());}
        assert!(reject_kind_alias(&TeamAcceptedInput{action_kind:2,..ack.clone()}).is_err());
        assert!(reject_kind_alias(&TeamAcceptedInput{sanitized_payload:PlayerInput{action:Some(Action::SummonCommand(client()))}.encode_to_vec(),..ack}).is_err());
    }
    #[test]
    fn summon_wire_projected_ack_does_not_reexecute_hero_or_private_summon_movement() {
        use std::collections::{BTreeSet,BTreeMap};
        let owner=(2_u64<<32)|7;let summon=(3_u64<<32)|8;let visible=BTreeSet::from([owner,summon]);
        let baseline=|kind| {
            let mut components=vec![(DEMO_RENDER_COMPONENT_SCHEMA_ID,encode_demo_render_state(DemoRenderState{x_raw:0,y_raw:0,team_id:1,kind,owner_player_id:7}))];
            if kind==1 {components.extend([
                (DISCLOSED_HERO_COMPONENT_SCHEMA_ID,canonical_disclosed_json(&Hero::default())),
                (DISCLOSED_FACING_COMPONENT_SCHEMA_ID,canonical_disclosed_json(&Facing::default())),
                (DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID,canonical_disclosed_json(&TurnSpeed::default())),
                (DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID,canonical_disclosed_json(&CollisionRadius::default())),
            ]);}
            encode_disclosed_baseline(&components)
        };
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig{hash_checkpoint_interval_ticks:1,..Default::default()});
        let start=projector.build_team_game_start(1,60,0);let allow=secure_replica_component_allowlist();let resources=BTreeSet::new();
        let mut replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),resources.clone()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow,resources);let graph=ProjectionDependencyGraph::default();
        let first=projector.build_frame_with_inputs(1,1,&visible,vec![
            VisibilityTransition::Reveal{canonical_id:owner,effective_tick:1,baseline:baseline(1)},
            VisibilityTransition::Reveal{canonical_id:summon,effective_tick:1,baseline:baseline(4)}],&[],&graph,vec![]).unwrap().frame;
        assert!(matches!(replica.apply_frame(first,&mut stepper).unwrap(),FrameApplyResult::Applied{..}));
        let mut order=OwnedSummonOrder{owner_entity:7,owner_generation:2,summon_entity:8,summon_generation:3,operation:1,point:Some(Vec2I{x:500*1024,y:0})};redact(&mut order).unwrap();
        projector.refresh_expected_visible_components(2,&visible,&BTreeMap::from([(owner,baseline(1)),(summon,baseline(4))]));
        let second=projector.build_frame_with_inputs(2,2,&visible,vec![],&[],&graph,vec![CanonicalAcceptedInput::from_authoritative_acceptance(
            1,7,91,ACTION_KIND,owner,Some(summon),PlayerInput{action:Some(Action::OwnedSummonOrder(order))}.encode_to_vec())]).unwrap().frame;
        let ack=&second.step.as_ref().unwrap().accepted_inputs[0];validate_projected_ack(ack).unwrap();
        let mut bad=second.clone();bad.step.as_mut().unwrap().accepted_inputs[0].action_kind=2;
        assert!(replica.apply_frame(bad,&mut stepper).is_err());
        let expected=second.post_step.as_ref().unwrap().hash_checkpoint.as_ref().unwrap().canonical_team_hash.clone();
        assert!(matches!(replica.apply_frame(second,&mut stepper).unwrap(),FrameApplyResult::Applied{..}));assert_eq!(replica.canonical_team_hash().as_slice(),expected);
        for entity in replica.world().entities.values(){let render=decode_demo_render_state(&entity.components[&DEMO_RENDER_COMPONENT_SCHEMA_ID]).unwrap();assert_eq!(render.x_raw,0);}
    }
}
