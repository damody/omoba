//! 融影術（H05，來源 A02T current）：20/22/24 秒隱形（攻擊不現形）、攻速 ×2
//! （基礎攻擊間隔 1.8 → 0.9）、移速 +60，並且無單位碰撞。隱形與無碰撞都是 host
//! 通用狀態：`invisibility_level` buff 與 `no_unit_collision` 狀態 buff。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_runtime::NO_UNIT_COLLISION;
use omoba_template_ids::{ABILITY_A02_SHADOW_MELD,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A02_SHADOW_MELD;
pub(crate) const BUFF_ID:&str="a02_shadow_meld";
const DURATION:&str="stat_buff_duration";
const MOVE:&str="stat_buff_move_speed_bonus";
const SPEED:&str="stat_buff_attack_speed_multiplier";
const STEALTH:&str="stat_buff_invisibility_level";

pub(crate) fn resolve(rank:u8)->Option<(String,Fixed64)> {
    let content=active_ability_const(ID)?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return None;}
    let (duration,mv,speed,stealth)=(extra_at_id(ID,DURATION,rank),extra_at_id(ID,MOVE,rank),extra_at_id(ID,SPEED,rank),extra_at_id(ID,STEALTH,rank));
    if duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60) || mv<Fixed64::ZERO || speed<=Fixed64::ZERO
        || stealth<Fixed64::ONE || stealth.raw()%1024!=0 {return None;}
    Some((serde_json::json!({(StatKey::InvisibilityLevel.as_str()):stealth.raw(),(StatKey::AttackSpeedMultiplier.as_str()):speed.raw(),
        (StatKey::MoveSpeedBonusBuff.as_str()):mv.raw()}).to_string(),duration))
}

pub struct ShadowMeldHandler;
impl AbilityScript for ShadowMeldHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,_:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        if !matches!(target,Target::None) {return RErr("shadow meld is a self cast".into());}
        let Some((payload,duration))=resolve(level) else {return RErr("invalid shadow meld rank".into());};
        if !world.is_alive(caster) {return RErr("caster is not alive".into());}
        world.add_stat_buff(caster,RStr::from_str(BUFF_ID),duration,payload.as_str().into());
        world.add_buff(caster,RStr::from_str(NO_UNIT_COLLISION),duration);
        ROk(())
    }
}

pub fn a02_shadow_meld_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::MoveSpeedBonusBuff.as_str().into(),extra_at_id_f32(ID,MOVE,1));
    modifiers.insert(StatKey::AttackSpeedMultiplier.as_str().into(),extra_at_id_f32(ID,SPEED,1));
    build_ability_ffi(ID,ShadowMeldHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:Some(extra_at_id_f32(ID,DURATION,1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A02_inahime::a02_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn every_rank_grants_stealth_double_attack_speed_move_speed_and_phasing() {
        for (rank,seconds) in [(1u8,20),(2,22),(3,24)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&ShadowMeldHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let buff=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).expect("buff");
            assert_eq!(buff.1,Fixed64::from_i32(seconds),"rank {rank}");
            assert_eq!(buff.0[StatKey::InvisibilityLevel.as_str()].as_i64(),Some(1024));
            assert_eq!(buff.0[StatKey::AttackSpeedMultiplier.as_str()].as_i64(),Some(2048));
            assert_eq!(buff.0[StatKey::MoveSpeedBonusBuff.as_str()].as_i64(),Some(60*1024));
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,duration,..} if buff_id==NO_UNIT_COLLISION && *duration==Fixed64::from_i32(seconds))));
        }
    }

    #[test]
    fn stealth_level_counts_for_visibility_and_wrong_casts_are_rejected() {
        let mut t=TestWorld::new();let source=t.source;
        t.world.write_resource::<omoba_core::runtime::BuffStore>().add(source,BUFF_ID,Fixed64::from_i32(5),
            serde_json::from_str(&resolve(1).unwrap().0).unwrap());
        assert_eq!(t.world.read_resource::<omoba_core::runtime::BuffStore>().stealth_level(source),1);
        assert!(t.cast(&ShadowMeldHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&ShadowMeldHandler,Target::None,4,"{}").is_err());
    }
}
