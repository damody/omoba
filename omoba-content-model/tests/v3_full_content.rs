//! Full v3 content (scripts/gen_v3_content.lua): every history_nobu hero and shop item, listed or
//! not, loads as content-model V3Block and passes validate_block. Run through
//! `tools\lua\lua.exe scripts\check_v3_content.lua`, which writes the file and sets OM_V3_CONTENT.
//! Without OM_V3_CONTENT the test checks the default file when it exists and skips otherwise.
//! Until V3Block has an `ir` field (M3 ir.rs), each block's `ir` is taken off before loading and
//! only counted; once the field exists the block loads with it.
use omoba_content_model::v3::{validate_block, V3Block};
use std::path::PathBuf;

fn content_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OM_V3_CONTENT") {
        return Some(PathBuf::from(p));
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/v3-content/v3-content.json");
    p.is_file().then_some(p)
}

#[test]
fn every_v3_block_loads() {
    let Some(path) = content_path() else {
        eprintln!("skip: no v3 content (run tools\\lua\\lua.exe scripts\\check_v3_content.lua)");
        return;
    };
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let content: serde_json::Value = serde_json::from_str(&text).expect("v3 content is JSON");
    assert_eq!(content["kind"], "full", "kind = full");
    let heroes = content["heroes"].as_array().expect("heroes");
    assert!(heroes.len() >= 97, "every history_nobu hero, got {}", heroes.len());
    let listed = heroes.iter().filter(|h| h["listed"] == true).count();
    assert!(listed >= 1 && listed < heroes.len(), "listed and unlisted heroes ({listed} listed)");
    let mut loaded = 0usize;
    for (key, kind) in [("abilities", "ability"), ("items", "item")] {
        let blocks = content[key].as_object().unwrap_or_else(|| panic!("{key} is a map"));
        for (id, value) in blocks {
            // M3' ④: no behavior IR in the catalog; generated Rust runs the behavior.
            assert!(value.get("ir").is_none(), "{key}.{id}: the catalog carries no IR");
            let block: V3Block = serde_json::from_value::<V3Block>(value.clone()).unwrap_or_else(|e| panic!("{key}.{id}: {e}"));
            validate_block(&block, kind, block.max_level).unwrap_or_else(|e| panic!("{key}.{id}: {e}"));
            loaded += 1;
        }
    }
    for h in heroes {
        for a in h["abilities"].as_array().expect("hero abilities") {
            let id = a.as_str().expect("ability id");
            assert!(content["abilities"].get(id).is_some(), "hero {} names missing ability {id}", h["source_id"]);
        }
    }
    eprintln!("v3 content: {loaded} blocks loaded");
}
