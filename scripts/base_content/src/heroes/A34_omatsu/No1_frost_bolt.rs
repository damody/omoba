//! 寒霜彈（H03，來源 A34W current）：對敵人或敵方建築投出追蹤寒霜彈（速度 2000），
//! 命中時造成魔法傷害並結凍（暈眩）。傷害在命中時才結算；目標失效則消失。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_projectile as host;
use omoba_content_model::ControlEffectKind;
use omoba_template_ids::{ABILITY_A34_FROST_BOLT,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A34_FROST_BOLT;

pub(crate) struct FrostBolt {pub range:Fixed64,pub speed:Fixed64,pub radius:Fixed64,pub lifetime:Fixed64,pub damage:Fixed64,pub stun:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<FrostBolt,&'static str> {
    let content=active_ability_const(ID).ok_or("missing frost bolt content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid frost bolt rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid frost bolt rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid frost bolt metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid frost bolt range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing frost bolt extra");
    Ok(FrostBolt {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),
        speed:extra("projectile_speed")?,radius:extra("projectile_radius")?,lifetime:extra("projectile_lifetime")?,
        damage:extra("damage_amount")?,stun:extra("control_duration")?})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:FrostBolt)->Result<(),&'static str> {
    let Target::Entity(victim)=target else {return Err("frost bolt requires a unit target");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let at=world.get_pos(victim).into_option().ok_or("missing target position")?;
    if victim==caster || !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
        || !world.query_enemies_in_range(at,Fixed64::ZERO,caster).iter().any(|e|*e==victim) {return Err("frost bolt target is not a live enemy");}
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("frost bolt target outside cast range");}
    let envelope=host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),
        motion:host::Motion::Homing {target_id:victim.id,target_generation:victim.gen},
        speed:spec.speed,radius:spec.radius,distance:Fixed64::ZERO,lifetime:spec.lifetime,max_hits:1,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::COLD.bits(),slow:None,
        control:Some((ControlEffectKind::Stun,spec.stun)),dot:None,
        // Source: 對著敵人或建築物投出寒霜彈.
        buildings:true,forced:None,
    };
    if !envelope.valid() {return Err("invalid frost bolt projectile envelope");}
    world.add_stat_buff(caster,host::BUFF.into(),spec.lifetime,envelope.payload().to_string().as_str().into());
    Ok(())
}

pub struct FrostBoltHandler;
impl AbilityScript for FrostBoltHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a34_frost_bolt_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,FrostBoltHandler,vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"projectile_speed",1),on_hit:vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"control_duration",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A34_omatsu::a34_test_support::*;
    use omoba_core::Outcome;

    fn envelope(outcomes:&[Outcome])->Option<host::Spec> {
        outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>
            host::Spec::from_payload(host::BUFF,*duration,payload),_=>None})
    }

    #[test]
    fn every_rank_launches_a_homing_bolt_with_rank_damage_and_freeze() {
        for (rank,damage,stun) in [(1u8,100,1.0f64),(2,200,1.5),(3,300,2.0),(4,400,2.5)] {
            let mut t=TestWorld::new();let enemy=t.body(p(400,0),2);
            let outcomes=t.cast(&FrostBoltHandler,Target::Entity(t.handle(enemy)),rank,&rank_json(ID,rank)).unwrap();
            let spec=envelope(&outcomes).expect("launch");
            assert_eq!(spec.amount,Fixed64::from_i32(damage),"rank {rank}");
            assert_eq!(spec.control,Some((ControlEffectKind::Stun,Fixed64::from_raw((stun*1024.0).round() as i64))),"rank {rank}");
            assert_eq!(spec.speed,Fixed64::from_i32(2000));
            assert!(spec.buildings && matches!(spec.motion,host::Motion::Homing {..}));
            assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {..})),"damage only on arrival");
        }
    }

    #[test]
    fn buildings_are_valid_targets_but_allies_and_out_of_range_are_not() {
        let mut t=TestWorld::new();
        let tower=t.building(p(500,0),2);let ally=t.body(p(300,0),1);let far=t.body(p(601,0),2);
        assert!(envelope(&t.cast(&FrostBoltHandler,Target::Entity(t.handle(tower)),1,&rank_json(ID,1)).unwrap()).is_some());
        assert!(t.cast(&FrostBoltHandler,Target::Entity(t.handle(ally)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FrostBoltHandler,Target::Entity(t.handle(far)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FrostBoltHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
    }
}
