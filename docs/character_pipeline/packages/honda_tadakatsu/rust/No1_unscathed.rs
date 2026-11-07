//! 毫髮無傷（本多忠勝，來源 A07W；行為依 history_nobu A07_Honda_Tadakatsu.lua
//! modifier_A07W:OnTakeDamage）：持續 duration 秒，受到的所有傷害無效，
//! 魔法與純粹傷害的 absorb_magical／absorb_pure 回復為生命（見 v3.lua）。
//! 自身 stat buff：`absolute_no_damage_*` 與 `absorb_{magical,pure}_as_heal` 由 host 在傷害
//! 結算（omoba-core damage_immunity）套用，權威端與 replica 相同。學習時放 rank 標記，
//! 讓戰國最強的 A07T_Transform 以目前等級施放。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec,TargetSelector};
use omoba_core::runtime::native::damage_immunity;
use omoba_template_ids::{ABILITY_A07_UNSCATHED,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

pub(crate) const ID:omoba_template_ids::AbilityId=ABILITY_A07_UNSCATHED;

pub(crate) fn buff_id(caster:EntityHandle)->String {format!("a07_unscathed:{}:{}",caster.id,caster.gen)}

#[derive(Clone,Copy,Debug)]
pub(crate) struct Guard {pub duration:Fixed64,pub physical:bool,pub magical:bool,pub pure:bool,pub absorb_magical:Fixed64,pub absorb_pure:Fixed64}

/// Content values of `rank` (1-based).
pub(crate) fn spec(rank:u8)->Result<Guard,&'static str> {
    let content=active_ability_const(ID).ok_or("missing unscathed content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid unscathed rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid unscathed rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing unscathed extra");
    let on=|v:Fixed64|v>=Fixed64::ONE;
    // Percent becomes a fraction of one, rounded (§5.3).
    let fraction=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    let spec=Guard {duration:extra("duration")?,physical:on(extra("physical_immunity")?),
        magical:on(extra("magical_immunity")?),pure:on(extra("pure_immunity")?),
        absorb_magical:fraction(extra("absorb_magical")?),absorb_pure:fraction(extra("absorb_pure")?)};
    if spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(30) {return Err("invalid unscathed duration");}
    if [spec.absorb_magical,spec.absorb_pure].iter().any(|v|*v<Fixed64::ZERO || *v>Fixed64::ONE) {return Err("invalid unscathed absorption");}
    Ok(spec)
}

pub(crate) fn payload(spec:&Guard)->String {
    let mut row=serde_json::Map::new();
    for (on,key) in [(spec.physical,StatKey::AbsoluteNoDamagePhysical),(spec.magical,StatKey::AbsoluteNoDamageMagical),(spec.pure,StatKey::AbsoluteNoDamagePure)] {
        if on {row.insert(key.as_str().into(),Fixed64::ONE.raw().into());}
    }
    if spec.absorb_magical>Fixed64::ZERO {row.insert(damage_immunity::ABSORB_MAGICAL.into(),spec.absorb_magical.raw().into());}
    if spec.absorb_pure>Fixed64::ZERO {row.insert(damage_immunity::ABSORB_PURE.into(),spec.absorb_pure.raw().into());}
    serde_json::Value::Object(row).to_string()
}

/// The guard on the caster; shared by the cast and 戰國最強's transform.
pub(crate) fn apply(world:&mut GameWorldDyn<'_>,caster:EntityHandle,spec:&Guard) {
    let payload=payload(spec);
    world.add_stat_buff(caster,buff_id(caster).as_str().into(),spec.duration,payload.as_str().into());
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Guard)->Result<(),&'static str> {
    if !matches!(target,Target::None) {return Err("unscathed has no target");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    apply(world,caster,&spec);
    Ok(())
}

pub struct UnscathedHandler;
impl AbilityScript for UnscathedHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let checked=serde_json::from_str::<AbilityLevelData>(json.as_str()).map_err(|_|"invalid unscathed metadata");
        match checked.and_then(|_|spec(level)).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {super::a07_ranks::learn(world,ID,caster,rank);}
}

pub fn a07_unscathed_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AbsoluteNoDamagePhysical.as_str().into(),1.0);
    modifiers.insert(StatKey::AbsoluteNoDamageMagical.as_str().into(),1.0);
    modifiers.insert(StatKey::AbsoluteNoDamagePure.as_str().into(),1.0);
    build_ability_ffi(ID,UnscathedHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A07_honda_tadakatsu::a07_test_support::*;
    use omoba_core::Outcome;
    use omoba_core::runtime::BuffStore;
    use specs::WorldExt;

    #[test]
    fn every_rank_guards_the_caster_for_its_duration() {
        for (rank,seconds) in [(1u8,1.25),(2,2.25),(3,3.25),(4,4.25)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&UnscathedHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let rows=buffs(&outcomes,&buff_id(t.handle(t.source)));
            assert_eq!(rows.len(),1,"rank {rank}");
            let (target,payload,duration)=&rows[0];
            assert_eq!((*target,*duration),(t.source,q(seconds)));
            for key in [StatKey::AbsoluteNoDamagePhysical,StatKey::AbsoluteNoDamageMagical,StatKey::AbsoluteNoDamagePure] {
                assert_eq!(payload[key.as_str()],1024);
            }
            assert_eq!(payload[damage_immunity::ABSORB_MAGICAL],307,"30%");
            assert_eq!(payload[damage_immunity::ABSORB_PURE],307);
        }
    }

    /// Through the real host settlement: every kind is ignored, 30% of magical and pure heals.
    #[test]
    fn host_settlement_ignores_all_damage_and_heals_30_percent_of_spell_damage() {
        let mut t=TestWorld::new();
        let enemy=t.body(p(100,0),2);
        t.world.write_storage::<omoba_core::CProperty>().get_mut(t.source).unwrap().hp=Fixed64::from_i32(500);
        let outcomes=t.cast(&UnscathedHandler,Target::None,4,&rank_json(ID,4)).unwrap();t.apply(outcomes);
        let me=t.source;
        let hit=|phys:i32,magi:i32,real:i32|Outcome::Damage {pos:p(0,0),phys:Fixed64::from_i32(phys),magi:Fixed64::from_i32(magi),
            real:Fixed64::from_i32(real),source:enemy,target:me,damage_profile:omb_script_abi::types::DamageProfile::NORMAL.bits(),predeclared:false};
        let hp=|t:&TestWorld|t.world.read_storage::<omoba_core::CProperty>().get(t.source).unwrap().hp;
        t.apply(vec![hit(300,0,0)]);assert_eq!(hp(&t),Fixed64::from_i32(500),"physical ignored, nothing healed");
        t.apply(vec![hit(0,200,0)]);assert_eq!(hp(&t),Fixed64::from_i32(500)+Fixed64::from_raw(200*307),"30% of magical heals");
        let before=hp(&t);
        t.apply(vec![hit(100,0,100)]);assert_eq!(hp(&t),before+Fixed64::from_raw(100*307),"pure: 30% heals, physical nothing");
        t.world.write_resource::<BuffStore>().tick(Fixed64::from_i32(5));
        let before=hp(&t);
        t.apply(vec![hit(100,0,0)]);assert_eq!(hp(&t),before-Fixed64::from_i32(100),"guard expired");
    }

    #[test]
    fn learning_marks_the_rank_for_the_transform() {
        let mut t=TestWorld::new();
        let first=t.learn(&UnscathedHandler,1);t.apply(first);
        let second=t.learn(&UnscathedHandler,3);t.apply(second);
        let b=t.world.read_resource::<BuffStore>();let h=t.handle(t.source);
        assert!(b.has(t.source,&super::super::a07_ranks::marker(ID,h,3)));
        assert!(!b.has(t.source,&super::super::a07_ranks::marker(ID,h,1)));
    }

    #[test]
    fn wrong_target_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        assert!(t.cast(&UnscathedHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&UnscathedHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&UnscathedHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&UnscathedHandler,Target::None,5,&rank_json(ID,1)).is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&UnscathedHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_history_cooldown_and_mana() {
        let content=active_ability_const(ID).unwrap();
        for (rank,mana) in [(1usize,100),(2,110),(3,120),(4,130)] {
            let level=&content.levels[rank-1];
            assert_eq!(level.cast_time,Fixed64::ZERO);assert_eq!(level.cooldown,Fixed64::from_i32(13));assert_eq!(level.mana_cost,Fixed64::from_i32(mana));
        }
    }
}
