//! 爆裂箭（玉子，來源 C15E）：朝方向射出的箭沿 2000 路徑鋪下一排半寬 170 的火焰，
//! 隨箭以 1666 速度逐段鋪出，持續 12 秒；火焰內的敵人身上維持一個灼燒（每 0.25 秒刷新，
//! 離開後最多再燒 1.01 秒）。火焰是一個 host 線形獨立區域
//! （ground_area_line：center→end 的膠囊，grow／linger），同一敵人只有一個灼燒，施法者死亡後照常持續。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec;
use omoba_template_ids::{ABILITY_C15_BURST_ARROW,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C15_BURST_ARROW;

#[derive(Clone,Copy,Debug)]
pub(crate) struct BurstArrow {pub length:Fixed64,pub radius:Fixed64,pub amount:Fixed64,pub interval:Fixed64,pub duration:Fixed64,
    pub speed:Fixed64,pub linger:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<BurstArrow,&'static str> {
    let content=active_ability_const(ID).ok_or("missing burst arrow content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid burst arrow rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid burst arrow rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid burst arrow metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing burst arrow extra");
    let spec=BurstArrow {length:extra("damage_zone_length")?,radius:extra("damage_zone_radius")?,
        amount:extra("damage_zone_amount_per_tick")?,interval:extra("damage_zone_interval")?,duration:extra("damage_zone_duration")?,
        speed:extra("projectile_speed")?,linger:extra("damage_zone_linger")?};
    if spec.length<=Fixed64::ZERO || spec.duration<spec.interval || spec.duration>Fixed64::from_i32(30) {
        return Err("invalid burst arrow line or lifetime");
    }
    Ok(spec)
}

/// The fire line for a cast from `origin` toward `aim`.
pub(crate) fn zone(origin:Vec2,aim:Vec2,spec:&BurstArrow)->Option<DamageZoneSpec> {
    let dir=(aim-origin).normalized();
    if dir==Vec2::ZERO {return None;}
    let end=origin+Vec2::new(dir.x*spec.length,dir.y*spec.length);
    let zone=DamageZoneSpec {end:Some(end),grow:Some(spec.speed),linger:Some(spec.linger),independent:true,center:origin,radius:spec.radius,amount:spec.amount,
        interval:spec.interval,kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits()};
    zone.valid().then_some(zone)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:BurstArrow)->Result<(),&'static str> {
    let Target::Point(aim)=target else {return Err("burst arrow requires a direction");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let zone=zone(origin,aim,&spec).ok_or("burst arrow needs a direction")?;
    let key=format!("__ground_damage_zone:{}:{}:{}",ID.as_str(),caster.id,caster.gen);
    world.add_stat_buff(caster,key.as_str().into(),spec.duration,zone.payload().to_string().as_str().into());
    Ok(())
}

pub struct BurstArrowHandler;
impl AbilityScript for BurstArrowHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c15_burst_arrow_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,BurstArrowHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"damage_zone_radius",1),
        duration:extra_at_id_f32(ID,"damage_zone_duration",1),tick_interval:extra_at_id_f32(ID,"damage_zone_interval",1),
        effects:vec![EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_zone_amount_per_tick",1),
            damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C15_tamako::c15_test_support::*;
    use omoba_core::Outcome;

    fn line(outcomes:&[Outcome])->(DamageZoneSpec,Fixed64) {
        outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id.starts_with("__ground_damage_zone:")=>
            DamageZoneSpec::from_payload(payload).map(|z|(z,*duration)),_=>None}).expect("fire line")
    }

    #[test]
    fn every_rank_lays_a_2000_long_340_wide_fire_line_for_12_seconds() {
        for (rank,tick) in [(1u8,25),(2,30),(3,35),(4,40)] {
            let mut t=TestWorld::new();
            let (zone,duration)=line(&t.cast(&BurstArrowHandler,Target::Point(p(0,300)),rank,&rank_json(ID,rank)).unwrap());
            assert_eq!((zone.center,zone.end),(p(0,0),Some(p(0,2000))),"rank {rank}");
            assert_eq!((zone.radius,zone.amount,zone.interval),(Fixed64::from_i32(170),Fixed64::from_i32(tick),Fixed64::from_raw(256)),"rank {rank}");
            assert!(zone.independent,"the fire outlives the caster");
            assert_eq!(zone.grow,Some(Fixed64::from_i32(1666)),"laid along the flying arrow");
            assert_eq!(zone.linger,Some(Fixed64::from_raw((1.01f64*1024.0).round() as i64)),"burn lingers 1.01s after leaving");
            assert_eq!(duration,Fixed64::from_i32(12));
        }
    }

    #[test]
    fn no_direction_or_wrong_target_is_rejected() {
        let mut t=TestWorld::new();
        assert!(t.cast(&BurstArrowHandler,Target::Point(p(0,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&BurstArrowHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&BurstArrowHandler,Target::Point(p(1,0)),5,&rank_json(ID,1)).is_err());
    }
}
