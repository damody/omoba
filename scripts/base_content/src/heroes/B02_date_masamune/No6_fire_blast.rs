//! 火焰強襲（H02，來源 B04E current）：指定點圓形魔法傷害＋暈眩，不帶燃燒區。
//! 目標只有敵方英雄與一般單位（B04.txt:178-179 HERO | BASIC；B04.lua:96-104），不含建築。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_content_model::ControlEffectKind;
use omoba_template_ids::ABILITY_B04_FIRE_BLAST;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B04_FIRE_BLAST;

pub struct FireBlastHandler;
impl AbilityScript for FireBlastHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::point_blast::resolve(ID,level,json.as_str(),"damage","radius",
            Some((ControlEffectKind::Stun,"stun")),DamageKind::Magical,DamageProfile::FIRE.union(DamageProfile::SPELL))
            .map(crate::point_blast::PointBlast::without_buildings)
            // B04.lua:96-123: the stun lands before the damage.
            .map(crate::point_blast::PointBlast::control_first)
            .and_then(|spec|crate::point_blast::execute(world,caster,target,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn b04_fire_blast_ffi()->AbilityDefFFI {
    let effects=vec![EffectSpec::AreaEffect {
        radius:extra_at_id_f32(ID,"radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,
            amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::AllEnemies,modifier_type:"stun".into(),value:1.0,
                duration:Some(extra_at_id_f32(ID,"stun",1))}],
    }];
    build_ability_ffi(ID,FireBlastHandler,effects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B02_date_masamune::h02_test_support::*;
    use omb_script_abi::types::Fixed64;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_damages_and_stuns_inside_500_only() {
        for (rank,damage,stun) in [(1u8,150,0.5f64),(2,250,0.8),(3,350,1.1),(4,450,1.4)] {
            let mut t=TestWorld::new();
            let center=t.body(p(600,0),2);let edge=t.body(p(600,500),2);let outside=t.body(p(600,501),2);
            let ally=t.body(p(600,0),1);
            let outcomes=t.cast(&FireBlastHandler,Target::Point(p(600,0)),rank,&rank_json(ID,rank)).unwrap();
            let mut damaged:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
            damaged.sort_by_key(|(e,_)|e.id());
            assert_eq!(damaged,vec![(center,Fixed64::from_i32(damage)),(edge,Fixed64::from_i32(damage))],"rank {rank}");
            let stunned:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {target,buff_id,duration,..} if buff_id=="stun"=>Some((*target,*duration)),_=>None}).collect();
            let expected=Fixed64::from_raw((stun*1024.0).round() as i64);
            assert_eq!(stunned.len(),2,"rank {rank}");
            assert!(stunned.iter().all(|(e,d)|(*e==center||*e==edge) && *d==expected),"rank {rank}: {stunned:?}");
            assert!(!damaged.iter().any(|(e,_)|*e==outside||*e==ally));
        }
    }

    #[test]
    fn enemy_buildings_inside_the_blast_are_never_damaged_or_stunned() {
        let mut t=TestWorld::new();
        let tower=t.building(p(600,0),2);let enemy=t.body(p(600,100),2);
        let outcomes=t.cast(&FireBlastHandler,Target::Point(p(600,0)),1,&rank_json(ID,1)).unwrap();
        let touched=|e:specs::Entity|outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,..}|Outcome::AddBuff {target,..} if *target==e));
        assert!(touched(enemy),"heroes and basic units are hit");
        assert!(!touched(tower),"source HERO | BASIC excludes buildings");
    }

    #[test]
    fn out_of_range_or_wrong_target_is_rejected_without_outcomes() {
        let mut t=TestWorld::new();t.body(p(1000,0),2);
        assert!(t.cast(&FireBlastHandler,Target::Point(p(1000,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FireBlastHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FireBlastHandler,Target::Point(p(300,0)),0,&rank_json(ID,1)).is_err());
    }
}
