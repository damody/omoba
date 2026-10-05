use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scripts_dir = crate_dir.parent().expect("base_content parent");
    let repo_root = scripts_dir.parent().expect("scripts parent");
    let content_root = scripts_dir.join("lua_data");
    let generator = scripts_dir.join("gen_hero_registry.lua");
    let lua = repo_root.join("tools/lua/lua.exe");
    for path in [
        &generator,
        // Includes may add arbitrary shared template files. Watching the full
        // template directory prevents stale FFI registrations after edits.
        &content_root.join("templates"),
        &content_root.join("templates/heroes.lua"),
        &content_root.join("templates/abilities.lua"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    for (mode, file) in [
        ("registry", "hero_ability_registry.rs"),
        ("ids", "hero_ability_ids.rs"),
    ] {
        let output = Command::new(&lua)
            .arg(&generator)
            .arg(&content_root)
            .arg(mode)
            .output()
            .unwrap_or_else(|error| panic!("run {}: {error}", lua.display()));
        if !output.status.success() {
            panic!(
                "hero registry generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        fs::write(out_dir.join(file), output.stdout)
            .unwrap_or_else(|error| panic!("write generated {file}: {error}"));
    }
}
