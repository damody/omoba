//! Match-scoped mana agreement covers cost admission, committed fact 26 and HUD v2.
pub const MANA_PROTOCOL_VERSION: u32 = 1;

pub fn negotiate_mana_protocol(version: u32, rules_hash: &str) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() { return Ok(false); }
    if version != MANA_PROTOCOL_VERSION { return Err("MANA_PROTOCOL_VERSION_MISMATCH"); }
    if rules_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
        return Err("MANA_RULES_HASH_MISMATCH");
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mana_agreement_rejects_partial_unknown_and_stale_declarations() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_mana_protocol(0, ""), Ok(false));
        assert_eq!(negotiate_mana_protocol(MANA_PROTOCOL_VERSION, hash), Ok(true));
        for (version, hash) in [(0,hash),(2,hash),(1,""),(1,"stale")] {
            assert!(negotiate_mana_protocol(version, hash).is_err());
        }
    }
}
