//! Shared directional fang (H06 Q 一刀．牙 and R 一刀．空牙): a piercing G-1
//! linear projectile whose flight distance is the cast range. Each hit deals
//! the rank's magical damage once; buildings are never hit.
use omb_script_abi::types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target};
use omb_script_abi::world::GameWorldDyn;
use omoba_core::ability_meta::AbilityLevelData;
use omoba_core::runtime::native::ability_projectile as host;
use omoba_template_ids::{AbilityId,active_ability_const};

pub(crate) struct Fang {pub distance:Fixed64,pub speed:Fixed64,pub radius:Fixed64,pub max_hits:u16,pub damage:Fixed64}

pub(crate) fn resolve(id:AbilityId,rank:u8,json:&str)->Result<Fang,&'static str> {
    let content=active_ability_const(id).ok_or("missing fang content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid fang rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid fang rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid fang metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid fang range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing fang extra");
    let hits=extra("projectile_max_hits")?;
    if hits.raw()%1024!=0 {return Err("fang hit budget must be whole");}
    Ok(Fang {distance:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),speed:extra("projectile_speed")?,
        radius:extra("projectile_radius")?,max_hits:u16::try_from(hits.raw()/1024).map_err(|_|"fang hit budget")?,damage:extra("damage_amount")?})
}

pub(crate) fn launch(world:&mut GameWorldDyn<'_>,id:AbilityId,caster:EntityHandle,target:Target,spec:Fang)->Result<(),&'static str> {
    let Target::Point(aim)=target else {return Err("fang requires a direction");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if aim==origin {return Err("fang needs a direction");}
    let lifetime=Fixed64::from_raw((i128::from(spec.distance.raw())*1024/i128::from(spec.speed.raw().max(1))) as i64)+Fixed64::ONE;
    let envelope=host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:id.as_str().to_owned(),motion:host::Motion::Linear {aim},
        speed:spec.speed,radius:spec.radius,distance:spec.distance,lifetime,max_hits:spec.max_hits,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::SHARP.bits(),slow:None,control:None,dot:None,buildings:false,forced:None,
    };
    if !envelope.valid() {return Err("invalid fang projectile envelope");}
    world.add_stat_buff(caster,host::BUFF.into(),lifetime,envelope.payload().to_string().as_str().into());
    Ok(())
}
