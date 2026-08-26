//! Deterministic output collection shared by the authoritative server and replicas.
//!
//! Parallel Specs systems may finish in any order.  They therefore write into
//! independent shards and attach a canonical key at the production site.  Only
//! the merged, validated stream is allowed to cross an event/projection bridge.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Reserved maximum values are rejected so accidental sentinel/uninitialised
/// keys fail closed instead of being silently placed at the end of a tick.
pub const MAX_CANONICAL_SOURCE_ORDER: u64 = u64::MAX - 1;
pub const MAX_LOCAL_ORDINAL: u32 = u32::MAX - 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum FactPhase {
    PreStep = 0,
    Step = 1,
    PostStep = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum FactKind {
    Movement = 0,
    Spawn = 1,
    Death = 2,
    Ownership = 3,
    DirectCombat = 4,
    Projectile = 5,
    AreaEffect = 6,
    Buff = 7,
    Ability = 8,
    Tower = 9,
    Item = 10,
    Hud = 11,
    Terminal = 12,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FactOrderingKey {
    pub tick: u64,
    pub phase: FactPhase,
    pub canonical_source_order: u64,
    pub local_ordinal: u32,
    pub fact_kind: FactKind,
}

impl FactOrderingKey {
    pub fn validate(self) -> Result<Self, StableOutputError> {
        if self.canonical_source_order > MAX_CANONICAL_SOURCE_ORDER {
            return Err(StableOutputError::MalformedCanonicalSourceOrder);
        }
        if self.local_ordinal > MAX_LOCAL_ORDINAL {
            return Err(StableOutputError::MalformedLocalOrdinal);
        }
        Ok(self)
    }
}

/// Integer/fixed-point-only public metadata. It deliberately cannot carry an
/// ECS entity handle, pointer, arbitrary JSON value, or server-only component.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ObservableFact {
    Movement { source: u64, x_mm: i64, y_mm: i64 },
    Spawn { source: u64, template_id: u64, team: u32 },
    Death { source: u64, killer: Option<u64> },
    Ownership { source: u64, team: u32 },
    DirectCombat { source: u64, target: u64, amount_milli: i64 },
    Projectile { source: u64, target: Option<u64>, effect_id: u64 },
    AreaEffect { source: u64, x_mm: i64, y_mm: i64, radius_mm: u64 },
    Buff { source: u64, target: u64, effect_id: u64, active: bool },
    Ability { source: u64, ability_id: u64, target: Option<u64> },
    Tower { source: u64, action_id: u64 },
    Item { source: u64, item_id: u64, target: Option<u64> },
    Hud { team: u32, metric_id: u64, value: i64 },
    Terminal { result_code: u32, winning_team: Option<u32> },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OrderedFact {
    pub key: FactOrderingKey,
    pub fact: ObservableFact,
}

/// Existing outcomes/events use this wrapper while they are migrated to facts.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderedOutput<T> {
    pub key: FactOrderingKey,
    pub value: T,
}

pub trait StableKeyed {
    fn stable_key(&self) -> FactOrderingKey;
}

impl StableKeyed for OrderedFact {
    fn stable_key(&self) -> FactOrderingKey { self.key }
}

impl<T> StableKeyed for OrderedOutput<T> {
    fn stable_key(&self) -> FactOrderingKey { self.key }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StableOutputError {
    NoShards,
    InvalidShard,
    PoisonedShard,
    MalformedCanonicalSourceOrder,
    MalformedLocalOrdinal,
}

/// Cloneable Specs resource handle. Systems receive a stable shard index from
/// dispatcher construction; lock acquisition order never affects merge order.
#[derive(Clone)]
pub struct ShardedStableBuffer<T> {
    shards: Arc<Vec<Mutex<Vec<T>>>>,
}

impl<T> ShardedStableBuffer<T> {
    pub fn new(shard_count: usize) -> Result<Self, StableOutputError> {
        if shard_count == 0 { return Err(StableOutputError::NoShards); }
        Ok(Self { shards: Arc::new((0..shard_count).map(|_| Mutex::new(Vec::new())).collect()) })
    }

    pub fn push(&self, shard: usize, value: T) -> Result<(), StableOutputError> {
        let slot = self.shards.get(shard).ok_or(StableOutputError::InvalidShard)?;
        slot.lock().map_err(|_| StableOutputError::PoisonedShard)?.push(value);
        Ok(())
    }
}

impl<T: StableKeyed> ShardedStableBuffer<T> {
    pub fn drain_sorted(&self) -> Result<Vec<T>, StableOutputError> {
        let mut merged = Vec::new();
        for shard in self.shards.iter() {
            merged.append(&mut *shard.lock().map_err(|_| StableOutputError::PoisonedShard)?);
        }
        for value in &merged { value.stable_key().validate()?; }
        merged.sort_by_key(StableKeyed::stable_key);
        Ok(merged)
    }
}

impl ShardedStableBuffer<OrderedFact> {
    pub fn drain_sorted_deduped(&self) -> Result<Vec<OrderedFact>, StableOutputError> {
        let mut facts = self.drain_sorted()?;
        let mut seen = BTreeSet::new();
        facts.retain(|fact| seen.insert(fact.clone()));
        Ok(facts)
    }
}

pub trait TeamProjectionBridge {
    type Error;
    fn project_ordered_facts(&mut self, facts: &[OrderedFact]) -> Result<(), Self::Error>;
}
