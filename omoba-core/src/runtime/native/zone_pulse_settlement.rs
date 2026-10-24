//! Host-only zone resolution after earlier serial damage/lifetime changes.
use specs::{World,WorldExt,Entity,Join};
use omoba_sim::Fixed64;
use crate::runtime::{comp::*,BuffStore,ability_runtime::damage_zone::DamageZoneSpec};

pub(super) fn expand(world:&mut World,source:Entity,payload:&serde_json::Value,removed:&[Entity])->Vec<Outcome> {
    let Some(spec)=DamageZoneSpec::from_payload(payload)else{return Vec::new();};
    let faction=world.read_storage::<Faction>().get(source).cloned();
    if removed.contains(&source)||!world.entities().is_alive(source)||faction.is_none()
        ||world.read_storage::<CProperty>().get(source).is_none_or(|p|p.hp<=Fixed64::ZERO) {
        if let Some(mut buffs)=world.try_fetch_mut::<BuffStore>() {buffs.cancel_damage_zones(source);}
        return Vec::new();
    }
    let faction=faction.unwrap();
    let mut targets={
        let entities=world.entities();let positions=world.read_storage::<Pos>();let factions=world.read_storage::<Faction>();
        let health=world.read_storage::<CProperty>();let buildings=world.read_storage::<IsBuilding>();
        (&entities,&positions,&factions,&health).join()
            .filter(|(target,pos,other,hp)|*target!=source&&!removed.contains(target)&&hp.hp>Fixed64::ZERO
                &&buildings.get(*target).is_none()&&faction.is_hostile_to(other)
                &&omoba_sim::terrain::point_in_swept_circle(pos.0,spec.center,spec.center,spec.radius))
            .map(|(target,pos,_,_)|(target,pos.0)).collect::<Vec<_>>()
    };
    targets.sort_by_key(|(entity,_)|(entity.id(),entity.gen().id()));
    let zero=Fixed64::ZERO;let amount=spec.amount;
    let (phys,magi,real)=match spec.kind {
        omb_script_abi::types::DamageKind::Physical=>(amount,zero,zero),
        omb_script_abi::types::DamageKind::Magical=>(zero,amount,zero),
        omb_script_abi::types::DamageKind::Pure=>(zero,zero,amount),
    };
    let mut outcomes:Vec<_>=targets.into_iter().map(|(target,pos)|Outcome::RawDamage {
        pos,phys,magi,real,source,target,damage_profile:spec.profile,origin:None}).collect();
    outcomes.push(Outcome::Explosion {pos:spec.center,radius:spec.radius,duration:Fixed64::from_raw(204)});
    outcomes
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;
    use omoba_sim::Vec2;
    fn world()->World {
        let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        w.insert(crate::runtime::TowerTemplateRegistry::default());w
    }
    fn body(w:&mut World,x:i32,team:i32)->Entity {
        w.create_entity().with(Pos(Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO)))
            .with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
            .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::from_i32(25)}).build()
    }
    fn spec()->DamageZoneSpec {DamageZoneSpec {center:Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO),radius:Fixed64::from_i32(100),
        amount:Fixed64::from_i32(40),interval:Fixed64::from_raw(512),kind:omb_script_abi::types::DamageKind::Magical,
        profile:omb_script_abi::types::DamageProfile::FIRE.bits()}}
    #[test]
    fn serial_pulse_roundtrip_samples_current_targets_and_settles_defenses_once() {
        let mut w=world();let source=body(&mut w,0,1);let entering=body(&mut w,1000,2);
        let leaving=body(&mut w,500,2);let ally=body(&mut w,500,1);let dead=body(&mut w,500,2);let building=body(&mut w,500,2);
        w.write_storage::<CProperty>().get_mut(dead).unwrap().hp=Fixed64::ZERO;
        w.write_storage::<IsBuilding>().insert(building,IsBuilding).unwrap();
        let envelope=Outcome::DamageZonePulse {source,payload:spec().payload()};
        let envelope:Outcome=serde_json::from_slice(&serde_json::to_vec(&envelope).unwrap()).unwrap();
        w.write_resource::<Vec<Outcome>>().push(envelope);
        w.write_storage::<Pos>().get_mut(entering).unwrap().0.x=Fixed64::from_i32(500);
        w.write_storage::<Pos>().get_mut(leaving).unwrap().0.x=Fixed64::from_i32(1000);
        let late=body(&mut w,600,2);
        crate::runtime::process_outcomes(&mut w,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
        for e in [entering,late] {assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(970));}
        for e in [leaving,ally,building] {assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(1000));}
        assert_eq!(w.read_storage::<CProperty>().get(dead).unwrap().hp,Fixed64::ZERO);
    }
    #[test]
    fn retired_missing_and_reused_sources_cancel_only_their_exact_generation_without_cues() {
        for mode in 0..5 {
            let mut w=world();let source=body(&mut w,0,1);let target=body(&mut w,500,2);
            w.write_resource::<BuffStore>().add(source,"active",Fixed64::from_i32(3),spec().payload());
            let mut replacement=None;let mut removed=Vec::new();
            match mode {
                0=>w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO,
                1=>{w.delete_entity(source).unwrap();w.maintain();let e=body(&mut w,0,1);assert_eq!(e.id(),source.id());assert_ne!(e,source);
                    w.write_resource::<BuffStore>().add(e,"replacement",Fixed64::from_i32(3),spec().payload());replacement=Some(e);},
                2=>removed.push(source),
                3=>{w.write_storage::<Faction>().remove(source);},
                _=>{w.write_storage::<CProperty>().remove(source);},
            }
            assert!(expand(&mut w,source,&spec().payload(),&removed).is_empty());
            assert!(!w.read_resource::<BuffStore>().has(source,"active"));
            if let Some(e)=replacement {assert!(w.read_resource::<BuffStore>().has(e,"replacement"));}
            assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(1000));
        }
    }
    #[test]
    fn invalid_pulse_payloads_emit_nothing_and_do_not_revoke_live_zone_owners() {
        let mut w=world();let source=body(&mut w,0,1);let _target=body(&mut w,500,2);
        w.write_resource::<BuffStore>().add(source,"active",Fixed64::from_i32(3),spec().payload());
        for mode in 0..4 {let mut payload=spec().payload();match mode {
            0=>payload["damage_zone"]["radius"]=serde_json::json!(0),
            1=>payload["damage_zone"]["kind"]=serde_json::json!("invalid"),
            2=>payload["damage_zone"]["x"]=serde_json::json!(i64::MAX),
            _=>payload["extra"]=serde_json::json!(1),
        };assert!(expand(&mut w,source,&payload,&[]).is_empty());assert!(w.read_resource::<BuffStore>().has(source,"active"));}
    }
    #[test]
    fn managed_script_casts_cannot_insert_host_only_deferred_pulses() {
        let mut w=world();let source=body(&mut w,0,1);let faction=w.read_storage::<Faction>().get(source).unwrap().clone();
        assert!(super::super::cast_settlement::admit(&w,source,"fixture",0,Some(&faction),None,None,&[],0));
        assert!(!super::super::cast_settlement::admit(&w,source,"fixture",0,Some(&faction),None,None,
            &[Outcome::DamageZonePulse {source,payload:spec().payload()}],0));
    }
}
