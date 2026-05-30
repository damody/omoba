## Pregame UI Restore Inventory

### Old commits reviewed

- `9f88f55`: introduced `backend_session`, `pregame`, pregame runtime wiring in `native.rs`, delayed lockstep/sim startup, and session launcher flow.
- `8fe49fd`: refined the reusable pregame button layout, especially compact/two-column sizing in `update_pregame_ui`.
- `06baa16`: current file contents for `backend_session.rs` and `pregame.rs` match this commit for the reviewed files; the later regression is that current `native.rs` no longer imports or calls those modules.

### Portable pieces

- `PregameRuntime`, `PregameCatalog`, `PregameAction`, and `SessionSelection` from `pregame.rs` are still present and usable.
- `BackendLaunchConfig` and `BackendSession` from `backend_session.rs` are still present and preserve session-owned/external backend behavior.
- Old `native.rs` functions to port in targeted form: `default_session_selection`, `start_game_session`, `shutdown_game_session`, `update_pregame_ui`, `hide_gameplay_ui_for_pregame`, `hide_pregame_ui`, `current_pregame_buttons`, and `handle_pregame_click`.
- Old `native.rs` fields/types to port in targeted form: `PregameButtonUi`, `PregameUi`, `pregame_runtime`, `backend_session`, and `pregame_button_rects`.

### Rewrite or adapt points

- Current `native.rs` has newer TD sidebar, tooltip, tower pops, and resource registry changes; do not replace those sections from old commits.
- Current `lockstep_client.rs` has 10s join timeout and 30s stall reconnect; do not modify it.
- Current `native.rs` starts lockstep/sim directly in `Plugin::init`; this must be moved behind `start_game_session` while keeping the current config helpers and gameplay code paths.
- Current `native.rs` lacks `frontend_env_truthy` and `absolute_existing_or_joined_path`; restore small helper equivalents instead of pulling unrelated old code.

### Protected current code paths

- `lockstep_client.rs`: `join_lockstep` timeout after 10s and TickBatch stall reconnect after 30s.
- `native.rs`: TD selected tower panel, tooltip, upgrade info panel, tower pops (`tower_pops` / `pops_text`), top HUD, and ability/tower input routing.
- `backend_session.rs`: `launcher_enabled_from_env`, external backend mode, `BackendSession::shutdown`, and metadata environment construction.

### Verification

- `cargo test --manifest-path omfx\Cargo.toml -p omfx backend_session::tests` passed: 5 tests.
- `cargo test --manifest-path omfx\Cargo.toml -p omfx pregame::tests` passed: 6 tests.
- `cargo test --manifest-path omfx\Cargo.toml -p omfx input_latency_tests` passed: 24 tests.
- `cargo test --manifest-path omfx\Cargo.toml -p omfx` passed: 55 tests.
- `cargo build --manifest-path omfx\Cargo.toml -p executor` passed.
- `omfx/game` search found no `omobab =`, `omobab::`, `Command::new("cargo")`, or `cargo build` backend startup path.
- No `.bat` files were modified.
- `omfx` implementation committed as `64b0701` (`前端: 還原 pregame UI 與場次流程`).
- Root repository updates the `omfx` submodule pointer to `64b0701`.
- Manual native-window smoke tasks remain pending because they require launching and inspecting the Fyrox app interactively.
