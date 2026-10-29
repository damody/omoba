//! 極光守護（H04，來源 C17T current，C17.txt:391-475；C17.lua:C17T:282-323）：施法前搖 1 秒後在
//! 指定地點 800 範圍結算，建築除外（309）：
//! - 敵方英雄與小兵受 650/700/750 魔法傷害。腳本讀不存在的 `hpx`（299）得 0，最大生命 %
//!   分支永不成立，依腳本實際行為只造成固定傷害（與 tooltip 不同）。
//! - 友軍（含自己）`SetHealth(目前生命 + 最大生命 × 40/45/50%)`（318-319）：host
//!   ally_area_restore 封包以 `direct`（不經治療倍率）與 `exclude_buildings` 依提交位置結算。
use abi_stable::std_types::{RErr,ROk,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ally_area_restore;
use omoba_template_ids::{ABILITY_C17_AURORA_GUARD,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_AURORA_GUARD;

pub(crate) struct Aurora {pub range:Fixed64,pub radius:Fixed64,pub damage:Fixed64,pub heal:Fixed64}

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
    Ok(Aurora {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),radius:extra("radius")?,
        damage:extra("damage")?,heal:fraction(extra("heal")?)})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Aurora)->Result<(),&'static str> {
    let Target::Point(at)=target else {return Err("aurora requires a point");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("aurora point outside cast range");}
    let restore=ally_area_restore::Spec {exclude_buildings:true,direct:true,
        ..ally_area_restore::Spec::new(caster.id,caster.gen,at,spec.radius,spec.heal,Fixed64::ZERO,spec.range)}.checked().ok_or("invalid aurora heal")?;
    let mut enemies:Vec<EntityHandle>=world.query_enemies_in_range(at,spec.radius,caster).into_iter()
        .filter(|e|*e!=caster && !world.is_building(*e) && world.is_alive(*e) && world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO)).collect();
    enemies.sort_by_key(|e|(e.id,e.gen));enemies.dedup();
    for e in enemies {
        // AMHC:Damage: nothing once Oichi has died (amhc.lua:540).
        world.deal_damage(e,spec.damage,DamageKind::Magical,DamageProfile::ENERGY.union(DamageProfile::REQUIRE_SOURCE_ALIVE),RSome(caster));
    }
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
    build_ability_ffi(ID,AuroraGuardHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    #[test]
    fn every_rank_damages_enemies_by_the_flat_amount_and_sends_one_direct_ally_heal() {
        for (rank,flat,heal) in [(1u8,650,410),(2,700,461),(3,750,512)] {
            let mut t=TestWorld::new();
            let enemy=t.body(p(900,0),2);let far=t.body(p(900,801),2);let tower=t.building(p(900,10),2);
            let outcomes=t.cast(&AuroraGuardHandler,Target::Point(p(900,0)),rank,&rank_json(ID,rank)).unwrap();
            let damage:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
            assert_eq!(damage,vec![(enemy,Fixed64::from_i32(flat))],"rank {rank}");
            assert!(!damage.iter().any(|(e,_)|*e==far||*e==tower));
            let packet=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}=>
                ally_area_restore::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("ally heal packet");
            assert_eq!((packet.hp_fraction,packet.radius,packet.center),(Fixed64::from_raw(heal),Fixed64::from_i32(800),p(900,0)),"rank {rank}");
            assert!(packet.exclude_buildings && packet.direct,"SetHealth on heroes and units only");
        }
    }

    /// 原作 hpx 不存在 → 最大生命 % 分支永不成立：巨大生命的目標也只受固定傷害。
    #[test]
    fn huge_targets_take_only_the_flat_amount_and_out_of_range_points_are_rejected() {
        let mut t=TestWorld::new();let boss=t.body(p(500,0),2);
        t.world.write_storage::<omoba_core::CProperty>().get_mut(boss).unwrap().mhp=Fixed64::from_i32(10_000);
        let outcomes=t.cast(&AuroraGuardHandler,Target::Point(p(500,0)),1,&rank_json(ID,1)).unwrap();
        assert!(outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {target,magi,..} if *target==boss && *magi==Fixed64::from_i32(650))));
        assert!(t.cast(&AuroraGuardHandler,Target::Point(p(1051,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&AuroraGuardHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    /// SetHealth：不經治療倍率（受治療減少的友軍也回滿比例），友方建築不回復。
    #[test]
    fn allies_are_set_directly_ignoring_healing_reduction_and_buildings_are_skipped() {
        let mut t=TestWorld::new();let ally=t.body(p(300,0),1);let tower=t.building(p(300,50),1);
        for e in [ally,tower,t.source] {t.world.write_storage::<omoba_core::CProperty>().get_mut(e).unwrap().hp=Fixed64::from_i32(100);}
        t.world.write_resource::<omoba_core::runtime::BuffStore>().grant_healing_reduction(ally,Fixed64::from_raw(512),Fixed64::from_i32(10));
        let outcomes=t.cast(&AuroraGuardHandler,Target::Point(p(300,0)),1,&rank_json(ID,1)).unwrap();
        t.apply(outcomes);
        let hp=|e:specs::Entity|t.world.read_storage::<omoba_core::CProperty>().get(e).unwrap().hp;
        assert_eq!(hp(ally),Fixed64::from_raw(100*1024+1000*410),"100 + 40% of 1000, healing reduction ignored");
        assert_eq!(hp(t.source),Fixed64::from_raw(100*1024+1000*410),"the caster too");
        assert_eq!(hp(tower),Fixed64::from_i32(100),"allied buildings are not restored");
    }
}
