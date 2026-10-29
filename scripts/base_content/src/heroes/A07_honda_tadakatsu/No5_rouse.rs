//! 奮起（本多忠勝，來源 A07D；行為依 history_nobu A07_Honda_Tadakatsu.lua
//! A07D_HealHP、A07R_Learn_Skill）：D 欄自動學會。回復最大生命 heal_pct，攻速
//! attack_speed_spear_<碎擊槍等級>（原作把奮起設為碎擊槍等級 +1），持續 duration 秒（見 v3.lua）。
//! 戰國最強期間換成零法力的 A07D2：施放後退還本次法力。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A07_ROUSE,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A07_ROUSE;

pub(crate) fn buff_id(caster:EntityHandle)->String {format!("a07_rouse:{}:{}",caster.id,caster.gen)}

pub(crate) struct Rouse {pub heal_pct:Fixed64,pub attack_speed:[Fixed64;5],pub duration:Fixed64,pub mana:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Rouse,&'static str> {
    let content=active_ability_const(ID).ok_or("missing rouse content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid rouse rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid rouse rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid rouse metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing rouse extra");
    let mut attack_speed=[Fixed64::ZERO;5];
    for (spear,slot) in attack_speed.iter_mut().enumerate() {*slot=extra(&format!("attack_speed_spear_{spear}"))?;}
    let spec=Rouse {heal_pct:extra("heal_pct")?,attack_speed,duration:extra("duration")?,mana:content.levels[index].mana_cost};
    if spec.heal_pct<Fixed64::ZERO || spec.heal_pct>Fixed64::from_i32(100) || spec.attack_speed.iter().any(|v|*v<Fixed64::ZERO)
        || spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) {return Err("invalid rouse values");}
    Ok(spec)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Rouse)->Result<(),&'static str> {
    if !matches!(target,Target::None) {return Err("rouse has no target");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let max_hp=world.get_max_hp(caster).into_option().ok_or("missing caster max hp")?;
    let spear=usize::from(super::a07_ranks::learned(world,super::No3_shattering_spear::ID,caster)).min(4);
    let heal=Fixed64::from_raw(i64::try_from(i128::from(max_hp.raw())*i128::from(spec.heal_pct.raw())/i128::from(Fixed64::from_i32(100).raw())).unwrap_or(0));
    let payload=serde_json::json!({(StatKey::AttackSpeedBonusConstant.as_str()):spec.attack_speed[spear].raw()}).to_string();
    let free=super::No4_strongest::active(world,caster).is_some();
    if heal>Fixed64::ZERO {world.heal(caster,heal);}
    world.add_stat_buff(caster,buff_id(caster).as_str().into(),spec.duration,payload.as_str().into());
    if free && spec.mana>Fixed64::ZERO {world.restore_mana(caster,spec.mana);}
    Ok(())
}

pub struct RouseHandler;
impl AbilityScript for RouseHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a07_rouse_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AttackSpeedBonusConstant.as_str().into(),extra_at_id_f32(ID,"attack_speed_spear_0",1));
    build_ability_ffi(ID,RouseHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A07_honda_tadakatsu::a07_test_support::*;
    use crate::heroes::A07_honda_tadakatsu::No3_shattering_spear::ShatteringSpearHandler;
    use omoba_core::Outcome;

    fn speed(t:&TestWorld,outcomes:&[Outcome])->Option<i64> {
        buffs(outcomes,&buff_id(t.handle(t.source))).first().and_then(|row|row.1[StatKey::AttackSpeedBonusConstant.as_str()].as_i64())
    }

    #[test]
    fn heals_three_percent_and_attack_speed_follows_the_spear_rank() {
        let mut t=TestWorld::new();
        let outcomes=t.cast(&RouseHandler,Target::None,1,&rank_json(ID,1)).unwrap();
        let heals:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::ScriptHeal {target,amount}=>Some((*target,*amount)),_=>None}).collect();
        assert_eq!(heals,vec![(t.source,Fixed64::from_i32(30))],"3% of 1000");
        assert_eq!(speed(&t,&outcomes),Some(60*1024),"spear not learned: level 1");
        assert_eq!(buffs(&outcomes,&buff_id(t.handle(t.source)))[0].2,Fixed64::from_i32(3));
        for (spear,expected) in [(1u8,70),(2,80),(3,90),(4,100)] {
            let learned=t.learn(&ShatteringSpearHandler,spear);t.apply(learned);
            let outcomes=t.cast(&RouseHandler,Target::None,1,&rank_json(ID,1)).unwrap();
            assert_eq!(speed(&t,&outcomes),Some(expected*1024),"spear rank {spear}");
        }
    }

    #[test]
    fn wrong_target_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&RouseHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RouseHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RouseHandler,Target::None,2,&rank_json(ID,1)).is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&RouseHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_source_cooldown_and_mana() {
        let content=active_ability_const(ID).unwrap();
        assert_eq!(content.max_level,1);
        assert_eq!(content.levels[0].cooldown,Fixed64::from_i32(5));assert_eq!(content.levels[0].mana_cost,Fixed64::from_i32(70));
    }
}
