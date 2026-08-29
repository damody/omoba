//! Secure team-bound stream routing and exact-byte replay retention.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::transport::{MatchCapabilityNegotiation, MatchProtocol, NegotiationError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecureSessionBinding {
    pub protocol: MatchProtocol,
    pub authenticated_team_id: u32,
    pub current_view_epoch: u64,
    pub secure_match_capability: bool,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecureJoinError {
    Negotiation(NegotiationError),
    MissingSecureCapability,
    InvalidAuthenticatedTeam,
    RuntimeDowngrade,
}

impl SecureSessionBinding {
    pub fn negotiate(
        request: &MatchCapabilityNegotiation,
        established_protocol: Option<MatchProtocol>,
        authenticated_team_id: u32,
        view_epoch: u64,
        secure_capability: bool,
    ) -> Result<Self, SecureJoinError> {
        if !secure_capability {
            return Err(SecureJoinError::MissingSecureCapability);
        }
        if authenticated_team_id == 0 {
            return Err(SecureJoinError::InvalidAuthenticatedTeam);
        }
        let protocol = request
            .resolve(established_protocol)
            .map_err(SecureJoinError::Negotiation)?;
        if protocol != MatchProtocol::SelectiveV2 {
            return Err(SecureJoinError::RuntimeDowngrade);
        }
        Ok(Self {
            protocol,
            authenticated_team_id,
            current_view_epoch: view_epoch,
            secure_match_capability: true,
            active: true,
        })
    }

    pub fn reject_runtime_downgrade(
        &self,
        requested: MatchProtocol,
    ) -> Result<(), SecureJoinError> {
        if self.active && self.protocol == MatchProtocol::SelectiveV2 && requested != self.protocol
        {
            Err(SecureJoinError::RuntimeDowngrade)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct EncodedTeamFrame {
    pub team_id: u32,
    pub sequence: u64,
    pub replica_tick: u64,
    pub bytes: Arc<[u8]>,
}

#[derive(Clone, Debug)]
pub struct TeamReplayRing {
    capacity: usize,
    frames: VecDeque<EncodedTeamFrame>,
    replay_responses: BTreeMap<u64, ReplayLookup>,
}

#[derive(Clone, Debug)]
pub enum ReplayLookup {
    Exact(Vec<Arc<[u8]>>),
    FilteredRebaseRequired {
        oldest_retained_sequence: Option<u64>,
    },
}

impl TeamReplayRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            frames: VecDeque::new(),
            replay_responses: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, frame: EncodedTeamFrame) {
        if self
            .frames
            .back()
            .is_some_and(|existing| frame.sequence <= existing.sequence)
        {
            return;
        }
        self.frames.push_back(frame);
        while self.frames.len() > self.capacity {
            self.frames.pop_front();
        }
        self.replay_responses.clear();
    }

    pub fn lookup(&mut self, request_id: u64, from_sequence: u64) -> ReplayLookup {
        if let Some(cached) = self.replay_responses.get(&request_id) {
            return cached.clone();
        }
        let oldest = self.frames.front().map(|frame| frame.sequence);
        let response = if oldest.is_some_and(|oldest| from_sequence < oldest) {
            ReplayLookup::FilteredRebaseRequired {
                oldest_retained_sequence: oldest,
            }
        } else {
            ReplayLookup::Exact(
                self.frames
                    .iter()
                    .filter(|frame| frame.sequence >= from_sequence)
                    .map(|frame| Arc::clone(&frame.bytes))
                    .collect(),
            )
        };
        self.replay_responses.insert(request_id, response.clone());
        response
    }

    pub fn frames_after(&self, sequence: u64) -> Vec<Arc<[u8]>> {
        self.frames
            .iter()
            .filter(|frame| frame.sequence > sequence)
            .map(|frame| Arc::clone(&frame.bytes))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct SecureSessionRoute {
    pub session_id: String,
    pub binding: SecureSessionBinding,
    pub rebase_resume_sequence: Option<u64>,
}

#[derive(Default)]
pub struct TeamStreamRouter {
    sessions: BTreeMap<String, SecureSessionRoute>,
    rings: BTreeMap<u32, TeamReplayRing>,
    ring_capacity: usize,
}

impl TeamStreamRouter {
    pub fn new(ring_capacity: usize) -> Self {
        Self {
            sessions: BTreeMap::new(),
            rings: BTreeMap::new(),
            ring_capacity: ring_capacity.max(1),
        }
    }

    pub fn bind_session(&mut self, session_id: String, binding: SecureSessionBinding) {
        self.sessions.insert(
            session_id.clone(),
            SecureSessionRoute {
                session_id,
                binding,
                rebase_resume_sequence: None,
            },
        );
    }

    pub fn unbind_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// The encoded payload is retained before targets are returned, so socket
    /// enqueue and observer taps use the identical Arc bytes.
    pub fn route_frame(&mut self, frame: EncodedTeamFrame) -> Vec<String> {
        self.rings
            .entry(frame.team_id)
            .or_insert_with(|| TeamReplayRing::new(self.ring_capacity))
            .insert(frame.clone());
        self.sessions
            .values()
            .filter(|route| {
                route.binding.active
                    && route.binding.protocol == MatchProtocol::SelectiveV2
                    && route.binding.authenticated_team_id == frame.team_id
                    && route.rebase_resume_sequence.is_none()
            })
            .map(|route| route.session_id.clone())
            .collect()
    }

    pub fn session_ids_for_team(&self, team_id: u32) -> Vec<String> {
        self.sessions
            .values()
            .filter(|route| {
                route.binding.active
                    && route.binding.protocol == MatchProtocol::SelectiveV2
                    && route.binding.authenticated_team_id == team_id
            })
            .map(|route| route.session_id.clone())
            .collect()
    }

    pub fn replay(&mut self, team: u32, request_id: u64, from_sequence: u64) -> ReplayLookup {
        self.rings
            .entry(team)
            .or_insert_with(|| TeamReplayRing::new(self.ring_capacity))
            .lookup(request_id, from_sequence)
    }

    pub fn begin_filtered_rebase(&mut self, session_id: &str, resume_sequence: u64) {
        if let Some(route) = self.sessions.get_mut(session_id) {
            route.rebase_resume_sequence = Some(resume_sequence);
        }
    }

    pub fn complete_filtered_rebase(&mut self, session_id: &str) -> Vec<Arc<[u8]>> {
        let Some(route) = self.sessions.get_mut(session_id) else {
            return Vec::new();
        };
        let Some(sequence) = route.rebase_resume_sequence.take() else {
            return Vec::new();
        };
        self.rings
            .get(&route.binding.authenticated_team_id)
            .map(|ring| ring.frames_after(sequence.saturating_sub(1)))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod rebase_resume_tests {
    use super::*;

    #[test]
    fn completed_rebase_replays_the_resume_sequence_inclusively() {
        let mut router = TeamStreamRouter::new(8);
        router.bind_session(
            "team-1".into(),
            SecureSessionBinding {
                protocol: MatchProtocol::SelectiveV2,
                authenticated_team_id: 1,
                current_view_epoch: 1,
                secure_match_capability: true,
                active: true,
            },
        );
        for sequence in 10..=12 {
            router.route_frame(EncodedTeamFrame {
                team_id: 1,
                sequence,
                replica_tick: sequence,
                bytes: Arc::from(vec![sequence as u8].into_boxed_slice()),
            });
        }
        router.begin_filtered_rebase("team-1", 11);
        let routed_while_staging = router.route_frame(EncodedTeamFrame {
            team_id: 1,
            sequence: 13,
            replica_tick: 13,
            bytes: Arc::from(vec![13].into_boxed_slice()),
        });
        assert!(routed_while_staging.is_empty());
        let replay = router.complete_filtered_rebase("team-1");
        assert_eq!(
            replay.iter().map(|bytes| bytes[0]).collect::<Vec<_>>(),
            [11, 12, 13]
        );
    }

    #[test]
    fn same_team_players_share_frames_without_cross_team_delivery() {
        let mut router = TeamStreamRouter::new(8);
        let binding = |team_id| SecureSessionBinding {
            protocol: MatchProtocol::SelectiveV2,
            authenticated_team_id: team_id,
            current_view_epoch: 1,
            secure_match_capability: true,
            active: true,
        };
        router.bind_session("team-1-player-1".into(), binding(1));
        router.bind_session("team-1-player-3".into(), binding(1));
        router.bind_session("team-2-player-2".into(), binding(2));

        let targets = router.route_frame(EncodedTeamFrame {
            team_id: 1,
            sequence: 1,
            replica_tick: 10,
            bytes: Arc::from([1_u8, 2, 3]),
        });

        assert_eq!(targets, vec!["team-1-player-1", "team-1-player-3"]);
        assert!(!targets.iter().any(|id| id == "team-2-player-2"));
    }

    #[test]
    fn disconnecting_one_same_team_player_preserves_the_other_route() {
        let mut router = TeamStreamRouter::new(8);
        let binding = SecureSessionBinding {
            protocol: MatchProtocol::SelectiveV2,
            authenticated_team_id: 1,
            current_view_epoch: 1,
            secure_match_capability: true,
            active: true,
        };
        router.bind_session("player-1".into(), binding.clone());
        router.bind_session("player-3".into(), binding);
        router.unbind_session("player-1");

        assert_eq!(router.session_ids_for_team(1), vec!["player-3"]);
    }
}
