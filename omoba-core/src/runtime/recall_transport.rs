//! Recall opt-in is independent of shop capability and gameplay admission.
// Version 2 requires player-scoped Recall/respawn HUD metrics.
pub const RECALL_PROTOCOL_VERSION: u32 = 2;

pub fn negotiate_recall_protocol(version: u32, rules_hash: &str) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() { return Ok(false); }
    if version != RECALL_PROTOCOL_VERSION { return Err("RECALL_PROTOCOL_VERSION_MISMATCH"); }
    if rules_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
        return Err("RECALL_RULES_HASH_MISMATCH");
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_capability_is_closed_for_legacy_and_mismatches() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_recall_protocol(0, ""), Ok(false));
        assert_eq!(negotiate_recall_protocol(RECALL_PROTOCOL_VERSION, hash), Ok(true));
        for (version, value) in [(0, hash), (1, hash), (3, hash), (RECALL_PROTOCOL_VERSION, ""), (RECALL_PROTOCOL_VERSION, "old")] {
            assert!(negotiate_recall_protocol(version, value).is_err());
        }
    }
}
