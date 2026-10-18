//! Durable renderer-request to authority-input map. This is not a shop undo
//! receipt and it does not rewrite a sealed owner outcome.

use std::collections::VecDeque;

use omoba_core::runtime::ward_transport::{
    accept_ward_correlations, ward_outcome_matches, WardCorrelationDraft, WARD_CORRELATION_CAPACITY,
    WARD_PHASE_APPLIED, WARD_PHASE_FORWARDED, WARD_PHASE_REJECTED, WARD_PHASE_UNCERTAIN,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    draft: WardCorrelationDraft,
}

/// Bounded by owner epoch. A newer epoch drops the previous rows. Eviction is
/// oldest-first and does not allocate a replacement placement.
#[derive(Clone, Debug, Default)]
pub struct WardCorrelationTable {
    epoch: Option<u64>,
    rows: VecDeque<Row>,
}

impl WardCorrelationTable {
    pub fn note(
        &mut self,
        epoch: u64,
        owner: u32,
        actor_generation: u32,
        authority_input: u32,
        renderer_request: u64,
        phase: u32,
    ) -> bool {
        if epoch == 0 || owner == 0 || renderer_request == 0 {
            return false;
        }
        let draft = WardCorrelationDraft {
            player_id: owner,
            view_epoch: epoch,
            actor_generation,
            renderer_request,
            authority_input,
            phase,
        };
        if omoba_core::runtime::ward_transport::ward_correlation_row_ok(owner, epoch, &draft) {
            // checked below after epoch adoption
        } else {
            return false;
        }
        self.adopt(epoch);
        if let Some(existing) = self.rows.iter_mut().find(|row| {
            row.draft.player_id == owner && row.draft.renderer_request == renderer_request
        }) {
            if existing.draft.authority_input != 0
                && authority_input != 0
                && existing.draft.authority_input != authority_input
            {
                return false;
            }
            if existing.draft.actor_generation == 0 && actor_generation != 0 {
                existing.draft.actor_generation = actor_generation;
            }
            let current = existing.draft.phase;
            let advance = matches!(
                (current, phase),
                (WARD_PHASE_FORWARDED, WARD_PHASE_APPLIED | WARD_PHASE_UNCERTAIN | WARD_PHASE_REJECTED)
                    | (WARD_PHASE_APPLIED, WARD_PHASE_UNCERTAIN)
                    | (WARD_PHASE_UNCERTAIN, WARD_PHASE_UNCERTAIN)
            );
            if advance {
                existing.draft.phase = phase;
            }
            if existing.draft.authority_input == 0 && authority_input != 0 && existing.draft.phase != WARD_PHASE_REJECTED {
                existing.draft.authority_input = authority_input;
            }
            return true;
        }
        if self.rows.len() == WARD_CORRELATION_CAPACITY {
            self.rows.pop_front();
        }
        self.rows.push_back(Row { draft });
        true
    }

    pub fn replace_all(&mut self, epoch: u64, rows: Vec<WardCorrelationDraft>) {
        self.rows.clear();
        self.epoch = Some(epoch);
        for row in rows {
            if self.rows.len() == WARD_CORRELATION_CAPACITY {
                break;
            }
            if omoba_core::runtime::ward_transport::ward_correlation_row_ok(row.player_id, epoch, &row) {
                self.rows.push_back(Row { draft: row });
            } else {
                self.rows.clear();
                return;
            }
        }
    }

    pub fn project(&self, owner: u32, epoch: u64) -> Vec<omoba_core::game_proto::WardInputCorrelation> {
        if self.epoch != Some(epoch) {
            return Vec::new();
        }
        let drafts: Vec<_> = self.rows.iter().map(|row| row.draft).filter(|row| row.player_id == owner).collect();
        let Some(accepted) = accept_ward_correlations(owner, epoch, &drafts) else {
            return Vec::new();
        };
        accepted.into_iter().map(|row| omoba_core::game_proto::WardInputCorrelation {
            player_id: row.player_id,
            view_epoch: row.view_epoch,
            actor_generation: row.actor_generation,
            renderer_request: row.renderer_request,
            authority_input: row.authority_input,
            phase: row.phase,
        }).collect()
    }

    pub fn authority_for(&self, owner: u32, epoch: u64, renderer_request: u64) -> Option<u32> {
        if self.epoch != Some(epoch) {
            return None;
        }
        self.rows.iter().find(|row| {
            row.draft.player_id == owner && row.draft.view_epoch == epoch && row.draft.renderer_request == renderer_request
        }).map(|row| row.draft.authority_input).filter(|authority| *authority != 0)
    }

    pub fn settles(
        &self,
        owner: u32,
        epoch: u64,
        renderer_request: u64,
        outcome_correlation: u32,
        outcome_actor: u32,
    ) -> bool {
        if self.epoch != Some(epoch) {
            return false;
        }
        self.rows.iter().any(|row| {
            row.draft.player_id == owner
                && row.draft.renderer_request == renderer_request
                && ward_outcome_matches(&row.draft, outcome_correlation, outcome_actor)
        })
    }

    fn adopt(&mut self, epoch: u64) {
        if self.epoch != Some(epoch) {
            self.rows.clear();
            self.epoch = Some(epoch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(table: &mut WardCorrelationTable, epoch: u64, owner: u32, actor: u32, authority: u32, request: u64, phase: u32) {
        assert!(table.note(epoch, owner, actor, authority, request, phase));
    }

    #[test]
    fn renderer_request_1_maps_to_authority_901_and_survives_replacement() {
        let mut table = WardCorrelationTable::default();
        note(&mut table, 4, 7, 3, 901, 1, WARD_PHASE_FORWARDED);
        assert_eq!(table.authority_for(7, 4, 1), Some(901));
        assert!(table.settles(7, 4, 1, 901, 3));
        assert!(!table.settles(7, 4, 1, 901, 9), "a different actor does not settle");
        assert!(!table.settles(8, 4, 1, 901, 3), "a foreign owner does not settle");
        assert!(!table.settles(7, 3, 1, 901, 3), "an old epoch does not settle");
        note(&mut table, 4, 7, 3, 902, 2, WARD_PHASE_FORWARDED);
        assert!(!table.settles(7, 4, 2, 901, 3), "the older authority does not settle the replacement");
        assert!(table.settles(7, 4, 2, 902, 3));
        assert!(!table.note(4, 7, 9, 909, 1, WARD_PHASE_FORWARDED), "a duplicate request cannot take a new authority id");
        assert_eq!(table.authority_for(7, 4, 1), Some(901));
        assert!(table.note(4, 7, 0, 901, 1, WARD_PHASE_UNCERTAIN));
        assert!(table.settles(7, 4, 1, 901, 3), "late settlement still matches the original authority");
        let projected = table.project(7, 4);
        assert_eq!(projected[0].renderer_request, 1);
        assert_eq!(projected[0].authority_input, 901);
        assert_ne!(projected[0].renderer_request, u64::from(projected[0].authority_input));
        assert!(table.project(7, 3).is_empty());
    }

    #[test]
    fn uncertain_then_local_rejection_and_bounded_eviction_keep_order() {
        let mut table = WardCorrelationTable::default();
        assert!(table.note(4, 7, 0, 901, 1, WARD_PHASE_UNCERTAIN));
        assert_eq!(table.project(7, 4)[0].phase, WARD_PHASE_UNCERTAIN);
        assert!(!table.settles(7, 4, 1, 7, 0), "a renderer id is not the authority correlation");
        assert!(table.note(4, 7, 3, 901, 1, WARD_PHASE_UNCERTAIN));
        assert!(table.settles(7, 4, 1, 901, 3), "the same authority settles after the uncertain phase");
        assert!(table.note(4, 7, 0, 0, 11, WARD_PHASE_REJECTED));
        assert!(!table.settles(7, 4, 11, 0, 0));
        assert_eq!(table.project(7, 4).iter().find(|row| row.renderer_request == 11).unwrap().phase, WARD_PHASE_REJECTED);
        let mut filled = WardCorrelationTable::default();
        for index in 1..=8 {
            note(&mut filled, 4, 7, 0, 900 + index, u64::from(index), WARD_PHASE_FORWARDED);
        }
        note(&mut filled, 4, 7, 0, 909, 9, WARD_PHASE_FORWARDED);
        assert!(filled.authority_for(7, 4, 1).is_none(), "the oldest correlation is evicted");
        assert_eq!(filled.authority_for(7, 4, 2), Some(902));
        assert_eq!(filled.authority_for(7, 4, 9), Some(909));
        let order: Vec<_> = filled.project(7, 4).into_iter().map(|row| row.renderer_request).collect();
        assert_eq!(order, vec![2, 3, 4, 5, 6, 7, 8, 9]);
        assert!(!filled.settles(7, 4, 1, 901, 0));
    }
}
