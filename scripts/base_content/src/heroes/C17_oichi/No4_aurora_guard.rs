//! 極光守護（H04，來源 C17T current）：施法前搖 1 秒後在指定地點 800 範圍釋放：
//! 敵方英雄與小兵受 max(650/700/750, 最大生命 20/25/30%) 魔法傷害；範圍內所有
//! 友軍（含自己）回復最大生命 40/45/50%。友軍由 host ally_area_restore 封包
//! 依提交位置選取。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ally_area_restore;
use omoba_template_ids::{ABILITY_C17_AURORA_GUARD,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_AURORA_GUARD;

pub(crate) struct Aurora {pub range:Fixed64,pub radius:Fixed64,pub damage:Fixed64,pub hp_pct:Fixed64,pub heal:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Aurora,&'static str> {
    let content=active_ability_const(ID).ok_or("missing aurora content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid aurora rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid aurora rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid aurora metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid aurora range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing aurora extra");
    // Percent extras become fractions of one.
    let fraction=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    Ok(Aurora {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),radius:extra("shape_radius")?,
        damage:extra("damage_amount")?,hp_pct:fraction(extra("damage_max_hp_pct")?),heal:fraction(extra("heal_amount")?)})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Aurora)->Result<(),&'static str> {
    let Target::Point(at)=target else {return Err("aurora requires a point");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("aurora point outside cast range");}
    let restore=ally_area_restore::Spec {source_id:caster.id,source_generation:caster.gen,center:at,radius:spec.radius,
        hp_fraction:spec.heal,mana_fraction:Fixed64::ZERO,cast_range:spec.range}.checked().ok_or("invalid aurora heal")?;
    let mut enemies:Vec<EntityHandle>=world.query_enemies_in_range(at,spec.radius,caster).into_iter()
        .filter(|e|*e!=caster && !world.is_building(*e) && world.is_alive(*e) && world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO)).collect();
    enemies.sort_by_key(|e|(e.id,e.gen));enemies.dedup();
    let hits:Vec<(EntityHandle,Fixed64)>=enemies.into_iter().map(|e|{
        let by_hp=world.get_max_hp(e).into_option().map_or(Fixed64::ZERO,|max|Fixed64::from_raw((i128::from(max.raw())*i128::from(spec.hp_pct.raw())/1024) as i64));
        (e,spec.damage.max(by_hp))
    }).collect();
    for (e,amount) in hits {world.deal_damage(e,amount,DamageKind::Magical,DamageProfile::ENERGY,RSome(caster));}
    world.add_stat_buff(caster,ally_area_restore::BUFF.into(),Fixed64::ONE,restore.payload().to_string().as_str().into());
    Ok(())
}

pub struct AuroraGuardHandler;
impl AbilityScript for AuroraGuardHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c17_aurora_guard_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,AuroraGuardHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"shape_radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn every_rank_damages_enemies_by_the_higher_rule_and_sends_one_ally_heal() {
        for (rank,flat,pct,heal) in [(1u8,650,205,410),(2,700,256,461),(3,750,307,512)] {
            let mut t=TestWorld::new();
            let enemy=t.body(p(900,0),2);let far=t.body(p(900,801),2);let tower=t.building(p(900,10),2);
            let outcomes=t.cast(&AuroraGuardHandler,Target::Point(p(900,0)),rank,&rank_json(ID,rank)).unwrap();
            let damage:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
            // Fixture bodies have 1000 max HP: percentage loses to the flat amount.
            assert_eq!(damage,vec![(enemy,Fixed64::from_i32(flat))],"rank {rank}");
            assert!(!damage.iter().any(|(e,_)|*e==far||*e==tower));
            let packet=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}=>
                ally_area_restore::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("ally heal packet");
            assert_eq!((packet.hp_fraction,packet.radius,packet.center),(Fixed64::from_raw(heal),Fixed64::from_i32(800),p(900,0)),"rank {rank}");
            assert_eq!(resolve(rank,&rank_json(ID,rank)).unwrap().hp_pct,Fixed64::from_raw(pct));
        }
    }

    #[test]
    fn huge_targets_take_the_percentage_and_out_of_range_points_are_rejected() {
        let mut t=TestWorld::new();let boss=t.body(p(500,0),2);
        t.world.write_storage::<omoba_core::CProperty>().get_mut(boss).unwrap().mhp=Fixed64::from_i32(10_000);
        let outcomes=t.cast(&AuroraGuardHandler,Target::Point(p(500,0)),1,&rank_json(ID,1)).unwrap();
        assert!(outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,magi,..} if *target==boss && *magi==Fixed64::from_raw(10_000*205))));
        assert!(t.cast(&AuroraGuardHandler,Target::Point(p(1051,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&AuroraGuardHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }
}
