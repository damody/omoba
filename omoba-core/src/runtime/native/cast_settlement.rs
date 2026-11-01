//! Exclusive serial settlement. Managed mana snapshots form a dependency chain;
//! a rejected predecessor cannot leak its debit into a later absolute write.
use super::*;
use crate::runtime::comp::CastSettlementCue;
use crate::runtime::ability_runtime::ManaPool;
#[cfg(test)]
use omoba_sim::Vec2 as SimVec2;
use std::collections::BTreeSet;

pub(super) fn admit(world:&World,caster:Entity,skill:&str,rank:u32,faction:Option<&Faction>,
    player:Option<u32>,expected:Option<&ManaPool>,effects:&[Outcome],cue_count:usize)->bool {
    admission(world,caster,skill,rank,faction,player,expected,effects,cue_count).is_ok()
}


/// [`admit`] with the reason of a refusal (the cast trace's `SettlementRefused`): the check that
/// failed, `effect:<module>` for an effect its module refused, or the outcome kind (`AddBuff`…) of
/// an effect this transaction may not carry.
#[allow(clippy::too_many_arguments)]
pub(super) fn admission(world:&World,caster:Entity,skill:&str,rank:u32,faction:Option<&Faction>,
    player:Option<u32>,expected:Option<&ManaPool>,effects:&[Outcome],cue_count:usize)->Result<(),&'static str> {
    let summons=effects.iter().any(|e|matches!(e,Outcome::SpawnSummoned {..}));
    let channels=effects.iter().filter(|effect|matches!(effect,Outcome::ScriptRecoveryChannel {..}|Outcome::ScriptAbilityChannel {..})).count();
    if channels>1 || (channels>0&&summons){return Err("channel_with_summons");}
    let ability_channel=effects.iter().any(|effect|matches!(effect,Outcome::ScriptAbilityChannel {..}));
    // A channel that cannot start now says why (E525: the target died, turned or left range since staging).
    for effect in effects {
        if let Outcome::ScriptAbilityChannel {caster:source,owner,spec,anchor,..}=effect {
            if *source==caster && Some(*owner)==player {
                if let Some(reason)=super::super::ability_channel::start_refusal(world,caster,*owner,*spec,*anchor) {return Err(reason);}
            }
        }
    }
    // Trace tags are observation only: the summon cap counts the effects alike with stats on or off.
    let counted=effects.iter().filter(|effect|!matches!(effect,Outcome::ScriptDamageTag {..})).count();
    if skill.is_empty() || (summons&&counted>512) || cue_count>16 || rank>u8::MAX as u32
        || !world.entities().is_alive(caster)
        || world.read_storage::<CProperty>().get(caster).is_none_or(|p|p.hp<=Fixed64::ZERO)
        || world.read_resource::<GamePause>().is_paused
        || world.try_fetch::<super::super::MobaMatch>().is_some_and(|state|state.phase!=super::super::MobaMatchPhase::Playing)
        || world.read_resource::<BuffStore>().is_stunned(caster)
        || world.read_resource::<BuffStore>().is_silenced(caster) {return Err("caster_state");}
    let factions=world.read_storage::<Faction>();
    let same_faction=match (factions.get(caster),faction) {
        (Some(a),Some(b))=>a.team_id==b.team_id&&a.faction_id==b.faction_id,
        (None,None)=>true,_=>false,
    };
    if !same_faction || world.read_storage::<PlayerOwner>().get(caster).map(|p|p.player_id)!=player {return Err("owner");}
    let heroes=world.read_storage::<Hero>();let hero=heroes.get(caster);
    // A non-hero casts its unit abilities at their rank (unit_abilities, E529): A28's phoenix (E637).
    let held=match hero {
        Some(h)=>h.get_ability_level(skill),
        None=>super::super::unit_abilities::level(&world.read_resource::<BuffStore>(),caster,skill).map_or(0,i32::from),
    };
    if hero.and_then(|h|h.mana_pool.as_ref())!=expected
        || hero.is_some_and(|h|!h.cast_ready(skill))
        || (rank>0 && held!=rank as i32) {return Err("mana_cooldown_or_rank");}
    // A charge ability spends exactly one stored use per admitted cast.
    let consumes=effects.iter().filter(|effect|matches!(effect,Outcome::ScriptConsumeCharge {..})).count();
    let charged=hero.is_some_and(|h|h.ability_charges.contains_key(skill));
    if consumes!=usize::from(charged) {return Err("charge");}
    let mut independent=Vec::new();
    let mut projectiles=0usize;
    for effect in effects {if let Outcome::AddBuff{target,buff_id,duration,payload}=effect {
        if payload.get(super::super::ability_projectile::KEY).is_some() || buff_id==super::super::ability_projectile::BUFF {
            // Same admission as the serial launch, plus the caster's remaining live budget.
            projectiles+=1;
            if super::super::ability_projectile::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none()
                || world.try_fetch::<super::super::ability_projectile::AbilityProjectiles>().map_or(0,|rows|rows.0.values().filter(|row|row.owner==caster).count())
                    +projectiles>super::super::ability_projectile::MAX_LIVE_PER_SOURCE {return Err("effect:ability_projectile");}
        }
        if (payload.get(super::super::ally_cleanse::KEY).is_some() || buff_id==super::super::ally_cleanse::BUFF)
            && super::super::ally_cleanse::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:ally_cleanse");}
        if (payload.get(super::super::aura::KEY).is_some() || buff_id.starts_with(super::super::aura::PREFIX))
            && !super::super::aura::admitted(world,*target,buff_id,*duration,payload){return Err("effect:aura");}
        if (payload.get(super::super::aura_emitter::KEY).is_some() || buff_id.starts_with(super::super::aura_emitter::PREFIX))
            && super::super::aura_emitter::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:aura_emitter");}
        if (payload.get(super::super::ally_area_buff::KEY).is_some() || buff_id==super::super::ally_area_buff::BUFF)
            && super::super::ally_area_buff::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:ally_area_buff");}
        if (payload.get(super::super::ally_area_restore::KEY).is_some() || buff_id==super::super::ally_area_restore::BUFF)
            && super::super::ally_area_restore::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:ally_area_restore");}
        if (payload.get(super::super::mana_drain::KEY).is_some() || buff_id==super::super::mana_drain::BUFF)
            && super::super::mana_drain::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:mana_drain");}
        if (payload.get(super::super::forced_motion::KEY).is_some() || buff_id==super::super::forced_motion::BUFF)
            && super::super::forced_motion::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:forced_motion");}
        if (payload.get(super::super::attributes::BASE_STAT_KEY).is_some() || buff_id==super::super::attributes::BASE_STAT_BUFF)
            && super::super::attributes::base_stat_spec(buff_id,payload).is_none(){return Err("effect:modify_base_stat");}
        if (payload.get(super::super::impulse::KEY).is_some() || buff_id==super::super::impulse::BUFF)
            && super::super::impulse::admitted(world,*target,buff_id,payload).is_none(){return Err("effect:impulse");}
        if (payload.get(super::super::blink_landing::KEY).is_some() || buff_id==super::super::blink_landing::BUFF)
            && super::super::blink_landing::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:blink_landing");}
        if (payload.get(super::super::delayed_effect::KEY).is_some() || buff_id.starts_with(super::super::delayed_effect::PREFIX))
            && !super::super::delayed_effect::admitted(*target,buff_id,*duration,payload,caster){return Err("effect:delayed_effect");}
        if (payload.get(super::super::scheduled_effect::KEY).is_some() || buff_id.starts_with(super::super::scheduled_effect::PREFIX))
            && !super::super::scheduled_effect::admitted(&world.read_resource::<BuffStore>(),*target,buff_id,*duration,payload){return Err("effect:scheduled_effect");}
        if (payload.get(super::super::unit_var::KEY).is_some() || buff_id.starts_with(super::super::unit_var::PREFIX))
            && !super::super::unit_var::admitted(&world.read_resource::<BuffStore>(),*target,buff_id,*duration,payload){return Err("effect:unit_var");}
        if (payload.get(super::super::rules::attack::MODIFY_KEY).is_some() || buff_id==super::super::rules::attack::MODIFY_BUFF)
            && super::super::rules::attack::modify_spec(buff_id,*duration,payload).is_none(){return Err("effect:rules::attack");}
        if (payload.get(super::super::absorb::KEY).is_some() || buff_id==super::super::absorb::BUFF)
            && !super::super::absorb::admitted(world,*target,buff_id,*duration,payload){return Err("effect:absorb");}
        if (payload.get(super::super::dispel::REMOVE_INSTANCE_KEY).is_some() || buff_id==super::super::dispel::REMOVE_INSTANCE_BUFF)
            && super::super::dispel::remove_instance_spec(buff_id,*duration,payload).is_none(){return Err("effect:dispel::remove_instance");}
        if (payload.get(super::super::dispel::PURGE_KEY).is_some() || buff_id==super::super::dispel::PURGE_BUFF)
            && super::super::dispel::PurgeSpec::from_payload(buff_id,*duration,payload).is_none(){return Err("effect:dispel::purge");}
        if (payload.get(super::super::mana_burn::KEY).is_some() || buff_id==super::super::mana_burn::BUFF)
            && super::super::mana_burn::admitted(world,*target,buff_id,*duration,payload).is_none(){return Err("effect:mana_burn");}
        if (buff_id==super::super::ability_slots::ACTIVE_BUFF || buff_id==super::super::ability_slots::SWAP_BUFF
            || buff_id==super::super::ability_slots::SLOT_BUFF || payload.get(super::super::ability_slots::SLOT_KEY).is_some()
            || payload.get(super::super::ability_slots::ACTIVE_KEY).is_some() || payload.get(super::super::ability_slots::SWAP_KEY).is_some())
            && !super::super::ability_slots::admitted(world,*target,buff_id,*duration,payload){return Err("effect:ability_slots");}
        if (payload.get(super::super::mana_burn::SET_KEY).is_some() || buff_id==super::super::mana_burn::SET_BUFF)
            && super::super::mana_burn::set_admitted(world,*target,buff_id,*duration,payload).is_none(){return Err("effect:mana_burn::set");}
        if (payload.get(super::super::script_gold::KEY).is_some() || buff_id==super::super::script_gold::BUFF)
            && super::super::script_gold::admitted(world,*target,buff_id,*duration,payload).is_none(){return Err("effect:script_gold");}
        if (payload.get(super::super::reveal_zone::KEY).is_some() || buff_id==super::super::reveal_zone::BUFF)
            && super::super::reveal_zone::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:reveal_zone");}
        if (payload.get(super::super::ability_projectile::DODGE_KEY).is_some() || buff_id==super::super::ability_projectile::DODGE_BUFF)
            && super::super::ability_projectile::dodge_admitted(world,*target,buff_id,payload,Some(caster)).is_none(){return Err("effect:ability_projectile::dodge");}
        if (payload.get(super::super::attack_orb::KEY).is_some() || buff_id.starts_with("attack_orb:"))
            && !super::super::attack_orb::admitted(world,*target,buff_id,payload,Some(caster)){return Err("effect:attack_orb");}
        if payload.get(super::super::ally_shield::KEY).is_some()
            && super::super::ally_shield::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return Err("effect:ally_shield");}
        if payload.get(super::super::ability_runtime::damage_zone::KEY).is_some() {
            if !super::zone_pulse_settlement::admit_zone(world,*target,buff_id,*duration,payload){return Err("effect:damage_zone");}
            let spec=super::super::ability_runtime::damage_zone::DamageZoneSpec::from_payload(payload).expect("checked zone");
            if spec.independent {if *target!=caster{return Err("effect:independent_zone_target");}independent.push(buff_id.as_str());}
        }
    }}
    if !world.read_resource::<BuffStore>().can_schedule_independent_zones(caster,&independent){return Err("independent_zones");}
    // Summon transactions plan every sibling's placement against the current world.
    // A summon's own setup (a buff on a summon this cast spawns: its assigned target,
    // markers) and buffs or damage on other units after the last planned spawn settle
    // after that plan, so they cannot move or kill anything the plan relied on.
    let spawned:BTreeSet<Entity>=effects.iter().filter_map(|effect|match effect {
        Outcome::SpawnSummoned {entity,..}=>Some(*entity),_=>None}).collect();
    // The plan reads terrain and the caster's team's occupancy only (summon_navigation::placement)
    // and every spawn carries its own position: a unit of another team is outside it (E416).
    let caster_team=factions.get(caster).map(|faction|faction.team_id);
    let outside_plan=|target:&Entity|*target!=caster && !spawned.contains(target)
        && factions.get(*target).zip(caster_team).is_some_and(|(faction,team)|faction.team_id!=team);
    let last_spawn=effects.iter().rposition(|effect|matches!(effect,Outcome::SpawnSummoned {..}));
    // Worlds without summons (tests, TD) hold none of the caster's.
    let own_summon=|target:&Entity|world.has_value::<specs::storage::MaskedStorage<SummonedUnit>>()
        && world.read_storage::<SummonedUnit>().get(*target).is_some_and(|summon|summon.summoner==caster);
    let refused=effects.iter().enumerate().find(|(index,effect)|!match effect {
        Outcome::ScriptCastTransaction {..}|Outcome::ScriptCastCommitted {..}|Outcome::DamageZonePulse {..}|Outcome::IndependentDamageZonePulse {..}|Outcome::DamageWithLeech {..}|Outcome::DamageWithCritical {..}=>false,
        Outcome::ScriptManaCommit {changes,notifications}=>mana_admitted(world,changes,notifications)
            && ((!summons&&channels==0) || changes.iter().all(|row|row.entity==caster)),
        Outcome::ScriptRecoveryChannel {caster:source,owner,amount,duration,interval}=>*source==caster&&Some(*owner)==player
            && super::super::recovery_channel::can_start(world,caster,*owner,super::super::recovery_channel::Spec {amount:*amount,duration:*duration,interval:*interval}),
        Outcome::ScriptAbilityChannel {caster:source,owner,spec,anchor,..}=>*source==caster&&Some(*owner)==player
            && super::super::ability_channel::can_start(world,caster,*owner,*spec,*anchor),
        Outcome::SpawnSummoned {owner,..}=>*owner==caster,
        Outcome::AddBuff {target,..} if spawned.contains(target)=>true,
        // A replace-stacking status on a summon this cast spawns removes before it applies (E444,
        // 原作 B33T 蜘蛛); a fresh unit holds nothing the plan relied on.
        Outcome::ScriptRemoveBuff {target,..} if spawned.contains(target)=>true,
        // v3 create_unit.facing: a summon this cast spawns faces where the script says.
        Outcome::ScriptSetFacing {entity,..} if spawned.contains(entity)=>true,
        // ... and stands where the script puts it (E590, 原作 B24W2 SetAbsOrigin puts the stele back
        // on the cursor): every planned spawn commits first, and a fresh unit holds nothing else.
        Outcome::ScriptSetPos {entity,..} if spawned.contains(entity)=>true,
        // The caster's own statuses, orders and schedules (each host row checked above) and its
        // relocation change neither its life nor its resources, and every planned spawn commits
        // before any of them: they may come before, between or after the summons (原作 C02E 幻影步
        // applies statuses, stops and teleports its caster around create_illusion; E373, E395).
        Outcome::AddBuff {target,..} if summons && channels==0 && *target==caster=>true,
        Outcome::ScriptSetPos {entity,..} if summons && channels==0 && *entity==caster=>true,
        // The caster's own summon leaving (v3 remove_unit: 原作 B16 replaces its moon before raising
        // the next): the plan only gains room, and nothing else of the caster's moves (E612).
        Outcome::AddBuff {target,buff_id,..} if summons && channels==0 && *target!=caster
            && buff_id==crate::runtime::native::script_kill::BUFF && own_summon(target)=>true,
        // Never the caster: its life and resources back the whole transaction.
        Outcome::AddBuff {target,..}|Outcome::RawDamage {target,..}|Outcome::Damage {target,..}|Outcome::ScriptDirectDamage {target,..}
            if summons && channels==0 && *target!=caster && last_spawn.is_some_and(|last|*index>last)=>true,
        // A unit of another team, anywhere among the spawns (原作 B11 鴉襲 phases its target before
        // its illusion may appear, E416).
        Outcome::AddBuff {target,..}|Outcome::RawDamage {target,..}|Outcome::Damage {target,..}|Outcome::ScriptDirectDamage {target,..}
            if summons && channels==0 && outside_plan(target)=>true,
        // A plain status on another unit (its catalog row, host state buffs and control-source
        // row; each checked above) moves nobody, and the plan reads positions only: anywhere among
        // the spawns (原作 B24T phases the allies in the ring before the rocks rise, E439).
        Outcome::AddBuff {target,buff_id,..} if summons && channels==0 && *target!=caster && plain_status(buff_id)=>true,
        // Conformance trace metadata for the next damage packet; it changes nothing (damage_trace).
        Outcome::ScriptDamageTag {..}=>true,
        // An area cue (ExplosionFxQueue) is presentation only: it moves and kills nothing (E621).
        Outcome::Explosion {..}=>true,
        // G-2: exactly the cast's own ability, at most once (count checked above).
        Outcome::ScriptConsumeCharge {entity,ability_id}=>*entity==caster&&ability_id==skill,
        _ if !summons&&channels==0=>true,
        // A v3 channel ability (E509, 原作 A11E 邪眼): its cast puts statuses, damage and motion on
        // any unit like any cast; only its caster's mana and cooldown keep the caster-only rules
        // below. A recovery channel (an item's) carries its own resource effects only.
        _ if ability_channel && !matches!(effect,Outcome::ScriptSetMana {..}|Outcome::ScriptManaGained {..}
            |Outcome::ScriptUnitManaDelta {..}|Outcome::ScriptManaSpent {..}|Outcome::ScriptStartCooldown {..})=>true,
        // A formation may update its caster's resources/facing, but cannot
        // insert arbitrary terrain/life mutations between planned siblings.
        Outcome::ScriptSetMana {entity,..}|Outcome::ScriptManaGained {entity,..}|Outcome::ScriptUnitManaDelta {entity,..}
            |Outcome::ScriptSetFacing {entity,..}=>*entity==caster,
        Outcome::ScriptManaSpent {entity,amount,ability_id}=>*entity==caster&&*amount>=Fixed64::ZERO&&ability_id==skill,
        Outcome::ScriptStartCooldown {entity,ability_id,duration}=>*entity==caster&&ability_id==skill
            && *duration>=Fixed64::ZERO&&*duration<=Fixed64::from_i32(1_000_000),
        // The cast breaking the caster's own invisibility (rules::status::invisibility_broken_by_cast, E226).
        Outcome::ScriptRemoveBuff {target,..}=>*target==caster,
        _=>false,
    });
    match refused {
        Some((_,effect))=>Err(super::outcome_kind(effect)),
        None=>Ok(()),
    }
}

#[cfg(test)]
#[test]
fn summon_casts_admit_their_summons_setup_and_trailing_effects_but_nothing_between_siblings() {
    let (mut world,hero)=crate::runtime::native::moba_match::stop_command_world();
    world.register::<Faction>();
    world.insert(BuffStore::default());
    world.insert(crate::runtime::native::moba_match::stop_command_match(hero,crate::runtime::MobaMatchPhase::Playing));
    let faction=world.read_storage::<Faction>().get(hero).cloned();
    let expected=world.read_storage::<Hero>().get(hero).and_then(|hero|hero.mana_pool.clone());
    let victim=world.create_entity().build();
    let team=faction.as_ref().map_or(1,|faction|faction.team_id);
    let enemy=world.create_entity().with(Faction::new(FactionType::Enemy,team+1)).build();
    let ally=world.create_entity().with(Faction::new(FactionType::Player,team)).build();
    let (first,second)=(world.create_entity().build(),world.create_entity().build());
    let spawn=|entity|Outcome::SpawnSummoned {entity,pos:SimVec2::ZERO,unit_id:"fixture".into(),owner:hero,
        faction:faction.clone().unwrap_or_else(||Faction::new(FactionType::Player,1)),player_owner:None,duration:Fixed64::ONE};
    let assign=|entity|Outcome::AddBuff {target:entity,buff_id:"summon_target".into(),duration:Fixed64::ONE,payload:serde_json::json!({})};
    let stun=Outcome::AddBuff {target:victim,buff_id:"stun".into(),duration:Fixed64::ONE,payload:serde_json::Value::Null};
    let player=world.read_storage::<PlayerOwner>().get(hero).map(|owner|owner.player_id);
    let accepts=|effects:&[Outcome]|admit(&world,hero,"fixture",0,faction.as_ref(),player,expected.as_ref(),effects,0);
    assert!(accepts(&[spawn(first),assign(first),spawn(second),assign(second),stun.clone()]),"own setup and a trailing stun");
    // E439: a plain status (here a host control) moves nobody: it may come between siblings.
    assert!(accepts(&[spawn(first),stun.clone(),spawn(second)]),"a status between planned siblings");
    {
        // E416: a unit of another team is outside the placement plan, so it may be hit anywhere.
        let on=|target|Outcome::AddBuff {target,buff_id:"stun".into(),duration:Fixed64::ONE,payload:serde_json::Value::Null};
        if faction.is_some() {
            assert!(accepts(&[on(enemy),spawn(first),on(enemy),spawn(second)]),"an enemy before and between the spawns");
            assert!(accepts(&[Outcome::ScriptDirectDamage {target:enemy,amount:Fixed64::ONE},spawn(first)]));
        }
        assert!(accepts(&[on(ally),spawn(first),spawn(second)]),"a status leaves an ally where it stands (E439)");
        let feign=|buff_id:&str|Outcome::AddBuff {target:ally,buff_id:buff_id.into(),duration:Fixed64::ONE,payload:serde_json::json!({})};
        assert!(accepts(&[feign("a05d_feign"),feign("__ctl:a05d_feign"),feign("no_unit_collision"),spawn(first)]),"a catalog status, its source row and states");
        let order=super::super::hero_command::Command::Order {kind:super::super::hero_command::OrderKind::Stop,at:(0,0)};
        let command=Outcome::AddBuff {target:ally,buff_id:super::super::hero_command::BUFF.into(),duration:Fixed64::ONE,payload:order.payload()};
        assert!(!accepts(&[command,spawn(first)]),"a host command on an ally is not a status");
    }
    assert!(!accepts(&[spawn(first),assign(victim),spawn(second)]),"a buff on another unit between siblings");
    let tag=Outcome::ScriptDamageTag {ability:"fixture".into(),path:"events.on_cast[1]".into()};
    assert!(accepts(&[spawn(first),tag.clone(),spawn(second),tag]),"trace tags change nothing (E439)");
    let remove=|target|Outcome::ScriptRemoveBuff {target,buff_id:"fixture_status".into()};
    assert!(accepts(&[spawn(first),remove(first),assign(first),spawn(second)]),"a replace on this cast's summon (E444)");
    assert!(!accepts(&[spawn(first),remove(victim),spawn(second)]),"never another unit's statuses");
    let face=|entity|Outcome::ScriptSetFacing {entity,facing:omoba_sim::Angle::ZERO};
    assert!(accepts(&[spawn(first),face(first),spawn(second)]),"a summon of this cast faces its way");
    assert!(!accepts(&[spawn(first),face(victim)]),"never another unit's facing");
    let place=|entity|Outcome::ScriptSetPos {entity,pos:SimVec2::ZERO};
    assert!(accepts(&[spawn(first),place(first),spawn(second)]),"a summon of this cast stands where the script puts it (E590)");
    assert!(!accepts(&[spawn(first),Outcome::ScriptDirectDamage {target:hero,amount:Fixed64::ONE}]),"never the caster, even trailing");
    // The caster's own schedule (b13_storm: create_unit, then repeat) goes with the summons, and
    // the refusal names what it refused.
    let spec=super::super::scheduled_effect::Spec {ability:"fixture".into(),level:1,step:1,instance:7,args:Vec::new()}.checked().unwrap();
    let repeat=Outcome::AddBuff {target:hero,buff_id:spec.buff_id(),duration:Fixed64::ONE,payload:spec.payload()};
    assert!(accepts(&[spawn(first),assign(first),repeat.clone()]),"a trailing repeat on the caster");
    assert!(accepts(&[repeat.clone(),spawn(first)]),"also before the summons: every spawn commits first");
    // E395: the caster's own statuses and relocation, before or after the summons (C02E 幻影步).
    let haste=Outcome::AddBuff {target:hero,buff_id:"haste".into(),duration:Fixed64::ONE,payload:serde_json::json!({"move_speed_bonus":1024})};
    let blink=Outcome::ScriptSetPos {entity:hero,pos:SimVec2::ZERO};
    assert!(accepts(&[haste.clone(),blink.clone(),spawn(first),assign(first),haste.clone(),spawn(second)]),"the caster's own statuses and move");
    assert!(!accepts(&[spawn(first),Outcome::ScriptSetPos {entity:victim,pos:SimVec2::ZERO}]),"never another unit's position");
    assert_eq!(admission(&world,hero,"fixture",0,faction.as_ref(),player,expected.as_ref(),
        &[spawn(first),Outcome::ScriptDirectDamage {target:hero,amount:Fixed64::ONE}],0),Err("ScriptDirectDamage"));
    {
        // E612: the caster's own summon removed before its replacement rises (B16 b16_summon_hound).
        world.register::<SummonedUnit>();
        let summon=|summoner|SummonedUnit {summoner,duration:None,time_remaining:None,summon_time:Fixed64::ZERO,
            navigation:Default::default(),immobile:false,no_attack:false,ward:false,expire_with_owner:false,
            spawn_tick:0,bounty:(0,0),killer_player:None};
        let old=world.create_entity().with(summon(hero)).build();
        let others=world.create_entity().with(summon(victim)).build();
        let remove=|target|Outcome::AddBuff {target,buff_id:crate::runtime::native::script_kill::BUFF.into(),duration:Fixed64::ONE,
            payload:crate::runtime::native::script_kill::payload(hero.id(),hero.gen().id() as u32,true)};
        let accepts=|effects:&[Outcome]|admit(&world,hero,"fixture",0,faction.as_ref(),player,expected.as_ref(),effects,0);
        assert!(accepts(&[remove(old),spawn(first),assign(first)]),"the caster's own summon leaves first");
        assert!(!accepts(&[remove(others),spawn(first)]),"never another owner's summon");
    }
}

/// E637: a non-hero caster settles at its unit ability's rank (A28's phoenix 天火), and only there.
#[cfg(test)]
#[test]
fn a_non_hero_cast_settles_at_its_unit_ability_rank() {
    let (mut world,hero)=crate::runtime::native::moba_match::stop_command_world();
    world.register::<Faction>();
    world.insert(BuffStore::default());
    world.insert(crate::runtime::native::moba_match::stop_command_match(hero,crate::runtime::MobaMatchPhase::Playing));
    let faction=world.read_storage::<Faction>().get(hero).cloned();
    let props=world.read_storage::<CProperty>().get(hero).cloned().unwrap();
    let mut builder=world.create_entity().with(props).with(PlayerOwner::new(7));
    if let Some(faction)=faction.clone() {builder=builder.with(faction);}
    let phoenix=builder.build();
    let ability="a28_heaven_fire";
    let row=super::super::unit_abilities::payload(ability,2).expect("catalog unit ability");
    world.write_resource::<BuffStore>().add(phoenix,&super::super::unit_abilities::row_id(ability),Fixed64::from_i32(30),row);
    let settles=|rank|admission(&world,phoenix,ability,rank,faction.as_ref(),Some(7),None,&[],0);
    assert_eq!(settles(2),Ok(()));
    assert_eq!(settles(1),Err("mana_cooldown_or_rank"),"another rank");
    assert_eq!(admission(&world,phoenix,"a28_spreading_fire",1,faction.as_ref(),Some(7),None,&[],0),Err("mana_cooldown_or_rank"),"an ability it does not hold");
}

#[cfg(test)]
#[test]
fn recovery_channel_transaction_admits_only_its_exact_live_owner_and_resource_effects() {
    let (mut world,hero)=crate::runtime::native::moba_match::stop_command_world();
    world.register::<Faction>();
    world.insert(BuffStore::default());
    world.insert(crate::runtime::native::moba_match::stop_command_match(hero,crate::runtime::MobaMatchPhase::Playing));
    let faction=world.read_storage::<Faction>().get(hero).cloned();
    let expected=world.read_storage::<Hero>().get(hero).and_then(|hero|hero.mana_pool.clone());
    let channel=Outcome::ScriptRecoveryChannel {caster:hero,owner:7,amount:Fixed64::from_i32(35),duration:Fixed64::from_i32(2),interval:Fixed64::from_raw(512)};
    let accepts=|effects:&[Outcome]|admit(&world,hero,"fixture_channel",0,faction.as_ref(),Some(7),expected.as_ref(),effects,0);
    assert!(accepts(&[channel.clone()]));
    assert!(!accepts(&[channel.clone(),channel.clone()]));
    assert!(!accepts(&[Outcome::ScriptHeal {target:hero,amount:Fixed64::ONE},channel.clone()]));
    assert!(!accepts(&[Outcome::ScriptRecoveryChannel {caster:hero,owner:8,amount:Fixed64::ONE,duration:Fixed64::ONE,interval:Fixed64::ONE}]));
    world.write_resource::<GamePause>().is_paused=true;assert!(!accepts(&[channel]));
}

/// E509: a channel ability's cast carries statuses on its target and its caster; another
/// unit's mana or another ability's cooldown stays out.
#[cfg(test)]
#[test]
fn ability_channel_casts_carry_statuses_on_any_unit_but_only_their_own_resources() {
    use super::super::ability_channel::{Anchor, Spec};
    use omoba_content_model::ChannelAnchor;
    let (mut world,hero)=crate::runtime::native::moba_match::stop_command_world();
    world.register::<Faction>();
    world.insert(BuffStore::default());
    world.insert(crate::runtime::native::moba_match::stop_command_match(hero,crate::runtime::MobaMatchPhase::Playing));
    let faction=world.read_storage::<Faction>().get(hero).cloned();
    let expected=world.read_storage::<Hero>().get(hero).and_then(|hero|hero.mana_pool.clone());
    let enemy=world.create_entity().with(Faction::new(FactionType::Enemy,faction.as_ref().map_or(1,|f|f.team_id)+1)).build();
    let spec=Spec {anchor:ChannelAnchor::SelfOrigin,interval:Fixed64::from_raw(512),pulses:4,radius:Some(Fixed64::from_i32(300)),range:Fixed64::ZERO};
    let channel=Outcome::ScriptAbilityChannel {caster:hero,owner:7,skill_id:"fixture_channel".into(),level:1,spec,anchor:Anchor::Origin};
    let accepts=|effects:&[Outcome]|admit(&world,hero,"fixture_channel",0,faction.as_ref(),Some(7),expected.as_ref(),effects,0);
    let status=|target:Entity|Outcome::AddBuff {target,buff_id:"stun".into(),duration:Fixed64::ONE,payload:serde_json::Value::Null};
    assert!(accepts(&[channel.clone()]));
    assert!(accepts(&[status(enemy),channel.clone(),status(hero)]), "statuses on the target and the caster");
    assert!(!accepts(&[channel.clone(),Outcome::ScriptManaGained {entity:enemy,amount:Fixed64::ONE}]), "another unit's mana");
    assert!(!accepts(&[channel,Outcome::ScriptStartCooldown {entity:hero,ability_id:"other".into(),duration:Fixed64::ONE}]), "another ability's cooldown");
}

/// A buff row an `apply_status` of a catalog status writes: the status row itself, a host state
/// buff its states hold, or its control-source row (`__ctl:<status>`).
fn plain_status(buff_id:&str)->bool {
    use super::super::dispel::{registry,CONTROL_SOURCE_PREFIX};
    let registry=registry();
    let known=|key:&str|!registry.statuses_of(key).is_empty()
        || registry.status(omoba_template_ids::instance_base(key)).is_some();
    known(buff_id) || super::super::rules::vocab::is_state_buff(buff_id)
        || buff_id.strip_prefix(CONTROL_SOURCE_PREFIX).is_some_and(known)
}

fn mana_admitted(world:&World,changes:&[crate::comp::ManaSettlementChange],notifications:&[crate::comp::ManaSettlementNotification])->bool {
    let entities=world.entities();let heroes=world.read_storage::<Hero>();let props=world.read_storage::<CProperty>();
    let mut seen=BTreeSet::new();
    !changes.is_empty() && changes.iter().all(|row|seen.insert(row.entity)&&entities.is_alive(row.entity)
        && props.get(row.entity).is_some_and(|p|p.hp>Fixed64::ZERO)
        && heroes.get(row.entity).and_then(|hero|hero.mana_pool.as_ref())==Some(&row.expected))
        && notifications.iter().all(|notification|match notification {
            crate::comp::ManaSettlementNotification::Gained {entity,amount}
                |crate::comp::ManaSettlementNotification::Spent {entity,amount,..}=>seen.contains(entity)&&*amount>Fixed64::ZERO,
        })
}

/// Batch pre-admission: settle an admitted cast's caster-only mana commit now
/// (removed from its effects). Mixed or non-caster commits stay in effects.
pub(super) fn apply_caster_mana(world:&World,caster:Entity,outcomes:&mut Vec<Outcome>) {
    let mut kept=Vec::with_capacity(outcomes.len());
    for outcome in outcomes.drain(..) {
        match outcome {
            Outcome::ScriptManaCommit {changes,notifications} if !changes.is_empty() && changes.iter().all(|row|row.entity==caster)
                => commit_mana(world,changes,notifications),
            other=>kept.push(other),
        }
    }
    *outcomes=kept;
}

pub(super) fn commit_mana(world:&World,changes:Vec<crate::comp::ManaSettlementChange>,notifications:Vec<crate::comp::ManaSettlementNotification>) {
    if !mana_admitted(world,&changes,&notifications){return;}
    {let mut heroes=world.write_storage::<Hero>();for change in changes {heroes.get_mut(change.entity).expect("checked mana owner").mana_pool=Some(change.pool);}}
    let mut queue=world.write_resource::<ScriptEventQueue>();
    for notification in notifications {queue.push(match notification {
        crate::comp::ManaSettlementNotification::Gained {entity,amount}=>ScriptEvent::ManaGained {e:entity,amount},
        crate::comp::ManaSettlementNotification::Spent {entity,amount,ability_id}=>ScriptEvent::SpentMana {caster:entity,cost:amount,ability_id},
    });}
}

pub(super) struct PlannedSummon {
    outcome:Outcome,
    stats:&'static omoba_template_ids::SummonStats,
}

pub(super) fn plan_summons(world:&World,effects:&[Outcome])->Option<Vec<PlannedSummon>> {
    // Ordinary managed casts do not need an entity-wide occupancy scan.
    if !effects.iter().any(|effect|matches!(effect,Outcome::SpawnSummoned {..})) {return Some(Vec::new());}
    let mut planned=Vec::new();let mut handles=BTreeSet::new();
    let entities=world.entities();let positions=world.read_storage::<Pos>();let radii=world.read_storage::<CollisionRadius>();
    let factions=world.read_storage::<Faction>();let props=world.read_storage::<CProperty>();
    let units=world.read_storage::<Unit>();let owners=world.read_storage::<PlayerOwner>();
    let mut occupied:Vec<_>=(&entities,&positions,&radii,&factions).join().map(|(_,p,r,f)|(f.team_id,p.0,r.0)).collect();
    let regions=world.read_resource::<BlockedRegions>();
    for effect in effects {
        let Outcome::SpawnSummoned {entity,pos,unit_id,owner,faction,player_owner,duration}=effect else {continue;};
        if planned.len()>=128 || !handles.insert(*entity) || !entities.is_alive(*entity)
            || units.get(*entity).is_some()||positions.get(*entity).is_some()||props.get(*entity).is_some()
            || !entities.is_alive(*owner)||props.get(*owner).is_none_or(|p|p.hp<=Fixed64::ZERO)
            || factions.get(*owner).is_none_or(|f|f.team_id!=faction.team_id||f.faction_id!=faction.faction_id)
            || owners.get(*owner).map(|p|p.player_id)!=*player_owner
            || *duration<Fixed64::ZERO || *duration>Fixed64::from_i32(3600) {return None;}
        let stats=omoba_template_ids::summon_by_name(unit_id).and_then(omoba_template_ids::active_summon_stats)?;
        if !super::super::dummy_unit::body_valid(stats)||stats.move_speed<Fixed64::ZERO||stats.collision_radius<Fixed64::ZERO
            ||stats.vision_radius<Fixed64::ZERO||stats.vision_radius>Fixed64::from_i32(10_000){return None;}
        // A pathing blocker ignores unit occupancy, as its spawn does (game_processor, E599).
        let friendly:Vec<_>=if stats.blocks_pathing {Vec::new()} else {
            occupied.iter().filter(|(team,_,_)|*team==faction.team_id).map(|(_,p,r)|(*p,*r)).collect()};
        let pos=super::super::summon_navigation::placement(*pos,positions.get(*owner).map(|p|p.0),stats.collision_radius,&regions,&friendly)?;
        occupied.push((faction.team_id,pos,stats.collision_radius));
        planned.push(PlannedSummon {outcome:Outcome::SpawnSummoned {entity:*entity,pos,unit_id:unit_id.clone(),owner:*owner,
            faction:faction.clone(),player_owner:*player_owner,duration:*duration},stats});
    }
    Some(planned)
}

impl PlannedSummon {
    pub(super) fn commit(self,world:&mut World,tick:u64,ordinal:&mut usize)->Result<(),failure::Error> {
        let fact=observable_fact_from_outcome(&self.outcome,tick,*ordinal as u32);*ordinal+=1;
        let Outcome::SpawnSummoned {entity,pos,unit_id,owner,faction,player_owner,duration}=self.outcome else {unreachable!()};
        if let Some(ledger)=world.try_fetch::<super::super::summon_navigation::PlacementReservations>() {ledger.release(entity);}
        // Every entity and immutable stats reference was checked before the
        // first insertion. No other outcome runs inside this exclusive plan.
        commit_summoned_spawn(world,entity,pos,&unit_id,owner,faction,player_owner,duration,self.stats)?;
        if let Some(fact)=fact {let _=world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(fact);}
        Ok(())
    }
}

pub(super) fn reject(world:&World,caster:Entity,skill:&str,effects:&[Outcome]) {
    release_spawns(world,effects);
    let tick=world.try_fetch::<Tick>().map_or(0,|tick|tick.0);
    if let Some(mut casts)=world.try_fetch_mut::<super::super::cast_windup::AbilityWindups>() {
        casts.result(caster,false);
        // E567: the replica that predicted it drops the cooldown it started.
        casts.failed_ability(tick,caster,skill.to_owned());
    }
}

/// The summons a refused transaction reserved go back (placement ledger, uncommitted entity).
pub(super) fn release_spawns(world:&World,effects:&[Outcome]) {
    for effect in effects {if let Outcome::SpawnSummoned {entity,..}=effect {
        if let Some(ledger)=world.try_fetch::<super::super::summon_navigation::PlacementReservations>() {ledger.release(*entity);}
        if world.read_storage::<Unit>().get(*entity).is_none()&&world.read_storage::<Pos>().get(*entity).is_none()
            &&world.read_storage::<CProperty>().get(*entity).is_none(){let _=world.entities().delete(*entity);}
    }}
}

pub(super) fn publish(world:&World,caster:Entity,cues:Vec<CastSettlementCue>) {
    let mut visuals:Vec<_>=cues.into_iter().map(|cue| {
        let kind=if cue.area.is_some(){super::super::scripting::ScriptVisualEventKind::SkillArea}else{super::super::scripting::ScriptVisualEventKind::SkillCast};
        let mut event=super::super::scripting::ScriptVisualEvent::new(kind,caster,cue.tick);
        event.secondary=cue.target;event.skill_id=cue.skill_id;event.ability_rank=cue.rank;
        event.caster_relocation=cue.relocation;event.ability_area=cue.area;event.target_point=cue.target_point;event
    }).collect();
    super::super::scripting::dispatch::publish_successful_cast_facts(world,&mut visuals);
    world.write_resource::<super::super::scripting::ScriptVisualEventQueue>().extend(visuals);
    if let Some(mut casts)=world.try_fetch_mut::<super::super::cast_windup::AbilityWindups>() {casts.result(caster,true);}
}

#[cfg(test)]
#[path="cast_settlement_tests.rs"]
mod tests;

/// E599: a cast's pathing blockers (原作 B22W's ring of rocks) stand where they are raised, beside
/// each other; another summon still keeps clear of them.
#[cfg(test)]
#[test]
fn pathing_blockers_of_one_cast_stand_beside_each_other() {
    let Some(rock)=omoba_template_ids::summon_by_name("b22w_rock") else {return;};
    assert!(omoba_template_ids::active_summon_stats(rock).unwrap().blocks_pathing);
    let (mut world,hero)=crate::runtime::native::moba_match::stop_command_world();
    world.register::<Faction>();world.register::<CollisionRadius>();world.register::<Unit>();
    let team=world.read_storage::<Faction>().get(hero).map_or(1,|faction|faction.team_id);
    if world.read_storage::<Faction>().get(hero).is_none() {
        world.write_storage::<Faction>().insert(hero,Faction::new(FactionType::Player,team)).unwrap();
    }
    if !world.has_value::<BlockedRegions>() {world.insert(BlockedRegions(Vec::new()));}
    let faction=world.read_storage::<Faction>().get(hero).cloned().unwrap();
    let player_owner=world.read_storage::<PlayerOwner>().get(hero).map(|owner|owner.player_id);
    let origin=world.read_storage::<Pos>().get(hero).unwrap().0;
    let points:Vec<SimVec2>=(0..4).map(|i|origin+SimVec2::new(Fixed64::from_i32(300+50*i),Fixed64::ZERO)).collect();
    let effects:Vec<Outcome>=points.iter().map(|pos|Outcome::SpawnSummoned {entity:world.create_entity().build(),pos:*pos,
        unit_id:"b22w_rock".into(),owner:hero,faction:faction.clone(),player_owner,duration:Fixed64::ONE}).collect();
    let planned=plan_summons(&world,&effects).expect("every rock is placed");
    let placed:Vec<SimVec2>=planned.iter().map(|plan|match &plan.outcome {Outcome::SpawnSummoned {pos,..}=>*pos,_=>unreachable!()}).collect();
    assert_eq!(placed,points,"each rock where the script raised it");
}
