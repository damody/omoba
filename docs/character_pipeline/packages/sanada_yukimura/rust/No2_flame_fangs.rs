//! 連牙炎突（真田幸村，來源 B06E；history_nobu B06_Sanada_Yukimura.lua b06e_start／
//! b06e_Shot）：對施法距離內的敵方英雄或一般單位發出追蹤炎彈（projectile_speed）。每次命中造成
//! damage 魔法傷害並暈眩 stun 秒，然後從命中目標 bounce_radius 內隨機彈向
//! 另一個可見的敵人（只排除當下目標），總共最多命中 hits 次（數值見 v3.lua）。彈跳由 host 的通用 bounce
//! chain（ability_projectile::with_bounce）處理，命中時才結算。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_projectile as host;
use omoba_content_model::ControlEffectKind;
use omoba_template_ids::{ABILITY_B06_FLAME_FANGS,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B06_FLAME_FANGS;

pub(crate) struct Fangs {pub range:Fixed64,pub speed:Fixed64,pub radius:Fixed64,pub lifetime:Fixed64,pub damage:Fixed64,pub stun:Fixed64,
    pub hits:u16,pub bounce_radius:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Fangs,&'static str> {
    let content=active_ability_const(ID).ok_or("missing flame fangs content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid flame fangs rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid flame fangs rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid flame fangs metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid flame fangs range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing flame fangs extra");
    let hits=u16::try_from(extra("hits")?.raw()/1024).map_err(|_|"invalid flame fangs bounce")?;
    Ok(Fangs {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),
        speed:extra("projectile_speed")?,radius:extra("projectile_radius")?,lifetime:extra("projectile_lifetime")?,
        damage:extra("damage")?,stun:extra("stun")?,hits,bounce_radius:extra("bounce_radius")?})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Fangs)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("flame fangs requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    if victim==caster || !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || world.is_building(victim)
        || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("flame fangs target is not a live enemy unit");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("flame fangs target outside cast range");}
    let envelope=host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),
        motion:host::Motion::Homing {target_id:victim.id,target_generation:victim.gen},
        speed:spec.speed,radius:spec.radius,distance:Fixed64::ZERO,lifetime:spec.lifetime,max_hits:1,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::FIRE.union(DamageProfile::SPELL).bits(),slow:None,
        control:Some((ControlEffectKind::Stun,spec.stun)),dot:None,buildings:false,forced:None,
    };
    if !envelope.valid() {return Err("invalid flame fangs projectile envelope");}
    let payload=host::with_bounce(envelope.payload(),spec.hits,spec.bounce_radius);
    if host::bounce_of(&payload).is_none() {return Err("invalid flame fangs bounce");}
    world.add_stat_buff(caster,host::BUFF.into(),spec.lifetime,payload.to_string().as_str().into());
    Ok(())
}

pub struct FlameFangsHandler;
impl AbilityScript for FlameFangsHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b06_flame_fangs_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,FlameFangsHandler,vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"projectile_speed",1),on_hit:vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"stun",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B06_sanada_yukimura::b06_test_support::*;
    use omoba_core::Outcome;

    fn launched(outcomes:&[Outcome])->Option<(host::Spec,Option<(u16,Fixed64)>)> {
        outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>
            host::Spec::from_payload(host::BUFF,*duration,payload).map(|s|(s,host::bounce_of(payload))),_=>None})
    }

    #[test]
    fn every_rank_launches_a_bouncing_homing_flame_with_rank_damage_and_stun() {
        for (rank,damage,stun) in [(1u8,100,0.5f64),(2,180,0.8),(3,260,1.1),(4,340,1.4)] {
            let mut t=TestWorld::new();let enemy=t.body(p(200,0),2);
            let outcomes=t.cast(&FlameFangsHandler,Target::Entity(t.handle(enemy)),rank,&rank_json(ID,rank)).unwrap();
            let (spec,bounce)=launched(&outcomes).expect("one projectile launch");
            assert_eq!(spec.motion,host::Motion::Homing {target_id:enemy.id(),target_generation:enemy.gen().id() as u32});
            assert_eq!((spec.speed,spec.amount,spec.kind),(Fixed64::from_i32(900),Fixed64::from_i32(damage),DamageKind::Magical),"rank {rank}");
            assert_eq!(spec.control,Some((ControlEffectKind::Stun,q(stun))));
            assert!(!spec.buildings);
            assert_eq!(bounce,Some((5,Fixed64::from_i32(500))),"five hits, 500 bounce radius");
        }
    }

    #[test]
    fn allies_buildings_out_of_range_and_bad_targets_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();
        let ally=t.body(p(100,0),1);let tower=t.building(p(100,0),2);let far=t.body(p(251,0),2);let edge=t.body(p(250,0),2);
        for e in [ally,tower,far] {assert!(t.cast(&FlameFangsHandler,Target::Entity(t.handle(e)),1,&rank_json(ID,1)).is_err());}
        assert!(t.cast(&FlameFangsHandler,Target::Entity(t.handle(edge)),1,&rank_json(ID,1)).is_ok(),"250 is in range");
        assert!(t.cast(&FlameFangsHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FlameFangsHandler,Target::Entity(t.handle(edge)),0,&rank_json(ID,1)).is_err());
        let dead=t.body(p(100,0),2);t.kill(dead);
        assert!(t.cast(&FlameFangsHandler,Target::Entity(t.handle(dead)),1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_history_cast_point_cooldown_mana_and_range() {
        let content=active_ability_const(ID).unwrap();
        for (rank,mana) in [(1usize,120),(2,140),(3,160),(4,180)] {
            let level=&content.levels[rank-1];
            assert_eq!(level.cast_time,q(0.25));assert_eq!(level.cooldown,Fixed64::from_i32(10));
            assert_eq!(level.mana_cost,Fixed64::from_i32(mana));assert_eq!(level.range,Fixed64::from_i32(250));
        }
    }
}
