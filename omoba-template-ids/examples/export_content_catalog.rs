//! Writes the v3 content catalog (V3_CATALOG_JSON: content hash, heroes, ability and item
//! v3 blocks) for the website generator and other offline readers.
//!
//!   cargo run -p omoba-template-ids --example export_content_catalog -- <out.json>
//!
//! Without an argument it prints the JSON to stdout.
fn main() {
    let json = omoba_template_ids::V3_CATALOG_JSON;
    match std::env::args().nth(1) {
        Some(path) => {
            std::fs::write(&path, json).unwrap_or_else(|e| panic!("write {path}: {e}"));
            eprintln!("wrote {path} ({} bytes, content {})", json.len(), omoba_template_ids::CONTENT_CATALOG_DATA_HASH);
        }
        None => println!("{json}"),
    }
}
