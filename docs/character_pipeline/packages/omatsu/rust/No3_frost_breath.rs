//! 冰霜之息（H03，來源 A34R current）：朝方向灑出直線冰霜（速度 1250、飛行 1200、
//! 平均半寬 162.5），穿透沿途敵人，各受一次魔法傷害並附 5 秒持續傷害（G-1 on-hit DoT）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_projectile as host;
use omoba_template_ids::{ABILITY_A34_FROST_BREATH,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A34_FROST_BREATH;

pub(crate) struct Breath {pub distance:Fixed64,pub speed:Fixed64,pub radius:Fixed64,pub max_hits:u16,
    pub damage:Fixed64,pub dot_rate:Fixed64,pub dot_duration:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Breath,&'static str> {
    let content=active_ability_const(ID).ok_or("missing frost breath content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid frost breath rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid frost breath rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid frost breath metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid frost breath range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing frost breath extra");
    let hits=extra("projectile_max_hits")?;
    if hits.raw()%1024!=0 {return Err("frost breath hit budget must be whole");}
    Ok(Breath {distance:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),speed:extra("projectile_speed")?,
        radius:extra("projectile_radius")?,max_hits:u16::try_from(hits.raw()/1024).map_err(|_|"frost breath hit budget")?,
        damage:extra("damage_amount")?,dot_rate:extra("damage_over_time_amount_per_second")?,dot_duration:extra("damage_over_time_duration")?})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Breath)->Result<(),&'static str> {
    let Target::Point(aim)=target else {return Err("frost breath requires a direction");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if aim==origin {return Err("frost breath needs a direction");}
    let lifetime=Fixed64::from_raw((i128::from(spec.distance.raw())*1024/i128::from(spec.speed.raw().max(1))) as i64)+Fixed64::ONE;
    let envelope=host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),motion:host::Motion::Linear {aim},
        speed:spec.speed,radius:spec.radius,distance:spec.distance,lifetime,max_hits:spec.max_hits,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::COLD.bits(),slow:None,control:None,
        dot:Some((spec.dot_rate,spec.dot_duration)),buildings:false,forced:None,
    };
    if !envelope.valid() {return Err("invalid frost breath projectile envelope");}
    world.add_stat_buff(caster,host::BUFF.into(),lifetime,envelope.payload().to_string().as_str().into());
    Ok(())
}

pub struct FrostBreathHandler;
impl AbilityScript for FrostBreathHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a34_frost_breath_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,FrostBreathHandler,vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"projectile_speed",1),on_hit:vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A34_omatsu::a34_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_launches_a_piercing_line_with_rank_damage_and_dot() {
        for (rank,damage,dot) in [(1u8,100,50),(2,175,60),(3,250,70),(4,325,80)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&FrostBreathHandler,Target::Point(p(100,0)),rank,&rank_json(ID,rank)).unwrap();
            let spec=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>
                host::Spec::from_payload(host::BUFF,*duration,payload),_=>None}).expect("launch");
            assert!(matches!(spec.motion,host::Motion::Linear {..}));
            assert_eq!((spec.distance,spec.speed,spec.max_hits),(Fixed64::from_i32(1200),Fixed64::from_i32(1250),64),"rank {rank}");
            assert_eq!(spec.radius,Fixed64::from_raw(162*1024+512),"average half-width 162.5");
            assert_eq!(spec.amount,Fixed64::from_i32(damage),"rank {rank}");
            assert_eq!(spec.dot,Some((Fixed64::from_i32(dot),Fixed64::from_i32(5))),"rank {rank}");
        }
    }

    #[test]
    fn no_direction_or_wrong_target_is_rejected() {
        let mut t=TestWorld::new();
        assert!(t.cast(&FrostBreathHandler,Target::Point(p(0,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FrostBreathHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }
}
