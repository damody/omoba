//! Closed instant hostile resource grant. Never store this packet as a buff.
use omoba_sim::Fixed64;
use serde_json::{Value,json};
use specs::{Entity,World,WorldExt};
use super::comp::Hero;
pub const KEY:&str="__enemy_mana_drain_v1";
pub const BUFF:&str="__enemy_mana_drain";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Spec {pub source_id:u32,pub source_generation:u32,pub amount:Fixed64,pub cast_range:Fixed64}
impl Spec {
    pub fn checked(source_id:u32,source_generation:u32,amount:Fixed64,cast_range:Fixed64)->Option<Self> {
        (source_id!=u32::MAX && source_generation>0 && source_generation<=i32::MAX as u32
            && amount>Fixed64::ZERO && amount<=Fixed64::from_i32(1_000_000)
            && cast_range>Fixed64::ZERO && cast_range<=Fixed64::from_i32(10_000))
            .then_some(Self{source_id,source_generation,amount,cast_range})
    }
    pub fn payload(self)->Value {json!({(KEY):{"source_id":self.source_id,"source_generation":self.source_generation,"amount_raw":self.amount.raw(),"range_raw":self.cast_range.raw()}})}
    pub fn from_payload(buff_id:&str,duration:Fixed64,payload:&Value)->Option<Self> {
        if buff_id!=BUFF || duration!=Fixed64::ONE || payload.as_object()?.len()!=1{return None;}
        let row=payload.get(KEY)?.as_object()?;if row.len()!=4{return None;}
        Self::checked(row.get("source_id")?.as_u64()?.try_into().ok()?,row.get("source_generation")?.as_u64()?.try_into().ok()?,
            Fixed64::from_raw(row.get("amount_raw")?.as_i64()?),Fixed64::from_raw(row.get("range_raw")?.as_i64()?))
    }
}
pub fn admitted(world:&World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value,expected_source:Option<Entity>)->Option<Spec> {
    let spec=Spec::from_payload(buff_id,duration,payload)?;
    super::targeted_effect::admitted(world,target,spec.source_id,spec.source_generation,spec.cast_range,false,expected_source).then_some(spec)
}
pub fn apply(world:&mut World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value)->bool {
    let Some(spec)=admitted(world,target,buff_id,duration,payload,None)else{return false;};
    if let Some(pool)=world.write_storage::<Hero>().get_mut(target).and_then(|h|h.mana_pool.as_mut()) {return pool.drain(spec.amount).is_ok();}
    true
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn mana_drain_packet_rejects_ambiguous_types_envelopes_and_transport_duration() {
        let valid=Spec::checked(0,1,Fixed64::ONE,Fixed64::from_i32(600)).unwrap();
        assert_eq!(Spec::from_payload(BUFF,Fixed64::ONE,&valid.payload()),Some(valid));
        for field in ["amount_raw","range_raw"]{for value in [json!(0),json!(-1),json!(0.5),json!("1024"),json!(i64::MAX)]{
            let mut bad=valid.payload();bad[KEY][field]=value;assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());}}
        for value in [0,u32::MAX]{let mut bad=valid.payload();bad[KEY]["source_generation"]=json!(value);assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());}
        let mut bad=valid.payload();bad["extra"]=json!(true);assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());
        let mut bad=valid.payload();bad[KEY]["extra"]=json!(true);assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());
        assert!(Spec::from_payload(BUFF,Fixed64::from_raw(1025),&valid.payload()).is_none());assert!(Spec::from_payload("damage_shield",Fixed64::ONE,&valid.payload()).is_none());
    }
}
