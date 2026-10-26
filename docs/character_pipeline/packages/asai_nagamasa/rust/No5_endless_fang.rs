//! 一刀．連牙斷空（H06，來源 B08D current，D 欄 16 級大招）：對指定敵人連斬
//! 18 刀，每刀 100 魔法傷害（共 1800），總共暈眩 2.2 秒。結算為一次傷害＋
//! 一次暈眩；handler 與欄位無關，D 欄由 loadout 決定。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B08_ENDLESS_FANG,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B08_ENDLESS_FANG;

pub(crate) struct Fang {pub range:Fixed64,pub damage:Fixed64,pub stun:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Fang,&'static str> {
    let content=active_ability_const(ID).ok_or("missing endless fang content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid endless fang rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid endless fang rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid endless fang metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid endless fang range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing endless fang extra");
    Ok(Fang {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),damage:extra("damage_amount")?,stun:extra("control_duration")?})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Fang)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("endless fang requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    if victim==caster || world.is_building(victim) || !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("endless fang target is not a live enemy unit");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("endless fang target outside cast range");}
    world.deal_damage(victim,spec.damage,DamageKind::Magical,DamageProfile::SHARP,RSome(caster));
    world.add_buff(victim,BuffId::Stun.as_rstr(),spec.stun);
    Ok(())
}

pub struct EndlessFangHandler;
impl AbilityScript for EndlessFangHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b08_endless_fang_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,EndlessFangHandler,vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"control_duration",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn content_learns_at_sixteen_and_the_cut_deals_1800_and_stuns() {
        let content=active_ability_const(ID).unwrap();
        assert_eq!((content.max_level,content.levels[0].required_hero_level),(1,16));
        let mut t=TestWorld::new();let enemy=t.body(p(800,0),2);
        let outcomes=t.cast(&EndlessFangHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        assert!(outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,magi,..} if *target==enemy && *magi==Fixed64::from_i32(1800))));
        assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,buff_id,duration,..}
            if *target==enemy && buff_id=="stun" && *duration==Fixed64::from_raw((2.2f64*1024.0).round() as i64))));
        let ally=t.body(p(100,0),1);let far=t.body(p(1101,0),2);
        for target in [ally,far] {assert!(t.cast(&EndlessFangHandler,Target::Entity(t.handle(target)),1,&rank_json(ID,1)).is_err());}
    }
}
