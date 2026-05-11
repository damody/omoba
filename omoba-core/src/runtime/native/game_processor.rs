use std::collections::BTreeMap;

use omoba_sim::Fixed64;
use serde_json::json;
use specs::{Builder, Entity, Join, LendJoin, World, WorldExt};
use vek::Vec2;

use crate::runtime::ability_runtime::{AbilityRegistry, BuffStore};
use crate::runtime::comp::{
    AttackCancelFx, AttackCancelFxQueue, AttackCancelPhase, AttackSequencePhase, BlockedRegions,
    Bounty, CProperty, CircularVision, CollisionRadius, Creep, CreepData, CreepStatus,
    ExplosionFx, ExplosionFxQueue, Facing, FacingBroadcast, Faction, FactionType, Gold, Hero,
    Inventory, IsBase, IsBuilding, MasterSeed, MoveTarget, Outcome, Path,
    PendingAbilityCastQueue, PendingAbilityUpgradeQueue, PendingItemUseQueue, PendingMoveQueue,
    PendingTowerSellQueue, PendingTowerSpawnQueue, PendingTowerUpgradeQueue, Pos, Projectile,
    RemovedEntitiesQueue, Searcher, TAttack, TProperty, Tick, Tower, TowerData, TowerTemplate,
    TowerTemplateRegistry, TowerUpgradeRegistry, TurnSpeed, Unit, INVENTORY_SLOTS,
};
use crate::runtime::comp::tower_upgrade_rules;
use crate::runtime::events::{RuntimeBroadcast, RuntimeEvent, RuntimeEventSink};
use crate::runtime::geometry::{circle_hits_polygon, point_segment_dist_sq};
use crate::runtime::item::{ActiveEffect, ItemRegistry};
use crate::runtime::scripting::{ScriptEvent, ScriptEventQueue, ScriptUnitTag, SkillTarget};
use crate::tower_meta::UpgradeEffect;
use omb_script_abi::stat_keys::StatKey;

const OP_PROJECTILE_ACCURACY: u32 = 20;
const OP_PROJECTILE_STUN_ROLL: u32 = 21;

fn player_hero_entity(
    world: &World,
    context: &str,
    owner_pid: u32,
) -> Result<Entity, failure::Error> {
    let entities = world.entities();
    let heroes = world.read_storage::<Hero>();
    let factions = world.read_storage::<Faction>();
    (&entities, &heroes, &factions)
        .join()
        .find(|(_, _, f)| f.faction_id == FactionType::Player)
        .map(|(e, _, _)| e)
        .ok_or_else(|| {
            failure::err_msg(format!(
                "{}: no Player-faction Hero entity (pid={})",
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

pub fn drain_pending_moves(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingMoveQueue>();
        std::mem::take(&mut q.requests)
    };
    if drained.is_empty() {
        return;
    }

    let hero_entity = {
        let entities = world.entities();
        let heroes = world.read_storage::<Hero>();
        let factions = world.read_storage::<Faction>();
        (&entities, &heroes, &factions)
            .join()
            .find(|(_, _, f)| f.faction_id == FactionType::Player)
            .map(|(e, _, _)| e)
    };
    let Some(hero) = hero_entity else {
        log::warn!(
            "MoveTo: no Player-faction hero found ({} requests dropped)",
            drained.len()
        );
        return;
    };

    for req in drained {
        log::info!(
            "MoveTo pid={} -> hero={:?} pos=({:.1},{:.1})",
            req.owner_pid,
            hero,
            req.pos.x.to_f32_for_render(),
            req.pos.y.to_f32_for_render(),
        );
        interrupt_attack_for_accepted_command(world, hero);
        let _ = world
            .write_storage::<MoveTarget>()
            .insert(hero, MoveTarget(req.pos));
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
}

pub fn handle_ability_cast_from_input(
    world: &mut World,
    ability_index: u32,
    target_pos: Option<omoba_sim::Vec2>,
    target_entity: Option<u32>,
    owner_pid: u32,
) -> Result<(), failure::Error> {
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
}

pub fn spawn_td_tower(world: &mut World, pos: Vec2<f32>, unit_id: &str) -> Option<Entity> {
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

    let entity = world
        .create_entity()
        .with(Pos::from_xy_f32(pos.x, pos.y))
        .with(Tower::new())
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
) -> Result<Entity, failure::Error> {
    let hero_entity = player_hero_entity(world, "TowerPlace", owner_pid)?;

    let has_gold = {
        let golds = world.read_storage::<Gold>();
        golds.get(hero_entity).map(|gold| gold.0).unwrap_or(0) >= tpl.cost
    };
    if !has_gold {
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
        let clear = placement_radius + PATH_HALF_WIDTH;
        let clear_sq = clear * clear;
        for (name, path) in paths.iter() {
            let cps = &path.check_points;
            for i in 0..cps.len().saturating_sub(1) {
                let a = cps[i].pos;
                let b = cps[i + 1].pos;
                if point_segment_dist_sq(pos, a, b) < clear_sq {
                    return Err(failure::err_msg(format!(
                        "TowerPlace: blocked by path '{}' pid={} unit_id='{}'",
                        name, owner_pid, tpl.unit_id
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
        for (_entity, _tower, position, tag) in (&entities, &towers, &positions, tags.maybe()).join()
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

    Ok(hero_entity)
}

pub fn handle_tower_spawn_from_input(
    world: &mut World,
    kind_id: u32,
    pos: omoba_sim::Vec2,
    owner_pid: u32,
) -> Result<Entity, failure::Error> {
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
        reg.get(unit_id).cloned().ok_or_else(|| {
            failure::err_msg(format!("TowerPlace: unknown unit_id '{}'", unit_id))
        })?
    };
    let hero_entity = validate_tower_place_from_input(world, &tpl, pos_f32, owner_pid)?;
    let entity = spawn_td_tower(world, pos_f32, unit_id).ok_or_else(|| {
        failure::err_msg(format!(
            "spawn_td_tower returned None for unit_id='{}'",
            unit_id
        ))
    })?;
    {
        let mut golds = world.write_storage::<Gold>();
        if let Some(gold) = golds.get_mut(hero_entity) {
            gold.0 -= tpl.cost;
        }
    }
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
}

pub fn handle_tower_sell_from_input(
    world: &mut World,
    tower_entity_id: u32,
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

    let refund = {
        let tags = world.read_storage::<ScriptUnitTag>();
        let reg = world.read_resource::<TowerTemplateRegistry>();
        let towers = world.read_storage::<Tower>();
        let ureg = world.read_resource::<TowerUpgradeRegistry>();
        let base_refund = tags
            .get(target_entity)
            .and_then(|tag| reg.get(&tag.unit_id))
            .map(|tpl| (tpl.cost as f32 * 0.85) as i32)
            .unwrap_or(0);
        let upgrade_refund = if let (Some(tower), Some(tag)) =
            (towers.get(target_entity), tags.get(target_entity))
        {
            let mut total = 0i32;
            for path in 0..3u8 {
                for level in 1..=tower.upgrade_levels[path as usize] {
                    if let Some(def) = ureg.get(&tag.unit_id, path, level) {
                        total += (def.cost as f32 * 0.75) as i32;
                    }
                }
            }
            total
        } else {
            0
        };
        base_refund + upgrade_refund
    };

    if let Ok(hero_entity) = player_hero_entity(world, "TowerSell", owner_pid) {
        let mut golds = world.write_storage::<Gold>();
        if let Some(gold) = golds.get_mut(hero_entity) {
            gold.0 += refund;
        }
    }

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
}

pub fn handle_tower_upgrade_from_input(
    world: &mut World,
    tower_entity_id: u32,
    path: u8,
    _level_hint: u8,
    owner_pid: u32,
) -> Result<(), failure::Error> {
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

    let hero_entity = player_hero_entity(world, "TowerUpgrade", owner_pid)?;
    let has_gold = {
        let golds = world.read_storage::<Gold>();
        golds.get(hero_entity).map(|gold| gold.0).unwrap_or(0) >= def.cost
    };
    if !has_gold {
        return Err(failure::err_msg(format!(
            "TowerUpgrade: insufficient gold (need {}) for kind={} path={} level={} (pid={})",
            def.cost, unit_id, path, next_level, owner_pid
        )));
    }

    {
        let mut golds = world.write_storage::<Gold>();
        if let Some(gold) = golds.get_mut(hero_entity) {
            gold.0 -= def.cost;
        }
    }

    let mut flags_to_add = Vec::new();
    let mut stat_mods = Vec::new();
    for (effect_idx, effect) in def.effects.iter().enumerate() {
        match effect {
            UpgradeEffect::BehaviorFlag { flag } => flags_to_add.push(flag.clone()),
            UpgradeEffect::StatMod { key, value, op: _ } => {
                let buff_id = format!("upgrade_{}_{}_{}", path, next_level, effect_idx);
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

pub fn drain_pending_tower_upgrades(world: &mut World) {
    let drained = {
        let mut q = world.write_resource::<PendingTowerUpgradeQueue>();
        std::mem::take(&mut q.requests)
    };
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
}

pub fn handle_item_use_from_input(
    world: &mut World,
    item_slot: u32,
    _target_pos: Option<omoba_sim::Vec2>,
    _target_entity: Option<u32>,
    owner_pid: u32,
) -> Result<(), failure::Error> {
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
}
