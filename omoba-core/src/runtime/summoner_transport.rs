//! Independent summoner wire opt-in. Command 2, Recall 2, and shop 1 do not
//! imply this capability. Version 0 with an empty hash stays disabled.
//! The maintained client boundary is ABI 22, presentation protocol 6, and HUD
//! schema 3. This module still only admits the wire shape and the owner-team
//! projection. It does not paint the Unreal D/F cards or settle their effects.

use omoba_sim::{Fixed64, Vec2};

use super::summoner_runtime::{SummonerCastRequest, SIM_TERRAIN_COORD_MAX_RAW};

pub const SUMMONER_PROTOCOL_VERSION: u32 = 1;
pub const SUMMONER_PROJECTION_NAMESPACE: &str = "summoner";
pub const SUMMONER_PROJECTION_KEY: &str = "owner-team-loadouts";
pub const SUMMONER_PROJECTION_SCHEMA: u32 = 1;

/// Admission queued the cast. Settlement has not run and no world state changed.
pub const RECEIPT_ADMITTED: &str = "ADMITTED";
/// `settle_summoner_cast` committed the effect and the roster clock.
pub const RECEIPT_SETTLED: &str = "SETTLED";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerReceiptStage {
    AdmissionRejected,
    Admitted,
    Settled,
    SettlementRejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectedSummonerSlot {
    pub catalog_id: u16,
    pub cooldown_raw: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectedSummonerPlayer {
    pub player_id: u32,
    pub slots: [ProjectedSummonerSlot; 2],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DisclosedSummonerProjection {
    pub players: Vec<ProjectedSummonerPlayer>,
}

pub fn negotiate_summoner_protocol(version: u32, rules_hash: &str) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() {
        return Ok(false);
    }
    if version != SUMMONER_PROTOCOL_VERSION {
        return Err("SUMMONER_PROTOCOL_VERSION_MISMATCH");
    }
    if rules_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
        return Err("SUMMONER_RULES_HASH_MISMATCH");
    }
    Ok(true)
}

/// Wire shape only. Spell identity, hero, and cost stay on the trusted catalog.
pub fn admit_summoner_cast(
    slot: u32,
    x_raw: Option<i64>,
    y_raw: Option<i64>,
) -> Result<SummonerCastRequest, &'static str> {
    if slot > 1 {
        return Err("INVALID_SUMMONER_SLOT");
    }
    let point = match (x_raw, y_raw) {
        (None, None) => None,
        (Some(x), Some(y)) => {
            if !in_envelope(x) || !in_envelope(y) {
                return Err("SUMMONER_POINT_OUT_OF_ENVELOPE");
            }
            Some(Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y)))
        }
        _ => return Err("MALFORMED_SUMMONER_POINT"),
    };
    Ok(SummonerCastRequest { slot: slot as u8, point })
}

fn in_envelope(raw: i64) -> bool {
    raw.unsigned_abs() <= SIM_TERRAIN_COORD_MAX_RAW as u64
}

pub fn catalog_id_for_spell(spell_id: Option<&str>) -> u16 {
    spell_id
        .and_then(omoba_template_ids::summoner_spell_by_name)
        .map(|spell| spell.catalog_id)
        .unwrap_or(0)
}

pub fn encode_projected_summoners(rows: &[ProjectedSummonerPlayer]) -> Vec<u8> {
    let mut rows = rows.to_vec();
    rows.sort_by_key(|row| row.player_id);
    let mut out = Vec::new();
    out.extend_from_slice(&(rows.len() as u32).to_be_bytes());
    for row in &rows {
        out.extend_from_slice(&row.player_id.to_be_bytes());
        for slot in &row.slots {
            out.extend_from_slice(&slot.catalog_id.to_be_bytes());
            out.extend_from_slice(&slot.cooldown_raw.to_be_bytes());
        }
    }
    out
}

pub fn decode_projected_summoners(bytes: &[u8]) -> Option<Vec<ProjectedSummonerPlayer>> {
    // Player id (u32) plus two interleaved catalog (u16) / clock (i64) slots.
    const PLAYER_BYTES: usize = 4 + 2 * (2 + 8);
    if bytes.len() < 4 || (bytes.len() - 4) % PLAYER_BYTES != 0 {
        return None;
    }
    let count = u32::from_be_bytes(bytes[0..4].try_into().ok()?) as usize;
    if count > 10 || 4 + count * PLAYER_BYTES != bytes.len() {
        return None;
    }
    let mut rows = Vec::with_capacity(count);
    let mut cursor = 4;
    for _ in 0..count {
        let player_id = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().ok()?);
        cursor += 4;
        if player_id == 0 {
            return None;
        }
        let mut slots = [ProjectedSummonerSlot { catalog_id: 0, cooldown_raw: 0 }; 2];
        for slot in &mut slots {
            slot.catalog_id = u16::from_be_bytes(bytes[cursor..cursor + 2].try_into().ok()?);
            cursor += 2;
            slot.cooldown_raw = i64::from_be_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
            cursor += 8;
            if !canonical_projected_clock(slot.catalog_id, slot.cooldown_raw) {
                return None;
            }
        }
        rows.push(ProjectedSummonerPlayer { player_id, slots });
    }
    rows.sort_by_key(|row| row.player_id);
    let ids = rows.iter().map(|row| row.player_id);
    (ids.clone().count() == ids.collect::<std::collections::BTreeSet<_>>().len()).then_some(rows)
}

/// Replica team that may refresh this projection. The value comes from the
/// admitted `TeamGameStart`, not from a HUD fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosedSummonerAuthority(pub u32);

/// Apply committed owner-team HUD facts onto players already present in the
/// bootstrap projection. A rejected fact leaves `current` unchanged. Clocks are
/// copied from the sanitized payload and are not advanced here.
pub fn refresh_disclosed_summoner_projection<'a>(
    current: &DisclosedSummonerProjection,
    authorized_team: u32,
    hud_payloads: impl IntoIterator<Item = &'a [u8]>,
) -> Option<DisclosedSummonerProjection> {
    if !matches!(authorized_team, 1 | 2) {
        return None;
    }
    let mut next = current.clone();
    let mut saw_summoner = false;
    for payload in hud_payloads {
        let update = match classify_summoner_hud(payload) {
            None => continue,
            Some(Err(())) => return None,
            Some(Ok(update)) => update,
        };
        let (team, player_id, slot, spell, value) = update;
        if team != authorized_team {
            return None;
        }
        let Some(row) = next.players.iter_mut().find(|row| row.player_id == player_id) else {
            return None;
        };
        saw_summoner = true;
        if spell {
            if !known_summoner_catalog(value) {
                return None;
            }
            row.slots[slot].catalog_id = value as u16;
        } else if value < 0 {
            return None;
        } else {
            row.slots[slot].cooldown_raw = value;
        }
    }
    // Validate the final batch, so catalog and clock facts may arrive in either
    // order without exposing a partially updated or illegal retained row.
    if next.players.iter().any(|row| row.slots.iter().any(|slot|
        !canonical_projected_clock(slot.catalog_id, slot.cooldown_raw))) {
        return None;
    }
    saw_summoner.then_some(next)
}

fn classify_summoner_hud(payload: &[u8]) -> Option<Result<(u32, u32, usize, bool, i64), ()>> {
    if payload.len() < 12 {
        return None;
    }
    let metric = u64::from_le_bytes(payload[4..12].try_into().ok()?);
    let namespace = metric & 0xffff_ffff_0000_0000;
    let (slot, spell) = match namespace {
        super::native::moba_match::SINGLE_LANE_SUMMONER_SLOT0_SPELL => (0, true),
        super::native::moba_match::SINGLE_LANE_SUMMONER_SLOT1_SPELL => (1, true),
        super::native::moba_match::SINGLE_LANE_SUMMONER_SLOT0_COOLDOWN => (0, false),
        super::native::moba_match::SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN => (1, false),
        _ => return None,
    };
    if payload.len() != 20 {
        return Some(Err(()));
    }
    let team = u32::from_le_bytes(payload[0..4].try_into().ok()?);
    let player_id = metric as u32;
    let value = i64::from_le_bytes(payload[12..20].try_into().ok()?);
    if player_id == 0 {
        return Some(Err(()));
    }
    Some(Ok((team, player_id, slot, spell, value)))
}

fn known_summoner_catalog(value: i64) -> bool {
    value == 0 || u16::try_from(value).ok().is_some_and(|id| {
        omoba_template_ids::SUMMONER_SPELL_CATALOG.iter().any(|spell| spell.catalog_id == id)
    })
}

/// Catalog 0 is unequipped and must carry a zero clock. An authored spell
/// cannot carry a remaining clock above its catalog total.
fn canonical_projected_clock(catalog_id: u16, cooldown_raw: i64) -> bool {
    if cooldown_raw < 0 || !known_summoner_catalog(i64::from(catalog_id)) {
        return false;
    }
    if catalog_id == 0 {
        return cooldown_raw == 0;
    }
    omoba_template_ids::SUMMONER_SPELL_CATALOG
        .iter()
        .find(|spell| spell.catalog_id == catalog_id)
        .is_some_and(|spell| cooldown_raw <= spell.cooldown.raw())
}

pub fn disclosed_summoner_projection_from_metadata(
    metadata: &[crate::game_proto::DeterministicMetadata],
) -> DisclosedSummonerProjection {
    let players = metadata
        .iter()
        .find(|row| {
            row.namespace == SUMMONER_PROJECTION_NAMESPACE
                && row.key == SUMMONER_PROJECTION_KEY
                && row.schema_version == SUMMONER_PROJECTION_SCHEMA
        })
        .and_then(|row| decode_projected_summoners(&row.value))
        .unwrap_or_default();
    DisclosedSummonerProjection { players }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summoner_capability_is_closed_for_legacy_partial_and_wrong_hash() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(SUMMONER_PROTOCOL_VERSION, 1);
        assert_eq!(negotiate_summoner_protocol(0, ""), Ok(false));
        assert_eq!(negotiate_summoner_protocol(SUMMONER_PROTOCOL_VERSION, hash), Ok(true));
        for (version, value) in [(0, hash), (2, hash), (1, ""), (1, "old")] {
            assert!(negotiate_summoner_protocol(version, value).is_err(), "{version} {value}");
        }
        assert_ne!(hash, "");
    }

    #[test]
    fn summoner_wire_admission_rejects_malformed_slot_point_and_keeps_shape() {
        assert_eq!(admit_summoner_cast(2, None, None).unwrap_err(), "INVALID_SUMMONER_SLOT");
        assert_eq!(admit_summoner_cast(4, None, None).unwrap_err(), "INVALID_SUMMONER_SLOT");
        assert_eq!(admit_summoner_cast(5, Some(1), Some(1)).unwrap_err(), "INVALID_SUMMONER_SLOT");
        assert_eq!(admit_summoner_cast(0, Some(1), None).unwrap_err(), "MALFORMED_SUMMONER_POINT");
        assert_eq!(admit_summoner_cast(1, None, Some(1)).unwrap_err(), "MALFORMED_SUMMONER_POINT");
        let far = SIM_TERRAIN_COORD_MAX_RAW.saturating_add(1);
        assert_eq!(admit_summoner_cast(0, Some(far), Some(0)).unwrap_err(), "SUMMONER_POINT_OUT_OF_ENVELOPE");
        assert_eq!(admit_summoner_cast(0, Some(i64::MIN), Some(0)).unwrap_err(), "SUMMONER_POINT_OUT_OF_ENVELOPE");
        let admitted = admit_summoner_cast(1, None, None).unwrap();
        assert_eq!(admitted.slot, 1);
        assert!(admitted.point.is_none());
        let point = admit_summoner_cast(0, Some(-12), Some(SIM_TERRAIN_COORD_MAX_RAW)).unwrap();
        assert_eq!(point.point.unwrap().x.raw(), -12);
    }

    #[test]
    fn owner_team_projection_roundtrip_rejects_hidden_or_negative_clocks() {
        let rows = [ProjectedSummonerPlayer {
            player_id: 7,
            slots: [
                ProjectedSummonerSlot { catalog_id: 1, cooldown_raw: 300 * 1024 },
                ProjectedSummonerSlot { catalog_id: 0, cooldown_raw: 0 },
            ],
        }];
        let encoded = encode_projected_summoners(&rows);
        assert_eq!(encoded.len(), 28);
        assert_eq!(decode_projected_summoners(&encoded).unwrap(), rows);
        assert!(decode_projected_summoners(&[]).is_none());
        assert_eq!(decode_projected_summoners(&encode_projected_summoners(&[])).unwrap(), Vec::new());
        let mut unknown = encoded.clone();
        unknown[8..10].copy_from_slice(&u16::MAX.to_be_bytes());
        assert!(decode_projected_summoners(&unknown).is_none());
        let mut negative = encoded.clone();
        negative[10] = 0xff;
        assert!(decode_projected_summoners(&negative).is_none());
        let blink = omoba_template_ids::summoner_spell_by_name("training_point_blink_v1").unwrap();
        assert_eq!(blink.cooldown.raw(), 300 * 1024);
        let unequipped_clock = encode_projected_summoners(&[ProjectedSummonerPlayer {
            player_id: 7,
            slots: [
                ProjectedSummonerSlot { catalog_id: 0, cooldown_raw: 1 },
                ProjectedSummonerSlot { catalog_id: 2, cooldown_raw: 0 },
            ],
        }]);
        assert!(decode_projected_summoners(&unequipped_clock).is_none());
        let above_total = encode_projected_summoners(&[ProjectedSummonerPlayer {
            player_id: 7,
            slots: [
                ProjectedSummonerSlot { catalog_id: 1, cooldown_raw: blink.cooldown.raw() + 1 },
                ProjectedSummonerSlot { catalog_id: 2, cooldown_raw: 0 },
            ],
        }]);
        assert!(decode_projected_summoners(&above_total).is_none());
        assert_eq!(catalog_id_for_spell(Some("training_self_heal_v1")), 2);
        assert_eq!(catalog_id_for_spell(Some("not_a_spell")), 0);
        assert_eq!(catalog_id_for_spell(None), 0);
    }
}
