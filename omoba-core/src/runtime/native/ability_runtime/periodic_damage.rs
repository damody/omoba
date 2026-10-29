//! Checked existing buff transport for continuous damage. No alternate damage sink.
use omoba_sim::Fixed64;
use omb_script_abi::types::{DamageKind,DamageProfile,EntityHandle};
use serde_json::{Value,json};
use specs::{Entity,world::Generation};

#[derive(Clone,Copy,Debug)]
pub struct PeriodicDamageSpec {
    pub rate:Fixed64,pub source:Entity,pub kind:DamageKind,pub profile:u32,
    /// Discrete ticks (`dot_interval`): `rate × interval` once per full interval
    /// of the buff's life (Dota ThinkInterval). None: continuous integration.
    pub interval:Option<Fixed64>,
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
    /// Adds a discrete tick interval (1 raw..=60 s) to a [`Self::payload`].
    pub fn with_interval(mut payload:Value,interval:Fixed64)->Value {
        if let Some(row)=payload.as_object_mut() {row.insert("dot_interval".into(),json!(interval.raw()));}
        payload
    }
    /// Marks a [`Self::payload`]'s ticks as spell damage (E188: the source modifier's own
    /// `Damage` action, so the source's spell amplification scales each tick).
    pub fn with_spell(mut payload:Value)->Value {
        if let Some(profile)=payload.get("damage_profile").and_then(Value::as_u64) {
            payload["damage_profile"]=json!(profile|u64::from(DamageProfile::SPELL.bits()));
        }
        payload
    }
    /// Damage of one discrete tick: `rate × interval`, floored to a raw quantum.
    pub fn pulse_amount(&self)->Option<Fixed64> {
        let interval=self.interval?;
        Some(Fixed64::from_raw((i128::from(self.rate.raw())*i128::from(interval.raw())/1024) as i64))
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
        let interval=match payload.get("dot_interval") {
            None=>None,
            Some(value)=>{
                let interval=Fixed64::from_raw(value.as_i64()?);
                if interval<Fixed64::from_raw(1) || interval>Fixed64::from_i32(60) {return None;}
                Some(interval)
            }
        };
        Some(Self{rate,source:Entity::new(id,Generation::new(generation)),kind,profile,interval})
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
    fn discrete_interval_pays_once_per_full_interval_including_the_last_at_expiry() {
        // A34R: 50 per 1 s for 5 s → pulses at 1,2,3,4,5 s on every tick profile.
        for profile in [SimulationTickProfile::Coarse15Hz,SimulationTickProfile::Production60Hz,SimulationTickProfile::Production120Hz] {
            let (mut buffs,_,target,payload)=fixture();
            let payload=PeriodicDamageSpec::with_interval({let mut p=payload;p["dot_damage"]=json!(50*1024);p},Fixed64::ONE);
            buffs.add(target,"breath",Fixed64::from_i32(5),payload);
            let mut pulses=Vec::new();
            for tick in 1..=u64::from(profile.ticks_per_game_second())*7 {
                let dt=Fixed64::from_raw(profile.fixed_raw_for_tick(tick));
                for row in buffs.drain_periodic_damage(dt) {pulses.push((tick,row.amount));}
                buffs.tick(dt);
            }
            let per_second=u64::from(profile.ticks_per_game_second());
            assert_eq!(pulses,(1..=5).map(|s|(s*per_second,Fixed64::from_i32(50))).collect::<Vec<_>>(),"{profile:?}");
            assert!(buffs.is_empty());
        }
        // Bounds: 1 raw..=60 s; the continuous default is unchanged.
        let (_,_,_,payload)=fixture();
        assert!(PeriodicDamageSpec::from_payload(&payload).unwrap().interval.is_none());
        for bad in [0,-1,61*1024] {assert!(PeriodicDamageSpec::from_payload(&PeriodicDamageSpec::with_interval(payload.clone(),Fixed64::from_raw(bad))).is_none());}
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
