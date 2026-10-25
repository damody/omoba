//! Closed script grant envelope; never publish it as a shield's mutable payload.
use omoba_sim::Fixed64;
use serde_json::{Value,json};
use specs::{Entity,World,WorldExt};
use super::ability_runtime::BuffStore;

pub const KEY:&str="__ally_shield_grant_v1";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Spec {pub source_id:u32,pub source_generation:u32,pub amount:Fixed64,pub cast_range:Fixed64}
impl Spec {
    pub fn checked(source_id:u32,source_generation:u32,amount:Fixed64,cast_range:Fixed64,duration:Fixed64)->Option<Self> {
        (source_id!=u32::MAX && source_generation>0 && source_generation<=i32::MAX as u32
            && amount>Fixed64::ZERO && amount<=Fixed64::from_i32(1_000_000)
            && cast_range>Fixed64::ZERO && cast_range<=Fixed64::from_i32(10_000)
            && duration>Fixed64::ZERO && duration<=Fixed64::from_i32(60))
            .then_some(Self {source_id,source_generation,amount,cast_range})
    }
    pub fn payload(self)->Value {json!({(KEY):{"source_id":self.source_id,"source_generation":self.source_generation,
        "amount_raw":self.amount.raw(),"range_raw":self.cast_range.raw()}})}
    pub fn from_payload(buff_id:&str,duration:Fixed64,payload:&Value)->Option<Self> {
        if buff_id!="damage_shield" || payload.as_object()?.len()!=1 {return None;}
        let row=payload.get(KEY)?.as_object()?;if row.len()!=4 {return None;}
        Self::checked(row.get("source_id")?.as_u64()?.try_into().ok()?,row.get("source_generation")?.as_u64()?.try_into().ok()?,
            Fixed64::from_raw(row.get("amount_raw")?.as_i64()?),Fixed64::from_raw(row.get("range_raw")?.as_i64()?),duration)
    }
}

pub fn admitted(world:&World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value,expected_source:Option<Entity>)->Option<Spec> {
    let spec=Spec::from_payload(buff_id,duration,payload)?;
    if !super::targeted_effect::admitted(world,target,spec.source_id,spec.source_generation,spec.cast_range,true,expected_source){return None;}
    Some(spec)
}
pub fn apply(world:&mut World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value)->bool {
    let Some(spec)=admitted(world,target,buff_id,duration,payload,None)else{return false;};
    world.write_resource::<BuffStore>().grant_damage_shield(target,spec.amount,duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ally_shield_packet_rejects_ambiguous_shapes_types_and_numeric_envelopes() {
        let valid=Spec::checked(0,1,Fixed64::ONE,Fixed64::from_i32(600),Fixed64::ONE).unwrap();
        assert_eq!(Spec::from_payload("damage_shield",Fixed64::ONE,&valid.payload()),Some(valid));
        for field in ["amount_raw","range_raw"] {for value in [json!(0),json!(-1),json!(0.5),json!("1024"),json!(i64::MAX)] {
            let mut bad=valid.payload();bad[KEY][field]=value;assert!(Spec::from_payload("damage_shield",Fixed64::ONE,&bad).is_none());}}
        for duration in [Fixed64::ZERO,Fixed64::from_raw(-1),Fixed64::from_i32(61)]{assert!(Spec::from_payload("damage_shield",duration,&valid.payload()).is_none());}
        for generation in [0,u32::MAX] {let mut bad=valid.payload();bad[KEY]["source_generation"]=json!(generation);assert!(Spec::from_payload("damage_shield",Fixed64::ONE,&bad).is_none());}
        let mut bad=valid.payload();bad["shield_remaining_raw"]=json!(1024);assert!(Spec::from_payload("damage_shield",Fixed64::ONE,&bad).is_none());
        let mut bad=valid.payload();bad[KEY]["unknown"]=json!(1);assert!(Spec::from_payload("damage_shield",Fixed64::ONE,&bad).is_none());
        assert!(Spec::from_payload("different_buff",Fixed64::ONE,&valid.payload()).is_none());
    }
}
