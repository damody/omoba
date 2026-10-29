//! Hero attributes: strength, agility and intelligence become combat stats with the coefficients of
//! templates/attribute_rules.lua (`omoba_template_ids::ATTRIBUTE_RULES`, Dota 2 rules), recomputed
//! every tick from the hero's level and its attribute modifiers (items, buffs).
//!
//! Max HP is written into `CProperty.mhp`, with what was added kept on the hero
//! (`Hero::attribute_hp`): the hero and its property are disclosed together, so a replica that
//! imports them never adds the attributes' HP a second time. Current HP keeps its ratio when the
//! maximum changes. Everything else (regen, armor, attack speed, max mana, magic resistance, attack
//! damage) is one permanent BuffStore entry that the existing readers sum. Authority and replicas
//! run the same settle.
//!
//! Only disclosed sources (the hero's own attributes and its item passives, rebuilt from the disclosed
//! inventory) count toward `Hero::attribute_hp`. Strength granted by any other buff (an ability, an
//! aura) lives only in the caster team's BuffStore: a replica of the other team never holds it, so its
//! max HP rides on the same entry as a HealthBonus, which `settle_health_bonus` applies where the buff
//! exists and an importing replica leaves alone (E163).
use super::ability_runtime::BuffStore;
use super::comp::{AttributeType, CProperty, Hero};
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use omoba_template_ids::AttributeRulesConst;
use specs::{Entity, Join, World, WorldExt};

/// The BuffStore entry that carries what a hero's attributes grant besides max HP.
pub const KEY: &str = "__attributes";

/// Attribute totals after level growth and attribute modifiers, never below zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttributeTotals {
    pub strength: Fixed64,
    pub agility: Fixed64,
    pub intelligence: Fixed64,
}

impl AttributeTotals {
    pub fn of(hero: &Hero, buffs: &BuffStore, entity: Entity) -> Self {
        let total = |base: Fixed64, key: StatKey| (base + buffs.sum_add(entity, key)).max(Fixed64::ZERO);
        Self {
            strength: total(hero.get_total_strength(), StatKey::StatsStrengthBonus),
            agility: total(hero.get_total_agility(), StatKey::StatsAgilityBonus),
            intelligence: total(hero.get_total_intelligence(), StatKey::StatsIntellectBonus),
        }
    }

    /// Totals from disclosed sources only: the hero's own attributes plus its item passives.
    pub fn disclosed(hero: &Hero, buffs: &BuffStore, entity: Entity) -> Self {
        let items = buffs.get(entity, super::item::ITEM_PASSIVES_KEY).map(|entry| &entry.payload);
        let total = |base: Fixed64, key: StatKey| (base + items.and_then(|payload| payload.get(key.as_str()))
            .and_then(serde_json::Value::as_i64).map_or(Fixed64::ZERO, Fixed64::from_raw)).max(Fixed64::ZERO);
        Self {
            strength: total(hero.get_total_strength(), StatKey::StatsStrengthBonus),
            agility: total(hero.get_total_agility(), StatKey::StatsAgilityBonus),
            intelligence: total(hero.get_total_intelligence(), StatKey::StatsIntellectBonus),
        }
    }

    fn primary(&self, primary: &AttributeType) -> Fixed64 {
        match primary {
            AttributeType::Strength => self.strength,
            AttributeType::Agility => self.agility,
            AttributeType::Intelligence => self.intelligence,
        }
    }
}

/// Max HP the attributes add.
pub fn max_hp(rules: &AttributeRulesConst, totals: &AttributeTotals) -> Fixed64 {
    totals.strength * rules.strength_health
}

/// A hero's own regeneration before attributes (template health_regen / mana_regen, from the
/// source's StatusHealthRegen / StatusManaRegen).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaseRegen {
    pub health: Fixed64,
    pub mana: Fixed64,
}

impl BaseRegen {
    pub fn of(hero: &Hero) -> Self {
        omoba_template_ids::hero_by_name(&hero.id).and_then(omoba_template_ids::active_hero_stats)
            .map_or(Self::default(), |stats| Self { health: stats.health_regen, mana: stats.mana_regen })
    }
}

/// The other stat modifiers the attributes (and the hero's base regeneration) grant, as a
/// BuffStore payload (raw Fixed64 values). `None` when they grant nothing.
pub fn modifiers(rules: &AttributeRulesConst, totals: &AttributeTotals, primary: &AttributeType, base: BaseRegen) -> Option<serde_json::Value> {
    let values = [
        (StatKey::HealthRegenConstant, base.health + totals.strength * rules.strength_health_regen),
        (StatKey::PhysicalArmorBonus, totals.agility * rules.agility_armor),
        (StatKey::AttackSpeedBonusConstant, totals.agility * rules.agility_attack_speed),
        (StatKey::ManaBonus, totals.intelligence * rules.intelligence_mana),
        (StatKey::ManaRegenConstant, base.mana + totals.intelligence * rules.intelligence_mana_regen),
        (StatKey::MagicalResistanceBonus, totals.intelligence * rules.intelligence_magic_resistance),
        (StatKey::BaseAttackBonusDamage, totals.primary(primary) * rules.primary_damage),
    ];
    let payload: serde_json::Map<String, serde_json::Value> = values.iter()
        .filter(|(_, value)| *value != Fixed64::ZERO)
        .map(|(key, value)| (key.as_str().to_owned(), value.raw().into()))
        .collect();
    (!payload.is_empty()).then_some(serde_json::Value::Object(payload))
}

/// Move a body's max HP by `delta`, keeping the current HP ratio. A dead body stays dead; the
/// maximum never drops below one raw HP.
fn shift_max_hp(property: &mut CProperty, delta: Fixed64) {
    let before = property.mhp;
    let after = (before + delta).max(Fixed64::from_raw(1));
    if property.hp > Fixed64::ZERO && before > Fixed64::ZERO {
        let scaled = i128::from(property.hp.raw()) * i128::from(after.raw()) / i128::from(before.raw());
        property.hp = Fixed64::from_raw(scaled.clamp(1, i128::from(after.raw())) as i64);
    }
    property.mhp = after;
}

/// Recompute every hero's attribute stats; unchanged heroes are left alone.
pub fn settle_hero_attributes(world: &mut World) {
    // The conversion is the original game's (equilibrium_constant.lua), for its heroes: a hero with
    // source unit stats (hero_source_stats). Test-only engine heroes keep their authored stats
    // (E172). A content decision, so the authority and every replica agree.
    settle_matching(world, &omoba_template_ids::ATTRIBUTE_RULES, |hero| {
        omoba_template_ids::hero_by_name(&hero.id).and_then(omoba_template_ids::hero_source_stats).is_some()
    });
}

/// The original game's hero move speed cap (templates/game_rules.lua) rides on the same entry.
fn with_hero_limits(payload: Option<serde_json::Value>, rules: Option<&omoba_template_ids::GameRulesConst>) -> Option<serde_json::Value> {
    let Some(rules) = rules else { return payload };
    let mut payload = payload.unwrap_or_else(|| serde_json::json!({}));
    payload[StatKey::MoveSpeedLimit.as_str()] = Fixed64::from_i32(rules.hero_max_move_speed.min(i32::MAX as u32) as i32).raw().into();
    Some(payload)
}

/// A hero's magic resistance (percent) under the match's game rules cap (templates/game_rules.lua).
pub fn cap_hero_magic_resistance(percent: Fixed64, rules: Option<&omoba_template_ids::GameRulesConst>) -> Fixed64 {
    rules.map_or(percent, |rules| percent.min(Fixed64::from_i32(rules.hero_max_magic_resistance.min(100) as i32)))
}

/// A hero's base magic resistance in percent (the CProperty::def_magic of a new body) under the
/// match's game rules; none without them.
pub fn hero_base_magic_resistance(rules: Option<&omoba_template_ids::GameRulesConst>) -> Fixed64 {
    Fixed64::from_i32(rules.map_or(0, |rules| rules.hero_base_magic_resistance.min(100) as i32))
}

fn settle_with(world: &mut World, rules: &AttributeRulesConst) {
    settle_matching(world, rules, |_| true);
}

fn settle_matching(world: &mut World, rules: &AttributeRulesConst, converts: impl Fn(&Hero) -> bool) {
    if world.try_fetch::<BuffStore>().is_none() { return; }
    // The move speed cap is a game rule of the match (none outside a match with them).
    let rules_of_match = world.try_fetch::<super::moba_match::MobaMatch>().and_then(|state| state.config.game_rules);
    let entities = world.entities();
    let mut heroes = world.write_storage::<Hero>();
    let mut properties = world.write_storage::<CProperty>();
    let mut buffs = world.write_resource::<BuffStore>();
    for (entity, hero) in (&entities, &mut heroes).join() {
        if !converts(hero) { continue; }
        let totals = AttributeTotals::of(hero, &buffs, entity);
        let disclosed = AttributeTotals::disclosed(hero, &buffs, entity);
        let mut next = modifiers(rules, &totals, &hero.primary_attribute, BaseRegen::of(hero));
        // Undisclosed strength: its max HP goes through HealthBonus (see the module doc).
        let undisclosed_hp = max_hp(rules, &totals) - max_hp(rules, &disclosed);
        if undisclosed_hp != Fixed64::ZERO {
            let row = next.get_or_insert_with(|| serde_json::Value::Object(Default::default()));
            row[StatKey::HealthBonus.as_str()] = undisclosed_hp.raw().into();
        }
        let next = with_hero_limits(next, rules_of_match.as_ref());
        if buffs.get(entity, KEY).map(|entry| &entry.payload) != next.as_ref() {
            match next {
                Some(payload) => buffs.add(entity, KEY, super::ability_runtime::buff_store::PERMANENT_BUFF_DURATION, payload),
                None => buffs.remove(entity, KEY),
            }
        }
        let target = max_hp(rules, &disclosed);
        if target == hero.attribute_hp { continue; }
        let Some(property) = properties.get_mut(entity) else { continue };
        shift_max_hp(property, target - hero.attribute_hp);
        hero.attribute_hp = target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ability_runtime::UnitStats;
    use specs::Builder;

    // The values of templates/attribute_rules.lua (6.87-7.05).
    fn rules() -> AttributeRulesConst {
        let f = |v: f32| Fixed64::from_raw((v * 1024.0).round() as i64);
        AttributeRulesConst { version: "6.87-7.05", strength_health: f(20.0), strength_health_regen: f(0.03),
            agility_armor: f(0.142857), agility_attack_speed: f(1.0), intelligence_mana: f(12.0),
            intelligence_mana_regen: f(0.04), intelligence_magic_resistance: f(0.0), primary_damage: f(1.0) }
    }

    fn hero(level: i32) -> Hero {
        let mut hero = Hero::default();
        hero.strength = 20; hero.agility = 30; hero.intelligence = 10;
        hero.primary_attribute = AttributeType::Agility;
        hero.level_growth.strength_per_level = Fixed64::from_i32(2);
        hero.level = level;
        hero
    }

    fn body(hp: i32, mhp: i32) -> CProperty {
        CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(mhp), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }
    }

    fn world() -> World {
        let mut world = World::new();
        world.register::<Hero>();
        world.register::<CProperty>();
        world.insert(BuffStore::new());
        world
    }

    #[test]
    fn attributes_follow_level_and_attribute_modifiers() {
        let mut world = world();
        let entity = world.create_entity().with(hero(1)).with(body(500, 500)).build();
        settle_with(&mut world, &rules());
        let mhp = |world: &World| { let p = world.read_storage::<CProperty>().get(entity).unwrap().clone(); (p.hp, p.mhp) };
        assert_eq!(mhp(&world), (Fixed64::from_i32(900), Fixed64::from_i32(900)), "a full body stays full: 500 + 20 strength * 20");
        {
            let buffs = world.read_resource::<BuffStore>();
            let stats = UnitStats::from_refs(&buffs, false);
            assert_eq!(stats.final_atk(Fixed64::ZERO, entity), Fixed64::from_i32(30), "primary agility is attack damage");
            assert_eq!(stats.final_armor(Fixed64::ZERO, entity), Fixed64::from_i32(30) * rules().agility_armor);
            assert_eq!(stats.final_attack_speed_mult(entity), Fixed64::ONE + Fixed64::from_i32(30) / Fixed64::from_i32(100));
            assert_eq!(stats.max_mp_bonus(entity), Fixed64::from_i32(120));
            assert_eq!(stats.max_hp_bonus(entity), Fixed64::ZERO, "max HP is not a HealthBonus buff");
        }
        // Half health; a level-up and an item's strength raise the maximum and keep the ratio.
        world.write_storage::<CProperty>().get_mut(entity).unwrap().hp = Fixed64::from_i32(450);
        world.write_storage::<Hero>().get_mut(entity).unwrap().level = 3;
        world.write_resource::<BuffStore>().add(entity, crate::runtime::native::item::ITEM_PASSIVES_KEY, Fixed64::from_i32(100),
            serde_json::json!({ StatKey::StatsStrengthBonus.as_str(): 6 * 1024 }));
        settle_with(&mut world, &rules());
        assert_eq!(mhp(&world), (Fixed64::from_i32(550), Fixed64::from_i32(1100)), "500 + (20 + 2*2 + 6) * 20, half of it");
        // Settling again changes nothing: the applied HP lives on the hero.
        let before = world.read_resource::<BuffStore>().get(entity, KEY).unwrap().payload.clone();
        settle_with(&mut world, &rules());
        assert_eq!(mhp(&world), (Fixed64::from_i32(550), Fixed64::from_i32(1100)));
        assert_eq!(world.read_resource::<BuffStore>().get(entity, KEY).unwrap().payload, before);
        // Losing the item lowers the maximum, ratio kept.
        world.write_resource::<BuffStore>().remove(entity, crate::runtime::native::item::ITEM_PASSIVES_KEY);
        settle_with(&mut world, &rules());
        assert_eq!(mhp(&world), (Fixed64::from_i32(490), Fixed64::from_i32(980)));
    }

    #[test]
    fn an_imported_hero_and_body_are_not_counted_twice() {
        // What a replica imports: the authority's body already holds the attributes' HP, and the
        // hero says how much. Settling locally changes nothing.
        let mut imported = hero(1);
        imported.attribute_hp = Fixed64::from_i32(400);
        let mut world = world();
        let entity = world.create_entity().with(imported).with(body(700, 900)).build();
        settle_with(&mut world, &rules());
        let p = world.read_storage::<CProperty>().get(entity).unwrap().clone();
        assert_eq!((p.hp, p.mhp), (Fixed64::from_i32(700), Fixed64::from_i32(900)));
    }

    #[test]
    fn undisclosed_strength_adds_max_hp_as_a_health_bonus_not_attribute_hp() {
        // An ability's strength is only in the caster team's BuffStore: its HP must not move
        // Hero::attribute_hp (disclosed), or a replica of the other team would diverge (E163).
        let mut world = world();
        let entity = world.create_entity().with(hero(1)).with(body(900, 900)).build();
        settle_with(&mut world, &rules());
        let attribute_hp = world.read_storage::<Hero>().get(entity).unwrap().attribute_hp;
        world.write_resource::<BuffStore>().add(entity, "ultimate", Fixed64::from_i32(8),
            serde_json::json!({ StatKey::StatsStrengthBonus.as_str(): 15 * 1024 }));
        settle_with(&mut world, &rules());
        assert_eq!(world.read_storage::<Hero>().get(entity).unwrap().attribute_hp, attribute_hp);
        assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().mhp, Fixed64::from_i32(1300), "unchanged here: settle_health_bonus applies the bonus");
        {
            let buffs = world.read_resource::<BuffStore>();
            assert_eq!(UnitStats::from_refs(&buffs, false).max_hp_bonus(entity), Fixed64::from_i32(300), "15 strength * 20");
        }
        world.write_resource::<BuffStore>().remove(entity, "ultimate");
        settle_with(&mut world, &rules());
        let buffs = world.read_resource::<BuffStore>();
        assert_eq!(UnitStats::from_refs(&buffs, false).max_hp_bonus(entity), Fixed64::ZERO);
    }

    #[test]
    fn a_dead_body_stays_dead_and_a_bare_hero_carries_nothing() {
        let mut world = world();
        let dead = world.create_entity().with(hero(1)).with(body(0, 500)).build();
        let mut bare = Hero::default();
        (bare.strength, bare.agility, bare.intelligence) = (0, 0, 0);
        let empty = world.create_entity().with(bare).with(body(100, 100)).build();
        settle_with(&mut world, &rules());
        let p = world.read_storage::<CProperty>().get(dead).unwrap().clone();
        assert_eq!((p.hp, p.mhp), (Fixed64::ZERO, Fixed64::from_i32(900)));
        // A hero with no attributes carries at most the game rules' move speed cap.
        assert!(world.read_resource::<BuffStore>().get(empty, KEY)
            .is_none_or(|entry| entry.payload.as_object().unwrap().keys().all(|key| key == StatKey::MoveSpeedLimit.as_str())));
        assert_eq!(world.read_storage::<CProperty>().get(empty).unwrap().mhp, Fixed64::from_i32(100));
    }

    #[test]
    fn negative_attribute_modifiers_floor_at_zero() {
        let mut buffs = BuffStore::new();
        let mut world = World::new();
        let entity = world.create_entity().build();
        buffs.add(entity, "curse", Fixed64::from_i32(5), serde_json::json!({ StatKey::StatsAgilityBonus.as_str(): -100 * 1024 }));
        let totals = AttributeTotals::of(&hero(1), &buffs, entity);
        assert_eq!(totals.agility, Fixed64::ZERO);
        let payload = modifiers(&rules(), &totals, &AttributeType::Agility, BaseRegen::default()).unwrap();
        assert!(payload.get(StatKey::PhysicalArmorBonus.as_str()).is_none());
    }
}
