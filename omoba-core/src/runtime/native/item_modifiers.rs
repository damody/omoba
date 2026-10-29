//! Source item datadriven actives (E160): named timed stat modifiers applied by an item use.
//! A modifier lands on the caster, or (`area`) on every living same-team hero within `radius`
//! of the caster, plus non-building units when `include_basic` (the source's
//! DOTA_UNIT_TARGET_BASIC). The modifier name is the BuffStore id, so reapplying it from any
//! item that shares the name refreshes instead of stacking, as the source's shared names do.
//! Recipients are a pure function of committed positions in canonical entity order, so every
//! world settling the use applies the same buffs.
use omoba_sim::Fixed64;
use serde_json::{Map, Value};
use specs::{Entity, Join, World, WorldExt};
use super::comp::{CProperty, Faction, Hero, IsBuilding, Pos, Tower};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemModifier {
    pub name: String,
    pub duration: Fixed64,
    #[serde(default)]
    pub area: bool,
    /// (StatKey name, raw stat value); percentages are fractions.
    pub stats: Vec<(String, Fixed64)>,
}

impl ItemModifier {
    /// Every stat names a known StatKey and the duration is a bounded positive lease.
    pub fn valid(&self) -> bool {
        (self.duration == Fixed64::ZERO || (self.duration > Fixed64::ZERO && self.duration <= Fixed64::from_i32(60))) && !self.stats.is_empty()
            && !self.name.is_empty() && !self.name.starts_with("__")
            && self.stats.iter().all(|(key, value)| omb_script_abi::stat_keys::StatKey::from_str_key(key).is_some()
                && *value != Fixed64::ZERO && value.raw().abs() <= Fixed64::from_i32(10_000).raw())
    }

    /// BuffStore lease: a source modifier without a Duration lasts until removed.
    pub fn lease(&self) -> Fixed64 {
        if self.duration == Fixed64::ZERO { crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION } else { self.duration }
    }

    pub fn payload(&self) -> Value {
        let mut map = Map::new();
        for (key, value) in &self.stats { map.insert(key.clone(), Value::from(value.raw())); }
        Value::Object(map)
    }
}

/// Living same-team heroes (plus non-building units with `include_basic`) within `radius` of
/// the caster, caster included, in canonical entity order.
pub fn area_recipients(world: &World, caster: Entity, radius: Fixed64, include_basic: bool) -> Vec<Entity> {
    let factions = world.read_storage::<Faction>();
    let positions = world.read_storage::<Pos>();
    let (Some(team), Some(center)) = (factions.get(caster).map(|f| f.team_id), positions.get(caster).map(|p| p.0)) else {
        return Vec::new();
    };
    let (entities, props) = (world.entities(), world.read_storage::<CProperty>());
    let (heroes, towers, buildings) = (world.read_storage::<Hero>(), world.read_storage::<Tower>(), world.read_storage::<IsBuilding>());
    let mut rows: Vec<Entity> = (&entities, &factions, &positions, &props).join()
        .filter(|(entity, faction, pos, prop)| faction.team_id == team && prop.hp > Fixed64::ZERO
            && (pos.0 - center).length_squared() <= radius * radius
            && (heroes.get(*entity).is_some()
                || (include_basic && towers.get(*entity).is_none() && buildings.get(*entity).is_none())))
        .map(|(entity, ..)| entity).collect();
    if !rows.contains(&caster) && props.get(caster).is_some_and(|p| p.hp > Fixed64::ZERO) { rows.push(caster); }
    rows.sort_by_key(|entity| (entity.id(), entity.gen().id()));
    rows
}

/// How long one aura refresh lasts: longer than a tick, so a receiver keeps the buff while it
/// stays in range and loses it this long after leaving (or after the holder dies or sells it).
pub const AURA_LINGER: Fixed64 = Fixed64::from_raw(512);

/// Passive item auras (E173), refreshed every settle: each living holder pulses each distinct
/// aura of its inventory through the same closed area settlement as hero auras
/// (ally_area_buff: living same-team non-building units in range, the holder included). The
/// aura name is the buff id, so the same aura from several holders never stacks. Authority and
/// replicas run this same settle from the inventories they know.
pub fn settle_item_auras(world: &mut World) {
    use super::comp::Inventory;
    let mut pulses = Vec::new();
    {
        let entities = world.entities();
        let (inventories, props, positions) = (world.read_storage::<Inventory>(), world.read_storage::<CProperty>(), world.read_storage::<Pos>());
        for (entity, inventory, prop, pos) in (&entities, &inventories, &props, &positions).join() {
            if prop.hp <= Fixed64::ZERO { continue; }
            let mut seen = std::collections::BTreeSet::new();
            for (_, item) in inventory.items() {
                let Some(aura) = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|row| row.id == item.item_id).and_then(|row| row.aura) else { continue };
                if !seen.insert(aura.name) { continue; }
                let spec = super::ally_area_buff::Spec { source_id: entity.id(), source_generation: entity.gen().id() as u32, center: pos.0,
                    radius: aura.radius, buff: aura.name.to_owned(), seconds: AURA_LINGER,
                    stats: aura.stats.iter().map(|(key, value)| ((*key).to_owned(), value.raw())).collect() };
                if let Some(spec) = spec.checked() { pulses.push(spec); }
            }
        }
    }
    // Hero (script) auras: emitter buffs on their casters pulse through the same settlement.
    pulses.extend(super::aura_emitter::pulses(world, AURA_LINGER));
    if pulses.is_empty() || world.try_fetch::<crate::runtime::BuffStore>().is_none() { return; }
    for spec in pulses {
        let stats = super::ally_area_buff::buff_payload(&spec);
        for ally in super::ally_area_buff::targets(world, &spec) {
            world.write_resource::<crate::runtime::BuffStore>().add(ally, &spec.buff, spec.seconds, stats.clone());
        }
    }
}

/// Every (recipient, modifier) the use applies, in deterministic order.
pub fn applications<'a>(world: &World, caster: Entity, radius: Fixed64, include_basic: bool, modifiers: &'a [ItemModifier])
    -> Vec<(Entity, &'a ItemModifier)>
{
    let area = modifiers.iter().any(|m| m.area).then(|| area_recipients(world, caster, radius, include_basic)).unwrap_or_default();
    let mut out = Vec::new();
    for modifier in modifiers {
        if modifier.area { out.extend(area.iter().map(|entity| (*entity, modifier))); } else { out.push((caster, modifier)); }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ability_runtime::BuffStore;
    use specs::Builder;

    fn modifier(name: &str, area: bool) -> ItemModifier {
        ItemModifier { name: name.into(), duration: Fixed64::from_i32(8), area,
            stats: vec![("movespeed_bonus_percentage".into(), Fixed64::from_raw(205))] }
    }

    fn unit(world: &mut World, team: i32, x: i32, hero: bool) -> Entity {
        let prop = CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
        let builder = world.create_entity()
            .with(Faction::new(super::super::comp::FactionType::Player, team))
            .with(Pos(omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO)))
            .with(prop);
        if hero { builder.with(Hero::default()).build() } else { builder.build() }
    }

    #[test]
    fn area_modifiers_reach_living_allies_in_radius_and_refresh_by_name() {
        let mut world = World::new();
        world.register::<Faction>(); world.register::<Pos>(); world.register::<CProperty>();
        world.register::<Hero>(); world.register::<Tower>(); world.register::<IsBuilding>();
        let caster = unit(&mut world, 1, 0, true);
        let near_hero = unit(&mut world, 1, 500, true);
        let near_creep = unit(&mut world, 1, 300, false);
        let far_hero = unit(&mut world, 1, 2000, true);
        let enemy = unit(&mut world, 2, 100, true);
        let building = unit(&mut world, 1, 100, false);
        world.write_storage::<IsBuilding>().insert(building, IsBuilding).unwrap();
        let mods = [modifier("modifier_self", false), modifier("modifier_area", true)];
        let heroes_only: Vec<_> = applications(&world, caster, Fixed64::from_i32(900), false, &mods).into_iter()
            .map(|(e, m)| (e, m.name.as_str())).collect();
        assert_eq!(heroes_only, vec![(caster, "modifier_self"), (caster, "modifier_area"), (near_hero, "modifier_area")]);
        let with_basic: Vec<_> = applications(&world, caster, Fixed64::from_i32(900), true, &mods[1..]).into_iter().map(|(e, _)| e).collect();
        assert!(with_basic.contains(&near_creep) && !with_basic.contains(&building));
        assert!(!with_basic.contains(&far_hero) && !with_basic.contains(&enemy));

        let mut buffs = BuffStore::new();
        for _ in 0..2 { buffs.add(caster, &mods[0].name, mods[0].duration, mods[0].payload()); }
        assert_eq!(buffs.iter_for(caster).count(), 1, "a shared name refreshes instead of stacking");
        assert_eq!(buffs.get(caster, "modifier_self").unwrap().payload["movespeed_bonus_percentage"].as_i64(), Some(205));
    }

    #[test]
    fn modifiers_must_name_known_stats() {
        assert!(modifier("m", false).valid());
        let mut bad = modifier("m", false); bad.stats[0].0 = "not_a_stat".into();
        assert!(!bad.valid());
        let mut reserved = modifier("__internal", false); reserved.area = true;
        assert!(!reserved.valid());
    }
}
