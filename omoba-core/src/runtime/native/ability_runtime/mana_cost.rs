//! Shared checked admission cost for authority and owner-side planners.
use super::ManaPoolError;
use omoba_sim::Fixed64;

pub fn checked_mana_cost(cost: f32, multiplier: Fixed64) -> Result<Fixed64, ManaPoolError> {
    if !cost.is_finite() || !(0.0..=1_000_000.0).contains(&cost)
        || (cost > 0.0 && cost < 1.0 / 1024.0) || multiplier < Fixed64::ZERO
    {
        return Err(ManaPoolError::InvalidAmount);
    }
    let base = (f64::from(cost) * 1024.0).round() as i64;
    // Positive fractional products round upward; only an explicit zero
    // multiplier can make a nonzero cost free. Widen before multiplying.
    let adjusted = (i128::from(base) * i128::from(multiplier.raw()) + 1023) / 1024;
    i64::try_from(adjusted).map(Fixed64::from_raw).map_err(|_| ManaPoolError::InvalidAmount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mana_budget_cost_is_checked_and_rounds_positive_products_up() {
        assert_eq!(checked_mana_cost(45.0, Fixed64::ONE), Ok(Fixed64::from_i32(45)));
        assert_eq!(checked_mana_cost(1.0 / 1024.0, Fixed64::from_raw(1)), Ok(Fixed64::from_raw(1)));
        assert_eq!(checked_mana_cost(45.0, Fixed64::from_raw(512)), Ok(Fixed64::from_raw(45 * 512)));
        assert_eq!(checked_mana_cost(45.0, Fixed64::ZERO), Ok(Fixed64::ZERO));
        assert_eq!(checked_mana_cost(0.0, Fixed64::from_raw(i64::MAX)), Ok(Fixed64::ZERO));
        for cost in [-1.0, f32::NAN, f32::INFINITY, 1.0 / 2048.0, 1_000_001.0] {
            assert_eq!(checked_mana_cost(cost, Fixed64::ONE), Err(ManaPoolError::InvalidAmount));
        }
        assert!(checked_mana_cost(45.0, Fixed64::from_raw(-1)).is_err());
        assert!(checked_mana_cost(1_000_000.0, Fixed64::from_raw(i64::MAX)).is_err());
    }
}
