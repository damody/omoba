//! Own-team host state the team replica cannot derive (split16).
//!
//! A hero's own team replays its accepted inputs, but stun, silence and root
//! from a hidden or enemy source never reach that replica, and a channel can
//! end for authority-only reasons (control, death, a hidden anchor). Cast
//! admission and hero ticks read these, so without them the replica admits a
//! cast the authority rejects, or keeps a channel's attack pause the authority
//! dropped.
//!
//! After every outcome batch the authority publishes, to the hero's team only,
//! the exact remaining time of each control and the remaining/total of each
//! channel status (0 = absent), while any is active and once more when the
//! last one ends. The replica applies that batch right after its own matching
//! `process_outcomes`, so both worlds read equal state in every later phase.
//! A rebase seeds the same statuses from the owner-only buff visual baseline.
use std::collections::BTreeMap;

use omb_script_abi::buff_ids::BuffId;
use omoba_sim::Fixed64;
use specs::{Entity, World, WorldExt};

use super::comp::Tick;
use super::{BuffStore, MobaMatch};

pub(crate) const CONTROLS: [BuffId; 3] = [BuffId::Stun, BuffId::Silence, BuffId::Root];
/// Channel statuses carry `{status:{"total_raw"}}`; the replica reads their presence.
pub(crate) const CHANNELS: [&str; 2] = [super::ability_channel::STATUS, super::recovery_channel::STATUS];
const MAX_REMAINING_RAW: i64 = 60 * 1024;
/// Two batches per active tick (pre- and post-script outcomes); a bound, not a phase name.
pub(crate) const MAX_BATCH: u8 = 7;

/// Remaining per control, then (remaining, total) per channel status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct OwnerControlRow {
    pub controls: [i64; 3],
    pub channels: [(i64, i64); 2],
}

impl OwnerControlRow {
    pub(crate) const ENCODED_LEN: usize = 1 + 8 * 3 + 16 * 2;

    fn is_empty(&self) -> bool { *self == Self::default() }

    fn of(buffs: &BuffStore, entity: Entity) -> Self {
        let controls = CONTROLS.map(|id| buffs.get(entity, id.as_str()).map_or(0, |entry| entry.remaining.raw().max(0)));
        let channels = CHANNELS.map(|status| buffs.get(entity, status)
            .and_then(|entry| Some((entry.remaining.raw(), entry.payload.get(status)?.get("total_raw")?.as_i64()?)))
            .filter(|(remaining, total)| *remaining > 0 && remaining <= total && *total <= MAX_REMAINING_RAW)
            .unwrap_or((0, 0)));
        Self { controls, channels }
    }

    pub(crate) fn encode(&self, batch: u8) -> Vec<u8> {
        let mut payload = Vec::with_capacity(Self::ENCODED_LEN);
        payload.push(batch);
        for raw in self.controls { payload.extend(raw.to_le_bytes()); }
        for (remaining, total) in self.channels { payload.extend(remaining.to_le_bytes()); payload.extend(total.to_le_bytes()); }
        payload
    }

    pub(crate) fn decode(payload: &[u8]) -> Option<(u8, Self)> {
        if payload.len() != Self::ENCODED_LEN || payload[0] > MAX_BATCH { return None; }
        let raw = |index: usize| i64::from_le_bytes(payload[1 + 8 * index..9 + 8 * index].try_into().unwrap());
        let row = Self {
            controls: [raw(0), raw(1), raw(2)],
            channels: [(raw(3), raw(4)), (raw(5), raw(6))],
        };
        let bounded = |value: i64| (0..=MAX_REMAINING_RAW).contains(&value);
        let channel_ok = |(remaining, total): (i64, i64)| (remaining == 0 && total == 0)
            || (remaining > 0 && remaining <= total && total <= MAX_REMAINING_RAW);
        (row.controls.iter().all(|value| bounded(*value)) && row.channels.iter().all(|value| channel_ok(*value)))
            .then_some((payload[0], row))
    }
}

/// Outcome batch clock, shared by the authority and the replica.
#[derive(Default)]
struct OutcomeBatchClock { tick: u64, next: u8 }

/// Last published row per hero life (authority only).
#[derive(Default)]
struct PublishedControls(BTreeMap<Entity, OwnerControlRow>);

/// Rows the replica received this tick: (hero, batch, row).
#[derive(Default)]
pub struct OwnerControlInjections(pub Vec<(Entity, u8, OwnerControlRow)>);

/// Called once at the end of every `process_outcomes`.
pub(crate) fn settle_batch(world: &mut World) {
    let tick = world.try_fetch::<Tick>().map_or(0, |tick| tick.0);
    if world.try_fetch::<OutcomeBatchClock>().is_none() { world.insert(OutcomeBatchClock::default()); }
    let batch = {
        let mut clock = world.write_resource::<OutcomeBatchClock>();
        if clock.tick != tick { clock.tick = tick; clock.next = 0; }
        let batch = clock.next;
        clock.next = clock.next.saturating_add(1);
        batch
    };
    if batch > MAX_BATCH { return; }
    if world.try_fetch::<MobaMatch>().is_some() { publish(world, tick, batch); } else { apply(world, batch); }
}

fn publish(world: &mut World, tick: u64, batch: u8) {
    use crate::runtime::{FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact};
    let rows_now: Vec<_> = {
        let Some(buffs) = world.try_fetch::<BuffStore>() else { return };
        let state = world.read_resource::<MobaMatch>();
        let entities = world.entities();
        state.heroes.iter().filter_map(|slot| {
            let entity = slot.entity.filter(|entity| entities.is_alive(*entity))?;
            Some((entity, state.config.teams[slot.side], OwnerControlRow::of(&buffs, entity)))
        }).collect()
    };
    if world.try_fetch::<PublishedControls>().is_none() { world.insert(PublishedControls::default()); }
    let mut published = world.write_resource::<PublishedControls>();
    let Some(facts) = world.try_fetch::<crate::runtime::ObservableFactBuffer>() else { return };
    published.0.retain(|entity, _| rows_now.iter().any(|(live, _, _)| live == entity));
    for (entity, team, row) in rows_now {
        let was_active = published.0.get(&entity).is_some_and(|last| !last.is_empty());
        if row.is_empty() && !was_active { continue; }
        published.0.insert(entity, row);
        let source = crate::runtime::canonical_entity_id(entity);
        facts.emit(OrderedFact {
            key: FactOrderingKey { tick, phase: FactPhase::PostStep, canonical_source_order: source,
                local_ordinal: u32::from(batch), fact_kind: FactKind::OwnerControl },
            audience: FactAudience::Team(team),
            fact: ObservableFact::OwnerControl { source, batch, row },
        }).expect("valid owner-team control state");
    }
}

/// Exact remaining with the replica's own payload when it has one.
fn set_remaining(buffs: &mut BuffStore, entity: Entity, key: &str, raw: i64, payload: impl FnOnce() -> serde_json::Value) {
    let current = buffs.get(entity, key).map(|entry| (entry.remaining.raw(), entry.payload.clone()));
    match current {
        None if raw == 0 => {}
        Some((remaining, _)) if remaining == raw => {}
        Some(_) if raw == 0 => buffs.remove(entity, key),
        current => {
            let payload = current.map_or_else(payload, |(_, payload)| payload);
            buffs.remove(entity, key);
            buffs.add(entity, key, Fixed64::from_raw(raw), payload);
        }
    }
}

/// Makes `entity`'s controls and channel statuses equal `row`.
pub(crate) fn install(world: &World, entity: Entity, row: &OwnerControlRow) {
    {
        let Some(mut buffs) = world.try_fetch_mut::<BuffStore>() else { return };
        for (id, raw) in CONTROLS.iter().zip(row.controls) {
            set_remaining(&mut buffs, entity, id.as_str(), raw, || serde_json::json!({}));
        }
        for (status, (remaining, total)) in CHANNELS.iter().zip(row.channels) {
            // The total is part of the public bar contract, so it is replaced too.
            if buffs.get(entity, status).is_some_and(|entry| entry.payload.get(*status)
                .and_then(|value| value.get("total_raw")).and_then(|value| value.as_i64()) != Some(total)) {
                buffs.remove(entity, status);
            }
            set_remaining(&mut buffs, entity, status, remaining, || serde_json::json!({(*status): {"total_raw": total}}));
        }
    }
    // The authority decides when a channel is over; a local predictor row ends with it.
    if row.channels[0].0 == 0 { super::ability_channel::forget(world, entity); }
}

fn apply(world: &mut World, batch: u8) {
    let Some(injections) = world.try_fetch::<OwnerControlInjections>().map(|rows| rows.0.clone()) else { return };
    for (entity, row_batch, row) in injections {
        if row_batch == batch { install(world, entity, &row); }
    }
}

/// Bootstrap/rebase seed from the owner-only buff visual baseline, which
/// carries the same exact remaining times and channel totals.
pub(crate) fn seed_from_visual(world: &World, entity: Entity, visual: &crate::runtime::buff_visual_state::BuffVisualSnapshot) {
    let raw = |name: &str| omoba_template_ids::buff_by_name(name)
        .and_then(|id| visual.buffs.0.iter().find(|row| row.0 == id.raw()).map(|row| (id.raw(), row.1)));
    let controls = CONTROLS.map(|id| raw(id.as_str()).map_or(0, |(_, remaining)| remaining.max(0)));
    let channels = CHANNELS.map(|status| raw(status)
        .and_then(|(id, remaining)| visual.channel_totals.iter().find(|row| row.0 == id).map(|row| (remaining, row.1)))
        .unwrap_or((0, 0)));
    install(world, entity, &OwnerControlRow { controls, channels });
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn row() -> OwnerControlRow { OwnerControlRow { controls: [512, 0, 1024], channels: [(300, 3072), (0, 0)] } }

    #[test]
    fn payload_round_trips_and_rejects_malformed_rows() {
        let bytes = row().encode(1);
        assert_eq!(bytes.len(), OwnerControlRow::ENCODED_LEN);
        assert_eq!(OwnerControlRow::decode(&bytes), Some((1, row())));
        assert_eq!(OwnerControlRow::decode(&bytes[1..]), None);
        assert_eq!(OwnerControlRow::decode(&row().encode(MAX_BATCH + 1)), None);
        for bad in [OwnerControlRow { controls: [-1, 0, 0], ..row() }, OwnerControlRow { controls: [MAX_REMAINING_RAW + 1, 0, 0], ..row() },
            OwnerControlRow { channels: [(301, 300), (0, 0)], ..row() }, OwnerControlRow { channels: [(0, 300), (0, 0)], ..row() },
            OwnerControlRow { channels: [(1, MAX_REMAINING_RAW + 1), (0, 0)], ..row() }] {
            assert_eq!(OwnerControlRow::decode(&bad.encode(0)), None, "{bad:?}");
        }
    }

    #[test]
    fn replica_applies_only_its_batch_with_exact_remaining_and_channel_totals() {
        let mut world = World::new();
        let hero = world.create_entity().build();
        let mut buffs = BuffStore::default();
        buffs.add(hero, BuffId::Root.as_str(), Fixed64::from_raw(300), serde_json::json!({"k": 1}));
        world.insert(buffs);
        world.insert(Tick(5));
        let later = OwnerControlRow { controls: [700, 0, 200], channels: [(0, 0), (512, 2048)] };
        world.insert(OwnerControlInjections(vec![(hero, 1, later), (hero, 0, row())]));
        settle_batch(&mut world);
        {
            let buffs = world.read_resource::<BuffStore>();
            assert_eq!(buffs.get(hero, "stun").unwrap().remaining.raw(), 512);
            assert!(!buffs.has(hero, "silence"));
            assert_eq!(buffs.get(hero, "root").unwrap().remaining.raw(), 1024);
            assert_eq!(buffs.get(hero, "root").unwrap().payload, serde_json::json!({"k": 1}), "a replica payload is kept");
            let channel = buffs.get(hero, CHANNELS[0]).unwrap();
            assert_eq!((channel.remaining.raw(), channel.payload[CHANNELS[0]]["total_raw"].as_i64()), (300, Some(3072)));
        }
        settle_batch(&mut world);
        let buffs = world.read_resource::<BuffStore>();
        assert_eq!(buffs.get(hero, "stun").unwrap().remaining.raw(), 700);
        assert_eq!(buffs.get(hero, "root").unwrap().remaining.raw(), 200);
        assert!(!buffs.has(hero, CHANNELS[0]), "the authority ended the channel");
        assert_eq!(buffs.get(hero, CHANNELS[1]).unwrap().payload, serde_json::json!({(CHANNELS[1]): {"total_raw": 2048}}));
        assert_eq!(OwnerControlRow::of(&buffs, hero), later);
    }
}
