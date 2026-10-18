//! Independent ward wire opt-in. QWER, items, summoners, and shop do not imply
//! this capability. Version 0 with an empty hash stays disabled. The client
//! names only local slot 0 and both explicit Fixed64 raw coordinates.
//! Owner, catalog, charges, lifetime, and entity identity stay on the server.

use sha2::{Digest, Sha256};

use omoba_sim::{Fixed64, Vec2};

use super::native::ward_world::WARD_LOCAL_SLOT;
use super::summoner_runtime::SIM_TERRAIN_COORD_MAX_RAW;

/// Admission requires a sealed owner envelope on bootstrap. Version 1 and the
/// previously accepted version 2 stay ward-disabled. That mismatch does not
/// reject the rest of the join. Schema 2 is 147 bytes and is not protocol 2.
pub const WARD_PROTOCOL_VERSION: u32 = 3;
/// One read-only recovery query, plus one retry if that answer is also rejected.
pub const OWNER_WARD_RECOVERY_ATTEMPT_LIMIT: u8 = 2;
/// Private projection codec. Schema 1 (47 bytes, charges only) is rejected.
/// Envelope schema stays 1; this byte is inside the hashed projection.
pub const OWNER_WARD_SCHEMA: u32 = 2;
const OWNER_WARD_CHARGE_LEN: usize = 47;
pub const OWNER_WARD_OUTCOME_CAPACITY: usize = 8;
const OWNER_WARD_OUTCOME_RECORD_LEN: usize = 12;
/// Charge prefix, uncertain watermark, and eight packed outcome records.
pub const OWNER_WARD_PROJECTION_LEN: usize =
    OWNER_WARD_CHARGE_LEN + 4 + OWNER_WARD_OUTCOME_CAPACITY * OWNER_WARD_OUTCOME_RECORD_LEN;
pub const OWNER_WARD_OUTCOME_ADMITTED: u8 = 1;
pub const OWNER_WARD_OUTCOME_SETTLED: u8 = 2;
pub const OWNER_WARD_OUTCOME_REJECTED: u8 = 3;
pub const DISCLOSED_WARD_BODY_LEN: usize = 9;
pub const DISCLOSED_WARD_DIRECT_ENEMY: u8 = 1;
pub const DISCLOSED_WARD_NOT_DIRECT: u8 = 2;
pub const DISCLOSED_WARD_AUTO_DISABLED: u8 = 1;
pub const DISCLOSED_WARD_AUTO_ENABLED: u8 = 2;

/// Admission queued the placement. Settlement has not run.
pub const RECEIPT_ADMITTED: &str = "ADMITTED";
/// `settle_ward_placement` committed the vision source and the roster charge.
pub const RECEIPT_SETTLED: &str = "SETTLED";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WardReceiptStage {
    AdmissionRejected,
    Admitted,
    Settled,
    SettlementRejected,
}

/// One owner-scoped command outcome. Not a charge delta and not an ACK.
/// `actor_generation` 0 means the admission captured no living actor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OwnerWardOutcome {
    pub admitted_correlation: u32,
    pub actor_generation: u32,
    pub stage: u8,
    pub slot: u8,
}

/// Bounded journal carried in the private projection. Correlations at or below
/// `uncertain_through` that are absent from `rows` are explicitly uncertain.
/// They are not success, not a fresh rejection, and must not be retried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerWardOutcomeBlock {
    pub uncertain_through: u32,
    pub count: u8,
    pub rows: [OwnerWardOutcome; OWNER_WARD_OUTCOME_CAPACITY],
}

impl Default for OwnerWardOutcomeBlock {
    fn default() -> Self {
        Self { uncertain_through: 0, count: 0, rows: [OwnerWardOutcome::default(); OWNER_WARD_OUTCOME_CAPACITY] }
    }
}

/// Canonical private owner trinket fact. Not a team roster and not an ACK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerWardProjection {
    pub player_id: u32,
    pub view_epoch: u64,
    pub catalog_id: u16,
    pub charges: u16,
    pub max_charges: u16,
    pub recharge_remaining_raw: i64,
    pub revision: u64,
    pub placement_capable: bool,
    pub elapsed_raw: i64,
}

/// Disclosed body identity. No lifetime, no charges, no hidden coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosedWardBody {
    pub catalog_id: u16,
    pub generation: u32,
    pub direct_attack: u8,
    pub automatic_acquisition: u8,
}

pub fn negotiate_ward_protocol(version: u32, rules_hash: &str) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() {
        return Ok(false);
    }
    if version == 0 {
        return Err("WARD_PROTOCOL_VERSION_MISMATCH");
    }
    // An older or unknown ward version disables only this capability.
    if version != WARD_PROTOCOL_VERSION {
        return Ok(false);
    }
    if rules_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
        return Err("WARD_RULES_HASH_MISMATCH");
    }
    Ok(true)
}

/// Tick the sealed bootstrap envelope must name. Snapshot tick wins when the
/// join fallback snapshot and the replica start tick differ.
pub fn owner_ward_bootstrap_tick(start: &crate::game_proto::TeamGameStart) -> u64 {
    start.filtered_snapshot.as_ref().map_or(start.replica_start_tick, |snapshot| snapshot.authoritative_tick)
}

/// Writes the private admission fields. A disabled session or unknown owner
/// carries neither an echo nor an envelope. An agreed owner with empty
/// projection bytes gets a verified unavailable envelope and keeps the binding.
pub fn stamp_owner_ward_admission(
    start: &mut crate::game_proto::TeamGameStart,
    agreed: bool,
    session_id: u64,
    projection: &[u8],
) {
    if !agreed || start.player_id == 0 || session_id == 0 {
        start.ward_protocol_version = 0;
        start.ward_rules_hash.clear();
        start.owner_ward_projection.clear();
        start.owner_ward_envelope = None;
        return;
    }
    let view_epoch = start.view_epoch.as_ref().map_or(0, |epoch| epoch.value);
    let tick = owner_ward_bootstrap_tick(start);
    start.ward_protocol_version = WARD_PROTOCOL_VERSION;
    start.ward_rules_hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH.to_owned();
    start.owner_ward_projection = projection.to_vec();
    start.owner_ward_envelope = Some(seal_owner_ward_envelope(
        start.player_id, session_id, view_epoch, tick, projection,
    ));
}

/// Coalesces read-only recovery queries for one view epoch.
#[derive(Clone, Copy, Debug, Default)]
pub struct OwnerWardRecoveryBudget {
    epoch: u64,
    attempts: u8,
}

impl OwnerWardRecoveryBudget {
    pub fn poll(&mut self, enabled: bool, needs_recovery: bool, view_epoch: u64) -> Option<u64> {
        if !enabled || !needs_recovery {
            return None;
        }
        if self.epoch != view_epoch {
            self.epoch = view_epoch;
            self.attempts = 0;
        }
        if self.attempts >= OWNER_WARD_RECOVERY_ATTEMPT_LIMIT {
            return None;
        }
        self.attempts = self.attempts.saturating_add(1);
        Some(view_epoch)
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    pub fn request_id(&self) -> u64 {
        u64::from(self.attempts).max(1)
    }
}

/// Both coordinates are required. A missing side is not a self cast.
pub fn admit_ward_placement(slot: u32, x_raw: Option<i64>, y_raw: Option<i64>) -> Result<Vec2, &'static str> {
    if slot != u32::from(WARD_LOCAL_SLOT) {
        return Err("INVALID_WARD_SLOT");
    }
    let (Some(x), Some(y)) = (x_raw, y_raw) else {
        return Err("MALFORMED_WARD_POINT");
    };
    if !in_envelope(x) || !in_envelope(y) {
        return Err("WARD_POINT_OUT_OF_ENVELOPE");
    }
    Ok(Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y)))
}

fn in_envelope(raw: i64) -> bool {
    raw.unsigned_abs() <= SIM_TERRAIN_COORD_MAX_RAW as u64
}

pub fn direct_attack_token(value: omoba_template_ids::TrinketDirectAttackConst) -> u8 {
    match value {
        omoba_template_ids::TrinketDirectAttackConst::DisclosedEnemy => DISCLOSED_WARD_DIRECT_ENEMY,
        omoba_template_ids::TrinketDirectAttackConst::NotDirectlyAttackable => DISCLOSED_WARD_NOT_DIRECT,
    }
}

pub fn automatic_acquisition_token(value: omoba_template_ids::TrinketAutomaticAcquisitionConst) -> u8 {
    match value {
        omoba_template_ids::TrinketAutomaticAcquisitionConst::Disabled => DISCLOSED_WARD_AUTO_DISABLED,
        omoba_template_ids::TrinketAutomaticAcquisitionConst::Enabled => DISCLOSED_WARD_AUTO_ENABLED,
    }
}

pub fn disclosed_direct_attack_allowed(token: u8) -> bool {
    token == DISCLOSED_WARD_DIRECT_ENEMY
}

pub fn disclosed_automatic_acquisition_allowed(token: u8) -> bool {
    token == DISCLOSED_WARD_AUTO_ENABLED
}

pub fn encode_owner_ward_projection(row: &OwnerWardProjection) -> Result<Vec<u8>, &'static str> {
    encode_owner_ward_private(row, &OwnerWardOutcomeBlock::default())
}

pub fn encode_owner_ward_private(
    row: &OwnerWardProjection,
    outcomes: &OwnerWardOutcomeBlock,
) -> Result<Vec<u8>, &'static str> {
    let mut out = Vec::with_capacity(OWNER_WARD_PROJECTION_LEN);
    out.extend_from_slice(&OWNER_WARD_SCHEMA.to_be_bytes());
    out.extend_from_slice(&row.player_id.to_be_bytes());
    out.extend_from_slice(&row.view_epoch.to_be_bytes());
    out.extend_from_slice(&row.catalog_id.to_be_bytes());
    out.extend_from_slice(&row.charges.to_be_bytes());
    out.extend_from_slice(&row.max_charges.to_be_bytes());
    out.extend_from_slice(&row.recharge_remaining_raw.to_be_bytes());
    out.extend_from_slice(&row.revision.to_be_bytes());
    out.push(u8::from(row.placement_capable));
    out.extend_from_slice(&row.elapsed_raw.to_be_bytes());
    out.extend_from_slice(&outcomes.uncertain_through.to_be_bytes());
    for index in 0..OWNER_WARD_OUTCOME_CAPACITY {
        if index < usize::from(outcomes.count) {
            let outcome = &outcomes.rows[index];
            out.extend_from_slice(&outcome.admitted_correlation.to_be_bytes());
            out.extend_from_slice(&outcome.actor_generation.to_be_bytes());
            out.push(outcome.stage);
            out.push(outcome.slot);
            out.extend_from_slice(&0_u16.to_be_bytes());
        } else {
            out.extend_from_slice(&[0_u8; OWNER_WARD_OUTCOME_RECORD_LEN]);
        }
    }
    decode_owner_ward_private(&out).map(|_| out)
}

pub fn decode_owner_ward_projection(bytes: &[u8]) -> Result<OwnerWardProjection, &'static str> {
    decode_owner_ward_private(bytes).map(|(projection, _)| projection)
}

pub fn decode_owner_ward_private(bytes: &[u8]) -> Result<(OwnerWardProjection, OwnerWardOutcomeBlock), &'static str> {
    if bytes.len() == OWNER_WARD_CHARGE_LEN {
        let schema = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        if schema != OWNER_WARD_SCHEMA {
            return Err("OWNER_WARD_SCHEMA_MISMATCH");
        }
    }
    if bytes.len() != OWNER_WARD_PROJECTION_LEN {
        return Err("MALFORMED_OWNER_WARD");
    }
    let schema = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    if schema != OWNER_WARD_SCHEMA {
        return Err("OWNER_WARD_SCHEMA_MISMATCH");
    }
    let player_id = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
    let view_epoch = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
    let catalog_id = u16::from_be_bytes(bytes[16..18].try_into().unwrap());
    let charges = u16::from_be_bytes(bytes[18..20].try_into().unwrap());
    let max_charges = u16::from_be_bytes(bytes[20..22].try_into().unwrap());
    let recharge_remaining_raw = i64::from_be_bytes(bytes[22..30].try_into().unwrap());
    let revision = u64::from_be_bytes(bytes[30..38].try_into().unwrap());
    let capable = bytes[38];
    let elapsed_raw = i64::from_be_bytes(bytes[39..47].try_into().unwrap());
    if player_id == 0 || elapsed_raw < 0 || recharge_remaining_raw < 0 || capable > 1 {
        return Err("NONCANONICAL_OWNER_WARD");
    }
    if catalog_id == 0 {
        if charges != 0 || max_charges != 0 || recharge_remaining_raw != 0 || capable != 0 {
            return Err("NONCANONICAL_OWNER_WARD");
        }
    } else {
        let row = omoba_template_ids::TRINKET_CATALOG
            .iter()
            .find(|row| row.catalog_id == catalog_id)
            .ok_or("UNKNOWN_WARD_CATALOG")?;
        if max_charges != row.max_charges || charges > max_charges {
            return Err("NONCANONICAL_OWNER_WARD");
        }
        if recharge_remaining_raw > row.recharge_seconds.raw() {
            return Err("NONCANONICAL_OWNER_WARD");
        }
        if charges == max_charges && recharge_remaining_raw != 0 {
            return Err("NONCANONICAL_OWNER_WARD");
        }
        if capable == 1 && charges == 0 {
            return Err("NONCANONICAL_OWNER_WARD");
        }
    }
    if recharge_remaining_raw > 0 && elapsed_raw.checked_add(recharge_remaining_raw).is_none() {
        return Err("NONCANONICAL_OWNER_WARD");
    }
    let outcomes = decode_owner_ward_outcomes(&bytes[OWNER_WARD_CHARGE_LEN..])?;
    Ok((
        OwnerWardProjection {
            player_id,
            view_epoch,
            catalog_id,
            charges,
            max_charges,
            recharge_remaining_raw,
            revision,
            placement_capable: capable == 1,
            elapsed_raw,
        },
        outcomes,
    ))
}

fn decode_owner_ward_outcomes(bytes: &[u8]) -> Result<OwnerWardOutcomeBlock, &'static str> {
    let uncertain_through = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let mut rows = [OwnerWardOutcome::default(); OWNER_WARD_OUTCOME_CAPACITY];
    let mut count = 0_u8;
    let mut packed_ended = false;
    let mut previous = uncertain_through;
    for index in 0..OWNER_WARD_OUTCOME_CAPACITY {
        let base = 4 + index * OWNER_WARD_OUTCOME_RECORD_LEN;
        let record = &bytes[base..base + OWNER_WARD_OUTCOME_RECORD_LEN];
        let correlation = u32::from_be_bytes(record[0..4].try_into().unwrap());
        let actor_generation = u32::from_be_bytes(record[4..8].try_into().unwrap());
        let stage = record[8];
        let slot = record[9];
        let reserved = u16::from_be_bytes(record[10..12].try_into().unwrap());
        let empty = correlation == 0 && actor_generation == 0 && stage == 0 && slot == 0 && reserved == 0;
        if empty {
            packed_ended = true;
            continue;
        }
        if packed_ended || reserved != 0 || slot != 0 || !matches!(stage, OWNER_WARD_OUTCOME_ADMITTED | OWNER_WARD_OUTCOME_SETTLED | OWNER_WARD_OUTCOME_REJECTED) {
            return Err("NONCANONICAL_OWNER_WARD");
        }
        if correlation <= previous {
            return Err("NONCANONICAL_OWNER_WARD");
        }
        previous = correlation;
        rows[usize::from(count)] = OwnerWardOutcome { admitted_correlation: correlation, actor_generation, stage, slot };
        count = count.saturating_add(1);
    }
    Ok(OwnerWardOutcomeBlock { uncertain_through, count, rows })
}

pub fn hash_owner_ward_projection(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    out
}

pub const OWNER_WARD_ENVELOPE_SCHEMA: u32 = 1;

/// Stable nonzero binding for one authenticated connection and owner.
pub fn owner_ward_session_binding(session_key: &str, player_id: u32) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(b"owner-ward-session-v1");
    hasher.update(session_key.as_bytes());
    hasher.update(player_id.to_be_bytes());
    let digest = hasher.finalize();
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&digest[..8]);
    let value = u64::from_be_bytes(raw);
    if value == 0 { 1 } else { value }
}

/// Canonical owner envelope. Empty projection is the terminal clear, and its
/// hash is empty. A nonempty projection's hash is `hash_owner_ward_projection`.
pub fn seal_owner_ward_envelope(
    player_id: u32,
    session_id: u64,
    view_epoch: u64,
    authoritative_tick: u64,
    projection: &[u8],
) -> crate::game_proto::OwnerWardEnvelope {
    let projection_hash = if projection.is_empty() {
        Vec::new()
    } else {
        hash_owner_ward_projection(projection).to_vec()
    };
    crate::game_proto::OwnerWardEnvelope {
        schema_version: OWNER_WARD_ENVELOPE_SCHEMA,
        player_id,
        session_id,
        view_epoch,
        authoritative_tick,
        projection: projection.to_vec(),
        projection_hash,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerWardEnvelopeBody {
    /// Verified terminal/unavailable state. Capability must be cleared.
    Clear,
    /// Verified private bytes. The caller still orders and binds them.
    Apply { projection: OwnerWardProjection, hash: [u8; 32] },
}

/// Hash, schema, and canonical-byte gate. Order, owner, and epoch are the replica's.
pub fn open_owner_ward_envelope(
    envelope: &crate::game_proto::OwnerWardEnvelope,
) -> Result<OwnerWardEnvelopeBody, &'static str> {
    if envelope.schema_version != OWNER_WARD_ENVELOPE_SCHEMA || envelope.player_id == 0 || envelope.session_id == 0 {
        return Err("MALFORMED_OWNER_WARD");
    }
    if envelope.projection.is_empty() {
        if !envelope.projection_hash.is_empty() {
            return Err("OWNER_WARD_HASH_MISMATCH");
        }
        return Ok(OwnerWardEnvelopeBody::Clear);
    }
    let hash = hash_owner_ward_projection(&envelope.projection);
    if envelope.projection_hash.as_slice() != hash {
        return Err("OWNER_WARD_HASH_MISMATCH");
    }
    let projection = decode_owner_ward_projection(&envelope.projection)?;
    if projection.player_id != envelope.player_id || projection.view_epoch != envelope.view_epoch {
        return Err("OWNER_WARD_OWNER_OR_EPOCH_MISMATCH");
    }
    Ok(OwnerWardEnvelopeBody::Apply { projection, hash })
}

/// Session stamp. Observers and missing owners receive empty bytes.
pub fn owner_projection_for_session(owner_wards: &std::collections::BTreeMap<u32, Vec<u8>>, player_id: u32) -> Vec<u8> {
    if player_id == 0 {
        return Vec::new();
    }
    owner_wards.get(&player_id).cloned().unwrap_or_default()
}

pub fn encode_disclosed_ward_body(catalog_id: u16, generation: u32, direct_attack: u8, automatic_acquisition: u8) -> Option<Vec<u8>> {
    let body = DisclosedWardBody { catalog_id, generation, direct_attack, automatic_acquisition };
    let mut out = Vec::with_capacity(DISCLOSED_WARD_BODY_LEN);
    out.push(1);
    out.extend_from_slice(&body.catalog_id.to_be_bytes());
    out.extend_from_slice(&body.generation.to_be_bytes());
    out.push(body.direct_attack);
    out.push(body.automatic_acquisition);
    decode_disclosed_ward_body(&out)?;
    Some(out)
}

pub fn decode_disclosed_ward_body(bytes: &[u8]) -> Option<DisclosedWardBody> {
    if bytes.len() != DISCLOSED_WARD_BODY_LEN || bytes[0] != 1 {
        return None;
    }
    let catalog_id = u16::from_be_bytes(bytes[1..3].try_into().ok()?);
    let generation = u32::from_be_bytes(bytes[3..7].try_into().ok()?);
    let direct_attack = bytes[7];
    let automatic_acquisition = bytes[8];
    if generation == 0 || !matches!(direct_attack, DISCLOSED_WARD_DIRECT_ENEMY | DISCLOSED_WARD_NOT_DIRECT) {
        return None;
    }
    if !matches!(automatic_acquisition, DISCLOSED_WARD_AUTO_DISABLED | DISCLOSED_WARD_AUTO_ENABLED) {
        return None;
    }
    if !omoba_template_ids::TRINKET_CATALOG.iter().any(|row| row.catalog_id == catalog_id) {
        return None;
    }
    Some(DisclosedWardBody { catalog_id, generation, direct_attack, automatic_acquisition })
}

/// Renderer admission was forwarded. This is not settlement.
pub const WARD_PHASE_FORWARDED: u32 = 1;
/// The command reached the presentation runtime. This is not settlement.
pub const WARD_PHASE_APPLIED: u32 = 2;
/// The transport result is unknown. Do not retry it as a new placement.
pub const WARD_PHASE_UNCERTAIN: u32 = 3;
/// The owner runtime rejected the command before authority settlement.
pub const WARD_PHASE_REJECTED: u32 = 4;
pub const WARD_CORRELATION_CAPACITY: usize = 8;

/// One durable identity. The sealed authority outcome stays a separate record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardCorrelationDraft {
    pub player_id: u32,
    pub view_epoch: u64,
    pub actor_generation: u32,
    pub renderer_request: u64,
    pub authority_input: u32,
    pub phase: u32,
}

pub fn ward_correlation_row_ok(owner: u32, view_epoch: u64, row: &WardCorrelationDraft) -> bool {
    owner != 0
        && view_epoch != 0
        && row.player_id == owner
        && row.view_epoch == view_epoch
        && row.renderer_request != 0
        && (1..=4).contains(&row.phase)
        && (row.authority_input != 0 || row.phase == WARD_PHASE_REJECTED)
}

/// `None` drops the whole list. An empty input is an empty list, not a failure.
pub fn accept_ward_correlations(owner: u32, view_epoch: u64, rows: &[WardCorrelationDraft]) -> Option<Vec<WardCorrelationDraft>> {
    if rows.len() > WARD_CORRELATION_CAPACITY {
        return None;
    }
    let mut seen_request = std::collections::BTreeSet::new();
    let mut seen_authority = std::collections::BTreeSet::new();
    for row in rows {
        if !ward_correlation_row_ok(owner, view_epoch, row) || !seen_request.insert(row.renderer_request) {
            return None;
        }
        if row.authority_input != 0 && !seen_authority.insert(row.authority_input) {
            return None;
        }
    }
    Some(rows.to_vec())
}

/// A sealed outcome settles one mapped request. Actor 0 does not compare generations.
/// A rejected local phase has no authority id and does not settle.
pub fn ward_outcome_matches(row: &WardCorrelationDraft, outcome_correlation: u32, outcome_actor: u32) -> bool {
    row.authority_input != 0
        && outcome_correlation == row.authority_input
        && (row.actor_generation == 0 || outcome_actor == row.actor_generation)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equipped(charges: u16, remaining: i64, capable: bool) -> OwnerWardProjection {
        let row = omoba_template_ids::TRINKET_CATALOG.iter().find(|row| row.catalog_id == 1).unwrap();
        OwnerWardProjection {
            player_id: 7,
            view_epoch: 3,
            catalog_id: row.catalog_id,
            charges,
            max_charges: row.max_charges,
            recharge_remaining_raw: remaining,
            revision: 4,
            placement_capable: capable,
            elapsed_raw: 100,
        }
    }

    #[test]
    fn ward_capability_is_closed_for_legacy_partial_and_wrong_hash() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_ward_protocol(0, ""), Ok(false));
        assert_eq!(WARD_PROTOCOL_VERSION, 3);
        assert_eq!(negotiate_ward_protocol(WARD_PROTOCOL_VERSION, hash), Ok(true));
        assert_eq!(negotiate_ward_protocol(1, hash), Ok(false));
        assert_eq!(negotiate_ward_protocol(1, ""), Ok(false));
        assert_eq!(negotiate_ward_protocol(1, "old"), Ok(false));
        assert_eq!(negotiate_ward_protocol(2, hash), Ok(false));
        assert_eq!(negotiate_ward_protocol(2, ""), Ok(false));
        assert_eq!(negotiate_ward_protocol(2, "old"), Ok(false));
        assert_eq!(negotiate_ward_protocol(0, hash), Err("WARD_PROTOCOL_VERSION_MISMATCH"));
        assert_eq!(negotiate_ward_protocol(WARD_PROTOCOL_VERSION, ""), Err("WARD_RULES_HASH_MISMATCH"));
        assert_eq!(negotiate_ward_protocol(WARD_PROTOCOL_VERSION, "old"), Err("WARD_RULES_HASH_MISMATCH"));
    }

    #[test]
    fn ward_wire_requires_slot_zero_and_both_points() {
        assert_eq!(admit_ward_placement(1, Some(1), Some(1)).unwrap_err(), "INVALID_WARD_SLOT");
        assert_eq!(admit_ward_placement(0, None, None).unwrap_err(), "MALFORMED_WARD_POINT");
        assert_eq!(admit_ward_placement(0, Some(1), None).unwrap_err(), "MALFORMED_WARD_POINT");
        assert_eq!(admit_ward_placement(0, None, Some(1)).unwrap_err(), "MALFORMED_WARD_POINT");
        let far = SIM_TERRAIN_COORD_MAX_RAW.saturating_add(1);
        assert_eq!(admit_ward_placement(0, Some(far), Some(0)).unwrap_err(), "WARD_POINT_OUT_OF_ENVELOPE");
        assert!(admit_ward_placement(0, Some(3), Some(4)).is_ok());
    }

    #[test]
    fn owner_ward_codec_rejects_malformed_old_schema_and_noncanonical_ranges() {
        assert_eq!(OWNER_WARD_SCHEMA, 2);
        assert_eq!(OWNER_WARD_PROJECTION_LEN, 147);
        assert_eq!(OWNER_WARD_ENVELOPE_SCHEMA, 1);
        let bytes = encode_owner_ward_projection(&equipped(1, 0, true)).unwrap();
        assert_eq!(bytes.len(), OWNER_WARD_PROJECTION_LEN);
        assert_eq!(decode_owner_ward_projection(&bytes).unwrap().player_id, 7);
        assert!(decode_owner_ward_projection(&bytes[..46]).is_err());
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(decode_owner_ward_projection(&extra).is_err());
        let mut old = bytes.clone();
        old[3] = 0;
        assert_eq!(decode_owner_ward_projection(&old).unwrap_err(), "OWNER_WARD_SCHEMA_MISMATCH");
        let mut full = equipped(2, 1, false);
        assert!(encode_owner_ward_projection(&full).is_err());
        full.recharge_remaining_raw = 0;
        full.placement_capable = false;
        assert!(encode_owner_ward_projection(&full).is_ok());
        let mut capable = equipped(0, 10, true);
        assert!(encode_owner_ward_projection(&capable).is_err());
        capable.placement_capable = false;
        assert!(encode_owner_ward_projection(&capable).is_ok());
        let mut unequipped = equipped(0, 0, false);
        unequipped.catalog_id = 0;
        unequipped.max_charges = 0;
        assert!(encode_owner_ward_projection(&unequipped).is_ok());
        unequipped.charges = 1;
        assert!(encode_owner_ward_projection(&unequipped).is_err());
        let mut unknown = equipped(1, 0, false);
        unknown.catalog_id = 99;
        assert!(encode_owner_ward_projection(&unknown).is_err());
        let mut legacy = [0_u8; 47];
        legacy[3] = 1;
        legacy[7] = 7;
        assert_eq!(decode_owner_ward_projection(&legacy).unwrap_err(), "OWNER_WARD_SCHEMA_MISMATCH");
    }

    fn settled_block(correlation: u32, actor: u32) -> OwnerWardOutcomeBlock {
        let mut block = OwnerWardOutcomeBlock::default();
        block.count = 1;
        block.rows[0] = OwnerWardOutcome {
            admitted_correlation: correlation,
            actor_generation: actor,
            stage: OWNER_WARD_OUTCOME_SETTLED,
            slot: 0,
        };
        block
    }

    #[test]
    fn owner_ward_outcome_trailer_is_canonical_and_rejects_stale_duplicates() {
        let row = equipped(1, 0, true);
        let bytes = encode_owner_ward_private(&row, &settled_block(9, 4)).unwrap();
        assert_eq!(bytes.len(), OWNER_WARD_PROJECTION_LEN);
        let (decoded, outcomes) = decode_owner_ward_private(&bytes).unwrap();
        assert_eq!(decoded.charges, 1);
        assert_eq!(outcomes.count, 1);
        assert_eq!(outcomes.rows[0].stage, OWNER_WARD_OUTCOME_SETTLED);
        assert_eq!(outcomes.rows[0].actor_generation, 4);
        assert_eq!(outcomes.uncertain_through, 0);
        let mut duplicate = settled_block(9, 4);
        duplicate.count = 2;
        duplicate.rows[1] = duplicate.rows[0];
        assert_eq!(encode_owner_ward_private(&row, &duplicate).unwrap_err(), "NONCANONICAL_OWNER_WARD");
        let mut late = settled_block(3, 4);
        late.uncertain_through = 3;
        assert_eq!(encode_owner_ward_private(&row, &late).unwrap_err(), "NONCANONICAL_OWNER_WARD");
        let mut padded = encode_owner_ward_private(&row, &settled_block(9, 4)).unwrap();
        let record = OWNER_WARD_CHARGE_LEN + 4;
        padded[record + 10] = 1;
        assert_eq!(decode_owner_ward_private(&padded).unwrap_err(), "NONCANONICAL_OWNER_WARD");
        let mut hole = encode_owner_ward_private(&row, &settled_block(9, 4)).unwrap();
        let start = OWNER_WARD_CHARGE_LEN + 4;
        let first = hole[start..start + 12].to_vec();
        hole[start..start + 12].fill(0);
        hole[start + 12..start + 24].copy_from_slice(&first);
        assert_eq!(decode_owner_ward_private(&hole).unwrap_err(), "NONCANONICAL_OWNER_WARD");
        let mut foreign = seal_owner_ward_envelope(7, 1, 3, 1, &bytes);
        foreign.player_id = 8;
        assert_eq!(open_owner_ward_envelope(&foreign).unwrap_err(), "OWNER_WARD_OWNER_OR_EPOCH_MISMATCH");
        let mut epoch = seal_owner_ward_envelope(7, 1, 3, 1, &bytes);
        epoch.view_epoch = 9;
        assert_eq!(open_owner_ward_envelope(&epoch).unwrap_err(), "OWNER_WARD_OWNER_OR_EPOCH_MISMATCH");
    }

    #[test]
    fn owner_outcome_journal_bounds_epoch_duplicate_and_uncertain_watermark() {
        use crate::comp::OwnerWardOutcomeJournal;
        let mut journal = OwnerWardOutcomeJournal::default();
        let make = |correlation, stage| OwnerWardOutcome {
            admitted_correlation: correlation,
            actor_generation: 4,
            stage,
            slot: 0,
        };
        journal.note(1, 4, make(8, OWNER_WARD_OUTCOME_SETTLED));
        journal.note(1, 4, make(8, OWNER_WARD_OUTCOME_REJECTED));
        let kept = journal.block_for(1, 4);
        assert_eq!(kept.count, 1);
        assert_eq!(kept.rows[0].stage, OWNER_WARD_OUTCOME_SETTLED);
        journal.note(1, 4, make(8, OWNER_WARD_OUTCOME_ADMITTED));
        assert_eq!(journal.block_for(1, 4).rows[0].stage, OWNER_WARD_OUTCOME_SETTLED);
        for correlation in 1..=9 {
            journal.note(1, 4, make(correlation, OWNER_WARD_OUTCOME_REJECTED));
        }
        let bounded = journal.block_for(1, 4);
        assert_eq!(bounded.count, 8);
        assert!(bounded.uncertain_through >= 1);
        assert!(bounded.rows.iter().take(bounded.count as usize).all(|row| row.admitted_correlation > bounded.uncertain_through));
        assert!(journal.block_for(1, 5).count == 0, "a new epoch must not keep the previous journal");
        assert_eq!(journal.block_for(2, 4).count, 0);
        journal.note(1, 4, make(bounded.uncertain_through, OWNER_WARD_OUTCOME_SETTLED));
        assert_eq!(journal.block_for(1, 4).count, 8);
    }

    #[test]
    fn disclosed_body_keeps_direct_and_automatic_policy_independent() {
        let bytes = encode_disclosed_ward_body(1, 4, DISCLOSED_WARD_NOT_DIRECT, DISCLOSED_WARD_AUTO_ENABLED).unwrap();
        let body = decode_disclosed_ward_body(&bytes).unwrap();
        assert!(!disclosed_direct_attack_allowed(body.direct_attack));
        assert!(disclosed_automatic_acquisition_allowed(body.automatic_acquisition));
        assert!(encode_disclosed_ward_body(1, 0, DISCLOSED_WARD_DIRECT_ENEMY, DISCLOSED_WARD_AUTO_DISABLED).is_none());
        assert!(encode_disclosed_ward_body(1, 4, 9, DISCLOSED_WARD_AUTO_DISABLED).is_none());
        assert!(encode_disclosed_ward_body(99, 4, DISCLOSED_WARD_DIRECT_ENEMY, DISCLOSED_WARD_AUTO_DISABLED).is_none());
        assert_eq!(bytes.len(), DISCLOSED_WARD_BODY_LEN);
    }

    #[test]
    fn session_stamp_returns_only_the_authenticated_owner_bytes() {
        let mut rows = std::collections::BTreeMap::new();
        rows.insert(1, vec![1]);
        rows.insert(2, vec![2]);
        assert_eq!(owner_projection_for_session(&rows, 1), vec![1]);
        assert_eq!(owner_projection_for_session(&rows, 2), vec![2]);
        assert!(owner_projection_for_session(&rows, 0).is_empty());
        assert!(owner_projection_for_session(&rows, 3).is_empty());
    }

    #[test]
    fn owner_ward_envelope_consumes_the_canonical_hash_and_rejects_a_mismatch() {
        let bytes = encode_owner_ward_projection(&equipped(1, 0, true)).unwrap();
        let session = owner_ward_session_binding("kcp_test", 7);
        assert_ne!(session, 0);
        let sealed = seal_owner_ward_envelope(7, session, 3, 11, &bytes);
        assert_eq!(sealed.projection_hash.as_slice(), hash_owner_ward_projection(&bytes));
        match open_owner_ward_envelope(&sealed).unwrap() {
            OwnerWardEnvelopeBody::Apply { projection, hash } => {
                assert_eq!(projection.player_id, 7);
                assert_eq!(hash.as_slice(), sealed.projection_hash.as_slice());
            }
            OwnerWardEnvelopeBody::Clear => panic!("equipped projection must apply"),
        }
        let mut flipped = sealed.clone();
        flipped.projection_hash[0] ^= 0xff;
        assert_eq!(open_owner_ward_envelope(&flipped).unwrap_err(), "OWNER_WARD_HASH_MISMATCH");
        let clear = seal_owner_ward_envelope(7, session, 3, 12, &[]);
        assert!(clear.projection_hash.is_empty());
        assert_eq!(open_owner_ward_envelope(&clear).unwrap(), OwnerWardEnvelopeBody::Clear);
        let mut hashed_clear = clear.clone();
        hashed_clear.projection_hash = vec![1];
        assert!(open_owner_ward_envelope(&hashed_clear).is_err());
    }

    fn admitted_start(player: u32, epoch: u64, tick: u64, projection: &[u8], session_key: &str) -> crate::game_proto::TeamGameStart {
        use crate::game_proto::{FilteredTeamSnapshot, TeamGameStart, ViewEpoch};
        use sha2::{Digest, Sha256};
        let disclosed_world = Vec::new();
        let mut start = TeamGameStart {
            protocol_version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            player_id: player,
            team_id: 1,
            replica_start_tick: tick,
            next_team_sequence: 1,
            tick_rate_hz: 20,
            view_epoch: Some(ViewEpoch { value: epoch }),
            filtered_snapshot: Some(FilteredTeamSnapshot {
                team_id: 1,
                authoritative_tick: tick,
                view_epoch: Some(ViewEpoch { value: epoch }),
                disclosed_world: disclosed_world.clone(),
                filtered_snapshot_hash: Sha256::digest(&disclosed_world).to_vec(),
                ..FilteredTeamSnapshot::default()
            }),
            ..TeamGameStart::default()
        };
        let session = if player == 0 { 0 } else { owner_ward_session_binding(session_key, player) };
        stamp_owner_ward_admission(&mut start, player != 0, session, projection);
        start
    }

    fn boot(start: &crate::game_proto::TeamGameStart) -> crate::runtime::SelectiveReplicaRuntime {
        crate::runtime::SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            start, std::collections::BTreeSet::new(), std::collections::BTreeSet::new(),
        ).expect("ward bootstrap")
    }

    #[test]
    fn verified_owner_ward_receiver_rejects_unverified_order_and_restores_by_recovery() {
        assert_eq!(WARD_PROTOCOL_VERSION, 3);
        assert_eq!(crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION, 13);
        let bytes = encode_owner_ward_projection(&equipped(1, 0, true)).unwrap();
        let start = admitted_start(7, 3, 11, &bytes, "receiver");
        assert_eq!(start.ward_protocol_version, 3);
        assert_eq!(start.protocol_version, crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION);
        let echo = start.owner_ward_projection.clone();
        let sealed = start.owner_ward_envelope.clone().unwrap();
        assert_eq!(echo, sealed.projection);
        assert_eq!(sealed.projection_hash.as_slice(), hash_owner_ward_projection(&bytes));
        let mut replica = boot(&start);
        let mut other_row = equipped(1, 0, true);
        other_row.player_id = 8;
        let other_bytes = encode_owner_ward_projection(&other_row).unwrap();
        let other = boot(&admitted_start(8, 3, 11, &other_bytes, "other-owner"));
        assert_eq!(replica.canonical_team_hash(), other.canonical_team_hash());
        assert_eq!(replica.owner_ward_projection(), bytes.as_slice());
        assert_eq!(replica.owner_ward_player(), 7);
        assert_eq!(replica.owner_ward_tick(), 11);
        assert!(!replica.owner_ward_needs_recovery());
        let frontier = replica.owner_ward_revision();
        let session = replica.owner_ward_session();

        let mut poisoned = sealed.clone();
        poisoned.authoritative_tick = u64::MAX;
        poisoned.projection_hash[0] ^= 0xff;
        assert!(replica.apply_verified_owner_ward_envelope(&poisoned).is_err());
        assert!(replica.owner_ward_projection().is_empty());
        assert_eq!(replica.owner_ward_tick(), 11);
        assert_eq!(replica.owner_ward_revision(), frontier);
        assert_eq!(replica.owner_ward_session(), session);
        assert!(replica.owner_ward_needs_recovery());

        let mut budget = OwnerWardRecoveryBudget::default();
        assert_eq!(budget.poll(true, replica.owner_ward_needs_recovery(), replica.view_epoch()), Some(3));
        assert_eq!(budget.poll(true, true, 3), Some(3));
        assert_eq!(budget.poll(true, true, 3), None);

        let mut schema = sealed.clone();
        schema.schema_version = 9;
        schema.authoritative_tick = u64::MAX;
        assert!(replica.apply_verified_owner_ward_envelope(&schema).is_err());
        assert_eq!(replica.owner_ward_tick(), 11);
        let mut foreign = seal_owner_ward_envelope(8, other.owner_ward_session(), 3, 11, &bytes);
        assert!(replica.apply_verified_owner_ward_envelope(&foreign).is_err());
        foreign.player_id = 7;
        foreign.session_id = owner_ward_session_binding("foreign", 7);
        foreign.projection_hash = hash_owner_ward_projection(&bytes).to_vec();
        assert!(replica.apply_verified_owner_ward_envelope(&foreign).is_err());
        let mut epoch = sealed.clone();
        epoch.view_epoch = 99;
        assert!(replica.apply_verified_owner_ward_envelope(&epoch).is_err());
        assert_eq!(replica.owner_ward_tick(), 11);
        assert_eq!(replica.owner_ward_player(), 7);

        let restored = seal_owner_ward_envelope(7, session, 3, 11, &bytes);
        replica.apply_verified_owner_ward_envelope(&restored).unwrap();
        assert_eq!(replica.owner_ward_projection(), bytes.as_slice());
        assert!(!replica.owner_ward_needs_recovery());
        replica.apply_verified_owner_ward_envelope(&restored).unwrap();
        let mut older = restored.clone();
        older.authoritative_tick = 10;
        assert_eq!(replica.apply_verified_owner_ward_envelope(&older).unwrap_err(), "OWNER_WARD_STALE");
        assert_eq!(replica.owner_ward_projection(), bytes.as_slice());

        let mut changed = equipped(1, 0, true);
        changed.revision = changed.revision.saturating_add(1);
        changed.charges = changed.charges.saturating_sub(1);
        changed.placement_capable = changed.charges != 0;
        let spent_revision = changed.revision;
        let spent = encode_owner_ward_projection(&changed).unwrap();
        let spent_envelope = seal_owner_ward_envelope(7, session, 3, 11, &spent);
        replica.apply_verified_owner_ward_envelope(&spent_envelope).unwrap();
        assert_eq!(replica.owner_ward_projection(), spent.as_slice());
        let mut twin = changed;
        twin.elapsed_raw = 101;
        let twin_bytes = encode_owner_ward_projection(&twin).unwrap();
        let twin_envelope = seal_owner_ward_envelope(7, session, 3, 11, &twin_bytes);
        assert_eq!(replica.apply_verified_owner_ward_envelope(&twin_envelope).unwrap_err(), "NONCANONICAL_OWNER_WARD");
        assert!(replica.owner_ward_projection().is_empty());
        assert_eq!(replica.owner_ward_tick(), 11);
        assert_eq!(replica.owner_ward_revision(), spent_revision);
        assert!(replica.owner_ward_needs_recovery());
        replica.apply_verified_owner_ward_envelope(&spent_envelope).unwrap();
        assert_eq!(replica.owner_ward_projection(), spent.as_slice());
        assert_eq!(replica.apply_verified_owner_ward_envelope(&sealed).unwrap_err(), "OWNER_WARD_STALE");
        assert_eq!(replica.owner_ward_projection(), spent.as_slice());
    }

    #[test]
    fn owner_ward_bootstrap_verifies_the_sealed_envelope_and_keeps_unavailable_binding() {
        let unavailable = admitted_start(7, 4, 2, &[], "unavailable");
        let replica = boot(&unavailable);
        assert!(replica.owner_ward_projection().is_empty());
        assert_eq!(replica.owner_ward_player(), 7);
        assert_ne!(replica.owner_ward_session(), 0);
        assert!(!replica.owner_ward_needs_recovery());
        let mut later_row = equipped(1, 0, true);
        later_row.view_epoch = 4;
        let later = encode_owner_ward_projection(&later_row).unwrap();
        let session = replica.owner_ward_session();
        let mut resume = replica;
        resume.apply_verified_owner_ward_envelope(&seal_owner_ward_envelope(7, session, 4, 3, &later)).unwrap();
        assert_eq!(resume.owner_ward_projection(), later.as_slice());

        let mut echo = admitted_start(7, 4, 2, &later, "echo");
        echo.owner_ward_projection = vec![9];
        assert!(crate::runtime::SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            &echo, std::collections::BTreeSet::new(), std::collections::BTreeSet::new(),
        ).is_err());
        let mut missing = admitted_start(7, 4, 2, &later, "missing");
        missing.owner_ward_envelope = None;
        assert!(crate::runtime::SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            &missing, std::collections::BTreeSet::new(), std::collections::BTreeSet::new(),
        ).is_err());
        let observer = admitted_start(0, 4, 2, &later, "observer");
        assert!(observer.owner_ward_envelope.is_none());
        assert!(observer.owner_ward_projection.is_empty());
        let mut observer_runtime = boot(&observer);
        assert_eq!(observer_runtime.owner_ward_player(), 0);
        assert!(observer_runtime.apply_verified_owner_ward_envelope(
            &seal_owner_ward_envelope(7, session, 4, 3, &later),
        ).is_err());
        assert!(observer_runtime.owner_ward_projection().is_empty());
    }

    #[test]
    fn previously_accepted_ward_protocol_disables_only_ward_during_selective_bootstrap() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_ward_protocol(2, hash), Ok(false));
        let mut start = admitted_start(7, 3, 11, &[], "legacy-empty");
        start.ward_protocol_version = 2;
        start.ward_rules_hash = hash.to_owned();
        start.owner_ward_projection.clear();
        start.owner_ward_envelope = None;
        assert_eq!(start.protocol_version, crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION);
        let replica = boot(&start);
        assert_eq!(replica.view_epoch(), 3);
        assert_eq!(replica.owner_ward_player(), 0);
        assert!(replica.owner_ward_projection().is_empty());
        assert!(!replica.owner_ward_needs_recovery());

        let mut legacy = [0_u8; 47];
        legacy[3] = 1;
        legacy[7] = 7;
        assert_eq!(decode_owner_ward_projection(&legacy).unwrap_err(), "OWNER_WARD_SCHEMA_MISMATCH");
        let mut carrying = start.clone();
        carrying.owner_ward_projection = legacy.to_vec();
        carrying.owner_ward_envelope = Some(seal_owner_ward_envelope(7, 1, 3, 11, &legacy));
        let disabled = boot(&carrying);
        assert_eq!(disabled.owner_ward_player(), 0);
        assert!(disabled.owner_ward_projection().is_empty());
        assert_eq!(disabled.view_epoch(), 3);
        assert!(!disabled.owner_ward_needs_recovery());

        let mut explicit = start.clone();
        explicit.ward_protocol_version = 0;
        explicit.ward_rules_hash.clear();
        explicit.owner_ward_projection = legacy.to_vec();
        let smuggled = crate::runtime::SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            &explicit, std::collections::BTreeSet::new(), std::collections::BTreeSet::new(),
        );
        assert_eq!(smuggled.err(), Some(crate::runtime::ReplicaRuntimeError::MalformedBaseline));
    }
}
