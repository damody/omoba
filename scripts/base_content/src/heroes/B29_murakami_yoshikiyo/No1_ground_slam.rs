//! 動地斧（村上義清，來源 B29W current）：猛擊地面，對自身周圍 radius（B29.lua:15
//! 寫死；KV AOERadius 只是指示圈）內存活的敵方英雄與一般單位（不含建築）造成
//! damage 魔法傷害並暈眩 stun 秒；數值見 v3.lua 的 radius、damage、stun。
//! 前搖由 host windup 處理；本 handler 在結算時執行。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B29_GROUND_SLAM,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B29_GROUND_SLAM;

pub(crate) struct Slam {pub radius:Fixed64,pub damage:Fixed64,pub stun:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Slam,&'static str> {
    let content=active_ability_const(ID).ok_or("missing ground slam content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid ground slam rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid ground slam rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid ground slam metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing ground slam extra");
    let spec=Slam {radius:extra("radius")?,damage:extra("damage")?,stun:extra("stun")?};
    if spec.radius<=Fixed64::ZERO || spec.radius>Fixed64::from_i32(2_000) {return Err("invalid ground slam radius");}
    if spec.damage<Fixed64::ZERO || spec.stun<=Fixed64::ZERO || spec.stun>Fixed64::from_i32(60) {return Err("invalid ground slam values");}
    Ok(spec)
}

/// Live enemy heroes and basic units (no buildings) within `radius` of `at`,
/// endpoint included, in canonical identity order.
pub(crate) fn enemy_units_in_circle(world:&GameWorldDyn<'_>,caster:EntityHandle,at:Vec2,radius:Fixed64)->Vec<EntityHandle> {
    let mut hits:Vec<EntityHandle>=world.query_enemies_in_range(at,radius,caster).into_iter()
        .filter(|e|*e!=caster && world.is_alive(*e) && world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
            && !world.is_building(*e)
            && world.get_pos(*e).into_option().is_some_and(|pos|omoba_sim::terrain::point_in_swept_circle(pos,at,at,radius)))
        .collect();
    hits.sort_by_key(|e|(e.id,e.gen));hits.dedup();hits
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Slam)->Result<usize,&'static str> {
    if !matches!(target,Target::None) {return Err("ground slam is self centered");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let hits=enemy_units_in_circle(world,caster,origin,spec.radius);
    // All checks done; apply outcomes.
    for e in &hits {
        world.deal_damage(*e,spec.damage,DamageKind::Magical,DamageProfile::CRUSHING.union(DamageProfile::SPELL),RSome(caster));
        world.add_buff(*e,BuffId::Stun.as_rstr(),spec.stun);
    }
    Ok(hits.len())
}

pub struct GroundSlamHandler;
impl AbilityScript for GroundSlamHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(_)=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b29_ground_slam_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,GroundSlamHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![
            EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::AllEnemies,modifier_type:"stun".into(),value:1.0,
                duration:Some(extra_at_id_f32(ID,"stun",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B29_murakami_yoshikiyo::b29_test_support::*;

    #[test]
    fn every_rank_damages_and_stuns_each_enemy_within_550_once() {
        for (rank,damage,stun) in [(1u8,150,1.0),(2,250,1.3),(3,350,1.6),(4,450,1.9)] {
            let mut t=TestWorld::new();
            let near=t.body(p(100,0),2);let edge=t.body(p(0,550),2);let out=t.body(p(0,-551),2);
            let ally=t.body(p(50,0),1);let tower=t.building(p(-100,0),2);
            let outcomes=t.cast(&GroundSlamHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            assert_eq!(magic_damage(&outcomes),vec![(near,Fixed64::from_i32(damage)),(edge,Fixed64::from_i32(damage))],"rank {rank}");
            let stunned:Vec<_>=buffs(&outcomes,"stun").into_iter().map(|(e,_,d)|(e,d)).collect();
            assert_eq!(stunned,vec![(near,q(stun)),(edge,q(stun))],"rank {rank}");
            for e in [out,ally,tower] {assert!(!outcomes.iter().any(|o|matches!(o,omoba_core::Outcome::RawDamage {target,..} if *target==e)));}
        }
    }

    #[test]
    fn dead_enemies_are_skipped_and_hits_follow_identity_order() {
        let mut t=TestWorld::new();
        let first=t.body(p(300,0),2);let dead=t.body(p(10,0),2);let second=t.body(p(-300,0),2);t.kill(dead);
        let outcomes=t.cast(&GroundSlamHandler,Target::None,1,&rank_json(ID,1)).unwrap();
        let order:Vec<_>=buffs(&outcomes,"stun").into_iter().map(|(e,_,_)|e).collect();
        let mut expected=vec![first,second];expected.sort_by_key(|e|e.id());
        assert_eq!(order,expected);
    }

    #[test]
    fn wrong_target_shape_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&GroundSlamHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&GroundSlamHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&GroundSlamHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&GroundSlamHandler,Target::None,5,&rank_json(ID,1)).is_err());
        assert!(t.cast(&GroundSlamHandler,Target::None,1,"{").is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&GroundSlamHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_source_cast_point_cooldown_and_mana() {
        let content=active_ability_const(ID).unwrap();
        for (rank,mana) in [(1usize,100),(2,120),(3,140),(4,160)] {
            let level=&content.levels[rank-1];
            assert_eq!(level.cast_time,q(0.3));assert_eq!(level.cooldown,Fixed64::from_i32(14));assert_eq!(level.mana_cost,Fixed64::from_i32(mana));
        }
    }
}
