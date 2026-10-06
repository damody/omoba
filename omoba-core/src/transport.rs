use serde::{Deserialize, Serialize};
use serde_json::json;

/// 共用的 selective 線路版本；V2 名稱仍表示模式，不表示目前線路版本。
pub const SELECTIVE_LOCKSTEP_PROTOCOL_VERSION: u32 = 6;
pub const SELECTIVE_LOCKSTEP_PROTOCOL_V2: u32 = SELECTIVE_LOCKSTEP_PROTOCOL_VERSION;

/// Only these protobuf message types may cross the secure V2 player boundary.
/// Legacy `GameStart`, `SnapshotResp`, `TickBatch`, and `StateHash` are
/// intentionally absent because they expose global state or global cadence.
pub const V2_PLAYER_MESSAGE_ALLOWLIST: &[&str] = &[
    "TeamGameStart",
    "TeamTickFrame",
    "TeamViewRebaseChunk",
    "TeamViewRebase",
];

/// Defense-in-depth field-name denylist applied by the V2 encoder/schema audit.
pub const V2_PLAYER_FORBIDDEN_FIELDS: &[&str] = &[
    "master_seed",
    "canonical_entity_id",
    "raw_ecs_id",
    "specs_entity_id",
];

pub fn is_v2_player_message_allowed(message_name: &str) -> bool {
    V2_PLAYER_MESSAGE_ALLOWLIST.contains(&message_name)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchProtocol {
    LegacyV1,
    SelectiveV2,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchCapabilityNegotiation {
    pub requested_protocol: u32,
    pub supported_protocols: Vec<u32>,
    pub secure_fog_required: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NegotiationError {
    V2Required,
    UnsupportedProtocol,
    MixedMatchProtocol,
}

impl MatchCapabilityNegotiation {
    pub fn resolve(
        &self,
        established_match_protocol: Option<MatchProtocol>,
    ) -> Result<MatchProtocol, NegotiationError> {
        let supports_requested = self.supported_protocols.contains(&self.requested_protocol);
        if !supports_requested {
            return Err(NegotiationError::UnsupportedProtocol);
        }
        let selected = match self.requested_protocol {
            1 if !self.secure_fog_required => MatchProtocol::LegacyV1,
            SELECTIVE_LOCKSTEP_PROTOCOL_VERSION => MatchProtocol::SelectiveV2,
            1 => return Err(NegotiationError::V2Required),
            _ => return Err(NegotiationError::UnsupportedProtocol),
        };
        if established_match_protocol.is_some_and(|protocol| protocol != selected) {
            return Err(NegotiationError::MixedMatchProtocol);
        }
        Ok(selected)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OutboundMsg {
    pub topic: String,
    pub msg: String,
    #[serde(skip)]
    pub entity_pos: Option<(f32, f32)>,
}

#[cfg(test)]
mod selective_version_tests {
    use super::*;

    #[test]
    fn attack_visual_state_protocol_rejects_old_selective_version() {
        let mut request = MatchCapabilityNegotiation {
            requested_protocol: SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            supported_protocols: vec![SELECTIVE_LOCKSTEP_PROTOCOL_VERSION],
            secure_fog_required: true,
        };
        assert_eq!(request.resolve(None), Ok(MatchProtocol::SelectiveV2));
        request.requested_protocol = SELECTIVE_LOCKSTEP_PROTOCOL_VERSION - 1;
        request.supported_protocols = vec![SELECTIVE_LOCKSTEP_PROTOCOL_VERSION - 1];
        assert_eq!(request.resolve(None), Err(NegotiationError::UnsupportedProtocol));
        request.requested_protocol = SELECTIVE_LOCKSTEP_PROTOCOL_VERSION;
        assert_eq!(request.resolve(None), Err(NegotiationError::UnsupportedProtocol));
        request.supported_protocols = vec![SELECTIVE_LOCKSTEP_PROTOCOL_VERSION];
        assert_eq!(request.resolve(Some(MatchProtocol::LegacyV1)), Err(NegotiationError::MixedMatchProtocol));
    }
}

impl OutboundMsg {
    pub fn new_s(topic: &str, t: &str, a: &str, v: serde_json::Value) -> Self {
        Self {
            topic: topic.to_string(),
            msg: json!({ "t": t, "a": a, "d": v }).to_string(),
            entity_pos: None,
        }
    }

    pub fn new_s_all(topic: &str, t: &str, a: &str, v: serde_json::Value) -> Self {
        Self::new_s(topic, t, a, v)
    }

    pub fn new_s_at(topic: &str, t: &str, a: &str, v: serde_json::Value, x: f32, y: f32) -> Self {
        let mut msg = Self::new_s(topic, t, a, v);
        msg.entity_pos = Some((x, y));
        msg
    }
}
