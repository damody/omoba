## ADDED Requirements

### Requirement: pregame UI restoration preserves current runtime behavior

`omfx` SHALL restore the previous player-visible pregame UI flow without regressing current backend session lifecycle, lockstep timeout behavior, gameplay HUD, TD sidebar, tooltip, or kill-count display.

#### Scenario: restored UI does not revert current session logic
- **WHEN** pregame UI restoration is implemented
- **THEN** `backend_session` ownership remains session-scoped
- **AND** lockstep client stall timeout and join timeout behavior from current `master` remains present
- **AND** existing gameplay HUD, TD sidebar, tooltip, and kill-count code paths are not replaced by older versions

#### Scenario: implementation avoids whole-file rollback
- **WHEN** reviewing the implementation diff
- **THEN** `omfx/game/src/native.rs`, `omfx/game/src/pregame.rs`, and `omfx/game/src/backend_session.rs` are not wholesale restored from an older commit
- **AND** old pregame UI logic is ported as targeted changes compatible with current APIs

### Requirement: restored pregame flow is visible before gameplay

`omfx` SHALL display the restored pregame UI before gameplay starts. The flow SHALL include a main menu, start action, map selection, difficulty selection, loading or error state, and a path into the existing gameplay screen.

#### Scenario: startup shows restored main menu
- **WHEN** native `omfx` starts without `OMFX_LEGACY_AUTOSTART`
- **THEN** the first player-visible UI is the restored pregame main menu
- **AND** the menu includes the start control and layout style restored from the previous pregame UI work
- **AND** gameplay map clicks, tower placement, ability hotkeys, and start-round inputs are not submitted while this menu is active

#### Scenario: start opens map selection
- **WHEN** the player activates the restored main menu start control
- **THEN** `omfx` transitions to the restored map selection screen
- **AND** no backend session is started by the start control alone

#### Scenario: map and difficulty lead to gameplay
- **WHEN** the player selects an enabled map
- **AND** the player selects an enabled difficulty
- **THEN** `omfx` creates a session config from the selected catalog entries
- **AND** enters a loading state
- **AND** starts the current session-owned backend lifecycle
- **AND** transitions into the existing gameplay screen after the session is ready

### Requirement: restored UI remains catalog-driven

The restored pregame UI SHALL use scripts-owned pregame catalog data for map cards, difficulty cards, labels, image paths, and whitelisted actions. `omfx` SHALL NOT reintroduce canonical hard-coded Rust tables for normal pregame menu content.

#### Scenario: catalog drives restored menu content
- **WHEN** `scripts/base_content/assets/pregame_ui/catalog.json` or the active scripts content catalog defines menu, map, difficulty, and action data
- **THEN** the restored pregame UI renders from that catalog data
- **AND** changing valid catalog text or entries does not require editing `omfx` Rust source

#### Scenario: invalid catalog entries fail safely
- **WHEN** a catalog entry used by the restored UI has an unknown action, missing required story id, missing difficulty id, or missing optional image asset
- **THEN** `omfx` logs a diagnostic
- **AND** unknown or incomplete action entries are disabled or treated as no-op
- **AND** missing optional images use fallback visuals without crashing the pregame flow

### Requirement: pregame restoration has regression coverage

The restored pregame UI SHALL be covered by automated or smoke verification that proves both UI restoration and current behavior preservation.

#### Scenario: automated checks cover pregame action flow
- **WHEN** the pregame action dispatch tests run
- **THEN** start, back, map select, difficulty select, disabled entry, and unknown action behavior are verified
- **AND** menu-only startup remains separate from gameplay session startup

#### Scenario: build verifies compatibility
- **WHEN** `cargo build --manifest-path omfx/Cargo.toml -p executor` runs
- **THEN** native `omfx` builds successfully
- **AND** the build does not require an `omobab` crate dependency

#### Scenario: smoke verifies player path
- **WHEN** manual or scripted smoke starts native `omfx`
- **THEN** the restored main menu appears
- **AND** the player can go through start, map select, difficulty select, loading, and gameplay
- **AND** returning to menu or ending the game tears down the session-owned backend before another session starts
