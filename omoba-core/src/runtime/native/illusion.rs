//! v3 `create_illusion { of, at, duration, control, copy, outgoing, incoming }` (原作
//! CreateIllusions): a summon of the `illusion` template (summons.lua; placement, owner orders,
//! AI, lifetime and disclosure are the summon's) that a one-shot envelope (never stored) turns into
//! a copy of `of` right after its spawn, in the same batch:
//! - life, speed, armor, magic resistance, attack, collision, vision and facing are `of`'s at that
//!   moment, with the stat rows of its level and items ([`BAKED`]) kept as they are then; `hp` sets
//!   its life (capped at its max);
//! - `abilities` (v3 `copy.abilities`, less `except_abilities`): `of`'s learned ranks, which the
//!   illusion's scripts read as its own ([`ability_level`]), and its learned passives' rows, whose
//!   events the illusion hears as their source;
//! - `mana` (v3 `copy.mana`): a mana pool of `of`'s capacity holding that much, regenerating at
//!   `of`'s natural rate (unit_mana);
//! - `statuses` (v3 `copy.statuses`): the status rows `of` holds, with their remaining time, only
//!   those carrying one of `tags` (catalog status tags) when any are named, never those `except`
//!   names; `original_duration` gives each its full duration instead (the kit records it as
//!   [`DURATION_KEY`]), `illusion_caster` makes the illusion its applier (its `{caster}` key, its
//!   events' source). A status that holds a shared control comes with its control-source row and
//!   the controls it holds. Other host bookkeeping rows (`__…`) are not copied;
//! - the unit holds the [`STATE`] status (`is_illusion`) for its lifetime, whose payload carries the
//!   damage it deals and takes as fractions per type (`outgoing` / `incoming`, 原作
//!   outgoing_damage / incoming_damage): [`scale_damage`] (rules::damage, before mitigation)
//!   multiplies a packet by 1 + x of its source's outgoing and of its target's incoming row.
//! Presentation: the [`STATE`] payload's `look` is `of`'s presentation identity, which every team
//! that sees the illusion receives as its identity ([`look`], visibility); only the owner team
//! learns it is an illusion (the `illusion` state, runtime_fact_state).
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use super::comp::{CProperty, CollisionRadius, Facing, SummonedUnit, TAttack, Tick, Unit, VisionSource};
use crate::runtime::scripting::ScriptUnitTag;
use crate::runtime::BuffStore;

pub const TEMPLATE: &str = "illusion";
pub const BUFF: &str = "__illusion_v1";
pub const STATE: &str = super::rules::vocab::state::ILLUSION;
const LIMIT: i64 = 64 * 1024;
/// Host stat rows that carry a hero's level and items (attributes.rs, item.rs).
pub const BAKED: [&str; 2] = [super::attributes::KEY, super::item::ITEM_PASSIVES_KEY];

/// Per damage type fractions (physical, magical, pure): `-1` = none, `0` = unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Scale(pub [Fixed64; 3]);

impl Scale {
    fn row(self) -> Value { json!(self.0.map(|x| x.raw())) }
    fn of(value: Option<&Value>) -> Option<Scale> {
        let raw = value?.as_array().filter(|row| row.len() == 3)?;
        let mut out = [Fixed64::ZERO; 3];
        for (slot, value) in out.iter_mut().zip(raw) {
            let raw = value.as_i64().filter(|raw| (-1024..=LIMIT).contains(raw))?;
            *slot = Fixed64::from_raw(raw);
        }
        Some(Scale(out))
    }
}

/// The payload key where the kit records a status's full duration (raw), read by
/// `status_duration = "original"`.
pub const DURATION_KEY: &str = "status_duration_raw";

/// v3 `copy.statuses` with its options.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatusCopy {
    /// Only statuses carrying one of these catalog tags (empty: every status).
    pub tags: Vec<String>,
    /// Never these v3 status ids (`status_except`).
    pub except: Vec<String>,
    /// `status_duration = "original"`: each status's full duration, not its remaining time.
    pub original_duration: bool,
    /// `status_caster = "illusion"`: the illusion is the copied statuses' applier.
    pub illusion_caster: bool,
}

/// What an illusion copies besides its body, level and items.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Copy {
    /// Its life (None = `of`'s current life).
    pub hp: Option<Fixed64>,
    /// Its mana (None = no mana pool).
    pub mana: Option<Fixed64>,
    /// `copy.abilities`: Some(except_abilities) copies the learned ranks and passives.
    pub abilities: Option<Vec<String>>,
    pub statuses: Option<StatusCopy>,
    pub outgoing: Scale,
    pub incoming: Scale,
}

/// The envelope payload.
pub fn payload(of_id: u32, of_generation: u32, copy: &Copy) -> Value {
    json!({ "of": [of_id, of_generation], "hp": copy.hp.map(|hp| hp.raw()), "mana": copy.mana.map(|mana| mana.raw()),
        "abilities": copy.abilities, "statuses": copy.statuses.as_ref().map(|s| json!({ "tags": s.tags, "except": s.except,
            "original_duration": s.original_duration, "illusion_caster": s.illusion_caster })),
        "outgoing": copy.outgoing.row(), "incoming": copy.incoming.row() })
}

fn strings(value: &Value) -> Option<Vec<String>> {
    value.as_array()?.iter().map(|v| v.as_str().map(str::to_owned)).collect()
}

fn parse(payload: &Value) -> Option<(u32, u32, Copy)> {
    let row = payload.as_object().filter(|row| row.len() == 7)?;
    // None when absent; Some(None) when null; a value must be at least `min` raw.
    let amount = |key: &str, min: i64| match row.get(key)? { Value::Null => Some(None), value =>
        value.as_i64().filter(|raw| *raw >= min).map(|raw| Some(Fixed64::from_raw(raw))) };
    let hp = amount("hp", 1)?;
    let abilities = match row.get("abilities")? { Value::Null => None, value => Some(strings(value)?) };
    let statuses = match row.get("statuses")? {
        Value::Null => None,
        value => {
            let s = value.as_object().filter(|s| s.len() == 4)?;
            Some(StatusCopy { tags: strings(s.get("tags")?)?, except: strings(s.get("except")?)?,
                original_duration: s.get("original_duration")?.as_bool()?, illusion_caster: s.get("illusion_caster")?.as_bool()? })
        }
    };
    let of = row.get("of").and_then(Value::as_array).filter(|pair| pair.len() == 2)?;
    let (id, generation) = (u32::try_from(of[0].as_u64()?).ok()?, u32::try_from(of[1].as_u64()?).ok()?);
    Some((id, generation, Copy { hp, mana: amount("mana", 0)?, abilities, statuses,
        outgoing: Scale::of(row.get("outgoing"))?, incoming: Scale::of(row.get("incoming"))? }))
}

/// Settles one envelope on `illusion`, an `illusion` summon spawned this tick, and returns `of`;
/// None when it is malformed, `of` is not a living unit, or `illusion` is anything else.
/// Its max life is `of`'s with `of`'s folded max-life bonus; the caller refolds it from the
/// illusion's own rows (game_processor `settle_health_bonus`).
pub fn apply(world: &mut World, illusion: Entity, payload: &Value) -> Option<Entity> {
    apply_with(super::dispel::registry(), world, illusion, payload)
}

/// [`apply`] with the status catalog `registry` (tags, status ids, per-source keys).
pub(crate) fn apply_with(registry: &super::dispel::Registry, world: &mut World, illusion: Entity, payload: &Value) -> Option<Entity> {
    let (id, generation, copy) = parse(payload)?;
    let of = { let entity = world.entities().entity(id);
        (entity.gen().id() as u32 == generation && world.entities().is_alive(entity)).then_some(entity)? };
    let hp = copy.hp;
    let tick = world.read_resource::<Tick>().0;
    let fresh = world.entities().is_alive(illusion) && of != illusion
        && world.read_storage::<SummonedUnit>().get(illusion).is_some_and(|summon| summon.spawn_tick == tick)
        && world.read_storage::<ScriptUnitTag>().get(illusion).is_some_and(|tag| tag.unit_id == TEMPLATE);
    let source = world.read_storage::<CProperty>().get(of).filter(|p| p.hp > Fixed64::ZERO).cloned()?;
    if !fresh { return None; }
    let look = world.read_storage::<ScriptUnitTag>().get(of).map(|tag| tag.unit_id.clone()).filter(|id| !id.is_empty())
        .or_else(|| world.read_storage::<Unit>().get(of).map(|unit| unit.id.clone()));
    let life = hp.unwrap_or(source.hp).min(source.mhp);
    world.write_storage::<CProperty>().insert(illusion, CProperty { hp: life, ..source.clone() }).ok();
    let attack = world.read_storage::<TAttack>().get(of).cloned();
    if let Some(mut copy) = attack {
        copy.asd_count = Fixed64::ZERO; copy.attack_seq = 0; copy.attack_phase = Default::default(); copy.committed_target = None;
        world.write_storage::<TAttack>().insert(illusion, copy).ok();
    }
    for_copy::<CollisionRadius>(world, of, illusion);
    for_copy::<Facing>(world, of, illusion);
    for_copy::<VisionSource>(world, of, illusion);
    let body = world.read_storage::<Unit>().get(of).map(|unit| (unit.base_damage, unit.base_armor, unit.magic_resistance, unit.attack_range, unit.attack_speed));
    if let Some(unit) = world.write_storage::<Unit>().get_mut(illusion) {
        unit.max_hp = (source.mhp.raw() / 1024) as i32; unit.current_hp = (life.raw() / 1024) as i32; unit.move_speed = source.msd;
        if let Some((damage, armor, resistance, range, speed)) = body {
            (unit.base_damage, unit.base_armor, unit.magic_resistance, unit.attack_range, unit.attack_speed) = (damage, armor, resistance, range, speed);
        }
    }
    let remaining = world.read_storage::<SummonedUnit>().get(illusion).and_then(|summon| summon.time_remaining)
        .unwrap_or(crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION);
    if let Some(amount) = copy.mana { copy_mana(world, of, illusion, amount); }
    let ranks: serde_json::Map<String, Value> = match &copy.abilities {
        Some(except) => world.read_storage::<super::comp::Hero>().get(of).map(|hero| hero.ability_levels.iter()
            .filter(|(id, rank)| **rank > 0 && !except.contains(id)).map(|(id, rank)| (id.clone(), json!(rank))).collect())
            .unwrap_or_default(),
        None => Default::default(),
    };
    let mut buffs = world.write_resource::<BuffStore>();
    for (id, left, payload) in copied_rows(registry, &buffs, of, illusion, &copy) {
        if left > Fixed64::ZERO { buffs.add(illusion, &id, left, payload); }
    }
    for id in BAKED {
        if let Some(entry) = buffs.get(of, id).cloned() { buffs.add(illusion, id, entry.remaining, entry.payload); }
    }
    buffs.add(illusion, STATE, remaining, json!({ "outgoing": copy.outgoing.row(), "incoming": copy.incoming.row(), "look": look,
        "abilities": ranks }));
    Some(of)
}

/// A mana pool of `of`'s capacity holding `amount`, regenerating at `of`'s natural rate (a hero's
/// is the MOBA rate; its bonuses come with the copied rows).
fn copy_mana(world: &mut World, of: Entity, illusion: Entity, amount: Fixed64) {
    use super::unit_mana::{registered, UnitMana};
    let Some(source) = super::unit_mana::pool(world, of) else { return };
    if !registered(world) { return; }
    let regen = world.read_storage::<UnitMana>().get(of).map_or_else(
        || Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32), |mana| mana.regen);
    let Ok(pool) = crate::runtime::ability_runtime::ManaPool::new(amount.min(source.maximum()), source.maximum()) else { return };
    let _ = world.write_storage::<UnitMana>().insert(illusion, UnitMana { pool, regen });
}

/// The rows of `of` the illusion takes: its learned passives with `abilities` (their events'
/// source the illusion), the statuses `statuses` selects with their control-source rows and
/// controls.
fn copied_rows(registry: &super::dispel::Registry, buffs: &BuffStore, of: Entity, illusion: Entity, copy: &Copy) -> Vec<(String, Fixed64, Value)> {
    use crate::runtime::native::ability_runtime::buff_store::is_passive;
    use super::dispel::CONTROL_SOURCE_PREFIX;
    let handle = |e: Entity| (e.id(), e.gen().id() as u32);
    let (from, to) = (handle(of), handle(illusion));
    let mut rows = Vec::new();
    // Learned passives: the illusion's own abilities.
    if let Some(except) = &copy.abilities {
        for (id, entry) in buffs.iter_for(of).filter(|(_, entry)| is_passive(&entry.payload)) {
            if except.iter().any(|ability| ability == omoba_template_ids::instance_base(id)) { continue; }
            rows.push((id.to_owned(), entry.remaining, recast(&entry.payload, to)));
        }
    }
    let Some(statuses) = &copy.statuses else { return rows };
    // The shared controls the source rows hold come with their statuses, not on their own.
    let controls: std::collections::BTreeSet<String> = buffs.iter_for(of).filter(|(id, _)| id.starts_with(CONTROL_SOURCE_PREFIX))
        .flat_map(|(_, entry)| super::dispel::control_states(&entry.payload)).collect();
    for (id, entry) in buffs.iter_for(of) {
        let control_row = id.strip_prefix(CONTROL_SOURCE_PREFIX);
        if (id.starts_with("__") && control_row.is_none()) || id == STATE || BAKED.contains(&id)
            || is_passive(&entry.payload) || controls.contains(id) { continue; }
        let status = control_row.unwrap_or(id);
        let ids = registry.statuses_of(status);
        let named = |list: &[String]| list.iter().any(|s| s == omoba_template_ids::instance_base(status) || ids.contains(&s.as_str()));
        if named(&statuses.except) { continue; }
        if !statuses.tags.is_empty() && !registry.status_tags(status).iter().any(|tag| statuses.tags.iter().any(|t| t == tag)) { continue; }
        let left = if statuses.original_duration {
            entry.payload.get(DURATION_KEY).and_then(Value::as_i64).map_or(entry.remaining, Fixed64::from_raw)
        } else { entry.remaining };
        let key = if statuses.illusion_caster { registry.recast_key(id, from, to) } else { id.to_owned() };
        rows.push((key, left, if statuses.illusion_caster { recast(&entry.payload, to) } else { entry.payload.clone() }));
        // The controls this status holds, for its time.
        if control_row.is_some() {
            for state in super::dispel::control_states(&entry.payload) {
                let payload = buffs.get(of, &state).map_or(Value::Null, |entry| entry.payload.clone());
                rows.push((state, left, payload));
            }
        }
    }
    rows
}

/// The payload with the illusion as its applier (`source_entity_id/gen`) and as its status events'
/// source.
fn recast(payload: &Value, to: (u32, u32)) -> Value {
    let mut payload = payload.clone();
    if let Some(row) = payload.as_object_mut() {
        if row.contains_key("source_entity_id") {
            row.insert("source_entity_id".into(), to.0.into());
            row.insert("source_entity_gen".into(), to.1.into());
        }
        if let Some(events) = row.get_mut(super::status_event::KEY).and_then(Value::as_object_mut) {
            events.insert("source".into(), json!([to.0, to.1]));
        }
    }
    payload
}

/// The rank of `ability` an illusion copied (v3 `copy.abilities`); 0 for anything else.
pub fn ability_level(buffs: &BuffStore, entity: Entity, ability: &str) -> i32 {
    buffs.get(entity, STATE).and_then(|row| row.payload.get("abilities")?.get(ability)?.as_i64())
        .map_or(0, |rank| i32::try_from(rank).unwrap_or(0).max(0))
}

fn for_copy<C: specs::Component + Clone>(world: &mut World, from: Entity, to: Entity) {
    let value = world.read_storage::<C>().get(from).cloned();
    if let Some(value) = value { let _ = world.write_storage::<C>().insert(to, value); }
}

/// The presentation identity an illusion shows: the unit it copies.
pub fn look(buffs: &BuffStore, entity: Entity) -> Option<&str> {
    buffs.get(entity, STATE)?.payload.get("look")?.as_str()
}

/// rules::damage, before mitigation: an illusion's outgoing and incoming fractions per type.
pub fn scale_damage(world: &World, packet: &mut super::rules::damage::Packet) {
    let buffs = world.read_resource::<BuffStore>();
    let scale = |entity: Entity, key: &str| buffs.get(entity, STATE).and_then(|row| Scale::of(row.payload.get(key)));
    let outgoing = if packet.without_source_bonuses() { None } else { scale(packet.source, "outgoing") };
    for scale in [outgoing, scale(packet.target, "incoming")].into_iter().flatten() {
        let [p, m, t] = scale.0.map(|x| (Fixed64::ONE + x).max(Fixed64::ZERO));
        (packet.physical, packet.magical, packet.pure) = (packet.physical * p, packet.magical * m, packet.pure * t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;
    use super::super::comp::Faction;

    fn world() -> World {
        let mut world = World::new();
        world.register::<CProperty>(); world.register::<TAttack>(); world.register::<SummonedUnit>(); world.register::<Unit>();
        world.register::<CollisionRadius>(); world.register::<Facing>(); world.register::<VisionSource>(); world.register::<ScriptUnitTag>();
        world.register::<Faction>(); world.register::<super::super::comp::Hero>(); world.register::<super::super::unit_mana::UnitMana>();
        world.insert(Tick(7)); world.insert(BuffStore::new());
        world
    }

    fn summon(world: &mut World, template: &str, tick: u64) -> Entity {
        let mut summoned = SummonedUnit::new_fixed(world.create_entity().build(), Some(Fixed64::from_i32(20)));
        summoned.spawn_tick = tick;
        summoned.time_remaining = Some(Fixed64::from_i32(20));
        world.create_entity().with(summoned).with(ScriptUnitTag { unit_id: template.into() })
            .with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::from_i32(300), def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(TAttack::new(Fixed64::ZERO, Fixed64::ONE, Fixed64::from_i32(100), Fixed64::from_i32(900)))
            .build()
    }

    fn scale(p: i32, m: i32, t: i32) -> Scale { Scale([p, m, t].map(|x| Fixed64::from_raw(x as i64 * 1024 / 100))) }

    #[test]
    fn an_illusion_copies_the_unit_its_level_and_items_and_optionally_its_statuses() {
        let mut world = world();
        let s = Fixed64::from_i32;
        let of = world.create_entity()
            .with(CProperty { hp: s(500), mhp: s(800), msd: s(320), def_physic: s(5), def_magic: s(25) })
            .with(TAttack::new(s(60), Fixed64::from_raw(1700), s(150), s(900)))
            .with(CollisionRadius(s(30))).with(VisionSource { team: 0, radius: s(1800), detection_level: 0 })
            .with(ScriptUnitTag { unit_id: "inahime".into() })
            .build();
        {
            let mut buffs = world.write_resource::<BuffStore>();
            buffs.add(of, "battle_trance", s(4), json!({ "attackspeed_bonus_constant": 40 }));
            buffs.add(of, super::super::attributes::KEY, crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION, json!({ "physical_armor_bonus": 2048 }));
            buffs.add(of, "__status_scope_v1", s(9), json!({}));
        }
        let illusion = summon(&mut world, TEMPLATE, 7);
        let (of_id, of_gen) = (of.id(), of.gen().id() as u32);
        let copy = Copy { statuses: Some(StatusCopy::default()), outgoing: scale(-65, -65, -65), incoming: scale(200, 200, 200), ..Copy::default() };
        assert_eq!(apply(&mut world, illusion, &payload(of_id, of_gen, &copy)), Some(of));
        let props = world.read_storage::<CProperty>().get(illusion).unwrap().clone();
        assert_eq!((props.hp, props.mhp, props.msd, props.def_physic, props.def_magic), (s(500), s(800), s(320), s(5), s(25)));
        let attack = world.read_storage::<TAttack>().get(illusion).unwrap().clone();
        assert_eq!((attack.atk_physic.bv, attack.range.bv, attack.asd_count), (s(60), s(150), Fixed64::ZERO));
        assert_eq!(world.read_storage::<CollisionRadius>().get(illusion), Some(&CollisionRadius(s(30))));
        assert_eq!(world.read_storage::<VisionSource>().get(illusion).unwrap().radius, s(1800));
        {
            let buffs = world.read_resource::<BuffStore>();
            assert_eq!(buffs.get(illusion, "battle_trance").unwrap().remaining, s(4), "a status keeps its remaining time");
            assert!(buffs.has(illusion, super::super::attributes::KEY), "level and items come as stat rows");
            assert!(!buffs.has(illusion, "__status_scope_v1"), "other host rows stay with the unit");
            assert_eq!(buffs.get(illusion, STATE).unwrap().remaining, s(20), "is_illusion for its lifetime");
            assert_eq!(look(&buffs, illusion), Some("inahime"), "it looks like the unit it copies");
        }
        // Without statuses only the baked rows come; `hp` sets its life within the max.
        let bare = summon(&mut world, TEMPLATE, 7);
        assert!(apply(&mut world, bare, &payload(of_id, of_gen, &Copy { hp: Some(s(5000)), ..Copy::default() })).is_some());
        assert_eq!(world.read_storage::<CProperty>().get(bare).unwrap().hp, s(800));
        assert!(!world.read_resource::<BuffStore>().has(bare, "battle_trance"));
        assert!(world.read_resource::<BuffStore>().has(bare, super::super::attributes::KEY));
        // Closed: another template, an older summon, a dead or stale `of`, a malformed row.
        let other = summon(&mut world, "saika_gunner", 7);
        let old = summon(&mut world, TEMPLATE, 3);
        let row = payload(of_id, of_gen, &Copy::default());
        assert!(apply(&mut world, other, &row).is_none() && apply(&mut world, old, &row).is_none());
        assert!(apply(&mut world, illusion, &payload(of_id, of_gen + 1, &Copy::default())).is_none());
        assert!(apply(&mut world, illusion, &json!({ "of": [of_id, of_gen] })).is_none());
        let fresh = summon(&mut world, TEMPLATE, 7);
        world.write_storage::<CProperty>().get_mut(of).unwrap().hp = Fixed64::ZERO;
        assert!(apply(&mut world, fresh, &row).is_none());
    }

    /// v3 `copy` (原作 CreateIllusions / CreateMirror): abilities less `except_abilities` with their
    /// learned passives, mana, and statuses chosen by tag and `status_except`, at their full
    /// duration with the illusion as their applier.
    #[test]
    fn an_illusion_copies_abilities_mana_and_the_chosen_statuses_as_the_original_does() {
        use super::super::dispel::{control_source_payload, Registry, CONTROL_SOURCE_PREFIX};
        use super::super::comp::Hero;
        let mut world = world();
        let s = Fixed64::from_i32;
        let mut hero = Hero::new("ninja".into(), "ninja".into(), String::new());
        for (id, rank) in [("b11_q", 2), ("b11_interception", 1), ("b11_passive", 3), ("b11_r", 0)] { hero.ability_levels.insert(id.into(), rank); }
        hero.mana_pool = Some(crate::runtime::ability_runtime::ManaPool::new(s(120), s(500)).unwrap());
        let of = world.create_entity().with(CProperty { hp: s(500), mhp: s(800), msd: s(320), def_physic: s(5), def_magic: s(25) })
            .with(hero).with(ScriptUnitTag { unit_id: "ninja".into() }).build();
        let src = (of.id(), of.gen().id() as u32);
        let registry = Registry::from_catalog(&json!({"abilities": [{"v3": {"statuses": [
            {"id": "b11_ghost", "runtime_id": "b11_ghost:{caster}", "dispel": "basic", "visibility": "buff", "tags": ["from_ability"]},
            {"id": "b11_bind", "dispel": "strong", "visibility": "debuff", "tags": ["from_ability"]},
            {"id": "b11_exorcism", "dispel": "basic", "visibility": "buff", "tags": ["from_ability"]},
            {"id": "enemy_slow", "dispel": "basic", "visibility": "debuff"}]}}]}));
        let events = |source: (u32, u32)| json!({ "ability": "b11_q", "level": 2, "status": "b11_ghost", "source": [source.0, source.1], "mask": 1 });
        {
            let mut buffs = world.write_resource::<BuffStore>();
            buffs.add(of, "b11_passive", crate::runtime::native::ability_runtime::buff_store::PERMANENT_BUFF_DURATION,
                json!({ "__passive": true, "level": 3, (super::super::status_event::KEY): events(src) }));
            buffs.add(of, "b11_interception", s(9), json!({ "__passive": true }));
            buffs.add(of, &format!("b11_ghost:{}:{}", src.0, src.1), s(2),
                json!({ DURATION_KEY: s(12).raw(), "source_entity_id": src.0, "source_entity_gen": src.1, (super::super::status_event::KEY): events(src) }));
            buffs.add(of, "b11_bind", s(3), json!({ DURATION_KEY: s(5).raw() }));
            buffs.add(of, "root", s(3), json!({}));
            buffs.add(of, &format!("{CONTROL_SOURCE_PREFIX}b11_bind"), s(3), control_source_payload(&["root"]));
            buffs.add(of, "b11_exorcism", s(4), json!({}));
            buffs.add(of, "enemy_slow", s(4), json!({}));
        }
        let illusion = summon(&mut world, TEMPLATE, 7);
        let ill = (illusion.id(), illusion.gen().id() as u32);
        let copy = Copy { mana: Some(s(500)), abilities: Some(vec!["b11_interception".into()]),
            statuses: Some(StatusCopy { tags: vec!["from_ability".into()], except: vec!["b11_exorcism".into()],
                original_duration: true, illusion_caster: true }), ..Copy::default() };
        assert_eq!(apply_with(&registry, &mut world, illusion, &payload(src.0, src.1, &copy)), Some(of));
        let buffs = world.read_resource::<BuffStore>();
        assert_eq!((ability_level(&buffs, illusion, "b11_q"), ability_level(&buffs, illusion, "b11_passive")), (2, 3));
        assert_eq!((ability_level(&buffs, illusion, "b11_interception"), ability_level(&buffs, illusion, "b11_r")), (0, 0),
            "except_abilities and unlearned ranks stay");
        let passive = buffs.get(illusion, "b11_passive").expect("a learned passive comes with the abilities");
        assert_eq!(passive.payload[super::super::status_event::KEY]["source"], json!([ill.0, ill.1]), "its events are the illusion's");
        assert!(!buffs.has(illusion, "b11_interception"), "an excepted ability's passive stays");
        let ghost = buffs.get(illusion, &format!("b11_ghost:{}:{}", ill.0, ill.1)).expect("the per-source key names the illusion");
        assert_eq!(ghost.remaining, s(12), "status_duration = original");
        assert_eq!((ghost.payload["source_entity_id"].as_u64(), ghost.payload[super::super::status_event::KEY]["source"].clone()),
            (Some(u64::from(ill.0)), json!([ill.0, ill.1])));
        assert_eq!(buffs.get(illusion, "b11_bind").unwrap().remaining, s(5));
        assert!(buffs.has(illusion, &format!("{CONTROL_SOURCE_PREFIX}b11_bind")) && buffs.has(illusion, "root"), "its control comes with it");
        assert!(!buffs.has(illusion, "b11_exorcism") && !buffs.has(illusion, "enemy_slow"), "status_except and an untagged status stay");
        drop(buffs);
        let mana = world.read_storage::<super::super::unit_mana::UnitMana>().get(illusion).cloned().expect("copy.mana");
        assert_eq!((mana.pool.current(), mana.pool.maximum()), (s(500), s(500)));
        // Without abilities or statuses only the baked rows come.
        let bare = summon(&mut world, TEMPLATE, 7);
        assert!(apply_with(&registry, &mut world, bare, &payload(src.0, src.1, &Copy::default())).is_some());
        let buffs = world.read_resource::<BuffStore>();
        assert!(!buffs.has(bare, "b11_passive") && !buffs.has(bare, "b11_bind") && ability_level(&buffs, bare, "b11_q") == 0);
        assert!(world.read_storage::<super::super::unit_mana::UnitMana>().get(bare).is_none());
    }

    #[test]
    fn an_illusion_scales_the_damage_it_deals_and_takes_per_type() {
        let mut world = world();
        let s = Fixed64::from_i32;
        let (hero, illusion) = (world.create_entity().build(), world.create_entity().build());
        world.write_resource::<BuffStore>().add(illusion, STATE, s(20),
            json!({ "outgoing": scale(-65, -100, 0).row(), "incoming": scale(200, 0, -50).row() }));
        let packet = |source, target| super::super::rules::damage::Packet { source, credit: source, target,
            physical: s(100), magical: s(100), pure: s(100), profile: 0 };
        let mut dealt = packet(illusion, hero);
        scale_damage(&world, &mut dealt);
        assert_eq!((dealt.physical, dealt.magical, dealt.pure), (s(100) * Fixed64::from_raw(1024 - 665), Fixed64::ZERO, s(100)), "-65% keeps 35% (raw -665)");
        let mut taken = packet(hero, illusion);
        scale_damage(&world, &mut taken);
        assert_eq!((taken.physical, taken.magical, taken.pure), (s(300), s(100), s(50)));
        let mut plain = packet(hero, hero);
        scale_damage(&world, &mut plain);
        assert_eq!((plain.physical, plain.magical, plain.pure), (s(100), s(100), s(100)));
    }
}
