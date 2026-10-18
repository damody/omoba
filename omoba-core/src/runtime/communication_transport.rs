//! Authenticated communication wire, authority admission, and client inbox.
//!
//! Negotiation is tags `0x2F` and `0x30`. Submits, receipts, events, and replay
//! use `0x31`..`0x35`. None of those tags is RTT, shop, or owner ward. Receipts
//! are not `CriticalInputResult`. The ledger in [`super::communication`] remains
//! the only admission implementation. This module does not keep a roster.

use std::collections::{BTreeMap, VecDeque};

use prost::Message;
use specs::World;

use crate::game_proto::{
    CommunicationHello, CommunicationHelloResult, CommunicationPresentation, CommunicationReceipt,
    CommunicationReplayBatch, CommunicationReplayQuery, CommunicationSubmit, CommunicationWireEvent,
};
use crate::wire_tags::{
    TAG_COMMUNICATION_EVENT, TAG_COMMUNICATION_HELLO, TAG_COMMUNICATION_HELLO_RESULT,
    TAG_COMMUNICATION_RECEIPT, TAG_COMMUNICATION_REPLAY, TAG_COMMUNICATION_REPLAY_QUERY,
    TAG_COMMUNICATION_SUBMIT, TAG_OWNER_WARD_ENVELOPE, TAG_OWNER_WARD_RECOVERY_QUERY,
    TAG_PING_REQ, TAG_PING_RESP, TAG_SHOP_RECEIPT_QUERY, TAG_SHOP_RECEIPT_REPLAY,
};

use super::communication::{
    AdmissionError, AdmittedBody, AdmittedCommunication, ChatChannel, ClaimedIdentity,
    CommunicationAudience, CommunicationLedger, CommunicationRequest, HudClaim, HudFact,
    NativeCommunicationFacts, PingSemantic, TargetClaim, TrustedSession, MAX_HISTORY,
    MAX_OBJECTIVE_ID_BYTES, MAX_TEXT_BYTES, MAX_TEXT_SCALARS,
};
use super::native::moba_match::MobaMatch;
use super::selective::{MappingVisibility, TeamIdentityError, TeamIdentityState};
use omoba_sim::{Fixed64, Vec2};

pub const COMMUNICATION_PROTOCOL_VERSION: u32 = 3;
pub const COMMUNICATION_SCHEMA_VERSION: u32 = 3;
/// Bounded cross-thread queue. A full queue refuses before admission.
pub const COMMUNICATION_AUTHORITY_QUEUE: usize = 64;
/// Requests drained from that queue on one authority tick.
pub const COMMUNICATION_AUTHORITY_DRAIN_PER_TICK: usize = 32;
/// One hello may be outstanding. A newer connection, view, or first live tick
/// coalesces into one follow-up. Identical ticks do not poll.
pub const COMMUNICATION_HELLO_ATTEMPTS: u8 = 1;
/// Settled request ids remembered for one client binding.
pub const COMMUNICATION_SETTLED_WINDOW: usize = 64;
/// Nonces older than this distance from the highest accepted nonce are retired.
pub const COMMUNICATION_REQUEST_WINDOW: u64 = 64;
pub const COMMUNICATION_PENDING_SUBMISSIONS: usize = 8;

pub const HELLO_MAX_BYTES: usize = 96;
pub const SUBMIT_MAX_BYTES: usize = 512;
pub const REPLAY_QUERY_MAX_BYTES: usize = 96;
pub const RECEIPT_MAX_BYTES: usize = 96;
pub const EVENT_MAX_BYTES: usize = 768;
pub const REPLAY_BATCH_MAX_BYTES: usize = 24 * 1024;

pub const KIND_PING: u32 = 1;
pub const KIND_CHAT: u32 = 2;
pub const KIND_OBJECTIVE: u32 = 3;
pub const KIND_MUTE: u32 = 4;
pub const KIND_EMOTE: u32 = 5;
pub const KIND_STATUS: u32 = 6;
pub const CHANNEL_TEAM: u32 = 1;
pub const CHANNEL_ALL: u32 = 2;
pub const AUDIENCE_TEAM: u32 = 1;
pub const AUDIENCE_ALL: u32 = 2;
pub const HUD_NONE: u32 = 0;
pub const HUD_ABILITY: u32 = 1;
pub const HUD_ITEM: u32 = 2;
pub const HUD_SCORE: u32 = 3;

pub const OUTCOME_ADMITTED_EVENT: u32 = 1;
pub const OUTCOME_MUTE_UPDATED: u32 = 2;
pub const OUTCOME_REFUSED: u32 = 3;
pub const OUTCOME_REPLAY: u32 = 4;
pub const OUTCOME_HELLO: u32 = 5;

pub const REFUSAL_ADMITTED: u32 = 0;
pub const REFUSAL_INCOMPLETE_SESSION: u32 = 1;
pub const REFUSAL_IDENTITY_MISMATCH: u32 = 2;
pub const REFUSAL_OBSERVER_DENIED: u32 = 3;
pub const REFUSAL_UNSUPPORTED: u32 = 4;
pub const REFUSAL_UNAUTHENTICATED: u32 = 5;
pub const REFUSAL_MALFORMED: u32 = 6;
pub const REFUSAL_OVERSIZED: u32 = 7;
pub const REFUSAL_NOT_NEGOTIATED: u32 = 8;
pub const REFUSAL_BACKPRESSURE: u32 = 9;
pub const REFUSAL_INVALID_NONCE: u32 = 10;
pub const REFUSAL_INVALID_CLOCK: u32 = 11;
pub const REFUSAL_REGRESSED_CLOCK: u32 = 12;
pub const REFUSAL_CLOCK_OVERFLOW: u32 = 13;
pub const REFUSAL_INVALID_ROSTER: u32 = 14;
pub const REFUSAL_UNRESOLVED_TEAM: u32 = 15;
pub const REFUSAL_ABSENT_MEMBERSHIP: u32 = 16;
pub const REFUSAL_FOREIGN_MEMBERSHIP: u32 = 17;
pub const REFUSAL_RESULTS_CLOSED: u32 = 18;
pub const REFUSAL_EMPTY_TEXT: u32 = 19;
pub const REFUSAL_TEXT_LIMIT: u32 = 20;
pub const REFUSAL_TEXT_CONTROL: u32 = 21;
pub const REFUSAL_POINT_OUT_OF_BOUNDS: u32 = 22;
pub const REFUSAL_STALE_OR_HIDDEN_TARGET: u32 = 23;
pub const REFUSAL_DISCLOSURE_TEAM_MISMATCH: u32 = 24;
pub const REFUSAL_HUD_FACT_UNAVAILABLE: u32 = 25;
pub const REFUSAL_OBJECTIVE_UNAVAILABLE: u32 = 26;
pub const REFUSAL_OBJECTIVE_PHASE: u32 = 27;
pub const REFUSAL_OBJECTIVE_CAPACITY: u32 = 28;
pub const REFUSAL_DUPLICATE: u32 = 29;
pub const REFUSAL_RATE_LIMITED: u32 = 30;
pub const REFUSAL_SEQUENCE_EXHAUSTED: u32 = 31;
pub const REFUSAL_MUTE_CAPACITY: u32 = 32;
pub const REFUSAL_MUTE_UNAUTHORIZED: u32 = 33;
pub const REFUSAL_EMOTE_CONSUMER_MISSING: u32 = 34;
pub const REFUSAL_STATUS_CONSUMER_MISSING: u32 = 35;
pub const REFUSAL_ALREADY_REPLAYED: u32 = 36;
pub const REFUSAL_SCOPE_UNAVAILABLE: u32 = 37;
pub const REFUSAL_VIEW_MISMATCH: u32 = 38;
pub const REFUSAL_CONTENT_UNAVAILABLE: u32 = 39;
pub const REFUSAL_LOCAL_TRANSPORT: u32 = 40;

/// Server match identity. Room is the authoritative master seed. Run starts at 1
/// and advances only when that seed changes or the host retires the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationScope {
    pub room_id: u64,
    pub run_id: u64,
}

impl CommunicationScope {
    pub fn from_seed_and_generation(seed: u64, generation: u64) -> Option<Self> {
        if seed == 0 || generation == 0 {
            None
        } else {
            Some(Self { room_id: seed, run_id: generation })
        }
    }
}

/// Identity copied from the authenticated session and the published scope.
/// Payload fields never construct this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundCommunicationSession {
    pub secure_joined: bool,
    pub negotiated: bool,
    pub observer: bool,
    pub room_id: u64,
    pub run_id: u64,
    pub player_id: u32,
    pub team_id: u32,
    pub view_epoch: u64,
}

/// Replica identity a filtered client can already see. It has no canonical id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientVisibleTarget {
    pub replica_id: u64,
    pub generation: u32,
    pub disclosure_epoch: u64,
    pub view_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodedBody {
    Ping {
        semantic: PingSemantic,
        point: Vec2,
        target: Option<ClientVisibleTarget>,
        hud: Option<HudClaim>,
    },
    Chat { channel: ChatChannel, text: String },
    Objective { objective_id: String },
    Mute { sender_id: u32, muted: bool },
    Emote { content_id: u16 },
    Status { hud: HudClaim },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedSubmit {
    pub request_id: u64,
    pub nonce: u64,
    pub claimed: ClaimedIdentity,
    pub claimed_view_epoch: u64,
    pub body: DecodedBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommunicationRequestBody {
    Submit(DecodedSubmit),
    Replay { request_id: u64, claimed: ClaimedIdentity, claimed_view_epoch: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommunicationRequestMsg {
    pub origin_session: String,
    pub lifetime: CommunicationLifetime,
    pub bound: BoundCommunicationSession,
    pub body: CommunicationRequestBody,
}

/// Identity of one authenticated connection. Generation 0 is absent and fails closed.
/// Values are never wrapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationLifetime {
    pub generation: u64,
    pub binding_generation: u64,
    pub room_id: u64,
    pub run_id: u64,
    pub player_id: u32,
    pub team_id: u32,
    pub view_epoch: u64,
    pub scope_generation: u64,
}

/// One authorized recipient payload. Bytes are not shared across recipients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommunicationRecipientFrame {
    pub player_id: u32,
    pub team_id: u32,
    pub view_epoch: u64,
    pub session_generation: u64,
    pub bytes: Vec<u8>,
    pub expires_at_raw: i64,
    pub admitted_at_raw: i64,
    pub kind: u32,
}

/// Team disclosure available while encoding. The network thread never receives this.
#[derive(Clone, Copy)]
pub struct TeamViewDisclosure<'a> {
    pub team_id: u32,
    pub view_epoch: u64,
    pub identity: Option<&'a TeamIdentityState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommunicationDelivery {
    pub origin_session: String,
    pub origin_lifetime: CommunicationLifetime,
    pub receipt: Vec<u8>,
    pub recipient_frames: Vec<CommunicationRecipientFrame>,
    pub recipients: Vec<u32>,
    pub audience: u32,
    pub sender_team: u32,
    pub sender_id: u32,
    pub room_id: u64,
    pub run_id: u64,
    pub view_epoch: u64,
    pub scope_generation: u64,
    pub elapsed_raw: i64,
    pub elapsed_known: bool,
    pub replay: Option<Vec<u8>>,
    pub consume_replay: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationRouteSession<'a> {
    pub session_id: &'a str,
    pub player_id: u32,
    pub team_id: u32,
    pub view_epoch: u64,
    pub room_id: u64,
    pub run_id: u64,
    pub generation: u64,
    pub negotiated: bool,
    pub secure_joined: bool,
    pub observer: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CommunicationAuthority {
    ledger: CommunicationLedger,
    scope: Option<CommunicationScope>,
}

impl CommunicationAuthority {
    pub fn scope(&self) -> Option<CommunicationScope> {
        self.scope
    }

    pub fn set_scope(&mut self, scope: Option<CommunicationScope>) {
        self.scope = scope;
    }

    /// Production admission. Checks the live session token and the stream window
    /// before the ledger, then commits a nonce only after a new event is stored.
    pub fn admit_live(
        &mut self,
        registry: &mut CommunicationSessionRegistry,
        request: &CommunicationRequestMsg,
        match_state: &MobaMatch,
        world: Option<&World>,
        disclosure: Option<&TeamIdentityState>,
        current_view_epoch: u64,
        views: &[TeamViewDisclosure<'_>],
    ) -> CommunicationDelivery {
        let request_id = match &request.body {
            CommunicationRequestBody::Submit(submit) => submit.request_id,
            CommunicationRequestBody::Replay { request_id, .. } => *request_id,
        };
        if let Err(refusal) = registry.authorize(request) {
            return refused(request, request_id, refusal, false);
        }
        let nonce = match &request.body {
            CommunicationRequestBody::Submit(submit) => match submit.body {
                DecodedBody::Ping { .. } | DecodedBody::Chat { .. } | DecodedBody::Objective { .. }
                | DecodedBody::Emote { .. } | DecodedBody::Status { .. } => Some(submit.nonce),
                _ => None,
            },
            CommunicationRequestBody::Replay { .. } => None,
        };
        if let Some(nonce) = nonce {
            if let Err(refusal) = registry.preview_nonce(request.bound.room_id, request.bound.run_id, request.bound.player_id, nonce) {
                return refused(request, request_id, refusal, false);
            }
        }
        let cursor = self.ledger.admission_cursor();
        let player_epochs = registry.player_epochs(request.bound.room_id, request.bound.run_id);
        let delivery = self.handle_addressed(
            request,
            match_state,
            world,
            disclosure,
            current_view_epoch,
            views,
            &player_epochs,
        );
        if let Some(nonce) = nonce {
            if self.ledger.admission_cursor() != cursor {
                let _ = registry.commit_nonce(request.bound.room_id, request.bound.run_id, request.bound.player_id, nonce);
            }
        }
        delivery
    }

    pub fn ledger(&self) -> &CommunicationLedger {
        &self.ledger
    }

    /// No match resource is present, so nothing is admitted and nothing is replayed.
    pub fn refuse_without_match(&self, request: &CommunicationRequestMsg) -> CommunicationDelivery {
        let request_id = match &request.body {
            CommunicationRequestBody::Submit(submit) => submit.request_id,
            CommunicationRequestBody::Replay { request_id, .. } => *request_id,
        };
        refused(request, request_id, REFUSAL_SCOPE_UNAVAILABLE, false)
    }

    pub fn handle(
        &mut self,
        request: &CommunicationRequestMsg,
        match_state: &MobaMatch,
        world: Option<&World>,
        disclosure: Option<&TeamIdentityState>,
        current_view_epoch: u64,
    ) -> CommunicationDelivery {
        self.handle_addressed(request, match_state, world, disclosure, current_view_epoch, &[], &[])
    }

    /// Admission plus per-recipient encoding. `player_epochs` is
    /// `(player_id, view_epoch, session_generation)` captured from the live registry.
    pub fn handle_addressed(
        &mut self,
        request: &CommunicationRequestMsg,
        match_state: &MobaMatch,
        world: Option<&World>,
        disclosure: Option<&TeamIdentityState>,
        current_view_epoch: u64,
        views: &[TeamViewDisclosure<'_>],
        player_epochs: &[(u32, u64, u64)],
    ) -> CommunicationDelivery {
        match &request.body {
            CommunicationRequestBody::Submit(submit) => self.submit(
                request,
                submit,
                match_state,
                world,
                disclosure,
                current_view_epoch,
                views,
                player_epochs,
            ),
            CommunicationRequestBody::Replay { request_id, claimed, claimed_view_epoch } => self.replay(
                request,
                *request_id,
                claimed,
                *claimed_view_epoch,
                match_state,
                disclosure,
                current_view_epoch,
            ),
        }
    }

    fn submit(
        &mut self,
        request: &CommunicationRequestMsg,
        submit: &DecodedSubmit,
        match_state: &MobaMatch,
        world: Option<&World>,
        disclosure: Option<&TeamIdentityState>,
        current_view_epoch: u64,
        views: &[TeamViewDisclosure<'_>],
        player_epochs: &[(u32, u64, u64)],
    ) -> CommunicationDelivery {
        let bound = request.bound;
        if let Some(refusal) = session_gate(&bound, true) {
            return refused(request, submit.request_id, refusal, false);
        }
        if self.scope != Some(CommunicationScope { room_id: bound.room_id, run_id: bound.run_id }) {
            return refused(request, submit.request_id, REFUSAL_SCOPE_UNAVAILABLE, false);
        }
        if bound.view_epoch != current_view_epoch
            || submit.claimed_view_epoch == 0
            || submit.claimed_view_epoch != bound.view_epoch
            || submit.claimed_view_epoch != current_view_epoch
        {
            return refused(request, submit.request_id, REFUSAL_VIEW_MISMATCH, false);
        }
        if submit.claimed.room_id != bound.room_id
            || submit.claimed.run_id != bound.run_id
            || submit.claimed.player_id != bound.player_id
            || submit.claimed.team_id != bound.team_id
            || submit.claimed.observer != bound.observer
        {
            return refused(request, submit.request_id, REFUSAL_IDENTITY_MISMATCH, false);
        }
        let trusted = trusted_from(&bound);
        match &submit.body {
            DecodedBody::Emote { content_id } => self.admit_request(
                request, submit, &CommunicationRequest::Emote { content_id: *content_id },
                match_state, world, disclosure, views, player_epochs,
            ),
            DecodedBody::Status { hud } => self.admit_request(
                request, submit, &CommunicationRequest::Status { hud: *hud },
                match_state, world, disclosure, views, player_epochs,
            ),
            DecodedBody::Mute { sender_id, muted } => {
                match self.ledger.set_mute(trusted, &submit.claimed, match_state, *sender_id, *muted) {
                    Ok(()) => receipt_delivery(
                        request,
                        receipt(submit.request_id, REFUSAL_ADMITTED, OUTCOME_MUTE_UPDATED, &bound, 0),
                        Vec::new(),
                        Vec::new(),
                        0,
                        false,
                        match_state.elapsed.raw(),
                        match_state.elapsed.raw() >= 0,
                    ),
                    Err(error) => refused(request, submit.request_id, refusal_from(error), false),
                }
            }
            DecodedBody::Ping { semantic, point, target, hud } => {
                let target = match authorize_filtered_target(
                    *target,
                    disclosure,
                    request.bound.team_id,
                    current_view_epoch,
                ) {
                    Ok(target) => target,
                    Err(refusal) => return refused(request, submit.request_id, refusal, false),
                };
                self.admit_request(
                    request,
                    submit,
                    &CommunicationRequest::Ping {
                        semantic: *semantic,
                        point: *point,
                        target,
                        hud: *hud,
                    },
                    match_state,
                    world,
                    disclosure,
                    views,
                    player_epochs,
                )
            }
            DecodedBody::Chat { channel, text } => self.admit_request(
                request,
                submit,
                &CommunicationRequest::Chat { channel: *channel, text: text.clone() },
                match_state,
                world,
                disclosure,
                views,
                player_epochs,
            ),
            DecodedBody::Objective { objective_id } => self.admit_request(
                request,
                submit,
                &CommunicationRequest::ObjectiveVote { objective_id: objective_id.clone() },
                match_state,
                world,
                disclosure,
                views,
                player_epochs,
            ),
        }
    }

    fn admit_request(
        &mut self,
        request: &CommunicationRequestMsg,
        submit: &DecodedSubmit,
        body: &CommunicationRequest,
        match_state: &MobaMatch,
        world: Option<&World>,
        disclosure: Option<&TeamIdentityState>,
        views: &[TeamViewDisclosure<'_>],
        player_epochs: &[(u32, u64, u64)],
    ) -> CommunicationDelivery {
        let trusted = trusted_from(&request.bound);
        let admitted = if let Some(world) = world {
            let facts = NativeCommunicationFacts::new(world, match_state);
            self.ledger.admit(
                trusted,
                &submit.claimed,
                submit.nonce,
                body,
                match_state,
                disclosure,
                Some(&facts),
            )
        } else {
            self.ledger.admit(
                trusted,
                &submit.claimed,
                submit.nonce,
                body,
                match_state,
                disclosure,
                None,
            )
        };
        match admitted {
            Ok(event) => {
                let roster = match_state.communication_roster().ok();
                let sender_view = TeamViewDisclosure {
                    team_id: request.bound.team_id,
                    view_epoch: request.bound.view_epoch,
                    identity: disclosure,
                };
                let mut addressed = vec![sender_view];
                for view in views {
                    if view.team_id != sender_view.team_id && view.view_epoch != 0 {
                        addressed.push(*view);
                    }
                }
                let frames = recipient_frames(
                    &event,
                    submit.request_id,
                    request.bound.view_epoch,
                    &addressed,
                    player_epochs,
                    roster.as_deref(),
                );
                if frames.is_empty() {
                    return refused(request, submit.request_id, REFUSAL_MALFORMED, false);
                }
                receipt_delivery(
                    request,
                    receipt(
                        submit.request_id,
                        REFUSAL_ADMITTED,
                        OUTCOME_ADMITTED_EVENT,
                        &request.bound,
                        event.id.sequence,
                    ),
                    frames,
                    event.recipients,
                    audience_code(event.audience),
                    false,
                    match_state.elapsed.raw(),
                    true,
                )
            }
            Err(error) => refused(request, submit.request_id, refusal_from(error), false),
        }
    }

    fn replay(
        &mut self,
        request: &CommunicationRequestMsg,
        request_id: u64,
        claimed: &ClaimedIdentity,
        claimed_view_epoch: u64,
        match_state: &MobaMatch,
        disclosure: Option<&TeamIdentityState>,
        current_view_epoch: u64,
    ) -> CommunicationDelivery {
        let bound = request.bound;
        if let Some(refusal) = session_gate(&bound, true) {
            return refused(request, request_id, refusal, false);
        }
        if self.scope != Some(CommunicationScope { room_id: bound.room_id, run_id: bound.run_id })
            || claimed.room_id != bound.room_id
            || claimed.run_id != bound.run_id
            || claimed.player_id != bound.player_id
            || claimed.team_id != bound.team_id
            || claimed.observer != bound.observer
        {
            return refused(request, request_id, REFUSAL_IDENTITY_MISMATCH, false);
        }
        if bound.view_epoch != current_view_epoch
            || claimed_view_epoch == 0
            || claimed_view_epoch != bound.view_epoch
            || claimed_view_epoch != current_view_epoch
        {
            return refused(request, request_id, REFUSAL_VIEW_MISMATCH, false);
        }
        match self.ledger.replay_for(trusted_from(&bound), claimed, match_state, disclosure) {
            Ok(events) => {
                let mut wire = Vec::new();
                for event in events {
                    let Some(encoded) = wire_event(&event, bound.view_epoch, 0) else {
                        return refused(request, request_id, REFUSAL_MALFORMED, false);
                    };
                    wire.push(encoded);
                }
                let elapsed_raw = match_state.elapsed.raw();
                if elapsed_raw < 0 {
                    return refused(request, request_id, REFUSAL_INVALID_CLOCK, false);
                }
                let batch = CommunicationReplayBatch {
                    schema_version: COMMUNICATION_SCHEMA_VERSION,
                    request_id,
                    refusal: REFUSAL_ADMITTED,
                    room_id: bound.room_id,
                    run_id: bound.run_id,
                    view_epoch: bound.view_epoch,
                    already_replayed: false,
                    events: wire,
                    elapsed_raw,
                };
                let bytes = batch.encode_to_vec();
                if bytes.len() > REPLAY_BATCH_MAX_BYTES {
                    return refused(request, request_id, REFUSAL_OVERSIZED, false);
                }
                let mut delivery = receipt_delivery(
                    request,
                    receipt(request_id, REFUSAL_ADMITTED, OUTCOME_REPLAY, &bound, 0),
                    Vec::new(),
                    Vec::new(),
                    0,
                    true,
                    elapsed_raw,
                    true,
                );
                delivery.replay = Some(bytes);
                delivery
            }
            Err(error) => refused(request, request_id, refusal_from(error), false),
        }
    }
}

/// Admission seam used by `State::drain_authenticated_communication`.
/// A missing match, a missing published scope, or a stale lifetime refuses
/// before the ledger. The caller supplies disclosures it already cloned.
pub fn admit_authenticated(
    authority: &mut CommunicationAuthority,
    registry: &mut CommunicationSessionRegistry,
    request: &CommunicationRequestMsg,
    match_state: Option<&MobaMatch>,
    world: Option<&World>,
    disclosure: Option<&TeamIdentityState>,
    current_view_epoch: u64,
    views: &[TeamViewDisclosure<'_>],
) -> CommunicationDelivery {
    let request_id = match &request.body {
        CommunicationRequestBody::Submit(submit) => submit.request_id,
        CommunicationRequestBody::Replay { request_id, .. } => *request_id,
    };
    let Some(match_state) = match_state else {
        let refusal = registry.authorize(request).err().unwrap_or(REFUSAL_SCOPE_UNAVAILABLE);
        return refused(request, request_id, refusal, false);
    };
    if registry.published_scope().is_none() {
        let refusal = registry.authorize(request).err().unwrap_or(REFUSAL_SCOPE_UNAVAILABLE);
        return refused(request, request_id, refusal, false);
    }
    authority.admit_live(registry, request, match_state, world, disclosure, current_view_epoch, views)
}

enum ScopeCounter {
    Generation,
    Binding,
    Scope,
}

#[derive(Clone, Debug)]
struct SessionSlot {
    lifetime: CommunicationLifetime,
    negotiated: bool,
    observer: bool,
    secure: bool,
    replay_consumed: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct NonceStream {
    floor: u64,
    highest: u64,
    bits: u64,
}

/// Shared current-session table. The network thread and the authority tick
/// both hold this through `std::sync::Mutex`. Neither copies a World.
#[derive(Clone, Debug, Default)]
pub struct CommunicationSessionRegistry {
    next_generation: u64,
    next_binding: u64,
    next_scope_generation: u64,
    poisoned: bool,
    sessions: BTreeMap<String, SessionSlot>,
    streams: BTreeMap<(u64, u64, u32), NonceStream>,
    published_scope: Option<CommunicationScope>,
    published_scope_generation: u64,
    elapsed_raw: i64,
    elapsed_known: bool,
}

impl CommunicationSessionRegistry {
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub fn published_scope_generation(&self) -> u64 {
        self.published_scope_generation
    }

    pub fn published_scope(&self) -> Option<CommunicationScope> {
        if self.poisoned { None } else { self.published_scope }
    }

    pub fn elapsed(&self) -> (i64, bool) {
        (self.elapsed_raw, self.elapsed_known)
    }

    fn allocate_field(&mut self, field: ScopeCounter) -> Option<u64> {
        if self.poisoned {
            return None;
        }
        let next = match field {
            ScopeCounter::Generation => &mut self.next_generation,
            ScopeCounter::Binding => &mut self.next_binding,
            ScopeCounter::Scope => &mut self.next_scope_generation,
        };
        let value = next.checked_add(1)?;
        *next = value;
        Some(value)
    }

    pub fn open_session(&mut self, session_id: &str) -> Option<CommunicationLifetime> {
        self.sessions.remove(session_id);
        let generation = self.allocate_field(ScopeCounter::Generation)?;
        let binding_generation = self.allocate_field(ScopeCounter::Binding)?;
        let lifetime = CommunicationLifetime {
            generation,
            binding_generation,
            room_id: 0,
            run_id: 0,
            player_id: 0,
            team_id: 0,
            view_epoch: 0,
            scope_generation: self.published_scope_generation,
        };
        self.sessions.insert(session_id.to_string(), SessionSlot {
            lifetime,
            negotiated: false,
            observer: false,
            secure: false,
            replay_consumed: false,
        });
        Some(lifetime)
    }

    pub fn close_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn lifetime(&self, session_id: &str) -> Option<CommunicationLifetime> {
        self.sessions.get(session_id).map(|slot| slot.lifetime)
    }

    pub fn replay_consumed(&self, session_id: &str) -> bool {
        self.sessions.get(session_id).is_some_and(|slot| slot.replay_consumed)
    }

    pub fn set_replay_consumed(&mut self, session_id: &str, lifetime: CommunicationLifetime) -> bool {
        let Some(slot) = self.sessions.get_mut(session_id) else { return false };
        if slot.lifetime != lifetime {
            return false;
        }
        slot.replay_consumed = true;
        true
    }

    pub fn note_view(&mut self, session_id: &str, view_epoch: u64) -> Option<CommunicationLifetime> {
        if view_epoch == 0 {
            return self.lifetime(session_id);
        }
        let binding_generation = if self.sessions.get(session_id).is_some_and(|slot| slot.lifetime.view_epoch != view_epoch) {
            self.allocate_field(ScopeCounter::Binding)?
        } else {
            return self.lifetime(session_id);
        };
        let slot = self.sessions.get_mut(session_id)?;
        slot.lifetime.binding_generation = binding_generation;
        slot.lifetime.view_epoch = view_epoch;
        Some(slot.lifetime)
    }

    /// Same room, run, player, and team keeps replay consumption and the nonce floor.
    pub fn note_hello(&mut self, session_id: &str, result: &CommunicationHelloResult, observer: bool, secure: bool) -> Option<CommunicationLifetime> {
        if self.poisoned {
            return None;
        }
        let same = self.sessions.get(session_id).is_some_and(|slot| {
            result.agreed
                && slot.lifetime.room_id == result.room_id
                && slot.lifetime.run_id == result.run_id
                && slot.lifetime.player_id == result.player_id
                && slot.lifetime.team_id == result.team_id
                && slot.lifetime.room_id != 0
        });
        if !result.agreed {
            if let Some(slot) = self.sessions.get_mut(session_id) {
                slot.negotiated = false;
                return Some(slot.lifetime);
            }
            return None;
        }
        if same {
            let view_changed = result.view_epoch != 0
                && self.sessions.get(session_id).is_some_and(|slot| slot.lifetime.view_epoch != result.view_epoch);
            let binding_generation = if view_changed {
                self.allocate_field(ScopeCounter::Binding)?
            } else {
                0
            };
            let slot = self.sessions.get_mut(session_id)?;
            slot.negotiated = true;
            slot.observer = observer;
            slot.secure = secure;
            if view_changed {
                slot.lifetime.binding_generation = binding_generation;
                slot.lifetime.view_epoch = result.view_epoch;
            }
            return Some(slot.lifetime);
        }
        let binding_generation = self.allocate_field(ScopeCounter::Binding)?;
        let slot = self.sessions.get_mut(session_id)?;
        slot.lifetime.binding_generation = binding_generation;
        slot.lifetime.room_id = result.room_id;
        slot.lifetime.run_id = result.run_id;
        slot.lifetime.player_id = result.player_id;
        slot.lifetime.team_id = result.team_id;
        slot.lifetime.view_epoch = result.view_epoch;
        slot.lifetime.scope_generation = self.published_scope_generation;
        slot.negotiated = true;
        slot.observer = observer;
        slot.secure = secure;
        slot.replay_consumed = false;
        let player_id = result.player_id;
        let room_id = result.room_id;
        let run_id = result.run_id;
        if let Some(stream) = self.streams.get_mut(&(room_id, run_id, player_id)) {
            stream.floor = stream.highest;
        }
        self.lifetime(session_id)
    }

    pub fn request_floor(&self, room_id: u64, run_id: u64, player_id: u32) -> u64 {
        self.streams.get(&(room_id, run_id, player_id)).map_or(0, |stream| stream.floor.max(stream.highest))
    }

    pub fn publish_scope(&mut self, scope: Option<CommunicationScope>) {
        if self.poisoned {
            self.published_scope = None;
            return;
        }
        if self.published_scope == scope {
            return;
        }
        match self.allocate_field(ScopeCounter::Scope) {
            Some(generation) => {
                self.published_scope_generation = generation;
                self.published_scope = scope;
                if let Some(scope) = scope {
                    self.streams.retain(|(room, run, _), _| *room == scope.room_id && *run == scope.run_id);
                }
                for slot in self.sessions.values_mut() {
                    slot.negotiated = false;
                    slot.lifetime.scope_generation = generation;
                }
            }
            None => {
                self.poisoned = true;
                self.published_scope = None;
                self.published_scope_generation = 0;
            }
        }
    }

    pub fn publish_clock(&mut self, elapsed_raw: i64, elapsed_known: bool) {
        if elapsed_raw < 0 || !elapsed_known {
            self.elapsed_known = false;
            self.elapsed_raw = elapsed_raw;
            return;
        }
        self.elapsed_raw = elapsed_raw;
        self.elapsed_known = true;
    }
}

/// One authority tick's publication inputs. Elapsed is the match clock raw value.
/// The caller does not clone `MobaMatch` to fill this in.
#[derive(Clone, Copy, Debug)]
pub struct CommunicationTick {
    pub scope: Option<CommunicationScope>,
    pub elapsed_raw: i64,
    pub elapsed_known: bool,
    pub queue_empty: bool,
    pub delivery_full: bool,
}

/// Publishes the registry scope and clock on every real tick, then reports
/// whether the request queue may be drained. `State::drain_authenticated_communication`
/// is the production caller. An unchanged scope does not renegotiate.
pub fn begin_communication_tick(registry: &mut CommunicationSessionRegistry, tick: CommunicationTick) -> bool {
    registry.publish_scope(tick.scope);
    registry.publish_clock(tick.elapsed_raw, tick.elapsed_known);
    !tick.queue_empty && !tick.delivery_full
}

impl CommunicationSessionRegistry {
    pub fn player_epochs(&self, room_id: u64, run_id: u64) -> Vec<(u32, u64, u64)> {
        self.sessions.values().filter(|slot| {
            slot.negotiated && slot.secure && !slot.observer && slot.lifetime.room_id == room_id && slot.lifetime.run_id == run_id
        }).map(|slot| (slot.lifetime.player_id, slot.lifetime.view_epoch, slot.lifetime.generation)).collect()
    }

    pub fn authorize(&self, request: &CommunicationRequestMsg) -> Result<(), u32> {
        if self.poisoned || request.lifetime.generation == 0 || request.lifetime.scope_generation == 0 {
            return Err(REFUSAL_UNAUTHENTICATED);
        }
        let Some(slot) = self.sessions.get(&request.origin_session) else {
            return Err(REFUSAL_UNAUTHENTICATED);
        };
        if slot.lifetime != request.lifetime || !slot.negotiated || !slot.secure || slot.observer {
            return Err(REFUSAL_UNAUTHENTICATED);
        }
        let Some(scope) = self.published_scope else {
            return Err(REFUSAL_SCOPE_UNAVAILABLE);
        };
        if scope.room_id != request.lifetime.room_id
            || scope.run_id != request.lifetime.run_id
            || request.lifetime.scope_generation != self.published_scope_generation
            || request.bound.room_id != scope.room_id
            || request.bound.run_id != scope.run_id
            || request.bound.player_id != request.lifetime.player_id
            || request.bound.team_id != request.lifetime.team_id
            || request.bound.view_epoch != request.lifetime.view_epoch
        {
            return Err(REFUSAL_SCOPE_UNAVAILABLE);
        }
        Ok(())
    }

    pub fn preview_nonce(&self, room_id: u64, run_id: u64, player_id: u32, nonce: u64) -> Result<(), u32> {
        let stream = self.streams.get(&(room_id, run_id, player_id)).copied().unwrap_or_default();
        nonce_status(&stream, nonce)
    }

    pub fn commit_nonce(&mut self, room_id: u64, run_id: u64, player_id: u32, nonce: u64) -> Result<(), u32> {
        let stream = self.streams.entry((room_id, run_id, player_id)).or_default();
        nonce_status(stream, nonce)?;
        if stream.highest == 0 || nonce > stream.highest {
            let shift = if stream.highest == 0 { 64 } else { nonce - stream.highest };
            if shift >= COMMUNICATION_REQUEST_WINDOW {
                stream.bits = 1;
            } else {
                stream.bits = (stream.bits << shift) | 1;
            }
            stream.highest = nonce;
        } else {
            let age = stream.highest - nonce;
            stream.bits |= 1_u64 << age;
        }
        Ok(())
    }
}

fn nonce_status(stream: &NonceStream, nonce: u64) -> Result<(), u32> {
    if nonce == 0 || nonce <= stream.floor {
        return Err(REFUSAL_INVALID_NONCE);
    }
    if stream.highest == 0 {
        return Ok(());
    }
    if nonce > stream.highest {
        return Ok(());
    }
    let age = stream.highest - nonce;
    if age >= COMMUNICATION_REQUEST_WINDOW {
        return Err(REFUSAL_INVALID_NONCE);
    }
    if stream.bits & (1_u64 << age) != 0 {
        Err(REFUSAL_DUPLICATE)
    } else {
        Ok(())
    }
}

/// Room/run identity for one match seed. A missing match retires the run.
/// The same seed does not start again at run 1. Overflow fails closed.
#[derive(Clone, Debug, Default)]
pub struct CommunicationScopeClock {
    room: u64,
    run: u64,
    retired_room: u64,
    retired_next_run: u64,
    poisoned: bool,
}

impl CommunicationScopeClock {
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub fn published(&self) -> Option<CommunicationScope> {
        if self.poisoned {
            None
        } else {
            CommunicationScope::from_seed_and_generation(self.room, self.run)
        }
    }

    pub fn retire(&mut self) {
        if self.poisoned {
            self.room = 0;
            self.run = 0;
            return;
        }
        if self.room != 0 && self.run != 0 {
            match self.run.checked_add(1) {
                Some(next) => {
                    self.retired_room = self.room;
                    self.retired_next_run = next;
                }
                None => {
                    self.poisoned = true;
                    self.retired_next_run = 0;
                }
            }
        }
        self.room = 0;
        self.run = 0;
    }

    pub fn observe(&mut self, seed: u64, has_match: bool) -> Option<CommunicationScope> {
        if self.poisoned {
            self.room = 0;
            self.run = 0;
            return None;
        }
        if !has_match || seed == 0 {
            if self.room != 0 {
                self.retire();
            }
            return None;
        }
        if self.room != seed {
            let next = if self.room != 0 {
                match self.run.checked_add(1) {
                    Some(next) => next,
                    None => {
                        self.poisoned = true;
                        self.room = 0;
                        self.run = 0;
                        return None;
                    }
                }
            } else if self.retired_room == seed && self.retired_next_run != 0 {
                self.retired_next_run
            } else {
                1
            };
            if next == 0 {
                self.poisoned = true;
                self.room = 0;
                self.run = 0;
                return None;
            }
            self.room = seed;
            self.run = next;
            self.retired_room = 0;
            self.retired_next_run = 0;
        } else if self.run == 0 {
            self.poisoned = true;
            return None;
        }
        self.published()
    }
}

pub fn communication_tag(tag: u8) -> bool {
    matches!(
        tag,
        TAG_COMMUNICATION_HELLO
            | TAG_COMMUNICATION_HELLO_RESULT
            | TAG_COMMUNICATION_SUBMIT
            | TAG_COMMUNICATION_RECEIPT
            | TAG_COMMUNICATION_EVENT
            | TAG_COMMUNICATION_REPLAY_QUERY
            | TAG_COMMUNICATION_REPLAY
    )
}

pub fn reserved_tags_unchanged() -> bool {
    !communication_tag(TAG_PING_REQ)
        && !communication_tag(TAG_PING_RESP)
        && !communication_tag(TAG_SHOP_RECEIPT_QUERY)
        && !communication_tag(TAG_SHOP_RECEIPT_REPLAY)
        && !communication_tag(TAG_OWNER_WARD_ENVELOPE)
        && !communication_tag(TAG_OWNER_WARD_RECOVERY_QUERY)
}

pub fn negotiate_hello(
    bound: &BoundCommunicationSession,
    scope: Option<CommunicationScope>,
    hello: &CommunicationHello,
) -> CommunicationHelloResult {
    let mut result = CommunicationHelloResult {
        schema_version: COMMUNICATION_SCHEMA_VERSION,
        protocol_version: 0,
        agreed: false,
        refusal: REFUSAL_UNSUPPORTED,
        ..Default::default()
    };
    if !bound.secure_joined || bound.player_id == 0 || bound.team_id == 0 || bound.view_epoch == 0 {
        result.refusal = REFUSAL_UNAUTHENTICATED;
        return result;
    }
    if hello.claimed_observer != bound.observer {
        result.refusal = REFUSAL_IDENTITY_MISMATCH;
        return result;
    }
    if bound.observer || hello.claimed_observer {
        result.refusal = REFUSAL_OBSERVER_DENIED;
        return result;
    }
    if hello.protocol_version != COMMUNICATION_PROTOCOL_VERSION {
        result.refusal = REFUSAL_UNSUPPORTED;
        return result;
    }
    let Some(scope) = scope else {
        result.refusal = REFUSAL_SCOPE_UNAVAILABLE;
        return result;
    };
    if hello.claimed_player_id != bound.player_id || hello.claimed_team_id != bound.team_id {
        result.refusal = REFUSAL_IDENTITY_MISMATCH;
        return result;
    }
    if hello.claimed_view_epoch != 0 && hello.claimed_view_epoch != bound.view_epoch {
        result.refusal = REFUSAL_VIEW_MISMATCH;
        return result;
    }
    if (hello.claimed_room_id != 0 || hello.claimed_run_id != 0)
        && (hello.claimed_room_id != scope.room_id || hello.claimed_run_id != scope.run_id)
    {
        result.refusal = REFUSAL_IDENTITY_MISMATCH;
        return result;
    }
    result.protocol_version = COMMUNICATION_PROTOCOL_VERSION;
    result.agreed = true;
    result.refusal = REFUSAL_ADMITTED;
    result.room_id = scope.room_id;
    result.run_id = scope.run_id;
    result.player_id = bound.player_id;
    result.team_id = bound.team_id;
    result.view_epoch = bound.view_epoch;
    result
}

pub fn decode_hello(bytes: &[u8]) -> Result<CommunicationHello, u32> {
    bounded_decode(bytes, HELLO_MAX_BYTES)
}

pub fn decode_submit(bytes: &[u8]) -> Result<DecodedSubmit, u32> {
    if bytes.len() > SUBMIT_MAX_BYTES {
        return Err(REFUSAL_OVERSIZED);
    }
    match reserved_canonical_guess(bytes) {
        Ok(true) | Err(()) => return Err(REFUSAL_MALFORMED),
        Ok(false) => {}
    }
    let message: CommunicationSubmit = bounded_decode(bytes, SUBMIT_MAX_BYTES)?;
    if message.schema_version != COMMUNICATION_SCHEMA_VERSION || message.request_id == 0 {
        return Err(REFUSAL_MALFORMED);
    }
    let claimed = ClaimedIdentity {
        room_id: message.claimed_room_id,
        run_id: message.claimed_run_id,
        player_id: message.claimed_player_id,
        team_id: message.claimed_team_id,
        observer: message.claimed_observer,
    };
    let body = decode_body(&message)?;
    Ok(DecodedSubmit {
        request_id: message.request_id,
        nonce: message.nonce,
        claimed,
        claimed_view_epoch: message.claimed_view_epoch,
        body,
    })
}

pub fn decode_replay_query(bytes: &[u8]) -> Result<(u64, ClaimedIdentity, u64), u32> {
    let message: CommunicationReplayQuery = bounded_decode(bytes, REPLAY_QUERY_MAX_BYTES)?;
    if message.schema_version != COMMUNICATION_SCHEMA_VERSION || message.request_id == 0 {
        return Err(REFUSAL_MALFORMED);
    }
    Ok((
        message.request_id,
        ClaimedIdentity {
            room_id: message.claimed_room_id,
            run_id: message.claimed_run_id,
            player_id: message.claimed_player_id,
            team_id: message.claimed_team_id,
            observer: message.claimed_observer,
        },
        message.claimed_view_epoch,
    ))
}

pub fn decode_receipt(bytes: &[u8]) -> Result<CommunicationReceipt, u32> {
    let message: CommunicationReceipt = bounded_decode(bytes, RECEIPT_MAX_BYTES)?;
    if !receipt_shape(&message) {
        return Err(REFUSAL_MALFORMED);
    }
    Ok(message)
}

pub fn decode_event(bytes: &[u8]) -> Result<CommunicationWireEvent, u32> {
    let message: CommunicationWireEvent = bounded_decode(bytes, EVENT_MAX_BYTES)?;
    if !wire_event_shape(&message) {
        return Err(REFUSAL_MALFORMED);
    }
    Ok(message)
}

pub fn decode_replay_batch(bytes: &[u8]) -> Result<CommunicationReplayBatch, u32> {
    if bytes.len() > REPLAY_BATCH_MAX_BYTES {
        return Err(REFUSAL_OVERSIZED);
    }
    let message = CommunicationReplayBatch::decode(bytes).map_err(|_| REFUSAL_MALFORMED)?;
    if message.schema_version != COMMUNICATION_SCHEMA_VERSION
        || message.elapsed_raw < 0
        || message.events.len() > MAX_HISTORY
        || message.events.iter().any(|event| !wire_event_shape(event))
        || message.events.iter().any(|event| {
            event.room_id != message.room_id || event.run_id != message.run_id || event.view_epoch != message.view_epoch
        })
        || (message.refusal == REFUSAL_ADMITTED && message.room_id == 0)
        || (message.refusal != REFUSAL_ADMITTED && !message.events.is_empty())
        || (message.already_replayed && !message.events.is_empty())
    {
        return Err(REFUSAL_MALFORMED);
    }
    Ok(message)
}

pub fn route_communication_event(
    sessions: &[CommunicationRouteSession<'_>],
    recipients: &[u32],
    audience: u32,
    sender_team: u32,
    room_id: u64,
    run_id: u64,
    view_epoch: u64,
) -> Vec<String> {
    sessions
        .iter()
        .filter(|session| {
            session.negotiated
                && session.secure_joined
                && !session.observer
                && session.player_id != 0
                && session.team_id != 0
                && recipients.contains(&session.player_id)
                && session.room_id == room_id
                && session.run_id == run_id
                && session.view_epoch == view_epoch
                && match audience {
                    AUDIENCE_TEAM => session.team_id == sender_team,
                    AUDIENCE_ALL => true,
                    _ => false,
                }
        })
        .map(|session| session.session_id.to_string())
        .collect()
}

/// Fanout helper used by the delivery thread. A frame is delivered only to the
/// session whose current epoch, generation, and published scope still match.
pub fn route_live_frame(
    sessions: &[CommunicationRouteSession<'_>],
    frame: &CommunicationRecipientFrame,
    audience: u32,
    sender_team: u32,
    room_id: u64,
    run_id: u64,
    scope_generation: u64,
    published_scope_generation: u64,
    elapsed_raw: i64,
    elapsed_known: bool,
) -> Option<String> {
    if room_id == 0 || run_id == 0 || scope_generation == 0 || scope_generation != published_scope_generation {
        return None;
    }
    if matches!(frame.kind, KIND_PING | KIND_OBJECTIVE | KIND_EMOTE | KIND_STATUS) {
        if !elapsed_known || elapsed_raw < 0 || elapsed_raw < frame.admitted_at_raw || elapsed_raw >= frame.expires_at_raw {
            return None;
        }
    }
    sessions.iter().find(|session| {
        session.negotiated
            && session.secure_joined
            && !session.observer
            && session.player_id == frame.player_id
            && session.team_id == frame.team_id
            && session.view_epoch == frame.view_epoch
            && session.room_id == room_id
            && session.run_id == run_id
            && (frame.session_generation == 0 || session.generation == frame.session_generation)
            && match audience {
                AUDIENCE_TEAM => session.team_id == sender_team,
                AUDIENCE_ALL => true,
                _ => false,
            }
    }).map(|session| session.session_id.to_string())
}

/// What the delivery thread may write after it has the current session table.
/// Receipt and replay follow the origin lifetime. Each event follows that
/// recipient's current lifetime and [`route_live_frame`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedDeliveryGrant {
    pub deliver_receipt: bool,
    pub deliver_replay: bool,
    pub mark_replay_consumed: bool,
    pub event_sessions: Vec<String>,
}

fn live_route_session(registry: &CommunicationSessionRegistry, session: &CommunicationRouteSession<'_>) -> bool {
    let Some(life) = registry.lifetime(session.session_id) else {
        return false;
    };
    life.generation != 0
        && life.scope_generation != 0
        && life.scope_generation == registry.published_scope_generation()
        && life.generation == session.generation
        && life.binding_generation != 0
        && life.player_id == session.player_id
        && life.team_id == session.team_id
        && life.view_epoch == session.view_epoch
        && life.room_id == session.room_id
        && life.run_id == session.run_id
        && session.negotiated
        && session.secure_joined
        && !session.observer
}

/// Authorizes one queued delivery against the registry and sessions observed
/// after the delivery task has finished waiting for the session table.
/// Callers must not cache the result across that wait.
pub fn authorize_queued_delivery(
    registry: &CommunicationSessionRegistry,
    delivery: &CommunicationDelivery,
    sessions: &[CommunicationRouteSession<'_>],
) -> QueuedDeliveryGrant {
    let published = registry.published_scope_generation();
    let (elapsed_raw, elapsed_known) = registry.elapsed();
    let scope_ok = registry.published_scope().is_some_and(|scope| {
        scope.room_id == delivery.room_id && scope.run_id == delivery.run_id && delivery.room_id != 0 && delivery.run_id != 0
    });
    let origin_ok = scope_ok
        && registry.lifetime(&delivery.origin_session) == Some(delivery.origin_lifetime)
        && delivery.origin_lifetime.scope_generation == published
        && sessions.iter().any(|session| session.session_id == delivery.origin_session && live_route_session(registry, session));
    let replay_pending = origin_ok && delivery.replay.is_some() && !registry.replay_consumed(&delivery.origin_session);
    let mut event_sessions = Vec::new();
    if scope_ok {
        for frame in &delivery.recipient_frames {
            let Some(id) = route_live_frame(
                sessions,
                frame,
                delivery.audience,
                delivery.sender_team,
                delivery.room_id,
                delivery.run_id,
                delivery.scope_generation,
                published,
                elapsed_raw,
                elapsed_known,
            ) else {
                continue;
            };
            let current = sessions.iter().any(|session| session.session_id == id && live_route_session(registry, session));
            if current && !event_sessions.iter().any(|existing| existing == &id) {
                event_sessions.push(id);
            }
        }
    }
    QueuedDeliveryGrant {
        deliver_receipt: origin_ok,
        deliver_replay: replay_pending,
        mark_replay_consumed: replay_pending && delivery.consume_replay,
        event_sessions,
    }
}

pub fn communication_presentation_acceptable(
    batch: &CommunicationPresentation,
    team_id: u32,
    player_id: u32,
) -> bool {
    batch.schema_version == COMMUNICATION_SCHEMA_VERSION
        && batch.negotiated
        && batch.player_id == player_id
        && batch.team_id == team_id
        && batch.room_id != 0
        && batch.run_id != 0
        && batch.view_epoch != 0
        && batch.source_generation != 0
        && batch.presentation_epoch != 0
        && batch.elapsed_raw >= 0
        && batch.events.len() <= MAX_HISTORY
        && batch.receipt.as_ref().is_none_or(|r| receipt_shape(r) && r.room_id == batch.room_id
            && r.run_id == batch.run_id && r.view_epoch == batch.view_epoch && r.sender_id == batch.player_id)
        && batch.input_result.as_ref().is_none_or(|r| r.renderer_connection_id != 0 && r.request_token != 0
            && match r.phase {
                1 => r.network_request_id != 0 && r.refusal == 0,
                2 => batch.receipt.as_ref().is_some_and(|receipt| receipt.request_id == r.network_request_id
                    && r.network_request_id != 0 && receipt.refusal == r.refusal),
                3 => known_refusal(r.refusal),
                _ => false,
            })
        && batch.events.iter().all(wire_event_shape)
        && batch.events.iter().all(|event| {
            event.room_id == batch.room_id
                && event.run_id == batch.run_id
                && event.view_epoch == batch.view_epoch
        })
}

/// A binding retirement carries no renderable event or local input result.
pub fn communication_presentation_cleared(batch: &CommunicationPresentation) -> bool {
    batch.schema_version == COMMUNICATION_SCHEMA_VERSION && !batch.negotiated
        && batch.room_id == 0 && batch.run_id == 0 && batch.view_epoch == 0
        && batch.player_id == 0 && batch.team_id == 0 && batch.source_generation != 0
        && batch.presentation_epoch != 0 && batch.elapsed_raw >= 0 && batch.events.is_empty()
        && batch.input_result.is_none() && batch.receipt.as_ref().is_none_or(receipt_shape)
}

#[derive(Clone, Debug)]
pub struct CommunicationInbox {
    hello_attempts: u8,
    hello_due: bool,
    hello_outstanding: bool,
    hello_follow_up: bool,
    transition_view: u64,
    live_tick_armed: bool,
    agreed: bool,
    room_id: u64,
    run_id: u64,
    player_id: u32,
    team_id: u32,
    view_epoch: u64,
    source_generation: u64,
    presentation_epoch: u64,
    request_floor: u64,
    settled_floor: u64,
    events: VecDeque<CommunicationWireEvent>,
    seen: VecDeque<(u32, u64)>,
    pending: VecDeque<u64>,
    settled: VecDeque<u64>,
    last_receipt: Option<CommunicationReceipt>,
    replay_requested: bool,
    elapsed_raw: i64,
}

impl Default for CommunicationInbox {
    fn default() -> Self {
        Self {
            hello_attempts: 0,
            hello_due: true,
            hello_outstanding: false,
            hello_follow_up: false,
            transition_view: 0,
            live_tick_armed: false,
            agreed: false,
            room_id: 0,
            run_id: 0,
            player_id: 0,
            team_id: 0,
            view_epoch: 0,
            source_generation: 0,
            presentation_epoch: 0,
            request_floor: 0,
            settled_floor: 0,
            events: VecDeque::new(),
            seen: VecDeque::new(),
            pending: VecDeque::new(),
            settled: VecDeque::new(),
            last_receipt: None,
            replay_requested: false,
            elapsed_raw: 0,
        }
    }
}

impl CommunicationInbox {
    pub fn hello_allowed(&self) -> bool {
        self.hello_due()
    }

    /// Identical view and tick values do not arm another hello.
    pub fn hello_retry_for_scope(&self) -> bool {
        false
    }

    pub fn hello_due(&self) -> bool {
        self.hello_due && !self.hello_outstanding && !self.agreed
    }

    /// Arms at most one hello for a new view epoch, and one more when the
    /// first nonzero server tick arrives after a scope refusal.
    pub fn note_transition(&mut self, view_epoch: u64, server_tick: u64) {
        if view_epoch != 0 && view_epoch != self.transition_view {
            self.transition_view = view_epoch;
            self.arm_hello();
        }
        if !self.live_tick_armed && server_tick != 0 {
            self.live_tick_armed = true;
            if self.last_receipt.as_ref().is_some_and(|receipt| receipt.refusal == REFUSAL_SCOPE_UNAVAILABLE) {
                self.arm_hello();
            }
        }
    }

    fn arm_hello(&mut self) {
        if self.agreed && self.transition_view != 0 && self.transition_view != self.view_epoch {
            self.retire_binding();
        }
        if self.agreed {
            return;
        }
        if self.hello_outstanding {
            self.hello_follow_up = true;
        } else {
            self.hello_due = true;
        }
    }

    pub fn note_hello_sent(&mut self) {
        self.hello_due = false;
        self.hello_outstanding = true;
        self.hello_attempts = self.hello_attempts.saturating_add(1);
    }

    pub fn agreed(&self) -> bool {
        self.agreed
    }

    /// Current negotiated identity, without cloning retained history.
    pub fn binding(&self) -> Option<TrustedSession> {
        self.agreed.then_some(TrustedSession {
            room_id: self.room_id, run_id: self.run_id, player_id: self.player_id,
            team_id: self.team_id, view_epoch: self.view_epoch,
        })
    }

    /// The producer supplies intent only. Identity always comes from the agreed
    /// inbox, and pending state changes only after the complete wire shape validates.
    pub fn prepare_submit(&mut self, request_id: u64, nonce: u64, body: &DecodedBody) -> Result<CommunicationSubmit, u32> {
        let bound = self.binding().ok_or(REFUSAL_NOT_NEGOTIATED)?;
        let mut wire = CommunicationSubmit {
            schema_version: COMMUNICATION_SCHEMA_VERSION, request_id, nonce,
            claimed_room_id: bound.room_id, claimed_run_id: bound.run_id,
            claimed_player_id: bound.player_id, claimed_team_id: bound.team_id,
            claimed_view_epoch: bound.view_epoch, ..Default::default()
        };
        let hud = match body {
            DecodedBody::Ping { semantic, point, target, hud } => {
                wire.kind = KIND_PING; wire.semantic = *semantic as u32; wire.has_point = true;
                wire.point_x_raw = point.x.raw(); wire.point_y_raw = point.y.raw();
                if let Some(target) = target {
                    wire.has_target = true; wire.replica_id = target.replica_id;
                    wire.generation = target.generation; wire.disclosure_epoch = target.disclosure_epoch;
                    wire.target_view_epoch = target.view_epoch;
                }
                *hud
            }
            DecodedBody::Chat { channel, text } => {
                if !text_is_clean(text) { return Err(REFUSAL_MALFORMED); }
                wire.kind = KIND_CHAT; wire.text = text.clone();
                wire.chat_channel = match channel { ChatChannel::Team => CHANNEL_TEAM, ChatChannel::All => CHANNEL_ALL };
                None
            }
            DecodedBody::Objective { objective_id } => {
                if objective_id.is_empty() || objective_id.len() > MAX_OBJECTIVE_ID_BYTES
                    || objective_id.chars().any(|c| c.is_control()) { return Err(REFUSAL_MALFORMED); }
                wire.kind = KIND_OBJECTIVE; wire.objective_id = objective_id.clone(); None
            }
            DecodedBody::Mute { sender_id, muted } => {
                wire.kind = KIND_MUTE; wire.mute_sender_id = *sender_id; wire.muted = *muted; None
            }
            DecodedBody::Emote { content_id } => {
                if omoba_template_ids::communication_content(*content_id).is_none() { return Err(REFUSAL_CONTENT_UNAVAILABLE); }
                wire.kind = KIND_EMOTE; wire.content_id = u32::from(*content_id); None
            }
            DecodedBody::Status { hud } => { wire.kind = KIND_STATUS; Some(*hud) }
        };
        if let Some(hud) = hud {
            match hud {
                HudClaim::OwnedAbility { index } => { wire.hud_kind = HUD_ABILITY; wire.hud_index = u32::from(index); }
                HudClaim::OwnedItem { slot } => { wire.hud_kind = HUD_ITEM; wire.hud_index = u32::from(slot); }
                HudClaim::PublicScore { player_id } => { wire.hud_kind = HUD_SCORE; wire.hud_player_id = player_id; }
            }
        }
        decode_submit(&wire.encode_to_vec())?;
        self.begin_submit(request_id)?;
        Ok(wire)
    }

    pub fn replay_query_allowed(&self) -> bool {
        self.agreed && !self.replay_requested && self.room_id != 0 && self.run_id != 0
    }

    pub fn note_replay_query_sent(&mut self) {
        self.replay_requested = true;
    }

    /// Identity is the agreed hello result. A caller cannot substitute another room.
    pub fn replay_query(&self, request_id: u64) -> Option<CommunicationReplayQuery> {
        if !self.replay_query_allowed() || request_id == 0 {
            return None;
        }
        Some(CommunicationReplayQuery {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id,
            claimed_room_id: self.room_id,
            claimed_run_id: self.run_id,
            claimed_player_id: self.player_id,
            claimed_team_id: self.team_id,
            claimed_view_epoch: self.view_epoch,
            claimed_observer: false,
        })
    }

    pub fn source_generation(&self) -> u64 {
        self.source_generation
    }

    pub fn presentation_epoch(&self) -> u64 {
        self.presentation_epoch
    }

    pub fn begin_submit(&mut self, request_id: u64) -> Result<(), u32> {
        if !self.agreed || request_id == 0 {
            return Err(REFUSAL_NOT_NEGOTIATED);
        }
        if request_id <= self.request_floor || request_id <= self.settled_floor || self.settled.contains(&request_id) {
            return Err(REFUSAL_DUPLICATE);
        }
        if self.pending.len() >= COMMUNICATION_PENDING_SUBMISSIONS || self.pending.contains(&request_id) {
            return Err(REFUSAL_BACKPRESSURE);
        }
        self.pending.push_back(request_id);
        Ok(())
    }

    pub fn fail_submit_locally(&mut self, request_id: u64) -> Option<CommunicationPresentation> {
        if !self.pending.contains(&request_id) {
            return None;
        }
        self.pending.retain(|pending| *pending != request_id);
        self.last_receipt = Some(local_receipt(self, request_id, REFUSAL_LOCAL_TRANSPORT));
        Some(self.presentation())
    }

    pub fn apply_hello(&mut self, result: &CommunicationHelloResult) -> Option<CommunicationPresentation> {
        if !hello_result_shape(result) {
            self.hello_outstanding = false;
            if !self.agreed {
                self.hello_follow_up = false;
                self.hello_due = true;
            }
            return None;
        }
        self.hello_outstanding = false;
        self.last_receipt = Some(CommunicationReceipt {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: 0,
            refusal: result.refusal,
            room_id: result.room_id,
            run_id: result.run_id,
            sender_id: result.player_id,
            sequence: 0,
            view_epoch: result.view_epoch,
            outcome_kind: if result.agreed { OUTCOME_HELLO } else { OUTCOME_REFUSED },
        });
        if !result.agreed {
            if self.agreed || self.room_id != 0 || !self.events.is_empty() {
                self.retire_binding();
            }
            self.finish_follow_up();
            return Some(self.presentation());
        }
        let source_changed = !self.agreed
            || self.room_id != result.room_id
            || self.run_id != result.run_id
            || self.player_id != result.player_id
            || self.team_id != result.team_id;
        if source_changed {
            self.events.clear();
            self.seen.clear();
            self.pending.clear();
            self.settled.clear();
            self.settled_floor = 0;
            self.replay_requested = false;
            if !self.bump_source() {
                self.finish_follow_up();
                return Some(self.presentation());
            }
        }
        let view_changed = self.view_epoch != result.view_epoch;
        self.agreed = true;
        self.hello_due = false;
        self.room_id = result.room_id;
        self.run_id = result.run_id;
        self.player_id = result.player_id;
        self.team_id = result.team_id;
        self.view_epoch = result.view_epoch;
        self.request_floor = result.request_floor;
        if source_changed || view_changed || self.presentation_epoch == 0 {
            self.events.clear();
            self.seen.clear();
            if !self.bump_presentation() {
                self.agreed = false;
                self.finish_follow_up();
                return Some(self.presentation());
            }
        }
        self.finish_follow_up();
        Some(self.presentation())
    }

    fn finish_follow_up(&mut self) {
        if self.hello_follow_up && !self.agreed {
            self.hello_follow_up = false;
            self.hello_due = true;
        } else {
            self.hello_follow_up = false;
        }
    }

    fn retire_binding(&mut self) {
        self.agreed = false;
        self.hello_due = false;
        self.room_id = 0;
        self.run_id = 0;
        self.player_id = 0;
        self.team_id = 0;
        self.view_epoch = 0;
        self.request_floor = 0;
        self.events.clear();
        self.seen.clear();
        self.pending.clear();
        self.settled.clear();
        self.settled_floor = 0;
        self.replay_requested = false;
        let _ = self.bump_source();
        let _ = self.bump_presentation();
    }

    fn bump_source(&mut self) -> bool {
        match self.source_generation.checked_add(1) {
            Some(next) => {
                self.source_generation = next;
                true
            }
            None => {
                self.agreed = false;
                self.hello_due = false;
                false
            }
        }
    }

    fn bump_presentation(&mut self) -> bool {
        match self.presentation_epoch.checked_add(1) {
            Some(next) => {
                self.presentation_epoch = next;
                true
            }
            None => {
                self.agreed = false;
                self.hello_due = false;
                false
            }
        }
    }

    fn note_settled(&mut self, request_id: u64) {
        if request_id == 0 || request_id <= self.settled_floor || self.settled.contains(&request_id) {
            return;
        }
        self.settled.push_back(request_id);
        while self.settled.len() > COMMUNICATION_SETTLED_WINDOW {
            if let Some(retired) = self.settled.pop_front() {
                if retired > self.settled_floor {
                    self.settled_floor = retired;
                }
            }
        }
    }

    pub fn observe_view(&mut self, view_epoch: u64) -> Option<CommunicationPresentation> {
        if !self.agreed || view_epoch == 0 || view_epoch == self.view_epoch {
            return None;
        }
        self.view_epoch = view_epoch;
        self.last_receipt = None;
        self.events.clear();
        self.seen.clear();
        self.pending.clear();
        self.replay_requested = false;
        if !self.bump_presentation() {
            self.agreed = false;
        }
        Some(self.presentation())
    }

    pub fn apply_receipt(&mut self, receipt: &CommunicationReceipt) -> Option<CommunicationPresentation> {
        if !receipt_shape(receipt) || !self.matches_binding(receipt.room_id, receipt.run_id, receipt.view_epoch) {
            return None;
        }
        if receipt.request_id != 0 && !self.pending.contains(&receipt.request_id) {
            return None;
        }
        if receipt.request_id != 0 {
            self.pending.retain(|pending| *pending != receipt.request_id);
            self.note_settled(receipt.request_id);
        }
        self.last_receipt = Some(receipt.clone());
        Some(self.presentation())
    }

    pub fn apply_event(&mut self, event: &CommunicationWireEvent) -> Option<CommunicationPresentation> {
        if !self.accepts_event(event) {
            return None;
        }
        if self.seen.iter().any(|(sender, sequence)| *sender == event.sender_id && *sequence == event.sequence) {
            return None;
        }
        if self.seen.len() == super::communication::MAX_DEDUP {
            self.seen.pop_front();
        }
        self.seen.push_back((event.sender_id, event.sequence));
        self.events.push_back(event.clone());
        while self.events.len() > MAX_HISTORY {
            self.events.pop_front();
        }
        if event.admitted_at_raw >= self.elapsed_raw {
            self.elapsed_raw = event.admitted_at_raw;
        }
        Some(self.presentation())
    }

    /// Validates every event before changing retained rows. A bad batch leaves
    /// the previous source generation, presentation epoch, and events in place.
    pub fn apply_replay(&mut self, batch: &CommunicationReplayBatch) -> Option<CommunicationPresentation> {
        if batch.schema_version != COMMUNICATION_SCHEMA_VERSION
            || batch.elapsed_raw < 0
            || batch.elapsed_raw < self.elapsed_raw
            || batch.events.len() > MAX_HISTORY
            || !self.matches_binding(batch.room_id, batch.run_id, batch.view_epoch)
        {
            return None;
        }
        if batch.refusal != REFUSAL_ADMITTED {
            return None;
        }
        if batch.events.iter().any(|event| !self.accepts_event(event)) {
            return None;
        }
        let mut seen = self.seen.clone();
        let mut events = self.events.clone();
        for event in &batch.events {
            if seen.iter().any(|(sender, sequence)| *sender == event.sender_id && *sequence == event.sequence) {
                continue;
            }
            if seen.len() == super::communication::MAX_DEDUP {
                seen.pop_front();
            }
            seen.push_back((event.sender_id, event.sequence));
            events.push_back(event.clone());
            while events.len() > MAX_HISTORY {
                events.pop_front();
            }
        }
        self.seen = seen;
        self.events = events;
        self.elapsed_raw = batch.elapsed_raw;
        self.replay_requested = true;
        self.last_receipt = Some(CommunicationReceipt {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: batch.request_id,
            refusal: REFUSAL_ADMITTED,
            room_id: batch.room_id,
            run_id: batch.run_id,
            sender_id: self.player_id,
            sequence: 0,
            view_epoch: batch.view_epoch,
            outcome_kind: OUTCOME_REPLAY,
        });
        if batch.request_id != 0 {
            self.pending.retain(|pending| *pending != batch.request_id);
        }
        Some(self.presentation())
    }

    pub fn presentation(&self) -> CommunicationPresentation {
        CommunicationPresentation {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            view_epoch: self.view_epoch,
            room_id: self.room_id,
            run_id: self.run_id,
            player_id: self.player_id,
            team_id: self.team_id,
            source_generation: self.source_generation,
            presentation_epoch: self.presentation_epoch,
            events: self.events.iter().cloned().collect(),
            receipt: self.last_receipt.clone(),
            elapsed_raw: self.elapsed_raw,
            negotiated: self.agreed,
            input_result: None,
        }
    }

    fn matches_binding(&self, room_id: u64, run_id: u64, view_epoch: u64) -> bool {
        self.agreed && self.room_id == room_id && self.run_id == run_id && self.view_epoch == view_epoch
    }

    fn accepts_event(&self, event: &CommunicationWireEvent) -> bool {
        self.matches_binding(event.room_id, event.run_id, event.view_epoch)
            && wire_event_shape(event)
            && match event.audience {
                AUDIENCE_TEAM => event.sender_team == self.team_id,
                AUDIENCE_ALL => event.sender_team != 0,
                _ => false,
            }
    }
}

fn local_receipt(inbox: &CommunicationInbox, request_id: u64, refusal: u32) -> CommunicationReceipt {
    CommunicationReceipt {
        schema_version: COMMUNICATION_SCHEMA_VERSION,
        request_id,
        refusal,
        room_id: inbox.room_id,
        run_id: inbox.run_id,
        sender_id: inbox.player_id,
        sequence: 0,
        view_epoch: inbox.view_epoch,
        outcome_kind: OUTCOME_REFUSED,
    }
}

fn session_gate(bound: &BoundCommunicationSession, require_negotiated: bool) -> Option<u32> {
    if !bound.secure_joined || bound.player_id == 0 || bound.team_id == 0 || bound.view_epoch == 0 || bound.room_id == 0 || bound.run_id == 0 {
        return Some(REFUSAL_UNAUTHENTICATED);
    }
    if bound.observer {
        return Some(REFUSAL_OBSERVER_DENIED);
    }
    if require_negotiated && !bound.negotiated {
        return Some(REFUSAL_NOT_NEGOTIATED);
    }
    None
}

fn trusted_from(bound: &BoundCommunicationSession) -> TrustedSession {
    TrustedSession {
        room_id: bound.room_id,
        run_id: bound.run_id,
        player_id: bound.player_id,
        team_id: bound.team_id,
        view_epoch: bound.view_epoch,
    }
}

fn refusal_from(error: AdmissionError) -> u32 {
    match error {
        AdmissionError::IncompleteSession => REFUSAL_INCOMPLETE_SESSION,
        AdmissionError::IdentityMismatch => REFUSAL_IDENTITY_MISMATCH,
        AdmissionError::ObserverDenied => REFUSAL_OBSERVER_DENIED,
        AdmissionError::InvalidNonce => REFUSAL_INVALID_NONCE,
        AdmissionError::InvalidClock => REFUSAL_INVALID_CLOCK,
        AdmissionError::RegressedClock => REFUSAL_REGRESSED_CLOCK,
        AdmissionError::ClockOverflow => REFUSAL_CLOCK_OVERFLOW,
        AdmissionError::InvalidRoster => REFUSAL_INVALID_ROSTER,
        AdmissionError::UnresolvedTeam => REFUSAL_UNRESOLVED_TEAM,
        AdmissionError::AbsentMembership => REFUSAL_ABSENT_MEMBERSHIP,
        AdmissionError::ForeignMembership => REFUSAL_FOREIGN_MEMBERSHIP,
        AdmissionError::ResultsClosed => REFUSAL_RESULTS_CLOSED,
        AdmissionError::EmptyText => REFUSAL_EMPTY_TEXT,
        AdmissionError::TextLimit => REFUSAL_TEXT_LIMIT,
        AdmissionError::TextControl => REFUSAL_TEXT_CONTROL,
        AdmissionError::PointOutOfBounds => REFUSAL_POINT_OUT_OF_BOUNDS,
        AdmissionError::StaleOrHiddenTarget => REFUSAL_STALE_OR_HIDDEN_TARGET,
        AdmissionError::DisclosureTeamMismatch => REFUSAL_DISCLOSURE_TEAM_MISMATCH,
        AdmissionError::HudFactUnavailable => REFUSAL_HUD_FACT_UNAVAILABLE,
        AdmissionError::ContentUnavailable => REFUSAL_CONTENT_UNAVAILABLE,
        AdmissionError::ObjectiveUnavailable => REFUSAL_OBJECTIVE_UNAVAILABLE,
        AdmissionError::ObjectivePhase => REFUSAL_OBJECTIVE_PHASE,
        AdmissionError::ObjectiveCapacity => REFUSAL_OBJECTIVE_CAPACITY,
        AdmissionError::Duplicate => REFUSAL_DUPLICATE,
        AdmissionError::RateLimited => REFUSAL_RATE_LIMITED,
        AdmissionError::SequenceExhausted => REFUSAL_SEQUENCE_EXHAUSTED,
        AdmissionError::MuteCapacity => REFUSAL_MUTE_CAPACITY,
        AdmissionError::MuteUnauthorized => REFUSAL_MUTE_UNAUTHORIZED,
    }
}

fn audience_code(audience: CommunicationAudience) -> u32 {
    match audience {
        CommunicationAudience::Team => AUDIENCE_TEAM,
        CommunicationAudience::All => AUDIENCE_ALL,
    }
}

fn receipt(
    request_id: u64,
    refusal: u32,
    outcome_kind: u32,
    bound: &BoundCommunicationSession,
    sequence: u64,
) -> CommunicationReceipt {
    CommunicationReceipt {
        schema_version: COMMUNICATION_SCHEMA_VERSION,
        request_id,
        refusal,
        room_id: bound.room_id,
        run_id: bound.run_id,
        sender_id: bound.player_id,
        sequence,
        view_epoch: bound.view_epoch,
        outcome_kind,
    }
}

fn refused(
    request: &CommunicationRequestMsg,
    request_id: u64,
    refusal: u32,
    consume_replay: bool,
) -> CommunicationDelivery {
    receipt_delivery(
        request,
        receipt(request_id, refusal, OUTCOME_REFUSED, &request.bound, 0),
        Vec::new(),
        Vec::new(),
        0,
        consume_replay,
        0,
        false,
    )
}

fn receipt_delivery(
    request: &CommunicationRequestMsg,
    receipt: CommunicationReceipt,
    recipient_frames: Vec<CommunicationRecipientFrame>,
    recipients: Vec<u32>,
    audience: u32,
    consume_replay: bool,
    elapsed_raw: i64,
    elapsed_known: bool,
) -> CommunicationDelivery {
    let sender_team = recipient_frames.first().map_or(request.bound.team_id, |frame| frame.team_id);
    let sender_team = if request.bound.team_id != 0 { request.bound.team_id } else { sender_team };
    CommunicationDelivery {
        origin_session: request.origin_session.clone(),
        origin_lifetime: request.lifetime,
        receipt: receipt.encode_to_vec(),
        recipient_frames,
        recipients,
        audience,
        sender_team,
        sender_id: request.bound.player_id,
        room_id: request.bound.room_id,
        run_id: request.bound.run_id,
        view_epoch: request.bound.view_epoch,
        scope_generation: request.lifetime.scope_generation,
        elapsed_raw,
        elapsed_known,
        replay: None,
        consume_replay,
    }
}

fn project_target(
    target: Option<&super::communication::DisclosedTarget>,
    sender_team: u32,
    recipient_team: u32,
    view: Option<&TeamViewDisclosure<'_>>,
) -> Option<super::communication::DisclosedTarget> {
    use super::communication::DisclosedTarget;
    use super::selective::{CanonicalEntityKey, MappingVisibility};
    let disclosed = *target?;
    let Some(view) = view else {
        return (recipient_team == sender_team).then_some(disclosed);
    };
    let Some(identity) = view.identity else {
        return (recipient_team == sender_team).then_some(DisclosedTarget { view_epoch: view.view_epoch, ..disclosed });
    };
    if identity.team_id() != recipient_team {
        return None;
    }
    let canonical = CanonicalEntityKey { id: disclosed.canonical_id, generation: disclosed.generation };
    let mapping = identity.replica_for(canonical)?;
    if mapping.visibility != MappingVisibility::Disclosed {
        return None;
    }
    Some(DisclosedTarget {
        replica_id: mapping.replica_id.get(),
        canonical_id: disclosed.canonical_id,
        generation: disclosed.generation,
        disclosure_epoch: mapping.disclosure_epoch,
        view_epoch: view.view_epoch,
    })
}

fn recipient_frames(
    event: &AdmittedCommunication,
    request_id: u64,
    sender_view: u64,
    views: &[TeamViewDisclosure<'_>],
    player_epochs: &[(u32, u64, u64)],
    roster: Option<&[crate::runtime::native::moba_match::CommunicationMember]>,
) -> Vec<CommunicationRecipientFrame> {
    let mut frames = Vec::new();
    for player_id in &event.recipients {
        let team_id = roster
            .and_then(|rows| rows.iter().find(|member| member.player_id == *player_id).map(|member| member.team_id))
            .unwrap_or(event.sender_team);
        let (view_epoch, generation) = if let Some((_, epoch, generation)) = player_epochs.iter().find(|row| row.0 == *player_id) {
            if *epoch == 0 {
                continue;
            }
            (*epoch, *generation)
        } else if let Some(view) = views.iter().find(|view| view.team_id == team_id) {
            if view.view_epoch == 0 {
                continue;
            }
            (view.view_epoch, 0)
        } else if team_id == event.sender_team && sender_view != 0 {
            (sender_view, 0)
        } else {
            continue;
        };
        let mut projected = event.clone();
        if let AdmittedBody::Ping { target, .. } = &mut projected.body {
            let view = views.iter().find(|view| view.team_id == team_id);
            *target = project_target(target.as_ref(), event.sender_team, team_id, view);
        }
        let correlation = if *player_id == event.id.sender_id { request_id } else { 0 };
        let Some(wire) = wire_event(&projected, view_epoch, correlation) else {
            continue;
        };
        frames.push(CommunicationRecipientFrame {
            player_id: *player_id,
            team_id,
            view_epoch,
            session_generation: generation,
            bytes: wire.encode_to_vec(),
            expires_at_raw: wire.expires_at_raw,
            admitted_at_raw: wire.admitted_at_raw,
            kind: wire.kind,
        });
    }
    frames
}

fn wire_event(
    event: &AdmittedCommunication,
    view_epoch: u64,
    request_id: u64,
) -> Option<CommunicationWireEvent> {
    let mut wire = CommunicationWireEvent {
        schema_version: COMMUNICATION_SCHEMA_VERSION,
        room_id: event.id.room_id,
        run_id: event.id.run_id,
        sender_id: event.id.sender_id,
        sender_team: event.sender_team,
        sequence: event.id.sequence,
        view_epoch,
        audience: audience_code(event.audience),
        receipt_request_id: request_id,
        ..Default::default()
    };
    match &event.body {
        AdmittedBody::Emote { content_id, admitted_at_raw, expires_at_raw } => {
            wire.kind = KIND_EMOTE;
            wire.content_id = u32::from(*content_id);
            wire.admitted_at_raw = *admitted_at_raw;
            wire.expires_at_raw = *expires_at_raw;
        }
        AdmittedBody::Status { hud, admitted_at_raw, expires_at_raw } => {
            wire.kind = KIND_STATUS;
            encode_hud(&mut wire, hud);
            wire.admitted_at_raw = *admitted_at_raw;
            wire.expires_at_raw = *expires_at_raw;
        }
        AdmittedBody::Ping { semantic, point, target, hud, admitted_at_raw, expires_at_raw } => {
            wire.kind = KIND_PING;
            wire.semantic = *semantic as u32;
            wire.has_point = true;
            wire.point_x_raw = point.x.raw();
            wire.point_y_raw = point.y.raw();
            wire.admitted_at_raw = *admitted_at_raw;
            wire.expires_at_raw = *expires_at_raw;
            if let Some(target) = target {
                wire.has_target = true;
                wire.replica_id = target.replica_id;
                wire.generation = target.generation;
                wire.disclosure_epoch = target.disclosure_epoch;
            }
            if let Some(hud) = hud {
                encode_hud(&mut wire, hud);
            }
        }
        AdmittedBody::Chat { channel, text, admitted_at_raw } => {
            wire.kind = KIND_CHAT;
            wire.chat_channel = match channel {
                ChatChannel::Team => CHANNEL_TEAM,
                ChatChannel::All => CHANNEL_ALL,
            };
            wire.text = text.clone();
            wire.admitted_at_raw = *admitted_at_raw;
        }
        AdmittedBody::ObjectiveVote { objective_id, home, admitted_at_raw, expires_at_raw } => {
            wire.kind = KIND_OBJECTIVE;
            wire.admitted_at_raw = *admitted_at_raw;
            wire.objective_id = objective_id.clone();
            wire.objective_x_raw = home.x.raw();
            wire.objective_y_raw = home.y.raw();
            wire.expires_at_raw = *expires_at_raw;
        }
    }
    let encoded = wire.encode_to_vec();
    if encoded.len() > EVENT_MAX_BYTES || !wire_event_shape(&wire) {
        None
    } else {
        Some(wire)
    }
}

fn quiet_hud(event: &CommunicationWireEvent) -> bool {
    event.hud_kind == HUD_NONE
        && event.hud_index == 0
        && event.ability_remaining_raw == 0
        && event.catalog_id == 0
        && event.cooldown_bits == 0
        && event.score_player_id == 0
        && event.kills == 0
        && event.deaths == 0
        && event.assists == 0
}

fn encode_hud(wire: &mut CommunicationWireEvent, hud: &HudFact) {
    match hud {
        HudFact::OwnedAbility { index, remaining_raw } => {
            wire.hud_kind = HUD_ABILITY; wire.hud_index = u32::from(*index);
            wire.ability_remaining_raw = *remaining_raw;
        }
        HudFact::OwnedItem { slot, catalog_id, cooldown_bits } => {
            wire.hud_kind = HUD_ITEM; wire.hud_index = u32::from(*slot);
            wire.catalog_id = u32::from(*catalog_id); wire.cooldown_bits = *cooldown_bits;
        }
        HudFact::PublicScore { player_id, kills, deaths, assists } => {
            wire.hud_kind = HUD_SCORE; wire.score_player_id = *player_id;
            wire.kills = *kills; wire.deaths = *deaths; wire.assists = *assists;
        }
    }
}

fn hud_fact_shape(event: &CommunicationWireEvent) -> bool {
    match event.hud_kind {
        HUD_NONE => quiet_hud(event),
        HUD_ABILITY => event.hud_index <= u32::from(u8::MAX) && event.ability_remaining_raw >= 0
            && event.catalog_id == 0 && event.cooldown_bits == 0 && event.score_player_id == 0
            && event.kills == 0 && event.deaths == 0 && event.assists == 0,
        HUD_ITEM => event.hud_index <= u32::from(u8::MAX) && event.ability_remaining_raw == 0
            && u16::try_from(event.catalog_id).ok().is_some_and(|id| omoba_template_ids::MOBA_ITEM_CATALOG.iter().any(|r| r.catalog_id == id))
            && f32::from_bits(event.cooldown_bits).is_finite() && f32::from_bits(event.cooldown_bits) >= 0.0
            && event.score_player_id == 0 && event.kills == 0 && event.deaths == 0 && event.assists == 0,
        HUD_SCORE => event.hud_index == 0 && event.score_player_id != 0 && event.catalog_id == 0
            && event.ability_remaining_raw == 0 && event.cooldown_bits == 0,
        _ => false,
    }
}

fn quiet_objective(event: &CommunicationWireEvent) -> bool {
    event.objective_id.is_empty() && event.objective_x_raw == 0 && event.objective_y_raw == 0
}

fn text_is_clean(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_TEXT_BYTES
        && text.chars().count() <= MAX_TEXT_SCALARS
        && text.chars().any(|scalar| !scalar.is_whitespace())
        && text.chars().all(|scalar| !scalar.is_control() && scalar != '\u{2028}' && scalar != '\u{2029}')
}

fn wire_event_shape(event: &CommunicationWireEvent) -> bool {
    if event.schema_version != COMMUNICATION_SCHEMA_VERSION
        || event.room_id == 0
        || event.run_id == 0
        || event.sender_id == 0
        || event.sender_team == 0
        || event.sequence == 0
        || event.view_epoch == 0
        || event.admitted_at_raw < 0
        || (event.kind != KIND_EMOTE && event.content_id != 0)
    {
        return false;
    }
    let audience_ok = match event.kind {
        KIND_PING | KIND_OBJECTIVE | KIND_EMOTE | KIND_STATUS => event.audience == AUDIENCE_TEAM,
        KIND_CHAT => match (event.audience, event.chat_channel) {
            (AUDIENCE_TEAM, CHANNEL_TEAM) | (AUDIENCE_ALL, CHANNEL_ALL) => true,
            _ => false,
        },
        _ => false,
    };
    if !audience_ok {
        return false;
    }
    match event.kind {
        KIND_EMOTE | KIND_STATUS => {
            !event.has_point && !event.has_target && event.point_x_raw == 0 && event.point_y_raw == 0
                && event.semantic == 0 && event.replica_id == 0 && event.generation == 0
                && event.disclosure_epoch == 0 && event.text.is_empty() && event.chat_channel == 0
                && quiet_objective(event) && event.expires_at_raw > event.admitted_at_raw
                && if event.kind == KIND_EMOTE {
                    quiet_hud(event) && u16::try_from(event.content_id).ok()
                        .and_then(omoba_template_ids::communication_content)
                        .is_some_and(|row| event.expires_at_raw.checked_sub(event.admitted_at_raw) == Some(i64::from(row.duration_raw)))
                } else {
                    event.hud_kind != HUD_NONE && hud_fact_shape(event)
                        && event.expires_at_raw.checked_sub(event.admitted_at_raw) == Some(super::communication::PING_MARKER_TTL_RAW)
                }
        }
        KIND_PING => {
            event.has_point
                && semantic_from(event.semantic).is_some()
                && event.text.is_empty()
                && event.chat_channel == 0
                && quiet_objective(event)
                && event.expires_at_raw > event.admitted_at_raw
                && (!event.has_target || (event.replica_id != 0 && event.generation != 0 && event.disclosure_epoch != 0))
                && (event.has_target || (event.replica_id == 0 && event.generation == 0 && event.disclosure_epoch == 0))
                && hud_fact_shape(event)
        }
        KIND_CHAT => {
            !event.has_point
                && !event.has_target
                && event.point_x_raw == 0
                && event.point_y_raw == 0
                && event.semantic == 0
                && quiet_objective(event)
                && event.expires_at_raw == 0
                && quiet_hud(event)
                && event.replica_id == 0
                && text_is_clean(&event.text)
        }
        KIND_OBJECTIVE => {
            !event.has_point
                && !event.has_target
                && event.semantic == 0
                && event.text.is_empty()
                && event.chat_channel == 0
                && quiet_hud(event)
                && !event.objective_id.is_empty()
                && event.objective_id.len() <= MAX_OBJECTIVE_ID_BYTES
                && event.objective_id.chars().all(|scalar| !scalar.is_control())
                && event.expires_at_raw > event.admitted_at_raw
        }
        _ => false,
    }
}

fn known_refusal(refusal: u32) -> bool {
    (1..=39).contains(&refusal) || refusal == REFUSAL_LOCAL_TRANSPORT
}

fn receipt_shape(receipt: &CommunicationReceipt) -> bool {
    if receipt.schema_version != COMMUNICATION_SCHEMA_VERSION {
        return false;
    }
    match receipt.outcome_kind {
        OUTCOME_ADMITTED_EVENT => {
            receipt.view_epoch != 0
                && receipt.refusal == REFUSAL_ADMITTED
                && receipt.request_id != 0
                && receipt.sequence != 0
                && receipt.room_id != 0
                && receipt.run_id != 0
                && receipt.sender_id != 0
        }
        OUTCOME_MUTE_UPDATED => {
            receipt.view_epoch != 0
                && receipt.refusal == REFUSAL_ADMITTED
                && receipt.request_id != 0
                && receipt.sequence == 0
                && receipt.room_id != 0
                && receipt.run_id != 0
        }
        OUTCOME_REFUSED => {
            receipt.sequence == 0 && receipt.refusal != REFUSAL_ADMITTED && known_refusal(receipt.refusal)
        }
        OUTCOME_REPLAY => {
            receipt.view_epoch != 0
                && receipt.refusal == REFUSAL_ADMITTED
                && receipt.request_id != 0
                && receipt.sequence == 0
                && receipt.room_id != 0
                && receipt.run_id != 0
        }
        OUTCOME_HELLO => {
            receipt.request_id == 0 && receipt.sequence == 0 && (receipt.refusal == REFUSAL_ADMITTED || known_refusal(receipt.refusal))
        }
        _ => false,
    }
}

fn hello_result_shape(result: &CommunicationHelloResult) -> bool {
    if result.schema_version != COMMUNICATION_SCHEMA_VERSION {
        return false;
    }
    if result.agreed {
        result.refusal == REFUSAL_ADMITTED
            && result.protocol_version == COMMUNICATION_PROTOCOL_VERSION
            && result.room_id != 0
            && result.run_id != 0
            && result.player_id != 0
            && result.team_id != 0
            && result.view_epoch != 0
    } else {
        !result.agreed && result.protocol_version == 0 && result.refusal != REFUSAL_ADMITTED && known_refusal(result.refusal)
    }
}

/// Field 17 is reserved. A nonzero varint, or any non-varint encoding of that
/// field, is a canonical guess and must not reach admission. A zero varint does
/// not end the scan: a later duplicate can still be a guess. A truncated field
/// is `Err`, including a key with no value.
fn reserved_canonical_guess(bytes: &[u8]) -> Result<bool, ()> {
    let mut index = 0;
    let mut guess = false;
    while index < bytes.len() {
        let (key, next) = read_proto_varint(bytes, index)?;
        let field = key >> 3;
        let wire = (key & 7) as u8;
        if field == 17 && wire == 0 {
            let (value, after) = read_proto_varint(bytes, next)?;
            if value != 0 {
                guess = true;
            }
            index = after;
        } else if field == 17 {
            guess = true;
            index = skip_proto_field(bytes, next, wire)?;
        } else {
            index = skip_proto_field(bytes, next, wire)?;
        }
    }
    Ok(guess)
}

fn read_proto_varint(bytes: &[u8], mut index: usize) -> Result<(u64, usize), ()> {
    let mut value = 0u64;
    let mut shift = 0;
    loop {
        if index >= bytes.len() || shift >= 64 {
            return Err(());
        }
        let byte = bytes[index];
        index += 1;
        value |= u64::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, index));
        }
        shift += 7;
    }
}

fn skip_proto_field(bytes: &[u8], index: usize, wire: u8) -> Result<usize, ()> {
    match wire {
        0 => Ok(read_proto_varint(bytes, index)?.1),
        1 => index.checked_add(8).filter(|end| *end <= bytes.len()).ok_or(()),
        2 => {
            let (len, next) = read_proto_varint(bytes, index)?;
            let len = usize::try_from(len).map_err(|_| ())?;
            next.checked_add(len).filter(|end| *end <= bytes.len()).ok_or(())
        }
        5 => index.checked_add(4).filter(|end| *end <= bytes.len()).ok_or(()),
        _ => Err(()),
    }
}

/// Resolves one filtered replica to the internal canonical claim. The lookup is
/// `TeamIdentityState::canonical_for_replica`. Callers do not build `ReplicaEntityId`.
fn authorize_filtered_target(
    target: Option<ClientVisibleTarget>,
    disclosure: Option<&TeamIdentityState>,
    sender_team: u32,
    current_view_epoch: u64,
) -> Result<Option<TargetClaim>, u32> {
    let Some(target) = target else {
        return Ok(None);
    };
    if target.replica_id == 0
        || target.disclosure_epoch == 0
        || target.view_epoch == 0
        || target.view_epoch != current_view_epoch
    {
        return Err(REFUSAL_STALE_OR_HIDDEN_TARGET);
    }
    let Some(disclosure) = disclosure else {
        return Err(REFUSAL_STALE_OR_HIDDEN_TARGET);
    };
    if disclosure.team_id() != sender_team {
        return Err(REFUSAL_DISCLOSURE_TEAM_MISMATCH);
    }
    let canonical = match disclosure.canonical_for_replica(target.replica_id, target.disclosure_epoch) {
        Ok(canonical) => canonical,
        Err(
            TeamIdentityError::UnknownReplica
            | TeamIdentityError::RetiredReplica
            | TeamIdentityError::StaleDisclosureEpoch
            | TeamIdentityError::UnknownCanonical
            | TeamIdentityError::AllocationExhausted,
        ) => return Err(REFUSAL_STALE_OR_HIDDEN_TARGET),
    };
    if target.generation != 0 && canonical.generation != target.generation {
        return Err(REFUSAL_STALE_OR_HIDDEN_TARGET);
    }
    let Some(mapping) = disclosure.replica_for(canonical) else {
        return Err(REFUSAL_STALE_OR_HIDDEN_TARGET);
    };
    if mapping.visibility != MappingVisibility::Disclosed
        || mapping.replica_id.get() != target.replica_id
        || mapping.disclosure_epoch != target.disclosure_epoch
    {
        return Err(REFUSAL_STALE_OR_HIDDEN_TARGET);
    }
    Ok(Some(TargetClaim {
        replica_id: target.replica_id,
        canonical_id: canonical.id,
        generation: canonical.generation,
        disclosure_epoch: target.disclosure_epoch,
        view_epoch: current_view_epoch,
    }))
}

fn decode_body(message: &CommunicationSubmit) -> Result<DecodedBody, u32> {
    if message.kind != KIND_EMOTE && message.content_id != 0 { return Err(REFUSAL_MALFORMED); }
    let quiet_target = !message.has_target
        && message.replica_id == 0
        && message.generation == 0
        && message.disclosure_epoch == 0
        && message.target_view_epoch == 0;
    let quiet_hud = message.hud_kind == HUD_NONE && message.hud_index == 0 && message.hud_player_id == 0;
    let quiet_point = !message.has_point && message.point_x_raw == 0 && message.point_y_raw == 0;
    match message.kind {
        KIND_PING => {
            if message.nonce == 0
                || !message.has_point
                || !message.text.is_empty()
                || !message.objective_id.is_empty()
                || message.chat_channel != 0
                || message.mute_sender_id != 0
                || message.muted
            {
                return Err(REFUSAL_MALFORMED);
            }
            let semantic = semantic_from(message.semantic).ok_or(REFUSAL_MALFORMED)?;
            let target = if message.has_target {
                if message.replica_id == 0
                    || message.disclosure_epoch == 0
                    || message.target_view_epoch == 0
                {
                    return Err(REFUSAL_MALFORMED);
                }
                Some(ClientVisibleTarget {
                    replica_id: message.replica_id,
                    generation: message.generation,
                    disclosure_epoch: message.disclosure_epoch,
                    view_epoch: message.target_view_epoch,
                })
            } else if !quiet_target {
                return Err(REFUSAL_MALFORMED);
            } else {
                None
            };
            let hud = match message.hud_kind {
                HUD_NONE if quiet_hud => None,
                HUD_ABILITY if message.hud_player_id == 0 => {
                    Some(HudClaim::OwnedAbility { index: u8::try_from(message.hud_index).map_err(|_| REFUSAL_MALFORMED)? })
                }
                HUD_ITEM if message.hud_player_id == 0 => {
                    Some(HudClaim::OwnedItem { slot: u8::try_from(message.hud_index).map_err(|_| REFUSAL_MALFORMED)? })
                }
                HUD_SCORE if message.hud_index == 0 && message.hud_player_id != 0 => {
                    Some(HudClaim::PublicScore { player_id: message.hud_player_id })
                }
                _ => return Err(REFUSAL_MALFORMED),
            };
            Ok(DecodedBody::Ping {
                semantic,
                point: Vec2::new(Fixed64::from_raw(message.point_x_raw), Fixed64::from_raw(message.point_y_raw)),
                target,
                hud,
            })
        }
        KIND_CHAT => {
            if message.nonce == 0
                || !quiet_point
                || !quiet_target
                || !quiet_hud
                || message.semantic != 0
                || !message.objective_id.is_empty()
                || message.mute_sender_id != 0
                || message.muted
            {
                return Err(REFUSAL_MALFORMED);
            }
            let channel = match message.chat_channel {
                CHANNEL_TEAM => ChatChannel::Team,
                CHANNEL_ALL => ChatChannel::All,
                _ => return Err(REFUSAL_MALFORMED),
            };
            Ok(DecodedBody::Chat { channel, text: message.text.clone() })
        }
        KIND_OBJECTIVE => {
            if message.nonce == 0
                || !quiet_point
                || !quiet_target
                || !quiet_hud
                || message.semantic != 0
                || !message.text.is_empty()
                || message.chat_channel != 0
                || message.mute_sender_id != 0
            {
                return Err(REFUSAL_MALFORMED);
            }
            Ok(DecodedBody::Objective { objective_id: message.objective_id.clone() })
        }
        KIND_MUTE => {
            if !quiet_point
                || !quiet_target
                || !quiet_hud
                || message.semantic != 0
                || !message.text.is_empty()
                || !message.objective_id.is_empty()
                || message.chat_channel != 0
                || message.mute_sender_id == 0
            {
                return Err(REFUSAL_MALFORMED);
            }
            Ok(DecodedBody::Mute { sender_id: message.mute_sender_id, muted: message.muted })
        }
        KIND_STATUS => {
            if message.nonce == 0 || !quiet_point || !quiet_target || message.semantic != 0
                || !message.text.is_empty() || !message.objective_id.is_empty()
                || message.chat_channel != 0 || message.mute_sender_id != 0 || message.muted {
                return Err(REFUSAL_MALFORMED);
            }
            let hud = match message.hud_kind {
                HUD_ABILITY if message.hud_player_id == 0 => HudClaim::OwnedAbility { index: u8::try_from(message.hud_index).map_err(|_| REFUSAL_MALFORMED)? },
                HUD_ITEM if message.hud_player_id == 0 => HudClaim::OwnedItem { slot: u8::try_from(message.hud_index).map_err(|_| REFUSAL_MALFORMED)? },
                HUD_SCORE if message.hud_index == 0 && message.hud_player_id != 0 => HudClaim::PublicScore { player_id: message.hud_player_id },
                _ => return Err(REFUSAL_MALFORMED),
            };
            Ok(DecodedBody::Status { hud })
        }
        KIND_EMOTE => {
            if message.nonce == 0 || !quiet_point
                || !quiet_target
                || !quiet_hud
                || message.semantic != 0
                || !message.text.is_empty()
                || !message.objective_id.is_empty()
                || message.chat_channel != 0
                || message.mute_sender_id != 0
                || message.muted
            {
                return Err(REFUSAL_MALFORMED);
            }
            Ok(DecodedBody::Emote { content_id: u16::try_from(message.content_id).map_err(|_| REFUSAL_MALFORMED)? })
        }
        _ => Err(REFUSAL_MALFORMED),
    }
}

fn semantic_from(value: u32) -> Option<PingSemantic> {
    Some(match value {
        1 => PingSemantic::Retreat,
        2 => PingSemantic::Push,
        3 => PingSemantic::OnMyWay,
        4 => PingSemantic::AllIn,
        5 => PingSemantic::Assist,
        6 => PingSemantic::NeedVision,
        7 => PingSemantic::EnemyMissing,
        8 => PingSemantic::EnemyVision,
        9 => PingSemantic::General,
        10 => PingSemantic::Warning,
        _ => return None,
    })
}

fn bounded_decode<T: Message + Default>(bytes: &[u8], max: usize) -> Result<T, u32> {
    if bytes.is_empty() {
        return Err(REFUSAL_MALFORMED);
    }
    if bytes.len() > max {
        return Err(REFUSAL_OVERSIZED);
    }
    T::decode(bytes).map_err(|_| REFUSAL_MALFORMED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::native::moba_match::{communication_roster_match, shop_fixture_slot, MobaMatchPhase};
    use crate::runtime::native::{Hero, Inventory, ItemInstance, PlayerOwner};
    use crate::runtime::selective::{CanonicalEntityKey, TeamIdentityState};
    use prost::Message;
    use specs::{Builder, WorldExt};

    fn scope() -> CommunicationScope {
        CommunicationScope { room_id: 9, run_id: 3 }
    }

    fn bound(player: u32, team: u32) -> BoundCommunicationSession {
        BoundCommunicationSession {
            secure_joined: true,
            negotiated: true,
            observer: false,
            room_id: 9,
            run_id: 3,
            player_id: player,
            team_id: team,
            view_epoch: 4,
        }
    }

    fn claimed(player: u32, team: u32) -> ClaimedIdentity {
        ClaimedIdentity { room_id: 9, run_id: 3, player_id: player, team_id: team, observer: false }
    }

    fn roster() -> MobaMatch {
        let hero = |id| {
            shop_fixture_slot(id, if id <= 2 || id == 5 { 0 } else { 1 }, Some(specs::Entity::new(id, specs::world::Generation::new(1))))
        };
        let mut dead = shop_fixture_slot(2, 0, None);
        dead.deaths = 1;
        communication_roster_match(vec![hero(1), dead, hero(3), hero(4), hero(5)], MobaMatchPhase::Playing, Fixed64::ZERO, None)
    }

    fn lifetime_of(bound: BoundCommunicationSession) -> CommunicationLifetime {
        CommunicationLifetime {
            generation: 1,
            binding_generation: 1,
            room_id: bound.room_id,
            run_id: bound.run_id,
            player_id: bound.player_id,
            team_id: bound.team_id,
            view_epoch: bound.view_epoch,
            scope_generation: 1,
        }
    }

    fn origin_bytes(delivery: &CommunicationDelivery) -> Option<&Vec<u8>> {
        delivery.recipient_frames.iter().find(|frame| frame.player_id == delivery.sender_id).map(|frame| &frame.bytes)
    }

    fn submit_ping(player: u32, team: u32, nonce: u64) -> CommunicationRequestMsg {
        let bound = bound(player, team);
        CommunicationRequestMsg {
            origin_session: format!("s{player}"),
            lifetime: lifetime_of(bound),
            bound,
            body: CommunicationRequestBody::Submit(DecodedSubmit {
                request_id: nonce,
                nonce,
                claimed: claimed(player, team),
                claimed_view_epoch: 4,
                body: DecodedBody::Ping {
                    semantic: PingSemantic::Assist,
                    point: Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                    target: None,
                    hud: None,
                },
            }),
        }
    }

    fn authority_fixture() -> CommunicationAuthority {
        let mut authority = CommunicationAuthority::default();
        authority.set_scope(Some(scope()));
        authority
    }

    fn handle(authority: &mut CommunicationAuthority, request: &CommunicationRequestMsg, match_state: &MobaMatch) -> CommunicationDelivery {
        authority.handle(request, match_state, None, None, 4)
    }

    #[test]
    fn communication_transport_tags_are_distinct_from_rtt_shop_and_ward() {
        assert!(reserved_tags_unchanged());
        assert_eq!(TAG_COMMUNICATION_HELLO, 0x2F);
        assert_eq!(TAG_COMMUNICATION_REPLAY, 0x35);
        assert!(!communication_tag(TAG_PING_REQ));
        let ping = crate::game_proto::PingRequest { client_send_us: 5 };
        assert!(decode_submit(&ping.encode_to_vec()).is_err());
    }

    #[test]
    fn communication_transport_rejects_truncated_and_oversized_payloads() {
        assert_eq!(decode_submit(&[]), Err(REFUSAL_MALFORMED));
        assert_eq!(decode_submit(&[0xFF, 0x01]), Err(REFUSAL_MALFORMED));
        assert_eq!(decode_submit(&vec![1; SUBMIT_MAX_BYTES + 1]), Err(REFUSAL_OVERSIZED));
        assert_eq!(decode_replay_batch(&vec![1; REPLAY_BATCH_MAX_BYTES + 1]), Err(REFUSAL_OVERSIZED));
        let submit = CommunicationSubmit {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: 7,
            nonce: 1,
            claimed_room_id: 9,
            claimed_run_id: 3,
            claimed_player_id: 1,
            claimed_team_id: 1,
            claimed_view_epoch: 4,
            kind: KIND_CHAT,
            chat_channel: CHANNEL_TEAM,
            text: "team".into(),
            ..Default::default()
        };
        let decoded = decode_submit(&submit.encode_to_vec()).unwrap();
        assert!(matches!(decoded.body, DecodedBody::Chat { channel: ChatChannel::Team, .. }));
        let mut both = submit.clone();
        both.chat_channel = 0;
        both.kind = KIND_PING;
        both.has_point = true;
        both.semantic = 9;
        both.text = "no".into();
        assert_eq!(decode_submit(&both.encode_to_vec()).err(), Some(REFUSAL_MALFORMED));
    }

    #[test]
    fn communication_transport_refuses_unsupported_unauthenticated_and_foreign_sessions() {
        let mut cold = BoundCommunicationSession { secure_joined: false, negotiated: false, ..bound(1, 1) };
        let hello = CommunicationHello {
            protocol_version: COMMUNICATION_PROTOCOL_VERSION,
            claimed_player_id: 1,
            claimed_team_id: 1,
            ..Default::default()
        };
        assert_eq!(negotiate_hello(&cold, Some(scope()), &hello).refusal, REFUSAL_UNAUTHENTICATED);
        cold.secure_joined = true;
        cold.observer = true;
        let observer = CommunicationHello { claimed_observer: true, ..hello.clone() };
        assert_eq!(negotiate_hello(&cold, Some(scope()), &observer).refusal, REFUSAL_OBSERVER_DENIED);
        let player = bound(1, 1);
        let mut unsupported = hello.clone();
        for version in [1, 2, COMMUNICATION_PROTOCOL_VERSION + 1] {
            unsupported.protocol_version = version;
            assert!(!negotiate_hello(&BoundCommunicationSession { negotiated: false, ..player }, Some(scope()), &unsupported).agreed);
        }
        let agreed = negotiate_hello(&player, Some(scope()), &hello);
        assert!(agreed.agreed);
        assert_eq!((agreed.room_id, agreed.run_id, agreed.view_epoch), (9, 3, 4));
        let foreign = CommunicationHello { claimed_player_id: 4, ..hello };
        assert_eq!(negotiate_hello(&player, Some(scope()), &foreign).refusal, REFUSAL_IDENTITY_MISMATCH);
        let mut other_room = hello.clone();
        other_room.claimed_room_id = 8;
        other_room.claimed_run_id = 3;
        assert_eq!(negotiate_hello(&player, Some(scope()), &other_room).refusal, REFUSAL_IDENTITY_MISMATCH);
        assert_eq!(negotiate_hello(&player, None, &hello).refusal, REFUSAL_SCOPE_UNAVAILABLE);

        let mut authority = authority_fixture();
        let mut request = submit_ping(1, 1, 1);
        request.bound.negotiated = false;
        let match_state = roster();
        let before = authority.ledger().history().len();
        let denied = handle(&mut authority, &request, &match_state);
        let receipt = decode_receipt(&denied.receipt).unwrap();
        assert_eq!(receipt.refusal, REFUSAL_NOT_NEGOTIATED);
        assert!(origin_bytes(&denied).is_none());
        assert_eq!(authority.ledger().history().len(), before);
        request.bound.negotiated = true;
        request.body = CommunicationRequestBody::Submit(DecodedSubmit {
            request_id: 1,
            nonce: 1,
            claimed: ClaimedIdentity { player_id: 9, ..claimed(1, 1) },
            claimed_view_epoch: 4,
            body: DecodedBody::Ping {
                semantic: PingSemantic::General,
                point: Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                target: None,
                hud: None,
            },
        });
        assert_eq!(decode_receipt(&handle(&mut authority, &request, &match_state).receipt).unwrap().refusal, REFUSAL_IDENTITY_MISMATCH);
        assert!(authority.ledger().history().is_empty());
    }

    #[test]
    fn communication_transport_team_and_all_recipients_stay_exact() {
        let mut authority = authority_fixture();
        let match_state = roster();
        let team = handle(&mut authority, &submit_ping(1, 1, 1), &match_state);
        assert_eq!(team.recipients, vec![1, 2, 5]);
        assert_eq!(team.audience, AUDIENCE_TEAM);
        let event = decode_event(origin_bytes(&team).unwrap()).unwrap();
        assert_eq!(event.audience, AUDIENCE_TEAM);
        assert_eq!(event.receipt_request_id, 1);
        let mut chat = submit_ping(1, 1, 2);
        if let CommunicationRequestBody::Submit(submit) = &mut chat.body {
            submit.body = DecodedBody::Chat { channel: ChatChannel::All, text: "all channel".into() };
        }
        let all = handle(&mut authority, &chat, &match_state);
        assert_eq!(all.recipients, vec![1, 2, 3, 4, 5]);
        assert_eq!(all.audience, AUDIENCE_ALL);
        let mut bad = submit_ping(1, 1, 3);
        if let CommunicationRequestBody::Submit(submit) = &mut bad.body {
            submit.body = DecodedBody::Chat { channel: ChatChannel::Team, text: " \n".into() };
        }
        let refused = handle(&mut authority, &bad, &match_state);
        assert_eq!(decode_receipt(&refused.receipt).unwrap().refusal, REFUSAL_TEXT_CONTROL);
        assert!(origin_bytes(&refused).is_none());
        assert_eq!(authority.ledger().history().len(), 2);
    }

    #[test]
    fn communication_transport_roster_death_phase_results_and_spectator() {
        let mut authority = authority_fixture();
        let mut match_state = roster();
        let dead = handle(&mut authority, &submit_ping(2, 1, 1), &match_state);
        assert_eq!(decode_receipt(&dead.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        assert!(dead.recipients.contains(&2));
        match_state.phase = MobaMatchPhase::Finished { winner: Some(0), tick: 8 };
        let closed = handle(&mut authority, &submit_ping(1, 1, 2), &match_state);
        assert_eq!(decode_receipt(&closed.receipt).unwrap().refusal, REFUSAL_RESULTS_CLOSED);
        let mut spectator = submit_ping(1, 1, 3);
        spectator.bound.observer = true;
        if let CommunicationRequestBody::Submit(submit) = &mut spectator.body {
            submit.claimed.observer = true;
        }
        let before = authority.ledger().history().len();
        assert_eq!(decode_receipt(&handle(&mut authority, &spectator, &match_state).receipt).unwrap().refusal, REFUSAL_OBSERVER_DENIED);
        assert_eq!(authority.ledger().history().len(), before);
    }

    #[test]
    fn communication_transport_rejects_hidden_targets_and_strips_them_on_replay() {
        let mut authority = authority_fixture();
        let match_state = communication_roster_match(
            vec![
                shop_fixture_slot(1, 0, Some(specs::Entity::new(1, specs::world::Generation::new(1)))),
                shop_fixture_slot(2, 0, Some(specs::Entity::new(2, specs::world::Generation::new(1)))),
            ],
            MobaMatchPhase::Playing,
            Fixed64::ZERO,
            Some("three_lane_training"),
        );
        let mut identity = TeamIdentityState::new(1);
        let canonical = CanonicalEntityKey { id: 0xA1B2_C3D4, generation: 2 };
        let mapping = identity.disclose(canonical).unwrap();
        let inside = Vec2::new(Fixed64::from_i32(0), Fixed64::from_i32(1400));
        let mut request = submit_ping(1, 1, 1);
        if let CommunicationRequestBody::Submit(submit) = &mut request.body {
            submit.body = DecodedBody::Ping {
                semantic: PingSemantic::EnemyMissing,
                point: inside,
                target: Some(ClientVisibleTarget {
                    replica_id: mapping.replica_id.get(),
                    generation: canonical.generation,
                    disclosure_epoch: mapping.disclosure_epoch,
                    view_epoch: 4,
                }),
                hud: None,
            };
        }
        let admitted = authority.handle(&request, &match_state, None, Some(&identity), 4);
        let event = decode_event(origin_bytes(&admitted).unwrap()).unwrap();
        assert!(event.has_target);
        assert_eq!(event.replica_id, mapping.replica_id.get());
        assert!(!event.encode_to_vec().windows(4).any(|bytes| bytes == canonical.id.to_le_bytes()));
        identity.forget(canonical).unwrap();
        let replay = authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s2".into(),
                lifetime: lifetime_of(bound(2, 1)),
                bound: bound(2, 1),
                body: CommunicationRequestBody::Replay { request_id: 9, claimed: claimed(2, 1), claimed_view_epoch: 4 },
            },
            &match_state,
            None,
            Some(&identity),
            4,
        );
        let batch = decode_replay_batch(replay.replay.as_ref().unwrap()).unwrap();
        assert_eq!(batch.events.len(), 1);
        assert!(!batch.events[0].has_target);
        assert!(batch.events[0].has_point);
        assert_eq!(batch.events[0].point_y_raw, inside.y.raw());
    }

    #[test]
    fn communication_transport_clock_rate_dedup_mute_and_objective_expiry() {
        let mut rate_authority = authority_fixture();
        let mut match_state = roster();
        for nonce in 1..=4 {
            let admitted = handle(&mut rate_authority, &submit_ping(1, 1, nonce), &match_state);
            assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        }
        assert_eq!(decode_receipt(&handle(&mut rate_authority, &submit_ping(1, 1, 5), &match_state).receipt).unwrap().refusal, REFUSAL_RATE_LIMITED);
        match_state.elapsed = Fixed64::from_raw(super::super::communication::RATE_WINDOW_RAW + 1);
        assert_eq!(decode_receipt(&handle(&mut rate_authority, &submit_ping(1, 1, 1), &match_state).receipt).unwrap().refusal, REFUSAL_DUPLICATE);
        // Refused duplicates are atomic and do not advance the ledger clock.
        assert_eq!(decode_receipt(&handle(&mut rate_authority, &submit_ping(1, 1, 6), &match_state).receipt).unwrap().refusal, REFUSAL_ADMITTED);
        match_state.elapsed = Fixed64::ZERO;
        assert_eq!(decode_receipt(&handle(&mut rate_authority, &submit_ping(1, 1, 7), &match_state).receipt).unwrap().refusal, REFUSAL_REGRESSED_CLOCK);

        let mut fresh_authority = authority_fixture();
        let mut match_state = roster();
        let mute = CommunicationRequestMsg {
            origin_session: "s2".into(),
            lifetime: lifetime_of(bound(2, 1)),
            bound: bound(2, 1),
            body: CommunicationRequestBody::Submit(DecodedSubmit {
                request_id: 1,
                nonce: 0,
                claimed: claimed(2, 1),
                claimed_view_epoch: 4,
                body: DecodedBody::Mute { sender_id: 1, muted: true },
            }),
        };
        assert_eq!(decode_receipt(&handle(&mut fresh_authority, &mute, &match_state).receipt).unwrap().outcome_kind, OUTCOME_MUTE_UPDATED);
        let hidden = handle(&mut fresh_authority, &submit_ping(1, 1, 1), &match_state);
        assert!(!hidden.recipients.contains(&2));
        assert!(hidden.recipients.contains(&5));
        let mut vote = submit_ping(1, 1, 2);
        match_state.elapsed = Fixed64::from_raw(super::super::communication::RATE_WINDOW_RAW + 1);
        let map = communication_roster_match(
            match_state.heroes.clone(),
            MobaMatchPhase::Playing,
            match_state.elapsed,
            Some("three_lane_training"),
        );
        let objective = map.jungle_camps[0].definition.id;
        if let CommunicationRequestBody::Submit(submit) = &mut vote.body {
            submit.nonce = 2;
            submit.request_id = 2;
            submit.body = DecodedBody::Objective { objective_id: objective.to_string() };
        }
        assert_eq!(decode_receipt(&fresh_authority.handle(&vote, &map, None, None, 4).receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let mut expired = map.clone();
        expired.elapsed = Fixed64::from_raw(map.elapsed.raw() + super::super::communication::OBJECTIVE_VOTE_TTL_RAW);
        let replay = fresh_authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s1".into(),
                lifetime: lifetime_of(bound(1, 1)),
                bound: bound(1, 1),
                body: CommunicationRequestBody::Replay { request_id: 4, claimed: claimed(1, 1), claimed_view_epoch: 4 },
            },
            &expired,
            None,
            None,
            4,
        );
        let batch = decode_replay_batch(replay.replay.as_ref().unwrap()).unwrap();
        assert!(batch.events.iter().all(|event| event.kind != KIND_OBJECTIVE));
    }

    #[test]
    fn communication_transport_rejoin_and_replacement_replay_are_isolated() {
        let mut authority = authority_fixture();
        let match_state = roster();
        handle(&mut authority, &submit_ping(1, 1, 1), &match_state);
        let query = CommunicationRequestMsg {
            origin_session: "s5".into(),
            lifetime: lifetime_of(bound(5, 1)),
            bound: bound(5, 1),
            body: CommunicationRequestBody::Replay { request_id: 3, claimed: claimed(5, 1), claimed_view_epoch: 4 },
        };
        let first = handle(&mut authority, &query, &match_state);
        assert!(first.consume_replay);
        let batch = decode_replay_batch(first.replay.as_ref().unwrap()).unwrap();
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].sender_team, 1);
        let mut enemy = bound(3, 2);
        enemy.player_id = 3;
        let leaked = authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s3".into(),
                lifetime: lifetime_of(enemy),
                bound: enemy,
                body: CommunicationRequestBody::Replay { request_id: 8, claimed: claimed(3, 2), claimed_view_epoch: 4 },
            },
            &match_state,
            None,
            None,
            4,
        );
        assert!(decode_replay_batch(leaked.replay.as_ref().unwrap()).unwrap().events.is_empty());
        authority.set_scope(Some(CommunicationScope { room_id: 10, run_id: 4 }));
        let mut replaced = query.clone();
        replaced.bound.room_id = 10;
        replaced.bound.run_id = 4;
        if let CommunicationRequestBody::Replay { claimed, .. } = &mut replaced.body {
            claimed.room_id = 10;
            claimed.run_id = 4;
        }
        let empty = authority.handle(&replaced, &match_state, None, None, 4);
        assert!(decode_replay_batch(empty.replay.as_ref().unwrap()).unwrap().events.is_empty());
        assert_eq!(authority.ledger().history().len(), 1);
    }

    #[test]
    fn communication_inbox_producer_owns_identity_and_rejects_before_pending_mutation() {
        let body = DecodedBody::Emote { content_id: 103 };
        let mut inbox = CommunicationInbox::default();
        assert_eq!(inbox.prepare_submit(1, 1, &body), Err(REFUSAL_NOT_NEGOTIATED));
        let hello = negotiate_hello(&bound(1, 1), Some(scope()), &hello_for(1, 1));
        inbox.apply_hello(&hello).unwrap();
        assert_eq!(inbox.prepare_submit(1, 1, &DecodedBody::Chat { channel: ChatChannel::Team, text: "\n".into() }), Err(REFUSAL_MALFORMED));
        assert!(inbox.pending.is_empty());
        for id in 1..=COMMUNICATION_PENDING_SUBMISSIONS as u64 {
            let submit = inbox.prepare_submit(id, id, &body).unwrap();
            let decoded = decode_submit(&submit.encode_to_vec()).unwrap();
            assert_eq!(decoded.body, body);
            assert_eq!(decoded.claimed, claimed(1, 1));
            assert_eq!(decoded.claimed_view_epoch, 4);
        }
        assert_eq!(inbox.prepare_submit(9, 9, &body), Err(REFUSAL_BACKPRESSURE));
        assert_eq!(inbox.pending.len(), COMMUNICATION_PENDING_SUBMISSIONS);
        inbox.fail_submit_locally(1).unwrap();
        let status = inbox.prepare_submit(9, 9, &DecodedBody::Status { hud: HudClaim::PublicScore { player_id: 3 } }).unwrap();
        assert_eq!((status.kind, status.hud_kind, status.hud_player_id), (KIND_STATUS, HUD_SCORE, 3));
        assert!(!status.has_point && !status.has_target);
    }

    #[test]
    fn communication_transport_client_inbox_and_renderer_roundtrip() {
        let mut inbox = CommunicationInbox::default();
        let hello = negotiate_hello(&bound(1, 1), Some(scope()), &CommunicationHello {
            protocol_version: COMMUNICATION_PROTOCOL_VERSION,
            claimed_player_id: 1,
            claimed_team_id: 1,
            ..Default::default()
        });
        let presented = inbox.apply_hello(&hello).unwrap();
        assert!(communication_presentation_acceptable(&presented, 1, 1));
        assert!(presented.negotiated);
        assert_eq!(presented.source_generation, 1);
        assert_eq!(presented.presentation_epoch, 1);
        assert_ne!(presented.presentation_epoch, presented.events.first().map(|event| u64::from(event.generation)).unwrap_or(0));
        inbox.begin_submit(11).unwrap();
        let mut authority = authority_fixture();
        let delivery = handle(&mut authority, &submit_ping(1, 1, 11), &roster());
        let receipt = decode_receipt(&delivery.receipt).unwrap();
        let event = decode_event(origin_bytes(&delivery).unwrap()).unwrap();
        let shown = inbox.apply_event(&event).unwrap();
        let correlated = inbox.apply_receipt(&receipt).unwrap();
        assert_eq!(shown.events[0].receipt_request_id, correlated.receipt.as_ref().unwrap().request_id);
        assert_eq!(correlated.receipt.as_ref().unwrap().outcome_kind, OUTCOME_ADMITTED_EVENT);
        assert!(inbox.begin_submit(11).is_err());
        let local = inbox.fail_submit_locally(11);
        assert!(local.is_none());
        inbox.begin_submit(12).unwrap();
        let failed = inbox.fail_submit_locally(12).unwrap();
        assert_eq!(failed.receipt.as_ref().unwrap().refusal, REFUSAL_LOCAL_TRANSPORT);
        assert_eq!(failed.events.len(), 1);
        let retired = inbox.observe_view(5).unwrap();
        assert!(retired.events.is_empty());
        assert_eq!(retired.presentation_epoch, presented.presentation_epoch + 1);
        assert_eq!(retired.source_generation, presented.source_generation);
        assert!(!communication_presentation_acceptable(&retired, 2, 1));
    }

    #[test]
    fn communication_transport_malformed_replay_batch_is_atomic() {
        let mut inbox = CommunicationInbox::default();
        inbox.apply_hello(&negotiate_hello(&bound(1, 1), Some(scope()), &CommunicationHello {
            protocol_version: COMMUNICATION_PROTOCOL_VERSION,
            claimed_player_id: 1,
            claimed_team_id: 1,
            ..Default::default()
        })).unwrap();
        let mut authority = authority_fixture();
        let delivery = handle(&mut authority, &submit_ping(1, 1, 1), &roster());
        let good = decode_event(origin_bytes(&delivery).unwrap()).unwrap();
        inbox.apply_event(&good).unwrap();
        let epoch = inbox.presentation_epoch();
        let generation = inbox.source_generation();
        let mut bad = CommunicationReplayBatch {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: 4,
            refusal: REFUSAL_ADMITTED,
            room_id: 9,
            run_id: 3,
            view_epoch: 4,
            already_replayed: false,
            events: vec![good.clone()],
            elapsed_raw: 0,
        };
        bad.events.push(CommunicationWireEvent { room_id: 10, ..good.clone() });
        assert!(inbox.apply_replay(&bad).is_none());
        assert_eq!(inbox.presentation().events.len(), 1);
        assert_eq!((inbox.presentation_epoch(), inbox.source_generation()), (epoch, generation));
        bad.events.pop();
        bad.room_id = 8;
        assert!(inbox.apply_replay(&bad).is_none());
        assert_eq!(inbox.presentation().events[0].sequence, good.sequence);
        bad.room_id = 9;
        let applied = inbox.apply_replay(&bad).unwrap();
        assert_eq!(applied.events.len(), 1);
    }

    #[test]
    fn communication_transport_unknown_content_and_missing_hud_fact_fail_atomically() {
        let mut authority = authority_fixture();
        let match_state = roster();
        for body in [DecodedBody::Emote { content_id: 0 }, DecodedBody::Status { hud: HudClaim::OwnedAbility { index: 0 } }] {
            let request = CommunicationRequestMsg {
                origin_session: "s1".into(),
                lifetime: lifetime_of(bound(1, 1)),
                bound: bound(1, 1),
                body: CommunicationRequestBody::Submit(DecodedSubmit {
                    request_id: 1,
                    nonce: 1,
                    claimed: claimed(1, 1),
                    claimed_view_epoch: 4,
                    body: body.clone(),
                }),
            };
            let delivery = handle(&mut authority, &request, &match_state);
            let receipt = decode_receipt(&delivery.receipt).unwrap();
            assert!(matches!(receipt.refusal, REFUSAL_CONTENT_UNAVAILABLE | REFUSAL_HUD_FACT_UNAVAILABLE));
            assert_eq!(receipt.outcome_kind, OUTCOME_REFUSED);
            assert!(authority.ledger().history().is_empty());
        }
    }

    #[test]
    fn communication_transport_emote_and_status_decode_admit_roundtrip_and_expire() {
        let mut authority = authority_fixture();
        let match_state = roster();
        for (nonce, body, kind) in [
            (1, DecodedBody::Emote { content_id: 103 }, KIND_EMOTE),
            (2, DecodedBody::Status { hud: HudClaim::PublicScore { player_id: 3 } }, KIND_STATUS),
        ] {
            let mut wire = CommunicationSubmit {
                schema_version: COMMUNICATION_SCHEMA_VERSION, request_id: nonce, nonce,
                claimed_room_id: 9, claimed_run_id: 3, claimed_player_id: 1, claimed_team_id: 1,
                claimed_view_epoch: 4, kind, ..Default::default()
            };
            if kind == KIND_EMOTE { wire.content_id = 103; }
            else { wire.hud_kind = HUD_SCORE; wire.hud_player_id = 3; }
            let decoded = decode_submit(&wire.encode_to_vec()).unwrap();
            assert_eq!(decoded.body, body);
            let request = CommunicationRequestMsg {
                origin_session: "s1".into(), lifetime: lifetime_of(bound(1, 1)), bound: bound(1, 1),
                body: CommunicationRequestBody::Submit(decoded),
            };
            let delivery = handle(&mut authority, &request, &match_state);
            assert_eq!(decode_receipt(&delivery.receipt).unwrap().refusal, REFUSAL_ADMITTED);
            let frame = delivery.recipient_frames.iter().find(|f| f.player_id == 1).unwrap();
            let event = decode_event(&frame.bytes).unwrap();
            assert_eq!(event.kind, kind); assert!(!event.has_point); assert!(!event.has_target);
            assert!(event.point_x_raw == 0 && event.point_y_raw == 0 && event.text.is_empty());
            assert!(event.expires_at_raw > event.admitted_at_raw);
            let mut forged = event.clone(); forged.point_x_raw = 1;
            assert!(decode_event(&forged.encode_to_vec()).is_err());
            let mut mixed = wire.clone(); mixed.text = "client prose".into();
            assert!(decode_submit(&mixed.encode_to_vec()).is_err());
            let mut old = wire.clone(); old.schema_version = 1;
            assert!(decode_submit(&old.encode_to_vec()).is_err());
        }
        assert_eq!(authority.ledger().history().len(), 2);
    }

    #[test]
    fn communication_transport_route_selects_negotiated_sessions_only() {
        let rows = [
            CommunicationRouteSession {
                session_id: "ally",
                player_id: 2,
                team_id: 1,
                view_epoch: 4,
                room_id: 9,
                run_id: 3,
                generation: 1,
                negotiated: true,
                secure_joined: true,
                observer: false,
            },
            CommunicationRouteSession {
                session_id: "old",
                player_id: 2,
                team_id: 1,
                view_epoch: 3,
                room_id: 9,
                run_id: 3,
                generation: 1,
                negotiated: true,
                secure_joined: true,
                observer: false,
            },
            CommunicationRouteSession {
                session_id: "enemy",
                player_id: 3,
                team_id: 2,
                view_epoch: 4,
                room_id: 9,
                run_id: 3,
                generation: 1,
                negotiated: true,
                secure_joined: true,
                observer: false,
            },
            CommunicationRouteSession {
                session_id: "silent",
                player_id: 5,
                team_id: 1,
                view_epoch: 4,
                room_id: 9,
                run_id: 3,
                generation: 1,
                negotiated: false,
                secure_joined: true,
                observer: false,
            },
        ];
        assert_eq!(route_communication_event(&rows, &[1, 2, 5], AUDIENCE_TEAM, 1, 9, 3, 4), vec!["ally".to_string()]);
        assert_eq!(route_communication_event(&rows, &[1, 2, 3, 4, 5], AUDIENCE_ALL, 1, 9, 3, 4), vec!["ally".to_string(), "enemy".to_string()]);
    }

    #[test]
    fn communication_transport_live_hud_fact_uses_world_and_hides_canonical() {
        let mut world = World::new();
        world.register::<Hero>();
        world.register::<Inventory>();
        world.register::<PlayerOwner>();
        let mut hero = Hero::new("spark_hero".into(), "Spark".into(), "Test".into());
        hero.abilities.push("spark".into());
        hero.ability_cooldowns.insert("spark".into(), Fixed64::from_i32(5));
        let mut inventory = Inventory::new();
        inventory.slots[0] = Some(ItemInstance { item_id: omoba_template_ids::MOBA_ITEM_CATALOG[0].id.to_string(), cooldown_remaining: 1.5 });
        let entity = world.create_entity().with(hero).with(inventory).with(PlayerOwner::new(1)).build();
        let mut match_state = communication_roster_match(
            vec![shop_fixture_slot(1, 0, Some(entity)), shop_fixture_slot(2, 0, None)],
            MobaMatchPhase::Playing,
            Fixed64::ZERO,
            None,
        );
        match_state.heroes[0].poison_cached_progression_for_test();
        let mut request = submit_ping(1, 1, 1);
        if let CommunicationRequestBody::Submit(submit) = &mut request.body {
            submit.body = DecodedBody::Ping {
                semantic: PingSemantic::General,
                point: Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                target: None,
                hud: Some(HudClaim::OwnedAbility { index: 0 }),
            };
        }
        let mut authority = authority_fixture();
        let missing = authority.handle(&request, &match_state, None, None, 4);
        assert_eq!(decode_receipt(&missing.receipt).unwrap().refusal, REFUSAL_HUD_FACT_UNAVAILABLE);
        let delivery = authority.handle(&request, &match_state, Some(&world), None, 4);
        let event = decode_event(origin_bytes(&delivery).unwrap()).unwrap();
        assert_eq!(event.hud_kind, HUD_ABILITY);
        assert_eq!(event.ability_remaining_raw, Fixed64::from_i32(5).raw());
        assert_ne!(event.ability_remaining_raw, Fixed64::from_i32(99).raw());
    }

    fn hello_for(player: u32, team: u32) -> CommunicationHello {
        CommunicationHello {
            protocol_version: COMMUNICATION_PROTOCOL_VERSION,
            claimed_player_id: player,
            claimed_team_id: team,
            claimed_view_epoch: 4,
            ..Default::default()
        }
    }

    fn bind_live(registry: &mut CommunicationSessionRegistry, session: &str, player: u32, team: u32) -> CommunicationLifetime {
        registry.publish_scope(Some(scope()));
        registry.publish_clock(0, true);
        registry.open_session(session).expect("session generation");
        let result = negotiate_hello(&bound(player, team), Some(scope()), &hello_for(player, team));
        assert!(result.agreed);
        registry.note_hello(session, &result, false, true).expect("binding")
    }

    fn with_nonce(request: &CommunicationRequestMsg, nonce: u64) -> CommunicationRequestMsg {
        let mut request = request.clone();
        if let CommunicationRequestBody::Submit(submit) = &mut request.body {
            submit.nonce = nonce;
            submit.request_id = nonce;
        }
        request
    }

    #[test]
    fn communication_e752_settled_request_floor_blocks_reuse() {
        let mut inbox = CommunicationInbox::default();
        let mut hello = negotiate_hello(&bound(1, 1), Some(scope()), &hello_for(1, 1));
        hello.request_floor = 10;
        inbox.apply_hello(&hello).unwrap();
        assert!(inbox.begin_submit(10).is_err());
        assert!(inbox.begin_submit(11).is_ok());
        let mut authority = authority_fixture();
        let delivery = handle(&mut authority, &submit_ping(1, 1, 11), &roster());
        let receipt = decode_receipt(&delivery.receipt).unwrap();
        inbox.apply_receipt(&receipt).unwrap();
        assert!(inbox.begin_submit(11).is_err());
        assert!(inbox.begin_submit(12).is_ok());
    }

    #[test]
    fn communication_e752_live_session_token_blocks_replaced_scope() {
        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let match_state = roster();
        let life = bind_live(&mut registry, "kcp_a", 1, 1);
        let request = with_nonce(&{
            let mut request = submit_ping(1, 1, 1);
            request.origin_session = "kcp_a".into();
            request.lifetime = life;
            request
        }, 1);
        let admitted = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let before = authority.ledger().history().len();

        registry.close_session("kcp_a");
        let stale = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&stale.receipt).unwrap().refusal, REFUSAL_UNAUTHENTICATED);
        assert_eq!(authority.ledger().history().len(), before);

        let replaced = bind_live(&mut registry, "kcp_a", 1, 1);
        assert_ne!(replaced.generation, life.generation);
        let resurrected = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_ne!(decode_receipt(&resurrected.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        assert_eq!(authority.ledger().history().len(), before);

        registry.publish_scope(Some(CommunicationScope { room_id: 10, run_id: 1 }));
        let mut current = submit_ping(1, 1, 2);
        current.origin_session = "kcp_a".into();
        current.lifetime = replaced;
        let retired = admit_authenticated(&mut authority, &mut registry, &current, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&retired.receipt).unwrap().refusal, REFUSAL_UNAUTHENTICATED);
        assert_eq!(authority.ledger().history().len(), before);
        let absent = admit_authenticated(&mut authority, &mut registry, &current, None, None, None, 4, &[]);
        assert_eq!(decode_receipt(&absent.receipt).unwrap().refusal, REFUSAL_UNAUTHENTICATED);
        assert_eq!(authority.ledger().history().len(), before);
    }

    #[test]
    fn communication_e752_recipient_epochs_correlation_and_queued_scope() {
        let mut authority = authority_fixture();
        let match_state = roster();
        let mut chat = submit_ping(1, 1, 1);
        if let CommunicationRequestBody::Submit(submit) = &mut chat.body {
            submit.body = DecodedBody::Chat { channel: ChatChannel::All, text: "visible".into() };
        }
        let views = [
            TeamViewDisclosure { team_id: 1, view_epoch: 4, identity: None },
            TeamViewDisclosure { team_id: 2, view_epoch: 9, identity: None },
        ];
        let epochs = [(1_u32, 4_u64, 3_u64), (3, 9, 8)];
        let all = authority.handle_addressed(&chat, &match_state, None, None, 4, &views, &epochs);
        let sender = decode_event(origin_bytes(&all).unwrap()).unwrap();
        assert_eq!(sender.receipt_request_id, 1);
        assert_eq!(sender.view_epoch, 4);
        let enemy = all.recipient_frames.iter().find(|frame| frame.player_id == 3).unwrap();
        let enemy_event = decode_event(&enemy.bytes).unwrap();
        assert_eq!(enemy_event.receipt_request_id, 0);
        assert_eq!(enemy_event.view_epoch, 9);
        assert_ne!(enemy_event.view_epoch, sender.view_epoch);

        let mut team = submit_ping(1, 1, 2);
        if let CommunicationRequestBody::Submit(submit) = &mut team.body {
            submit.body = DecodedBody::Chat { channel: ChatChannel::Team, text: "allies".into() };
        }
        let team_delivery = authority.handle_addressed(&team, &match_state, None, None, 4, &views, &epochs);
        assert!(team_delivery.recipient_frames.iter().all(|frame| frame.team_id == 1));

        let mut identity = TeamIdentityState::new(1);
        let canonical = CanonicalEntityKey { id: 0x11, generation: 1 };
        let mapping = identity.disclose(canonical).unwrap();
        let mut targeted = submit_ping(1, 1, 3);
        if let CommunicationRequestBody::Submit(submit) = &mut targeted.body {
            submit.body = DecodedBody::Ping {
                semantic: PingSemantic::EnemyMissing,
                point: Vec2::new(Fixed64::from_i32(1), Fixed64::ZERO),
                target: Some(ClientVisibleTarget {
                    replica_id: mapping.replica_id.get(),
                    generation: canonical.generation,
                    disclosure_epoch: mapping.disclosure_epoch,
                    view_epoch: 4,
                }),
                hud: None,
            };
        }
        let disclosed_views = [
            TeamViewDisclosure { team_id: 1, view_epoch: 4, identity: Some(&identity) },
            TeamViewDisclosure { team_id: 2, view_epoch: 9, identity: None },
        ];
        let hidden = authority.handle_addressed(&targeted, &match_state, None, Some(&identity), 4, &disclosed_views, &epochs);
        assert_eq!(decode_receipt(&hidden.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        assert!(hidden.recipient_frames.iter().all(|frame| frame.team_id == 1));
        let ally = hidden.recipient_frames.iter().find(|frame| frame.player_id == 5).unwrap();
        let ally_event = decode_event(&ally.bytes).unwrap();
        assert!(ally_event.has_target);
        assert_eq!(ally_event.receipt_request_id, 0);
        assert!(decode_event(origin_bytes(&hidden).unwrap()).unwrap().has_target);
        assert!(!ally.bytes.windows(4).any(|bytes| bytes == canonical.id.to_le_bytes()));
        identity.forget(canonical).unwrap();
        let replaced = authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s5".into(),
                lifetime: lifetime_of(bound(5, 1)),
                bound: bound(5, 1),
                body: CommunicationRequestBody::Replay { request_id: 20, claimed: claimed(5, 1), claimed_view_epoch: 4 },
            },
            &match_state, None, Some(&identity), 4,
        );
        let replayed = decode_replay_batch(replaced.replay.as_ref().unwrap()).unwrap();
        assert!(replayed.events.iter().filter(|event| event.kind == KIND_PING).all(|event| !event.has_target));

        let session = |id, player, team, epoch, generation| CommunicationRouteSession {
            session_id: id,
            player_id: player,
            team_id: team,
            view_epoch: epoch,
            room_id: 9,
            run_id: 3,
            generation,
            negotiated: true,
            secure_joined: true,
            observer: false,
        };
        let rows = [session("ally", 1, 1, 4, 3), session("enemy", 3, 2, 9, 8)];
        let frame = all.recipient_frames.iter().find(|frame| frame.player_id == 3).unwrap();
        assert_eq!(route_live_frame(&rows, frame, AUDIENCE_ALL, 1, 9, 3, all.scope_generation, all.scope_generation, 0, true).as_deref(), Some("enemy"));
        assert!(route_live_frame(&rows, frame, AUDIENCE_ALL, 1, 9, 3, all.scope_generation, all.scope_generation.saturating_add(1), 0, true).is_none());
        assert!(route_live_frame(&rows, frame, AUDIENCE_ALL, 1, 10, 1, all.scope_generation, all.scope_generation, 0, true).is_none());
        let mut muted = rows;
        muted[1].player_id = 99;
        assert!(route_live_frame(&muted, frame, AUDIENCE_ALL, 1, 9, 3, all.scope_generation, all.scope_generation, 0, true).is_none());
    }

    #[test]
    fn communication_e752_request_window_survives_sixty_four_and_hello() {
        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let mut match_state = roster();
        let life = bind_live(&mut registry, "kcp_a", 1, 1);
        let mut template = submit_ping(1, 1, 1);
        template.origin_session = "kcp_a".into();
        template.lifetime = life;
        for nonce in 1..=COMMUNICATION_REQUEST_WINDOW + 1 {
            if nonce > 1 && (nonce - 1) % 4 == 0 {
                match_state.elapsed = Fixed64::from_raw(match_state.elapsed.raw() + super::super::communication::RATE_WINDOW_RAW + 1);
                registry.publish_clock(match_state.elapsed.raw(), true);
            }
            let request = with_nonce(&template, nonce);
            let delivery = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
            assert_eq!(decode_receipt(&delivery.receipt).unwrap().refusal, REFUSAL_ADMITTED, "nonce {nonce}");
        }
        let ancient = admit_authenticated(&mut authority, &mut registry, &with_nonce(&template, 1), Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&ancient.receipt).unwrap().refusal, REFUSAL_INVALID_NONCE);
        assert!(registry.set_replay_consumed("kcp_a", life));
        let result = negotiate_hello(&bound(1, 1), Some(scope()), &hello_for(1, 1));
        let again = registry.note_hello("kcp_a", &result, false, true).unwrap();
        assert_eq!(again.binding_generation, life.binding_generation);
        assert!(registry.replay_consumed("kcp_a"));
        assert!(registry.request_floor(9, 3, 1) >= COMMUNICATION_REQUEST_WINDOW + 1);
        let still_ancient = admit_authenticated(&mut authority, &mut registry, &with_nonce(&template, 1), Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&still_ancient.receipt).unwrap().refusal, REFUSAL_INVALID_NONCE);

        registry.close_session("kcp_a");
        let reconnected = bind_live(&mut registry, "kcp_a", 1, 1);
        assert_ne!(reconnected.generation, life.generation);
        let mut revived = with_nonce(&template, 1);
        revived.lifetime = reconnected;
        let blocked = admit_authenticated(&mut authority, &mut registry, &revived, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&blocked.receipt).unwrap().refusal, REFUSAL_INVALID_NONCE);
        let fresh_nonce = COMMUNICATION_REQUEST_WINDOW + 2;
        let mut next = with_nonce(&template, fresh_nonce);
        next.lifetime = reconnected;
        match_state.elapsed = Fixed64::from_raw(match_state.elapsed.raw() + super::super::communication::RATE_WINDOW_RAW + 1);
        registry.publish_clock(match_state.elapsed.raw(), true);
        let admitted = admit_authenticated(&mut authority, &mut registry, &next, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
    }

    #[test]
    fn communication_e752_marker_expiry_superseded_vote_and_paused_clock() {
        let mut authority = authority_fixture();
        let mut match_state = roster();
        let ping = handle(&mut authority, &submit_ping(1, 1, 1), &match_state);
        assert_eq!(decode_receipt(&ping.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let paused = authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s1".into(),
                lifetime: lifetime_of(bound(1, 1)),
                bound: bound(1, 1),
                body: CommunicationRequestBody::Replay { request_id: 8, claimed: claimed(1, 1), claimed_view_epoch: 4 },
            },
            &match_state, None, None, 4,
        );
        let paused_batch = decode_replay_batch(paused.replay.as_ref().unwrap()).unwrap();
        assert_eq!(paused_batch.events.iter().filter(|event| event.kind == KIND_PING).count(), 1);

        match_state.elapsed = Fixed64::from_raw(super::super::communication::PING_MARKER_TTL_RAW);
        let expired = authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s5".into(),
                lifetime: lifetime_of(bound(5, 1)),
                bound: bound(5, 1),
                body: CommunicationRequestBody::Replay { request_id: 9, claimed: claimed(5, 1), claimed_view_epoch: 4 },
            },
            &match_state, None, None, 4,
        );
        let expired_batch = decode_replay_batch(expired.replay.as_ref().unwrap()).unwrap();
        assert!(expired_batch.events.iter().all(|event| event.kind != KIND_PING));

        match_state.elapsed = Fixed64::from_raw(super::super::communication::PING_MARKER_TTL_RAW + super::super::communication::RATE_WINDOW_RAW + 1);
        assert_eq!(decode_receipt(&handle(&mut authority, &submit_ping(1, 1, 2), &match_state).receipt).unwrap().refusal, REFUSAL_ADMITTED);
        match_state.elapsed = Fixed64::from_raw(super::super::communication::PING_MARKER_TTL_RAW);
        let regressed = authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s2".into(),
                lifetime: lifetime_of(bound(2, 1)),
                bound: bound(2, 1),
                body: CommunicationRequestBody::Replay { request_id: 10, claimed: claimed(2, 1), claimed_view_epoch: 4 },
            },
            &match_state, None, None, 4,
        );
        assert_eq!(decode_receipt(&regressed.receipt).unwrap().refusal, REFUSAL_REGRESSED_CLOCK);
        assert!(regressed.replay.is_none());

        let mut votes_authority = authority_fixture();
        let mut map = communication_roster_match(
            roster().heroes,
            MobaMatchPhase::Playing,
            Fixed64::from_raw(super::super::communication::RATE_WINDOW_RAW + 1),
            Some("three_lane_training"),
        );
        let first_id = map.jungle_camps[0].definition.id.to_string();
        let second_id = map.jungle_camps.get(1).map(|camp| camp.definition.id.to_string()).unwrap_or_else(|| first_id.clone());
        let mut vote = submit_ping(1, 1, 2);
        if let CommunicationRequestBody::Submit(submit) = &mut vote.body {
            submit.body = DecodedBody::Objective { objective_id: first_id };
        }
        assert_eq!(decode_receipt(&votes_authority.handle(&vote, &map, None, None, 4).receipt).unwrap().refusal, REFUSAL_ADMITTED);
        map.elapsed = Fixed64::from_raw(map.elapsed.raw() + super::super::communication::RATE_WINDOW_RAW + 1);
        if let CommunicationRequestBody::Submit(submit) = &mut vote.body {
            submit.nonce = 3;
            submit.request_id = 3;
            submit.body = DecodedBody::Objective { objective_id: second_id.clone() };
        }
        assert_eq!(decode_receipt(&votes_authority.handle(&vote, &map, None, None, 4).receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let votes = votes_authority.handle(
            &CommunicationRequestMsg {
                origin_session: "s1".into(),
                lifetime: lifetime_of(bound(1, 1)),
                bound: bound(1, 1),
                body: CommunicationRequestBody::Replay { request_id: 12, claimed: claimed(1, 1), claimed_view_epoch: 4 },
            },
            &map, None, None, 4,
        );
        let objectives: Vec<_> = decode_replay_batch(votes.replay.as_ref().unwrap()).unwrap().events.into_iter().filter(|event| event.kind == KIND_OBJECTIVE).collect();
        assert_eq!(objectives.len(), 1);
        assert_eq!(objectives[0].objective_id, second_id);

        let frame = ping.recipient_frames.iter().find(|frame| frame.player_id == 1).unwrap();
        let row = [CommunicationRouteSession {
            session_id: "s1", player_id: 1, team_id: 1, view_epoch: 4, room_id: 9, run_id: 3,
            generation: 0, negotiated: true, secure_joined: true, observer: false,
        }];
        assert!(route_live_frame(&row, frame, AUDIENCE_TEAM, 1, 9, 3, 1, 1, frame.expires_at_raw, true).is_none());
        assert_eq!(route_live_frame(&row, frame, AUDIENCE_TEAM, 1, 9, 3, 1, 1, 0, true).as_deref(), Some("s1"));
        assert!(route_live_frame(&row, frame, AUDIENCE_TEAM, 1, 9, 3, 1, 1, -1, false).is_none());
    }

    #[test]
    fn communication_e752_stale_claim_and_malformed_shapes() {
        let mut authority = authority_fixture();
        let match_state = roster();
        let mut stale = submit_ping(1, 1, 1);
        if let CommunicationRequestBody::Submit(submit) = &mut stale.body {
            submit.claimed_view_epoch = 3;
        }
        let before = authority.ledger().history().len();
        assert_eq!(decode_receipt(&handle(&mut authority, &stale, &match_state).receipt).unwrap().refusal, REFUSAL_VIEW_MISMATCH);
        assert_eq!(authority.ledger().history().len(), before);

        let mut wire = CommunicationSubmit {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: 4,
            nonce: 4,
            claimed_room_id: 9,
            claimed_run_id: 3,
            claimed_player_id: 1,
            claimed_team_id: 1,
            claimed_view_epoch: 0,
            kind: KIND_PING,
            semantic: 9,
            has_point: true,
            point_x_raw: 10,
            text: "plain".into(),
            ..Default::default()
        };
        assert!(decode_submit(&wire.encode_to_vec()).is_err());
        wire.text.clear();
        wire.claimed_view_epoch = 3;
        let decoded = decode_submit(&wire.encode_to_vec()).unwrap();
        let mut request = submit_ping(1, 1, 4);
        request.body = CommunicationRequestBody::Submit(decoded);
        assert_eq!(decode_receipt(&handle(&mut authority, &request, &match_state).receipt).unwrap().refusal, REFUSAL_VIEW_MISMATCH);

        let mut event = decode_event(origin_bytes(&handle(&mut authority, &submit_ping(1, 1, 5), &match_state)).unwrap()).unwrap();
        event.audience = AUDIENCE_ALL;
        event.chat_channel = CHANNEL_TEAM;
        assert!(decode_event(&event.encode_to_vec()).is_err());
        event.audience = AUDIENCE_TEAM;
        event.text = "\n".into();
        event.kind = KIND_CHAT;
        assert!(decode_event(&event.encode_to_vec()).is_err());

        let mut receipt = decode_receipt(&handle(&mut authority, &submit_ping(1, 1, 6), &match_state).receipt).unwrap();
        receipt.outcome_kind = 99;
        assert!(decode_receipt(&receipt.encode_to_vec()).is_err());
        receipt.outcome_kind = OUTCOME_REFUSED;
        receipt.refusal = 39;
        assert!(decode_receipt(&receipt.encode_to_vec()).is_err());

        let batch = CommunicationReplayBatch {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: 1,
            refusal: REFUSAL_ADMITTED,
            room_id: 9,
            run_id: 3,
            view_epoch: 4,
            elapsed_raw: -1,
            ..Default::default()
        };
        assert!(decode_replay_batch(&batch.encode_to_vec()).is_err());
    }

    #[test]
    fn communication_e752_hello_refusal_replacement_and_scope_run() {
        let mut inbox = CommunicationInbox::default();
        // The production sender observes the view before sending its first hello.
        inbox.note_transition(4, 0);
        inbox.note_hello_sent();
        let malformed = CommunicationHelloResult { schema_version: 0, agreed: true, refusal: REFUSAL_ADMITTED, ..Default::default() };
        assert!(inbox.apply_hello(&malformed).is_none());
        assert!(!inbox.agreed());
        assert!(inbox.hello_due());
        let agreed = negotiate_hello(&bound(1, 1), Some(scope()), &hello_for(1, 1));
        inbox.apply_hello(&agreed).unwrap();
        inbox.begin_submit(1).unwrap();
        inbox.apply_event(&decode_event(origin_bytes(&{
            let mut authority = authority_fixture();
            handle(&mut authority, &submit_ping(1, 1, 1), &roster())
        }).unwrap()).unwrap()).unwrap();
        let source = inbox.source_generation();
        let retained = inbox.presentation().events.len();
        assert!(inbox.apply_hello(&malformed).is_none());
        assert!(inbox.agreed());
        assert_eq!(inbox.source_generation(), source);
        assert_eq!(inbox.presentation().events.len(), retained);

        let mut refused = agreed.clone();
        refused.agreed = false;
        refused.protocol_version = 0;
        refused.refusal = REFUSAL_SCOPE_UNAVAILABLE;
        refused.room_id = 0;
        refused.run_id = 0;
        refused.player_id = 0;
        refused.team_id = 0;
        refused.view_epoch = 0;
        let retired = inbox.apply_hello(&refused).unwrap();
        assert!(!inbox.agreed());
        assert!(retired.events.is_empty());
        assert!(inbox.presentation().events.is_empty());
        assert!(!inbox.hello_retry_for_scope());
        inbox.note_transition(4, 0);
        assert!(!inbox.hello_due());
        inbox.note_transition(4, 8);
        assert!(inbox.hello_due());
        inbox.note_hello_sent();
        inbox.note_transition(4, 9);
        assert!(!inbox.hello_due());

        inbox.apply_hello(&agreed).unwrap();
        let source = inbox.source_generation();
        let mut moved = agreed.clone();
        moved.team_id = 2;
        let replaced = inbox.apply_hello(&moved).unwrap();
        assert!(replaced.source_generation > source);
        assert!(replaced.events.is_empty());
        inbox.note_transition(5, 9);
        assert!(inbox.hello_due());
        assert!(!inbox.agreed());

        let mut clock = CommunicationScopeClock::default();
        assert_eq!(clock.observe(9, true).unwrap().run_id, 1);
        assert_eq!(clock.observe(9, true).unwrap().run_id, 1);
        assert!(clock.observe(0, false).is_none());
        assert_eq!(clock.observe(9, true).unwrap().run_id, 2);
        clock.retire();
        assert!(clock.published().is_none());
        assert_eq!(clock.observe(9, true).unwrap().run_id, 3);
        assert!(clock.observe(0, true).is_none());
    }

    fn filtered_target_submit(
        nonce: u64,
        replica_id: u64,
        generation: u32,
        disclosure_epoch: u64,
        view_epoch: u64,
    ) -> Vec<u8> {
        CommunicationSubmit {
            schema_version: COMMUNICATION_SCHEMA_VERSION,
            request_id: nonce,
            nonce,
            claimed_room_id: 9,
            claimed_run_id: 3,
            claimed_player_id: 1,
            claimed_team_id: 1,
            claimed_view_epoch: 4,
            kind: KIND_PING,
            semantic: PingSemantic::EnemyMissing as u32,
            has_point: true,
            point_x_raw: 10,
            has_target: true,
            replica_id,
            generation,
            disclosure_epoch,
            target_view_epoch: view_epoch,
            ..Default::default()
        }
        .encode_to_vec()
    }

    #[test]
    fn communication_target_filtered_replica_resolves_without_canonical_wire() {
        let canonical = CanonicalEntityKey { id: 0xA1B2_C3D4, generation: 2 };
        let mut identity = TeamIdentityState::new(1);
        let mapping = identity.disclose(canonical).unwrap();
        let clean = filtered_target_submit(1, mapping.replica_id.get(), canonical.generation, mapping.disclosure_epoch, 4);
        assert!(!clean.windows(4).any(|bytes| bytes == canonical.id.to_le_bytes()));
        let mut guessed = clean.clone();
        guessed.extend_from_slice(&[0x88, 0x01, 0x01]);
        assert_eq!(super::reserved_canonical_guess(&guessed), Ok(true));
        assert_eq!(decode_submit(&guessed), Err(REFUSAL_MALFORMED));
        let mut zeroed = clean.clone();
        zeroed.extend_from_slice(&[0x88, 0x01, 0x00]);
        assert_eq!(super::reserved_canonical_guess(&zeroed), Ok(false));
        let visible = decode_submit(&zeroed).unwrap();
        let DecodedBody::Ping { target: Some(target), .. } = &visible.body else {
            panic!("decoded filtered target");
        };
        assert_eq!(target.replica_id, mapping.replica_id.get());
        assert_eq!(target.generation, canonical.generation);
        assert_eq!(target.disclosure_epoch, mapping.disclosure_epoch);

        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let match_state = roster();
        let life = bind_live(&mut registry, "kcp_a", 1, 1);
        let admit = |authority: &mut CommunicationAuthority, registry: &mut CommunicationSessionRegistry, disclosure: &TeamIdentityState, nonce: u64, replica: u64, generation: u32, epoch: u64, view: u64| {
            let decoded = decode_submit(&filtered_target_submit(nonce, replica, generation, epoch, view)).unwrap();
            let mut request = submit_ping(1, 1, nonce);
            request.origin_session = "kcp_a".into();
            request.lifetime = life;
            request.body = CommunicationRequestBody::Submit(decoded);
            admit_authenticated(authority, registry, &request, Some(&match_state), None, Some(disclosure), 4, &[])
        };
        assert_eq!(authority.ledger().history().len(), 0);
        // Real filtered clients know replica/disclosure/view, not canonical generation.
        let admitted = admit(&mut authority, &mut registry, &identity, 1, mapping.replica_id.get(), 0, mapping.disclosure_epoch, 4);
        assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let event = decode_event(origin_bytes(&admitted).unwrap()).unwrap();
        assert!(event.has_target);
        assert_eq!(event.replica_id, mapping.replica_id.get());
        assert_eq!(event.generation, canonical.generation);
        assert_eq!(event.disclosure_epoch, mapping.disclosure_epoch);
        assert!(!event.encode_to_vec().windows(4).any(|bytes| bytes == canonical.id.to_le_bytes()));
        let before = authority.ledger().history().len();

        let stale = admit(&mut authority, &mut registry, &identity, 2, mapping.replica_id.get(), canonical.generation, mapping.disclosure_epoch + 1, 4);
        assert_eq!(decode_receipt(&stale.receipt).unwrap().refusal, REFUSAL_STALE_OR_HIDDEN_TARGET);
        let wrong_generation = admit(&mut authority, &mut registry, &identity, 3, mapping.replica_id.get(), canonical.generation + 1, mapping.disclosure_epoch, 4);
        assert_eq!(decode_receipt(&wrong_generation.receipt).unwrap().refusal, REFUSAL_STALE_OR_HIDDEN_TARGET);
        let wrong_view = admit(&mut authority, &mut registry, &identity, 4, mapping.replica_id.get(), canonical.generation, mapping.disclosure_epoch, 3);
        assert_eq!(decode_receipt(&wrong_view.receipt).unwrap().refusal, REFUSAL_STALE_OR_HIDDEN_TARGET);

        let mut foreign = TeamIdentityState::new(2);
        let foreign_mapping = foreign.disclose(canonical).unwrap();
        let foreign_delivery = admit(&mut authority, &mut registry, &foreign, 5, foreign_mapping.replica_id.get(), canonical.generation, foreign_mapping.disclosure_epoch, 4);
        assert_eq!(decode_receipt(&foreign_delivery.receipt).unwrap().refusal, REFUSAL_DISCLOSURE_TEAM_MISMATCH);

        let mut hidden = identity.clone();
        hidden.remember(canonical).unwrap();
        let hidden_delivery = admit(&mut authority, &mut registry, &hidden, 6, mapping.replica_id.get(), canonical.generation, mapping.disclosure_epoch, 4);
        assert_eq!(decode_receipt(&hidden_delivery.receipt).unwrap().refusal, REFUSAL_STALE_OR_HIDDEN_TARGET);

        let mut retired = identity.clone();
        retired.forget(canonical).unwrap();
        let reused = admit(&mut authority, &mut registry, &retired, 7, mapping.replica_id.get(), canonical.generation, mapping.disclosure_epoch, 4);
        assert_eq!(decode_receipt(&reused.receipt).unwrap().refusal, REFUSAL_STALE_OR_HIDDEN_TARGET);
        assert_eq!(authority.ledger().history().len(), before);
    }

    #[test]
    fn communication_e755_nonce_commit_after_history_eviction() {
        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let mut match_state = roster();
        let life = bind_live(&mut registry, "kcp_a", 1, 1);
        let mut template = submit_ping(1, 1, 1);
        template.origin_session = "kcp_a".into();
        template.lifetime = life;
        let admitted_count = COMMUNICATION_REQUEST_WINDOW * 2 + 1;
        assert!(admitted_count > 128);
        for nonce in 1..=admitted_count {
            if nonce > 1 && (nonce - 1) % 4 == 0 {
                match_state.elapsed = Fixed64::from_raw(match_state.elapsed.raw() + super::super::communication::RATE_WINDOW_RAW + 1);
                registry.publish_clock(match_state.elapsed.raw(), true);
            }
            let delivery = admit_authenticated(
                &mut authority,
                &mut registry,
                &with_nonce(&template, nonce),
                Some(&match_state),
                None,
                None,
                4,
                &[],
            );
            assert_eq!(decode_receipt(&delivery.receipt).unwrap().refusal, REFUSAL_ADMITTED, "nonce {nonce}");
        }
        assert_eq!(authority.ledger().history().len(), MAX_HISTORY);
        assert_eq!(
            authority.ledger().history().front().unwrap().id.sequence,
            admitted_count - MAX_HISTORY as u64 + 1
        );
        assert_eq!(authority.ledger().admission_cursor().next_sequence, admitted_count + 1);
        assert_eq!(authority.ledger().admission_cursor().room_id, Some(9));
        assert_eq!(authority.ledger().admission_cursor().run_id, Some(3));
        assert!(registry.request_floor(9, 3, 1) >= admitted_count);
        let cursor = authority.ledger().admission_cursor();
        let floor = registry.request_floor(9, 3, 1);
        let history_len = authority.ledger().history().len();

        let repeated = admit_authenticated(
            &mut authority, &mut registry, &with_nonce(&template, admitted_count), Some(&match_state), None, None, 4, &[],
        );
        assert_eq!(decode_receipt(&repeated.receipt).unwrap().refusal, REFUSAL_DUPLICATE);
        let evicted_inside_window = admitted_count - MAX_HISTORY as u64;
        assert!(admitted_count - evicted_inside_window < COMMUNICATION_REQUEST_WINDOW);
        let evicted = admit_authenticated(
            &mut authority, &mut registry, &with_nonce(&template, evicted_inside_window), Some(&match_state), None, None, 4, &[],
        );
        assert_eq!(decode_receipt(&evicted.receipt).unwrap().refusal, REFUSAL_DUPLICATE);
        assert!(admitted_count - 1 >= COMMUNICATION_REQUEST_WINDOW);
        let ancient = admit_authenticated(
            &mut authority, &mut registry, &with_nonce(&template, 1), Some(&match_state), None, None, 4, &[],
        );
        assert_eq!(decode_receipt(&ancient.receipt).unwrap().refusal, REFUSAL_INVALID_NONCE);

        let mut mute = submit_ping(1, 1, 1);
        mute.origin_session = "kcp_a".into();
        mute.lifetime = life;
        if let CommunicationRequestBody::Submit(submit) = &mut mute.body {
            submit.request_id = admitted_count + 10;
            submit.nonce = 0;
            submit.body = DecodedBody::Mute { sender_id: 5, muted: true };
        }
        let muted = admit_authenticated(&mut authority, &mut registry, &mute, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&muted.receipt).unwrap().outcome_kind, OUTCOME_MUTE_UPDATED);
        assert_eq!(authority.ledger().admission_cursor(), cursor);
        assert_eq!(registry.request_floor(9, 3, 1), floor);
        assert_eq!(authority.ledger().history().len(), history_len);

        registry.close_session("kcp_a");
        let reconnected = bind_live(&mut registry, "kcp_a", 1, 1);
        assert_ne!(reconnected.generation, life.generation);
        let mut revived = with_nonce(&template, 1);
        revived.lifetime = reconnected;
        let blocked = admit_authenticated(&mut authority, &mut registry, &revived, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&blocked.receipt).unwrap().refusal, REFUSAL_INVALID_NONCE);
        assert_eq!(authority.ledger().admission_cursor(), cursor);
        let mut next = with_nonce(&template, admitted_count + 1);
        next.lifetime = reconnected;
        let admitted = admit_authenticated(&mut authority, &mut registry, &next, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        assert_eq!(authority.ledger().history().len(), MAX_HISTORY);
        assert_eq!(authority.ledger().admission_cursor().next_sequence, cursor.next_sequence + 1);
        assert_eq!(authority.ledger().admission_cursor().room_id, cursor.room_id);
        assert_eq!(authority.ledger().admission_cursor().run_id, cursor.run_id);
        assert!(registry.request_floor(9, 3, 1) >= admitted_count + 1);
        assert_eq!(
            authority.ledger().history().front().unwrap().id.sequence,
            admitted_count - MAX_HISTORY as u64 + 2
        );
    }

    #[test]
    fn communication_e755_idle_tick_publishes_scope_clock_before_hello() {
        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let match_state = roster();
        registry.open_session("kcp_a").expect("session");
        let result = negotiate_hello(&bound(1, 1), Some(scope()), &hello_for(1, 1));
        assert!(result.agreed);
        let early = registry.note_hello("kcp_a", &result, false, true).expect("early hello");
        assert_eq!(early.scope_generation, 0);
        let mut request = submit_ping(1, 1, 1);
        request.origin_session = "kcp_a".into();
        request.lifetime = early;
        let closed = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&closed.receipt).unwrap().refusal, REFUSAL_UNAUTHENTICATED);
        assert_eq!(authority.ledger().history().len(), 0);
        assert_eq!(registry.published_scope_generation(), 0);

        let idle = begin_communication_tick(
            &mut registry,
            CommunicationTick {
                scope: Some(scope()),
                elapsed_raw: 0,
                elapsed_known: true,
                queue_empty: true,
                delivery_full: false,
            },
        );
        assert!(!idle);
        assert_ne!(registry.published_scope_generation(), 0);
        assert_eq!(registry.elapsed(), (0, true));
        let life = registry.note_hello("kcp_a", &result, false, true).expect("hello after publish");
        assert_eq!(life.scope_generation, registry.published_scope_generation());
        request.lifetime = life;
        let admitted = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let frame = admitted.recipient_frames.iter().find(|frame| frame.player_id == 1).expect("sender frame");
        assert!(frame.expires_at_raw > frame.admitted_at_raw);
        let published = registry.published_scope_generation();
        let row = [CommunicationRouteSession {
            session_id: "kcp_a",
            player_id: frame.player_id,
            team_id: frame.team_id,
            view_epoch: frame.view_epoch,
            room_id: 9,
            run_id: 3,
            generation: frame.session_generation,
            negotiated: true,
            secure_joined: true,
            observer: false,
        }];
        assert_eq!(
            route_live_frame(&row, frame, AUDIENCE_TEAM, 1, 9, 3, published, registry.published_scope_generation(), 0, true).as_deref(),
            Some("kcp_a")
        );

        let expired = begin_communication_tick(
            &mut registry,
            CommunicationTick {
                scope: Some(scope()),
                elapsed_raw: frame.expires_at_raw,
                elapsed_known: true,
                queue_empty: true,
                delivery_full: false,
            },
        );
        assert!(!expired);
        assert_eq!(registry.published_scope_generation(), published);
        assert_eq!(registry.elapsed(), (frame.expires_at_raw, true));
        let (elapsed_raw, elapsed_known) = registry.elapsed();
        assert!(route_live_frame(
            &row, frame, AUDIENCE_TEAM, 1, 9, 3, published, registry.published_scope_generation(), elapsed_raw, elapsed_known,
        ).is_none());

        let held = begin_communication_tick(
            &mut registry,
            CommunicationTick {
                scope: Some(scope()),
                elapsed_raw: frame.expires_at_raw + 1,
                elapsed_known: true,
                queue_empty: false,
                delivery_full: true,
            },
        );
        assert!(!held);
        assert_eq!(registry.elapsed(), (frame.expires_at_raw + 1, true));
        assert_eq!(registry.published_scope_generation(), published);

        let before = authority.ledger().history().len();
        let retired = begin_communication_tick(
            &mut registry,
            CommunicationTick {
                scope: None,
                elapsed_raw: frame.expires_at_raw + 1,
                elapsed_known: true,
                queue_empty: true,
                delivery_full: false,
            },
        );
        assert!(!retired);
        assert!(registry.published_scope().is_none());
        assert_ne!(registry.published_scope_generation(), published);
        let stale = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&stale.receipt).unwrap().refusal, REFUSAL_UNAUTHENTICATED);
        let renegotiated = registry.note_hello("kcp_a", &result, false, true).expect("hello after retire");
        assert_ne!(renegotiated.scope_generation, 0);
        assert_eq!(renegotiated.scope_generation, registry.published_scope_generation());
        request.lifetime = renegotiated;
        let unavailable = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&unavailable.receipt).unwrap().refusal, REFUSAL_SCOPE_UNAVAILABLE);
        assert_eq!(authority.ledger().history().len(), before);
    }

    #[test]
    fn communication_e755_field17_scans_every_duplicate() {
        let clean = filtered_target_submit(1, 42, 2, 7, 4);
        let with = |extra: &[u8]| {
            let mut bytes = clean.clone();
            bytes.extend_from_slice(extra);
            bytes
        };
        assert_eq!(super::reserved_canonical_guess(&clean), Ok(false));
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x01])), Err(()));
        assert_eq!(decode_submit(&with(&[0x88, 0x01])), Err(REFUSAL_MALFORMED));
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x00])), Err(()));
        let zero = with(&[0x88, 0x01, 0x00]);
        assert_eq!(super::reserved_canonical_guess(&zero), Ok(false));
        let visible = decode_submit(&zero).unwrap();
        let DecodedBody::Ping { target: Some(target), .. } = &visible.body else {
            panic!("zero field 17 still decodes the filtered target");
        };
        assert_eq!(target.replica_id, 42);
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x01, 0x01])), Ok(true));
        assert_eq!(decode_submit(&with(&[0x88, 0x01, 0x01])), Err(REFUSAL_MALFORMED));
        let two_zeros = with(&[0x88, 0x01, 0x00, 0x88, 0x01, 0x00]);
        assert_eq!(super::reserved_canonical_guess(&two_zeros), Ok(false));
        assert_eq!(decode_submit(&two_zeros).unwrap().request_id, 1);
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x01, 0x00, 0x88, 0x01, 0x02])), Ok(true));
        assert_eq!(decode_submit(&with(&[0x88, 0x01, 0x00, 0x88, 0x01, 0x02])), Err(REFUSAL_MALFORMED));
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x01, 0x7F, 0x88, 0x01, 0x00])), Ok(true));
        assert_eq!(super::reserved_canonical_guess(&with(&[0x8A, 0x01, 0x01, 0xFF])), Ok(true));
        assert_eq!(decode_submit(&with(&[0x8A, 0x01, 0x01, 0xFF])), Err(REFUSAL_MALFORMED));
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x01, 0x00, 0x8A, 0x01, 0x01, 0xFF])), Ok(true));
        assert_eq!(super::reserved_canonical_guess(&with(&[0x88, 0x01, 0x00, 0x88, 0x01])), Err(()));
        assert_eq!(decode_submit(&with(&[0x88, 0x01, 0x00, 0x88, 0x01])), Err(REFUSAL_MALFORMED));
    }

    fn route_row<'a>(id: &'a str, life: CommunicationLifetime) -> CommunicationRouteSession<'a> {
        CommunicationRouteSession {
            session_id: id,
            player_id: life.player_id,
            team_id: life.team_id,
            view_epoch: life.view_epoch,
            room_id: life.room_id,
            run_id: life.run_id,
            generation: life.generation,
            negotiated: true,
            secure_joined: true,
            observer: false,
        }
    }

    #[test]
    fn communication_e756_one_event_scope_replacement_commits_nonce() {
        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let match_state = roster();
        let life = bind_live(&mut registry, "kcp_a", 1, 1);
        let mut template = submit_ping(1, 1, 1);
        template.origin_session = "kcp_a".into();
        template.lifetime = life;
        let first = admit_authenticated(&mut authority, &mut registry, &template, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&first.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let before = authority.ledger().admission_cursor();
        assert_eq!(before.next_sequence, 2);
        assert_eq!(before.room_id, Some(9));
        assert_eq!(before.run_id, Some(3));
        assert!(registry.request_floor(9, 3, 1) >= 1);
        assert_eq!(authority.ledger().history().len(), 1);

        let new_scope = CommunicationScope { room_id: 9, run_id: 4 };
        registry.publish_scope(Some(new_scope));
        authority.set_scope(Some(new_scope));
        let result = negotiate_hello(&bound(1, 1), Some(new_scope), &hello_for(1, 1));
        assert!(result.agreed);
        let rebound = registry.note_hello("kcp_a", &result, false, true).expect("new run hello");
        assert_eq!(rebound.run_id, 4);
        assert_ne!(rebound.scope_generation, life.scope_generation);
        let mut opened = template.clone();
        opened.lifetime = rebound;
        opened.bound.room_id = 9;
        opened.bound.run_id = 4;
        if let CommunicationRequestBody::Submit(submit) = &mut opened.body {
            submit.claimed.room_id = 9;
            submit.claimed.run_id = 4;
        }
        let admitted = admit_authenticated(&mut authority, &mut registry, &opened, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&admitted.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let after = authority.ledger().admission_cursor();
        assert_eq!(after.next_sequence, before.next_sequence);
        assert_eq!(after.run_id, Some(4));
        assert_ne!(before, after);
        assert_eq!(authority.ledger().history().len(), 1);
        assert!(registry.request_floor(9, 4, 1) >= 1);
        let repeated = admit_authenticated(&mut authority, &mut registry, &opened, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&repeated.receipt).unwrap().refusal, REFUSAL_DUPLICATE);
        assert_eq!(authority.ledger().admission_cursor(), after);
        assert_eq!(authority.ledger().history().len(), 1);

        registry.close_session("kcp_a");
        registry.open_session("kcp_a").expect("reopen");
        let hello_again = registry.note_hello("kcp_a", &result, false, true).expect("reconnect");
        let mut revived = opened.clone();
        revived.lifetime = hello_again;
        let blocked = admit_authenticated(&mut authority, &mut registry, &revived, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&blocked.receipt).unwrap().refusal, REFUSAL_INVALID_NONCE);
        assert_eq!(authority.ledger().history().len(), 1);
        // Retired streams are discarded; replay prevention belongs to the current run.
        assert_eq!(registry.request_floor(9, 3, 1), 0);
        assert!(registry.request_floor(9, 4, 1) >= 1);
        let mut next = with_nonce(&opened, 2);
        next.lifetime = hello_again;
        let continued = admit_authenticated(&mut authority, &mut registry, &next, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&continued.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        assert_eq!(authority.ledger().admission_cursor().next_sequence, 3);
        assert_eq!(authority.ledger().admission_cursor().run_id, Some(4));
    }

    #[test]
    fn communication_e756_queued_delivery_rechecks_binding_after_wait() {
        let mut registry = CommunicationSessionRegistry::default();
        let mut authority = authority_fixture();
        let match_state = roster();
        let life = bind_live(&mut registry, "kcp_a", 1, 1);
        let ally = bind_live(&mut registry, "kcp_b", 5, 1);
        let mut request = submit_ping(1, 1, 1);
        request.origin_session = "kcp_a".into();
        request.lifetime = life;
        let delivery = admit_authenticated(&mut authority, &mut registry, &request, Some(&match_state), None, None, 4, &[]);
        assert_eq!(decode_receipt(&delivery.receipt).unwrap().refusal, REFUSAL_ADMITTED);
        let rows = [route_row("kcp_a", life), route_row("kcp_b", ally)];
        let mut queued = delivery.clone();
        queued.replay = Some(vec![9, 9]);
        queued.consume_replay = true;
        let granted = authorize_queued_delivery(&registry, &queued, &rows);
        assert!(granted.deliver_receipt);
        assert!(granted.deliver_replay);
        assert!(granted.mark_replay_consumed);
        assert!(granted.event_sessions.iter().any(|id| id == "kcp_a"));
        assert!(granted.event_sessions.iter().any(|id| id == "kcp_b"));
        assert!(registry.set_replay_consumed("kcp_a", life));
        let consumed = authorize_queued_delivery(&registry, &queued, &rows);
        assert!(consumed.deliver_receipt);
        assert!(!consumed.deliver_replay);
        assert!(!consumed.mark_replay_consumed);

        let mut shifted = bound(1, 1);
        shifted.view_epoch = 8;
        let mut hello = hello_for(1, 1);
        hello.claimed_view_epoch = 8;
        let result = negotiate_hello(&shifted, Some(scope()), &hello);
        assert!(result.agreed);
        let rebound = registry.note_hello("kcp_a", &result, false, true).expect("rebind");
        assert_eq!(rebound.generation, life.generation);
        assert_ne!(rebound.binding_generation, life.binding_generation);
        assert_ne!(rebound.view_epoch, life.view_epoch);
        let rebound_rows = [route_row("kcp_a", life), route_row("kcp_b", ally)];
        let replaced = authorize_queued_delivery(&registry, &queued, &rebound_rows);
        assert!(!replaced.deliver_receipt);
        assert!(!replaced.deliver_replay);
        assert!(replaced.event_sessions.iter().all(|id| id != "kcp_a"));
        assert!(replaced.event_sessions.iter().any(|id| id == "kcp_b"));

        registry.publish_scope(None);
        let retired = authorize_queued_delivery(&registry, &queued, &rebound_rows);
        assert!(!retired.deliver_receipt);
        assert!(!retired.deliver_replay);
        assert!(retired.event_sessions.is_empty());
    }
}
