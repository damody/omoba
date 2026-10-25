//! Host-only zone resolution after earlier serial damage/lifetime changes.
use specs::{World,WorldExt,Entity,Join};
use omoba_sim::Fixed64;
use crate::runtime::{comp::*,BuffStore,ability_runtime::damage_zone::DamageZoneSpec};

pub(super) fn expand(world:&mut World,source:Entity,payload:&serde_json::Value,removed:&[Entity])->Vec<Outcome> {
    let Some(spec)=DamageZoneSpec::from_payload(payload)else{return Vec::new();};
    if spec.independent {return Vec::new();}
    let faction=world.read_storage::<Faction>().get(source).cloned();
    if removed.contains(&source)||!world.entities().is_alive(source)||faction.is_none()
        ||world.read_storage::<CProperty>().get(source).is_none_or(|p|p.hp<=Fixed64::ZERO) {
        if let Some(mut buffs)=world.try_fetch_mut::<BuffStore>() {buffs.cancel_damage_zones(source);}
        return Vec::new();
    }
    expand_checked(world,source,spec,faction.unwrap(),None,removed)
}

pub(super) fn expand_independent(world:&World,source:Entity,key:&str,lease:u64,removed:&[Entity])->Vec<Outcome> {
    let zone=world.read_resource::<BuffStore>().independent_zone(source,key,lease).cloned();
    let Some(zone)=zone else{return Vec::new();};
    expand_checked(world,source,zone.spec,zone.faction,Some(zone.origin),removed)
}

pub(super) fn admit_zone(world:&World,source:Entity,key:&str,duration:Fixed64,payload:&serde_json::Value)->bool {
    let Some(spec)=DamageZoneSpec::from_payload(payload)else{return false;};
    duration>=spec.interval && duration<=Fixed64::from_i32(30) && world.entities().is_alive(source)
        && world.read_storage::<Faction>().get(source).is_some()
        && world.read_storage::<CProperty>().get(source).is_some_and(|p|p.hp>Fixed64::ZERO)
        && (!spec.independent || world.read_resource::<BuffStore>().can_schedule_independent_zones(source,&[key]))
}

fn expand_checked(world:&World,source:Entity,spec:DamageZoneSpec,faction:Faction,origin:Option<ProjectileOrigin>,removed:&[Entity])->Vec<Outcome> {
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
        pos,phys,magi,real,source,target,damage_profile:spec.profile,origin:origin.clone()}).collect();
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
    fn spec()->DamageZoneSpec {DamageZoneSpec {independent:false,center:Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO),radius:Fixed64::from_i32(100),
        amount:Fixed64::from_i32(40),interval:Fixed64::from_raw(512),kind:omb_script_abi::types::DamageKind::Magical,
        profile:omb_script_abi::types::DamageProfile::FIRE.bits()}}
    #[test]
    fn independent_zone_managed_batch_reserves_all_keys_before_any_effect() {
        let mut w=world();let source=body(&mut w,0,1);let mut s=spec();s.independent=true;
        let faction=Faction::new(FactionType::Player,1);
        let mut origin=crate::runtime::moba_match::capture_projectile_origin(&w,source).unwrap();origin.script=None;
        for n in 0..7 {assert!(w.write_resource::<BuffStore>().schedule_independent_zone(source,&format!("k{n}"),Fixed64::ONE,s,origin.clone(),faction.clone()));}
        let effects:Vec<_>=["new-a","new-b"].into_iter().map(|key|Outcome::AddBuff{target:source,buff_id:key.into(),duration:Fixed64::ONE,payload:s.payload()}).collect();
        assert!(super::super::cast_settlement::admit(&w,source,"fixture",0,Some(&faction),None,None,&effects[..1],0));
        let frozen=w.read_resource::<BuffStore>().damage_zone_diagnostic();
        assert!(!super::super::cast_settlement::admit(&w,source,"fixture",0,Some(&faction),None,None,&effects,0));
        assert_eq!(w.read_resource::<BuffStore>().damage_zone_diagnostic(),frozen);
    }
    #[test]
    fn independent_zone_survives_source_death_and_generation_reuse_with_original_allegiance() {
        for mode in 0..3 {
            let mut w=world();let source=body(&mut w,0,1);let target=body(&mut w,500,2);let ally=body(&mut w,500,1);
            let mut s=spec();s.independent=true;
            let mut origin=crate::runtime::moba_match::capture_projectile_origin(&w,source).unwrap();origin.script=None;
            assert!(w.write_resource::<BuffStore>().schedule_independent_zone(source,"zone",Fixed64::ONE,s,origin,Faction::new(FactionType::Player,1)));
            match mode {
                0=>w.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO,
                1=>{w.delete_entity(source).unwrap();w.maintain();let replacement=body(&mut w,0,2);assert_eq!(replacement.id(),source.id());assert_ne!(replacement,source);},
                _=>{*w.write_storage::<Faction>().get_mut(source).unwrap()=Faction::new(FactionType::Enemy,2);},
            }
            w.write_resource::<BuffStore>().remove_all_for(source);
            let pulses=w.write_resource::<BuffStore>().drain_damage_zones(Fixed64::from_raw(512));
            w.write_resource::<BuffStore>().tick(Fixed64::from_raw(512));
            for pulse in pulses {w.write_resource::<Vec<Outcome>>().push(Outcome::IndependentDamageZonePulse{source,key:pulse.key,lease:pulse.lease.unwrap()});}
            crate::runtime::process_outcomes(&mut w,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
            assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(970));
            assert_eq!(w.read_storage::<CProperty>().get(ally).unwrap().hp,Fixed64::from_i32(1000));
        }
    }
    #[test]
    fn independent_zone_exact_expiry_settles_but_explicit_cancel_or_refresh_revokes_old_queue() {
        for mode in 0..3 {
            let mut w=world();let source=body(&mut w,0,1);let target=body(&mut w,500,2);let mut s=spec();s.independent=true;
            let mut origin=crate::runtime::moba_match::capture_projectile_origin(&w,source).unwrap();origin.script=None;
            let faction=Faction::new(FactionType::Player,1);
            assert!(w.write_resource::<BuffStore>().schedule_independent_zone(source,"zone",Fixed64::ONE,s,origin.clone(),faction.clone()));
            let pulses=w.write_resource::<BuffStore>().drain_damage_zones(Fixed64::ONE);assert_eq!(pulses.len(),2);
            w.write_resource::<BuffStore>().tick(Fixed64::ONE);
            if mode==1 {w.write_resource::<BuffStore>().remove(source,"zone");}
            if mode==2 {assert!(w.write_resource::<BuffStore>().schedule_independent_zone(source,"zone",Fixed64::ONE,s,origin,faction));}
            for pulse in pulses {w.write_resource::<Vec<Outcome>>().push(Outcome::IndependentDamageZonePulse{source,key:pulse.key,lease:pulse.lease.unwrap()});}
            crate::runtime::process_outcomes(&mut w,&mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
            assert_eq!(w.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(if mode==0 {940}else{1000}));
        }
    }
    #[test]
    fn independent_zone_tokens_are_host_only_and_cannot_spoof_unknown_lease_or_payload() {
        let mut w=world();let source=body(&mut w,0,1);let _target=body(&mut w,500,2);let mut s=spec();s.independent=true;
        assert!(expand(&mut w,source,&s.payload(),&[]).is_empty());
        assert!(expand_independent(&w,source,"unknown",1,&[]).is_empty());
        let faction=w.read_storage::<Faction>().get(source).unwrap().clone();
        let token=Outcome::IndependentDamageZonePulse{source,key:"unknown".into(),lease:1};
        let encoded=serde_json::to_value(&token).unwrap();assert!(!encoded.to_string().contains("credit_player"));
        assert!(!super::super::cast_settlement::admit(&w,source,"fixture",0,Some(&faction),None,None,&[token],0));
    }
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
