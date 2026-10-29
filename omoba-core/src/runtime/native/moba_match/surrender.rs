//! Team surrender vote (E187), the original game's chat "-ff"
//! (history_nobu vscripts/computer_system/surrender.lua). Numbers come from
//! templates/game_rules.lua `surrender`; a match without them has no surrender.
//!
//! The server calls [`toggle_surrender_vote`] when an authenticated player asks
//! to surrender. A passed vote only records the conceding side; the next
//! `finish_moba_match_tick` ends the match through the same finish path as a
//! destroyed base (phase, terminal fact, `game.end`, ward cleanup).
//!
//! "Connected" (surrender.lua:76-88 counts the voting team's heroes whose
//! connection state is connected): every seated roster player of the team,
//! except the seats the server marked absent with [`set_surrender_voter_absent`]
//! (bot seats, which never vote, and humans that dropped). Like the source, the
//! count is taken only when a vote is cast (:48, :71-113).
use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SurrenderState {
    /// Players whose vote stands. surrender.lua:40-46: a first "-ff" votes, every
    /// later one flips it; no timeout or cooldown.
    votes: BTreeSet<u32>,
    /// Seated players the server excludes from the count (bots, dropped humans).
    absent: BTreeSet<u32>,
    /// Lane side whose vote passed; the finish tick turns it into the other side's win.
    conceded: Option<u8>,
}

/// Why a vote was not taken. The server tells only the typer (the source broadcast
/// the "too early" line to everyone, surrender.lua:49-57).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurrenderRejection {
    /// The match has no `game_rules.surrender`.
    Disabled,
    /// Warmup, finished, or a surrender already passed (surrender.lua:21, :110-113).
    NotPlaying,
    /// Game time is stopped; the vote would be decided on a frozen match.
    Paused,
    /// The player has no seat in the match roster.
    NotSeated,
    /// The server marked this seat absent (a bot or a dropped player).
    Absent,
    /// Fewer than `available_after_seconds` of playing time (surrender.lua:36-58).
    TooEarly { remaining: Fixed64 },
}

impl SurrenderRejection {
    /// Whole seconds until votes open, rounded up like surrender.lua:37 math.ceil.
    pub fn remaining_seconds(&self) -> Option<u32> {
        let Self::TooEarly { remaining } = self else { return None };
        let scale = omoba_sim::fixed::SCALE;
        Some(((remaining.raw().max(0) + scale - 1) / scale) as u32)
    }
}

/// One team's standing vote: surrender.lua:94 "同意:%d 門檻:%d".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurrenderTally {
    pub side: u8,
    /// Authenticated wire team id of `side`.
    pub team_id: u32,
    pub agree: u32,
    /// Counted (seated, not absent) players of the team.
    pub counted: u32,
    pub threshold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurrenderVote {
    pub player_id: u32,
    /// The player's vote after this toggle.
    pub voted: bool,
    pub tally: SurrenderTally,
    /// The team reached the threshold; the match ends on the next finish tick.
    pub conceded: bool,
}

fn rules(state: &MobaMatch) -> Option<omoba_template_ids::SurrenderRulesConst> {
    state.config.game_rules.and_then(|rules| rules.surrender)
}

/// Playing time still needed before votes open; zero once they are open.
/// `elapsed` stops while paused, matching the source's game-time timer (surrender.lua:117-123).
fn remaining(state: &MobaMatch, rules: omoba_template_ids::SurrenderRulesConst) -> Fixed64 {
    let playing = (state.elapsed - state.config.warmup).max(Fixed64::ZERO);
    (Fixed64::from_i32(rules.available_after_seconds.min(i32::MAX as u32) as i32) - playing).max(Fixed64::ZERO)
}

fn tally(state: &MobaMatch, rules: omoba_template_ids::SurrenderRulesConst, side: usize) -> SurrenderTally {
    let surrender = &state.surrender;
    let counted: Vec<u32> = state.heroes.iter()
        .filter(|slot| slot.side == side && !surrender.absent.contains(&slot.player_id))
        .map(|slot| slot.player_id).collect();
    let agree = counted.iter().filter(|id| surrender.votes.contains(id)).count() as u32;
    SurrenderTally {
        side: side as u8,
        team_id: state.config.teams[side],
        agree,
        counted: counted.len() as u32,
        threshold: rules.votes_needed(counted.len() as u32),
    }
}

fn seat(state: &MobaMatch, player_id: u32) -> Result<usize, SurrenderRejection> {
    let mut seats = state.heroes.iter().filter(|slot| slot.player_id == player_id);
    let side = seats.next().map(|slot| slot.side).ok_or(SurrenderRejection::NotSeated)?;
    if seats.next().is_some() { return Err(SurrenderRejection::NotSeated); }
    Ok(side)
}

/// Toggle `player_id`'s surrender vote (surrender.lua:20-59) and decide its team's
/// vote (:71-113). Only while playing, unpaused, and after `available_after_seconds`.
pub fn toggle_surrender_vote(world: &mut World, player_id: u32) -> Result<SurrenderVote, SurrenderRejection> {
    let paused = world.try_fetch::<GamePause>().is_some_and(|pause| pause.is_paused);
    let tick = world.try_fetch::<Tick>().map_or(0, |tick| tick.0);
    let vote = {
        let mut state = world.write_resource::<MobaMatch>();
        let rules = rules(&state).ok_or(SurrenderRejection::Disabled)?;
        let side = seat(&state, player_id)?;
        if state.phase != MobaMatchPhase::Playing || state.surrender.conceded.is_some() {
            return Err(SurrenderRejection::NotPlaying);
        }
        if paused { return Err(SurrenderRejection::Paused); }
        if state.surrender.absent.contains(&player_id) { return Err(SurrenderRejection::Absent); }
        let remaining = remaining(&state, rules);
        if remaining > Fixed64::ZERO { return Err(SurrenderRejection::TooEarly { remaining }); }
        let voted = state.surrender.votes.insert(player_id) || !state.surrender.votes.remove(&player_id);
        let tally = tally(&state, rules, side);
        let conceded = tally.agree >= tally.threshold;
        if conceded {
            state.surrender.conceded = Some(side as u8);
            log::info!("[match] surrender passed tick={} side={} agree={}/{}", tick, side, tally.agree, tally.threshold);
        }
        SurrenderVote { player_id, voted, tally, conceded }
    };
    // surrender.lua:94-108 announces every vote to both teams.
    if let Some(mut events) = world.try_fetch_mut::<crate::runtime::RuntimeEvents>() {
        events.push(crate::runtime::RuntimeEvent::new("surrender.vote", "surrender", "vote", serde_json::json!({
            "player_id": vote.player_id, "voted": vote.voted, "team": vote.tally.team_id,
            "agree": vote.tally.agree, "threshold": vote.tally.threshold, "conceded": vote.conceded, "tick": tick,
        })).with_broadcast(crate::runtime::RuntimeBroadcast::All));
    }
    Ok(vote)
}

/// Exclude (or restore) a seated player from the count: bot seats at match start,
/// a human on disconnect/reconnect. Their standing vote is kept but not counted.
pub fn set_surrender_voter_absent(world: &mut World, player_id: u32, absent: bool) -> Result<(), SurrenderRejection> {
    let mut state = world.write_resource::<MobaMatch>();
    rules(&state).ok_or(SurrenderRejection::Disabled)?;
    seat(&state, player_id)?;
    if absent { state.surrender.absent.insert(player_id); } else { state.surrender.absent.remove(&player_id); }
    Ok(())
}

/// Read-only HUD/status view of `side`'s vote; None without surrender rules.
pub fn surrender_tally(world: &World, side: u8) -> Option<SurrenderTally> {
    let state = world.try_fetch::<MobaMatch>()?;
    let rules = rules(&state)?;
    (usize::from(side) < 2).then(|| tally(&state, rules, usize::from(side)))
}

/// Playing time until votes open (zero when open); None without surrender rules.
pub fn surrender_available_in(world: &World) -> Option<Fixed64> {
    let state = world.try_fetch::<MobaMatch>()?;
    let rules = rules(&state)?;
    Some(remaining(&state, rules))
}

/// The finish path's surrender result: `Some(Some(winner))` once a side conceded.
/// Settled only on an unpaused tick, like every other gameplay result.
pub(super) fn conceded_winner(world: &World, state: &MobaMatch) -> Option<Option<u8>> {
    if world.try_fetch::<GamePause>().is_some_and(|pause| pause.is_paused) { return None; }
    state.surrender.conceded.map(|side| Some(1 - side.min(1)))
}

/// Replay digest material. A match where nobody voted or was marked absent adds
/// nothing, so every existing digest is unchanged.
pub(super) fn hash_material(state: &MobaMatch) -> Option<String> {
    (state.surrender != SurrenderState::default()).then(|| format!("|surrender:{:?}", state.surrender))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TowerTemplateRegistry;

    const RULES: omoba_template_ids::SurrenderRulesConst = omoba_template_ids::SurrenderRulesConst {
        available_after_seconds: 900, vote_player_numerator: 5, vote_bias_numerator: 1, vote_denominator: 10,
    };

    /// Players 1, 3 on side 0 (team 1); 2, 4 on side 1 (team 2). Playing, votes open.
    fn fixture(with_rules: bool) -> World {
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut w = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        w.insert(TowerTemplateRegistry::default());
        let game_rules = with_rules.then(|| omoba_template_ids::GameRulesConst {
            surrender: Some(RULES),
            ..omoba_template_ids::GAME_RULES.expect("game_rules.lua")
        });
        setup_single_lane_match(&mut w, SingleLaneConfig {
            additional_players: vec![SingleLanePlayerConfig { player_id: 3, team_id: 1, hero: "training_luminary".into() },
                SingleLanePlayerConfig { player_id: 4, team_id: 2, hero: "training_luminary".into() }],
            game_rules, ..Default::default()
        }).unwrap();
        {
            let mut state = w.write_resource::<MobaMatch>();
            state.phase = MobaMatchPhase::Playing;
            state.elapsed = state.config.warmup + Fixed64::from_i32(900);
        }
        w
    }
    fn end_events(w: &World) -> usize {
        w.read_resource::<crate::runtime::RuntimeEvents>().iter().filter(|event| event.topic == "game.end").count()
    }

    #[test]
    fn threshold_is_ceil_half_plus_a_tenth() {
        // surrender.lua:91 math.ceil(connected_players*0.5 + 0.1).
        assert_eq!((1..=5).map(|n| RULES.votes_needed(n)).collect::<Vec<_>>(), [1, 2, 2, 3, 3]);
    }

    #[test]
    fn too_early_rejects_with_the_remaining_playing_time_and_pause_freezes_it() {
        let mut w = fixture(true);
        {
            let mut state = w.write_resource::<MobaMatch>();
            state.elapsed = state.config.warmup + Fixed64::from_i32(900) - Fixed64::from_raw(omoba_sim::fixed::SCALE * 61 - 1);
        }
        let rejection = toggle_surrender_vote(&mut w, 1).unwrap_err();
        assert!(matches!(rejection, SurrenderRejection::TooEarly { .. }));
        assert_eq!(rejection.remaining_seconds(), Some(61), "rounded up like surrender.lua:37");
        assert_eq!(surrender_available_in(&w).map(|t| t > Fixed64::ZERO), Some(true));
        w.write_resource::<GamePause>().is_paused = true;
        assert_eq!(toggle_surrender_vote(&mut w, 1), Err(SurrenderRejection::Paused));
        w.write_resource::<GamePause>().is_paused = false;
        w.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup;
        assert_eq!(toggle_surrender_vote(&mut w, 1), Err(SurrenderRejection::NotPlaying));
        assert_eq!(w.read_resource::<MobaMatch>().surrender, SurrenderState::default());
    }

    #[test]
    fn a_second_vote_takes_the_first_back_and_only_the_own_team_counts() {
        let mut w = fixture(true);
        let first = toggle_surrender_vote(&mut w, 1).unwrap();
        assert_eq!((first.voted, first.conceded), (true, false));
        assert_eq!(first.tally, SurrenderTally { side: 0, team_id: 1, agree: 1, counted: 2, threshold: 2 });
        let undo = toggle_surrender_vote(&mut w, 1).unwrap();
        assert_eq!((undo.voted, undo.tally.agree), (false, 0));
        // The enemy's votes never count for team 1.
        toggle_surrender_vote(&mut w, 1).unwrap();
        let enemy = toggle_surrender_vote(&mut w, 2).unwrap();
        assert_eq!((enemy.tally.side, enemy.tally.agree, enemy.conceded), (1, 1, false));
        assert_eq!(surrender_tally(&w, 0).map(|t| t.agree), Some(1));
        assert_eq!(toggle_surrender_vote(&mut w, 99), Err(SurrenderRejection::NotSeated));
        // A bot seat (marked absent) leaves a team of one: one vote passes.
        set_surrender_voter_absent(&mut w, 4, true).unwrap();
        assert_eq!(toggle_surrender_vote(&mut w, 4), Err(SurrenderRejection::Absent));
        assert_eq!(surrender_tally(&w, 1).map(|t| (t.agree, t.counted, t.threshold)), Some((1, 1, 1)));
    }

    #[test]
    fn a_passed_vote_ends_the_match_through_the_normal_finish_once() {
        let mut w = fixture(true);
        let digest = single_lane_replay_digest(&w);
        toggle_surrender_vote(&mut w, 1).unwrap();
        assert_ne!(single_lane_replay_digest(&w), digest, "votes are replay state");
        let passed = toggle_surrender_vote(&mut w, 3).unwrap();
        assert!(passed.conceded);
        assert_eq!(toggle_surrender_vote(&mut w, 2), Err(SurrenderRejection::NotPlaying));
        // A pause between the vote and the tick holds the result.
        w.write_resource::<GamePause>().is_paused = true;
        finish_moba_match_tick(&mut w);
        assert_eq!(w.read_resource::<MobaMatch>().phase, MobaMatchPhase::Playing);
        w.write_resource::<GamePause>().is_paused = false;
        let tick = w.read_resource::<Tick>().0;
        finish_moba_match_tick(&mut w);
        assert_eq!(w.read_resource::<MobaMatch>().phase, MobaMatchPhase::Finished { winner: Some(1), tick });
        finish_moba_match_tick(&mut w);
        assert_eq!(end_events(&w), 1);
        let end = w.read_resource::<crate::runtime::RuntimeEvents>().iter().find(|e| e.topic == "game.end").cloned().unwrap();
        assert_eq!(end.data["winner_team"], serde_json::json!(2));
    }

    #[test]
    fn nothing_happens_without_game_rules_surrender() {
        let mut w = fixture(false);
        let digest = single_lane_replay_digest(&w);
        assert_eq!(toggle_surrender_vote(&mut w, 1), Err(SurrenderRejection::Disabled));
        assert_eq!(set_surrender_voter_absent(&mut w, 2, true), Err(SurrenderRejection::Disabled));
        assert_eq!((surrender_tally(&w, 0), surrender_available_in(&w)), (None, None));
        assert_eq!(single_lane_replay_digest(&w), digest);
        finish_moba_match_tick(&mut w);
        assert_eq!(w.read_resource::<MobaMatch>().phase, MobaMatchPhase::Playing);
        assert_eq!(end_events(&w), 0);
    }
}
