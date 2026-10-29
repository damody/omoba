//! 蠍毒箭（H05，來源 A02W current，A02.lua:3-69）：切換 G-2 attack orb。handler 只切換
//! 施法者身上的標記 buff；每次普攻開始時扣魔、沒藍自動關閉、目標是建築時不扣魔也不附加、
//! 命中加傷、移速減益（%）與攻速減益（攻速點，多來源取最強）都由 host 依當前 rank 從
//! content `attack_orb` 結算。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::attack_orb;
use omoba_template_ids::{ABILITY_A02_VENOM_ARROW,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A02_VENOM_ARROW;

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,rank:u8)->Result<(),&'static str> {
    let content=active_ability_const(ID).ok_or("missing venom arrow content")?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return Err("invalid venom arrow rank");}
    if content.attack_orb_at(rank).is_none() {return Err("venom arrow content has no attack orb");}
    if !matches!(target,Target::None) {return Err("venom arrow is a no-target toggle");}
    if !world.is_alive(caster) {return Err("caster is not alive");}
    let buff=attack_orb::buff_id(ID.as_str());
    if world.has_buff(caster,buff.as_str().into()) {
        world.remove_buff(caster,buff.as_str().into());
    } else {
        // Until toggled off; the host switches it off for mana or on death.
        let payload=attack_orb::payload(ID.as_str()).to_string();
        world.add_stat_buff(caster,buff.as_str().into(),Fixed64::from_raw(i64::from(i32::MAX)),payload.as_str().into());
    }
    Ok(())
}

pub struct VenomArrowHandler;
impl AbilityScript for VenomArrowHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,_:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match execute(world,caster,target,level) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a02_venom_arrow_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,VenomArrowHandler,vec![EffectSpec::Damage {target:TargetSelector::Target,
        amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A02_inahime::a02_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn content_carries_the_rank_orb_values() {
        for (rank,amount,slow) in [(1u8,30,0.09f64),(2,50,0.13),(3,70,0.17),(4,90,0.21)] {
            let v=active_ability_const(ID).unwrap().attack_orb_at(rank).expect("orb values");
            assert_eq!(v.mana_per_attack,Fixed64::from_i32(10),"rank {rank}");
            assert_eq!(v.amount,Fixed64::from_i32(amount),"rank {rank}");
            let q=Fixed64::from_raw((slow*1024.0).round() as i64);
            assert_eq!((v.move_slow,v.attack_slow),(Some(q),None),"rank {rank}: attack slow is points, not a percentage");
            // A02.txt:57 debuff_attack_speed: slow_self ATTACKSPEED_BONUS_CONSTANT points.
            let points=crate::ability_builder::extra_at_id(ID,attack_orb::ATTACK_SPEED_CONSTANT_KEY,rank);
            assert_eq!(points,Fixed64::from_raw((slow*100.0*1024.0).round() as i64),"rank {rank}");
            assert_eq!(v.slow_duration,Fixed64::from_i32(4));
        }
    }

    #[test]
    fn toggling_adds_then_removes_the_orb_marker() {
        let mut t=TestWorld::new();
        let on=t.cast(&VenomArrowHandler,Target::None,1,&rank_json(ID,1)).unwrap();
        let marker=attack_orb::buff_id(ID.as_str());
        assert!(on.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,payload,..}
            if *buff_id==marker && attack_orb::orb_ability(buff_id,payload)==Some(ID.as_str()))));
        let source=t.source;
        t.world.write_resource::<omoba_core::runtime::BuffStore>().add(source,&marker,Fixed64::from_i32(100),attack_orb::payload(ID.as_str()));
        let off=t.cast(&VenomArrowHandler,Target::None,1,&rank_json(ID,1)).unwrap();
        assert!(off.iter().any(|o|matches!(o,Outcome::ScriptRemoveBuff {buff_id,..} if *buff_id==marker)));
        assert!(t.cast(&VenomArrowHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&VenomArrowHandler,Target::None,5,&rank_json(ID,1)).is_err());
    }
}
