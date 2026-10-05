//! Integer-only mana accounting. Gameplay admission and projection remain host responsibilities.
use omoba_sim::Fixed64;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManaPoolError {
    InvalidAmount,
    InvalidState,
    InsufficientMana,
}

/// Absolute current mana: increasing capacity does not silently restore mana.
/// Private fields and checked deserialization keep all callers on the same rules.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "ManaPoolWire", into = "ManaPoolWire")]
pub struct ManaPool {
    current: Fixed64,
    maximum: Fixed64,
    regeneration_remainder: u16,
}

#[derive(Serialize, Deserialize)]
struct ManaPoolWire {
    current: Fixed64,
    maximum: Fixed64,
    #[serde(default)]
    regeneration_remainder: u16,
}

impl TryFrom<ManaPoolWire> for ManaPool {
    type Error = &'static str;
    fn try_from(value: ManaPoolWire) -> Result<Self, Self::Error> {
        if value.maximum < Fixed64::ZERO
            || value.current < Fixed64::ZERO
            || value.current > value.maximum
            || value.regeneration_remainder >= 1024
            || (value.current == value.maximum && value.regeneration_remainder != 0)
        {
            return Err("invalid mana pool state");
        }
        Ok(Self {
            current: value.current,
            maximum: value.maximum,
            regeneration_remainder: value.regeneration_remainder,
        })
    }
}

impl From<ManaPool> for ManaPoolWire {
    fn from(value: ManaPool) -> Self {
        Self {
            current: value.current,
            maximum: value.maximum,
            regeneration_remainder: value.regeneration_remainder,
        }
    }
}

impl ManaPool {
    pub fn raw_state(&self) -> (i64, i64, u16) {
        (
            self.current.raw(),
            self.maximum.raw(),
            self.regeneration_remainder,
        )
    }

    pub fn from_raw_state(
        current_raw: i64,
        maximum_raw: i64,
        remainder: u16,
    ) -> Result<Self, ManaPoolError> {
        ManaPoolWire {
            current: Fixed64::from_raw(current_raw),
            maximum: Fixed64::from_raw(maximum_raw),
            regeneration_remainder: remainder,
        }
        .try_into()
        .map_err(|_| ManaPoolError::InvalidState)
    }
    pub fn new(current: Fixed64, maximum: Fixed64) -> Result<Self, ManaPoolError> {
        ManaPoolWire {
            current,
            maximum,
            regeneration_remainder: 0,
        }
        .try_into()
        .map_err(|_| ManaPoolError::InvalidState)
    }

    pub fn full(maximum: Fixed64) -> Result<Self, ManaPoolError> {
        Self::new(maximum, maximum)
    }

    pub fn current(&self) -> Fixed64 {
        self.current
    }
    pub fn maximum(&self) -> Fixed64 {
        self.maximum
    }

    /// Reject negative costs and insufficient balance without mutating any state.
    pub fn spend(&mut self, amount: Fixed64) -> Result<(), ManaPoolError> {
        if amount < Fixed64::ZERO {
            return Err(ManaPoolError::InvalidAmount);
        }
        if amount > self.current {
            return Err(ManaPoolError::InsufficientMana);
        }
        self.current -= amount;
        Ok(())
    }

    /// Returns the amount actually restored; addition cannot overflow.
    pub fn restore(&mut self, amount: Fixed64) -> Result<Fixed64, ManaPoolError> {
        if amount < Fixed64::ZERO {
            return Err(ManaPoolError::InvalidAmount);
        }
        let restored = amount.min(self.maximum - self.current);
        self.current += restored;
        if self.current == self.maximum {
            self.regeneration_remainder = 0;
        }
        Ok(restored)
    }

    /// Preserve absolute mana on growth, clamp on shrink. No automatic level-up refill.
    pub fn set_maximum(&mut self, maximum: Fixed64) -> Result<(), ManaPoolError> {
        if maximum < Fixed64::ZERO {
            return Err(ManaPoolError::InvalidAmount);
        }
        self.maximum = maximum;
        self.current = self.current.min(maximum);
        if self.current == maximum {
            self.regeneration_remainder = 0;
        }
        Ok(())
    }

    /// Carry sub-Q10 product fractions rather than losing them every 60Hz tick.
    /// dt is active simulation time, not wall time or an inferred tick frequency.
    pub fn regenerate(
        &mut self,
        per_second: Fixed64,
        dt: Fixed64,
    ) -> Result<Fixed64, ManaPoolError> {
        if per_second < Fixed64::ZERO || dt < Fixed64::ZERO {
            return Err(ManaPoolError::InvalidAmount);
        }
        if per_second == Fixed64::ZERO || dt == Fixed64::ZERO || self.current == self.maximum {
            return Ok(Fixed64::ZERO);
        }
        let product = i128::from(per_second.raw()) * i128::from(dt.raw())
            + i128::from(self.regeneration_remainder);
        let amount = (product / 1024).min(i128::from((self.maximum - self.current).raw()));
        self.regeneration_remainder = (product % 1024) as u16;
        self.restore(Fixed64::from_raw(amount as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(value: i32) -> Fixed64 {
        Fixed64::from_i32(value)
    }

    #[test]
    fn mana_pool_spending_is_atomic_and_sequential() {
        let mut pool = ManaPool::full(f(100)).unwrap();
        assert_eq!(pool.spend(f(-1)), Err(ManaPoolError::InvalidAmount));
        assert_eq!(pool.current(), f(100));
        pool.spend(f(60)).unwrap();
        let before = pool.clone();
        assert_eq!(pool.spend(f(60)), Err(ManaPoolError::InsufficientMana));
        assert_eq!(pool, before);
        pool.spend(f(40)).unwrap();
        pool.spend(Fixed64::ZERO).unwrap();
        assert_eq!(pool.current(), Fixed64::ZERO);
        assert_eq!(
            pool.spend(Fixed64::from_raw(1)),
            Err(ManaPoolError::InsufficientMana)
        );
    }

    #[test]
    fn mana_pool_restore_and_capacity_do_not_overflow_or_refill_on_growth() {
        let mut pool = ManaPool::new(f(40), f(100)).unwrap();
        pool.set_maximum(f(200)).unwrap();
        assert_eq!(pool.current(), f(40));
        assert_eq!(pool.restore(Fixed64::from_raw(i64::MAX)), Ok(f(160)));
        assert_eq!(pool.current(), f(200));
        pool.set_maximum(f(20)).unwrap();
        assert_eq!(pool.current(), f(20));
        let before = pool.clone();
        assert_eq!(pool.restore(f(-1)), Err(ManaPoolError::InvalidAmount));
        assert_eq!(pool.set_maximum(f(-1)), Err(ManaPoolError::InvalidAmount));
        assert_eq!(pool, before);
        pool.set_maximum(Fixed64::ZERO).unwrap();
        assert_eq!(pool.current(), Fixed64::ZERO);
    }

    #[test]
    fn mana_pool_regeneration_preserves_low_rates_across_tick_partitions() {
        let mut partitioned = ManaPool::new(Fixed64::ZERO, f(100)).unwrap();
        let mut aggregate = partitioned.clone();
        for _ in 0..1024 {
            partitioned
                .regenerate(Fixed64::from_raw(1), Fixed64::from_raw(17))
                .unwrap();
        }
        aggregate
            .regenerate(Fixed64::from_raw(1), Fixed64::from_raw(17 * 1024))
            .unwrap();
        assert_eq!(partitioned, aggregate);
        assert_eq!(partitioned.current(), Fixed64::from_raw(17));
        let before = partitioned.clone();
        assert_eq!(
            partitioned.regenerate(f(-1), f(1)),
            Err(ManaPoolError::InvalidAmount)
        );
        assert_eq!(
            partitioned.regenerate(f(1), f(-1)),
            Err(ManaPoolError::InvalidAmount)
        );
        partitioned.regenerate(f(1), Fixed64::ZERO).unwrap();
        assert_eq!(partitioned, before);
    }

    #[test]
    fn mana_pool_full_discards_fractional_credit_and_handles_extreme_products() {
        let mut pool = ManaPool::new(Fixed64::ZERO, Fixed64::from_raw(1)).unwrap();
        pool.regenerate(Fixed64::from_raw(1), Fixed64::from_raw(1023))
            .unwrap();
        pool.regenerate(Fixed64::from_raw(1), Fixed64::from_raw(2))
            .unwrap();
        pool.spend(Fixed64::from_raw(1)).unwrap();
        assert_eq!(
            pool.regenerate(Fixed64::from_raw(1), Fixed64::from_raw(1023)),
            Ok(Fixed64::ZERO)
        );
        let huge = Fixed64::from_raw(i64::MAX);
        assert_eq!(pool.regenerate(huge, huge), Ok(Fixed64::from_raw(1)));
        assert_eq!(pool.current(), pool.maximum());
    }

    #[test]
    fn mana_pool_serialization_validates_state_and_preserves_fractional_replay() {
        let mut pool = ManaPool::new(Fixed64::ZERO, f(100)).unwrap();
        pool.regenerate(Fixed64::from_raw(1), Fixed64::from_raw(17))
            .unwrap();
        let encoded = serde_json::to_string(&pool).unwrap();
        let mut restored: ManaPool = serde_json::from_str(&encoded).unwrap();
        pool.regenerate(Fixed64::from_raw(1), Fixed64::from_raw(1007))
            .unwrap();
        restored
            .regenerate(Fixed64::from_raw(1), Fixed64::from_raw(1007))
            .unwrap();
        assert_eq!(pool, restored);
        for (current, maximum, remainder) in [
            (f(-1), f(100), 0),
            (f(101), f(100), 0),
            (f(0), f(-1), 0),
            (f(0), f(100), 1024),
            (f(100), f(100), 1),
        ] {
            let invalid = ManaPoolWire {
                current,
                maximum,
                regeneration_remainder: remainder,
            };
            assert!(
                serde_json::from_str::<ManaPool>(&serde_json::to_string(&invalid).unwrap())
                    .is_err()
            );
        }
    }
}
