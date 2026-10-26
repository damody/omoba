//! 百箭齊發（玉子，來源 C15T）：主動擊暈目標並造成 5 點魔法傷害，同時放出 10 支致命箭矢；
//! 被動每第 8 次普攻命中放出 1 支。箭矢是 `invulnerable` 的不動召喚物（G-3 emplacement AI），
//! 存活 10 秒、每秒攻擊一次；`summon_target` 指定它們鎖定的目標，目標死亡後由 host 取最近的
//! 敵人（summon_forced_target + summon_invulnerable）。每個 rank 一個箭矢模板（傷害 50/100/150）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids,types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_C15_HUNDRED_ARROWS,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C15_HUNDRED_ARROWS;
pub(crate) const UNITS:[&str;3]=["c15_lethal_arrow_r1","c15_lethal_arrow_r2","c15_lethal_arrow_r3"];

#[derive(Clone,Copy,Debug)]
pub(crate) struct HundredArrows {pub rank:u8,pub range:Fixed64,pub stun:Fixed64,pub damage:Fixed64,pub count:u32,pub lifetime:Fixed64,
    pub passive_every:u32}

pub(crate) fn resolve(rank:u8,json:&str)->Result<HundredArrows,&'static str> {
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid hundred arrows metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid hundred arrows range");}
    Ok(HundredArrows {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),..content_spec(rank)?})
}

/// Rank values from authored content (the passive needs no cast metadata).
pub(crate) fn content_spec(rank:u8)->Result<HundredArrows,&'static str> {
    let content=active_ability_const(ID).ok_or("missing hundred arrows content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid hundred arrows rank")?);
    if index>=content.levels.len() || rank>content.max_level || index>=UNITS.len() {return Err("invalid hundred arrows rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing hundred arrows extra");
    let whole=|v:Fixed64,max:i64|if v.raw()%1024==0 && v.raw()>=1024 && v.raw()<=max*1024 {Ok((v.raw()/1024) as u32)} else {Err("hundred arrows count must be whole")};
    let spec=HundredArrows {rank,range:Fixed64::ZERO,
        stun:extra("control_duration")?,damage:extra("damage_amount")?,count:whole(extra("summon_count")?,32)?,
        lifetime:extra("summon_lifetime")?,passive_every:whole(extra("summon_2_every_nth_attack")?,100)?};
    if spec.stun<=Fixed64::ZERO || spec.stun>Fixed64::from_i32(60) || spec.lifetime<=Fixed64::ZERO || spec.lifetime>Fixed64::from_i32(600) {
        return Err("invalid hundred arrows duration");
    }
    if omoba_template_ids::summon_by_name(UNITS[index]).is_none() {return Err("missing lethal arrow template");}
    Ok(spec)
}

/// `count` arrows at the caster, each assigned to `victim`. All or nothing:
/// the first rejected placement fails the whole cast (no outcome survives).
pub(crate) fn release(world:&mut GameWorldDyn<'_>,caster:EntityHandle,victim:EntityHandle,rank:u8,count:u32,lifetime:Fixed64)
    ->Result<(),&'static str> {
    let at=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let unit=UNITS[usize::from(rank-1)];
    let payload=buff_ids::summon_target_payload(victim);
    for _ in 0..count {
        let arrow=world.spawn_summoned_unit(at,unit.into(),caster,lifetime);
        if !arrow.is_valid() {return Err("lethal arrow placement rejected");}
        world.add_stat_buff(arrow,buff_ids::SUMMON_TARGET_BUFF.into(),lifetime,payload.as_str().into());
    }
    Ok(())
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:HundredArrows)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("hundred arrows requires a unit target");};
    if victim==caster || !world.is_alive(victim) || world.get_hp(victim).into_option().is_none_or(|hp|hp<=Fixed64::ZERO) {
        return Err("hundred arrows target is not a live unit");
    }
    let (Some(from),Some(to))=(world.get_pos(caster).into_option(),world.get_pos(victim).into_option()) else {return Err("missing position");};
    if !omoba_sim::terrain::point_in_swept_circle(to,from,from,spec.range) {return Err("hundred arrows target out of range");}
    if !world.query_enemies_in_range(to,Fixed64::ONE,caster).into_iter().any(|e|e==victim) {return Err("hundred arrows target is not an enemy");}
    release(world,caster,victim,spec.rank,spec.count,spec.lifetime)?;
    world.add_buff(victim,"stun".into(),spec.stun);
    world.deal_damage(victim,spec.damage,DamageKind::Magical,DamageProfile::NORMAL,abi_stable::std_types::RSome(caster));
    Ok(())
}

/// Learned-rank marker on the caster for the hero tick (`c15_arrows_r{rank}:{id}:{gen}`).
pub(crate) fn rank_marker(caster:EntityHandle,rank:u8)->String {format!("c15_arrows_r{rank}:{}:{}",caster.id,caster.gen)}
/// Hit counter marker: `c15_arrows_hits{n}:{id}:{gen}` holds n landed attacks since the last arrow.
fn hit_marker(caster:EntityHandle,n:u32)->String {format!("c15_arrows_hits{n}:{}:{}",caster.id,caster.gen)}

pub(crate) fn learned_rank(world:&GameWorldDyn<'_>,caster:EntityHandle)->Option<u8> {
    (1..=UNITS.len() as u8).rev().find(|rank|world.has_buff(caster,rank_marker(caster,*rank).as_str().into()))
}

/// Hero UnitScript hook: every `passive_every`-th landed attack releases one arrow on the victim.
pub(crate) fn on_owner_attack_hit(world:&mut GameWorldDyn<'_>,caster:EntityHandle,victim:EntityHandle) {
    let Some(rank)=learned_rank(world,caster) else {return;};
    let Ok(spec)=content_spec(rank) else {return;};
    let seen=(1..spec.passive_every).find(|n|world.has_buff(caster,hit_marker(caster,*n).as_str().into())).unwrap_or(0);
    if seen>0 {world.remove_buff(caster,hit_marker(caster,seen).as_str().into());}
    let next=seen+1;
    if next<spec.passive_every {
        world.add_stat_buff(caster,hit_marker(caster,next).as_str().into(),Fixed64::from_raw(i64::MAX),"{}".into());
        return;
    }
    let _=release(world,caster,victim,rank,1,spec.lifetime);
}

pub struct HundredArrowsHandler;
impl AbilityScript for HundredArrowsHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
    fn on_learn(&self,caster:EntityHandle,new_level:u8,world:&mut GameWorldDyn<'_>) {
        for rank in 1..=UNITS.len() as u8 {world.remove_buff(caster,rank_marker(caster,rank).as_str().into());}
        world.add_stat_buff(caster,rank_marker(caster,new_level).as_str().into(),Fixed64::from_raw(i64::MAX),"{}".into());
    }
}

pub fn c15_hundred_arrows_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,HundredArrowsHandler,vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
        EffectSpec::Summon {unit_type:UNITS[0].into(),count:extra_at_id_f32(ID,"summon_count",1) as u32,
            duration:Some(extra_at_id_f32(ID,"summon_lifetime",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C15_tamako::c15_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn each_rank_stuns_and_releases_ten_arrows_assigned_to_the_target() {
        for (rank,stun) in [(1u8,1024),(2,1536),(3,2048)] {
            let mut t=TestWorld::new();let enemy=t.body(p(400,0),2);let victim=t.handle(enemy);
            let outcomes=t.cast(&HundredArrowsHandler,Target::Entity(victim),rank,&rank_json(ID,rank)).unwrap();
            let spawned=outcomes.iter().filter(|o|matches!(o,Outcome::SpawnSummoned {..})).count();
            assert_eq!(spawned,10,"rank {rank}");
            let assigned=outcomes.iter().filter(|o|matches!(o,Outcome::AddBuff {buff_id,payload,..}
                if buff_id==buff_ids::SUMMON_TARGET_BUFF && payload[buff_ids::SUMMON_TARGET_BUFF]==serde_json::json!([victim.id,victim.gen]))).count();
            assert_eq!(assigned,10,"every arrow locks the cast target");
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,duration,..} if buff_id=="stun" && duration.raw()==stun)),"rank {rank}");
        }
    }

    #[test]
    fn allies_out_of_range_and_bad_ranks_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let ally=t.body(p(300,0),1);let far=t.body(p(900,0),2);
        let (ally,far)=(t.handle(ally),t.handle(far));
        assert!(t.cast(&HundredArrowsHandler,Target::Entity(ally),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&HundredArrowsHandler,Target::Entity(far),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&HundredArrowsHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&HundredArrowsHandler,Target::Entity(far),4,&rank_json(ID,1)).is_err());
    }
}
