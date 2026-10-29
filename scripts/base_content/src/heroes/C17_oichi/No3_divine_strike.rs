//! 神助一擊（H04，來源 C17R current）：指定地點 200 範圍內的敵方英雄與小兵各受
//! 200/300/400/500 魔法傷害一次。不打建築；範圍內沒有敵人照常施放。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_C17_DIVINE_STRIKE,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_DIVINE_STRIKE;

pub(crate) struct Strike {pub range:Fixed64,pub radius:Fixed64,pub damage:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Strike,&'static str> {
    let content=active_ability_const(ID).ok_or("missing divine strike content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid divine strike rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid divine strike rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid divine strike metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid divine strike range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing divine strike extra");
    Ok(Strike {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),radius:extra("radius")?,damage:extra("damage")?})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Strike)->Result<(),&'static str> {
    let Target::Point(at)=target else {return Err("divine strike requires a point");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("divine strike point outside cast range");}
    let mut hit:Vec<EntityHandle>=world.query_enemies_in_range(at,spec.radius,caster).into_iter()
        .filter(|e|*e!=caster && !world.is_building(*e) && world.is_alive(*e)
            && world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO)).collect();
    hit.sort_by_key(|e|(e.id,e.gen));hit.dedup();
    for e in hit {
        // AMHC:Damage: nothing once Oichi has died (amhc.lua:540).
        world.deal_damage(e,spec.damage,DamageKind::Magical,DamageProfile::ENERGY.union(DamageProfile::REQUIRE_SOURCE_ALIVE),RSome(caster));
    }
    Ok(())
}

pub struct DivineStrikeHandler;
impl AbilityScript for DivineStrikeHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c17_divine_strike_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,DivineStrikeHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;

    fn damage(outcomes:&[Outcome])->Vec<(specs::Entity,Fixed64)> {
        let mut rows:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
        rows.sort_by_key(|(e,_)|e.id());rows
    }

    #[test]
    fn every_rank_damages_each_enemy_in_the_radius_once() {
        for (rank,amount,range) in [(1u8,200,600),(2,300,700),(3,400,800),(4,500,1000)] {
            let mut t=TestWorld::new();
            let centre=t.body(p(range-200,0),2);let edge=t.body(p(range-200,200),2);let out=t.body(p(range-200,201),2);
            let ally=t.body(p(range-200,10),1);let tower=t.building(p(range-190,0),2);
            let rows=damage(&t.cast(&DivineStrikeHandler,Target::Point(p(range-200,0)),rank,&rank_json(ID,rank)).unwrap());
            assert_eq!(rows,vec![(centre,Fixed64::from_i32(amount)),(edge,Fixed64::from_i32(amount))],"rank {rank}");
            assert!(!rows.iter().any(|(e,_)|*e==out||*e==ally||*e==tower));
        }
    }

    #[test]
    fn range_is_per_rank_and_an_empty_point_still_casts() {
        let mut t=TestWorld::new();
        assert!(t.cast(&DivineStrikeHandler,Target::Point(p(601,0)),1,&rank_json(ID,1)).is_err());
        assert!(damage(&t.cast(&DivineStrikeHandler,Target::Point(p(1000,0)),4,&rank_json(ID,4)).unwrap()).is_empty());
        assert!(t.cast(&DivineStrikeHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }
}
