//! v3 `create_illusion { of, at, duration, control, copy, outgoing, incoming }` (原作
//! CreateIllusions): a summon of the `illusion` template (summons.lua; placement, owner orders,
//! AI, lifetime and disclosure are the summon's) that a one-shot envelope (never stored) turns into
//! a copy of `of` right after its spawn, in the same batch:
//! - life, speed, armor, magic resistance, attack, collision, vision and facing are `of`'s at that
//!   moment, with the stat rows of its level and items ([`BAKED`]) kept as they are then; `hp` sets
//!   its life (capped at its max);
//! - `statuses`: every status row `of` holds (learned passives' rows included), with its remaining
//!   time; other host bookkeeping rows (`__…`) are not copied;
//! - the unit holds the [`STATE`] status (`is_illusion`) for its lifetime, whose payload carries the
//!   damage it deals and takes as fractions per type (`outgoing` / `incoming`, 原作
//!   outgoing_damage / incoming_damage): [`scale_damage`] (rules::damage, before mitigation)
//!   multiplies a packet by 1 + x of its source's outgoing and of its target's incoming row.
//! Mana and abilities are not copied: a summon has neither (its passives' stat rows come with
//! `statuses`).
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

/// The envelope payload. `hp`: the illusion's life (None = `of`'s current life).
pub fn payload(of_id: u32, of_generation: u32, hp: Option<Fixed64>, statuses: bool, outgoing: Scale, incoming: Scale) -> Value {
    json!({ "of": [of_id, of_generation], "hp": hp.map(|hp| hp.raw()), "statuses": statuses,
        "outgoing": outgoing.row(), "incoming": incoming.row() })
}

/// Settles one envelope on `illusion`, an `illusion` summon spawned this tick, and returns `of`;
/// None when it is malformed, `of` is not a living unit, or `illusion` is anything else.
/// Its max life is `of`'s with `of`'s folded max-life bonus; the caller refolds it from the
/// illusion's own rows (game_processor `settle_health_bonus`).
pub fn apply(world: &mut World, illusion: Entity, payload: &Value) -> Option<Entity> {
    let row = payload.as_object().filter(|row| row.len() == 5)?;
    let (outgoing, incoming) = (Scale::of(row.get("outgoing"))?, Scale::of(row.get("incoming"))?);
    let statuses = row.get("statuses").and_then(Value::as_bool)?;
    let hp = match row.get("hp")? { Value::Null => None, value => match value.as_i64() {
        Some(raw) if raw > 0 => Some(Fixed64::from_raw(raw)), _ => return None } };
    let of = row.get("of").and_then(Value::as_array).filter(|pair| pair.len() == 2).and_then(|pair| {
        let (id, generation) = (u32::try_from(pair[0].as_u64()?).ok()?, u32::try_from(pair[1].as_u64()?).ok()?);
        let entity = world.entities().entity(id);
        (entity.gen().id() as u32 == generation && world.entities().is_alive(entity)).then_some(entity)
    })?;
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
    let mut buffs = world.write_resource::<BuffStore>();
    if statuses {
        let rows: Vec<(String, Fixed64, Value)> = buffs.iter_for(of)
            .filter(|(id, _)| !id.starts_with("__") && *id != STATE)
            .map(|(id, entry)| (id.to_owned(), entry.remaining, entry.payload.clone())).collect();
        for (id, left, payload) in rows { if left > Fixed64::ZERO { buffs.add(illusion, &id, left, payload); } }
    }
    for id in BAKED {
        if let Some(entry) = buffs.get(of, id).cloned() { buffs.add(illusion, id, entry.remaining, entry.payload); }
    }
    buffs.add(illusion, STATE, remaining, json!({ "outgoing": outgoing.row(), "incoming": incoming.row(), "look": look }));
    Some(of)
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
    for scale in [scale(packet.source, "outgoing"), scale(packet.target, "incoming")].into_iter().flatten() {
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
        world.register::<Faction>();
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
        assert_eq!(apply(&mut world, illusion, &payload(of_id, of_gen, None, true, scale(-65, -65, -65), scale(200, 200, 200))), Some(of));
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
        assert!(apply(&mut world, bare, &payload(of_id, of_gen, Some(s(5000)), false, Scale::default(), Scale::default())).is_some());
        assert_eq!(world.read_storage::<CProperty>().get(bare).unwrap().hp, s(800));
        assert!(!world.read_resource::<BuffStore>().has(bare, "battle_trance"));
        assert!(world.read_resource::<BuffStore>().has(bare, super::super::attributes::KEY));
        // Closed: another template, an older summon, a dead or stale `of`, a malformed row.
        let other = summon(&mut world, "saika_gunner", 7);
        let old = summon(&mut world, TEMPLATE, 3);
        let row = payload(of_id, of_gen, None, false, Scale::default(), Scale::default());
        assert!(apply(&mut world, other, &row).is_none() && apply(&mut world, old, &row).is_none());
        assert!(apply(&mut world, illusion, &payload(of_id, of_gen + 1, None, false, Scale::default(), Scale::default())).is_none());
        assert!(apply(&mut world, illusion, &json!({ "of": [of_id, of_gen] })).is_none());
        let fresh = summon(&mut world, TEMPLATE, 7);
        world.write_storage::<CProperty>().get_mut(of).unwrap().hp = Fixed64::ZERO;
        assert!(apply(&mut world, fresh, &row).is_none());
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
