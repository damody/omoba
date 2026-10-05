//! Versioned transport admission for newly introduced formal hero commands.
//! Version 1 includes HoldPosition; it does not enable shops or Recall.
pub const COMMAND_PROTOCOL_VERSION: u32 = 1;

pub fn negotiate_command_protocol(version: u32, rules_hash: &str) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() { return Ok(false); }
    if version != COMMAND_PROTOCOL_VERSION { return Err("COMMAND_PROTOCOL_VERSION_MISMATCH"); }
    if rules_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
        return Err("COMMAND_RULES_HASH_MISMATCH");
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_capability_is_closed_for_legacy_and_rejects_partial_agreement() {
        let hash=omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_command_protocol(0,""),Ok(false));
        assert_eq!(negotiate_command_protocol(COMMAND_PROTOCOL_VERSION,hash),Ok(true));
        for (version,value) in [(0,hash),(2,hash),(1,""),(1,"old")] {
            assert!(negotiate_command_protocol(version,value).is_err());
        }
    }
}
