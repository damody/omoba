//! 一刀．閃瞬（H06，來源 B08E current，B08.txt:421-528；B08_Asai_Nagamasa.lua:B08E_Action:110-171）：
//! 瞬移到指定敵人所在位置並斬擊，造成 damage 魔法傷害、暈眩 stun 秒，燒掉
//! mana_burn 法力並追加「實際燒掉法力」點魔法傷害（163-169）。結尾自身 command_lock 秒
//! command_restricted（modifier_B08E2，170、499-507）。
//! 瞬移不受路徑阻擋：host blink_landing 以目標位置為落點（原作 SetAbsOrigin 直接站到
//! 目標點，118-119）；該點對施法者身體不可走時留在原地。面向目標。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},buff_ids::BuffId,
    types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::{blink_landing,command_restriction,mana_drain};
use omoba_template_ids::{ABILITY_B08_FLASH_CUT,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B08_FLASH_CUT;

pub(crate) struct FlashCut {pub range:Fixed64,pub damage:Fixed64,pub stun:Fixed64,pub burn:Fixed64,pub restrict:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<FlashCut,&'static str> {
    let content=active_ability_const(ID).ok_or("missing flash cut content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid flash cut rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid flash cut rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid flash cut metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid flash cut range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing flash cut extra");
    let restrict=extra("command_lock")?;
    if restrict<=Fixed64::ZERO || restrict>Fixed64::from_i32(5) {return Err("invalid flash cut command restriction");}
    Ok(FlashCut {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),damage:extra("damage")?,
        stun:extra("stun")?,burn:extra("mana_burn")?,restrict})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:FlashCut)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("flash cut requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    if victim==caster || !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || world.is_building(victim)
        || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("flash cut target is not a live enemy unit");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("flash cut target outside cast range");}
    // The burned mana is dealt again as magical damage by the host (only it knows the target's mana).
    let burn=mana_drain::Spec::checked(caster.id,caster.gen,spec.burn,spec.range).ok_or("invalid flash cut mana burn")?
        .with_damage(DamageKind::Magical);
    // The host settles a walkable landing (terrain is not visible to scripts) and faces the target.
    let land=blink_landing::Spec {source_id:caster.id,source_generation:caster.gen,anchor:at,preferred:at,
        cast_range:spec.range}.checked().ok_or("invalid flash cut landing")?;
    // All checks done; apply outcomes in source order (B08E_Action 111-170).
    // Source B08_Asai_Nagamasa.lua:111: B08E_Action opens with ProjectileManager:ProjectileDodge(caster).
    world.add_stat_buff(caster,omoba_core::runtime::native::ability_projectile::DODGE_BUFF.into(),Fixed64::ONE,
        omoba_core::runtime::native::ability_projectile::dodge_payload(caster.id,caster.gen).to_string().as_str().into());
    world.add_stat_buff(caster,blink_landing::BUFF.into(),Fixed64::ONE,land.payload().to_string().as_str().into());
    world.add_buff(victim,BuffId::Stun.as_rstr(),spec.stun);
    world.deal_damage(victim,spec.damage,DamageKind::Magical,DamageProfile::SHARP,RSome(caster));
    world.add_stat_buff(victim,mana_drain::BUFF.into(),Fixed64::ONE,burn.payload().to_string().as_str().into());
    world.add_stat_buff(caster,command_restriction::BUFF.into(),spec.restrict,"{}".into());
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
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"stun",1))}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn every_rank_blinks_onto_the_target_damages_stuns_burns_mana_and_restricts_commands() {
        for (rank,damage,burn) in [(1u8,100,100),(2,200,160),(3,300,220),(4,400,280)] {
            let mut t=TestWorld::new();let enemy=t.body(p(1000,0),2);
            let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),rank,&rank_json(ID,rank)).unwrap();
            let land=landing_request(&outcomes).expect("blink landing packet");
            assert_eq!((land.preferred,land.anchor),(p(1000,0),p(1000,0)),"rank {rank}: lands on the target's point");
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,magi,..} if *target==enemy && *magi==Fixed64::from_i32(damage))));
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,buff_id,duration,..}
                if *target==enemy && buff_id=="stun" && *duration==Fixed64::from_raw(307))),"0.3 s stun");
            let drain=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {target,buff_id,duration,payload} if *target==enemy && buff_id==mana_drain::BUFF=>
                mana_drain::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("mana burn packet");
            assert_eq!((drain.amount,drain.damage),(Fixed64::from_i32(burn),Some(DamageKind::Magical)),"rank {rank}");
            // B08_Asai_Nagamasa.lua:111: the cast first dodges projectiles tracking the caster.
            let dodge=omoba_core::runtime::native::ability_projectile::DODGE_BUFF;
            assert!(matches!(outcomes.first(),Some(Outcome::AddBuff {target,buff_id,..}) if *target==t.source && buff_id==dodge),"rank {rank}");
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,buff_id,duration,..}
                if *target==t.source && buff_id==command_restriction::BUFF && *duration==Fixed64::from_raw(102))),"0.1 s command restriction");
        }
        let content=active_ability_const(ID).unwrap();
        assert!(content.levels.iter().all(|l|l.cast_time==Fixed64::from_raw(102)),"cast point 0.1");
    }

    /// 原作 163-169：燒魔後另造成「實際燒掉的法力」點魔法傷害；法力不足時只燒剩下的。
    #[test]
    fn the_burned_mana_is_dealt_again_as_magical_damage() {
        for (mana,expected) in [(1000,280),(150,150),(0,0)] {
            let mut t=TestWorld::new();let enemy=t.body(p(500,0),2);
            t.world.write_storage::<omoba_core::Hero>().insert(enemy,omoba_core::Hero::new("fixture".into(),"fixture".into(),String::new())).unwrap();
            t.world.write_storage::<omoba_core::Hero>().get_mut(enemy).unwrap().mana_pool=
                Some(omoba_core::runtime::ability_runtime::ManaPool::new(Fixed64::from_i32(mana),Fixed64::from_i32(1000)).unwrap());
            let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),4,&rank_json(ID,4)).unwrap();
            t.apply(outcomes);
            let hp=t.world.read_storage::<omoba_core::CProperty>().get(enemy).unwrap().hp;
            assert_eq!(hp,Fixed64::from_i32(1000-400-expected),"mana {mana}: 400 slash + {expected} burn");
            let left=t.world.read_storage::<omoba_core::Hero>().get(enemy).unwrap().mana_pool.as_ref().unwrap().current();
            assert_eq!(left,Fixed64::from_i32(mana-expected));
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
    fn the_caster_lands_on_the_target_point() {
        let mut t=TestWorld::new();let enemy=t.body(p(60,0),2);
        let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        assert_eq!(t.world.read_storage::<omoba_core::Pos>().get(t.source).unwrap().0,p(60,0));
    }

    #[test]
    fn an_unwalkable_target_point_keeps_the_caster_in_place_but_the_slash_lands() {
        // H06 W: the caster's body (radius 50) would overlap the wall at the target's point.
        let mut t=TestWorld::new();let enemy=t.body(p(1000,0),2);
        t.world.write_storage::<omoba_core::CollisionRadius>().insert(t.source,omoba_core::CollisionRadius(Fixed64::from_i32(50))).unwrap();
        t.world.insert(omoba_core::BlockedRegions(vec![omoba_core::BlockedRegion {name:"wall".into(),points:vec![
            vek::Vec2::new(850.0,-60.0),vek::Vec2::new(960.0,-60.0),vek::Vec2::new(960.0,60.0),vek::Vec2::new(850.0,60.0)]}]));
        let outcomes=t.cast(&FlashCutHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        assert_eq!(t.world.read_storage::<omoba_core::Pos>().get(t.source).unwrap().0,p(0,0),"never inside the wall");
        assert!(t.world.read_storage::<omoba_core::CProperty>().get(enemy).unwrap().hp<Fixed64::from_i32(1000),"the slash still lands");
    }
}
