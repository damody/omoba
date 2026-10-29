//! 大地震撼（村上義清，來源 B29T current，B29.txt:394-483、B29.lua:136-177、241-280）：
//! 引導 4.5/5.2/5.9 秒，每 0.1 秒（共 45/52/59 次）在施法點 10～350/400/450
//! 距離內選一個打擊點（`RandomInt(10, B29T_Radius*0.5)`＋隨機方向）；每擊產生後
//! 0.45 秒才落下：340 內存活的敵方英雄與一般單位受 45 魔法傷害並暈眩 0.2 秒
//! （皆為腳本寫死；AbilityDamage 60、B29T_Duration 未使用），建築受 30% 傷害、不暈眩。
//!
//! 生命週期由 host G-2 pulse channel 擁有（前搖 0.3 秒後開始；中斷、計時、扣費一次）。
//! 每個 pulse（[`EarthTremorHandler::on_channel_pulse`]）選打擊點並以 omoba-core
//! `delayed_effect` 排程，0.45 秒後由 [`EarthTremorHandler::on_delayed_effect`] 結算，
//! 所以引導結束或中斷前已出手的打擊仍會落下（施法者死亡會清掉排程）。
//! 每個 pulse 也對施法者隊伍在施法點開半徑 spread＋100、0.5 秒的一般視野（B29.lua:166，
//! reveal_zone detect=false）。打擊點由 (施法者, 施法點, 擊序) 的決定性雜湊選出：
//! 不依賴 host 亂數序號，權威端與隊伍 replica 必定算出同一點。
use abi_stable::std_types::{RErr,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,
    types::{ChannelPulse,DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::{delayed_effect,reveal_zone};
use omoba_template_ids::{ABILITY_B29_EARTH_TREMOR,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use super::No1_ground_slam::enemy_units_in_circle;

const ID:omoba_template_ids::AbilityId=ABILITY_B29_EARTH_TREMOR;
pub(crate) const INTERVAL_KEY:&str="channel_interval";
pub(crate) const COUNT_KEY:&str="channel_pulses";
pub(crate) const AREA_KEY:&str="shape_radius";

#[derive(Clone,Copy,Debug)]
pub(crate) struct Tremor {
    /// Strike point distance from the cast point: [spread_min, spread].
    pub spread_min:Fixed64,pub spread:Fixed64,
    pub wave_radius:Fixed64,pub damage:Fixed64,pub stun:Fixed64,
    /// Seconds between a strike's creation and its landing.
    pub delay:Fixed64,
    /// Share of `damage` dealt to buildings (no stun).
    pub building_ratio:Fixed64,
    /// Caster-team sight at the cast point: radius spread + margin, `vision_linger` seconds per pulse.
    pub vision_margin:Fixed64,pub vision_linger:Fixed64,
    /// Both teams' plain sight around the caster (E185): radius, seconds at the first pulse;
    /// every pulse refreshes it for `vision_linger`.
    pub caster_vision_radius:Fixed64,pub caster_vision_duration:Fixed64,
}

pub(crate) fn rank_spec(rank:u8)->Result<Tremor,&'static str> {
    let content=active_ability_const(ID).ok_or("missing earth tremor content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid earth tremor rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid earth tremor rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing earth tremor extra");
    let spec=Tremor {spread_min:extra("spread_min")?,spread:extra("spread")?,
        wave_radius:extra("wave_radius")?,damage:extra("damage")?,stun:extra("stun")?,
        delay:extra("strike_delay")?,building_ratio:extra("building_ratio")?,
        vision_margin:extra("vision_margin")?,vision_linger:extra("vision_linger")?,
        caster_vision_radius:extra("caster_vision_radius")?,caster_vision_duration:extra("caster_vision_duration")?};
    if spec.spread_min<Fixed64::ZERO || spec.spread<spec.spread_min || spec.spread>Fixed64::from_i32(2_000)
        || spec.wave_radius<=Fixed64::ZERO || spec.wave_radius>Fixed64::from_i32(2_000)
        || spec.damage<Fixed64::ZERO || spec.stun<=Fixed64::ZERO || spec.stun>Fixed64::from_i32(60)
        || !delayed_effect::valid_delay(spec.delay) || spec.building_ratio<Fixed64::ZERO || spec.building_ratio>Fixed64::ONE
        || spec.vision_margin<Fixed64::ZERO || !reveal_zone::valid_duration(spec.vision_linger)
        || !reveal_zone::valid_duration(spec.caster_vision_duration) {return Err("invalid earth tremor values");}
    Ok(spec)
}

fn mix(mut z:u64)->u64 {
    z=z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z=(z^(z>>30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z=(z^(z>>27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z^(z>>31)
}

/// Strike point `index`: uniform direction and uniform distance in
/// [min, max] from `anchor` (原作 `point + RandomVector(RandomInt(10, sk_radius))`),
/// chosen by a deterministic hash.
pub(crate) fn wave_center(caster:EntityHandle,anchor:Vec2,index:u16,min:Fixed64,max:Fixed64)->Vec2 {
    let seed=[u64::from(caster.id),u64::from(caster.gen),anchor.x.raw() as u64,anchor.y.raw() as u64,u64::from(index)]
        .into_iter().fold(0x5B29_7E3E_u64,|acc,v|mix(acc^v));
    let angle=omoba_sim::Angle::from_ticks((seed%omoba_sim::trig::TAU_TICKS as u64) as i32);
    let fraction=Fixed64::from_raw(((seed>>16)%1025) as i64);
    let distance=min+(max-min)*fraction;
    let offset=Vec2::new(omoba_sim::trig::cos(angle)*distance,omoba_sim::trig::sin(angle)*distance);
    // The cosine/sine table can overshoot `max` by a rounding step; pull back inside.
    let mut center=anchor+offset;
    if !omoba_sim::terrain::point_in_swept_circle(center,anchor,anchor,max) {center=omoba_sim::navigation::step_toward(anchor,center,max);}
    center
}

/// The strike scheduled by pulse `pulse.index`.
pub(crate) fn strike_effect(caster:EntityHandle,pulse:ChannelPulse,spec:&Tremor)->Option<delayed_effect::Spec> {
    let point=wave_center(caster,pulse.anchor,pulse.index,spec.spread_min,spec.spread);
    delayed_effect::Spec {ability:ID.as_str().to_owned(),level:pulse.level,index:pulse.index,point}.checked()
}

/// One channel pulse: schedules its strike and refreshes the cast-point sight.
/// Applies nothing for an invalid rank or caster.
pub(crate) fn pulse(world:&mut GameWorldDyn<'_>,caster:EntityHandle,pulse:ChannelPulse)->Result<Vec2,&'static str> {
    let spec=rank_spec(pulse.level)?;
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let strike=strike_effect(caster,pulse,&spec).ok_or("invalid earth tremor strike")?;
    let sight=reveal_zone::Spec {source_id:caster.id,source_generation:caster.gen,center:pulse.anchor,
        radius:spec.spread+spec.vision_margin,cast_range:Fixed64::from_i32(10_000),detect:false}.checked().ok_or("invalid earth tremor sight")?;
    world.add_stat_buff(caster,strike.buff_id().as_str().into(),spec.delay,strike.payload().to_string().as_str().into());
    world.add_stat_buff(caster,reveal_zone::BUFF.into(),spec.vision_linger,sight.payload().to_string().as_str().into());
    // Both teams see the channelling caster: 6 s from the first pulse, then 0.5 s per pulse.
    let at=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let exposed=reveal_zone::sight_payload(caster.id,caster.gen,at,spec.caster_vision_radius,true,None).ok_or("invalid earth tremor caster sight")?;
    let lasting=if pulse.index==0 {spec.caster_vision_duration} else {spec.vision_linger};
    world.add_stat_buff(caster,reveal_zone::BUFF.into(),lasting,exposed.to_string().as_str().into());
    Ok(strike.point)
}

/// A strike lands (B29T_Effect, B29.lua:246-276): 340 around its point, enemy
/// units take `damage` magical and the stun; enemy buildings take
/// `damage × building_ratio`, no stun. Returns (units, buildings) hit.
pub(crate) fn land(world:&mut GameWorldDyn<'_>,caster:EntityHandle,effect:ChannelPulse)->Result<(Vec<EntityHandle>,Vec<EntityHandle>),&'static str> {
    let spec=rank_spec(effect.level)?;
    let center=effect.anchor;
    let units=enemy_units_in_circle(world,caster,center,spec.wave_radius);
    let mut buildings:Vec<EntityHandle>=world.query_enemies_in_range(center,spec.wave_radius,caster).into_iter()
        .filter(|e|world.is_building(*e) && world.is_alive(*e) && world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
            && world.get_pos(*e).into_option().is_some_and(|pos|omoba_sim::terrain::point_in_swept_circle(pos,center,center,spec.wave_radius)))
        .collect();
    buildings.sort_by_key(|e|(e.id,e.gen));buildings.dedup();
    for e in &units {
        world.deal_damage(*e,spec.damage,DamageKind::Magical,DamageProfile::CRUSHING,RSome(caster));
        // SPELL_IMMUNITY_ENEMIES_YES: the stun lands through magic immunity (E184); the
        // magical damage stays 0 on a magic-immune unit (eventfordamage.lua:30-32).
        // modifier_B29T is one of the source's ultimate_modifiers (events.lua:246-269):
        // status resistance does not shorten it (rules::status).
        world.add_stat_buff(*e,BuffId::Stun.as_rstr(),spec.stun,RStr::from_str(omoba_core::runtime::native::rules::status::PIERCING_ULTIMATE_PAYLOAD_JSON));
    }
    let building_damage=spec.damage*spec.building_ratio;
    for e in &buildings {world.deal_damage(*e,building_damage,DamageKind::Magical,DamageProfile::CRUSHING,RSome(caster));}
    Ok((units,buildings))
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
    fn on_delayed_effect(&self,caster:EntityHandle,effect:ChannelPulse,world:&mut GameWorldDyn<'_>) {
        let _=land(world,caster,effect);
    }
}

pub fn b29_earth_tremor_ffi()->AbilityDefFFI {
    let wave=extra_at_id_f32(ID,"wave_radius",1);
    build_ability_ffi(ID,EarthTremorHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,AREA_KEY,1),
        duration:extra_at_id_f32(ID,INTERVAL_KEY,1)*extra_at_id_f32(ID,COUNT_KEY,1),tick_interval:extra_at_id_f32(ID,INTERVAL_KEY,1),
        effects:vec![
            EffectSpec::Damage {target:TargetSelector::Custom(format!("enemies_near_delayed_random_strike(radius={wave})")),
                amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::Custom(format!("enemies_near_delayed_random_strike(radius={wave})")),
                modifier_type:"stun".into(),value:1.0,duration:Some(extra_at_id_f32(ID,"stun",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B29_murakami_yoshikiyo::b29_test_support::*;
    use omoba_core::Outcome;

    fn wave(level:u8,index:u16,anchor:Vec2)->ChannelPulse {
        ChannelPulse {level,index,total:45,elapsed:Fixed64::from_raw(102*i64::from(index+1)),anchor}
    }
    fn spread(rank:u8)->(Fixed64,Fixed64) {(Fixed64::from_i32(10),Fixed64::from_i32([350,400,450][usize::from(rank-1)]))}

    #[test]
    fn every_pulse_schedules_its_strike_0_45_seconds_later_and_refreshes_the_cast_point_sight() {
        for rank in 1..=3u8 {
            let mut t=TestWorld::new();let anchor=p(1000,0);let caster=t.handle(t.source);
            let enemy=t.body(anchor,2);
            let outcomes=t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(rank,7,anchor),7);
            assert!(magic_damage(&outcomes).is_empty() && buffs(&outcomes,"stun").is_empty(),"rank {rank}: nothing lands yet");
            assert!(!outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,..} if *target==enemy)));
            let rows=delayed_effects(&outcomes);
            assert_eq!(rows.len(),1);
            let (on,strike,delay)=&rows[0];
            let (min,max)=spread(rank);
            assert_eq!((*on,*delay,strike.level,strike.index),(t.source,q(0.45),rank,7),"rank {rank}");
            assert_eq!(strike.point,wave_center(caster,anchor,7,min,max));
            let sight=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}=>
                reveal_zone::Spec::from_payload(buff_id,*duration,payload).map(|s|(s,*duration)),_=>None}).expect("cast point sight");
            assert_eq!((sight.0.center,sight.0.radius,sight.0.detect,sight.1),(anchor,max+Fixed64::from_i32(100),false,q(0.5)),"rank {rank}");
        }
    }

    #[test]
    fn a_landing_strike_hits_340_with_45_and_a_0_2_stun_and_buildings_take_30_percent() {
        for rank in 1..=3u8 {
            let mut t=TestWorld::new();
            let center=p(1000,0);
            let edge=t.body(center+p(340,0),2);let out=t.body(center+p(0,341),2);
            let ally=t.body(center,1);let tower=t.building(center+p(10,0),2);let own_tower=t.building(center+p(-10,0),1);
            let outcomes=t.delayed(&EarthTremorHandler,wave(rank,3,center));
            assert_eq!(magic_damage(&outcomes),vec![(edge,Fixed64::from_i32(45)),(tower,Fixed64::from_i32(45)*q(0.3))],"rank {rank}: buildings 45 × 0.3 (Q10 0.3)");
            let stunned:Vec<_>=buffs(&outcomes,"stun").into_iter().map(|(e,_,d)|(e,d)).collect();
            assert_eq!(stunned,vec![(edge,q(0.2))],"rank {rank}: 0.2 at every rank, buildings not stunned");
            assert!(buffs(&outcomes,"stun").iter().all(|(_,payload,_)|*payload==serde_json::from_str::<serde_json::Value>(omoba_core::runtime::native::rules::status::PIERCING_ULTIMATE_PAYLOAD_JSON).unwrap()),"rank {rank}: the stun pierces magic immunity and ignores status resistance");
            for e in [out,ally,own_tower] {assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,..} if *target==e)));}
        }
    }

    #[test]
    fn strike_points_stay_10_to_half_the_radius_from_the_cast_point_and_spread_out() {
        let caster=omb_script_abi::types::EntityHandle {id:3,gen:1};
        for (rank,strikes) in [(1u8,45u16),(2,52),(3,59)] {
            let (min,max)=spread(rank);
            let anchor=p(-2000,3000);
            let centers:Vec<_>=(0..strikes).map(|i|wave_center(caster,anchor,i,min,max)).collect();
            let rounding=Fixed64::from_i32(1);
            assert!(centers.iter().all(|c|{let d=(*c-anchor).length();d>=min-rounding && d<=max}),"rank {rank}");
            let distinct:std::collections::BTreeSet<_>=centers.iter().map(|c|(c.x.raw(),c.y.raw())).collect();
            assert!(distinct.len()>strikes as usize*9/10,"strikes spread over the area");
            let far=centers.iter().filter(|c|(**c-anchor).length()>(min+max)/Fixed64::from_i32(2)).count();
            assert!(far>0 && far<strikes as usize,"both inner and outer half are used");
            assert_eq!(centers,(0..strikes).map(|i|wave_center(caster,anchor,i,min,max)).collect::<Vec<_>>(),"deterministic");
        }
    }

    #[test]
    fn a_zero_spread_strike_lands_on_the_anchor() {
        let caster=omb_script_abi::types::EntityHandle {id:9,gen:2};
        assert_eq!(wave_center(caster,p(50,60),3,Fixed64::ZERO,Fixed64::ZERO),p(50,60));
    }

    #[test]
    fn invalid_level_or_dead_caster_pulses_apply_nothing() {
        let mut t=TestWorld::new();let anchor=p(0,0);
        for dx in [0,100,200,300] {t.body(p(dx,0),2);}
        assert!(t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(0,0,anchor),0).is_empty());
        assert!(t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(4,0,anchor),0).is_empty());
        assert!(t.delayed(&EarthTremorHandler,wave(4,0,anchor)).is_empty());
        let source=t.source;t.kill(source);
        assert!(t.pulse(&EarthTremorHandler,Target::Point(anchor),wave(1,0,anchor),0).is_empty());
    }

    #[test]
    fn execute_is_never_a_cast_path() {
        let mut t=TestWorld::new();
        assert!(t.cast(&EarthTremorHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn ffi_declares_a_host_point_channel_with_the_source_channel_time() {
        let ffi=b29_earth_tremor_ffi();
        let def:omoba_core::ability_meta::AbilityDef=serde_json::from_str(ffi.def_json.as_str()).unwrap();
        use omoba_core::ability_meta::HostAbilityExecution;
        assert!(matches!(&def.host_execution,Some(HostAbilityExecution::PulseChannel {anchor:omoba_content_model::ChannelAnchor::Point,
            interval_key,pulses_key,radius_key:Some(radius)}) if interval_key==INTERVAL_KEY && pulses_key==COUNT_KEY && radius==AREA_KEY));
        assert_eq!(def.cast_type,omoba_core::ability_meta::CastType::Channeled);
        assert_eq!(def.target_type,omoba_core::ability_meta::TargetType::Point);
        // B29T_Skill_Duration 4.5/5.2/5.9 s at one strike per 0.1 s.
        for (rank,strikes,area,mana) in [(1u8,45.0,700.0,200.0),(2,52.0,800.0,225.0),(3,59.0,900.0,250.0)] {
            let level=def.get_level_data(rank).unwrap();
            assert!((level.cast_time-0.3).abs()<1e-3,"0.3 s windup before the channel");
            assert_eq!(level.range,1100.0);assert_eq!(level.cooldown,100.0);assert_eq!(level.mana_cost,mana);
            let extra=|key:&str|level.extra.get(key).and_then(|v|v.as_f64()).unwrap();
            assert_eq!(extra(COUNT_KEY),strikes);assert_eq!(extra(AREA_KEY),area);
            assert!((extra(INTERVAL_KEY)-0.1).abs()<1e-3);
        }
    }
}
