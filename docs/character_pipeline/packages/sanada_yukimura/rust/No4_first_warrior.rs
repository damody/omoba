//! 日本第一武士（真田幸村，來源 B06T；history_nobu npc/B_Unified/B06.txt）：
//! - 持續 duration 秒：BAT 縮短（以 AttackSpeedMultiplier attack_speed_mult 表達）、生命上限 +max_hp
//!   並立即回復 heal、基礎攻擊 +base_attack、移速 +move_speed；
//! - modifier_B06T_Aura：aura_duration 秒內全圖（aura_radius）友方英雄與一般單位（含自己）移速 +aura_move_speed。
//!   施放時在自身掛上通用光環發射器（aura_emitter，E173，與道具被動光環同一機制）aura_duration 秒；每次 settle 由
//!   核心把光環刷新到範圍內友軍，視窗到期即停止。數值見 v3.lua。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec,TargetSelector};
use omoba_core::runtime::native::aura_emitter;
use omoba_template_ids::{ABILITY_B06_FIRST_WARRIOR,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B06_FIRST_WARRIOR;

fn max_rank()->u8 {active_ability_const(ID).map(|c|c.max_level).unwrap_or(0)}
pub(crate) fn buff_id(caster:EntityHandle)->String {format!("b06_first_warrior:{}:{}",caster.id,caster.gen)}
/// The aura window; the rank is in the id because a hook cannot read payloads.
pub(crate) fn aura_marker(caster:EntityHandle,rank:u8)->String {format!("b06_first_warrior_aura_r{rank}:{}:{}",caster.id,caster.gen)}
/// The buff the aura keeps refreshed on allies.
pub(crate) fn aura_buff(caster:EntityHandle)->String {format!("b06_first_warrior_haste:{}:{}",caster.id,caster.gen)}

pub(crate) struct Warrior {pub duration:Fixed64,pub attack_speed:Fixed64,pub max_hp:Fixed64,pub move_pct:Fixed64,
    pub attack:Fixed64,pub heal:Fixed64,pub aura_seconds:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Warrior,&'static str> {
    let content=active_ability_const(ID).ok_or("missing first warrior content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid first warrior rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid first warrior rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid first warrior metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing first warrior extra");
    let spec=Warrior {duration:extra("duration")?,attack_speed:extra("attack_speed_mult")?,
        max_hp:extra("max_hp")?,move_pct:extra("move_speed")?,
        attack:extra("base_attack")?,heal:extra("heal")?,aura_seconds:extra("aura_duration")?};
    if spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(120) || spec.aura_seconds<=Fixed64::ZERO || spec.aura_seconds>spec.duration
        || spec.attack_speed<Fixed64::ONE || spec.max_hp<Fixed64::ZERO || spec.attack<Fixed64::ZERO || spec.heal<Fixed64::ZERO {return Err("invalid first warrior values");}
    Ok(spec)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,rank:u8,spec:Warrior)->Result<(),&'static str> {
    if !matches!(target,Target::None) {return Err("first warrior has no target");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let payload=serde_json::json!({
        (StatKey::AttackSpeedMultiplier.as_str()):spec.attack_speed.raw(),
        (StatKey::HealthBonus.as_str()):spec.max_hp.raw(),
        (StatKey::BaseAttackBonusDamage.as_str()):spec.attack.raw(),
        // Authored percent becomes a fraction of one, rounded (§5.3).
        (StatKey::MoveSpeedBonusPercentage.as_str()):(spec.move_pct.raw()+50)/100,
    }).to_string();
    world.add_stat_buff(caster,buff_id(caster).as_str().into(),spec.duration,payload.as_str().into());
    for r in 1..=max_rank() {if r!=rank {world.remove_buff(caster,aura_marker(caster,r).as_str().into());}}
    world.add_buff(caster,aura_marker(caster,rank).as_str().into(),spec.aura_seconds);
    // The team-wide aura: an emitter on the caster for the aura window (the core pulses it).
    let aura=aura_spec(caster,rank).ok_or("invalid first warrior aura")?;
    world.add_stat_buff(caster,aura_emitter::buff_id(&aura.name).as_str().into(),spec.aura_seconds,aura.payload().to_string().as_str().into());
    // The heal lands after the max HP bonus is settled the same tick (Heal %Bonus_Health).
    if spec.heal>Fixed64::ZERO {world.heal(caster,spec.heal);}
    Ok(())
}

/// The aura at `rank` (aura_emitter spec): per-caster buff id, so two Yukimura auras stack as before.
pub(crate) fn aura_spec(caster:EntityHandle,rank:u8)->Option<aura_emitter::Spec> {
    let pct=extra_at_id(ID,"aura_move_speed",rank);
    let radius=extra_at_id(ID,"aura_radius",rank);
    aura_emitter::Spec {name:aura_buff(caster),radius,
        stats:vec![(StatKey::MoveSpeedBonusPercentage.as_str().to_owned(),(pct.raw()+50)/100)]}.checked()
}

pub struct FirstWarriorHandler;
impl AbilityScript for FirstWarriorHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,level,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b06_first_warrior_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AttackSpeedMultiplier.as_str().into(),extra_at_id_f32(ID,"attack_speed_mult",1));
    modifiers.insert(StatKey::HealthBonus.as_str().into(),extra_at_id_f32(ID,"max_hp",1));
    modifiers.insert(StatKey::BaseAttackBonusDamage.as_str().into(),extra_at_id_f32(ID,"base_attack",1));
    modifiers.insert(StatKey::MoveSpeedBonusPercentage.as_str().into(),extra_at_id_f32(ID,"move_speed",1)/100.0);
    build_ability_ffi(ID,FirstWarriorHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B06_sanada_yukimura::b06_test_support::*;
    use omoba_core::Outcome;
    use omoba_core::runtime::BuffStore;
    use specs::WorldExt;

    #[test]
    fn every_rank_grants_the_history_stats_and_heals_the_bonus() {
        for (rank,seconds,hp,attack) in [(1u8,40,500,40),(2,50,750,50),(3,60,1000,60)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&FirstWarriorHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let rows=buffs(&outcomes,&buff_id(t.handle(t.source)));
            assert_eq!(rows.len(),1,"rank {rank}");
            let (_,payload,duration)=&rows[0];
            assert_eq!(*duration,Fixed64::from_i32(seconds));
            assert_eq!(payload[StatKey::AttackSpeedMultiplier.as_str()].as_i64(),Some(2*1024));
            assert_eq!(payload[StatKey::HealthBonus.as_str()].as_i64(),Some(i64::from(hp)*1024));
            assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()].as_i64(),Some(i64::from(attack)*1024));
            assert_eq!(payload[StatKey::MoveSpeedBonusPercentage.as_str()],307,"+30%");
            let heals:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::ScriptHeal {amount,..}=>Some(*amount),_=>None}).collect();
            assert_eq!(heals,vec![Fixed64::from_i32(hp)]);
            assert_eq!(buffs(&outcomes,&aura_marker(t.handle(t.source),rank))[0].2,Fixed64::from_i32(15),"aura window 15 s");
        }
    }

    #[test]
    fn the_aura_reaches_far_allies_including_the_caster_for_15_seconds_only() {
        let mut t=TestWorld::new();
        let far_ally=t.body(p(20_000,0),1);let enemy=t.body(p(100,0),2);
        let outcomes=t.cast(&FirstWarriorHandler,Target::None,1,&rank_json(ID,1)).unwrap();t.apply(outcomes);
        let key=aura_buff(t.handle(t.source));
        assert_eq!(t.world.read_resource::<BuffStore>().get(t.source,&aura_emitter::buff_id(&key)).unwrap().remaining,
            Fixed64::from_i32(15),"emitter for the 15 s aura window");
        omoba_core::runtime::native::item_modifiers::settle_item_auras(&mut t.world);
        {
            let b=t.world.read_resource::<BuffStore>();
            for e in [t.source,far_ally] {assert_eq!(b.get(e,&key).unwrap().payload[StatKey::MoveSpeedBonusPercentage.as_str()],82,"+8% on {e:?}");}
            assert!(!b.has(enemy,&key));
        }
        // After the window the emitter is gone: no further refresh, the last one lingers out.
        t.world.write_resource::<BuffStore>().tick(Fixed64::from_i32(16));
        omoba_core::runtime::native::item_modifiers::settle_item_auras(&mut t.world);
        let b=t.world.read_resource::<BuffStore>();
        assert!(!b.has(t.source,&key) && !b.has(far_ally,&key),"no aura after 15 s");
    }

    #[test]
    fn wrong_target_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&FirstWarriorHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FirstWarriorHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FirstWarriorHandler,Target::None,4,&rank_json(ID,1)).is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&FirstWarriorHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_history_cooldown_and_mana() {
        let content=active_ability_const(ID).unwrap();
        for (rank,cooldown,mana) in [(1usize,155,300),(2,145,400),(3,135,500)] {
            let level=&content.levels[rank-1];
            assert_eq!(level.cooldown,Fixed64::from_i32(cooldown));assert_eq!(level.mana_cost,Fixed64::from_i32(mana));
        }
    }
}
