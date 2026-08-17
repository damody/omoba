use std::collections::{BTreeMap, VecDeque};

use crate::runtime::PlayerEconomy;

pub const TD_LEDGER_RECENT_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TdEconomyCategory {
    Initialize,
    LayerIncome,
    RoundBonus,
    TowerPlace,
    TowerUpgrade,
    TowerSell,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TdEconomyEntry {
    pub tick: u64,
    pub serial: u64,
    pub player_id: Option<u32>,
    pub category: TdEconomyCategory,
    pub amount: i32,
    pub resulting_balance: Option<i32>,
    pub source_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TdEconomyRules {
    pub starting_cash: i32,
    pub sellback_numerator: i32,
    pub sellback_denominator: i32,
}

impl Default for TdEconomyRules {
    fn default() -> Self {
        Self {
            starting_cash: 650,
            // One ratio for base and upgrades. Integer division is deterministic floor rounding.
            sellback_numerator: 3,
            sellback_denominator: 4,
        }
    }
}

impl TdEconomyRules {
    pub fn round_bonus(self, round: usize) -> i32 {
        100i32.saturating_add(round.try_into().unwrap_or(i32::MAX))
    }

    pub fn sell_refund(self, total_spend: i32) -> i32 {
        if total_spend <= 0 {
            return 0;
        }
        total_spend
            .saturating_mul(self.sellback_numerator)
            .checked_div(self.sellback_denominator)
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
pub struct TdEconomyLedger {
    next_serial: u64,
    totals: BTreeMap<(Option<u32>, TdEconomyCategory), i64>,
    digest: u64,
    recent: VecDeque<TdEconomyEntry>,
    full_observer: Option<Vec<TdEconomyEntry>>,
    pub unattributed_layer_cash: u64,
}

impl Default for TdEconomyLedger {
    fn default() -> Self {
        Self {
            next_serial: 0,
            totals: BTreeMap::new(),
            digest: 0xcbf29ce484222325,
            recent: VecDeque::new(),
            full_observer: None,
            unattributed_layer_cash: 0,
        }
    }
}

impl TdEconomyLedger {
    pub fn enable_full_observer(&mut self) {
        if self.full_observer.is_none() {
            self.full_observer = Some(self.recent.iter().cloned().collect());
        }
    }

    pub fn digest(&self) -> u64 {
        self.digest
    }

    pub fn recent(&self) -> &VecDeque<TdEconomyEntry> {
        &self.recent
    }

    pub fn totals(&self) -> &BTreeMap<(Option<u32>, TdEconomyCategory), i64> {
        &self.totals
    }

    pub fn observed(&self) -> Option<&[TdEconomyEntry]> {
        self.full_observer.as_deref()
    }

    pub fn apply(
        &mut self,
        economy: &mut PlayerEconomy,
        tick: u64,
        player_id: Option<u32>,
        category: TdEconomyCategory,
        amount: i32,
        source_id: impl Into<String>,
    ) -> Result<Option<i32>, String> {
        validate_sign(category, amount)?;
        let resulting_balance = match (player_id, category) {
            (Some(player_id), TdEconomyCategory::Initialize) => {
                economy.initialize(player_id, amount);
                economy.balance(player_id)
            }
            (Some(player_id), _) if amount < 0 => Some(
                economy
                    .try_debit(player_id, amount.saturating_abs())
                    .map_err(|error| error.to_string())?,
            ),
            (Some(player_id), _) => Some(
                economy
                    .credit_saturating(player_id, amount)
                    .map_err(|error| error.to_string())?,
            ),
            (None, TdEconomyCategory::LayerIncome) => {
                self.unattributed_layer_cash = self
                    .unattributed_layer_cash
                    .saturating_add(amount.max(0) as u64);
                None
            }
            (None, _) => return Err("only layer income may be unattributed".to_string()),
        };

        self.next_serial = self.next_serial.wrapping_add(1);
        let entry = TdEconomyEntry {
            tick,
            serial: self.next_serial,
            player_id,
            category,
            amount,
            resulting_balance,
            source_id: source_id.into(),
        };
        *self.totals.entry((player_id, category)).or_default() += amount as i64;
        self.digest = digest_entry(self.digest, &entry);
        if self.recent.len() == TD_LEDGER_RECENT_CAPACITY {
            self.recent.pop_front();
        }
        self.recent.push_back(entry.clone());
        if let Some(observer) = &mut self.full_observer {
            observer.push(entry);
        }
        Ok(resulting_balance)
    }
}

fn validate_sign(category: TdEconomyCategory, amount: i32) -> Result<(), String> {
    let valid = match category {
        TdEconomyCategory::TowerPlace | TdEconomyCategory::TowerUpgrade => amount <= 0,
        _ => amount >= 0,
    };
    valid
        .then_some(())
        .ok_or_else(|| format!("invalid {:?} ledger amount={amount}", category))
}

fn digest_entry(mut digest: u64, entry: &TdEconomyEntry) -> u64 {
    fn feed(digest: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *digest ^= *byte as u64;
            *digest = digest.wrapping_mul(0x100000001b3);
        }
    }
    feed(&mut digest, &entry.tick.to_le_bytes());
    feed(&mut digest, &entry.serial.to_le_bytes());
    feed(
        &mut digest,
        &entry.player_id.unwrap_or(u32::MAX).to_le_bytes(),
    );
    feed(&mut digest, &(entry.category as u8).to_le_bytes());
    feed(&mut digest, &entry.amount.to_le_bytes());
    feed(
        &mut digest,
        &entry.resulting_balance.unwrap_or(i32::MIN).to_le_bytes(),
    );
    feed(&mut digest, entry.source_id.as_bytes());
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_use_one_sell_ratio_and_separate_round_bonus() {
        let rules = TdEconomyRules::default();
        assert_eq!(rules.sell_refund(201), 150);
        assert_eq!(rules.round_bonus(1), 101);
        assert_eq!(rules.round_bonus(100), 200);
    }

    #[test]
    fn ledger_is_atomic_bounded_and_replayable() {
        let mut economy = PlayerEconomy::default();
        let mut ledger = TdEconomyLedger::default();
        ledger.enable_full_observer();
        ledger
            .apply(
                &mut economy,
                0,
                Some(1),
                TdEconomyCategory::Initialize,
                650,
                "easy",
            )
            .unwrap();
        let digest_before_reject = ledger.digest();
        assert!(ledger
            .apply(
                &mut economy,
                1,
                Some(1),
                TdEconomyCategory::TowerPlace,
                -651,
                "tower_dart",
            )
            .is_err());
        assert_eq!(ledger.digest(), digest_before_reject);
        assert_eq!(economy.balance(1), Some(650));
        for tick in 1..=140 {
            ledger
                .apply(
                    &mut economy,
                    tick,
                    Some(1),
                    TdEconomyCategory::LayerIncome,
                    1,
                    "red",
                )
                .unwrap();
        }
        assert_eq!(ledger.recent().len(), TD_LEDGER_RECENT_CAPACITY);
        assert_eq!(ledger.observed().unwrap().len(), 141);
        assert_eq!(economy.balance(1), Some(790));
    }

    #[test]
    fn enabling_full_observer_backfills_existing_entries() {
        let mut economy = PlayerEconomy::default();
        let mut ledger = TdEconomyLedger::default();
        ledger
            .apply(
                &mut economy,
                0,
                Some(1),
                TdEconomyCategory::Initialize,
                650,
                "easy",
            )
            .unwrap();

        ledger.enable_full_observer();

        let observed = ledger.observed().unwrap();
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].serial, 1);
        assert_eq!(observed[0].resulting_balance, Some(650));
    }

    #[test]
    fn ownerless_layer_income_is_diagnostic_only() {
        let mut economy = PlayerEconomy::default();
        let mut ledger = TdEconomyLedger::default();
        ledger
            .apply(
                &mut economy,
                2,
                None,
                TdEconomyCategory::LayerIncome,
                17,
                "ceramic",
            )
            .unwrap();
        assert_eq!(ledger.unattributed_layer_cash, 17);
        assert!(economy.balances().is_empty());
    }

    #[test]
    fn identical_replica_entries_produce_identical_digest_and_balance() {
        fn run() -> (u64, i32) {
            let mut economy = PlayerEconomy::default();
            let mut ledger = TdEconomyLedger::default();
            for (tick, category, amount, source) in [
                (0, TdEconomyCategory::Initialize, 650, "easy"),
                (4, TdEconomyCategory::TowerPlace, -200, "tower_dart"),
                (8, TdEconomyCategory::LayerIncome, 23, "zebra"),
                (9, TdEconomyCategory::RoundBonus, 101, "round:1"),
                (12, TdEconomyCategory::TowerUpgrade, -90, "dart:0:1"),
                (16, TdEconomyCategory::TowerSell, 217, "tower_dart"),
            ] {
                ledger
                    .apply(&mut economy, tick, Some(1), category, amount, source)
                    .unwrap();
            }
            (ledger.digest(), economy.balance(1).unwrap())
        }
        assert_eq!(run(), run());
    }
}
