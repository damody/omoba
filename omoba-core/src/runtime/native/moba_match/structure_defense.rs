//! The original game's structure defense (templates/game_rules.lua `structure_defense`, E187). Source:
//! damody/history_nobu @ f966720b, game/dota_addons/nobu/scripts/: vscripts/computer_system/chubing.lua:
//! 221-247 gives every npc_dota_tower (lane towers com_tower_* and general towers com_general_*)
//! modifier_unit_armor and modifier_tower_armor, and every tower, soldier camp and main base buff_tower;
//! vscripts/library/common/dummy.lua:510-639 defines them (MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE).
//!
//! The damage a structure takes is multiplied, in the damage pipeline's mitigation step
//! (`rules::damage::mitigate`), by 100 + percent over 100 for each modifier that applies: on lane and
//! general towers the attacker kind's `tower_armor` and `unit_armor` percents, and on every structure
//! the pressure bonus for a hero attacker while debuff_tower holds. Several incoming damage
//! percentages multiply (the engine's modifier stacking). Attacker kinds, from the original classes:
//!
//! | original attacker | here |
//! |---|---|
//! | hero (IsHero) | a `Hero` |
//! | npc_dota_creep_lane (the lane soldiers com_infantry/archer/gunner/cavalry) | a lane minion |
//! | npc_dota_tower com_tower_* / com_general_* | a lane tower / general (nexus) tower |
//! | npc_dota_creature (hero summons) | a `SummonedUnit` |
//! | npc_dota_creep_neutral (camps, bosses) | a jungle camp body |
//! | anything else | any other source, also one already removed |
//!
//! Only the authority settles structure damage (replicas skip disclosed targets in `rules::damage::admit`),
//! and nothing disclosed changes except the HP the damage facts already carry. Differences from the
//! source: modifier_tower_armor starts at 10 s there (before the first wave, so no lane damage
//! differs); debuff_tower is decided when the structure takes damage and holds for the next hits there,
//! here it is decided at the hit itself; dead heroes within the radius (FLAG_DEAD) are not counted.
use super::*;
use omoba_template_ids::{AttackerPercentsConst, StructureDefenseRulesConst};
use specs::Join;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AttackerKind { Hero, LaneCreep, LaneTower, GeneralTower, Summon, Neutral, Other }

impl AttackerKind {
    fn percent(self, percents: &AttackerPercentsConst) -> i32 {
        match self {
            Self::Hero => percents.hero, Self::LaneCreep => percents.lane_creep, Self::LaneTower => percents.lane_tower,
            Self::GeneralTower => percents.general_tower, Self::Summon => percents.summon, Self::Neutral => percents.neutral,
            Self::Other => percents.other,
        }
    }
}

pub(super) fn attacker_kind(world: &World, state: &MobaMatch, source: Entity) -> AttackerKind {
    if !world.entities().is_alive(source) { return AttackerKind::Other; }
    if world.read_storage::<Hero>().get(source).is_some() { return AttackerKind::Hero; }
    if let Some(unit) = state.units.get(&entity_key(source)).filter(|unit| unit.entity == source) {
        return match unit.role {
            LaneRole::Hero => AttackerKind::Hero,
            LaneRole::Creep => AttackerKind::LaneCreep,
            LaneRole::Tower => AttackerKind::LaneTower,
            LaneRole::NexusTower => AttackerKind::GeneralTower,
            LaneRole::Inhibitor | LaneRole::Base => AttackerKind::Other,
        };
    }
    if world.read_storage::<SummonedUnit>().get(source).is_some() { return AttackerKind::Summon; }
    if state.jungle_camps.iter().any(|camp| camp.contains(source)) { return AttackerKind::Neutral; }
    AttackerKind::Other
}

/// debuff_tower (dummy.lua:510-534): more than 3 enemy units without an owner (FindUnitsInRadius HERO
/// + BASIC, `not v:GetOwner()`: not heroes, which belong to their player, nor summons or wards) and at
/// least one enemy hero within the radius of the structure. Living units only; buildings are not BASIC.
fn under_pressure(world: &World, rules: &StructureDefenseRulesConst, structure: Entity) -> bool {
    let (positions, factions, properties) = (world.read_storage::<Pos>(), world.read_storage::<Faction>(), world.read_storage::<CProperty>());
    let (Some(center), Some(side)) = (positions.get(structure).map(|p| p.0), factions.get(structure)) else { return false };
    let (heroes, owners, summons, buildings) = (world.read_storage::<Hero>(), world.read_storage::<PlayerOwner>(),
        world.read_storage::<SummonedUnit>(), world.read_storage::<IsBuilding>());
    let radius = Fixed64::from_i32(rules.pressure_radius.min(i32::MAX as u32) as i32);
    let (mut unowned, mut hero_count) = (0u32, 0u32);
    for (entity, pos, faction, property) in (&world.entities(), &positions, &factions, &properties).join() {
        if property.hp <= Fixed64::ZERO || buildings.get(entity).is_some() || !faction.is_hostile_to(side)
            || (pos.0 - center).length_squared() > radius * radius { continue; }
        if heroes.get(entity).is_some() { hero_count += 1; }
        else if owners.get(entity).is_none() && summons.get(entity).is_none() { unowned += 1; }
    }
    unowned >= rules.pressure_min_unowned_units && hero_count >= rules.pressure_min_heroes
}

/// Exact factor (numerator, denominator) on the damage `source` deals to `target`, when `target` is a
/// structure of a match played by the original rules and the factor is not 1.
pub(crate) fn structure_damage_factor(world: &World, source: Entity, target: Entity) -> Option<(i64, i64)> {
    let state = world.try_fetch::<MobaMatch>()?;
    let rules = state.config.game_rules?.structure_defense?;
    let structure = state.units.get(&entity_key(target)).filter(|unit| unit.entity == target)?;
    let tower = match structure.role {
        LaneRole::Tower | LaneRole::NexusTower => true,
        LaneRole::Inhibitor | LaneRole::Base => false,
        LaneRole::Hero | LaneRole::Creep => return None,
    };
    let kind = attacker_kind(world, &state, source);
    let mut factor = (1i64, 1i64);
    let mut apply = |percent: i32| { factor = (factor.0 * i64::from(100 + percent), factor.1 * 100); };
    if tower {
        apply(kind.percent(&rules.tower_armor));
        apply(kind.percent(&rules.unit_armor));
    }
    if kind == AttackerKind::Hero && under_pressure(world, &rules, target) { apply(rules.pressure_hero_bonus_percent); }
    (factor.0 != factor.1).then_some(factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{process_outcomes, RuntimeEventVecSink, TowerTemplateRegistry};

    fn rules() -> StructureDefenseRulesConst {
        omoba_template_ids::GAME_RULES.and_then(|rules| rules.structure_defense).expect("game_rules.lua structure_defense")
    }
    fn fixture(map_id: &str, with_rules: bool) -> World {
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        let map = omoba_template_ids::moba_map_by_name(map_id).unwrap();
        setup_single_lane_match(&mut world, SingleLaneConfig { map_id: Some(map_id.into()), lane_length: Fixed64::from_i32(map.lane_length),
            authored_minion_waves: true, game_rules: with_rules.then(|| omoba_template_ids::GAME_RULES.unwrap()),
            ..SingleLaneConfig::default() }).unwrap();
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        let mut state = world.remove::<MobaMatch>().unwrap();
        state.spawn_wave(&mut world);
        jungle::spawn_waiting_camps(&mut world, &mut state);
        world.insert(state);
        world
    }
    fn structure(world: &World, team: u8, role: LaneRole) -> Entity {
        world.read_resource::<MobaMatch>().units.values().find(|u| u.team == team && u.role == role).unwrap().entity
    }
    fn creeps(world: &World, team: u8) -> Vec<Entity> {
        world.read_resource::<MobaMatch>().units.values().filter(|u| u.team == team && u.role == LaneRole::Creep).map(|u| u.entity).collect()
    }
    fn hero(world: &World, side: usize) -> Entity {
        world.read_resource::<MobaMatch>().heroes.iter().find(|slot| slot.side == side).unwrap().entity.unwrap()
    }
    fn put(world: &World, entity: Entity, at: Vec2) { world.write_storage::<Pos>().get_mut(entity).unwrap().0 = at; }
    /// Jungle camps are enemies of both sides (DOTA_UNIT_TARGET_TEAM_ENEMY finds neutrals too); keep
    /// them out of the counted radius.
    fn move_neutrals_away(world: &World) {
        let bodies: Vec<_> = world.read_resource::<MobaMatch>().jungle_camps.iter().flat_map(|camp| camp.entities().collect::<Vec<_>>()).collect();
        for body in bodies { put(world, body, Vec2::new(Fixed64::from_i32(100_000), Fixed64::from_i32(100_000))); }
    }
    fn at(world: &World, entity: Entity) -> Vec2 { world.read_storage::<Pos>().get(entity).unwrap().0 }
    fn factor(world: &World, source: Entity, target: Entity) -> f64 {
        structure_damage_factor(world, source, target).map_or(1.0, |(n, d)| n as f64 / d as f64)
    }

    #[test]
    fn structure_damage_follows_the_attacker_kind_on_towers_only() {
        // E187: dummy.lua:604-639 modifier_tower_armor x :545-568 modifier_unit_armor on npc_dota_tower.
        let r = rules();
        let mut world = fixture("three_lane_training", true);
        let (tower, general) = (structure(&world, 1, LaneRole::Tower), structure(&world, 1, LaneRole::NexusTower));
        let (inhibitor, base) = (structure(&world, 1, LaneRole::Inhibitor), structure(&world, 1, LaneRole::Base));
        let attacker_hero = hero(&world, 0);
        let creep = creeps(&world, 0)[0];
        let (own_tower, own_general) = (structure(&world, 0, LaneRole::Tower), structure(&world, 0, LaneRole::NexusTower));
        let neutral = world.read_resource::<MobaMatch>().jungle_camps.iter().find_map(|camp| camp.entities().next()).unwrap();
        let summon = world.create_entity().with(SummonedUnit::new_fixed(attacker_hero, None)).build();
        let other = world.create_entity().build();
        let gone = world.create_entity().build();world.delete_entity(gone).unwrap();world.maintain();
        let state = world.read_resource::<MobaMatch>();
        for (source, kind) in [(attacker_hero, AttackerKind::Hero), (creep, AttackerKind::LaneCreep), (own_tower, AttackerKind::LaneTower),
            (own_general, AttackerKind::GeneralTower), (summon, AttackerKind::Summon), (neutral, AttackerKind::Neutral),
            (other, AttackerKind::Other), (gone, AttackerKind::Other)] {
            assert_eq!(attacker_kind(&world, &state, source), kind);
        }
        drop(state);
        // The table follows game_rules.lua; the products are the source's numbers.
        for (source, kind, expected) in [(attacker_hero, AttackerKind::Hero, 1.0), (creep, AttackerKind::LaneCreep, 1.3 * 0.4),
            (own_tower, AttackerKind::LaneTower, 0.25), (own_general, AttackerKind::GeneralTower, 1.5),
            (summon, AttackerKind::Summon, 0.5), (neutral, AttackerKind::Neutral, 0.4), (other, AttackerKind::Other, 0.4)] {
            let data = f64::from(100 + kind.percent(&r.tower_armor)) * f64::from(100 + kind.percent(&r.unit_armor)) / 10_000.0;
            for target in [tower, general] {
                assert!((factor(&world, source, target) - data).abs() < 1e-12, "{kind:?}");
                assert!((factor(&world, source, target) - expected).abs() < 1e-12, "{kind:?}: the source's percents");
            }
            // Soldier camps and the main base carry only debuff_tower (npc_dota_building).
            assert_eq!(factor(&world, source, inhibitor), 1.0, "{kind:?}");
            assert_eq!(factor(&world, source, base), 1.0, "{kind:?}");
        }
        // Not a structure, or a match without the original rules: nothing.
        assert_eq!(structure_damage_factor(&world, creep, hero(&world, 1)), None);
        let world = fixture("three_lane_training", false);
        let (tower, creep) = (structure(&world, 1, LaneRole::Tower), creeps(&world, 0)[0]);
        assert_eq!(structure_damage_factor(&world, creep, tower), None);
    }

    #[test]
    fn heroes_deal_more_to_a_structure_with_four_unowned_enemies_and_a_hero_around() {
        // E187: dummy.lua:510-534 debuff_tower, :571-586 modifier_tower_debuff (+40 % from heroes).
        let r = rules();
        let world = fixture("three_lane_training", true);
        move_neutrals_away(&world);
        let bonus = f64::from(100 + r.pressure_hero_bonus_percent) / 100.0;
        let radius = Fixed64::from_i32(r.pressure_radius as i32);
        let attacker = hero(&world, 0);
        let creeps = creeps(&world, 0);
        assert!(creeps.len() as u32 >= r.pressure_min_unowned_units);
        for role in [LaneRole::Tower, LaneRole::NexusTower, LaneRole::Inhibitor, LaneRole::Base] {
            let target = structure(&world, 1, role);
            let center = at(&world, target);
            let near = center + Vec2::new(radius - Fixed64::from_i32(1), Fixed64::ZERO);
            let far = center + Vec2::new(radius + Fixed64::from_i32(1), Fixed64::ZERO);
            let armor = if matches!(role, LaneRole::Tower | LaneRole::NexusTower) {
                f64::from(100 + r.tower_armor.hero) * f64::from(100 + r.unit_armor.hero) / 10_000.0
            } else {1.0};
            put(&world, attacker, near);
            let min = r.pressure_min_unowned_units as usize;
            for (index, creep) in creeps.iter().enumerate() { put(&world, *creep, if index < min - 1 {near} else {far}); }
            assert_eq!(factor(&world, attacker, target), armor, "{role:?}: three are not more than 3");
            put(&world, creeps[min - 1], near);
            assert!((factor(&world, attacker, target) - armor * bonus).abs() < 1e-12, "{role:?}: four and a hero");
            // Only hero damage; other attackers keep their own factor.
            let creep_factor = factor(&world, creeps[0], target);
            put(&world, attacker, far);
            assert_eq!(factor(&world, creeps[0], target), creep_factor, "{role:?}");
            // The attacking hero itself must be in range (a hero in the radius is needed).
            assert_eq!(factor(&world, attacker, target), armor, "{role:?}: no hero around");
            put(&world, attacker, near);
            // A dead unit does not count.
            world.write_storage::<CProperty>().get_mut(creeps[0]).unwrap().hp = Fixed64::ZERO;
            assert_eq!(factor(&world, attacker, target), armor, "{role:?}: a dead creep");
            let hp = world.read_storage::<CProperty>().get(creeps[1]).unwrap().hp;
            world.write_storage::<CProperty>().get_mut(creeps[0]).unwrap().hp = hp;
        }
        // Summons have an owner and do not count; a friendly unit is not an enemy.
        let mut world = fixture("three_lane_training", true);
        move_neutrals_away(&world);
        let target = structure(&world, 1, LaneRole::Tower);
        let center = at(&world, target);
        put(&world, attacker_of(&world), center);
        let faction = world.read_storage::<Faction>().get(attacker_of(&world)).unwrap().clone();
        for _ in 0..r.pressure_min_unowned_units {
            let owner = attacker_of(&world);
            world.create_entity().with(SummonedUnit::new_fixed(owner, None)).with(Pos(center)).with(faction.clone())
                .with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
        }
        for creep in creeps_of(&world, 1) { put(&world, creep, center); }
        assert_eq!(factor(&world, attacker_of(&world), target), 1.0);
    }
    fn attacker_of(world: &World) -> Entity { hero(world, 0) }
    fn creeps_of(world: &World, team: u8) -> Vec<Entity> { creeps(world, team) }

    #[test]
    fn the_damage_pipeline_applies_the_structure_factor_to_the_hp_taken() {
        // E187: a lane soldier's 100 pure damage takes 52 HP from a lane tower (x1.3 x0.4), a hero's 100.
        let mut world = fixture("three_lane_training", true);
        let tower = world.read_resource::<MobaMatch>().lane_towers[0][1].unwrap();
        let creep = creeps(&world, 0)[0];
        let attacker = hero(&world, 0);
        put(&world, attacker, at(&world, tower) + Vec2::new(Fixed64::from_i32(5000), Fixed64::ZERO));
        let hp = |world: &World| world.read_storage::<CProperty>().get(tower).unwrap().hp;
        let hit = |world: &mut World, source: Entity| {
            world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO,
                real: Fixed64::from_i32(100), source, target: tower, damage_profile: omb_script_abi::types::DamageProfile::NORMAL.bits(),
                predeclared: false });
            process_outcomes(world, &mut RuntimeEventVecSink::default()).unwrap();
        };
        let before = hp(&world);
        hit(&mut world, creep);
        let r = rules();
        let expected = Fixed64::from_raw(100 * 1024 * i64::from(100 + r.tower_armor.lane_creep) * i64::from(100 + r.unit_armor.lane_creep) / 10_000);
        assert_eq!(before - hp(&world), expected);
        assert_eq!(expected, Fixed64::from_i32(52));
        let before = hp(&world);
        hit(&mut world, attacker);
        assert_eq!(before - hp(&world), Fixed64::from_i32(100));
    }

    #[test]
    fn the_main_base_cannot_be_damaged_while_a_general_tower_stands_within_the_guard_radius() {
        // E187: war_magic.lua:838-861 check_general_live: the base is invulnerable while a friendly
        // com_general is within 500. Every shipped map puts its general towers that close, and the base
        // stays locked until all of them have fallen (MobaMatch::base_unlocked), which covers the rule.
        let radius = Fixed64::from_i32(rules().base_general_radius as i32);
        for map_id in ["three_lane_training", "three_lane_layered_training", "three_lane_quick_training"] {
            let mut world = fixture(map_id, true);
            for team in 0..2usize {
                let (base, generals) = {
                    let state = world.read_resource::<MobaMatch>();
                    (state.bases[team].unwrap(), state.base_structures.iter().filter(|n| n.team == team && n.role == LaneRole::NexusTower)
                        .filter_map(|n| n.entity).collect::<Vec<_>>())
                };
                assert!(!generals.is_empty(), "{map_id}");
                for general in &generals {
                    let distance = (at(&world, *general) - at(&world, base)).length_squared();
                    assert!(distance <= radius * radius, "{map_id} team {team}: a general tower beyond the guard radius");
                }
                // Every lane tower and soldier camp of the side down: still locked while a general stands.
                let others: Vec<_> = {
                    let state = world.read_resource::<MobaMatch>();
                    state.lane_tower_layers.iter().flat_map(|lane| lane.iter().filter_map(|layer| layer[team])).chain(
                        state.base_structures.iter().filter(|n| n.team == team && n.role == LaneRole::Inhibitor).filter_map(|n| n.entity)).collect()
                };
                for entity in others { record_moba_death(&mut world, entity); world.delete_entity(entity).unwrap(); world.maintain(); }
                for general in &generals {
                    assert!(!moba_damage_allowed(&world, base), "{map_id} team {team}: a general within the radius stands");
                    record_moba_death(&mut world, *general); world.delete_entity(*general).unwrap(); world.maintain();
                }
                assert!(moba_damage_allowed(&world, base), "{map_id} team {team}: no general left");
            }
        }
    }
}
