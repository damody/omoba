//! 大地震撼（村上義清，來源 B29T current）：引導；在指定地點半徑 700/800/900
//! 的範圍內每 0.1 秒打出一個震波，共 50/60/70 波；每個震波使其中心 340 內存活
//! 的敵方英雄與一般單位（不含建築）受到 60 魔法傷害並暈眩 0.2/0.3/0.4 秒。
//!
//! 生命週期由 host G-2 pulse channel 擁有（前搖 0.3 秒後開始；中斷、計時、扣費一次）。
//! package 的 `timing.channel` 生成 content `channel`，`build_ability_ffi` 會把它
//! 帶成 host execution；本 handler 只實作每波的 `on_channel_pulse`。
//! 每波的中心由 (施法者, 施法點, 波序) 的決定性雜湊選出：不依賴 host 亂數序號，
//! 權威端與隊伍 replica 必定算出同一點。
use abi_stable::std_types::{RErr,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,
    types::{ChannelPulse,DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B29_EARTH_TREMOR,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use super::No1_ground_slam::enemy_units_in_circle;

const ID:omoba_template_ids::AbilityId=ABILITY_B29_EARTH_TREMOR;
pub(crate) const INTERVAL_KEY:&str="channel_interval";
pub(crate) const COUNT_KEY:&str="channel_pulses";
pub(crate) const AREA_KEY:&str="shape_radius";

#[derive(Clone,Copy,Debug)]
pub(crate) struct Tremor {pub area:Fixed64,pub wave_radius:Fixed64,pub damage:Fixed64,pub stun:Fixed64}

pub(crate) fn rank_spec(rank:u8)->Result<Tremor,&'static str> {
    let content=active_ability_const(ID).ok_or("missing earth tremor content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid earth tremor rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid earth tremor rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing earth tremor extra");
    let spec=Tremor {area:extra(AREA_KEY)?,wave_radius:extra("damage_wave_radius")?,damage:extra("damage_amount")?,stun:extra("control_duration")?};
    if spec.area<Fixed64::ZERO || spec.area>Fixed64::from_i32(2_000) || spec.wave_radius<=Fixed64::ZERO || spec.wave_radius>Fixed64::from_i32(2_000)
        || spec.damage<Fixed64::ZERO || spec.stun<=Fixed64::ZERO || spec.stun>Fixed64::from_i32(60) {return Err("invalid earth tremor values");}
    Ok(spec)
}

fn mix(mut z:u64)->u64 {
    z=z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z=(z^(z>>30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z=(z^(z>>27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z^(z>>31)
}

/// Center of wave `index`: uniform direction and uniform distance in
/// [0, area] from `anchor`, chosen by a deterministic hash.
pub(crate) fn wave_center(caster:EntityHandle,anchor:Vec2,index:u16,area:Fixed64)->Vec2 {
    let seed=[u64::from(caster.id),u64::from(caster.gen),anchor.x.raw() as u64,anchor.y.raw() as u64,u64::from(index)]
        .into_iter().fold(0x5B29_7E3E_u64,|acc,v|mix(acc^v));
    let angle=omoba_sim::Angle::from_ticks((seed%omoba_sim::trig::TAU_TICKS as u64) as i32);
    let fraction=Fixed64::from_raw(((seed>>16)%1025) as i64);
    let distance=area*fraction;
    let offset=Vec2::new(omoba_sim::trig::cos(angle)*distance,omoba_sim::trig::sin(angle)*distance);
    // The cosine/sine table can overshoot `area` by a rounding step; pull back inside.
    let mut center=anchor+offset;
    if !omoba_sim::terrain::point_in_swept_circle(center,anchor,anchor,area) {center=omoba_sim::navigation::step_toward(anchor,center,area);}
    center
}

/// One wave. Returns the hit units; applies nothing for an invalid rank or caster.
pub(crate) fn pulse(world:&mut GameWorldDyn<'_>,caster:EntityHandle,pulse:ChannelPulse)->Result<Vec<EntityHandle>,&'static str> {
    let spec=rank_spec(pulse.level)?;
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let center=wave_center(caster,pulse.anchor,pulse.index,spec.area);
    let hits=enemy_units_in_circle(world,caster,center,spec.wave_radius);
    for e in &hits {
        world.deal_damage(*e,spec.damage,DamageKind::Magical,DamageProfile::CRUSHING,RSome(caster));
        world.add_buff(*e,BuffId::Stun.as_rstr(),spec.stun);
    }
    Ok(hits)
}

pub struct EarthTremorHandler;
impl AbilityScript for EarthTremorHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    /// A valid pulse channel never reaches `execute` (the host stages the
    /// channel instead); reaching it means the channel metadata was rejected.
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("earth tremor runs only as a host pulse channel".into())
    }
    fn on_channel_pulse(&self,caster:EntityHandle,_:Target,p:ChannelPulse,world:&mut GameWorldDyn<'_>) {
        let _=pulse(world,caster,p);
    }
}

pub fn b29_earth_tremor_ffi()->AbilityDefFFI {
    let wave=extra_at_id_f32(ID,"damage_wave_radius",1);
    build_ability_ffi(ID,EarthTremorHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,AREA_KEY,1),
        duration:extra_at_id_f32(ID,INTERVAL_KEY,1)*extra_at_id_f32(ID,COUNT_KEY,1),tick_interval:extra_at_id_f32(ID,INTERVAL_KEY,1),
        effects:vec![
            EffectSpec::Damage {target:TargetSelector::Custom(format!("enemies_near_random_wave(radius={wave})")),
                amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::Custom(format!("enemies_near_random_wave(radius={wave})")),
                modifier_type:"stun".into(),value:1.0,duration:Some(extra_at_id_f32(ID,"control_duration",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B29_murakami_yoshikiyo::b29_test_support::*;
    use omoba_core::Outcome;

    fn wave(level:u8,index:u16,anchor:Vec2)->ChannelPulse {
        ChannelPulse {level,index,total:50,elapsed:Fixed64::from_raw(102*i64::from(index+1)),anchor}
    }

    #[test]
    fn every_rank_wave_hits_340_around_its_center_with_60_and_the_rank_stun() {
        for (rank,stun,area) in [(1u8,0.2,700),(2,0.3,800),(3,0.4,900)] {
            let mut t=TestWorld::new();
            let anchor=p(1000,0);let caster=t.handle(t.source);
            let center=wave_center(caster,anchor,7,Fixed64::from_i32(area));
            let edge=t.body(center+p(340,0),2);let out=t.body(center+p(0,341),2);
            let ally=t.body(center,1);let tower=t.building(center+p(10,0),2);
            let outcomes=t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(rank,7,anchor),7);
            assert_eq!(magic_damage(&outcomes),vec![(edge,Fixed64::from_i32(60))],"rank {rank}");
            let stunned:Vec<_>=buffs(&outcomes,"stun").into_iter().map(|(e,_,d)|(e,d)).collect();
            assert_eq!(stunned,vec![(edge,q(stun))],"rank {rank}");
            for e in [out,ally,tower] {assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,..} if *target==e)));}
        }
    }

    #[test]
    fn wave_centers_stay_inside_the_rank_area_and_spread_out() {
        let caster=omb_script_abi::types::EntityHandle {id:3,gen:1};
        for (area,waves) in [(700,50u16),(800,60),(900,70)] {
            let anchor=p(-2000,3000);let area=Fixed64::from_i32(area);
            let centers:Vec<_>=(0..waves).map(|i|wave_center(caster,anchor,i,area)).collect();
            assert!(centers.iter().all(|c|omoba_sim::terrain::point_in_swept_circle(*c,anchor,anchor,area)));
            let distinct:std::collections::BTreeSet<_>=centers.iter().map(|c|(c.x.raw(),c.y.raw())).collect();
            assert!(distinct.len()>waves as usize*9/10,"waves spread over the area");
            let far=centers.iter().filter(|c|(**c-anchor).length()>area/Fixed64::from_i32(2)).count();
            assert!(far>0 && far<waves as usize,"both inner and outer half are used");
            assert_eq!(centers,(0..waves).map(|i|wave_center(caster,anchor,i,area)).collect::<Vec<_>>(),"deterministic");
        }
    }

    #[test]
    fn a_zero_area_wave_lands_on_the_anchor() {
        let caster=omb_script_abi::types::EntityHandle {id:9,gen:2};
        assert_eq!(wave_center(caster,p(50,60),3,Fixed64::ZERO),p(50,60));
    }

    #[test]
    fn invalid_level_or_dead_caster_pulses_apply_nothing() {
        let mut t=TestWorld::new();let anchor=p(0,0);
        for dx in [0,100,200,300] {t.body(p(dx,0),2);}
        assert!(t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(0,0,anchor),0).is_empty());
        assert!(t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(4,0,anchor),0).is_empty());
        let source=t.source;t.kill(source);
        assert!(t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(1,0,anchor),0).is_empty());
    }

    #[test]
    fn execute_is_never_a_cast_path() {
        let mut t=TestWorld::new();
        assert!(t.cast(&EarthTremorHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn ffi_declares_a_host_point_channel_with_the_authored_ranks() {
        let ffi=b29_earth_tremor_ffi();
        let def:omoba_core::ability_meta::AbilityDef=serde_json::from_str(ffi.def_json.as_str()).unwrap();
        use omoba_core::ability_meta::HostAbilityExecution;
        assert!(matches!(&def.host_execution,Some(HostAbilityExecution::PulseChannel {anchor:omoba_content_model::ChannelAnchor::Point,
            interval_key,pulses_key,radius_key:Some(radius)}) if interval_key==INTERVAL_KEY && pulses_key==COUNT_KEY && radius==AREA_KEY));
        assert_eq!(def.cast_type,omoba_core::ability_meta::CastType::Channeled);
        assert_eq!(def.target_type,omoba_core::ability_meta::TargetType::Point);
        for (rank,waves,area,mana) in [(1u8,50.0,700.0,200.0),(2,60.0,800.0,225.0),(3,70.0,900.0,250.0)] {
            let level=def.get_level_data(rank).unwrap();
            assert!((level.cast_time-0.3).abs()<1e-3,"0.3 s windup before the channel");
            assert_eq!(level.range,1100.0);assert_eq!(level.cooldown,100.0);assert_eq!(level.mana_cost,mana);
            let extra=|key:&str|level.extra.get(key).and_then(|v|v.as_f64()).unwrap();
            assert_eq!(extra(COUNT_KEY),waves);assert_eq!(extra(AREA_KEY),area);
            assert!((extra(INTERVAL_KEY)-0.1).abs()<1e-3);
        }
    }
}
