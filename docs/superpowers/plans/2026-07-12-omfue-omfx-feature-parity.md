# omfue and omfx Feature Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `run_ue.bat` launch an Unreal frontend that shares configuration and behavior with `omfx` and matches its current menu, nine-map flow, settings, hotkeys, audio, resolution, speed, auto-round, and HUD presentation.

**Architecture:** Extend `omoba-core`, `omoba-sim`, `omoba-template-ids`, and `scripts`; do not add a crate or top-level module. External JSON/assets remain authoritative, existing generators produce Rust/UE projections, and each frontend retains only engine-specific presentation and device integration.

**Tech Stack:** Rust 1.95.0, serde/serde_json, existing Lua/template-id generation, Fyrox, Unreal Engine 5.7 C++/UMG, C ABI/cbindgen, PowerShell and Windows batch.

## Global Constraints

- Use Rust 1.95.0 for host and script DLL builds.
- Do not add a crate or top-level repository module.
- Shared configuration belongs outside `omfx` and `omfue`, in the existing `omoba-core`, `omoba-sim`, `omoba-template-ids`, and `scripts` boundaries.
- `scripts/base_content/assets/pregame_ui/catalog.json` remains the authoritative nine-map catalog.
- Keep all `.bat` files CRLF and UTF-8 without BOM.
- Preserve unrelated changes, including the existing dirty `omfue` worktree.
- Commit changes in the repository that owns them; commit `omfue` changes inside its submodule, then bump the root pointer.
- Use characterization tests before moving behavior out of `omfx`.

---

## Phase A — Shared contracts and generated configuration

### Task 1: Characterize current omfx behavior

**Files:**
- Modify: `omfx/game/src/pregame.rs`
- Modify: `omfx/game/src/hotkeys.rs`
- Modify: `omfx/game/src/native.rs`

**Interfaces:**
- Consumes: shipped `scripts/base_content/assets/pregame_ui/catalog.json`.
- Produces: regression tests defining menu transitions, nine-map filtering, hotkey defaults, speed values, and auto-round semantics.

- [ ] **Step 1: Add failing/characterization tests for the public behavior**

```rust
#[test]
fn shipped_flow_selects_difficulty_before_one_of_three_maps() {
    let mut runtime = PregameRuntime::new_for_menu(PregameCatalog::load());
    assert_eq!(runtime.state, PregameState::MainMenu);
    runtime.dispatch(&PregameAction::Navigate { target: "difficulty_select".into() });
    runtime.dispatch(&PregameAction::SelectDifficulty { difficulty_id: "novice".into() });
    assert_eq!(runtime.catalog.maps_for_difficulty("novice").len(), 3);
}

#[test]
fn shipped_speed_cycle_is_one_two_three() {
    assert_eq!(next_td_speed(1), 2);
    assert_eq!(next_td_speed(2), 3);
    assert_eq!(next_td_speed(3), 1);
}
```

- [ ] **Step 2: Run the focused tests and record the baseline**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx pregame -- --nocapture`

Expected: all existing tests pass; any assertion that exposes a real current value is corrected to that observed value before proceeding.

- [ ] **Step 3: Add a hotkey definition snapshot test**

```rust
#[test]
fn shipped_hotkey_actions_are_unique_and_stable() {
    let defs = hotkey_defs();
    let ids: std::collections::BTreeSet<_> = defs.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids.len(), defs.len());
    assert!(ids.contains("tower_dart"));
    assert!(ids.contains("tower_boomerang"));
}
```

- [ ] **Step 4: Run all omfx game tests**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx`

Expected: PASS.

- [ ] **Step 5: Commit in the omfx submodule**

```powershell
git -C omfx add game/src/pregame.rs game/src/hotkeys.rs game/src/native.rs
git -C omfx commit -m "test: characterize shared frontend behavior"
```

### Task 2: Add stable frontend IDs to the existing template-id generator

**Files:**
- Modify: `omoba-template-ids/build.rs`
- Modify: `omoba-template-ids/src/lib.rs`
- Modify: `omoba-template-ids/src/runtime_content.rs`
- Create: `scripts/base_content/assets/frontend/ids.json`

**Interfaces:**
- Consumes: `ids.json` arrays `maps`, `difficulties`, `hotkey_actions`, `setting_keys`, `screens`, `widget_roles`, and `audio_cues`.
- Produces: generated `frontend_ids::{MAP_*, DIFFICULTY_*, HOTKEY_*, SETTING_*, SCREEN_*, WIDGET_*, AUDIO_*}` string constants and `validate_frontend_id(&str) -> bool`.

- [ ] **Step 1: Add a failing generator test**

```rust
#[test]
fn frontend_ids_reject_duplicates() {
    let json = r#"{"maps":["td_1","td_1"],"difficulties":[],"hotkey_actions":[],"setting_keys":[],"screens":[],"widget_roles":[],"audio_cues":[]}"#;
    assert!(parse_frontend_ids(json).unwrap_err().to_string().contains("duplicate frontend id 'td_1'"));
}
```

- [ ] **Step 2: Verify the test fails**

Run: `cargo test --manifest-path omoba-template-ids/Cargo.toml frontend_ids_reject_duplicates`

Expected: FAIL because `parse_frontend_ids` does not exist.

- [ ] **Step 3: Add the external ID source and parser**

```json
{
  "maps": ["td_green_crossroads", "td_riverside_path", "td_farmstead_bends", "td_frozen_bridge", "td_mine_corridor", "td_twin_gate_outpost", "td_molten_fork", "td_tidal_harbor", "td_twilight_maze"],
  "difficulties": ["novice", "intermediate", "advanced"],
  "hotkey_actions": ["tower_dart", "tower_tack", "tower_bomb", "tower_ice", "tower_cake_splash", "tower_arty", "tower_boomerang", "start_round", "toggle_speed", "toggle_auto_round"],
  "setting_keys": ["master_volume", "music_volume", "sfx_volume", "resolution", "window_mode", "hotkeys"],
  "screens": ["main_menu", "settings", "difficulty_select", "map_select", "starting_session", "in_game", "session_ended"],
  "widget_roles": ["button", "decoration", "map_card", "setting_row", "hotkey_row"],
  "audio_cues": ["bgm", "button_click", "tower_place", "cookie_crunch"]
}
```

Implement `parse_frontend_ids` with serde, flatten all groups into a `BTreeSet`, and return an error containing the exact duplicate value.

- [ ] **Step 4: Generate constants and expose the module**

```rust
pub mod frontend_ids {
    include!(concat!(env!("OUT_DIR"), "/frontend_ids_gen.rs"));
}
```

Add `cargo:rerun-if-changed=../scripts/base_content/assets/frontend/ids.json` to `build.rs`.

- [ ] **Step 5: Run tests**

Run: `cargo test --manifest-path omoba-template-ids/Cargo.toml`

Expected: PASS, including duplicate and shipped-ID tests.

- [ ] **Step 6: Commit**

```powershell
git add omoba-template-ids scripts/base_content/assets/frontend/ids.json
git commit -m "feat: generate stable frontend ids"
```

### Task 3: Add simulation-facing session options to omoba-sim

**Files:**
- Create: `omoba-sim/src/session_options.rs`
- Modify: `omoba-sim/src/lib.rs`

**Interfaces:**
- Produces: `TdSpeed::try_from(u32)`, `TdSpeed::next()`, `AutoRoundGate::should_submit(bool, bool)`, and `SessionOptions { story_id, difficulty_id, speed, auto_round }`.

- [ ] **Step 1: Write failing unit tests**

```rust
#[test]
fn speed_cycles_through_supported_values() {
    assert_eq!(TdSpeed::One.next(), TdSpeed::Two);
    assert_eq!(TdSpeed::Two.next(), TdSpeed::Three);
    assert_eq!(TdSpeed::Three.next(), TdSpeed::One);
}

#[test]
fn auto_round_submits_once_per_idle_round() {
    assert!(AutoRoundGate::should_submit(true, false));
    assert!(!AutoRoundGate::should_submit(true, true));
    assert!(!AutoRoundGate::should_submit(false, false));
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omoba-sim/Cargo.toml session_options`

Expected: FAIL because the module is missing.

- [ ] **Step 3: Implement the deterministic value types**

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TdSpeed { #[default] One = 1, Two = 2, Three = 3 }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionOptions {
    pub story_id: String,
    pub difficulty_id: String,
    pub speed: TdSpeed,
    pub auto_round: bool,
}
```

Keep the module free of filesystem, frontend, Fyrox, and UE dependencies.

- [ ] **Step 4: Run tests and commit**

Run: `cargo test --manifest-path omoba-sim/Cargo.toml`

Expected: PASS.

```powershell
git add omoba-sim/src/session_options.rs omoba-sim/src/lib.rs
git commit -m "feat: share TD session options"
```

### Task 4: Move frontend state and settings contracts into omoba-core

**Files:**
- Create: `omoba-core/src/frontend/mod.rs`
- Create: `omoba-core/src/frontend/catalog.rs`
- Create: `omoba-core/src/frontend/pregame.rs`
- Create: `omoba-core/src/frontend/hotkeys.rs`
- Create: `omoba-core/src/frontend/settings.rs`
- Create: `omoba-core/src/frontend/layout.rs`
- Modify: `omoba-core/src/lib.rs`
- Modify: `omoba-core/Cargo.toml`

**Interfaces:**
- Produces: `FrontendCatalog::from_json_str`, `PregameRuntime::dispatch`, `FrontendAction`, `FrontendSettingsV1`, `HotkeyBinding`, `LayoutManifest`, and `load_or_migrate_settings`.
- Consumes: `omoba_sim::{SessionOptions, TdSpeed}` and generated `omoba_template_ids::frontend_ids`.

- [ ] **Step 1: Write failing state, migration, and conflict tests**

```rust
#[test]
fn difficulty_then_map_produces_session_options() {
    let mut runtime = PregameRuntime::new(test_catalog());
    runtime.dispatch(FrontendAction::SelectDifficulty("novice".into())).unwrap();
    let started = runtime.dispatch(FrontendAction::SelectMap("td_green_crossroads".into())).unwrap();
    assert_eq!(started.session.unwrap().story_id, "TD_GREEN_CROSSROADS");
}

#[test]
fn duplicate_hotkey_binding_is_rejected() {
    let mut settings = FrontendSettingsV1::default();
    assert!(settings.bind("tower_dart", "Digit1").is_ok());
    assert_eq!(settings.bind("tower_tack", "Digit1").unwrap_err(), SettingsError::BindingConflict("tower_dart".into()));
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omoba-core/Cargo.toml frontend --no-default-features --features game-proto`

Expected: FAIL because `frontend` is missing.

- [ ] **Step 3: Implement focused files and stable interfaces**

```rust
pub enum FrontendAction {
    Navigate(ScreenId), Back, SelectDifficulty(String), SelectMap(String), StartSession,
    SetVolume { channel: VolumeChannel, value: f32 }, SetResolution(Resolution),
    SetWindowMode(WindowMode), BindHotkey { action_id: String, chord: String },
    CycleSpeed, SetAutoRound(bool), ReturnToMenu,
}

pub struct FrontendUpdate {
    pub state: PregameState,
    pub session: Option<SessionOptions>,
    pub settings_changed: bool,
}
```

Use serde defaults, reject non-finite/out-of-range volume, reject unknown IDs, and keep parsing available under `--no-default-features` for wasm.

- [ ] **Step 4: Implement atomic settings persistence and legacy import**

`load_or_migrate_settings(path, legacy_hotkeys_path)` must parse V1, import the legacy `{ action_id: chord }` object once when V1 is absent, write `path.tmp`, then rename it over `path`. A malformed file is renamed with `.broken` before defaults are written.

- [ ] **Step 5: Run native and wasm-compatible tests**

Run: `cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features game-proto frontend`

Run: `cargo test --manifest-path omoba-core/Cargo.toml frontend`

Expected: both PASS.

- [ ] **Step 6: Commit**

```powershell
git add omoba-core/Cargo.toml omoba-core/src/lib.rs omoba-core/src/frontend
git commit -m "feat: add shared frontend contracts"
```

### Task 5: Externalize layout, settings, hotkeys, and audio configuration

**Files:**
- Create: `scripts/base_content/assets/frontend/layout.json`
- Create: `scripts/base_content/assets/frontend/settings.json`
- Create: `scripts/base_content/assets/frontend/hotkeys.json`
- Create: `scripts/base_content/assets/frontend/audio.json`
- Modify: `scripts/base_content/assets/pregame_ui/catalog.json`
- Modify: `scripts/base_content/assets/pregame_ui/README.md`

**Interfaces:**
- Produces: schema-versioned source files consumed by both frontends and UE codegen.

- [ ] **Step 1: Add failing shipped-manifest tests in omoba-core**

```rust
#[test]
fn shipped_frontend_manifests_validate_together() {
    let bundle = FrontendBundle::load_from_root(Path::new("../scripts/base_content/assets" )).unwrap();
    assert_eq!(bundle.catalog.maps.len(), 9);
    assert_eq!(bundle.layout.reference_size, [1920, 1080]);
    bundle.validate_references().unwrap();
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omoba-core/Cargo.toml shipped_frontend_manifests_validate_together`

Expected: FAIL because the four manifests are absent.

- [ ] **Step 3: Add schema-versioned external files**

Use this common envelope in every file:

```json
{ "schema_version": 1, "content": {} }
```

Populate layout from the current `pregame_ref_rect` and TD HUD constants in `omfx/game/src/native.rs`; populate hotkeys from `hotkey_defs()`; populate audio from the four shipped cues; populate Settings with volume ranges, supported window modes, and resolution presets.

- [ ] **Step 4: Replace duplicated catalog fallback values with shipped-data validation**

Keep a minimal recovery catalog containing one disabled diagnostic entry only; normal builds and tests must load the external nine-map catalog.

- [ ] **Step 5: Run validation tests and commit**

Run: `cargo test --manifest-path omoba-core/Cargo.toml frontend`

Expected: PASS with nine maps and no dangling asset/ID references.

```powershell
git add scripts/base_content/assets/frontend scripts/base_content/assets/pregame_ui omoba-core/src/frontend
git commit -m "content: externalize shared frontend configuration"
```

## Phase B — Switch omfx to the shared contracts

### Task 6: Replace omfx-local state/config implementations

**Files:**
- Modify: `omfx/game/Cargo.toml`
- Modify: `omfx/game/src/lib.rs`
- Modify: `omfx/game/src/pregame.rs`
- Modify: `omfx/game/src/hotkeys.rs`
- Modify: `omfx/game/src/native.rs`
- Modify: `omfx/game/src/sim_runner.rs`

**Interfaces:**
- Consumes: all `omoba_core::frontend` contracts and `omoba_sim::session_options`.
- Produces: unchanged omfx player behavior backed by shared data.

- [ ] **Step 1: Change characterization tests to import shared types**

```rust
use omoba_core::frontend::{FrontendAction, FrontendBundle, PregameRuntime};
use omoba_sim::session_options::TdSpeed;
```

- [ ] **Step 2: Run tests to verify the migration is incomplete**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx`

Expected: FAIL on unresolved imports or type mismatches.

- [ ] **Step 3: Make local modules thin engine adapters**

`pregame.rs` may re-export shared types and contain Fyrox-specific button mapping only. `hotkeys.rs` keeps key-code conversion and file-location discovery only. `native.rs` loads `FrontendBundle`, dispatches `FrontendAction`, and renders returned state/layout. `sim_runner.rs` accepts `SessionOptions` without rebuilding difficulty semantics.

- [ ] **Step 4: Remove duplicated fallback catalogs, hotkey definitions, and speed rules**

Use `rg` to prove there is exactly one source:

Run: `rg -n "fallback_catalog_has_three|fn hotkey_defs|fn next_td_speed" omfx/game omoba-core omoba-sim`

Expected: definitions occur only in shared modules; omfx contains calls/re-exports.

- [ ] **Step 5: Run omfx tests and smoke build**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx`

Run: `cargo build --manifest-path omfx/Cargo.toml -p executor --features runtime-lua-content`

Expected: PASS.

- [ ] **Step 6: Commit in omfx**

```powershell
git -C omfx add game
git -C omfx commit -m "refactor: consume shared frontend contracts"
```

## Phase C — Generate and bridge the shared contract to Unreal

### Task 7: Extend existing omfue codegen and freshness checks

**Files:**
- Modify: `omfue/codegen/src/lib.rs`
- Modify: `omfue/codegen/src/main.rs`
- Modify: `omfue/codegen/Cargo.toml`
- Modify: `omfue/build_bridge.bat`
- Modify: `omfue/check_om_fresh.ps1`
- Generated: `omfue/Plugins/OmRuntime/Source/OmGenerated/OmFrontendConfig.generated.h`
- Generated: `omfue/Plugins/OmRuntime/Content/Generated/Frontend/*`

**Interfaces:**
- Consumes: `FrontendBundle` and stable IDs.
- Produces: deterministic UE header, staged JSON/images/audio, and `frontend_freshness.json` containing schema version plus source hashes.

- [ ] **Step 1: Add a failing codegen test**

```rust
#[test]
fn frontend_codegen_is_deterministic_and_complete() {
    let first = generate_frontend_bundle(test_assets()).unwrap();
    let second = generate_frontend_bundle(test_assets()).unwrap();
    assert_eq!(first, second);
    assert!(first.files.contains_key("OmFrontendConfig.generated.h"));
    assert!(first.files.contains_key("frontend_freshness.json"));
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omfue/codegen/Cargo.toml frontend_codegen`

Expected: FAIL because the frontend generator is missing.

- [ ] **Step 3: Implement deterministic generation and asset copying**

Sort every emitted map by stable ID, normalize JSON with `serde_json::to_string_pretty`, and copy only changed bytes. The generated header exposes schema version, manifest relative paths, and stable FName literals; it must not duplicate the full editable JSON.

- [ ] **Step 4: Wire build and freshness**

Pass `--frontend-assets "%ROOT%\scripts\base_content\assets"` to existing codegen. Extend `check_om_fresh.ps1` to regenerate into `Saved\FreshnessCheck`, byte-compare the header/manifests, and SHA-256-check staged binary assets.

- [ ] **Step 5: Preserve CRLF and run checks**

Run: `cargo test --manifest-path omfue/codegen/Cargo.toml`

Run: `cmd /c omfue\build_bridge.bat`

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File omfue\check_om_fresh.ps1`

Expected: all PASS; a second build reports unchanged outputs.

- [ ] **Step 6: Commit inside omfue**

```powershell
git -C omfue add codegen build_bridge.bat check_om_fresh.ps1 Plugins/OmRuntime/Source/OmGenerated Plugins/OmRuntime/Content/Generated
git -C omfue commit -m "feat: generate shared frontend configuration"
```

### Task 8: Extend the Rust bridge ABI with menu/settings state and actions

**Files:**
- Modify: `omfue/bridge/src/lib.rs`
- Modify: `omfue/bridge/src/driver.rs`
- Create: `omfue/bridge/src/frontend.rs`
- Modify: `omfue/bridge/smoke/om_bridge_header_smoke.cpp`
- Modify: `omfue/bridge/Cargo.toml`

**Interfaces:**
- Produces: `OmFrontendState`, `OmFrontendAction`, `om_frontend_state_acquire`, `om_frontend_state_release`, and `om_frontend_dispatch`.
- Consumes: `FrontendBundle`, `PregameRuntime`, `FrontendSettingsV1`, and `SessionOptions`.

- [ ] **Step 1: Add failing Rust ABI tests**

```rust
#[test]
fn frontend_action_selects_difficulty_and_map() {
    let runtime = test_runtime();
    assert_eq!(dispatch(&runtime, OmFrontendAction::select_difficulty("novice")).unwrap().screen, OM_SCREEN_MAP_SELECT);
}

#[test]
fn frontend_state_owns_strings_until_release() {
    let lease = acquire_test_frontend_state();
    assert!(!lease.state.maps.is_null());
    assert_eq!(unsafe { (*lease.state.maps).id.as_str() }, "td_green_crossroads");
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omfue/bridge/Cargo.toml frontend_state`

Expected: FAIL because ABI types/functions are absent.

- [ ] **Step 3: Implement versioned tail-compatible ABI structs**

```rust
#[repr(C)]
pub struct OmFrontendState {
    pub struct_size: u32,
    pub abi_version: u32,
    pub screen: u32,
    pub map_count: u32,
    pub maps: *const OmFrontendMap,
    pub settings: OmFrontendSettings,
    pub last_error: OmStringRef,
}
```

Use the bridge's existing lease/string-table ownership pattern. Reject mismatched `struct_size`, unknown action IDs, and invalid values with `set_last_error`.

- [ ] **Step 4: Connect session start to the existing runtime driver**

When shared dispatch returns `FrontendUpdate.session`, pass the exact `SessionOptions` into the existing local/networked startup path. On failure call `recover_to_map(error)` and publish a new state.

- [ ] **Step 5: Run ABI and header smoke tests**

Run: `cargo test --manifest-path omfue/bridge/Cargo.toml`

Run: `cmd /c omfue\build_bridge.bat`

Expected: PASS; cbindgen header smoke compiles with new fields/functions.

- [ ] **Step 6: Commit inside omfue**

```powershell
git -C omfue add bridge Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/om_bridge.h
git -C omfue commit -m "feat: bridge shared frontend state"
```

## Phase D — Unreal-native presentation and device integration

### Task 9: Add an Unreal frontend state model and menu widgets

**Files:**
- Create: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmFrontendState.h`
- Create: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmFrontendState.cpp`
- Create: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmFrontendRootWidget.h`
- Create: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmFrontendRootWidget.cpp`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmRuntimeBridgeSubsystem.h`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmRuntimeBridgeSubsystem.cpp`
- Modify: `omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmEditorAutomationTests.cpp`

**Interfaces:**
- Produces: Blueprint-readable `FOmFrontendViewState`, `UOmFrontendRootWidget::ApplyState`, and subsystem multicast `OnFrontendStateChanged`.

- [ ] **Step 1: Add failing Unreal automation tests**

```cpp
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FOmFrontendMenuFlowTest, "Om.Frontend.MenuFlow", EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FOmFrontendMenuFlowTest::RunTest(const FString& Parameters)
{
    FOmFrontendViewState State = FOmFrontendTestData::NineMapState();
    TestEqual(TEXT("Nine maps"), State.Maps.Num(), 9);
    TestEqual(TEXT("Reference width"), State.Layout.ReferenceWidth, 1920);
    return true;
}
```

- [ ] **Step 2: Build to verify failure**

Run: `cmd /c run_ue.bat --build-only`

Expected: FAIL because frontend view/widget types do not exist.

- [ ] **Step 3: Implement the model, bridge polling, and generic root widget**

Copy ABI data immediately into owned UE `USTRUCT` arrays/strings. `ApplyState` rebuilds only when revision changes and dispatches button actions through the subsystem. Generate menu rows/cards from data; do not hard-code nine widget instances.

- [ ] **Step 4: Build and run menu automation**

Run: `cmd /c run_ue.bat --build`

Run: `"D:\UE_5.7\Engine\Binaries\Win64\UnrealEditor-Cmd.exe" "D:\code\omoba\omfue\om.uproject" -ExecCmds="Automation RunTests Om.Frontend.MenuFlow;Quit" -unattended -NullRHI -log`

Expected: test PASS and no missing frontend asset diagnostics.

- [ ] **Step 5: Commit inside omfue**

```powershell
git -C omfue add Plugins/OmRuntime/Source/OmRuntime Plugins/OmRuntime/Source/OmEditor
git -C omfue commit -m "feat: render shared Unreal frontend menus"
```

### Task 10: Implement settings, hotkey rebinding, audio, and resolution adapters

**Files:**
- Create: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmFrontendDeviceAdapter.h`
- Create: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmFrontendDeviceAdapter.cpp`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmPlayerController.h`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmPlayerController.cpp`
- Modify: `omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmEditorAutomationTests.cpp`

**Interfaces:**
- Consumes: shared `FOmFrontendViewState.Settings` and hotkey actions.
- Produces: UE implementations for volume/audio cues, resolution/window mode, and key-to-shared-chord conversion.

- [ ] **Step 1: Add failing adapter tests**

```cpp
TestEqual(TEXT("Ctrl digit chord"), Adapter.ToSharedChord(EKeys::One, true), FString(TEXT("Ctrl+Digit1")));
TestEqual(TEXT("Safe fallback width"), Adapter.SafeFallbackResolution().X, 1280);
TestEqual(TEXT("Safe fallback height"), Adapter.SafeFallbackResolution().Y, 720);
```

- [ ] **Step 2: Verify the tests fail**

Run the `Om.Frontend.DeviceAdapter` automation group with UnrealEditor-Cmd.

Expected: FAIL because adapter methods are absent.

- [ ] **Step 3: Implement engine adapters**

Use `UGameUserSettings` for resolution/window mode, UE audio classes for master/music/SFX, and PlayerController key events for rebinding. Apply a new resolution only after validation; on failure restore the previous mode, then use 1280×720 windowed if no previous mode exists. Missing audio device disables playback and reports a non-fatal diagnostic.

- [ ] **Step 4: Wire actions and verify persistence**

All UI changes dispatch `FrontendAction`; bridge persists the shared file. Restart a headless editor instance and assert the chosen volume/chord round-trips.

- [ ] **Step 5: Run automation and commit**

Run: `"D:\UE_5.7\Engine\Binaries\Win64\UnrealEditor-Cmd.exe" "D:\code\omoba\omfue\om.uproject" -ExecCmds="Automation RunTests Om.Frontend.DeviceAdapter;Quit" -unattended -NullRHI -log`

Expected: PASS.

```powershell
git -C omfue add Plugins/OmRuntime/Source/OmRuntime Plugins/OmRuntime/Source/OmEditor
git -C omfue commit -m "feat: add Unreal frontend settings adapters"
```

### Task 11: Match the omfx in-game HUD and controls

**Files:**
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmUiEventTypes.h`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmPlayerController.h`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmPlayerController.cpp`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Public/OmFrontendRootWidget.h`
- Modify: `omfue/Plugins/OmRuntime/Source/OmRuntime/Private/OmFrontendRootWidget.cpp`
- Modify: `omfue/Plugins/OmRuntime/Source/OmEditor/Private/OmEditorAutomationTests.cpp`
- Modify through Unreal Editor: `omfue/Content/RustBP/UI/WBP_OmHudRoot.uasset`

**Interfaces:**
- Consumes: existing tower/hero/ability/buff/overlay events plus shared layout/hotkeys/session options.
- Produces: data-driven TD shop, selected tower panel, three upgrade paths, speed, auto-round, pause, return-menu, and F1 hotkey panel.

- [ ] **Step 1: Add failing HUD state tests**

```cpp
TestTrue(TEXT("Seven tower cards"), Hud->GetTowerCards().Num() >= 7);
TestEqual(TEXT("Three upgrade paths"), Hud->GetUpgradePathCount(), 3);
TestEqual(TEXT("Speed label"), Hud->GetSpeedLabel(), FString(TEXT("x2")));
TestTrue(TEXT("Auto round checked"), Hud->IsAutoRoundEnabled());
```

- [ ] **Step 2: Run the HUD automation group and confirm failure**

Run the `Om.Frontend.HudParity` automation group.

Expected: FAIL on missing data-driven controls.

- [ ] **Step 3: Implement controls from shared layout and existing bridge events**

Keep the existing input priority: UI capture, pending ability/attack-move/placement, tower selection, then world commands. Speed and auto-round dispatch shared actions. F1 opens the shared hotkey list. Ctrl+Escape requests return-to-menu. UI hover/capture keeps both HUD and camera guards active.

- [ ] **Step 4: Update the existing Blueprint asset in Unreal Editor**

Bind `WBP_OmHudRoot` to the native root/state model, preserve existing hero/ability/buff bindings, compile, save, and run `validate_assets`. Do not replace unrelated handcrafted visuals.

- [ ] **Step 5: Run HUD, input, and gameplay smoke tests**

Run Unreal automation groups `Om.Frontend.HudParity`, `Om.Input`, and `Om.Runtime`.

Expected: PASS with no Blueprint compile errors.

- [ ] **Step 6: Commit inside omfue**

```powershell
git -C omfue add Plugins/OmRuntime Content/RustBP/UI/WBP_OmHudRoot.uasset
git -C omfue commit -m "feat: match omfx Unreal gameplay HUD"
```

## Phase E — Launcher, visual QA, and integration

### Task 12: Fix run_ue.bat defaults, direct-session mode, and freshness

**Files:**
- Modify: `run_ue.bat`
- Modify: `run_ue_tests.bat`
- Modify: `scripts/dev_run_freshness.ps1`

**Interfaces:**
- Produces: menu-first default launch; `--direct-session <story> --difficulty <id>` diagnostic launch; shared frontend artifact freshness checks.

- [ ] **Step 1: Add failing batch parser tests**

Add cases in `run_ue_tests.bat` that invoke a parse-only test hook and assert:

```bat
call run_ue.bat --test-args
if not "%RUN_MODE%"=="game" exit /b 1
if not "%UE_RUNTIME_ARG%"=="-om-menu" exit /b 1
call run_ue.bat --test-args --direct-session TD_GREEN_CROSSROADS --difficulty novice
if not "%DIRECT_STORY%"=="TD_GREEN_CROSSROADS" exit /b 1
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cmd /c run_ue_tests.bat`

Expected: FAIL because menu/direct-session parsing is absent.

- [ ] **Step 3: Implement menu-first and strict argument parsing**

Set `UE_RUNTIME_ARG=-om-menu`; remove the unconditional `OMB_STORY=TD_1`. Require values after `--direct-session`, `--difficulty`, and `--seconds`. Set story/difficulty environment variables only for direct-session. Preserve all existing modes and backend behavior.

- [ ] **Step 4: Add frontend-manifest freshness to build_all**

Run the shared generator before bridge build; fail if schema/hash mismatch remains after generation. Stage generated frontend assets into the cooked plugin path alongside `base_content.dll`.

- [ ] **Step 5: Normalize batch files and run launcher tests**

```powershell
$paths = 'run_ue.bat','run_ue_tests.bat'
foreach ($p in $paths) {
  $c = (Get-Content -Raw $p) -replace "(?<!`r)`n","`r`n"
  [IO.File]::WriteAllText((Resolve-Path $p), $c, [Text.UTF8Encoding]::new($false))
}
```

Run: `cmd /c run_ue_tests.bat`

Run: `cmd /c run_ue.bat --build-only`

Expected: PASS; default diagnostics say `runtime: menu`, and direct-session diagnostics include the requested story/difficulty.

- [ ] **Step 6: Commit root changes**

```powershell
git add run_ue.bat run_ue_tests.bat scripts/dev_run_freshness.ps1
git commit -m "fix: launch Unreal through shared pregame flow"
```

### Task 13: Visual regression and end-to-end acceptance

**Files:**
- Create: `omfue/Docs/FrontendParity.md`
- Create: `omfue/Content/Tests/FrontendReference/README.md`
- Modify as defects require: files owned by Tasks 6–12 only.
- Modify: root `.gitmodules` pointer state through normal submodule commit tracking.

**Interfaces:**
- Consumes: completed shared contracts, generators, omfx adapter, UE bridge/widgets, and launcher.
- Produces: reproducible screenshots and a signed-off parity matrix.

- [ ] **Step 1: Capture reference screens in both frontends**

Capture main menu, Settings, difficulty select, each map tier, in-game HUD, selected tower, hotkey panel, and paused state at 1920×1080. Also capture UE at 1280×720 and one ultrawide resolution.

- [ ] **Step 2: Record objective acceptance checks**

`FrontendParity.md` must list every screen/action, source config key, omfx result, omfue result, screenshot filename, and PASS/FAIL. A screen passes when reference rectangles differ by at most 4 px at 1920×1080, text/content match, and no primary control overlaps at the two responsive sizes.

- [ ] **Step 3: Run the complete Rust suite**

Run:

```powershell
cargo test --manifest-path omoba-template-ids/Cargo.toml
cargo test --manifest-path omoba-sim/Cargo.toml
cargo test --manifest-path omoba-core/Cargo.toml frontend
cargo test --manifest-path omfx/Cargo.toml -p omfx
cargo test --manifest-path omfue/codegen/Cargo.toml
cargo test --manifest-path omfue/bridge/Cargo.toml
```

Expected: all PASS.

- [ ] **Step 4: Run launcher and Unreal acceptance**

Run:

```powershell
cmd /c run_ue_tests.bat
cmd /c run_ue.bat --build-only
cmd /c run_ue.bat --headless-smoke --seconds 30
cmd /c run_ue.bat --headless-smoke --networked --seconds 30
cmd /c run_ue.bat --game-smoke --direct-session TD_GREEN_CROSSROADS --difficulty novice --seconds 30
```

Expected: all exit 0; menu smoke log contains the shared frontend startup marker, direct smoke contains the requested story/difficulty, and networked smoke reaches bridge runtime startup.

- [ ] **Step 5: Verify shared-source and freshness invariants**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File omfue/check_om_fresh.ps1`

Run: `git status --short` in root, `omfx`, and `omfue`.

Expected: freshness PASS; only intentional docs/submodule pointer changes remain.

- [ ] **Step 6: Commit docs, submodule pointers, and final fixes**

```powershell
git -C omfue add Docs/FrontendParity.md Content/Tests/FrontendReference
git -C omfue commit -m "test: document Unreal frontend parity"
git add omfx omfue docs/superpowers/plans/2026-07-12-omfue-omfx-feature-parity.md
git commit -m "feat: synchronize Unreal and Fyrox frontends"
```

## Execution checkpoints

- After Phase A: review schemas and generated IDs before either frontend migrates.
- After Phase B: verify `omfx` behavior has not changed.
- After Phase C: review ABI ownership/versioning and generated-file freshness.
- After Phase D: review Unreal interaction and visual parity before launcher defaults change.
- After Phase E: require the full acceptance matrix and clean intended repository states.
