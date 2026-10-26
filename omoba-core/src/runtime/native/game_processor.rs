use std::borrow::Cow;

/// One common transition for item and authored-skill recovery channels.
fn begin_recovery_channel(world:&mut World,hero:Entity,owner:u32,spec:super::recovery_channel::Spec)->bool {
    if !super::recovery_channel::can_start(world,hero,owner,spec){return false;}
    clear_hero_command_queue(world,hero);
    super::cast_windup::cancel(world,hero);
    interrupt_attack_for_accepted_command(world,hero);
    if let Some(mut state)=world.try_fetch_mut::<super::MobaMatch>() {state.neutralize_direction(owner,hero);}
    super::recovery_channel::start(world,hero,owner,spec);true
}

/// Same command transition as the recovery channel; the host then owns every pulse.
fn begin_ability_channel(world:&mut World,hero:Entity,owner:u32,skill_id:String,level:u8,
    spec:super::ability_channel::Spec,anchor:super::ability_channel::Anchor)->bool {
    if !super::ability_channel::can_start(world,hero,owner,spec,anchor){return false;}
    clear_hero_command_queue(world,hero);
    super::cast_windup::cancel(world,hero);
    interrupt_attack_for_accepted_command(world,hero);
    if let Some(mut state)=world.try_fetch_mut::<super::MobaMatch>() {state.neutralize_direction(owner,hero);}
    super::ability_channel::start(world,hero,owner,skill_id,level,spec,anchor);true
}
use std::collections::BTreeMap;

use omoba_sim::Fixed64;
use serde_json::json;
use specs::{Builder, Entity, Join, LendJoin, World, WorldExt};
use vek::Vec2;

use crate::runtime::ability_runtime::{AbilityRegistry, BuffStore};
use crate::runtime::comp::tower_upgrade_rules;
use crate::runtime::comp::{
    AttackCancelFx, AttackCancelFxQueue, AttackCancelPhase, AttackPhaseFx, AttackPhaseFxQueue,
    AttackSequencePhase, BlockedRegions, Bounty, CProperty, CircularVision, CollisionRadius, Creep,
    CreepData, CreepStatus, ExplosionFx, ExplosionFxQueue, Facing, FacingBroadcast, Faction,
    FactionType, GamePause, Gold, Hero, HeroCommand, HeroCommandQueue, Inventory, IsBase, IsBuilding,
    KnowledgeBonusResource, MasterSeed, MoveTarget, Outcome, Path, PendingAbilityCastQueue,
    PendingAbilityUpgradeQueue, PendingHeroCommandClearQueue, PendingHeroCommandKind,
    PendingItemUseQueue, PendingMoveQueue, PendingTowerAbilityActivation,
    PendingTowerAbilityActivationQueue, PendingTowerAbilityCastQueue, PendingTowerSellQueue,
    PendingTowerSpawnQueue, PendingTowerTargetPriorityQueue, PendingTowerUpgradeQueue,
    PlayerEconomy, PlayerOwner, Pos, Projectile, ProjectileOrigins, ProjectileImpactOrigin, RemovedEntitiesQueue, ReplicationScope,
    ReplicationScopeKind, Searcher, StealthProfile, TAttack, TProperty, Tick, Tower,
    TowerAbilityCastResult, TowerAbilityCastResults, VisionOccluderSet, VisionSource,
    VisibilityOverride, VisibilityOverrideKind,
    TowerActiveAbilityState, TowerData, TowerFireFxQueue, TowerSpawnOrderCounter,
    TowerTargetPriority, TowerTemplate, TowerTemplateRegistry, TowerUpgradeRegistry, TurnSpeed,
    Unit, UnitType, BountyType, SummonedUnit, INVENTORY_SLOTS,
};
use crate::runtime::events::{RuntimeBroadcast, RuntimeEvent, RuntimeEventSink};
use crate::runtime::geometry::{circle_hits_polygon, point_segment_dist_sq};
use crate::game_proto::{player_input::Action as ProtoAction, ItemUse, PlayerInput};
use crate::runtime::item::{ActiveEffect, ItemRegistry};
use crate::runtime::scripting::{ScriptEvent, ScriptEventQueue, ScriptUnitTag, SkillTarget};
use super::command_outcome::CommandRejection;
use crate::tower_meta::UpgradeEffect;
use omb_script_abi::stat_keys::StatKey;
use omb_script_abi::types::DamageProfile as AbiDamageProfile;

fn register_projection_policy(world: &World, action_id: &str, source_module_path: &str) {
    if let Some(mut registry) = world.try_fetch_mut::<crate::runtime::ProjectionPolicyRegistry>() {
        registry.register_str(action_id, source_module_path);
    }
}

const OP_PROJECTILE_ACCURACY: u32 = 20;
const OP_PROJECTILE_STUN_ROLL: u32 = 21;

fn player_hero_entity(
    world: &World,
    context: &str,
    owner_pid: u32,
) -> Result<Entity, failure::Error> {
    let entities = world.entities();
    let heroes = world.read_storage::<Hero>();
    let owners = world.read_storage::<PlayerOwner>();
    let properties = world.read_storage::<CProperty>();
    (&entities, &heroes, &owners)
        .join()
        .find(|(entity, _, owner)| owner.player_id == owner_pid
            && properties.get(*entity).is_none_or(|p| p.hp > Fixed64::ZERO))
        .map(|(e, _, _)| e)
        .ok_or_else(|| {
            failure::err_msg(format!(
                "{}: no Hero entity owned by player_id={}",
                context, owner_pid
            ))
        })
}

pub fn interrupt_attack_for_accepted_command(
    world: &mut World,
    entity: Entity,
) -> Option<AttackCancelPhase> {
    let current_tick = world.read_resource::<Tick>().0 as u32;
    let cancel = {
        let mut attacks = world.write_storage::<TAttack>();
        let Some(attack) = attacks.get_mut(entity) else {
            return None;
        };

        let cancel_phase = if attack.attack_phase == AttackSequencePhase::Windup
            && attack.asd_count < omoba_sim::Fixed64::ZERO
        {
            Some((AttackCancelPhase::Windup, false))
        } else if attack.attack_phase == AttackSequencePhase::Backswing {
            Some((AttackCancelPhase::Backswing, true))
        } else {
            None
        };

        cancel_phase.map(|(phase, impact_committed)| {
            let fx = AttackCancelFx {
                entity_id: entity.id(),
                entity_gen: entity.gen().id() as u32,
                spawn_tick: current_tick,
                attack_seq: attack.attack_seq,
                phase,
                impact_committed,
            };
            attack.asd_count = attack.asd.v;
            attack.clear_attack_sequence();
            fx
        })
    };

    if let Some(fx) = cancel {
        let phase = fx.phase;
        world
            .write_resource::<AttackCancelFxQueue>()
            .pending
            .push(fx);
        Some(phase)
    } else {
        None
    }
}

fn entity_by_id(world: &World, entity_id: u32) -> Option<Entity> {
    let entities = world.entities();
    (&entities).join().find(|e| e.id() == entity_id)
}

pub(crate) fn clear_hero_command_queue(world: &mut World, hero: Entity) {
    {
        let mut queues = world.write_storage::<HeroCommandQueue>();
        if let Some(queue) = queues.get_mut(hero) {
            queue.clear_all();
        } else {
            let _ = queues.insert(hero, HeroCommandQueue::default());
        }
    }
    world.write_storage::<MoveTarget>().remove(hero);
}

fn clear_hero_command_queue_for_player(
    world: &mut World,
    context: &str,
    owner_pid: u32,
) -> Result<Entity, failure::Error> {
    let hero = player_hero_entity(world, context, owner_pid)?;
    clear_hero_command_queue(world, hero);
    Ok(hero)
}

fn validate_attack_target(
    world: &World,
    hero: Entity,
    target_entity_id: u32,
    owner_pid: u32,
) -> Result<Entity, failure::Error> {
    let target = entity_by_id(world, target_entity_id).ok_or_else(|| {
        failure::err_msg(format!(
            "AttackTarget: entity id={} not found (pid={})",
            target_entity_id, owner_pid
        ))
    })?;
    let factions = world.read_storage::<Faction>();
    let hero_faction = factions.get(hero).ok_or_else(|| {
        failure::err_msg(format!(
            "AttackTarget: hero {:?} has no Faction (pid={})",
            hero, owner_pid
        ))
    })?;
    let target_faction = factions.get(target).ok_or_else(|| {
        failure::err_msg(format!(
            "AttackTarget: target id={} has no Faction (pid={})",
            target_entity_id, owner_pid
        ))
    })?;
    if hero_faction.team_id == target_faction.team_id {
        return Err(failure::err_msg(format!(
            "AttackTarget: target id={} is allied with pid={}",
            target_entity_id, owner_pid
        )));
    }
    Ok(target)
}

/// An active hero action requires the living owned hero. A present match also
/// requires Playing, an unpaused world, and the roster slot for that hero.
/// Worlds without a match keep the owner/alive gate and still honor GamePause.
fn active_hero_context_allows(world: &World, hero: Entity, owner_pid: u32) -> bool {
    if world.try_fetch::<GamePause>().is_some_and(|pause| pause.is_paused) {
        return false;
    }
    let Some(state) = world.try_fetch::<crate::runtime::MobaMatch>() else {
        return true;
    };
    state.phase == crate::runtime::MobaMatchPhase::Playing
        && state.heroes.iter().any(|slot| slot.player_id == owner_pid && slot.entity == Some(hero))
}

/// MOBA move orders head for the closest reachable point when the clicked
/// destination is inside terrain or cut off (one deterministic rule on the
/// authority and every replica). `None` only when the hero cannot get closer.
fn resolve_move_destination(world: &World, hero: Entity, target: omoba_sim::Vec2) -> Option<omoba_sim::Vec2> {
    let Some(from) = world.read_storage::<Pos>().get(hero).map(|pos| pos.0) else { return Some(target) };
    let radius = world.read_storage::<CollisionRadius>().get(hero).map(|r| r.0)
        .unwrap_or_else(|| Fixed64::from_i32(20));
    let regions = world.read_resource::<crate::comp::BlockedRegions>();
    let terrain = crate::tick::hero_command_tick::TerrainIndex::new(&regions);
    let mut memo = crate::tick::hero_command_tick::NavMemo::default();
    let resolved = crate::tick::hero_command_tick::static_nearest_reachable(from, target, radius, &terrain, &mut memo);
    if let Some(point) = resolved.filter(|point| *point != target) {
        log::info!(
            "HeroCommand destination unreachable; heading for nearest reachable point hero={:?} target=({:.1},{:.1}) nearest=({:.1},{:.1})",
            hero, target.x.to_f32_for_render(), target.y.to_f32_for_render(),
            point.x.to_f32_for_render(), point.y.to_f32_for_render()
        );
    }
    resolved
}

fn reject_unreachable_move(world: &World, owner_pid: u32, correlation: u32, hero: Entity, target: omoba_sim::Vec2) {
    log::warn!(
        "HeroCommand rejected pid={} hero={:?} reason=UNREACHABLE target=({:.1},{:.1})",
        owner_pid, hero, target.x.to_f32_for_render(), target.y.to_f32_for_render()
    );
    super::command_outcome::reject(world, owner_pid, correlation, super::command_outcome::UNREACHABLE);
}

pub fn drain_pending_moves(world: &mut World) {
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::MOVEMENT,
        "runtime/native/game_processor/drain_pending_moves",
    );
    let drained = {
        let mut q = world.write_resource::<PendingMoveQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_moves",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    if drained.is_empty() {
        drop(drain_span);
        return;
    }

    for req in drained {
        let hero = match player_hero_entity(world, "HeroCommand", req.owner_pid) {
            Ok(e) => e,
            Err(e) => {
                log::warn!("{}", e);
                continue;
            }
        };
        let command = match req.kind {
            PendingHeroCommandKind::OwnedSummonOrder {order} => {
                if req.queued {continue;}
                if let Err(reason)=super::summon_orders::apply(world,req.owner_pid,hero,&order) {
                    log::debug!("OwnedSummonOrder rejected pid={} reason={}",req.owner_pid,reason);
                }
                continue;
            }
            PendingHeroCommandKind::Stop => {
                if req.queued {
                    log::warn!(
                        "HeroCommand rejected pid={} hero={:?} kind=stop reason=not_queueable",
                        req.owner_pid, hero,
                    );
                    continue;
                }
                if !active_hero_context_allows(world, hero, req.owner_pid) {
                    log::warn!(
                        "Stop rejected pid={} hero={:?} reason=inactive_or_unowned",
                        req.owner_pid, hero,
                    );
                    continue;
                }
                clear_hero_command_queue(world, hero);
                super::cast_windup::cancel(world,hero);
                if let Some(mut state)=world.try_fetch_mut::<crate::runtime::MobaMatch>() {
                    state.neutralize_direction(req.owner_pid, hero);
                }
                interrupt_attack_for_accepted_command(world, hero);
                continue;
            }
            PendingHeroCommandKind::HoldPosition => HeroCommand::HoldPosition,
            PendingHeroCommandKind::MoveTo { pos } => match resolve_move_destination(world, hero, pos) {
                Some(pos) => HeroCommand::MoveTo { pos },
                None => {
                    reject_unreachable_move(world, req.owner_pid, req.admitted_correlation, hero, pos);
                    continue;
                }
            },
            PendingHeroCommandKind::AttackMove { pos } => match resolve_move_destination(world, hero, pos) {
                Some(pos) => HeroCommand::AttackMove { pos },
                None => {
                    reject_unreachable_move(world, req.owner_pid, req.admitted_correlation, hero, pos);
                    continue;
                }
            },
            PendingHeroCommandKind::AttackTarget { target_entity_id } => {
                match validate_attack_target(world, hero, target_entity_id, req.owner_pid) {
                    Ok(target) => HeroCommand::AttackTarget {
                        target,
                        chase_origin: None,
                    },
                    Err(e) => {
                        log::warn!("{}", e);
                        continue;
                    }
                }
            }
        };
        log::info!(
            "HeroCommand pid={} -> hero={:?} kind={} queued={}",
            req.owner_pid,
            hero,
            command.command_type(),
            req.queued,
        );
        if req.queued {
            let accepted = {
                let mut queues = world.write_storage::<HeroCommandQueue>();
                if queues.get(hero).is_none() {
                    let _ = queues.insert(hero, HeroCommandQueue::default());
                }
                let queue = queues
                    .get_mut(hero)
                    .expect("HeroCommandQueue just inserted but missing");
                queue.append(command)
            };
            if !accepted {
                log::warn!(
                    "HeroCommand rejected pid={} hero={:?} kind={} reason=queue_limit limit={}",
                    req.owner_pid,
                    hero,
                    command.command_type(),
                    HeroCommandQueue::LIMIT,
                );
            }
            continue;
        }

        clear_hero_command_queue(world, hero);
        super::cast_windup::cancel(world,hero);
        if !matches!(command, HeroCommand::AttackTarget { .. }) {
            if let Some(mut state)=world.try_fetch_mut::<crate::runtime::MobaMatch>() {
                state.neutralize_direction(req.owner_pid, hero);
            }
        }
        if !matches!(command, HeroCommand::AttackMove { .. }) {
            interrupt_attack_for_accepted_command(world, hero);
        }
        let mut queues = world.write_storage::<HeroCommandQueue>();
        if queues.get(hero).is_none() {
            let _ = queues.insert(hero, HeroCommandQueue::default());
        }
        let queue = queues
            .get_mut(hero)
            .expect("HeroCommandQueue just inserted but missing");
        queue.replace(command);
    }
    drop(drain_span);
}

pub fn drain_pending_hero_command_clears(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingHeroCommandClearQueue>();
        std::mem::take(&mut q.requests)
    };
    for owner_pid in drained {
        if let Err(e) = clear_hero_command_queue_for_player(world, "HeroCommandClear", owner_pid) {
            log::warn!("{}", e);
        }
    }
}

pub fn handle_ability_upgrade_from_input(
    world: &mut World,
    ability_index: u32,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    admit_ability_upgrade(world, ability_index, owner_pid).map_err(|rejection| failure::err_msg(rejection.message))
}

/// Every rejection carries a stable code for the owner's command result.
fn admit_ability_upgrade(
    world: &mut World,
    ability_index: u32,
    owner_pid: u32,
) -> Result<(), CommandRejection> {
    use super::command_outcome as code;
    if ability_index as usize >= crate::runtime::HERO_ABILITY_SLOTS {
        return Err(CommandRejection::new(code::INVALID_SLOT, format!(
            "AbilityUpgrade: invalid ability_index={} (must be below {}) pid={}",
            ability_index, crate::runtime::HERO_ABILITY_SLOTS, owner_pid
        )));
    }
    let slot = ability_index as usize;

    let hero_entity = player_hero_entity(world, "AbilityUpgrade", owner_pid)
        .map_err(|error| CommandRejection::new(code::NO_HERO, error.to_string()))?;

    let lane = world.try_fetch::<crate::runtime::MobaMatch>();
    let is_lane = lane.is_some();
    if let Some(state) = lane.as_ref() {
        if state.phase != crate::runtime::MobaMatchPhase::Playing
            || world.read_resource::<crate::runtime::GamePause>().is_paused
        {
            return Err(CommandRejection::new(code::GAME_NOT_ACTIVE, "AbilityUpgrade: inactive or invalid lane hero"));
        }
        if !state.heroes.iter().any(|slot| slot.player_id == owner_pid && slot.entity == Some(hero_entity))
            || world.read_storage::<CProperty>().get(hero_entity).is_none_or(|p| p.hp <= Fixed64::ZERO)
        {
            return Err(CommandRejection::new(code::HERO_INACTIVE, "AbilityUpgrade: inactive or invalid lane hero"));
        }
    }
    drop(lane);

    let ability_id = {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(hero_entity).ok_or_else(|| {
            CommandRejection::new(code::NO_HERO, format!(
                "AbilityUpgrade: hero entity vanished before read (pid={})",
                owner_pid
            ))
        })?;
        hero.abilities
            .get(slot)
            .filter(|id| !id.is_empty())
            .cloned()
            .ok_or_else(|| {
                CommandRejection::new(code::ABILITY_NOT_BOUND, format!(
                    "AbilityUpgrade: hero has no ability bound at slot={} (pid={})",
                    slot, owner_pid
                ))
            })?
    };

    let max_level = {
        let registry = world.read_resource::<AbilityRegistry>();
        let definition = registry.get(&ability_id);
        if is_lane && definition.is_none() {
            return Err(CommandRejection::new(code::UPGRADE_REJECTED, "AbilityUpgrade: missing lane ability metadata"));
        }
        definition.map(|def| i32::from(def.max_level).max(1)).unwrap_or(5)
    };
    // Learned automatically at its level gates; a skill point can never buy it.
    if omoba_template_ids::ability_by_name(&ability_id).and_then(omoba_template_ids::active_ability_const)
        .is_some_and(|definition| definition.auto_learn) {
        return Err(CommandRejection::new(code::UPGRADE_REJECTED, format!(
            "AbilityUpgrade: slot={} ability='{}' is learned automatically pid={}", slot, ability_id, owner_pid)));
    }

    let new_level = {
        let mut heroes = world.write_storage::<Hero>();
        let hero = heroes.get_mut(hero_entity).ok_or_else(|| {
            CommandRejection::new(code::NO_HERO, format!(
                "AbilityUpgrade: hero entity vanished before write (pid={})",
                owner_pid
            ))
        })?;
        if hero.skill_points <= 0 {
            return Err(CommandRejection::new(code::SKILL_POINTS, format!(
                "AbilityUpgrade: no skill points slot={} ability='{}' pid={}",
                slot, ability_id, owner_pid
            )));
        }
        let current = hero.ability_levels.get(&ability_id).copied().unwrap_or(0);
        if is_lane && current < 0 {
            return Err(CommandRejection::new(code::UPGRADE_REJECTED, "AbilityUpgrade: invalid lane ability rank"));
        }
        if current >= max_level {
            return Err(CommandRejection::new(code::MAX_LEVEL, format!(
                "AbilityUpgrade: slot={} ability='{}' already maxed ({}/{}) pid={}",
                slot, ability_id, current, max_level, owner_pid
            )));
        }
        let next = current + 1;
        if is_lane {
            let required = omoba_template_ids::ability_by_name(&ability_id)
                .and_then(omoba_template_ids::active_ability_const)
                .and_then(|definition| definition.levels.get(current as usize))
                .ok_or_else(|| CommandRejection::new(code::UPGRADE_REJECTED, "AbilityUpgrade: missing lane rank requirements"))?
                .required_hero_level;
            if hero.level < i32::from(required) {
                return Err(CommandRejection::new(code::LEVEL_REQUIRED, format!("AbilityUpgrade: requires hero level {}", required)));
            }
        }
        hero.ability_levels.insert(ability_id.clone(), next);
        hero.skill_points -= 1;
        next
    };

    clear_hero_command_queue(world, hero_entity);

    world
        .write_resource::<ScriptEventQueue>()
        .push(ScriptEvent::SkillLearn {
            caster: hero_entity,
            skill_id: ability_id.clone(),
            new_level: new_level.max(1) as u8,
        });

    log::info!(
        "AbilityUpgrade ok pid={} slot={} ability='{}' level={}",
        owner_pid,
        slot,
        ability_id,
        new_level
    );
    Ok(())
}

/// Hero lives (entities) whose held ranks already ran their learn hooks in this
/// world. A respawn is a new entity, so each life relearns exactly once. World-local
/// bookkeeping: every world sees the same life start on the same tick.
#[derive(Default)]
struct LifeRanksLearned(std::collections::BTreeSet<Entity>);

/// Every rank a hero holds when a life starts (MOBA birth loadout, respawn with
/// persistent ranks) runs that skill's learn hook once for the life, so permanent
/// learn effects (passive stat buffs, rank markers) exist from the life's first
/// upgrade phase, not only after a skill point is spent (split45: born-rank-1
/// passives never applied, and death erased learned ones). Learn hooks are
/// rank-idempotent (fixed buff ids). Runs in the AbilityUpgrades phase on the
/// authority and every replica alike.
fn relearn_life_ranks(world: &mut World) {
    if world.try_fetch::<LifeRanksLearned>().is_none() { world.insert(LifeRanksLearned::default()); }
    let mut learn = Vec::new();
    {
        let entities = world.entities();
        let properties = world.read_storage::<CProperty>();
        let heroes = world.read_storage::<Hero>();
        let mut lives = world.write_resource::<LifeRanksLearned>();
        lives.0.retain(|entity| entities.is_alive(*entity));
        for (entity, hero, property) in (&entities, &heroes, &properties).join() {
            if property.hp <= Fixed64::ZERO || !lives.0.insert(entity) { continue; }
            for id in hero.abilities.iter().filter(|id| !id.is_empty()) {
                let rank = hero.ability_levels.get(id).copied().unwrap_or(0);
                if rank > 0 { learn.push((entity, id.clone(), rank)); }
            }
        }
    }
    if learn.is_empty() { return; }
    let mut queue = world.write_resource::<ScriptEventQueue>();
    for (caster, skill_id, rank) in learn {
        queue.push(ScriptEvent::SkillLearn { caster, skill_id, new_level: rank.clamp(1, i32::from(u8::MAX)) as u8 });
    }
}

/// Max-HP bonus already folded into each life's `CProperty.mhp`.
#[derive(Default)]
struct AppliedHealthBonus(BTreeMap<Entity, Fixed64>);

/// Max-HP stat buffs (HealthBonus / ExtraHealthBonus) are folded into
/// `CProperty.mhp` as a delta against what the life already received, like item
/// stats and level growth: a rising bonus raises HP by the same amount, a falling
/// one clamps HP. Every reader (disclosure, HUD, heals, regen) then sees one max HP
/// and `hp <= mhp` holds on the authority and every replica (split45: regen above
/// the disclosed max made the team replica reject the baseline).
fn settle_health_bonus(world: &mut World) {
    if world.try_fetch::<crate::runtime::BuffStore>().is_none() { return; }
    if world.try_fetch::<AppliedHealthBonus>().is_none() { world.insert(AppliedHealthBonus::default()); }
    let entities = world.entities();
    let buffs = world.read_resource::<crate::runtime::BuffStore>();
    let mut applied = world.write_resource::<AppliedHealthBonus>();
    let mut properties = world.write_storage::<CProperty>();
    applied.0.retain(|entity, _| entities.is_alive(*entity));
    let mut subjects: Vec<Entity> = [omb_script_abi::stat_keys::StatKey::HealthBonus, omb_script_abi::stat_keys::StatKey::ExtraHealthBonus]
        .iter().flat_map(|key| buffs.entities_with_key(key.as_str())).chain(applied.0.keys().copied()).collect();
    subjects.sort_by_key(|entity| (entity.id(), entity.gen().id()));
    subjects.dedup();
    let stats = crate::runtime::ability_runtime::UnitStats::from_refs(&buffs, false);
    for entity in subjects {
        let Some(property) = properties.get_mut(entity) else { applied.0.remove(&entity); continue };
        let before = applied.0.get(&entity).copied().unwrap_or(Fixed64::ZERO);
        let bonus = stats.max_hp_bonus(entity);
        let delta = bonus - before;
        if delta == Fixed64::ZERO { continue; }
        // Never below one raw HP of capacity, whatever the buffs say.
        let mhp = (property.mhp + delta).max(Fixed64::from_raw(1));
        let applied_delta = mhp - property.mhp;
        property.mhp = mhp;
        if applied_delta > Fixed64::ZERO && property.hp > Fixed64::ZERO { property.hp += applied_delta; }
        if property.hp > property.mhp { property.hp = property.mhp; }
        let total = before + applied_delta;
        if total == Fixed64::ZERO { applied.0.remove(&entity); } else { applied.0.insert(entity, total); }
    }
}

#[cfg(test)]
#[test]
fn every_hero_life_relearns_its_held_ranks_once() {
    let mut world = World::new();
    world.register::<Hero>();
    world.register::<CProperty>();
    world.insert(ScriptEventQueue::default());
    let mut hero = Hero::default();
    hero.abilities = vec!["q".into(), "w".into(), "e".into(), "r".into()];
    for (id, rank) in [("q", 1), ("w", 1), ("e", 2), ("r", 0)] { hero.ability_levels.insert(id.into(), rank); }
    let life = |hp: i32| CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::ONE, msd: Fixed64::ZERO,
        def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
    let first = world.create_entity().with(hero.clone()).with(life(1)).build();
    world.create_entity().with(hero.clone()).with(life(0)).build();
    relearn_life_ranks(&mut world);
    assert_eq!(world.read_resource::<ScriptEventQueue>().len(), 3, "every held rank, never an unlearned slot or a dead hero");
    relearn_life_ranks(&mut world);
    assert_eq!(world.read_resource::<ScriptEventQueue>().len(), 3, "once per life");
    // Death and respawn: a new entity is a new life and learns again.
    world.delete_entity(first).unwrap(); world.maintain();
    world.create_entity().with(hero).with(life(1)).build();
    relearn_life_ranks(&mut world);
    assert_eq!(world.read_resource::<ScriptEventQueue>().len(), 6, "a new life relearns");
}

#[cfg(test)]
#[test]
fn health_bonus_folds_into_max_hp_and_keeps_hp_within_it() {
    let mut world = World::new();
    world.register::<CProperty>();
    world.insert(crate::runtime::BuffStore::default());
    let unit = world.create_entity().with(CProperty { hp: Fixed64::from_i32(400), mhp: Fixed64::from_i32(500), msd: Fixed64::ZERO,
        def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
    let hp = |world: &World| { let p = world.read_storage::<CProperty>().get(unit).unwrap().clone(); (p.hp, p.mhp) };
    world.write_resource::<crate::runtime::BuffStore>().add(unit, "bonus", crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
        serde_json::json!({ omb_script_abi::stat_keys::StatKey::HealthBonus.as_str(): 100.0 }));
    settle_health_bonus(&mut world);
    assert_eq!(hp(&world), (Fixed64::from_i32(500), Fixed64::from_i32(600)), "max and current rise together");
    settle_health_bonus(&mut world);
    assert_eq!(hp(&world), (Fixed64::from_i32(500), Fixed64::from_i32(600)), "applied once");
    world.write_resource::<crate::runtime::BuffStore>().remove(unit, "bonus");
    settle_health_bonus(&mut world);
    assert_eq!(hp(&world), (Fixed64::from_i32(500), Fixed64::from_i32(500)), "removal restores the max and clamps nothing below it");
}

/// The rank an auto-learned ability has at `hero_level`: every next rank whose
/// requirement is met, up to `max_level`. Never lowers a rank.
pub(crate) fn auto_learned_rank(current: i32, max_level: u8, required: impl IntoIterator<Item = u8>, hero_level: i32) -> i32 {
    let required: Vec<u8> = required.into_iter().collect();
    let mut rank = current.max(0);
    while rank < i32::from(max_level) && required.get(rank as usize).is_some_and(|level| hero_level >= i32::from(*level)) { rank += 1; }
    rank
}

#[cfg(test)]
#[test]
fn auto_learned_ranks_follow_each_level_gate_and_never_drop() {
    // H06 style: one rank at hero level 16.
    assert_eq!(auto_learned_rank(0, 1, [16], 15), 0);
    assert_eq!(auto_learned_rank(0, 1, [16], 16), 1);
    assert_eq!(auto_learned_rank(1, 1, [16], 25), 1, "max level caps it");
    // H04 style: born at rank 1 with no further rank.
    assert_eq!(auto_learned_rank(1, 1, [1], 1), 1);
    // Several gates reached at once (a big XP reward) learn every rank.
    assert_eq!(auto_learned_rank(0, 3, [6, 11, 16], 12), 2);
    assert_eq!(auto_learned_rank(3, 3, [6, 11, 16], 1), 3, "a rank is never removed");
    assert_eq!(auto_learned_rank(-1, 2, [1, 1], 1), 2, "a negative rank is treated as unlearned");
}

/// (hero life, ability, rank) already announced by the learn log. Log bookkeeping
/// only: never hashed, never disclosed, never read by gameplay.
#[derive(Default)]
struct AutoLearnAnnounced(std::collections::BTreeSet<(Entity, String, i32)>);

/// Source NOT_LEARNABLE abilities (`auto_learn`): every rank whose
/// `required_hero_level` the hero has reached is learned now, without a skill
/// point, in any slot. It runs at the start of the AbilityUpgrades phase in the
/// authority and in every replica, from the hero level they already share, so a
/// replica learns on the same tick. A dead hero catches up when it respawns.
fn auto_learn_gated_ranks(world: &mut World) {
    if world.try_fetch::<AutoLearnAnnounced>().is_none() { world.insert(AutoLearnAnnounced::default()); }
    let mut learned = Vec::new();
    let mut already = Vec::new();
    {
        let entities = world.entities();
        let properties = world.read_storage::<CProperty>();
        let mut heroes = world.write_storage::<Hero>();
        for (entity, hero, property) in (&entities, &mut heroes, &properties).join() {
            if property.hp <= Fixed64::ZERO { continue; }
            for (slot, id) in hero.abilities.clone().into_iter().enumerate() {
                let Some(definition) = omoba_template_ids::ability_by_name(&id)
                    .and_then(omoba_template_ids::active_ability_const).filter(|definition| definition.auto_learn) else { continue };
                let start = hero.ability_levels.get(&id).copied().unwrap_or(0).max(0);
                let rank = auto_learned_rank(start, definition.max_level,
                    definition.levels.iter().map(|level| level.required_hero_level), hero.level);
                if rank != start {
                    hero.ability_levels.insert(id.clone(), rank);
                    learned.push((entity, slot, id, rank));
                } else if start > 0 {
                    already.push((entity, id, start));
                }
            }
        }
    }
    // A rank the hero already holds (its birth loadout, or a replica restoring a
    // disclosed hero) is announced once per life with the learn hook's log line, so
    // log-based acceptance sees every auto-learned rank. Log only: no on_learn rerun.
    {
        let mut announced = world.write_resource::<AutoLearnAnnounced>();
        for (caster, skill_id, rank) in already {
            if announced.0.insert((caster, skill_id.clone(), rank)) {
                log::info!("[scripting] learn ability='{}' caster={} rank={}", skill_id, caster.id(), rank);
            }
        }
        for (caster, _, skill_id, rank) in &learned { announced.0.insert((*caster, skill_id.clone(), *rank)); }
        if announced.0.len() > 4096 {
            let entities = world.entities();
            announced.0.retain(|(entity, _, _)| entities.is_alive(*entity));
        }
    }
    for (caster, slot, skill_id, rank) in learned {
        log::info!("AbilityAutoLearn slot={} ability='{}' level={}", slot, skill_id, rank);
        world.write_resource::<ScriptEventQueue>().push(ScriptEvent::SkillLearn {
            caster, skill_id, new_level: rank.clamp(1, i32::from(u8::MAX)) as u8 });
    }
}

#[cfg(test)]
#[test]
fn auto_learned_birth_ranks_are_announced_once_per_life() {
    let mut world = World::new();
    world.register::<Hero>();
    world.register::<CProperty>();
    world.insert(ScriptEventQueue::default());
    let born = |world: &mut World| {
        let mut hero = Hero::default();
        hero.abilities = vec!["c17_devotion".into()];
        hero.ability_levels.insert("c17_devotion".into(), 1);
        hero.level = 1;
        world.create_entity().with(hero).with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build()
    };
    let first = born(&mut world);
    let announced = |world: &World| world.read_resource::<AutoLearnAnnounced>().0.clone();
    auto_learn_gated_ranks(&mut world);
    assert!(announced(&world).contains(&(first, "c17_devotion".to_owned(), 1)), "birth rank announced");
    let once = announced(&world);
    auto_learn_gated_ranks(&mut world);
    assert_eq!(announced(&world), once, "announced once per life");
    assert_eq!(world.read_resource::<ScriptEventQueue>().len(), 0, "log only: no learn hook rerun, no rank change");
    let second = born(&mut world);
    auto_learn_gated_ranks(&mut world);
    assert!(announced(&world).contains(&(second, "c17_devotion".to_owned(), 1)), "a new life announces again");
}

pub fn drain_pending_ability_upgrades(world: &mut World) {
    super::command_outcome::ensure(world);
    settle_health_bonus(world);
    relearn_life_ranks(world);
    auto_learn_gated_ranks(world);
    let drained = {
        let mut q = world.write_resource::<PendingAbilityUpgradeQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_ability_upgrades",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        if let Err(rejection) = admit_ability_upgrade(world, req.ability_index, req.owner_pid) {
            log::warn!(
                "AbilityUpgrade failed pid={} ability_index={} code={}: {}",
                req.owner_pid,
                req.ability_index,
                rejection.code,
                rejection
            );
            super::command_outcome::reject(world, req.owner_pid, req.correlation, rejection.code);
        }
    }
    drop(drain_span);
}

pub fn handle_ability_cast_from_input(
    world: &mut World,
    ability_index: u32,
    target_pos: Option<omoba_sim::Vec2>,
    target_entity: Option<u32>,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    admit_ability_cast(world, ability_index, target_pos, target_entity, owner_pid)
        .map(|_| ())
        .map_err(|rejection| failure::err_msg(rejection.message))
}

/// Admission only. The queued SkillCast can still be rejected later in the
/// same tick (rank, mana, range, handler); `drain_pending_ability_casts`
/// tracks the returned caster and skill so dispatch can report that too.
fn admit_ability_cast(
    world: &mut World,
    ability_index: u32,
    target_pos: Option<omoba_sim::Vec2>,
    target_entity: Option<u32>,
    owner_pid: u32,
) -> Result<(Entity, String, Option<&'static str>), CommandRejection> {
    use super::command_outcome as code;
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::HERO_ABILITY,
        "runtime/native/game_processor/handle_ability_cast_from_input",
    );
    if ability_index as usize >= crate::runtime::HERO_ABILITY_SLOTS {
        return Err(CommandRejection::new(code::INVALID_SLOT, format!(
            "AbilityCast: invalid ability_index={} (must be below {}) pid={}",
            ability_index, crate::runtime::HERO_ABILITY_SLOTS, owner_pid
        )));
    }
    let slot = ability_index as usize;

    let caster = player_hero_entity(world, "AbilityCast", owner_pid)
        .map_err(|error| CommandRejection::new(code::NO_HERO, error.to_string()))?;

    let ability_id = {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(caster).ok_or_else(|| {
            CommandRejection::new(code::NO_HERO, format!(
                "AbilityCast: hero entity vanished before read (pid={})",
                owner_pid
            ))
        })?;
        hero.abilities
            .get(slot)
            .filter(|id| !id.is_empty())
            .cloned()
            .ok_or_else(|| {
                CommandRejection::new(code::ABILITY_NOT_BOUND, format!(
                    "AbilityCast: hero has no ability bound at slot={} (pid={})",
                    slot, owner_pid
                ))
            })?
    };

    {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(caster).ok_or_else(|| {
            CommandRejection::new(code::NO_HERO, format!(
                "AbilityCast: hero entity vanished before gate (pid={})",
                owner_pid
            ))
        })?;
        if !hero.can_use_ability(&ability_id) {
            return Err(CommandRejection::new(code::NOT_LEARNED, format!(
                "AbilityCast: slot={} ability='{}' not learned (pid={})",
                slot, ability_id, owner_pid
            )));
        }
        if !hero.is_on_cooldown(&ability_id) && !hero.has_charge(&ability_id) {
            return Err(CommandRejection::new(code::NO_CHARGE, format!(
                "AbilityCast: slot={} ability='{}' has no stored charge pid={}", slot, ability_id, owner_pid
            )));
        }
        if hero.is_on_cooldown(&ability_id) {
            return Err(CommandRejection::new(code::ON_COOLDOWN, format!(
                "AbilityCast: slot={} ability='{}' still on cooldown ({:.2}s) pid={}",
                slot,
                ability_id,
                hero.get_cooldown(&ability_id).to_f32_for_render(),
                owner_pid
            )));
        }
    }

    // Input shape only: a unit or point ability submitted without that target.
    // The cast is still queued (settlement decides); a later failure reports this.
    let shape = world.try_fetch::<AbilityRegistry>().and_then(|registry| registry.get(&ability_id).map(|def| def.target_type.clone()))
        .and_then(|kind| match kind {
            crate::ability_meta::TargetType::Unit if target_entity.is_none() => Some(code::TARGET_REQUIRED),
            crate::ability_meta::TargetType::Point if target_pos.is_none() => Some(code::TARGET_REQUIRED),
            _ => None,
        });

    let target = if let Some(entity_id) = target_entity {
        let entities = world.entities();
        (&entities)
            .join()
            .find(|e| e.id() == entity_id)
            .map(SkillTarget::Entity)
            .unwrap_or(SkillTarget::None)
    } else if let Some(pos) = target_pos {
        SkillTarget::Point { x: pos.x, y: pos.y }
    } else {
        SkillTarget::None
    };

    clear_hero_command_queue(world, caster);
    super::cast_windup::cancel(world,caster);
    interrupt_attack_for_accepted_command(world, caster);

    world
        .write_resource::<ScriptEventQueue>()
        .push(ScriptEvent::SkillCast {
            caster,
            skill_id: ability_id.clone(),
            target,
        });

    log::info!(
        "AbilityCast ok pid={} slot={} ability='{}'",
        owner_pid,
        slot,
        ability_id
    );
    Ok((caster, ability_id, shape))
}

pub fn drain_pending_ability_casts(world: &mut World) {
    super::command_outcome::ensure(world);
    world.write_resource::<super::command_outcome::OwnerCommandOutcomes>().begin_casts();
    let drained = {
        let mut q = world.write_resource::<PendingAbilityCastQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_ability_casts",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        match admit_ability_cast(
            world,
            req.ability_index,
            req.target_pos,
            req.target_entity,
            req.owner_pid,
        ) {
            Ok((caster, skill_id, shape)) => world
                .write_resource::<super::command_outcome::OwnerCommandOutcomes>()
                .track_cast(caster, &skill_id, req.owner_pid, req.correlation, shape),
            Err(rejection) => {
                log::warn!(
                    "AbilityCast failed pid={} ability_index={} code={}: {}",
                    req.owner_pid,
                    req.ability_index,
                    rejection.code,
                    rejection
                );
                super::command_outcome::reject(world, req.owner_pid, req.correlation, rejection.code);
            }
        }
    }
    drop(drain_span);
}

pub fn hero_knowledge_category_for_unit_id(unit_id: &str) -> &'static str {
    match unit_id {
        "tower_dart" => "tower_dart",
        "tower_bomb" => "tower_bomb",
        "tower_ice" => "tower_ice",
        "tower_tack" => "tower_tack",
        "tower_cake_splash" => "tower_cake_splash",
        "tower_boomerang" => "tower_boomerang",
        "tower_arty" => "tower_arty",
        _ => "",
    }
}

fn player_economy_balance(
    world: &World,
    context: &str,
    player_id: u32,
) -> Result<i32, failure::Error> {
    let economy = world
        .try_fetch::<PlayerEconomy>()
        .ok_or_else(|| failure::err_msg(format!("{context}: PlayerEconomy resource is missing")))?;
    economy.balance(player_id).ok_or_else(|| {
        failure::err_msg(format!(
            "{context}: missing player economy account for player_id={player_id}"
        ))
    })
}

fn debit_player_economy(
    world: &mut World,
    context: &str,
    player_id: u32,
    cost: i32,
) -> Result<i32, failure::Error> {
    let is_td = world
        .try_fetch::<crate::runtime::GameMode>()
        .map(|mode| mode.is_td())
        .unwrap_or(false);
    if is_td {
        let tick = world.try_fetch::<Tick>().map(|tick| tick.0).unwrap_or(0);
        let category = match context {
            "TowerPlace" => crate::runtime::TdEconomyCategory::TowerPlace,
            "TowerUpgrade" => crate::runtime::TdEconomyCategory::TowerUpgrade,
            _ => {
                return Err(failure::err_msg(format!(
                    "unknown TD debit context={context}"
                )))
            }
        };
        let mut ledger = world.write_resource::<crate::runtime::TdEconomyLedger>();
        let mut economy = world.write_resource::<PlayerEconomy>();
        return ledger
            .apply(
                &mut economy,
                tick,
                Some(player_id),
                category,
                cost.saturating_neg(),
                context,
            )
            .map(|balance| balance.unwrap_or_default())
            .map_err(|error| failure::err_msg(format!("{context}: {error}")));
    }
    let mut economy = world
        .try_fetch_mut::<PlayerEconomy>()
        .ok_or_else(|| failure::err_msg(format!("{context}: PlayerEconomy resource is missing")))?;
    economy
        .try_debit(player_id, cost)
        .map_err(|error| failure::err_msg(format!("{context}: {error}")))
}

fn credit_player_economy(
    world: &mut World,
    context: &str,
    player_id: u32,
    amount: i32,
) -> Result<i32, failure::Error> {
    let is_td = world
        .try_fetch::<crate::runtime::GameMode>()
        .map(|mode| mode.is_td())
        .unwrap_or(false);
    if is_td {
        let tick = world.try_fetch::<Tick>().map(|tick| tick.0).unwrap_or(0);
        let category = match context {
            "TowerSell" => crate::runtime::TdEconomyCategory::TowerSell,
            "TDCreepBounty" => crate::runtime::TdEconomyCategory::LayerIncome,
            _ => {
                return Err(failure::err_msg(format!(
                    "unknown TD credit context={context}"
                )))
            }
        };
        let mut ledger = world.write_resource::<crate::runtime::TdEconomyLedger>();
        let mut economy = world.write_resource::<PlayerEconomy>();
        return ledger
            .apply(
                &mut economy,
                tick,
                Some(player_id),
                category,
                amount,
                context,
            )
            .map(|balance| balance.unwrap_or_default())
            .map_err(|error| failure::err_msg(format!("{context}: {error}")));
    }
    let mut economy = world
        .try_fetch_mut::<PlayerEconomy>()
        .ok_or_else(|| failure::err_msg(format!("{context}: PlayerEconomy resource is missing")))?;
    economy
        .credit_saturating(player_id, amount)
        .map_err(|error| failure::err_msg(format!("{context}: {error}")))
}

pub fn spawn_td_tower(world: &mut World, pos: Vec2<f32>, unit_id: &str) -> Option<Entity> {
    spawn_td_tower_with_owner(world, pos, unit_id, None)
}

pub fn spawn_td_tower_with_owner(
    world: &mut World,
    pos: Vec2<f32>,
    unit_id: &str,
    owner_pid: Option<u32>,
) -> Option<Entity> {
    let tpl = {
        let reg = world.read_resource::<TowerTemplateRegistry>();
        reg.get(unit_id).cloned()
    };
    let Some(tpl) = tpl else {
        log::warn!("spawn_td_tower: unknown unit_id '{}'", unit_id);
        return None;
    };

    let f32_to_fx = |v: f32| Fixed64::from_raw((v * omoba_sim::fixed::SCALE as f32) as i64);
    let tpl_hp = f32_to_fx(tpl.hp);
    let tprop = TProperty::new(tpl_hp, 0, Fixed64::from_i32(120));
    let tatk = TAttack::new(
        f32_to_fx(tpl.atk),
        f32_to_fx(tpl.asd_interval),
        f32_to_fx(tpl.range),
        f32_to_fx(tpl.bullet_speed),
    );
    let faction = Faction::new(FactionType::Player, 0);
    let vision = CircularVision::new(tpl.range + 100.0, 40.0).with_precision(120);
    let cprop = CProperty {
        hp: tpl_hp,
        mhp: tpl_hp,
        msd: Fixed64::ZERO,
        def_physic: Fixed64::ZERO,
        def_magic: Fixed64::ZERO,
    };

    let spawn_order = world.write_resource::<TowerSpawnOrderCounter>().allocate();
    let entity = world
        .create_entity()
        .with(Pos::from_xy_f32(pos.x, pos.y))
        .with(Tower::new())
        .with(spawn_order)
        .with(IsBuilding)
        .with(tprop)
        .with(cprop)
        .with(tatk)
        .with(faction)
        .with(vision)
        .with(Facing(omoba_sim::Angle::ZERO))
        .with(FacingBroadcast(None))
        .with(TurnSpeed(Fixed64::from_raw(
            (tpl.turn_speed_deg.to_radians() * 1024.0) as i64,
        )))
        .with(CollisionRadius(Fixed64::from_raw(
            (tpl.footprint * 1024.0) as i64,
        )))
        .with(ScriptUnitTag {
            unit_id: unit_id.to_string(),
        })
        .build();

    // 套用英雄知識 buff（enabled + category 有對應才套用）
    {
        let category = hero_knowledge_category_for_unit_id(unit_id);
        let gk_buffs: Vec<(String, String)> = {
            let gk = world.read_resource::<KnowledgeBonusResource>();
            log::info!(
                "[gk_spawn] unit={} category='{}' enabled={} unlocked_nodes={} category_buffs={} global_buffs={}",
                unit_id,
                category,
                gk.enabled,
                gk.unlocked_nodes.len(),
                gk.bonuses_for(category).len(),
                gk.global_bonuses().len(),
            );
            if gk.enabled && !category.is_empty() {
                gk.bonuses_for(category)
                    .iter()
                    .chain(gk.global_bonuses().iter())
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };
        if !gk_buffs.is_empty() {
            let mut buff_store = world.write_resource::<BuffStore>();
            for (buff_id, payload_str) in &gk_buffs {
                log::info!(
                    "[gk_spawn] applying buff_id='{}' payload={}",
                    buff_id,
                    payload_str
                );
                let payload: serde_json::Value = serde_json::from_str(payload_str)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                buff_store.add(entity, buff_id, Fixed64::from_raw(i64::MAX), payload);
            }
        }
    }

    if let Some(player_id) = owner_pid {
        let inserted = world
            .write_storage::<PlayerOwner>()
            .insert(entity, PlayerOwner::new(player_id));
        if let Err(e) = inserted {
            log::warn!(
                "spawn_td_tower: failed to attach PlayerOwner({}) to {:?}: {}",
                player_id,
                entity,
                e
            );
        } else {
            let source = crate::runtime::canonical_entity_id(entity);
            let tick = world.try_fetch::<Tick>().map(|tick| tick.0).unwrap_or(0);
            let _ = world
                .read_resource::<crate::runtime::ObservableFactBuffer>()
                .emit(crate::runtime::OrderedFact {
                    key: crate::runtime::FactOrderingKey {
                        tick,
                        phase: crate::runtime::FactPhase::PreStep,
                        canonical_source_order: source,
                        local_ordinal: 0,
                        fact_kind: crate::runtime::FactKind::Ownership,
                    },
                    audience: crate::runtime::FactAudience::VisibilityPolicy(
                        omb_script_abi::types::projection_policy_ids::OWNERSHIP.to_owned(),
                    ),
                    fact: crate::runtime::ObservableFact::Ownership {
                        source,
                        team: player_id,
                    },
                });
        }
    }

    world
        .write_resource::<ScriptEventQueue>()
        .push(ScriptEvent::Spawn { e: entity });
    world.write_resource::<Searcher>().tower.mark_dirty();

    Some(entity)
}

fn validate_tower_place_from_input(
    world: &World,
    tpl: &TowerTemplate,
    pos: Vec2<f32>,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    if player_economy_balance(world, "TowerPlace", owner_pid)? < tpl.cost {
        return Err(failure::err_msg(format!(
            "TowerPlace: insufficient gold pid={} unit_id='{}' cost={}",
            owner_pid, tpl.unit_id, tpl.cost
        )));
    }

    let placement_radius = tpl.placement_radius;
    {
        let regions = world.read_resource::<BlockedRegions>();
        for region in regions.0.iter() {
            if circle_hits_polygon(pos, placement_radius, &region.points) {
                return Err(failure::err_msg(format!(
                    "TowerPlace: blocked by region '{}' pid={} unit_id='{}'",
                    region.name, owner_pid, tpl.unit_id
                )));
            }
        }
    }

    const PATH_HALF_WIDTH: f32 = 64.0;
    {
        let paths = world.read_resource::<BTreeMap<String, Path>>();
        let clear = tower_path_clearance(tpl.placement_radius, PATH_HALF_WIDTH);
        let clear_sq = clear * clear;
        for (name, path) in paths.iter() {
            let cps = &path.check_points;
            for i in 0..cps.len().saturating_sub(1) {
                let a = cps[i].pos;
                let b = cps[i + 1].pos;
                let distance_sq = point_segment_dist_sq(pos, a, b);
                if distance_sq < clear_sq {
                    return Err(failure::err_msg(format!(
                        "TowerPlace: blocked by path '{}' segment={} pid={} unit_id='{}' pos=({:.1},{:.1}) distance={:.2} road_half_width={:.2} footprint={:.2} placement_radius={:.2} required_clearance={:.2}",
                        name,
                        i + 1,
                        owner_pid,
                        tpl.unit_id,
                        pos.x,
                        pos.y,
                        distance_sq.sqrt(),
                        PATH_HALF_WIDTH,
                        tpl.footprint,
                        tpl.placement_radius,
                        clear,
                    )));
                }
            }
        }
    }

    {
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        let positions = world.read_storage::<Pos>();
        let tags = world.read_storage::<ScriptUnitTag>();
        let registry = world.read_resource::<TowerTemplateRegistry>();
        for (_entity, _tower, position, tag) in
            (&entities, &towers, &positions, tags.maybe()).join()
        {
            let Some(existing_radius) = tag
                .and_then(|tag| registry.get(&tag.unit_id))
                .map(|tpl| tpl.placement_radius)
            else {
                return Err(failure::err_msg(
                    "TowerPlace: existing tower missing script-owned placement_radius metadata",
                ));
            };
            let (px, py) = position.xy_f32();
            let dx = px - pos.x;
            let dy = py - pos.y;
            let min_d = placement_radius + existing_radius;
            if dx * dx + dy * dy < min_d * min_d {
                return Err(failure::err_msg(format!(
                    "TowerPlace: overlaps tower pid={} unit_id='{}'",
                    owner_pid, tpl.unit_id
                )));
            }
        }
    }

    Ok(())
}

#[inline]
fn tower_path_clearance(placement_radius: f32, path_half_width: f32) -> f32 {
    placement_radius + path_half_width
}

pub fn handle_tower_spawn_from_input(
    world: &mut World,
    kind_id: u32,
    pos: omoba_sim::Vec2,
    owner_pid: u32,
) -> Result<Entity, failure::Error> {
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::TOWER,
        "runtime/native/game_processor/handle_tower_spawn_from_input",
    );
    let tid = omoba_template_ids::TowerId(kind_id as u16);
    let unit_id = omoba_template_ids::tower_id_str(tid);
    if unit_id.is_empty() || unit_id == "?" {
        return Err(failure::err_msg(format!(
            "TowerPlace: unknown tower_kind_id {} (pid={})",
            kind_id, owner_pid
        )));
    }

    let pos_f32 = Vec2::new(pos.x.to_f32_for_render(), pos.y.to_f32_for_render());
    let tpl = {
        let reg = world.read_resource::<TowerTemplateRegistry>();
        reg.get(unit_id)
            .cloned()
            .ok_or_else(|| failure::err_msg(format!("TowerPlace: unknown unit_id '{}'", unit_id)))?
    };
    validate_tower_place_from_input(world, &tpl, pos_f32, owner_pid)?;
    let entity =
        spawn_td_tower_with_owner(world, pos_f32, unit_id, Some(owner_pid)).ok_or_else(|| {
            failure::err_msg(format!(
                "spawn_td_tower returned None for unit_id='{}'",
                unit_id
            ))
        })?;
    debit_player_economy(world, "TowerPlace", owner_pid, tpl.cost)?;
    log::info!(
        "TowerPlace ok pid={} kind_id={} unit_id='{}' cost={} pos=({:.1},{:.1}) entity={:?}",
        owner_pid,
        kind_id,
        unit_id,
        tpl.cost,
        pos_f32.x,
        pos_f32.y,
        entity
    );
    Ok(entity)
}

pub fn drain_pending_tower_spawns(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingTowerSpawnQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_tower_spawns",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        if let Err(e) = handle_tower_spawn_from_input(world, req.kind_id, req.pos, req.owner_pid) {
            log::warn!(
                "TowerPlace failed pid={} kind_id={}: {}",
                req.owner_pid,
                req.kind_id,
                e
            );
        }
    }
    drop(drain_span);
}

pub fn handle_tower_sell_from_input(
    world: &mut World,
    tower_entity_id: u32,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::TOWER,
        "runtime/native/game_processor/handle_tower_sell_from_input",
    );
    let target_entity = {
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        (&entities, &towers)
            .join()
            .find(|(entity, _)| entity.id() == tower_entity_id)
            .map(|(entity, _)| entity)
    }
    .ok_or_else(|| {
        failure::err_msg(format!(
            "TowerSell: tower entity id={} not found / not a Tower (pid={})",
            tower_entity_id, owner_pid
        ))
    })?;

    {
        let factions = world.read_storage::<Faction>();
        match factions.get(target_entity) {
            Some(f) if f.faction_id == FactionType::Player => {}
            Some(f) => {
                return Err(failure::err_msg(format!(
                    "TowerSell: tower id={} not Player-owned (faction={:?}, pid={})",
                    tower_entity_id, f.faction_id, owner_pid
                )));
            }
            None => {
                return Err(failure::err_msg(format!(
                    "TowerSell: tower id={} has no Faction component (pid={})",
                    tower_entity_id, owner_pid
                )));
            }
        }
    }
    {
        let owners = world.read_storage::<PlayerOwner>();
        match owners.get(target_entity) {
            Some(owner) if owner.player_id == owner_pid => {}
            Some(owner) => {
                return Err(failure::err_msg(format!(
                    "TowerSell: tower id={} owner_pid={} rejected requester pid={}",
                    tower_entity_id, owner.player_id, owner_pid
                )));
            }
            None => {
                return Err(failure::err_msg(format!(
                    "TowerSell: tower id={} has no PlayerOwner (pid={})",
                    tower_entity_id, owner_pid
                )));
            }
        }
    }

    let refund = {
        let tags = world.read_storage::<ScriptUnitTag>();
        let reg = world.read_resource::<TowerTemplateRegistry>();
        let towers = world.read_storage::<Tower>();
        let ureg = world.read_resource::<TowerUpgradeRegistry>();
        let base_spend = tags
            .get(target_entity)
            .and_then(|tag| reg.get(&tag.unit_id))
            .map(|tpl| tpl.cost)
            .unwrap_or(0);
        let upgrade_spend = if let (Some(tower), Some(tag)) =
            (towers.get(target_entity), tags.get(target_entity))
        {
            let mut total = 0i32;
            for path in 0..3u8 {
                for level in 1..=tower.upgrade_levels[path as usize] {
                    if let Some(def) = ureg.get(&tag.unit_id, path, level) {
                        total = total.saturating_add(def.cost);
                    }
                }
            }
            total
        } else {
            0
        };
        world
            .try_fetch::<crate::runtime::TdEconomyRules>()
            .map(|rules| rules.sell_refund(base_spend.saturating_add(upgrade_spend)))
            .unwrap_or_else(|| {
                // Non-TD compatibility path; TD worlds always install TdEconomyRules.
                base_spend.saturating_mul(3) / 4 + upgrade_spend.saturating_mul(3) / 4
            })
    };

    credit_player_economy(world, "TowerSell", owner_pid, refund)?;

    world
        .write_resource::<BuffStore>()
        .remove_all_for(target_entity);
    world
        .write_resource::<Vec<Outcome>>()
        .push(Outcome::EntityRemoved {
            entity: target_entity,
        });

    log::info!(
        "TowerSell ok pid={} entity_id={} refund={}",
        owner_pid,
        tower_entity_id,
        refund
    );
    Ok(())
}

pub fn drain_pending_tower_sells(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingTowerSellQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_tower_sells",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        if let Err(e) = handle_tower_sell_from_input(world, req.tower_entity_id, req.owner_pid) {
            log::warn!(
                "TowerSell failed pid={} entity_id={}: {}",
                req.owner_pid,
                req.tower_entity_id,
                e
            );
        }
    }
    drop(drain_span);
}

pub fn handle_tower_upgrade_from_input(
    world: &mut World,
    tower_entity_id: u32,
    path: u8,
    _level_hint: u8,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::TOWER,
        "runtime/native/game_processor/handle_tower_upgrade_from_input",
    );
    if path >= 3 {
        return Err(failure::err_msg(format!(
            "TowerUpgrade: invalid path={} (must be 0..=2) pid={}",
            path, owner_pid
        )));
    }

    let target = {
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        let tags = world.read_storage::<ScriptUnitTag>();
        (&entities, &towers, &tags)
            .join()
            .find(|(entity, _, _)| entity.id() == tower_entity_id)
            .map(|(entity, tower, tag)| (entity, tower.upgrade_levels, tag.unit_id.clone()))
    };
    let (target_entity, levels, unit_id) = target.ok_or_else(|| {
        failure::err_msg(format!(
            "TowerUpgrade: tower id={} not found / not a Tower (pid={})",
            tower_entity_id, owner_pid
        ))
    })?;

    {
        let factions = world.read_storage::<Faction>();
        match factions.get(target_entity) {
            Some(f) if f.faction_id == FactionType::Player => {}
            Some(f) => {
                return Err(failure::err_msg(format!(
                    "TowerUpgrade: tower id={} not Player-owned (faction={:?}, pid={})",
                    tower_entity_id, f.faction_id, owner_pid
                )));
            }
            None => {
                return Err(failure::err_msg(format!(
                    "TowerUpgrade: tower id={} has no Faction component (pid={})",
                    tower_entity_id, owner_pid
                )));
            }
        }
    }
    {
        let owners = world.read_storage::<PlayerOwner>();
        match owners.get(target_entity) {
            Some(owner) if owner.player_id == owner_pid => {}
            Some(owner) => {
                return Err(failure::err_msg(format!(
                    "TowerUpgrade: tower id={} owner_pid={} rejected requester pid={}",
                    tower_entity_id, owner.player_id, owner_pid
                )));
            }
            None => {
                return Err(failure::err_msg(format!(
                    "TowerUpgrade: tower id={} has no PlayerOwner (pid={})",
                    tower_entity_id, owner_pid
                )));
            }
        }
    }

    if let Err(rej) = tower_upgrade_rules::validate_upgrade(levels, path) {
        return Err(failure::err_msg(format!(
            "TowerUpgrade: rule rejection eid={} path={} levels={:?} -> {:?} (pid={})",
            tower_entity_id, path, levels, rej, owner_pid
        )));
    }
    let next_level = levels[path as usize] + 1;

    let def = {
        let reg = world.read_resource::<TowerUpgradeRegistry>();
        reg.get(&unit_id, path, next_level).cloned()
    }
    .ok_or_else(|| {
        failure::err_msg(format!(
            "TowerUpgrade: no UpgradeDef for kind={} path={} level={} (pid={})",
            unit_id, path, next_level, owner_pid
        ))
    })?;

    if player_economy_balance(world, "TowerUpgrade", owner_pid)? < def.cost {
        return Err(failure::err_msg(format!(
            "TowerUpgrade: insufficient gold (need {}) for kind={} path={} level={} (pid={})",
            def.cost, unit_id, path, next_level, owner_pid
        )));
    }

    debit_player_economy(world, "TowerUpgrade", owner_pid, def.cost)?;

    let mut flags_to_add = Vec::new();
    let mut stat_mods = Vec::new();
    for (effect_idx, effect) in def.effects.iter().enumerate() {
        match effect {
            UpgradeEffect::BehaviorFlag { flag } => flags_to_add.push(flag.clone()),
            UpgradeEffect::StatMod { key, value, op: _ } => {
                let buff_id = format!("upgrade_{}_{}_{}", path, next_level, effect_idx);
                let key = canonical_upgrade_stat_key(key).into_owned();
                stat_mods.push((buff_id, json!({ key: *value })));
            }
        }
    }
    for (buff_id, payload) in stat_mods {
        world.write_resource::<BuffStore>().add(
            target_entity,
            &buff_id,
            omoba_sim::Fixed64::from_raw(i64::MAX),
            payload,
        );
    }

    {
        let mut towers = world.write_storage::<Tower>();
        if let Some(tower) = towers.get_mut(target_entity) {
            for flag in flags_to_add {
                if !tower.upgrade_flags.iter().any(|existing| existing == &flag) {
                    tower.upgrade_flags.push(flag);
                }
            }
            tower.upgrade_levels[path as usize] = next_level;
            if let Some(active_ability) = &def.active_ability {
                tower.active_ability = Some(TowerActiveAbilityState::ready(
                    active_ability.ability_id.clone(),
                ));
            }
        }
    }

    log::info!(
        "TowerUpgrade ok pid={} eid={} kind={} path={} level={} cost={}",
        owner_pid,
        tower_entity_id,
        unit_id,
        path,
        next_level,
        def.cost
    );
    Ok(())
}

fn canonical_upgrade_stat_key(key: &str) -> Cow<'_, str> {
    if let Some(stat_key) = StatKey::from_str_key(key) {
        return Cow::Borrowed(stat_key.as_str());
    }
    if let Some(stat_key) = omb_script_abi::stat_keys::ALL
        .iter()
        .copied()
        .find(|stat_key| format!("{:?}", stat_key) == key)
    {
        return Cow::Borrowed(stat_key.as_str());
    }
    Cow::Borrowed(key)
}

pub fn handle_tower_target_priority_from_input(
    world: &mut World,
    tower_entity_id: u32,
    priority: TowerTargetPriority,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    let target_entity = {
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        (&entities, &towers)
            .join()
            .find(|(entity, _)| entity.id() == tower_entity_id)
            .map(|(entity, _)| entity)
    }
    .ok_or_else(|| {
        failure::err_msg(format!(
            "SetTowerTargetPriority: tower id={} not found / not a Tower (pid={})",
            tower_entity_id, owner_pid
        ))
    })?;

    {
        let owners = world.read_storage::<PlayerOwner>();
        match owners.get(target_entity) {
            Some(owner) if owner.player_id == owner_pid => {}
            Some(owner) => {
                return Err(failure::err_msg(format!(
                    "SetTowerTargetPriority: tower id={} owner_pid={} rejected requester pid={}",
                    tower_entity_id, owner.player_id, owner_pid
                )));
            }
            None => {
                return Err(failure::err_msg(format!(
                    "SetTowerTargetPriority: tower id={} has no PlayerOwner (pid={})",
                    tower_entity_id, owner_pid
                )));
            }
        }
    }
    {
        let mut towers = world.write_storage::<Tower>();
        let Some(tower) = towers.get_mut(target_entity) else {
            return Err(failure::err_msg(format!(
                "SetTowerTargetPriority: tower id={} vanished before write (pid={})",
                tower_entity_id, owner_pid
            )));
        };
        tower.target_priority = priority;
    }
    log::info!(
        "SetTowerTargetPriority ok pid={} eid={} priority={}",
        owner_pid,
        tower_entity_id,
        priority.as_str()
    );
    Ok(())
}

pub fn drain_pending_tower_target_priorities(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingTowerTargetPriorityQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_tower_target_priorities",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        if let Err(e) = handle_tower_target_priority_from_input(
            world,
            req.tower_entity_id,
            req.priority,
            req.owner_pid,
        ) {
            log::warn!(
                "SetTowerTargetPriority failed pid={} eid={} priority={}: {}",
                req.owner_pid,
                req.tower_entity_id,
                req.priority.as_str(),
                e
            );
        }
    }
    drop(drain_span);
}

pub fn drain_pending_tower_upgrades(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingTowerUpgradeQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_tower_upgrades",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        if let Err(e) = handle_tower_upgrade_from_input(
            world,
            req.tower_entity_id,
            req.path,
            req.level,
            req.owner_pid,
        ) {
            log::warn!(
                "TowerUpgrade failed pid={} eid={} path={}: {}",
                req.owner_pid,
                req.tower_entity_id,
                req.path,
                e
            );
        }
    }
    drop(drain_span);
}

fn catalog_item_target(item_id: &str) -> omoba_template_ids::MobaItemTargetConst {
    omoba_template_ids::MOBA_ITEM_CATALOG
        .iter()
        .find(|item| item.id == item_id)
        .map(|item| item.target)
        .unwrap_or(omoba_template_ids::MobaItemTargetConst::SELF)
}

fn item_effect_matches_target(active: &ActiveEffect, shape: omoba_template_ids::MobaItemTargetShapeConst) -> bool {
    use omoba_template_ids::MobaItemTargetShapeConst::*;
    match (shape, active) {
        (Self_, ActiveEffect::SelfControlCleanse {} | ActiveEffect::ChannelHeal { .. } | ActiveEffect::Shield { .. } | ActiveEffect::SprintBuff { .. } | ActiveEffect::RestoreMana { .. }
            | ActiveEffect::DamageReduce { .. } | ActiveEffect::HeadshotNext { .. }) => true,
        (Point, ActiveEffect::PointNova { .. }) => true,
        (Unit, ActiveEffect::UnitHex { .. }) => true,
        _ => false,
    }
}

/// Authored floats are quantized once at the catalog boundary. Execution rejects
/// a non-positive or oversized fixed amount. It does not convert through f32
/// and it does not replace the effect with the catalog value.
fn checked_item_fixed(value: Fixed64, maximum_units: i64) -> Result<Fixed64, failure::Error> {
    let maximum = maximum_units.saturating_mul(1024);
    if value <= Fixed64::ZERO || value.raw() > maximum {
        return Err(failure::err_msg("ItemUse: invalid targeted amount"));
    }
    Ok(value)
}

fn item_life_from_parts(index: u32, generation: u32) -> Option<Entity> {
    let generation = i32::try_from(generation).ok().filter(|value| *value > 0)?;
    Some(Entity::new(index, specs::world::Generation::new(generation)))
}

fn owner_item_life_intact(world: &World, owner: Entity, owner_pid: u32) -> bool {
    let alive = world.entities().is_alive(owner);
    let owns_player = world.read_storage::<PlayerOwner>().get(owner)
        .is_some_and(|owner_component| owner_component.player_id == owner_pid);
    alive && owns_player && player_hero_entity(world, "ItemUse", owner_pid).ok() == Some(owner)
}

/// Overwrite any client owner fields with the packed authoritative identity.
/// Generation is the high 32 bits and the ECS index is the low 32 bits.
/// Generation 0 clears both fields. It is not the current hero.
pub fn overwrite_item_owner_canonical(item: &mut ItemUse, canonical_actor: u64) {
    let index = (canonical_actor & u64::from(u32::MAX)) as u32;
    let generation = (canonical_actor >> 32) as u32;
    if generation == 0 {
        item.owner_entity = None;
        item.owner_generation = None;
        return;
    }
    item.owner_entity = Some(index);
    item.owner_generation = Some(generation);
}

/// Write this world's actor index and generation. Authority generation is not copied.
pub fn write_local_item_actor(item: &mut ItemUse, actor: Entity) {
    let generation = actor.gen().id();
    if generation <= 0 {
        item.owner_entity = None;
        item.owner_generation = None;
        return;
    }
    item.owner_entity = Some(actor.id());
    item.owner_generation = Some(generation as u32);
}

/// Drop forged owner life and forged target life from a reference-free payload.
/// `target_entity` stays, because legacy non-secure commands still name an index.
pub fn clear_untrusted_item_life(input: &mut PlayerInput) {
    if let Some(ProtoAction::ItemUse(item)) = input.action.as_mut() {
        item.owner_entity = None;
        item.owner_generation = None;
        item.target_generation = None;
    }
}

/// Earliest ECS owner boundary. A missing pair stamps the submitting player's
/// current hero. A present pair is kept only when that exact entity still owns
/// the player and is still that player's current hero. Failure does not rewrite
/// the command onto whoever is current. A half pair is refused.
pub fn seal_item_use_owner(world: &World, player_id: u32, input: &mut PlayerInput) -> bool {
    let Some(ProtoAction::ItemUse(item)) = input.action.as_mut() else {
        return true;
    };
    match (item.owner_entity, item.owner_generation) {
        (None, None) => match player_hero_entity(world, "ItemUse", player_id) {
            Ok(hero) => {
                write_local_item_actor(item, hero);
                true
            }
            Err(_) => false,
        },
        (Some(index), Some(generation)) => item_life_from_parts(index, generation)
            .is_some_and(|owner| owner_item_life_intact(world, owner, player_id)),
        _ => false,
    }
}

/// Canonical id of an already sealed ItemUse owner. None drops the input.
/// This does not fall back to whichever hero the player controls now.
pub fn canonical_item_owner(world: &World, player_id: u32, item: &ItemUse) -> Option<u64> {
    let owner = item_life_from_parts(item.owner_entity?, item.owner_generation?)?;
    owner_item_life_intact(world, owner, player_id).then(|| crate::runtime::canonical_entity_id(owner))
}

/// Same pending record the input tick queues. Tests and the tick must not diverge.
pub fn pending_item_use_from_command(owner_pid: u32, item: &ItemUse) -> crate::comp::PendingItemUse {
    let target_pos = item.target_pos.as_ref().map(|point| {
        omoba_sim::Vec2::new(
            omoba_sim::Fixed64::from_raw(point.x as i64),
            omoba_sim::Fixed64::from_raw(point.y as i64),
        )
    });
    crate::comp::PendingItemUse {
        item_slot: item.item_slot,
        target_pos,
        point_explicit: item.point_explicit.unwrap_or(false),
        target_entity: item.target_entity,
        target_generation: item.target_generation,
        owner_pid,
        owner_index: item.owner_entity,
        owner_generation: item.owner_generation,
        lives_bound: false,
        owner_entity: None,
        target_life: None,
    }
}

/// Bind owner and disclosed-unit life once. A paired owner index and generation
/// was stamped at trusted acceptance; this first bind uses that pair and does
/// not search for the current hero. A missing pair is the legacy in-process
/// resolution of the current hero, once. A half pair is not a hero. A later
/// call must not replace a bound life.
pub fn bind_queued_item_lives(world: &World) {
    let mut queue = world.write_resource::<PendingItemUseQueue>();
    for action in &mut queue.requests {
        let crate::comp::PendingItemAction::Use(request) = action else { continue; };
        if request.lives_bound { continue; }
        request.owner_entity = match (request.owner_index, request.owner_generation) {
            (Some(index), Some(generation)) => item_life_from_parts(index, generation),
            (None, None) => player_hero_entity(world, "ItemUse", request.owner_pid).ok(),
            _ => None,
        };
        // Resolution may be None. The raw index and generation stay; drain reads
        // them as shape presence and must not treat this None as a missing field.
        request.target_life = match (request.target_entity, request.target_generation) {
            (Some(index), Some(generation)) => item_life_from_parts(index, generation),
            (Some(index), None) => entity_by_id(world, index),
            (None, _) => None,
        };
        request.lives_bound = true;
    }
}

fn within_item_range(origin: omoba_sim::Vec2, point: omoba_sim::Vec2, range: Fixed64) -> bool {
    let delta = point - origin;
    delta.length_squared() <= range * range
}

fn item_damage_boundary_open(world: &World, target: Entity) -> bool {
    crate::runtime::moba_damage_allowed(world, target)
        && !world.try_fetch::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>()
            .is_some_and(|targets| targets.0.contains(&target))
}

fn same_live_item_target(world: &World, entity: Entity) -> bool {
    world.entities().is_alive(entity) && entity_by_id(world, entity.id()) == Some(entity)
}

/// Same precedence as committed `entity_visible_to_team`, read from ECS components.
/// Occluder checks use the public segment test. A missing occluder resource is an empty set.
pub(crate) fn item_entity_visible_to_team(world: &World, target: Entity, viewer_team: u32) -> bool {
    let (scope_kind, owner_team, override_kind) = {
        let factions = world.read_storage::<Faction>();
        let scopes = world.read_storage::<ReplicationScope>();
        let overrides = world.read_storage::<VisibilityOverride>();
        let team = factions.get(target).map(|faction| faction.team_id.max(0) as u32).unwrap_or(0);
        let scope = scopes.get(target).copied().unwrap_or(ReplicationScope {
            kind: ReplicationScopeKind::Vision,
            owner_team: Some(team),
        });
        let override_kind = overrides.get(target).and_then(|rule| {
            (rule.team.is_none() || rule.team == Some(viewer_team)).then_some(rule.kind)
        });
        (scope.kind, scope.owner_team, override_kind)
    };
    if scope_kind == ReplicationScopeKind::ServerOnly {
        return false;
    }
    if override_kind == Some(VisibilityOverrideKind::ForceHide) {
        return false;
    }
    if scope_kind == ReplicationScopeKind::Public || override_kind == Some(VisibilityOverrideKind::ForceShow) {
        return true;
    }
    if scope_kind == ReplicationScopeKind::OwnerTeam && owner_team == Some(viewer_team) {
        return true;
    }
    let (target_pos, origins) = {
        let positions = world.read_storage::<Pos>();
        let Some(target_pos) = positions.get(target).map(|pos| pos.0) else { return false; };
        let stealth = world.read_storage::<StealthProfile>().get(target).map(|profile| profile.stealth_level).unwrap_or(0)
            .max(world.try_fetch::<BuffStore>().map_or(0, |buffs| buffs.stealth_level(target)));
        let sources = world.read_storage::<VisionSource>();
        let entities = world.entities();
        let mut origins: Vec<omoba_sim::Vec2> = (&entities, &positions, &sources).join().filter_map(|(_, pos, source)| {
            if source.team != viewer_team || source.detection_level < stealth {
                return None;
            }
            let delta = target_pos - pos.0;
            (delta.length_squared() <= source.radius * source.radius).then_some(pos.0)
        }).collect();
        // True-sight zones detect every stealth level (same rule as Wave B).
        if let Some(zones) = world.try_fetch::<super::reveal_zone::RevealZones>() {
            origins.extend(zones.sources().filter(|(_, zone)| zone.team == viewer_team
                && (target_pos - zone.center).length_squared() <= zone.radius * zone.radius).map(|(_, zone)| zone.center));
        }
        (target_pos, origins)
    };
    let occluders = world.try_fetch::<VisionOccluderSet>().map(|set| set.0.clone()).unwrap_or_default();
    let brushes = world.try_fetch::<crate::runtime::brush_vision::BrushVisionRegions>();
    let brush_revealed=override_kind==Some(VisibilityOverrideKind::RevealBrush)
        || world.try_fetch::<crate::runtime::MobaMatch>().is_some_and(|state|
            state.brush_reveal_teams(target).any(|team|team==viewer_team)
                && world.read_storage::<CProperty>().get(target).is_some_and(|p|p.hp>Fixed64::ZERO));
    origins.into_iter().any(|origin| {
        (brush_revealed || brushes.as_ref().is_none_or(|regions| crate::runtime::brush_vision::brush_allows_vision(regions, origin, target_pos)))
            && crate::runtime::comp::vision_occluder::line_of_sight(&occluders, origin, target_pos)
            == crate::runtime::comp::vision_occluder::LosResult::Clear
    })
}

fn slot_still_ready(world: &World, hero: Entity, slot_i: usize, item_id: &str) -> bool {
    world.read_storage::<Inventory>().get(hero)
        .and_then(|inventory| inventory.slots.get(slot_i))
        .and_then(|slot| slot.as_ref())
        .is_some_and(|instance| instance.item_id == item_id && instance.cooldown_remaining <= 0.0)
}

/// Point and unit items never queue, never approach, and never clear hero commands.
/// A legal point with zero victims still commits cooldown. Any refusal returns before writes.
fn item_faction_matches(rule: omoba_template_ids::MobaItemTeamConst, source: &Faction, target: &Faction) -> bool {
    use omoba_template_ids::MobaItemTeamConst;
    match rule {
        MobaItemTeamConst::Enemy => target.team_id > 0 && target.team_id != source.team_id,
        MobaItemTeamConst::Hostile => source.is_hostile_to(target),
        MobaItemTeamConst::Any => false,
    }
}

fn commit_targeted_item_use(
    world: &mut World,
    hero_entity: Entity,
    owner_pid: u32,
    slot_i: usize,
    item_id: &str,
    cooldown: f32,
    active: &ActiveEffect,
    rule: omoba_template_ids::MobaItemTargetConst,
    target_pos: Option<omoba_sim::Vec2>,
    target_life: Option<Entity>,
    point_explicit: bool,
    target_named: bool,
) -> Result<(), failure::Error> {
    use omoba_template_ids::{MobaItemOutOfRangeConst, MobaItemTargetShapeConst, MobaItemTeamConst};
    if rule.out_of_range != MobaItemOutOfRangeConst::Reject || rule.range <= Fixed64::ZERO
        || !matches!(rule.team, MobaItemTeamConst::Enemy | MobaItemTeamConst::Hostile) || !rule.requires_visible {
        return Err(failure::err_msg("ItemUse: targeted contract is not reject-visible-offensive"));
    }
    let playing = world.try_fetch::<crate::runtime::MobaMatch>()
        .is_some_and(|state| state.phase == crate::runtime::MobaMatchPhase::Playing);
    if !playing {
        return Err(failure::err_msg("ItemUse: match is not playing"));
    }
    if !owner_item_life_intact(world, hero_entity, owner_pid) {
        return Err(failure::err_msg("ItemUse: owner life changed before commit"));
    }
    if world.try_fetch::<BuffStore>().is_none() {
        return Err(failure::err_msg("ItemUse: buff store unavailable"));
    }
    let owner_hp = world.read_storage::<CProperty>().get(hero_entity).map(|property| property.hp);
    let owner_pos = world.read_storage::<Pos>().get(hero_entity).map(|pos| pos.0);
    let owner_faction = world.read_storage::<Faction>().get(hero_entity).cloned();
    let (Some(owner_hp), Some(owner_pos), Some(owner_faction)) = (owner_hp, owner_pos, owner_faction) else {
        return Err(failure::err_msg("ItemUse: owner state unavailable"));
    };
    if owner_hp <= Fixed64::ZERO {
        return Err(failure::err_msg("ItemUse: owner is dead"));
    }
    if owner_faction.team_id <= 0 {
        return Err(failure::err_msg("ItemUse: owner team is not targeted"));
    }
    let viewer_team = owner_faction.team_id as u32;
    let victims = match (rule.shape, active) {
        (MobaItemTargetShapeConst::Point, ActiveEffect::PointNova { damage, radius }) => {
            // Failed resolution is not an absent field. A named index or generation,
            // including a malformed or orphan pair, is the wrong shape for a point.
            if !point_explicit || target_pos.is_none() || target_named || target_life.is_some() {
                return Err(failure::err_msg("ItemUse: point item requires an explicit point and no entity"));
            }
            let point = target_pos.expect("explicit point checked");
            if !within_item_range(owner_pos, point, rule.range) {
                return Err(failure::err_msg("ItemUse: point is out of range"));
            }
            let damage = checked_item_fixed(*damage, 1_000_000)?;
            let radius = checked_item_fixed(*radius, 2_000)?;
            let mut found: Vec<Entity> = {
                let entities = world.entities();
                let properties = world.read_storage::<CProperty>();
                let factions = world.read_storage::<Faction>();
                let positions = world.read_storage::<Pos>();
                (&entities, &properties, &factions, &positions).join().filter_map(|(entity, property, faction, pos)| {
                    if property.hp <= Fixed64::ZERO || !item_faction_matches(rule.team, &owner_faction, faction) {
                        return None;
                    }
                    let delta = pos.0 - point;
                    (delta.length_squared() <= radius * radius).then_some(entity)
                }).collect()
            };
            found.retain(|entity| {
                item_entity_visible_to_team(world, *entity, viewer_team)
                    && item_damage_boundary_open(world, *entity)
            });
            found.sort_by_key(|entity| (entity.id(), entity.gen().id()));
            (damage, found)
        }
        (MobaItemTargetShapeConst::Unit, ActiveEffect::UnitHex { damage,healing_reduction,control }) => {
            if let Some(control)=control {checked_item_fixed(control.duration,60)?;}
            if let Some(reduction)=healing_reduction {
                if reduction.multiplier<Fixed64::ZERO || reduction.multiplier>=Fixed64::ONE {return Err(failure::err_msg("ItemUse: invalid healing multiplier"));}
                checked_item_fixed(reduction.duration,60)?;
            }
            if point_explicit || !target_named || target_life.is_none() {
                return Err(failure::err_msg("ItemUse: unit item requires a disclosed entity and no explicit point"));
            }
            let target = target_life.expect("disclosed entity checked");
            if control.is_some()&&world.read_storage::<IsBuilding>().get(target).is_some(){
                return Err(failure::err_msg("ItemUse: control requires a non-building target"));
            }
            if !same_live_item_target(world, target) {
                return Err(failure::err_msg("ItemUse: target is stale"));
            }
            let hp = world.read_storage::<CProperty>().get(target).map(|property| property.hp);
            let faction = world.read_storage::<Faction>().get(target).cloned();
            let pos = world.read_storage::<Pos>().get(target).map(|position| position.0);
            let (Some(hp), Some(faction), Some(pos)) = (hp, faction, pos) else {
                return Err(failure::err_msg("ItemUse: target is not a living enemy"));
            };
            if hp <= Fixed64::ZERO || !item_faction_matches(rule.team, &owner_faction, &faction) {
                return Err(failure::err_msg("ItemUse: target is not a living enemy"));
            }
            if !within_item_range(owner_pos, pos, rule.range) {
                return Err(failure::err_msg("ItemUse: target is out of range"));
            }
            if !item_entity_visible_to_team(world, target, viewer_team) {
                return Err(failure::err_msg("ItemUse: target is not visible"));
            }
            if !item_damage_boundary_open(world, target) {
                return Err(failure::err_msg("ItemUse: target is not attackable"));
            }
            (checked_item_fixed(*damage, 1_000_000)?, vec![target])
        }
        _ => return Err(failure::err_msg("ItemUse: effect does not match authored target contract")),
    };
    if !slot_still_ready(world, hero_entity, slot_i, item_id) {
        return Err(failure::err_msg("ItemUse: source item changed before commit"));
    }
    let (damage, mut victims) = victims;
    if matches!(rule.shape, MobaItemTargetShapeConst::Unit) {
        if victims.len() != 1 || !same_live_item_target(world, victims[0]) {
            return Err(failure::err_msg("ItemUse: target changed before commit"));
        }
    } else {
        victims.retain(|entity| same_live_item_target(world, *entity));
    }
    if !owner_item_life_intact(world, hero_entity, owner_pid) {
        return Err(failure::err_msg("ItemUse: owner life changed before commit"));
    }
    for target in victims {
        apply_checked_direct_damage(world, Some(hero_entity), target, damage);
        if let ActiveEffect::UnitHex {control:Some(control),..}=active {
            if world.read_storage::<CProperty>().get(target).is_some_and(|p|p.hp>Fixed64::ZERO) {
                world.write_resource::<BuffStore>().add(target,control.kind.as_str(),control.duration,serde_json::json!({}));
            }
        }
        if let ActiveEffect::UnitHex {healing_reduction:Some(reduction),..}=active {
            if world.read_storage::<CProperty>().get(target).is_some_and(|p|p.hp>Fixed64::ZERO) {
                world.write_resource::<BuffStore>().grant_healing_reduction(target,reduction.multiplier,reduction.duration);
            }
        }
    }
    {
        let mut inventories = world.write_storage::<Inventory>();
        if let Some(inventory) = inventories.get_mut(hero_entity) {
            if let Some(Some(instance)) = inventory.slots.get_mut(slot_i) {
                if instance.item_id == item_id {
                    instance.cooldown_remaining = cooldown;
                }
            }
        }
    }
    log::info!("ItemUse targeted ok slot={} item={} cooldown={}s", slot_i, item_id, cooldown);
    super::shop::note_shop_undo_item_committed(world, owner_pid);
    Ok(())
}

pub fn handle_item_use_from_input(
    world: &mut World,
    item_slot: u32,
    target_pos: Option<omoba_sim::Vec2>,
    target_entity: Option<u32>,
    owner_pid: u32,
    point_explicit: bool,
) -> Result<(), failure::Error> {
    let hero_entity = player_hero_entity(world, "ItemUse", owner_pid)?;
    // Direct calls resolve the current hero and the current target index.
    // Queued commands carry the owner and target lives stamped at trusted
    // acceptance. This function does not read those lives. A failed lookup
    // leaves the raw index named; it does not become a no-target point.
    let target_named = target_entity.is_some();
    let target_life = target_entity.and_then(|id| entity_by_id(world, id));
    apply_resolved_item_use(
        world, item_slot, target_pos, target_life, hero_entity, owner_pid, point_explicit, target_named,
    )
}

fn apply_resolved_item_use(
    world: &mut World,
    item_slot: u32,
    target_pos: Option<omoba_sim::Vec2>,
    target_life: Option<Entity>,
    hero_entity: Entity,
    owner_pid: u32,
    point_explicit: bool,
    target_named: bool,
) -> Result<(), failure::Error> {
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::ITEM,
        "runtime/native/game_processor/handle_item_use_from_input",
    );
    if !owner_item_life_intact(world, hero_entity, owner_pid) {
        return Err(failure::err_msg("ItemUse: owner life changed before commit"));
    }
    if !active_hero_context_allows(world, hero_entity, owner_pid) {
        return Err(failure::err_msg("ItemUse: inactive, paused or mismatched owner context"));
    }
    let slot_i = item_slot as usize;
    if slot_i >= INVENTORY_SLOTS {
        return Err(failure::err_msg(format!(
            "ItemUse: invalid slot={} (max {}) pid={}",
            slot_i, INVENTORY_SLOTS, owner_pid
        )));
    }

    let (item_cfg, can_use) = {
        let invs = world.read_storage::<Inventory>();
        let reg = world.read_resource::<ItemRegistry>();
        if let Some(inv) = invs.get(hero_entity) {
            if let Some(Some(inst)) = inv.slots.get(slot_i) {
                let cfg = reg.get(&inst.item_id);
                let ready = inst.cooldown_remaining <= 0.0;
                (cfg, ready)
            } else {
                (None, false)
            }
        } else {
            (None, false)
        }
    };
    let cfg = match item_cfg {
        Some(c) => c,
        None => {
            return Err(failure::err_msg(format!(
                "ItemUse: empty slot={} or unknown item (pid={})",
                slot_i, owner_pid
            )));
        }
    };
    if !can_use {
        return Err(failure::err_msg(format!(
            "ItemUse: slot={} on cooldown (pid={})",
            slot_i, owner_pid
        )));
    }
    let active = match &cfg.active {
        Some(a) => a.clone(),
        None => {
            return Err(failure::err_msg(format!(
                "ItemUse: slot={} item has no active effect (pid={})",
                slot_i, owner_pid
            )));
        }
    };

    // Validate the complete supported effect before changing cooldown or commands.
    if !cfg.cooldown.is_finite() || cfg.cooldown < 0.0 || cfg.cooldown > 3600.0 {
        return Err(failure::err_msg("ItemUse: invalid cooldown"));
    }
    let catalog_target = catalog_item_target(&cfg.id);
    if !item_effect_matches_target(&active, catalog_target.shape) {
        return Err(failure::err_msg(format!(
            "ItemUse: slot={} effect does not match authored target contract (pid={})",
            slot_i, owner_pid
        )));
    }
    if catalog_target.shape != omoba_template_ids::MobaItemTargetShapeConst::Self_ {
        return commit_targeted_item_use(
            world, hero_entity, owner_pid, slot_i, &cfg.id, cfg.cooldown, &active, catalog_target,
            target_pos, target_life, point_explicit, target_named,
        );
    }
    // Self items ignore attached point, entity, generation, and explicit-origin bytes.
    let _ = (target_pos, target_life, point_explicit, target_named);
    let mut prepared_mana = None;
    match &active {
        ActiveEffect::ChannelHeal {amount,duration,interval} => {
            let spec=super::recovery_channel::Spec {amount:*amount,duration:*duration,interval:*interval};
            if !super::recovery_channel::can_start(world,hero_entity,owner_pid,spec) {
                return Err(failure::err_msg("ItemUse: recovery channel unavailable or invalid"));
            }
        }
        ActiveEffect::RestoreMana { amount } => {
            let amount = crate::runtime::ability_runtime::checked_mana_cost(*amount, Fixed64::ONE)
                .map_err(|_| failure::err_msg("ItemUse: invalid mana restore amount"))?;
            if amount <= Fixed64::ZERO {
                return Err(failure::err_msg("ItemUse: mana restore amount must be positive"));
            }
            let heroes = world.read_storage::<Hero>();
            let hero = heroes.get(hero_entity).ok_or_else(|| failure::err_msg("ItemUse: hero unavailable"))?;
            let mut pool = hero.mana_pool.clone()
                .ok_or_else(|| failure::err_msg("ItemUse: managed mana pool unavailable"))?;
            if world.try_fetch::<crate::runtime::MobaMatch>().is_some_and(|state| state.config.mana_enabled) {
                let base = hero.moba_mana_capacity()
                    .map_err(|_| failure::err_msg("ItemUse: invalid base mana capacity"))?;
                let buffs = world.try_fetch::<BuffStore>()
                    .ok_or_else(|| failure::err_msg("ItemUse: mana modifiers unavailable"))?;
                let maximum = crate::runtime::UnitStats::from_refs(&buffs, false)
                    .checked_mana_capacity(base, hero_entity)
                    .ok_or_else(|| failure::err_msg("ItemUse: invalid mana capacity modifiers"))?;
                pool.set_maximum(maximum)
                    .map_err(|_| failure::err_msg("ItemUse: invalid mana capacity"))?;
            }
            let restored = pool.restore(amount)
                .map_err(|_| failure::err_msg("ItemUse: invalid mana restore"))?;
            if restored > Fixed64::ZERO && world.try_fetch::<ScriptEventQueue>().is_none() {
                return Err(failure::err_msg("ItemUse: mana event queue unavailable"));
            }
            prepared_mana = Some((pool, restored));
        }
        ActiveEffect::SprintBuff { ms_bonus, duration } => {
            if !ms_bonus.is_finite() || *ms_bonus < 1.0 / 1024.0 || *ms_bonus > 10_000.0
                || !duration.is_finite() || *duration < 1.0 / 1024.0 || *duration > 60.0 {
                return Err(failure::err_msg("ItemUse: invalid sprint bonus or duration"));
            }
            if world.read_storage::<CProperty>().get(hero_entity).is_none()
                || world.try_fetch::<BuffStore>().is_none() {
                return Err(failure::err_msg("ItemUse: sprint stats or buff store unavailable"));
            }
        }
        ActiveEffect::DamageReduce { percent, duration } => {
            if !percent.is_finite() || !(1.0 / 1024.0..=1.0).contains(percent)
                || !duration.is_finite() || !(1.0 / 1024.0..=60.0).contains(duration) {
                return Err(failure::err_msg("ItemUse: invalid damage reduction or duration"));
            }
            if world.read_storage::<CProperty>().get(hero_entity).is_none()
                || world.try_fetch::<BuffStore>().is_none() {
                return Err(failure::err_msg("ItemUse: reduction stats or buff store unavailable"));
            }
        }
        ActiveEffect::HeadshotNext { bonus_damage } => {
            let amount = crate::runtime::ability_runtime::checked_mana_cost(*bonus_damage, Fixed64::ONE)
                .map_err(|_| failure::err_msg("ItemUse: invalid next attack bonus"))?;
            if amount <= Fixed64::ZERO || world.read_storage::<TAttack>().get(hero_entity).is_none()
                || world.try_fetch::<BuffStore>().is_none() {
                return Err(failure::err_msg("ItemUse: next attack bonus or attack resources unavailable"));
            }
        }
        ActiveEffect::Shield { amount, duration } => {
            let amount = crate::runtime::ability_runtime::checked_mana_cost(*amount, Fixed64::ONE)
                .map_err(|_| failure::err_msg("ItemUse: invalid shield amount"))?;
            if amount <= Fixed64::ZERO || !duration.is_finite()
                || !(1.0 / 1024.0..=60.0).contains(duration)
                || world.read_storage::<CProperty>().get(hero_entity).is_none()
                || world.try_fetch::<BuffStore>().is_none() {
                return Err(failure::err_msg("ItemUse: invalid shield duration or resources"));
            }
        }
        ActiveEffect::SelfControlCleanse {} => {
            if world.try_fetch::<BuffStore>().is_none() {return Err(failure::err_msg("ItemUse: control store unavailable"));}
        }
        ActiveEffect::PointNova { .. } | ActiveEffect::UnitHex { .. } => {
            return Err(failure::err_msg("ItemUse: targeted effect reached the self transaction"));
        }
    }

    match &active {
        ActiveEffect::SelfControlCleanse {} => {world.write_resource::<BuffStore>().cleanse_controls(hero_entity);}
        ActiveEffect::ChannelHeal {amount,duration,interval} => {
            let started=begin_recovery_channel(world,hero_entity,owner_pid,super::recovery_channel::Spec {amount:*amount,duration:*duration,interval:*interval});
            debug_assert!(started,"item recovery channel was preflighted");
        }
        ActiveEffect::HeadshotNext { bonus_damage } => {
            let amount = crate::runtime::ability_runtime::checked_mana_cost(*bonus_damage, Fixed64::ONE)
                .expect("next attack bonus was preflighted");
            let armed = world.write_resource::<BuffStore>().arm_next_attack_bonus(hero_entity, amount);
            debug_assert!(armed);
        }
        ActiveEffect::DamageReduce { percent, duration } => {
            let applied = world.write_resource::<BuffStore>().grant_item_timed_modifier(
                hero_entity, &cfg.id, crate::runtime::ability_runtime::ItemTimedModifier::DamageReduction,
                Fixed64::from_raw((f64::from(*percent) * 1024.0).round() as i64),
                Fixed64::from_raw((*duration * 1024.0) as i64));
            debug_assert!(applied, "item reduction was preflighted");
        }
        ActiveEffect::RestoreMana { .. } => {
            let (pool, restored) = prepared_mana.expect("mana effect was preflighted");
            world.write_storage::<Hero>().get_mut(hero_entity)
                .expect("preflighted hero remains live within item transaction").mana_pool = Some(pool);
            if restored > Fixed64::ZERO {
                world.write_resource::<ScriptEventQueue>()
                    .push(ScriptEvent::ManaGained { e: hero_entity, amount: restored });
            }
        }
        ActiveEffect::SprintBuff { ms_bonus, duration } => {
            // A stable item source refreshes instead of permanently modifying base MSD.
            // Different sprint sources share the existing strongest-family aggregation.
            let applied = world.write_resource::<BuffStore>().grant_item_timed_modifier(
                hero_entity, &cfg.id, crate::runtime::ability_runtime::ItemTimedModifier::Sprint,
                Fixed64::from_raw((*ms_bonus * 1024.0) as i64),
                Fixed64::from_raw((*duration * 1024.0) as i64));
            debug_assert!(applied, "item sprint was preflighted");
        }
        ActiveEffect::Shield { amount, duration } => {
            let amount = crate::runtime::ability_runtime::checked_mana_cost(*amount, Fixed64::ONE)
                .expect("shield amount was preflighted");
            let granted = world.write_resource::<BuffStore>().grant_damage_shield(
                hero_entity, amount, Fixed64::from_raw((*duration * 1024.0) as i64));
            debug_assert!(granted);
        }
        ActiveEffect::PointNova { .. } | ActiveEffect::UnitHex { .. } => {
            return Err(failure::err_msg("ItemUse: targeted effect reached the self transaction"));
        }
    }

    {
        let mut invs = world.write_storage::<Inventory>();
        if let Some(inv) = invs.get_mut(hero_entity) {
            if let Some(Some(inst)) = inv.slots.get_mut(slot_i) {
                inst.cooldown_remaining = cfg.cooldown;
            }
        }
    }
    // Instant defensive cleanse must not replace an existing movement/attack order.
    if !matches!(active, ActiveEffect::SelfControlCleanse {}) {clear_hero_command_queue(world, hero_entity);}

    log::info!(
        "ItemUse ok pid={} slot={} item={} cooldown={}s",
        owner_pid,
        slot_i,
        cfg.id,
        cfg.cooldown
    );
    super::shop::note_shop_undo_item_committed(world, owner_pid);
    Ok(())
}

pub fn drain_pending_item_uses(world: &mut World) {
    bind_queued_item_lives(world);
    let drained = {
        let mut q = world.write_resource::<PendingItemUseQueue>();
        std::mem::take(&mut q.requests)
    };
    let drain_span = tracing::trace_span!(
        "omoba_core::runtime::drain_pending_item_uses",
        perfetto = true,
        request_count = drained.len(),
    )
    .entered();
    for req in drained {
        let req = match req {
            crate::comp::PendingItemAction::Use(req) => req,
            crate::comp::PendingItemAction::Shop { owner_pid, command } => {
                let result = crate::runtime::shop::transact_moba_shop(world, owner_pid, command.clone());
                if let Err(error) = &result {
                    log::warn!("Shop input rejected pid={}: {:?}", owner_pid, error);
                }
                world.write_resource::<PendingItemUseQueue>().settlements.push(
                    crate::runtime::shop::ShopSettlement { player_id: owner_pid, command, result }
                );
                continue;
            }
            crate::comp::PendingItemAction::Undo { owner_pid, request } => {
                let result = if crate::runtime::shop_undo_wire::undo_hash_contract_active(world) {
                    crate::runtime::shop::undo_moba_shop(world, owner_pid, request)
                } else {
                    Err(crate::runtime::shop::ShopError::MatchUnavailable)
                };
                if let Err(error) = &result {
                    log::warn!("Shop undo rejected pid={}: {:?}", owner_pid, error);
                }
                let (gold, catalogs, cooldowns) = crate::runtime::shop::projected_owner_wallet(world, owner_pid);
                world.write_resource::<PendingItemUseQueue>().undo_settlements.push(
                    crate::runtime::shop::ShopUndoSettlement { player_id: owner_pid, request, result, gold, catalogs, cooldowns }
                );
                continue;
            }
        };
        let Some(owner) = req.owner_entity else {
            log::warn!("ItemUse failed pid={} slot={}: owner life is unavailable", req.owner_pid, req.item_slot);
            continue;
        };
        if let Err(e) = apply_resolved_item_use(
            world,
            req.item_slot,
            req.target_pos,
            req.target_life,
            owner,
            req.owner_pid,
            req.point_explicit,
            req.target_entity.is_some() || req.target_generation.is_some(),
        ) {
            log::warn!(
                "ItemUse failed pid={} slot={}: {}",
                req.owner_pid,
                req.item_slot,
                e
            );
        }
    }
    drop(drain_span);
}

fn outcome_kind(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::DamageWithLeech {..} => "DamageWithLeech",
        Outcome::DamageWithCritical {..} => "DamageWithCritical",
        Outcome::DamageZonePulse {..} => "DamageZonePulse",
        Outcome::IndependentDamageZonePulse {..} => "IndependentDamageZonePulse",
        Outcome::AbilityProjectileStep {..} => "AbilityProjectileStep",
        Outcome::RawDamage {..} => "RawDamage",
        Outcome::ScriptCastTransaction { .. } => "ScriptCastTransaction",
        Outcome::ScriptCastPreAdmitted { .. } => "ScriptCastPreAdmitted",
        Outcome::ScriptCastCommitted { .. } => "ScriptCastCommitted",
        Outcome::ProjectileContext { .. } => "ProjectileContext",
        Outcome::Damage { .. } => "Damage",
        Outcome::ProjectileDamage { .. } => "ProjectileDamage",
        Outcome::ProjectileHit { .. } => "ProjectileHit",
        Outcome::ProjectileLine2 { .. } => "ProjectileLine2",
        Outcome::Death { .. } => "Death",
        Outcome::Creep { .. } => "Creep",
        Outcome::CreepStop { .. } => "CreepStop",
        Outcome::CreepWalk { .. } => "CreepWalk",
        Outcome::CreepUpdate { .. } => "CreepUpdate",
        Outcome::Tower { .. } => "Tower",
        Outcome::Heal { .. } => "Heal",
        Outcome::UpdateAttack { .. } => "UpdateAttack",
        Outcome::GainExperience { .. } => "GainExperience",
        Outcome::GainGold { .. } => "GainGold",
        Outcome::SpawnUnit { .. } => "SpawnUnit",
        Outcome::SpawnSummoned { .. } => "SpawnSummoned",
        Outcome::CreepLeaked { .. } => "CreepLeaked",
        Outcome::AddBuff { .. } => "AddBuff",
        Outcome::Explosion { .. } => "Explosion",
        Outcome::ProjectileDirectional { .. } => "ProjectileDirectional",
        Outcome::AttackPhaseCue { .. } => "AttackPhaseCue",
        Outcome::ScriptSetPos { .. } => "ScriptSetPos",
        Outcome::ScriptSetFacing { .. } => "ScriptSetFacing",
        Outcome::ScriptSetAsdCount { .. } => "ScriptSetAsdCount",
        Outcome::ScriptSetTowerAtk { .. } => "ScriptSetTowerAtk",
        Outcome::ScriptSetTowerRange { .. } => "ScriptSetTowerRange",
        Outcome::ScriptSetAsdInterval { .. } => "ScriptSetAsdInterval",
        Outcome::ScriptSetTowerInternalCooldown { .. } => "ScriptSetTowerInternalCooldown",
        Outcome::ScriptDirectDamage { .. } => "ScriptDirectDamage",
        Outcome::ScriptHeal { .. } => "ScriptHeal",
        Outcome::ScriptRecoveryChannel {..}=>"ScriptRecoveryChannel",
        Outcome::ScriptAbilityChannel {..}=>"ScriptAbilityChannel",
        Outcome::ScriptRemoveBuff { .. } => "ScriptRemoveBuff",
        Outcome::ScriptProjectile { .. } => "ScriptProjectile",
        Outcome::ScriptTowerFireFx { .. } => "ScriptTowerFireFx",
        Outcome::ScriptAttackPhaseCue { .. } => "ScriptAttackPhaseCue",
        Outcome::ScriptStartCooldown { .. } => "ScriptStartCooldown",
        Outcome::ScriptConsumeCharge { .. } => "ScriptConsumeCharge",
        Outcome::ScriptSetMana { .. } => "ScriptSetMana",
        Outcome::ScriptManaCommit { .. } => "ScriptManaCommit",
        Outcome::ScriptManaGained { .. } => "ScriptManaGained",
        Outcome::ScriptManaSpent { .. } => "ScriptManaSpent",
        Outcome::EntityRemoved { .. } => "EntityRemoved",
    }
}

fn game_lives_event(lives: i32) -> RuntimeEvent {
    RuntimeEvent::new("td/all/res", "game", "lives", json!({ "lives": lives }))
        .with_broadcast(RuntimeBroadcast::All)
}

fn game_end_event(_winner: &str, data: serde_json::Value) -> RuntimeEvent {
    RuntimeEvent::new("td/all/res", "game", "end", data).with_broadcast(RuntimeBroadcast::All)
}

#[cfg(test)]
#[path = "summon_settlement_tests.rs"]
mod summon_settlement_tests;

#[path = "cast_settlement.rs"]
mod cast_settlement;
#[path = "raw_damage.rs"]
mod raw_damage;
#[path = "zone_pulse_settlement.rs"]
mod zone_pulse_settlement;

pub fn process_outcomes(
    world: &mut World,
    events: &mut impl RuntimeEventSink,
) -> Result<(), failure::Error> {
    prune_projectile_origins(world);
    let mut remove_uids = Vec::new();
    let mut next_outcomes = Vec::new();
    let mut committed_vitals = BTreeMap::new();
    let mut committed_cooldowns = BTreeMap::new();
    let mut committed_charges = BTreeMap::new();

    let outcomes = {
        let mut outcomes = world.write_resource::<Vec<Outcome>>();
        let mut raw = Vec::new();
        raw.append(&mut outcomes);
        merge_damage_outcomes(raw)
    };
    let outcomes_span = tracing::trace_span!(
        "omoba_core::runtime::process_outcomes",
        perfetto = true,
        outcome_count = outcomes.len(),
    )
    .entered();

    let projection_tick = world.try_fetch::<Tick>().map(|tick| tick.0).unwrap_or(0);
    fn pre_admit(world:&World,remove_uids:&[Entity],transaction:Outcome) -> Outcome {
        let mut transaction = transaction;
        let admitted = match &mut transaction {
            Outcome::ScriptCastTransaction {caster,skill_id,rank,faction,player_owner,expected_mana,outcomes,cues,..} => {
                let admitted = !remove_uids.contains(caster)
                    && cast_settlement::admit(world,*caster,skill_id,*rank,faction.as_ref(),*player_owner,expected_mana.as_ref(),outcomes,cues.len())
                    // Summon placement is judged on the same batch-start world as mana.
                    && cast_settlement::plan_summons(world,outcomes).is_some();
                if admitted {cast_settlement::apply_caster_mana(world,*caster,outcomes);}
                admitted
            }
            _ => return transaction,
        };
        Outcome::ScriptCastPreAdmitted {transaction: Box::new(transaction), admitted}
    }
    // Admit every managed cast of this batch against the batch-start world,
    // in order, before any effect settles. A hidden same-tick effect of
    // another player (mana drain, control, lethal damage) must not flip
    // admission: the casting team's replica cannot see it and would diverge.
    // Only the admitted cast's own mana commit applies now, so the same
    // caster's next cast in this batch admits against its reserved ledger.
    let mut pending: std::collections::VecDeque<_> = outcomes.into_iter().map(|outcome|pre_admit(world,&remove_uids,outcome)).collect();
    let mut next_ordinal = 0usize;
    while let Some(outcome) = pending.pop_front() {
        let outcome_ordinal = next_ordinal;
        next_ordinal += 1;
        let (outcome, projectile_origin) = match outcome {
            Outcome::ProjectileContext {origin, outcome} => {
                let source = match &*outcome {
                    Outcome::RawDamage {source,..} | Outcome::Damage {source,..} | Outcome::ProjectileHit {source,..} => Some(*source),
                    Outcome::ScriptProjectile {owner,..} => Some(*owner),
                    _ => None,
                };
                if source != Some(origin.source) {continue;}
                match *outcome {
                    Outcome::RawDamage {pos,phys,magi,real,source,target,damage_profile,..} =>
                        (Outcome::RawDamage {pos,phys,magi,real,source,target,damage_profile,origin:Some(origin)},None),
                    Outcome::Damage {pos,phys,magi,real,source,target,damage_profile,predeclared} =>
                        (Outcome::ProjectileDamage {origin,pos,phys,magi,real,source,target,damage_profile,predeclared},None),
                    other => (other,Some(origin)),
                }
            }
            other => (other,None),
        };
        // Raw packets cannot be inferred from predeclared or a damage profile.
        // Convert before facts/vitals tracking and never feed settled packets back here.
        let outcome=match outcome {
            Outcome::IndependentDamageZonePulse {source,key,lease}=>{
                for child in zone_pulse_settlement::expand_independent(world,source,&key,lease,&remove_uids).into_iter().rev(){pending.push_front(child);}
                continue;
            },
            Outcome::AbilityProjectileStep {projectile,from,to,arrival,terminal}=>{
                for child in super::ability_projectile::settle_step(world,projectile,from,to,arrival,terminal,&remove_uids).into_iter().rev(){pending.push_front(child);}
                continue;
            },
            // Launch envelope: never stored as a buff, never published as AddBuff.
            Outcome::AddBuff {target,buff_id,duration,payload}
                if payload.get(super::ability_projectile::KEY).is_some() || buff_id==super::ability_projectile::BUFF=>{
                if !remove_uids.contains(&target) {
                    if let Some(projectile)=super::ability_projectile::launch(world,target,&buff_id,duration,&payload) {
                        super::ability_projectile::emit_launch(world,projectile);
                    }
                }
                continue;
            },
            Outcome::AddBuff {target,buff_id,duration,payload} if payload.get(super::ability_runtime::damage_zone::KEY).is_some()=>{
                if !zone_pulse_settlement::admit_zone(world,target,&buff_id,duration,&payload){continue;}
                Outcome::AddBuff {target,buff_id,duration,payload}
            },
            Outcome::DamageZonePulse {source,payload}=>{
                for child in zone_pulse_settlement::expand(world,source,&payload,&remove_uids).into_iter().rev() {
                    pending.push_front(child);
                }
                continue;
            },
            Outcome::RawDamage {pos,phys,magi,real,source,target,damage_profile,origin}=>{
                if remove_uids.contains(&target) {continue;}
                if origin.as_ref().is_some_and(|origin|origin.source!=source) {continue;}
                let Some(damage)=raw_damage::settle(world,target,phys,magi,real,damage_profile) else {continue;};
                match origin {
                    Some(origin)=>Outcome::ProjectileDamage {origin,pos,phys:damage.physical,magi:damage.magical,real:damage.pure,
                        source,target,damage_profile,predeclared:false},
                    None=>Outcome::Damage {pos,phys:damage.physical,magi:damage.magical,real:damage.pure,
                        source,target,damage_profile,predeclared:false},
                }
            },
            other=>other,
        };
        let (outcome,critical)=match outcome {
            Outcome::DamageWithCritical{damage}=>{if !matches!(&*damage,Outcome::Damage{..}|Outcome::DamageWithLeech{..}){continue;}(*damage,true)},other=>(other,false),
        };
        let (outcome,mut leech)=match outcome {
            Outcome::DamageWithLeech {damage,physical_leech,magical_leech}=>{
                if !matches!(&*damage,Outcome::Damage {..}) {continue;}
                let leech=raw_damage::LeechCommit::capture(world,&damage,physical_leech,magical_leech,&remove_uids);
                (*damage,leech)
            },
            other=>(other,None),
        };
        let critical=match &outcome {
            Outcome::Damage{target,phys,..} if critical=>raw_damage::CriticalCommit::capture(world,*target,*phys,&remove_uids),
            Outcome::ProjectileDamage{origin,source,target,phys,..} if origin.source==*source && !origin.attack_missed && origin.attack_critical && (origin.normal_attack||origin.script_attack)=>raw_damage::CriticalCommit::capture(world,*target,*phys,&remove_uids),_=>None,
        };
        if world.try_fetch::<crate::runtime::MobaMatch>().is_some() {
            match &outcome {
                Outcome::Damage { target, .. } | Outcome::ProjectileDamage { target, .. } | Outcome::Heal { target, .. }
                | Outcome::ScriptDirectDamage { target, .. } | Outcome::ScriptHeal { target, .. } => {
                    committed_vitals.insert(crate::runtime::canonical_entity_id(*target), *target);
                }
                Outcome::AddBuff { target, buff_id, duration, payload } if buff_id.as_str()==super::ally_area_restore::BUFF => {
                    if let Some(spec)=super::ally_area_restore::admitted(world,*target,buff_id,*duration,payload,None) {
                        for (ally,_) in super::ally_area_restore::heals(world,&spec) {
                            committed_vitals.insert(crate::runtime::canonical_entity_id(ally), ally);
                        }
                    }
                }
                Outcome::ScriptStartCooldown { entity, ability_id, .. } => {
                    if let Some(slot) = world.read_storage::<Hero>().get(*entity)
                        .and_then(|hero| hero.abilities.iter().position(|id| id == ability_id)) {
                        committed_cooldowns.insert((crate::runtime::canonical_entity_id(*entity), slot as u32), (*entity, ability_id.clone()));
                    }
                    for (slot, id) in world.read_storage::<Hero>().get(*entity)
                        .map(|hero| super::cooldown_refund::refunded_slots(hero, ability_id)).unwrap_or_default() {
                        committed_cooldowns.insert((crate::runtime::canonical_entity_id(*entity), slot as u32), (*entity, id));
                    }
                }
                Outcome::ScriptConsumeCharge { entity, ability_id } => {
                    if let Some(slot) = world.read_storage::<Hero>().get(*entity)
                        .and_then(|hero| hero.abilities.iter().position(|id| id == ability_id)) {
                        committed_charges.insert((crate::runtime::canonical_entity_id(*entity), slot as u32), (*entity, ability_id.clone()));
                    }
                    for (slot, id) in world.read_storage::<Hero>().get(*entity)
                        .map(|hero| super::cooldown_refund::refunded_slots(hero, ability_id)).unwrap_or_default() {
                        committed_cooldowns.insert((crate::runtime::canonical_entity_id(*entity), slot as u32), (*entity, id));
                    }
                }
                _ => {}
            }
        }
        // Summon intent is not a committed spawn: settlement can still reject
        // owner life/ownership or legal placement. Other outcome facts retain
        // their existing ordering contract.
        let mut committed_fact = observable_fact_from_outcome(&outcome, projection_tick, outcome_ordinal as u32);
        if let Some(fact) = (!matches!(&outcome, Outcome::SpawnSummoned {..}))
            .then(|| committed_fact.take()).flatten() {
            let _ = world
                .read_resource::<crate::runtime::ObservableFactBuffer>()
                .emit(fact);
        }
        let kind = outcome_kind(&outcome);
        match outcome {
            Outcome::DamageWithLeech {..}|Outcome::DamageWithCritical {..}=>unreachable!("combat envelope opened before serial settlement"),
            Outcome::DamageZonePulse {..}|Outcome::IndependentDamageZonePulse {..}=>unreachable!("zone expanded before serial settlement"),
            Outcome::AbilityProjectileStep {..}=>unreachable!("ability projectile step expanded before serial settlement"),
            Outcome::RawDamage {..}=>unreachable!("raw packet converted before settlement"),
            // A transaction produced mid-batch (not seen by pre-admission) admits here.
            transaction @ Outcome::ScriptCastTransaction {..} => pending.push_front(pre_admit(world,&remove_uids,transaction)),
            Outcome::ScriptCastPreAdmitted {transaction, admitted} => {
                let Outcome::ScriptCastTransaction {caster,skill_id,mut outcomes,cues,aggression_target,..} = *transaction
                    else {unreachable!("pre-admission wraps cast transactions only")};
                let planned = admitted.then(||cast_settlement::plan_summons(world,&outcomes)).flatten();
                if let Some(planned) = planned {
                    for spawn in planned {spawn.commit(world,projection_tick,&mut next_ordinal)?;}
                    outcomes.retain(|outcome|!matches!(outcome,Outcome::SpawnSummoned {..}));
                    if let Some(target)=aggression_target {super::moba_match::record_moba_champion_aggression(world,caster,target,true);}
                    pending.push_front(Outcome::ScriptCastCommitted {caster,cues});
                    for effect in outcomes.into_iter().rev() {pending.push_front(effect);}
                } else {
                    // Late serial admission: the owner's input is rejected, never silently scheduled.
                    // Only an authoritative world can judge it (host lifecycles need MobaMatch).
                    let authoritative=world.try_fetch::<super::MobaMatch>().is_some();
                    if let Some(mut journal)=world.try_fetch_mut::<super::command_outcome::OwnerCommandOutcomes>() {
                        journal.reject_cast_if(caster,&skill_id,super::command_outcome::CAST_REJECTED,authoritative);
                    }
                    cast_settlement::reject(world,caster,&outcomes);
                }
            }
            Outcome::ScriptCastCommitted {caster,cues} => {cast_settlement::publish(world,caster,cues);}
            Outcome::ProjectileContext {..} => unreachable!("nested projectile contexts are rejected"),
            Outcome::Death { ent, .. } => {
                if remove_uids.contains(&ent) || !world.entities().is_alive(ent) {
                    continue;
                }
                remove_uids.push(ent);
                handle_death(world, &mut next_outcomes, events, ent)?;
            }
            Outcome::ProjectileLine2 {
                pos,
                source,
                target,
            } => handle_projectile(world, pos, source, target)?,
            Outcome::ProjectileDirectional {
                pos,
                source,
                end_pos,
            } => handle_projectile_directional(world, pos, source, end_pos)?,
            Outcome::Creep { cd } => handle_creep_spawn(world, cd)?,
            Outcome::Tower { pos, td } => handle_tower_spawn(world, pos, td)?,
            Outcome::CreepStop { source, target } => handle_creep_stop(world, source, target)?,
            Outcome::CreepWalk { target } => handle_creep_walk(world, target)?,
            Outcome::CreepUpdate {
                entity,
                pos,
                status,
                pidx,
                path_remaining_distance,
                facing,
                facing_broadcast,
            } => handle_creep_update(
                world,
                entity,
                pos,
                status,
                pidx,
                path_remaining_distance,
                facing,
                facing_broadcast,
            )?,
            Outcome::Damage {
                pos,
                phys,
                magi,
                real,
                source,
                target,
                damage_profile,
                predeclared,
            } => handle_damage(
                world,
                &mut next_outcomes,
                pos,
                phys,
                magi,
                real,
                source,
                target,
                damage_profile,
                predeclared,
            )?,
            Outcome::ProjectileDamage { origin, pos, phys, magi, real, source, target,
                damage_profile, predeclared } => {
                if origin.source==source && (origin.normal_attack||origin.script_attack)
                    && !origin.attack_missed && origin.attack_lifesteal>Fixed64::ZERO
                    && raw_damage::ordinary_target(world,source,target) {
                    let damage=Outcome::Damage {pos,phys,magi,real,source,target,damage_profile,predeclared};
                    leech=raw_damage::LeechCommit::capture(world,&damage,
                        origin.attack_lifesteal.clamp(Fixed64::ZERO,Fixed64::ONE),Fixed64::ZERO,&remove_uids);
                }
                world.insert(ProjectileImpactOrigin(origin));
                let result = handle_damage(world, &mut next_outcomes, pos, phys, magi, real,
                    source, target, damage_profile, predeclared);
                world.remove::<ProjectileImpactOrigin>();
                result?;
            }
            Outcome::ProjectileHit {
                source,
                target,
                kind_id,
                generation,
            } => world
                .write_resource::<ScriptEventQueue>()
                .push(ScriptEvent::ProjectileHit {
                    attacker: source,
                    victim: target,
                    kind_id,
                    generation,
                    // Attribution survives callbacks; the primary's shared
                    // leech entitlement does not apply to authored follow-ups.
                    origin: projectile_origin.map(|mut origin| {
                        origin.attack_lifesteal=Fixed64::ZERO;origin.attack_critical=false;origin
                    }),
                }),
            Outcome::Heal { target, amount, .. } => handle_heal(world, target, amount)?,
            Outcome::UpdateAttack {
                target,
                asd_count,
                cooldown_reset,
            } => handle_attack_update(world, target, asd_count, cooldown_reset)?,
            Outcome::GainExperience { target, amount } => {
                handle_experience_gain(world, target, amount as u32)?
            }
            Outcome::GainGold { target, amount } => handle_gold_gain(world, target, amount)?,
            Outcome::CreepLeaked { ent } => {
                remove_uids.push(ent);
                handle_creep_leaked(world, events, ent)?;
            }
            Outcome::AddBuff {
                target,
                buff_id,
                duration,
                payload,
            } => handle_add_buff(world, target, buff_id, duration, payload)?,
            Outcome::Explosion {
                pos,
                radius,
                duration,
            } => handle_explosion(world, pos, radius, duration),
            Outcome::AttackPhaseCue {
                entity,
                attack_seq,
                is_critical,
                target,
                target_pos,
                windup_ms,
                backswing_ms,
                dir_rad,
            } => handle_attack_phase_cue(
                world,
                entity,
                attack_seq,
                is_critical,
                target,
                target_pos,
                windup_ms,
                backswing_ms,
                dir_rad,
            ),
            Outcome::ScriptSetPos { entity, pos } => handle_script_set_pos(world, entity, pos),
            Outcome::ScriptSetFacing { entity, facing } => {
                handle_script_set_facing(world, entity, facing)
            }
            Outcome::ScriptSetAsdCount { entity, asd_count } => {
                handle_script_set_asd_count(world, entity, asd_count)
            }
            Outcome::ScriptSetTowerAtk { entity, value } => {
                handle_script_set_tower_atk(world, entity, value)
            }
            Outcome::ScriptSetTowerRange { entity, value } => {
                handle_script_set_tower_range(world, entity, value)
            }
            Outcome::ScriptSetAsdInterval { entity, value } => {
                handle_script_set_asd_interval(world, entity, value)
            }
            Outcome::ScriptSetTowerInternalCooldown { entity, duration } => {
                if let Some(tower) = world.write_storage::<Tower>().get_mut(entity) {
                    tower.ultimate_cooldown = duration.max(Fixed64::ZERO);
                }
            }
            Outcome::ScriptDirectDamage { target, amount } => {
                handle_script_direct_damage(world, target, amount)
            }
            Outcome::ScriptHeal { target, amount } => handle_script_heal(world, target, amount),
            Outcome::ScriptRecoveryChannel {caster,owner,amount,duration,interval}=>{
                begin_recovery_channel(world,caster,owner,super::recovery_channel::Spec {amount,duration,interval});
            },
            Outcome::ScriptAbilityChannel {caster,owner,skill_id,level,spec,anchor}=>{
                begin_ability_channel(world,caster,owner,skill_id,level,spec,anchor);
            },
            Outcome::ScriptRemoveBuff { target, buff_id } => {
                handle_script_remove_buff(world, target, buff_id)
            }
            Outcome::ScriptProjectile {
                basic_attack,
                source_missed,source_critical,
                pos,
                owner,
                target,
                tpos,
                radius,
                msd,
                damage_phys,
                damage_magi,
                damage_real,
                damage_profile,
                slow_factor,
                slow_duration,
                hit_radius,
                stun_duration,
                kind_id,
                generation,
            } => handle_script_projectile(
                world,
                basic_attack,
                source_missed,source_critical,
                pos,
                owner,
                target,
                tpos,
                radius,
                msd,
                damage_phys,
                damage_magi,
                damage_real,
                damage_profile,
                slow_factor,
                slow_duration,
                hit_radius,
                stun_duration,
                kind_id,
                generation,
                projectile_origin,
            ),
            Outcome::ScriptTowerFireFx { entity, dir_rad } => {
                handle_script_tower_fire_fx(world, entity, dir_rad)
            }
            Outcome::ScriptAttackPhaseCue {
                entity,
                target,
                target_pos,
                windup_ms,
                backswing_ms,
                dir_rad,
            } => handle_script_attack_phase_cue(
                world,
                entity,
                target,
                target_pos,
                windup_ms,
                backswing_ms,
                dir_rad,
            ),
            Outcome::ScriptConsumeCharge { entity, ability_id } => {
                if let Some(hero) = world.write_storage::<Hero>().get_mut(entity) {
                    // Admission checked the use; a failed spend here is a no-op, never negative.
                    hero.consume_charge(&ability_id);
                    super::cooldown_refund::apply(hero, &ability_id);
                }
            }
            Outcome::ScriptStartCooldown {
                entity,
                ability_id,
                duration,
            } => {
                handle_script_start_cooldown(world, entity, ability_id.clone(), duration);
                if let Some(hero) = world.write_storage::<Hero>().get_mut(entity) {
                    super::cooldown_refund::apply(hero, &ability_id);
                }
            }
            Outcome::ScriptSetMana { entity, pool } => {
                if let Some(hero) = world.write_storage::<Hero>().get_mut(entity) {
                    if hero.mana_pool.is_some() { hero.mana_pool = Some(pool); }
                }
            }
            Outcome::ScriptManaCommit {changes,notifications} => {
                if changes.iter().all(|row|!remove_uids.contains(&row.entity)) {
                    cast_settlement::commit_mana(world,changes,notifications);
                }
            }
            Outcome::ScriptManaGained { entity, amount } => {
                world.write_resource::<ScriptEventQueue>().push(ScriptEvent::ManaGained { e: entity, amount });
            }
            Outcome::ScriptManaSpent { entity, amount, ability_id } => {
                world.write_resource::<ScriptEventQueue>().push(ScriptEvent::SpentMana { caster: entity, cost: amount, ability_id });
            }
            Outcome::EntityRemoved { entity } => {
                world.write_resource::<BuffStore>().remove_all_for(entity);
                world
                    .write_resource::<RemovedEntitiesQueue>()
                    .pending
                    .push(entity.id());
                let _ = world.entities().delete(entity);
            }
            Outcome::SpawnUnit { .. } => {}
            Outcome::SpawnSummoned {entity,pos,unit_id,owner,faction,player_owner,duration} => {
                if handle_summoned_spawn(world,entity,pos,&unit_id,owner,faction,player_owner,duration)? {
                    if let Some(fact) = committed_fact.take() {
                        let _ = world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(fact);
                    }
                }
            }
        }
        if let Some(critical)=critical {critical.emit(world,projection_tick,outcome_ordinal as u32);}
        if let Some(leech)=leech {
            if let Some((source,amount))=leech.healing(world) {
                handle_heal(world,source,amount)?;
                if world.try_fetch::<crate::runtime::MobaMatch>().is_some() {
                    committed_vitals.insert(crate::runtime::canonical_entity_id(source),source);
                }
            }
        }
        log::trace!("processed outcome {}", kind);
    }

    emit_committed_actor_state(world, projection_tick, committed_vitals, committed_cooldowns, committed_charges);
    super::owner_control::settle_batch(world);

    let _ = world.delete_entities(&remove_uids[..]);
    world.write_resource::<Vec<Outcome>>().clear();
    world
        .write_resource::<Vec<Outcome>>()
        .append(&mut next_outcomes);
    drop(outcomes_span);

    Ok(())
}

/// Serial outcome reduction order, separate from gameplay and script ABI state.
#[derive(Default)]
struct CommittedActorStateOrder { tick: u64, ordinal: u32 }

fn emit_committed_actor_state(world: &mut World, tick: u64,
    vitals: BTreeMap<u64, Entity>, cooldowns: BTreeMap<(u64, u32), (Entity, String)>,
    charges: BTreeMap<(u64, u32), (Entity, String)>) {
    use crate::runtime::{FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact};
    if vitals.is_empty() && cooldowns.is_empty() && charges.is_empty() { return; }
    if world.try_fetch::<CommittedActorStateOrder>().is_none() {
        world.insert(CommittedActorStateOrder::default());
    }
    let mut facts = Vec::new();
    {
        let properties = world.read_storage::<CProperty>();
        let heroes = world.read_storage::<Hero>();
        for (source, entity) in vitals {
            if let Some(property) = properties.get(entity) {
                facts.push((source, FactKind::CommittedVitals, ObservableFact::CommittedVitals {
                    source, hp_raw: property.hp.raw(), max_hp_raw: property.mhp.raw(),
                }));
            }
        }
        for ((source, slot), (entity, ability_id)) in cooldowns {
            if let Some(hero) = heroes.get(entity) {
                facts.push((source, FactKind::CommittedCooldown, ObservableFact::CommittedCooldown {
                    source, slot, remaining_raw: hero.get_cooldown(&ability_id).raw().max(0),
                }));
            }
        }
        // Settled after every spend of this batch: one absolute row per slot.
        for ((source, slot), (entity, ability_id)) in charges {
            if let Some(state) = heroes.get(entity).and_then(|hero| hero.ability_charges.get(&ability_id)) {
                facts.push((source, FactKind::CommittedCharge, ObservableFact::CommittedCharge {
                    source, slot, charges: state.charges, max: state.max,
                    recharge_raw: state.recharge.raw(), remaining_raw: state.remaining.raw(),
                }));
            }
        }
    }
    let mut order = world.write_resource::<CommittedActorStateOrder>();
    if order.tick != tick { order.tick = tick; order.ordinal = 0; }
    for (source, fact_kind, fact) in facts {
        let local_ordinal = order.ordinal;
        order.ordinal = order.ordinal.checked_add(1).expect("bounded committed outcomes");
        world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(OrderedFact {
            key: FactOrderingKey { tick, phase: FactPhase::PostStep, canonical_source_order: source, local_ordinal, fact_kind },
            audience: FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::HERO_ABILITY.to_owned()), fact,
        }).expect("valid committed actor state");
    }
}

fn observable_fact_from_outcome(
    outcome: &Outcome,
    tick: u64,
    local_ordinal: u32,
) -> Option<crate::runtime::OrderedFact> {
    use crate::runtime::{
        FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact,
    };
    use omb_script_abi::types::projection_policy_ids as policy;
    let entity_id = crate::runtime::canonical_entity_id;
    let text_id = |text: &str| {
        text.bytes().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        })
    };
    let (source_order, fact_kind, policy_id, fact) = match outcome {
        Outcome::Creep { .. } => (
            local_ordinal as u64,
            FactKind::Spawn,
            policy::SPAWN,
            ObservableFact::Spawn {
                source: None,
                template_id: text_id("creep"),
                team: 0,
            },
        ),
        Outcome::Tower { .. } => (
            local_ordinal as u64,
            FactKind::Spawn,
            policy::TOWER,
            ObservableFact::Spawn {
                source: None,
                template_id: text_id("tower"),
                team: 0,
            },
        ),
        Outcome::SpawnUnit { faction, .. } => (
            local_ordinal as u64,
            FactKind::Spawn,
            policy::SPAWN,
            ObservableFact::Spawn {
                source: None,
                template_id: text_id("unit"),
                team: faction.team_id.max(0) as u32,
            },
        ),
        Outcome::SpawnSummoned {entity,owner,unit_id,faction,..} => (
            entity_id(*entity),FactKind::Spawn,policy::SPAWN,
            ObservableFact::Spawn {source:Some(entity_id(*owner)),template_id:text_id(unit_id),
                team:faction.team_id.max(0) as u32},
        ),
        Outcome::Death { ent, .. } => (
            entity_id(*ent),
            FactKind::Death,
            policy::DEATH,
            ObservableFact::Death {
                source: entity_id(*ent),
                killer: None,
            },
        ),
        Outcome::ProjectileLine2 { source, target, .. } => {
            let source = source.map(entity_id).unwrap_or(local_ordinal as u64);
            (
                source,
                FactKind::Projectile,
                policy::PROJECTILE,
                ObservableFact::Projectile {
                    source,
                    target: target.map(entity_id),
                    effect_id: 0,
                    active: true,
                },
            )
        }
        Outcome::ProjectileDirectional { source, .. } => {
            let source = source.map(entity_id).unwrap_or(local_ordinal as u64);
            (
                source,
                FactKind::Projectile,
                policy::PROJECTILE,
                ObservableFact::Projectile {
                    source,
                    target: None,
                    effect_id: 0,
                    active: true,
                },
            )
        }
        Outcome::ScriptProjectile {
            owner,
            target,
            kind_id,
            ..
        } => {
            let source = entity_id(*owner);
            (
                source,
                FactKind::Projectile,
                policy::PROJECTILE,
                ObservableFact::Projectile {
                    source,
                    target: target.map(entity_id),
                    effect_id: u64::from(*kind_id),
                    active: true,
                },
            )
        }
        Outcome::ProjectileHit { target, .. } => (
            entity_id(*target),
            FactKind::ProjectileImpact,
            policy::PROJECTILE,
            ObservableFact::ProjectileImpact { target: entity_id(*target) },
        ),
        Outcome::Explosion { pos, radius, .. } => (
            local_ordinal as u64,
            FactKind::AreaEffect,
            policy::AOE,
            ObservableFact::AreaEffect {
                source: local_ordinal as u64,
                x_mm: pos.x.raw(),
                y_mm: pos.y.raw(),
                radius_mm: radius.raw().max(0) as u64,
            },
        ),
        Outcome::AddBuff {
            target, buff_id, ..
        } => {
            let target = entity_id(*target);
            (
                target,
                FactKind::Buff,
                policy::BUFF_DEBUFF,
                ObservableFact::Buff {
                    source: target,
                    target,
                    effect_id: text_id(buff_id),
                    active: true,
                },
            )
        }
        Outcome::ScriptRemoveBuff { target, buff_id } => {
            let target = entity_id(*target);
            (
                target,
                FactKind::Buff,
                policy::BUFF_DEBUFF,
                ObservableFact::Buff {
                    source: target,
                    target,
                    effect_id: text_id(buff_id),
                    active: false,
                },
            )
        }
        _ => return None,
    };
    Some(OrderedFact {
        key: FactOrderingKey {
            tick,
            phase: FactPhase::PostStep,
            canonical_source_order: source_order,
            local_ordinal,
            fact_kind,
        },
        audience: FactAudience::VisibilityPolicy(policy_id.to_owned()),
        fact,
    })
}

fn merge_damage_outcomes(raw: Vec<Outcome>) -> Vec<Outcome> {
    let mut first_dmg_idx: std::collections::HashMap<(Entity, Entity, u32), usize> =
        std::collections::HashMap::new();
    let mut aggregated = Vec::with_capacity(raw.len());
    for outcome in raw {
        if let Outcome::Damage {
            phys,
            magi,
            real,
            source,
            target,
            damage_profile,
            predeclared,
            ..
        } = &outcome
        {
            let key = (*target, *source, *damage_profile);
            if let Some(&idx) = first_dmg_idx.get(&key) {
                if let Outcome::Damage {
                    phys: acc_phys,
                    magi: acc_magi,
                    real: acc_real,
                    damage_profile: acc_profile,
                    predeclared: acc_predeclared,
                    ..
                } = &mut aggregated[idx]
                {
                    debug_assert_eq!(*acc_profile, *damage_profile);
                    *acc_phys += *phys;
                    *acc_magi += *magi;
                    *acc_real += *real;
                    *acc_predeclared = *acc_predeclared && *predeclared;
                    continue;
                }
            }
            first_dmg_idx.insert(key, aggregated.len());
        }
        aggregated.push(outcome);
    }
    aggregated
}

fn handle_death(
    world: &mut World,
    next_outcomes: &mut Vec<Outcome>,
    events: &mut impl RuntimeEventSink,
    entity: Entity,
) -> Result<(), failure::Error> {
    let is_lane_match = world.try_fetch::<crate::runtime::MobaMatch>().is_some();
    world.write_resource::<BuffStore>().cancel_damage_zones(entity);
    crate::runtime::record_moba_death(world, entity);
    super::ward_world::retire_if_placed_ward(world, entity);
    let is_enemy_base = {
        let bases = world.read_storage::<IsBase>();
        let factions = world.read_storage::<Faction>();
        bases.get(entity).is_some()
            && factions
                .get(entity)
                .map(|faction| faction.faction_id == FactionType::Enemy)
                .unwrap_or(false)
    };

    distribute_bounty(world, entity);

    // 戰績記錄：Outcome::Death 只會在小兵 hp<=0（玩家擊殺）時產生（見
    // creep_tick.rs）；小兵「漏過終點」走的是完全不同的 Outcome::CreepLeaked
    // 路徑，不會進到這裡，因此這裡判定「是小兵」即可視為玩家擊殺，不會誤計漏怪。
    let is_creep_kill = world.read_storage::<Creep>().get(entity).is_some();
    if is_creep_kill {
        world.write_resource::<crate::comp::MatchKillCounter>().0 += 1;
    }

    {
        let mut creeps = world.write_storage::<Creep>();
        let mut towers = world.write_storage::<Tower>();
        if let Some(creep) = creeps.get_mut(entity) {
            if let Some(blocking_tower) = creep.block_tower {
                if let Some(tower) = towers.get_mut(blocking_tower) {
                    tower.block_creeps.retain(|&x| x != entity);
                }
            }
        } else if let Some(tower) = towers.get_mut(entity) {
            let blocked = tower.block_creeps.clone();
            for creep_entity in blocked {
                if let Some(creep) = creeps.get_mut(creep_entity) {
                    creep.block_tower = None;
                    next_outcomes.push(Outcome::CreepWalk {
                        target: creep_entity,
                    });
                }
            }
        }
    }

    if is_enemy_base && !is_lane_match {
        log::info!("enemy base entity {:?} destroyed", entity);
        events.emit(game_end_event(
            "player",
            json!({ "winner": "player", "base_entity_id": entity.id() }),
        ));
    }
    Ok(())
}

fn distribute_bounty(world: &mut World, dead: Entity) {
    // Opt-in lane rules own all rewards, including duplicate death gating.
    // Legacy (non-lane) MOBA and TD retain their original settlement paths.
    if world.try_fetch::<crate::runtime::MobaMatch>().is_some()
        || world.read_resource::<crate::comp::GameMode>().is_td() {
        return;
    }
    let bounty = match world.read_storage::<Bounty>().get(dead).copied() {
        Some(bounty) => bounty,
        None => return,
    };
    let dead_pos = match world.read_storage::<Pos>().get(dead).map(|pos| pos.0) {
        Some(pos) => pos,
        None => return,
    };
    let dead_faction = world.read_storage::<Faction>().get(dead).cloned();

    let hero_entity = {
        let entities = world.entities();
        let heroes = world.read_storage::<Hero>();
        let factions = world.read_storage::<Faction>();
        let positions = world.read_storage::<Pos>();
        let mut best = None;
        for (entity, _hero, faction, pos) in (&entities, &heroes, &factions, &positions).join() {
            if faction.faction_id != FactionType::Player {
                continue;
            }
            if let Some(ref dead_faction) = dead_faction {
                if !faction.is_hostile_to(dead_faction) {
                    continue;
                }
            }
            let (px, py) = pos.xy_f32();
            let dx = px - dead_pos.x.to_f32_for_render();
            let dy = py - dead_pos.y.to_f32_for_render();
            let d2 = dx * dx + dy * dy;
            if d2 > 1200.0 * 1200.0 {
                continue;
            }
            if best.map(|(_, d)| d2 < d).unwrap_or(true) {
                best = Some((entity, d2));
            }
        }
        match best {
            Some((entity, _)) => entity,
            None => return,
        }
    };

    if let Some(gold) = world.write_storage::<Gold>().get_mut(hero_entity) {
        gold.0 += bounty.gold;
    }
    let leveled_up = {
        let mut heroes = world.write_storage::<Hero>();
        if let Some(hero) = heroes.get_mut(hero_entity) {
            let before = hero.level;
            let _ = hero.add_experience(bounty.exp);
            hero.level != before
        } else {
            false
        }
    };
    if leveled_up {
        log::info!("hero entity {:?} leveled up", hero_entity);
    }
}

fn credit_td_bounty_for_source(
    world: &mut World,
    source: Entity,
    target: Entity,
) -> Result<bool, failure::Error> {
    if !world.read_resource::<crate::comp::GameMode>().is_td() {
        return Ok(false);
    }
    let player_id = world
        .read_storage::<PlayerOwner>()
        .get(source)
        .map(|owner| owner.player_id);
    let bounty = world.read_storage::<Bounty>().get(target).copied();
    let (Some(player_id), Some(bounty)) = (player_id, bounty) else {
        return Ok(false);
    };
    if bounty.gold <= 0 {
        return Ok(false);
    }
    credit_player_economy(world, "TDCreepBounty", player_id, bounty.gold)?;
    Ok(true)
}

fn remember_projectile_origin(world: &mut World, projectile: Entity, source: Entity) {
    let Some(origin) = super::moba_match::capture_projectile_origin(world, source) else { return; };
    if world.try_fetch::<ProjectileOrigins>().is_none() {
        world.insert(ProjectileOrigins::default());
    }
    world.write_resource::<ProjectileOrigins>().0.insert(projectile, origin);
}

fn prune_projectile_origins(world: &World) {
    let Some(mut origins) = world.try_fetch_mut::<ProjectileOrigins>() else { return; };
    let entities = world.entities();
    let projectiles = world.read_storage::<Projectile>();
    origins.0.retain(|entity, _| entities.is_alive(*entity) && projectiles.get(*entity).is_some());
    drop(origins);
    if let Some(mut orbs) = world.try_fetch_mut::<super::attack_orb::OrbAttacks>() {
        orbs.0.retain(|entity, _| entities.is_alive(*entity) && projectiles.get(*entity).is_some());
    }
}

fn handle_projectile(
    world: &mut World,
    pos: omoba_sim::Vec2,
    source: Option<Entity>,
    target: Option<Entity>,
) -> Result<(), failure::Error> {
    use omoba_sim::Vec2 as SimVec2;

    let source_entity = source.ok_or_else(|| failure::err_msg("Missing source entity"))?;
    let target_entity = target.ok_or_else(|| failure::err_msg("Missing target entity"))?;
    let master_seed = world.read_resource::<MasterSeed>().0;
    let tick = world.read_resource::<Tick>().0 as u32;
    let attacker_id = source_entity.id();

    let (msd, target_pos, atk_phys, stun_duration_roll, visual_count, missed, critical) = {
        let positions = world.read_storage::<Pos>();
        let attacks = world.read_storage::<TAttack>();
        let buff_store = world.read_resource::<BuffStore>();
        let buildings = world.read_storage::<IsBuilding>();

        let _source_pos = positions
            .get(source_entity)
            .ok_or_else(|| failure::err_msg("Source position not found"))?;
        let target_pos = positions
            .get(target_entity)
            .ok_or_else(|| failure::err_msg("Target position not found"))?;
        let attack = attacks
            .get(source_entity)
            .ok_or_else(|| failure::err_msg("Source attack properties not found"))?;
        let is_building = buildings.get(source_entity).is_some();
        let stats =
            crate::runtime::ability_runtime::UnitStats::from_refs(&*buff_store, is_building);
        let mut final_atk = stats.normal_attack_physical(attack.atk_physic.v, source_entity);

        let accuracy_bonus = buff_store.sum_add(source_entity, StatKey::AccuracyBonus);
        let accuracy = (Fixed64::ONE + accuracy_bonus).clamp(Fixed64::ZERO, Fixed64::ONE);
        let mut missed=super::attack_accuracy::source_misses(master_seed,tick,source_entity,stats.miss_chance(source_entity));
        if accuracy < Fixed64::ONE {
            let roll = Fixed64::from_raw(
                (crate::runtime::tick_random_u64(
                    master_seed,
                    u64::from(tick),
                    (u64::from(attacker_id) << 16) | u64::from(OP_PROJECTILE_ACCURACY),
                ) % omoba_sim::fixed::SCALE as u64) as i64,
            );
            if roll >= accuracy {
                missed = true;
            }
        }
        let mut critical=false;
        if missed {final_atk=Fixed64::ZERO;} else {
            // Projectile attacks historically have no implicit base critical rate.
            // Shared buffs opt in; commit the amount once, before visual copies.
            let (chance,multiplier)=stats.crit_with_base(source_entity,Fixed64::ZERO,Fixed64::from_i32(2));
            critical=super::attack_accuracy::source_crits(master_seed,tick,source_entity,chance);
            if critical {final_atk=final_atk*multiplier;}
        }

        let mut stun_chance = 0.0f32;
        let mut stun_duration = 0.0f32;
        for (_, entry) in buff_store.iter_for(source_entity) {
            let chance = entry
                .payload
                .get("attack_stun_chance")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let duration = entry
                .payload
                .get("attack_stun_duration")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            if chance > stun_chance {
                stun_chance = chance;
                stun_duration = duration;
            }
        }
        let stun_duration_roll = if !missed && stun_chance > 0.0 && stun_duration > 0.0 {
            let roll = crate::runtime::tick_random_u64(
                master_seed,
                u64::from(tick),
                (u64::from(attacker_id) << 16) | u64::from(OP_PROJECTILE_STUN_ROLL),
            ) % omoba_sim::fixed::SCALE as u64;
            if roll as f32 / (omoba_sim::fixed::SCALE as f32) < stun_chance {
                Fixed64::from_raw((stun_duration * omoba_sim::fixed::SCALE as f32) as i64)
            } else {
                Fixed64::ZERO
            }
        } else {
            Fixed64::ZERO
        };

        let visual_count = buff_store
            .sum_add(source_entity, StatKey::MultiShotVisual)
            .to_f32_for_render();
        let visual_count = if visual_count >= 2.0 {
            visual_count.round().max(1.0) as u32
        } else {
            1
        };

        (
            attack.bullet_speed,
            target_pos.0,
            final_atk,
            stun_duration_roll,
            visual_count,
            missed,critical,
        )
    };

    // All launch prerequisites were resolved above. Consume once, including misses,
    // before creating the one real projectile and any zero-damage visual copies.
    world.write_resource::<BuffStore>().consume_next_attack_bonus(source_entity);
    let initial_dist = (target_pos - pos).length();
    let flight_time_s = if msd.to_f32_for_render() > 0.0 {
        (initial_dist.to_f32_for_render() / msd.to_f32_for_render()).max(0.01)
    } else {
        0.01
    };
    let safety_time_left =
        Fixed64::from_raw(((flight_time_s * 3.0 + 3.0) * omoba_sim::fixed::SCALE as f32) as i64);

    let delta = target_pos - pos;
    let dir = if delta.length_squared() > Fixed64::from_raw(1) {
        delta.normalized()
    } else {
        SimVec2::new(Fixed64::ONE, Fixed64::ZERO)
    };
    let perp = SimVec2::new(-dir.y, dir.x);
    let lateral_step = Fixed64::from_i32(24);

    for i in 0..visual_count {
        let is_real = i == 0;
        let target_this = if is_real { target } else { None };
        let lateral = if visual_count > 1 {
            let half_raw = (visual_count as i64 - 1) * 512;
            let i_scaled = i as i64 * omoba_sim::fixed::SCALE;
            Fixed64::from_raw(i_scaled - half_raw) * lateral_step
        } else {
            Fixed64::ZERO
        };
        let start_pos = pos + perp * lateral;
        let projectile = world
            .create_entity()
            .with(Pos(start_pos))
            .with(Projectile {
                damage_stage: crate::runtime::comp::DamageStage::Raw,
                time_left: safety_time_left,
                owner: source_entity,
                tpos: target_pos,
                target: target_this,
                radius: Fixed64::ZERO,
                msd,
                damage_phys: if is_real { atk_phys } else { Fixed64::ZERO },
                damage_magi: Fixed64::ZERO,
                damage_real: Fixed64::ZERO,
                damage_profile: AbiDamageProfile::NORMAL.bits(),
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                hit_radius: Fixed64::ZERO,
                stun_duration: if is_real {
                    stun_duration_roll
                } else {
                    Fixed64::ZERO
                },
                kind_id: 0,
                generation: 0,
            })
            .build();
    remember_projectile_origin(world, projectile, source_entity);
    if is_real {
        let leech=crate::runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).lifesteal(source_entity);
        if let Some(origin)=world.write_resource::<ProjectileOrigins>().0.get_mut(&projectile) {
            origin.normal_attack=atk_phys>Fixed64::ZERO;origin.attack_missed=missed;origin.attack_critical=critical&&!missed&&origin.normal_attack;
            origin.attack_lifesteal=if origin.normal_attack&&!missed {leech}else{Fixed64::ZERO};
        }
        // G-2 toggle orb: a committed ordinary attack pays per launch (even a
        // missed one); only a later non-evaded hit applies the orb.
        if atk_phys>Fixed64::ZERO {super::attack_orb::on_launch(world,source_entity,projectile);}
    }
    }
    Ok(())
}

fn handle_summoned_spawn(world:&mut World,entity:Entity,pos:omoba_sim::Vec2,unit_id:&str,
    owner:Entity,faction:Faction,player_owner:Option<u32>,duration:Fixed64) -> Result<bool,failure::Error> {
    if let Some(ledger)=world.try_fetch::<super::summon_navigation::PlacementReservations>() {ledger.release(entity);}
    // A duplicate result must neither overwrite nor retire a committed body.
    if world.read_storage::<Unit>().get(entity).is_some()
        || world.read_storage::<Pos>().get(entity).is_some()
        || world.read_storage::<CProperty>().get(entity).is_some() {return Ok(false);}
    let stats=omoba_template_ids::summon_by_name(unit_id).and_then(omoba_template_ids::active_summon_stats);
    let valid_owner=world.entities().is_alive(owner)
        && world.read_storage::<CProperty>().get(owner).is_some_and(|p|p.hp>Fixed64::ZERO)
        && world.read_storage::<Faction>().get(owner).is_some_and(|f|
            f.team_id==faction.team_id && f.faction_id==faction.faction_id)
        && world.read_storage::<PlayerOwner>().get(owner).map(|p|p.player_id)==player_owner;
    let Some(stats)=stats.filter(|_|valid_owner && world.entities().is_alive(entity)
        && duration>=Fixed64::ZERO && duration<=Fixed64::from_i32(3600)) else {
        let _=world.entities().delete(entity);return Ok(false);
    };
    let occupied={
        let entities=world.entities();let positions=world.read_storage::<Pos>();
        let radii=world.read_storage::<CollisionRadius>();let factions=world.read_storage::<Faction>();
        (&entities,&positions,&radii,&factions).join().filter(|(_,_,_,f)|f.team_id==faction.team_id)
            .map(|(_,p,r,_)|(p.0,r.0)).collect::<Vec<_>>()
    };
    let anchor=world.read_storage::<Pos>().get(owner).map(|p|p.0);
    let Some(pos)=super::summon_navigation::placement(pos,anchor,stats.collision_radius,
        &world.read_resource::<BlockedRegions>(),&occupied) else {
        log::warn!("summon placement retired reservation {:?}: no legal public-terrain position",entity);
        let _=world.entities().delete(entity);return Ok(false);
    };
    commit_summoned_spawn(world,entity,pos,unit_id,owner,faction,player_owner,duration,stats)?;
    Ok(true)
}

fn commit_summoned_spawn(world:&mut World,entity:Entity,pos:omoba_sim::Vec2,unit_id:&str,
    owner:Entity,faction:Faction,player_owner:Option<u32>,duration:Fixed64,
    stats:&omoba_template_ids::SummonStats) -> Result<(),failure::Error> {
    let id=omoba_template_ids::summon_by_name(unit_id).expect("validated summon");
    let mut unit=Unit::new(unit_id.into(),omoba_template_ids::active_summon_display(id).into(),UnitType::Summon);
    unit.max_hp=(stats.hp.raw()/1024) as i32;unit.current_hp=unit.max_hp;
    unit.base_damage=(stats.damage.raw()/1024) as i32;unit.base_armor=stats.armor;
    unit.magic_resistance=stats.magic_resistance;unit.attack_range=stats.attack_range;
    unit.move_speed=stats.move_speed;unit.attack_speed=Fixed64::ONE/stats.attack_interval;
    unit.spawn_position=(pos.x.to_f32_for_render(),pos.y.to_f32_for_render());
    unit.exp_reward=0;unit.gold_reward=0;unit.bounty_type=BountyType::None;
    let facing=world.read_storage::<Facing>().get(owner).copied().unwrap_or(Facing(omoba_sim::Angle::ZERO));
    // G-3: owner-level attack scaling is sampled once, at the committed spawn.
    let owner_level=world.read_storage::<Hero>().get(owner).map_or(1,|hero|hero.level.max(1));
    let damage=stats.damage+stats.damage_per_owner_level*Fixed64::from_i32(owner_level-1);
    unit.base_damage=(damage.raw()/1024) as i32;
    world.write_storage::<Unit>().insert(entity,unit)?;
    world.write_storage::<Pos>().insert(entity,Pos(pos))?;
    world.write_storage::<Faction>().insert(entity,faction)?;
    world.write_storage::<CProperty>().insert(entity,CProperty {hp:stats.hp,mhp:stats.hp,
        msd:stats.move_speed,def_physic:stats.armor,def_magic:stats.magic_resistance})?;
    world.write_storage::<TAttack>().insert(entity,TAttack::new(damage,stats.attack_interval,stats.attack_range,Fixed64::from_i32(900)))?;
    world.write_storage::<Facing>().insert(entity,facing)?;
    world.write_storage::<FacingBroadcast>().insert(entity,FacingBroadcast(None))?;
    world.write_storage::<CollisionRadius>().insert(entity,CollisionRadius(stats.collision_radius))?;
    if stats.vision_radius>Fixed64::ZERO {
        world.write_storage::<VisionSource>().insert(entity,VisionSource {
            team:world.read_storage::<Faction>().get(entity).expect("summon faction").team_id.max(0) as u32,
            radius:stats.vision_radius,detection_level:0})?;
    }
    world.write_storage::<ScriptUnitTag>().insert(entity,ScriptUnitTag {unit_id:unit_id.into()})?;
    let mut summoned=SummonedUnit::new_fixed(owner,(duration>Fixed64::ZERO).then_some(duration));
    summoned.navigation.follow_start_distance=stats.follow_start_distance;
    summoned.navigation.follow_stop_distance=stats.follow_stop_distance;
    summoned.immobile=stats.immobile;
    summoned.expire_with_owner=stats.owner_death==omoba_template_ids::SummonOwnerDeath::Expire;
    world.write_storage::<SummonedUnit>().insert(entity,summoned)?;
    if stats.invulnerable {
        if let Some(mut state)=world.try_fetch_mut::<crate::runtime::MobaMatch>() {state.add_invulnerable_summon(&world.entities(),entity);}
    }
    if let Some(player_id)=player_owner {world.write_storage::<PlayerOwner>().insert(entity,PlayerOwner::new(player_id))?;}
    world.write_resource::<ScriptEventQueue>().push(ScriptEvent::Spawn {e:entity});
    Ok(())
}

fn handle_creep_spawn(world: &mut World, cd: CreepData) -> Result<(), failure::Error> {
    let creep_name = cd.creep.name.clone();
    let bounty = creep_bounty_from_template(&creep_name);
    let faction = match cd.faction_name.as_str() {
        "Player" | "player" => Faction::new(FactionType::Player, 0),
        _ => Faction::new(FactionType::Enemy, 1),
    };
    let turn_speed_rad_f = cd.turn_speed_deg.to_f32_for_render().to_radians();
    let unit_id = format!("creep_{}", creep_name);
    let entity = world
        .create_entity()
        .with(Pos(cd.pos))
        .with(cd.creep)
        .with(cd.cdata)
        .with(faction)
        .with(bounty)
        .with(Facing(omoba_sim::Angle::ZERO))
        .with(FacingBroadcast(None))
        .with(TurnSpeed(Fixed64::from_raw(
            (turn_speed_rad_f * 1024.0) as i64,
        )))
        .with(ScriptUnitTag { unit_id })
        .build();
    world
        .write_resource::<ScriptEventQueue>()
        .push(ScriptEvent::Spawn { e: entity });
    world.write_resource::<BuffStore>().add(
        entity,
        "creep_min_speed_floor",
        Fixed64::from_raw(i64::MAX),
        json!({ "movespeed_absolute_min": 10.0 }),
    );
    Ok(())
}

fn creep_bounty_from_template(creep_name: &str) -> Bounty {
    if creep_name.starts_with("ally_") || creep_name.starts_with("td_btd_") {
        return Bounty { gold: 0, exp: 0 };
    }
    if let Some(stats) = omoba_template_ids::creep_by_name(creep_name)
        .and_then(omoba_template_ids::active_creep_stats)
    {
        return Bounty {
            gold: stats.gold_reward,
            exp: stats.exp_reward,
        };
    }
    match creep_name {
        "melee_minion" => Bounty { gold: 18, exp: 55 },
        "ranged_minion" => Bounty { gold: 15, exp: 45 },
        "siege_minion" => Bounty { gold: 40, exp: 110 },
        _ => Bounty { gold: 10, exp: 25 },
    }
}

fn handle_script_set_pos(world: &mut World, entity: Entity, pos: omoba_sim::Vec2) {
    if let Some(pos_comp) = world.write_storage::<Pos>().get_mut(entity) {
        pos_comp.0 = pos;
    }
}

fn handle_script_set_facing(world: &mut World, entity: Entity, facing: omoba_sim::Angle) {
    if let Some(facing_comp) = world.write_storage::<Facing>().get_mut(entity) {
        facing_comp.0 = facing;
    }
}

fn handle_script_set_asd_count(world: &mut World, entity: Entity, asd_count: Fixed64) {
    if let Some(attack) = world.write_storage::<TAttack>().get_mut(entity) {
        attack.asd_count = asd_count;
    }
}

fn handle_script_set_tower_atk(world: &mut World, entity: Entity, value: Fixed64) {
    if let Some(attack) = world.write_storage::<TAttack>().get_mut(entity) {
        attack.atk_physic.bv = value;
        attack.atk_physic.v = value;
    }
}

fn handle_script_set_tower_range(world: &mut World, entity: Entity, value: Fixed64) {
    if let Some(attack) = world.write_storage::<TAttack>().get_mut(entity) {
        attack.range.bv = value;
        attack.range.v = value;
    }
}

fn handle_script_set_asd_interval(world: &mut World, entity: Entity, value: Fixed64) {
    if let Some(attack) = world.write_storage::<TAttack>().get_mut(entity) {
        attack.asd.bv = value;
        attack.asd.v = value;
    }
}

fn handle_script_direct_damage(world: &mut World, target: Entity, amount: Fixed64) {
    apply_checked_direct_damage(world, None, target, amount);
}

fn apply_checked_direct_damage(world: &mut World, source: Option<Entity>, target: Entity, amount: Fixed64) {
    if !crate::runtime::native::moba_match::moba_damage_from_allowed(world,source,target) { return; }
    // Same authority-settlement boundary as regular Damage. A disclosed MOBA
    // actor must not locally redo Lua damage (including immune returners).
    if world.try_fetch::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>()
        .is_some_and(|targets| targets.0.contains(&target)) { return; }
    let before = world.read_storage::<CProperty>().get(target).map(|p| p.hp);
    let took_damage = amount > Fixed64::ZERO && before.is_some_and(|hp| hp > Fixed64::ZERO);
    if took_damage {
        crate::runtime::native::moba_match::interrupt_moba_recall(world, target);
    }
    let amount = if took_damage {
        world.write_resource::<BuffStore>().absorb_damage_with_shield(target, amount)
    } else { amount };
    let applied = {
        let mut properties = world.write_storage::<CProperty>();
        if let Some(prop) = properties.get_mut(target) {
            prop.hp = (prop.hp - amount).max(Fixed64::ZERO);
            true
        } else { false }
    };
    if applied {
        if took_damage {
            // A shield may absorb the remainder. Armor and max-health benefits
            // still participated in a settled packet, so undo closes.
            super::shop::note_shop_undo_combat(world, source, target);
        }
        if took_damage && amount > Fixed64::ZERO {
            super::cast_windup::interrupt_for_damage(world,target);
            crate::runtime::native::moba_match::record_moba_hero_damage(world, source, target,
                before.is_some_and(|hp| amount >= hp));
            crate::runtime::native::moba_match::record_moba_jungle_damage(world, source, target,
                before.is_some_and(|hp| amount >= hp));
            crate::runtime::native::moba_match::record_moba_lane_damage(world, source, target,
                before.is_some_and(|hp| amount >= hp));
            crate::runtime::native::moba_match::record_moba_building_damage(world, source, target,
                before.is_some_and(|hp| amount >= hp));
        }
        return;
    }
    if let Some(unit) = world.write_storage::<Unit>().get_mut(target) {
        let amount_i = amount.to_f32_for_render() as i32;
        unit.current_hp = (unit.current_hp - amount_i).max(0);
    }
}

pub(crate) fn handle_script_heal(world: &mut World, target: Entity, amount: Fixed64) {
    let amount=received_healing(world,target,amount);
    if let Some(prop) = world.write_storage::<CProperty>().get_mut(target) {
        prop.hp = healed_hp(prop.hp,prop.mhp,amount);
        return;
    }
    if let Some(unit) = world.write_storage::<Unit>().get_mut(target) {
        unit.current_hp = (healed_hp(Fixed64::from_i32(unit.current_hp),Fixed64::from_i32(unit.max_hp),amount).raw()/1024) as i32;
    }
}

fn handle_script_remove_buff(world: &mut World, target: Entity, buff_id: String) {
    world.write_resource::<BuffStore>().remove(target, &buff_id);
}

#[allow(clippy::too_many_arguments)]
fn handle_script_projectile(
    world: &mut World,
    basic_attack: bool,
    source_missed: bool,
    source_critical: bool,
    pos: omoba_sim::Vec2,
    owner: Entity,
    target: Option<Entity>,
    tpos: omoba_sim::Vec2,
    radius: Fixed64,
    msd: Fixed64,
    damage_phys: Fixed64,
    damage_magi: Fixed64,
    damage_real: Fixed64,
    damage_profile: u32,
    slow_factor: Fixed64,
    slow_duration: Fixed64,
    hit_radius: Fixed64,
    stun_duration: Fixed64,
    kind_id: u16,
    generation: u8,
    origin: Option<crate::comp::ProjectileOrigin>,
) {
    let initial_dist = (tpos - pos).length();
    let speed_f = msd.to_f32_for_render();
    let flight_time_s = if speed_f > 0.0 {
        (initial_dist.to_f32_for_render() / speed_f).max(0.01)
    } else {
        0.01
    };
    let safety =
        Fixed64::from_raw(((flight_time_s * 3.0 + 1.5) * omoba_sim::fixed::SCALE as f32) as i64);

    let projectile = world
        .create_entity()
        .with(Pos(pos))
        .with(Projectile {
            damage_stage: crate::runtime::comp::DamageStage::Raw,
            time_left: safety,
            owner,
            target,
            tpos,
            radius,
            msd,
            damage_phys,
            damage_magi,
            damage_real,
            damage_profile,
            slow_factor,
            slow_duration,
            hit_radius,
            stun_duration,
            kind_id,
            generation,
        })
        .build();
    let direct_attack=origin.is_none();
    if let Some(mut origin) = origin {
        origin.normal_attack=false;
        origin.script_attack=false;
        origin.attack_missed=false;
        origin.attack_lifesteal=Fixed64::ZERO;origin.attack_critical=false;
        if world.try_fetch::<ProjectileOrigins>().is_none() {world.insert(ProjectileOrigins::default());}
        world.write_resource::<ProjectileOrigins>().0.insert(projectile,origin);
    } else {remember_projectile_origin(world, projectile, owner);}
    if direct_attack && (basic_attack||source_missed) && target.is_some()
        && radius==Fixed64::ZERO && hit_radius==Fixed64::ZERO && msd>Fixed64::ZERO && generation==0 {
        let leech=crate::runtime::UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).lifesteal(owner);
        if let Some(origin)=world.write_resource::<ProjectileOrigins>().0.get_mut(&projectile) {
            origin.script_attack=basic_attack&&!source_missed;origin.attack_missed=source_missed;origin.attack_critical=origin.script_attack&&source_critical&&damage_phys>Fixed64::ZERO;
            origin.attack_lifesteal=if origin.script_attack {leech}else{Fixed64::ZERO};
        }
    }
}

fn handle_script_tower_fire_fx(world: &mut World, entity: Entity, dir_rad: f32) {
    if world.read_storage::<Tower>().get(entity).is_none() {
        return;
    }
    let spawn_tick = world.read_resource::<Tick>().0 as u32;
    let entity_id = entity.id();
    let mut queue = world.write_resource::<TowerFireFxQueue>();
    if queue
        .pending
        .iter()
        .any(|fx| fx.entity_id == entity_id && fx.spawn_tick == spawn_tick)
    {
        return;
    }
    queue.pending.push(crate::runtime::comp::TowerFireFx {
        entity_id,
        entity_gen: entity.gen().id() as u32,
        spawn_tick,
        dir_rad,
    });
}

fn handle_script_attack_phase_cue(
    world: &mut World,
    entity: Entity,
    target: Option<Entity>,
    target_pos: Option<omoba_sim::Vec2>,
    windup_ms: u32,
    backswing_ms: u32,
    dir_rad: f32,
) {
    if let Some(target)=target {
        crate::runtime::native::moba_match::record_brush_attack_reveal(world,entity,target);
    }
    let current_tick = world.read_resource::<Tick>().0 as u32;
    let mut queue = world.write_resource::<AttackPhaseFxQueue>();
    let attack_seq = queue.next_seq;
    queue.next_seq = queue.next_seq.wrapping_add(1);
    queue.pending.push(AttackPhaseFx {
        entity_id: entity.id(),
        entity_gen: entity.gen().id() as u32,
        spawn_tick: current_tick,
        attack_seq,
        is_critical: false,
        windup_ms,
        impact_at_ms: windup_ms,
        backswing_ms,
        dir_rad,
        target_entity_id: target.map(|target| target.id()),
        target_pos_x: target_pos.map(|pos| pos.x.to_f32_for_render()),
        target_pos_y: target_pos.map(|pos| pos.y.to_f32_for_render()),
    });
}

fn handle_script_start_cooldown(
    world: &mut World,
    entity: Entity,
    ability_id: String,
    duration: Fixed64,
) {
    if let Some(hero) = world.write_storage::<Hero>().get_mut(entity) {
        hero.start_cooldown(&ability_id, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hostile_item_relation_is_opt_in_and_preserves_legacy_enemy_matrix() {
        use omoba_template_ids::MobaItemTeamConst as Scope;
        let source=Faction {faction_id:FactionType::Player,team_id:1};
        for (kind,team,enemy,hostile) in [(FactionType::Player,1,false,false),(FactionType::Enemy,2,true,true),
            (FactionType::Player,2,true,true),(FactionType::HostileNeutral,0,false,true),
            (FactionType::Neutral,0,false,false),(FactionType::Neutral,2,true,false),
            (FactionType::HostileNeutral,1,false,false)] {
            let target=Faction {faction_id:kind,team_id:team};
            assert_eq!(item_faction_matches(Scope::Enemy,&source,&target),enemy);
            assert_eq!(item_faction_matches(Scope::Hostile,&source,&target),hostile);
            assert!(!item_faction_matches(Scope::Any,&source,&target));
        }
    }
    #[test]
    fn native_and_script_heals_share_received_policy_clamps_and_never_resurrect() {
        let mut w=World::new();w.register::<CProperty>();w.insert(BuffStore::default());
        let e=w.create_entity().with(CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(1000),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        w.write_resource::<BuffStore>().grant_healing_reduction(e,Fixed64::from_raw(512),Fixed64::from_i32(4));
        handle_heal(&mut w,e,Fixed64::from_i32(40)).unwrap();handle_script_heal(&mut w,e,Fixed64::from_i32(40));
        assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(140));
        handle_heal(&mut w,e,Fixed64::from_i32(-40)).unwrap();assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(140));
        handle_script_heal(&mut w,e,Fixed64::from_raw(i64::MAX));assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(1000));
        w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ZERO;
        handle_heal(&mut w,e,Fixed64::from_i32(40)).unwrap();handle_script_heal(&mut w,e,Fixed64::from_i32(40));
        assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::ZERO);
    }

    #[test]
    fn legacy_unit_healing_and_strongest_reduction_refresh_expire_safely() {
        let mut w=World::new();w.register::<CProperty>();w.register::<Unit>();w.insert(BuffStore::default());
        let mut unit=Unit::new("heal".into(),"heal".into(),UnitType::Creep);unit.current_hp=50;unit.max_hp=100;
        let e=w.create_entity().with(unit).build();
        {let mut buffs=w.write_resource::<BuffStore>();
            assert!(buffs.grant_healing_reduction(e,Fixed64::from_raw(512),Fixed64::from_i32(4)));
            assert!(buffs.grant_healing_reduction(e,Fixed64::from_raw(768),Fixed64::from_i32(2)));
            assert_eq!(buffs.get(e,"healing_reduction").unwrap().remaining,Fixed64::from_i32(4));
            assert!(!buffs.grant_healing_reduction(e,Fixed64::ONE,Fixed64::from_i32(2)));
        }
        handle_script_heal(&mut w,e,Fixed64::from_i32(80));assert_eq!(w.read_storage::<Unit>().get(e).unwrap().current_hp,90);
        w.write_resource::<BuffStore>().tick(Fixed64::from_i32(4));handle_script_heal(&mut w,e,Fixed64::from_i32(10));
        assert_eq!(w.read_storage::<Unit>().get(e).unwrap().current_hp,100);
        w.write_storage::<Unit>().get_mut(e).unwrap().current_hp=0;handle_script_heal(&mut w,e,Fixed64::from_i32(100));
        assert_eq!(w.read_storage::<Unit>().get(e).unwrap().current_hp,0);
    }
    #[test]
    fn targeted_item_visibility_uses_same_brush_sources_as_public_disclosure() {
        use crate::runtime::brush_vision::{BrushVisionRegion,BrushVisionRegions};
        let mut world=World::new();
        world.register::<Pos>();world.register::<Faction>();world.register::<ReplicationScope>();
        world.register::<VisibilityOverride>();world.register::<StealthProfile>();world.register::<VisionSource>();
        let p=|x,y|omoba_sim::Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let target=world.create_entity().with(Pos(p(15,0))).build();
        let source=world.create_entity().with(Pos(p(0,0))).with(VisionSource {
            team:1,radius:Fixed64::from_i32(100),detection_level:0}).build();
        world.insert(BrushVisionRegions::from(vec![BrushVisionRegion {min:p(10,-5),max:p(20,5)}]));
        assert!(!item_entity_visible_to_team(&world,target,1));
        world.write_storage::<Pos>().get_mut(source).unwrap().0=p(12,0);
        assert!(item_entity_visible_to_team(&world,target,1));
        world.write_storage::<Pos>().get_mut(source).unwrap().0=p(0,0);
        world.write_storage::<Pos>().get_mut(target).unwrap().0=p(30,0);
        assert!(item_entity_visible_to_team(&world,target,1),"brush never blocks targets behind it");
        world.insert(BrushVisionRegions::from(vec![
            BrushVisionRegion {min:p(10,-5),max:p(20,5)},
            BrushVisionRegion {min:p(18,-5),max:p(40,5)}]));
        world.write_storage::<Pos>().get_mut(source).unwrap().0=p(15,0);
        assert!(item_entity_visible_to_team(&world,target,1),"live target admission crosses connected capsules");
        world.write_storage::<Pos>().get_mut(source).unwrap().0=p(0,0);
        assert!(!item_entity_visible_to_team(&world,target,1),"outside source still cannot disclose either branch");
    }
    #[test]
    fn disclosed_authority_targets_do_not_reapply_lua_direct_damage() {
        let mut world = World::new();
        world.register::<CProperty>();
        world.insert(BuffStore::default());
        let entity = world.create_entity().with(CProperty {
            hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,
            def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO,
        }).build();
        world.insert(crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets(
            std::collections::BTreeSet::from([entity])));
        handle_script_direct_damage(&mut world,entity,Fixed64::from_i32(1000));
        assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(100));
        world.remove::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>();
        handle_script_direct_damage(&mut world,entity,Fixed64::from_i32(10));
        assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(90));
    }
    use crate::runtime::comp::{
        GameMode, PendingTowerAbilityActivationQueue, PendingTowerAbilityCastQueue, PlayerLives,
        TowerAbilityCastResult, TowerAbilityCastResults, TowerActiveAbilityState,
    };
    use crate::runtime::UnitStats;
    use omoba_template_ids::{active_creep_stats, creep_id_str, CreepId};
    use specs::Join;

    #[test]
    fn td_layer_commit_materializes_only_ordered_survivors() {
        let mut world = World::new();
        world.register::<Creep>();
        world.register::<CProperty>();
        world.register::<Pos>();
        world.register::<Faction>();
        world.register::<Bounty>();
        world.register::<Facing>();
        world.register::<FacingBroadcast>();
        world.register::<TurnSpeed>();
        world.register::<ScriptUnitTag>();
        world.insert(ScriptEventQueue::default());
        world.insert(crate::runtime::ObservableFactBuffer::default());
        world.insert(BuffStore::default());

        let state = crate::runtime::TdLayerState {
            base_archetype: "ceramic".to_string(),
            current_layer: "ceramic".to_string(),
            properties: 0,
            regrow_ceiling: "ceramic".to_string(),
            regrow_elapsed: Fixed64::ZERO,
            remaining_leak_value: 104,
            spawn_lineage: 77,
        };
        let target = world
            .create_entity()
            .with(Creep {
                name: "td_btd_ceramic".to_string(),
                label: Some("Ceramic".to_string()),
                path: "main".to_string(),
                pidx: 2,
                path_remaining_distance: Fixed64::from_i32(500),
                block_tower: None,
                status: CreepStatus::Walk,
                td_layer: Some(state.clone()),
            })
            .with(CProperty {
                hp: Fixed64::from_i32(10),
                mhp: Fixed64::from_i32(10),
                msd: Fixed64::from_i32(210),
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .with(Pos(omoba_sim::Vec2::ZERO))
            .with(Faction::new(FactionType::Enemy, 1))
            .with(Bounty::default())
            .with(Facing::default())
            .with(FacingBroadcast::default())
            .with(TurnSpeed::default())
            .build();
        let plan = crate::runtime::resolve_td_layer_damage(
            omoba_template_ids::active_td_layer_catalog(),
            &state,
            Fixed64::from_i32(10),
            Fixed64::from_i32(10),
            crate::runtime::DamageProfile::new(AbiDamageProfile::TRUE.bits()).unwrap(),
            crate::runtime::HitProvenance {
                source_entity_id: 1,
                owner_player_id: Some(1),
                hit_serial: 1,
            },
        )
        .unwrap();
        assert_eq!(plan.pop_count(), 1);
        assert_eq!(
            plan.original.as_ref().unwrap().state.current_layer,
            "rainbow"
        );
        assert_eq!(plan.children.len(), 1);

        assert!(
            !commit_td_layer_plan(&mut world, target, &state, Fixed64::from_i32(10), &plan,)
                .unwrap()
        );
        world.maintain();
        let creeps = world.read_storage::<Creep>();
        let entities = world.entities();
        let layers: Vec<_> = (&entities, &creeps)
            .join()
            .map(|(entity, creep)| {
                (
                    entity.id(),
                    creep.td_layer.as_ref().unwrap().current_layer.clone(),
                    creep.td_layer.as_ref().unwrap().spawn_lineage,
                )
            })
            .collect();
        assert_eq!(layers.len(), 2, "no transient intermediate layer entities");
        assert_eq!(layers[0].0, target.id(), "first survivor reuses original");
        assert_eq!(layers[0].1, "rainbow");
        assert_eq!(layers[1].1, "rainbow");
        assert_ne!(layers[0].2, layers[1].2, "child lineage stays unique");
    }

    #[test]
    fn td_leak_value_saturates_lives_at_zero() {
        assert_eq!(apply_td_life_loss(100, 47), 53);
        assert_eq!(apply_td_life_loss(50, 616), 0);
        assert_eq!(apply_td_life_loss(0, i32::MAX), 0);
        assert_eq!(apply_td_life_loss(10, -1), 10);
    }

    #[test]
    fn creep_bounty_uses_active_template_rewards() {
        let id = CreepId(1);
        let name = creep_id_str(id);
        assert_ne!(name, "?");
        let stats = active_creep_stats(id).expect("generated creep stats");
        let bounty = creep_bounty_from_template(name);
        assert_eq!(bounty.gold, stats.gold_reward);
        assert_eq!(bounty.exp, stats.exp_reward);
        let td_bounty = creep_bounty_from_template("td_btd_red");
        assert_eq!(td_bounty.gold, 0);
        assert_eq!(td_bounty.exp, 0);
    }

    #[test]
    fn legacy_moba_minion_bounty_and_experience_stay_unchanged() {
        for name in ["melee_minion", "ranged_minion", "siege_minion"] {
            let id = omoba_template_ids::creep_by_name(name).expect("generated MOBA creep id");
            let stats =
                omoba_template_ids::active_creep_stats(id).expect("generated MOBA creep rewards");
            let bounty = creep_bounty_from_template(name);
            assert_eq!(bounty.gold, stats.gold_reward, "{name} gold");
            assert_eq!(bounty.exp, stats.exp_reward, "{name} exp");
        }
    }

    fn world_for_td_bounty_test(mode: GameMode) -> World {
        let mut world = World::new();
        world.register::<PlayerOwner>();
        world.register::<Bounty>();
        world.insert(mode);
        world.insert(Tick(0));
        world.insert(crate::runtime::TdEconomyRules::default());
        world.insert(crate::runtime::TdEconomyLedger::default());
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, 650);
        economy.initialize(2, 650);
        world.insert(economy);
        world
    }

    #[test]
    fn td_bounty_credits_owned_source_without_heroes() {
        let mut world = world_for_td_bounty_test(GameMode::TowerDefense);
        let source = world.create_entity().with(PlayerOwner::new(2)).build();
        let target = world
            .create_entity()
            .with(Bounty { gold: 25, exp: 7 })
            .build();

        assert!(credit_td_bounty_for_source(&mut world, source, target).unwrap());
        let economy = world.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(650));
        assert_eq!(economy.balance(2), Some(675));
    }

    #[test]
    fn td_bounty_without_owned_source_does_not_guess_player() {
        let mut world = world_for_td_bounty_test(GameMode::TowerDefense);
        let source = world.create_entity().build();
        let target = world
            .create_entity()
            .with(Bounty { gold: 25, exp: 7 })
            .build();

        assert!(!credit_td_bounty_for_source(&mut world, source, target).unwrap());
        let economy = world.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(650));
        assert_eq!(economy.balance(2), Some(650));
    }

    #[test]
    fn moba_bounty_does_not_use_td_player_economy() {
        let mut world = world_for_td_bounty_test(GameMode::Moba);
        let source = world.create_entity().with(PlayerOwner::new(1)).build();
        let target = world
            .create_entity()
            .with(Bounty { gold: 25, exp: 7 })
            .build();

        assert!(!credit_td_bounty_for_source(&mut world, source, target).unwrap());
        assert_eq!(world.read_resource::<PlayerEconomy>().balance(1), Some(650));
    }

    #[test]
    fn authoritative_road_clearance_uses_visual_placement_radius() {
        let clear = tower_path_clearance(90.0, 64.0);
        let clear_sq = clear * clear;
        let road_start = Vec2::new(-500.0, 0.0);
        let road_end = Vec2::new(500.0, 0.0);

        assert_eq!(clear, 154.0);
        assert!(point_segment_dist_sq(Vec2::new(0.0, 153.0), road_start, road_end) < clear_sq);
        assert!(point_segment_dist_sq(Vec2::new(0.0, 155.0), road_start, road_end) >= clear_sq);
    }

    fn world_for_script_outcome_tests() -> (World, Entity) {
        let mut world = World::new();
        world.register::<Pos>();
        world.register::<Facing>();
        world.register::<TAttack>();
        world.register::<Tower>();
        world.register::<Projectile>();
        world.register::<CProperty>();
        world.insert(Vec::<Outcome>::new());
        world.insert(Tick(7));
        world.insert(TowerFireFxQueue::default());
        world.insert(AttackPhaseFxQueue::default());
        world.insert(ExplosionFxQueue::default());
        world.insert(RemovedEntitiesQueue::default());
        world.insert(BuffStore::default());
        world.insert(ScriptEventQueue::default());
        world.insert(crate::runtime::ObservableFactBuffer::default());

        let entity = world
            .create_entity()
            .with(Pos(omoba_sim::Vec2::new(
                Fixed64::from_i32(1),
                Fixed64::from_i32(2),
            )))
            .with(Facing(omoba_sim::Angle::ZERO))
            .with(Tower::new())
            .with(TAttack::new(
                Fixed64::from_i32(10),
                Fixed64::from_i32(1),
                Fixed64::from_i32(100),
                Fixed64::from_i32(900),
            ))
            .build();
        (world, entity)
    }

    #[test]
    fn next_attack_bonus_consumes_only_valid_launch_and_never_visual_copies() {
        let (mut w, source) = world_for_script_outcome_tests();
        w.register::<IsBuilding>();
        w.insert(MasterSeed(42));
        let invalid_target = w.create_entity().build();
        let target = w.create_entity().with(Pos(omoba_sim::Vec2::ZERO)).build();
        {
            let mut buffs = w.write_resource::<BuffStore>();
            assert!(buffs.arm_next_attack_bonus(source, Fixed64::from_i32(60)));
            assert!(buffs.arm_next_attack_bonus(source, Fixed64::from_i32(20)));
            assert!(!buffs.arm_next_attack_bonus(source, Fixed64::ZERO));
            assert_eq!(buffs.next_attack_bonus(source), Fixed64::from_i32(60));
            buffs.add(source, "fixture_visual_copies", Fixed64::from_i32(10),
                json!({ StatKey::MultiShotVisual.as_str(): 3 * 1024 }));
        }
        assert!(handle_projectile(&mut w, omoba_sim::Vec2::ZERO, Some(source), Some(invalid_target)).is_err());
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source), Fixed64::from_i32(60));
        handle_projectile(&mut w, omoba_sim::Vec2::ZERO, Some(source), Some(target)).unwrap();
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source), Fixed64::ZERO);
        let damages: Vec<_> = w.read_storage::<Projectile>().join().map(|p| (p.target, p.damage_phys)).collect();
        assert_eq!(damages.iter().filter(|(t, d)| *t == Some(target) && *d == Fixed64::from_i32(70)).count(), 1);
        assert_eq!(damages.iter().filter(|(t, d)| t.is_none() && *d == Fixed64::ZERO).count(), 2);
        handle_projectile(&mut w, omoba_sim::Vec2::ZERO, Some(source), Some(target)).unwrap();
        assert_eq!(w.read_storage::<Projectile>().join().filter(|p| p.target == Some(target) && p.damage_phys == Fixed64::from_i32(10)).count(), 1);
        {
            let mut buffs = w.write_resource::<BuffStore>();
            buffs.arm_next_attack_bonus(source, Fixed64::from_i32(60));
            buffs.add(source, "fixture_miss", Fixed64::from_i32(10),
                json!({ StatKey::AccuracyBonus.as_str(): -1024 }));
        }
        handle_projectile(&mut w, omoba_sim::Vec2::ZERO, Some(source), Some(target)).unwrap();
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source), Fixed64::ZERO);
        assert_eq!(w.read_storage::<Projectile>().join().filter(|p| p.target == Some(target) && p.damage_phys == Fixed64::ZERO).count(), 1);
        w.write_resource::<BuffStore>().arm_next_attack_bonus(source, Fixed64::from_i32(60));
        w.write_resource::<BuffStore>().remove_all_for(source);
        assert_eq!(w.read_resource::<BuffStore>().next_attack_bonus(source), Fixed64::ZERO);
    }

    fn world_for_owner_tests() -> World {
        let mut world = World::new();
        world.register::<Hero>();
        world.register::<Tower>();
        world.register::<Faction>();
        world.register::<PlayerOwner>();
        world.register::<Gold>();
        world.register::<MoveTarget>();
        world.register::<HeroCommandQueue>();
        world.register::<Pos>();
        world.register::<ScriptUnitTag>();
        world.register::<TAttack>();
        world.register::<CProperty>();
        world.insert(PendingMoveQueue::default());
        world.insert(PendingTowerTargetPriorityQueue::default());
        world.insert(Tick(0));
        world.insert(BuffStore::default());
        world.insert(Vec::<Outcome>::new());
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, 1_000);
        economy.initialize(2, 1_000);
        world.insert(economy);
        world
    }

    fn test_tower_template(cost: i32) -> TowerTemplate {
        use crate::runtime::comp::tower_registry::{
            AttackTimingMetadata, TowerRecoil, TowerRenderAnimation, TowerRenderMetadata,
            TowerRenderPoint,
        };
        TowerTemplate {
            unit_id: "tower_dart".to_string(),
            label: "Dart".to_string(),
            atk: 10.0,
            asd_interval: 1.0,
            range: 350.0,
            bullet_speed: 900.0,
            splash_radius: 0.0,
            hit_radius: 0.0,
            slow_factor: 0.0,
            slow_duration: 0.0,
            cost,
            footprint: 30.0,
            placement_radius: 90.0,
            hp: 100.0,
            turn_speed_deg: 360.0,
            render: TowerRenderMetadata {
                render_mode: "base_barrel".to_string(),
                base: String::new(),
                barrel: String::new(),
                visual_size: 180.0,
                barrel_frames: Vec::new(),
                body_frames: Vec::new(),
                barrel_animation: TowerRenderAnimation {
                    fps: 0.0,
                    loop_animation: false,
                    fire_fps: 0.0,
                    fire_once: false,
                },
                body_animation: TowerRenderAnimation {
                    fps: 0.0,
                    loop_animation: false,
                    fire_fps: 0.0,
                    fire_once: false,
                },
                rotation_mode: "targeted".to_string(),
                barrel_layout: "single".to_string(),
                barrel_variants: Vec::new(),
                barrel_offset: TowerRenderPoint { x: 0.0, y: 0.0 },
                barrel_pivot: TowerRenderPoint { x: 0.5, y: 0.5 },
                muzzle_offset: TowerRenderPoint { x: 0.0, y: 0.0 },
                default_angle_deg: 0.0,
                recoil: TowerRecoil {
                    mode: String::new(),
                    distance: 0.0,
                    scale: 1.0,
                    duration_ms: 0,
                    return_ms: 0,
                },
            },
            attack_timing: AttackTimingMetadata {
                windup: 0,
                backswing: 0,
            },
        }
    }

    fn hero_free_tower_spawn_world(player_1_gold: i32, include_player_2: bool) -> World {
        let thread_pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&thread_pool);
        *world.write_resource::<GameMode>() = GameMode::TowerDefense;
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, player_1_gold);
        if include_player_2 {
            economy.initialize(2, 650);
        }
        world.insert(economy);
        let mut registry = TowerTemplateRegistry::default();
        registry.insert(test_tower_template(200));
        world.insert(registry);
        world
    }

    #[test]
    fn hero_free_tower_spawn_debits_only_requesting_player() {
        let mut world = hero_free_tower_spawn_world(650, true);
        let kind_id = omoba_template_ids::tower_by_name("tower_dart")
            .expect("tower_dart id")
            .0 as u32;

        let tower = handle_tower_spawn_from_input(
            &mut world,
            kind_id,
            omoba_sim::Vec2::new(Fixed64::from_i32(500), Fixed64::from_i32(500)),
            1,
        )
        .expect("hero-free player can place tower");

        assert!(world.read_storage::<Hero>().join().next().is_none());
        assert_eq!(
            world
                .read_storage::<PlayerOwner>()
                .get(tower)
                .unwrap()
                .player_id,
            1
        );
        let economy = world.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(450));
        assert_eq!(economy.balance(2), Some(650));
    }

    #[test]
    fn hero_free_tower_spawn_rejections_do_not_mutate_world() {
        let kind_id = omoba_template_ids::tower_by_name("tower_dart")
            .expect("tower_dart id")
            .0 as u32;
        for (mut world, player_id) in [
            (hero_free_tower_spawn_world(199, true), 1),
            (hero_free_tower_spawn_world(650, false), 2),
        ] {
            assert!(handle_tower_spawn_from_input(
                &mut world,
                kind_id,
                omoba_sim::Vec2::new(Fixed64::from_i32(500), Fixed64::from_i32(500)),
                player_id,
            )
            .is_err());
            assert_eq!(world.read_storage::<Tower>().join().count(), 0);
            assert_eq!(
                world.read_resource::<PlayerEconomy>().balance(1),
                Some(if player_id == 1 { 199 } else { 650 })
            );
        }
    }

    #[test]
    fn cake_splash_has_hero_knowledge_category() {
        assert_eq!(
            hero_knowledge_category_for_unit_id("tower_cake_splash"),
            "tower_cake_splash"
        );
    }

    fn add_owned_hero(world: &mut World, player_id: u32, name: &str) -> Entity {
        world
            .create_entity()
            .with(Hero {
                name: format!("[P{}] {}", player_id, name),
                ..Hero::default()
            })
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(player_id))
            .with(Gold(1000))
            .build()
    }

    #[test]
    fn player_hero_entity_uses_owner_not_team_id() {
        let mut world = world_for_owner_tests();
        let p1 = add_owned_hero(&mut world, 1, "Hero");
        let p2 = add_owned_hero(&mut world, 2, "Hero");

        assert_eq!(player_hero_entity(&world, "test", 1).unwrap(), p1);
        assert_eq!(player_hero_entity(&world, "test", 2).unwrap(), p2);
        assert_eq!(
            world.read_storage::<Faction>().get(p1).unwrap().team_id,
            world.read_storage::<Faction>().get(p2).unwrap().team_id
        );
        assert!(world
            .read_storage::<Hero>()
            .get(p1)
            .unwrap()
            .name
            .starts_with("[P1]"));
        assert!(world
            .read_storage::<Hero>()
            .get(p2)
            .unwrap()
            .name
            .starts_with("[P2]"));
    }

    #[test]
    fn drain_pending_moves_routes_each_player_to_own_hero() {
        let mut world = world_for_owner_tests();
        let p1 = add_owned_hero(&mut world, 1, "Hero");
        let p2 = add_owned_hero(&mut world, 2, "Hero");
        {
            let mut q = world.write_resource::<PendingMoveQueue>();
            q.requests.push(crate::comp::PendingHeroCommand { admitted_correlation: 0,
                owner_pid: 2,
                queued: false,
                kind: crate::comp::PendingHeroCommandKind::MoveTo {
                    pos: omoba_sim::Vec2::new(Fixed64::from_i32(10), Fixed64::from_i32(20)),
                },
            });
        }

        drain_pending_moves(&mut world);

        assert!(world.read_storage::<HeroCommandQueue>().get(p1).is_none());
        assert!(matches!(
            world
                .read_storage::<HeroCommandQueue>()
                .get(p2)
                .and_then(|q| q.active),
            Some(HeroCommand::MoveTo { .. })
        ));
    }

    #[test]
    fn queued_hero_commands_cap_at_sixteen_and_nonqueued_replaces_all() {
        let mut world = world_for_owner_tests();
        let hero = add_owned_hero(&mut world, 1, "Hero");
        {
            let mut q = world.write_resource::<PendingMoveQueue>();
            for i in 0..(HeroCommandQueue::LIMIT + 1) {
                q.requests.push(crate::comp::PendingHeroCommand { admitted_correlation: 0,
                    owner_pid: 1,
                    queued: true,
                    kind: crate::comp::PendingHeroCommandKind::MoveTo {
                        pos: omoba_sim::Vec2::new(
                            Fixed64::from_i32(i as i32),
                            Fixed64::from_i32(0),
                        ),
                    },
                });
            }
        }

        drain_pending_moves(&mut world);

        assert_eq!(
            world
                .read_storage::<HeroCommandQueue>()
                .get(hero)
                .unwrap()
                .total_len(),
            HeroCommandQueue::LIMIT
        );

        {
            let _ = world.write_storage::<MoveTarget>().insert(
                hero,
                MoveTarget(omoba_sim::Vec2::new(Fixed64::ONE, Fixed64::ONE)),
            );
            let mut q = world.write_resource::<PendingMoveQueue>();
            q.requests.push(crate::comp::PendingHeroCommand { admitted_correlation: 0,
                owner_pid: 1,
                queued: false,
                kind: crate::comp::PendingHeroCommandKind::AttackMove {
                    pos: omoba_sim::Vec2::new(Fixed64::from_i32(50), Fixed64::from_i32(0)),
                },
            });
        }

        drain_pending_moves(&mut world);

        let queues = world.read_storage::<HeroCommandQueue>();
        let queue = queues.get(hero).unwrap();
        assert_eq!(queue.total_len(), 1);
        assert!(matches!(queue.active, Some(HeroCommand::AttackMove { .. })));
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
    }

    #[test]
    fn attack_target_rejects_allied_target_without_replacing_queue() {
        let mut world = world_for_owner_tests();
        let hero = add_owned_hero(&mut world, 1, "Hero");
        let allied = world
            .create_entity()
            .with(Faction::new(FactionType::Player, 0))
            .build();
        {
            let _ = world.write_storage::<HeroCommandQueue>().insert(
                hero,
                HeroCommandQueue {
                    active: Some(HeroCommand::MoveTo {
                        pos: omoba_sim::Vec2::new(Fixed64::from_i32(1), Fixed64::from_i32(0)),
                    }),
                    queued: Vec::new(),
                },
            );
            let mut q = world.write_resource::<PendingMoveQueue>();
            q.requests.push(crate::comp::PendingHeroCommand { admitted_correlation: 0,
                owner_pid: 1,
                queued: false,
                kind: crate::comp::PendingHeroCommandKind::AttackTarget {
                    target_entity_id: allied.id(),
                },
            });
        }

        drain_pending_moves(&mut world);

        assert!(matches!(
            world
                .read_storage::<HeroCommandQueue>()
                .get(hero)
                .and_then(|q| q.active),
            Some(HeroCommand::MoveTo { .. })
        ));
    }

    #[test]
    fn accepted_attack_move_does_not_cancel_backswing() {
        let mut world = world_for_owner_tests();
        let hero = add_owned_hero(&mut world, 1, "Hero");
        {
            let mut attack = TAttack::new(
                Fixed64::from_i32(10),
                Fixed64::from_i32(1),
                Fixed64::from_i32(100),
                Fixed64::from_i32(900),
            );
            attack.attack_phase = AttackSequencePhase::Backswing;
            attack.asd_count = Fixed64::from_raw(123);
            let _ = world.write_storage::<TAttack>().insert(hero, attack);

            let mut q = world.write_resource::<PendingMoveQueue>();
            q.requests.push(crate::comp::PendingHeroCommand { admitted_correlation: 0,
                owner_pid: 1,
                queued: false,
                kind: crate::comp::PendingHeroCommandKind::AttackMove {
                    pos: omoba_sim::Vec2::new(Fixed64::from_i32(50), Fixed64::from_i32(0)),
                },
            });
        }

        drain_pending_moves(&mut world);

        let attack = *world.read_storage::<TAttack>().get(hero).unwrap();
        assert_eq!(attack.attack_phase, AttackSequencePhase::Backswing);
        assert_eq!(attack.asd_count, Fixed64::from_raw(123));
    }

    fn seed_hold(world: &mut World, hero: Entity) {
        let _ = world.write_storage::<MoveTarget>().insert(
            hero,
            MoveTarget(omoba_sim::Vec2::new(Fixed64::from_i32(9), Fixed64::from_i32(4))),
        );
        let _ = world.write_storage::<HeroCommandQueue>().insert(
            hero,
            HeroCommandQueue {
                active: Some(HeroCommand::HoldPosition),
                queued: vec![HeroCommand::MoveTo {
                    pos: omoba_sim::Vec2::new(Fixed64::from_i32(3), Fixed64::ZERO),
                }],
            },
        );
    }

    fn push_kind(world: &mut World, owner: u32, queued: bool, kind: PendingHeroCommandKind) {
        world.write_resource::<PendingMoveQueue>().requests.push(crate::comp::PendingHeroCommand { admitted_correlation: 0,
            owner_pid: owner,
            queued,
            kind,
        });
    }

    fn held(world: &World, hero: Entity) -> bool {
        world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| {
            queue.active == Some(HeroCommand::HoldPosition) && queue.queued.len() == 1
        }) && world.read_storage::<MoveTarget>().get(hero).is_some()
    }

    fn priority(world: &World, hero: Entity) -> bool {
        crate::runtime::command_claims_movement_priority(
            world.read_storage::<MoveTarget>().get(hero).is_some(),
            world.read_storage::<HeroCommandQueue>().get(hero).and_then(|queue| queue.active),
        )
    }

    #[test]
    fn stop_command_clears_move_target_current_and_queued() {
        let mut world = world_for_owner_tests();
        let first = add_owned_hero(&mut world, 1, "First");
        let second = add_owned_hero(&mut world, 2, "Second");
        seed_hold(&mut world, first);
        seed_hold(&mut world, second);
        push_kind(&mut world, 1, false, PendingHeroCommandKind::MoveTo {
            pos: omoba_sim::Vec2::new(Fixed64::from_i32(8), Fixed64::ZERO),
        });
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        push_kind(&mut world, 2, false, PendingHeroCommandKind::Stop);
        push_kind(&mut world, 2, false, PendingHeroCommandKind::MoveTo {
            pos: omoba_sim::Vec2::new(Fixed64::from_i32(6), Fixed64::ZERO),
        });
        drain_pending_moves(&mut world);
        assert!(world.read_storage::<MoveTarget>().get(first).is_none());
        assert!(world.read_storage::<HeroCommandQueue>().get(first).is_some_and(|queue| {
            queue.active.is_none() && queue.queued.is_empty()
        }));
        assert!(!priority(&world, first));
        assert!(matches!(
            world.read_storage::<HeroCommandQueue>().get(second).and_then(|queue| queue.active),
            Some(HeroCommand::MoveTo { .. })
        ));
        assert!(world.read_storage::<MoveTarget>().get(second).is_none());
        assert!(!priority(&world, second));
    }

    #[test]
    fn stop_command_hold_stays_distinct_until_a_later_command() {
        let mut world = world_for_owner_tests();
        let hero = add_owned_hero(&mut world, 1, "Hero");
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        push_kind(&mut world, 1, false, PendingHeroCommandKind::HoldPosition);
        drain_pending_moves(&mut world);
        assert_eq!(
            world.read_storage::<HeroCommandQueue>().get(hero).and_then(|queue| queue.active),
            Some(HeroCommand::HoldPosition)
        );
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
        assert!(priority(&world, hero));
        assert!(crate::runtime::command_claims_movement_priority(false, Some(HeroCommand::HoldPosition)));
        assert!(!crate::runtime::command_claims_movement_priority(false, Some(HeroCommand::MoveTo {
            pos: omoba_sim::Vec2::ZERO,
        })));
        push_kind(&mut world, 1, false, PendingHeroCommandKind::MoveTo {
            pos: omoba_sim::Vec2::new(Fixed64::from_i32(4), Fixed64::ZERO),
        });
        drain_pending_moves(&mut world);
        assert!(matches!(
            world.read_storage::<HeroCommandQueue>().get(hero).and_then(|queue| queue.active),
            Some(HeroCommand::MoveTo { .. })
        ));
        assert!(!priority(&world, hero));
        push_kind(&mut world, 1, false, PendingHeroCommandKind::HoldPosition);
        drain_pending_moves(&mut world);
        assert!(priority(&world, hero));
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| queue.active.is_none()));
        assert!(!priority(&world, hero));
    }

    #[test]
    fn stop_command_cancels_windup_and_preserves_idle_cooldown() {
        let mut world = world_for_owner_tests();
        world.insert(AttackCancelFxQueue::default());
        let windup_hero = add_owned_hero(&mut world, 1, "Windup");
        let idle_hero = add_owned_hero(&mut world, 2, "Idle");
        let mut windup = TAttack::new(
            Fixed64::from_i32(10), Fixed64::from_i32(2), Fixed64::from_i32(100), Fixed64::from_i32(20),
        );
        let cooldown = windup.asd.v;
        let seq = windup.begin_attack_windup();
        windup.asd_count = Fixed64::from_i32(-1);
        let _ = world.write_storage::<TAttack>().insert(windup_hero, windup);
        let mut idle = TAttack::new(
            Fixed64::from_i32(10), Fixed64::from_i32(2), Fixed64::from_i32(100), Fixed64::from_i32(20),
        );
        idle.asd_count = Fixed64::from_raw(400);
        let idle_count = idle.asd_count;
        let _ = world.write_storage::<TAttack>().insert(idle_hero, idle);
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        push_kind(&mut world, 2, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        let windup = *world.read_storage::<TAttack>().get(windup_hero).unwrap();
        assert_eq!(windup.attack_phase, AttackSequencePhase::Idle);
        assert_eq!(windup.asd_count, cooldown);
        let idle = *world.read_storage::<TAttack>().get(idle_hero).unwrap();
        assert_eq!(idle.attack_phase, AttackSequencePhase::Idle);
        assert_eq!(idle.asd_count, idle_count);
        let fx = world.read_resource::<AttackCancelFxQueue>();
        assert_eq!(fx.pending.len(), 1);
        assert_eq!(fx.pending[0].entity_id, windup_hero.id());
        assert_eq!(fx.pending[0].phase, AttackCancelPhase::Windup);
        assert!(!fx.pending[0].impact_committed);
        assert_eq!(fx.pending[0].attack_seq, seq);
    }

    #[test]
    fn stop_command_backswing_keeps_committed_projectile() {
        let mut world = world_for_owner_tests();
        world.register::<Projectile>();
        world.insert(AttackCancelFxQueue::default());
        let hero = add_owned_hero(&mut world, 1, "Hero");
        let mut attack = TAttack::new(
            Fixed64::from_i32(10), Fixed64::from_i32(2), Fixed64::from_i32(100), Fixed64::from_i32(20),
        );
        attack.begin_attack_windup();
        attack.mark_attack_impact();
        attack.asd_count = Fixed64::from_raw(123);
        let _ = world.write_storage::<TAttack>().insert(hero, attack);
        let shot = Projectile {
            damage_stage: Default::default(),
            time_left: Fixed64::from_i32(5),
            owner: hero,
            target: None,
            tpos: omoba_sim::Vec2::new(Fixed64::from_i32(8), Fixed64::from_i32(9)),
            radius: Fixed64::ONE,
            msd: Fixed64::from_i32(12),
            damage_phys: Fixed64::from_i32(77),
            damage_magi: Fixed64::from_i32(3),
            damage_real: Fixed64::from_i32(1),
            damage_profile: 4,
            slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO,
            hit_radius: Fixed64::ZERO,
            stun_duration: Fixed64::ZERO,
            kind_id: 9,
            generation: 1,
        };
        let projectile = world.create_entity().with(shot).build();
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        let attack = *world.read_storage::<TAttack>().get(hero).unwrap();
        assert_eq!(attack.attack_phase, AttackSequencePhase::Idle);
        let projectiles = world.read_storage::<Projectile>();
        let kept = projectiles.get(projectile).unwrap();
        assert_eq!(kept.damage_phys, Fixed64::from_i32(77));
        assert_eq!(kept.damage_magi, Fixed64::from_i32(3));
        assert_eq!(kept.damage_real, Fixed64::from_i32(1));
        assert_eq!(kept.time_left, Fixed64::from_i32(5));
        assert_eq!(kept.owner, hero);
        assert_eq!(kept.kind_id, 9);
        assert_eq!(world.read_storage::<Projectile>().join().count(), 1);
        let fx = world.read_resource::<AttackCancelFxQueue>();
        assert_eq!(fx.pending.len(), 1);
        assert_eq!(fx.pending[0].phase, AttackCancelPhase::Backswing);
        assert!(fx.pending[0].impact_committed);
    }

    #[test]
    fn stop_command_rejects_dead_wrong_owner_and_queued() {
        let mut world = world_for_owner_tests();
        let hero = add_owned_hero(&mut world, 1, "Hero");
        seed_hold(&mut world, hero);
        push_kind(&mut world, 1, true, PendingHeroCommandKind::Stop);
        push_kind(&mut world, 99, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        assert!(held(&world, hero));
        world.insert(GamePause { is_paused: true });
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        assert!(held(&world, hero));
        {
            world.write_resource::<GamePause>().is_paused = false;
        }
        let _ = world.write_storage::<CProperty>().insert(hero, CProperty {
            hp: Fixed64::ZERO,
            mhp: Fixed64::from_i32(100),
            msd: Fixed64::ONE,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        });
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        assert!(held(&world, hero));
        {
            let mut properties = world.write_storage::<CProperty>();
            properties.get_mut(hero).unwrap().hp = Fixed64::from_i32(50);
        }
        push_kind(&mut world, 1, false, PendingHeroCommandKind::Stop);
        drain_pending_moves(&mut world);
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_some_and(|queue| {
            queue.active.is_none() && queue.queued.is_empty()
        }));
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
        assert!(!priority(&world, hero));
    }

    fn add_owned_tower(world: &mut World, player_id: u32) -> Entity {
        world
            .create_entity()
            .with(Tower::new())
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(player_id))
            .with(ScriptUnitTag {
                unit_id: "tower_dart".to_string(),
            })
            .build()
    }

    #[test]
    fn tower_target_priority_updates_only_for_owner() {
        let mut world = world_for_owner_tests();
        add_owned_hero(&mut world, 1, "Hero");
        add_owned_hero(&mut world, 2, "Hero");
        let tower = add_owned_tower(&mut world, 1);

        assert!(handle_tower_target_priority_from_input(
            &mut world,
            tower.id(),
            TowerTargetPriority::LowestHealth,
            2,
        )
        .is_err());
        assert_eq!(
            world
                .read_storage::<Tower>()
                .get(tower)
                .unwrap()
                .target_priority,
            TowerTargetPriority::First
        );

        handle_tower_target_priority_from_input(
            &mut world,
            tower.id(),
            TowerTargetPriority::HighestHealth,
            1,
        )
        .expect("owner may update priority");
        assert_eq!(
            world
                .read_storage::<Tower>()
                .get(tower)
                .unwrap()
                .target_priority,
            TowerTargetPriority::HighestHealth
        );
    }

    #[test]
    fn tower_sell_and_upgrade_reject_non_owner_before_state_change() {
        let mut world = world_for_owner_tests();
        let tower = add_owned_tower(&mut world, 2);

        assert!(handle_tower_sell_from_input(&mut world, tower.id(), 1).is_err());
        assert!(world.read_resource::<Vec<Outcome>>().is_empty());
        assert!(handle_tower_upgrade_from_input(&mut world, tower.id(), 0, 1, 1).is_err());
        assert_eq!(
            world
                .read_storage::<Tower>()
                .get(tower)
                .unwrap()
                .upgrade_levels,
            [0; 3]
        );
        let economy = world.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(1_000));
        assert_eq!(economy.balance(2), Some(1_000));
    }

    #[test]
    fn hero_free_upgrade_and_sell_use_owner_economy() {
        let mut world = world_for_owner_tests();
        let upgrades = TowerUpgradeRegistry::new();
        let upgrade_cost = upgrades.get("tower_dart", 0, 1).unwrap().cost;
        world.insert(upgrades);
        let mut templates = TowerTemplateRegistry::default();
        templates.insert(test_tower_template(200));
        world.insert(templates);
        let tower = add_owned_tower(&mut world, 1);

        handle_tower_upgrade_from_input(&mut world, tower.id(), 0, 1, 1)
            .expect("hero-free owner can upgrade");
        assert_eq!(
            world.read_resource::<PlayerEconomy>().balance(1),
            Some(1_000 - upgrade_cost)
        );

        handle_tower_sell_from_input(&mut world, tower.id(), 1).expect("hero-free owner can sell");
        let expected_refund =
            crate::runtime::TdEconomyRules::default().sell_refund(200 + upgrade_cost);
        assert_eq!(
            world.read_resource::<PlayerEconomy>().balance(1),
            Some(1_000 - upgrade_cost + expected_refund)
        );
        assert_eq!(
            world.read_resource::<PlayerEconomy>().balance(2),
            Some(1_000)
        );
        assert!(world.read_resource::<Vec<Outcome>>().iter().any(
            |outcome| matches!(outcome, Outcome::EntityRemoved { entity } if *entity == tower)
        ));
    }

    #[test]
    fn dart_range_upgrade_updates_effective_attack_range() {
        let mut world = world_for_owner_tests();
        world.insert(TowerUpgradeRegistry::new());
        let hero = add_owned_hero(&mut world, 1, "Hero");
        let tower = add_owned_tower(&mut world, 1);
        world
            .write_storage::<TAttack>()
            .insert(
                tower,
                TAttack::new(
                    Fixed64::from_i32(10),
                    Fixed64::from_i32(1),
                    Fixed64::from_i32(350),
                    Fixed64::from_i32(900),
                ),
            )
            .unwrap();

        handle_tower_upgrade_from_input(&mut world, tower.id(), 0, 1, 1)
            .expect("owner can buy dart path 0 level 1");

        let buff_store = world.read_resource::<BuffStore>();
        let stats = UnitStats::from_refs(&*buff_store, true);
        let attack = world.read_storage::<TAttack>();
        let range = stats
            .final_attack_range(attack.get(tower).unwrap().range.v, tower)
            .to_f32_for_render();
        assert_eq!(range, 400.0);
        assert_eq!(world.read_storage::<Gold>().get(hero).unwrap().0, 1_000);
        assert!(world.read_resource::<PlayerEconomy>().balance(1).unwrap() < 1_000);
    }

    #[test]
    fn boomerang_path_two_level_four_unlocks_active_ability() {
        let mut world = world_for_owner_tests();
        world.insert(TowerUpgradeRegistry::new());
        add_owned_hero(&mut world, 1, "Hero");
        world
            .write_resource::<PlayerEconomy>()
            .initialize(1, 10_000);
        let tower = world
            .create_entity()
            .with(Tower::new())
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(1))
            .with(ScriptUnitTag {
                unit_id: "tower_boomerang".to_string(),
            })
            .build();

        for _ in 0..4 {
            handle_tower_upgrade_from_input(&mut world, tower.id(), 1, 1, 1)
                .expect("owner can finish boomerang path two");
        }

        let towers = world.read_storage::<Tower>();
        let active = towers
            .get(tower)
            .and_then(|tower| tower.active_ability.as_ref())
            .expect("boomerang path two level four should unlock its active ability");
        assert_eq!(active.ability_id, "boomerang_turbo_charge");
    }

    #[test]
    fn script_outcomes_apply_in_order() {
        let (mut world, entity) = world_for_script_outcome_tests();
        let pos = omoba_sim::Vec2::new(Fixed64::from_i32(5), Fixed64::from_i32(6));
        let facing = omoba_sim::Angle::from_degrees_i32(90);
        world.write_resource::<Vec<Outcome>>().extend([
            Outcome::ScriptSetAsdCount {
                entity,
                asd_count: Fixed64::from_raw(111),
            },
            Outcome::ScriptSetAsdCount {
                entity,
                asd_count: Fixed64::from_raw(222),
            },
            Outcome::ScriptSetPos { entity, pos },
            Outcome::ScriptSetFacing { entity, facing },
            Outcome::ScriptTowerFireFx {
                entity,
                dir_rad: 1.25,
            },
            Outcome::ScriptProjectile {
                basic_attack:false,source_missed:false,source_critical:false,
                pos,
                owner: entity,
                target: None,
                tpos: omoba_sim::Vec2::new(Fixed64::from_i32(10), Fixed64::from_i32(6)),
                radius: Fixed64::ZERO,
                msd: Fixed64::from_i32(900),
                damage_phys: Fixed64::from_i32(10),
                damage_magi: Fixed64::ZERO,
                damage_real: Fixed64::ZERO,
                damage_profile: AbiDamageProfile::NORMAL.bits(),
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                hit_radius: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: 0,
                generation: 0,
            },
        ]);

        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        process_outcomes(&mut world, &mut sink).expect("script outcomes apply");

        assert_eq!(world.read_storage::<Pos>().get(entity).unwrap().0, pos);
        assert_eq!(
            world.read_storage::<Facing>().get(entity).unwrap().0,
            facing
        );
        assert_eq!(
            world
                .read_storage::<TAttack>()
                .get(entity)
                .unwrap()
                .asd_count,
            Fixed64::from_raw(222)
        );
        assert_eq!(
            (&world.entities(), &world.read_storage::<Projectile>())
                .join()
                .count(),
            1
        );
        assert_eq!(world.read_resource::<TowerFireFxQueue>().pending.len(), 1);
    }

    #[test]
    fn script_direct_damage_and_heal_match_adapter_semantics() {
        let (mut world, entity) = world_for_script_outcome_tests();
        let target = world
            .create_entity()
            .with(CProperty {
                hp: Fixed64::from_i32(30),
                mhp: Fixed64::from_i32(40),
                msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO,
                def_magic: Fixed64::ZERO,
            })
            .build();
        world.write_resource::<Vec<Outcome>>().extend([
            Outcome::ScriptDirectDamage {
                target,
                amount: Fixed64::from_i32(50),
            },
            Outcome::ScriptHeal {
                target,
                amount: Fixed64::from_i32(15),
            },
            Outcome::ScriptRemoveBuff {
                target: entity,
                buff_id: "missing".to_string(),
            },
        ]);

        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        process_outcomes(&mut world, &mut sink).expect("damage and heal apply");

        assert_eq!(
            world.read_storage::<CProperty>().get(target).unwrap().hp,
            Fixed64::ZERO // Ordinary healing cannot reverse committed lethal damage.
        );
        let live=world.create_entity().with(CProperty {hp:Fixed64::from_i32(30),mhp:Fixed64::from_i32(40),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        world.write_resource::<Vec<Outcome>>().extend([
            Outcome::ScriptDirectDamage {target:live,amount:Fixed64::from_i32(10)},
            Outcome::ScriptHeal {target:live,amount:Fixed64::from_i32(15)}]);
        process_outcomes(&mut world,&mut sink).expect("living target still heals normally");
        assert_eq!(world.read_storage::<CProperty>().get(live).unwrap().hp,Fixed64::from_i32(35));
    }

    fn world_for_tower_ability_cast_tests() -> (World, Entity) {
        let mut world = world_for_owner_tests();
        world.insert(TowerUpgradeRegistry::new());
        world.insert(GameMode::TowerDefense);
        world.insert(PlayerLives(100));
        world.insert(PendingTowerAbilityCastQueue::default());
        world.insert(PendingTowerAbilityActivationQueue::default());
        world.insert(TowerAbilityCastResult::default());
        world.insert(TowerAbilityCastResults::default());

        let tower = world
            .create_entity()
            .with(Tower {
                upgrade_levels: [0, 4, 0],
                active_ability: Some(TowerActiveAbilityState::ready("boomerang_turbo_charge")),
                ..Tower::new()
            })
            .with(Faction::new(FactionType::Player, 0))
            .with(PlayerOwner::new(7))
            .with(ScriptUnitTag {
                unit_id: "tower_boomerang".to_string(),
            })
            .build();
        (world, tower)
    }

    fn tower_ability_state_snapshot(world: &World, tower: Entity) -> serde_json::Value {
        serde_json::to_value(
            world
                .read_storage::<Tower>()
                .get(tower)
                .unwrap()
                .active_ability
                .as_ref()
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn tower_ability_cast_accepts_owned_unlocked_ready_tower() {
        let (mut world, tower) = world_for_tower_ability_cast_tests();

        assert!(handle_tower_ability_cast_from_input(
            &mut world,
            tower.id(),
            "boomerang_turbo_charge",
            7,
        )
        .is_ok());

        let towers = world.read_storage::<Tower>();
        let state = towers.get(tower).unwrap().active_ability.as_ref().unwrap();
        assert_eq!(state.cooldown_remaining, Fixed64::from_i32(10));
        assert_eq!(state.active_remaining, Fixed64::from_i32(5));
        assert_eq!(state.activation_serial, 1);
        drop(towers);
        let callbacks = world.read_resource::<PendingTowerAbilityActivationQueue>();
        assert_eq!(callbacks.requests.len(), 1);
        assert_eq!(callbacks.requests[0].entity, tower);
    }

    #[test]
    fn tower_ability_cast_rejections_preserve_state() {
        for expected in ["not_owner", "tower_missing", "ability_mismatch"] {
            let (mut world, tower) = world_for_tower_ability_cast_tests();
            let (tower_id, ability_id, requester) = match expected {
                "not_owner" => (tower.id(), "boomerang_turbo_charge", 8),
                "tower_missing" => (u32::MAX, "boomerang_turbo_charge", 7),
                "ability_mismatch" => (tower.id(), "wrong_ability", 7),
                _ => unreachable!(),
            };
            let before = tower_ability_state_snapshot(&world, tower);
            let error =
                handle_tower_ability_cast_from_input(&mut world, tower_id, ability_id, requester)
                    .unwrap_err();
            assert_eq!(error.to_string(), expected);
            assert_eq!(tower_ability_state_snapshot(&world, tower), before);
        }
    }

    #[test]
    fn tower_ability_cast_locked_cooldown_and_game_ended_preserve_state() {
        for expected in ["ability_locked", "cooldown_active", "game_ended"] {
            let (mut world, tower) = world_for_tower_ability_cast_tests();
            match expected {
                "ability_locked" => {
                    world
                        .write_storage::<Tower>()
                        .get_mut(tower)
                        .unwrap()
                        .upgrade_levels = [0; 3];
                }
                "cooldown_active" => {
                    world
                        .write_storage::<Tower>()
                        .get_mut(tower)
                        .unwrap()
                        .active_ability
                        .as_mut()
                        .unwrap()
                        .cooldown_remaining = Fixed64::ONE;
                }
                "game_ended" => *world.write_resource::<PlayerLives>() = PlayerLives(0),
                _ => unreachable!(),
            }
            let before = tower_ability_state_snapshot(&world, tower);
            let error = handle_tower_ability_cast_from_input(
                &mut world,
                tower.id(),
                "boomerang_turbo_charge",
                7,
            )
            .unwrap_err();
            assert_eq!(error.to_string(), expected);
            assert_eq!(tower_ability_state_snapshot(&world, tower), before);
        }
    }

    #[test]
    fn tower_ability_cast_drain_records_every_processed_result_serial() {
        let (mut world, tower) = world_for_tower_ability_cast_tests();
        {
            let mut queue = world.write_resource::<PendingTowerAbilityCastQueue>();
            queue.requests.push(crate::comp::PendingTowerAbilityCast {
                tower_entity_id: u32::MAX,
                ability_id: "boomerang_turbo_charge".to_string(),
                owner_pid: 7,
            });
            queue.requests.push(crate::comp::PendingTowerAbilityCast {
                tower_entity_id: tower.id(),
                ability_id: "boomerang_turbo_charge".to_string(),
                owner_pid: 7,
            });
        }

        drain_pending_tower_ability_casts(&mut world);

        let result = world.read_resource::<TowerAbilityCastResult>();
        assert_eq!(result.result_serial, 2);
        assert!(result.accepted);
        assert_eq!(result.player_id, 7);
        assert_eq!(result.tower_entity_id, tower.id());
        assert_eq!(result.reason, "");
    }

    #[test]
    fn tower_ability_cast_drain_retains_latest_result_for_each_player() {
        let (mut world, _) = world_for_tower_ability_cast_tests();
        {
            let mut queue = world.write_resource::<PendingTowerAbilityCastQueue>();
            queue.requests.push(crate::comp::PendingTowerAbilityCast {
                tower_entity_id: u32::MAX,
                ability_id: "ability_seven".to_string(),
                owner_pid: 7,
            });
            queue.requests.push(crate::comp::PendingTowerAbilityCast {
                tower_entity_id: u32::MAX - 1,
                ability_id: "ability_eight".to_string(),
                owner_pid: 8,
            });
        }

        drain_pending_tower_ability_casts(&mut world);

        let results = world.read_resource::<TowerAbilityCastResults>();
        assert_eq!(results.latest_by_player.len(), 2);
        assert_eq!(results.latest_by_player[&7].ability_id, "ability_seven");
        assert_eq!(results.latest_by_player[&7].result_serial, 1);
        assert_eq!(results.latest_by_player[&8].ability_id, "ability_eight");
        assert_eq!(results.latest_by_player[&8].result_serial, 2);
    }

    #[test]
    fn projectile_hit_outcome_queues_provenance_aware_script_event() {
        let (mut world, source) = world_for_script_outcome_tests();
        let target = world.create_entity().build();
        let hit = Outcome::ProjectileHit {source,target,kind_id:41,generation:2};
        let fact = observable_fact_from_outcome(&hit,17,3).unwrap();
        assert_eq!(fact.key.fact_kind,crate::runtime::FactKind::ProjectileImpact);
        assert_eq!(fact.fact,crate::runtime::ObservableFact::ProjectileImpact {
            target:crate::runtime::canonical_entity_id(target)});
        let removed = Outcome::Death {ent:source, pos:Default::default()};
        assert!(!matches!(observable_fact_from_outcome(&removed,17,4).unwrap().fact,
            crate::runtime::ObservableFact::ProjectileImpact {..}));
        world
            .write_resource::<Vec<Outcome>>()
            .push(Outcome::ProjectileHit {
                source,
                target,
                kind_id: 41,
                generation: 2,
            });

        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        process_outcomes(&mut world, &mut sink).expect("projectile hit outcome applies");

        let facts = world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap();
        assert_eq!(facts.len(),1);
        assert_eq!(facts[0].key.tick,7);
        assert_eq!(facts[0].fact,crate::runtime::ObservableFact::ProjectileImpact {
            target:crate::runtime::canonical_entity_id(target)});

        let events = world.write_resource::<ScriptEventQueue>().drain();
        assert!(matches!(
            events.as_slice(),
            [ScriptEvent::ProjectileHit {
                attacker,
                victim,
                kind_id: 41,
                generation: 2,
                origin: None,
            }] if *attacker == source && *victim == target
        ));
    }
}

pub fn handle_tower_ability_cast_from_input(
    world: &mut World,
    tower_entity_id: u32,
    ability_id: &str,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    let target = {
        let entities = world.entities();
        let towers = world.read_storage::<Tower>();
        let tags = world.read_storage::<ScriptUnitTag>();
        (&entities, &towers)
            .join()
            .find(|(entity, _)| entity.id() == tower_entity_id)
            .map(|(entity, tower)| {
                (
                    entity,
                    tower.upgrade_levels,
                    tags.get(entity).map(|tag| tag.unit_id.clone()),
                )
            })
    }
    .ok_or_else(|| failure::err_msg("tower_missing"))?;
    let (target_entity, levels, unit_id) = target;

    let owns_tower = world
        .read_storage::<PlayerOwner>()
        .get(target_entity)
        .is_some_and(|owner| owner.player_id == owner_pid);
    if !owns_tower {
        return Err(failure::err_msg("not_owner"));
    }

    let unlocked = unit_id.as_deref().and_then(|unit_id| {
        let registry = world.read_resource::<TowerUpgradeRegistry>();
        levels
            .iter()
            .enumerate()
            .filter(|(_, level)| **level >= 4)
            .find_map(|(path, _)| {
                registry
                    .get(unit_id, path as u8, 4)
                    .and_then(|def| def.active_ability.clone())
            })
    });
    let def = unlocked.ok_or_else(|| failure::err_msg("ability_locked"))?;

    let state_snapshot = world
        .read_storage::<Tower>()
        .get(target_entity)
        .and_then(|tower| tower.active_ability.as_ref())
        .map(|state| (state.ability_id.clone(), state.cooldown_remaining));
    let Some((state_ability_id, cooldown_remaining)) = state_snapshot else {
        return Err(failure::err_msg("ability_locked"));
    };
    if def.ability_id != ability_id || state_ability_id != ability_id {
        return Err(failure::err_msg("ability_mismatch"));
    }
    if cooldown_remaining > Fixed64::ZERO {
        return Err(failure::err_msg("cooldown_active"));
    }

    let game_ended = world.read_resource::<crate::comp::GameMode>().is_td()
        && world.read_resource::<crate::comp::PlayerLives>().0 <= 0;
    if game_ended {
        return Err(failure::err_msg("game_ended"));
    }

    let activation_serial = {
        let mut towers = world.write_storage::<Tower>();
        let state = towers
            .get_mut(target_entity)
            .and_then(|tower| tower.active_ability.as_mut())
            .ok_or_else(|| failure::err_msg("ability_locked"))?;
        state
            .activate(
                def.cooldown,
                def.duration,
                def.pulse_interval,
                def.pulse_count,
            )
            .map_err(|_| failure::err_msg("cooldown_active"))?;
        state.activation_serial
    };
    world
        .write_resource::<PendingTowerAbilityActivationQueue>()
        .requests
        .push(PendingTowerAbilityActivation {
            entity: target_entity,
            ability_id: ability_id.to_string(),
            activation_serial,
        });
    Ok(())
}

pub fn drain_pending_tower_ability_casts(world: &mut World) {
    let drained = {
        let mut queue = world.write_resource::<PendingTowerAbilityCastQueue>();
        std::mem::take(&mut queue.requests)
    };
    for request in drained {
        let outcome = handle_tower_ability_cast_from_input(
            world,
            request.tower_entity_id,
            &request.ability_id,
            request.owner_pid,
        );
        let reason = outcome
            .as_ref()
            .err()
            .map(ToString::to_string)
            .unwrap_or_default();
        let next_serial = world
            .read_resource::<TowerAbilityCastResult>()
            .result_serial
            .wrapping_add(1);
        let next_result = TowerAbilityCastResult {
            player_id: request.owner_pid,
            tower_entity_id: request.tower_entity_id,
            ability_id: request.ability_id,
            accepted: outcome.is_ok(),
            reason,
            result_serial: next_serial,
        };
        *world.write_resource::<TowerAbilityCastResult>() = next_result.clone();
        world
            .write_resource::<TowerAbilityCastResults>()
            .latest_by_player
            .insert(next_result.player_id, next_result);
    }
}

pub(crate) fn acknowledge_tower_ability_pulse(
    world: &mut World,
    entity: Entity,
    ability_id: &str,
    activation_serial: u32,
    consumed: bool,
) -> bool {
    let mut towers = world.write_storage::<Tower>();
    let Some(state) = towers
        .get_mut(entity)
        .and_then(|tower| tower.active_ability.as_mut())
        .filter(|state| {
            state.ability_id == ability_id && state.activation_serial == activation_serial
        })
    else {
        return false;
    };
    state.acknowledge_pulse(consumed);
    true
}

pub(crate) fn cancel_tower_active_ability(
    world: &mut World,
    entity: Entity,
    ability_id: &str,
    activation_serial: u32,
) -> bool {
    let mut towers = world.write_storage::<Tower>();
    let Some(state) = towers
        .get_mut(entity)
        .and_then(|tower| tower.active_ability.as_mut())
        .filter(|state| {
            state.ability_id == ability_id && state.activation_serial == activation_serial
        })
    else {
        return false;
    };
    let was_active = state.active_remaining > Fixed64::ZERO
        || state.pulses_remaining > 0
        || state.pending_due > 0
        || state.opportunity_outstanding;
    state.active_remaining = Fixed64::ZERO;
    state.pulse_accumulator = Fixed64::ZERO;
    state.pulses_remaining = 0;
    state.pending_due = 0;
    state.opportunity_outstanding = false;
    was_active
}

fn handle_add_buff(
    world: &mut World,
    target: Entity,
    buff_id: String,
    duration: Fixed64,
    payload: serde_json::Value,
) -> Result<(), failure::Error> {
    // Orb markers are only stored for a learned orb on its own caster.
    if (payload.get(super::attack_orb::KEY).is_some() || buff_id.starts_with("attack_orb:"))
        && !super::attack_orb::admitted(world,target,&buff_id,&payload,None) {
        return Ok(());
    }
    if payload.get(super::ally_cleanse::KEY).is_some() || buff_id==super::ally_cleanse::BUFF {
        super::ally_cleanse::apply(world,target,&buff_id,duration,&payload);
        return Ok(());
    }
    if payload.get(super::mana_drain::KEY).is_some() || buff_id==super::mana_drain::BUFF {
        super::mana_drain::apply(world,target,&buff_id,duration,&payload);
        return Ok(());
    }
    if payload.get(super::forced_motion::KEY).is_some() || buff_id==super::forced_motion::BUFF {
        super::forced_motion::apply(world,target,&buff_id,duration,&payload);
        return Ok(());
    }
    // Launch envelope: opens a team vision zone, never stored as a buff.
    if payload.get(super::reveal_zone::KEY).is_some() || buff_id==super::reveal_zone::BUFF {
        super::reveal_zone::apply(world,target,&buff_id,duration,&payload);
        return Ok(());
    }
    if payload.get(super::ally_area_buff::KEY).is_some() || buff_id==super::ally_area_buff::BUFF {
        if let Some(spec)=super::ally_area_buff::admitted(world,target,&buff_id,duration,&payload,None) {
            let stats=super::ally_area_buff::buff_payload(&spec);
            for ally in super::ally_area_buff::targets(world,&spec) {
                world.write_resource::<BuffStore>().add(ally,&spec.buff,spec.seconds,stats.clone());
            }
        }
        return Ok(());
    }
    if payload.get(super::ally_area_restore::KEY).is_some() || buff_id==super::ally_area_restore::BUFF {
        if let Some(spec)=super::ally_area_restore::admitted(world,target,&buff_id,duration,&payload,None) {
            for (ally,amount) in super::ally_area_restore::heals(world,&spec) {handle_script_heal(world,ally,amount);}
            super::ally_area_restore::restore_mana(world,&spec);
        }
        return Ok(());
    }
    if payload.get(super::ally_shield::KEY).is_some() {
        super::ally_shield::apply(world,target,&buff_id,duration,&payload);
        return Ok(());
    }
    if payload.get(super::ability_runtime::damage_zone::KEY).is_some() {
        let Some(zone)=super::ability_runtime::damage_zone::DamageZoneSpec::from_payload(&payload)else{return Ok(());};
        if !zone_pulse_settlement::admit_zone(world,target,&buff_id,duration,&payload){return Ok(());}
        if zone.independent {
            let Some(mut origin)=super::moba_match::capture_projectile_origin(world,target)else{return Ok(());};
            origin.script=None;
            let faction=world.read_storage::<Faction>().get(target).cloned().expect("checked zone faction");
            world.write_resource::<BuffStore>().schedule_independent_zone(target,&buff_id,duration,zone,origin,faction);
            return Ok(());
        }
    }
    world
        .write_resource::<BuffStore>()
        .add(target, &buff_id, duration, payload);
    Ok(())
}

fn handle_creep_leaked(
    world: &mut World,
    events: &mut impl RuntimeEventSink,
    entity: Entity,
) -> Result<(), failure::Error> {
    let leak_value = world
        .read_storage::<Creep>()
        .get(entity)
        .and_then(|creep| creep.td_layer.as_ref())
        .map(|state| state.remaining_leak_value.min(i32::MAX as u32) as i32)
        .unwrap_or(1)
        .max(0);
    let (previous, remaining) = {
        let mut lives = world.write_resource::<crate::runtime::comp::PlayerLives>();
        let previous = lives.0;
        lives.0 = apply_td_life_loss(lives.0, leak_value);
        (previous, lives.0)
    };
    log::debug!(
        "creep leaked; leak_value={} lives={} entity={:?}",
        leak_value,
        remaining,
        entity
    );
    if previous <= 0 {
        return Ok(());
    }
    events.emit(game_lives_event(remaining));
    if remaining <= 0 {
        events.emit(game_end_event(
            "defeat",
            json!({ "result": "defeat", "reason": "lives_depleted" }),
        ));
        log::warn!("TD mode: player lives depleted");
    }
    Ok(())
}

fn apply_td_life_loss(lives: i32, leak_value: i32) -> i32 {
    lives.saturating_sub(leak_value.max(0)).max(0)
}

fn handle_projectile_directional(
    world: &mut World,
    pos: omoba_sim::Vec2,
    source: Option<Entity>,
    end_pos: omoba_sim::Vec2,
) -> Result<(), failure::Error> {
    let source_entity =
        source.ok_or_else(|| failure::err_msg("ProjectileDirectional missing source"))?;
    let (msd, atk_phys) = {
        let attacks = world.read_storage::<TAttack>();
        let attack = attacks
            .get(source_entity)
            .ok_or_else(|| failure::err_msg("Source attack properties not found"))?;
        (attack.bullet_speed, attack.atk_physic.v)
    };

    let initial_dist = (end_pos - pos).length();
    let flight_time_s = if msd.to_f32_for_render() > 0.0 {
        (initial_dist.to_f32_for_render() / msd.to_f32_for_render()).max(0.01)
    } else {
        0.01
    };
    let safety_time_left =
        Fixed64::from_raw(((flight_time_s * 1.5 + 0.5) * omoba_sim::fixed::SCALE as f32) as i64);

    let projectile = world
        .create_entity()
        .with(Pos(pos))
        .with(Projectile {
            damage_stage: crate::runtime::comp::DamageStage::Raw,
            time_left: safety_time_left,
            owner: source_entity,
            tpos: end_pos,
            target: None,
            radius: Fixed64::ZERO,
            msd,
            damage_phys: atk_phys,
            damage_magi: Fixed64::ZERO,
            damage_real: Fixed64::ZERO,
            damage_profile: AbiDamageProfile::NORMAL.bits(),
            slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO,
            hit_radius: Fixed64::ZERO,
            stun_duration: Fixed64::ZERO,
            kind_id: 0,
            generation: 0,
        })
        .build();
    remember_projectile_origin(world, projectile, source_entity);
    Ok(())
}

fn handle_tower_spawn(
    world: &mut World,
    pos: omoba_sim::Vec2,
    td: TowerData,
) -> Result<(), failure::Error> {
    let spawn_order = world.write_resource::<TowerSpawnOrderCounter>().allocate();
    world
        .create_entity()
        .with(Pos(pos))
        .with(Tower::new())
        .with(spawn_order)
        .with(td.tpty)
        .with(td.tatk)
        .build();
    world.write_resource::<Searcher>().tower.mark_dirty();
    Ok(())
}

fn handle_creep_stop(
    world: &mut World,
    source: Entity,
    target: Entity,
) -> Result<(), failure::Error> {
    let mut creeps = world.write_storage::<Creep>();
    let creep = creeps
        .get_mut(target)
        .ok_or_else(|| failure::err_msg("Creep not found"))?;
    creep.block_tower = Some(source);
    creep.status = CreepStatus::Stop;
    Ok(())
}

fn handle_creep_walk(world: &mut World, target: Entity) -> Result<(), failure::Error> {
    let mut creeps = world.write_storage::<Creep>();
    let creep = creeps
        .get_mut(target)
        .ok_or_else(|| failure::err_msg("Creep not found"))?;
    creep.status = CreepStatus::PreWalk;
    Ok(())
}

fn handle_creep_update(
    world: &mut World,
    entity: Entity,
    pos: omoba_sim::Vec2,
    status: CreepStatus,
    pidx: usize,
    path_remaining_distance: Fixed64,
    facing: omoba_sim::Angle,
    facing_broadcast: Option<f32>,
) -> Result<(), failure::Error> {
    if let Some(creep) = world.write_storage::<Creep>().get_mut(entity) {
        creep.status = status;
        creep.pidx = pidx;
        creep.path_remaining_distance = path_remaining_distance;
    }
    if let Some(pos_comp) = world.write_storage::<Pos>().get_mut(entity) {
        pos_comp.0 = pos;
    }
    if let Some(facing_comp) = world.write_storage::<Facing>().get_mut(entity) {
        facing_comp.0 = facing;
    }
    if let Some(facing_bc_comp) = world.write_storage::<FacingBroadcast>().get_mut(entity) {
        facing_bc_comp.0 = facing_broadcast;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn commit_td_layer_plan(
    world: &mut World,
    target: Entity,
    expected_state: &crate::runtime::TdLayerState,
    expected_hp: Fixed64,
    plan: &crate::runtime::TdLayerResolutionPlan,
) -> Result<bool, failure::Error> {
    let snapshot = {
        let creeps = world.read_storage::<Creep>();
        let properties = world.read_storage::<CProperty>();
        let positions = world.read_storage::<Pos>();
        let factions = world.read_storage::<Faction>();
        let bounties = world.read_storage::<Bounty>();
        let facings = world.read_storage::<Facing>();
        let facing_broadcasts = world.read_storage::<FacingBroadcast>();
        let turn_speeds = world.read_storage::<TurnSpeed>();
        let tags = world.read_storage::<ScriptUnitTag>();
        let creep = creeps
            .get(target)
            .cloned()
            .ok_or_else(|| failure::err_msg("TD layer commit target lost Creep"))?;
        let property = properties
            .get(target)
            .cloned()
            .ok_or_else(|| failure::err_msg("TD layer commit target lost CProperty"))?;
        if creep.td_layer.as_ref() != Some(expected_state) || property.hp != expected_hp {
            return Err(failure::err_msg(format!(
                "TD layer commit stale target={} expected={}:{:?} actual={:?}:{:?}",
                target.id(),
                expected_state.current_layer,
                expected_hp,
                creep
                    .td_layer
                    .as_ref()
                    .map(|state| state.current_layer.as_str()),
                property.hp
            )));
        }
        (
            creep,
            property,
            *positions
                .get(target)
                .ok_or_else(|| failure::err_msg("TD layer commit target lost Pos"))?,
            factions
                .get(target)
                .cloned()
                .ok_or_else(|| failure::err_msg("TD layer commit target lost Faction"))?,
            bounties.get(target).copied().unwrap_or_default(),
            facings.get(target).copied().unwrap_or_default(),
            facing_broadcasts.get(target).copied().unwrap_or_default(),
            turn_speeds.get(target).copied().unwrap_or_default(),
            tags.get(target).cloned(),
        )
    };

    let (
        base_creep,
        base_property,
        pos,
        faction,
        _bounty,
        facing,
        facing_broadcast,
        turn_speed,
        tag,
    ) = snapshot;
    if let Some(original) = &plan.original {
        let layer = omoba_template_ids::active_td_layer_by_name(&original.state.current_layer)
            .ok_or_else(|| failure::err_msg("TD layer commit missing original metadata"))?;
        let mut creeps = world.write_storage::<Creep>();
        let mut properties = world.write_storage::<CProperty>();
        let creep = creeps
            .get_mut(target)
            .ok_or_else(|| failure::err_msg("TD layer commit original Creep disappeared"))?;
        creep.name = format!("td_btd_{}", original.state.current_layer);
        creep.label = Some(layer.label.to_string());
        creep.td_layer = Some(original.state.clone());
        let property = properties
            .get_mut(target)
            .ok_or_else(|| failure::err_msg("TD layer commit original CProperty disappeared"))?;
        property.hp = original.hp;
        property.mhp = original.max_hp;
        property.msd = Fixed64::from_i32(layer.move_speed as i32);
    } else if let Some(property) = world.write_storage::<CProperty>().get_mut(target) {
        property.hp = Fixed64::ZERO;
    }

    for child in &plan.children {
        let layer = omoba_template_ids::active_td_layer_by_name(&child.state.current_layer)
            .ok_or_else(|| failure::err_msg("TD layer commit missing child metadata"))?;
        let mut creep = base_creep.clone();
        creep.name = format!("td_btd_{}", child.state.current_layer);
        creep.label = Some(layer.label.to_string());
        creep.block_tower = None;
        creep.td_layer = Some(child.state.clone());
        let mut property = base_property.clone();
        property.hp = child.hp;
        property.mhp = child.max_hp;
        property.msd = Fixed64::from_i32(layer.move_speed as i32);
        let mut builder = world
            .create_entity()
            .with(pos)
            .with(creep)
            .with(property)
            .with(faction.clone())
            // Layer cash is credited at pop time; generic final-death bounty must stay disabled.
            .with(Bounty::default())
            .with(facing)
            .with(facing_broadcast)
            .with(turn_speed);
        if let Some(mut child_tag) = tag.clone() {
            child_tag.unit_id = format!("creep_td_btd_{}", child.state.current_layer);
            builder = builder.with(child_tag);
        }
        let entity = builder.build();
        world
            .write_resource::<ScriptEventQueue>()
            .push(ScriptEvent::Spawn { e: entity });
        world.write_resource::<BuffStore>().add(
            entity,
            "creep_min_speed_floor",
            Fixed64::from_raw(i64::MAX),
            json!({ "movespeed_absolute_min": 10.0 }),
        );
    }

    Ok(plan.original.is_none())
}

#[allow(clippy::too_many_arguments)]
fn handle_damage(
    world: &mut World,
    next_outcomes: &mut Vec<Outcome>,
    pos: omoba_sim::Vec2,
    phys: Fixed64,
    magi: Fixed64,
    real: Fixed64,
    source: Entity,
    target: Entity,
    damage_profile: u32,
    _predeclared: bool,
) -> Result<(), failure::Error> {
    if !crate::runtime::native::moba_match::moba_damage_from_allowed(world,Some(source),target) {
        return Ok(());
    }
    if world.try_fetch::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>()
        .is_some_and(|targets| targets.0.contains(&target)) {
        return Ok(());
    }
    let total_damage = crate::runtime::ability_runtime::UnitStats::from_refs(
        &world.read_resource::<BuffStore>(), false,
    ).incoming_damage_packet(phys, magi, real, target);
    let td_snapshot = {
        let creeps = world.read_storage::<Creep>();
        let properties = world.read_storage::<CProperty>();
        creeps.get(target).and_then(|creep| {
            creep
                .td_layer
                .clone()
                .zip(properties.get(target).map(|property| property.hp))
        })
    };
    if let Some((state, current_hp)) = td_snapshot {
        // Multiple same-tick outcomes (for example projectile damage followed
        // by an on-hit critical bonus) can target a layer already reduced to
        // zero earlier in this outcome batch. Its Death is committed after the
        // batch, so treat the later hit as stale overkill: no duplicate pop,
        // cash, or children.
        if current_hp <= Fixed64::ZERO {
            log::debug!(
                "ignoring stale TD damage source={} target={} layer={} hp_raw={}",
                source.id(),
                target.id(),
                state.current_layer,
                current_hp.raw(),
            );
            return Ok(());
        }
        let profile = crate::runtime::DamageProfile::new(damage_profile).map_err(|error| {
            failure::err_msg(format!(
                "invalid TD damage profile source={} target={} mask={:#x}: {:?}",
                source.id(),
                target.id(),
                damage_profile,
                error
            ))
        })?;
        let hit_serial = {
            let mut serial = world.write_resource::<crate::runtime::TdLayerCommitSerial>();
            serial.0 = serial.0.wrapping_add(1);
            serial.0
        };
        let owner_player_id = world
            .read_storage::<PlayerOwner>()
            .get(source)
            .map(|owner| owner.player_id);
        let plan = crate::runtime::resolve_td_layer_damage(
            omoba_template_ids::active_td_layer_catalog(),
            &state,
            current_hp,
            total_damage,
            profile,
            crate::runtime::HitProvenance {
                source_entity_id: source.id(),
                owner_player_id,
                hit_serial,
            },
        )
        .map_err(|error| {
            failure::err_msg(format!(
                "TD layer resolve failed source={} target={} hit={} mask={:#x}: {:?}",
                source.id(),
                target.id(),
                hit_serial,
                damage_profile,
                error
            ))
        })?;
        let died = commit_td_layer_plan(world, target, &state, current_hp, &plan)?;
        if let Some(tower) = world.write_storage::<Tower>().get_mut(source) {
            tower.pops = tower.pops.saturating_add(plan.pop_count());
        }
        if plan.pop_count() > 0 {
            let tick = world.try_fetch::<Tick>().map(|tick| tick.0).unwrap_or(0);
            let source_layers = plan
                .popped
                .iter()
                .map(|layer| layer.layer_id.as_str())
                .collect::<Vec<_>>()
                .join("+");
            let mut ledger = world.write_resource::<crate::runtime::TdEconomyLedger>();
            let mut economy = world.write_resource::<PlayerEconomy>();
            ledger
                .apply(
                    &mut economy,
                    tick,
                    owner_player_id,
                    crate::runtime::TdEconomyCategory::LayerIncome,
                    plan.cash().min(i32::MAX as u32) as i32,
                    source_layers,
                )
                .map_err(|error| {
                    failure::err_msg(format!(
                        "TD layer economy commit failed hit={} source={}: {}",
                        hit_serial,
                        source.id(),
                        error
                    ))
                })?;
        }
        log::debug!(
            "TD layer commit hit={} source={} target={} popped={} cash={} children={} immune={:?}",
            hit_serial,
            source.id(),
            target.id(),
            plan.pop_count(),
            plan.cash(),
            plan.children.len(),
            plan.immune_layer
        );
        if died {
            next_outcomes.push(Outcome::Death { pos, ent: target });
        }
        return Ok(());
    }

    let was_hit = total_damage > Fixed64::ZERO
        && world.read_storage::<CProperty>().get(target).is_some_and(|p| p.hp > Fixed64::ZERO);
    let total_damage = if was_hit {
        world.write_resource::<BuffStore>().absorb_damage_with_shield(target, total_damage)
    } else { total_damage };
    let mut died = false;
    let mut first_lethal_hit = false;
    let mut took_damage = false;
    {
        let mut properties = world.write_storage::<CProperty>();
        if let Some(target_props) = properties.get_mut(target) {
            let hp_before = target_props.hp;
            target_props.hp = target_props.hp - total_damage;
            took_damage = hp_before > Fixed64::ZERO && total_damage > Fixed64::ZERO;
            let (source_name, target_name) = get_entity_names(world, source, target);
            log::debug!(
                "{} attacked {} | damage {:.1} | HP {:.1} -> {:.1}/{:.1}",
                source_name,
                target_name,
                total_damage.to_f32_for_render(),
                hp_before.to_f32_for_render(),
                target_props.hp.to_f32_for_render(),
                target_props.mhp.to_f32_for_render()
            );
            if target_props.hp <= Fixed64::ZERO {
                target_props.hp = Fixed64::ZERO;
                died = true;
                first_lethal_hit = hp_before > Fixed64::ZERO && total_damage > Fixed64::ZERO;
                if target_props.mhp > Fixed64::from_i32(100) {
                    log::info!(
                        "{} died | max_hp={} hp_before={} dmg={:.1} source={}",
                        target_name,
                        target_props.mhp.to_f32_for_render(),
                        hp_before.to_f32_for_render(),
                        total_damage.to_f32_for_render(),
                        source_name
                    );
                }
            }
        }
    }

    if was_hit {
        crate::runtime::native::moba_match::interrupt_moba_recall(world, target);
        // Invalidate before shield absorption removes the packet. Assist and
        // kill credit stay on the post-shield `took_damage` path below.
        super::shop::note_shop_undo_combat(world, Some(source), target);
    }
    if took_damage {
        super::cast_windup::interrupt_for_damage(world,target);
        crate::runtime::native::moba_match::record_moba_hero_damage(world, Some(source), target, first_lethal_hit);
        crate::runtime::native::moba_match::record_moba_jungle_damage(world, Some(source), target, first_lethal_hit);
        crate::runtime::native::moba_match::record_moba_lane_damage(world, Some(source), target, first_lethal_hit);
        crate::runtime::native::moba_match::record_moba_building_damage(world, Some(source), target, first_lethal_hit);
    }
    if died {
        let mut towers = world.write_storage::<Tower>();
        if let Some(tower) = towers.get_mut(source) {
            tower.pops = tower.pops.saturating_add(1);
        }
        drop(towers);
        if let Err(error) = credit_td_bounty_for_source(world, source, target) {
            log::warn!("TDCreepBounty: failed to credit source={source:?}: {error}");
        }
        next_outcomes.push(Outcome::Death { pos, ent: target });
    }
    Ok(())
}

fn handle_heal(world: &mut World, target: Entity, amount: Fixed64) -> Result<(), failure::Error> {
    let amount=received_healing(world,target,amount);
    let mut properties = world.write_storage::<CProperty>();
    if let Some(target_props) = properties.get_mut(target) {
        target_props.hp=healed_hp(target_props.hp,target_props.mhp,amount);
    }
    Ok(())
}

fn received_healing(world:&World,target:Entity,amount:Fixed64)->Fixed64 {
    world.try_fetch::<BuffStore>().map_or(amount.max(Fixed64::ZERO),|buffs|
        crate::runtime::UnitStats::from_refs(&buffs,false).received_healing(amount,target))
}

fn healed_hp(current:Fixed64,maximum:Fixed64,amount:Fixed64)->Fixed64 {
    if current<=Fixed64::ZERO || maximum<=Fixed64::ZERO || amount<=Fixed64::ZERO {return current;}
    Fixed64::from_raw((i128::from(current.raw())+i128::from(amount.raw())).min(i128::from(maximum.raw())) as i64)
}

fn handle_attack_update(
    world: &mut World,
    target: Entity,
    asd_count: Option<Fixed64>,
    cooldown_reset: bool,
) -> Result<(), failure::Error> {
    let mut attacks = world.write_storage::<TAttack>();
    if let Some(attack) = attacks.get_mut(target) {
        if let Some(new_count) = asd_count {
            attack.asd_count = new_count;
        }
        if cooldown_reset {
            attack.asd_count = attack.asd.v;
        }
    }
    Ok(())
}

fn handle_experience_gain(
    world: &mut World,
    target: Entity,
    amount: u32,
) -> Result<(), failure::Error> {
    let mut heroes = world.write_storage::<Hero>();
    if let Some(hero) = heroes.get_mut(target) {
        let leveled_up = hero.add_experience(amount as i32);
        if leveled_up {
            log::info!(
                "Hero '{}' gained {} experience and leveled up",
                hero.name,
                amount
            );
        } else {
            log::info!("Hero '{}' gained {} experience", hero.name, amount);
        }
    }
    Ok(())
}

fn handle_gold_gain(world: &mut World, target: Entity, amount: i32) -> Result<(), failure::Error> {
    if amount == 0 {
        return Ok(());
    }
    let mut golds = world.write_storage::<Gold>();
    let before=golds.get(target).map_or(0,|gold|gold.0);
    match golds.get_mut(target) {
        Some(gold) => gold.0 = gold.0.saturating_add(amount),
        None => {
            let _ = golds.insert(target, Gold(amount.max(0)));
        }
    }
    let after=golds.get(target).map_or(0,|gold|gold.0);
    drop(golds);
    super::moba_match::record_moba_gold_gain(world,target,(i64::from(after)-i64::from(before)).max(0) as u64);
    Ok(())
}

fn get_entity_names(world: &World, source: Entity, target: Entity) -> (String, String) {
    let creeps = world.read_storage::<Creep>();
    let heroes = world.read_storage::<Hero>();
    let units = world.read_storage::<Unit>();
    let towers = world.read_storage::<Tower>();
    let script_tags = world.read_storage::<ScriptUnitTag>();
    let registry = world.read_resource::<TowerTemplateRegistry>();

    let name_of = |entity: Entity| -> String {
        if let Some(creep) = creeps.get(entity) {
            return creep.name.clone();
        }
        if let Some(hero) = heroes.get(entity) {
            return hero.name.clone();
        }
        if let Some(unit) = units.get(entity) {
            return unit.name.clone();
        }
        if let Some(tag) = script_tags.get(entity) {
            if let Some(tpl) = registry.get(&tag.unit_id) {
                return tpl.label.clone();
            }
        }
        if towers.get(entity).is_some() {
            return "Tower".to_string();
        }
        format!("Entity({})", entity.id())
    };

    (name_of(source), name_of(target))
}

fn handle_explosion(world: &mut World, pos: omoba_sim::Vec2, radius: Fixed64, duration: Fixed64) {
    let current_tick = world.read_resource::<Tick>().0 as u32;
    let duration_ms = (duration.to_f32_for_render() * 1000.0).clamp(0.0, u32::MAX as f32) as u32;
    world
        .write_resource::<ExplosionFxQueue>()
        .pending
        .push(ExplosionFx {
            pos_x: pos.x.to_f32_for_render(),
            pos_y: pos.y.to_f32_for_render(),
            radius: radius.to_f32_for_render(),
            duration_ms,
            spawn_tick: current_tick,
        });
}

#[allow(clippy::too_many_arguments)]
fn handle_attack_phase_cue(
    world: &mut World,
    entity: Entity,
    attack_seq: u32,
    is_critical: bool,
    target: Option<Entity>,
    target_pos: Option<omoba_sim::Vec2>,
    windup_ms: u32,
    backswing_ms: u32,
    dir_rad: f32,
) {
    if let Some(target) = target {
        crate::runtime::native::moba_match::record_brush_attack_reveal(world,entity,target);
        crate::runtime::native::moba_match::record_moba_champion_aggression(world, entity, target, true);
    }
    let current_tick = world.read_resource::<Tick>().0 as u32;
    world
        .write_resource::<crate::runtime::comp::AttackPhaseFxQueue>()
        .pending
        .push(crate::runtime::comp::AttackPhaseFx {
            entity_id: entity.id(),
            entity_gen: entity.gen().id() as u32,
            spawn_tick: current_tick,
            attack_seq,
            is_critical,
            windup_ms,
            impact_at_ms: windup_ms,
            backswing_ms,
            dir_rad,
            target_entity_id: target.map(|target| target.id()),
            target_pos_x: target_pos.map(|pos| pos.x.to_f32_for_render()),
            target_pos_y: target_pos.map(|pos| pos.y.to_f32_for_render()),
        });
}
