use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerEconomy {
    balances: BTreeMap<u32, i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerEconomyError {
    MissingAccount {
        player_id: u32,
    },
    InvalidCost {
        cost: i32,
    },
    InsufficientFunds {
        player_id: u32,
        available: i32,
        required: i32,
    },
}

impl fmt::Display for PlayerEconomyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAccount { player_id } => {
                write!(f, "missing player economy account for player_id={player_id}")
            }
            Self::InvalidCost { cost } => write!(f, "invalid player economy cost={cost}"),
            Self::InsufficientFunds {
                player_id,
                available,
                required,
            } => write!(
                f,
                "insufficient gold for player_id={player_id}: available={available} required={required}"
            ),
        }
    }
}

impl std::error::Error for PlayerEconomyError {}

impl PlayerEconomy {
    pub fn initialize(&mut self, player_id: u32, balance: i32) {
        self.balances.insert(player_id, balance.max(0));
    }

    pub fn balance(&self, player_id: u32) -> Option<i32> {
        self.balances.get(&player_id).copied()
    }

    pub fn balances(&self) -> &BTreeMap<u32, i32> {
        &self.balances
    }

    pub fn try_debit(&mut self, player_id: u32, cost: i32) -> Result<i32, PlayerEconomyError> {
        if cost < 0 {
            return Err(PlayerEconomyError::InvalidCost { cost });
        }
        let balance = self
            .balances
            .get_mut(&player_id)
            .ok_or(PlayerEconomyError::MissingAccount { player_id })?;
        if *balance < cost {
            return Err(PlayerEconomyError::InsufficientFunds {
                player_id,
                available: *balance,
                required: cost,
            });
        }
        *balance -= cost;
        Ok(*balance)
    }

    pub fn credit_saturating(
        &mut self,
        player_id: u32,
        amount: i32,
    ) -> Result<i32, PlayerEconomyError> {
        let balance = self
            .balances
            .get_mut(&player_id)
            .ok_or(PlayerEconomyError::MissingAccount { player_id })?;
        *balance = balance.saturating_add(amount).max(0);
        Ok(*balance)
    }

    pub fn credit_all_saturating(&mut self, amount: i32) {
        for balance in self.balances.values_mut() {
            *balance = balance.saturating_add(amount).max(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transactions_are_atomic_and_checked() {
        let mut economy = PlayerEconomy::default();
        economy.initialize(1, 650);
        assert_eq!(economy.try_debit(1, 200), Ok(450));
        assert_eq!(economy.balance(1), Some(450));

        assert!(matches!(
            economy.try_debit(1, 451),
            Err(PlayerEconomyError::InsufficientFunds { .. })
        ));
        assert_eq!(economy.balance(1), Some(450));
        assert!(matches!(
            economy.try_debit(1, -1),
            Err(PlayerEconomyError::InvalidCost { .. })
        ));
        assert!(matches!(
            economy.try_debit(2, 1),
            Err(PlayerEconomyError::MissingAccount { .. })
        ));
    }

    #[test]
    fn credits_saturate_and_accounts_are_ordered() {
        let mut economy = PlayerEconomy::default();
        economy.initialize(2, 10);
        economy.initialize(1, i32::MAX - 1);
        assert_eq!(economy.credit_saturating(1, 10), Ok(i32::MAX));
        assert_eq!(
            economy.balances().keys().copied().collect::<Vec<_>>(),
            vec![1, 2]
        );
    }
}
