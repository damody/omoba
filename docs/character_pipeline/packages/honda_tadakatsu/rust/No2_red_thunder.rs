//! 赤雷擊（本多忠勝，來源 A07E current）：巨槍猛擊地面，自身周圍 radius 內存活的
//! 敵方英雄與一般單位（不含建築）受 damage 魔法傷害，移動速度 −slow
//! slow_duration 秒（數值見 v3.lua）。前搖由 host windup 處理。
//! 依 history_nobu A07E 的 DelayedAction（A07E_Unslow）：施放 unslow_delay 秒後解除自身的移動減速，
//! 由英雄 UnitScript tick（[`on_owner_tick`]）送通用的 self_slow_cleanse 封包。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A07_RED_THUNDER,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

pub(crate) const ID:omoba_template_ids::AbilityId=ABILITY_A07_RED_THUNDER;

pub(crate) fn unslow_wait(caster:EntityHandle)->String {format!("a07_unslow_wait:{}:{}",caster.id,caster.gen)}
pub(crate) fn unslow_pending(caster:EntityHandle)->String {format!("a07_unslow_pending:{}:{}",caster.id,caster.gen)}

#[derive(Clone,Copy,Debug)]
pub(crate) struct Thunder {pub radius:Fixed64,pub damage:Fixed64,pub slow:Fixed64,pub slow_duration:Fixed64,pub unslow_delay:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Thunder,&'static str> {
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid red thunder metadata")?;
    spec(rank)
}

/// Content values of `rank` (1-based).
pub(crate) fn spec(rank:u8)->Result<Thunder,&'static str> {
    let content=active_ability_const(ID).ok_or("missing red thunder content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid red thunder rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid red thunder rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing red thunder extra");
    // Percent (slow) becomes a fraction of one, rounded (§5.3); applied as a negative bonus.
    let pct=extra("slow")?;
    let spec=Thunder {radius:extra("radius")?,damage:extra("damage")?,
        slow:Fixed64::from_raw((pct.raw()+50)/100),slow_duration:extra("slow_duration")?,
        unslow_delay:extra("unslow_delay")?};
    if spec.radius<=Fixed64::ZERO || spec.radius>Fixed64::from_i32(2_000) {return Err("invalid red thunder radius");}
    if spec.damage<Fixed64::ZERO || spec.slow<=Fixed64::ZERO || spec.slow>Fixed64::ONE
        || spec.slow_duration<=Fixed64::ZERO || spec.slow_duration>Fixed64::from_i32(60)
        || spec.unslow_delay<=Fixed64::ZERO || spec.unslow_delay>Fixed64::from_i32(10) {return Err("invalid red thunder values");}
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

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Thunder)->Result<usize,&'static str> {
    if !matches!(target,Target::None) {return Err("red thunder is self centered");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    apply(world,caster,&spec)
}

/// The slam around the caster; shared by the cast and 戰國最強's transform.
pub(crate) fn apply(world:&mut GameWorldDyn<'_>,caster:EntityHandle,spec:&Thunder)->Result<usize,&'static str> {
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let hits=enemy_units_in_circle(world,caster,origin,spec.radius);
    let slow_key=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),caster);
    let slow_payload=serde_json::json!({"move_speed_bonus":-spec.slow.raw(),"__aggregation_family":"generic_ability_slow"}).to_string();
    // All checks done; apply outcomes.
    for e in &hits {
        world.deal_damage(*e,spec.damage,DamageKind::Magical,DamageProfile::CRUSHING.union(DamageProfile::SPELL),RSome(caster));
        world.add_stat_buff(*e,slow_key.as_str().into(),spec.slow_duration,slow_payload.as_str().into());
    }
    // A07E_Unslow: the caster's own slows end unslow_delay seconds later.
    world.add_buff(caster,unslow_wait(caster).as_str().into(),spec.unslow_delay);
    world.add_buff(caster,unslow_pending(caster).as_str().into(),spec.unslow_delay+Fixed64::ONE);
    Ok(hits.len())
}

/// Hero UnitScript tick: once the delay has run out, cleanse the caster's own slows.
pub(crate) fn on_owner_tick(world:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    if !world.has_buff(caster,unslow_pending(caster).as_str().into()) || world.has_buff(caster,unslow_wait(caster).as_str().into()) {return;}
    world.remove_buff(caster,unslow_pending(caster).as_str().into());
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return;}
    let packet=omoba_core::runtime::native::ally_cleanse::self_slow_payload(caster.id,caster.gen).to_string();
    world.add_stat_buff(caster,omoba_core::runtime::native::ally_cleanse::SELF_SLOW_BUFF.into(),Fixed64::ONE,packet.as_str().into());
}

pub struct RedThunderHandler;
impl AbilityScript for RedThunderHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(_)=>ROk(()),Err(e)=>RErr(e.into())}
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {super::a07_ranks::learn(world,ID,caster,rank);}
}

pub fn a07_red_thunder_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,RedThunderHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![
            EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::AllEnemies,modifier_type:"slow".into(),
                value:extra_at_id_f32(ID,"slow",1)/100.0,duration:Some(extra_at_id_f32(ID,"slow_duration",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A07_honda_tadakatsu::a07_test_support::*;

    #[test]
    fn every_rank_damages_and_slows_each_enemy_within_525_once() {
        for (rank,damage,seconds) in [(1u8,150,2),(2,250,3),(3,350,4),(4,450,5)] {
            let mut t=TestWorld::new();
            let near=t.body(p(100,0),2);let edge=t.body(p(0,525),2);let out=t.body(p(0,-526),2);
            let ally=t.body(p(50,0),1);let tower=t.building(p(-100,0),2);
            let outcomes=t.cast(&RedThunderHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            assert_eq!(magic_damage(&outcomes),vec![(near,Fixed64::from_i32(damage)),(edge,Fixed64::from_i32(damage))],"rank {rank}");
            let key=omb_script_abi::buff_ids::generic_slow_buff_id(ID.as_str(),t.handle(t.source));
            let slowed:Vec<_>=buffs(&outcomes,&key).into_iter().map(|(e,payload,d)|(e,payload["move_speed_bonus"].as_i64(),d)).collect();
            assert_eq!(slowed,vec![(near,Some(-768),Fixed64::from_i32(seconds)),(edge,Some(-768),Fixed64::from_i32(seconds))],"rank {rank}");
            for e in [out,ally,tower] {assert!(!outcomes.iter().any(|o|matches!(o,omoba_core::Outcome::RawDamage {target,..} if *target==e)));}
        }
    }

    #[test]
    fn the_caster_s_own_slows_end_one_second_after_the_slam() {
        use crate::heroes::A07_honda_tadakatsu::hero_unit::hero_unit_script;
        use omoba_core::runtime::BuffStore;use specs::WorldExt;
        let mut t=TestWorld::new();let me=t.source;
        t.world.write_resource::<BuffStore>().add(me,"generic_slow:x:9:1",Fixed64::from_i32(10),
            serde_json::json!({"move_speed_bonus":-768,"__aggregation_family":"generic_ability_slow"}));
        let outcomes=t.cast(&RedThunderHandler,Target::None,1,&rank_json(ID,1)).unwrap();t.apply(outcomes);
        let unit=hero_unit_script();
        let early=t.tick(&unit);assert!(early.is_empty(),"nothing before the delay");
        t.world.write_resource::<BuffStore>().tick(Fixed64::from_raw(1100));
        let late=t.tick(&unit);t.apply(late);
        let b=t.world.read_resource::<BuffStore>();
        assert!(!b.has(me,"generic_slow:x:9:1"),"own slow cleansed");
        assert!(!b.has(me,&unslow_pending(t.handle(me))));
    }

    #[test]
    fn wrong_target_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&RedThunderHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RedThunderHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RedThunderHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RedThunderHandler,Target::None,5,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RedThunderHandler,Target::None,1,"{").is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&RedThunderHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_source_cast_point_cooldown_and_mana() {
        let content=active_ability_const(ID).unwrap();
        for (rank,mana) in [(1usize,100),(2,120),(3,140),(4,160)] {
            let level=&content.levels[rank-1];
            assert_eq!(level.cast_time,q(0.2));assert_eq!(level.cooldown,Fixed64::from_i32(10));assert_eq!(level.mana_cost,Fixed64::from_i32(mana));
        }
    }
}
