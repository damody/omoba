//! 冰山之襲（H03，來源 A34E current，A34.txt:147-330）：先對指定敵人造成 aoe_damage
//! （v3.lua target_damage），再對其周圍 radius 內所有敵人（含指定目標）依序套緩速、造成
//! AbilityDamage（damage）；緩速為移速 −slow、攻速常數 −attack_slow 點，持續 slow_duration。目標與範圍都只有英雄
//! 與一般單位（不含建築）。瞬發、無投射物。急速冷卻主動期間每個被擊中者各觸發一次
//! A34T_OnHit（host hit_stack）。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::hit_stack;
use omoba_template_ids::{ABILITY_A34_ICEBERG_STRIKE,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A34_ICEBERG_STRIKE;
/// One attack slow per target, like the source's `as_slow["A34E"]` key.
pub(crate) const ATTACK_SLOW_BUFF:&str="a34_iceberg_attack_slow";

pub(crate) struct Iceberg {pub range:Fixed64,pub radius:Fixed64,pub area:Fixed64,pub target_bonus:Fixed64,
    pub move_slow:Fixed64,pub attack_slow:Fixed64,pub duration:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Iceberg,&'static str> {
    let content=active_ability_const(ID).ok_or("missing iceberg content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid iceberg rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid iceberg rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid iceberg metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid iceberg range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing iceberg extra");
    // The move slow is authored in percent (kit `slow`, unit pct); the attack slow is
    // attack speed points (ATTACKSPEED_BONUS_CONSTANT, A34.txt:314 + slow_self 6016-6024).
    let percent=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    let spec=Iceberg {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),radius:extra("radius")?,
        area:extra("damage")?,target_bonus:extra("target_damage")?,move_slow:percent(extra("slow")?),
        attack_slow:extra("attack_slow")?,duration:extra("slow_duration")?};
    if spec.move_slow<=Fixed64::ZERO || spec.move_slow>Fixed64::ONE || spec.attack_slow<Fixed64::ZERO || spec.attack_slow>Fixed64::from_i32(1_000)
        || spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) {return Err("invalid iceberg slow");}
    Ok(spec)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Iceberg)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("iceberg requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    // Source targets HERO | BASIC only (A34.txt:154-155, 213-214).
    let live=|world:&GameWorldDyn<'_>,e:EntityHandle|e!=caster && world.is_alive(e) && !world.is_building(e)
        && world.get_hp(e).into_option().is_some_and(|hp|hp>Fixed64::ZERO);
    if !live(world,victim) || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("iceberg target is not a live enemy unit");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("iceberg target outside cast range");}
    let mut hit:Vec<EntityHandle>=world.query_enemies_in_range(at,spec.radius,caster).into_iter()
        .filter(|e|*e!=victim && live(world,*e)).collect();
    hit.push(victim);
    hit.sort_by_key(|e|(e.id,e.gen));hit.dedup();
    let shred=world.has_buff(caster,super::No4_rapid_cooling::ACTIVE_BUFF.into());
    // All checks done; apply outcomes in source order (A34.txt:201-236).
    world.deal_damage(victim,spec.target_bonus,DamageKind::Magical,DamageProfile::COLD.union(DamageProfile::SPELL),RSome(caster));
    let slow_key=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),caster);
    let move_payload=serde_json::json!({"move_speed_bonus":-spec.move_slow.raw(),"__aggregation_family":"generic_ability_slow"}).to_string();
    let attack_payload=serde_json::json!({(StatKey::AttackSpeedBonusConstant.as_str()):-spec.attack_slow.raw()}).to_string();
    let stack=hit_stack::apply_payload(caster.id,caster.gen,super::No4_rapid_cooling::ACTIVE_BUFF).to_string();
    for e in hit {
        world.add_stat_buff(e,slow_key.as_str().into(),spec.duration,move_payload.as_str().into());
        world.add_stat_buff(e,ATTACK_SLOW_BUFF.into(),spec.duration,attack_payload.as_str().into());
        world.deal_damage(e,spec.area,DamageKind::Magical,DamageProfile::COLD.union(DamageProfile::SPELL),RSome(caster));
        if shred {world.add_stat_buff(e,hit_stack::APPLY_BUFF.into(),Fixed64::ONE,stack.as_str().into());}
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
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"target_damage",1),damage_type:DamageType::Magical},
        EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"radius",1),duration:0.2,tick_interval:0.0,
            effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A34_omatsu::a34_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_hits_the_target_twice_splashes_all_in_radius_and_slows_all_hit() {
        for (rank,area) in [(1u8,100),(2,200),(3,300),(4,400)] {
            let mut t=TestWorld::new();
            let target=t.body(p(400,0),2);let near=t.body(p(400,335),2);let far=t.body(p(400,336),2);let ally=t.body(p(400,10),1);
            let tower=t.building(p(400,-100),2);
            let outcomes=t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(target)),rank,&rank_json(ID,rank)).unwrap();
            let damage:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
            // aoe_damage 100 on the target first, then AbilityDamage on everyone in radius (target included).
            let mut expected=vec![(target,Fixed64::from_i32(area)),(near,Fixed64::from_i32(area))];
            expected.sort_by_key(|(e,_)|e.id());
            expected.insert(0,(target,Fixed64::from_i32(100)));
            assert_eq!(damage,expected,"rank {rank}");
            let slowed:std::collections::BTreeSet<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {target,payload,duration,..}
                if payload.get("move_speed_bonus").is_some()=>{assert_eq!(*duration,Fixed64::from_i32(5));
                    assert_eq!(payload["move_speed_bonus"].as_i64(),Some(-307));Some(target.id())},_=>None}).collect();
            assert_eq!(slowed,[target.id(),near.id()].into_iter().collect(),"rank {rank}");
            let attack:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id==ATTACK_SLOW_BUFF=>
                Some(payload[StatKey::AttackSpeedBonusConstant.as_str()].as_i64()),_=>None}).collect();
            assert_eq!(attack,vec![Some(-20*1024);2],"attack speed −20 points");
            assert!(!damage.iter().any(|(e,_)|*e==far||*e==ally||*e==tower));
            assert!(!outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,..} if buff_id==hit_stack::APPLY_BUFF)),"no rapid cooling: no shred");
        }
    }

    #[test]
    fn rapid_cooling_active_requests_one_shred_stack_per_unit_hit() {
        let mut t=TestWorld::new();
        let target=t.body(p(400,0),2);let near=t.body(p(400,300),2);
        t.source_buff(super::super::No4_rapid_cooling::ACTIVE_BUFF);
        let outcomes=t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(target)),1,&rank_json(ID,1)).unwrap();
        let mut shred:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {target,buff_id,payload,..} if buff_id==hit_stack::APPLY_BUFF=>{
            assert_eq!(payload[hit_stack::APPLY_KEY]["caster_buff"],super::super::No4_rapid_cooling::ACTIVE_BUFF);Some(target.id())},_=>None}).collect();
        shred.sort();
        let mut expected=vec![target.id(),near.id()];expected.sort();
        assert_eq!(shred,expected);
    }

    #[test]
    fn invalid_targets_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let ally=t.body(p(100,0),1);let far=t.body(p(701,0),2);let tower=t.building(p(300,0),2);
        assert!(t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(ally)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(far)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&IcebergStrikeHandler,Target::Entity(t.handle(tower)),1,&rank_json(ID,1)).is_err(),"not on buildings");
        assert!(t.cast(&IcebergStrikeHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }
}
