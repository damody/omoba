//! 祈勝光輝（H04，來源 C17E current）：對自身或全區域一名友軍（含建築）施放，
//! 護甲 +5、總攻擊 +7.5/15/22.5/30%、每秒回血 5/10/15/20，持續 16 秒。
//! 同一個施法者重複施放只刷新同一個 buff。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_C17_VICTORY_BLESSING,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_VICTORY_BLESSING;

pub(crate) struct Blessing {pub range:Fixed64,pub armor:Fixed64,pub damage_pct:Fixed64,pub regen:Fixed64,pub duration:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Blessing,&'static str> {
    let content=active_ability_const(ID).ok_or("missing blessing content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid blessing rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid blessing rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid blessing metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid blessing range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing blessing extra");
    let spec=Blessing {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),
        armor:extra("stat_buff_armor_bonus")?,
        // Authored in percent (7.5 = +7.5%); the stat is a fraction of one.
        damage_pct:Fixed64::from_raw((extra("stat_buff_base_damage_outgoing_pct")?.raw()+50)/100),
        regen:extra("stat_buff_health_regen")?,duration:extra("stat_buff_duration")?};
    if spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) {return Err("invalid blessing duration");}
    Ok(spec)
}

pub(crate) fn buff_id(caster:EntityHandle)->String {format!("c17_victory_blessing:{}:{}",caster.id,caster.gen)}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Blessing)->Result<(),&'static str> {
    let ally=match target {Target::Entity(e)=>e,Target::None=>caster,Target::Point(_)=>return Err("blessing requires an ally")};
    let team=world.faction_of(caster).into_option().map(|f|f.as_str().to_owned()).ok_or("missing caster team")?;
    if !world.is_alive(ally) || !world.get_hp(ally).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || world.faction_of(ally).into_option().map(|f|f.as_str()!=team).unwrap_or(true) {return Err("blessing target is not a live ally");}
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(ally).into_option().ok_or("missing target position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("blessing target outside cast range");}
    let payload=serde_json::json!({
        (StatKey::PhysicalArmorBonus.as_str()):spec.armor.raw(),
        (StatKey::BaseDamageOutgoingPercentage.as_str()):spec.damage_pct.raw(),
        (StatKey::HealthRegenConstant.as_str()):spec.regen.raw(),
    }).to_string();
    world.add_stat_buff(ally,buff_id(caster).as_str().into(),spec.duration,payload.as_str().into());
    Ok(())
}

pub struct VictoryBlessingHandler;
impl AbilityScript for VictoryBlessingHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c17_victory_blessing_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,VictoryBlessingHandler,vec![EffectSpec::StatusModifier {target:TargetSelector::Target,
        modifier_type:"victory_blessing".into(),value:extra_at_id_f32(ID,"stat_buff_base_damage_outgoing_pct",1)/100.0,
        duration:Some(extra_at_id_f32(ID,"stat_buff_duration",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;

    fn blessing(outcomes:&[Outcome])->Vec<(specs::Entity,String,Fixed64,serde_json::Value)> {
        outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {target,buff_id,duration,payload,..} if buff_id.starts_with("c17_victory_blessing:")=>
            Some((*target,buff_id.clone(),*duration,payload.clone())),_=>None}).collect()
    }

    #[test]
    fn every_rank_buffs_a_far_ally_with_rank_values_for_sixteen_seconds() {
        for (rank,pct_raw,regen) in [(1u8,77,5),(2,154,10),(3,230,15),(4,307,20)] {
            let mut t=TestWorld::new();let ally=t.body(p(9000,0),1);
            let rows=blessing(&t.cast(&VictoryBlessingHandler,Target::Entity(t.handle(ally)),rank,&rank_json(ID,rank)).unwrap());
            assert_eq!(rows.len(),1,"rank {rank}");
            let (target,_,duration,payload)=&rows[0];
            assert_eq!((*target,*duration),(ally,Fixed64::from_i32(16)));
            assert_eq!(payload[StatKey::PhysicalArmorBonus.as_str()].as_i64(),Some(5*1024));
            assert_eq!(payload[StatKey::BaseDamageOutgoingPercentage.as_str()].as_i64(),Some(pct_raw),"rank {rank}");
            assert_eq!(payload[StatKey::HealthRegenConstant.as_str()].as_i64(),Some(regen*1024),"rank {rank}");
        }
    }

    #[test]
    fn self_and_allied_buildings_are_valid_but_enemies_are_not() {
        let mut t=TestWorld::new();
        let tower=t.building(p(500,0),1);let enemy=t.body(p(300,0),2);let beyond=t.body(p(10_001,0),1);
        let source=t.source;
        assert_eq!(blessing(&t.cast(&VictoryBlessingHandler,Target::Entity(t.handle(tower)),1,&rank_json(ID,1)).unwrap())[0].0,tower);
        assert_eq!(blessing(&t.cast(&VictoryBlessingHandler,Target::None,1,&rank_json(ID,1)).unwrap())[0].0,source);
        assert!(t.cast(&VictoryBlessingHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&VictoryBlessingHandler,Target::Entity(t.handle(beyond)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&VictoryBlessingHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn recasting_refreshes_one_buff_per_caster() {
        let mut t=TestWorld::new();let ally=t.body(p(100,0),1);
        let a=blessing(&t.cast(&VictoryBlessingHandler,Target::Entity(t.handle(ally)),1,&rank_json(ID,1)).unwrap());
        let b=blessing(&t.cast(&VictoryBlessingHandler,Target::Entity(t.handle(ally)),2,&rank_json(ID,2)).unwrap());
        assert_eq!(a[0].1,b[0].1,"same buff id refreshes instead of stacking");
    }
}
