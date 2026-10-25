//! Checked caster-owned anchored pulse zones carried by native stat-buff data.
use omoba_sim::{Fixed64,Vec2};
use omb_script_abi::types::{DamageKind,DamageProfile};
use serde_json::{Value,json};
use specs::Entity;

pub const KEY:&str="damage_zone";
#[derive(Clone,Copy,Debug)]
pub struct DamageZoneSpec {
    pub independent:bool,
    pub center:Vec2,pub radius:Fixed64,pub amount:Fixed64,pub interval:Fixed64,
    pub kind:DamageKind,pub profile:u32,
}
#[derive(Clone,Debug)]
pub struct DamageZonePulse {
    pub source:Entity,pub spec:DamageZoneSpec,
    pub key:String,pub lease:Option<u64>,
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
        let mut payload=json!({KEY:{"x":self.center.x.raw(),"y":self.center.y.raw(),"radius":self.radius.raw(),
            "amount":self.amount.raw(),"interval":self.interval.raw(),"kind":match self.kind {
                DamageKind::Physical=>"physical",DamageKind::Magical=>"magical",DamageKind::Pure=>"pure"},"profile":self.profile}});
        if self.independent {payload[KEY]["lifetime"]=json!("independent");}
        payload
    }
    pub fn from_payload(payload:&Value)->Option<Self> {
        if payload.as_object()?.len()!=1 {return None;}
        let row=payload.get(KEY)?;
        let independent=match (row.as_object()?.len(),row.get("lifetime")) {
            (7,None)=>false,(8,Some(value))if value.as_str()==Some("independent")=>true,_=>return None,
        };
        let raw=|name:&str|row.get(name)?.as_i64().map(Fixed64::from_raw);
        let spec=Self {independent,center:Vec2::new(raw("x")?,raw("y")?),radius:raw("radius")?,amount:raw("amount")?,interval:raw("interval")?,
            kind:match row.get("kind")?.as_str()? {"physical"=>DamageKind::Physical,"magical"=>DamageKind::Magical,"pure"=>DamageKind::Pure,_=>return None},
            profile:u32::try_from(row.get("profile")?.as_u64()?).ok()?};
        spec.valid().then_some(spec)
    }
}

pub const MAX_INDEPENDENT_ZONES:usize=128;
pub const MAX_INDEPENDENT_ZONES_PER_SOURCE:usize=8;

#[derive(Clone,Debug)]
pub struct IndependentDamageZone {
    pub spec:DamageZoneSpec,pub remaining:Fixed64,pub clock:i64,pub lease:u64,
    pub origin:crate::runtime::comp::ProjectileOrigin,pub faction:crate::runtime::comp::Faction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::BuffStore;
    use specs::{Builder,World,WorldExt};
    fn spec()->DamageZoneSpec {DamageZoneSpec {independent:false,center:Vec2::ZERO,radius:Fixed64::from_i32(100),amount:Fixed64::ONE,
        interval:Fixed64::from_raw(512),kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits()}}
    fn origin(source:Entity)->crate::runtime::comp::ProjectileOrigin {
        crate::runtime::comp::ProjectileOrigin {source,normal_attack:false,script_attack:false,attack_missed:false,
            attack_critical:false,attack_lifesteal:Fixed64::ZERO,launch_position:None,credit_team:Some(1),credit_player:Some(7),script:None}
    }
    #[test]
    fn independent_zone_caps_are_atomic_and_replacement_and_diagnostics_preserve_identity() {
        let mut world=World::new();let owners:Vec<_>=(0..17).map(|_|world.create_entity().build()).collect();
        let mut buffs=BuffStore::default();let mut s=spec();s.independent=true;
        let faction=crate::runtime::comp::Faction::new(crate::runtime::comp::FactionType::Player,1);
        for source in &owners[..16] {for n in 0..8 {assert!(buffs.schedule_independent_zone(*source,&format!("k{n}"),Fixed64::ONE,s,origin(*source),faction.clone()));}}
        let frozen=buffs.damage_zone_diagnostic();
        for (source,key) in [(owners[0],"ninth"),(owners[16],"global"),(owners[0],"bad\nkey")] {
            assert!(!buffs.schedule_independent_zone(source,key,Fixed64::ONE,s,origin(source),faction.clone()));assert_eq!(buffs.damage_zone_diagnostic(),frozen);
        }
        assert!(buffs.schedule_independent_zone(owners[0],"k0",Fixed64::ONE,s,origin(owners[0]),faction.clone()));
        assert!(buffs.independent_zone(owners[0],"k0",1).is_none());
        assert_ne!(buffs.damage_zone_diagnostic(),frozen);
        buffs.remove(owners[0],"k1");assert!(buffs.schedule_independent_zone(owners[16],"global",Fixed64::ONE,s,origin(owners[16]),faction));
        let before=buffs.damage_zone_diagnostic();assert!(buffs.drain_damage_zones(Fixed64::from_raw(1)).is_empty());
        assert_ne!(buffs.damage_zone_diagnostic(),before,"phase belongs to canonical future state");
    }
    #[test]
    fn independent_zone_partition_death_and_closed_expiry_preserve_six_exact_pulses() {
        for hz in [15,60,120] {
            let mut world=World::new();let source=world.create_entity().build();let mut buffs=BuffStore::default();
            let mut s=spec();s.independent=true;
            assert!(buffs.schedule_independent_zone(source,"zone",Fixed64::from_i32(3),s,origin(source),crate::runtime::comp::Faction::new(crate::runtime::comp::FactionType::Player,1)));
            let frozen=buffs.damage_zone_diagnostic();assert!(buffs.drain_damage_zones(Fixed64::ZERO).is_empty());
            assert!(buffs.drain_damage_zones(Fixed64::from_raw(-1)).is_empty());assert_eq!(buffs.damage_zone_diagnostic(),frozen);
            let mut elapsed=0;let mut due=Vec::new();
            for tick in 1..=hz*3 {
                let next=tick*3072/(hz*3);let dt=Fixed64::from_raw(next-elapsed);
                for pulse in buffs.drain_damage_zones(dt) {assert_eq!(pulse.lease,Some(1));due.push(elapsed+pulse.due_offset.raw());}
                buffs.tick(dt);elapsed=next;
                if tick==1 {buffs.remove_all_for(source);assert!(!buffs.has(source,"zone"));}
            }
            assert_eq!(due,vec![512,1024,1536,2048,2560,3072]);
            assert!(buffs.independent_zone(source,"zone",1).is_some());
            assert!(buffs.drain_damage_zones(Fixed64::ONE).is_empty());assert!(buffs.independent_zone(source,"zone",1).is_none());
        }
    }
    #[test]
    fn independent_zone_refresh_cancel_and_strict_payload_revoke_queued_leases() {
        let mut world=World::new();let source=world.create_entity().build();let mut buffs=BuffStore::default();
        let mut s=spec();s.independent=true;
        assert!(DamageZoneSpec::from_payload(&s.payload()).unwrap().independent);
        for value in [json!(true),json!("caster_bound"),json!(7)] {let mut p=s.payload();p[KEY]["lifetime"]=value;assert!(DamageZoneSpec::from_payload(&p).is_none());}
        buffs.add(source,"zone",Fixed64::ONE,s.payload());assert!(!buffs.has(source,"zone"));
        let faction=crate::runtime::comp::Faction::new(crate::runtime::comp::FactionType::Player,1);
        assert!(buffs.schedule_independent_zone(source,"zone",Fixed64::ONE,s,origin(source),faction.clone()));
        assert_eq!(buffs.drain_damage_zones(Fixed64::from_raw(512))[0].lease,Some(1));
        s.center.x=Fixed64::from_i32(50);
        assert!(buffs.schedule_independent_zone(source,"zone",Fixed64::ONE,s,origin(source),faction));
        assert!(buffs.independent_zone(source,"zone",1).is_none());
        assert!(buffs.drain_damage_zones(Fixed64::from_raw(256)).is_empty());
        assert_eq!(buffs.independent_zone(source,"zone",2).unwrap().spec.center,s.center);
        buffs.remove(source,"zone");assert!(buffs.independent_zone(source,"zone",2).is_none());
        assert!(buffs.drain_damage_zones(Fixed64::ONE).is_empty());
    }
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
