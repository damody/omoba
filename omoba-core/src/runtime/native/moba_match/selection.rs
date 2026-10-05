//! Host-owned pre-match selection. No World, Lua VM, renderer or sockets.
use super::bots::{RoleBotMatchPlan, RoleBotPlayerPlan};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub mod service;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSelectionOption {
    pub catalog_id: u16,
    pub hero: String,
    pub display_name: String,
    pub title: String,
    pub portrait: String,
    pub abilities: Vec<String>,
}

/// Same compiled active catalog as admission; no UI-side hero whitelist.
pub fn hero_selection_catalog() -> Vec<HeroSelectionOption> {
    use omoba_template_ids as ids;
    ids::HERO_CATALOG_IDS
        .iter()
        .copied()
        .filter(|id| ids::hero_stats(*id).is_some())
        .map(|id| HeroSelectionOption {
            catalog_id: id.raw(),
            hero: id.as_str().into(),
            display_name: ids::hero_display(id).into(),
            title: ids::hero_title(id).into(),
            portrait: ids::hero_portrait(id).into(),
            abilities: ids::hero_abilities(id)
                .iter()
                .map(|ability| ability.as_str().into())
                .collect(),
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeroSelectionAction {
    Select { hero: String },
    Lock,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSelectionRequest {
    pub expected_revision: u64,
    pub action: HeroSelectionAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSelectionSeat {
    pub player: RoleBotPlayerPlan,
    pub locked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroSelectionSnapshot {
    pub schema_version: u32,
    pub catalog_data_hash: String,
    pub revision: u64,
    pub ready: bool,
    pub finalized: bool,
    pub seats: Vec<HeroSelectionSeat>,
}

/// This is a rules kernel, NOT authentication. The host must obtain the player
/// ID from its admitted connection; requests deliberately contain no player ID.
pub struct HeroSelectionSession {
    plan: RoleBotMatchPlan,
    locked: BTreeSet<u32>,
    revision: u64,
    finalized: bool,
    seed: u64,
    tick_rate_hz: u32,
}

impl HeroSelectionSession {
    pub fn new(mut plan: RoleBotMatchPlan, seed: u64, tick_rate_hz: u32) -> Result<Self, String> {
        plan.compile_with_tick_rate(seed, tick_rate_hz)?;
        plan.players.sort_by_key(|p| p.player_id);
        let locked = plan
            .players
            .iter()
            .filter(|p| p.bot)
            .map(|p| p.player_id)
            .collect();
        Ok(Self {
            plan,
            locked,
            revision: 0,
            finalized: false,
            seed,
            tick_rate_hz,
        })
    }

    pub fn snapshot(&self) -> HeroSelectionSnapshot {
        HeroSelectionSnapshot {
            schema_version: 1,
            catalog_data_hash: omoba_template_ids::CONTENT_CATALOG_DATA_HASH.to_string(),
            revision: self.revision,
            ready: self
                .plan
                .players
                .iter()
                .all(|p| self.locked.contains(&p.player_id)),
            finalized: self.finalized,
            seats: self
                .plan
                .players
                .iter()
                .map(|p| HeroSelectionSeat {
                    player: p.clone(),
                    locked: self.locked.contains(&p.player_id),
                })
                .collect(),
        }
    }

    pub fn apply(
        &mut self,
        admitted_player_id: u32,
        request: HeroSelectionRequest,
    ) -> Result<HeroSelectionSnapshot, String> {
        if self.finalized {
            return Err("hero selection already finalized".into());
        }
        if request.expected_revision != self.revision {
            return Err("stale hero selection revision".into());
        }
        let index = self
            .plan
            .players
            .iter()
            .position(|p| p.player_id == admitted_player_id)
            .ok_or("hero selection player is not admitted in roster")?;
        if self.plan.players[index].bot {
            return Err("human selection cannot control a bot seat".into());
        }
        if self.locked.contains(&admitted_player_id) {
            return Err("hero selection seat is locked".into());
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or("hero selection revision exhausted")?;
        match request.action {
            HeroSelectionAction::Select { hero } => {
                let mut candidate = self.plan.clone();
                candidate.players[index].hero = hero;
                // Same production plan/catalog validator; reject without mutation.
                candidate.compile_with_tick_rate(self.seed, self.tick_rate_hz)?;
                self.plan = candidate;
            }
            HeroSelectionAction::Lock => {
                self.locked.insert(admitted_player_id);
            }
        }
        self.revision = next_revision;
        Ok(self.snapshot())
    }

    pub fn finalize(&mut self, expected_revision: u64) -> Result<RoleBotMatchPlan, String> {
        if self.finalized {
            return Err("hero selection already finalized".into());
        }
        if expected_revision != self.revision {
            return Err("stale hero selection revision".into());
        }
        if !self
            .plan
            .players
            .iter()
            .all(|p| self.locked.contains(&p.player_id))
        {
            return Err("all human seats must lock before match creation".into());
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or("hero selection revision exhausted")?;
        self.plan
            .compile_with_tick_rate(self.seed, self.tick_rate_hz)?;
        self.finalized = true;
        self.revision = next_revision;
        Ok(self.plan.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::super::bots::BotRole;
    use super::*;

    pub(super) fn plan() -> RoleBotMatchPlan {
        RoleBotMatchPlan {
            schema_version: 1,
            map_id: "three_lane_training".into(),
            think_hz: 5,
            mana_enabled: false,
            ability_policies: vec![],
            ability_learning: vec![],
            sustain: None,
            item_builds: vec![],
            players: vec![
                RoleBotPlayerPlan {
                    player_id: 7,
                    team_id: 1,
                    hero: "training_luminary".into(),
                    role: BotRole::Top,
                    lane: "top".into(),
                    bot: false,
                },
                RoleBotPlayerPlan {
                    player_id: 8,
                    team_id: 2,
                    hero: "training_luminary".into(),
                    role: BotRole::Top,
                    lane: "top".into(),
                    bot: false,
                },
                RoleBotPlayerPlan {
                    player_id: 9,
                    team_id: 2,
                    hero: "training_luminary".into(),
                    role: BotRole::Mid,
                    lane: "mid".into(),
                    bot: true,
                },
            ],
        }
    }
    fn request(revision: u64, action: HeroSelectionAction) -> HeroSelectionRequest {
        HeroSelectionRequest {
            expected_revision: revision,
            action,
        }
    }

    #[test]
    fn hero_selection_select_lock_finalize_preserves_roster_and_rules() {
        let original = plan();
        let mut session = HeroSelectionSession::new(original.clone(), 42, 60).unwrap();
        assert!(session.snapshot().seats[2].locked);
        assert!(session.finalize(0).is_err());
        session
            .apply(
                7,
                request(
                    0,
                    HeroSelectionAction::Select {
                        hero: "training_ranger".into(),
                    },
                ),
            )
            .unwrap();
        session
            .apply(7, request(1, HeroSelectionAction::Lock))
            .unwrap();
        assert!(!session.snapshot().ready);
        session
            .apply(8, request(2, HeroSelectionAction::Lock))
            .unwrap();
        assert!(session.snapshot().ready);
        let finalized = session.finalize(3).unwrap();
        assert_eq!(finalized.players[0].hero, "training_ranger");
        assert_eq!(original.players[0].hero, "training_luminary");
        assert_eq!(finalized.players[0].team_id, 1);
        assert_eq!(finalized.players[0].role, BotRole::Top);
        assert!(finalized.players[2].bot);
        assert_eq!(finalized.think_hz, original.think_hz);
        assert!(session.snapshot().finalized && session.snapshot().revision == 4);
        assert!(session.finalize(4).is_err());
        assert!(session
            .apply(8, request(4, HeroSelectionAction::Lock))
            .is_err());
        let (_, bots) = finalized.compile_with_tick_rate(42, 60).unwrap();
        assert_eq!(bots.assignments.len(), 1);
    }

    #[test]
    fn hero_selection_rejection_is_atomic_and_requires_host_identity() {
        let mut session = HeroSelectionSession::new(plan(), 1, 60).unwrap();
        let initial = serde_json::to_string(&session.snapshot()).unwrap();
        for (id, revision, action) in [
            (0, 0, HeroSelectionAction::Lock),
            (9, 0, HeroSelectionAction::Lock),
            (7, 1, HeroSelectionAction::Lock),
            (
                7,
                0,
                HeroSelectionAction::Select {
                    hero: "missing".into(),
                },
            ),
        ] {
            assert!(session.apply(id, request(revision, action)).is_err());
            assert_eq!(serde_json::to_string(&session.snapshot()).unwrap(), initial);
        }
        assert!(serde_json::from_str::<HeroSelectionRequest>(
            r#"{"expected_revision":0,"player_id":8,"action":{"kind":"lock"}}"#
        )
        .is_err());
        session
            .apply(7, request(0, HeroSelectionAction::Lock))
            .unwrap();
        assert!(session
            .apply(
                7,
                request(
                    1,
                    HeroSelectionAction::Select {
                        hero: "training_ranger".into()
                    }
                )
            )
            .is_err());
        assert!(!session.snapshot().seats[1].locked);
        session.revision = u64::MAX;
        assert!(session
            .apply(8, request(u64::MAX, HeroSelectionAction::Lock))
            .is_err());
        assert!(!session.snapshot().seats[1].locked);
    }

    #[test]
    fn hero_selection_all_bot_and_declaration_order_are_deterministic() {
        let mut first = plan();
        for p in &mut first.players {
            p.bot = true;
        }
        let mut reversed = first.clone();
        reversed.players.reverse();
        let mut left = HeroSelectionSession::new(first, 1, 60).unwrap();
        let mut right = HeroSelectionSession::new(reversed, 1, 60).unwrap();
        assert_eq!(
            serde_json::to_string(&left.snapshot()).unwrap(),
            serde_json::to_string(&right.snapshot()).unwrap()
        );
        assert!(left.snapshot().ready);
        assert_eq!(
            serde_json::to_string(&left.finalize(0).unwrap()).unwrap(),
            serde_json::to_string(&right.finalize(0).unwrap()).unwrap()
        );
    }

    #[test]
    fn hero_selection_catalog_is_generated_and_every_option_passes_admission() {
        let catalog = hero_selection_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog
            .windows(2)
            .all(|pair| pair[0].catalog_id < pair[1].catalog_id));
        assert!(catalog.iter().any(|hero| hero.hero == "training_ranger"));
        for hero in catalog {
            let mut session = HeroSelectionSession::new(plan(), 1, 60).unwrap();
            session
                .apply(
                    7,
                    request(
                        0,
                        HeroSelectionAction::Select {
                            hero: hero.hero.clone(),
                        },
                    ),
                )
                .unwrap();
            assert_eq!(session.snapshot().seats[0].player.hero, hero.hero);
            assert_eq!(
                omoba_template_ids::hero_by_name(&hero.hero).unwrap().raw(),
                hero.catalog_id
            );
            for ability in hero.abilities {
                assert!(omoba_template_ids::ability_by_name(&ability).is_some());
            }
        }
    }
}
