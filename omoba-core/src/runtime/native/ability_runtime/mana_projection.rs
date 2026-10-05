//! Absolute, versioned mana projection. No input, hidden target or caster identity.
use super::{ManaPool, ManaPoolError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CommittedManaState(pub Option<ManaPool>);

impl CommittedManaState {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![1, u8::from(self.0.is_some())];
        if let Some(pool) = &self.0 {
            let (current, maximum, remainder) = pool.raw_state();
            bytes.extend(current.to_le_bytes());
            bytes.extend(maximum.to_le_bytes());
            bytes.extend(remainder.to_le_bytes());
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ManaPoolError> {
        match bytes {
            [1, 0] => Ok(Self(None)),
            [1, 1, ..] if bytes.len() == 20 => {
                let current = i64::from_le_bytes(bytes[2..10].try_into().unwrap());
                let maximum = i64::from_le_bytes(bytes[10..18].try_into().unwrap());
                let remainder = u16::from_le_bytes(bytes[18..20].try_into().unwrap());
                Ok(Self(Some(ManaPool::from_raw_state(
                    current, maximum, remainder,
                )?)))
            }
            _ => Err(ManaPoolError::InvalidState),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;

    #[test]
    fn mana_projection_codec_preserves_zero_disabled_and_fractional_credit() {
        for pool in [
            None,
            Some(ManaPool::new(Fixed64::ZERO, Fixed64::from_i32(100)).unwrap()),
            Some(ManaPool::from_raw_state(20, 100, 17).unwrap()),
        ] {
            let state = CommittedManaState(pool);
            assert_eq!(CommittedManaState::decode(&state.encode()), Ok(state));
        }
    }

    #[test]
    fn mana_projection_codec_rejects_invalid_framing_and_pool_without_clamping() {
        for bytes in [
            vec![],
            vec![0, 0],
            vec![1, 2],
            vec![1, 0, 0],
            vec![1, 1],
            vec![1; 21],
        ] {
            assert!(CommittedManaState::decode(&bytes).is_err());
        }
        for (current, maximum, remainder) in [
            (-1i64, 100i64, 0u16),
            (101, 100, 0),
            (0, -1, 0),
            (0, 100, 1024),
            (100, 100, 1),
        ] {
            let bytes = [
                vec![1, 1],
                current.to_le_bytes().to_vec(),
                maximum.to_le_bytes().to_vec(),
                remainder.to_le_bytes().to_vec(),
            ]
            .concat();
            assert!(CommittedManaState::decode(&bytes).is_err());
        }
    }
}
