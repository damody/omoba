//! Generates every shop item from the item documents before every build
//! (docs/character_pipeline/tools/v3_rust.lua --items --single $OUT_DIR/generated.rs), like the
//! hero crates' build.rs: checkouts, land_to_live and hot reloads never compile stale code.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // The repository root: the first ancestor with the pinned Lua and the generator (also from a shadow tree).
    let root = crate_dir.ancestors()
        .find(|p| p.join("tools/lua/lua.exe").is_file() && p.join("docs/character_pipeline/tools/v3_rust.lua").is_file())
        .expect("repository root with tools/lua/lua.exe")
        .to_path_buf();
    for path in [
        "docs/character_pipeline/tools",
        "docs/character_pipeline/schema",
        "docs/character_pipeline/items",
        "omoba-core/src/runtime/native/rules/vocab.rs",
        "scripts/lua_data/templates/game_rules.lua",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("generated.rs");
    let output = Command::new(root.join("tools/lua/lua.exe"))
        .current_dir(&root)
        .arg("docs/character_pipeline/tools/v3_rust.lua")
        .arg("--items")
        .arg("--single").arg(&out)
        .output()
        .unwrap_or_else(|error| panic!("run v3_rust.lua: {error}"));
    if !output.status.success() {
        panic!("v3_rust.lua failed: {}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    }
}
