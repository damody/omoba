//! Closed instant hostile resource grant. Never store this packet as a buff.
//!
//! Optional `damage` (canonical: present only when set): when the drain
//! settles, the target also takes damage of that kind equal to the mana
//! actually removed (source B08E_Action: `AMHC:Damage(caster, target,
//! burned)`). Only serial settlement knows the target's mana, so the host,
//! not the script, sizes that damage; it runs right after the drain, before
//! any later outcome of the batch.
use omoba_sim::Fixed64;
use omb_script_abi::types::{DamageKind,DamageProfile};
use serde_json::{Value,json};
use specs::{Entity,World,WorldExt};
use super::comp::{Hero,Outcome,Pos};
pub const KEY:&str="__enemy_mana_drain_v1";
pub const BUFF:&str="__enemy_mana_drain";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Spec {pub source_id:u32,pub source_generation:u32,pub amount:Fixed64,pub cast_range:Fixed64,
    /// Damage of this kind equal to the mana removed.
    pub damage:Option<DamageKind>}
fn kind_name(kind:DamageKind)->&'static str {match kind {DamageKind::Physical=>"physical",DamageKind::Magical=>"magical",DamageKind::Pure=>"pure"}}
impl Spec {
    pub fn checked(source_id:u32,source_generation:u32,amount:Fixed64,cast_range:Fixed64)->Option<Self> {
        (source_id!=u32::MAX && source_generation>0 && source_generation<=i32::MAX as u32
            && amount>Fixed64::ZERO && amount<=Fixed64::from_i32(1_000_000)
            && cast_range>Fixed64::ZERO && cast_range<=Fixed64::from_i32(10_000))
            .then_some(Self{source_id,source_generation,amount,cast_range,damage:None})
    }
    /// The same drain that also deals the removed amount as `kind` damage.
    pub fn with_damage(self,kind:DamageKind)->Self {Self{damage:Some(kind),..self}}
    pub fn payload(self)->Value {
        let mut row=json!({"source_id":self.source_id,"source_generation":self.source_generation,"amount_raw":self.amount.raw(),"range_raw":self.cast_range.raw()});
        if let Some(kind)=self.damage {row["damage"]=json!(kind_name(kind));}
        json!({(KEY):row})
    }
    pub fn from_payload(buff_id:&str,duration:Fixed64,payload:&Value)->Option<Self> {
        if buff_id!=BUFF || duration!=Fixed64::ONE || payload.as_object()?.len()!=1{return None;}
        let row=payload.get(KEY)?.as_object()?;
        let damage=match row.get("damage") {
            None=>None,
            Some(kind)=>Some(match kind.as_str()? {"physical"=>DamageKind::Physical,"magical"=>DamageKind::Magical,"pure"=>DamageKind::Pure,_=>return None}),
        };
        if row.len()!=4+usize::from(damage.is_some()){return None;}
        let spec=Self::checked(row.get("source_id")?.as_u64()?.try_into().ok()?,row.get("source_generation")?.as_u64()?.try_into().ok()?,
            Fixed64::from_raw(row.get("amount_raw")?.as_i64()?),Fixed64::from_raw(row.get("range_raw")?.as_i64()?))?;
        Some(Self{damage,..spec})
    }
    fn source(&self)->Entity {Entity::new(self.source_id,specs::world::Generation::new(self.source_generation as i32))}
}
/// Whether this AddBuff is a drain packet that also deals damage (settled by [`apply_with_damage`]).
pub fn carries_damage(payload:&Value)->bool {payload.get(KEY).and_then(|row|row.get("damage")).is_some()}
pub fn admitted(world:&World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value,expected_source:Option<Entity>)->Option<Spec> {
    let spec=Spec::from_payload(buff_id,duration,payload)?;
    super::targeted_effect::admitted(world,target,spec.source_id,spec.source_generation,spec.cast_range,false,expected_source).then_some(spec)
}
/// Drains and returns the mana actually removed (0 for a target without a pool).
fn drain(world:&mut World,target:Entity,spec:&Spec)->Fixed64 {
    world.write_storage::<Hero>().get_mut(target).and_then(|h|h.mana_pool.as_mut()).and_then(|pool|pool.drain(spec.amount).ok()).unwrap_or(Fixed64::ZERO)
}
pub fn apply(world:&mut World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value)->bool {
    let Some(spec)=admitted(world,target,buff_id,duration,payload,None)else{return false;};
    drain(world,target,&spec);
    true
}
/// Drains, then returns the follow-up damage packet (the removed amount as the
/// packet's kind), or `None` when nothing was removed or the packet is invalid.
pub fn apply_with_damage(world:&mut World,target:Entity,buff_id:&str,duration:Fixed64,payload:&Value)->Option<Outcome> {
    let spec=admitted(world,target,buff_id,duration,payload,None)?;
    let removed=drain(world,target,&spec);
    let kind=spec.damage?;
    if removed<=Fixed64::ZERO {return None;}
    let zero=Fixed64::ZERO;
    let (phys,magi,real,profile)=match kind {
        DamageKind::Physical=>(removed,zero,zero,DamageProfile::NORMAL),
        DamageKind::Magical=>(zero,removed,zero,DamageProfile::ENERGY),
        DamageKind::Pure=>(zero,zero,removed,DamageProfile::TRUE),
    };
    let pos=world.read_storage::<Pos>().get(target).map(|p|p.0).unwrap_or_default();
    Some(Outcome::RawDamage{pos,phys,magi,real,source:spec.source(),target,damage_profile:profile.bits(),origin:None})
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
    #[test]fn damage_option_is_canonical() {
        let plain=Spec::checked(0,1,Fixed64::ONE,Fixed64::from_i32(600)).unwrap();
        assert!(!carries_damage(&plain.payload()));
        let burn=plain.with_damage(DamageKind::Magical);
        assert!(carries_damage(&burn.payload()));
        assert_eq!(Spec::from_payload(BUFF,Fixed64::ONE,&burn.payload()),Some(burn));
        for value in [json!("fire"),json!(1),json!(null)] {
            let mut bad=burn.payload();bad[KEY]["damage"]=value;assert!(Spec::from_payload(BUFF,Fixed64::ONE,&bad).is_none());
        }
    }
}
