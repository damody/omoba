//! Closed instant friendly control cleansing; transport is never a stored buff.
use omoba_sim::Fixed64;
use serde_json::{Value,json};
use specs::{Entity,World,WorldExt};
use super::BuffStore;
pub const KEY:&str="__ally_control_cleanse_v1";
pub const BUFF:&str="__ally_control_cleanse";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Spec {pub source_id:u32,pub source_generation:u32,pub cast_range:Fixed64}
impl Spec {
    pub fn checked(source_id:u32,source_generation:u32,cast_range:Fixed64)->Option<Self> {
        (source_id!=u32::MAX && source_generation>0 && source_generation<=i32::MAX as u32
            && cast_range>Fixed64::ZERO && cast_range<=Fixed64::from_i32(10_000))
            .then_some(Self{source_id,source_generation,cast_range})
    }
    pub fn payload(self)->Value {json!({(KEY):{"source_id":self.source_id,"source_generation":self.source_generation,"range_raw":self.cast_range.raw()}})}
    pub fn from_payload(buff_id:&str,duration:Fixed64,payload:&Value)->Option<Self> {
        if buff_id!=BUFF || duration!=Fixed64::ONE || payload.as_object()?.len()!=1{return None;}
        let row=payload.get(KEY)?.as_object()?;if row.len()!=3{return None;}
        Self::checked(row.get("source_id")?.as_u64()?.try_into().ok()?,row.get("source_generation")?.as_u64()?.try_into().ok()?,
            Fixed64::from_raw(row.get("range_raw")?.as_i64()?))
    }
}
pub fn admitted(world:&World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value,expected_source:Option<Entity>)->Option<Spec> {
    let spec=Spec::from_payload(buff_id,duration,payload)?;
    super::targeted_effect::admitted(world,target,spec.source_id,spec.source_generation,spec.cast_range,true,expected_source).then_some(spec)
}
pub fn apply(world:&mut World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value)->bool {
    if admitted(world,target,buff_id,duration,payload,None).is_none(){return false;}
    world.write_resource::<BuffStore>().cleanse_controls(target);true
}
/// Instant self slow cleanse: removes only the target's own movement slows (stuns, roots and
/// silences stay). The packet names its source, which must be the target itself.
pub const SELF_SLOW_BUFF:&str="__self_slow_cleanse";
pub fn self_slow_payload(source_id:u32,source_generation:u32)->Value {json!({"source_id":source_id,"source_generation":source_generation})}
pub fn apply_self_slow(world:&mut World,target:Entity,payload:&Value)->bool {
    let source=|key:&str|payload.get(key).and_then(Value::as_u64);
    if payload.as_object().is_none_or(|row|row.len()!=2) || source("source_id")!=Some(u64::from(target.id()))
        || source("source_generation")!=Some(target.gen().id() as u64) || !world.entities().is_alive(target) {return false;}
    world.write_resource::<BuffStore>().cleanse_slows(target);true
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn ally_cleanse_packet_is_closed_bounded_and_instant() {
        let spec=Spec::checked(0,1,Fixed64::from_i32(600)).unwrap();assert_eq!(Spec::from_payload(BUFF,Fixed64::ONE,&spec.payload()),Some(spec));
        for value in [json!(0),json!(-1),json!(0.5),json!("1024"),json!(i64::MAX)]{let mut bad=spec.payload();bad[KEY]["range_raw"]=value;assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());}
        for value in [0,u32::MAX]{let mut bad=spec.payload();bad[KEY]["source_generation"]=json!(value);assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());}
        let mut bad=spec.payload();bad[KEY]["buffs"]=json!(["anything"]);assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());
        let mut bad=spec.payload();bad["extra"]=json!(true);assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());
        assert!(Spec::from_payload(BUFF,Fixed64::from_raw(1025),&spec.payload()).is_none());assert!(Spec::from_payload("stun",Fixed64::ONE,&spec.payload()).is_none());
    }
    #[test]fn self_slow_cleanse_removes_only_the_named_sources_own_slows() {
        use specs::Builder;
        let mut world=World::new();world.insert(BuffStore::new());
        let me=world.create_entity().build();let other=world.create_entity().build();
        let slow=json!({"move_speed_bonus":-768,"__aggregation_family":"generic_ability_slow"});
        {let mut b=world.write_resource::<BuffStore>();b.add(me,"generic_slow:x:9:1",Fixed64::from_i32(3),slow.clone());b.add(me,"stun",Fixed64::from_i32(3),Value::Null);
         b.add(other,"generic_slow:x:9:1",Fixed64::from_i32(3),slow);}
        assert!(!apply_self_slow(&mut world,me,&self_slow_payload(other.id(),other.gen().id() as u32)),"source must be the target");
        assert!(apply_self_slow(&mut world,me,&self_slow_payload(me.id(),me.gen().id() as u32)));
        let b=world.read_resource::<BuffStore>();
        assert!(!b.has(me,"generic_slow:x:9:1"));assert!(b.is_stunned(me),"stuns stay");assert!(b.has(other,"generic_slow:x:9:1"));
    }
}
