//! Strict, bounded admission for instantaneous self transformations.
//! Resolve every field before emitting the single refreshable buff outcome.
use omb_script_abi::{stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::AbilityLevelData;
use omoba_template_ids::{active_ability_const,AbilityId};

/// `Percent`: authored in percent (200 = +200%), emitted as the raw fraction.
pub(crate) enum Encoding { Raw, Normalized, Count, Percent }
pub(crate) struct Field {
    pub key: &'static str,
    pub stat: StatKey,
    pub maximum: i32,
    pub encoding: Encoding,
}
pub(crate) struct Transform {
    duration: Fixed64,
    payload: String,
}

fn scalar(value:f64,maximum:i32,positive:bool)->Result<Fixed64,&'static str> {
    if !value.is_finite() || value<0.0 || value>f64::from(maximum)
        || (value>0.0 && value<1.0/1024.0) {
        return Err("invalid transform scalar");
    }
    let result=Fixed64::from_raw((value*1024.0).round() as i64);
    if positive && result<=Fixed64::ZERO {return Err("transform scalar below quantum");}
    Ok(result)
}

pub(crate) fn resolve(id:AbilityId,rank:u8,json:&str,fields:&[Field],visual:Option<&str>)
    ->Result<Transform,&'static str> {
    resolve_keyed(id,rank,json,fields,visual,"duration",false)
}

/// Same strict admission with an authored duration key and, when allowed, a
/// bounded cast windup (the host runs the windup before `execute`; the
/// transform itself stays instantaneous).
pub(crate) fn resolve_keyed(id:AbilityId,rank:u8,json:&str,fields:&[Field],visual:Option<&str>,
    duration_key:&str,allow_windup:bool)->Result<Transform,&'static str> {
    let content=active_ability_const(id).ok_or("missing transform content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid transform rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid transform rank");}
    if json.len()>65_536 || fields.is_empty() || fields.len()>16 {return Err("invalid transform metadata budget");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid transform metadata")?;
    scalar(f64::from(data.cooldown),86_400,false)?;
    scalar(f64::from(data.mana_cost),1_000_000,false)?;
    if data.range!=0.0 || !data.cast_time.is_finite() || !(0.0..=60.0).contains(&data.cast_time)
        || (!allow_windup && data.cast_time!=0.0) {return Err("transform requires instant self metadata");}
    let read=|key:&str,maximum:i32,positive:bool|->Result<Fixed64,&'static str> {
        if let Some(value)=data.extra.get(key) {
            return scalar(value.as_f64().ok_or("invalid transform extra type")?,maximum,positive);
        }
        let value=content.extras.iter().find(|(name,_)|*name==key)
            .and_then(|(_,values)|values.get(index)).copied().ok_or("missing transform extra")?;
        // Authored fixed-point values stay exact; fallback never indexes unchecked ranks.
        if value<Fixed64::ZERO || value>Fixed64::from_i32(maximum)
            || (positive && value==Fixed64::ZERO) {return Err("invalid authored transform scalar");}
        Ok(value)
    };
    let duration=read(duration_key,3_600,true)?;
    let mut payload=serde_json::Map::new();
    for field in fields {
        if matches!(field.encoding,Encoding::Count) && data.extra.get(field.key)
            .is_some_and(|value|value.as_f64().is_some_and(|value|value.fract()!=0.0)) {
            return Err("transform count must be a whole number");
        }
        let value=read(field.key,field.maximum,matches!(field.encoding,Encoding::Count))?;
        if matches!(field.encoding,Encoding::Count) && value.raw()%1024!=0 {
            return Err("transform count must be a whole number");
        }
        let encoded=match field.encoding {
            Encoding::Raw|Encoding::Count=>serde_json::json!(value.raw()),
            Encoding::Percent=>serde_json::json!(value.raw()/100),
            // These two legacy attack-stun consumers explicitly read normalized numbers.
            Encoding::Normalized=>serde_json::json!(value.raw() as f64/1024.0),
        };
        if payload.insert(field.stat.as_str().into(),encoded).is_some() {
            return Err("duplicate transform modifier");
        }
    }
    if let Some(visual)=visual {payload.insert("visual_effect".into(),visual.into());}
    Ok(Transform {duration,payload:serde_json::Value::Object(payload).to_string()})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,
    buff:&str,spec:Transform)->Result<(),&'static str> {
    if !matches!(target,Target::None) || !world.is_alive(caster)
        || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {
        return Err("transform requires a living self target");
    }
    world.add_stat_buff(caster,buff.into(),spec.duration,spec.payload.as_str().into());
    Ok(())
}
