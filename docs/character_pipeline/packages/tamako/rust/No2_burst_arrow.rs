//! 爆裂箭（玉子，來源 C15E current，C15.txt:124-272、C15.lua:new_C15E:1-48）：
//! 朝方向在施法者前方 fire_offset 起、長 fire_length 逐段放火焰（間距 vision_spacing），每段 fire_duration 秒；
//! 火焰 fire_radius 內的敵人身上維持一個灼燒（每 tick_interval 秒 tick_damage 魔法傷害，離開後最多再燒
//! burn_linger 秒）。火焰是一個 host 線形獨立區域（ground_area_line：center→end 的膠囊，
//! grow = lay_speed，linger），同一敵人只有一個灼燒，施法者死亡後照常持續。
//! 每段也對施法者隊伍開 vision_radius、vision_duration 秒的一般視野（AddFOWViewer，C15.lua:34；reveal_zone
//! detect=false，不揭露隱形），第 k 段按原作的放置間隔延後出現，持續時間同步延長。數值見 v3.lua。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec;
use omoba_core::runtime::native::reveal_zone;
use omoba_template_ids::{ABILITY_C15_BURST_ARROW,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C15_BURST_ARROW;

#[derive(Clone,Copy,Debug)]
pub(crate) struct BurstArrow {
    /// First segment's distance ahead of the caster, and the laid length after it.
    pub offset:Fixed64,pub length:Fixed64,
    pub radius:Fixed64,pub amount:Fixed64,pub interval:Fixed64,pub duration:Fixed64,
    /// Laying speed (`lay_speed`: segment spacing over the source timer interval).
    pub grow:Fixed64,pub linger:Fixed64,
    /// Caster-team sight per segment: radius, seconds, and the segment spacing.
    pub vision_radius:Fixed64,pub vision_duration:Fixed64,pub spacing:Fixed64,
}

pub(crate) fn resolve(rank:u8,json:&str)->Result<BurstArrow,&'static str> {
    let content=active_ability_const(ID).ok_or("missing burst arrow content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid burst arrow rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid burst arrow rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid burst arrow metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing burst arrow extra");
    let spec=BurstArrow {offset:extra("fire_offset")?,length:extra("fire_length")?,radius:extra("fire_radius")?,
        amount:extra("tick_damage")?,interval:extra("tick_interval")?,duration:extra("fire_duration")?,
        grow:extra("lay_speed")?,linger:extra("burn_linger")?,
        vision_radius:extra("vision_radius")?,vision_duration:extra("vision_duration")?,spacing:extra("vision_spacing")?};
    if spec.length<=Fixed64::ZERO || spec.offset<Fixed64::ZERO || spec.duration<spec.interval || spec.duration>Fixed64::from_i32(30) {
        return Err("invalid burst arrow line or lifetime");
    }
    if spec.spacing<Fixed64::from_i32(10) || spec.length/spec.spacing>Fixed64::from_i32(48) || spec.grow<Fixed64::ONE
        || !reveal_zone::valid_duration(spec.vision_duration) {return Err("invalid burst arrow vision");}
    Ok(spec)
}

fn direction(origin:Vec2,aim:Vec2)->Option<Vec2> {
    let dir=(aim-origin).normalized();
    (dir!=Vec2::ZERO).then_some(dir)
}
fn ahead(origin:Vec2,dir:Vec2,distance:Fixed64)->Vec2 {origin+Vec2::new(dir.x*distance,dir.y*distance)}

/// The fire line for a cast from `origin` toward `aim`.
pub(crate) fn zone(origin:Vec2,aim:Vec2,spec:&BurstArrow)->Option<DamageZoneSpec> {
    let dir=direction(origin,aim)?;
    let center=ahead(origin,dir,spec.offset);
    let end=ahead(origin,dir,spec.offset+spec.length);
    let zone=DamageZoneSpec {end:Some(end),grow:Some(spec.grow),linger:Some(spec.linger),independent:true,center,radius:spec.radius,amount:spec.amount,
        interval:spec.interval,kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits()};
    zone.valid().then_some(zone)
}

/// One plain sight circle per segment: (spec, seconds).
pub(crate) fn sights(caster:EntityHandle,origin:Vec2,aim:Vec2,spec:&BurstArrow)->Option<Vec<(reveal_zone::Spec,Fixed64)>> {
    let dir=direction(origin,aim)?;
    let segments=(spec.length/spec.spacing).raw()/1024+1;
    let step=spec.spacing/spec.grow;
    let reach=spec.offset+spec.length+Fixed64::ONE;
    (0..segments).map(|k|{
        let k=Fixed64::from_i32(k as i32);
        let sight=reveal_zone::Spec {source_id:caster.id,source_generation:caster.gen,center:ahead(origin,dir,spec.offset+spec.spacing*k),
            radius:spec.vision_radius,cast_range:reach,detect:false}.checked()?;
        let seconds=spec.vision_duration+step*(k+Fixed64::ONE);
        reveal_zone::valid_duration(seconds).then_some((sight,seconds))
    }).collect()
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:BurstArrow)->Result<(),&'static str> {
    let Target::Point(aim)=target else {return Err("burst arrow requires a direction");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let zone=zone(origin,aim,&spec).ok_or("burst arrow needs a direction")?;
    let sights=sights(caster,origin,aim,&spec).ok_or("invalid burst arrow vision")?;
    let key=format!("__ground_damage_zone:{}:{}:{}",ID.as_str(),caster.id,caster.gen);
    world.add_stat_buff(caster,key.as_str().into(),spec.duration,zone.payload().to_string().as_str().into());
    for (sight,seconds) in sights {
        world.add_stat_buff(caster,reveal_zone::BUFF.into(),seconds,sight.payload().to_string().as_str().into());
    }
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
    build_ability_ffi(ID,BurstArrowHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"fire_radius",1),
        duration:extra_at_id_f32(ID,"fire_duration",1),tick_interval:extra_at_id_f32(ID,"tick_interval",1),
        effects:vec![EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"tick_damage",1),
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
    fn q(v:f64)->Fixed64 {Fixed64::from_raw((v*1024.0).round() as i64)}

    #[test]
    fn every_rank_lays_fire_from_200_to_2600_ahead_for_15_seconds() {
        for (rank,tick) in [(1u8,25),(2,30),(3,35),(4,40)] {
            let mut t=TestWorld::new();
            let (zone,duration)=line(&t.cast(&BurstArrowHandler,Target::Point(p(0,300)),rank,&rank_json(ID,rank)).unwrap());
            assert_eq!((zone.center,zone.end),(p(0,200),Some(p(0,2600))),"rank {rank}: segment centres 200..2600");
            assert_eq!((zone.radius,zone.amount,zone.interval),(Fixed64::from_i32(170),Fixed64::from_i32(tick),Fixed64::from_raw(256)),"rank {rank}");
            assert!(zone.independent,"the fire outlives the caster");
            let grow=zone.grow.unwrap();
            assert!((grow-Fixed64::from_raw(3_413_333)).raw().abs()<=1,"100 every 0.03 s ≈ 3333/s: {grow:?}");
            assert_eq!(zone.linger,Some(q(1.01)),"burn lingers 1.01s after leaving");
            assert_eq!(duration,Fixed64::from_i32(15),"modifier_C15E_2 Duration 15 (duration 12 is unused)");
        }
    }

    #[test]
    fn every_segment_gives_the_caster_team_plain_sight_for_12_seconds() {
        let mut t=TestWorld::new();
        let outcomes=t.cast(&BurstArrowHandler,Target::Point(p(300,0)),2,&rank_json(ID,2)).unwrap();
        let sights:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {target,buff_id,duration,payload}=>
            reveal_zone::Spec::from_payload(buff_id,*duration,payload).map(|s|(*target,s,*duration)),_=>None}).collect();
        assert_eq!(sights.len(),25,"200, 300 … 2600");
        for (k,(on,sight,seconds)) in sights.iter().enumerate() {
            assert_eq!(*on,t.source);
            assert_eq!((sight.center,sight.radius,sight.detect),(p(200+100*k as i32,0),Fixed64::from_i32(200),false),"segment {k}");
            let expected=Fixed64::from_i32(12)+q(0.03)*Fixed64::from_i32(k as i32+1);
            assert!((*seconds-expected).raw().abs()<=k as i64+2,"segment {k}: 12 s after it appears ({seconds:?})");
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
