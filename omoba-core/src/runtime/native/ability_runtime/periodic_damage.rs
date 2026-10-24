//! Checked existing buff transport for continuous damage. No alternate damage sink.
use omoba_sim::Fixed64;
use omb_script_abi::types::{DamageKind,DamageProfile,EntityHandle};
use serde_json::{Value,json};
use specs::{Entity,world::Generation};

#[derive(Clone,Copy,Debug)]
pub struct PeriodicDamageSpec {
    pub rate:Fixed64,pub source:Entity,pub kind:DamageKind,pub profile:u32,
}
#[derive(Clone,Copy,Debug)]
pub struct PeriodicDamageTick {pub target:Entity,pub spec:PeriodicDamageSpec,pub amount:Fixed64}
impl PeriodicDamageSpec {
    pub fn payload(rate:Fixed64,source:EntityHandle,kind:DamageKind)->Value {
        let (kind,profile)=match kind {DamageKind::Physical=>("physical",DamageProfile::NORMAL),
            DamageKind::Magical=>("magical",DamageProfile::ENERGY),DamageKind::Pure=>("pure",DamageProfile::TRUE)};
        json!({"dot_damage":rate.raw(),"dot_kind":kind,"damage_profile":profile.bits(),
            "source_entity_id":source.id,"source_entity_gen":source.gen})
    }
    pub fn from_payload(payload:&Value)->Option<Self> {
        let rate=Fixed64::from_raw(payload.get("dot_damage")?.as_i64()?);
        if rate<=Fixed64::ZERO || rate>Fixed64::from_i32(1_000_000) {return None;}
        let id=u32::try_from(payload.get("source_entity_id")?.as_u64()?).ok()?;
        if id==u32::MAX {return None;}
        let generation=i32::try_from(payload.get("source_entity_gen")?.as_i64()?).ok()?;
        if generation<=0 {return None;}
        let profile=u32::try_from(payload.get("damage_profile")?.as_u64()?).ok()?;
        DamageProfile::from_bits(profile)?;
        // Older tower payloads are explicitly magical and omit this new field.
        let kind=match payload.get("dot_kind") {None=>DamageKind::Magical,
            Some(Value::String(s))=>match s.as_str() {"physical"=>DamageKind::Physical,"magical"=>DamageKind::Magical,
                "pure"=>DamageKind::Pure,_=>return None},_=>return None};
        Some(Self{rate,source:Entity::new(id,Generation::new(generation)),kind,profile})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World,WorldExt,Builder};
    use crate::runtime::{BuffStore,SimulationTickProfile};

    fn fixture()->(BuffStore,Entity,Entity,Value) {
        let mut world=World::new();let source=world.create_entity().build();let target=world.create_entity().build();
        let payload=PeriodicDamageSpec::payload(Fixed64::from_raw(1031),EntityHandle{id:source.id(),gen:source.gen().id() as u32},DamageKind::Physical);
        (BuffStore::default(),source,target,payload)
    }
    #[test]
    fn fractional_rate_final_partial_lifetime_is_identical_across_profiles() {
        for profile in [SimulationTickProfile::Coarse15Hz,SimulationTickProfile::Production60Hz,SimulationTickProfile::Production120Hz] {
            let (mut buffs,source,target,payload)=fixture();
            buffs.add(target,"bleed",Fixed64::from_raw(1377),payload);
            let mut total=0;
            for tick in 1..=u64::from(profile.ticks_per_game_second())*2 {
                let dt=Fixed64::from_raw(profile.fixed_raw_for_tick(tick));
                for row in buffs.drain_periodic_damage(dt) {assert_eq!(row.spec.source,source);assert_eq!(row.spec.kind,DamageKind::Physical);total+=row.amount.raw();}
                buffs.tick(dt);
            }
            assert_eq!(total,1031*1377/1024);assert!(buffs.is_empty());
            assert!(buffs.drain_periodic_damage(Fixed64::ONE).is_empty());
        }
    }
    #[test]
    fn refresh_carries_fraction_removal_resets_and_pause_never_integrates() {
        let (mut buffs,_,target,payload)=fixture();
        buffs.add(target,"bleed",Fixed64::ONE,payload.clone());
        assert!(buffs.drain_periodic_damage(Fixed64::ZERO).is_empty());
        assert!(buffs.drain_periodic_damage(Fixed64::from_raw(-1)).is_empty());
        let one=buffs.drain_periodic_damage(Fixed64::from_raw(1));assert_eq!(one[0].amount.raw(),1);
        buffs.tick(Fixed64::from_raw(1));
        buffs.add(target,"bleed",Fixed64::ONE,payload.clone());
        let rest=buffs.drain_periodic_damage(Fixed64::from_raw(1023));
        assert_eq!(one[0].amount.raw()+rest[0].amount.raw(),1031);
        buffs.remove(target,"bleed");buffs.add(target,"bleed",Fixed64::ONE,payload);
        assert_eq!(buffs.drain_periodic_damage(Fixed64::from_raw(1))[0].amount.raw(),1);
        buffs.remove(target,"bleed");assert!(buffs.drain_periodic_damage(Fixed64::ONE).is_empty());
    }
    #[test]
    fn malformed_sources_and_damage_fail_closed_without_narrowing() {
        let (_,_,_,payload)=fixture();
        for (key,value) in [("source_entity_id",json!(u64::MAX)),("source_entity_id",json!(u32::MAX)),
            ("source_entity_gen",json!(0)),("source_entity_gen",json!(-1)),("source_entity_gen",json!(u32::MAX)),
            ("damage_profile",json!(u64::MAX)),("damage_profile",json!(1u32<<31)),("dot_kind",json!("unknown")),
            ("dot_damage",json!(-1)),("dot_damage",json!(1_000_001i64*1024))] {
            let mut bad=payload.clone();bad[key]=value;assert!(PeriodicDamageSpec::from_payload(&bad).is_none(),"{key}: {bad}");
        }
        let mut legacy=payload;legacy.as_object_mut().unwrap().remove("dot_kind");
        assert_eq!(PeriodicDamageSpec::from_payload(&legacy).unwrap().kind,DamageKind::Magical);
    }
    #[test]
    fn changed_source_never_inherits_another_generation_fraction() {
        let (mut buffs,_,target,mut payload)=fixture();payload["dot_damage"]=json!(1);
        buffs.add(target,"key",Fixed64::ONE,payload.clone());
        assert!(buffs.drain_periodic_damage(Fixed64::from_raw(512)).is_empty());
        payload["source_entity_gen"]=json!(2);buffs.add(target,"key",Fixed64::ONE,payload);
        assert!(buffs.drain_periodic_damage(Fixed64::from_raw(512)).is_empty());
        let rows=buffs.drain_periodic_damage(Fixed64::from_raw(512));assert_eq!(rows[0].amount.raw(),1);
        assert_eq!(rows[0].spec.source.gen().id(),2);
    }
    #[test]
    fn multiple_sources_and_keys_have_stable_damage_order_and_independent_carry() {
        let (mut buffs,_,target,payload)=fixture();
        buffs.add(target,"z",Fixed64::ONE,payload.clone());
        let mut other=payload;other["dot_damage"]=json!(2062);
        buffs.add(target,"a",Fixed64::ONE,other);
        let hits=buffs.drain_periodic_damage(Fixed64::ONE);assert_eq!(hits.len(),2);
        assert_eq!(hits.iter().map(|h|h.amount.raw()).collect::<Vec<_>>(),vec![2062,1031]);
        assert_eq!(buffs.tick(Fixed64::ONE).iter().map(|(_,key,_)|key.as_str()).collect::<Vec<_>>(),vec!["a","z"]);
    }
}
