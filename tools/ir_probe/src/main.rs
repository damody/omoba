//! IR load probe (docs/plans/2026-10-28-history-nobu-full-port/ir-blockers.md): for every block of the
//! full v3 content (scripts/gen_v3_content.lua) that carries behavior IR, loads it with omoba-core's
//! `IrAbility::from_block` and prints one JSON line `{"id", "kind", "ok", "error"}`. Read-only; the
//! interpreter decides what it runs, this only reports it. `@tether` gets a generic runtime id so a
//! tether is judged by its shape, not by a missing switch-table entry.
use omoba_content_model::v3::V3Block;
use omoba_core::runtime::native::ir::IrAbility;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "target/v3-content/v3-content.json".into());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let content: serde_json::Value = serde_json::from_str(&text).expect("v3 content is JSON");
    for (key, kind) in [("abilities", "ability"), ("items", "item")] {
        let Some(blocks) = content[key].as_object() else { continue };
        for (id, value) in blocks {
            if value.get("ir").is_none() { continue; }
            let line = match serde_json::from_value::<V3Block>(value.clone()) {
                Err(e) => serde_json::json!({ "id": id, "kind": kind, "ok": false, "error": format!("V3Block: {e}") }),
                Ok(block) => {
                    let tether = format!("{id}:{{caster}}");
                    match IrAbility::from_block(id, &block, &[("@tether", tether.as_str())]) {
                        Ok(_) => serde_json::json!({ "id": id, "kind": kind, "ok": true }),
                        Err(e) => serde_json::json!({ "id": id, "kind": kind, "ok": false, "error": e }),
                    }
                }
            };
            println!("{line}");
        }
    }
}
