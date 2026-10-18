//! Versioned transport admission for newly introduced formal hero commands.
//! Version 2 includes immediate Stop and persistent HoldPosition.
//! Version 1 admitted Hold only. It is rejected so it cannot advertise Stop.
//! Version 0 with an empty hash remains legacy and leaves commands disabled.
//! Shops and Recall stay on their own protocols.
pub const COMMAND_PROTOCOL_VERSION: u32 = 2;

/// True only for the negotiated version whose command set is Hold and Stop.
pub fn command_protocol_includes_stop_and_hold(version: u32) -> bool {
    version == COMMAND_PROTOCOL_VERSION
}

/// Name of the command set enabled by a successful negotiation. Legacy stays `none`.
pub fn agreed_command_set(enabled: bool) -> &'static str {
    if enabled { "hold+stop" } else { "none" }
}

pub fn negotiate_command_protocol(version: u32, rules_hash: &str) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() { return Ok(false); }
    // Version 1 must fail closed before the current-version check. Accepting it
    // would turn a Hold-only peer into a session that can submit Stop.
    if version == 1 { return Err("COMMAND_PROTOCOL_VERSION_MISMATCH"); }
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
        assert_eq!(agreed_command_set(true), "hold+stop");
        for (version,value) in [(0,hash),(1,hash),(3,hash),(2,""),(2,"old")] {
            assert!(negotiate_command_protocol(version,value).is_err());
        }
    }

    #[test]
    fn stop_command_version_two_includes_hold_and_stop_and_rejects_v1() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(COMMAND_PROTOCOL_VERSION, 2);
        assert!(command_protocol_includes_stop_and_hold(2));
        assert!(!command_protocol_includes_stop_and_hold(1));
        assert!(!command_protocol_includes_stop_and_hold(0));
        assert_eq!(negotiate_command_protocol(2, hash), Ok(true));
        assert_eq!(agreed_command_set(true), "hold+stop");
        assert_eq!(agreed_command_set(false), "none");
        assert_eq!(negotiate_command_protocol(1, hash), Err("COMMAND_PROTOCOL_VERSION_MISMATCH"));
        assert_eq!(negotiate_command_protocol(2, "old"), Err("COMMAND_RULES_HASH_MISMATCH"));
        assert_eq!(negotiate_command_protocol(0, ""), Ok(false));
        assert_eq!(negotiate_command_protocol(0, hash), Err("COMMAND_PROTOCOL_VERSION_MISMATCH"));
    }
}
