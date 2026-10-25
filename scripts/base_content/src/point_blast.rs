//! Reusable instantaneous point-circle damage/control with complete admission
//! before outcomes. This does not add a persistent zone or bypass host cast gates.
use abi_stable::std_types::RSome;
use omb_script_abi::{types::{EntityHandle,Fixed64,Vec2,Target,DamageKind,DamageProfile},world::GameWorldDyn,buff_ids::BuffId};
use omoba_core::ability_meta::AbilityLevelData;
use omoba_template_ids::{AbilityId,active_ability_const};
use omoba_content_model::ControlEffectKind;

pub(crate) struct PointBlast {
    pub range:Fixed64,pub radius:Fixed64,pub damage:Fixed64,
    pub kind:DamageKind,pub profile:DamageProfile,pub control:Option<(ControlEffectKind,Fixed64)>,
    pub zone:Option<(omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec,Fixed64,AbilityId)>,
}

/// Host-supplied rank JSON is strict; omitted extras may use the same authored
/// rank constant, but a present invalid value must never silently fall back.
pub(crate) fn resolve(id:AbilityId,rank:u8,json:&str,damage_key:&str,radius_key:&str,
    control:Option<(ControlEffectKind,&str)>,kind:DamageKind,profile:DamageProfile)
    ->Result<PointBlast,&'static str> {
    let content=active_ability_const(id).ok_or("missing point blast content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid point blast rank")?);
    if index>=content.levels.len() {return Err("invalid point blast rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid point blast metadata")?;
    let fixed=|value:f64,positive:bool,max:f64| {
        if !value.is_finite() || value<0.0 || value>max || (value>0.0 && value<1.0/1024.0) {return Err("invalid point blast scalar");}
        let value=Fixed64::from_raw((value*1024.0).round() as i64);
        if positive && value<=Fixed64::ZERO {return Err("point blast scalar below quantum");}
        Ok(value)
    };
    let read=|key:&str,positive:bool,max:f64|->Result<Fixed64,&'static str> {
        if let Some(value)=data.extra.get(key) {
            return fixed(value.as_f64().ok_or("invalid point blast extra type")?,positive,max);
        }
        let value=content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,values)|values.get(index))
            .copied().ok_or("missing point blast extra")?;
        fixed(f64::from(value.to_f32_for_render()),positive,max)
    };
    Ok(PointBlast {range:fixed(f64::from(data.range),true,10_000.0)?,
        radius:read(radius_key,true,10_000.0)?,damage:read(damage_key,false,1_000_000.0)?,
        control:control.map(|(kind,key)|read(key,true,60.0).map(|duration|(kind,duration))).transpose()?,kind,profile,zone:None})
}

/// Optional reusable post-blast zone. Complete validation precedes execute outputs.
pub(crate) fn with_zone(id:AbilityId,rank:u8,json:&str,mut blast:PointBlast,
    amount_key:&str,interval_key:&str,duration_key:&str)->Result<PointBlast,&'static str> {
    use omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec;
    let content=active_ability_const(id).ok_or("missing zone content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid zone rank")?);
    if index>=content.levels.len() {return Err("invalid zone rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid zone metadata")?;
    if let Some(policy)=data.extra.get("zone_lifetime") {
        let expected=if content.zone_independent {"independent"}else{"caster_bound"};
        if policy.as_str()!=Some(expected){return Err("zone lifetime differs from authored policy");}
    }
    let read=|key:&str,max:f64|->Result<Fixed64,&'static str> {
        let value=if let Some(value)=data.extra.get(key) {value.as_f64().ok_or("invalid zone extra type")?}
            else {f64::from(content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,values)|values.get(index))
                .copied().ok_or("missing zone extra")?.to_f32_for_render())};
        if !value.is_finite() || value<1.0/1024.0 || value>max {return Err("invalid zone scalar");}
        Ok(Fixed64::from_raw((value*1024.0).round() as i64))
    };
    let duration=read(duration_key,30.0)?;
    let zone=DamageZoneSpec {independent:content.zone_independent,center:Vec2::ZERO,radius:blast.radius,amount:read(amount_key,1_000_000.0)?,
        interval:read(interval_key,10.0)?,kind:blast.kind,profile:blast.profile.bits()};
    if !zone.valid() || duration<zone.interval {return Err("invalid zone lifetime or interval");}
    blast.zone=Some((zone,duration,id));Ok(blast)
}

fn supported(p:Vec2)->bool {
    let max=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
    p.x.raw().unsigned_abs()<=max && p.y.raw().unsigned_abs()<=max
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:PointBlast)
    ->Result<(),&'static str> {
    let Target::Point(center)=target else {return Err("point blast requires a ground point");};
    if spec.range<=Fixed64::ZERO || spec.range>Fixed64::from_i32(10_000)
        || spec.radius<=Fixed64::ZERO || spec.radius>Fixed64::from_i32(10_000)
        || spec.damage<Fixed64::ZERO || spec.damage>Fixed64::from_i32(1_000_000)
        || spec.zone.is_some_and(|(zone,duration,_)|!zone.valid() || duration<zone.interval || duration>Fixed64::from_i32(30))
        || spec.control.is_some_and(|(_,time)|time<=Fixed64::ZERO || time>Fixed64::from_i32(60))
        || !supported(center) || !world.is_alive(caster)
        || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {
        return Err("invalid point blast source or budget");
    }
    let origin=world.get_pos(caster).into_option().filter(|p|supported(*p)).ok_or("missing point blast origin")?;
    if !omoba_sim::terrain::point_in_swept_circle(center,origin,origin,spec.range) {return Err("point blast outside cast range");}
    let mut hits:Vec<_>=world.query_enemies_in_range(center,spec.radius,caster).into_iter()
        .filter(|target|*target!=caster && world.is_alive(*target))
        .filter(|target|world.get_hp(*target).into_option().is_some_and(|hp|hp>Fixed64::ZERO))
        .filter(|target|world.get_pos(*target).into_option().is_some_and(|position|
            omoba_sim::terrain::point_in_swept_circle(position,center,center,spec.radius))).collect();
    hits.sort_by_key(|entity|(entity.id,entity.gen));hits.dedup();
    // No outcomes are emitted until every budget/admission/geometry check ends.
    if let Some(zone)=spec.zone.filter(|(zone,_,_)|zone.independent) {emit_zone(world,caster,center,zone);}
    world.emit_explosion(center,spec.radius,Fixed64::from_raw(204));
    for target in hits {
        world.deal_damage(target,spec.damage,spec.kind,spec.profile,RSome(caster));
        if let Some((kind,duration))=spec.control {
            let buff=match kind {ControlEffectKind::Stun=>BuffId::Stun,ControlEffectKind::Root=>BuffId::Root,ControlEffectKind::Silence=>BuffId::Silence};
            world.add_buff(target,buff.as_rstr(),duration);
        }
    }
    if let Some(zone)=spec.zone.filter(|(zone,_,_)|!zone.independent) {emit_zone(world,caster,center,zone);}
    Ok(())
}

fn emit_zone(world:&mut GameWorldDyn<'_>,caster:EntityHandle,center:Vec2,
    (mut zone,duration,id):(omoba_core::runtime::ability_runtime::damage_zone::DamageZoneSpec,Fixed64,AbilityId)) {
    zone.center=center;
    let key=format!("__ground_damage_zone:{}:{}:{}",id.as_str(),caster.id,caster.gen);
    let payload=zone.payload().to_string();world.add_stat_buff(caster,key.as_str().into(),duration,payload.as_str().into());
}

