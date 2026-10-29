//! Host-owned pre-match selection. No World, Lua VM, renderer or sockets.
use super::bots::{RoleBotMatchPlan, RoleBotPlayerPlan};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

/// Same compiled catalog as admission, playable (source) heroes only; no UI-side whitelist.
/// Test-only heroes stay compiled for fixtures but are never offered.
pub fn hero_selection_catalog() -> Vec<HeroSelectionOption> {
    use omoba_template_ids as ids;
    ids::HERO_PLAYABLE_IDS
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

/// The playable hero a seat gets when nothing selected one: the playable catalog in order,
/// cycling by seat. Never a test-only hero.
pub fn default_playable_hero(seat: usize) -> &'static str {
    let playable = omoba_template_ids::HERO_PLAYABLE_IDS;
    assert!(!playable.is_empty(), "the compiled catalog has no playable source hero");
    playable[seat % playable.len()].as_str()
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
    /// Host-installed trusted slots. Client Select and Lock never write this map.
    #[serde(default)]
    pub summoner_loadouts: BTreeMap<u32, [Option<String>; 2]>,
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
    summoner_loadouts: BTreeMap<u32, [Option<String>; 2]>,
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
        let summoner_loadouts = plan.summoner_loadouts.clone();
        Ok(Self {
            plan,
            locked,
            revision: 0,
            finalized: false,
            seed,
            tick_rate_hz,
            summoner_loadouts,
        })
    }

    /// Host authentication only. A rejected map leaves the plan and retained map unchanged.
    /// An identical map does not publish a new revision. A real change increments the
    /// revision before any later lock can consent, and a locked human seat refuses the change.
    pub fn install_trusted_loadouts(
        &mut self,
        loadouts: BTreeMap<u32, [Option<String>; 2]>,
    ) -> Result<(), String> {
        if self.finalized {
            return Err("hero selection already finalized".into());
        }
        let players = self.plan.players.iter().map(|player| player.player_id);
        crate::runtime::native::summoner_world::validated_loadouts(players, &loadouts)
            .map_err(|error| format!("trusted summoner loadout rejected: {error:?}"))?;
        if loadouts == self.summoner_loadouts && loadouts == self.plan.summoner_loadouts {
            return Ok(());
        }
        if self.plan.players.iter().any(|player| !player.bot && self.locked.contains(&player.player_id)) {
            return Err("trusted summoner loadouts cannot change after a human seat locks".into());
        }
        let next_revision = self.revision.checked_add(1).ok_or("hero selection revision exhausted")?;
        self.plan.summoner_loadouts = loadouts.clone();
        self.summoner_loadouts = loadouts;
        self.revision = next_revision;
        Ok(())
    }

    pub fn trusted_loadouts(&self) -> &BTreeMap<u32, [Option<String>; 2]> {
        &self.summoner_loadouts
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
            summoner_loadouts: self.summoner_loadouts.clone(),
        }
    }

    /// Read-only distribution to admitted seats, not the host's one-time launch handoff.
    fn finalized_plan_snapshot(&self) -> Option<RoleBotMatchPlan> {
        self.finalized.then(|| self.plan.clone())
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
                let team = candidate.players[index].team_id;
                if candidate.players.iter().any(|p| !p.bot && p.player_id != admitted_player_id && p.team_id == team && p.hero == hero) {
                    return Err(format!("hero '{hero}' is already picked by a teammate"));
                }
                candidate.players[index].hero = hero;
                // A bot teammate holding the pick moves to a hero its team does not use.
                candidate.resolve_bot_hero_conflicts();
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
            summoner_loadouts: BTreeMap::new(),
            players: vec![
                RoleBotPlayerPlan {
                    player_id: 7,
                    team_id: 1,
                    hero: "murakami_yoshikiyo".into(),
                    role: BotRole::Top,
                    lane: "top".into(),
                    bot: false,
                },
                RoleBotPlayerPlan {
                    player_id: 8,
                    team_id: 2,
                    hero: "murakami_yoshikiyo".into(),
                    role: BotRole::Top,
                    lane: "top".into(),
                    bot: false,
                },
                RoleBotPlayerPlan {
                    player_id: 9,
                    team_id: 2,
                    hero: "saika_magoichi".into(),
                    role: BotRole::Mid,
                    lane: "mid".into(),
                    bot: true,
                },
            ], showcase: None
        }
    }
    fn request(revision: u64, action: HeroSelectionAction) -> HeroSelectionRequest {
        HeroSelectionRequest {
            expected_revision: revision,
            action,
        }
    }

    #[test]
    fn hero_selection_keeps_one_hero_per_team_and_moves_the_clashing_bot() {
        // Team 2: human 8 (murakami) and bot 9 (saika). Team 1: human 7 (murakami).
        let mut session = HeroSelectionSession::new(plan(), 1, 60).unwrap();
        // Human 8 picks the bot's hero: the bot moves to the first playable hero team 2 lacks.
        let snapshot = session.apply(8, request(0, HeroSelectionAction::Select { hero: "saika_magoichi".into() })).unwrap();
        let heroes = |s: &HeroSelectionSnapshot, id: u32| s.seats.iter().find(|seat| seat.player.player_id == id).unwrap().player.hero.clone();
        assert_eq!(heroes(&snapshot, 8), "saika_magoichi");
        let moved = heroes(&snapshot, 9);
        let expected = omoba_template_ids::HERO_PLAYABLE_IDS.iter().map(|id| id.as_str())
            .find(|hero| *hero != "saika_magoichi").unwrap();
        assert_eq!(moved, expected, "first playable hero the team does not use");
        // The other team may pick the same hero (cross-team clashes are allowed).
        let snapshot = session.apply(7, request(1, HeroSelectionAction::Select { hero: "saika_magoichi".into() })).unwrap();
        assert_eq!(heroes(&snapshot, 7), "saika_magoichi");
        // A human teammate's pick is refused without mutation.
        let mut two = plan();
        two.players[2].bot = false;
        let mut session = HeroSelectionSession::new(two, 1, 60).unwrap();
        assert!(session.apply(8, request(0, HeroSelectionAction::Select { hero: "saika_magoichi".into() }))
            .unwrap_err().contains("teammate"));
        assert_eq!(session.snapshot().revision, 0);
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
                        hero: "tamako".into(),
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
        assert_eq!(finalized.players[0].hero, "tamako");
        assert_eq!(original.players[0].hero, "murakami_yoshikiyo");
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
                        hero: "tamako".into()
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
        assert!(catalog.iter().any(|hero| hero.hero == "tamako"));
        // Only source heroes are offered; the training fixtures are compiled but never listed.
        assert!(catalog.iter().all(|hero| omoba_template_ids::hero_playable(omoba_template_ids::hero_by_name(&hero.hero).unwrap())));
        assert!(!catalog.iter().any(|hero| hero.hero.starts_with("training_")));
        for seat in 0..20 {
            assert!(omoba_template_ids::hero_playable(omoba_template_ids::hero_by_name(default_playable_hero(seat)).unwrap()));
        }
        let mut session = HeroSelectionSession::new(plan(), 1, 60).unwrap();
        assert!(session.apply(7, request(0, HeroSelectionAction::Select { hero: "training_ranger".into() })).is_err(),
            "a test-only hero cannot be selected");
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

    #[test]
    fn trusted_summoner_loadouts_reject_unknown_players_and_ignore_client_select() {
        let mut session = HeroSelectionSession::new(plan(), 1, 60).unwrap();
        assert!(session.install_trusted_loadouts(BTreeMap::from([(99, [None, None])])).unwrap_err().contains("WrongOwner"));
        assert!(session.trusted_loadouts().is_empty());
        let good = BTreeMap::from([(7, [Some("training_self_heal_v1".into()), None])]);
        let revision = session.snapshot().revision;
        session.install_trusted_loadouts(good.clone()).unwrap();
        assert_eq!(session.snapshot().revision, revision + 1);
        session.install_trusted_loadouts(good.clone()).unwrap();
        assert_eq!(session.snapshot().revision, revision + 1);
        assert!(serde_json::from_str::<HeroSelectionRequest>(
            r#"{"expected_revision":1,"action":{"kind":"select","hero":"tamako","summoner_loadouts":{"7":[null,null]}}}"#
        ).is_err());
        session.apply(7, request(revision + 1, HeroSelectionAction::Select { hero: "tamako".into() })).unwrap();
        assert_eq!(session.snapshot().summoner_loadouts, good);
        assert_eq!(session.plan.summoner_loadouts, good);
        assert_eq!(session.snapshot().seats[0].player.hero, "tamako");
        session.apply(7, request(revision + 2, HeroSelectionAction::Lock)).unwrap();
        assert!(session.install_trusted_loadouts(BTreeMap::new()).unwrap_err().contains("human seat locks"));
        assert_eq!(session.snapshot().summoner_loadouts, good);
        session.apply(8, request(revision + 3, HeroSelectionAction::Lock)).unwrap();
        let finalized = session.finalize(revision + 4).unwrap();
        assert_eq!(finalized.summoner_loadouts, good);
        assert_eq!(session.snapshot().summoner_loadouts, good);
        assert_eq!(session.snapshot().catalog_data_hash, omoba_template_ids::CONTENT_CATALOG_DATA_HASH);
        assert!(session.install_trusted_loadouts(BTreeMap::new()).is_err());
        assert_eq!(session.snapshot().summoner_loadouts, good);
    }

    fn lock_plan(plan: RoleBotMatchPlan) -> (HeroSelectionSnapshot, RoleBotMatchPlan) {
        let mut session = HeroSelectionSession::new(plan, 0, 60).unwrap();
        let humans: Vec<_> = session.snapshot().seats.iter()
            .filter(|seat| !seat.player.bot).map(|seat| seat.player.player_id).collect();
        let mut revision = session.snapshot().revision;
        for player_id in humans {
            session.apply(player_id, request(revision, HeroSelectionAction::Lock)).unwrap();
            revision += 1;
        }
        let finalized = session.finalize(revision).unwrap();
        (session.snapshot(), finalized)
    }

    #[test]
    fn host_plan_loadouts_round_trip_through_finalize_and_second_lock() {
        let cases = [
            BTreeMap::new(),
            BTreeMap::from([(7, [Some("training_point_blink_v1".into()), None])]),
            BTreeMap::from([(8, [None, Some("training_self_heal_v1".into())])]),
            BTreeMap::from([(7, [Some("training_point_blink_v1".into()), Some("training_self_heal_v1".into())])]),
        ];
        for loadouts in cases {
            let mut original = plan();
            original.summoner_loadouts = loadouts.clone();
            let (snapshot, finalized) = lock_plan(original);
            assert_eq!(snapshot.summoner_loadouts, loadouts);
            assert_eq!(finalized.summoner_loadouts, loadouts);
            assert_eq!(snapshot.catalog_data_hash, omoba_template_ids::CONTENT_CATALOG_DATA_HASH);
            let encoded = serde_json::to_string(&finalized).unwrap();
            if loadouts.is_empty() { assert!(!encoded.contains("summoner_loadouts")); }
            let (again_snapshot, again) = lock_plan(serde_json::from_str(&encoded).unwrap());
            assert_eq!(again.summoner_loadouts, loadouts);
            assert_eq!(again_snapshot.summoner_loadouts, loadouts);
            assert_eq!(again_snapshot.catalog_data_hash, snapshot.catalog_data_hash);
        }
    }

    #[test]
    fn host_plan_loadouts_reject_unknown_catalog_duplicate_and_bad_rows_before_session() {
        let mut unknown_player = plan();
        unknown_player.summoner_loadouts.insert(99, [None, None]);
        assert!(HeroSelectionSession::new(unknown_player, 0, 60).err().expect("unknown player must reject").contains("WrongOwner"));
        let mut unknown_spell = plan();
        unknown_spell.summoner_loadouts.insert(7, [Some("no-such-spell".into()), None]);
        assert!(HeroSelectionSession::new(unknown_spell, 0, 60).err().expect("unknown spell must reject").contains("UnknownSpell"));
        let mut duplicate = plan();
        duplicate.summoner_loadouts.insert(7, [Some("training_self_heal_v1".into()), Some("training_self_heal_v1".into())]);
        assert!(HeroSelectionSession::new(duplicate, 0, 60).err().expect("duplicate spell must reject").contains("DuplicateSpell"));
        let mut raw = serde_json::to_value(plan()).unwrap();
        raw["summoner_loadouts"] = serde_json::json!({"7": ["training_self_heal_v1"]});
        assert!(serde_json::from_value::<RoleBotMatchPlan>(raw).is_err());
    }
}
