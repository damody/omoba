//! 冰山之襲（H03，來源 A34E current）：主目標魔法傷害，周圍 335 內其他敵人受冰爆
//! 傷害；所有被擊中者移動速度 −30%、攻擊速度 −20%，持續 5 秒。瞬發、無投射物。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A34_ICEBERG_STRIKE,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A34_ICEBERG_STRIKE;

pub(crate) struct Iceberg {pub range:Fixed64,pub radius:Fixed64,pub main:Fixed64,pub splash:Fixed64,
    pub move_slow:Fixed64,pub attack_slow:Fixed64,pub duration:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Iceberg,&'static str> {
    let content=active_ability_const(ID).ok_or("missing iceberg content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid iceberg rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid iceberg rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid iceberg metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid iceberg range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing iceberg extra");
    // Slow extras are authored in percent (30 = −30%); fractions round to nearest.
    let percent=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    let spec=Iceberg {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),radius:extra("shape_radius")?,
        main:extra("damage_amount")?,splash:extra("damage_2_amount")?,move_slow:percent(extra("slow_move_speed_pct")?),
        attack_slow:percent(extra("slow_attack_speed_reduction")?),duration:extra("slow_duration")?};
    if spec.move_slow<=Fixed64::ZERO || spec.move_slow>Fixed64::ONE || spec.attack_slow<Fixed64::ZERO || spec.attack_slow>=Fixed64::ONE
        || spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) {return Err("invalid iceberg slow");}
    Ok(spec)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Iceberg)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("iceberg requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    let live=|world:&GameWorldDyn<'_>,e:EntityHandle|e!=caster && world.is_alive(e) && world.get_hp(e).into_option().is_some_and(|hp|hp>Fixed64::ZERO);
    if !live(world,victim) || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("iceberg target is not a live enemy");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("iceberg target outside cast range");}
    let mut splash:Vec<EntityHandle>=world.query_enemies_in_range(at,spec.radius,caster).into_iter()
        .filter(|e|*e!=victim && live(world,*e)).collect();
    splash.sort_by_key(|e|(e.id,e.gen));splash.dedup();
    // All checks done; apply outcomes.
    world.deal_damage(victim,spec.main,DamageKind::Magical,DamageProfile::COLD,RSome(caster));
    for e in &splash {world.deal_damage(*e,spec.splash,DamageKind::Magical,DamageProfile::COLD,RSome(caster));}
    let slow_key=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),caster);
    let move_payload=serde_json::json!({"move_speed_bonus":-spec.move_slow.raw(),"__aggregation_family":"generic_ability_slow"}).to_string();
    let attack_key=format!("a34_iceberg_attack_slow:{}:{}",caster.id,caster.gen);
    let attack_payload=serde_json::json!({"attack_speed_multiplier":1024-spec.attack_slow.raw()}).to_string();
    for e in std::iter::once(victim).chain(splash) {
        world.add_stat_buff(e,slow_key.as_str().into(),spec.duration,move_payload.as_str().into());
        world.add_stat_buff(e,attack_key.as_str().into(),spec.duration,attack_payload.as_str().into());
    }
    Ok(())
}

pub struct IcebergStrikeHandler;
impl AbilityScript for IcebergStrikeHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a34_iceberg_strike_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,IcebergStrikeHandler,vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
        EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"shape_radius",1),duration:0.2,tick_interval:0.0,
            effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage_2_amount",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A34_omatsu::a34_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_hits_the_target_splashes_others_once_and_slows_all_hit() {
        for (rank,main) in [(1u8,100),(2,200),(3,300),(4,400)] {
            let mut t=TestWorld::new();
            let target=t.body(p(400,0),2);let near=t.body(p(400,335),2);let far=t.body(p(400,336),2);let ally=t.body(p(400,10),1);
            let outcomes=t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(target)),rank,&rank_json(ID,rank)).unwrap();
            let mut damage:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
            damage.sort_by_key(|(e,_)|e.id());
            assert_eq!(damage,vec![(target,Fixed64::from_i32(main)),(near,Fixed64::from_i32(100))],"rank {rank}");
            let slowed:std::collections::BTreeSet<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {target,payload,duration,..}
                if payload.get("move_speed_bonus").is_some()=>{assert_eq!(*duration,Fixed64::from_i32(5));
                    assert_eq!(payload["move_speed_bonus"].as_i64(),Some(-307));Some(target.id())},_=>None}).collect();
            assert_eq!(slowed,[target.id(),near.id()].into_iter().collect(),"rank {rank}");
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {payload,..} if payload["attack_speed_multiplier"].as_i64()==Some(1024-205))));
            assert!(!damage.iter().any(|(e,_)|*e==far||*e==ally));
        }
    }

    #[test]
    fn invalid_targets_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let ally=t.body(p(100,0),1);let far=t.body(p(701,0),2);
        assert!(t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(ally)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(far)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&IcebergStrikeHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }
}
