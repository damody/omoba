use omoba_core::game_proto::{PresentationEffect, TeamPresentationSnapshot};
use omoba_core::runtime::presentation_cue::PresentationCue;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const MAX_CUES: usize = omoba_core::runtime::presentation_cue::MAX_PRESENTATION_CUES_PER_SNAPSHOT;
const WINDOW_TICKS: u64 = 4096;

struct Entry {
    effect: PresentationEffect,
    cue: PresentationCue,
    sent: Option<(u64, u64)>, // connection identity, first successfully sent snapshot
}

#[derive(Default)]
pub(crate) struct CueRetention {
    view_epoch: Option<u64>,
    high_tick: u64,
    entries: BTreeMap<u64, Entry>,
    // Delivery retirement must not make an already admitted ID new again.
    // IDs encode (tick, ordinal); compacting history advances a fail-closed
    // floor rather than allowing evicted IDs to replay.
    seen: BTreeSet<u64>,
    retired_through: u64,
    baseline_tick: Option<u64>,
    pub(crate) dropped: u64,
}

impl CueRetention {
    pub(crate) fn capture(&mut self, view_epoch: u64, tick: u64, effects: Vec<PresentationEffect>) {
        if self.view_epoch != Some(view_epoch) {
            self.entries.clear();
            self.seen.clear();
            self.retired_through = 0;
            self.baseline_tick = None;
            self.high_tick = tick;
            self.view_epoch = Some(view_epoch);
        }
        self.high_tick = self.high_tick.max(tick);
        let floor = self.high_tick.saturating_sub(WINDOW_TICKS);
        self.entries.retain(|_, entry| entry.cue.tick() >= floor);
        // Sorted IDs put expired ticks first; do not scan the whole history
        // on every applied simulation step when nothing has expired.
        while self.seen.first().is_some_and(|id| id >> 32 < floor) {
            self.seen.pop_first();
        }
        for effect in effects {
            let Some(cue) = PresentationCue::from_effect(effect.effect_id, &effect.safe_payload, tick) else {
                continue;
            };
            if cue.tick() < floor
                || self.baseline_tick.is_some_and(|baseline| cue.tick() <= baseline)
                || effect.effect_id <= self.retired_through
                || self.seen.contains(&effect.effect_id)
            {
                continue;
            }
            if self.entries.contains_key(&effect.effect_id) {
                continue;
            }
            if self.entries.len() >= MAX_CUES {
                self.dropped = self.dropped.saturating_add(1);
                if self.dropped == 1 {
                    log::warn!("presentation cue retention full (1024); excess one-shots dropped");
                }
                continue;
            }
            self.seen.insert(effect.effect_id);
            if self.seen.len() > MAX_CUES {
                let oldest = self.seen.pop_first().expect("nonempty cue history");
                self.retired_through = self.retired_through.max(oldest);
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
            visible.contains(&entry.cue.disclosed_entity())
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
        self.baseline_tick = Some(self.baseline_tick.map_or(floor, |old| old.max(floor)));
        self.entries.retain(|_, entry| entry.cue.tick() > floor);
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
            entry.cue.disclosed_entity() != (target, epoch)
        });
    }
    pub(crate) fn reset(&mut self) {
        self.entries.clear();
        self.seen.clear();
        self.retired_through = 0;
        self.baseline_tick = None;
        self.view_epoch = None;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use omoba_core::runtime::presentation_cue::{DamagePresentationCue, AbilityPresentationCue};
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

    pub(crate) fn ability_effect(tick: u64, ordinal: u32) -> PresentationEffect {
        ability_effect_with_rank(tick, ordinal, 0)
    }

    pub(crate) fn ranked_ability_effect(tick: u64, ordinal: u32) -> PresentationEffect {
        ability_effect_with_rank(tick, ordinal, 3)
    }

    pub(crate) fn relocation_effect(tick: u64, ordinal: u32) -> PresentationEffect {
        PresentationEffect {
            effect_id: presentation_effect_id(tick, ordinal).unwrap(),
            safe_payload: AbilityPresentationCue {tick, caster_id: 4, disclosure_epoch: 3,
                ability_id: 123, rank: 3, caster_relocation: Some((0, 300 * 1024))}.encode(),
        }
    }

    pub(crate) fn area_effect(tick:u64,ordinal:u32)->PresentationEffect {
        use omoba_core::runtime::presentation_cue::AbilityAreaPresentationCue;
        PresentationEffect {effect_id:presentation_effect_id(tick,ordinal).unwrap(),safe_payload:AbilityAreaPresentationCue {
            cast:AbilityPresentationCue {tick,caster_id:4,disclosure_epoch:3,ability_id:123,rank:3,caster_relocation:None},
            center:(0,300*1024),radius_raw:200*1024,duration_raw:1024}.encode()}
    }

    pub(crate) fn impact_effect(tick:u64,ordinal:u32)->PresentationEffect {
        use omoba_core::runtime::presentation_cue::ProjectileImpactPresentationCue;
        PresentationEffect {effect_id:presentation_effect_id(tick,ordinal).unwrap(),
            safe_payload:ProjectileImpactPresentationCue {tick,target_id:4,disclosure_epoch:3}.encode()}
    }

    fn ability_effect_with_rank(tick: u64, ordinal: u32, rank: u32) -> PresentationEffect {
        PresentationEffect {
            effect_id: presentation_effect_id(tick, ordinal).unwrap(),
            safe_payload: AbilityPresentationCue {tick, caster_id: 4, disclosure_epoch: 3, ability_id: 123, rank, caster_relocation: None}.encode(),
        }
    }

    #[test]
    fn retired_cues_cannot_be_recaptured_after_ack_hide_or_baseline() {
        let cues = vec![effect(10,0), ability_effect(10,1), relocation_effect(10,2),
            area_effect(10,3), impact_effect(10,4)];
        let mut ledger=CueRetention::default();
        ledger.capture(7,10,cues.clone());
        let mut view=snapshot(10);
        ledger.project(&mut view);
        assert_eq!(view.effects.len(),5);
        ledger.sent(2,40,&view);
        ledger.consumed(3,40); // A stale connection cannot retire anything.
        assert_eq!(ledger.entries.len(),5);
        ledger.consumed(2,40);
        ledger.capture(7,11,cues.clone());
        assert!(ledger.entries.is_empty(),"ACK must not reopen admission");
        ledger.capture(7,11,vec![effect(11,0)]);
        ledger.invalidate(4,3);
        ledger.capture(7,11,vec![effect(11,0)]);
        assert!(ledger.entries.is_empty(),"Hide/Forget must not resurrect old cue IDs");
        assert_eq!(ledger.baseline(10),11);
        ledger.capture(7,12,vec![effect(11,99),effect(12,0)]);
        assert_eq!(ledger.entries.len(),1,"late historical IDs cannot cross reconnect floor");
        ledger.capture(8,10,cues);
        assert_eq!(ledger.entries.len(),5,"a new view epoch has its own identity domain");
        ledger.reset();
        ledger.capture(8,10,vec![effect(10,0)]);
        assert_eq!(ledger.entries.len(),1);
    }

    #[test]
    fn retired_cue_history_is_bounded_without_reopening_evicted_ids() {
        let mut ledger=CueRetention::default();
        // ACK each entry so this exercises history capacity, not pending capacity.
        for ordinal in 0..MAX_CUES as u32 + 20 {
            ledger.capture(7,20,vec![effect(20,ordinal)]);
            let mut view=snapshot(20);
            ledger.project(&mut view);
            assert_eq!(view.effects.len(),1);
            ledger.sent(2,u64::from(ordinal)+1,&view);
            ledger.consumed(2,u64::from(ordinal)+1);
        }
        assert_eq!(ledger.seen.len(),MAX_CUES);
        assert_eq!(ledger.dropped,0);
        ledger.capture(7,21,vec![effect(20,0),effect(20,MAX_CUES as u32+19)]);
        assert!(ledger.entries.is_empty());
        ledger.capture(7,21,vec![effect(21,0)]);
        assert_eq!(ledger.entries.len(),1);
        ledger.capture(7,21+WINDOW_TICKS+1,vec![effect(20,0)]);
        assert!(ledger.entries.is_empty());
        assert!(ledger.seen.len()<=MAX_CUES);
    }

    #[test]
    fn mixed_cues_share_ack_visibility_capacity_and_reconnect_contract() {
        let mut ledger = CueRetention::default();
        let damage = effect(3, 0);
        let ability = ability_effect(3, 1);
        ledger.capture(7, 3, vec![damage.clone(), ability.clone(), ability.clone()]);
        let mut view = snapshot(3);
        ledger.project(&mut view);
        assert_eq!(view.effects, vec![damage, ability]);
        ledger.sent(8, 10, &view);
        ledger.consumed(9, 10);
        assert_eq!(ledger.entries.len(), 2);
        ledger.consumed(8, 10);
        assert!(ledger.entries.is_empty());
        ledger.capture(7, 4, vec![effect(4, 0), ability_effect(4, 1)]);
        ledger.invalidate(4, 2);
        assert_eq!(ledger.entries.len(), 2);
        ledger.invalidate(4, 3);
        assert!(ledger.entries.is_empty());
        ledger.capture(7, 5, vec![ability_effect(5, 0)]);
        let mut new_disclosure = snapshot(5);
        new_disclosure.entities[0].disclosure_epoch = 4;
        ledger.project(&mut new_disclosure);
        assert!(ledger.entries.is_empty() && new_disclosure.effects.is_empty());
        ledger.capture(7, 6, vec![effect(6, 0), ability_effect(6, 1)]);
        assert_eq!(ledger.baseline(5), 6);
        assert!(ledger.entries.is_empty());
        ledger.capture(7, 7, (0..MAX_CUES as u32 + 1).map(|i|
            if i % 2 == 0 {effect(7, i)} else {ability_effect(7, i)}).collect());
        assert_eq!(ledger.entries.len(), MAX_CUES);
        assert_eq!(ledger.dropped, 1);
    }

    #[test]
    fn unpublished_steps_survive_until_sent_and_consumed_on_matching_connection() {
        let mut ledger = CueRetention::default();
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
        let mut ledger = CueRetention::default();
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
        let mut ledger = CueRetention::default();
        ledger.capture(
            7,
            2,
            (0..MAX_CUES as u32 + 1).map(|i| effect(2, i)).collect(),
        );
        assert_eq!(ledger.entries.len(), MAX_CUES);
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
