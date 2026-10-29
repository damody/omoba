//! Generates this hero's abilities from its v3 document before every build
//! (docs/character_pipeline/tools/v3_rust.lua --single $OUT_DIR/generated.rs), like base_content's
//! build.rs runs gen_hero_registry.lua: checkouts, land_to_live and hot reloads (cargo only) never
//! compile stale code, and a value change rebuilds only this hero.
use std::env;
use std::path::PathBuf;
use std::process::Command;

const PACKAGE: &str = "tamako";

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
    println!("cargo:rerun-if-changed={}", root.join("docs/character_pipeline/packages").join(PACKAGE).display());
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("generated.rs");
    let output = Command::new(root.join("tools/lua/lua.exe"))
        .current_dir(&root)
        .arg("docs/character_pipeline/tools/v3_rust.lua")
        .arg("--hero").arg(PACKAGE)
        .arg("--single").arg(&out)
        .output()
        .unwrap_or_else(|error| panic!("run v3_rust.lua: {error}"));
    if !output.status.success() {
        panic!("v3_rust.lua failed: {}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    }
}
