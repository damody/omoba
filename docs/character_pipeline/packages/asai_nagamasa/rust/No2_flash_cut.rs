//! 一刀．閃瞬（H06，來源 B08E current）：瞬移到指定敵人身旁並斬擊，造成
//! 100/200/300/400 魔法傷害、暈眩 0.3 秒，並燒掉 100/160/220/280 法力。
//! 瞬移不受路徑阻擋；落點在目標朝施法者方向 LAND 處，面向目標。該點不可走（目標貼牆）時，
//! host blink_landing 改落在目標周圍同距離、最近的可走點；都不可走就留在原地。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::{blink_landing,mana_drain};
use omoba_template_ids::{ABILITY_B08_FLASH_CUT,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B08_FLASH_CUT;
/// Landing distance from the target, inside melee attack range.
const LAND:Fixed64=Fixed64::from_raw(100*1024);

pub(crate) struct FlashCut {pub range:Fixed64,pub damage:Fixed64,pub stun:Fixed64,pub burn:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<FlashCut,&'static str> {
    let content=active_ability_const(ID).ok_or("missing flash cut content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid flash cut rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid flash cut rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid flash cut metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid flash cut range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing flash cut extra");
    Ok(FlashCut {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),damage:extra("damage_amount")?,
        stun:extra("control_duration")?,burn:extra("mana_drain_amount")?})
}

/// Point LAND before `target` on the segment from `origin`; the target itself
/// when they already stand closer than that.
fn landing(origin:Vec2,target:Vec2)->Vec2 {
    let gap=(origin-target).length();
    if gap<=LAND {return origin;}
    omoba_sim::navigation::step_toward(target,origin,LAND)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:FlashCut)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("flash cut requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    if victim==caster || !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || world.is_building(victim)
        || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("flash cut target is not a live enemy unit");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("flash cut target outside cast range");}
    let burn=mana_drain::Spec::checked(caster.id,caster.gen,spec.burn,spec.range).ok_or("invalid flash cut mana burn")?;
    // The host settles a walkable landing (terrain is not visible to scripts) and faces the target.
    let land=blink_landing::Spec {source_id:caster.id,source_generation:caster.gen,anchor:at,preferred:landing(origin,at),
        cast_range:spec.range}.checked().ok_or("invalid flash cut landing")?;
    // All checks done; apply outcomes.
    world.add_stat_buff(caster,blink_landing::BUFF.into(),Fixed64::ONE,land.payload().to_string().as_str().into());
    world.deal_damage(victim,spec.damage,DamageKind::Magical,DamageProfile::SHARP,RSome(caster));
    world.add_buff(victim,BuffId::Stun.as_rstr(),spec.stun);
    world.add_stat_buff(victim,mana_drain::BUFF.into(),Fixed64::ONE,burn.payload().to_string().as_str().into());
    Ok(())
}

pub struct FlashCutHandler;
impl AbilityScript for FlashCutHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b08_flash_cut_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,FlashCutHandler,vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"control_duration",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn every_rank_blinks_beside_the_target_damages_stuns_and_burns_mana() {
        for (rank,damage,burn) in [(1u8,100,100),(2,200,160),(3,300,220),(4,400,280)] {
            let mut t=TestWorld::new();let enemy=t.body(p(1000,0),2);
            let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),rank,&rank_json(ID,rank)).unwrap();
            let land=landing_request(&outcomes).expect("blink landing packet");
            assert_eq!((land.preferred,land.anchor),(p(900,0),p(1000,0)),"rank {rank}: lands 100 before the target");
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,magi,..} if *target==enemy && *magi==Fixed64::from_i32(damage))));
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,buff_id,duration,..}
                if *target==enemy && buff_id=="stun" && *duration==Fixed64::from_raw(307))),"0.3 s stun");
            let drain=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {target,buff_id,duration,payload} if *target==enemy && buff_id==mana_drain::BUFF=>
                mana_drain::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("mana burn packet");
            assert_eq!(drain.amount,Fixed64::from_i32(burn),"rank {rank}");
        }
    }

    #[test]
    fn allies_buildings_and_out_of_range_targets_are_rejected() {
        let mut t=TestWorld::new();
        let ally=t.body(p(300,0),1);let tower=t.building(p(300,0),2);let far=t.body(p(1101,0),2);
        for target in [ally,tower,far] {assert!(t.cast(&FlashCutHandler,Target::Entity(t.handle(target)),1,&rank_json(ID,1)).is_err());}
        assert!(t.cast(&FlashCutHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn a_target_already_in_reach_keeps_the_caster_in_place() {
        let mut t=TestWorld::new();let enemy=t.body(p(60,0),2);
        let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        assert_eq!(landing_request(&outcomes).map(|land|land.preferred),Some(p(0,0)));
        t.apply(outcomes);
        assert_eq!(t.world.read_storage::<omoba_core::Pos>().get(t.source).unwrap().0,p(0,0));
    }

    #[test]
    fn a_target_hugging_a_wall_still_gets_a_legal_landing_beside_it() {
        // H06 W: the wall covers the caster's side of the target, so the preferred point is not walkable.
        let mut t=TestWorld::new();let enemy=t.body(p(1000,0),2);
        t.world.insert(omoba_core::BlockedRegions(vec![omoba_core::BlockedRegion {name:"wall".into(),points:vec![
            vek::Vec2::new(850.0,-60.0),vek::Vec2::new(960.0,-60.0),vek::Vec2::new(960.0,60.0),vek::Vec2::new(850.0,60.0)]}]));
        let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        let landed=t.world.read_storage::<omoba_core::Pos>().get(t.source).unwrap().0;
        assert_ne!(landed,p(900,0),"never inside the wall");
        assert!(blink_landing::point_walkable(&t.world.read_resource::<omoba_core::BlockedRegions>(),landed,Fixed64::from_i32(10)),
            "walkable for the caster's body");
        let gap=(landed-p(1000,0)).length();
        assert!(gap>=Fixed64::from_i32(99)&&gap<=Fixed64::from_i32(101),"beside the target, in melee reach");
        assert!(t.world.read_storage::<omoba_core::CProperty>().get(enemy).unwrap().hp<Fixed64::from_i32(1000),"the slash still lands");
    }
}
