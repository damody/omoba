//! Negotiated undo wire. Receipt v1 stays 40 bytes and result codes 0..=13.
//! Undo availability is an owner-team fact, not a change to the 95-byte economy codec.
use crate::game_proto::{player_input::Action, DeterministicMetadata, PlayerInput, TeamPublicEvent};
use crate::runtime::shop::{ShopError, ShopUndoLedger, ShopUndoRequest, ShopUndoSettlement};
use crate::runtime::{CanonicalAcceptedInput, FactKind, FactPhase, ProjectionError};
use std::hash::{Hash, Hasher};

/// Authority `PlayerInput` / `CanonicalAcceptedInput` action. Not item-use (8),
/// buy (17), sell (18), or the KCP ping tag (23). Renderer intent 25 is reserved
/// and is not added on this increment.
pub const SHOP_UNDO_ACTION_KIND: u32 = 24;
pub const SHOP_UNDO_MODE_NEWEST: u32 = 1;
pub const SHOP_UNDO_MODE_SEQUENCE: u32 = 2;
/// Server economy domain mixed only under selective protocol 7's hash contract.
/// It is not the team SHA-256 and it is not implied by `MobaMatch` alone.
pub const LOCKSTEP_ECONOMY_V2_DOMAIN: &[u8] = b"omoba-lockstep-economy-v2\0";
/// Team checkpoint domain for selective protocol 7. Protocol 6 used v1 and is rejected.
pub const CANONICAL_TEAM_VIEW_DOMAIN: &[u8] = b"omoba-canonical-team-view-v2\0";

/// Installed by moba match setup. Version must equal the selective protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockstepHashContract {
    pub version: u32,
}

/// Undo may mutate, and the server hash may include the ledger, only when
/// protocol 7's contract is installed and a moba match is present.
pub fn undo_hash_contract_active(world: &specs::World) -> bool {
    world.try_fetch::<LockstepHashContract>().is_some_and(|contract| {
        contract.version == crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION
    }) && world.try_fetch::<crate::runtime::MobaMatch>().is_some()
}

pub fn mix_lockstep_economy_v2(base: u64, material: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    LOCKSTEP_ECONOMY_V2_DOMAIN.hash(&mut hasher);
    base.hash(&mut hasher);
    material.hash(&mut hasher);
    hasher.finish()
}

/// Rewrite Newest to the owner's actual newest sequence before execution.
/// The journal keeps the original bytes, so a retry of the same input id cannot
/// be pinned again onto a purchase that happens later. A malformed shape becomes
/// sequence 0 and settles as unavailable. Execution must not read Newest.
pub fn pin_shop_undo_request(world: &specs::World, player_id: u32, input: &mut PlayerInput) {
    let Some(Action::ShopUndo(undo)) = input.action.as_mut() else { return };
    let sequence = match (undo.mode, undo.sequence) {
        (SHOP_UNDO_MODE_NEWEST, 0) => world
            .try_fetch::<ShopUndoLedger>()
            .and_then(|ledger| ledger.owners.get(&player_id).and_then(|owner| owner.records.last().map(|record| record.sequence)))
            .unwrap_or(0),
        (SHOP_UNDO_MODE_SEQUENCE, sequence) if sequence != 0 => sequence,
        _ => 0,
    };
    undo.mode = SHOP_UNDO_MODE_SEQUENCE;
    undo.sequence = sequence;
}

pub fn pinned_undo_request(input: &PlayerInput) -> Option<ShopUndoRequest> {
    match input.action.as_ref()? {
        Action::ShopUndo(undo) if undo.mode == SHOP_UNDO_MODE_SEQUENCE => Some(ShopUndoRequest::Sequence(undo.sequence)),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopUndoReceipt {
    pub player_id: u32,
    pub input_id: u64,
    pub tick: u64,
    pub sequence: u64,
    pub result_code: u32,
    pub gold: i32,
    pub catalogs: [u32; 6],
    pub cooldowns: [u32; 6],
}

impl ShopUndoReceipt {
    pub const VERSION: u32 = 2;
    pub const PAYLOAD_LEN: usize = 92;
    pub fn valid(&self) -> bool {
        let sequence_ok = match self.result_code {
            0 | 15 => self.sequence != 0,
            code if code <= ShopError::UndoDuplicate.receipt_code() => true,
            _ => false,
        };
        self.player_id != 0 && self.input_id != 0 && sequence_ok && undo_wallet_valid(self.gold, self.catalogs, self.cooldowns)
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::PAYLOAD_LEN);
        bytes.extend(Self::VERSION.to_le_bytes());
        bytes.extend(self.player_id.to_le_bytes());
        bytes.extend(self.input_id.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(SHOP_UNDO_ACTION_KIND.to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.result_code.to_le_bytes());
        bytes.extend(self.gold.to_le_bytes());
        for value in self.catalogs { bytes.extend(value.to_le_bytes()); }
        for value in self.cooldowns { bytes.extend(value.to_le_bytes()); }
        debug_assert_eq!(bytes.len(), Self::PAYLOAD_LEN);
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        decode_undo_receipt(bytes)
    }
}

fn decode_undo_receipt(bytes: &[u8]) -> Option<ShopUndoReceipt> {
    if bytes.len() != ShopUndoReceipt::PAYLOAD_LEN || u32::from_le_bytes(bytes[0..4].try_into().ok()?) != ShopUndoReceipt::VERSION {
        return None;
    }
    if u32::from_le_bytes(bytes[24..28].try_into().ok()?) != SHOP_UNDO_ACTION_KIND {
        return None;
    }
    let mut catalogs = [0; 6];
    let mut cooldowns = [0; 6];
    for (index, slot) in catalogs.iter_mut().enumerate() {
        let at = 44 + index * 4;
        *slot = u32::from_le_bytes(bytes[at..at + 4].try_into().ok()?);
    }
    for (index, slot) in cooldowns.iter_mut().enumerate() {
        let at = 68 + index * 4;
        *slot = u32::from_le_bytes(bytes[at..at + 4].try_into().ok()?);
    }
    let receipt = ShopUndoReceipt {
        player_id: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
        input_id: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        tick: u64::from_le_bytes(bytes[16..24].try_into().ok()?),
        sequence: u64::from_le_bytes(bytes[28..36].try_into().ok()?),
        result_code: u32::from_le_bytes(bytes[36..40].try_into().ok()?),
        gold: i32::from_le_bytes(bytes[40..44].try_into().ok()?),
        catalogs,
        cooldowns,
    };
    receipt.valid().then_some(receipt)
}

fn undo_wallet_valid(gold: i32, catalogs: [u32; 6], cooldowns: [u32; 6]) -> bool {
    use crate::runtime::native::economy_projection::CommittedEconomyState;
    let mut item_ids = [0u16; 6];
    for (index, catalog) in catalogs.iter().copied().enumerate() {
        let Ok(id) = u16::try_from(catalog) else { return false };
        item_ids[index] = id;
    }
    CommittedEconomyState {
        gold,
        item_ids,
        cooldown_bits: cooldowns,
        effect_bits: [0; 10],
        dirty: false,
    }
    .components()
    .is_ok()
}

impl ShopUndoReceipt {
    pub fn from_settlement(player_id: u32, input_id: u64, tick: u64, settlement: &ShopUndoSettlement) -> Self {
        let sequence = match settlement.request {
            ShopUndoRequest::Sequence(sequence) => sequence,
            ShopUndoRequest::Newest => 0,
        };
        Self {
            player_id,
            input_id,
            tick,
            sequence,
            result_code: settlement.result.as_ref().err().map_or(0, ShopError::receipt_code),
            gold: settlement.gold,
            catalogs: settlement.catalogs,
            cooldowns: settlement.cooldowns,
        }
    }
}

/// Per-player undo FIFO. Buy/sell settlements are not visible here.
pub fn project_shop_undo_receipts(
    team: u32,
    tick: u64,
    accepted: &[CanonicalAcceptedInput],
    settlements: &[ShopUndoSettlement],
) -> Result<Vec<TeamPublicEvent>, ProjectionError> {
    use prost::Message;
    use std::collections::{BTreeMap, VecDeque};
    let mut queues: BTreeMap<u32, VecDeque<&ShopUndoSettlement>> = BTreeMap::new();
    for result in settlements {
        queues.entry(result.player_id).or_default().push_back(result);
    }
    let mut events = Vec::new();
    for input in accepted.iter().filter(|input| input.team_id == team && input.action_kind == SHOP_UNDO_ACTION_KIND) {
        let Some(result) = queues.get_mut(&input.player_id).and_then(VecDeque::pop_front) else { continue };
        let decoded = PlayerInput::decode(input.sanitized_payload.as_slice()).map_err(|_| ProjectionError::MalformedDisclosedState)?;
        let Some(request) = pinned_undo_request(&decoded) else {
            return Err(ProjectionError::MalformedDisclosedState);
        };
        if request != result.request || matches!(result.request, ShopUndoRequest::Newest) {
            return Err(ProjectionError::MalformedDisclosedState);
        }
        if input.input_id == 0 {
            continue;
        }
        let receipt = ShopUndoReceipt::from_settlement(input.player_id, input.input_id, tick, result);
        if !receipt.valid() || decode_undo_receipt(&receipt.encode()).as_ref() != Some(&receipt) {
            return Err(ProjectionError::MalformedDisclosedState);
        }
        events.push(TeamPublicEvent {
            event_kind: FactKind::ShopUndoReceipt as u32,
            stable_sub_index: events.len() as u32,
            sanitized_payload: receipt.encode(),
            ..Default::default()
        });
    }
    Ok(events)
}

/// Schema 2 only. A version-1 40-byte receipt, including codes 14 and 15, is rejected.
pub fn validate_undo_receipt_replay(
    msg: &crate::game_proto::ShopReceiptReplay,
    player: u32,
) -> Result<Option<ShopUndoReceipt>, &'static str> {
    if msg.schema_version != 2 || player == 0 || msg.player_id != player || msg.request_id == 0 || msg.input_id == 0 {
        return Err("INVALID_SHOP_UNDO_REPLAY_IDENTITY");
    }
    if msg.status > 4 {
        return Err("INVALID_SHOP_UNDO_REPLAY_STATUS");
    }
    if msg.status != 2 {
        return if msg.receipt.is_empty() { Ok(None) } else { Err("NONTERMINAL_SHOP_UNDO_REPLAY_PAYLOAD") };
    }
    let receipt = decode_undo_receipt(&msg.receipt).ok_or("INVALID_SHOP_UNDO_REPLAY_RECEIPT")?;
    if receipt.player_id != player || receipt.input_id != u64::from(msg.input_id) {
        return Err("CONFLICTING_SHOP_UNDO_REPLAY_RECEIPT");
    }
    Ok(Some(receipt))
}

/// Fixed owner-team scalar. 48 bytes: version, phase, player, can-undo, reason,
/// newest sequence, and a 32-byte ledger commitment. Not the purchase list.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct OwnerShopUndoProjection {
    pub phase: u8,
    pub player_id: u32,
    pub can_undo: bool,
    pub reason: u8,
    pub newest_sequence: u64,
    pub commitment: [u8; 32],
}

impl OwnerShopUndoProjection {
    pub const VERSION: u8 = 2;
    pub const LEN: usize = 48;
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::LEN);
        bytes.push(Self::VERSION);
        bytes.push(self.phase);
        bytes.extend(self.player_id.to_le_bytes());
        bytes.push(u8::from(self.can_undo));
        bytes.push(self.reason);
        bytes.extend(self.newest_sequence.to_le_bytes());
        bytes.extend(self.commitment);
        debug_assert_eq!(bytes.len(), Self::LEN);
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::LEN || bytes[0] != Self::VERSION || bytes[1] > 1 {
            return None;
        }
        let player_id = u32::from_le_bytes(bytes[2..6].try_into().ok()?);
        if player_id == 0 { return None; }
        let can_undo = match bytes[6] { 0 => false, 1 => true, _ => return None };
        let reason = bytes[7];
        if reason > crate::runtime::shop::UNDO_REASON_MAX { return None; }
        if can_undo != (reason == crate::runtime::shop::UNDO_REASON_CAN_UNDO) { return None; }
        let newest_sequence = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        if can_undo && newest_sequence == 0 { return None; }
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&bytes[16..48]);
        Some(Self { phase: bytes[1], player_id, can_undo, reason, newest_sequence, commitment })
    }
}

pub fn owner_shop_undo_projection(
    world: &specs::World,
    state: &crate::runtime::MobaMatch,
    player_id: u32,
    phase: u8,
) -> OwnerShopUndoProjection {
    let admission = crate::runtime::shop::inspect_owner_undo(world, state, player_id);
    OwnerShopUndoProjection {
        phase,
        player_id,
        can_undo: admission.can_undo,
        reason: admission.reason,
        newest_sequence: admission.newest_sequence,
        commitment: admission.commitment,
    }
}

pub fn owner_undo_hash_material(events: &[TeamPublicEvent]) -> Vec<u8> {
    let mut material = Vec::new();
    for event in events.iter().filter(|event| event.event_kind == FactKind::OwnerShopUndo as u32 && event.subject.is_none()) {
        material.extend(event.stable_sub_index.to_be_bytes());
        material.extend((event.sanitized_payload.len() as u64).to_be_bytes());
        material.extend(&event.sanitized_payload);
    }
    material
}

pub fn phase_byte(phase: FactPhase) -> u8 {
    match phase {
        FactPhase::PreStep => 0,
        FactPhase::PostStep => 1,
        FactPhase::Step => 2,
    }
}

/// Length-prefixed owner-undo fact bytes. Empty when the frame has no such fact,
/// which leaves a legacy team digest unchanged.
pub fn mix_team_undo_material(hasher: &mut sha2::Sha256, material: &[u8]) {
    use sha2::Digest;
    if material.is_empty() {
        return;
    }
    hasher.update(LOCKSTEP_ECONOMY_V2_DOMAIN);
    hasher.update((material.len() as u64).to_be_bytes());
    hasher.update(material);
}

/// Team-private bootstrap slot for the last step's owner-undo hash material.
/// Schema 1 is the canonical record stream produced by [`owner_undo_hash_material`].
/// Absence is the empty/retired/no-fact state. A present entry must be schema 1.
pub const OWNER_UNDO_RETAINED_NAMESPACE: &str = "owner-shop-undo";
pub const OWNER_UNDO_RETAINED_KEY: &str = "retained-step";
pub const OWNER_UNDO_RETAINED_SCHEMA: u32 = 1;
/// One pre-step and one post-step fact per owner, with spare for a full roster.
const OWNER_UNDO_RETAINED_MAX_RECORDS: usize = 64;

fn retained_record_len() -> usize {
    4 + 8 + OwnerShopUndoProjection::LEN
}

/// Accept only the exact bytes [`owner_undo_hash_material`] emits.
/// Empty is the no-fact state. A non-empty stream must be strictly increasing
/// sub-indexes, fixed 48-byte canonical projections, and unique owner phases.
pub fn canonical_retained_owner_undo_material(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.is_empty() {
        return Some(Vec::new());
    }
    let max_len = OWNER_UNDO_RETAINED_MAX_RECORDS.checked_mul(retained_record_len())?;
    if bytes.len() > max_len {
        return None;
    }
    let mut cursor = 0;
    let mut last_index: Option<u32> = None;
    let mut owners = std::collections::BTreeSet::new();
    let mut count = 0usize;
    while cursor < bytes.len() {
        if count == OWNER_UNDO_RETAINED_MAX_RECORDS {
            return None;
        }
        let index = u32::from_be_bytes(bytes.get(cursor..cursor + 4)?.try_into().ok()?);
        cursor += 4;
        if last_index.is_some_and(|previous| index <= previous) {
            return None;
        }
        last_index = Some(index);
        let len = u64::from_be_bytes(bytes.get(cursor..cursor + 8)?.try_into().ok()?);
        cursor += 8;
        if len != OwnerShopUndoProjection::LEN as u64 {
            return None;
        }
        let payload = bytes.get(cursor..cursor + OwnerShopUndoProjection::LEN)?;
        cursor += OwnerShopUndoProjection::LEN;
        let projection = OwnerShopUndoProjection::decode(payload)?;
        if projection.encode().as_slice() != payload {
            return None;
        }
        if !owners.insert((projection.player_id, projection.phase)) {
            return None;
        }
        count += 1;
    }
    if cursor != bytes.len() || count == 0 {
        return None;
    }
    Some(bytes.to_vec())
}

pub fn retained_owner_undo_metadata(material: &[u8]) -> Option<DeterministicMetadata> {
    let canonical = canonical_retained_owner_undo_material(material)?;
    if canonical.is_empty() {
        return None;
    }
    Some(DeterministicMetadata {
        namespace: OWNER_UNDO_RETAINED_NAMESPACE.into(),
        key: OWNER_UNDO_RETAINED_KEY.into(),
        schema_version: OWNER_UNDO_RETAINED_SCHEMA,
        value: canonical,
    })
}

/// `Ok` empty when the slot is absent. `None` when the slot is malformed,
/// truncated, duplicated, empty-but-present, or not schema 1.
pub fn retained_owner_undo_from_metadata(entries: &[DeterministicMetadata]) -> Option<Vec<u8>> {
    let mut found = None;
    for entry in entries {
        if entry.namespace != OWNER_UNDO_RETAINED_NAMESPACE || entry.key != OWNER_UNDO_RETAINED_KEY {
            continue;
        }
        if found.is_some() || entry.schema_version != OWNER_UNDO_RETAINED_SCHEMA {
            return None;
        }
        let material = canonical_retained_owner_undo_material(&entry.value)?;
        if material.is_empty() {
            return None;
        }
        found = Some(material);
    }
    Some(found.unwrap_or_default())
}

pub fn metadata_declares_owner_undo(entries: &[DeterministicMetadata]) -> bool {
    entries.iter().any(|entry| {
        entry.namespace == OWNER_UNDO_RETAINED_NAMESPACE && entry.key == OWNER_UNDO_RETAINED_KEY
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_proto::{ShopReceiptReplay, ShopUndo as ShopUndoMsg};
    use crate::runtime::shop::{ShopCommand, ShopError, ShopSettlement};
    use crate::runtime::shop_receipt::{project_shop_receipts, ShopReceipt};
    use crate::runtime::shop_transport::{negotiate_shop_protocol, negotiate_shop_undo_protocol, validate_receipt_replay, SHOP_PROTOCOL_VERSION, SHOP_UNDO_PROTOCOL_VERSION};
    use prost::Message;

    fn undo_input(team: u32, player: u32, correlation: u64, sequence: u64) -> CanonicalAcceptedInput {
        CanonicalAcceptedInput::from_authoritative_acceptance(
            team, player, correlation, SHOP_UNDO_ACTION_KIND, 0, None,
            PlayerInput { action: Some(Action::ShopUndo(ShopUndoMsg { mode: SHOP_UNDO_MODE_SEQUENCE, sequence })) }.encode_to_vec(),
        )
    }
    fn settlement(player: u32, sequence: u64, result: Result<u64, ShopError>, gold: i32) -> ShopUndoSettlement {
        ShopUndoSettlement { player_id: player, request: ShopUndoRequest::Sequence(sequence), result, gold, catalogs: [1, 0, 0, 0, 0, 0], cooldowns: [0; 6] }
    }

    #[test]
    fn shop_undo_wire_v1_rejects_unknown_codes_and_v2_layout() {
        let mut legacy = ShopReceipt { player_id: 7, input_id: 4, tick: 3, action_kind: 17, catalog_id: 1, slot: 0, result_code: 0 };
        assert!(ShopReceipt::decode(&legacy.encode()).is_some());
        legacy.result_code = 14;
        assert!(ShopReceipt::decode(&legacy.encode()).is_none());
        legacy.result_code = 15;
        assert!(ShopReceipt::decode(&legacy.encode()).is_none());
        let undo = settlement(7, 1, Err(ShopError::UndoUnavailable), 400);
        let receipt = ShopUndoReceipt::from_settlement(7, 4, 3, &undo);
        assert_ne!(receipt.encode().len(), ShopReceipt::PAYLOAD_LEN);
        assert!(ShopReceipt::decode(&receipt.encode()).is_none());
        assert_eq!(decode_undo_receipt(&receipt.encode()).unwrap().result_code, 14);
        let mut reply = ShopReceiptReplay { schema_version: 1, request_id: 1, player_id: 7, input_id: 4, status: 2, receipt: receipt.encode() };
        assert!(validate_receipt_replay(&reply, 7).is_err());
        reply.schema_version = 2;
        reply.receipt = receipt.encode();
        assert!(validate_receipt_replay(&reply, 7).is_err());
        assert_eq!(validate_undo_receipt_replay(&reply, 7).unwrap().unwrap().result_code, 14);
        reply.schema_version = 1;
        assert!(validate_undo_receipt_replay(&reply, 7).is_err());
        reply.schema_version = 2;
        reply.receipt = legacy.encode();
        assert!(validate_undo_receipt_replay(&reply, 7).is_err());
        assert_eq!(SHOP_PROTOCOL_VERSION, 1);
        assert_eq!(SHOP_UNDO_PROTOCOL_VERSION, 2);
        assert_eq!(negotiate_shop_protocol(1, omoba_template_ids::CONTENT_CATALOG_DATA_HASH, true), Ok(true));
    }

    #[test]
    fn shop_undo_wire_buy_sell_fifo_ignores_undo_and_duplicate_keeps_economy() {
        let buy = CanonicalAcceptedInput::from_authoritative_acceptance(
            1, 7, 9, 17, 0, None,
            PlayerInput { action: Some(Action::ItemBuy(crate::game_proto::ItemBuy { item_id: "moba_sword".into() })) }.encode_to_vec(),
        );
        let accepted = [undo_input(1, 7, 8, 1), buy];
        let buy_result = ShopSettlement { player_id: 7, command: ShopCommand::Buy("moba_sword".into()), result: Ok(()) };
        let events = project_shop_receipts(1, 5, &accepted, &[buy_result]).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(ShopReceipt::decode(&events[0].sanitized_payload).unwrap().input_id, 9);
        let undo_events = project_shop_undo_receipts(1, 5, &accepted, &[settlement(7, 1, Ok(1), 650)]).unwrap();
        assert_eq!(decode_undo_receipt(&undo_events[0].sanitized_payload).unwrap().gold, 650);
        let again = project_shop_undo_receipts(1, 5, &accepted, &[settlement(7, 1, Err(ShopError::UndoDuplicate), 650)]).unwrap();
        let duplicate = decode_undo_receipt(&again[0].sanitized_payload).unwrap();
        assert_eq!(duplicate.result_code, 15);
        assert_eq!(duplicate.gold, 650);
        assert!(project_shop_undo_receipts(1, 5, &accepted, &[settlement(7, 2, Ok(2), 100)]).is_err());
    }

    #[test]
    fn shop_undo_protocol_projection_roundtrip_rejects_disagreement() {
        let projection = OwnerShopUndoProjection {
            phase: 1, player_id: 7, can_undo: true, reason: 0, newest_sequence: 3, commitment: [9; 32],
        };
        assert_eq!(projection.encode().len(), OwnerShopUndoProjection::LEN);
        assert_eq!(OwnerShopUndoProjection::decode(&projection.encode()).unwrap(), projection);
        let mut bad = projection.clone();
        bad.can_undo = false;
        assert!(OwnerShopUndoProjection::decode(&bad.encode()).is_none());
        bad.can_undo = true;
        bad.reason = crate::runtime::shop::UNDO_REASON_MAX + 1;
        assert!(OwnerShopUndoProjection::decode(&bad.encode()).is_none());
        bad.reason = 0;
        bad.newest_sequence = 0;
        assert!(OwnerShopUndoProjection::decode(&bad.encode()).is_none());
        assert!(owner_undo_hash_material(&[]).is_empty());
    }

    #[test]
    fn shop_undo_protocol_receipt_rejects_malformed_wallet() {
        let mut receipt = ShopUndoReceipt {
            player_id: 7, input_id: 4, tick: 3, sequence: 1, result_code: 0, gold: 400,
            catalogs: [1, 0, 0, 0, 0, 0], cooldowns: [0; 6],
        };
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_some());
        receipt.gold = -1;
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.gold = 400;
        receipt.catalogs[1] = u32::MAX;
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.catalogs[1] = 0;
        receipt.cooldowns[0] = f32::NAN.to_bits();
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.cooldowns[0] = (-1.0f32).to_bits();
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.cooldowns[0] = 86400.5f32.to_bits();
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.cooldowns[0] = 0;
        receipt.cooldowns[2] = 1.0f32.to_bits();
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.cooldowns[2] = 0;
        receipt.sequence = 0;
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.result_code = 15;
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_none());
        receipt.sequence = 2;
        assert_eq!(ShopUndoReceipt::decode(&receipt.encode()).unwrap().result_code, 15);
        receipt.result_code = 14;
        receipt.sequence = 0;
        assert!(ShopUndoReceipt::decode(&receipt.encode()).is_some());
        let legacy = ShopReceipt { player_id: 7, input_id: 4, tick: 3, action_kind: 17, catalog_id: 1, slot: 0, result_code: 0 };
        assert!(ShopReceipt::decode(&legacy.encode()).is_some());
        assert!(ShopUndoReceipt::decode(&legacy.encode()).is_none());
    }

    #[test]
    fn shop_undo_protocol_owner_projection_is_team_private_and_old_versions_fail_closed() {
        use sha2::Digest;
        use crate::runtime::{
            FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact,
            ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector,
        };
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_shop_undo_protocol(1, hash, true), Ok(false));
        assert_eq!(negotiate_shop_undo_protocol(2, hash, true), Ok(true));
        assert_eq!(negotiate_shop_protocol(2, hash, true), Ok(true));
        assert!(negotiate_shop_undo_protocol(3, hash, true).is_err());
        assert!(negotiate_shop_protocol(3, hash, true).is_err());
        assert_eq!(negotiate_shop_undo_protocol(0, "", false), Ok(false));
        let projection = OwnerShopUndoProjection {
            phase: 0, player_id: 7, can_undo: false, reason: crate::runtime::shop::UNDO_REASON_NO_RECORD,
            newest_sequence: 0, commitment: [0; 32],
        };
        let fact = OrderedFact {
            key: FactOrderingKey {
                tick: 1, phase: FactPhase::PreStep, canonical_source_order: 0, local_ordinal: 0,
                fact_kind: FactKind::OwnerShopUndo,
            },
            audience: FactAudience::Team(1),
            fact: ObservableFact::OwnerShopUndo { team: 1, state: projection },
        };
        let own = TeamViewProjector::new(1, TeamProjectorConfig::default())
            .build_frame(1, 1, &Default::default(), vec![], &[fact.clone()], &ProjectionDependencyGraph::default())
            .unwrap();
        let enemy = TeamViewProjector::new(2, TeamProjectorConfig::default())
            .build_frame(1, 1, &Default::default(), vec![], &[fact], &ProjectionDependencyGraph::default())
            .unwrap();
        let own_events = own.frame.step.unwrap().public_events;
        assert!(own_events.iter().any(|event| event.event_kind == FactKind::OwnerShopUndo as u32));
        assert!(enemy.frame.step.unwrap().public_events.iter().all(|event| event.event_kind != FactKind::OwnerShopUndo as u32));
        let mut untouched = sha2::Sha256::new();
        let mut mixed = sha2::Sha256::new();
        untouched.update(b"prefix");
        mixed.update(b"prefix");
        mix_team_undo_material(&mut mixed, &[]);
        assert_eq!(untouched.finalize().as_slice(), mixed.finalize().as_slice());
    }

    #[test]
    fn retained_owner_undo_metadata_rejects_truncated_duplicate_and_noncanonical_bytes() {
        let purchase = OwnerShopUndoProjection {
            phase: 1, player_id: 7, can_undo: true, reason: crate::runtime::shop::UNDO_REASON_CAN_UNDO,
            newest_sequence: 4, commitment: [9; 32],
        };
        let retired = OwnerShopUndoProjection {
            phase: 0, player_id: 7, can_undo: false, reason: crate::runtime::shop::UNDO_REASON_RETIRED,
            newest_sequence: 0, commitment: [0; 32],
        };
        let event = |index: u32, state: &OwnerShopUndoProjection| TeamPublicEvent {
            event_kind: FactKind::OwnerShopUndo as u32,
            stable_sub_index: index,
            sanitized_payload: state.encode(),
            ..Default::default()
        };
        let material = owner_undo_hash_material(&[event(2, &purchase)]);
        assert_eq!(canonical_retained_owner_undo_material(&material).as_deref(), Some(material.as_slice()));
        assert!(retained_owner_undo_metadata(&[]).is_none());
        assert!(retained_owner_undo_from_metadata(&[]).unwrap().is_empty());
        let entry = retained_owner_undo_metadata(&material).unwrap();
        assert_eq!(entry.schema_version, OWNER_UNDO_RETAINED_SCHEMA);
        assert_eq!(retained_owner_undo_from_metadata(std::slice::from_ref(&entry)).unwrap(), material);
        let retired_material = owner_undo_hash_material(&[event(1, &retired)]);
        assert!(canonical_retained_owner_undo_material(&retired_material).is_some());
        let both = owner_undo_hash_material(&[event(1, &retired), event(4, &purchase)]);
        assert_eq!(canonical_retained_owner_undo_material(&both).as_deref(), Some(both.as_slice()));

        let mut truncated = entry.clone();
        truncated.value.pop();
        assert!(retained_owner_undo_from_metadata(std::slice::from_ref(&truncated)).is_none());
        let mut empty_value = entry.clone();
        empty_value.value.clear();
        assert!(retained_owner_undo_from_metadata(std::slice::from_ref(&empty_value)).is_none());
        let mut bad_schema = entry.clone();
        bad_schema.schema_version = OWNER_UNDO_RETAINED_SCHEMA + 1;
        assert!(retained_owner_undo_from_metadata(std::slice::from_ref(&bad_schema)).is_none());
        assert!(retained_owner_undo_from_metadata(&[entry.clone(), entry.clone()]).is_none());

        let mut duplicate_index = both.clone();
        let record = 4 + 8 + OwnerShopUndoProjection::LEN;
        duplicate_index.copy_within(0..4, record);
        assert!(canonical_retained_owner_undo_material(&duplicate_index).is_none());
        let mut disorder = both.clone();
        let first = disorder[0..4].to_vec();
        let second = disorder[record..record + 4].to_vec();
        disorder[0..4].copy_from_slice(&second);
        disorder[record..record + 4].copy_from_slice(&first);
        assert!(canonical_retained_owner_undo_material(&disorder).is_none());
        let same_phase = owner_undo_hash_material(&[event(1, &purchase), event(2, &purchase)]);
        assert!(canonical_retained_owner_undo_material(&same_phase).is_none());
        let mut trailing = material.clone();
        trailing.push(0);
        assert!(canonical_retained_owner_undo_material(&trailing).is_none());
        let mut short_payload = material.clone();
        short_payload[4..12].copy_from_slice(&47u64.to_be_bytes());
        assert!(canonical_retained_owner_undo_material(&short_payload).is_none());
        assert!(metadata_declares_owner_undo(std::slice::from_ref(&entry)));
        assert!(!metadata_declares_owner_undo(&[crate::game_proto::DeterministicMetadata {
            namespace: "fog-evidence".into(),
            key: "team-sentinel".into(),
            schema_version: 1,
            value: vec![1],
        }]));
    }
}
