//! Authority-only, per-victim-life damage participation. Never replicated.
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssistParticipant {
    pub player_id: u32,
    pub team_id: u32,
}

#[derive(Clone, Debug, Default)]
pub struct AssistLedger {
    // Stable generation-packed victim identity -> latest positive damage per player.
    victims: BTreeMap<u64, BTreeMap<u32, (u32, i64)>>,
}

impl AssistLedger {
    pub fn record(&mut self, victim: u64, victim_player: AssistParticipant,
        source: AssistParticipant, now_raw: i64, positive_damage: bool) {
        if !positive_damage || now_raw < 0 || victim == 0 || source.player_id == 0
            || victim_player.player_id == 0 || source.player_id == victim_player.player_id
            || source.team_id == 0 || victim_player.team_id == 0 || source.team_id == victim_player.team_id { return; }
        // The match caller supplies only its bounded authenticated hero roster.
        if !self.victims.contains_key(&victim) && self.victims.len() >= 10 { return; }
        let entries = self.victims.entry(victim).or_default();
        if !entries.contains_key(&source.player_id) && entries.len() >= 10 { return; }
        if let Some((_, previous)) = entries.get(&source.player_id) {
            if now_raw < *previous { return; }
        }
        entries.insert(source.player_id, (source.team_id, now_raw));
    }

    pub fn retire(&mut self, victim: u64) { self.victims.remove(&victim); }

    /// Consume once even for an uncredited death. Exact window endpoint is eligible.
    /// Roster members may be dead; player identity survives their hero respawn.
    pub fn settle(&mut self, victim: u64, killer: Option<AssistParticipant>,
        now_raw: i64, window_raw: i64, roster: &[AssistParticipant]) -> Vec<u32> {
        let Some(entries) = self.victims.remove(&victim) else { return Vec::new(); };
        let Some(killer) = killer else { return Vec::new(); };
        if window_raw <= 0 || now_raw < 0 || killer.player_id == 0 || killer.team_id == 0 || !roster.contains(&killer) { return Vec::new(); }
        entries.into_iter().filter_map(|(player_id, (team_id, at))| {
            let player = AssistParticipant {player_id, team_id};
            (player_id != killer.player_id && team_id == killer.team_id && roster.contains(&player)
                && now_raw.checked_sub(at).is_some_and(|age| age >= 0 && age <= window_raw))
                .then_some(player_id)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn player(id: u32, team: u32) -> AssistParticipant { AssistParticipant {player_id: id, team_id: team} }
    #[test]
    fn repeated_damage_multiple_helpers_once_and_sorted() {
        let roster = [player(1,1), player(3,1), player(5,1), player(2,2)];
        let mut ledger = AssistLedger::default();
        for id in [5,3,3,1] { ledger.record(99, roster[3], player(id,1), 100, true); }
        assert_eq!(ledger.settle(99, Some(roster[0]), 110, 10, &roster), vec![3,5]);
        assert!(ledger.settle(99, Some(roster[0]), 110, 10, &roster).is_empty());
    }
    #[test]
    fn window_endpoint_expiry_future_and_latest_hit() {
        let roster = [player(1,1), player(3,1), player(2,2)];
        for (age, expected) in [(0,true),(10,true),(11,false),(-1,false)] {
            let mut ledger = AssistLedger::default();
            ledger.record(99, roster[2], roster[1], 100, true);
            assert_eq!(!ledger.settle(99, Some(roster[0]), 100+age, 10, &roster).is_empty(), expected);
        }
        let mut ledger = AssistLedger::default();
        for at in [50,100,40] { ledger.record(99, roster[2], roster[1], at, true); }
        assert_eq!(ledger.settle(99, Some(roster[0]), 110, 10, &roster), vec![3]);
    }
    #[test]
    fn reject_self_friendly_zero_and_changed_roster() {
        let roster = [player(1,1), player(3,1), player(2,2)];
        let mut ledger = AssistLedger::default();
        ledger.record(99, roster[2], roster[1], 100, false);
        ledger.record(99, roster[2], roster[2], 100, true);
        ledger.record(99, roster[0], roster[1], 100, true);
        assert!(ledger.settle(99, Some(roster[0]), 110, 10, &roster).is_empty());
        ledger.record(99, roster[2], roster[1], 100, true);
        assert!(ledger.settle(99, Some(roster[0]), 110, 10, &[roster[0],player(3,2),roster[2]]).is_empty());
    }
    #[test]
    fn uncredited_death_and_generation_retirement_do_not_leak() {
        let roster = [player(1,1), player(3,1), player(2,2)];
        let mut ledger = AssistLedger::default();
        ledger.record(99, roster[2], roster[1], 100, true);
        assert!(ledger.settle(99, None, 100, 10, &roster).is_empty());
        ledger.record(99, roster[2], roster[1], 100, true);
        ledger.retire(99);
        assert!(ledger.settle(100, Some(roster[0]), 100, 10, &roster).is_empty());
        assert!(ledger.settle(99, Some(roster[0]), 100, 10, &roster).is_empty());
    }
    #[test]
    fn capacity_is_bounded_and_duplicate_updates_still_work() {
        let mut ledger = AssistLedger::default();
        for target in 1..=11 { ledger.record(target, player(30,2), player(3,1), 100, true); }
        assert_eq!(ledger.victims.len(), 10);
        for source in 1..=11 { ledger.record(1, player(30,2), player(source,1), 100, true); }
        assert_eq!(ledger.victims[&1].len(), 10);
        ledger.record(1, player(30,2), player(3,1), 200, true);
        assert_eq!(ledger.victims[&1][&3].1, 200);
        ledger.retire(2);
        ledger.record(11, player(30,2), player(3,1), 200, true);
        assert!(ledger.victims.contains_key(&11));
    }
}
