use omoba_core::game_proto::{PresentationEffect, TeamPresentationSnapshot};
use omoba_core::runtime::presentation_cue::DamagePresentationCue;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const MAX_DAMAGE: usize = 1024;
const WINDOW_TICKS: u64 = 4096;

struct Entry {
    effect: PresentationEffect,
    cue: DamagePresentationCue,
    sent: Option<(u64, u64)>, // connection identity, first successfully sent snapshot
}

#[derive(Default)]
pub(crate) struct DamageRetention {
    view_epoch: Option<u64>,
    high_tick: u64,
    entries: BTreeMap<u64, Entry>,
    pub(crate) dropped: u64,
}

impl DamageRetention {
    pub(crate) fn capture(&mut self, view_epoch: u64, tick: u64, effects: Vec<PresentationEffect>) {
        if self.view_epoch != Some(view_epoch) {
            self.entries.clear();
            self.high_tick = tick;
            self.view_epoch = Some(view_epoch);
        }
        self.high_tick = self.high_tick.max(tick);
        let floor = self.high_tick.saturating_sub(WINDOW_TICKS);
        self.entries.retain(|_, entry| entry.cue.tick >= floor);
        for effect in effects {
            let Some(cue) = DamagePresentationCue::decode(&effect.safe_payload) else {
                continue;
            };
            if effect.effect_id >> 32 != cue.tick
                || effect.effect_id as u32 == 0
                || cue.tick > tick
                || cue.tick < floor
            {
                continue;
            }
            if self.entries.contains_key(&effect.effect_id) {
                continue;
            }
            if self.entries.len() >= MAX_DAMAGE {
                self.dropped = self.dropped.saturating_add(1);
                if self.dropped == 1 {
                    log::warn!("damage retention full (1024); excess one-shots dropped");
                }
                continue;
            }
            self.entries.insert(
                effect.effect_id,
                Entry {
                    effect,
                    cue,
                    sent: None,
                },
            );
        }
    }

    pub(crate) fn project(&mut self, snapshot: &mut TeamPresentationSnapshot) {
        if self.entries.is_empty() || self.view_epoch != Some(snapshot.view_epoch) {
            return;
        }
        let visible = snapshot
            .entities
            .iter()
            .map(|entity| (entity.render_id, entity.disclosure_epoch))
            .collect::<BTreeSet<_>>();
        self.entries.retain(|_, entry| {
            visible.contains(&(entry.cue.target_id, entry.cue.disclosure_epoch))
        });
        let mut projected = snapshot
            .effects
            .iter()
            .map(|effect| effect.effect_id)
            .collect::<BTreeSet<_>>();
        for entry in self.entries.values() {
            if projected.insert(entry.effect.effect_id) {
                snapshot.effects.push(entry.effect.clone());
            }
        }
    }

    pub(crate) fn baseline(&mut self, tick: u64) -> u64 {
        // The saved latest view can be stale while no renderer is connected.
        // Do not replay events accumulated since that view during the outage.
        let floor = tick.max(self.high_tick);
        self.entries.retain(|_, entry| entry.cue.tick > floor);
        for entry in self.entries.values_mut() {
            entry.sent = None;
        }
        floor
    }

    pub(crate) fn sent(
        &mut self,
        connection: u64,
        sequence: u64,
        snapshot: &TeamPresentationSnapshot,
    ) {
        for effect in &snapshot.effects {
            if let Some(entry) = self.entries.get_mut(&effect.effect_id) {
                if entry
                    .sent
                    .is_none_or(|(generation, _)| generation != connection)
                {
                    entry.sent = Some((connection, sequence));
                }
            }
        }
    }

    pub(crate) fn consumed(&mut self, connection: u64, sequence: u64) {
        self.entries.retain(|_, entry| {
            !entry
                .sent
                .is_some_and(|(generation, first)| generation == connection && first <= sequence)
        });
    }

    pub(crate) fn contains(&self, id: u64) -> bool {
        self.entries.contains_key(&id)
    }
    pub(crate) fn invalidate(&mut self, target: u64, epoch: u64) {
        self.entries.retain(|_, entry| {
            entry.cue.target_id != target || entry.cue.disclosure_epoch != epoch
        });
    }
    pub(crate) fn reset(&mut self) {
        self.entries.clear();
        self.view_epoch = None;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use omoba_core::{
        game_proto::PresentationRenderEntity, runtime::presentation_cue::presentation_effect_id,
    };

    pub(crate) fn effect(tick: u64, ordinal: u32) -> PresentationEffect {
        PresentationEffect {
            effect_id: presentation_effect_id(tick, ordinal).unwrap(),
            safe_payload: DamagePresentationCue {
                tick,
                target_id: 4,
                disclosure_epoch: 3,
                amount_milli: 1000,
            }
            .encode(),
        }
    }

    pub(crate) fn snapshot(tick: u64) -> TeamPresentationSnapshot {
        TeamPresentationSnapshot {
            replica_tick: tick,
            view_epoch: 7,
            entities: vec![PresentationRenderEntity {
                render_id: 4,
                disclosure_epoch: 3,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn unpublished_steps_survive_until_sent_and_consumed_on_matching_connection() {
        let mut ledger = DamageRetention::default();
        ledger.capture(7, 2, vec![effect(2, 0)]);
        ledger.capture(7, 3, vec![effect(3, 0), effect(2, 0)]);
        let mut view = snapshot(3);
        ledger.project(&mut view);
        ledger.project(&mut view);
        assert_eq!(view.effects.len(), 2);
        ledger.consumed(8, 100); // Preparing/publishing is not sending.
        assert_eq!(ledger.entries.len(), 2);
        ledger.sent(8, 10, &view);
        ledger.sent(8, 20, &view); // Preserve first successful delivery.
        ledger.consumed(9, 100);
        ledger.consumed(8, 9);
        assert_eq!(ledger.entries.len(), 2);
        ledger.consumed(8, 10);
        ledger.consumed(8, 10);
        assert!(ledger.entries.is_empty());
    }

    #[test]
    fn reconnect_baseline_visibility_epoch_and_reset_retire_old_cues() {
        let mut ledger = DamageRetention::default();
        ledger.capture(7, 3, vec![effect(2, 0), effect(3, 0)]);
        assert_eq!(ledger.baseline(2), 3); // Stale saved view, current replica tick 3.
        assert!(!ledger.contains(effect(2, 0).effect_id));
        assert!(!ledger.contains(effect(3, 0).effect_id));
        ledger.capture(7, 4, vec![effect(4, 0)]);
        assert!(ledger.contains(effect(4, 0).effect_id));
        ledger.invalidate(4, 2); // Stale hide cannot remove a new disclosure.
        assert_eq!(ledger.entries.len(), 1);
        ledger.invalidate(4, 3);
        assert!(ledger.entries.is_empty());
        ledger.capture(7, 4, vec![effect(4, 0)]);
        let mut wrong_epoch = snapshot(4);
        wrong_epoch.view_epoch = 6;
        ledger.project(&mut wrong_epoch);
        assert!(wrong_epoch.effects.is_empty());
        let mut hidden = snapshot(4);
        hidden.entities.clear();
        ledger.project(&mut hidden);
        assert!(ledger.entries.is_empty());
        ledger.capture(8, 5, vec![effect(5, 0)]);
        ledger.capture(9, 6, vec![]);
        assert!(ledger.entries.is_empty());
        ledger.capture(9, 7, vec![effect(7, 0)]);
        ledger.reset();
        assert!(ledger.entries.is_empty());
    }

    #[test]
    fn retention_is_bounded_and_rejects_malformed_future_and_expired_cues() {
        let mut ledger = DamageRetention::default();
        ledger.capture(
            7,
            2,
            (0..MAX_DAMAGE as u32 + 1).map(|i| effect(2, i)).collect(),
        );
        assert_eq!(ledger.entries.len(), MAX_DAMAGE);
        assert_eq!(ledger.dropped, 1);
        ledger.capture(
            7,
            4099,
            vec![
                effect(2, 0),
                effect(4100, 0),
                PresentationEffect {
                    effect_id: 1,
                    safe_payload: vec![0],
                },
            ],
        );
        assert!(ledger.entries.is_empty());
        ledger.capture(7, 4099, vec![effect(4099, 0)]);
        assert_eq!(ledger.entries.len(), 1);
    }
}
