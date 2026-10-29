use std::collections::BTreeSet;
pub mod buff_visual_sources;
pub mod animation_metadata;
pub mod moba_items;
pub mod v3;
pub mod objective_comeback;
pub mod moba_summoners;
pub mod moba_trinkets;
pub mod moba_communication;
pub mod recall_rules;
pub mod attribute_rules;
pub mod game_rules;
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod aim_geometry_tests;
#[cfg(test)]
mod periodic_damage_tests;

/// Shared gameplay and presentation-facing ability data parsed from Lua.
/// Editor-only `ue` metadata stays with the Unreal generator.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AbilityDefinition {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub tombstone: bool,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub ability_type: String,
    #[serde(default)]
    pub cast_type: String,
    #[serde(default)]
    pub target_type: String,
    #[serde(default = "default_ability_max_level")]
    pub max_level: u8,
    #[serde(default)]
    pub levels: Vec<AbilityLevel>,
    #[serde(default)]
    pub extras: std::collections::BTreeMap<String, Vec<f32>>,
    #[serde(default)]
    pub effects: Vec<AbilityEffect>,
    /// Authored aiming contract for special Rust handlers; never inferred from cosmetics.
    #[serde(default)]
    pub aim_geometry: Option<AbilityAimGeometry>,
    /// Package `targeting.shape`: presentation-only footprint (never a gameplay contract).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targeting_shape: Option<AbilityTargetingShape>,
    /// Package movement reach: a range-0 point ability whose handler moves the
    /// caster at most `extras[key]` toward the requested point and acts where it
    /// lands (a dash's `max_distance`). Presentation-only: the indicator clamps its
    /// footprint there and autopilot reach adds it to the footprint. The cast
    /// itself stays range-free (a farther click still leaps the full distance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targeting_reach_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_lifetime: Option<GroundZoneLifetime>,
    /// Host-owned flight that delivers this ability's enemy unit effects on hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile: Option<ProjectileDelivery>,
    /// Stored uses (G-2). A cast spends one; one recharges every `recharge_key` seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charges: Option<AbilityCharges>,
    /// G-3 delivery: spawn one summon template at the requested ground point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summon: Option<AbilitySummonDelivery>,
    /// G-2 toggle orb: while toggled on, each ordinary attack spends mana and,
    /// on hit, adds the declared damage/slows. Auto-off when mana is short.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_orb: Option<AttackOrb>,
    /// G-2 pulse channel: host-owned, `effects` apply once per pulse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<ChannelDelivery>,
    /// Source NOT_LEARNABLE: each rank is learned automatically, without a skill point,
    /// as soon as the hero reaches its `required_hero_level`; players cannot spend points on it.
    /// Works in any slot (e.g. a level-16 extra-slot ultimate, a passive learned at birth).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto_learn: bool,
    /// Authored unit-target allegiance ("ally" or "enemy") for a unit-target
    /// ability whose effects do not imply one (a custom Rust handler).
    /// Declarative effects still decide when they do; a contradiction is invalid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_target: Option<String>,
    /// The source lets this ability target, control and debuff a magic-immune enemy (E179;
    /// Dota SPELL_IMMUNITY_ENEMIES_YES). Its piercing debuffs carry magic_immunity::PIERCE_KEY.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pierces_magic_immunity: bool,
    /// Owner cast hook (G-4): while learned, every committed cast of another
    /// non-ultimate ability of the same hero shortens the remaining cooldowns
    /// of the hero's other non-ultimate abilities. Host-owned, so team replicas
    /// recompute it from the same cast; independent of how the ability executes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cast_cooldown_refund: Option<CastCooldownRefund>,
    /// Package schema v3 block (values, info line, tooltip templates); `description`,
    /// `levels` and `extras` are generated from it and deprecated (v3-content-model-proposal.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v3: Option<v3::V3Block>,
}

/// Seconds (per rank, `seconds_key` extras in (0, 60]) taken off each other
/// non-ultimate cooldown when the owner casts a non-ultimate ability.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CastCooldownRefund {
    pub seconds_key: String,
}

fn validate_cast_cooldown_refund(ability:&AbilityDefinition,refund:&CastCooldownRefund)->Result<(),String> {
    let rows=ability.extras.get(&refund.seconds_key)
        .filter(|rows|!refund.seconds_key.is_empty()&&ability.max_level>0&&rows.len()==usize::from(ability.max_level))
        .ok_or_else(||format!("ability '{}': cast_cooldown_refund missing per-rank extras '{}'",ability.id,refund.seconds_key))?;
    if rows.iter().any(|v|!v.is_finite()||*v<MIN_ABILITY_SCALAR||*v>60.0) {
        return Err(format!("ability '{}': cast_cooldown_refund extras '{}' outside [{MIN_ABILITY_SCALAR},60]",ability.id,refund.seconds_key));
    }
    Ok(())
}

/// G-2 pulse channel. The caster stays in place for `pulses_key` pulses spaced
/// `interval_key` seconds apart. Each pulse applies `effects` (at most one
/// `damage`, one `slow_enemy`, one `stun_enemy`/`control_enemy`) to every enemy
/// within `radius_key` of the anchor; a `unit` anchor without a radius hits
/// only that unit. Control, silence, movement, death or a lost target end it.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChannelDelivery {
    pub anchor: ChannelAnchor,
    pub interval_key: String,
    pub pulses_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius_key: Option<String>,
}

/// `self` = caster origin (target none), `point` = cast point, `unit` = the
/// selected enemy, followed while it lives and stays visible.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelAnchor { #[serde(rename = "self")] SelfOrigin, Point, Unit }

/// Shortest pulse spacing: one tick at 10 Hz, the B29T authored cadence.
pub const MIN_CHANNEL_INTERVAL: f32 = 0.1;
pub const MAX_CHANNEL_PULSES: f32 = 200.0;
pub const MAX_CHANNEL_SECONDS: f32 = 60.0;

#[cfg(test)]
#[test]
fn channel_delivery_is_closed_bounded_and_matches_its_anchor() {
    let base=serde_json::json!({"id":"quake","ability_type":"ultimate","cast_type":"channeled","target_type":"point","max_level":3,
        "levels":[{"range":700,"cooldown":90,"mana_cost":200},{"range":700,"cooldown":80,"mana_cost":250},{"range":700,"cooldown":70,"mana_cost":300}],
        "channel":{"anchor":"point","interval_key":"wave_interval","pulses_key":"wave_count","radius_key":"wave_radius"},
        "extras":{"wave_interval":[0.1,0.1,0.1],"wave_count":[50,60,70],"wave_radius":[340,340,340],"wave_damage":[60,60,60],"wave_stun":[0.2,0.3,0.4]},
        "effects":[{"kind":"damage","amount_key":"wave_damage","damage_kind":"magical"},{"kind":"stun_enemy","duration_key":"wave_stun"}]});
    let parse=|v:&serde_json::Value|serde_json::from_value::<AbilityDefinition>(v.clone());
    validate_ability_effects(&parse(&base).unwrap()).unwrap();
    let mut unit=base.clone();unit["target_type"]="unit".into();unit["channel"]["anchor"]="unit".into();
    unit["channel"].as_object_mut().unwrap().remove("radius_key");validate_ability_effects(&parse(&unit).unwrap()).unwrap();
    let mut own=base.clone();own["target_type"]="none".into();own["channel"]["anchor"]="self".into();
    validate_ability_effects(&parse(&own).unwrap()).unwrap();
    let edits:Vec<(&str,serde_json::Value)>=vec![
        ("/cast_type",serde_json::json!("instant")),("/target_type",serde_json::json!("unit")),
        ("/extras/wave_interval",serde_json::json!([0.05,0.1,0.1])),("/extras/wave_count",serde_json::json!([50.5,60,70])),
        ("/extras/wave_count",serde_json::json!([201,60,70])),("/extras/wave_interval",serde_json::json!([1,1,1])),
        ("/extras/wave_radius",serde_json::json!([3000,340,340])),("/levels/0",serde_json::json!({"range":700,"cooldown":90,"mana_cost":200,"cast_time":61})),
        ("/effects/1",serde_json::json!({"kind":"heal_self","amount_key":"wave_damage"})),
    ];
    for (pointer,value) in edits {
        let mut bad=base.clone();*bad.pointer_mut(pointer).unwrap()=value;
        assert!(validate_ability_effects(&parse(&bad).unwrap()).is_err(),"{pointer}");
    }
    // A windup may precede the channel, and a Rust handler may own the pulses.
    let mut windup=base.clone();windup["levels"][0]["cast_time"]=0.3.into();validate_ability_effects(&parse(&windup).unwrap()).unwrap();
    let mut scripted=base.clone();scripted["effects"]=serde_json::json!([]);validate_ability_effects(&parse(&scripted).unwrap()).unwrap();
    let mut doubled=base.clone();doubled["effects"].as_array_mut().unwrap().push(serde_json::json!({"kind":"stun_enemy","duration_key":"wave_stun"}));
    assert!(validate_ability_effects(&parse(&doubled).unwrap()).is_err(),"two stuns");
    let mut no_radius=base.clone();no_radius["channel"].as_object_mut().unwrap().remove("radius_key");
    assert!(validate_ability_effects(&parse(&no_radius).unwrap()).is_err());
    let mut unknown=base.clone();unknown["channel"]["refund"]=true.into();assert!(parse(&unknown).is_err());
    let mut legacy=base;legacy.as_object_mut().unwrap().remove("channel");
    assert!(serde_json::to_value(parse(&legacy).unwrap()).unwrap().get("channel").is_none());
}

fn validate_channel_delivery(ability:&AbilityDefinition,delivery:&ChannelDelivery)->Result<(),String> {
    let fail=|why:&str|Err(format!("ability '{}': channel {why}",ability.id));
    if !matches!(ability.ability_type.as_str(),"active"|"ultimate") || ability.cast_type!="channeled" {
        return fail("requires a channeled active/ultimate");
    }
    if ability.aim_geometry.is_some() || ability.zone_lifetime.is_some() || ability.projectile.is_some()
        || ability.summon.is_some() || ability.attack_orb.is_some() || ability.charges.is_some() {
        return fail("cannot combine with another delivery");
    }
    let target=match delivery.anchor {ChannelAnchor::SelfOrigin=>"none",ChannelAnchor::Point=>"point",ChannelAnchor::Unit=>"unit"};
    if ability.target_type!=target {return fail("anchor does not match target_type");}
    if delivery.radius_key.is_none() && delivery.anchor!=ChannelAnchor::Unit {return fail("self and point anchors need radius_key");}
    if ability.max_level==0 || ability.levels.len()!=usize::from(ability.max_level) {return fail("needs one level row per rank");}
    // A windup may precede the channel: the release re-enters the same cast path.
    if ability.levels.iter().any(|level|!level.cast_time.is_finite()||!(0.0..=60.0).contains(&level.cast_time)) {
        return fail("needs a cast_time in [0, 60] seconds");
    }
    let rows=|key:&str,min:f32,max:f32|->Result<&Vec<f32>,String>{
        let rows=ability.extras.get(key).filter(|rows|!key.is_empty()&&rows.len()==usize::from(ability.max_level))
            .ok_or_else(||format!("ability '{}': channel missing per-rank extras '{key}'",ability.id))?;
        if rows.iter().any(|v|!v.is_finite()||*v<min||*v>max) {
            return Err(format!("ability '{}': channel extras '{key}' outside [{min},{max}]",ability.id));
        }
        Ok(rows)
    };
    let intervals=rows(&delivery.interval_key,MIN_CHANNEL_INTERVAL,10.0)?;
    let pulses=rows(&delivery.pulses_key,1.0,MAX_CHANNEL_PULSES)?;
    if pulses.iter().any(|count|count.fract()!=0.0) {return fail("pulse counts must be whole numbers");}
    if intervals.iter().zip(pulses).any(|(interval,count)|interval*count>MAX_CHANNEL_SECONDS) {return fail("lasts at most 60 seconds");}
    if let Some(key)=&delivery.radius_key {rows(key,0.0,2_000.0)?;}
    if delivery.anchor!=ChannelAnchor::SelfOrigin {
        for level in &ability.levels {
            if !level.range.is_finite() || level.range<MIN_ABILITY_SCALAR || level.range>10_000.0 {return fail("needs bounded per-rank range");}
        }
    }
    let (mut damage,mut slow,mut control)=(0,0,0);
    for effect in &ability.effects {
        match effect {
            AbilityEffect::Damage{amount_key,..}=>{damage+=1;rows(amount_key,0.0,MAX_ABILITY_SCALAR)?;}
            AbilityEffect::SlowEnemy{reduction_key,duration_key}=>{slow+=1;rows(reduction_key,MIN_ABILITY_SCALAR,1.0)?;rows(duration_key,MIN_ABILITY_SCALAR,60.0)?;}
            AbilityEffect::StunEnemy{duration_key}|AbilityEffect::ControlEnemy{duration_key,..}=>{control+=1;rows(duration_key,MIN_ABILITY_SCALAR,60.0)?;}
            _=>return fail("pulses only damage, slow_enemy and stun_enemy/control_enemy"),
        }
    }
    // No declarative pulse effects: the ability's Rust handler applies each
    // pulse (`on_channel_pulse`); the host still owns the lifecycle.
    if damage>1 || slow>1 || control>1 {return fail("pulse effects must be distinct (at most one of each)");}
    Ok(())
}

/// Per-rank extras keys. `mana_per_attack_key` >= 0; `amount_key` bonus damage;
/// optional slows are fractions in (0,1] applied for `slow_duration_key` seconds.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AttackOrb {
    pub mana_per_attack_key: String,
    pub amount_key: String,
    pub damage_kind: EffectDamageKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub move_slow_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_slow_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slow_duration_key: Option<String>,
}

fn validate_attack_orb(ability:&AbilityDefinition,orb:&AttackOrb)->Result<(),String> {
    let fail=|why:&str|Err(format!("ability '{}': attack_orb {why}",ability.id));
    if ability.ability_type!="toggle" || ability.target_type!="none" || ability.cast_type!="instant"
        || !ability.effects.is_empty() || ability.projectile.is_some() || ability.summon.is_some() || ability.charges.is_some() {
        return fail("requires an exclusive instant toggle with no target or effects");
    }
    if ability.max_level==0 || ability.levels.len()!=usize::from(ability.max_level) {return fail("needs one level row per rank");}
    let rows=|key:&str,min:f32,max:f32|->Result<(),String>{
        let rows=ability.extras.get(key).filter(|rows|!key.is_empty()&&rows.len()==usize::from(ability.max_level))
            .ok_or_else(||format!("ability '{}': attack_orb missing per-rank extras '{key}'",ability.id))?;
        if rows.iter().any(|v|!v.is_finite()||*v<min||*v>max) {return Err(format!("ability '{}': attack_orb extras '{key}' outside [{min},{max}]",ability.id));}
        Ok(())
    };
    rows(&orb.mana_per_attack_key,0.0,MAX_ABILITY_SCALAR)?;
    rows(&orb.amount_key,0.0,MAX_ABILITY_SCALAR)?;
    let slows=orb.move_slow_key.is_some()||orb.attack_slow_key.is_some();
    if slows!=orb.slow_duration_key.is_some() {return fail("slows and slow_duration_key must be declared together");}
    for key in [&orb.move_slow_key,&orb.attack_slow_key].into_iter().flatten() {rows(key,MIN_ABILITY_SCALAR,1.0)?;}
    if let Some(key)=&orb.slow_duration_key {rows(key,MIN_ABILITY_SCALAR,60.0)?;}
    Ok(())
}

/// Exclusive point-target summon. `unit` must name a live summon template
/// (checked where templates are loaded); `duration_key` overrides the
/// template lifetime per rank, in (0, 600] seconds.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AbilitySummonDelivery {
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_key: Option<String>,
}

fn validate_summon_delivery(ability:&AbilityDefinition,delivery:&AbilitySummonDelivery)->Result<(),String> {
    let fail=|why:&str|Err(format!("ability '{}': summon {why}",ability.id));
    if !matches!(ability.ability_type.as_str(),"active"|"ultimate") || ability.cast_type!="instant"
        || ability.target_type!="point" || !ability.effects.is_empty() || ability.projectile.is_some() || ability.zone_lifetime.is_some() {
        return fail("requires an instant active/ultimate point ability without effects, projectile or zone");
    }
    if delivery.unit.is_empty() || !delivery.unit.chars().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||c=='_') {return fail("unit must be a template id");}
    if ability.max_level==0 || ability.levels.len()!=usize::from(ability.max_level)
        || ability.levels.iter().any(|l|!l.range.is_finite()||l.range<MIN_ABILITY_SCALAR||l.range>10_000.0) {
        return fail("needs bounded per-rank range");
    }
    if let Some(key)=&delivery.duration_key {
        let rows=ability.extras.get(key).filter(|rows|rows.len()==usize::from(ability.max_level));
        if rows.is_none_or(|rows|rows.iter().any(|v|!v.is_finite()||*v<MIN_ABILITY_SCALAR||*v>600.0)) {return fail("duration must be per-rank in (0,600]");}
    }
    Ok(())
}

/// Per-rank extras keys: `max_key` holds whole counts in 1..=10, `recharge_key`
/// seconds per restored use in [0.25, 600]. The normal cooldown still applies
/// between casts (it may be zero).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AbilityCharges {
    pub max_key: String,
    pub recharge_key: String,
}

fn validate_ability_charges(ability:&AbilityDefinition)->Result<(),String> {
    let Some(charges)=&ability.charges else {return Ok(());};
    let fail=|why:&str|Err(format!("ability '{}': charges {why}",ability.id));
    if !matches!(ability.ability_type.as_str(),"active"|"ultimate") {return fail("require an active or ultimate ability");}
    let rows=|key:&str|ability.extras.get(key).filter(|rows|!key.is_empty()&&rows.len()==usize::from(ability.max_level));
    let (Some(max),Some(recharge))=(rows(&charges.max_key),rows(&charges.recharge_key)) else {return fail("need per-rank max and recharge extras");};
    if max.iter().any(|v|!v.is_finite()||v.fract()!=0.0||*v<1.0||*v>10.0) {return fail("max must be a whole number in 1..=10");}
    if recharge.iter().any(|v|!v.is_finite()||*v<0.25||*v>600.0) {return fail("recharge must be in [0.25,600] seconds");}
    if max.windows(2).any(|pair|pair[1]<pair[0]) {return fail("max must not decrease with rank");}
    Ok(())
}

/// G-1 projectile delivery. `effects` must be one `damage`, plus at most one
/// `slow_enemy` and one `stun_enemy`/`control_enemy`; they apply to each hit.
/// Linear flies the per-rank cast `range` toward a ground point; homing tracks
/// the selected enemy unit and hits only it.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProjectileDelivery {
    pub motion: ProjectileMotion,
    pub speed_key: String,
    pub radius_key: String,
    /// Required for homing; linear defaults to `range / speed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifetime_key: Option<String>,
    /// Distinct targets before the flight ends. Linear 1..=64; homing 1.
    #[serde(default = "default_projectile_hits")]
    pub max_hits: u16,
    #[serde(default)]
    pub wall: ProjectileWall,
    #[serde(default)]
    pub target_lost: ProjectileTargetLost,
    #[serde(default)]
    pub caster_death: ProjectileCasterDeath,
}
fn default_projectile_hits() -> u16 { 1 }

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileMotion { Linear, Homing }
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileWall { #[default] Pass, Stop }
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileTargetLost { #[default] Expire, LastPosition }
#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileCasterDeath { #[default] Continue, Expire }

/// Host bounds of `forced_motion`: distance in [1/1024,3000], duration in [1/1024,5].
fn forced_motion_rows(ability:&AbilityDefinition,distance_key:&str,duration_key:&str)->Result<(),String> {
    let bad=|key:&str,max:f32|ability.extras.get(key).is_none_or(|rows|rows.iter().any(|v|!v.is_finite()||*v<MIN_ABILITY_SCALAR||*v>max));
    if bad(distance_key,3_000.0) || bad(duration_key,5.0) {
        return Err(format!("ability '{}': forced_move_enemy requires distance in [1/1024,3000] and duration in [1/1024,5]",ability.id));
    }
    Ok(())
}

fn validate_projectile_delivery(ability:&AbilityDefinition,delivery:&ProjectileDelivery)->Result<(),String> {
    let fail=|why:&str|Err(format!("ability '{}': projectile {why}",ability.id));
    if !matches!(ability.ability_type.as_str(),"active"|"ultimate") || ability.cast_type!="instant"
        || ability.aim_geometry.is_some() || ability.zone_lifetime.is_some() {
        return fail("requires an instant active/ultimate without aim_geometry or zone_lifetime");
    }
    let linear=delivery.motion==ProjectileMotion::Linear;
    if ability.target_type!=if linear {"point"}else{"unit"} {return fail("linear needs a point target and homing a unit target");}
    let (mut damage,mut slow,mut control,mut forced)=(0,0,0,0);
    for effect in &ability.effects {
        match effect {
            AbilityEffect::Damage{..}=>damage+=1,
            AbilityEffect::SlowEnemy{..}=>slow+=1,
            AbilityEffect::StunEnemy{..}|AbilityEffect::ControlEnemy{..}=>control+=1,
            AbilityEffect::ForcedMoveEnemy{..}=>forced+=1,
            _=>return fail("delivers only damage, slow_enemy, stun_enemy/control_enemy and forced_move_enemy"),
        }
    }
    if damage!=1 || slow>1 || control>1 || forced>1 {return fail("needs one damage and at most one slow, one control and one forced move");}
    if !(1..=64).contains(&delivery.max_hits) || (!linear && delivery.max_hits!=1) {return fail("max_hits must be 1..=64 (homing 1)");}
    if !linear && delivery.wall!=ProjectileWall::Pass {return fail("homing cannot stop at walls");}
    if ability.max_level==0 || ability.levels.len()!=usize::from(ability.max_level) {return fail("needs one level row per rank");}
    let rows=|key:&str,min:f32,max:f32|->Result<&Vec<f32>,String>{
        let rows=ability.extras.get(key).filter(|rows|!key.is_empty()&&rows.len()==usize::from(ability.max_level))
            .ok_or_else(||format!("ability '{}': projectile missing per-rank extras '{key}'",ability.id))?;
        if rows.iter().any(|v|!v.is_finite()||*v<min||*v>max) {
            return Err(format!("ability '{}': projectile extras '{key}' outside [{min},{max}]",ability.id));
        }
        Ok(rows)
    };
    let speeds=rows(&delivery.speed_key,1.0,20_000.0)?;
    rows(&delivery.radius_key,0.0,1_000.0)?;
    let lifetimes=match &delivery.lifetime_key {
        Some(key)=>Some(rows(key,MIN_ABILITY_SCALAR,30.0)?),
        None if linear=>None,
        None=>return fail("homing requires lifetime_key"),
    };
    for (rank,level) in ability.levels.iter().enumerate() {
        if !level.range.is_finite() || level.range<MIN_ABILITY_SCALAR || level.range>10_000.0 {return fail("needs bounded per-rank range");}
        if lifetimes.is_none() && level.range/speeds[rank]>30.0 {return fail("linear flight exceeds 30 seconds");}
    }
    for effect in &ability.effects {
        match effect {
            AbilityEffect::Damage{amount_key,..}=>{rows(amount_key,0.0,MAX_ABILITY_SCALAR)?;}
            AbilityEffect::SlowEnemy{reduction_key,duration_key}=>{rows(reduction_key,MIN_ABILITY_SCALAR,1.0)?;rows(duration_key,MIN_ABILITY_SCALAR,60.0)?;}
            AbilityEffect::StunEnemy{duration_key}|AbilityEffect::ControlEnemy{duration_key,..}=>{rows(duration_key,MIN_ABILITY_SCALAR,60.0)?;}
            AbilityEffect::ForcedMoveEnemy{distance_key,duration_key,..}=>{rows(distance_key,0.0,f32::MAX)?;rows(duration_key,0.0,f32::MAX)?;forced_motion_rows(ability,distance_key,duration_key)?;}
            _=>unreachable!("checked above"),
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundZoneLifetime { CasterBound, Independent }

fn validate_zone_lifetime(ability:&AbilityDefinition)->Result<(),String> {
    if ability.zone_lifetime.is_none(){return Ok(());}
    let fail=||Err(format!("ability '{}': invalid ground-zone lifetime contract",ability.id));
    if ability.target_type!="point" || ability.cast_type!="instant"
        || !matches!(&ability.aim_geometry,Some(AbilityAimGeometry::PointCircle{..})) {return fail();}
    for key in ["zone_damage","zone_interval","zone_duration"] {
        if ability.extras.get(key).is_none_or(|v|v.len()!=usize::from(ability.max_level)){return fail();}
    }
    for rank in 0..usize::from(ability.max_level) {
        let amount=ability.extras["zone_damage"][rank];let interval=ability.extras["zone_interval"][rank];let duration=ability.extras["zone_duration"][rank];
        if !amount.is_finite()||!((1.0/1024.0)..=1_000_000.0).contains(&amount)
            ||!interval.is_finite()||!(0.25..=10.).contains(&interval)
            ||!duration.is_finite()||duration<interval||duration>30. {return fail();}
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn ground_zone_lifetime_is_closed_typed_and_requires_bounded_complete_circle_ranks() {
    let base=serde_json::json!({"id":"arbitrary_zone","ability_type":"active","cast_type":"instant","target_type":"point","max_level":1,
        "levels":[{"range":700,"cooldown":10,"mana_cost":100}],"aim_geometry":{"kind":"point_circle","radius_key":"radius"},
        "zone_lifetime":"independent","extras":{"radius":[100],"zone_damage":[10],"zone_interval":[0.5],"zone_duration":[2]}});
    for policy in ["independent","caster_bound"] {let mut v=base.clone();v["zone_lifetime"]=serde_json::json!(policy);
        validate_ability_progression(&serde_json::from_value(v).unwrap()).unwrap();}
    for bad in [serde_json::json!(true),serde_json::json!(1),serde_json::json!("other")] {let mut v=base.clone();v["zone_lifetime"]=bad;assert!(serde_json::from_value::<AbilityDefinition>(v).is_err());}
    for (key,value) in [("zone_interval",serde_json::json!([0.1])),("zone_duration",serde_json::json!([31])),
        ("zone_duration",serde_json::json!([0.25])),("zone_damage",serde_json::json!([]))] {
        let mut v=base.clone();v["extras"][key]=value;assert!(validate_ability_progression(&serde_json::from_value(v).unwrap()).is_err());
    }
    let mut v=base.clone();v["target_type"]=serde_json::json!("unit");assert!(validate_ability_progression(&serde_json::from_value(v).unwrap()).is_err());
    let mut legacy=base;legacy.as_object_mut().unwrap().remove("zone_lifetime");
    let legacy:AbilityDefinition=serde_json::from_value(legacy).unwrap();assert!(serde_json::to_value(legacy).unwrap().get("zone_lifetime").is_none());
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AbilityAimGeometry {
    /// Center-point capsule along an instantaneous, range-clamped dash.
    /// Terrain may shorten the actual path; this is an intended path, not an ACK.
    SweptPath { width_key: String },
    /// Special-handler circle centered at the requested point.
    PointCircle { radius_key: String },
    /// Standard summon_count/row_spacing/col_spacing/front_row_distance extras.
    FacingFormation { rows: u16 },
}

/// Aiming footprint a package declares for its Rust handler (`targeting.shape`). It is published
/// for presentation only, and only where no gameplay contract (aim_geometry, effects, projectile)
/// already publishes a footprint; it never changes cast admission or execution.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AbilityTargetingShape {
    /// Circle centered at the requested point; per-rank radius in world units.
    Circle { radius_key: String },
    /// Capsule from the source toward the requested point; per-rank full width.
    Line { width_key: String },
    /// Source-centered sector toward the requested point; per-rank half angle in radians.
    Cone { half_angle_key: String },
}

/// Gameplay displacement requires an unrooted source. Presentation must use the
/// same authored contract, never a skill ID or a cosmetic movement animation.
pub fn ability_requires_movement(ability: &AbilityDefinition) -> bool {
    matches!(&ability.aim_geometry, Some(AbilityAimGeometry::SweptPath { .. }))
        || ability.effects.iter().any(|effect| matches!(effect, AbilityEffect::DashToPoint))
}

/// Shared row count for executor constants and generated presentation. Zero is absent.
pub fn facing_formation_rows(ability:&AbilityDefinition)->u16 {
    match &ability.aim_geometry {Some(AbilityAimGeometry::FacingFormation{rows})=>*rows,_=>0}
}

/// Canonical executable intent, shared by native generation and reload guards.
/// Cosmetic cues and nonspatial balance extras do not participate.
#[derive(Debug,Serialize)]
pub struct AbilityAimPreview {
    pub geometry:Option<AbilityAimGeometry>,
    pub ability_type:String,
    pub cast_type:String,
    pub target_type:String,
    pub requires_movement:bool,
    pub formation_rows:u16,
    /// The published footprint comes only from the package's declared `targeting_shape`.
    pub shape_declared_only:bool,
    pub ranks:Vec<AbilityAimPreviewRank>,
}
#[derive(Debug,Serialize)]
pub struct AbilityAimPreviewRank {
    pub cone_half_angle_ticks:u16,
    pub range:f32,
    pub point_radius:f32,
    pub path_width:f32,
    pub formation_count:u16,
    pub row_spacing:f32,
    pub column_spacing:f32,
    pub front_distance:f32,
    /// Presentation-only movement reach (`targeting_reach_key`); 0 when absent.
    pub clamp_range:f32,
}
pub fn ability_aim_preview(ability:&AbilityDefinition)->Result<AbilityAimPreview,String> {
    validate_ability_progression(ability)?;
    let mut ranks=Vec::with_capacity(ability.levels.len());
    let mut shape_declared_only=false;
    for (rank,level) in ability.levels.iter().enumerate() {
        let extra=|key:&str|ability.extras.get(key).and_then(|values|values.get(rank)).copied().unwrap_or(0.0);
        let point_radius=if let Some(AbilityAimGeometry::PointCircle{radius_key})=&ability.aim_geometry {extra(radius_key)}
            else if ability.target_type=="point" {ability.effects.iter().filter_map(|effect|match effect {
                AbilityEffect::AreaDamage{radius_key,..}|AbilityEffect::RevealArea{radius_key,..}=>Some(extra(radius_key)),_=>None})
                // A point-anchored host channel pulses within its radius around the cast point.
                .chain(ability.channel.iter().filter(|c|c.anchor==ChannelAnchor::Point).filter_map(|c|c.radius_key.as_deref().map(extra)))
                .fold(0.0_f32,f32::max)}else{0.0};
        let path_width=match &ability.aim_geometry {Some(AbilityAimGeometry::SweptPath{width_key})=>extra(width_key),_=>ability.effects.iter().find_map(|effect|match effect {AbilityEffect::LineDamage{width_key,..}=>Some(extra(width_key)),_=>None})
            .or_else(||ability.projectile.as_ref().filter(|p|p.motion==ProjectileMotion::Linear).map(|p|2.0*extra(&p.radius_key))).unwrap_or(0.0)};
        let formation=facing_formation_rows(ability)>0;
        let mut cone_half_angle_ticks=ability.effects.iter().find_map(|effect|match effect {
            AbilityEffect::ConeDamage{half_angle_key,..}=>omoba_sim::sector::half_angle_ticks(
                omoba_sim::Fixed64::from_raw((f64::from(extra(half_angle_key))*1024.).round() as i64)),_=>None}).unwrap_or(0);
        let (mut point_radius,mut path_width)=(point_radius,path_width);
        // A declared footprint only fills an otherwise unpublished point-target ability.
        if ability.target_type=="point" && !formation && point_radius==0.0 && path_width==0.0 && cone_half_angle_ticks==0 {
            match &ability.targeting_shape {
                Some(AbilityTargetingShape::Circle{radius_key})=>{point_radius=extra(radius_key);shape_declared_only|=point_radius>0.0;}
                Some(AbilityTargetingShape::Line{width_key})=>{path_width=extra(width_key);shape_declared_only|=path_width>0.0;}
                Some(AbilityTargetingShape::Cone{half_angle_key})=>{cone_half_angle_ticks=omoba_sim::sector::half_angle_ticks(
                    omoba_sim::Fixed64::from_raw((f64::from(extra(half_angle_key))*1024.).round() as i64)).unwrap_or(0);shape_declared_only|=cone_half_angle_ticks>0;}
                None=>{}
            }
        }
        ranks.push(AbilityAimPreviewRank{cone_half_angle_ticks,range:level.range,point_radius,path_width,
            formation_count:if formation {extra("summon_count") as u16}else{0},
            row_spacing:if formation {extra("row_spacing")}else{0.0},
            column_spacing:if formation {extra("col_spacing")}else{0.0},
            front_distance:if formation {extra("front_row_distance")}else{0.0},
            clamp_range:ability.targeting_reach_key.as_deref().filter(|_|ability.target_type=="point" && level.range==0.0).map_or(0.0,extra)});
    }
    Ok(AbilityAimPreview{geometry:ability.aim_geometry.clone(),ability_type:ability.ability_type.clone(),cast_type:ability.cast_type.clone(),target_type:ability.target_type.clone(),
        requires_movement:ability_requires_movement(ability),formation_rows:facing_formation_rows(ability),shape_declared_only,ranks})
}
pub fn ability_aim_preview_signature(ability:&AbilityDefinition)->Result<String,String> {
    serde_json::to_string(&ability_aim_preview(ability)?).map_err(|error|error.to_string())
}

/// Executable subset of Lua-authored effects. Amount-bearing effects resolve
/// `amount_key` from per-level `extras`; special skills still use a Rust handler.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AbilityEffect {
    /// Exclusive stationary, host-owned recovery channel; amounts are per pulse.
    RecoveryChannelSelf { amount_key:String, duration_key:String, interval_key:String },
    /// Instant source-to-point capsule; no displacement or terrain collision.
    LineDamage { amount_key:String, width_key:String, damage_kind:EffectDamageKind },
    /// Source-centered directional sector; the requested ground point supplies its axis.
    ConeDamage { amount_key:String, half_angle_key:String, damage_kind:EffectDamageKind },
    DamageOverTimeEnemy { amount_key:String, duration_key:String, damage_kind:EffectDamageKind },
    ManaBuffSelf { stat: ManaBuffStat, value_key:String, duration_key:String },
    SlowEnemy { reduction_key:String, duration_key:String },
    StunEnemy { duration_key:String },
    ControlEnemy { control:ControlEffectKind, duration_key:String },
    /// Forced displacement of the enemy unit target (G-4): `distance_key` world
    /// units straight away from (knockback) or toward (pull, stopping at body
    /// contact) the caster over `duration_key` seconds. The target is stunned
    /// for that time and terrain stops it; other units never block it.
    ForcedMoveEnemy { motion:ForcedMotionKind, distance_key:String, duration_key:String },
    /// True-sight area at the point target (G-4 detection): the caster's team
    /// sees within `radius_key` and detects every stealth level there for
    /// `duration_key` seconds. Line of sight still applies; nothing is damaged.
    RevealArea { radius_key:String, duration_key:String },
    /// Instant relocation with the normal swept-terrain collision contract.
    /// Deliberately exclusive until ordered movement/effect semantics exist.
    DashToPoint,
    Damage {
        amount_key: String,
        damage_kind: EffectDamageKind,
    },
    HealSelf {
        amount_key: String,
    },
    HealAlly { amount_key:String },
    ShieldAlly { amount_key:String, duration_key:String },
    CleanseAlly {},
    DrainManaEnemy { amount_key:String },
    RestoreManaSelf { amount_key: String },
    /// Additional script cost, not the host's metadata mana_cost reservation.
    SpendManaSelf { amount_key: String },
    AreaDamage {
        amount_key: String,
        radius_key: String,
        damage_kind: EffectDamageKind,
    },
}

/// Direction of a forced displacement relative to its caster.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForcedMotionKind { Knockback, Pull }

impl ForcedMotionKind {
    pub const fn as_str(self)->&'static str { match self { Self::Knockback=>"knockback",Self::Pull=>"pull" } }
}

/// Closed gameplay status set; never accept an arbitrary authored Buff ID.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlEffectKind { Stun, Root, Silence }

impl ControlEffectKind {
    pub const fn as_str(self)->&'static str {
        match self { Self::Stun=>"stun",Self::Root=>"root",Self::Silence=>"silence" }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectDamageKind {
    Physical,
    Magical,
    Pure,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="snake_case")]
pub enum ManaBuffStat {
    BaseManaRegen, ManaRegenConstant, ManaRegenConstantUnique,
    ManaRegenPercentage, ManaRegenTotalPercentage, ManaBonus, ExtraManaBonus,
}
impl ManaBuffStat {
    pub fn as_str(self)->&'static str {
        match self {
            Self::BaseManaRegen=>"base_mana_regen",Self::ManaRegenConstant=>"mana_regen_constant",
            Self::ManaRegenConstantUnique=>"mana_regen_constant_unique",Self::ManaRegenPercentage=>"mana_regen_percentage",
            Self::ManaRegenTotalPercentage=>"mana_regen_total_percentage",Self::ManaBonus=>"mana_bonus",Self::ExtraManaBonus=>"extra_mana_bonus",
        }
    }
    pub fn bounds(self)->(i32,i32) {
        match self {
            Self::BaseManaRegen=>(0,1_000_000),
            Self::ManaRegenPercentage|Self::ManaRegenTotalPercentage=>(-1,16),
            _=>(-1_000_000,1_000_000),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AbilityLevel {
    #[serde(default = "default_required_hero_level")]
    pub required_hero_level: u8,
    #[serde(default)]
    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f32,
    #[serde(default)]
    pub cast_time: f32,
    #[serde(default)]
    pub cast_interrupt_on_damage: bool,
    #[serde(default)]
    pub range: f32,
}

fn default_required_hero_level() -> u8 { 1 }

/// Authored numeric envelope, not a balance rule. Positive values must survive
/// Q10 quantization; the upper bound leaves headroom for composed effects.
/// Keep the fixed Lua FFI generator's envelope in sync with these constants.
pub const MIN_ABILITY_SCALAR: f32 = 1.0 / 1024.0;
pub const MAX_ABILITY_SCALAR: f32 = 1_000_000.0;

fn valid_ability_scalar(value: f32) -> bool {
    value.is_finite() && (value == 0.0 || (MIN_ABILITY_SCALAR..=MAX_ABILITY_SCALAR).contains(&value))
}

impl Default for AbilityLevel {
    fn default() -> Self {
        Self { required_hero_level: 1, cooldown: 0.0, mana_cost: 0.0, cast_time: 0.0, cast_interrupt_on_damage: false, range: 0.0 }
    }
}

pub fn validate_ability_progression(ability: &AbilityDefinition) -> Result<(), String> {
    if ability.tombstone { return Ok(()); }
    if ability.max_level == 0 || ability.levels.len() != usize::from(ability.max_level) {
        return Err(format!("ability '{}': levels must match nonzero max_level", ability.id));
    }
    let mut previous = 1;
    for (rank, level) in ability.levels.iter().enumerate() {
        for (field, value) in [("cooldown", level.cooldown), ("mana_cost", level.mana_cost),
            ("cast_time", level.cast_time), ("range", level.range)] {
            if !valid_ability_scalar(value) {
                return Err(format!("ability '{}': rank {} {field} must be zero or finite in [1/1024,1000000]", ability.id, rank + 1));
            }
        }
        if level.cast_interrupt_on_damage && !(MIN_ABILITY_SCALAR..=60.0).contains(&level.cast_time) {
            return Err(format!("ability '{}': rank {} cast_interrupt_on_damage requires cast_time in [1/1024,60]",ability.id,rank+1));
        }
        if !(1..=25).contains(&level.required_hero_level) || level.required_hero_level < previous {
            return Err(format!("ability '{}': rank {} required_hero_level must be monotonic in 1..=25", ability.id, rank + 1));
        }
        previous = level.required_hero_level;
    }
    validate_zone_lifetime(ability)?;
    validate_ability_charges(ability)?;
    validate_ability_aim_geometry(ability)?;
    validate_ability_targeting_shape(ability)?;
    validate_ability_targeting_reach(ability)?;
    if let Some(block) = &ability.v3 {
        v3::validate_block(block, "ability", ability.max_level).map_err(|e| format!("ability '{}': {e}", ability.id))?;
    }
    validate_ability_effects(ability)
}

/// Every rank of a declared footprint is a finite positive dimension within the aiming bounds.
/// `targeting_reach_key` only describes a range-free point ability's movement.
pub fn validate_ability_targeting_reach(ability: &AbilityDefinition) -> Result<(), String> {
    let Some(key) = &ability.targeting_reach_key else { return Ok(()); };
    let valid = ability.target_type == "point" && ability.levels.iter().all(|level| level.range == 0.0)
        && !key.is_empty() && ability.extras.get(key).is_some_and(|values| values.len() == usize::from(ability.max_level)
            && values.iter().all(|value| value.is_finite() && (MIN_ABILITY_SCALAR..=10_000.0).contains(value)));
    if valid { Ok(()) } else {
        Err(format!("ability '{}': targeting_reach_key needs point targeting, range 0 on every rank and a complete positive per-rank '{}'", ability.id, key))
    }
}

pub fn validate_ability_targeting_shape(ability: &AbilityDefinition) -> Result<(), String> {
    let Some(shape) = &ability.targeting_shape else { return Ok(()); };
    let (key, maximum) = match shape {
        AbilityTargetingShape::Circle { radius_key } => (radius_key, 10_000.0),
        AbilityTargetingShape::Line { width_key } => (width_key, 5_000.0),
        AbilityTargetingShape::Cone { half_angle_key } => (half_angle_key, std::f32::consts::FRAC_PI_2),
    };
    let valid = !key.is_empty() && ability.extras.get(key).is_some_and(|values| values.len() == usize::from(ability.max_level)
        && values.iter().all(|value| value.is_finite() && (MIN_ABILITY_SCALAR..=maximum).contains(value)));
    if valid { Ok(()) } else { Err(format!("ability '{}': targeting_shape requires a complete positive per-rank '{}'", ability.id, key)) }
}

pub fn validate_ability_aim_geometry(ability: &AbilityDefinition) -> Result<(), String> {
    let Some(geometry) = &ability.aim_geometry else { return Ok(()); };
    if let AbilityAimGeometry::FacingFormation {rows}=geometry {
        let fail=||Err(format!("ability '{}': invalid facing formation contract",ability.id));
        if !matches!(ability.ability_type.as_str(),"active"|"ultimate")||ability.cast_type!="instant"
            ||ability.target_type!="point"||!ability.effects.is_empty()||*rows==0||*rows>128 {return fail();}
        let keys=["summon_count","row_spacing","col_spacing","front_row_distance"];
        for key in keys {if !ability.extras.get(key).is_some_and(|values|values.len()==usize::from(ability.max_level)
            &&values.iter().all(|v|v.is_finite()&&*v>=if key=="front_row_distance"{0.}else{MIN_ABILITY_SCALAR}&&*v<=10_000.)) {return fail();}}
        for rank in 0..usize::from(ability.max_level) {
            let count=ability.extras["summon_count"][rank];let row=ability.extras["row_spacing"][rank];
            let col=ability.extras["col_spacing"][rank];let front=ability.extras["front_row_distance"][rank];
            if !(2.0..=128.0).contains(&count)||count.fract()!=0.||count as u16%*rows!=0
                ||front+row*f32::from(*rows-1)>10_000.||col*(count/f32::from(*rows)-1.)*0.5>10_000.
                ||!ability.levels.get(rank).is_some_and(|level|level.range.is_finite()&&(MIN_ABILITY_SCALAR..=10_000.).contains(&level.range)) {return fail();}
        }
        return Ok(());
    }
    let (key,maximum)=match geometry {
        AbilityAimGeometry::FacingFormation {..}=>unreachable!(),
        AbilityAimGeometry::SweptPath {width_key}=>(width_key,5_000.0),
        AbilityAimGeometry::PointCircle {radius_key}=>{
            if !ability.effects.is_empty() {return Err(format!("ability '{}': point_circle is a special-handler contract; executable effects already declare their geometry",ability.id));}
            (radius_key,10_000.0)
        },
    };
    let valid = matches!(ability.ability_type.as_str(), "active" | "ultimate")
        && ability.cast_type == "instant" && ability.target_type == "point"
        && ability.levels.iter().all(|level| level.range.is_finite()
            && (MIN_ABILITY_SCALAR..=10_000.0).contains(&level.range))
        && !key.is_empty() && ability.extras.get(key).is_some_and(|widths|
            widths.len() == usize::from(ability.max_level) && widths.iter().all(|width|
                width.is_finite() && (MIN_ABILITY_SCALAR..=maximum).contains(width)));
    if valid { Ok(()) } else { Err(format!("ability '{}': aim geometry requires instant point targeting, bounded rank range and a complete positive dimension key", ability.id)) }
}

/// Unit-target allegiance of an authored effect list.
///
/// `HealAlly` is ally. `Damage`, `SlowEnemy`, `StunEnemy`, and `ControlEnemy`
/// are enemy. Self, area, dash, and empty or custom lists are unspecified.
/// Area damage is a point effect and does not make the unit target hostile.
/// A list that contains both sides is the same conflict `validate_ability_effects` rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitTargetAllegiance {
    Ally,
    Enemy,
    Unspecified,
}

impl UnitTargetAllegiance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ally => "ally",
            Self::Enemy => "enemy",
            Self::Unspecified => "unspecified",
        }
    }
}

pub fn unit_target_allegiance(effects: &[AbilityEffect]) -> Result<UnitTargetAllegiance, &'static str> {
    let ally = effects.iter().any(|effect| matches!(effect, AbilityEffect::HealAlly { .. } | AbilityEffect::ShieldAlly { .. } | AbilityEffect::CleanseAlly {}));
    let enemy = effects.iter().any(|effect| matches!(effect,
        AbilityEffect::Damage { .. } | AbilityEffect::DrainManaEnemy {..} | AbilityEffect::DamageOverTimeEnemy {..} | AbilityEffect::SlowEnemy { .. }
            | AbilityEffect::StunEnemy { .. } | AbilityEffect::ControlEnemy { .. } | AbilityEffect::ForcedMoveEnemy { .. }));
    match (ally, enemy) {
        (true, true) => Err("conflicting unit target allegiance"),
        (true, false) => Ok(UnitTargetAllegiance::Ally),
        (false, true) => Ok(UnitTargetAllegiance::Enemy),
        (false, false) => Ok(UnitTargetAllegiance::Unspecified),
    }
}

/// Published unit-target allegiance of an ability: its declarative effects
/// when they imply one, otherwise the authored `unit_target` of a unit-target
/// ability, otherwise unspecified.
pub fn ability_unit_allegiance(ability: &AbilityDefinition) -> Result<UnitTargetAllegiance, String> {
    let derived = unit_target_allegiance(&ability.effects).map_err(|why| format!("ability '{}': {why}", ability.id))?;
    let authored = match ability.unit_target.as_deref() {
        None => None,
        Some("ally") => Some(UnitTargetAllegiance::Ally),
        Some("enemy") => Some(UnitTargetAllegiance::Enemy),
        Some(other) => return Err(format!("ability '{}': unit_target '{other}' must be ally or enemy", ability.id)),
    };
    if authored.is_some() && ability.target_type != "unit" {
        return Err(format!("ability '{}': unit_target requires a unit target_type", ability.id));
    }
    match (derived, authored) {
        (UnitTargetAllegiance::Unspecified, Some(authored)) => Ok(authored),
        (derived, Some(authored)) if derived != authored => Err(format!("ability '{}': unit_target contradicts its effects", ability.id)),
        (derived, _) => Ok(derived),
    }
}

#[cfg(test)]
#[test]
fn authored_unit_target_fills_only_what_effects_leave_unspecified() {
    let ability=|unit_target:Option<&str>,target_type:&str,effects:serde_json::Value|->AbilityDefinition {
        let mut v=serde_json::json!({"id":"fixture","max_level":1,"ability_type":"active","cast_type":"instant","target_type":target_type,
            "levels":[{"range":600}],"extras":{"heal":[10]},"effects":effects});
        if let Some(t)=unit_target {v["unit_target"]=t.into();}
        serde_json::from_value(v).unwrap()
    };
    let none=serde_json::json!([]);let heal=serde_json::json!([{"kind":"heal_ally","amount_key":"heal"}]);
    assert_eq!(ability_unit_allegiance(&ability(Some("ally"),"unit",none.clone())).unwrap(),UnitTargetAllegiance::Ally);
    assert_eq!(ability_unit_allegiance(&ability(Some("enemy"),"unit",none.clone())).unwrap(),UnitTargetAllegiance::Enemy);
    assert_eq!(ability_unit_allegiance(&ability(None,"unit",none.clone())).unwrap(),UnitTargetAllegiance::Unspecified);
    assert_eq!(ability_unit_allegiance(&ability(Some("ally"),"unit",heal.clone())).unwrap(),UnitTargetAllegiance::Ally);
    assert!(ability_unit_allegiance(&ability(Some("enemy"),"unit",heal)).is_err(),"contradiction");
    assert!(ability_unit_allegiance(&ability(Some("ally"),"point",none.clone())).is_err(),"unit targets only");
    assert!(ability_unit_allegiance(&ability(Some("both"),"unit",none.clone())).is_err(),"closed vocabulary");
    let encoded=serde_json::to_value(ability(None,"unit",none)).unwrap();assert!(encoded.get("unit_target").is_none());
    let fixture=ability(Some("ally"),"unit",serde_json::json!([]));
    assert!(validate_ability_effects(&fixture).is_ok());
    let mut wrong=fixture.clone();wrong.unit_target=Some("neutral".into());
    assert!(validate_ability_effects(&wrong).is_err());
}

/// Shared by template IDs, runtime content and Unreal codegen.
#[cfg(test)]
#[test]
fn recovery_channel_authoring_is_closed_exclusive_and_exactly_quantized() {
    let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
        "id":"arbitrary_channel","max_level":1,"ability_type":"active","cast_type":"channeled","target_type":"none",
        "levels":[{}],"extras":{"pulse":[35],"duration":[2],"interval":[0.5]},
        "effects":[{"kind":"recovery_channel_self","amount_key":"pulse","duration_key":"duration","interval_key":"interval"}]
    })).unwrap();
    assert!(validate_ability_progression(&valid).is_ok());
    for (key,value) in [("pulse",0.0),("pulse",0.1),("pulse",1_000_001.0),("duration",61.0),("duration",1.25),("interval",0.1),("interval",11.0)] {
        let mut bad=valid.clone();bad.extras.get_mut(key).unwrap()[0]=value;assert!(validate_ability_progression(&bad).is_err(),"{key}={value}");
    }
    for case in 0..6 {
        let mut bad=valid.clone();match case {
            0=>bad.cast_type="instant".into(),1=>bad.target_type="point".into(),2=>bad.ability_type="passive".into(),
            3=>bad.effects.push(AbilityEffect::HealSelf {amount_key:"pulse".into()}),
            4=>bad.levels[0].cast_time=0.5,5=>{bad.extras.remove("interval");},_=>unreachable!(),
        };assert!(validate_ability_progression(&bad).is_err(),"case{case}");
    }
    let encoded=serde_json::to_value(&valid).unwrap();assert_eq!(encoded["effects"][0]["kind"],"recovery_channel_self");
}

pub fn validate_ability_effects(ability: &AbilityDefinition) -> Result<(), String> {
    if ability.tombstone {return Ok(());}
    if let Some(refund)=&ability.cast_cooldown_refund {validate_cast_cooldown_refund(ability,refund)?;}
    if ability.unit_target.is_some() {ability_unit_allegiance(ability)?;}
    if let Some(orb)=&ability.attack_orb {return validate_attack_orb(ability,orb);}
    if let Some(delivery)=&ability.summon {return validate_summon_delivery(ability,delivery);}
    if let Some(delivery)=&ability.projectile {return validate_projectile_delivery(ability,delivery);}
    if let Some(delivery)=&ability.channel {return validate_channel_delivery(ability,delivery);}
    if ability.effects.is_empty() {return Ok(());}
    if let [AbilityEffect::RecoveryChannelSelf {amount_key,duration_key,interval_key}]=ability.effects.as_slice() {
        let valid=matches!(ability.ability_type.as_str(),"active"|"ultimate")
            && ability.cast_type=="channeled" && ability.target_type=="none"
            && ability.max_level>0 && ability.levels.len()==usize::from(ability.max_level)
            && ability.levels.iter().all(|level|level.cast_time==0.0);
        if !valid {return Err(format!("ability '{}': recovery channel requires exclusive channeled self targeting with no preparation",ability.id));}
        let values=|key:&str,min:f32,max:f32|->Result<Vec<i64>,String>{
            let rows=ability.extras.get(key).filter(|rows|!key.is_empty()&&rows.len()==usize::from(ability.max_level))
                .ok_or_else(||format!("ability '{}': missing channel extras '{key}'",ability.id))?;
            rows.iter().map(|&value|{
                let raw=f64::from(value)*1024.0;
                if !value.is_finite()||value<min||value>max||raw.fract()!=0.0 {
                    Err(format!("ability '{}': invalid exactly representable channel extras '{key}'",ability.id))
                }else{Ok(raw as i64)}
            }).collect()
        };
        values(amount_key,MIN_ABILITY_SCALAR,MAX_ABILITY_SCALAR)?;
        let duration=values(duration_key,MIN_ABILITY_SCALAR,60.0)?;
        let interval=values(interval_key,0.25,10.0)?;
        if duration.iter().zip(interval).any(|(duration,interval)|*duration<interval||*duration%interval!=0){
            return Err(format!("ability '{}': channel duration must contain whole intervals",ability.id));
        }
        return Ok(());
    }
    if ability.effects.len()>32 || !matches!(ability.ability_type.as_str(),"active"|"ultimate")
        || ability.cast_type!="instant" {return Err(format!("ability '{}': effects require bounded instant active/ultimate",ability.id));}
    // All unit effects share one target; it cannot be both an ally and enemy.
    if let Err(conflict) = unit_target_allegiance(&ability.effects) {
        return Err(format!("ability '{}': {conflict}", ability.id));
    }
    if ability.effects.iter().any(|effect|matches!(effect,AbilityEffect::DashToPoint)) {
        if ability.effects.len()!=1 || ability.target_type!="point" {
            return Err(format!("ability '{}': dash_to_point requires one exclusive point effect",ability.id));
        }
        if ability.levels.len()!=usize::from(ability.max_level) || ability.levels.iter()
            .any(|l|!l.range.is_finite() || l.range<MIN_ABILITY_SCALAR || l.range>10_000.0) {
            return Err(format!("ability '{}': dash_to_point requires positive bounded per-rank range",ability.id));
        }
        return Ok(());
    }
    let values=|key:&str,radius:bool| -> Result<(),String> {
        let entries=ability.extras.get(key).ok_or_else(||format!("ability '{}': missing effect extras '{key}'",ability.id))?;
        if key.is_empty() || entries.len()!=usize::from(ability.max_level)
            || entries.iter().any(|v|!valid_ability_scalar(*v) || (radius && (*v<1.0/1024.0 || *v>10_000.0))) {
            return Err(format!("ability '{}': invalid per-rank effect extras '{key}'",ability.id));
        }
        Ok(())
    };
    let mut mana_stats=BTreeSet::new();
    let mut dot_seen=false;
    for effect in &ability.effects {
        let (amount,target)=match effect {
            AbilityEffect::ManaBuffSelf {stat,value_key,duration_key}=>{
                if ability.target_type!="none" || !mana_stats.insert(*stat) {
                    return Err(format!("ability '{}': mana_buff_self requires none target and unique stat",ability.id));
                }
                values(duration_key,false)?;
                if ability.extras[duration_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>60.0) {
                    return Err(format!("ability '{}': mana buff duration must be in [1/1024,60]",ability.id));
                }
                let entries=ability.extras.get(value_key).ok_or_else(||format!("ability '{}': missing mana buff value '{value_key}'",ability.id))?;
                let (minimum,maximum)=stat.bounds();
                if value_key.is_empty() || entries.len()!=usize::from(ability.max_level)
                    || entries.iter().any(|v|!v.is_finite() || *v<minimum as f32 || *v>maximum as f32
                        || (*v!=0.0 && v.abs()<MIN_ABILITY_SCALAR)) {
                    return Err(format!("ability '{}': invalid mana buff value '{value_key}'",ability.id));
                }
                continue;
            },
            AbilityEffect::DashToPoint=>unreachable!("exclusive movement checked above"),
            AbilityEffect::DamageOverTimeEnemy {amount_key,duration_key,..}=>{
                if dot_seen {return Err(format!("ability '{}': duplicate DoT effect",ability.id));}dot_seen=true;
                values(duration_key,false)?;
                if ability.extras[duration_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>60.0) {
                    return Err(format!("ability '{}': DoT duration must be in [1/1024,60]",ability.id));
                }
                (amount_key,"unit")
            },
            AbilityEffect::RecoveryChannelSelf {..}=>return Err(format!("ability '{}': recovery channel must be exclusive",ability.id)),
            AbilityEffect::Damage {amount_key,..}=>(amount_key,"unit"),
            AbilityEffect::DrainManaEnemy {amount_key}=>{
                values(amount_key,false)?;
                if ability.extras[amount_key].iter().any(|v|*v<MIN_ABILITY_SCALAR){return Err(format!("ability '{}': mana drain requires positive amount",ability.id));}
                (amount_key,"unit")
            },
            AbilityEffect::CleanseAlly {}=>{
                if ability.target_type!="unit" || ability.levels.iter().any(|l|!l.range.is_finite()||l.range<MIN_ABILITY_SCALAR||l.range>10_000.) {
                    return Err(format!("ability '{}': ally cleanse requires bounded unit range",ability.id));
                }
                continue;
            },
            AbilityEffect::HealAlly {amount_key}=>(amount_key,"unit"),
            AbilityEffect::ShieldAlly {amount_key,duration_key}=>{
                values(amount_key,false)?;values(duration_key,false)?;
                if ability.extras[amount_key].iter().any(|v|*v<MIN_ABILITY_SCALAR)
                    || ability.extras[duration_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>60.) {return Err(format!("ability '{}': shield requires positive amount and duration in [1/1024,60]",ability.id));}
                (amount_key,"unit")
            },
            AbilityEffect::StunEnemy {duration_key} | AbilityEffect::ControlEnemy {duration_key,..}=>{
                values(duration_key,false)?;
                if ability.extras[duration_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>60.0) {
                    return Err(format!("ability '{}': control duration must be in [1/1024,60]",ability.id));
                }
                (duration_key,"unit")
            },
            AbilityEffect::ForcedMoveEnemy {distance_key,duration_key,..}=>{
                values(distance_key,false)?;values(duration_key,false)?;
                forced_motion_rows(ability,distance_key,duration_key)?;
                (distance_key,"unit")
            },
            AbilityEffect::RevealArea {radius_key,duration_key}=>{
                values(radius_key,false)?;values(duration_key,false)?;
                let bad=|key:&String,max:f32|ability.extras[key].iter().any(|v|*v<MIN_ABILITY_SCALAR||*v>max);
                if bad(radius_key,2_000.) || bad(duration_key,30.)
                    || ability.levels.iter().any(|l|!l.range.is_finite()||l.range<MIN_ABILITY_SCALAR||l.range>10_000.) {
                    return Err(format!("ability '{}': reveal_area requires radius in [1/1024,2000], duration in [1/1024,30] and bounded range",ability.id));
                }
                (radius_key,"point")
            },
            AbilityEffect::SlowEnemy {reduction_key,duration_key}=>{
                values(reduction_key,false)?;values(duration_key,false)?;
                if ability.extras[reduction_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>1.0)
                    || ability.extras[duration_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>60.0) {
                    return Err(format!("ability '{}': slow_enemy requires reduction in [1/1024,1] and duration in [1/1024,60]",ability.id));
                }
                (reduction_key,"unit")
            },
            AbilityEffect::RestoreManaSelf {amount_key}=>{
                // Caster-local restoration does not consume the selected target.
                // A targeted spell still needs a separate, shape-checked consumer.
                let consumes_target=ability.effects.iter().any(|effect|matches!(effect,
                    AbilityEffect::Damage {..}|AbilityEffect::DrainManaEnemy {..}|AbilityEffect::HealAlly {..}|AbilityEffect::ShieldAlly {..}|AbilityEffect::CleanseAlly {}|AbilityEffect::AreaDamage {..}
                    |AbilityEffect::SlowEnemy {..}|AbilityEffect::StunEnemy {..}|AbilityEffect::ControlEnemy {..}|AbilityEffect::ForcedMoveEnemy {..}
                    |AbilityEffect::DamageOverTimeEnemy {..}|AbilityEffect::LineDamage {..}|AbilityEffect::ConeDamage {..}));
                (amount_key,if consumes_target {ability.target_type.as_str()}else{"none"})
            },
            AbilityEffect::HealSelf {amount_key}
            | AbilityEffect::SpendManaSelf {amount_key}=>(amount_key,"none"),
            AbilityEffect::AreaDamage {amount_key,radius_key,..}=>{
                values(radius_key,true)?;
                (amount_key,"point")
            }
            AbilityEffect::LineDamage {amount_key,width_key,..}=>{
                values(width_key,true)?;
                if ability.effects.len()!=1 || ability.aim_geometry.is_some()
                    || !matches!(ability.ability_type.as_str(),"active"|"ultimate") || ability.cast_type!="instant"
                    || ability.extras[width_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>5_000.) {
                    return Err(format!("ability '{}': line_damage requires one exclusive instant point effect and width in [1/1024,5000]",ability.id));
                }
                (amount_key,"point")
            }
            AbilityEffect::ConeDamage {amount_key,half_angle_key,..}=>{
                values(half_angle_key,false)?;
                if ability.effects.len()!=1 || ability.aim_geometry.is_some()
                    || !matches!(ability.ability_type.as_str(),"active"|"ultimate") || ability.cast_type!="instant"
                    || ability.extras[half_angle_key].iter().any(|v|*v<MIN_ABILITY_SCALAR || *v>std::f32::consts::FRAC_PI_2) {
                    return Err(format!("ability '{}': cone_damage requires one instant point effect and half-angle in [1/1024,pi/2]",ability.id));
                }
                (amount_key,"point")
            }
        };
        if ability.target_type!=target {return Err(format!("ability '{}': effect does not match target_type",ability.id));}
        if target!="none" && ability.levels.iter().any(|l|!l.range.is_finite() || l.range<1.0/1024.0 || l.range>10_000.0) {
            return Err(format!("ability '{}': targeted effects require positive bounded cast range",ability.id));
        }
        values(amount,false)?;
    }
    Ok(())
}

fn default_ability_max_level() -> u8 {
    4
}

/// Shared hero identity and gameplay values. Keep authoritative whole-number
/// attributes as integers; Unreal converts them only at its rendering boundary.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeroDefinition {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub portrait: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub tombstone: bool,
    #[serde(default)]
    pub abilities: Vec<String>,
    /// MOBA-only birth loadout; legacy Story and TD initialization is unchanged.
    #[serde(default)]
    pub moba_loadout: MobaLoadout,
    #[serde(default)]
    pub strength: i32,
    #[serde(default)]
    pub agility: i32,
    #[serde(default)]
    pub intelligence: i32,
    #[serde(default)]
    pub primary_attribute: String,
    #[serde(default)]
    pub attack_range: f32,
    #[serde(default)]
    pub base_damage: i32,
    #[serde(default)]
    pub base_armor: f32,
    #[serde(default)]
    pub base_hp: i32,
    #[serde(default)]
    pub base_mana: i32,
    #[serde(default)]
    pub move_speed: f32,
    #[serde(default)]
    pub turn_speed: f32,
    /// Seconds between attacks at no attack speed bonus (Dota AttackRate). None keeps the
    /// runtime's legacy interval for heroes without source stats.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_attack_time: Option<f32>,
    /// Base HP and mana per second before attributes (Dota StatusHealthRegen / StatusManaRegen).
    #[serde(default)]
    pub health_regen: f32,
    #[serde(default)]
    pub mana_regen: f32,
    /// Auto-acquisition range (Dota AttackAcquisitionRange). None uses the game rules' hero default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition_range: Option<f32>,
    #[serde(default)]
    pub render: Option<HeroRender>,
    #[serde(default)]
    pub level_growth: HeroLevelGrowth,
    /// The source-site hero this hero is (`history_nobu@<commit>:B01`). Only heroes with a
    /// source_ref that names a hero of the generated source roster are playable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    /// Not a source hero: compiled for automated tests and fixtures only. Never offered for
    /// selection, never seated by a bot or a match recipe.
    #[serde(default)]
    pub test_only: bool,
    /// A test_only hero built from a source hero's v3 document (scripts/v3_test_heroes.lua): the
    /// roster id whose unit stats it uses, attribute conversion included. It stays unplayable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_stats: Option<String>,
}

/// Q, W, E, R. Every hero has these slots.
pub const LEGACY_HERO_ABILITY_SLOTS: usize = 4;
/// Plus the optional extra action slot (D, G-3), e.g. a passive or a level-gated ultimate.
pub const HERO_ABILITY_SLOTS: usize = 5;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct MobaLoadout {
    /// Birth rank per slot. Content may list four (the extra slot is then unlearned) or five.
    #[serde(deserialize_with = "four_or_five_ranks")]
    pub ranks: [u8; HERO_ABILITY_SLOTS],
    pub skill_points: u8,
}

impl Default for MobaLoadout {
    fn default() -> Self { Self { ranks: [1, 1, 1, 1, 0], skill_points: 0 } }
}

fn four_or_five_ranks<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<[u8; HERO_ABILITY_SLOTS], D::Error> {
    let ranks = Vec::<u8>::deserialize(deserializer)?;
    if !(LEGACY_HERO_ABILITY_SLOTS..=HERO_ABILITY_SLOTS).contains(&ranks.len()) {
        return Err(serde::de::Error::custom(format!("moba_loadout.ranks needs {LEGACY_HERO_ABILITY_SLOTS} or {HERO_ABILITY_SLOTS} entries, got {}", ranks.len())));
    }
    let mut padded = [0; HERO_ABILITY_SLOTS];
    padded[..ranks.len()].copy_from_slice(&ranks);
    Ok(padded)
}

/// Both Rust and Unreal generators validate the same MOBA-only birth rules.
pub fn validate_moba_loadout<'a>(hero: &HeroDefinition, abilities: impl IntoIterator<Item = &'a AbilityDefinition>) -> Result<(), String> {
    if hero.tombstone { return Ok(()); }
    if hero.abilities.len() > HERO_ABILITY_SLOTS { return Err(format!("hero '{}': MOBA loadout exceeds {HERO_ABILITY_SLOTS} slots",hero.id)); }
    let definitions: std::collections::BTreeMap<_,_> = abilities.into_iter().map(|a| (a.id.as_str(),a)).collect();
    for (slot,id) in hero.abilities.iter().enumerate() {
        let rank = hero.moba_loadout.ranks[slot];
        let ability = definitions.get(id.as_str()).filter(|a| !a.tombstone)
            .ok_or_else(||format!("hero '{}': unknown MOBA ability '{id}'",hero.id))?;
        validate_ability_progression(ability)?;
        if rank > ability.max_level || (rank > 0 && ability.levels[usize::from(rank-1)].required_hero_level > 1) {
            return Err(format!("hero '{}': invalid MOBA birth rank {rank} for '{id}'",hero.id));
        }
    }
    if hero.moba_loadout.ranks[hero.abilities.len()..].iter().any(|rank| *rank > 1) {
        return Err(format!("hero '{}': MOBA loadout assigns rank to an empty slot",hero.id));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroLevelGrowth {
    #[serde(default)]
    pub strength_per_level: f32,
    #[serde(default)]
    pub agility_per_level: f32,
    #[serde(default)]
    pub intelligence_per_level: f32,
    #[serde(default)]
    pub damage_per_level: f32,
    #[serde(default)]
    pub hp_per_level: f32,
    #[serde(default)]
    pub mana_per_level: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroRender {
    #[serde(default)]
    pub render_mode: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub texture: String,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub pitch_offset_deg: f32,
    #[serde(default)]
    pub roll_offset_deg: f32,
    #[serde(default)]
    pub yaw_offset_deg: f32,
    #[serde(default)]
    pub z_offset: f32,
    #[serde(default)]
    pub muzzle_bone: String,
    #[serde(default)]
    pub animation_sources: std::collections::BTreeMap<String, HeroAnimationSource>,
    #[serde(default)]
    pub animations: std::collections::BTreeMap<String, HeroAnimationBinding>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroAnimationSource {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub animation: String,
    #[serde(default)]
    pub duration_ticks: f32,
    #[serde(default)]
    pub ticks_per_second: f32,
    #[serde(default)]
    pub timeline_offset_ticks: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HeroAnimationBinding {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub start_tick: f32,
    #[serde(default)]
    pub repeat_start_tick: f32,
    #[serde(default)]
    pub impact_tick: Option<f32>,
    #[serde(default)]
    pub end_tick: f32,
    #[serde(default, rename = "loop")]
    pub loop_animation: bool,
}

/// Validated authoring ticks converted once; runtimes never evaluate Lua.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeroAnimationClipTiming {
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub impact_seconds: Option<f32>,
}

impl HeroAnimationBinding {
    pub fn clip_timing(&self, source: &HeroAnimationSource) -> Result<HeroAnimationClipTiming, String> {
        if !source.ticks_per_second.is_finite() || source.ticks_per_second <= 0.0
            || !self.start_tick.is_finite() || !self.end_tick.is_finite()
            || self.start_tick < 0.0 || self.end_tick <= self.start_tick {
            return Err("invalid clip timing".into());
        }
        if let Some(impact) = self.impact_tick {
            if !impact.is_finite() || impact <= self.start_tick || impact >= self.end_tick || self.loop_animation {
                return Err("impact_tick requires a non-looping clip and must be strictly inside its bounds".into());
            }
        }
        let timing = HeroAnimationClipTiming {
            start_seconds: self.start_tick / source.ticks_per_second,
            end_seconds: self.end_tick / source.ticks_per_second,
            impact_seconds: self.impact_tick.map(|tick| tick / source.ticks_per_second),
        };
        if !timing.start_seconds.is_finite() || !timing.end_seconds.is_finite()
            || timing.end_seconds <= timing.start_seconds
            || timing.impact_seconds.is_some_and(|impact| !impact.is_finite()
                || impact <= timing.start_seconds || impact >= timing.end_seconds) {
            return Err("clip timing cannot be represented as seconds".into());
        }
        Ok(timing)
    }
}

pub fn validate_hero_animation_clips(hero: &HeroDefinition) -> Result<(), String> {
    if hero.tombstone { return Ok(()); }
    if let Some(render) = &hero.render {
        for (slot, binding) in &render.animations {
            let source = render.animation_sources.get(&binding.source)
                .ok_or_else(|| format!("hero '{}' animation '{}' references missing source '{}'", hero.id, slot, binding.source))?;
            binding.clip_timing(source).map_err(|error| format!("hero '{}' animation '{}': {error}", hero.id, slot))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentId {
    pub id: String,
    pub tombstone: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeroId {
    pub id: String,
    pub tombstone: bool,
    pub abilities: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentCatalog {
    pub heroes: Vec<HeroId>,
    pub abilities: Vec<ContentId>,
}

impl ContentCatalog {
    pub fn validate(&self) -> Result<(), String> {
        let mut hero_ids = BTreeSet::new();
        for hero in &self.heroes {
            validate_id("hero", &hero.id)?;
            if !hero_ids.insert(hero.id.as_str()) {
                return Err(format!("duplicate hero id '{}'", hero.id));
            }
        }

        let mut ability_ids = BTreeSet::new();
        let mut active_abilities = BTreeSet::new();
        for ability in &self.abilities {
            validate_id("ability", &ability.id)?;
            if !ability_ids.insert(ability.id.as_str()) {
                return Err(format!("duplicate ability id '{}'", ability.id));
            }
            if !ability.tombstone {
                active_abilities.insert(ability.id.as_str());
            }
        }

        for hero in self.heroes.iter().filter(|hero| !hero.tombstone) {
            let mut seen = BTreeSet::new();
            for ability in &hero.abilities {
                if !active_abilities.contains(ability.as_str()) {
                    return Err(format!(
                        "hero '{}' references missing or tombstoned ability '{}'",
                        hero.id, ability
                    ));
                }
                if !seen.insert(ability.as_str()) {
                    return Err(format!(
                        "hero '{}' references ability '{}' more than once",
                        hero.id, ability
                    ));
                }
            }
        }
        Ok(())
    }

    /// Stable across platforms. Declaration order matters because numeric IDs use it.
    pub fn identity_hash(&self) -> Result<String, String> {
        self.validate()?;
        let mut hash = 0xcbf29ce484222325_u64;
        hash_field(&mut hash, "content-catalog-v1");
        for hero in &self.heroes {
            hash_field(&mut hash, "hero");
            hash_field(&mut hash, &hero.id);
            hash_field(&mut hash, if hero.tombstone { "1" } else { "0" });
            for ability in &hero.abilities {
                hash_field(&mut hash, ability);
            }
            hash_field(&mut hash, "end-hero");
        }
        for ability in &self.abilities {
            hash_field(&mut hash, "ability");
            hash_field(&mut hash, &ability.id);
            hash_field(&mut hash, if ability.tombstone { "1" } else { "0" });
        }
        Ok(format!("{hash:016x}"))
    }
}

fn validate_id(kind: &str, id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!(
            "invalid {kind} id '{id}': use lowercase ASCII, digits, and '_'"
        ));
    }
    Ok(())
}

fn hash_field(hash: &mut u64, field: &str) {
    for byte in (field.len() as u64)
        .to_le_bytes()
        .into_iter()
        .chain(field.bytes())
    {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

/// Hash the complete evaluated `templates.lua` value, including gameplay and
/// presentation fields. `serde_json::Value` sorts object keys deterministically.
pub fn canonical_template_hash(value: &serde_json::Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("serialize template catalog: {error}"))?;
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in b"omoba-template-v1".iter().chain(bytes.iter()) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("{hash:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_clip_timing_preserves_impact_and_rejects_invalid_authoring() {
        let source = HeroAnimationSource { ticks_per_second: 30.0, ..Default::default() };
        let binding = HeroAnimationBinding { source: "strike".into(), start_tick: 3.0,
            impact_tick: Some(9.0), end_tick: 24.0, ..Default::default() };
        let timing = binding.clip_timing(&source).unwrap();
        assert_eq!(timing.start_seconds, 0.1);
        assert_eq!(timing.impact_seconds, Some(0.3));
        assert_eq!(timing.end_seconds, 0.8);
        let mut optional = binding.clone();
        optional.impact_tick = None;
        optional.loop_animation = true;
        assert_eq!(optional.clip_timing(&source).unwrap().impact_seconds, None);
        for impact in [f32::NAN, f32::INFINITY, -1.0, 3.0, 24.0, 25.0] {
            let mut invalid = binding.clone();
            invalid.impact_tick = Some(impact);
            assert!(invalid.clip_timing(&source).is_err());
        }
        let mut invalid = binding.clone();
        invalid.loop_animation = true;
        assert!(invalid.clip_timing(&source).is_err());
        for fps in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::MIN_POSITIVE / 100.0] {
            assert!(binding.clip_timing(&HeroAnimationSource { ticks_per_second: fps, ..Default::default() }).is_err());
        }
        let mut hero: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"arbitrary",
            "render":{"animation_sources":{"strike":{"ticks_per_second":30}},
                "animations":{"attack":{"source":"strike","start_tick":3,"end_tick":24,"impact_tick":9}}}})).unwrap();
        validate_hero_animation_clips(&hero).unwrap();
        hero.render.as_mut().unwrap().animations.get_mut("attack").unwrap().source = "missing".into();
        assert!(validate_hero_animation_clips(&hero).unwrap_err().contains("arbitrary"));
        hero.tombstone = true;
        validate_hero_animation_clips(&hero).unwrap();
    }

    fn catalog() -> ContentCatalog {
        ContentCatalog {
            heroes: vec![HeroId {
                id: "first_hero".into(),
                tombstone: false,
                abilities: vec!["first_skill".into()],
            }],
            abilities: vec![ContentId {
                id: "first_skill".into(),
                tombstone: false,
            }],
        }
    }

    #[test]
    fn rejects_missing_or_tombstoned_ability() {
        let mut c = catalog();
        c.abilities[0].tombstone = true;
        assert!(c.validate().unwrap_err().contains("first_hero"));
        c.abilities[0].tombstone = false;
        c.heroes[0].abilities[0] = "missing".into();
        assert!(c.validate().unwrap_err().contains("missing"));
    }

    #[test]
    fn rejects_duplicate_ids_and_hero_slots() {
        let mut c = catalog();
        c.abilities.push(c.abilities[0].clone());
        assert!(c.validate().unwrap_err().contains("duplicate ability"));
        c.abilities.pop();
        c.heroes[0].abilities.push("first_skill".into());
        assert!(c.validate().unwrap_err().contains("more than once"));
    }

    #[test]
    fn moba_loadout_allows_late_first_learning_but_not_prelearned_locked_rank() {
        let mut hero: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"mage","abilities":["ultimate"],
            "moba_loadout":{"ranks":[0,0,0,0],"skill_points":1}})).unwrap();
        let ability: AbilityDefinition = serde_json::from_value(serde_json::json!({"id":"ultimate","max_level":1,
            "levels":[{"required_hero_level":6}]})).unwrap();
        validate_moba_loadout(&hero,[&ability]).unwrap();
        hero.moba_loadout.ranks[0]=1;
        assert!(validate_moba_loadout(&hero,[&ability]).is_err());
        hero.moba_loadout.ranks[0]=2;
        assert!(validate_moba_loadout(&hero,[&ability]).is_err());
        assert!(serde_json::from_value::<MobaLoadout>(serde_json::json!({"rank":[0,0,0,0]})).is_err());
    }

    #[test]
    fn moba_birth_loadout_is_explicit_and_legacy_defaults_are_preserved() {
        let legacy: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"mage"})).unwrap();
        assert_eq!(legacy.moba_loadout, MobaLoadout::default());
        let unlearned: HeroDefinition = serde_json::from_value(serde_json::json!({"id":"mage",
            "moba_loadout":{"ranks":[0,0,0,0],"skill_points":1}})).unwrap();
        assert_eq!(unlearned.moba_loadout.ranks, [0;5], "four ranks leave the extra slot unlearned");
        let five: MobaLoadout = serde_json::from_value(serde_json::json!({"ranks":[1,1,1,1,1]})).unwrap();
        assert_eq!(five.ranks, [1;5]);
        assert!(serde_json::from_value::<MobaLoadout>(serde_json::json!({"ranks":[0,0,0,0,0,0]})).is_err());
        assert_eq!(unlearned.moba_loadout.skill_points, 1);
        for ranks in [serde_json::json!([0,0,0]),serde_json::json!([0,0,0,256]),serde_json::json!([0,0,0,-1])] {
            assert!(serde_json::from_value::<MobaLoadout>(serde_json::json!({"ranks":ranks})).is_err());
        }
    }

    #[test]
    fn declaration_order_affects_identity_hash() {
        let mut c = catalog();
        let original = c.identity_hash().unwrap();
        c.abilities.push(ContentId {
            id: "second_skill".into(),
            tombstone: false,
        });
        assert_ne!(original, c.identity_hash().unwrap());
        c.abilities.swap(0, 1);
        assert_ne!(original, c.identity_hash().unwrap());
    }

    #[test]
    fn template_hash_is_stable_for_object_key_order_but_changes_with_values() {
        let a = serde_json::json!({"hero": {"hp": 500, "name": "Mage"}, "abilities": ["spark"]});
        let b = serde_json::json!({"abilities": ["spark"], "hero": {"name": "Mage", "hp": 500}});
        let c = serde_json::json!({"abilities": ["spark"], "hero": {"name": "Mage", "hp": 501}});
        assert_eq!(canonical_template_hash(&a), canonical_template_hash(&b));
        assert_ne!(canonical_template_hash(&a), canonical_template_hash(&c));
    }

    #[test]
    fn shared_ability_schema_preserves_levels_extras_and_default_max_level() {
        let input = serde_json::json!({
            "id": "arc_bolt",
            "display_name": "Arc Bolt",
            "ability_type": "active",
            "cast_type": "instant",
            "target_type": "unit",
            "levels": [{"cooldown": 8.0, "mana_cost": 25.0, "range": 600.0}],
            "extras": {"damage": [80.0, 120.0]}
        });
        let ability: AbilityDefinition = serde_json::from_value(input).unwrap();
        assert_eq!(ability.max_level, 4);
        assert_eq!(ability.levels[0].range, 600.0);
        assert_eq!(ability.levels[0].required_hero_level, 1);
        assert_eq!(ability.extras["damage"], vec![80.0, 120.0]);
        let roundtrip = serde_json::to_value(&ability).unwrap();
        assert_eq!(roundtrip["target_type"], "unit");
        assert_eq!(roundtrip["levels"][0]["mana_cost"], 25.0);
    }

    #[test]
    fn ability_progression_requires_bounded_monotonic_complete_ranks() {
        let make = |requirements: &[u8]| -> AbilityDefinition {
            serde_json::from_value(serde_json::json!({"id":"progression", "max_level":requirements.len(),
                "levels":requirements.iter().map(|v| serde_json::json!({"required_hero_level":v})).collect::<Vec<_>>() })).unwrap()
        };
        assert!(validate_ability_progression(&make(&[1,6,11,16])).is_ok());
        for values in [vec![],vec![0],vec![26],vec![6,5]] { assert!(validate_ability_progression(&make(&values)).is_err()); }
        let mut incomplete = make(&[1,2]); incomplete.levels.pop();
        assert!(validate_ability_progression(&incomplete).is_err());
    }

    #[test]
    fn authored_numeric_envelope_checks_every_rank_and_field() {
        let valid: AbilityDefinition = serde_json::from_value(serde_json::json!({
            "id":"numeric", "max_level":2, "levels":[{},{}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for field in ["cooldown", "mana_cost", "cast_time", "range"] {
            for value in [-1.0, 0.0001, 1_000_001.0, f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
                let mut bad=valid.clone();
                let level=&mut bad.levels[1];
                match field {"cooldown"=>level.cooldown=value,"mana_cost"=>level.mana_cost=value,
                    "cast_time"=>level.cast_time=value,_=>level.range=value}
                let error=validate_ability_progression(&bad).unwrap_err();
                assert!(error.contains("numeric") && error.contains("rank 2") && error.contains(field),"{error}");
            }
        }
        let mut boundaries=valid.clone();
        boundaries.levels[0].cooldown=MIN_ABILITY_SCALAR;
        boundaries.levels[1].mana_cost=MAX_ABILITY_SCALAR;
        assert!(validate_ability_progression(&boundaries).is_ok());
        let rounded: AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"rounded", "max_level":1,
            "levels":[{"cooldown":0.00097656249,"mana_cost":1000000.01,"range":10000.0001}]
        })).unwrap();
        assert!(validate_ability_progression(&rounded).is_ok());
        // Deleted content does not need valid runtime data.
        boundaries.tombstone=true;boundaries.levels[1].range=f32::NAN;
        assert!(validate_ability_progression(&boundaries).is_ok());
    }

    #[test]
    fn dash_effect_requires_exclusive_point_and_all_rank_ranges() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"dash", "max_level":2,"ability_type":"active","cast_type":"instant",
            "target_type":"point","levels":[{"range":450},{"range":600}],
            "effects":[{"kind":"dash_to_point"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for range in [0.0,0.0001,10_001.0,f32::INFINITY,f32::NAN] {
            let mut bad=valid.clone();bad.levels[1].range=range;
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.target_type="none".into();
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.effects.push(AbilityEffect::DashToPoint);
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid;bad.effects.push(AbilityEffect::AreaDamage {
            amount_key:"damage".into(),radius_key:"radius".into(),damage_kind:EffectDamageKind::Physical});
        assert!(validate_ability_progression(&bad).unwrap_err().contains("exclusive"));
    }

    #[test]
    fn control_effect_validates_closed_status_and_rank_contract() {
        for control in ["stun","root","silence"] {
            let source=serde_json::json!({"id":"control", "max_level":2,
                "ability_type":"active","cast_type":"instant","target_type":"unit",
                "levels":[{"range":300},{"range":350}],"extras":{"duration":[0.75,1.5]},
                "effects":[{"kind":"control_enemy","control":control,"duration_key":"duration"}]});
            let valid:AbilityDefinition=serde_json::from_value(source.clone()).unwrap();
            assert!(validate_ability_progression(&valid).is_ok());
            for value in [0.0,-1.0,0.0001,60.1,f32::NAN,f32::INFINITY] {
                let mut bad=valid.clone();bad.extras.get_mut("duration").unwrap()[1]=value;
                assert!(validate_ability_progression(&bad).is_err());
            }
            let mut bad=valid.clone();bad.extras.get_mut("duration").unwrap().pop();
            assert!(validate_ability_progression(&bad).is_err());
            let mut bad=valid.clone();bad.levels[1].range=0.0;
            assert!(validate_ability_progression(&bad).is_err());
            for target in ["none","point"] {
                let mut bad=valid.clone();bad.target_type=target.into();
                assert!(validate_ability_progression(&bad).is_err());
            }
            for control in [serde_json::json!("burn"),serde_json::json!("Root"),serde_json::json!(0),serde_json::Value::Null] {
                let mut bad=source.clone();bad["effects"][0]["control"]=control;
                assert!(serde_json::from_value::<AbilityDefinition>(bad).is_err());
            }
        }
    }

    #[test]
    fn stun_effect_validates_duration_range_and_enemy_allegiance() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"control", "max_level":2,"ability_type":"active","cast_type":"instant",
            "target_type":"unit","levels":[{"range":300},{"range":350}],
            "extras":{"duration":[0.75,1.5]},
            "effects":[{"kind":"stun_enemy","duration_key":"duration"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for value in [0.0,-1.0,0.0001,60.1,f32::NAN,f32::INFINITY] {
            let mut bad=valid.clone();bad.extras.get_mut("duration").unwrap()[1]=value;
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.extras.get_mut("duration").unwrap().pop();
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.levels[1].range=0.0;
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid;bad.target_type="point".into();
        assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn slow_effect_validates_rank_reduction_duration_range_and_target() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"slow", "max_level":2,"ability_type":"active","cast_type":"instant",
            "target_type":"unit","levels":[{"range":450},{"range":600}],
            "extras":{"reduction":[0.25,0.5],"duration":[2,3]},
            "effects":[{"kind":"slow_enemy","reduction_key":"reduction","duration_key":"duration"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for (key,values) in [("reduction",vec![0.0,1.01,f32::NAN]),("duration",vec![0.0,60.1,f32::INFINITY])] {
            for value in values {
                let mut bad=valid.clone();bad.extras.get_mut(key).unwrap()[1]=value;
                assert!(validate_ability_progression(&bad).is_err());
            }
        }
        let mut bad=valid.clone();bad.levels[1].range=0.0;
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.remove("duration");
        assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid;bad.target_type="point".into();
        assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn unit_target_allegiance_follows_validated_effects_and_rejects_conflicts() {
        assert_eq!(unit_target_allegiance(&[AbilityEffect::HealAlly { amount_key: "heal".into() }]).unwrap(), UnitTargetAllegiance::Ally);
        for effect in [
            AbilityEffect::Damage { amount_key: "damage".into(), damage_kind: EffectDamageKind::Magical },
            AbilityEffect::SlowEnemy { reduction_key: "reduction".into(), duration_key: "duration".into() },
            AbilityEffect::StunEnemy { duration_key: "duration".into() },
            AbilityEffect::ControlEnemy { control: ControlEffectKind::Root, duration_key: "duration".into() },
            AbilityEffect::ControlEnemy { control: ControlEffectKind::Silence, duration_key: "duration".into() },
        ] {
            assert_eq!(unit_target_allegiance(&[effect]).unwrap(), UnitTargetAllegiance::Enemy);
        }
        assert_eq!(unit_target_allegiance(&[]).unwrap(), UnitTargetAllegiance::Unspecified);
        assert_eq!(unit_target_allegiance(&[AbilityEffect::HealSelf { amount_key: "heal".into() }]).unwrap(), UnitTargetAllegiance::Unspecified);
        assert_eq!(unit_target_allegiance(&[AbilityEffect::DashToPoint]).unwrap(), UnitTargetAllegiance::Unspecified);
        assert_eq!(unit_target_allegiance(&[AbilityEffect::AreaDamage {
            amount_key: "damage".into(), radius_key: "radius".into(), damage_kind: EffectDamageKind::Physical,
        }]).unwrap(), UnitTargetAllegiance::Unspecified);
        assert_eq!(unit_target_allegiance(&[
            AbilityEffect::HealAlly { amount_key: "heal".into() },
            AbilityEffect::AreaDamage { amount_key: "damage".into(), radius_key: "radius".into(), damage_kind: EffectDamageKind::Physical },
        ]).unwrap(), UnitTargetAllegiance::Ally);
        assert_eq!(
            unit_target_allegiance(&[
                AbilityEffect::HealAlly { amount_key: "heal".into() },
                AbilityEffect::Damage { amount_key: "damage".into(), damage_kind: EffectDamageKind::Pure },
            ]).unwrap_err(),
            "conflicting unit target allegiance",
        );
        assert_eq!(
            (UnitTargetAllegiance::Ally.as_str(), UnitTargetAllegiance::Enemy.as_str(), UnitTargetAllegiance::Unspecified.as_str()),
            ("ally", "enemy", "unspecified"),
        );
    }

    #[test]
    fn unit_effects_reject_conflicting_target_allegiance_in_either_order() {
        let valid: AbilityDefinition = serde_json::from_value(serde_json::json!({
            "id":"ally_aid", "max_level":1,"ability_type":"active","cast_type":"instant",
            "target_type":"unit","levels":[{"range":600}],
            "extras":{"heal":[140],"damage":[20],"reduction":[0.25],"duration":[2]},
            "effects":[{"kind":"heal_ally","amount_key":"heal"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for hostile in [
            AbilityEffect::Damage {amount_key:"damage".into(),damage_kind:EffectDamageKind::Physical},
            AbilityEffect::SlowEnemy {reduction_key:"reduction".into(),duration_key:"duration".into()},
            AbilityEffect::StunEnemy {duration_key:"duration".into()},
            AbilityEffect::ControlEnemy {control:ControlEffectKind::Root,duration_key:"duration".into()},
            AbilityEffect::ControlEnemy {control:ControlEffectKind::Silence,duration_key:"duration".into()},
        ] {
            let mut bad=valid.clone();bad.effects.push(hostile);
            for _ in 0..2 {
                assert!(validate_ability_progression(&bad).unwrap_err().contains("conflicting unit target allegiance"));
                bad.effects.reverse();
            }
        }
    }

    #[test]
    fn authored_numeric_envelope_bounds_only_executable_effect_extras() {
        let mut ability: AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"numeric_heal","max_level":1,"ability_type":"active","cast_type":"instant",
            "target_type":"none","levels":[{}],"extras":{"heal":[0],"unused":[-5]},
            "effects":[{"kind":"heal_self","amount_key":"heal"}]
        })).unwrap();
        assert!(validate_ability_progression(&ability).is_ok());
        for value in [MIN_ABILITY_SCALAR,MAX_ABILITY_SCALAR] {
            ability.extras.get_mut("heal").unwrap()[0]=value;
            assert!(validate_ability_progression(&ability).is_ok());
        }
        for value in [0.0001,1_000_001.0,f32::INFINITY,f32::NAN] {
            ability.extras.get_mut("heal").unwrap()[0]=value;
            assert!(validate_ability_progression(&ability).unwrap_err().contains("heal"));
        }
    }

    #[test]
    fn area_effects_validate_shared_target_rank_data_and_limits() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"area_fixture","ability_type":"active","cast_type":"instant","target_type":"point","max_level":1,
            "levels":[{"range":700}],"extras":{"damage":[110],"radius":[220]},
            "effects":[{"kind":"area_damage","amount_key":"damage","radius_key":"radius","damage_kind":"physical"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for radius in [0.0,-1.0,0.0001,10_001.0,f32::NAN] {
            let mut bad=valid.clone();bad.extras.insert("radius".into(),vec![radius]);
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.target_type="unit".into();assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.remove("damage");assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.levels[0].range=0.0;assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.effects=vec![bad.effects[0].clone();33];assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn cast_preflight_unit_effects_require_representable_per_rank_range() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"range_fixture","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":2,
            "levels":[{"range":600},{"range":700}],"extras":{"damage":[80,125]},
            "effects":[{"kind":"damage","amount_key":"damage","damage_kind":"magical"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for range in [0.0,-1.0,0.0001,10_001.0,f32::INFINITY,f32::NAN] {
            let mut bad=valid.clone();bad.levels[1].range=range;
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut heal=valid;heal.target_type="none".into();
        heal.effects=vec![AbilityEffect::HealSelf {amount_key:"damage".into()}];
        for rank in &mut heal.levels {rank.range=0.0;}
        assert!(validate_ability_progression(&heal).is_ok());
    }

    #[test]
    fn mana_buff_declaration_validates_whitelist_signed_values_duration_and_duplicate_stats() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"buff_fixture","ability_type":"active","cast_type":"instant","target_type":"none","max_level":2,
            "levels":[{},{}],"extras":{"value":[-1,16],"duration":[1.0/1024.0,60]},
            "effects":[{"kind":"mana_buff_self","stat":"mana_regen_percentage","value_key":"value","duration_key":"duration"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        for value in [-1.1,16.1,0.0001,f32::INFINITY,f32::NAN] {
            let mut bad=valid.clone();bad.extras.insert("value".into(),vec![value;2]);
            assert!(validate_ability_progression(&bad).is_err());
        }
        for duration in [0.0,-1.0,60.1,0.0001] {
            let mut bad=valid.clone();bad.extras.insert("duration".into(),vec![duration;2]);
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.effects.push(bad.effects[0].clone());assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.remove("duration");assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.target_type="unit".into();assert!(validate_ability_progression(&bad).is_err());
        assert!(serde_json::from_value::<AbilityEffect>(serde_json::json!({"kind":"mana_buff_self","stat":"damage",
            "value_key":"value","duration_key":"duration"})).is_err());
    }

    #[test]
    fn mana_drain_schema_checks_enemy_target_and_rank_envelopes() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({"id":"arbitrary_loss","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":2,
            "levels":[{"range":600},{"range":600}],"extras":{"amount":[20,30]},"effects":[{"kind":"drain_mana_enemy","amount_key":"amount"}]})).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());assert_eq!(unit_target_allegiance(&valid.effects).unwrap(),UnitTargetAllegiance::Enemy);
        let mut edge=valid.clone();edge.extras.insert("amount".into(),vec![MIN_ABILITY_SCALAR,1_000_000.]);assert!(validate_ability_progression(&edge).is_ok());
        for value in [0.,-1.,0.0001,1_000_001.,f32::NAN,f32::INFINITY]{let mut bad=valid.clone();bad.extras.get_mut("amount").unwrap()[1]=value;assert!(validate_ability_progression(&bad).is_err());}
        for target in ["none","point"]{let mut bad=valid.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());}
        for range in [0.,-1.,10_001.,f32::NAN]{let mut bad=valid.clone();bad.levels[1].range=range;assert!(validate_ability_progression(&bad).is_err());}
        let mut bad=valid.clone();bad.extras.clear();assert!(validate_ability_progression(&bad).is_err());
        for reverse in [false,true]{let mut bad=valid.clone();bad.effects.push(AbilityEffect::HealAlly{amount_key:"amount".into()});if reverse{bad.effects.reverse();}assert!(validate_ability_progression(&bad).is_err());}
    }

    #[test]
    fn ally_cleanse_schema_requires_closed_friendly_unit_range_and_composes() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({"id":"arbitrary_cleanse","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":2,
            "levels":[{"range":600},{"range":600}],"extras":{"mana":[10,15]},"effects":[{"kind":"cleanse_ally"},{"kind":"restore_mana_self","amount_key":"mana"}]})).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());assert_eq!(unit_target_allegiance(&valid.effects).unwrap(),UnitTargetAllegiance::Ally);
        let mut reverse=valid.clone();reverse.effects.reverse();assert!(validate_ability_progression(&reverse).is_ok());
        let mut alone=valid.clone();alone.effects.truncate(1);alone.extras.clear();assert!(validate_ability_progression(&alone).is_ok());
        for range in [0.,-1.,0.0001,10_001.,f32::NAN,f32::INFINITY]{let mut bad=valid.clone();bad.levels[1].range=range;assert!(validate_ability_progression(&bad).is_err());}
        for target in ["none","point"]{let mut bad=valid.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());}
        let mut bad=valid.clone();bad.effects.push(AbilityEffect::Damage {amount_key:"mana".into(),damage_kind:EffectDamageKind::Magical});assert!(validate_ability_progression(&bad).is_err());
        assert!(serde_json::from_value::<AbilityEffect>(serde_json::json!({"kind":"cleanse_ally","buffs":["anything"]})).is_err());
    }

    #[test]
    fn ally_shield_schema_preserves_friendly_target_and_positive_rank_envelopes() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({"id":"arbitrary_guard","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":2,
            "levels":[{"range":600},{"range":600}],"extras":{"amount":[70,100],"duration":[2,2],"mana":[10,15]},
            "effects":[{"kind":"shield_ally","amount_key":"amount","duration_key":"duration"},{"kind":"restore_mana_self","amount_key":"mana"}]})).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());assert_eq!(unit_target_allegiance(&valid.effects).unwrap(),UnitTargetAllegiance::Ally);
        let mut edge=valid.clone();edge.extras.insert("amount".into(),vec![MIN_ABILITY_SCALAR,1_000_000.]);edge.extras.insert("duration".into(),vec![MIN_ABILITY_SCALAR,60.]);
        edge.levels[0].range=MIN_ABILITY_SCALAR;edge.levels[1].range=10_000.;assert!(validate_ability_progression(&edge).is_ok());
        let mut reverse=valid.clone();reverse.effects.reverse();assert!(validate_ability_progression(&reverse).is_ok());
        for (key,values) in [("amount",vec![0.,100.]),("amount",vec![-1.,100.]),("amount",vec![70.,1000001.]),
            ("duration",vec![2.,0.]),("duration",vec![2.,60.01]),("duration",vec![2.,f32::NAN])] {
            let mut bad=valid.clone();bad.extras.insert(key.into(),values);assert!(validate_ability_progression(&bad).is_err());}
        for target in ["none","point"]{let mut bad=valid.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());}
        for key in ["amount","duration"]{let mut bad=valid.clone();bad.extras.remove(key);assert!(validate_ability_progression(&bad).is_err());}
        let mut bad=valid.clone();bad.levels[1].range=0.;assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.effects.push(AbilityEffect::Damage {amount_key:"amount".into(),damage_kind:EffectDamageKind::Magical});
        assert!(validate_ability_progression(&bad).is_err());bad.effects.reverse();assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn composed_mana_requires_a_valid_target_consumer_and_preserves_self_only_ops() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"arbitrary_support","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":2,
            "levels":[{"range":600},{"range":600}],"extras":{"heal":[140,210],"resource":[10,15]},
            "effects":[{"kind":"heal_ally","amount_key":"heal"},{"kind":"restore_mana_self","amount_key":"resource"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        let mut reversed=valid.clone();reversed.effects.reverse();assert!(validate_ability_progression(&reversed).is_ok());
        assert_eq!(unit_target_allegiance(&valid.effects).unwrap(),UnitTargetAllegiance::Ally);
        let mut bad=valid.clone();bad.effects.remove(0);assert!(validate_ability_progression(&bad).is_err());
        for target in ["none","point"] {let mut bad=valid.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());}
        let mut bad=valid.clone();bad.levels[1].range=0.;assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.remove("resource");assert!(validate_ability_progression(&bad).is_err());
        for kind in ["spend_mana_self","heal_self"] {
            let mut bad=valid.clone();bad.effects[1]=serde_json::from_value(serde_json::json!({"kind":kind,"amount_key":"resource"})).unwrap();
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut enemy=valid.clone();enemy.effects[0]=serde_json::from_value(serde_json::json!({"kind":"damage","amount_key":"heal","damage_kind":"magical"})).unwrap();
        assert!(validate_ability_progression(&enemy).is_ok());
        let mut area=enemy.clone();area.target_type="point".into();area.extras.insert("radius".into(),vec![100.;2]);
        area.effects[0]=serde_json::from_value(serde_json::json!({"kind":"area_damage","amount_key":"heal","radius_key":"radius","damage_kind":"magical"})).unwrap();
        assert!(validate_ability_progression(&area).is_ok());
    }

    #[test]
    fn mana_effect_schema_validates_both_resource_ops_and_rank_envelope() {
        let valid:AbilityDefinition=serde_json::from_value(serde_json::json!({
            "id":"mana_fixture","ability_type":"active","cast_type":"instant","target_type":"none","max_level":2,
            "levels":[{},{}],"extras":{"resource":[0,1000000]},
            "effects":[{"kind":"restore_mana_self","amount_key":"resource"},
                {"kind":"spend_mana_self","amount_key":"resource"}]
        })).unwrap();
        assert!(validate_ability_progression(&valid).is_ok());
        assert!(matches!(valid.effects[0],AbilityEffect::RestoreManaSelf {..}));
        assert!(matches!(valid.effects[1],AbilityEffect::SpendManaSelf {..}));
        let mut minimum=valid.clone();minimum.extras.insert("resource".into(),vec![1.0/1024.0;2]);
        assert!(validate_ability_progression(&minimum).is_ok());
        for value in [-1.0,0.0001,1000001.0,f32::INFINITY,f32::NAN] {
            let mut bad=valid.clone();bad.extras.insert("resource".into(),vec![0.0,value]);
            assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.extras.remove("resource");assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.extras.insert("resource".into(),vec![1.0]);assert!(validate_ability_progression(&bad).is_err());
        for target in ["unit","point"] {
            let mut bad=valid.clone();bad.target_type=target.into();assert!(validate_ability_progression(&bad).is_err());
        }
        let mut bad=valid.clone();bad.cast_type="channel".into();assert!(validate_ability_progression(&bad).is_err());
        let mut bad=valid.clone();bad.effects.push(AbilityEffect::DashToPoint);assert!(validate_ability_progression(&bad).is_err());
    }

    #[test]
    fn shared_hero_schema_parses_authoritative_stats_and_animation_bindings() {
        let hero: HeroDefinition = serde_json::from_value(serde_json::json!({
            "id": "arc_mage", "strength": 17, "base_hp": 560,
            "base_damage": 42, "abilities": ["arc_bolt"]
        })).unwrap();
        assert_eq!(hero.base_hp, 560);
        assert_eq!(hero.abilities, ["arc_bolt"]);
        assert!(serde_json::from_value::<HeroDefinition>(serde_json::json!({
            "id": "arc_mage", "base_hp": 560.5
        })).is_err());

        let render: HeroRender = serde_json::from_value(serde_json::json!({
            "render_mode": "model_3d",
            "animation_sources": {"idle": {"model": "idle.fbx", "animation": "Take 001"}},
            "animations": {"idle": {"source": "idle", "loop": true}}
        })).unwrap();
        assert_eq!(render.animation_sources["idle"].model, "idle.fbx");
        assert!(render.animations["idle"].loop_animation);
    }
}

#[cfg(test)]
mod projectile_delivery_tests {
    use super::*;
    fn homing() -> serde_json::Value {
        serde_json::json!({"id":"fixture_bolt","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":1,
            "levels":[{"range":700}],"extras":{"damage":[90],"speed":[900],"radius":[16],"lifetime":[3],"slow":[0.25],"slow_time":[1.5]},
            "effects":[{"kind":"damage","amount_key":"damage","damage_kind":"magical"},
                {"kind":"slow_enemy","reduction_key":"slow","duration_key":"slow_time"}],
            "projectile":{"motion":"homing","speed_key":"speed","radius_key":"radius","lifetime_key":"lifetime"}})
    }
    fn linear() -> serde_json::Value {
        serde_json::json!({"id":"fixture_lance","ability_type":"active","cast_type":"instant","target_type":"point","max_level":1,
            "levels":[{"range":1000}],"extras":{"damage":[70],"speed":[1200],"radius":[60]},
            "effects":[{"kind":"damage","amount_key":"damage","damage_kind":"magical"}],
            "projectile":{"motion":"linear","speed_key":"speed","radius_key":"radius","max_hits":3,"wall":"stop"}})
    }
    fn check(value: serde_json::Value) -> Result<(), String> {
        validate_ability_progression(&serde_json::from_value::<AbilityDefinition>(value).map_err(|e| e.to_string())?)
    }
    #[test]
    fn projectile_delivery_accepts_homing_and_linear_and_defaults_policies() {
        check(homing()).unwrap();
        check(linear()).unwrap();
        let parsed: AbilityDefinition = serde_json::from_value(homing()).unwrap();
        let delivery = parsed.projectile.unwrap();
        assert_eq!((delivery.max_hits, delivery.wall, delivery.target_lost, delivery.caster_death),
            (1, ProjectileWall::Pass, ProjectileTargetLost::Expire, ProjectileCasterDeath::Continue));
        // Linear aim preview is a swept path of the projectile diameter over the cast range.
        let preview = ability_aim_preview(&serde_json::from_value(linear()).unwrap()).unwrap();
        assert_eq!((preview.ranks[0].path_width, preview.ranks[0].range), (120.0, 1000.0));
        // Abilities without a delivery keep their exact serialized form.
        let mut plain = linear(); plain.as_object_mut().unwrap().remove("projectile");
        let plain: AbilityDefinition = serde_json::from_value(plain).unwrap();
        assert!(serde_json::to_value(plain).unwrap().get("projectile").is_none());
    }
    #[test]
    fn projectile_delivery_rejects_mismatched_or_unbounded_authoring() {
        let cases: Vec<(serde_json::Value, &str)> = vec![
            ({let mut v = homing(); v["target_type"] = "point".into(); v}, "homing needs unit"),
            ({let mut v = linear(); v["target_type"] = "unit".into(); v}, "linear needs point"),
            ({let mut v = homing(); v["projectile"].as_object_mut().unwrap().remove("lifetime_key"); v}, "homing lifetime"),
            ({let mut v = homing(); v["projectile"]["max_hits"] = 2.into(); v}, "homing hits"),
            ({let mut v = homing(); v["projectile"]["wall"] = "stop".into(); v}, "homing wall"),
            ({let mut v = linear(); v["projectile"]["max_hits"] = 65.into(); v}, "hit cap"),
            ({let mut v = linear(); v["projectile"]["max_hits"] = 0.into(); v}, "zero hits"),
            ({let mut v = linear(); v["extras"]["speed"] = serde_json::json!([0.5]); v}, "speed floor"),
            ({let mut v = linear(); v["extras"]["radius"] = serde_json::json!([1001]); v}, "radius cap"),
            ({let mut v = linear(); v["extras"]["speed"] = serde_json::json!([30]); v}, "flight over 30 s"),
            ({let mut v = linear(); v["effects"] = serde_json::json!([]); v}, "needs damage"),
            ({let mut v = linear(); v["effects"].as_array_mut().unwrap().push(serde_json::json!({"kind":"heal_self","amount_key":"damage"})); v}, "payload kinds"),
            ({let mut v = linear(); v["cast_type"] = "channeled".into(); v}, "instant only"),
            ({let mut v = linear(); v["aim_geometry"] = serde_json::json!({"kind":"swept_path","width_key":"radius"}); v}, "no aim geometry"),
            ({let mut v = linear(); v["projectile"]["bounce"] = true.into(); v}, "unknown field"),
            ({let mut v = linear(); v["projectile"]["motion"] = "arc".into(); v}, "unknown motion"),
        ];
        for (value, label) in cases { assert!(check(value).is_err(), "{label}"); }
    }
}

#[cfg(test)]
mod forced_move_tests {
    use super::*;
    fn shove() -> serde_json::Value {
        serde_json::json!({"id":"fixture_shove","ability_type":"active","cast_type":"instant","target_type":"unit","max_level":1,
            "levels":[{"range":500}],"extras":{"damage":[40],"push":[350],"push_time":[0.35]},
            "effects":[{"kind":"damage","amount_key":"damage","damage_kind":"magical"},
                {"kind":"forced_move_enemy","motion":"knockback","distance_key":"push","duration_key":"push_time"}]})
    }
    fn hook() -> serde_json::Value {
        serde_json::json!({"id":"fixture_hook","ability_type":"active","cast_type":"instant","target_type":"point","max_level":1,
            "levels":[{"range":1100}],"extras":{"damage":[60],"speed":[1600],"radius":[60],"pull":[1100],"pull_time":[0.5]},
            "effects":[{"kind":"damage","amount_key":"damage","damage_kind":"magical"},
                {"kind":"forced_move_enemy","motion":"pull","distance_key":"pull","duration_key":"pull_time"}],
            "projectile":{"motion":"linear","speed_key":"speed","radius_key":"radius","max_hits":1,"wall":"stop"}})
    }
    fn check(value: serde_json::Value) -> Result<(), String> {
        validate_ability_progression(&serde_json::from_value::<AbilityDefinition>(value).map_err(|e| e.to_string())?)
    }
    #[test]
    fn forced_move_accepts_unit_knockback_and_projectile_pull() {
        check(shove()).unwrap();
        check(hook()).unwrap();
        let parsed: AbilityDefinition = serde_json::from_value(hook()).unwrap();
        assert!(matches!(parsed.effects[1], AbilityEffect::ForcedMoveEnemy { motion: ForcedMotionKind::Pull, .. }));
        assert_eq!(unit_target_allegiance(&serde_json::from_value::<AbilityDefinition>(shove()).unwrap().effects), Ok(UnitTargetAllegiance::Enemy));
    }
    #[test]
    fn forced_move_rejects_unbounded_or_misplaced_authoring() {
        let cases: Vec<(serde_json::Value, &str)> = vec![
            ({let mut v = shove(); v["extras"]["push"] = serde_json::json!([0]); v}, "zero distance"),
            ({let mut v = shove(); v["extras"]["push"] = serde_json::json!([3001]); v}, "distance cap"),
            ({let mut v = shove(); v["extras"]["push_time"] = serde_json::json!([0]); v}, "zero duration"),
            ({let mut v = shove(); v["extras"]["push_time"] = serde_json::json!([5.5]); v}, "duration cap"),
            ({let mut v = shove(); v["extras"].as_object_mut().unwrap().remove("push"); v}, "missing distance"),
            ({let mut v = shove(); v["effects"][1]["motion"] = "throw".into(); v}, "unknown motion"),
            ({let mut v = shove(); v["effects"][1]["speed"] = 1.into(); v}, "unknown field"),
            ({let mut v = hook(); v["extras"]["pull"] = serde_json::json!([3500]); v}, "projectile distance cap"),
            ({let mut v = hook(); let extra = v["effects"][1].clone(); v["effects"].as_array_mut().unwrap().push(extra); v}, "one forced move per hit"),
        ];
        for (value, label) in cases { assert!(check(value).is_err(), "{label}"); }
    }
    fn flare() -> serde_json::Value {
        serde_json::json!({"id":"fixture_flare","ability_type":"active","cast_type":"instant","target_type":"point","max_level":1,
            "levels":[{"range":1000}],"extras":{"radius":[450],"time":[4]},
            "effects":[{"kind":"reveal_area","radius_key":"radius","duration_key":"time"}]})
    }
    #[test]
    fn reveal_area_accepts_a_bounded_point_area_and_rejects_the_rest() {
        check(flare()).unwrap();
        let cases: Vec<(serde_json::Value, &str)> = vec![
            ({let mut v = flare(); v["extras"]["radius"] = serde_json::json!([0]); v}, "zero radius"),
            ({let mut v = flare(); v["extras"]["radius"] = serde_json::json!([2001]); v}, "radius cap"),
            ({let mut v = flare(); v["extras"]["time"] = serde_json::json!([30.5]); v}, "duration cap"),
            ({let mut v = flare(); v["levels"] = serde_json::json!([{"range":0}]); v}, "unbounded range"),
            ({let mut v = flare(); v["target_type"] = "unit".into(); v}, "point only"),
            ({let mut v = flare(); v["effects"][0]["detection"] = 2.into(); v}, "unknown field"),
        ];
        for (value, label) in cases { assert!(check(value).is_err(), "{label}"); }
    }
}

#[cfg(test)]
mod charge_and_summon_delivery_tests {
    use super::*;
    fn sentry() -> serde_json::Value {
        serde_json::json!({"id":"fixture_sentry","ability_type":"active","cast_type":"instant","target_type":"point","max_level":2,
            "levels":[{"range":600},{"range":600}],"extras":{"charges":[2,3],"charge_time":[15,15],"life":[30,40]},
            "charges":{"max_key":"charges","recharge_key":"charge_time"},
            "summon":{"unit":"fixture_turret","duration_key":"life"}})
    }
    fn check(value: serde_json::Value) -> Result<(), String> {
        validate_ability_progression(&serde_json::from_value::<AbilityDefinition>(value).map_err(|e| e.to_string())?)
    }
    #[test]
    fn charges_and_summon_delivery_accept_valid_authoring() {
        check(sentry()).unwrap();
        let mut no_summon = sentry(); no_summon.as_object_mut().unwrap().remove("summon");
        no_summon["effects"] = serde_json::json!([{"kind":"area_damage","amount_key":"life","radius_key":"life","damage_kind":"magical"}]);
        check(no_summon).unwrap();
    }
    #[test]
    fn charges_and_summon_delivery_reject_invalid_authoring() {
        let cases: Vec<(serde_json::Value, &str)> = vec![
            ({let mut v = sentry(); v["extras"]["charges"] = serde_json::json!([2.5,3]); v}, "fractional max"),
            ({let mut v = sentry(); v["extras"]["charges"] = serde_json::json!([0,3]); v}, "zero max"),
            ({let mut v = sentry(); v["extras"]["charges"] = serde_json::json!([3,2]); v}, "decreasing max"),
            ({let mut v = sentry(); v["extras"]["charges"] = serde_json::json!([11,11]); v}, "max over 10"),
            ({let mut v = sentry(); v["extras"]["charge_time"] = serde_json::json!([0.1,15]); v}, "recharge floor"),
            ({let mut v = sentry(); v["ability_type"] = "passive".into(); v}, "passive charges"),
            ({let mut v = sentry(); v["charges"]["extra"] = 1.into(); v}, "unknown charge field"),
            ({let mut v = sentry(); v["target_type"] = "unit".into(); v}, "summon needs point"),
            ({let mut v = sentry(); v["effects"] = serde_json::json!([{"kind":"heal_self","amount_key":"life"}]); v}, "summon is exclusive"),
            ({let mut v = sentry(); v["summon"]["unit"] = "../x".into(); v}, "unit id"),
            ({let mut v = sentry(); v["extras"]["life"] = serde_json::json!([30]); v}, "per-rank duration"),
            ({let mut v = sentry(); v["summon"]["count"] = 2.into(); v}, "unknown summon field"),
        ];
        for (value, label) in cases { assert!(check(value).is_err(), "{label}"); }
    }
}

#[cfg(test)]
mod cast_cooldown_refund_tests {
    use super::*;
    fn refund()->serde_json::Value {
        serde_json::json!({"id":"cooling","ability_type":"ultimate","cast_type":"instant","target_type":"none","max_level":2,
            "levels":[{},{}],"extras":{"cut":[2.0,3.0]},"cast_cooldown_refund":{"seconds_key":"cut"}})
    }
    #[test]
    fn refund_needs_per_rank_seconds_in_range_and_round_trips() {
        let parse=|v:&serde_json::Value|serde_json::from_value::<AbilityDefinition>(v.clone());
        let ok=parse(&refund()).unwrap();
        assert!(validate_ability_effects(&ok).is_ok());
        assert_eq!(serde_json::to_value(&ok).unwrap()["cast_cooldown_refund"]["seconds_key"],"cut");
        for (bad,why) in [
            ({let mut v=refund();v["extras"]["cut"]=serde_json::json!([2.0]);v},"one row per rank"),
            ({let mut v=refund();v["extras"]["cut"]=serde_json::json!([2.0,61.0]);v},"above 60"),
            ({let mut v=refund();v["extras"]["cut"]=serde_json::json!([0.0,1.0]);v},"zero"),
            ({let mut v=refund();v["cast_cooldown_refund"]["seconds_key"]="missing".into();v},"missing key"),
        ] {assert!(validate_ability_effects(&parse(&bad).unwrap()).is_err(),"{why}");}
        let mut unknown=refund();unknown["cast_cooldown_refund"]["scope"]="all".into();
        assert!(parse(&unknown).is_err(),"unknown field");
    }
}

#[cfg(test)]
mod attack_orb_tests {
    use super::*;
    fn orb() -> serde_json::Value {
        serde_json::json!({"id":"fixture_orb","ability_type":"toggle","cast_type":"instant","target_type":"none","max_level":1,
            "levels":[{}],"extras":{"mana":[10],"damage":[30],"slow":[0.09],"time":[4]},
            "attack_orb":{"mana_per_attack_key":"mana","amount_key":"damage","damage_kind":"magical",
                "move_slow_key":"slow","attack_slow_key":"slow","slow_duration_key":"time"}})
    }
    fn check(value: serde_json::Value) -> Result<(), String> {
        validate_ability_progression(&serde_json::from_value::<AbilityDefinition>(value).map_err(|e| e.to_string())?)
    }
    #[test]
    fn attack_orb_accepts_toggle_and_rejects_other_shapes() {
        check(orb()).unwrap();
        let mut bare = orb();
        for key in ["move_slow_key","attack_slow_key","slow_duration_key"] { bare["attack_orb"].as_object_mut().unwrap().remove(key); }
        check(bare).unwrap();
        let cases: Vec<(serde_json::Value, &str)> = vec![
            ({let mut v = orb(); v["ability_type"] = "active".into(); v}, "toggle only"),
            ({let mut v = orb(); v["target_type"] = "unit".into(); v}, "no target"),
            ({let mut v = orb(); v["effects"] = serde_json::json!([{"kind":"heal_self","amount_key":"damage"}]); v}, "exclusive"),
            ({let mut v = orb(); v["attack_orb"].as_object_mut().unwrap().remove("slow_duration_key"); v}, "slows need duration"),
            ({let mut v = orb(); v["extras"]["slow"] = serde_json::json!([1.5]); v}, "slow fraction"),
            ({let mut v = orb(); v["extras"]["mana"] = serde_json::json!([-1]); v}, "negative mana"),
            ({let mut v = orb(); v["attack_orb"]["stacks"] = 2.into(); v}, "unknown field"),
        ];
        for (value, label) in cases { assert!(check(value).is_err(), "{label}"); }
    }
}
