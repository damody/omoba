//! 震地斧（村上義清，來源 B29E current）：
//! - 朝指定點跳躍最多 300（瞬間位移，受地形碰撞截斷；無法位移時原地猛擊）。
//! - 落點周圍 500 內存活敵方英雄／一般單位受 200 魔法傷害、移速 −18/22/26/30%，8 秒。
//! - 破軍靈光 8 秒：施法者身上的 rank 標記 buff。hero UnitScript 的 on_tick
//!   （hero_unit.rs → [`on_owner_tick`]）每 0.5 秒對施法者周圍 800 內敵人套上
//!   1 秒的護甲 −4/7/10/13（同一個 buff id，刷新不疊加）。施放當下立即套一次。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B29_QUAKE_AXE,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use super::No1_ground_slam::enemy_units_in_circle;

const ID:omoba_template_ids::AbilityId=ABILITY_B29_QUAKE_AXE;

#[derive(Clone,Copy,Debug)]
pub(crate) struct Quake {
    pub leap:Fixed64,pub radius:Fixed64,pub damage:Fixed64,
    /// Move-speed slow as a fraction of one (raw), e.g. 18% → 184.
    pub slow:Fixed64,pub slow_duration:Fixed64,
    pub aura_duration:Fixed64,
    /// Armor change (negative raw armor points).
    pub armor:Fixed64,pub aura_radius:Fixed64,pub refresh:Fixed64,pub linger:Fixed64,
}

/// Rank values from content alone (the aura tick has no rank metadata).
pub(crate) fn rank_spec(rank:u8)->Result<Quake,&'static str> {
    let content=active_ability_const(ID).ok_or("missing quake axe content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid quake axe rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid quake axe rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing quake axe extra");
    // Percent extras (18 = 18%) become fractions of one, rounded.
    let fraction=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    let spec=Quake {leap:extra("dash_max_distance")?,radius:extra("shape_radius")?,damage:extra("damage_amount")?,
        slow:fraction(extra("slow_move_speed_pct")?),slow_duration:extra("slow_duration")?,
        aura_duration:extra("stat_buff_duration")?,armor:extra("stat_buff_armor_bonus")?,aura_radius:extra("stat_buff_radius")?,
        refresh:extra("stat_buff_refresh_interval")?,linger:extra("stat_buff_linger")?};
    let seconds=|v:Fixed64|v>Fixed64::ZERO && v<=Fixed64::from_i32(60);
    if spec.leap<=Fixed64::ZERO || spec.leap>Fixed64::from_i32(2_000) || spec.radius<=Fixed64::ZERO || spec.radius>Fixed64::from_i32(2_000)
        || spec.aura_radius<=Fixed64::ZERO || spec.aura_radius>Fixed64::from_i32(2_000) || spec.damage<Fixed64::ZERO
        || spec.slow<=Fixed64::ZERO || spec.slow>Fixed64::ONE || spec.armor>=Fixed64::ZERO
        || !seconds(spec.slow_duration) || !seconds(spec.aura_duration) || !seconds(spec.refresh) || !seconds(spec.linger) {
        return Err("invalid quake axe values");
    }
    Ok(spec)
}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Quake,&'static str> {
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid quake axe metadata")?;
    rank_spec(rank)
}

fn max_rank()->u8 {active_ability_const(ID).map(|c|c.max_level).unwrap_or(0)}
/// Aura marker on the caster; the rank is in the id because a hook cannot read payloads.
pub(crate) fn aura_marker(caster:EntityHandle,rank:u8)->String {format!("b29_breaker_aura_r{rank}:{}:{}",caster.id,caster.gen)}
pub(crate) fn think_marker(caster:EntityHandle)->String {format!("b29_breaker_aura_think:{}:{}",caster.id,caster.gen)}
/// The armor debuff a given caster's aura puts on enemies.
pub(crate) fn armor_debuff(caster:EntityHandle)->String {format!("b29_breaker_armor:{}:{}",caster.id,caster.gen)}

fn apply_aura_pulse(world:&mut GameWorldDyn<'_>,caster:EntityHandle,spec:&Quake,victims:&[EntityHandle]) {
    let payload=serde_json::json!({(StatKey::PhysicalArmorBonus.as_str()):spec.armor.raw()}).to_string();
    let id=armor_debuff(caster);
    for e in victims {world.add_stat_buff(*e,id.as_str().into(),spec.linger,payload.as_str().into());}
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,rank:u8,spec:Quake)->Result<Vec2,&'static str> {
    let Target::Point(requested)=target else {return Err("quake axe requires a point")};
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    // Leap: clipped to `leap` and by terrain. No displacement possible (rooted,
    // wall-adjacent or the point under the caster) slams in place.
    let plan=if requested==origin {None} else {
        match crate::swept_dash::plan(world,caster,requested,spec.leap,Fixed64::ONE) {
            Ok(plan)=>Some(plan),
            Err("dash is immobilized or blocked")|Err("dash requires displacement")=>None,
            Err(error)=>return Err(error),
        }
    };
    let landing=plan.as_ref().map(|plan|plan.end).unwrap_or(origin);
    let slammed=enemy_units_in_circle(world,caster,landing,spec.radius);
    let aura=enemy_units_in_circle(world,caster,landing,spec.aura_radius);
    // All checks done; apply outcomes.
    if let Some(plan)=&plan {crate::swept_dash::commit(world,caster,plan);}
    let slow_id=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),caster);
    let slow=serde_json::json!({"move_speed_bonus":-spec.slow.raw(),"__aggregation_family":"generic_ability_slow"}).to_string();
    for e in &slammed {
        world.deal_damage(*e,spec.damage,DamageKind::Magical,DamageProfile::CRUSHING,RSome(caster));
        world.add_stat_buff(*e,slow_id.as_str().into(),spec.slow_duration,slow.as_str().into());
    }
    // A recast replaces the aura; a different rank's marker is cleared.
    for other in 1..=max_rank() {if other!=rank {world.remove_buff(caster,aura_marker(caster,other).as_str().into());}}
    world.add_buff(caster,aura_marker(caster,rank).as_str().into(),spec.aura_duration);
    world.add_buff(caster,think_marker(caster).as_str().into(),spec.refresh);
    apply_aura_pulse(world,caster,&spec,&aura);
    Ok(landing)
}

/// Hero UnitScript tick: refreshes the 破軍靈光 debuff every `refresh` seconds
/// while the caster lives and carries an aura marker.
pub(crate) fn on_owner_tick(world:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return;}
    let Some(rank)=(1..=max_rank()).rev().find(|rank|world.has_buff(caster,aura_marker(caster,*rank).as_str().into())) else {return};
    if world.has_buff(caster,think_marker(caster).as_str().into()) {return;}
    let Ok(spec)=rank_spec(rank) else {return};
    let Some(center)=world.get_pos(caster).into_option() else {return};
    let victims=enemy_units_in_circle(world,caster,center,spec.aura_radius);
    world.add_buff(caster,think_marker(caster).as_str().into(),spec.refresh);
    apply_aura_pulse(world,caster,&spec,&victims);
}

pub struct QuakeAxeHandler;
impl AbilityScript for QuakeAxeHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,level,spec)) {Ok(_)=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b29_quake_axe_ffi()->AbilityDefFFI {
    let mut armor=HashMap::new();
    armor.insert(StatKey::PhysicalArmorBonus.as_str().into(),extra_at_id_f32(ID,"stat_buff_armor_bonus",1));
    build_ability_ffi(ID,QuakeAxeHandler,vec![
        EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"shape_radius",1),duration:0.2,tick_interval:0.0,effects:vec![
            EffectSpec::Damage {target:TargetSelector::Custom(format!("enemies_near_leap_landing(max_leap={})",extra_at_id_f32(ID,"dash_max_distance",1))),
                amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::AllEnemies,modifier_type:"slow".into(),
                value:extra_at_id_f32(ID,"slow_move_speed_pct",1)/100.0,duration:Some(extra_at_id_f32(ID,"slow_duration",1))}]},
        EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"stat_buff_radius",1),duration:extra_at_id_f32(ID,"stat_buff_duration",1),
            tick_interval:extra_at_id_f32(ID,"stat_buff_refresh_interval",1),
            effects:vec![EffectSpec::Buff {target:TargetSelector::AllEnemies,duration:Some(extra_at_id_f32(ID,"stat_buff_linger",1)),modifiers:armor}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B29_murakami_yoshikiyo::{b29_test_support::*,hero_unit::hero_unit_script};
    use omoba_core::{BlockedRegions,BlockedRegion};
    use specs::WorldExt;

    const SLOWS:[(u8,i64,i64);4]=[(1,184,-4),(2,225,-7),(3,266,-10),(4,307,-13)];

    #[test]
    fn every_rank_leaps_at_most_300_slams_slows_and_starts_the_aura() {
        for (rank,slow,armor) in SLOWS {
            let mut t=TestWorld::new();
            // Landing at (300,0): 500 reaches x=800 and x=-200; aura 800 reaches x=1100.
            let edge=t.body(p(800,0),2);let past=t.body(p(801,0),2);let aura_only=t.body(p(1100,0),2);let beyond=t.body(p(1101,0),2);
            let behind=t.body(p(-200,0),2);let ally=t.body(p(300,10),1);let tower=t.building(p(400,0),2);
            let outcomes=t.cast(&QuakeAxeHandler,Target::Point(p(2000,0)),rank,&rank_json(ID,rank)).unwrap();
            assert_eq!(final_position(&outcomes),Some(p(300,0)),"rank {rank}: leap capped at 300");
            let mut slammed=vec![(behind,Fixed64::from_i32(200)),(edge,Fixed64::from_i32(200))];slammed.sort_by_key(|(e,_)|e.id());
            assert_eq!(magic_damage(&outcomes),slammed,"rank {rank}");
            let slow_id=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),t.handle(t.source));
            for (e,payload,duration) in buffs(&outcomes,&slow_id) {
                assert!(e==edge||e==behind);assert_eq!(duration,Fixed64::from_i32(8));
                assert_eq!(payload["move_speed_bonus"].as_i64(),Some(-slow),"rank {rank}");
            }
            let marker=aura_marker(t.handle(t.source),rank);
            assert_eq!(buffs(&outcomes,&marker).len(),1);
            assert!(outcomes.iter().any(|o|matches!(o,omoba_core::Outcome::AddBuff {buff_id,duration,..} if *buff_id==marker && *duration==Fixed64::from_i32(8))));
            let debuffed:Vec<_>=buffs(&outcomes,&armor_debuff(t.handle(t.source))).into_iter().map(|(e,payload,duration)|{
                assert_eq!(duration,Fixed64::ONE);assert_eq!(payload[StatKey::PhysicalArmorBonus.as_str()].as_i64(),Some(armor*1024));e}).collect();
            let mut expected=vec![edge,past,aura_only,behind];expected.sort_by_key(|e|e.id());
            assert_eq!(debuffed,expected,"rank {rank}: 800 endpoint in, 801 out, no ally or building");
            for e in [beyond,ally,tower] {assert!(!debuffed.contains(&e));}
        }
    }

    #[test]
    fn a_short_point_lands_on_it_and_terrain_clips_the_leap() {
        let mut t=TestWorld::new();
        let outcomes=t.cast(&QuakeAxeHandler,Target::Point(p(120,0)),1,&rank_json(ID,1)).unwrap();
        assert_eq!(final_position(&outcomes),Some(p(120,0)));
        let mut t=TestWorld::new();
        t.world.insert(BlockedRegions(vec![BlockedRegion {name:"wall".into(),points:vec![
            vek::Vec2::new(150.0,-200.0),vek::Vec2::new(151.0,-200.0),vek::Vec2::new(151.0,200.0),vek::Vec2::new(150.0,200.0)]}]));
        let end=final_position(&t.cast(&QuakeAxeHandler,Target::Point(p(900,0)),1,&rank_json(ID,1)).unwrap()).unwrap();
        assert!(end.x>Fixed64::from_i32(120) && end.x<Fixed64::from_i32(150),"clipped before the wall: {end:?}");
    }

    #[test]
    fn rooted_or_self_point_slams_in_place() {
        for case in ["root","self"] {
            let mut t=TestWorld::new();let enemy=t.body(p(400,0),2);
            if case=="root" {t.buff_source("root",10);}
            let target=if case=="self" {p(0,0)} else {p(300,0)};
            let outcomes=t.cast(&QuakeAxeHandler,Target::Point(target),2,&rank_json(ID,2)).unwrap();
            assert_eq!(final_position(&outcomes),None,"{case}: no displacement");
            assert_eq!(magic_damage(&outcomes),vec![(enemy,Fixed64::from_i32(200))],"{case}: 400 is inside 500 of the origin");
        }
    }

    #[test]
    fn the_hero_tick_refreshes_the_aura_every_half_second_around_the_moving_caster() {
        let mut t=TestWorld::new();let unit=hero_unit_script();
        let enemy=t.body(p(700,0),2);
        let outcomes=t.cast(&QuakeAxeHandler,Target::Point(p(1,0)),3,&rank_json(ID,3)).unwrap();
        t.apply(outcomes);
        let caster=t.handle(t.source);let debuff=armor_debuff(caster);
        // The cast placed the think marker: no refresh until it expires.
        assert!(buffs(&t.tick(&unit),&debuff).is_empty());
        t.world.write_resource::<omoba_core::runtime::BuffStore>().remove(t.source,&think_marker(caster));
        // Caster walks away: the aura follows it (enemy now 1201 away, then 800).
        let source=t.source;t.move_to(source,p(-501,0));
        let rows=t.tick(&unit);
        assert!(buffs(&rows,&debuff).is_empty(),"1201 is outside 800");
        assert!(rows.iter().any(|o|matches!(o,omoba_core::Outcome::AddBuff {buff_id,..} if *buff_id==think_marker(caster))),"think restarts even with no enemy");
        t.apply(rows);
        t.world.write_resource::<omoba_core::runtime::BuffStore>().remove(t.source,&think_marker(caster));
        t.move_to(source,p(-100,0));
        let rows=t.tick(&unit);
        let applied=buffs(&rows,&debuff);
        assert_eq!(applied.len(),1);assert_eq!(applied[0].0,enemy);assert_eq!(applied[0].2,Fixed64::ONE);
        assert_eq!(applied[0].1[StatKey::PhysicalArmorBonus.as_str()].as_i64(),Some(-10*1024),"rank 3 armor");
        assert!(rows.iter().any(|o|matches!(o,omoba_core::Outcome::AddBuff {buff_id,duration,..} if *buff_id==think_marker(caster) && *duration==q(0.5))));
    }

    #[test]
    fn no_marker_or_a_dead_caster_ticks_nothing() {
        let mut t=TestWorld::new();let unit=hero_unit_script();t.body(p(100,0),2);
        assert!(t.tick(&unit).is_empty(),"never cast");
        let outcomes=t.cast(&QuakeAxeHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        let caster=t.handle(t.source);
        t.world.write_resource::<omoba_core::runtime::BuffStore>().remove(t.source,&think_marker(caster));
        let source=t.source;t.kill(source);
        assert!(t.tick(&unit).is_empty(),"dead caster");
    }

    #[test]
    fn a_recast_at_a_new_rank_clears_the_old_marker() {
        let mut t=TestWorld::new();
        let outcomes=t.cast(&QuakeAxeHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).unwrap();t.apply(outcomes);
        let outcomes=t.cast(&QuakeAxeHandler,Target::Point(p(2,0)),2,&rank_json(ID,2)).unwrap();
        let caster=t.handle(t.source);
        assert!(outcomes.iter().any(|o|matches!(o,omoba_core::Outcome::ScriptRemoveBuff {buff_id,..} if *buff_id==aura_marker(caster,1))));
    }

    #[test]
    fn wrong_targets_bad_rank_and_dead_caster_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&QuakeAxeHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&QuakeAxeHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&QuakeAxeHandler,Target::Point(p(100,0)),0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&QuakeAxeHandler,Target::Point(p(100,0)),5,&rank_json(ID,1)).is_err());
        assert!(t.cast(&QuakeAxeHandler,Target::Point(p(100,0)),1,"{").is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&QuakeAxeHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }
}
