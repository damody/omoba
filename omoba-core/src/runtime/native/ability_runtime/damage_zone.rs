//! Checked caster-owned anchored pulse zones carried by native stat-buff data.
use omoba_sim::{Fixed64,Vec2};
use omb_script_abi::types::{DamageKind,DamageProfile};
use serde_json::{Value,json};
use specs::Entity;

pub const KEY:&str="damage_zone";
#[derive(Clone,Copy,Debug)]
pub struct DamageZoneSpec {
    pub center:Vec2,pub radius:Fixed64,pub amount:Fixed64,pub interval:Fixed64,
    pub kind:DamageKind,pub profile:u32,
}
#[derive(Clone,Copy,Debug)]
pub struct DamageZonePulse {
    pub source:Entity,pub spec:DamageZoneSpec,
    /// Exact positive offset from this drain's tick start, capped by active lifetime.
    pub due_offset:Fixed64,
}
impl DamageZoneSpec {
    pub fn valid(&self)->bool {
        let max=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
        self.center.x.raw().unsigned_abs()<=max && self.center.y.raw().unsigned_abs()<=max
            && self.radius>Fixed64::ZERO && self.radius<=Fixed64::from_i32(10_000)
            && self.amount>=Fixed64::ZERO && self.amount<=Fixed64::from_i32(1_000_000)
            && self.interval>=Fixed64::from_raw(256) && self.interval<=Fixed64::from_i32(10)
            && DamageProfile::from_bits(self.profile).is_some()
    }
    pub fn payload(&self)->Value {
        json!({KEY:{"x":self.center.x.raw(),"y":self.center.y.raw(),"radius":self.radius.raw(),
            "amount":self.amount.raw(),"interval":self.interval.raw(),"kind":match self.kind {
                DamageKind::Physical=>"physical",DamageKind::Magical=>"magical",DamageKind::Pure=>"pure"},"profile":self.profile}})
    }
    pub fn from_payload(payload:&Value)->Option<Self> {
        if payload.as_object()?.len()!=1 {return None;}
        let row=payload.get(KEY)?;
        if row.as_object()?.len()!=7 {return None;}
        let raw=|name:&str|row.get(name)?.as_i64().map(Fixed64::from_raw);
        let spec=Self {center:Vec2::new(raw("x")?,raw("y")?),radius:raw("radius")?,amount:raw("amount")?,interval:raw("interval")?,
            kind:match row.get("kind")?.as_str()? {"physical"=>DamageKind::Physical,"magical"=>DamageKind::Magical,"pure"=>DamageKind::Pure,_=>return None},
            profile:u32::try_from(row.get("profile")?.as_u64()?).ok()?};
        spec.valid().then_some(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::BuffStore;
    use specs::{Builder,World,WorldExt};
    fn spec()->DamageZoneSpec {DamageZoneSpec {center:Vec2::ZERO,radius:Fixed64::from_i32(100),amount:Fixed64::ONE,
        interval:Fixed64::from_raw(512),kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits()}}
    #[test]
    fn checked_zone_rejects_malformed_integer_geometry_kind_and_profile() {
        let s=spec();assert!(DamageZoneSpec::from_payload(&s.payload()).is_some());
        for (key,value) in [("x",json!(i64::MAX)),("y",json!("0")),("radius",json!(-1)),("amount",json!(1.5)),
            ("interval",json!(255)),("kind",json!("other")),("profile",json!(0)),("profile",json!(u64::MAX))] {
            let mut p=s.payload();p[KEY][key]=value;assert!(DamageZoneSpec::from_payload(&p).is_none(),"{key}");
        }
    }
    #[test]
    fn zone_pulses_integrate_before_expiry_without_early_or_lost_catchup_hits() {
        for hz in [15,60,120] {
            let mut world=World::new();let source=world.create_entity().build();let mut buffs=BuffStore::default();
            buffs.add(source,"zone",Fixed64::from_i32(3),spec().payload());
            let mut count=0;let mut elapsed=0;
            for tick in 1..=hz*3 {
                let next=tick*1024/hz;let dt=Fixed64::from_raw(next-elapsed);elapsed=next;
                let pulses=buffs.drain_damage_zones(dt);if elapsed<512 {assert!(pulses.is_empty());}
                count+=pulses.len();buffs.tick(dt);
            }
            assert_eq!(count,6);assert!(!buffs.has(source,"zone"));
        }
        let mut world=World::new();let source=world.create_entity().build();let mut buffs=BuffStore::default();
        buffs.add(source,"zone",Fixed64::from_i32(3),spec().payload());
        assert_eq!(buffs.drain_damage_zones(Fixed64::from_i32(10)).len(),6);
    }
    #[test]
    fn zone_due_time_precedes_source_id_and_simultaneous_keys_are_canonical() {
        let mut world=World::new();let low=world.create_entity().build();let high=world.create_entity().build();
        let mut buffs=BuffStore::default();let mut late=spec();late.interval=Fixed64::from_raw(256);late.amount=Fixed64::from_i32(3);
        buffs.add(low,"late",Fixed64::from_i32(3),late.payload());
        assert!(buffs.drain_damage_zones(Fixed64::from_raw(252)).is_empty());buffs.tick(Fixed64::from_raw(252));
        // Insert reverse lexical order deliberately; both high-ID zones share phase.
        for (key,amount) in [("z",2),("a",1)] {let mut s=spec();s.amount=Fixed64::from_i32(amount);buffs.add(high,key,Fixed64::from_i32(3),s.payload());}
        assert_eq!(buffs.drain_damage_zones(Fixed64::from_raw(500)).len(),2);buffs.tick(Fixed64::from_raw(500));
        let pulses=buffs.drain_damage_zones(Fixed64::from_raw(20));
        assert_eq!(pulses.iter().map(|p|(p.source,p.due_offset.raw(),p.spec.amount.raw())).collect::<Vec<_>>(),
            vec![(high,12,1024),(high,12,2048),(low,16,3072)]);
    }
    #[test]
    fn zone_absolute_due_stream_is_partition_invariant_and_expiry_closed() {
        let stream=|hz:Option<i64>| {
            let mut world=World::new();let a=world.create_entity().build();let b=world.create_entity().build();
            let mut buffs=BuffStore::default();let mut first=spec();first.interval=Fixed64::from_raw(256);
            buffs.add(a,"first",Fixed64::from_i32(3),first.payload());
            assert!(buffs.drain_damage_zones(Fixed64::from_raw(13)).is_empty());buffs.tick(Fixed64::from_raw(13));
            buffs.add(b,"second",Fixed64::from_i32(3),spec().payload());
            assert!(buffs.drain_damage_zones(Fixed64::ZERO).is_empty());
            assert!(buffs.drain_damage_zones(Fixed64::from_raw(-1)).is_empty());
            let mut result=Vec::new();let mut elapsed=0;
            let steps=hz.map(|n|n*3).unwrap_or(1);
            for tick in 1..=steps {
                let next=if hz.is_some(){tick*3072/steps}else{4096};let dt=Fixed64::from_raw(next-elapsed);
                for pulse in buffs.drain_damage_zones(dt) {
                    assert!(pulse.due_offset>Fixed64::ZERO&&pulse.due_offset<=dt);
                    result.push((elapsed+pulse.due_offset.raw(),pulse.source.id()));
                }
                buffs.tick(dt);elapsed=next;
            }
            assert!(buffs.drain_damage_zones(Fixed64::ONE).is_empty());result
        };
        let expected=stream(None);assert_eq!(expected.len(),18);
        assert!(expected.windows(2).all(|pair|pair[0]<=pair[1]));
        assert_eq!(expected.last().unwrap().0,3072,"exact expiry endpoint still pulses");
        for hz in [15,60,120] {assert_eq!(stream(Some(hz)),expected,"{hz}Hz");}
    }
    #[test]
    fn zone_refresh_reanchors_resets_clock_removal_stops_and_order_is_canonical() {
        let mut world=World::new();let a=world.create_entity().build();let b=world.create_entity().build();let mut buffs=BuffStore::default();
        for source in [b,a] {for key in ["z","a"] {buffs.add(source,key,Fixed64::from_i32(3),spec().payload());}}
        assert!(buffs.drain_damage_zones(Fixed64::from_raw(256)).is_empty());buffs.tick(Fixed64::from_raw(256));
        let mut moved=spec();moved.center.x=Fixed64::from_i32(50);
        buffs.add(a,"a",Fixed64::ONE,moved.payload());
        assert_eq!(buffs.get(a,"a").unwrap().remaining,Fixed64::ONE,"zone replacement resets lifetime as well as clock");
        let pulses=buffs.drain_damage_zones(Fixed64::from_raw(256));assert_eq!(pulses.iter().map(|p|p.source).collect::<Vec<_>>(),vec![a,b,b]);
        buffs.tick(Fixed64::from_raw(256));buffs.remove(a,"z");buffs.remove(b,"a");buffs.remove(b,"z");
        assert!(buffs.drain_damage_zones(Fixed64::ZERO).is_empty());
        let pulses=buffs.drain_damage_zones(Fixed64::from_raw(256));assert_eq!(pulses.len(),1);assert_eq!(pulses[0].spec.center,moved.center);
        buffs.remove(a,"a");assert!(buffs.drain_damage_zones(Fixed64::from_i32(5)).is_empty());
    }
}
