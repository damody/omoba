//! Deterministic, data-driven ability effects generated from Lua declarations.
//! Immediate effects and host-owned timed damage/stat buffs share admission.
//! Continuous movement, summons and passive hooks remain explicit Rust handlers.

use abi_stable::std_types::{RErr, ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{ChannelPulse, DamageKind, DamageProfile, EntityHandle, Fixed64, Target, Vec2},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{DamageType, EffectSpec, TargetSelector};
use omoba_template_ids::{active_ability_const, AbilityId};
use omoba_content_model::ControlEffectKind;
use omoba_content_model::ForcedMotionKind;
use omoba_content_model::ManaBuffStat;

fn host_forced_mode(motion: ForcedMotionKind) -> omoba_core::runtime::native::forced_motion::Mode {
    use omoba_core::runtime::native::forced_motion::Mode;
    match motion { ForcedMotionKind::Knockback => Mode::Knockback, ForcedMotionKind::Pull => Mode::Pull }
}

use crate::ability_builder::{extra_at, extra_at_id_f32};

#[derive(Clone, Copy, Debug)]
pub enum EffectOp {
    CleanseAlly,
    DrainManaEnemy {amount_key:&'static str},
    RecoveryChannelSelf {amount_key:&'static str,duration_key:&'static str,interval_key:&'static str},
    LineDamage {amount_key:&'static str,width_key:&'static str,kind:DamageKind},
    ConeDamage {amount_key:&'static str,half_angle_key:&'static str,kind:DamageKind},
    DamageOverTimeEnemy {amount_key:&'static str,duration_key:&'static str,kind:DamageKind},
    HealAlly {amount_key:&'static str},
    ShieldAlly {amount_key:&'static str,duration_key:&'static str},
    ManaBuffSelf {stat:ManaBuffStat,value_key:&'static str,duration_key:&'static str},
    SlowEnemy { reduction_key:&'static str, duration_key:&'static str },
    StunEnemy { duration_key:&'static str },
    ControlEnemy { control:ControlEffectKind, duration_key:&'static str },
    /// G-4 forced displacement of the enemy target (host `forced_motion`).
    ForcedMoveEnemy { motion:ForcedMotionKind, distance_key:&'static str, duration_key:&'static str },
    /// G-4 true-sight area at the point target (host `reveal_zone`).
    RevealArea { radius_key:&'static str, duration_key:&'static str },
    DashToPoint,
    Damage {
        amount_key: &'static str,
        kind: DamageKind,
    },
    HealSelf {
        amount_key: &'static str,
    },
    RestoreManaSelf { amount_key: &'static str },
    SpendManaSelf { amount_key: &'static str },
    AreaDamage { amount_key: &'static str, radius_key: &'static str, kind: DamageKind },
    /// G-1 delivery. Always first; the following Damage/Slow/Control ops become
    /// the on-hit payload of one host-owned projectile instead of instant effects.
    Projectile {
        motion: omoba_content_model::ProjectileMotion,
        speed_key: &'static str, radius_key: &'static str, lifetime_key: Option<&'static str>,
        max_hits: u16, wall: omoba_content_model::ProjectileWall,
        target_lost: omoba_content_model::ProjectileTargetLost,
        caster_death: omoba_content_model::ProjectileCasterDeath,
    },
    /// G-3 exclusive point summon of a generated template.
    SummonAtPoint { unit: &'static str, duration_key: Option<&'static str> },
    /// G-2 toggle orb: flips the caster's host-owned orb marker.
    ToggleAttackOrb,
    /// G-2 pulse channel. Always first; the host owns the lifecycle and the
    /// following Damage/Slow/Control ops apply once per pulse.
    Channel { anchor: omoba_content_model::ChannelAnchor, interval_key: &'static str, pulses_key: &'static str, radius_key: Option<&'static str> },
}

#[derive(Debug)]
enum ProjectileAim { Point(Vec2), Unit(EntityHandle) }

#[derive(Debug)]
enum ResolvedEffect {
    ToggleAttackOrb,
    SummonAtPoint { point: Vec2, cast_range: Fixed64, unit: &'static str, duration: Fixed64 },
    Projectile {
        aim: ProjectileAim, cast_range: Fixed64, speed: Fixed64, radius: Fixed64, lifetime: Fixed64, max_hits: u16,
        wall: omoba_content_model::ProjectileWall, target_lost: omoba_content_model::ProjectileTargetLost,
        caster_death: omoba_content_model::ProjectileCasterDeath, amount: Fixed64, kind: DamageKind,
        slow: Option<(Fixed64, Fixed64)>, control: Option<(ControlEffectKind, Fixed64)>,
        forced: Option<(ForcedMotionKind, Fixed64, Fixed64)>,
    },
    CleanseAlly {victim:EntityHandle,cast_range:Fixed64},
    ForcedMoveEnemy { victim:EntityHandle, motion:ForcedMotionKind, distance:Fixed64, duration:Fixed64, cast_range:Fixed64 },
    RevealArea { center:Vec2, radius:Fixed64, duration:Fixed64, cast_range:Fixed64 },
    DrainManaEnemy {victim:EntityHandle,amount:Fixed64,cast_range:Option<Fixed64>},
    LineDamage {aim:Vec2,cast_range:Fixed64,width:Fixed64,amount:Fixed64,kind:DamageKind},
    ConeDamage {aim:Vec2,cast_range:Fixed64,half_ticks:u16,amount:Fixed64,kind:DamageKind},
    DamageOverTimeEnemy {victim:EntityHandle,rate:Fixed64,duration:Fixed64,kind:DamageKind,cast_range:Fixed64},
    HealAlly {victim:EntityHandle,amount:Fixed64,cast_range:Fixed64},
    ShieldAlly {victim:EntityHandle,amount:Fixed64,duration:Fixed64,cast_range:Fixed64},
    ManaBuffSelf {stat:ManaBuffStat,value:Fixed64,duration:Fixed64},
    SlowEnemy { victim:EntityHandle, reduction:Fixed64, duration:Fixed64, cast_range:Option<Fixed64> },
    ControlEnemy { control:ControlEffectKind, victim:EntityHandle, duration:Fixed64, cast_range:Option<Fixed64> },
    DashToPoint { target:Vec2, cast_range:Fixed64 },
    Damage {
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
        /// None only for area hits whose center was already range-validated.
        cast_range: Option<Fixed64>,
    },
    HealSelf {
        amount: Fixed64,
    },
    RestoreManaSelf { amount: Fixed64 },
    SpendManaSelf { amount: Fixed64 },
    AreaDamage { center: Vec2, radius: Fixed64, cast_range: Fixed64, amount: Fixed64, kind: DamageKind },
}

pub struct GenericEffectHandler {
    id: AbilityId,
    effects: &'static [EffectOp],
}

trait EffectSink {
    fn cleanse(&mut self,caster:EntityHandle,victim:EntityHandle,cast_range:Fixed64)->bool;
    fn drain_mana(&mut self,caster:EntityHandle,victim:EntityHandle,amount:Fixed64,cast_range:Fixed64)->bool;
    fn is_building(&self,_:EntityHandle)->bool {false}
    fn dot(&mut self,caster:EntityHandle,victim:EntityHandle,ability:AbilityId,rate:Fixed64,duration:Fixed64,kind:DamageKind);
    fn area_cue(&mut self, center:Vec2, radius:Fixed64);
    fn is_ally(&self,caster:EntityHandle,victim:EntityHandle)->bool;
    fn mana_buff(&mut self,caster:EntityHandle,ability:AbilityId,stat:ManaBuffStat,value:Fixed64,duration:Fixed64);
    fn restore_mana(&mut self, caster:EntityHandle, amount:Fixed64);
    fn shield(&mut self,caster:EntityHandle,victim:EntityHandle,amount:Fixed64,duration:Fixed64,cast_range:Fixed64)->bool;
    fn spend_mana(&mut self, caster:EntityHandle, amount:Fixed64, ability:AbilityId)->bool;
    fn slow(&mut self, caster:EntityHandle, victim:EntityHandle, ability:AbilityId, reduction:Fixed64, duration:Fixed64);
    fn control(&mut self, victim:EntityHandle, control:ControlEffectKind, duration:Fixed64);
    /// Host-owned displacement; the host plans and integrates the motion.
    fn forced_move(&mut self, victim:EntityHandle, spec:&omoba_core::runtime::native::forced_motion::Spec, duration:Fixed64);
    /// Host-owned team vision zone; the host admits and times it.
    fn reveal_area(&mut self, caster:EntityHandle, spec:&omoba_core::runtime::native::reveal_zone::Spec, duration:Fixed64);
    fn collision_destination(&mut self, caster:EntityHandle, target:Vec2, range:Fixed64) -> Vec2;
    fn relocate(&mut self, caster:EntityHandle, destination:Vec2);
    fn position(&self, entity: EntityHandle) -> Option<Vec2>;
    fn enemies_in_radius(&self, caster:EntityHandle, center:Vec2, radius:Fixed64) -> Vec<EntityHandle>;
    fn is_alive(&self, entity: EntityHandle) -> bool;
    fn is_enemy(&self, caster: EntityHandle, victim: EntityHandle) -> bool;
    fn damage(
        &mut self,
        caster: EntityHandle,
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
    );
    fn heal(&mut self, caster: EntityHandle, amount: Fixed64);
    /// Host-owned launch envelope; the host admits and owns the whole flight.
    fn launch_projectile(&mut self, caster: EntityHandle, lifetime: Fixed64, payload: &str);
    /// Host placement may reject (terrain/occupancy); false aborts the cast.
    fn summon(&mut self, caster: EntityHandle, point: Vec2, unit: &str, duration: Fixed64) -> bool;
    /// Toggle the caster's orb marker; the host admits and prices each attack.
    fn toggle_attack_orb(&mut self, caster: EntityHandle, ability: &str);
}

fn damage_profile(kind: DamageKind) -> DamageProfile {
    match kind {
        DamageKind::Physical => DamageProfile::NORMAL,
        DamageKind::Magical => DamageProfile::ENERGY,
        DamageKind::Pure => DamageProfile::TRUE,
    }
}

impl EffectSink for GameWorldDyn<'_> {
    fn cleanse(&mut self,caster:EntityHandle,victim:EntityHandle,cast_range:Fixed64)->bool {
        let Some(spec)=omoba_core::runtime::native::ally_cleanse::Spec::checked(caster.id,caster.gen as u32,cast_range)else{return false;};
        let payload=spec.payload().to_string();self.add_stat_buff(victim,omoba_core::runtime::native::ally_cleanse::BUFF.into(),Fixed64::ONE,payload.as_str().into());true
    }
    fn drain_mana(&mut self,caster:EntityHandle,victim:EntityHandle,amount:Fixed64,cast_range:Fixed64)->bool {
        let Some(spec)=omoba_core::runtime::native::mana_drain::Spec::checked(caster.id,caster.gen as u32,amount,cast_range)else{return false;};
        let payload=spec.payload().to_string();self.add_stat_buff(victim,omoba_core::runtime::native::mana_drain::BUFF.into(),Fixed64::ONE,payload.as_str().into());true
    }
    fn shield(&mut self,caster:EntityHandle,victim:EntityHandle,amount:Fixed64,duration:Fixed64,cast_range:Fixed64)->bool {
        let Some(spec)=omoba_core::runtime::native::ally_shield::Spec::checked(caster.id,caster.gen,amount,cast_range,duration)else{return false;};
        let payload=spec.payload().to_string();self.add_stat_buff(victim,"damage_shield".into(),duration,payload.as_str().into());true
    }
    fn is_building(&self,entity:EntityHandle)->bool {GameWorldDyn::is_building(self,entity)}
    fn dot(&mut self,caster:EntityHandle,victim:EntityHandle,ability:AbilityId,rate:Fixed64,duration:Fixed64,kind:DamageKind) {
        let key=omb_script_abi::buff_ids::generic_dot_buff_id(ability.as_str(),caster);
        let payload=omoba_core::runtime::ability_runtime::periodic_damage::PeriodicDamageSpec::payload(rate,caster,kind).to_string();
        self.add_stat_buff(victim,(&*key).into(),duration,(&*payload).into());
    }
    fn area_cue(&mut self, center:Vec2, radius:Fixed64) {
        self.emit_explosion(center, radius, Fixed64::ONE);
    }
    fn is_ally(&self,caster:EntityHandle,victim:EntityHandle)->bool {
        match (self.faction_of(caster).into_option(),self.faction_of(victim).into_option()) {
            (Some(a),Some(b))=>a==b,
            _=>false,
        }
    }
    fn mana_buff(&mut self,caster:EntityHandle,ability:AbilityId,stat:ManaBuffStat,value:Fixed64,duration:Fixed64) {
        let key=omb_script_abi::buff_ids::generic_mana_buff_id(ability.as_str(),stat.as_str(),caster);
        let payload=serde_json::json!({stat.as_str():value.raw()}).to_string();
        self.add_stat_buff(caster,(&*key).into(),duration,(&*payload).into());
    }
    fn restore_mana(&mut self,caster:EntityHandle,amount:Fixed64) {
        GameWorldDyn::restore_mana(self,caster,amount);
    }
    fn spend_mana(&mut self,caster:EntityHandle,amount:Fixed64,ability:AbilityId)->bool {
        GameWorldDyn::spend_mana(self,caster,amount,ability.as_str().into())
    }
    fn control(&mut self,victim:EntityHandle,control:ControlEffectKind,duration:Fixed64) {
        // Standard status identity intentionally shares max-remaining refresh
        // with legacy stun. Never shorten an existing control effect.
        let buff=match control {
            ControlEffectKind::Stun=>omb_script_abi::buff_ids::BuffId::Stun,
            ControlEffectKind::Root=>omb_script_abi::buff_ids::BuffId::Root,
            ControlEffectKind::Silence=>omb_script_abi::buff_ids::BuffId::Silence,
        };
        self.add_buff(victim,buff.as_rstr(),duration);
    }
    fn forced_move(&mut self,victim:EntityHandle,spec:&omoba_core::runtime::native::forced_motion::Spec,duration:Fixed64) {
        let payload=spec.payload().to_string();
        self.add_stat_buff(victim,omoba_core::runtime::native::forced_motion::BUFF.into(),duration,payload.as_str().into());
    }
    fn reveal_area(&mut self,caster:EntityHandle,spec:&omoba_core::runtime::native::reveal_zone::Spec,duration:Fixed64) {
        let payload=spec.payload().to_string();
        self.add_stat_buff(caster,omoba_core::runtime::native::reveal_zone::BUFF.into(),duration,payload.as_str().into());
    }
    fn slow(&mut self,caster:EntityHandle,victim:EntityHandle,ability:AbilityId,reduction:Fixed64,duration:Fixed64) {
        let key=omb_script_abi::buff_ids::generic_slow_buff_id(ability.as_str(),caster);
        let payload=serde_json::json!({"move_speed_bonus":-reduction.raw(),"__aggregation_family":"generic_ability_slow"}).to_string();
        self.add_stat_buff(victim,(&*key).into(),duration,(&*payload).into());
    }
    fn collision_destination(&mut self,caster:EntityHandle,target:Vec2,range:Fixed64) -> Vec2 {
        self.advance_with_collision(caster,target,range)
    }
    fn relocate(&mut self,caster:EntityHandle,destination:Vec2) {self.set_pos(caster,destination);}
    fn position(&self, entity:EntityHandle) -> Option<Vec2> {self.get_pos(entity).into_option()}
    fn enemies_in_radius(&self,caster:EntityHandle,center:Vec2,radius:Fixed64) -> Vec<EntityHandle> {
        self.query_enemies_in_range(center,radius,caster).into_iter().collect()
    }
    fn is_alive(&self, entity: EntityHandle) -> bool {
        GameWorldDyn::is_alive(self, entity)
            && self.get_hp(entity).into_option().is_some_and(|hp| hp > Fixed64::ZERO)
    }

    fn is_enemy(&self, caster: EntityHandle, victim: EntityHandle) -> bool {
        let Some(position) = self.get_pos(victim).into_option() else {
            return false;
        };
        self.query_enemies_in_range(position, Fixed64::ZERO, caster)
            .iter()
            .any(|candidate| *candidate == victim)
    }

    fn damage(
        &mut self,
        caster: EntityHandle,
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
    ) {
        self.deal_damage(victim, amount, kind, damage_profile(kind), RSome(caster));
    }

    fn heal(&mut self, caster: EntityHandle, amount: Fixed64) {
        GameWorldDyn::heal(self, caster, amount);
    }

    fn launch_projectile(&mut self, caster: EntityHandle, lifetime: Fixed64, payload: &str) {
        self.add_stat_buff(caster, omoba_core::runtime::native::ability_projectile::BUFF.into(), lifetime, payload.into());
    }

    fn summon(&mut self, caster: EntityHandle, point: Vec2, unit: &str, duration: Fixed64) -> bool {
        self.spawn_summoned_unit(point, unit.into(), caster, duration).is_valid()
    }

    fn toggle_attack_orb(&mut self, caster: EntityHandle, ability: &str) {
        use omoba_core::runtime::native::attack_orb;
        let buff = attack_orb::buff_id(ability);
        if self.has_buff(caster, buff.as_str().into()) {
            self.remove_buff(caster, buff.as_str().into());
        } else {
            // Until toggled off, auto-off for mana or death cleanup.
            let payload = attack_orb::payload(ability).to_string();
            self.add_stat_buff(caster, buff.as_str().into(), Fixed64::from_raw(i32::MAX as i64), payload.as_str().into());
        }
    }
}

/// Exclusive point summon: range-checked here, placement admitted by the host.
fn apply_summon(id: AbilityId, caster: EntityHandle, effect: ResolvedEffect, sink: &mut impl EffectSink) -> Result<(), String> {
    let ResolvedEffect::SummonAtPoint { point, cast_range, unit, duration } = effect else { unreachable!("summon only") };
    let origin = sink.position(caster).ok_or_else(|| format!("ability '{}': missing caster position", id.as_str()))?;
    if cast_range <= Fixed64::ZERO || cast_range > Fixed64::from_i32(10_000)
        || !omoba_sim::terrain::point_in_swept_circle(point, origin, origin, cast_range) {
        return Err(format!("ability '{}': summon point outside cast range", id.as_str()));
    }
    if duration <= Fixed64::ZERO || duration > Fixed64::from_i32(600) {
        return Err(format!("ability '{}': invalid summon duration", id.as_str()));
    }
    if !sink.summon(caster, point, unit, duration) {
        return Err(format!("ability '{}': summon placement rejected", id.as_str()));
    }
    Ok(())
}

fn resolve_summon(id: AbilityId, unit: &'static str, duration_key: Option<&'static str>, target: &Target, level: u8)
    -> Result<ResolvedEffect, String> {
    let ability = active_ability_const(id).ok_or_else(|| format!("unknown generated ability '{}'", id.as_str()))?;
    let Target::Point(point) = target else { return Err(format!("ability '{}': summon requires a ground point", id.as_str())); };
    let rank = usize::from(level - 1);
    let duration = match duration_key {
        Some(key) => ability.extras.iter().find(|(name, _)| *name == key).and_then(|(_, rows)| rows.get(rank)).copied()
            .ok_or_else(|| format!("ability '{}': missing summon duration", id.as_str()))?,
        None => omoba_template_ids::summon_by_name(unit).and_then(omoba_template_ids::active_summon_stats)
            .map(|stats| stats.duration).ok_or_else(|| format!("ability '{}': unknown summon '{unit}'", id.as_str()))?,
    };
    Ok(ResolvedEffect::SummonAtPoint { point: *point, cast_range: ability.levels[rank].range, unit, duration })
}

fn forced_spec(caster:EntityHandle,motion:ForcedMotionKind,distance:Fixed64,cast_range:Fixed64)
    ->Option<omoba_core::runtime::native::forced_motion::Spec> {
    omoba_core::runtime::native::forced_motion::Spec { to:None, source_id:caster.id, source_generation:caster.gen,
        mode:host_forced_mode(motion), distance, cast_range }.checked()
}

fn reveal_spec(caster:EntityHandle,center:Vec2,radius:Fixed64,cast_range:Fixed64)
    ->Option<omoba_core::runtime::native::reveal_zone::Spec> {
    omoba_core::runtime::native::reveal_zone::Spec { source_id:caster.id, source_generation:caster.gen, center, radius, cast_range, detect:true }.checked()
}

/// Exclusive projectile launch. Every check precedes the single host envelope.
fn apply_projectile(id: AbilityId, caster: EntityHandle, effect: ResolvedEffect, sink: &mut impl EffectSink) -> Result<(), String> {
    use omoba_core::runtime::native::ability_projectile as host;
    let ResolvedEffect::Projectile { aim, cast_range, speed, radius, lifetime, max_hits, wall, target_lost, caster_death,
        amount, kind, slow, control, forced } = effect else { unreachable!("projectile only") };
    let forced = forced.map(|(motion, distance, duration)| (host_forced_mode(motion), distance, duration));
    let origin = sink.position(caster).ok_or_else(|| format!("ability '{}': missing caster position", id.as_str()))?;
    let motion = match aim {
        ProjectileAim::Point(point) => {
            if point == origin { return Err(format!("ability '{}': projectile needs a direction", id.as_str())); }
            host::Motion::Linear { aim: point }
        }
        ProjectileAim::Unit(victim) => {
            if !victim.is_valid() || victim == caster || !sink.is_alive(victim) || !sink.is_enemy(caster, victim) {
                return Err(format!("ability '{}': projectile target is not a live enemy", id.as_str()));
            }
            let position = sink.position(victim).ok_or("missing projectile target position")?;
            if (position - origin).length_squared() > cast_range * cast_range {
                return Err(format!("ability '{}': projectile target outside cast range", id.as_str()));
            }
            host::Motion::Homing { target_id: victim.id, target_generation: victim.gen }
        }
    };
    let linear = matches!(motion, host::Motion::Linear { .. });
    let spec = host::Spec {
        source_id: caster.id, source_generation: caster.gen, ability: id.as_str().to_owned(), motion, speed, radius,
        distance: if linear { cast_range } else { Fixed64::ZERO }, lifetime, max_hits,
        wall: match wall { omoba_content_model::ProjectileWall::Pass => host::WallPolicy::Pass,
            omoba_content_model::ProjectileWall::Stop => host::WallPolicy::Stop },
        target_lost: match target_lost { omoba_content_model::ProjectileTargetLost::Expire => host::TargetLostPolicy::Expire,
            omoba_content_model::ProjectileTargetLost::LastPosition => host::TargetLostPolicy::LastPosition },
        caster_death: match caster_death { omoba_content_model::ProjectileCasterDeath::Continue => host::CasterDeathPolicy::Continue,
            omoba_content_model::ProjectileCasterDeath::Expire => host::CasterDeathPolicy::Expire },
        amount, kind, profile: damage_profile(kind).bits(), slow, control, dot: None, buildings: false, forced,
    };
    if !spec.valid() { return Err(format!("ability '{}': invalid projectile envelope", id.as_str())); }
    sink.launch_projectile(caster, lifetime, &spec.payload().to_string());
    Ok(())
}

/// Delivery plus payload ops -> one exclusive resolved launch.
fn resolve_projectile(id: AbilityId, effects: &[EffectOp], target: &Target, level: u8) -> Result<ResolvedEffect, String> {
    let ability = active_ability_const(id).ok_or_else(|| format!("unknown generated ability '{}'", id.as_str()))?;
    let [EffectOp::Projectile { motion, speed_key, radius_key, lifetime_key, max_hits, wall, target_lost, caster_death }, payload @ ..] = effects
        else { return Err(format!("ability '{}': projectile delivery must lead its payload", id.as_str())); };
    let rank = usize::from(level - 1);
    let read = |key: &str| ability.extras.iter().find(|(name, _)| *name == key).and_then(|(_, rows)| rows.get(rank)).copied()
        .ok_or_else(|| format!("ability '{}': missing projectile extras '{key}'", id.as_str()));
    let cast_range = ability.levels[rank].range;
    let speed = read(speed_key)?;
    let lifetime = match lifetime_key {
        Some(key) => read(key)?,
        // Linear default: exactly the time to fly the cast range, rounded up.
        None => {
            if speed <= Fixed64::ZERO { return Err(format!("ability '{}': invalid projectile speed", id.as_str())); }
            let (distance, rate) = (i128::from(cast_range.raw()) * 1024, i128::from(speed.raw()));
            Fixed64::from_raw(((distance + rate - 1) / rate) as i64)
        }
    };
    let aim = match (motion, target) {
        (omoba_content_model::ProjectileMotion::Linear, Target::Point(point)) => ProjectileAim::Point(*point),
        (omoba_content_model::ProjectileMotion::Homing, Target::Entity(victim)) => ProjectileAim::Unit(*victim),
        _ => return Err(format!("ability '{}': projectile motion does not match target", id.as_str())),
    };
    let (mut amount, mut kind, mut slow, mut control, mut forced) = (None, DamageKind::Pure, None, None, None);
    for op in payload {
        match op {
            EffectOp::Damage { amount_key, kind: damage } if amount.is_none() => { amount = Some(read(amount_key)?); kind = *damage; }
            EffectOp::SlowEnemy { reduction_key, duration_key } if slow.is_none() => slow = Some((read(reduction_key)?, read(duration_key)?)),
            EffectOp::StunEnemy { duration_key } if control.is_none() => control = Some((ControlEffectKind::Stun, read(duration_key)?)),
            EffectOp::ControlEnemy { control: kind, duration_key } if control.is_none() => control = Some((*kind, read(duration_key)?)),
            EffectOp::ForcedMoveEnemy { motion, distance_key, duration_key } if forced.is_none() =>
                forced = Some((*motion, read(distance_key)?, read(duration_key)?)),
            _ => return Err(format!("ability '{}': projectile delivers one damage, slow, control and forced move", id.as_str())),
        }
    }
    Ok(ResolvedEffect::Projectile { aim, cast_range, speed, radius: read(radius_key)?, lifetime, max_hits: *max_hits,
        wall: *wall, target_lost: *target_lost, caster_death: *caster_death,
        amount: amount.ok_or_else(|| format!("ability '{}': projectile requires damage", id.as_str()))?, kind, slow, control, forced })
}

fn apply_effects(
    id: AbilityId,
    caster: EntityHandle,
    effects: Vec<ResolvedEffect>,
    sink: &mut impl EffectSink,
) -> Result<(), String> {
    if !caster.is_valid() || !sink.is_alive(caster) {
        return Err(format!("ability '{}': caster is not alive",id.as_str()));
    }
    if effects.iter().any(|e|matches!(e,ResolvedEffect::ToggleAttackOrb)) {
        if effects.len()!=1 {return Err(format!("ability '{}': attack orb toggle is exclusive",id.as_str()));}
        sink.toggle_attack_orb(caster,id.as_str());
        return Ok(());
    }
    if effects.iter().any(|e|matches!(e,ResolvedEffect::SummonAtPoint {..})) {
        let mut effects=effects;
        if effects.len()!=1 {return Err(format!("ability '{}': summon is an exclusive delivery",id.as_str()));}
        return apply_summon(id,caster,effects.pop().expect("one effect"),sink);
    }
    if effects.iter().any(|e|matches!(e,ResolvedEffect::Projectile {..})) {
        let mut effects=effects;
        if effects.len()!=1 {return Err(format!("ability '{}': projectile is an exclusive delivery",id.as_str()));}
        return apply_projectile(id,caster,effects.pop().expect("one effect"),sink);
    }
    if effects.iter().filter(|e|matches!(e,ResolvedEffect::DamageOverTimeEnemy {..})).count()>1 {
        return Err("duplicate DoT effect".into());
    }
    if effects.iter().any(|effect|matches!(effect,ResolvedEffect::DashToPoint {..})) {
        let [ResolvedEffect::DashToPoint {target,cast_range}]=effects.as_slice() else {
            return Err(format!("ability '{}': movement cannot be combined with other effects",id.as_str()));
        };
        let origin=sink.position(caster).ok_or_else(||format!("ability '{}': missing caster position",id.as_str()))?;
        if *target==origin || *cast_range<=Fixed64::ZERO || *cast_range>Fixed64::from_i32(10_000)
            || (*target-origin).length_squared()>*cast_range * *cast_range {
            return Err(format!("ability '{}': invalid dash destination/range",id.as_str()));
        }
        // This existing ABI call only computes a destination; set_pos commits.
        let destination=sink.collision_destination(caster,*target,*cast_range);
        if destination!=*target {
            return Err(format!("ability '{}': dash path blocked",id.as_str()));
        }
        sink.relocate(caster,destination);
        return Ok(());
    }
    let mut expanded=Vec::new();
    let mut areas=Vec::new();
    for effect in effects {
        if let ResolvedEffect::LineDamage {aim,cast_range,width,amount,kind}=effect {
            let origin=sink.position(caster).ok_or("missing line source position")?;
            let end=omoba_sim::terrain::checked_line_endpoint(origin,aim,cast_range).ok_or("invalid line axis or range")?;
            if !sink.is_alive(caster)||width<Fixed64::from_raw(1)||width>Fixed64::from_i32(5_000)
                ||amount<Fixed64::ZERO||amount>Fixed64::from_i32(1_000_000){return Err("invalid line source or budget".into());}
            let radius=width/Fixed64::from_i32(2);
            // Source-centered broad phase includes both closed capsule end caps.
            let mut victims=sink.enemies_in_radius(caster,origin,cast_range+radius);victims.sort_by_key(|e|(e.id,e.gen));victims.dedup();
            for victim in victims {
                if victim==caster||!sink.is_alive(victim)||!sink.is_enemy(caster,victim)||sink.is_building(victim){continue;}
                if !sink.position(victim).is_some_and(|p|omoba_sim::terrain::point_in_swept_circle(p,origin,end,radius)){continue;}
                if expanded.len()>=128{return Err("line exceeds128 resolved effects".into());}
                expanded.push(ResolvedEffect::Damage{victim,amount,kind,cast_range:None});
            }
        } else if let ResolvedEffect::ConeDamage {aim,cast_range,half_ticks,amount,kind}=effect {
            let origin=sink.position(caster).ok_or("missing cone source position")?;
            let supported=|p:Vec2|p.x.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
                &&p.y.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
            if !sink.is_alive(caster)||!supported(origin)||!supported(aim)||origin==aim
                || cast_range<=Fixed64::ZERO||cast_range>Fixed64::from_i32(10_000)||half_ticks==0||half_ticks>1024
                || amount<Fixed64::ZERO||amount>Fixed64::from_i32(1_000_000)
                || !omoba_sim::terrain::point_in_swept_circle(aim,origin,origin,cast_range) {
                return Err("invalid cone source, direction or budget".into());
            }
            let delta=aim-origin;let axis=omoba_sim::trig::atan2(delta.y,delta.x);
            let mut victims=sink.enemies_in_radius(caster,origin,cast_range);victims.sort_by_key(|e|(e.id,e.gen));victims.dedup();
            for victim in victims {
                if victim==caster||!sink.is_alive(victim)||!sink.is_enemy(caster,victim){continue;}
                if !sink.position(victim).is_some_and(|p|omoba_sim::sector::point_in_sector(p,origin,axis,cast_range,half_ticks)){continue;}
                if expanded.len()>=128{return Err("cone exceeds128 resolved effects".into());}
                expanded.push(ResolvedEffect::Damage {victim,amount,kind,cast_range:None});
            }
        } else if let ResolvedEffect::AreaDamage {center,radius,cast_range,amount,kind}=effect {
            areas.push((center,radius));
            let position=sink.position(caster).ok_or_else(||format!("ability '{}': missing caster position",id.as_str()))?;
            if !sink.is_alive(caster) || (center-position).length_squared()>cast_range*cast_range {
                return Err(format!("ability '{}': point outside cast range",id.as_str()));
            }
            let mut victims=sink.enemies_in_radius(caster,center,radius);
            victims.sort_by_key(|e|(e.id,e.gen));victims.dedup_by_key(|e|(e.id,e.gen));
            for victim in victims {
                // Host query can include HP0 entities pending retirement.
                if sink.is_alive(victim) {
                    if expanded.len()>=128 {return Err(format!("ability '{}': exceeds 128 resolved effects",id.as_str()));}
                    expanded.push(ResolvedEffect::Damage {victim,amount,kind,cast_range:None});
                }
            }
        } else {expanded.push(effect);}
        if expanded.len()>128 {return Err(format!("ability '{}': exceeds 128 resolved effects",id.as_str()));}
    }
    // Validate the entire plan before any world mutation.
    for effect in &expanded {
        if let ResolvedEffect::CleanseAlly {victim,cast_range}=effect {
            if !victim.is_valid() || !sink.is_alive(*victim) || !sink.is_ally(caster,*victim)
                || *cast_range<=Fixed64::ZERO || *cast_range>Fixed64::from_i32(10_000){return Err("invalid ally cleanse target/range".into());}
            let origin=sink.position(caster).ok_or("missing caster position")?;
            let position=sink.position(*victim).ok_or("missing ally position")?;
            if (position-origin).length_squared()>*cast_range * *cast_range{return Err("ally cleanse outside cast range".into());}
        }
        if let ResolvedEffect::DrainManaEnemy {victim,amount,cast_range}=effect {
            if !victim.is_valid() || *amount<=Fixed64::ZERO || *amount>Fixed64::from_i32(1_000_000)
                || cast_range.is_none_or(|v|v<=Fixed64::ZERO||v>Fixed64::from_i32(10_000)){return Err("invalid enemy mana drain".into());}
        }
        if let ResolvedEffect::DamageOverTimeEnemy {victim,rate,duration,cast_range,..}=effect {
            if *rate<Fixed64::ZERO || *rate>Fixed64::from_i32(1_000_000)
                || *duration<=Fixed64::ZERO || *duration>Fixed64::from_i32(60)
                || *cast_range<=Fixed64::ZERO || *cast_range>Fixed64::from_i32(10_000)
                || !victim.is_valid() || !sink.is_alive(*victim) || !sink.is_enemy(caster,*victim) {
                return Err(format!("ability '{}': invalid DoT target/data",id.as_str()));
            }
            let origin=sink.position(caster).ok_or("missing caster position")?;
            let position=sink.position(*victim).ok_or("missing DoT target position")?;
            if (position-origin).length_squared()>*cast_range * *cast_range {return Err("DoT outside cast range".into());}
        }
        if let ResolvedEffect::ShieldAlly {amount,duration,..}=effect {
            if *amount<=Fixed64::ZERO || *duration<=Fixed64::ZERO || *duration>Fixed64::from_i32(60){return Err("invalid ally shield amount/duration".into());}
        }
        if let ResolvedEffect::HealAlly {victim,amount,cast_range}|ResolvedEffect::ShieldAlly {victim,amount,cast_range,..}=effect {
            if !victim.is_valid() || !sink.is_alive(*victim) || !sink.is_ally(caster,*victim)
                || *amount<Fixed64::ZERO || *amount>Fixed64::from_i32(1_000_000)
                || *cast_range<=Fixed64::ZERO || *cast_range>Fixed64::from_i32(10_000) {
                return Err(format!("ability '{}': invalid ally heal target/data",id.as_str()));
            }
            let origin=sink.position(caster).ok_or("missing caster position")?;
            let position=sink.position(*victim).ok_or("missing ally position")?;
            if (position-origin).length_squared()>*cast_range * *cast_range {
                return Err(format!("ability '{}': ally outside cast range",id.as_str()));
            }
        }
        if let ResolvedEffect::ManaBuffSelf {stat,value,duration}=effect {
            let (minimum,maximum)=stat.bounds();
            if *value<Fixed64::from_i32(minimum) || *value>Fixed64::from_i32(maximum)
                || *duration<=Fixed64::ZERO || *duration>Fixed64::from_i32(60) {
                return Err(format!("ability '{}': invalid mana buff data",id.as_str()));
            }
        }
        if let ResolvedEffect::RestoreManaSelf {amount} | ResolvedEffect::SpendManaSelf {amount}=effect {
            if *amount<Fixed64::ZERO || *amount>Fixed64::from_i32(1_000_000) {
                return Err(format!("ability '{}': invalid mana amount",id.as_str()));
            }
        }
        if let ResolvedEffect::ControlEnemy {duration,cast_range,..}=effect {
            if *duration<=Fixed64::ZERO || *duration>Fixed64::from_i32(60)
                || cast_range.is_none_or(|range|range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000)) {
                return Err(format!("ability '{}': invalid control data",id.as_str()));
            }
        }
        if let ResolvedEffect::RevealArea {center,radius,duration,cast_range}=effect {
            let origin=sink.position(caster).ok_or("missing caster position")?;
            if reveal_spec(caster,*center,*radius,*cast_range).is_none() || !omoba_core::runtime::native::reveal_zone::valid_duration(*duration)
                || (*center-origin).length_squared()>*cast_range * *cast_range {
                return Err(format!("ability '{}': invalid reveal area point/data",id.as_str()));
            }
        }
        if let ResolvedEffect::ForcedMoveEnemy {victim,motion,distance,duration,cast_range}=effect {
            let spec=forced_spec(caster,*motion,*distance,*cast_range);
            if spec.is_none() || !omoba_core::runtime::native::forced_motion::valid_duration(*duration)
                || !victim.is_valid() || *victim==caster || !sink.is_alive(*victim) || !sink.is_enemy(caster,*victim) || sink.is_building(*victim) {
                return Err(format!("ability '{}': invalid forced move target/data",id.as_str()));
            }
            let origin=sink.position(caster).ok_or("missing caster position")?;
            let position=sink.position(*victim).ok_or("missing forced move target position")?;
            if (position-origin).length_squared()>*cast_range * *cast_range {return Err("forced move outside cast range".into());}
        }
        if let ResolvedEffect::DrainManaEnemy {victim,cast_range,..} | ResolvedEffect::Damage { victim, cast_range, .. } | ResolvedEffect::SlowEnemy {victim,cast_range,..}
            | ResolvedEffect::ControlEnemy {victim,cast_range,..} = effect {
            if !sink.is_alive(*victim) {
                return Err(format!("ability '{}' target is not alive", id.as_str()));
            }
            if !sink.is_enemy(caster, *victim) {
                return Err(format!("ability '{}' target is not an enemy", id.as_str()));
            }
            if let Some(range)=cast_range {
                let origin=sink.position(caster).ok_or_else(||format!("ability '{}': missing caster position",id.as_str()))?;
                let target=sink.position(*victim).ok_or_else(||format!("ability '{}': missing target position",id.as_str()))?;
                if (target-origin).length_squared()>*range * *range {
                    return Err(format!("ability '{}': entity outside cast range",id.as_str()));
                }
            }
        }
    }
    // The host owns the transaction: on RErr it discards the resource overlay
    // and all other deferred outcomes, including its metadata reservation.
    // Keep authored resource order, using the actual capacity-limited balance,
    // and admit every extra spend before emitting any heal/damage/buff outcome.
    for effect in &expanded {
        match effect {
            ResolvedEffect::RestoreManaSelf {amount}=>sink.restore_mana(caster,*amount),
            ResolvedEffect::SpendManaSelf {amount}=>{
                if !sink.spend_mana(caster,*amount,id) {
                    return Err(format!("ability '{}': additional mana cost rejected",id.as_str()));
                }
            },
            _=>{},
        }
    }
    for effect in expanded {
        match effect {
            ResolvedEffect::CleanseAlly {victim,cast_range}=>{if !sink.cleanse(caster,victim,cast_range){return Err("ally cleanse rejected".into());}},
            ResolvedEffect::DrainManaEnemy {victim,amount,cast_range}=>{
                if !sink.drain_mana(caster,victim,amount,cast_range.expect("preflight range")){return Err("enemy mana drain rejected".into());}
            },
            ResolvedEffect::DamageOverTimeEnemy {victim,rate,duration,kind,..}=>sink.dot(caster,victim,id,rate,duration,kind),
            ResolvedEffect::HealAlly {victim,amount,..}=>sink.heal(victim,amount),
            ResolvedEffect::ShieldAlly {victim,amount,duration,cast_range}=>{
                if !sink.shield(caster,victim,amount,duration,cast_range){return Err("ally shield grant rejected".into());}
            },
            ResolvedEffect::ManaBuffSelf {stat,value,duration}=>sink.mana_buff(caster,id,stat,value,duration),
            ResolvedEffect::Damage {
                victim,
                amount,
                kind,
                ..
            } => sink.damage(caster, victim, amount, kind),
            ResolvedEffect::HealSelf { amount } => sink.heal(caster, amount),
            ResolvedEffect::RestoreManaSelf {..}|ResolvedEffect::SpendManaSelf {..}=>{},
            ResolvedEffect::SlowEnemy {victim,reduction,duration,..}=>sink.slow(caster,victim,id,reduction,duration),
            ResolvedEffect::ControlEnemy {control,victim,duration,..}=>sink.control(victim,control,duration),
            ResolvedEffect::ForcedMoveEnemy {victim,motion,distance,duration,cast_range}=>{
                let spec=forced_spec(caster,motion,distance,cast_range).expect("preflight forced move");
                sink.forced_move(victim,&spec,duration);
            },
            ResolvedEffect::RevealArea {center,radius,duration,cast_range}=>{
                let spec=reveal_spec(caster,center,radius,cast_range).expect("preflight reveal area");
                sink.reveal_area(caster,&spec,duration);
            },
            ResolvedEffect::AreaDamage {..}|ResolvedEffect::ConeDamage {..}|ResolvedEffect::LineDamage {..}=>unreachable!("expanded before commit"),
            ResolvedEffect::DashToPoint {..}=>unreachable!("exclusive movement handled before commit"),
            ResolvedEffect::Projectile {..}|ResolvedEffect::SummonAtPoint {..}|ResolvedEffect::ToggleAttackOrb=>unreachable!("exclusive projectile handled before commit"),
        }
    }
    // A resolved, fully admitted effect, including a valid empty-area cast.
    // Never emit during target expansion or a failed resource admission.
    for (center,radius) in areas { sink.area_cue(center,radius); }
    Ok(())
}

fn resolve_effects(
    id: AbilityId,
    effects: &[EffectOp],
    target: &Target,
    level: u8,
) -> Result<Vec<ResolvedEffect>, String> {
    let ability = active_ability_const(id)
        .ok_or_else(|| format!("unknown generated ability '{}'", id.as_str()))?;
    if level == 0 || level > ability.max_level {
        return Err(format!("ability '{}' invalid level {level}", id.as_str()));
    }
    if let [EffectOp::ToggleAttackOrb]=effects {
        if !matches!(target,Target::None) {return Err(format!("ability '{}': toggle takes no target",id.as_str()));}
        return Ok(vec![ResolvedEffect::ToggleAttackOrb]);
    }
    if effects.iter().any(|op|matches!(op,EffectOp::ToggleAttackOrb)) {
        return Err(format!("ability '{}': attack orb toggle is exclusive",id.as_str()));
    }
    if let [EffectOp::SummonAtPoint {unit,duration_key}]=effects {
        return resolve_summon(id,unit,*duration_key,target,level).map(|effect|vec![effect]);
    }
    if effects.iter().any(|op|matches!(op,EffectOp::SummonAtPoint {..})) {
        return Err(format!("ability '{}': summon is an exclusive delivery",id.as_str()));
    }
    if effects.iter().any(|op|matches!(op,EffectOp::Projectile {..})) {
        return resolve_projectile(id,effects,target,level).map(|effect|vec![effect]);
    }
    let mut resolved = Vec::with_capacity(effects.len());
    let mut mana_stats=std::collections::BTreeSet::new();
    for effect in effects {
        match effect {
            EffectOp::DrainManaEnemy {amount_key}=>{
                let Target::Entity(victim)=target else{return Err("mana drain requires entity target".into());};
                let amount=ability.extras.iter().find(|(key,_)|key==amount_key).and_then(|(_,v)|v.get(usize::from(level-1))).copied().ok_or("missing mana drain extras")?;
                resolved.push(ResolvedEffect::DrainManaEnemy{victim:*victim,amount,cast_range:Some(ability.levels[usize::from(level-1)].range)});
            },
            EffectOp::DamageOverTimeEnemy {amount_key,duration_key,kind}=>{
                let Target::Entity(victim)=target else {return Err("DoT requires entity target".into());};
                let lookup=|key:&&str|ability.extras.iter().find(|(name,_)|name==key)
                    .and_then(|(_,values)|values.get(usize::from(level-1))).copied().ok_or("missing DoT extras");
                resolved.push(ResolvedEffect::DamageOverTimeEnemy {victim:*victim,rate:lookup(amount_key)?,duration:lookup(duration_key)?,
                    kind:*kind,cast_range:ability.levels[usize::from(level-1)].range});
            },
            EffectOp::HealAlly {amount_key}=>{
                let Target::Entity(victim)=target else {return Err("ally heal requires entity target".into());};
                let amount=ability.extras.iter().find(|(key,_)|key==amount_key)
                    .and_then(|(_,values)|values.get(usize::from(level-1))).copied().ok_or("missing ally heal extras")?;
                resolved.push(ResolvedEffect::HealAlly {victim:*victim,amount,cast_range:ability.levels[usize::from(level-1)].range});
            },
            EffectOp::ManaBuffSelf {stat,value_key,duration_key}=>{
                if !matches!(target,Target::None) || !mana_stats.insert(*stat) {
                    return Err(format!("ability '{}': mana buff requires none target and unique stat",id.as_str()));
                }
                let lookup=|key:&&str|ability.extras.iter().find(|(name,_)|name==key)
                    .and_then(|(_,values)|values.get(usize::from(level-1))).copied()
                    .ok_or_else(||format!("ability '{}': missing mana buff extras '{key}'",id.as_str()));
                resolved.push(ResolvedEffect::ManaBuffSelf {stat:*stat,value:lookup(value_key)?,duration:lookup(duration_key)?});
            },
            EffectOp::StunEnemy {duration_key} | EffectOp::ControlEnemy {duration_key,..}=>{
                let control=match effect {EffectOp::ControlEnemy {control,..}=>*control,_=>ControlEffectKind::Stun};
                let Target::Entity(victim)=target else {return Err("stun requires entity target".into());};
                let duration=extra_at(ability,duration_key,level);
                let range=ability.levels[usize::from(level-1)].range;
                if !victim.is_valid() || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60)
                    || range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                    return Err(format!("ability '{}': invalid stun target/data",id.as_str()));
                }
                resolved.push(ResolvedEffect::ControlEnemy {control,victim:*victim,duration,cast_range:Some(range)});
            },
            EffectOp::ForcedMoveEnemy {motion,distance_key,duration_key}=>{
                let Target::Entity(victim)=target else {return Err(format!("ability '{}': forced move requires entity target",id.as_str()));};
                resolved.push(ResolvedEffect::ForcedMoveEnemy {victim:*victim,motion:*motion,distance:extra_at(ability,distance_key,level),
                    duration:extra_at(ability,duration_key,level),cast_range:ability.levels[usize::from(level-1)].range});
            },
            EffectOp::RevealArea {radius_key,duration_key}=>{
                let Target::Point(center)=target else {return Err(format!("ability '{}': reveal area requires a point target",id.as_str()));};
                resolved.push(ResolvedEffect::RevealArea {center:*center,radius:extra_at(ability,radius_key,level),
                    duration:extra_at(ability,duration_key,level),cast_range:ability.levels[usize::from(level-1)].range});
            },
            EffectOp::SlowEnemy {reduction_key,duration_key}=>{
                let Target::Entity(victim)=target else {return Err(format!("ability '{}': slow requires entity target",id.as_str()));};
                let reduction=extra_at(ability,reduction_key,level);let duration=extra_at(ability,duration_key,level);
                let range=ability.levels[usize::from(level-1)].range;
                if !victim.is_valid() || reduction<=Fixed64::ZERO || reduction>Fixed64::ONE
                    || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60)
                    || range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                    return Err(format!("ability '{}': invalid slow target/data",id.as_str()));
                }
                resolved.push(ResolvedEffect::SlowEnemy {victim:*victim,reduction,duration,cast_range:Some(range)});
            },
            EffectOp::DashToPoint=>{
                if effects.len()!=1 {return Err(format!("ability '{}': movement cannot be combined with other effects",id.as_str()));}
                let Target::Point(point)=target else {return Err(format!("ability '{}': dash requires a point target",id.as_str()));};
                let range=ability.levels[usize::from(level-1)].range;
                if range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                    return Err(format!("ability '{}': invalid dash range",id.as_str()));
                }
                resolved.push(ResolvedEffect::DashToPoint {target:*point,cast_range:range});
            },
            EffectOp::LineDamage {amount_key,width_key,kind}=>{
                if effects.len()!=1{return Err("line requires one exclusive effect".into());}
                let Target::Point(aim)=target else{return Err("line requires a direction point".into());};
                let lookup=|key:&&str|ability.extras.iter().find(|(name,_)|name==key).and_then(|(_,values)|values.get(usize::from(level-1))).copied().ok_or("missing line extras");
                resolved.push(ResolvedEffect::LineDamage{aim:*aim,cast_range:ability.levels[usize::from(level-1)].range,width:lookup(width_key)?,amount:lookup(amount_key)?,kind:*kind});
            }
            EffectOp::ConeDamage {amount_key,half_angle_key,kind}=>{
                if effects.len()!=1 {return Err("cone requires one exclusive effect".into());}
                let Target::Point(aim)=target else{return Err("cone requires a direction point".into());};
                let lookup=|key:&&str|ability.extras.iter().find(|(name,_)|name==key)
                    .and_then(|(_,values)|values.get(usize::from(level-1))).copied().ok_or("missing cone extras");
                let angle=lookup(half_angle_key)?;
                let half_ticks=omoba_sim::sector::half_angle_ticks(angle).filter(|ticks|*ticks>0&&*ticks<=1024).ok_or("invalid cone half angle")?;
                resolved.push(ResolvedEffect::ConeDamage {aim:*aim,cast_range:ability.levels[usize::from(level-1)].range,
                    half_ticks,amount:lookup(amount_key)?,kind:*kind});
            }
            EffectOp::Damage { amount_key, kind } => {
                let Target::Entity(victim) = target else {
                    return Err(format!(
                        "ability '{}' requires an entity target",
                        id.as_str()
                    ));
                };
                if !victim.is_valid() {
                    return Err(format!(
                        "ability '{}' has invalid entity target",
                        id.as_str()
                    ));
                }
                resolved.push(ResolvedEffect::Damage {
                    victim: *victim,
                    amount: extra_at(ability, amount_key, level),
                    kind: *kind,
                    cast_range:Some({
                        let range=ability.levels[usize::from(level-1)].range;
                        if range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                            return Err(format!("ability '{}': invalid unit cast range",id.as_str()));
                        }
                        range
                    }),
                });
            }
            EffectOp::RecoveryChannelSelf {..}=>return Err("recovery channel requires authoritative host execution".into()),
            EffectOp::Projectile {..}|EffectOp::SummonAtPoint {..}|EffectOp::ToggleAttackOrb=>unreachable!("projectile delivery resolved before payload ops"),
            EffectOp::Channel {..}=>return Err(format!("ability '{}': pulse channel requires authoritative host execution",id.as_str())),
            EffectOp::HealSelf { amount_key } => {
                if !matches!(target,Target::None) {return Err(format!("ability '{}': self heal requires no target",id.as_str()));}
                resolved.push(ResolvedEffect::HealSelf {amount:extra_at(ability,amount_key,level)});
            },
            EffectOp::CleanseAlly=>{
                let Target::Entity(victim)=target else{return Err("ally cleanse requires entity target".into());};
                resolved.push(ResolvedEffect::CleanseAlly {victim:*victim,cast_range:ability.levels[usize::from(level-1)].range});
            },
            EffectOp::ShieldAlly {amount_key,duration_key}=>{
                let Target::Entity(victim)=target else{return Err("ally shield requires entity target".into());};
                let read=|key:&str|ability.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(usize::from(level-1))).copied()
                    .ok_or_else(||format!("ability '{}': missing shield extras '{key}'",id.as_str()));
                resolved.push(ResolvedEffect::ShieldAlly {victim:*victim,amount:read(amount_key)?,duration:read(duration_key)?,cast_range:ability.levels[usize::from(level-1)].range});
            },
            EffectOp::RestoreManaSelf {amount_key}|EffectOp::SpendManaSelf {amount_key}=>{
                let consumes_target=match target {
                    Target::Entity(_)=>effects.iter().any(|op|matches!(op,EffectOp::CleanseAlly|EffectOp::DrainManaEnemy {..}|EffectOp::Damage {..}|EffectOp::HealAlly {..}|EffectOp::ShieldAlly {..}
                        |EffectOp::SlowEnemy {..}|EffectOp::StunEnemy {..}|EffectOp::ControlEnemy {..}|EffectOp::DamageOverTimeEnemy {..})),
                    Target::Point(_)=>effects.iter().any(|op|matches!(op,EffectOp::AreaDamage {..}|EffectOp::LineDamage {..}|EffectOp::ConeDamage {..})),
                    _=>false,
                };
                if !matches!(target,Target::None) && !(matches!(effect,EffectOp::RestoreManaSelf {..}) && consumes_target) {
                    return Err(format!("ability '{}': self mana effect requires no target",id.as_str()));
                }
                let amount=ability.extras.iter().find(|(key,_)|key==amount_key)
                    .and_then(|(_,values)|values.get(usize::from(level-1))).copied()
                    .ok_or_else(||format!("ability '{}': missing mana effect extras '{amount_key}'",id.as_str()))?;
                resolved.push(if matches!(effect,EffectOp::RestoreManaSelf {..}) {
                    ResolvedEffect::RestoreManaSelf {amount}
                } else {ResolvedEffect::SpendManaSelf {amount}});
            },
            EffectOp::AreaDamage {amount_key,radius_key,kind}=>{
                let Target::Point(center)=target else {return Err(format!("ability '{}' requires a point target",id.as_str()));};
                let radius=extra_at(ability,radius_key,level);
                let cast_range=ability.levels[usize::from(level-1)].range;
                if radius<=Fixed64::ZERO || radius>Fixed64::from_i32(10_000) || cast_range<=Fixed64::ZERO
                    || cast_range>Fixed64::from_i32(10_000) {return Err(format!("ability '{}': invalid area radius/range",id.as_str()));}
                resolved.push(ResolvedEffect::AreaDamage {center:*center,radius,cast_range,
                    amount:extra_at(ability,amount_key,level),kind:*kind});
            }
        }
    }
    Ok(resolved)
}

impl AbilityScript for GenericEffectHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(self.id.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        _level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let resolved = match resolve_effects(self.id, self.effects, &target, level) {
            Ok(value) => value,
            Err(error) => return RErr(error.into()),
        };
        match apply_effects(self.id, caster, resolved, world) {
            Ok(()) => ROk(()),
            Err(error) => RErr(error.into()),
        }
    }

    fn on_channel_pulse(&self, caster: EntityHandle, target: Target, pulse: ChannelPulse, world: &mut GameWorldDyn<'_>) {
        let Some(ability) = active_ability_const(self.id) else { return };
        if pulse.level == 0 || pulse.level > ability.max_level { return; }
        let rank = usize::from(pulse.level - 1);
        let read = |key: &str| ability.extras.iter().find(|(name, _)| *name == key).and_then(|(_, rows)| rows.get(rank)).copied()
            .ok_or_else(|| format!("ability '{}': missing channel extras '{key}'", self.id.as_str()));
        // A malformed pulse applies nothing; the host still ends the channel on schedule.
        let _ = apply_channel_pulse(self.id, self.effects, caster, &target, pulse, &read, world);
    }
}

/// One pulse: the payload ops after `EffectOp::Channel` hit every enemy in the
/// pulse radius around `pulse.anchor`, or only the anchored unit when the
/// channel has no radius. Returns how many units were hit.
fn apply_channel_pulse(
    id: AbilityId,
    effects: &[EffectOp],
    caster: EntityHandle,
    target: &Target,
    pulse: ChannelPulse,
    read: &dyn Fn(&str) -> Result<Fixed64, String>,
    sink: &mut impl EffectSink,
) -> Result<usize, String> {
    let [EffectOp::Channel { radius_key, .. }, payload @ ..] = effects else {
        return Err(format!("ability '{}': channel must lead its payload", id.as_str()));
    };
    let mut victims = match (radius_key, target) {
        (Some(key), _) => {
            let radius = read(key)?;
            sink.area_cue(pulse.anchor, radius);
            sink.enemies_in_radius(caster, pulse.anchor, radius)
        }
        (None, Target::Entity(unit)) => vec![*unit],
        (None, _) => return Err(format!("ability '{}': a channel without radius needs a unit anchor", id.as_str())),
    };
    victims.retain(|victim| victim.is_valid() && *victim != caster && sink.is_alive(*victim) && sink.is_enemy(caster, *victim));
    victims.sort_by_key(|victim| (victim.id, victim.gen));
    victims.dedup();
    for victim in &victims {
        for op in payload {
            match op {
                EffectOp::Damage { amount_key, kind } => sink.damage(caster, *victim, read(amount_key)?, *kind),
                EffectOp::SlowEnemy { reduction_key, duration_key } =>
                    sink.slow(caster, *victim, id, read(reduction_key)?, read(duration_key)?),
                EffectOp::StunEnemy { duration_key } => sink.control(*victim, ControlEffectKind::Stun, read(duration_key)?),
                EffectOp::ControlEnemy { control, duration_key } => sink.control(*victim, *control, read(duration_key)?),
                _ => return Err(format!("ability '{}': channel pulses only damage, slow and control", id.as_str())),
            }
        }
    }
    Ok(victims.len())
}

pub fn generic_effect_ffi(id: AbilityId, effects: &'static [EffectOp]) -> AbilityDefFFI {
    let preview = effects
        .iter()
        // Movement and mana have no matching tooltip EffectSpec. Never pretend
        // mana is HP healing or damage. Lua description/extras remain available.
        .filter(|effect|!matches!(effect,EffectOp::Projectile {..}|EffectOp::SummonAtPoint {..}|EffectOp::ToggleAttackOrb|EffectOp::Channel {..}|EffectOp::CleanseAlly|EffectOp::DrainManaEnemy {..}|EffectOp::RecoveryChannelSelf {..}|EffectOp::DashToPoint|EffectOp::RevealArea {..}|EffectOp::RestoreManaSelf {..}|EffectOp::SpendManaSelf {..}|EffectOp::ManaBuffSelf {..}))
        .map(|effect| match effect {
            EffectOp::RecoveryChannelSelf {..}=>unreachable!("host channels are execution metadata, not preview effects"),
            EffectOp::DamageOverTimeEnemy {amount_key,duration_key,kind}=>EffectSpec::StatusModifier {
                target:TargetSelector::Target,modifier_type:format!("damage_over_time_{kind:?}_per_second"),
                value:extra_at_id_f32(id,amount_key,1),duration:Some(extra_at_id_f32(id,duration_key,1)),
            },
            EffectOp::StunEnemy {duration_key}=>EffectSpec::StatusModifier {
                target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
                duration:Some(extra_at_id_f32(id,duration_key,1)),
            },
            EffectOp::ControlEnemy {control,duration_key}=>EffectSpec::StatusModifier {
                target:TargetSelector::Target,modifier_type:control.as_str().into(),value:1.0,
                duration:Some(extra_at_id_f32(id,duration_key,1)),
            },
            EffectOp::ForcedMoveEnemy {motion,distance_key,duration_key}=>EffectSpec::StatusModifier {
                target:TargetSelector::Target,modifier_type:motion.as_str().into(),
                value:extra_at_id_f32(id,distance_key,1),duration:Some(extra_at_id_f32(id,duration_key,1)),
            },
            EffectOp::SlowEnemy {reduction_key,duration_key}=>EffectSpec::StatusModifier {
                target:TargetSelector::Target,modifier_type:"slow".into(),
                value:extra_at_id_f32(id,reduction_key,1),duration:Some(extra_at_id_f32(id,duration_key,1)),
            },
            EffectOp::Projectile {..}|EffectOp::SummonAtPoint {..}|EffectOp::ToggleAttackOrb|EffectOp::Channel {..}|EffectOp::CleanseAlly|EffectOp::DrainManaEnemy {..}|EffectOp::DashToPoint|EffectOp::RevealArea {..}|EffectOp::RestoreManaSelf {..}|EffectOp::SpendManaSelf {..}|EffectOp::ManaBuffSelf {..}=>unreachable!("effect has no matching preview schema"),
            EffectOp::Damage { amount_key, kind } => EffectSpec::Damage {
                target: TargetSelector::Target,
                amount: extra_at_id_f32(id, amount_key, 1),
                damage_type: match kind {
                    DamageKind::Physical => DamageType::Physical,
                    DamageKind::Magical => DamageType::Magical,
                    DamageKind::Pure => DamageType::Pure,
                },
            },
            EffectOp::HealSelf { amount_key } => EffectSpec::Heal {
                target: TargetSelector::SelfUnit,
                amount: extra_at_id_f32(id, amount_key, 1),
            },
            EffectOp::HealAlly {amount_key}=>EffectSpec::Heal {target:TargetSelector::Target,amount:extra_at_id_f32(id,amount_key,1)},
            EffectOp::ShieldAlly {amount_key,duration_key}=>EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"damage_shield".into(),
                value:extra_at_id_f32(id,amount_key,1),duration:Some(extra_at_id_f32(id,duration_key,1))},
            EffectOp::LineDamage {amount_key,width_key,kind}=>EffectSpec::Damage {
                target:TargetSelector::Custom(format!("enemies_in_source_line(width={})",extra_at_id_f32(id,width_key,1))),
                amount:extra_at_id_f32(id,amount_key,1),damage_type:match kind {DamageKind::Physical=>DamageType::Physical,DamageKind::Magical=>DamageType::Magical,DamageKind::Pure=>DamageType::Pure},
            },
            EffectOp::ConeDamage {amount_key,half_angle_key,kind}=>EffectSpec::Damage {
                target:TargetSelector::Custom(format!("enemies_in_source_cone(half_angle_rad={})",extra_at_id_f32(id,half_angle_key,1))),
                amount:extra_at_id_f32(id,amount_key,1),damage_type:match kind {DamageKind::Physical=>DamageType::Physical,DamageKind::Magical=>DamageType::Magical,DamageKind::Pure=>DamageType::Pure},
            },
            EffectOp::AreaDamage {amount_key,radius_key,kind}=>EffectSpec::Damage {
                target:TargetSelector::Custom(format!("enemies_at_target_point(radius={})",extra_at_id_f32(id,radius_key,1))),
                amount:extra_at_id_f32(id,amount_key,1),
                damage_type:match kind {DamageKind::Physical=>DamageType::Physical,DamageKind::Magical=>DamageType::Magical,DamageKind::Pure=>DamageType::Pure},
            },
        })
        .collect();
    let execution=match effects {
        [EffectOp::RecoveryChannelSelf {amount_key,duration_key,interval_key}]=>Some(omoba_core::ability_meta::HostAbilityExecution::RecoveryChannelSelf {
            amount_key:(*amount_key).into(),duration_key:(*duration_key).into(),interval_key:(*interval_key).into(),
        }),
        [EffectOp::Channel {anchor,interval_key,pulses_key,radius_key},..]=>Some(omoba_core::ability_meta::HostAbilityExecution::PulseChannel {
            anchor:*anchor,interval_key:(*interval_key).into(),pulses_key:(*pulses_key).into(),radius_key:radius_key.map(Into::into),
        }),_=>None,
    };
    crate::ability_builder::build_ability_ffi_with_execution(id, GenericEffectHandler { id, effects }, preview,execution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_template_ids::ABILITY_FLAME_BLADE;

    #[test]
    fn authored_bleed_all_ranks_validate_before_any_effect_and_expose_rate_duration() {
        let id=omoba_template_ids::ability_by_name("ranger_shot").unwrap();
        let caster=EntityHandle{id:1,gen:1};let victim=EntityHandle{id:2,gen:1};
        let op=EffectOp::DamageOverTimeEnemy{amount_key:"bleed_rate",duration_key:"bleed_duration",kind:DamageKind::Physical};
        for rank in 1..=4 {
            let mut sink=HeadlessWorld {alive:true,enemy:true,..Default::default()};
            let resolve=||resolve_effects(id,&[op],&Target::Entity(victim),rank).unwrap();
            apply_effects(id,caster,resolve(),&mut sink).unwrap();assert_eq!(sink.dots,vec![(2,i64::from(rank)*5*1024+5*1024,4096)]);
            sink.enemy=false;assert!(apply_effects(id,caster,resolve(),&mut sink).is_err());assert_eq!(sink.dots.len(),1);
            sink.enemy=true;sink.positions.insert(2,Vec2::new(Fixed64::from_i32(751),Fixed64::ZERO));
            assert!(apply_effects(id,caster,resolve(),&mut sink).is_err());assert_eq!(sink.dots.len(),1);
            sink.positions.clear();let mut invalid=resolve();invalid.push(ResolvedEffect::SpendManaSelf{amount:Fixed64::from_i32(-1)});
            assert!(apply_effects(id,caster,invalid,&mut sink).is_err());assert_eq!(sink.dots.len(),1);
            let duplicate=resolve_effects(id,&[op,op],&Target::Entity(victim),rank).unwrap();
            assert!(apply_effects(id,caster,duplicate,&mut sink).is_err());assert_eq!(sink.dots.len(),1);
        }
        assert!(resolve_effects(id,&[op],&Target::None,1).is_err());
        assert!(resolve_effects(id,&[op],&Target::Entity(victim),0).is_err());
    }

    #[derive(Default)]
    struct HeadlessWorld {
        building:Option<u32>,
        dots:Vec<(u32,i64,i64)>,
        mana_buffs:Vec<(ManaBuffStat,i64,i64)>,
        mana:Option<omoba_core::runtime::ability_runtime::ManaPool>,
        mana_recipients:Vec<EntityHandle>,
        shields:Vec<(EntityHandle,i64,i64)>,
        cleanses:Vec<EntityHandle>,
        drains:Vec<(EntityHandle,i64)>,
        damage: Vec<(u32, i64)>,
        heals: Vec<i64>,
        alive: bool,
        enemy: bool,
        enemies: Vec<EntityHandle>,
        positions: std::collections::HashMap<u32,Vec2>,
        blocked: bool,
        slows:Vec<(u32,i64,i64)>,
        stuns:Vec<(u32,i64)>,
        controls:Vec<(u32,ControlEffectKind,i64)>,
        forced:Vec<(u32,omoba_core::runtime::native::forced_motion::Spec,i64)>,
        reveals:Vec<(omoba_core::runtime::native::reveal_zone::Spec,i64)>,
        launches:Vec<(u32,i64,String)>,
        summons:Vec<(u32,Vec2,String,i64)>,
        area_cues:Vec<(Vec2,Fixed64)>,
    }

    #[test]
    fn channel_pulse_hits_each_live_enemy_once_at_the_locked_rank_or_only_its_unit() {
        let id = omoba_template_ids::ability_by_name("ranger_shot").unwrap();
        let caster = EntityHandle { id: 1, gen: 1 };
        let rows = |key: &str| -> Result<Fixed64, String> {
            match key { "r" => Ok(Fixed64::from_i32(340)), "dmg" => Ok(Fixed64::from_i32(60)), "stun" => Ok(Fixed64::from_raw(307)),
                other => Err(format!("missing {other}")) }
        };
        let area = [
            EffectOp::Channel { anchor: omoba_content_model::ChannelAnchor::Point, interval_key: "i", pulses_key: "n", radius_key: Some("r") },
            EffectOp::Damage { amount_key: "dmg", kind: DamageKind::Magical },
            EffectOp::StunEnemy { duration_key: "stun" },
        ];
        let anchor = Vec2::new(Fixed64::from_i32(500), Fixed64::ZERO);
        let pulse = ChannelPulse { level: 2, index: 3, total: 60, elapsed: Fixed64::from_raw(306), anchor };
        let mut sink = HeadlessWorld { alive: true, enemy: true, ..Default::default() };
        // Unsorted, duplicated, the caster itself, and a dead unit (id 9).
        sink.enemies = vec![EntityHandle { id: 4, gen: 1 }, EntityHandle { id: 2, gen: 1 }, EntityHandle { id: 4, gen: 1 }, caster, EntityHandle { id: 9, gen: 1 }];
        let hit = apply_channel_pulse(id, &area, caster, &Target::Point(anchor), pulse, &rows, &mut sink).unwrap();
        assert_eq!(hit, 2);
        assert_eq!(sink.damage, vec![(2, 60 * 1024), (4, 60 * 1024)]);
        assert_eq!(sink.stuns, vec![(2, 307), (4, 307)]);
        assert_eq!(sink.area_cues, vec![(anchor, Fixed64::from_i32(340))]);
        // A unit anchor without a radius hits that unit alone.
        let unit = [
            EffectOp::Channel { anchor: omoba_content_model::ChannelAnchor::Unit, interval_key: "i", pulses_key: "n", radius_key: None },
            EffectOp::Damage { amount_key: "dmg", kind: DamageKind::Magical },
        ];
        let mut sink = HeadlessWorld { alive: true, enemy: true, ..Default::default() };
        sink.enemies = vec![EntityHandle { id: 5, gen: 1 }];
        let victim = EntityHandle { id: 7, gen: 1 };
        assert_eq!(apply_channel_pulse(id, &unit, caster, &Target::Entity(victim), pulse, &rows, &mut sink).unwrap(), 1);
        assert_eq!(sink.damage, vec![(7, 60 * 1024)]);
        sink.enemy = false;
        assert_eq!(apply_channel_pulse(id, &unit, caster, &Target::Entity(victim), pulse, &rows, &mut sink).unwrap(), 0, "allies are never hit");
        // Missing metadata applies nothing; a non-payload op is refused.
        let mut sink = HeadlessWorld { alive: true, enemy: true, ..Default::default() };
        sink.enemies = vec![EntityHandle { id: 2, gen: 1 }];
        let broken = [area[0], EffectOp::Damage { amount_key: "nope", kind: DamageKind::Magical }];
        assert!(apply_channel_pulse(id, &broken, caster, &Target::Point(anchor), pulse, &rows, &mut sink).is_err());
        let heal = [area[0], EffectOp::HealSelf { amount_key: "dmg" }];
        assert!(apply_channel_pulse(id, &heal, caster, &Target::Point(anchor), pulse, &rows, &mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.heals.is_empty());
    }

    #[test]
    fn ally_cleanse_resolves_friendly_target_composes_and_validates_before_mutation() {
        let id=omoba_template_ids::ability_by_name("lumen_aid").unwrap();let caster=EntityHandle{id:1,gen:1};let ally=EntityHandle{id:2,gen:1};
        let mut sink=HeadlessWorld{alive:true,enemy:false,..Default::default()};sink.positions.insert(1,Vec2::ZERO);sink.positions.insert(2,Vec2::new(Fixed64::from_i32(600),Fixed64::ZERO));
        let ops=[EffectOp::RestoreManaSelf{amount_key:"self_mana_restore"},EffectOp::CleanseAlly];
        apply_effects(id,caster,resolve_effects(id,&ops,&Target::Entity(ally),1).unwrap(),&mut sink).unwrap();assert_eq!(sink.cleanses,vec![ally]);assert_eq!(sink.mana_recipients,vec![caster]);
        for case in 0..4 {sink.cleanses.clear();sink.mana_recipients.clear();sink.enemy=case==0;sink.alive=case!=1;
            sink.positions.insert(2,Vec2::new(Fixed64::from_i32(600)+if case==2{Fixed64::from_raw(1)}else{Fixed64::ZERO},Fixed64::ZERO));
            let mut plan=resolve_effects(id,&ops,&Target::Entity(ally),1).unwrap();if case==3{plan.push(ResolvedEffect::CleanseAlly{victim:ally,cast_range:Fixed64::ZERO});}
            assert!(apply_effects(id,caster,plan,&mut sink).is_err());assert!(sink.cleanses.is_empty()&&sink.mana_recipients.is_empty());}
        for target in [Target::None,Target::Point(Vec2::ZERO)]{assert!(resolve_effects(id,&[EffectOp::CleanseAlly],&target,1).is_err());}
    }

    #[test]
    fn linear_damage_hits_closed_capsule_once_canonically_at_every_rank_without_a_circle_cue() {
        let id=omoba_template_ids::ability_by_name("lumen_ray").unwrap();let caster=EntityHandle{id:1,gen:1};
        let handle=|id|EntityHandle{id,gen:1};let p=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let mut world=HeadlessWorld{alive:true,enemy:true,enemies:vec![handle(8),handle(7),handle(6),handle(5),handle(4),handle(3),handle(2),handle(2),caster],building:Some(7),..Default::default()};
        world.positions.extend([(1,p(0,0)),(2,p(500,60)),(3,p(960,0)),(4,p(-60,0)),(5,p(500,61)),(6,p(961,0)),(7,p(400,0)),(8,p(-61,0))]);
        let op=EffectOp::LineDamage{amount_key:"damage",width_key:"line_width",kind:DamageKind::Magical};
        for rank in 1..=4 {world.damage.clear();apply_effects(id,caster,resolve_effects(id,&[op],&Target::Point(p(9000,0)),rank).unwrap(),&mut world).unwrap();
            assert_eq!(world.damage,vec![(2,(50+i64::from(rank)*50)*1024),(3,(50+i64::from(rank)*50)*1024),(4,(50+i64::from(rank)*50)*1024)]);assert!(world.area_cues.is_empty());}
        world.damage.clear();apply_effects(id,caster,resolve_effects(id,&[op],&Target::Point(p(100,0)),1).unwrap(),&mut world).unwrap();
        assert_eq!(world.damage,vec![(4,100*1024)],"short endpoint is not silently extended");
    }
    #[test]
    fn linear_damage_rejects_bad_axes_metadata_and_crowds_before_any_output() {
        let id=omoba_template_ids::ability_by_name("lumen_ray").unwrap();let caster=EntityHandle{id:1,gen:1};
        let op=EffectOp::LineDamage{amount_key:"damage",width_key:"line_width",kind:DamageKind::Magical};
        let mut world=HeadlessWorld{alive:true,enemy:true,..Default::default()};world.positions.insert(1,Vec2::ZERO);
        for aim in [Vec2::ZERO,Vec2::new(Fixed64::from_raw(i64::MAX),Fixed64::ZERO)]{
            assert!(apply_effects(id,caster,resolve_effects(id,&[op],&Target::Point(aim),1).unwrap(),&mut world).is_err());assert!(world.damage.is_empty());}
        let aim=Vec2::new(Fixed64::from_i32(900),Fixed64::ZERO);
        for n in 10..139{world.enemies.push(EntityHandle{id:n,gen:1});world.positions.insert(n,aim);}
        assert!(apply_effects(id,caster,resolve_effects(id,&[op],&Target::Point(aim),1).unwrap(),&mut world).is_err());assert!(world.damage.is_empty());
        assert!(resolve_effects(id,&[op,op],&Target::Point(aim),1).is_err());assert!(resolve_effects(id,&[op],&Target::None,1).is_err());
        assert!(resolve_effects(id,&[EffectOp::LineDamage{amount_key:"damage",width_key:"missing",kind:DamageKind::Magical}],&Target::Point(aim),1).is_err());
    }
    #[test]
    fn directional_cone_filters_canonical_candidates_and_rejects_before_any_commit() {
        let id=omoba_template_ids::ability_by_name("ranger_scatter").unwrap();
        let caster=EntityHandle{id:1,gen:1};let handle=|id|EntityHandle{id,gen:1};
        let point=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let op=EffectOp::ConeDamage{amount_key:"damage",half_angle_key:"half_angle",kind:DamageKind::Physical};
        let mut world=HeadlessWorld{alive:true,enemy:true,enemies:vec![handle(5),handle(3),handle(2),handle(2),handle(9),caster],..Default::default()};
        world.positions.extend([(1,point(0,0)),(2,point(700,0)),(3,point(100,0)),(5,point(-10,0))]);
        for rank in 1..=4 {
            world.damage.clear();let effects=resolve_effects(id,&[op],&Target::Point(point(1,0)),rank).unwrap();
            apply_effects(id,caster,effects,&mut world).unwrap();
            assert_eq!(world.damage,vec![(2,(110+(i64::from(rank)-1)*50)*1024),(3,(110+(i64::from(rank)-1)*50)*1024)]);
            assert!(world.area_cues.is_empty(),"never emit a misleading circle cue");
        }
        for aim in [point(0,0),point(701,0)] {
            world.damage.clear();assert!(apply_effects(id,caster,resolve_effects(id,&[op],&Target::Point(aim),1).unwrap(),&mut world).is_err());assert!(world.damage.is_empty());
        }
        world.enemies=vec![handle(2),handle(3),handle(4),handle(5)];
        world.positions.extend([(2,point(0,700)),(3,point(0,-700)),(4,point(701,0)),(5,point(-1,0))]);world.damage.clear();
        apply_effects(id,caster,vec![ResolvedEffect::ConeDamage{aim:point(1,0),cast_range:Fixed64::from_i32(700),half_ticks:1024,amount:Fixed64::ONE,kind:DamageKind::Physical}],&mut world).unwrap();
        assert_eq!(world.damage,vec![(2,1024),(3,1024)]);
        world.damage.clear();world.enemies=(10..139).map(handle).collect();
        assert!(apply_effects(id,caster,resolve_effects(id,&[op],&Target::Point(point(1,0)),1).unwrap(),&mut world).is_err());assert!(world.damage.is_empty());
        assert!(resolve_effects(id,&[op,op],&Target::Point(point(1,0)),1).is_err());
        assert!(resolve_effects(id,&[op],&Target::None,1).is_err());
        assert!(resolve_effects(id,&[EffectOp::ConeDamage{amount_key:"missing",half_angle_key:"half_angle",kind:DamageKind::Physical}],&Target::Point(point(1,0)),1).is_err());
    }

    impl EffectSink for HeadlessWorld {
        fn cleanse(&mut self,_caster:EntityHandle,victim:EntityHandle,_range:Fixed64)->bool {self.cleanses.push(victim);true}
        fn drain_mana(&mut self,_caster:EntityHandle,victim:EntityHandle,amount:Fixed64,_range:Fixed64)->bool {
            self.drains.push((victim,amount.raw()));true
        }
        fn shield(&mut self,_caster:EntityHandle,victim:EntityHandle,amount:Fixed64,duration:Fixed64,_range:Fixed64)->bool {
            self.shields.push((victim,amount.raw(),duration.raw()));true
        }
        fn is_building(&self,entity:EntityHandle)->bool {self.building==Some(entity.id)}
        fn dot(&mut self,_caster:EntityHandle,victim:EntityHandle,_ability:AbilityId,rate:Fixed64,duration:Fixed64,_kind:DamageKind) {
            self.dots.push((victim.id,rate.raw(),duration.raw()));
        }
        fn area_cue(&mut self,center:Vec2,radius:Fixed64) {self.area_cues.push((center,radius));}
        fn is_ally(&self,_caster:EntityHandle,_victim:EntityHandle)->bool { !self.enemy }
        fn mana_buff(&mut self,_caster:EntityHandle,_ability:AbilityId,stat:ManaBuffStat,value:Fixed64,duration:Fixed64) {
            self.mana_buffs.push((stat,value.raw(),duration.raw()));
        }
        fn restore_mana(&mut self,caster:EntityHandle,amount:Fixed64) {
            self.mana_recipients.push(caster);
            if let Some(pool)=&mut self.mana {let _=pool.restore(amount);}
        }
        fn spend_mana(&mut self,_caster:EntityHandle,amount:Fixed64,_ability:AbilityId)->bool {
            self.mana.as_mut().is_none_or(|pool|pool.spend(amount).is_ok())
        }
        fn control(&mut self,victim:EntityHandle,control:ControlEffectKind,duration:Fixed64) {
            if control==ControlEffectKind::Stun {self.stuns.push((victim.id,duration.raw()));}
            self.controls.push((victim.id,control,duration.raw()));
        }
        fn forced_move(&mut self,victim:EntityHandle,spec:&omoba_core::runtime::native::forced_motion::Spec,duration:Fixed64) {
            self.forced.push((victim.id,*spec,duration.raw()));
        }
        fn reveal_area(&mut self,_caster:EntityHandle,spec:&omoba_core::runtime::native::reveal_zone::Spec,duration:Fixed64) {
            self.reveals.push((*spec,duration.raw()));
        }
        fn slow(&mut self,_caster:EntityHandle,victim:EntityHandle,_ability:AbilityId,reduction:Fixed64,duration:Fixed64) {
            self.slows.push((victim.id,reduction.raw(),duration.raw()));
        }
        fn collision_destination(&mut self,caster:EntityHandle,target:Vec2,_range:Fixed64) -> Vec2 {
            if self.blocked {self.position(caster).unwrap()} else {target}
        }
        fn relocate(&mut self,caster:EntityHandle,destination:Vec2) {self.positions.insert(caster.id,destination);}
        fn position(&self,entity:EntityHandle) -> Option<Vec2> {Some(self.positions.get(&entity.id).copied().unwrap_or(Vec2::ZERO))}
        fn enemies_in_radius(&self,_caster:EntityHandle,_center:Vec2,_radius:Fixed64) -> Vec<EntityHandle> {self.enemies.clone()}
        fn is_alive(&self, entity: EntityHandle) -> bool {
            self.alive && entity.id!=9
        }
        fn is_enemy(&self, _caster: EntityHandle, _victim: EntityHandle) -> bool {
            self.enemy
        }
        fn damage(
            &mut self,
            _caster: EntityHandle,
            victim: EntityHandle,
            amount: Fixed64,
            _kind: DamageKind,
        ) {
            self.damage.push((victim.id, amount.raw()));
        }
        fn heal(&mut self, _caster: EntityHandle, amount: Fixed64) {
            self.heals.push(amount.raw());
        }
        fn launch_projectile(&mut self, caster: EntityHandle, lifetime: Fixed64, payload: &str) {
            self.launches.push((caster.id, lifetime.raw(), payload.to_owned()));
        }
        fn summon(&mut self, caster: EntityHandle, point: Vec2, unit: &str, duration: Fixed64) -> bool {
            self.summons.push((caster.id, point, unit.to_owned(), duration.raw()));
            !self.blocked
        }
        fn toggle_attack_orb(&mut self, caster: EntityHandle, ability: &str) {
            self.summons.push((caster.id, Vec2::ZERO, format!("orb:{ability}"), 0));
        }
    }

    #[test]
    fn ally_heal_generated_effect_checks_team_life_range_and_whole_plan_before_commit() {
        let id=omoba_template_ids::ability_by_name("lumen_aid").unwrap();
        let caster=EntityHandle {id:1,gen:1};let ally=EntityHandle {id:2,gen:1};
        let effects=resolve_effects(id,&[EffectOp::HealAlly {amount_key:"heal"}],&Target::Entity(ally),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,..Default::default()};
        apply_effects(id,caster,effects,&mut sink).unwrap();assert_eq!(sink.heals,vec![140*1024]);
        sink.enemy=true;
        assert!(apply_effects(id,caster,resolve_effects(id,&[EffectOp::HealAlly {amount_key:"heal"}],
            &Target::Entity(ally),1).unwrap(),&mut sink).is_err());assert_eq!(sink.heals.len(),1);
        sink.enemy=false;sink.positions.insert(2,Vec2::new(Fixed64::from_i32(601),Fixed64::ZERO));
        assert!(apply_effects(id,caster,resolve_effects(id,&[EffectOp::HealAlly {amount_key:"heal"}],
            &Target::Entity(ally),1).unwrap(),&mut sink).is_err());assert_eq!(sink.heals.len(),1);
        assert!(resolve_effects(id,&[EffectOp::HealAlly {amount_key:"missing"}],&Target::Entity(ally),1).is_err());
        assert!(resolve_effects(id,&[EffectOp::HealAlly {amount_key:"heal"}],&Target::None,1).is_err());
        sink.positions.clear();
        for victim in [EntityHandle::INVALID,EntityHandle {id:9,gen:1}] {
            assert!(apply_effects(id,caster,resolve_effects(id,&[EffectOp::HealAlly {amount_key:"heal"}],
                &Target::Entity(victim),1).unwrap(),&mut sink).is_err());
        }
        let mut mixed=resolve_effects(id,&[EffectOp::HealAlly {amount_key:"heal"}],&Target::Entity(ally),1).unwrap();
        mixed.push(ResolvedEffect::Damage {victim:ally,amount:Fixed64::ONE,kind:DamageKind::Pure,cast_range:None});
        assert!(apply_effects(id,caster,mixed,&mut sink).is_err());assert_eq!(sink.heals.len(),1);
    }

    #[test]
    fn mana_drain_generic_checks_ranks_targets_and_all_siblings_before_mutation() {
        let id=omoba_template_ids::ability_by_name("lumen_bolt").unwrap();let caster=EntityHandle{id:1,gen:1};let victim=EntityHandle{id:2,gen:1};
        let ops=[EffectOp::Damage{amount_key:"damage",kind:DamageKind::Magical},EffectOp::DrainManaEnemy{amount_key:"mana_drain"}];
        for rank in 1u8..=4{let plan=||resolve_effects(id,&ops,&Target::Entity(victim),rank).unwrap();
            let mut sink=HeadlessWorld{alive:true,enemy:true,..Default::default()};
            for failure in 0..3{sink.enemy=failure!=0;sink.alive=failure!=1;sink.positions.insert(2,Vec2::new(Fixed64::from_i32(600)+if failure==2{Fixed64::from_raw(1)}else{Fixed64::ZERO},Fixed64::ZERO));
                assert!(apply_effects(id,caster,plan(),&mut sink).is_err());assert!(sink.damage.is_empty()&&sink.drains.is_empty());}
            sink.enemy=true;sink.alive=true;sink.positions.insert(2,Vec2::new(Fixed64::from_i32(600),Fixed64::ZERO));apply_effects(id,caster,plan(),&mut sink).unwrap();
            assert_eq!(sink.drains,vec![(victim,i64::from(10+10*rank)*1024)]);assert_eq!(sink.damage.len(),1);
            for amount in [Fixed64::ZERO,Fixed64::from_raw(-1),Fixed64::from_i32(1_000_001)]{let mut bad=plan();bad.push(ResolvedEffect::DrainManaEnemy{victim,amount,cast_range:Some(Fixed64::from_i32(600))});assert!(apply_effects(id,caster,bad,&mut sink).is_err());assert_eq!(sink.damage.len(),1);assert_eq!(sink.drains.len(),1);}
        }
        assert!(resolve_effects(id,&[EffectOp::DrainManaEnemy{amount_key:"missing"}],&Target::Entity(victim),1).is_err());
        for target in [Target::None,Target::Point(Vec2::ZERO)]{assert!(resolve_effects(id,&ops,&target,1).is_err());}
    }

    #[test]
    fn ally_shield_generic_ranks_recipients_and_whole_plan_preflight() {
        use omoba_core::runtime::ability_runtime::ManaPool;
        let id=omoba_template_ids::ability_by_name("lumen_aid").unwrap();let caster=EntityHandle{id:1,gen:1};let ally=EntityHandle{id:2,gen:1};
        for rank in 1u8..=4 {for reverse in [false,true] {
            let mut ops=vec![EffectOp::ShieldAlly {amount_key:"shield",duration_key:"shield_duration"},EffectOp::RestoreManaSelf {amount_key:"self_mana_restore"}];if reverse{ops.reverse();}
            let plan=||resolve_effects(id,&ops,&Target::Entity(ally),rank).unwrap();
            let mut sink=HeadlessWorld {alive:true,mana:Some(ManaPool::new(Fixed64::from_i32(20),Fixed64::from_i32(100)).unwrap()),..Default::default()};
            for failure in 0..3 {sink.enemy=failure==0;sink.alive=failure!=1;
                sink.positions.insert(2,Vec2::new(Fixed64::from_i32(600)+if failure==2{Fixed64::from_raw(1)}else{Fixed64::ZERO},Fixed64::ZERO));
                assert!(apply_effects(id,caster,plan(),&mut sink).is_err());assert!(sink.shields.is_empty()&&sink.heals.is_empty()&&sink.mana_recipients.is_empty());
                assert_eq!(sink.mana.as_ref().unwrap().current(),Fixed64::from_i32(20));}
            sink.alive=true;sink.enemy=false;sink.positions.insert(2,Vec2::new(Fixed64::from_i32(600),Fixed64::ZERO));
            apply_effects(id,caster,plan(),&mut sink).unwrap();
            assert_eq!(sink.shields,vec![(ally,(70+30*i64::from(rank-1))*1024,2*1024)]);assert!(sink.heals.is_empty());
            assert_eq!(sink.mana_recipients,vec![caster]);
            for (amount,duration) in [(Fixed64::ZERO,Fixed64::ONE),(Fixed64::ONE,Fixed64::ZERO),(Fixed64::ONE,Fixed64::from_i32(61))] {
                assert!(apply_effects(id,caster,vec![ResolvedEffect::RestoreManaSelf {amount:Fixed64::ONE},ResolvedEffect::ShieldAlly {victim:ally,amount,duration,cast_range:Fixed64::from_i32(600)}],&mut sink).is_err());assert_eq!(sink.shields.len(),1);}
        }}
        assert!(resolve_effects(id,&[EffectOp::ShieldAlly {amount_key:"shield",duration_key:"missing"}],&Target::Entity(ally),1).is_err());
        for target in [Target::None,Target::Point(Vec2::ZERO)]{assert!(resolve_effects(id,&[EffectOp::ShieldAlly {amount_key:"shield",duration_key:"shield_duration"}],&target,1).is_err());}
    }

    #[test]
    fn composed_mana_targets_caster_caps_all_ranks_and_preflights_before_any_effect() {
        use omoba_core::runtime::ability_runtime::ManaPool;
        let id=omoba_template_ids::ability_by_name("lumen_aid").unwrap();
        let caster=EntityHandle {id:1,gen:1};let ally=EntityHandle {id:2,gen:1};
        for rank in 1..=4 {
            for reverse in [false,true] {
                let mut ops=vec![EffectOp::HealAlly {amount_key:"heal"},EffectOp::RestoreManaSelf {amount_key:"self_mana_restore"}];
                if reverse {ops.reverse();}
                let plan=||resolve_effects(id,&ops,&Target::Entity(ally),rank).unwrap();
                let mut sink=HeadlessWorld {alive:true,mana:Some(ManaPool::new(Fixed64::from_i32(20),Fixed64::from_i32(100)).unwrap()),..Default::default()};
                for failure in 0..3 {
                    sink.enemy=failure==0;sink.alive=failure!=1;
                    sink.positions.insert(2,Vec2::new(Fixed64::from_i32(if failure==2 {601}else{200}),Fixed64::ZERO));
                    assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
                    assert_eq!(sink.mana.as_ref().unwrap().current(),Fixed64::from_i32(20));
                    assert!(sink.mana_recipients.is_empty() && sink.heals.is_empty());
                }
                sink.alive=true;sink.enemy=false;sink.positions.clear();
                apply_effects(id,caster,plan(),&mut sink).unwrap();
                assert_eq!(sink.heals,vec![[140,220,300,380][usize::from(rank-1)]*1024]);
                assert_eq!(sink.mana_recipients,vec![caster]);
                assert_eq!(sink.mana.as_ref().unwrap().current(),Fixed64::from_i32(20+10+5*i32::from(rank-1)));
                sink.mana=Some(ManaPool::new(Fixed64::from_i32(98),Fixed64::from_i32(100)).unwrap());
                apply_effects(id,caster,plan(),&mut sink).unwrap();
                assert_eq!(sink.mana.as_ref().unwrap().current(),Fixed64::from_i32(100));
            }
        }
    }

    #[test]
    fn mana_buff_declaration_preflight_rejects_duplicate_missing_and_invalid_before_commit() {
        let id=omoba_template_ids::ability_by_name("ranger_patch").unwrap();
        let op=EffectOp::ManaBuffSelf {stat:ManaBuffStat::ManaBonus,value_key:"heal",duration_key:"heal"};
        assert!(resolve_effects(id,&[op,op],&Target::None,1).is_err());
        assert!(resolve_effects(id,&[op],&Target::Point(Vec2::ZERO),1).is_err());
        assert!(resolve_effects(id,&[EffectOp::ManaBuffSelf {stat:ManaBuffStat::ManaBonus,
            value_key:"missing",duration_key:"heal"}],&Target::None,1).is_err());
        let caster=EntityHandle {id:1,gen:1};let mut sink=HeadlessWorld {alive:true,..Default::default()};
        apply_effects(id,caster,resolve_effects(id,&[op],&Target::None,1).unwrap(),&mut sink).unwrap();
        assert_eq!(sink.mana_buffs,vec![(ManaBuffStat::ManaBonus,55*1024,55*1024)]);
        let invalid=resolve_effects(id,&[op],&Target::None,2).unwrap(); // rank2 duration85>60
        assert!(apply_effects(id,caster,invalid,&mut sink).is_err());
        assert_eq!(sink.mana_buffs.len(),1);
    }

    #[test]
    fn mana_effect_preflight_rejects_wrong_targets_missing_data_and_invalid_amounts() {
        let id=omoba_template_ids::ability_by_name("ranger_patch").unwrap();
        for op in [EffectOp::RestoreManaSelf {amount_key:"heal"},EffectOp::SpendManaSelf {amount_key:"heal"}] {
            for target in [Target::Entity(EntityHandle {id:2,gen:1}),Target::Point(Vec2::ZERO)] {
                assert!(resolve_effects(id,&[op],&target,1).is_err());
            }
            assert!(resolve_effects(id,&[op],&Target::None,0).is_err());
        }
        assert!(resolve_effects(id,&[EffectOp::RestoreManaSelf {amount_key:"missing"}],&Target::None,1).is_err());
        let caster=EntityHandle {id:1,gen:1};
        let mut sink=HeadlessWorld {alive:true,mana:Some(
            omoba_core::runtime::ability_runtime::ManaPool::new(Fixed64::from_i32(20),Fixed64::from_i32(100)).unwrap()),..Default::default()};
        for amount in [Fixed64::from_raw(-1),Fixed64::from_i32(1_000_001)] {
            assert!(apply_effects(id,caster,vec![
                ResolvedEffect::RestoreManaSelf {amount:Fixed64::ONE},
                ResolvedEffect::HealSelf {amount:Fixed64::ONE},
                ResolvedEffect::SpendManaSelf {amount},
            ],&mut sink).is_err());
            assert_eq!(sink.mana.as_ref().unwrap().current(),Fixed64::from_i32(20));
            assert!(sink.heals.is_empty());
        }
        // No new resource pool is invented for legacy rules.
        sink.mana=None;
        apply_effects(id,caster,vec![ResolvedEffect::RestoreManaSelf {amount:Fixed64::ONE},
            ResolvedEffect::SpendManaSelf {amount:Fixed64::ONE}],&mut sink).unwrap();
        assert!(sink.mana.is_none());assert!(sink.heals.is_empty());
    }

    #[test]
    fn preflight_rejects_invalid_target_and_level_before_effects() {
        let effects = [EffectOp::Damage {
            amount_key: "damage",
            kind: DamageKind::Magical,
        }];
        assert!(
            resolve_effects(ABILITY_FLAME_BLADE, &effects, &Target::None, 1)
                .unwrap_err()
                .contains("entity target")
        );
        assert!(resolve_effects(
            ABILITY_FLAME_BLADE,
            &effects,
            &Target::Entity(EntityHandle { id: 7, gen: 1 }),
            0,
        )
        .unwrap_err()
        .contains("invalid level"));
    }

    #[test]
    fn dash_effect_preflight_never_relocates_on_invalid_blocked_or_mixed_plan() {
        let id=omoba_template_ids::ability_by_name("vanguard_resolve").unwrap();
        let caster=EntityHandle {id:1,gen:1};
        let range=active_ability_const(id).unwrap().levels[0].range;
        let target=Vec2::new(range,Fixed64::ZERO);
        let plan=||resolve_effects(id,&[EffectOp::DashToPoint],&Target::Point(target),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,blocked:true,..Default::default()};
        assert!(apply_effects(id,caster,plan(),&mut sink).unwrap_err().contains("blocked"));
        assert_eq!(sink.position(caster),Some(Vec2::ZERO));
        sink.blocked=false;
        for point in [Vec2::ZERO,Vec2::new(Fixed64::from_raw(range.raw()+1),Fixed64::ZERO)] {
            assert!(apply_effects(id,caster,vec![ResolvedEffect::DashToPoint {target:point,cast_range:range}],&mut sink).is_err());
            assert_eq!(sink.position(caster),Some(Vec2::ZERO));
        }
        let mut mixed=vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}];mixed.extend(plan());
        assert!(apply_effects(id,caster,mixed,&mut sink).is_err());
        assert!(sink.heals.is_empty());assert_eq!(sink.position(caster),Some(Vec2::ZERO));
        for wrong in [Target::None,Target::Entity(caster)] {
            assert!(resolve_effects(id,&[EffectOp::DashToPoint],&wrong,1).is_err());
        }
        assert!(resolve_effects(id,&[EffectOp::DashToPoint,EffectOp::DashToPoint],&Target::Point(target),1).is_err());
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.position(caster),Some(target));
        assert!(sink.damage.is_empty() && sink.heals.is_empty());
    }

    #[test]
    fn control_effect_preflight_is_atomic_and_preserves_typed_status() {
        let id=omoba_template_ids::ability_by_name("vanguard_strike").unwrap();
        let caster=EntityHandle {id:1,gen:1};let victim=EntityHandle {id:2,gen:1};
        for control in [ControlEffectKind::Stun,ControlEffectKind::Root,ControlEffectKind::Silence] {
            let ops=[EffectOp::Damage {amount_key:"damage",kind:DamageKind::Physical},
                EffectOp::ControlEnemy {control,duration_key:"stun_duration"}];
            let plan=||resolve_effects(id,&ops,&Target::Entity(victim),1).unwrap();
            let mut sink=HeadlessWorld {alive:true,enemy:true,positions:std::collections::HashMap::from([
                (1,Vec2::ZERO),(2,Vec2::new(Fixed64::from_i32(301),Fixed64::ZERO))]),..Default::default()};
            assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
            assert!(sink.damage.is_empty() && sink.controls.is_empty());
            sink.positions.insert(2,Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO));
            sink.enemy=false;
            assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
            assert!(sink.damage.is_empty() && sink.controls.is_empty());
            sink.enemy=true;
            let mut invalid=plan();invalid.push(ResolvedEffect::ControlEnemy {control,victim,
                duration:Fixed64::ZERO,cast_range:Some(Fixed64::from_i32(300))});
            assert!(apply_effects(id,caster,invalid,&mut sink).is_err());
            assert!(sink.damage.is_empty() && sink.controls.is_empty());
            apply_effects(id,caster,plan(),&mut sink).unwrap();
            assert_eq!(sink.damage,vec![(2,65*1024)]);
            assert_eq!(sink.controls,vec![(2,control,768)]);
        }
    }

    #[test]
    fn stun_effect_preflight_is_atomic_with_damage() {
        let id=omoba_template_ids::ability_by_name("vanguard_strike").unwrap();
        let caster=EntityHandle {id:1,gen:1};let victim=EntityHandle {id:2,gen:1};
        let ops=[EffectOp::Damage {amount_key:"damage",kind:DamageKind::Physical},
            EffectOp::StunEnemy {duration_key:"stun_duration"}];
        let plan=||resolve_effects(id,&ops,&Target::Entity(victim),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,enemy:true,positions:std::collections::HashMap::from([
            (1,Vec2::ZERO),(2,Vec2::new(Fixed64::from_i32(301),Fixed64::ZERO))]),..Default::default()};
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.stuns.is_empty());
        sink.positions.insert(2,Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO));
        sink.enemy=false;
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.stuns.is_empty());
        sink.enemy=true;
        let mut invalid=plan();invalid.push(ResolvedEffect::ControlEnemy {control:ControlEffectKind::Stun,
            victim,duration:Fixed64::ZERO,cast_range:Some(Fixed64::from_i32(300))});
        assert!(apply_effects(id,caster,invalid,&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.stuns.is_empty());
        for target in [Target::None,Target::Point(Vec2::ZERO)] {
            assert!(resolve_effects(id,&ops,&target,1).is_err());
        }
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.damage,vec![(2,65*1024)]);
        assert_eq!(sink.stuns,vec![(2,768)]);
    }

    #[test]
    fn slow_effect_preflight_is_atomic_with_damage() {
        let id=omoba_template_ids::ability_by_name("ranger_shot").unwrap();
        let caster=EntityHandle {id:1,gen:1};let victim=EntityHandle {id:2,gen:1};
        let ops=[EffectOp::Damage {amount_key:"damage",kind:DamageKind::Physical},
            EffectOp::SlowEnemy {reduction_key:"slow_reduction",duration_key:"slow_duration"}];
        let plan=||resolve_effects(id,&ops,&Target::Entity(victim),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,..Default::default()};
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.slows.is_empty());
        sink.enemy=true;
        sink.positions.insert(victim.id,Vec2::new(Fixed64::from_i32(751),Fixed64::ZERO));
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.slows.is_empty());
        sink.positions.insert(victim.id,Vec2::ZERO);
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.damage,vec![(2,90*1024)]);
        assert_eq!(sink.slows,vec![(2,256,2*1024)]);
        for target in [Target::None,Target::Point(Vec2::ZERO)] {
            assert!(resolve_effects(id,&ops,&target,1).is_err());
        }
    }

    #[test]
    fn cast_preflight_unit_range_boundary_dead_caster_and_self_target_are_atomic() {
        let id=omoba_template_ids::ability_by_name("lumen_bolt").unwrap();
        let caster=EntityHandle {id:1,gen:1};let victim=EntityHandle {id:2,gen:1};
        let ops=[EffectOp::Damage {amount_key:"damage",kind:DamageKind::Magical}];
        let range=active_ability_const(id).unwrap().levels[0].range;
        let plan=||resolve_effects(id,&ops,&Target::Entity(victim),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,enemy:true,..Default::default()};
        sink.positions.insert(victim.id,Vec2::new(range,Fixed64::ZERO));
        apply_effects(id,caster,plan(),&mut sink).unwrap();assert_eq!(sink.damage.len(),1);
        sink.positions.insert(victim.id,Vec2::new(Fixed64::from_raw(range.raw()+1),Fixed64::ZERO));
        let mut effects=vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}];effects.extend(plan());
        assert!(apply_effects(id,caster,effects,&mut sink).unwrap_err().contains("outside cast range"));
        assert!(sink.heals.is_empty());assert_eq!(sink.damage.len(),1);
        sink.alive=false;
        assert!(apply_effects(id,caster,vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}],&mut sink).is_err());
        assert!(sink.heals.is_empty());
        let heal=omoba_template_ids::ability_by_name("lumen_touch").unwrap();
        let heal_ops=[EffectOp::HealSelf {amount_key:"heal"}];
        for wrong in [Target::Entity(caster),Target::Point(Vec2::ZERO)] {
            assert!(resolve_effects(heal,&heal_ops,&wrong,1).unwrap_err().contains("requires no target"));
        }
        assert!(resolve_effects(heal,&heal_ops,&Target::None,1).is_ok());
    }

    #[test]
    fn area_effects_expand_sorted_unique_live_targets_and_preflight_atomically() {
        let id=omoba_template_ids::ability_by_name("ranger_volley").unwrap();
        let caster=EntityHandle {id:1,gen:1};
        let ops=[EffectOp::AreaDamage {amount_key:"damage",radius_key:"radius",kind:DamageKind::Physical}];
        let plan=||resolve_effects(id,&ops,&Target::Point(Vec2::ZERO),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,enemy:true,
            enemies:[3,2,2,9].map(|id|EntityHandle {id,gen:1}).to_vec(),..Default::default()};
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.damage,vec![(2,Fixed64::from_i32(110).raw()),(3,Fixed64::from_i32(110).raw())]);
        assert_eq!(sink.area_cues.len(),1);
        assert_eq!(sink.area_cues[0].0,Vec2::ZERO);
        assert!(resolve_effects(id,&ops,&Target::Entity(caster),1).is_err());
        let outside=Target::Point(Vec2::new(Fixed64::from_i32(701),Fixed64::ZERO));
        assert!(apply_effects(id,caster,resolve_effects(id,&ops,&outside,1).unwrap(),&mut sink).is_err());
        assert_eq!(sink.damage.len(),2);
        assert_eq!(sink.area_cues.len(),1,"failed preflight cannot emit a cue");
        sink.enemy=false;
        let mut effects=vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}];effects.extend(plan());
        assert!(apply_effects(id,caster,effects,&mut sink).is_err());assert!(sink.heals.is_empty());
        sink.enemy=true;sink.enemies=(20..149).map(|id|EntityHandle {id,gen:1}).collect();
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());assert_eq!(sink.damage.len(),2);
        sink.enemies.clear();apply_effects(id,caster,plan(),&mut sink).unwrap();assert_eq!(sink.damage.len(),2);
        assert_eq!(sink.area_cues.len(),2,"successful empty area still has a presentation result");
    }

    #[test]
    fn authored_recovery_channel_requires_host_and_exposes_typed_execution() {
        let id=omoba_template_ids::ability_by_name("apprentice_mend").unwrap();
        let effect=EffectOp::RecoveryChannelSelf {amount_key:"pulse_heal",duration_key:"channel_duration",interval_key:"pulse_interval"};
        assert!(resolve_effects(id,&[effect],&Target::None,1).is_err(),"a direct legacy script invocation must fail closed");
        let ffi=generic_effect_ffi(id,Box::leak(vec![effect].into_boxed_slice()));
        let definition:omoba_core::ability_meta::AbilityDef=serde_json::from_str(&ffi.def_json).unwrap();
        assert_eq!(definition.cast_type,omoba_core::ability_meta::CastType::Channeled);
        assert!(definition.effects_preview.is_empty(),"execution must never be inferred from previews");
        assert!(matches!(definition.host_execution,Some(omoba_core::ability_meta::HostAbilityExecution::RecoveryChannelSelf {..})));
    }

    #[test]
    fn four_lua_generated_abilities_apply_in_headless_world() {
        let caster = EntityHandle { id: 1, gen: 1 };
        let victim = EntityHandle { id: 2, gen: 1 };
        let cases = [
            (
                "lumen_bolt",
                EffectOp::Damage {
                    amount_key: "damage",
                    kind: DamageKind::Magical,
                },
                80,
            ),
            ("lumen_touch", EffectOp::HealSelf { amount_key: "heal" }, 70),
            (
                "lumen_lance",
                EffectOp::Damage {
                    amount_key: "damage",
                    kind: DamageKind::Magical,
                },
                180,
            ),
            ("lumen_mend", EffectOp::HealSelf { amount_key: "heal" }, 140),
        ];
        let mut world = HeadlessWorld {
            alive: true,
            enemy: true,
            ..Default::default()
        };
        for (id, effect, expected) in cases {
            let ability = omoba_template_ids::ability_by_name(id).unwrap();
            let target = match effect {
                EffectOp::Damage { .. } => Target::Entity(victim),
                EffectOp::HealSelf { .. } => Target::None,
                EffectOp::CleanseAlly|EffectOp::DrainManaEnemy {..}|EffectOp::HealAlly {..}|EffectOp::ShieldAlly {..}=>unreachable!("fixture only lists damage/self heal"),
                EffectOp::LineDamage {..}|EffectOp::ConeDamage {..}|EffectOp::DamageOverTimeEnemy {..}|EffectOp::AreaDamage {..}|EffectOp::DashToPoint|EffectOp::SlowEnemy {..}|EffectOp::StunEnemy {..}|EffectOp::ControlEnemy {..}|EffectOp::ForcedMoveEnemy {..}|EffectOp::RevealArea {..}
                    |EffectOp::RestoreManaSelf {..}|EffectOp::SpendManaSelf {..}|EffectOp::ManaBuffSelf {..}|EffectOp::RecoveryChannelSelf {..}|EffectOp::Projectile {..}|EffectOp::SummonAtPoint {..}|EffectOp::ToggleAttackOrb|EffectOp::Channel {..}=>unreachable!("fixture only lists damage/heal"),
            };
            let resolved = resolve_effects(ability, &[effect], &target, 1).unwrap();
            match &resolved[0] {
                ResolvedEffect::Damage { amount, .. } | ResolvedEffect::HealSelf { amount } => {
                    assert_eq!(*amount, Fixed64::from_i32(expected));
                }
                ResolvedEffect::CleanseAlly {..}|ResolvedEffect::DrainManaEnemy {..}|ResolvedEffect::HealAlly {..}|ResolvedEffect::ShieldAlly {..}=>unreachable!("fixture only lists damage/self heal"),
                ResolvedEffect::LineDamage {..}|ResolvedEffect::ConeDamage {..}|ResolvedEffect::DamageOverTimeEnemy {..}|ResolvedEffect::AreaDamage {..}|ResolvedEffect::DashToPoint {..}|ResolvedEffect::SlowEnemy {..}|ResolvedEffect::ControlEnemy {..}|ResolvedEffect::ForcedMoveEnemy {..}|ResolvedEffect::RevealArea {..}
                    |ResolvedEffect::RestoreManaSelf {..}|ResolvedEffect::SpendManaSelf {..}|ResolvedEffect::ManaBuffSelf {..}|ResolvedEffect::Projectile {..}|ResolvedEffect::SummonAtPoint {..}|ResolvedEffect::ToggleAttackOrb=>unreachable!("fixture only lists damage/heal"),
            }
            apply_effects(ability, caster, resolved, &mut world).unwrap();
        }
        assert_eq!(world.damage.len(), 2);
        let ability = omoba_template_ids::ability_by_name("lumen_bolt").unwrap();
        world.alive = true;
        world.enemy = false;
        assert!(apply_effects(
            ability,
            caster,
            resolve_effects(ability, &[cases[0].1], &Target::Entity(victim), 1).unwrap(),
            &mut world
        )
        .is_err());
        assert_eq!(world.damage.len(), 2);
        assert_eq!(world.heals.len(), 2);
        world.alive = false;
        let resolved = resolve_effects(ability, &[cases[0].1], &Target::Entity(victim), 1).unwrap();
        assert!(apply_effects(ability, caster, resolved, &mut world).is_err());
        assert_eq!(world.damage.len(), 2);
    }
}
