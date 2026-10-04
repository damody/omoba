//! Transport catalog agreement. Not a gameplay resource or purchase capability.
pub const SHOP_CATALOG_VERSION: u32 = 1;
/// Explicit opt-in to owner-scoped transactions and read-only result recovery.
pub const SHOP_PROTOCOL_VERSION: u32 = 1;

pub fn item_id_for_catalog(catalog_id: u32) -> Option<&'static str> {
    let id = u16::try_from(catalog_id).ok()?;
    omoba_template_ids::MOBA_ITEM_CATALOG
        .iter()
        .find(|item| item.catalog_id == id)
        .map(|item| item.id)
}

pub fn negotiate_shop_protocol(
    version: u32,
    rules_hash: &str,
    catalog_agreed: bool,
) -> Result<bool, &'static str> {
    if version == 0 && rules_hash.is_empty() {
        return Ok(false);
    }
    if version != SHOP_PROTOCOL_VERSION {
        return Err("SHOP_PROTOCOL_VERSION_MISMATCH");
    }
    if !catalog_agreed || rules_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
        return Err("SHOP_RULES_HASH_MISMATCH");
    }
    Ok(true)
}

/// Transport-only budget. Caller supplies monotonic session elapsed time;
/// pausing gameplay does not freeze recovery. No client timestamp is accepted.
#[derive(Default)]
pub struct ShopQueryBudget {
    started_ms: u64,
    used: u32,
}
impl ShopQueryBudget {
    pub fn allow(&mut self, now_ms: u64) -> bool {
        if now_ms < self.started_ms {
            return false;
        }
        if now_ms - self.started_ms >= 1000 {
            self.started_ms = now_ms;
            self.used = 0;
        }
        if self.used >= 8 {
            return false;
        }
        self.used += 1;
        true
    }
}

/// Validate recovery without advancing, repairing, or replaying gameplay.
pub fn validate_receipt_replay(
    msg: &crate::game_proto::ShopReceiptReplay,
    player: u32,
) -> Result<Option<crate::runtime::shop_receipt::ShopReceipt>, &'static str> {
    if msg.schema_version != 1
        || player == 0
        || msg.player_id != player
        || msg.request_id == 0
        || msg.input_id == 0
    {
        return Err("INVALID_SHOP_REPLAY_IDENTITY");
    }
    if msg.status > 4 {
        return Err("INVALID_SHOP_REPLAY_STATUS");
    }
    if msg.status != 2 {
        return if msg.receipt.is_empty() {
            Ok(None)
        } else {
            Err("NONTERMINAL_SHOP_REPLAY_PAYLOAD")
        };
    }
    let receipt = crate::runtime::shop_receipt::ShopReceipt::decode(&msg.receipt)
        .ok_or("INVALID_SHOP_REPLAY_RECEIPT")?;
    if receipt.player_id != player || receipt.input_id != u64::from(msg.input_id) {
        return Err("CONFLICTING_SHOP_REPLAY_RECEIPT");
    }
    Ok(Some(receipt))
}

pub fn negotiate_shop_catalog(version: u32, hash: &str) -> Result<bool, &'static str> {
    if version == 0 && hash.is_empty() {
        return Ok(false);
    }
    if version != SHOP_CATALOG_VERSION {
        return Err("SHOP_CATALOG_VERSION_MISMATCH");
    }
    if hash != omoba_template_ids::MOBA_ITEM_CATALOG_HASH {
        return Err("SHOP_CATALOG_HASH_MISMATCH");
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_requires_catalog_full_rules_and_query_budget_is_independent() {
        let hash = omoba_template_ids::CONTENT_CATALOG_DATA_HASH;
        assert_eq!(negotiate_shop_protocol(0, "", false), Ok(false));
        assert_eq!(negotiate_shop_protocol(1, hash, true), Ok(true));
        for (version, hash, catalog) in [
            (0, hash, true),
            (2, hash, true),
            (1, "old", true),
            (1, hash, false),
        ] {
            assert!(negotiate_shop_protocol(version, hash, catalog).is_err());
        }
        let mut budget = ShopQueryBudget::default();
        for _ in 0..8 {
            assert!(budget.allow(0));
        }
        assert!(!budget.allow(999));
        assert!(budget.allow(1000));
        assert!(!budget.allow(1));
        assert_eq!(item_id_for_catalog(1), Some("moba_sword"));
        assert_eq!(item_id_for_catalog(u32::MAX), None);
    }
    #[test]
    fn receipt_recovery_identity_schema_status_and_payload_fail_closed() {
        use crate::game_proto::ShopReceiptReplay;
        use crate::runtime::shop_receipt::ShopReceipt;
        let receipt = ShopReceipt {
            player_id: 7,
            input_id: 42,
            tick: 10,
            action_kind: 17,
            catalog_id: 1,
            slot: 0,
            result_code: 6,
        };
        let mut reply = ShopReceiptReplay {
            schema_version: 1,
            request_id: 3,
            player_id: 7,
            input_id: 42,
            status: 2,
            receipt: receipt.encode(),
        };
        assert_eq!(validate_receipt_replay(&reply, 7), Ok(Some(receipt)));
        assert!(validate_receipt_replay(&reply, 2).is_err());
        reply.input_id = 43;
        assert!(validate_receipt_replay(&reply, 7).is_err());
        reply.input_id = 42;
        for status in [0, 1, 3, 4, 5] {
            reply.status = status;
            assert!(validate_receipt_replay(&reply, 7).is_err());
        }
        reply.receipt.clear();
        for status in [0, 1, 3, 4] {
            reply.status = status;
            assert_eq!(validate_receipt_replay(&reply, 7), Ok(None));
        }
        reply.status = 2;
        assert!(validate_receipt_replay(&reply, 7).is_err());
        reply.status = 0;
        reply.schema_version = 0;
        assert!(validate_receipt_replay(&reply, 7).is_err());
    }
    #[test]
    fn catalog_agreement_fails_closed_without_disabling_legacy_clients() {
        assert_eq!(negotiate_shop_catalog(0, ""), Ok(false));
        assert_eq!(
            negotiate_shop_catalog(
                SHOP_CATALOG_VERSION,
                omoba_template_ids::MOBA_ITEM_CATALOG_HASH
            ),
            Ok(true)
        );
        for (version, hash) in [
            (0, "old"),
            (1, ""),
            (1, "old"),
            (2, omoba_template_ids::MOBA_ITEM_CATALOG_HASH),
        ] {
            assert!(negotiate_shop_catalog(version, hash).is_err());
        }
    }
}
