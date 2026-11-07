//! 載入腳本 DLL（base_content 與每位英雄的 hero_<id>），每個都是一個 `Manifest_Ref`。
//!
//! M3'（docs/plans/2026-10-28-history-nobu-full-port/m3-rust-codegen-design.md §3）：
//! - abi_stable 的 `RootModule::load_from_file` 依型別快取第一個載入的 root module，同一型別的
//!   第二個 DLL 會拿回第一個的 manifest。這裡改用 `lib_header_from_path` 逐檔載入，再以
//!   `init_root_module` 檢查版本與 layout；不經過 per-type 快取。
//! - 目錄有 `scripts-manifest.json`（launch 寫的，run_artifacts.lua）時，只依它的順序載入列出的
//!   檔案，並核對每個檔案的 SHA-256；沒有時照舊掃描目錄（依檔名排序）。
//! - 每個 DLL 的內容標記（`content_marker`）必須等於 host 自己的 catalog，否則拒絕。
//! - 載入的整組模組記成 `script_modules()`／`script_modules_hash()`，握手與重播檔用它比對。
//! 驗證失敗時 panic：用錯的 DLL 開局比不開局更糟（fail closed）。

use abi_stable::library::lib_header_from_path;
use omb_script_abi::manifest::Manifest_Ref;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::runtime::scripting::registry::ScriptRegistry;

#[cfg(target_os = "windows")]
const DLL_EXT: &str = "dll";
#[cfg(target_os = "linux")]
const DLL_EXT: &str = "so";
#[cfg(target_os = "macos")]
const DLL_EXT: &str = "dylib";

/// The manifest a launch writes beside the DLLs it staged.
pub const SCRIPTS_MANIFEST: &str = "scripts-manifest.json";

/// One loaded script DLL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptModule {
    pub module: String,
    pub file: PathBuf,
    pub sha256: String,
}

static LOADED: RwLock<Vec<ScriptModule>> = RwLock::new(Vec::new());

/// The script DLLs this process loaded last, in load order.
pub fn script_modules() -> Vec<ScriptModule> {
    LOADED.read().map(|modules| modules.clone()).unwrap_or_default()
}

/// SHA-256 over `module=sha256` lines of the loaded set; empty when no DLL was loaded (tests
/// that register manifests in-process). The authority and its replicas must agree on it.
pub fn script_modules_hash() -> String {
    modules_hash(&script_modules())
}

/// The set hash a joining client declares: the loaded set, or, before the replica has loaded
/// its scripts (it loads them on GameStart, after the join), the set `OMB_SCRIPTS_DIR` would
/// load. A directory that cannot be read declares an empty set (the authority then refuses).
pub fn declared_script_modules_hash() -> String {
    let loaded = script_modules();
    if !loaded.is_empty() {
        return modules_hash(&loaded);
    }
    let dir = std::env::var("OMB_SCRIPTS_DIR").unwrap_or_else(|_| "./scripts".into());
    let Ok(planned) = planned_modules(Path::new(&dir)) else { return String::new() };
    let mut modules = Vec::new();
    for (expected, path) in planned {
        let Ok(sha256) = file_sha256(&path) else { return String::new() };
        let module = match expected {
            Some((module, _)) => module,
            None => match load_module(&path) {
                Ok(manifest) => (manifest.module_id())().as_str().to_owned(),
                Err(_) => continue,
            },
        };
        modules.push(ScriptModule { module, file: path, sha256 });
    }
    modules_hash(&modules)
}

pub fn modules_hash(modules: &[ScriptModule]) -> String {
    if modules.is_empty() {
        return String::new();
    }
    let mut hasher = Sha256::new();
    for module in modules {
        hasher.update(format!("{}={}\n", module.module, module.sha256).as_bytes());
    }
    hex(&hasher.finalize())
}

/// Replaces one module's entry (a dev hot reload swapped it); the set hash changes with it.
pub fn record_module_swap(module: ScriptModule) {
    if let Ok(mut loaded) = LOADED.write() {
        match loaded.iter_mut().find(|entry| entry.module == module.module) {
            Some(entry) => *entry = module,
            None => loaded.push(module),
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(hex(&Sha256::digest(&bytes)))
}

/// 載入 `dir` 下的腳本 DLL（見模組說明）。目錄不存在時回傳空的註冊表。
pub fn load_scripts_dir(dir: &Path) -> ScriptRegistry {
    let mut reg = ScriptRegistry::new();

    if !dir.is_dir() {
        log::warn!(
            "[scripting] scripts directory {:?} does not exist — running with zero scripts",
            dir
        );
        return reg;
    }

    let planned = match planned_modules(dir) {
        Ok(planned) => planned,
        Err(error) => panic!("[scripting] {error}"),
    };
    let mut loaded = Vec::new();
    for (expected, path) in planned {
        let sha256 = file_sha256(&path).unwrap_or_else(|error| panic!("[scripting] {error}"));
        if let Some((_, expected_sha)) = &expected {
            if !expected_sha.eq_ignore_ascii_case(&sha256) {
                panic!(
                    "[scripting] {} does not match {SCRIPTS_MANIFEST}: sha256 {sha256}, expected {expected_sha}",
                    path.display()
                );
            }
        }
        let manifest = match load_module(&path) {
            Ok(manifest) => manifest,
            // A scanned directory may hold other DLLs (a UE plugin's om_bridge.dll): not script
            // modules, skipped as before. Listed modules and content mismatches fail closed.
            Err(LoadError::NotScript(error)) if expected.is_none() => {
                log::error!("[scripting] skipped {error}");
                continue;
            }
            Err(LoadError::NotScript(error) | LoadError::Content(error)) => panic!("[scripting] {error}"),
        };
        let module: String = (manifest.module_id())().as_str().to_owned();
        if let Some((expected_module, _)) = &expected {
            if *expected_module != module {
                panic!("[scripting] {} is module `{module}`, {SCRIPTS_MANIFEST} says `{expected_module}`", path.display());
            }
        }
        if loaded.iter().any(|entry: &ScriptModule| entry.module == module) {
            panic!("[scripting] module `{module}` loaded twice ({})", path.display());
        }
        log::info!("[scripting] loaded {module} from {:?} sha256={sha256}", path.file_name().unwrap_or_default());
        reg.insert_manifest(manifest);
        loaded.push(ScriptModule { module, file: path, sha256 });
    }
    log::info!(
        "[scripting] registry ready — {} modules (set {}), {} unit scripts: {:?}",
        loaded.len(),
        modules_hash(&loaded),
        reg.len(),
        reg.keys().collect::<Vec<_>>()
    );
    if let Ok(mut current) = LOADED.write() {
        *current = loaded;
    }
    reg
}

/// The DLLs to load: from `scripts-manifest.json` (with their expected module and SHA-256), or
/// every DLL of the directory sorted by name.
fn planned_modules(dir: &Path) -> Result<Vec<(Option<(String, String)>, PathBuf)>, String> {
    let manifest = dir.join(SCRIPTS_MANIFEST);
    if manifest.is_file() {
        let text = std::fs::read_to_string(&manifest).map_err(|error| format!("read {}: {error}", manifest.display()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", manifest.display()))?;
        let modules = value["modules"].as_array().ok_or_else(|| format!("{}: no modules", manifest.display()))?;
        let mut planned = Vec::new();
        for row in modules {
            let field = |name: &str| {
                row[name].as_str().map(str::to_owned).ok_or_else(|| format!("{}: module row without {name}", manifest.display()))
            };
            let file = field("file")?;
            if file.contains(['/', '\\']) {
                return Err(format!("{}: {file} is not a file of this directory", manifest.display()));
            }
            planned.push((Some((field("module")?, field("sha256")?)), dir.join(file)));
        }
        return Ok(planned);
    }
    let entries = std::fs::read_dir(dir).map_err(|error| format!("read_dir({}): {error}", dir.display()))?;
    let mut candidates: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some(DLL_EXT))
        .collect();
    candidates.sort();
    Ok(candidates.into_iter().map(|path| (None, path)).collect())
}

#[derive(Debug)]
pub enum LoadError {
    /// Not a loadable script module (no root module, other abi_stable version or layout).
    NotScript(String),
    /// A script module built for other content.
    Content(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::NotScript(error) | LoadError::Content(error) => f.write_str(error),
        }
    }
}

fn open_module(path: &Path) -> Result<Manifest_Ref, LoadError> {
    let header = lib_header_from_path(path).map_err(|error| LoadError::NotScript(format!("{}: {error}", path.display())))?;
    header
        .init_root_module::<Manifest_Ref>()
        .map_err(|error| LoadError::NotScript(format!("{}: {error}", path.display())))
}

/// M3' ② dev hot reload: loads a rebuilt script DLL (a new file name: the same path would map
/// the already loaded one again). Only the catalog identity (hero and ability ids) has to match
/// this host: the rebuild exists to change values, which changes the data hash.
pub fn load_hot_module(path: &Path) -> Result<Manifest_Ref, LoadError> {
    let manifest = open_module(path)?;
    let identity = (manifest.content_identity())();
    if identity.as_str() != omoba_template_ids::CONTENT_CATALOG_IDENTITY_HASH {
        return Err(LoadError::Content(format!(
            "{} has catalog identity {identity}, this host {}: hero or ability ids changed; restart instead",
            path.display(),
            omoba_template_ids::CONTENT_CATALOG_IDENTITY_HASH
        )));
    }
    Ok(manifest)
}

/// Loads one script DLL as its own root module and checks it carries this host's content.
pub fn load_module(path: &Path) -> Result<Manifest_Ref, LoadError> {
    let manifest = open_module(path)?;
    let marker = (manifest.content_marker())();
    if marker.as_str() != omoba_template_ids::CONTENT_CATALOG_MARKER {
        return Err(LoadError::Content(format!(
            "{} was built for other content ({}); this host has {}. Rebuild the script DLLs.",
            path.display(),
            marker,
            omoba_template_ids::CONTENT_CATALOG_MARKER
        )));
    }
    Ok(manifest)
}

/// M3' ② the script DLLs the authority swapped before stepping the current tick; the team
/// projection announces them in every frame of that tick (`TeamTickFrame.script_module_swaps`).
/// The authority sets it each tick (empty when nothing was swapped).
#[derive(Clone, Debug, Default)]
pub struct ScriptSwapsThisTick(pub Vec<crate::game_proto::ScriptModuleSwap>);

/// Swaps one script DLL between two ticks (dev hot reload; authority and replicas alike): checks
/// the file digest, loads it under its own name, replaces what that module registers and the
/// ability metadata the world reads. All state stays in the host, so no state is lost.
pub fn apply_hot_swap(
    registry: &mut ScriptRegistry,
    world: &mut specs::World,
    swap: &crate::game_proto::ScriptModuleSwap,
) -> Result<(), String> {
    let path = Path::new(&swap.file);
    let sha256 = file_sha256(path)?;
    if !sha256.eq_ignore_ascii_case(&swap.sha256) {
        return Err(format!("{}: sha256 {sha256}, expected {}", path.display(), swap.sha256));
    }
    let manifest = load_hot_module(path).map_err(|error| error.to_string())?;
    let module = (manifest.module_id())().as_str().to_owned();
    if module != swap.module {
        return Err(format!("{} is module `{module}`, expected `{}`", path.display(), swap.module));
    }
    registry.replace_module(manifest)?;
    crate::runtime::populate_ability_registry(world, registry);
    record_module_swap(ScriptModule { module, file: path.to_path_buf(), sha256 });
    log::info!(
        "[scripting] hot swapped {} generation {} from {} (set {})",
        swap.module,
        swap.generation,
        path.display(),
        script_modules_hash()
    );
    Ok(())
}
