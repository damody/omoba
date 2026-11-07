//! 戰國最強．啟動（本多忠勝，來源 A07T；行為依 history_nobu npc/A_Oda/A07.txt 與
//! A07_Honda_Tadakatsu.lua A07T_Transform、AON_Cleave.lua AON_Cleave_A07E）：
//! - 施放時（A07T_Transform）以目前等級立刻施放毫髮無傷與赤雷擊（不耗法力、不動冷卻），
//!   並移除身上的暈眩；
//! - 持續 duration 秒：BAT 以 attack_speed_multiplier（AttackSpeedMultiplier）表達、力量 +strength、
//!   魔抗 +magic_resist、受到傷害 damage_taken（DamageTakenBonus）、狀態抗性 status_resistance、
//!   減速抗性 slow_resistance（omoba-core damage_immunity::resist；數值見 v3.lua）；
//! - 每次普攻命中，自身 cleave_radius 內除目標外的敵方英雄與一般單位受等同最終攻擊力的物理傷害
//!   （英雄 UnitScript on_attack_hit → [`on_owner_attack_hit`]，以 rank 標記判斷）；
//! - 期間奮起不耗法力（No5_rouse 讀 [`active`]）。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,stat_keys::StatKey,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec,TargetSelector};
use omoba_core::runtime::native::damage_immunity;
use omoba_template_ids::{ABILITY_A07_STRONGEST,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A07_STRONGEST;

fn max_rank()->u8 {active_ability_const(ID).map(|c|c.max_level).unwrap_or(0)}
/// The ultimate's buff; the rank is in the id because a hook cannot read payloads.
pub(crate) fn buff_id(caster:EntityHandle,rank:u8)->String {format!("a07_strongest_r{rank}:{}:{}",caster.id,caster.gen)}

/// Rank of the active ultimate, if any.
pub(crate) fn active(world:&GameWorldDyn<'_>,caster:EntityHandle)->Option<u8> {
    (1..=max_rank()).rev().find(|r|world.has_buff(caster,buff_id(caster,*r).as_str().into()))
}

#[derive(Clone,Copy,Debug)]
pub(crate) struct Strongest {pub duration:Fixed64,pub attack_speed:Fixed64,pub magic_resist:Fixed64,pub strength:Fixed64,
    pub damage_taken:Fixed64,pub status_resistance:Fixed64,pub slow_resistance:Fixed64,pub transform:bool}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Strongest,&'static str> {
    let content=active_ability_const(ID).ok_or("missing strongest content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid strongest rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid strongest rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid strongest metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing strongest extra");
    // Percent becomes a fraction of one, rounded toward the sign (§5.3).
    let fraction=|v:Fixed64|Fixed64::from_raw(if v.raw()<0 {(v.raw()-50)/100} else {(v.raw()+50)/100});
    let spec=Strongest {duration:extra("duration")?,attack_speed:extra("attack_speed_multiplier")?,
        magic_resist:extra("magic_resist")?,strength:extra("strength")?,
        damage_taken:fraction(extra("damage_taken")?),status_resistance:fraction(extra("status_resistance")?),
        slow_resistance:fraction(extra("slow_resistance")?),transform:extra("transform")?>=Fixed64::ONE};
    if spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) {return Err("invalid strongest duration");}
    if spec.attack_speed<Fixed64::ONE || spec.attack_speed>Fixed64::from_i32(10) || spec.strength<Fixed64::ZERO
        || spec.magic_resist<Fixed64::ZERO || spec.magic_resist>=Fixed64::from_i32(100)
        || spec.damage_taken< -Fixed64::ONE || spec.damage_taken>Fixed64::ZERO
        || [spec.status_resistance,spec.slow_resistance].iter().any(|v|*v<Fixed64::ZERO || *v>Fixed64::ONE) {return Err("invalid strongest values");}
    Ok(spec)
}

pub(crate) fn payload(spec:&Strongest)->String {
    serde_json::json!({
        (StatKey::AttackSpeedMultiplier.as_str()):spec.attack_speed.raw(),
        (StatKey::StatsStrengthBonus.as_str()):spec.strength.raw(),
        // 原作 A07T: dummy.lua Add_magical_resistance — the hero's magic resistance pool (capped
        // by the game rules), not an engine bonus (rules::magic_resist).
        (StatKey::MagicResistancePool.as_str()):spec.magic_resist.raw(),
        (StatKey::DamageTakenBonus.as_str()):spec.damage_taken.raw(),
        (damage_immunity::STATUS_RESISTANCE):spec.status_resistance.raw(),
        (damage_immunity::SLOW_RESISTANCE):spec.slow_resistance.raw(),
    }).to_string()
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,rank:u8,spec:Strongest)->Result<(),&'static str> {
    if !matches!(target,Target::None) {return Err("strongest has no target");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    // A07T_Transform at the learned ranks (checked before any write).
    let guard=match super::a07_ranks::learned(world,super::No1_unscathed::ID,caster) {0=>None,r=>Some(super::No1_unscathed::spec(r)?)};
    let thunder=match super::a07_ranks::learned(world,super::No2_red_thunder::ID,caster) {0=>None,r=>Some(super::No2_red_thunder::spec(r)?)};
    world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let payload=payload(&spec);
    for r in 1..=max_rank() {
        if r!=rank {world.remove_buff(caster,buff_id(caster,r).as_str().into());}
    }
    world.remove_buff(caster,BuffId::Stun.as_rstr());
    world.add_stat_buff(caster,buff_id(caster,rank).as_str().into(),spec.duration,payload.as_str().into());
    if spec.transform {
        if let Some(guard)=guard {super::No1_unscathed::apply(world,caster,&guard);}
        if let Some(thunder)=thunder {super::No2_red_thunder::apply(world,caster,&thunder)?;}
    }
    Ok(())
}

/// Hero UnitScript `on_attack_hit` (AON_Cleave_A07E): while the ultimate is
/// active, every landed attack deals physical damage equal to the final attack
/// damage to every other live enemy hero or basic unit within the radius of
/// the caster.
pub(crate) fn on_owner_attack_hit(world:&mut GameWorldDyn<'_>,caster:EntityHandle,victim:EntityHandle) {
    let Some(rank)=active(world,caster) else {return};
    let radius=extra_at_id(ID,"cleave_radius",rank);
    let pct=extra_at_id(ID,"cleave_pct",rank);
    if radius<=Fixed64::ZERO || radius>Fixed64::from_i32(2_000) || pct<=Fixed64::ZERO || pct>Fixed64::from_i32(1_000) {return;}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return;}
    if world.is_building(victim) {return;}
    let Some(at)=world.get_pos(caster).into_option() else {return};
    let damage=Fixed64::from_raw(i64::try_from(i128::from(world.get_final_atk(caster).raw())*i128::from(pct.raw())/i128::from(Fixed64::from_i32(100).raw())).unwrap_or(0));
    if damage<=Fixed64::ZERO {return;}
    let others:Vec<_>=super::No2_red_thunder::enemy_units_in_circle(world,caster,at,radius).into_iter().filter(|e|*e!=victim).collect();
    for e in others {world.deal_damage(e,damage,DamageKind::Physical,DamageProfile::NORMAL,RSome(caster));}
}

pub struct StrongestHandler;
impl AbilityScript for StrongestHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,level,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a07_strongest_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AttackSpeedMultiplier.as_str().into(),extra_at_id_f32(ID,"attack_speed_multiplier",1));
    modifiers.insert(StatKey::StatsStrengthBonus.as_str().into(),extra_at_id_f32(ID,"strength",1));
    modifiers.insert(StatKey::MagicResistancePool.as_str().into(),extra_at_id_f32(ID,"magic_resist",1));
    build_ability_ffi(ID,StrongestHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A07_honda_tadakatsu::a07_test_support::*;
    use crate::heroes::A07_honda_tadakatsu::hero_unit::hero_unit_script;
    use crate::heroes::A07_honda_tadakatsu::{No1_unscathed,No2_red_thunder};
    use omoba_core::{Outcome,TAttack};
    use omoba_core::runtime::BuffStore;
    use specs::WorldExt;

    fn arm(t:&mut TestWorld,attack:i32) {
        t.world.write_storage::<TAttack>().insert(t.source,TAttack::new(Fixed64::from_i32(attack),Fixed64::ONE,
            Fixed64::from_i32(150),Fixed64::from_i32(1000))).unwrap();
    }

    #[test]
    fn every_rank_grants_the_history_stats_for_its_duration() {
        for (rank,seconds,strength) in [(1u8,8,15),(2,14,25),(3,20,35)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&StrongestHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let rows=buffs(&outcomes,&buff_id(t.handle(t.source),rank));
            assert_eq!(rows.len(),1,"rank {rank}");
            let (_,payload,duration)=&rows[0];
            assert_eq!(*duration,Fixed64::from_i32(seconds));
            assert_eq!(payload[StatKey::AttackSpeedMultiplier.as_str()].as_i64(),Some(q(2.25).raw()));
            assert_eq!(payload[StatKey::StatsStrengthBonus.as_str()].as_i64(),Some(i64::from(strength)*1024));
            assert_eq!(payload[StatKey::MagicResistancePool.as_str()].as_i64(),Some(50*1024));
            assert_eq!(payload[StatKey::DamageTakenBonus.as_str()],-512);
            assert_eq!(payload[damage_immunity::STATUS_RESISTANCE],512);
            assert_eq!(payload[damage_immunity::SLOW_RESISTANCE],512);
        }
    }

    #[test]
    fn the_transform_casts_the_learned_guard_and_slam_and_clears_stuns() {
        let mut t=TestWorld::new();let enemy=t.body(p(300,0),2);
        // Nothing learned: only the ultimate itself.
        let bare=t.cast(&StrongestHandler,Target::None,1,&rank_json(ID,1)).unwrap();
        assert!(buffs(&bare,&No1_unscathed::buff_id(t.handle(t.source))).is_empty());
        let w=t.learn(&No1_unscathed::UnscathedHandler,2);t.apply(w);
        let e=t.learn(&No2_red_thunder::RedThunderHandler,3);t.apply(e);
        t.world.write_resource::<BuffStore>().add(t.source,"stun",Fixed64::from_i32(2),serde_json::Value::Null);
        let outcomes=t.cast(&StrongestHandler,Target::None,1,&rank_json(ID,1)).unwrap();
        assert_eq!(buffs(&outcomes,&No1_unscathed::buff_id(t.handle(t.source)))[0].2,q(2.25),"guard at rank 2");
        assert_eq!(magic_damage(&outcomes),vec![(enemy,Fixed64::from_i32(350))],"slam at rank 3");
        t.apply(outcomes);
        assert!(!t.world.read_resource::<BuffStore>().is_stunned(t.source),"stun removed");
    }

    #[test]
    fn the_ultimate_halves_damage_taken_and_enemy_controls() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        let outcomes=t.cast(&StrongestHandler,Target::None,1,&rank_json(ID,1)).unwrap();t.apply(outcomes);
        t.apply(vec![Outcome::Damage {pos:p(0,0),phys:Fixed64::from_i32(300),magi:Fixed64::ZERO,real:Fixed64::ZERO,source:enemy,target:t.source,
            damage_profile:omb_script_abi::types::DamageProfile::NORMAL.bits(),predeclared:false}]);
        assert_eq!(t.world.read_storage::<omoba_core::CProperty>().get(t.source).unwrap().hp,Fixed64::from_i32(850));
        t.apply(vec![Outcome::AddBuff {target:t.source,buff_id:"stun".into(),duration:Fixed64::from_i32(2),payload:serde_json::Value::Null}]);
        assert_eq!(t.world.read_resource::<BuffStore>().get(t.source,"stun").unwrap().remaining,Fixed64::ONE,"status resistance halves the stun");
    }

    #[test]
    fn attacks_cleave_around_the_caster_only_while_active() {
        let mut t=TestWorld::new();arm(&mut t,120);
        let victim=t.body(p(150,0),2);let near=t.body(p(0,365),2);let far=t.body(p(0,-366),2);
        let ally=t.body(p(160,0),1);let tower=t.building(p(140,10),2);
        let unit=hero_unit_script();
        let rows=|o:&[Outcome]|o.iter().filter_map(|o|match o {Outcome::RawDamage {target,phys,magi,..}=>Some((*target,*phys,*magi)),_=>None}).collect::<Vec<_>>();
        assert!(rows(&t.attack_hit(&unit,victim)).is_empty(),"no cleave before the ultimate");
        let outcomes=t.cast(&StrongestHandler,Target::None,1,&rank_json(ID,1)).unwrap();t.apply(outcomes);
        let hits=rows(&t.attack_hit(&unit,victim));
        assert_eq!(hits,vec![(near,Fixed64::from_i32(120),Fixed64::ZERO)],"only the other enemy inside 365 of the caster");
        for e in [victim,far,ally,tower] {assert!(!hits.iter().any(|(target,_,_)|*target==e));}
        t.world.write_resource::<BuffStore>().tick(Fixed64::from_i32(9));
        assert!(rows(&t.attack_hit(&unit,victim)).is_empty(),"cleave ends with the ultimate");
    }

    #[test]
    fn wrong_target_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&StrongestHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&StrongestHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&StrongestHandler,Target::None,4,&rank_json(ID,1)).is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&StrongestHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_source_cooldown_mana_and_levels() {
        let content=active_ability_const(ID).unwrap();
        for (rank,mana) in [(1usize,150),(2,200),(3,250)] {
            let level=&content.levels[rank-1];
            assert_eq!(level.cooldown,Fixed64::from_i32(100));assert_eq!(level.mana_cost,Fixed64::from_i32(mana));
        }
    }
}
