#[cfg(feature = "runtime-lua-content")]
#[test]
fn compiled_map_hash_covers_layout_and_unlock_rule() {
    use omoba_content_model::canonical_template_hash;
    use omoba_template_ids::{MOBA_MAP_CATALOG_HASH,MOBA_MAP_CATALOG_JSON};
    let original: serde_json::Value = serde_json::from_str(MOBA_MAP_CATALOG_JSON).unwrap();
    assert_eq!(canonical_template_hash(&original).unwrap(),MOBA_MAP_CATALOG_HASH);
    for (field,value) in [("id",serde_json::json!("changed")),
        ("tower_offset",serde_json::json!(701)),("base_unlock",serde_json::json!("changed")),
        ("lanes",serde_json::json!([]))] {
        let mut changed = original.clone(); changed[0][field] = value;
        assert_ne!(canonical_template_hash(&changed).unwrap(),MOBA_MAP_CATALOG_HASH);
    }
}

#[cfg(feature = "runtime-lua-content")]
#[test]
fn compiled_shop_hash_covers_prices_ids_names_and_recipes() {
    use omoba_content_model::canonical_template_hash;
    use omoba_template_ids::{MOBA_ITEM_CATALOG_HASH, MOBA_ITEM_CATALOG_JSON};
    let original: serde_json::Value = serde_json::from_str(MOBA_ITEM_CATALOG_JSON).unwrap();
    assert_eq!(
        canonical_template_hash(&original).unwrap(),
        MOBA_ITEM_CATALOG_HASH
    );
    for (field, value) in [
        ("cost", serde_json::json!(99999)),
        ("catalog_id", serde_json::json!(999)),
        ("name", serde_json::json!("changed")),
        ("recipe", serde_json::json!(["changed"])),
    ] {
        let mut changed = original.clone();
        changed[0][field] = value;
        assert_ne!(
            canonical_template_hash(&changed).unwrap(),
            MOBA_ITEM_CATALOG_HASH
        );
    }
}
