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
    let summons=effects.iter().any(|e|matches!(e,Outcome::SpawnSummoned {..}));
    let channels=effects.iter().filter(|effect|matches!(effect,Outcome::ScriptRecoveryChannel {..})).count();
    if channels>1 || (channels>0&&summons){return false;}
    if skill.is_empty() || (summons&&effects.len()>512) || cue_count>16 || rank>u8::MAX as u32
        || !world.entities().is_alive(caster)
        || world.read_storage::<CProperty>().get(caster).is_none_or(|p|p.hp<=Fixed64::ZERO)
        || world.read_resource::<GamePause>().is_paused
        || world.try_fetch::<super::super::MobaMatch>().is_some_and(|state|state.phase!=super::super::MobaMatchPhase::Playing)
        || world.read_resource::<BuffStore>().is_stunned(caster)
        || world.read_resource::<BuffStore>().is_silenced(caster) {return false;}
    let factions=world.read_storage::<Faction>();
    let same_faction=match (factions.get(caster),faction) {
        (Some(a),Some(b))=>a.team_id==b.team_id&&a.faction_id==b.faction_id,
        (None,None)=>true,_=>false,
    };
    if !same_faction || world.read_storage::<PlayerOwner>().get(caster).map(|p|p.player_id)!=player {return false;}
    let heroes=world.read_storage::<Hero>();let hero=heroes.get(caster);
    if hero.and_then(|h|h.mana_pool.as_ref())!=expected
        || hero.is_some_and(|h|h.is_on_cooldown(skill))
        || (rank>0 && hero.is_none_or(|h|h.get_ability_level(skill)!=rank as i32)) {return false;}
    let mut independent=Vec::new();
    for effect in effects {if let Outcome::AddBuff{target,buff_id,duration,payload}=effect {
        if (payload.get(super::super::ally_cleanse::KEY).is_some() || buff_id==super::super::ally_cleanse::BUFF)
            && super::super::ally_cleanse::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return false;}
        if (payload.get(super::super::mana_drain::KEY).is_some() || buff_id==super::super::mana_drain::BUFF)
            && super::super::mana_drain::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return false;}
        if payload.get(super::super::ally_shield::KEY).is_some()
            && super::super::ally_shield::admitted(world,*target,buff_id,*duration,payload,Some(caster)).is_none(){return false;}
        if payload.get(super::super::ability_runtime::damage_zone::KEY).is_some() {
            if !super::zone_pulse_settlement::admit_zone(world,*target,buff_id,*duration,payload){return false;}
            let spec=super::super::ability_runtime::damage_zone::DamageZoneSpec::from_payload(payload).expect("checked zone");
            if spec.independent {if *target!=caster{return false;}independent.push(buff_id.as_str());}
        }
    }}
    if !world.read_resource::<BuffStore>().can_schedule_independent_zones(caster,&independent){return false;}
    effects.iter().all(|effect|match effect {
        Outcome::ScriptCastTransaction {..}|Outcome::ScriptCastCommitted {..}|Outcome::DamageZonePulse {..}|Outcome::IndependentDamageZonePulse {..}|Outcome::DamageWithLeech {..}|Outcome::DamageWithCritical {..}=>false,
        Outcome::ScriptManaCommit {changes,notifications}=>mana_admitted(world,changes,notifications)
            && ((!summons&&channels==0) || changes.iter().all(|row|row.entity==caster)),
        Outcome::ScriptRecoveryChannel {caster:source,owner,amount,duration,interval}=>*source==caster&&Some(*owner)==player
            && super::super::recovery_channel::can_start(world,caster,*owner,super::super::recovery_channel::Spec {amount:*amount,duration:*duration,interval:*interval}),
        Outcome::SpawnSummoned {owner,..}=>*owner==caster,
        _ if !summons&&channels==0=>true,
        // A formation may update its caster's resources/facing, but cannot
        // insert arbitrary terrain/life mutations between planned siblings.
        Outcome::ScriptSetMana {entity,..}|Outcome::ScriptManaGained {entity,..}
            |Outcome::ScriptSetFacing {entity,..}=>*entity==caster,
        Outcome::ScriptManaSpent {entity,amount,ability_id}=>*entity==caster&&*amount>=Fixed64::ZERO&&ability_id==skill,
        Outcome::ScriptStartCooldown {entity,ability_id,duration}=>*entity==caster&&ability_id==skill
            && *duration>=Fixed64::ZERO&&*duration<=Fixed64::from_i32(1_000_000),
        _=>false,
    })
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
        if stats.hp<=Fixed64::ZERO||stats.attack_interval<=Fixed64::ZERO||stats.attack_range<=Fixed64::ZERO
            ||stats.move_speed<Fixed64::ZERO||stats.collision_radius<Fixed64::ZERO
            ||stats.vision_radius<Fixed64::ZERO||stats.vision_radius>Fixed64::from_i32(10_000){return None;}
        let friendly:Vec<_>=occupied.iter().filter(|(team,_,_)|*team==faction.team_id).map(|(_,p,r)|(*p,*r)).collect();
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

pub(super) fn reject(world:&World,caster:Entity,effects:&[Outcome]) {
    for effect in effects {if let Outcome::SpawnSummoned {entity,..}=effect {
        if let Some(ledger)=world.try_fetch::<super::super::summon_navigation::PlacementReservations>() {ledger.release(*entity);}
        if world.read_storage::<Unit>().get(*entity).is_none()&&world.read_storage::<Pos>().get(*entity).is_none()
            &&world.read_storage::<CProperty>().get(*entity).is_none(){let _=world.entities().delete(*entity);}
    }}
    if let Some(mut casts)=world.try_fetch_mut::<super::super::cast_windup::AbilityWindups>() {casts.result(caster,false);}
}

pub(super) fn publish(world:&World,caster:Entity,cues:Vec<CastSettlementCue>) {
    let mut visuals:Vec<_>=cues.into_iter().map(|cue| {
        let kind=if cue.area.is_some(){super::super::scripting::ScriptVisualEventKind::SkillArea}else{super::super::scripting::ScriptVisualEventKind::SkillCast};
        let mut event=super::super::scripting::ScriptVisualEvent::new(kind,caster,cue.tick);
        event.secondary=cue.target;event.skill_id=cue.skill_id;event.ability_rank=cue.rank;
        event.caster_relocation=cue.relocation;event.ability_area=cue.area;event
    }).collect();
    super::super::scripting::dispatch::publish_successful_cast_facts(world,&mut visuals);
    world.write_resource::<super::super::scripting::ScriptVisualEventQueue>().extend(visuals);
    if let Some(mut casts)=world.try_fetch_mut::<super::super::cast_windup::AbilityWindups>() {casts.result(caster,true);}
}

#[cfg(test)]
#[path="cast_settlement_tests.rs"]
mod tests;
