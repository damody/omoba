//! Local renderer correlation only. Authority admission remains in the core ledger.
use std::{collections::BTreeMap, time::{Duration, Instant}};
use omoba_core::{game_proto::{CommunicationInputResult, CommunicationIntent, CommunicationSubmit, RendererInput},
    runtime::communication_transport::{self as wire, CommunicationInbox, DecodedBody}};
use prost::Message;
use crate::replica_host::ReplicaHost;

pub const LOCAL_PENDING_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Scope { renderer: u64, source: u64, epoch: u64, view: u64 }

#[derive(Default)]
pub struct CommunicationInputGateway {
    scope: Option<Scope>,
    token_floor: u64,
    pending: BTreeMap<u64, (u64, Instant)>,
    last_result: Option<CommunicationInputResult>,
}

impl CommunicationInputGateway {
    pub fn synchronize(&mut self, inbox: &CommunicationInbox, renderer: u64) -> Vec<u64> {
        let next = inbox.binding().filter(|_| renderer != 0).map(|b| Scope {
            renderer, source: inbox.source_generation(), epoch: inbox.presentation_epoch(), view: b.view_epoch,
        });
        if next == self.scope { return Vec::new(); }
        let retired = self.pending.keys().copied().collect();
        self.pending.clear(); self.last_result = None; self.token_floor = 0; self.scope = next;
        retired
    }

    pub fn accepts_source(&self, input: &RendererInput, intent: &CommunicationIntent) -> bool {
        self.scope.is_some_and(|s| s.renderer == intent.renderer_connection_id && s.renderer != 0
            && s.source == intent.source_generation && s.epoch == intent.presentation_epoch
            && s.view == input.disclosure_epoch)
    }

    pub fn preflight(&self, input: &RendererInput, intent: &CommunicationIntent) -> Result<(), u32> {
        if !self.accepts_source(input, intent) { return Err(wire::REFUSAL_VIEW_MISMATCH); }
        if input.request_id == 0 || input.request_id <= self.token_floor { return Err(wire::REFUSAL_DUPLICATE); }
        if self.pending.len() >= wire::COMMUNICATION_PENDING_SUBMISSIONS { return Err(wire::REFUSAL_BACKPRESSURE); }
        Ok(())
    }

    pub fn track(&mut self, network: u64, token: u64, now: Instant) {
        let scope = self.scope.expect("preflight requires a current renderer scope");
        assert!(network != 0 && token > self.token_floor && self.pending.len() < wire::COMMUNICATION_PENDING_SUBMISSIONS);
        self.pending.insert(network, (token, now)); self.token_floor = token;
        self.last_result = Some(CommunicationInputResult { renderer_connection_id: scope.renderer,
            request_token: token, network_request_id: network, phase: 1, refusal: 0 });
    }

    pub fn settle(&mut self, network: u64, refusal: u32, local: bool) {
        let Some((token, _)) = self.pending.remove(&network) else { return };
        let Some(scope) = self.scope else { return };
        self.last_result = Some(CommunicationInputResult { renderer_connection_id: scope.renderer,
            request_token: token, network_request_id: network, phase: if local { 3 } else { 2 }, refusal });
    }

    pub fn refuse_locally(&mut self, token: u64, refusal: u32) {
        let Some(scope) = self.scope else { return };
        if token == 0 { return; }
        self.last_result = Some(CommunicationInputResult { renderer_connection_id: scope.renderer,
            request_token: token, network_request_id: 0, phase: 3, refusal });
    }

    pub fn due(&self, now: Instant) -> Vec<u64> {
        self.pending.iter().filter_map(|(id, (_, sent))|
            now.checked_duration_since(*sent).filter(|age| *age >= LOCAL_PENDING_TIMEOUT).map(|_| *id)).collect()
    }

    pub fn result(&self) -> Option<CommunicationInputResult> { self.last_result.clone() }
}

/// Uses the exact network decoder for strict field shape. Filtered target identity
/// comes from this ReplicaHost; canonical identity and generation remain authority-owned.
pub fn decode_intent(intent: &CommunicationIntent, inbox: &CommunicationInbox, replica: &ReplicaHost) -> Result<DecodedBody, u32> {
    let bound = inbox.binding().ok_or(wire::REFUSAL_NOT_NEGOTIATED)?;
    if intent.encoded_len() > wire::SUBMIT_MAX_BYTES { return Err(wire::REFUSAL_OVERSIZED); }
    let mut submit = CommunicationSubmit {
        schema_version: wire::COMMUNICATION_SCHEMA_VERSION, request_id: 1, nonce: 1,
        claimed_room_id: bound.room_id, claimed_run_id: bound.run_id, claimed_player_id: bound.player_id,
        claimed_team_id: bound.team_id, claimed_view_epoch: bound.view_epoch,
        kind: intent.kind, semantic: intent.semantic, point_x_raw: intent.point_x_raw,
        point_y_raw: intent.point_y_raw, has_point: intent.has_point,
        hud_kind: intent.hud_kind, hud_index: intent.hud_index, hud_player_id: intent.hud_player_id,
        chat_channel: intent.chat_channel, text: intent.text.clone(), objective_id: intent.objective_id.clone(),
        mute_sender_id: intent.mute_sender_id, muted: intent.muted, content_id: intent.content_id,
        ..Default::default()
    };
    if intent.target_render_id != 0 {
        let reference = replica.secure_reference(intent.target_render_id).ok_or(wire::REFUSAL_STALE_OR_HIDDEN_TARGET)?;
        let disclosed = reference.disclosure_epoch.ok_or(wire::REFUSAL_STALE_OR_HIDDEN_TARGET)?.value;
        let view = reference.view_epoch.ok_or(wire::REFUSAL_VIEW_MISMATCH)?.value;
        if intent.target_disclosure_epoch != disclosed || view != bound.view_epoch {
            return Err(wire::REFUSAL_STALE_OR_HIDDEN_TARGET);
        }
        submit.has_target = true; submit.replica_id = intent.target_render_id;
        submit.disclosure_epoch = disclosed; submit.target_view_epoch = view;
    } else if intent.target_disclosure_epoch != 0 { return Err(wire::REFUSAL_MALFORMED); }
    Ok(wire::decode_submit(&submit.encode_to_vec())?.body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_core::game_proto::CommunicationHelloResult;
    fn agreed() -> CommunicationInbox {
        let mut inbox = CommunicationInbox::default();
        inbox.apply_hello(&CommunicationHelloResult { schema_version: wire::COMMUNICATION_SCHEMA_VERSION,
            protocol_version: wire::COMMUNICATION_PROTOCOL_VERSION, agreed: true, room_id: 9, run_id: 3,
            player_id: 1, team_id: 1, view_epoch: 4, ..Default::default() }).unwrap();
        inbox
    }
    #[test]
    fn communication_gateway_retires_tokens_on_renderer_source_and_view_change() {
        let mut inbox = agreed(); let mut gateway = CommunicationInputGateway::default();
        assert!(gateway.synchronize(&inbox, 7).is_empty());
        let intent = CommunicationIntent { renderer_connection_id: 7, source_generation: inbox.source_generation(),
            presentation_epoch: inbox.presentation_epoch(), ..Default::default() };
        let input = RendererInput { request_id: 1, player_id: 1, disclosure_epoch: 4, ..Default::default() };
        gateway.preflight(&input, &intent).unwrap(); gateway.track(10, 1, Instant::now());
        assert_eq!(gateway.preflight(&input, &intent), Err(wire::REFUSAL_DUPLICATE));
        assert_eq!(gateway.synchronize(&inbox, 8), vec![10]);
        assert!(!gateway.accepts_source(&input, &intent)); assert!(gateway.result().is_none());
        inbox.observe_view(5); gateway.synchronize(&inbox, 8);
        let changed = CommunicationIntent { renderer_connection_id: 8, ..intent };
        assert!(!gateway.accepts_source(&input, &changed));
    }
    #[test]
    fn communication_gateway_timeout_is_local_uncertainty_and_releases_capacity() {
        let inbox = agreed(); let mut gateway = CommunicationInputGateway::default(); gateway.synchronize(&inbox, 7);
        let start = Instant::now(); gateway.track(11, 1, start);
        assert!(gateway.due(start + LOCAL_PENDING_TIMEOUT - Duration::from_millis(1)).is_empty());
        assert_eq!(gateway.due(start + LOCAL_PENDING_TIMEOUT), vec![11]);
        gateway.settle(11, wire::REFUSAL_LOCAL_TRANSPORT, true);
        let result = gateway.result().unwrap(); assert_eq!((result.phase, result.refusal, result.request_token), (3, 40, 1));
        gateway.settle(11, 0, false); assert_eq!(gateway.result().unwrap(), result);
        assert!(gateway.pending.is_empty());
    }
}
