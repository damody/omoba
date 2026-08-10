## 1. Runtime policy and hero creation

- [x] 1.1 Add a precisely resolved `OMB_NO_HEROES=1` policy helper in campaign initialization.
- [x] 1.2 Apply the policy at the campaign hero creation boundary, returning before any hero entity or spawn event is created and logging the active override once.
- [x] 1.3 Add tests that avoid mutating process-global environment state and prove both disabled and default behavior.

## 2. Windows launcher configuration

- [x] 2.1 Set `OMB_NO_HEROES=1` in `run.bat` while preserving all existing launcher behavior.
- [x] 2.2 Set `OMB_NO_HEROES=1` in `run_10000.bat` while preserving `OMB_TD_STARTING_GOLD=10000`.
- [x] 2.3 Preserve CRLF and UTF-8 without BOM for both batch files and verify them at byte level.

## 3. Verification and handoff

- [x] 3.1 Run focused hero initialization tests and `cargo test --manifest-path omoba-core/Cargo.toml --no-fail-fast`.
- [x] 3.2 Run `cargo test --manifest-path omb/Cargo.toml -p omobab --no-fail-fast` to cover backend integration.
- [x] 3.3 Run a bounded `cmd.exe` launcher smoke test through the freshness step, confirm no `'M' is not recognized` error, and leave no spawned process behind.
- [x] 3.4 Run `openspec validate hero-free-dev-launchers --strict`, review the final diff, and preserve unrelated `omfue` and `omfx` state.
