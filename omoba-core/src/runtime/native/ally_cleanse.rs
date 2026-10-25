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
}
