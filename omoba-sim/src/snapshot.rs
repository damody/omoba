//! Bincode snapshot serialize/deserialize wrappers.
//!
//! Used by Phase 5 observer rejoin: server periodically (every 30s) serializes the full
//! sim World into bytes; observer clients connecting mid-game receive the most recent
//! snapshot then play forward via subsequent TickBatches.
//!
//! This is a thin pass-through over bincode 1.x. Wrapping it here means consumers
//! depend only on `omoba_sim::snapshot::{serialize, deserialize}` rather than on bincode
//! directly — so a future migration to bincode 2.x or a different encoder is one file.

use serde::{de::DeserializeOwned, Serialize};

/// Serializes a value to bytes using bincode 1.x default config (little-endian, varint sizes).
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(value)
}

/// Deserializes a value from bytes using bincode 1.x default config.
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct DummyWorld {
        tick: u32,
        entities: Vec<(u32, i32, i32)>,
    }

    #[test]
    fn round_trip() {
        let w = DummyWorld {
            tick: 42,
            entities: vec![(1, 10, 20), (2, 30, 40)],
        };
        let bytes = serialize(&w).expect("serialize");
        let w2: DummyWorld = deserialize(&bytes).expect("deserialize");
        assert_eq!(w, w2);
    }

    #[test]
    fn empty_world_round_trip() {
        let w = DummyWorld {
            tick: 0,
            entities: vec![],
        };
        let bytes = serialize(&w).expect("serialize");
        let w2: DummyWorld = deserialize(&bytes).expect("deserialize");
        assert_eq!(w, w2);
    }
}
