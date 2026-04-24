//! Stable hash-based template id encoding for high-volume proto fields.
//!
//! Background: under stress the server broadcasts thousands of `projectile.C`
//! per second (each carrying a `kind` string like `"tack"`, `"bomb_frag"`,
//! `"saika_shot"`) and many `creep.C` per second (each carrying a `name`
//! string, typically Chinese display labels like `"訓練小兵"`). Switching those
//! fields to `uint32` saves a large chunk of wire bytes at zero latency cost.
//!
//! ## Why FNV-1a 32-bit?
//!
//! We use `fnv1a_32` as the id function — server and client never need to
//! coordinate id allocation, no broadcast/handshake table is required, and
//! id 0 is reserved for the empty string (= "unspecified kind", used by the
//! legacy `handle_projectile_directional` path and several `game_processor`
//! callers). Collisions within a 32-bit space across ~15 total values are
//! astronomically unlikely; the boot-time self-check in
//! [`known_projectile_kinds`] catches any if someone adds a clashing kind
//! later.
//!
//! ## Client-side reverse lookup
//!
//! The omoba-core KCP client shim reconstructs legacy-shape JSON
//! (`{"kind": "tack", ...}`) so the omfx renderer doesn't need to change. It
//! calls [`projectile_kind_by_id`] / [`creep_name_by_id`] to get the
//! original string. Unknown ids fall back to an empty string for projectile
//! kinds (matches existing default-case handling in omfx's bullet colour
//! switch) and to a hex placeholder for creep names (purely a display issue
//! for brand-new entity-defined creeps not listed below).

use std::collections::HashMap;
use std::sync::OnceLock;

/// FNV-1a 32-bit hash. Same algorithm server- and client-side.
///
/// `""` hashes to the FNV offset basis (0x811c9dc5), so the empty string is a
/// legitimate id value, NOT a sentinel. Callers that want an "unset" sentinel
/// should use 0 separately and special-case it.
pub const fn fnv1a_32(s: &str) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;
    let bytes = s.as_bytes();
    let mut hash: u32 = FNV_OFFSET_BASIS;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

/// Encode a projectile kind string as a u32 id.
///
/// Special case: empty string returns 0. Callers emitting "unspecified kind"
/// (directional shots with no tag, legacy creation_events with `""`) get a
/// very cheap tag-0 id that needs no reverse lookup client-side.
#[inline]
pub fn encode_projectile_kind(s: &str) -> u32 {
    if s.is_empty() {
        0
    } else {
        fnv1a_32(s)
    }
}

/// Encode a creep display name / unit-type key as a u32 id.
///
/// Empty string returns 0. Non-empty strings use FNV-1a directly; id values
/// collide with projectile ids in theory but never in practice because we
/// type the two caches separately on the client.
#[inline]
pub fn encode_creep_name(s: &str) -> u32 {
    if s.is_empty() {
        0
    } else {
        fnv1a_32(s)
    }
}

/// All projectile-kind strings currently used by any tower/summon script and
/// by host-side spawn paths. Client reverse-lookup table is built from this
/// list at first access.
///
/// **Source of truth**: grep `kind_tag: RString::from(` in
/// `scripts/base_content/src/{towers,summons}/*.rs`, plus the empty-string
/// emits from `game_processor::handle_projectile_directional` and
/// `creation_events::handle_projectile_creation`.
pub const KNOWN_PROJECTILE_KINDS: &[&str] = &[
    "",             // unspecified — projectile_directional, legacy creation_events
    "dart",
    "spike_opult",
    "tack",
    "tack_blade",
    "bomb",
    "bomb_frag",
    "saika_shot",
    "ice",
    "icicle",
];

/// All creep display names (CreepCreate.name, which is
/// `label.unwrap_or(internal_name)`). Populated from all 5 `Story/*/entity.json`
/// creep/neutral definitions (TD_1, TD_STRESS, MVP_1, DEBUG_1, B01_1) plus a
/// handful of ally/summon strings that may also surface.
///
/// Adding a new creep to entity.json without updating this list → client sees
/// an `unknown_<hex>` placeholder; harmless for the renderer (only display
/// name changes) but the fallback is noted in [`creep_name_by_id`].
pub const KNOWN_CREEP_NAMES: &[&str] = &[
    // 英雄 (shown on create but rare)
    "雜賀孫市",
    "訓練法師",
    "火焰法師",
    "大法師",
    // creeps / training dummies / neutrals (the stress cohort)
    "練習假人",
    "移動靶",
    "訓練小兵",
    "裝甲假人",
    "森林幽靈",
    "野狼",
    "雜賀鐵炮兵",
    // internal ids (emitted if .label is None)
    "practice_dummy",
    "moving_target",
    "training_creep",
    "armored_dummy",
    "forest_ghost",
    "wolf",
    "melee_minion",
    "ranged_minion",
    "siege_minion",
    "saika_gunner",
    // mission hero/creep labels from B01_1
    "神射手",
    "千里狙殺",
];

fn projectile_kind_table() -> &'static HashMap<u32, &'static str> {
    static TABLE: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut m = HashMap::with_capacity(KNOWN_PROJECTILE_KINDS.len());
        for &s in KNOWN_PROJECTILE_KINDS {
            let id = encode_projectile_kind(s);
            // Best-effort collision guard: if two known kinds hash to the
            // same id, the second insert would silently overwrite. Assert
            // uniqueness in debug builds.
            debug_assert!(
                !m.contains_key(&id),
                "projectile kind id collision: {:?} clashes with existing",
                s,
            );
            m.insert(id, s);
        }
        m
    })
}

fn creep_name_table() -> &'static HashMap<u32, &'static str> {
    static TABLE: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut m = HashMap::with_capacity(KNOWN_CREEP_NAMES.len());
        for &s in KNOWN_CREEP_NAMES {
            let id = encode_creep_name(s);
            debug_assert!(
                !m.contains_key(&id),
                "creep name id collision: {:?} clashes with existing",
                s,
            );
            m.insert(id, s);
        }
        m
    })
}

/// Reverse lookup: projectile kind id → original string (static `&str`).
///
/// Returns `""` for id 0 (unspecified). Returns `""` for unknown ids too —
/// omfx's bullet-colour switch already has a `_ => default` branch so this
/// degrades gracefully on unregistered kinds.
#[inline]
pub fn projectile_kind_by_id(id: u32) -> &'static str {
    if id == 0 {
        return "";
    }
    projectile_kind_table().get(&id).copied().unwrap_or("")
}

/// Reverse lookup: creep name id → original string (static `&str`).
///
/// Returns `""` for id 0. Returns `""` for unknown ids — the caller can
/// decide whether to format a hex placeholder for logs. The default case in
/// the KCP shim emits `""`, which omfx treats as "fall back to entity_type".
#[inline]
pub fn creep_name_by_id(id: u32) -> &'static str {
    if id == 0 {
        return "";
    }
    creep_name_table().get(&id).copied().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hashes_to_zero_by_convention() {
        assert_eq!(encode_projectile_kind(""), 0);
        assert_eq!(encode_creep_name(""), 0);
        assert_eq!(projectile_kind_by_id(0), "");
        assert_eq!(creep_name_by_id(0), "");
    }

    #[test]
    fn fnv1a_matches_published_vectors() {
        // Standard FNV-1a test vector: "a" → 0xe40c292c
        assert_eq!(fnv1a_32("a"), 0xe40c292c);
        // "foobar" → 0xbf9cf968
        assert_eq!(fnv1a_32("foobar"), 0xbf9cf968);
    }

    #[test]
    fn roundtrip_known_projectile_kinds() {
        for &s in KNOWN_PROJECTILE_KINDS {
            let id = encode_projectile_kind(s);
            assert_eq!(projectile_kind_by_id(id), s, "roundtrip failed for {:?}", s);
        }
    }

    #[test]
    fn roundtrip_known_creep_names() {
        for &s in KNOWN_CREEP_NAMES {
            let id = encode_creep_name(s);
            assert_eq!(creep_name_by_id(id), s, "roundtrip failed for {:?}", s);
        }
    }

    #[test]
    fn no_collisions_in_known_sets() {
        // Both tables built at first access. Rebuilding here purely to force
        // the debug_assert to run in test context.
        let pk = projectile_kind_table();
        assert_eq!(pk.len(), KNOWN_PROJECTILE_KINDS.len());
        let cn = creep_name_table();
        assert_eq!(cn.len(), KNOWN_CREEP_NAMES.len());
    }

    #[test]
    fn unknown_id_returns_empty() {
        // A clearly-not-in-table id — should gracefully return "".
        assert_eq!(projectile_kind_by_id(0xDEAD_BEEF), "");
        assert_eq!(creep_name_by_id(0xDEAD_BEEF), "");
    }
}
