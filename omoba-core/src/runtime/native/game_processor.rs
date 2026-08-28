use std::borrow::Cow;
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
    FactionType, Gold, Hero, HeroCommand, HeroCommandQueue, Inventory, IsBase, IsBuilding,
    KnowledgeBonusResource, MasterSeed, MoveTarget, Outcome, Path, PendingAbilityCastQueue,
    PendingAbilityUpgradeQueue, PendingHeroCommandClearQueue, PendingHeroCommandKind,
    PendingItemUseQueue, PendingMoveQueue, PendingTowerAbilityActivation,
    PendingTowerAbilityActivationQueue, PendingTowerAbilityCastQueue, PendingTowerSellQueue,
    PendingTowerSpawnQueue, PendingTowerTargetPriorityQueue, PendingTowerUpgradeQueue,
    PlayerEconomy, PlayerOwner, Pos, Projectile, RemovedEntitiesQueue, Searcher, TAttack,
    TProperty, Tick, Tower, TowerAbilityCastResult, TowerAbilityCastResults,
    TowerActiveAbilityState, TowerData, TowerFireFxQueue, TowerSpawnOrderCounter,
    TowerTargetPriority, TowerTemplate, TowerTemplateRegistry, TowerUpgradeRegistry, TurnSpeed,
    Unit, INVENTORY_SLOTS,
};
use crate::runtime::events::{RuntimeBroadcast, RuntimeEvent, RuntimeEventSink};
use crate::runtime::geometry::{circle_hits_polygon, point_segment_dist_sq};
use crate::runtime::item::{ActiveEffect, ItemRegistry};
use crate::runtime::scripting::{ScriptEvent, ScriptEventQueue, ScriptUnitTag, SkillTarget};
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
    (&entities, &heroes, &owners)
        .join()
        .find(|(_, _, owner)| owner.player_id == owner_pid)
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

fn clear_hero_command_queue(world: &mut World, hero: Entity) {
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
            PendingHeroCommandKind::MoveTo { pos } => HeroCommand::MoveTo { pos },
            PendingHeroCommandKind::AttackMove { pos } => HeroCommand::AttackMove { pos },
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
    if ability_index >= 4 {
        return Err(failure::err_msg(format!(
            "AbilityUpgrade: invalid ability_index={} (must be 0..=3) pid={}",
            ability_index, owner_pid
        )));
    }
    let slot = ability_index as usize;

    let hero_entity = player_hero_entity(world, "AbilityUpgrade", owner_pid)?;

    let ability_id = {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(hero_entity).ok_or_else(|| {
            failure::err_msg(format!(
                "AbilityUpgrade: hero entity vanished before read (pid={})",
                owner_pid
            ))
        })?;
        hero.abilities
            .get(slot)
            .filter(|id| !id.is_empty())
            .cloned()
            .ok_or_else(|| {
                failure::err_msg(format!(
                    "AbilityUpgrade: hero has no ability bound at slot={} (pid={})",
                    slot, owner_pid
                ))
            })?
    };

    let max_level = {
        let registry = world.read_resource::<AbilityRegistry>();
        registry
            .get(&ability_id)
            .map(|def| i32::from(def.max_level).max(1))
            .unwrap_or(5)
    };

    let new_level = {
        let mut heroes = world.write_storage::<Hero>();
        let hero = heroes.get_mut(hero_entity).ok_or_else(|| {
            failure::err_msg(format!(
                "AbilityUpgrade: hero entity vanished before write (pid={})",
                owner_pid
            ))
        })?;
        if hero.skill_points <= 0 {
            return Err(failure::err_msg(format!(
                "AbilityUpgrade: no skill points slot={} ability='{}' pid={}",
                slot, ability_id, owner_pid
            )));
        }
        let current = hero.ability_levels.get(&ability_id).copied().unwrap_or(0);
        if current >= max_level {
            return Err(failure::err_msg(format!(
                "AbilityUpgrade: slot={} ability='{}' already maxed ({}/{}) pid={}",
                slot, ability_id, current, max_level, owner_pid
            )));
        }
        let next = current + 1;
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

pub fn drain_pending_ability_upgrades(world: &mut World) {
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
        if let Err(e) = handle_ability_upgrade_from_input(world, req.ability_index, req.owner_pid) {
            log::warn!(
                "AbilityUpgrade failed pid={} ability_index={}: {}",
                req.owner_pid,
                req.ability_index,
                e
            );
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
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::HERO_ABILITY,
        "runtime/native/game_processor/handle_ability_cast_from_input",
    );
    if ability_index >= 4 {
        return Err(failure::err_msg(format!(
            "AbilityCast: invalid ability_index={} (must be 0..=3) pid={}",
            ability_index, owner_pid
        )));
    }
    let slot = ability_index as usize;

    let caster = player_hero_entity(world, "AbilityCast", owner_pid)?;

    let ability_id = {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(caster).ok_or_else(|| {
            failure::err_msg(format!(
                "AbilityCast: hero entity vanished before read (pid={})",
                owner_pid
            ))
        })?;
        hero.abilities
            .get(slot)
            .filter(|id| !id.is_empty())
            .cloned()
            .ok_or_else(|| {
                failure::err_msg(format!(
                    "AbilityCast: hero has no ability bound at slot={} (pid={})",
                    slot, owner_pid
                ))
            })?
    };

    {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(caster).ok_or_else(|| {
            failure::err_msg(format!(
                "AbilityCast: hero entity vanished before gate (pid={})",
                owner_pid
            ))
        })?;
        if !hero.can_use_ability(&ability_id) {
            return Err(failure::err_msg(format!(
                "AbilityCast: slot={} ability='{}' not learned (pid={})",
                slot, ability_id, owner_pid
            )));
        }
        if hero.is_on_cooldown(&ability_id) {
            return Err(failure::err_msg(format!(
                "AbilityCast: slot={} ability='{}' still on cooldown ({:.2}s) pid={}",
                slot,
                ability_id,
                hero.get_cooldown(&ability_id).to_f32_for_render(),
                owner_pid
            )));
        }
    }

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
    Ok(())
}

pub fn drain_pending_ability_casts(world: &mut World) {
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
        if let Err(e) = handle_ability_cast_from_input(
            world,
            req.ability_index,
            req.target_pos,
            req.target_entity,
            req.owner_pid,
        ) {
            log::warn!(
                "AbilityCast failed pid={} ability_index={}: {}",
                req.owner_pid,
                req.ability_index,
                e
            );
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

pub fn handle_item_use_from_input(
    world: &mut World,
    item_slot: u32,
    _target_pos: Option<omoba_sim::Vec2>,
    _target_entity: Option<u32>,
    owner_pid: u32,
) -> Result<(), failure::Error> {
    register_projection_policy(
        world,
        omb_script_abi::types::projection_policy_ids::ITEM,
        "runtime/native/game_processor/handle_item_use_from_input",
    );
    let slot_i = item_slot as usize;
    if slot_i >= INVENTORY_SLOTS {
        return Err(failure::err_msg(format!(
            "ItemUse: invalid slot={} (max {}) pid={}",
            slot_i, INVENTORY_SLOTS, owner_pid
        )));
    }

    let hero_entity = player_hero_entity(world, "ItemUse", owner_pid)?;

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

    {
        let mut props = world.write_storage::<CProperty>();
        if let Some(p) = props.get_mut(hero_entity) {
            match &active {
                ActiveEffect::Shield { amount, .. } => {
                    let amt_fx = omoba_sim::Fixed64::from_raw((*amount * 1024.0) as i64);
                    let summed = p.hp + amt_fx;
                    p.hp = if summed > p.mhp { p.mhp } else { summed };
                    log::info!("ItemUse Shield +{} HP pid={}", amount, owner_pid);
                }
                ActiveEffect::RestoreMana { amount } => {
                    log::info!(
                        "ItemUse RestoreMana +{} MP pid={} (mp not wired in MVP)",
                        amount,
                        owner_pid
                    );
                }
                ActiveEffect::SprintBuff { ms_bonus, duration } => {
                    let bonus_fx = omoba_sim::Fixed64::from_raw((*ms_bonus * 1024.0) as i64);
                    p.msd += bonus_fx;
                    log::info!(
                        "ItemUse SprintBuff +{} ms {}s pid={} (MVP no expiry)",
                        ms_bonus,
                        duration,
                        owner_pid
                    );
                }
                ActiveEffect::DamageReduce { percent, duration } => {
                    log::info!(
                        "ItemUse DamageReduce {}% {}s pid={} (buff pipeline TBD)",
                        percent * 100.0,
                        duration,
                        owner_pid
                    );
                }
                ActiveEffect::HeadshotNext { bonus_damage } => {
                    log::info!(
                        "ItemUse HeadshotNext +{} dmg pid={} (projectile hook TBD)",
                        bonus_damage,
                        owner_pid
                    );
                }
            }
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
    clear_hero_command_queue(world, hero_entity);

    log::info!(
        "ItemUse ok pid={} slot={} item={} cooldown={}s",
        owner_pid,
        slot_i,
        cfg.id,
        cfg.cooldown
    );
    Ok(())
}

pub fn drain_pending_item_uses(world: &mut World) {
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
        if let Err(e) = handle_item_use_from_input(
            world,
            req.item_slot,
            req.target_pos,
            req.target_entity,
            req.owner_pid,
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
        Outcome::Damage { .. } => "Damage",
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
        Outcome::ScriptRemoveBuff { .. } => "ScriptRemoveBuff",
        Outcome::ScriptProjectile { .. } => "ScriptProjectile",
        Outcome::ScriptTowerFireFx { .. } => "ScriptTowerFireFx",
        Outcome::ScriptAttackPhaseCue { .. } => "ScriptAttackPhaseCue",
        Outcome::ScriptStartCooldown { .. } => "ScriptStartCooldown",
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

pub fn process_outcomes(
    world: &mut World,
    events: &mut impl RuntimeEventSink,
) -> Result<(), failure::Error> {
    let mut remove_uids = Vec::new();
    let mut next_outcomes = Vec::new();

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
    for (outcome_ordinal, outcome) in outcomes.into_iter().enumerate() {
        if let Some(fact) =
            observable_fact_from_outcome(&outcome, projection_tick, outcome_ordinal as u32)
        {
            let _ = world
                .read_resource::<crate::runtime::ObservableFactBuffer>()
                .emit(fact);
        }
        let kind = outcome_kind(&outcome);
        match outcome {
            Outcome::Death { ent, .. } => {
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
            Outcome::ScriptRemoveBuff { target, buff_id } => {
                handle_script_remove_buff(world, target, buff_id)
            }
            Outcome::ScriptProjectile {
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
            Outcome::ScriptStartCooldown {
                entity,
                ability_id,
                duration,
            } => handle_script_start_cooldown(world, entity, ability_id, duration),
            Outcome::EntityRemoved { entity } => {
                world
                    .write_resource::<RemovedEntitiesQueue>()
                    .pending
                    .push(entity.id());
                let _ = world.entities().delete(entity);
            }
            Outcome::SpawnUnit { .. } => {}
        }
        log::trace!("processed outcome {}", kind);
    }

    let _ = world.delete_entities(&remove_uids[..]);
    world.write_resource::<Vec<Outcome>>().clear();
    world
        .write_resource::<Vec<Outcome>>()
        .append(&mut next_outcomes);
    drop(outcomes_span);

    Ok(())
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

    if is_enemy_base {
        log::info!("enemy base entity {:?} destroyed", entity);
        events.emit(game_end_event(
            "player",
            json!({ "winner": "player", "base_entity_id": entity.id() }),
        ));
    }
    Ok(())
}

fn distribute_bounty(world: &mut World, dead: Entity) {
    if world.read_resource::<crate::comp::GameMode>().is_td() {
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

    let (msd, target_pos, atk_phys, stun_duration_roll, visual_count) = {
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
        let mut final_atk = stats.final_atk(attack.atk_physic.v, source_entity);

        let accuracy_bonus = buff_store.sum_add(source_entity, StatKey::AccuracyBonus);
        let accuracy = (Fixed64::ONE + accuracy_bonus).clamp(Fixed64::ZERO, Fixed64::ONE);
        if accuracy < Fixed64::ONE {
            let roll = Fixed64::from_raw(
                (crate::runtime::tick_random_u64(
                    master_seed,
                    u64::from(tick),
                    (u64::from(attacker_id) << 16) | u64::from(OP_PROJECTILE_ACCURACY),
                ) % omoba_sim::fixed::SCALE as u64) as i64,
            );
            if roll >= accuracy {
                final_atk = Fixed64::ZERO;
            }
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
        let stun_duration_roll = if stun_chance > 0.0 && stun_duration > 0.0 {
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
        )
    };

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
        world
            .create_entity()
            .with(Pos(start_pos))
            .with(Projectile {
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
    }
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
    if let Some(prop) = world.write_storage::<CProperty>().get_mut(target) {
        prop.hp = (prop.hp - amount).max(Fixed64::ZERO);
        return;
    }
    if let Some(unit) = world.write_storage::<Unit>().get_mut(target) {
        let amount_i = amount.to_f32_for_render() as i32;
        unit.current_hp = (unit.current_hp - amount_i).max(0);
    }
}

fn handle_script_heal(world: &mut World, target: Entity, amount: Fixed64) {
    if let Some(prop) = world.write_storage::<CProperty>().get_mut(target) {
        prop.hp = (prop.hp + amount).min(prop.mhp);
        return;
    }
    if let Some(unit) = world.write_storage::<Unit>().get_mut(target) {
        let amount_i = amount.to_f32_for_render() as i32;
        unit.current_hp = (unit.current_hp + amount_i).min(unit.max_hp);
    }
}

fn handle_script_remove_buff(world: &mut World, target: Entity, buff_id: String) {
    world.write_resource::<BuffStore>().remove(target, &buff_id);
}

#[allow(clippy::too_many_arguments)]
fn handle_script_projectile(
    world: &mut World,
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

    world
        .create_entity()
        .with(Pos(pos))
        .with(Projectile {
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
            q.requests.push(crate::comp::PendingHeroCommand {
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
                q.requests.push(crate::comp::PendingHeroCommand {
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
            q.requests.push(crate::comp::PendingHeroCommand {
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
            q.requests.push(crate::comp::PendingHeroCommand {
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
            q.requests.push(crate::comp::PendingHeroCommand {
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
            Fixed64::from_i32(15)
        );
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

        let events = world.write_resource::<ScriptEventQueue>().drain();
        assert!(matches!(
            events.as_slice(),
            [ScriptEvent::ProjectileHit {
                attacker,
                victim,
                kind_id: 41,
                generation: 2,
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

    world
        .create_entity()
        .with(Pos(pos))
        .with(Projectile {
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
    let dmg_taken_bonus = world
        .read_resource::<BuffStore>()
        .sum_add(target, StatKey::DamageTakenBonus);
    let dmg_multiplier = (Fixed64::ONE + dmg_taken_bonus).max(Fixed64::ZERO);
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
        let total_damage = (phys + magi + real) * dmg_multiplier;
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

    let mut died = false;
    {
        let mut properties = world.write_storage::<CProperty>();
        if let Some(target_props) = properties.get_mut(target) {
            let hp_before = target_props.hp;
            let total_damage = (phys + magi + real) * dmg_multiplier;
            target_props.hp = target_props.hp - total_damage;
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
    let mut properties = world.write_storage::<CProperty>();
    if let Some(target_props) = properties.get_mut(target) {
        let summed = target_props.hp + amount;
        target_props.hp = if summed > target_props.mhp {
            target_props.mhp
        } else {
            summed
        };
    }
    Ok(())
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
    match golds.get_mut(target) {
        Some(gold) => gold.0 = gold.0.saturating_add(amount),
        None => {
            let _ = golds.insert(target, Gold(amount.max(0)));
        }
    }
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
