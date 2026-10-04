//! Bounded read-only recovery. Never resubmit a transaction with a new ID.
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PendingShopQueries {
    entries: BTreeMap<u32, u64>,
}
impl PendingShopQueries {
    pub fn can_track(&self) -> bool {
        self.entries.len() < 64
    }
    pub fn track(&mut self, id: u32, now_ms: u64) {
        self.entries.entry(id).or_insert(now_ms);
    }
    pub fn finish(&mut self, id: u32) {
        self.entries.remove(&id);
    }
    pub fn due(&mut self, now_ms: u64) -> Vec<u32> {
        let mut candidates: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, last)| now_ms.saturating_sub(**last) >= 1000)
            .map(|(id, last)| (*last, *id))
            .collect();
        candidates.sort_unstable();
        let ids: Vec<_> = candidates.into_iter().take(4).map(|(_, id)| id).collect();
        for id in &ids {
            self.entries.insert(*id, now_ms);
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_is_bounded_throttled_and_does_not_replace_original_ids() {
        let mut pending = PendingShopQueries::default();
        for id in 1..=64 {
            assert!(pending.can_track());
            pending.track(id, 0);
        }
        assert!(!pending.can_track());
        assert!(pending.due(999).is_empty());
        assert_eq!(pending.due(1000), vec![1, 2, 3, 4]);
        assert_eq!(pending.due(1000), vec![5, 6, 7, 8]);
        pending.finish(1);
        assert!(pending.can_track());
        assert_eq!(
            pending.due(2000),
            vec![9, 10, 11, 12],
            "oldest pending must not starve behind low IDs"
        );
    }
}
