//! 光鍊罰則（H04，來源 C17W current）：在阿市與目標之間牽起 3 秒光鍊。
//! - 對敵：目標身上掛 host tether buff；自然到期時依距離結算（最遠 750）：
//!   傷害 350/520/690/860 × d/750、定身 1.5/2.5/3.5/4.5 × (1 − d/750)。
//!   目標或阿市先死亡、或 buff 被驅散則不結算。
//! - 對友：3 秒內受到傷害 −30/40/50/60%。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{DamageKind,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::tether;
use omoba_template_ids::{ABILITY_C17_LIGHT_CHAIN,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_LIGHT_CHAIN;

pub(crate) struct Chain {pub range:Fixed64,pub duration:Fixed64,pub reach:Fixed64,pub damage:Fixed64,pub root:Fixed64,
    pub guard:Fixed64,pub guard_duration:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Chain,&'static str> {
    let content=active_ability_const(ID).ok_or("missing light chain content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid light chain rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid light chain rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid light chain metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid light chain range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing light chain extra");
    let spec=Chain {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),duration:extra("tether_duration")?,
        reach:extra("tether_max_distance")?,damage:extra("damage_amount")?,root:extra("control_duration")?,
        // Authored in percent (−30 = −30% damage taken); the stat is a fraction.
        guard:Fixed64::from_raw((extra("stat_buff_incoming_damage_pct")?.raw()-50)/100),guard_duration:extra("stat_buff_duration")?};
    if spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) || spec.guard>=Fixed64::ZERO || spec.guard<=-Fixed64::ONE
        || spec.guard_duration<=Fixed64::ZERO {return Err("invalid light chain values");}
    Ok(spec)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Chain)->Result<(),&'static str> {
    let Target::Entity(other)=target else {return Err("light chain requires a unit target");};
    if other==caster || world.is_building(other) || !world.is_alive(other)
        || !world.get_hp(other).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("light chain target is not a live unit");}
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(other).into_option().ok_or("missing target position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("light chain target outside cast range");}
    let enemy=world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==other);
    let ally=!enemy && world.faction_of(caster).into_option().map(|f|f.as_str().to_owned())==world.faction_of(other).into_option().map(|f|f.as_str().to_owned());
    if enemy {
        let link=tether::Spec {source_id:caster.id,source_generation:caster.gen,amount:spec.damage,kind:DamageKind::Magical,
            root:spec.root,range:spec.reach}.checked().ok_or("invalid light chain tether")?;
        let id=format!("c17_light_chain:{}:{}",caster.id,caster.gen);
        world.add_stat_buff(other,id.as_str().into(),spec.duration,link.payload().to_string().as_str().into());
    } else if ally {
        let id=format!("c17_light_chain_guard:{}:{}",caster.id,caster.gen);
        let payload=serde_json::json!({(StatKey::IncomingDamagePercentage.as_str()):spec.guard.raw()}).to_string();
        world.add_stat_buff(other,id.as_str().into(),spec.guard_duration,payload.as_str().into());
    } else {
        return Err("light chain target is neither enemy nor ally");
    }
    Ok(())
}

pub struct LightChainHandler;
impl AbilityScript for LightChainHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c17_light_chain_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,LightChainHandler,vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"root".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"control_duration",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn enemies_get_a_rank_tether_and_allies_a_damage_guard() {
        for (rank,damage,root,guard) in [(1u8,350,1.5f64,-307),(2,520,2.5,-410),(3,690,3.5,-512),(4,860,4.5,-614)] {
            let mut t=TestWorld::new();let enemy=t.body(p(500,0),2);let ally=t.body(p(300,0),1);
            let link=t.cast(&LightChainHandler,Target::Entity(t.handle(enemy)),rank,&rank_json(ID,rank)).unwrap().iter().find_map(|o|match o {
                Outcome::AddBuff {target,payload,duration,..} if *target==enemy=>Some((tether::Spec::from_payload(payload),*duration)),_=>None}).expect("tether");
            let spec=link.0.expect("strict tether payload");
            assert_eq!(link.1,Fixed64::from_i32(3));
            assert_eq!((spec.amount,spec.range),(Fixed64::from_i32(damage),Fixed64::from_i32(750)),"rank {rank}");
            assert_eq!(spec.root,Fixed64::from_raw((root*1024.0).round() as i64),"rank {rank}");
            let rows:Vec<_>=t.cast(&LightChainHandler,Target::Entity(t.handle(ally)),rank,&rank_json(ID,rank)).unwrap().iter().filter_map(|o|match o {
                Outcome::AddBuff {target,payload,duration,..} if *target==ally=>Some((payload[StatKey::IncomingDamagePercentage.as_str()].as_i64(),*duration)),_=>None}).collect();
            assert_eq!(rows,vec![(Some(guard),Fixed64::from_i32(3))],"rank {rank}");
        }
    }

    #[test]
    fn self_buildings_and_out_of_range_are_rejected() {
        let mut t=TestWorld::new();let tower=t.building(p(200,0),2);let far=t.body(p(601,0),2);let source=t.source;
        for target in [source,tower,far] {assert!(t.cast(&LightChainHandler,Target::Entity(t.handle(target)),1,&rank_json(ID,1)).is_err());}
        assert!(t.cast(&LightChainHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }
}
