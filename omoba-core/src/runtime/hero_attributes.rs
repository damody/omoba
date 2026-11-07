//! Current filtered-world attributes. These are presentation metadata, never
//! another gameplay state, a retained enemy observation, or a template estimate.
use crate::game_proto::HeroCombatAttributesPresentation;
use crate::runtime::{ability_runtime::UnitStats, CProperty, TAttack};
use specs::Entity;

#[derive(Clone, Debug, PartialEq)]
pub struct DisclosedHeroAttributes {
    pub player_id: u32,
    pub replica_tick: u64,
    pub values: HeroCombatAttributesPresentation,
}

pub fn valid_attributes(value: &HeroCombatAttributesPresentation) -> bool {
    const LIMIT: i64 = 1_000_000 * 1024;
    (0..=LIMIT).contains(&value.attack_damage_raw)
        && (-LIMIT..=LIMIT).contains(&value.physical_armor_raw)
        && (-1024..=1024).contains(&value.magic_resistance_raw)
        && (0..=LIMIT).contains(&value.move_speed_raw)
        && (0..=LIMIT).contains(&value.attack_range_raw)
        && (0..=LIMIT).contains(&value.attack_interval_raw)
}

/// Uses exactly the helpers consumed by combat/movement, after all currently
/// disclosed modifiers. Callers must have both real components; no defaults.
pub fn query_attributes(
    property: &CProperty,
    attack: &TAttack,
    stats: &UnitStats<'_>,
    entity: Entity,
) -> Option<HeroCombatAttributesPresentation> {
    let value = HeroCombatAttributesPresentation {
        attack_damage_raw: stats.final_atk(attack.atk_physic.v, entity).raw(),
        physical_armor_raw: stats.final_armor(property.def_physic, entity).raw(),
        // def_magic is percent (the damage paths' unit); the hero's pool is capped by the game rules.
        magic_resistance_raw: {
            let hundred = omoba_sim::Fixed64::from_i32(100);
            let percent = crate::runtime::native::rules::magic_resist::hero_percent(
                stats, entity, property.def_magic, omoba_template_ids::GAME_RULES.as_ref());
            (percent / hundred).raw()
        },
        move_speed_raw: stats.final_move_speed(property.msd, entity).raw(),
        attack_range_raw: stats.final_attack_range(attack.range.v, entity).raw(),
        attack_interval_raw: stats.final_attack_interval(attack.asd.v, entity).raw(),
    };
    valid_attributes(&value).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::*;
    use omb_script_abi::stat_keys::StatKey;
    use omoba_sim::Fixed64;
    use specs::{Builder, World, WorldExt};
    use std::collections::BTreeMap;

    fn property() -> CProperty {
        CProperty { hp: Fixed64::from_i32(50), mhp: Fixed64::from_i32(100), msd: Fixed64::from_i32(300),
            def_physic: Fixed64::from_i32(-3), def_magic: Fixed64::from_i32(25) }
    }

    fn modifier() -> serde_json::Value {
        serde_json::json!({
            StatKey::PreattackBonusDamage.as_str(): 10*1024,
            StatKey::PhysicalArmorBonus.as_str(): 5*1024,
            StatKey::MagicalResistanceBonus.as_str(): 25*1024,
            StatKey::MoveSpeedBonusEquipment.as_str(): 90*1024,
            StatKey::MoveSpeedBonus.as_str(): -512,
            StatKey::MoveSpeedBonusBuff.as_str(): 60*1024,
            StatKey::AttackRangeBonus.as_str(): 50*1024,
            StatKey::AttackSpeedBonusConstant.as_str(): 100*1024,
        })
    }

    #[test]
    fn current_modifiers_are_shared_combat_values_and_removal_is_observed() {
        let mut world = World::new(); let entity = world.create_entity().build();
        let attack = TAttack::new(Fixed64::from_i32(40), Fixed64::ONE, Fixed64::from_i32(550), Fixed64::ONE);
        let mut buffs = ability_runtime::BuffStore::new();
        let initial = query_attributes(&property(), &attack, &UnitStats::from_refs(&buffs, false), entity).unwrap();
        assert_eq!((initial.attack_damage_raw, initial.physical_armor_raw, initial.magic_resistance_raw), (40*1024, -3*1024, 256));
        buffs.add(entity, "attributes_test", Fixed64::from_i32(20), modifier());
        let current = query_attributes(&property(), &attack, &UnitStats::from_refs(&buffs, false), entity).unwrap();
        assert_eq!((current.attack_damage_raw, current.physical_armor_raw, current.magic_resistance_raw), (50*1024, 2*1024, 448));
        assert_eq!((current.move_speed_raw, current.attack_range_raw, current.attack_interval_raw), (255*1024, 600*1024, 512));
        buffs.remove(entity, "attributes_test");
        assert_eq!(query_attributes(&property(), &attack, &UnitStats::from_refs(&buffs, false), entity).unwrap(), initial);
    }

    #[test]
    fn attributes_validate_signed_defense_fraction_and_bounded_zero_values() {
        let zero = HeroCombatAttributesPresentation::default(); assert!(valid_attributes(&zero));
        for bad in [HeroCombatAttributesPresentation { attack_damage_raw: -1, ..zero.clone() },
            HeroCombatAttributesPresentation { physical_armor_raw: i64::MIN, ..zero.clone() },
            HeroCombatAttributesPresentation { magic_resistance_raw: 1025, ..zero.clone() },
            HeroCombatAttributesPresentation { magic_resistance_raw: -1025, ..zero.clone() },
            HeroCombatAttributesPresentation { move_speed_raw: 1_000_000*1024+1, ..zero.clone() },
            HeroCombatAttributesPresentation { attack_range_raw: -1, ..zero.clone() },
            HeroCombatAttributesPresentation { attack_interval_raw: -1, ..zero.clone() }] {
            assert!(!valid_attributes(&bad));
        }
        assert!(valid_attributes(&HeroCombatAttributesPresentation { physical_armor_raw: -1024, magic_resistance_raw: -1024, ..zero }));
    }

    fn disclosed(id: u64, epoch: u64, player: u32) -> ReplicaEntityState {
        let hero = Hero::new("training_luminary".into(), "".into(), "".into());
        let attack = TAttack::new(Fixed64::from_i32(40), Fixed64::ONE, Fixed64::from_i32(550), Fixed64::ONE);
        ReplicaEntityState { replica_id: id, disclosure_epoch: epoch, authority_revision: 1, entity_kind: 1,
            components: BTreeMap::from([
                (DEMO_RENDER_COMPONENT_SCHEMA_ID, encode_demo_render_state(DemoRenderState { x_raw: 0, y_raw: 0, team_id: 1, kind: 1, owner_player_id: player })),
                (DISCLOSED_HERO_COMPONENT_SCHEMA_ID, visibility::canonical_disclosed_json(&hero)),
                (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, encode_disclosed_property(&property())),
                (DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID, visibility::canonical_disclosed_json(&attack)),
            ]) }
    }

    #[test]
    fn real_filtered_stepper_scopes_current_body_and_drops_missing_or_conflicting_owners() {
        let start = crate::game_proto::TeamGameStart { team_id: 1, tick_rate_hz: 60, replica_start_tick: 7, ..Default::default() };
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, secure_replica_component_allowlist(), Default::default());
        let mut source = DisclosedReplicaWorld { entities: BTreeMap::from([(1, disclosed(1,1,7))]), ..Default::default() };
        stepper.bootstrap_membership(&source).unwrap();
        let entity = stepper.filtered.entities.0[&1].entity;
        stepper.filtered.world.write_resource::<ability_runtime::BuffStore>().add(entity, "attributes_test", Fixed64::from_i32(20), modifier());
        let current = stepper.disclosed_hero_attributes();
        assert_eq!((current[&(1,1)].player_id, current[&(1,1)].replica_tick, current[&(1,1)].values.attack_damage_raw), (7,7,50*1024));
        stepper.filtered.world.write_storage::<CProperty>().remove(entity);
        assert!(stepper.disclosed_hero_attributes().is_empty());
        stepper.filtered.world.write_storage::<CProperty>().insert(entity, property()).unwrap();
        source.entities.insert(2, disclosed(2,1,7)); stepper.bootstrap_membership(&source).unwrap();
        assert!(stepper.disclosed_hero_attributes().is_empty());
        source.entities.remove(&2);source.entities.insert(1,disclosed(1,2,7));stepper.bootstrap_membership(&source).unwrap();
        let replaced = stepper.disclosed_hero_attributes();
        assert!(!replaced.contains_key(&(1,1)));assert_eq!(replaced[&(1,2)].values.attack_damage_raw,40*1024);
        assert!(!stepper.filtered.world.read_resource::<ability_runtime::BuffStore>().has_any(entity));
        let replacement = stepper.filtered.entities.0[&1].entity;
        stepper.filtered.world.write_resource::<ability_runtime::BuffStore>().add(replacement,"attributes_test",Fixed64::from_i32(20),modifier());
        source.entities.clear();stepper.bootstrap_membership(&source).unwrap();assert!(stepper.disclosed_hero_attributes().is_empty());
        assert!(!stepper.filtered.world.read_resource::<ability_runtime::BuffStore>().has_any(replacement));
        source.entities=(1..=11).map(|id|(id,disclosed(id,1,id as u32))).collect();stepper.bootstrap_membership(&source).unwrap();
        assert!(stepper.disclosed_hero_attributes().is_empty());
    }
}

