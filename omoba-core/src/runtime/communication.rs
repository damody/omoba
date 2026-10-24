//! Shared communication admission. This is not network authentication, delivery,
//! or a gameplay acknowledgement.
//!
//! The caller supplies [`TrustedSession`] from an already authenticated server
//! session. The KCP consumer fills it from `ClientSession`
//! (`authenticated_player_id`, `authenticated_team_id`, `current_view_epoch`,
//! and the server's room/run), never from the payload. [`ClaimedIdentity`] is
//! checked against that session and then discarded. Membership comes from the
//! live [`MobaMatch`] roster on each call. This module does not keep a second
//! roster, does not read the wall clock, and does not allocate protocol tags.
//! The 64-entry nonce deque intentionally accepts an evicted nonce again.
//! Network replay protection lives in `communication_transport` and is not
//! implemented by enlarging this deque.
//! [`CommunicationLedger::history`] is not a network replay API.
//! [`CommunicationLedger::replay_for`] rechecks the current recipient, mute,
//! team audience, disclosure, and objective window. Wire encoding lives in
//! `communication_transport` and omits canonical target ids.
//!
//! Ability and item HUD facts come from [`NativeCommunicationFacts`]: the
//! roster entity's current ECS `Hero` and `Inventory`, checked against
//! `PlayerOwner` and entity generation. Cached slot copies are not a live
//! fallback. A dead owner's item fact uses the admitted respawn wallet only
//! when that predicate holds. Ability facts are unavailable without a live
//! hero. No stable account principal exists, so mutes belong to the trusted
//! room and run.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omoba_sim::{Fixed64, Vec2};
use specs::{Entity, World, WorldExt};

use super::native::moba_match::{
    CommunicationMember, CommunicationRosterError, MobaHeroSlot, MobaMatch, MobaMatchPhase,
};
use super::native::{Hero, Inventory, PlayerOwner};
use super::selective::{CanonicalEntityKey, TeamIdentityState};

/// Fixed64 raw units. 1024 raw = 1 second, the same scale as [`MobaMatch::elapsed`].
pub const RATE_WINDOW_RAW: i64 = 2 * 1024;
pub const RATE_LIMIT: usize = 4;
/// Inclusive while `clock < admitted + ttl`. At `ttl` the vote is expired.
pub const OBJECTIVE_VOTE_TTL_RAW: i64 = 15 * 1024;
/// Ping markers use the same match clock. Chat history has no expiry.
/// Inclusive while `clock < admitted + ttl`.
pub const PING_MARKER_TTL_RAW: i64 = 8 * 1024;
pub const MAX_TEXT_SCALARS: usize = 120;
pub const MAX_TEXT_BYTES: usize = 240;
pub const MAX_OBJECTIVE_ID_BYTES: usize = 64;
pub const MAX_HISTORY: usize = 32;
pub const MAX_DEDUP: usize = 64;
/// Mute entries stored for one recipient inside the current room and run.
pub const MAX_MUTES_PER_RECIPIENT: usize = 16;
/// Distinct recipients that can hold mutes in one room and run.
pub const MAX_MUTE_RECIPIENTS: usize = 32;
pub const MAX_OBJECTIVE_VOTES: usize = 16;

/// Eight wheel semantics. [`PingSemantic::General`] and [`PingSemantic::Warning`]
/// are separate shortcuts. Warning is not retreat.
pub const WHEEL_SEMANTICS: [PingSemantic; 8] = [
    PingSemantic::Retreat,
    PingSemantic::Push,
    PingSemantic::OnMyWay,
    PingSemantic::AllIn,
    PingSemantic::Assist,
    PingSemantic::NeedVision,
    PingSemantic::EnemyMissing,
    PingSemantic::EnemyVision,
];

const _: () = assert!((PingSemantic::Warning as u8) != (PingSemantic::Retreat as u8));
const _: () = assert!((PingSemantic::General as u8) != (PingSemantic::Warning as u8));
const _: () = assert!((PingSemantic::General as u8) != (PingSemantic::Retreat as u8));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PingSemantic {
    Retreat = 1,
    Push = 2,
    OnMyWay = 3,
    AllIn = 4,
    Assist = 5,
    NeedVision = 6,
    EnemyMissing = 7,
    EnemyVision = 8,
    /// G. Not one of the eight wheel semantics.
    General = 9,
    /// V warning/danger. Not retreat, and not a wheel semantic.
    Warning = 10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatChannel {
    Team,
    All,
}

/// Populated by the server session boundary. Zero fields are incomplete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrustedSession {
    pub room_id: u64,
    pub run_id: u64,
    pub player_id: u32,
    pub team_id: u32,
    pub view_epoch: u64,
}

/// Untrusted copy of identity. It never creates membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimedIdentity {
    pub room_id: u64,
    pub run_id: u64,
    pub player_id: u32,
    pub team_id: u32,
    pub observer: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetClaim {
    pub replica_id: u64,
    pub canonical_id: u32,
    pub generation: u32,
    pub disclosure_epoch: u64,
    pub view_epoch: u64,
}

/// Index or player id only. Cooldown text, item names, and scores are not claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudClaim {
    OwnedAbility { index: u8 },
    OwnedItem { slot: u8 },
    PublicScore { player_id: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommunicationRequest {
    Emote { content_id: u16 },
    Status { hud: HudClaim },
    Ping {
        semantic: PingSemantic,
        point: Vec2,
        target: Option<TargetClaim>,
        hud: Option<HudClaim>,
    },
    Chat {
        channel: ChatChannel,
        text: String,
    },
    ObjectiveVote {
        objective_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationEventId {
    pub room_id: u64,
    pub run_id: u64,
    pub sender_id: u32,
    pub sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosedTarget {
    pub replica_id: u64,
    pub canonical_id: u32,
    pub generation: u32,
    pub disclosure_epoch: u64,
    pub view_epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudFact {
    OwnedAbility { index: u8, remaining_raw: i64 },
    /// `cooldown_bits` is `f32::to_bits` of the owner's finite inventory cooldown.
    OwnedItem { slot: u8, catalog_id: u16, cooldown_bits: u32 },
    PublicScore { player_id: u32, kills: u32, deaths: u32, assists: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmittedBody {
    Emote { content_id: u16, admitted_at_raw: i64, expires_at_raw: i64 },
    Status { hud: HudFact, admitted_at_raw: i64, expires_at_raw: i64 },
    Ping {
        semantic: PingSemantic,
        point: Vec2,
        target: Option<DisclosedTarget>,
        hud: Option<HudFact>,
        admitted_at_raw: i64,
        expires_at_raw: i64,
    },
    Chat {
        channel: ChatChannel,
        text: String,
        admitted_at_raw: i64,
    },
    ObjectiveVote {
        objective_id: String,
        home: Vec2,
        admitted_at_raw: i64,
        expires_at_raw: i64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunicationAudience {
    Team,
    All,
}

/// An admitted recipient set. Not a socket write and not `CriticalInputResult`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedCommunication {
    pub id: CommunicationEventId,
    pub sender_team: u32,
    pub audience: CommunicationAudience,
    pub recipients: Vec<u32>,
    pub body: AdmittedBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishedObjectiveVote {
    pub team_id: u32,
    pub sender_id: u32,
    pub expires_at_raw: i64,
    pub home: Vec2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedObjectiveVoteText {
    pub vote: PublishedObjectiveVote,
    pub objective_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionError {
    IncompleteSession,
    IdentityMismatch,
    ObserverDenied,
    InvalidNonce,
    InvalidClock,
    RegressedClock,
    ClockOverflow,
    InvalidRoster,
    UnresolvedTeam,
    AbsentMembership,
    ForeignMembership,
    ResultsClosed,
    EmptyText,
    TextLimit,
    TextControl,
    PointOutOfBounds,
    StaleOrHiddenTarget,
    DisclosureTeamMismatch,
    HudFactUnavailable,
    ContentUnavailable,
    ObjectiveUnavailable,
    ObjectivePhase,
    ObjectiveCapacity,
    Duplicate,
    RateLimited,
    SequenceExhausted,
    MuteCapacity,
    MuteUnauthorized,
}

#[derive(Clone, Debug)]
struct StoredVote {
    team_id: u32,
    objective_id: String,
    home: Vec2,
    expires_at_raw: i64,
}

#[derive(Clone, Debug)]
pub struct CommunicationLedger {
    room_id: Option<u64>,
    run_id: Option<u64>,
    last_clock_raw: Option<i64>,
    next_sequence: u64,
    history: VecDeque<AdmittedCommunication>,
    dedup: VecDeque<(u32, u64)>,
    rates: BTreeMap<u32, VecDeque<i64>>,
    votes: BTreeMap<u32, StoredVote>,
    /// Mutes apply only while `mute_scope` equals the trusted room and run.
    mute_scope: Option<(u64, u64)>,
    /// `(recipient player, sender player)` inside `mute_scope`.
    mutes: BTreeSet<(u32, u32)>,
}

impl Default for CommunicationLedger {
    fn default() -> Self {
        Self {
            room_id: None,
            run_id: None,
            last_clock_raw: None,
            next_sequence: 1,
            history: VecDeque::new(),
            dedup: VecDeque::new(),
            rates: BTreeMap::new(),
            votes: BTreeMap::new(),
            mute_scope: None,
            mutes: BTreeSet::new(),
        }
    }
}

impl CommunicationLedger {
    pub fn history(&self) -> &VecDeque<AdmittedCommunication> {
        &self.history
    }

    /// Scope plus the next sequence. A new run stores its first event at sequence 1
    /// and leaves this counter at 2, which is also the counter after one event in
    /// the previous run. The room and run make that first new-run admission distinct.
    pub fn admission_cursor(&self) -> AdmissionCursor {
        AdmissionCursor {
            room_id: self.room_id,
            run_id: self.run_id,
            next_sequence: self.next_sequence,
        }
    }

    pub fn is_muted(&self, room_id: u64, run_id: u64, recipient: u32, sender: u32) -> bool {
        self.mute_scope == Some((room_id, run_id)) && self.mutes.contains(&(recipient, sender))
    }

    /// Votes that are still inside their clock window and whose objective id is
    /// still a current authored camp. Hidden camp status is not returned.
    pub fn current_objective_votes(&self, match_state: &MobaMatch) -> Vec<PublishedObjectiveVoteText> {
        let clock = match_state.elapsed.raw();
        if clock < 0 {
            return Vec::new();
        }
        let Ok(roster) = match_state.communication_roster() else {
            return Vec::new();
        };
        let mut votes = Vec::new();
        for (sender_id, vote) in &self.votes {
            if clock >= vote.expires_at_raw {
                continue;
            }
            let still_member = roster.iter().any(|member| {
                member.player_id == *sender_id && member.team_id == vote.team_id
            });
            if !still_member {
                continue;
            }
            let Some(home) = match_state.communication_objective_home(&vote.objective_id) else {
                continue;
            };
            if home != vote.home {
                continue;
            }
            votes.push(PublishedObjectiveVoteText {
                vote: PublishedObjectiveVote {
                    team_id: vote.team_id,
                    sender_id: *sender_id,
                    expires_at_raw: vote.expires_at_raw,
                    home,
                },
                objective_id: vote.objective_id.clone(),
            });
        }
        votes
    }

    /// Authorized retained events for one current recipient. Does not mutate.
    /// A different room or run returns an empty list. A team event is omitted
    /// when the recipient's current team is not the sender team captured at
    /// admission. A target that is no longer disclosed becomes a world point.
    pub fn replay_for(
        &self,
        trusted: TrustedSession,
        claimed: &ClaimedIdentity,
        match_state: &MobaMatch,
        disclosure: Option<&TeamIdentityState>,
    ) -> Result<Vec<AdmittedCommunication>, AdmissionError> {
        let auth = authenticate(trusted, claimed, match_state)?;
        let clock = match_clock(match_state)?;
        if self.room_id != Some(trusted.room_id) || self.run_id != Some(trusted.run_id) {
            return Ok(Vec::new());
        }
        let mut replay = Vec::new();
        for event in &self.history {
            if event.id.room_id != trusted.room_id
                || event.id.run_id != trusted.run_id
                || !event.recipients.contains(&trusted.player_id)
            {
                continue;
            }
            if event.audience == CommunicationAudience::Team && event.sender_team != trusted.team_id {
                continue;
            }
            if self.is_muted(trusted.room_id, trusted.run_id, trusted.player_id, event.id.sender_id) {
                continue;
            }
            if let AdmittedBody::Ping { admitted_at_raw, .. } = &event.body {
                if *admitted_at_raw < 0 || clock < *admitted_at_raw {
                    return Err(AdmissionError::RegressedClock);
                }
            }
            let body = match &event.body {
                AdmittedBody::Emote { admitted_at_raw, expires_at_raw, .. }
                | AdmittedBody::Status { admitted_at_raw, expires_at_raw, .. } => {
                    if *admitted_at_raw < 0 || clock < *admitted_at_raw {
                        return Err(AdmissionError::RegressedClock);
                    }
                    if *expires_at_raw <= *admitted_at_raw {
                        return Err(AdmissionError::InvalidClock);
                    }
                    if clock >= *expires_at_raw { continue; }
                    // Preserve the originally admitted fact and clock, never recompute a cooldown.
                    event.body.clone()
                }
                AdmittedBody::Ping { semantic, point, target, hud, admitted_at_raw, expires_at_raw } => {
                    if *expires_at_raw <= *admitted_at_raw {
                        return Err(AdmissionError::InvalidClock);
                    }
                    if clock >= *expires_at_raw {
                        continue;
                    }
                    let target = target.as_ref().and_then(|disclosed| {
                        let claim = TargetClaim {
                            replica_id: disclosed.replica_id,
                            canonical_id: disclosed.canonical_id,
                            generation: disclosed.generation,
                            disclosure_epoch: disclosed.disclosure_epoch,
                            view_epoch: trusted.view_epoch,
                        };
                        resolve_target(&auth, &claim, disclosure).ok()
                    });
                    AdmittedBody::Ping {
                        semantic: *semantic,
                        point: *point,
                        target,
                        hud: *hud,
                        admitted_at_raw: *admitted_at_raw,
                        expires_at_raw: *expires_at_raw,
                    }
                }
                AdmittedBody::Chat { channel, text, admitted_at_raw } => {
                    AdmittedBody::Chat { channel: *channel, text: text.clone(), admitted_at_raw: *admitted_at_raw }
                }
                AdmittedBody::ObjectiveVote { objective_id, home, admitted_at_raw, expires_at_raw } => {
                    if *admitted_at_raw < 0 || clock < *admitted_at_raw {
                        return Err(AdmissionError::RegressedClock);
                    }
                    let Some(current_vote) = self.votes.get(&event.id.sender_id) else {
                        continue;
                    };
                    if current_vote.objective_id != *objective_id
                        || current_vote.home != *home
                        || current_vote.expires_at_raw != *expires_at_raw
                        || current_vote.team_id != event.sender_team
                    {
                        continue;
                    }
                    if clock >= *expires_at_raw {
                        continue;
                    }
                    let Some(current) = match_state.communication_objective_home(objective_id) else {
                        continue;
                    };
                    if current != *home {
                        continue;
                    }
                    AdmittedBody::ObjectiveVote {
                        objective_id: objective_id.clone(),
                        home: *home,
                        admitted_at_raw: *admitted_at_raw,
                        expires_at_raw: *expires_at_raw,
                    }
                }
            };
            replay.push(AdmittedCommunication {
                id: event.id,
                sender_team: event.sender_team,
                audience: event.audience,
                recipients: vec![trusted.player_id],
                body,
            });
        }
        Ok(replay)
    }

    pub fn set_mute(
        &mut self,
        trusted: TrustedSession,
        claimed: &ClaimedIdentity,
        match_state: &MobaMatch,
        sender_id: u32,
        muted: bool,
    ) -> Result<(), AdmissionError> {
        let auth = authenticate(trusted, claimed, match_state)?;
        if !auth.roster.iter().any(|member| member.player_id == sender_id) {
            return Err(AdmissionError::MuteUnauthorized);
        }
        let scope = (trusted.room_id, trusted.run_id);
        if self.mute_scope != Some(scope) {
            self.mutes.clear();
            self.mute_scope = Some(scope);
            if muted {
                self.mutes.insert((trusted.player_id, sender_id));
            }
            return Ok(());
        }
        let key = (trusted.player_id, sender_id);
        if muted {
            if self.mutes.contains(&key) {
                return Ok(());
            }
            let owned = self.mutes.iter().filter(|(recipient, _)| *recipient == trusted.player_id).count();
            if owned >= MAX_MUTES_PER_RECIPIENT {
                return Err(AdmissionError::MuteCapacity);
            }
            let mut recipients = BTreeSet::new();
            for (recipient, _) in &self.mutes {
                recipients.insert(*recipient);
            }
            if !recipients.contains(&trusted.player_id) && recipients.len() >= MAX_MUTE_RECIPIENTS {
                return Err(AdmissionError::MuteCapacity);
            }
            self.mutes.insert(key);
        } else {
            self.mutes.remove(&key);
        }
        Ok(())
    }

    pub fn admit(
        &mut self,
        trusted: TrustedSession,
        claimed: &ClaimedIdentity,
        nonce: u64,
        request: &CommunicationRequest,
        match_state: &MobaMatch,
        disclosure: Option<&TeamIdentityState>,
        facts: Option<&NativeCommunicationFacts<'_>>,
    ) -> Result<AdmittedCommunication, AdmissionError> {
        let auth = authenticate(trusted, claimed, match_state)?;
        if nonce == 0 {
            return Err(AdmissionError::InvalidNonce);
        }
        let clock = match_clock(match_state)?;
        let same_run = self.room_id == Some(trusted.room_id) && self.run_id == Some(trusted.run_id);
        if same_run && self.last_clock_raw.is_some_and(|last| clock < last) {
            return Err(AdmissionError::RegressedClock);
        }
        if matches!(match_state.phase, MobaMatchPhase::Finished { .. }) {
            return Err(AdmissionError::ResultsClosed);
        }
        let prepared = prepare_body(
            &auth,
            clock,
            nonce,
            request,
            match_state,
            disclosure,
            facts,
            same_run,
            self,
        )?;
        let sequence = if same_run { self.next_sequence } else { 1 };
        let Some(next_sequence) = sequence.checked_add(1) else {
            return Err(AdmissionError::SequenceExhausted);
        };
        if !same_run {
            self.room_id = Some(trusted.room_id);
            self.run_id = Some(trusted.run_id);
            self.last_clock_raw = None;
            self.next_sequence = 1;
            self.history.clear();
            self.dedup.clear();
            self.rates.clear();
            self.votes.clear();
            if self.mute_scope.is_some_and(|scope| scope != (trusted.room_id, trusted.run_id)) {
                self.mutes.clear();
                self.mute_scope = None;
            }
        }
        self.last_clock_raw = Some(clock);
        self.next_sequence = next_sequence;
        let stamps = self.rates.entry(auth.member.player_id).or_default();
        stamps.retain(|stamp| in_rate_window(clock, *stamp));
        stamps.push_back(clock);
        if self.dedup.len() == MAX_DEDUP {
            self.dedup.pop_front();
        }
        self.dedup.push_back((auth.member.player_id, nonce));
        if let Some(vote) = prepared.vote {
            self.votes.insert(auth.member.player_id, vote);
        }
        let event = AdmittedCommunication {
            id: CommunicationEventId {
                room_id: trusted.room_id,
                run_id: trusted.run_id,
                sender_id: auth.member.player_id,
                sequence,
            },
            sender_team: auth.member.team_id,
            audience: prepared.audience,
            recipients: prepared.recipients,
            body: prepared.body,
        };
        self.history.push_back(event.clone());
        while self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
        Ok(event)
    }
}

/// Identity of the next stored event. `next_sequence` alone restarts on a new room or run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionCursor {
    pub room_id: Option<u64>,
    pub run_id: Option<u64>,
    pub next_sequence: u64,
}

struct Auth {
    member: CommunicationMember,
    roster: Vec<CommunicationMember>,
    view_epoch: u64,
    room_id: u64,
    run_id: u64,
}

struct Prepared {
    body: AdmittedBody,
    audience: CommunicationAudience,
    recipients: Vec<u32>,
    vote: Option<StoredVote>,
}

fn authenticate(
    trusted: TrustedSession,
    claimed: &ClaimedIdentity,
    match_state: &MobaMatch,
) -> Result<Auth, AdmissionError> {
    if trusted.room_id == 0
        || trusted.run_id == 0
        || trusted.player_id == 0
        || trusted.team_id == 0
        || trusted.view_epoch == 0
    {
        return Err(AdmissionError::IncompleteSession);
    }
    if claimed.room_id != trusted.room_id
        || claimed.run_id != trusted.run_id
        || claimed.player_id != trusted.player_id
        || claimed.team_id != trusted.team_id
    {
        return Err(AdmissionError::IdentityMismatch);
    }
    if claimed.observer {
        return Err(AdmissionError::ObserverDenied);
    }
    let roster = match match_state.communication_roster() {
        Ok(roster) => roster,
        Err(CommunicationRosterError::DuplicateOrZero) => {
            return Err(AdmissionError::InvalidRoster);
        }
        Err(CommunicationRosterError::UnresolvedTeam) => {
            return Err(AdmissionError::UnresolvedTeam);
        }
    };
    let Some(member) = roster.iter().copied().find(|member| member.player_id == trusted.player_id) else {
        return Err(AdmissionError::AbsentMembership);
    };
    if member.team_id != trusted.team_id {
        return Err(AdmissionError::ForeignMembership);
    }
    Ok(Auth {
        member,
        roster,
        view_epoch: trusted.view_epoch,
        room_id: trusted.room_id,
        run_id: trusted.run_id,
    })
}

fn match_clock(match_state: &MobaMatch) -> Result<i64, AdmissionError> {
    let clock = match_state.elapsed.raw();
    if clock < 0 {
        Err(AdmissionError::InvalidClock)
    } else {
        Ok(clock)
    }
}

fn in_rate_window(clock: i64, stamp: i64) -> bool {
    match clock.checked_sub(stamp) {
        Some(age) => age <= RATE_WINDOW_RAW,
        None => true,
    }
}

fn rate_open(ledger: &CommunicationLedger, sender: u32, clock: i64, same_run: bool) -> bool {
    if !same_run {
        return true;
    }
    let used = ledger
        .rates
        .get(&sender)
        .map(|stamps| stamps.iter().filter(|stamp| in_rate_window(clock, **stamp)).count())
        .unwrap_or(0);
    used < RATE_LIMIT
}

fn prepare_body(
    auth: &Auth,
    clock: i64,
    nonce: u64,
    request: &CommunicationRequest,
    match_state: &MobaMatch,
    disclosure: Option<&TeamIdentityState>,
    facts: Option<&NativeCommunicationFacts<'_>>,
    same_run: bool,
    ledger: &CommunicationLedger,
) -> Result<Prepared, AdmissionError> {
    let (body, audience, vote) = match request {
        CommunicationRequest::Emote { content_id } => {
            let content = omoba_template_ids::communication_content(*content_id)
                .ok_or(AdmissionError::ContentUnavailable)?;
            // Character gestures need a live owned hero during play. Wheel emotes
            // are team communication and remain available to dead roster members.
            if content.interaction_slot != 0 && (!matches!(match_state.phase, MobaMatchPhase::Playing)
                || !facts.is_some_and(|f| f.live_owned_entity(auth.member.player_id)
                    .is_some_and(|entity| f.world.read_storage::<Hero>().get(entity).is_some()))) {
                return Err(AdmissionError::ContentUnavailable);
            }
            let expires_at_raw = clock.checked_add(i64::from(content.duration_raw))
                .ok_or(AdmissionError::ClockOverflow)?;
            (AdmittedBody::Emote { content_id: *content_id, admitted_at_raw: clock, expires_at_raw }, CommunicationAudience::Team, None)
        }
        CommunicationRequest::Status { hud } => {
            let hud = resolve_hud(auth.member.player_id, *hud, match_state, facts)?;
            let expires_at_raw = clock.checked_add(PING_MARKER_TTL_RAW).ok_or(AdmissionError::ClockOverflow)?;
            (AdmittedBody::Status { hud, admitted_at_raw: clock, expires_at_raw }, CommunicationAudience::Team, None)
        }
        CommunicationRequest::Ping { semantic, point, target, hud } => {
            if !match_state.communication_point_allowed(*point) {
                return Err(AdmissionError::PointOutOfBounds);
            }
            let target = match target {
                None => None,
                Some(claim) => Some(resolve_target(auth, claim, disclosure)?),
            };
            let hud = match hud {
                None => None,
                Some(claim) => Some(resolve_hud(auth.member.player_id, *claim, match_state, facts)?),
            };
            let Some(expires_at_raw) = clock.checked_add(PING_MARKER_TTL_RAW) else {
                return Err(AdmissionError::ClockOverflow);
            };
            (
                AdmittedBody::Ping {
                    semantic: *semantic,
                    point: *point,
                    target,
                    hud,
                    admitted_at_raw: clock,
                    expires_at_raw,
                },
                CommunicationAudience::Team,
                None,
            )
        }
        CommunicationRequest::Chat { channel, text } => {
            validate_text(text)?;
            let audience = match channel {
                ChatChannel::Team => CommunicationAudience::Team,
                ChatChannel::All => CommunicationAudience::All,
            };
            (
                AdmittedBody::Chat { channel: *channel, text: text.clone(), admitted_at_raw: clock },
                audience,
                None,
            )
        }
        CommunicationRequest::ObjectiveVote { objective_id } => {
            if !matches!(match_state.phase, MobaMatchPhase::Playing) {
                return Err(AdmissionError::ObjectivePhase);
            }
            if objective_id.is_empty()
                || objective_id.len() > MAX_OBJECTIVE_ID_BYTES
                || objective_id.chars().any(rejected_scalar)
            {
                return Err(AdmissionError::ObjectiveUnavailable);
            }
            let Some(home) = match_state.communication_objective_home(objective_id) else {
                return Err(AdmissionError::ObjectiveUnavailable);
            };
            let Some(expires_at_raw) = clock.checked_add(OBJECTIVE_VOTE_TTL_RAW) else {
                return Err(AdmissionError::ClockOverflow);
            };
            let replacing = same_run && ledger.votes.contains_key(&auth.member.player_id);
            if !replacing && same_run && ledger.votes.len() >= MAX_OBJECTIVE_VOTES {
                return Err(AdmissionError::ObjectiveCapacity);
            }
            (
                AdmittedBody::ObjectiveVote {
                    objective_id: objective_id.clone(),
                    home,
                    admitted_at_raw: clock,
                    expires_at_raw,
                },
                CommunicationAudience::Team,
                Some(StoredVote {
                    team_id: auth.member.team_id,
                    objective_id: objective_id.clone(),
                    home,
                    expires_at_raw,
                }),
            )
        }
    };
    if same_run
        && ledger.dedup.iter().any(|(sender, seen)| *sender == auth.member.player_id && *seen == nonce)
    {
        return Err(AdmissionError::Duplicate);
    }
    if !rate_open(ledger, auth.member.player_id, clock, same_run) {
        return Err(AdmissionError::RateLimited);
    }
    let mutes = (ledger.mute_scope == Some((auth.room_id, auth.run_id))).then_some(&ledger.mutes);
    Ok(Prepared {
        recipients: recipients(&auth.roster, auth.member, audience, mutes),
        audience,
        body,
        vote,
    })
}

fn recipients(
    roster: &[CommunicationMember],
    sender: CommunicationMember,
    audience: CommunicationAudience,
    mutes: Option<&BTreeSet<(u32, u32)>>,
) -> Vec<u32> {
    roster
        .iter()
        .filter(|member| match audience {
            CommunicationAudience::Team => member.team_id == sender.team_id,
            CommunicationAudience::All => true,
        })
        .filter(|member| {
            mutes.is_none_or(|set| !set.contains(&(member.player_id, sender.player_id)))
        })
        .map(|member| member.player_id)
        .collect()
}

/// Read-only live HUD facts for one match. Holds no copied roster.
pub struct NativeCommunicationFacts<'a> {
    world: &'a World,
    match_state: &'a MobaMatch,
}

impl<'a> NativeCommunicationFacts<'a> {
    pub fn new(world: &'a World, match_state: &'a MobaMatch) -> Self {
        Self { world, match_state }
    }

    /// Current ability cooldown raw for a living owned hero. Missing map entry is ready (`0`).
    /// A negative stored raw value is invalid and returns none. Dead heroes have no ability fact.
    pub fn owned_ability(&self, player_id: u32, index: u8) -> Option<i64> {
        let entity = self.live_owned_entity(player_id)?;
        let heroes = self.world.read_storage::<Hero>();
        let hero = heroes.get(entity)?;
        let ability_id = hero.abilities.get(usize::from(index))?;
        let remaining = hero.ability_cooldowns.get(ability_id).copied().unwrap_or(Fixed64::ZERO);
        if remaining.raw() < 0 {
            return None;
        }
        Some(remaining.raw())
    }

    /// Living inventory, or the admitted dead wallet when the owner has no entity.
    /// Non-finite, negative, and unknown catalog rows are rejected.
    pub fn owned_item(&self, player_id: u32, slot_index: u8) -> Option<(u16, u32)> {
        let alive = unique_slot(self.match_state, player_id)?.entity.is_some();
        if alive {
            let entity = self.live_owned_entity(player_id)?;
            let inventories = self.world.read_storage::<Inventory>();
            return item_fact(inventories.get(entity)?, slot_index);
        }
        let (inventory, _) = unique_slot(self.match_state, player_id)?.admitted_dead_wallet_ref()?;
        item_fact(inventory, slot_index)
    }

    fn live_owned_entity(&self, player_id: u32) -> Option<Entity> {
        let slot = unique_slot(self.match_state, player_id)?;
        let entity = slot.entity?;
        if !self.world.entities().is_alive(entity) {
            return None;
        }
        let owners = self.world.read_storage::<PlayerOwner>();
        if owners.get(entity).map(|owner| owner.player_id) != Some(player_id) {
            return None;
        }
        Some(entity)
    }
}

fn unique_slot(match_state: &MobaMatch, player_id: u32) -> Option<&MobaHeroSlot> {
    let mut found = None;
    for slot in &match_state.heroes {
        if slot.player_id == player_id {
            if found.is_some() {
                return None;
            }
            found = Some(slot);
        }
    }
    found
}

fn item_fact(inventory: &Inventory, slot_index: u8) -> Option<(u16, u32)> {
    let item = inventory.slots.get(usize::from(slot_index))?.as_ref()?;
    if !item.cooldown_remaining.is_finite() || item.cooldown_remaining < 0.0 {
        return None;
    }
    let catalog_id = omoba_template_ids::MOBA_ITEM_CATALOG
        .iter()
        .find(|row| row.id == item.item_id)
        .map(|row| row.catalog_id)?;
    Some((catalog_id, item.cooldown_remaining.to_bits()))
}

fn resolve_target(
    auth: &Auth,
    claim: &TargetClaim,
    disclosure: Option<&TeamIdentityState>,
) -> Result<DisclosedTarget, AdmissionError> {
    if claim.replica_id == 0 || claim.view_epoch != auth.view_epoch {
        return Err(AdmissionError::StaleOrHiddenTarget);
    }
    let Some(disclosure) = disclosure else {
        return Err(AdmissionError::StaleOrHiddenTarget);
    };
    if disclosure.team_id() != auth.member.team_id {
        return Err(AdmissionError::DisclosureTeamMismatch);
    }
    let canonical = CanonicalEntityKey {
        id: claim.canonical_id,
        generation: claim.generation,
    };
    if !disclosure.disclosed_identity_matches(claim.replica_id, claim.disclosure_epoch, canonical) {
        return Err(AdmissionError::StaleOrHiddenTarget);
    }
    Ok(DisclosedTarget {
        replica_id: claim.replica_id,
        canonical_id: claim.canonical_id,
        generation: claim.generation,
        disclosure_epoch: claim.disclosure_epoch,
        view_epoch: auth.view_epoch,
    })
}

fn resolve_hud(
    sender_id: u32,
    claim: HudClaim,
    match_state: &MobaMatch,
    facts: Option<&NativeCommunicationFacts<'_>>,
) -> Result<HudFact, AdmissionError> {
    match claim {
        HudClaim::OwnedAbility { index } => {
            let remaining_raw = facts
                .ok_or(AdmissionError::HudFactUnavailable)?
                .owned_ability(sender_id, index)
                .ok_or(AdmissionError::HudFactUnavailable)?;
            Ok(HudFact::OwnedAbility { index, remaining_raw })
        }
        HudClaim::OwnedItem { slot } => {
            let (catalog_id, cooldown_bits) = facts
                .ok_or(AdmissionError::HudFactUnavailable)?
                .owned_item(sender_id, slot)
                .ok_or(AdmissionError::HudFactUnavailable)?;
            Ok(HudFact::OwnedItem { slot, catalog_id, cooldown_bits })
        }
        HudClaim::PublicScore { player_id } => {
            let (kills, deaths, assists) = match_state
                .communication_public_score(player_id)
                .ok_or(AdmissionError::HudFactUnavailable)?;
            Ok(HudFact::PublicScore { player_id, kills, deaths, assists })
        }
    }
}

fn validate_text(text: &str) -> Result<(), AdmissionError> {
    if text.is_empty() {
        return Err(AdmissionError::EmptyText);
    }
    if text.chars().count() > MAX_TEXT_SCALARS || text.len() > MAX_TEXT_BYTES {
        return Err(AdmissionError::TextLimit);
    }
    if text.chars().any(rejected_scalar) {
        return Err(AdmissionError::TextControl);
    }
    if text.chars().all(char::is_whitespace) {
        return Err(AdmissionError::EmptyText);
    }
    Ok(())
}

fn rejected_scalar(scalar: char) -> bool {
    scalar.is_control() || scalar == '\u{2028}' || scalar == '\u{2029}'
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    use specs::{Builder, Entity, World, WorldExt};
    use crate::runtime::native::moba_match::{
        communication_roster_match, shop_fixture_slot, MobaMatchPhase,
    };
    use crate::runtime::native::{Hero, Inventory, ItemInstance, PlayerOwner};
    use crate::runtime::selective::TeamIdentityState;

    fn trusted_for(player: u32, team: u32, room: u64, run: u64) -> (TrustedSession, ClaimedIdentity) {
        (
            TrustedSession { room_id: room, run_id: run, player_id: player, team_id: team, view_epoch: 4 },
            ClaimedIdentity { room_id: room, run_id: run, player_id: player, team_id: team, observer: false },
        )
    }

    fn roster_match(phase: MobaMatchPhase, map: Option<&str>) -> MobaMatch {
        let hero = |id| shop_fixture_slot(id, if id <= 2 || id == 5 { 0 } else { 1 }, Some(specs::Entity::new(id, specs::world::Generation::new(1))));
        let mut dead = shop_fixture_slot(2, 0, None);
        dead.deaths = 1;
        communication_roster_match(
            vec![hero(1), dead, hero(3), hero(4), hero(5)],
            phase,
            Fixed64::ZERO,
            map,
        )
    }

    #[test]
    fn communication_standalone_status_uses_live_facts_without_any_world_point() {
        let mut world = World::new();
        world.register::<Hero>(); world.register::<Inventory>(); world.register::<PlayerOwner>();
        let mut hero = Hero::new("spark_hero".into(), "Spark".into(), "Test".into());
        hero.abilities.push("spark".into());
        hero.ability_cooldowns.insert("spark".into(), Fixed64::from_i32(7));
        let catalog = &omoba_template_ids::MOBA_ITEM_CATALOG[0];
        let mut inventory = Inventory::new();
        inventory.slots[0] = Some(ItemInstance { item_id: catalog.id.into(), cooldown_remaining: 3.5 });
        let entity = world.create_entity().with(hero).with(inventory).with(PlayerOwner { player_id: 1 }).build();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        match_state.heroes[0].entity = Some(entity);
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let mut ledger = CommunicationLedger::default();
        for (nonce, hud) in [(1, HudClaim::OwnedAbility { index: 0 }), (2, HudClaim::OwnedItem { slot: 0 }), (3, HudClaim::PublicScore { player_id: 3 })] {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            let event = ledger.admit(trusted, &claimed, nonce, &CommunicationRequest::Status { hud }, &match_state, None, Some(&facts)).unwrap();
            assert_eq!(event.recipients, vec![1, 2, 5]);
            match event.body {
                AdmittedBody::Status { hud: HudFact::OwnedAbility { remaining_raw, .. }, .. } => assert_eq!(remaining_raw, 7 * 1024),
                AdmittedBody::Status { hud: HudFact::OwnedItem { catalog_id, cooldown_bits, .. }, .. } => assert_eq!((catalog_id, cooldown_bits), (catalog.catalog_id, 3.5f32.to_bits())),
                AdmittedBody::Status { hud: HudFact::PublicScore { player_id, .. }, .. } => assert_eq!(player_id, 3),
                other => panic!("unexpected {other:?}"),
            }
        }
        world.write_storage::<PlayerOwner>().get_mut(entity).unwrap().player_id = 3;
        let before = ledger.admission_cursor();
        let facts = NativeCommunicationFacts::new(&world, &match_state);
        assert_eq!(ledger.admit(trusted, &claimed, 4, &CommunicationRequest::Status { hud: HudClaim::OwnedAbility { index: 0 } }, &match_state, None, Some(&facts)), Err(AdmissionError::HudFactUnavailable));
        assert_eq!(ledger.admission_cursor(), before);
        match_state.elapsed = Fixed64::from_raw(PING_MARKER_TTL_RAW);
        assert!(ledger.replay_for(trusted, &claimed, &match_state, None).unwrap().is_empty());
    }

    #[test]
    fn communication_emote_catalog_death_mute_phase_and_expiry_policy() {
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        let (trusted, claimed) = trusted_for(2, 1, 9, 3); // dead roster member
        let mut ledger = CommunicationLedger::default();
        let (recipient, recipient_claim) = trusted_for(1, 1, 9, 3);
        ledger.set_mute(recipient, &recipient_claim, &match_state, 2, true).unwrap();
        let emote = CommunicationRequest::Emote { content_id: 103 };
        let event = ledger.admit(trusted, &claimed, 1, &emote, &match_state, None, None).unwrap();
        assert_eq!(event.recipients, vec![2, 5]);
        assert!(matches!(event.body, AdmittedBody::Emote { content_id: 103, .. }));
        assert_eq!(ledger.admit(trusted, &claimed, 1, &emote, &match_state, None, None), Err(AdmissionError::Duplicate));
        let before = ledger.admission_cursor();
        for content_id in [0, 999, 1] {
            assert_eq!(ledger.admit(trusted, &claimed, 2, &CommunicationRequest::Emote { content_id }, &match_state, None, None), Err(AdmissionError::ContentUnavailable));
            assert_eq!(ledger.admission_cursor(), before);
        }
        match_state.elapsed = Fixed64::from_raw(3072);
        assert!(ledger.replay_for(trusted, &claimed, &match_state, None).unwrap().is_empty());
    }

    #[test]
    fn communication_character_interactions_require_live_owned_hero_and_playing_phase() {
        let mut world = World::new(); world.register::<Hero>(); world.register::<PlayerOwner>();
        let entity = world.create_entity().with(Hero::new("test".into(), "Test".into(), "Original".into()))
            .with(PlayerOwner::new(1)).build();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        match_state.heroes[0].entity = Some(entity);
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let mut ledger = CommunicationLedger::default();
        for id in 1u16..=5 {
            match_state.elapsed = Fixed64::from_raw(i64::from(id) * (RATE_WINDOW_RAW + 1));
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            let event = ledger.admit(trusted, &claimed, u64::from(id), &CommunicationRequest::Emote { content_id: id }, &match_state, None, Some(&facts)).unwrap();
            assert!(matches!(event.body, AdmittedBody::Emote { content_id, .. } if content_id == id));
        }
        match_state.elapsed = Fixed64::from_raw(6 * (RATE_WINDOW_RAW + 1));
        for phase in [MobaMatchPhase::Warmup, MobaMatchPhase::Playing] {
            match_state.phase = phase;
            if matches!(match_state.phase, MobaMatchPhase::Playing) { world.write_storage::<Hero>().remove(entity); }
            let before = ledger.admission_cursor();
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            assert_eq!(ledger.admit(trusted, &claimed, 6, &CommunicationRequest::Emote { content_id: 1 }, &match_state, None, Some(&facts)), Err(AdmissionError::ContentUnavailable));
            assert_eq!(ledger.admission_cursor(), before);
        }
    }

    fn ping(semantic: PingSemantic) -> CommunicationRequest {
        CommunicationRequest::Ping {
            semantic,
            point: Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            target: None,
            hud: None,
        }
    }

    fn admit_ping(
        ledger: &mut CommunicationLedger,
        player: u32,
        team: u32,
        nonce: u64,
        match_state: &MobaMatch,
    ) -> Result<AdmittedCommunication, AdmissionError> {
        let (trusted, claimed) = trusted_for(player, team, 9, 3);
        ledger.admit(trusted, &claimed, nonce, &ping(PingSemantic::Assist), match_state, None, None)
    }

    #[test]
    fn communication_wheel_semantics_keep_general_and_warning_distinct() {
        assert_eq!(WHEEL_SEMANTICS.len(), 8);
        assert!(!WHEEL_SEMANTICS.contains(&PingSemantic::General));
        assert!(!WHEEL_SEMANTICS.contains(&PingSemantic::Warning));
        assert_ne!(PingSemantic::Warning, PingSemantic::Retreat);
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let warning = ledger.admit(trusted, &claimed, 1, &ping(PingSemantic::Warning), &match_state, None, None).unwrap();
        match_state.elapsed = Fixed64::from_raw(RATE_WINDOW_RAW + 1);
        let general = ledger.admit(trusted, &claimed, 2, &ping(PingSemantic::General), &match_state, None, None).unwrap();
        assert!(matches!(warning.body, AdmittedBody::Ping { semantic: PingSemantic::Warning, .. }));
        assert!(matches!(general.body, AdmittedBody::Ping { semantic: PingSemantic::General, .. }));
    }

    #[test]
    fn communication_accepts_owner_and_refuses_forged_absent_or_foreign_membership() {
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        let accepted = admit_ping(&mut ledger, 1, 1, 1, &match_state).unwrap();
        assert_eq!(accepted.id.sender_id, 1);
        assert_eq!(accepted.recipients, vec![1, 2, 5]);
        assert!(match_state.heroes[1].entity.is_none());
        let (mut trusted, claimed) = trusted_for(1, 1, 9, 3);
        trusted.view_epoch = 0;
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &ping(PingSemantic::Retreat), &match_state, None, None),
            Err(AdmissionError::IncompleteSession)
        );
        trusted.view_epoch = 4;
        assert_eq!(
            ledger.admit(trusted, &claimed, 0, &ping(PingSemantic::Retreat), &match_state, None, None),
            Err(AdmissionError::InvalidNonce)
        );

        let (trusted, mut claimed) = trusted_for(1, 1, 9, 3);
        claimed.team_id = 2;
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &ping(PingSemantic::Retreat), &match_state, None, None),
            Err(AdmissionError::IdentityMismatch)
        );
        claimed = ClaimedIdentity { room_id: 8, run_id: 3, player_id: 1, team_id: 1, observer: false };
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &ping(PingSemantic::Retreat), &match_state, None, None),
            Err(AdmissionError::IdentityMismatch)
        );
        claimed.room_id = 9;
        claimed.run_id = 4;
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &ping(PingSemantic::Retreat), &match_state, None, None),
            Err(AdmissionError::IdentityMismatch)
        );
        claimed.run_id = 3;
        claimed.player_id = 3;
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &ping(PingSemantic::Retreat), &match_state, None, None),
            Err(AdmissionError::IdentityMismatch)
        );
        assert_eq!(
            admit_ping(&mut ledger, 99, 1, 2, &match_state),
            Err(AdmissionError::AbsentMembership)
        );
        assert_eq!(
            admit_ping(&mut ledger, 1, 2, 2, &match_state),
            Err(AdmissionError::ForeignMembership)
        );
        match_state.heroes.push(shop_fixture_slot(1, 0, None));
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 2, &match_state),
            Err(AdmissionError::InvalidRoster)
        );
        match_state.heroes.pop();
        match_state.heroes[0].side = 7;
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 2, &match_state),
            Err(AdmissionError::UnresolvedTeam)
        );
        assert_eq!(ledger.history().len(), 1);
    }

    #[test]
    fn communication_team_recipients_are_exact_and_all_chat_is_explicit() {
        let mut ledger = CommunicationLedger::default();
        let match_state = roster_match(MobaMatchPhase::Warmup, None);
        let team = admit_ping(&mut ledger, 1, 1, 1, &match_state).unwrap();
        assert_eq!(team.recipients, vec![1, 2, 5]);
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let all = CommunicationRequest::Chat { channel: ChatChannel::All, text: "all channel".into() };
        let team_text = CommunicationRequest::Chat { channel: ChatChannel::Team, text: "team channel".into() };
        let all_event = ledger.admit(trusted, &claimed, 2, &all, &match_state, None, None).unwrap();
        let team_event = ledger.admit(trusted, &claimed, 3, &team_text, &match_state, None, None).unwrap();
        assert_eq!(all_event.recipients, vec![1, 2, 3, 4, 5]);
        assert_eq!(team_event.recipients, vec![1, 2, 5]);
        assert!(matches!(all_event.body, AdmittedBody::Chat { channel: ChatChannel::All, .. }));
        assert!(matches!(team_event.body, AdmittedBody::Chat { channel: ChatChannel::Team, .. }));
    }

    #[test]
    fn communication_observer_and_results_are_explicit() {
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        let (trusted, mut claimed) = trusted_for(1, 1, 9, 3);
        claimed.observer = true;
        assert_eq!(
            ledger.admit(trusted, &claimed, 1, &ping(PingSemantic::Push), &match_state, None, None),
            Err(AdmissionError::ObserverDenied)
        );
        let (outsider, mut outsider_claim) = trusted_for(50, 1, 9, 3);
        outsider_claim.observer = true;
        assert_eq!(
            ledger.admit(outsider, &outsider_claim, 1, &ping(PingSemantic::Push), &match_state, None, None),
            Err(AdmissionError::ObserverDenied)
        );
        match_state.phase = MobaMatchPhase::Finished { winner: Some(0), tick: 8 };
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 1, &match_state),
            Err(AdmissionError::ResultsClosed)
        );
        assert!(ledger.history().is_empty());
        assert!(ledger.set_mute(trusted, &ClaimedIdentity { observer: false, ..claimed }, &match_state, 3, true).is_ok());
        assert!(ledger.is_muted(9, 3, 1, 3));
    }

    #[test]
    fn communication_rejects_stale_or_hidden_targets_and_out_of_bounds_points() {
        let mut ledger = CommunicationLedger::default();
        let match_state = roster_match(MobaMatchPhase::Playing, Some("three_lane_training"));
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let edge=map.lanes.iter().flat_map(|lane|lane.waypoints.iter()).map(|p|p.1).max().unwrap();
        let inside = Vec2::new(Fixed64::from_i32(0), Fixed64::from_i32(edge));
        let outside = Vec2::new(Fixed64::from_i32(0), Fixed64::from_i32(edge+1));
        assert!(match_state.communication_point_allowed(inside));
        assert!(!match_state.communication_point_allowed(outside));
        assert!(!match_state.communication_point_allowed(Vec2::new(Fixed64::from_i32(-1), Fixed64::ZERO)));
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let mut identity = TeamIdentityState::new(1);
        let canonical = CanonicalEntityKey { id: 10, generation: 2 };
        let mapping = identity.disclose(canonical).unwrap();
        let claim = TargetClaim {
            replica_id: mapping.replica_id.get(),
            canonical_id: canonical.id,
            generation: canonical.generation,
            disclosure_epoch: mapping.disclosure_epoch,
            view_epoch: trusted.view_epoch,
        };
        let request = CommunicationRequest::Ping {
            semantic: PingSemantic::EnemyMissing,
            point: inside,
            target: Some(claim),
            hud: None,
        };
        let admitted = ledger.admit(trusted, &claimed, 1, &request, &match_state, Some(&identity), None).unwrap();
        match &admitted.body {
            AdmittedBody::Ping { target: Some(target), point, .. } => {
                assert_eq!(*point, inside);
                assert_eq!(target.canonical_id, 10);
                assert_eq!(target.generation, 2);
                assert_eq!(target.disclosure_epoch, mapping.disclosure_epoch);
            }
            other => panic!("expected disclosed target, {other:?}"),
        }
        identity.remember(canonical).unwrap();
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &request, &match_state, Some(&identity), None),
            Err(AdmissionError::StaleOrHiddenTarget)
        );
        identity.disclose(canonical).unwrap();
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &request, &match_state, Some(&identity), None),
            Err(AdmissionError::StaleOrHiddenTarget)
        );
        let mut wrong_generation = claim;
        wrong_generation.disclosure_epoch = identity.replica_for(canonical).unwrap().disclosure_epoch;
        wrong_generation.generation = 9;
        let wrong = CommunicationRequest::Ping {
            semantic: PingSemantic::EnemyVision,
            point: inside,
            target: Some(wrong_generation),
            hud: None,
        };
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &wrong, &match_state, Some(&identity), None),
            Err(AdmissionError::StaleOrHiddenTarget)
        );
        let mut stale_view = claim;
        stale_view.disclosure_epoch = identity.replica_for(canonical).unwrap().disclosure_epoch;
        stale_view.view_epoch = 99;
        let stale = CommunicationRequest::Ping {
            semantic: PingSemantic::EnemyVision,
            point: inside,
            target: Some(stale_view),
            hud: None,
        };
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &stale, &match_state, Some(&identity), None),
            Err(AdmissionError::StaleOrHiddenTarget)
        );
        let other_team = TeamIdentityState::new(2);
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &request, &match_state, Some(&other_team), None),
            Err(AdmissionError::DisclosureTeamMismatch)
        );
        let hidden = CommunicationRequest::Ping {
            semantic: PingSemantic::NeedVision,
            point: outside,
            target: Some(claim),
            hud: None,
        };
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &hidden, &match_state, Some(&identity), None),
            Err(AdmissionError::PointOutOfBounds)
        );
        assert_eq!(ledger.history().len(), 1);
    }

    #[test]
    fn communication_text_bounds_unicode_and_controls() {
        let mut ledger = CommunicationLedger::default();
        let match_state = roster_match(MobaMatchPhase::Playing, None);
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let chat = |text: &str| CommunicationRequest::Chat { channel: ChatChannel::Team, text: text.into() };
        let ok = ledger.admit(trusted, &claimed, 1, &chat("撤退 QWER"), &match_state, None, None).unwrap();
        assert!(matches!(ok.body, AdmittedBody::Chat { text, .. } if text == "撤退 QWER"));
        for (text, error) in [
            ("", AdmissionError::EmptyText),
            ("   ", AdmissionError::EmptyText),
            ("line\nbreak", AdmissionError::TextControl),
            ("line\rbreak", AdmissionError::TextControl),
            ("para\u{2028}break", AdmissionError::TextControl),
            ("tab\tbreak", AdmissionError::TextControl),
        ] {
            assert_eq!(
                ledger.admit(trusted, &claimed, 2, &chat(text), &match_state, None, None),
                Err(error),
                "{text:?}"
            );
        }
        let too_many_scalars = "a".repeat(MAX_TEXT_SCALARS + 1);
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &chat(&too_many_scalars), &match_state, None, None),
            Err(AdmissionError::TextLimit)
        );
        let too_many_bytes = "退".repeat(MAX_TEXT_BYTES / 3 + 1);
        assert!(too_many_bytes.chars().count() <= MAX_TEXT_SCALARS);
        assert!(too_many_bytes.len() > MAX_TEXT_BYTES);
        assert_eq!(
            ledger.admit(trusted, &claimed, 2, &chat(&too_many_bytes), &match_state, None, None),
            Err(AdmissionError::TextLimit)
        );
        assert_eq!(ledger.history().len(), 1);
    }

    #[test]
    fn communication_rate_boundary_and_regressed_or_overflow_clock() {
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Playing, Some("three_lane_training"));
        for nonce in 1..=RATE_LIMIT as u64 {
            admit_ping(&mut ledger, 1, 1, nonce, &match_state).unwrap();
        }
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 5, &match_state),
            Err(AdmissionError::RateLimited)
        );
        match_state.elapsed = Fixed64::from_raw(RATE_WINDOW_RAW);
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 6, &match_state),
            Err(AdmissionError::RateLimited)
        );
        match_state.elapsed = Fixed64::from_raw(RATE_WINDOW_RAW + 1);
        admit_ping(&mut ledger, 1, 1, 7, &match_state).unwrap();
        match_state.elapsed = Fixed64::from_raw(RATE_WINDOW_RAW);
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 8, &match_state),
            Err(AdmissionError::RegressedClock)
        );
        assert_eq!(ledger.history().len(), RATE_LIMIT + 1);
        match_state.elapsed = Fixed64::from_raw(i64::MAX - 10);
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let vote = CommunicationRequest::ObjectiveVote { objective_id: "north_guardian".into() };
        assert_eq!(
            ledger.admit(trusted, &claimed, 9, &vote, &match_state, None, None),
            Err(AdmissionError::ClockOverflow)
        );
        assert!(ledger.current_objective_votes(&match_state).is_empty());
        assert_eq!(ledger.history().len(), RATE_LIMIT + 1);
        match_state.elapsed = Fixed64::from_raw(-1);
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 10, &match_state),
            Err(AdmissionError::InvalidClock)
        );
    }

    #[test]
    fn communication_dedup_and_history_are_bounded() {
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        for nonce in 1..=MAX_DEDUP as u64 {
            match_state.elapsed = Fixed64::from_raw((nonce as i64) * (RATE_WINDOW_RAW + 1));
            admit_ping(&mut ledger, 1, 1, nonce, &match_state).unwrap();
        }
        assert_eq!(ledger.history().len(), MAX_HISTORY);
        assert_eq!(ledger.history().front().unwrap().id.sequence, (MAX_DEDUP - MAX_HISTORY + 1) as u64);
        let held = match_state.elapsed;
        assert_eq!(
            admit_ping(&mut ledger, 1, 1, 1, &match_state),
            Err(AdmissionError::Duplicate)
        );
        match_state.elapsed = Fixed64::from_raw(held.raw() + RATE_WINDOW_RAW + 1);
        admit_ping(&mut ledger, 1, 1, (MAX_DEDUP as u64) + 1, &match_state).unwrap();
        match_state.elapsed = Fixed64::from_raw(match_state.elapsed.raw() + RATE_WINDOW_RAW + 1);
        let reused = admit_ping(&mut ledger, 1, 1, 1, &match_state).unwrap();
        assert_eq!(reused.id.sequence, (MAX_DEDUP as u64) + 2);
        assert_eq!(ledger.history().len(), MAX_HISTORY);
    }

    #[test]
    fn communication_mute_is_per_recipient() {
        let mut ledger = CommunicationLedger::default();
        let match_state = roster_match(MobaMatchPhase::Playing, None);
        let (recipient, recipient_claim) = trusted_for(2, 1, 9, 3);
        ledger.set_mute(recipient, &recipient_claim, &match_state, 1, true).unwrap();
        let event = admit_ping(&mut ledger, 1, 1, 1, &match_state).unwrap();
        assert_eq!(event.recipients, vec![1, 5]);
        let (enemy, enemy_claim) = trusted_for(3, 2, 9, 3);
        let enemy_event = ledger.admit(enemy, &enemy_claim, 1, &ping(PingSemantic::OnMyWay), &match_state, None, None).unwrap();
        assert_eq!(enemy_event.recipients, vec![3, 4]);
        let all = CommunicationRequest::Chat { channel: ChatChannel::All, text: "visible to others".into() };
        let (sender, sender_claim) = trusted_for(1, 1, 9, 3);
        let all_event = ledger.admit(sender, &sender_claim, 2, &all, &match_state, None, None).unwrap();
        assert_eq!(all_event.recipients, vec![1, 3, 4, 5]);
        ledger.set_mute(recipient, &recipient_claim, &match_state, 1, false).unwrap();
        let restored = admit_ping(&mut ledger, 1, 1, 3, &match_state).unwrap();
        assert_eq!(restored.recipients, vec![1, 2, 5]);
    }

    #[test]
    fn communication_room_and_run_replacement_isolates_events_and_retires_slot_mutes() {
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Playing, None);
        match_state.elapsed = Fixed64::from_i32(20);
        let first = admit_ping(&mut ledger, 1, 1, 1, &match_state).unwrap();
        assert_eq!((first.id.room_id, first.id.run_id, first.id.sequence), (9, 3, 1));
        let (recipient, recipient_claim) = trusted_for(2, 1, 9, 3);
        ledger.set_mute(recipient, &recipient_claim, &match_state, 1, true).unwrap();
        let (forged, mut forged_claim) = trusted_for(2, 1, 10, 3);
        forged_claim.player_id = 99;
        assert_eq!(
            ledger.set_mute(forged, &forged_claim, &match_state, 1, true),
            Err(AdmissionError::IdentityMismatch)
        );
        assert!(ledger.is_muted(9, 3, 2, 1));
        match_state.elapsed = Fixed64::ZERO;
        let (mut trusted, mut claimed) = trusted_for(1, 1, 10, 3);
        let next_room = ledger.admit(trusted, &claimed, 1, &ping(PingSemantic::Push), &match_state, None, None).unwrap();
        assert_eq!(ledger.history().len(), 1);
        assert_eq!(next_room.id.sequence, 1);
        assert_eq!(next_room.recipients, vec![1, 2, 5]);
        assert!(!ledger.is_muted(10, 3, 2, 1));
        assert!(!ledger.is_muted(9, 3, 2, 1));
        assert_eq!(
            ledger.admit(trusted, &claimed, 1, &ping(PingSemantic::Push), &match_state, None, None),
            Err(AdmissionError::Duplicate)
        );
        trusted.run_id = 4;
        claimed.run_id = 4;
        let next_run = ledger.admit(trusted, &claimed, 1, &ping(PingSemantic::Push), &match_state, None, None).unwrap();
        assert_eq!(next_run.id.run_id, 4);
        assert_eq!(next_run.recipients, vec![1, 2, 5]);
        assert!(ledger.current_objective_votes(&match_state).is_empty());
    }

    #[test]
    fn communication_objective_vote_replaces_and_expires_without_hidden_status() {
        let mut ledger = CommunicationLedger::default();
        let mut match_state = roster_match(MobaMatchPhase::Warmup, Some("three_lane_training"));
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let north = CommunicationRequest::ObjectiveVote { objective_id: "north_guardian".into() };
        assert_eq!(
            ledger.admit(trusted, &claimed, 1, &north, &match_state, None, None),
            Err(AdmissionError::ObjectivePhase)
        );
        match_state.phase = MobaMatchPhase::Playing;
        match_state.jungle_camps.iter_mut().find(|camp| camp.definition.id == "north_guardian").unwrap().returning = true;
        let voted = ledger.admit(trusted, &claimed, 1, &north, &match_state, None, None).unwrap();
        match &voted.body {
            AdmittedBody::ObjectiveVote { objective_id, home, expires_at_raw, .. } => {
                assert_eq!(objective_id, "north_guardian");
                let camp = match_state.jungle_camps.iter().find(|c| c.definition.id == objective_id).unwrap();
                assert_eq!(*home, Vec2::new(Fixed64::from_i32(camp.definition.position.0), Fixed64::from_i32(camp.definition.position.1)));
                assert_eq!(*expires_at_raw, OBJECTIVE_VOTE_TTL_RAW);
            }
            other => panic!("expected vote, {other:?}"),
        }
        assert_eq!(voted.recipients, vec![1, 2, 5]);
        match_state.jungle_camps.retain(|camp| camp.definition.id != "north_guardian");
        assert!(ledger.current_objective_votes(&match_state).is_empty());
        let south = CommunicationRequest::ObjectiveVote { objective_id: "south_guardian".into() };
        ledger.admit(trusted, &claimed, 2, &south, &match_state, None, None).unwrap();
        let (ally, ally_claim) = trusted_for(2, 1, 9, 3);
        ledger.admit(ally, &ally_claim, 1, &south, &match_state, None, None).unwrap();
        let current = ledger.current_objective_votes(&match_state);
        assert_eq!(current.len(), 2);
        assert!(current.iter().all(|vote| vote.objective_id == "south_guardian"));
        match_state.elapsed = Fixed64::from_raw(OBJECTIVE_VOTE_TTL_RAW - 1);
        assert_eq!(ledger.current_objective_votes(&match_state).len(), 2);
        match_state.elapsed = Fixed64::from_raw(OBJECTIVE_VOTE_TTL_RAW);
        assert!(ledger.current_objective_votes(&match_state).is_empty());
        assert_eq!(
            ledger.admit(trusted, &claimed, 3, &CommunicationRequest::ObjectiveVote { objective_id: "hidden\nstate".into() }, &match_state, None, None),
            Err(AdmissionError::ObjectiveUnavailable)
        );
    }

    fn hud_ping(hud: Option<HudClaim>) -> CommunicationRequest {
        CommunicationRequest::Ping {
            semantic: PingSemantic::Assist,
            point: Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            target: None,
            hud,
        }
    }

    fn stored_ability_raw(world: &World, entity: Entity) -> i64 {
        let heroes = world.read_storage::<Hero>();
        let hero = heroes.get(entity).expect("live hero");
        assert_eq!(hero.abilities.first().map(String::as_str), Some("spark"));
        let ability_id = &hero.abilities[0];
        hero.ability_cooldowns.get(ability_id).copied().unwrap_or(Fixed64::ZERO).raw()
    }

    fn stored_item_fact(world: &World, entity: Entity) -> (u16, u32) {
        let inventories = world.read_storage::<Inventory>();
        let item = inventories.get(entity).expect("live inventory").slots[0].as_ref().expect("slot 0");
        assert_ne!(item.cooldown_remaining, 50.0);
        let catalog_id = omoba_template_ids::MOBA_ITEM_CATALOG
            .iter()
            .find(|row| row.id == item.item_id)
            .expect("catalog row")
            .catalog_id;
        (catalog_id, item.cooldown_remaining.to_bits())
    }

    fn ability_remaining(event: &AdmittedCommunication) -> i64 {
        match &event.body {
            AdmittedBody::Ping { hud: Some(HudFact::OwnedAbility { index: 0, remaining_raw }), .. } => *remaining_raw,
            other => panic!("expected ability fact, {other:?}"),
        }
    }

    fn item_fact_of(event: &AdmittedCommunication) -> (u16, u32) {
        match &event.body {
            AdmittedBody::Ping { hud: Some(HudFact::OwnedItem { slot: 0, catalog_id, cooldown_bits }), .. } => (*catalog_id, *cooldown_bits),
            other => panic!("expected item fact, {other:?}"),
        }
    }

    fn advance(match_state: &mut MobaMatch, step: &mut i64) {
        *step += RATE_WINDOW_RAW + 1;
        match_state.elapsed = Fixed64::from_raw(*step);
    }

    fn refuse_hud(
        ledger: &mut CommunicationLedger,
        trusted: TrustedSession,
        claimed: &ClaimedIdentity,
        match_state: &MobaMatch,
        world: &World,
        nonce: u64,
        request: &CommunicationRequest,
        with_facts: bool,
    ) {
        let before = ledger.history().len();
        let result = if with_facts {
            let facts = NativeCommunicationFacts::new(world, match_state);
            ledger.admit(trusted, claimed, nonce, request, match_state, None, Some(&facts))
        } else {
            ledger.admit(trusted, claimed, nonce, request, match_state, None, None)
        };
        assert_eq!(result, Err(AdmissionError::HudFactUnavailable));
        assert_eq!(ledger.history().len(), before);
        assert!(ledger.is_muted(9, 3, 2, 3));
        assert!(!ledger.is_muted(9, 3, 2, 1));
    }

    #[test]
    fn communication_hud_facts_read_live_ecs_and_refuse_stale_or_unavailable() {
        let mut world = World::new();
        world.register::<Hero>();
        world.register::<Inventory>();
        world.register::<PlayerOwner>();
        let catalog = &omoba_template_ids::MOBA_ITEM_CATALOG[0];
        let mut hero = Hero::new("spark_hero".to_string(), "Spark".to_string(), "Test".to_string());
        hero.abilities.push("spark".to_string());
        hero.ability_cooldowns.insert("spark".to_string(), Fixed64::from_i32(5));
        let mut inventory = Inventory::new();
        inventory.slots[0] = Some(ItemInstance { item_id: catalog.id.to_string(), cooldown_remaining: 1.5 });
        let entity = world.create_entity().with(hero).with(inventory).with(PlayerOwner::new(1)).build();

        let mut owner = shop_fixture_slot(1, 0, Some(entity));
        owner.poison_cached_progression_for_test();
        let mut ally = shop_fixture_slot(2, 0, None);
        ally.deaths = 1;
        let mut scored = shop_fixture_slot(3, 1, Some(specs::Entity::new(3, specs::world::Generation::new(1))));
        scored.kills = 6;
        scored.deaths = 1;
        scored.assists = 4;
        let mut match_state = communication_roster_match(
            vec![owner, ally, scored],
            MobaMatchPhase::Playing,
            Fixed64::ZERO,
            None,
        );
        let mut ledger = CommunicationLedger::default();
        let (recipient, recipient_claim) = trusted_for(2, 1, 9, 3);
        ledger.set_mute(recipient, &recipient_claim, &match_state, 3, true).unwrap();
        let (trusted, claimed) = trusted_for(1, 1, 9, 3);
        let mut nonce = 1u64;
        let mut step = 0i64;

        let plain = ledger.admit(trusted, &claimed, nonce, &hud_ping(None), &match_state, None, None).unwrap();
        nonce += 1;
        assert!(matches!(plain.body, AdmittedBody::Ping { hud: None, .. }));
        assert_eq!(plain.recipients, vec![1, 2]);
        advance(&mut match_state, &mut step);
        let chat = CommunicationRequest::Chat { channel: ChatChannel::Team, text: "no hud source".into() };
        let chat_event = ledger.admit(trusted, &claimed, nonce, &chat, &match_state, None, None).unwrap();
        nonce += 1;
        assert!(matches!(chat_event.body, AdmittedBody::Chat { .. }));
        advance(&mut match_state, &mut step);
        let score = hud_ping(Some(HudClaim::PublicScore { player_id: 3 }));
        let score_event = ledger.admit(trusted, &claimed, nonce, &score, &match_state, None, None).unwrap();
        nonce += 1;
        assert!(matches!(
            score_event.body,
            AdmittedBody::Ping { hud: Some(HudFact::PublicScore { player_id: 3, kills: 6, deaths: 1, assists: 4 }), .. }
        ));

        let ability = hud_ping(Some(HudClaim::OwnedAbility { index: 0 }));
        let item = hud_ping(Some(HudClaim::OwnedItem { slot: 0 }));
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &ability, false);
        nonce += 1;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, false);
        nonce += 1;

        advance(&mut match_state, &mut step);
        let ability_event = {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            ledger.admit(trusted, &claimed, nonce, &ability, &match_state, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        let live_ability = stored_ability_raw(&world, entity);
        assert_eq!(ability_remaining(&ability_event), live_ability);
        assert_ne!(live_ability, Fixed64::from_i32(99).raw());
        advance(&mut match_state, &mut step);
        let item_event = {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            ledger.admit(trusted, &claimed, nonce, &item, &match_state, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        let live_item = stored_item_fact(&world, entity);
        assert_eq!(item_fact_of(&item_event), live_item);
        assert_ne!(live_item.1, 50.0f32.to_bits());

        {
            let mut heroes = world.write_storage::<Hero>();
            heroes.get_mut(entity).expect("hero").ability_cooldowns.insert("spark".to_string(), Fixed64::from_i32(2));
        }
        {
            let mut inventories = world.write_storage::<Inventory>();
            inventories.get_mut(entity).expect("inventory").slots[0].as_mut().expect("item").cooldown_remaining = 0.25;
        }
        advance(&mut match_state, &mut step);
        let mutated_ability = {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            ledger.admit(trusted, &claimed, nonce, &ability, &match_state, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        assert_eq!(ability_remaining(&mutated_ability), stored_ability_raw(&world, entity));
        assert_eq!(stored_ability_raw(&world, entity), Fixed64::from_i32(2).raw());
        advance(&mut match_state, &mut step);
        let mutated_item = {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            ledger.admit(trusted, &claimed, nonce, &item, &match_state, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        assert_eq!(item_fact_of(&mutated_item), stored_item_fact(&world, entity));
        assert_eq!(stored_item_fact(&world, entity).1, 0.25f32.to_bits());

        world.write_storage::<Hero>().get_mut(entity).expect("hero").ability_cooldowns.remove("spark");
        advance(&mut match_state, &mut step);
        let ready = {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            ledger.admit(trusted, &claimed, nonce, &ability, &match_state, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        assert_eq!(ability_remaining(&ready), 0);
        assert_eq!(stored_ability_raw(&world, entity), 0);

        world.write_storage::<Hero>().get_mut(entity).expect("hero").ability_cooldowns.insert("spark".to_string(), Fixed64::from_raw(-1));
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &ability, true);
        nonce += 1;
        world.write_storage::<Hero>().get_mut(entity).expect("hero").ability_cooldowns.insert("spark".to_string(), Fixed64::from_i32(4));

        world.write_storage::<Inventory>().get_mut(entity).expect("inventory").slots[0].as_mut().expect("item").cooldown_remaining = f32::NAN;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;
        world.write_storage::<Inventory>().get_mut(entity).expect("inventory").slots[0].as_mut().expect("item").cooldown_remaining = -1.0;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;
        world.write_storage::<Inventory>().get_mut(entity).expect("inventory").slots[0].as_mut().expect("item").item_id = "not-a-catalog-item".to_string();
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;
        let missing_slot = hud_ping(Some(HudClaim::OwnedItem { slot: 1 }));
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &missing_slot, true);
        nonce += 1;
        let missing_ability = hud_ping(Some(HudClaim::OwnedAbility { index: 3 }));
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &missing_ability, true);
        nonce += 1;
        world.write_storage::<Inventory>().get_mut(entity).expect("inventory").slots[0] = Some(ItemInstance {
            item_id: catalog.id.to_string(),
            cooldown_remaining: 0.25,
        });

        world.write_storage::<PlayerOwner>().get_mut(entity).expect("owner").player_id = 99;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &ability, true);
        nonce += 1;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;
        world.write_storage::<PlayerOwner>().get_mut(entity).expect("owner").player_id = 1;

        let retired = specs::Entity::new(entity.id(), specs::world::Generation::new(entity.gen().id() + 1));
        match_state.heroes[0].entity = Some(retired);
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &ability, true);
        nonce += 1;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;
        match_state.heroes[0].entity = Some(entity);

        assert!(world.write_storage::<Hero>().remove(entity).is_some());
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &ability, true);
        nonce += 1;
        advance(&mut match_state, &mut step);
        let item_without_hero = {
            let facts = NativeCommunicationFacts::new(&world, &match_state);
            ledger.admit(trusted, &claimed, nonce, &item, &match_state, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        assert_eq!(item_fact_of(&item_without_hero), stored_item_fact(&world, entity));
        assert!(world.write_storage::<Inventory>().remove(entity).is_some());
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;

        world.delete_entity(entity).expect("delete live entity");
        world.maintain();
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &ability, true);
        nonce += 1;
        refuse_hud(&mut ledger, trusted, &claimed, &match_state, &world, nonce, &item, true);
        nonce += 1;

        let mut dead_owner = shop_fixture_slot(1, 0, None);
        dead_owner.poison_cached_progression_for_test();
        let mut wallet = Inventory::new();
        wallet.slots[0] = Some(ItemInstance { item_id: catalog.id.to_string(), cooldown_remaining: 3.25 });
        dead_owner.install_admitted_dead_wallet_for_test(wallet);
        let mut dead_ally = shop_fixture_slot(2, 0, None);
        dead_ally.deaths = 1;
        let mut dead_match = communication_roster_match(
            vec![dead_owner, dead_ally],
            MobaMatchPhase::Playing,
            Fixed64::from_raw(step),
            None,
        );
        refuse_hud(&mut ledger, trusted, &claimed, &dead_match, &world, nonce, &ability, true);
        nonce += 1;
        advance(&mut dead_match, &mut step);
        let wallet_event = {
            let facts = NativeCommunicationFacts::new(&world, &dead_match);
            ledger.admit(trusted, &claimed, nonce, &item, &dead_match, None, Some(&facts)).unwrap()
        };
        nonce += 1;
        assert_eq!(item_fact_of(&wallet_event), (catalog.catalog_id, 3.25f32.to_bits()));
        assert_ne!(item_fact_of(&wallet_event).1, 50.0f32.to_bits());
        assert_ne!(item_fact_of(&wallet_event).1, 0.25f32.to_bits());

        let mut bare = shop_fixture_slot(1, 0, None);
        bare.poison_cached_progression_for_test();
        let mut bare_ally = shop_fixture_slot(2, 0, None);
        bare_ally.deaths = 1;
        let bare_match = communication_roster_match(
            vec![bare, bare_ally],
            MobaMatchPhase::Playing,
            Fixed64::from_raw(step),
            None,
        );
        refuse_hud(&mut ledger, trusted, &claimed, &bare_match, &world, nonce, &ability, true);
        nonce += 1;
        refuse_hud(&mut ledger, trusted, &claimed, &bare_match, &world, nonce, &item, true);
        assert!(ledger.is_muted(9, 3, 2, 3));
    }

    #[test]
    fn communication_mute_scope_is_room_run_scoped_and_budgets_are_independent() {
        let wide = communication_roster_match(
            (1..=33).map(|id| {
                let side = usize::from(id == 2);
                shop_fixture_slot(id, side, Some(specs::Entity::new(id, specs::world::Generation::new(1))))
            }).collect(),
            MobaMatchPhase::Playing,
            Fixed64::ZERO,
            None,
        );
        let mut budgets = CommunicationLedger::default();
        let (recipient, recipient_claim) = trusted_for(1, 1, 9, 3);
        for sender in 2..=17 {
            budgets.set_mute(recipient, &recipient_claim, &wide, sender, true).unwrap();
            assert!(budgets.is_muted(9, 3, 1, sender));
        }
        budgets.set_mute(recipient, &recipient_claim, &wide, 2, true).unwrap();
        assert_eq!(
            budgets.set_mute(recipient, &recipient_claim, &wide, 18, true),
            Err(AdmissionError::MuteCapacity)
        );
        assert!(!budgets.is_muted(9, 3, 1, 18));
        let (other, other_claim) = trusted_for(2, 2, 9, 3);
        budgets.set_mute(other, &other_claim, &wide, 1, true).unwrap();
        assert!(budgets.is_muted(9, 3, 2, 1));
        assert_eq!(
            budgets.set_mute(recipient, &recipient_claim, &wide, 18, true),
            Err(AdmissionError::MuteCapacity)
        );
        assert_eq!(
            budgets.set_mute(recipient, &recipient_claim, &wide, 0, true),
            Err(AdmissionError::MuteUnauthorized)
        );
        assert_eq!(
            budgets.set_mute(recipient, &recipient_claim, &wide, 99, true),
            Err(AdmissionError::MuteUnauthorized)
        );
        assert!(!budgets.is_muted(9, 3, 1, 18));
        assert!(budgets.is_muted(9, 3, 1, 17));
        assert!(budgets.is_muted(9, 3, 2, 1));

        let mut recipients = CommunicationLedger::default();
        for id in 1..=32 {
            let team = if id == 2 { 2 } else { 1 };
            let (trusted, claimed) = trusted_for(id, team, 9, 3);
            recipients.set_mute(trusted, &claimed, &wide, 33, true).unwrap();
        }
        let (blocked, blocked_claim) = trusted_for(33, 1, 9, 3);
        assert_eq!(
            recipients.set_mute(blocked, &blocked_claim, &wide, 1, true),
            Err(AdmissionError::MuteCapacity)
        );
        assert!(!recipients.is_muted(9, 3, 33, 1));
        let (existing, existing_claim) = trusted_for(3, 1, 9, 3);
        recipients.set_mute(existing, &existing_claim, &wide, 1, true).unwrap();
        assert!(recipients.is_muted(9, 3, 3, 1));
        assert!(recipients.is_muted(9, 3, 32, 33));

        let mut ledger = CommunicationLedger::default();
        let match_state = roster_match(MobaMatchPhase::Playing, None);
        let (recipient, recipient_claim) = trusted_for(2, 1, 9, 3);
        ledger.set_mute(recipient, &recipient_claim, &match_state, 1, true).unwrap();
        assert!(ledger.history().is_empty());
        assert!(ledger.is_muted(9, 3, 2, 1));
        let first = admit_ping(&mut ledger, 1, 1, 1, &match_state).unwrap();
        assert_eq!(first.recipients, vec![1, 5]);
        let mut reconnect = match_state.clone();
        reconnect.elapsed = Fixed64::from_raw(RATE_WINDOW_RAW + 1);
        let (sender, sender_claim) = trusted_for(1, 1, 9, 3);
        let second = ledger.admit(sender, &sender_claim, 2, &ping(PingSemantic::Assist), &reconnect, None, None).unwrap();
        assert_eq!(second.recipients, vec![1, 5]);
        assert_eq!(ledger.history().len(), 2);
        let (forged, mut forged_claim) = trusted_for(2, 1, 10, 3);
        forged_claim.player_id = 99;
        assert_eq!(
            ledger.set_mute(forged, &forged_claim, &match_state, 1, true),
            Err(AdmissionError::IdentityMismatch)
        );
        assert_eq!(
            ledger.set_mute(recipient, &recipient_claim, &match_state, 99, true),
            Err(AdmissionError::MuteUnauthorized)
        );
        assert!(ledger.is_muted(9, 3, 2, 1));
        assert_eq!(ledger.history().len(), 2);
        let (next_room, next_claim) = trusted_for(2, 1, 10, 4);
        ledger.set_mute(next_room, &next_claim, &match_state, 1, true).unwrap();
        assert!(!ledger.is_muted(9, 3, 2, 1));
        assert!(ledger.is_muted(10, 4, 2, 1));
        assert_eq!(ledger.history().len(), 2);
        let mut same_room = reconnect.clone();
        same_room.elapsed = Fixed64::from_raw((RATE_WINDOW_RAW + 1) * 2);
        let still_here = ledger.admit(sender, &sender_claim, 3, &ping(PingSemantic::Assist), &same_room, None, None).unwrap();
        assert_eq!(still_here.recipients, vec![1, 2, 5]);
        assert!(ledger.is_muted(10, 4, 2, 1));
        assert_eq!(ledger.history().len(), 3);
        let mut replaced = match_state.clone();
        replaced.elapsed = Fixed64::ZERO;
        let (moved, moved_claim) = trusted_for(1, 1, 10, 4);
        let entered = ledger.admit(moved, &moved_claim, 1, &ping(PingSemantic::Push), &replaced, None, None).unwrap();
        assert_eq!(entered.recipients, vec![1, 5]);
        assert_eq!(ledger.history().len(), 1);
        assert!(ledger.is_muted(10, 4, 2, 1));
        assert!(!ledger.is_muted(9, 3, 2, 1));
        let (cleared, cleared_claim) = trusted_for(2, 1, 12, 1);
        ledger.set_mute(cleared, &cleared_claim, &match_state, 1, false).unwrap();
        assert!(!ledger.is_muted(10, 4, 2, 1));
        assert!(!ledger.is_muted(12, 1, 2, 1));
        assert_eq!(ledger.history().len(), 1);
        ledger.set_mute(cleared, &cleared_claim, &match_state, 1, true).unwrap();
        let (wrong, mut wrong_claim) = trusted_for(2, 1, 13, 1);
        wrong_claim.room_id = 14;
        assert_eq!(
            ledger.set_mute(wrong, &wrong_claim, &match_state, 1, true),
            Err(AdmissionError::IdentityMismatch)
        );
        assert_eq!(
            ledger.admit(cleared, &cleared_claim, 0, &ping(PingSemantic::Assist), &match_state, None, None),
            Err(AdmissionError::InvalidNonce)
        );
        assert!(ledger.is_muted(12, 1, 2, 1));
        assert_eq!(ledger.history().len(), 1);
        let (speaker, speaker_claim) = trusted_for(1, 1, 12, 1);
        let bound = ledger.admit(speaker, &speaker_claim, 1, &ping(PingSemantic::Assist), &replaced, None, None).unwrap();
        assert_eq!(bound.id.room_id, 12);
        assert_eq!(bound.recipients, vec![1, 5]);
        assert!(ledger.is_muted(12, 1, 2, 1));
        assert!(!ledger.is_muted(10, 4, 2, 1));
    }
}
