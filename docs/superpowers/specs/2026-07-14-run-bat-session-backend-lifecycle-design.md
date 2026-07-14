# `run.bat` Session Backend Lifecycle Design

## Problem

`run.bat` starts a backend before launching the frontend, while omfx also starts a backend for every selected game session. Both processes use the same KCP address. The prestarted backend wins the port and remains alive when the player returns to the title screen; the session-owned backend either cannot bind or is not the process serving the client. Starting another game therefore reconnects to the old backend world instead of a fresh game.

This regressed when commit `f5e4aa7` restored the old prestart flow after commit `f50e0a4` had moved backend ownership into the frontend session launcher. The later frontend worker-shutdown fix correctly stops lockstep and simulation workers, but cannot reset a backend process that `BackendSession` does not own.

## Goals

- Give each game session exactly one backend process owned by omfx.
- Make returning to the title terminate that process before another game starts.
- Preserve per-session map, story, difficulty, content, and test starting-gold environment values.
- Keep `run_10000.bat` working through its existing delegation to `run.bat`.
- Preserve CRLF line endings in Windows batch files.

## Non-goals

- Do not add a runtime world-reset protocol to the backend.
- Do not modify omfx session shutdown logic or the `omfue` submodule.
- Do not change external-backend support selected explicitly through `OMFX_EXTERNAL_BACKEND` or `OMFX_DISABLE_SESSION_LAUNCHER`.

## Design

`run.bat` will remain responsible for killing stale processes at initial launch, checking and building the script DLL, backend, and frontend, staging the DLL, and validating the executables. It will set `OMFX_BACKEND_EXE` to the freshly built backend and then launch only `executor.exe`.

The script will no longer call `:start_backend` before starting the frontend or call `:stop_backend` after it exits. The now-unused external backend helper labels will be removed so the file has one unambiguous ownership model. omfx's existing `BackendSession::start` will launch the configured executable after the player selects a map, and `Game::shutdown_game_session` will terminate and wait for that owned child when the player returns.

`run_10000.bat` will remain unchanged. Its `OMB_TD_STARTING_GOLD=10000` value is inherited by the frontend and forwarded by the session launcher to the session-owned backend.

## Failure Handling

If the configured backend executable is missing, `run.bat` continues to fail before launching the frontend. If a session backend cannot start, the existing omfx session-start error path shuts down partial session state and reports the failure without entering gameplay. Initial stale-process cleanup remains in place to recover from a prior crashed development run.

## Testing

- Add a static launcher regression test that reads `run.bat` and asserts it configures `OMFX_BACKEND_EXE`, launches the frontend, and does not invoke the external `:start_backend` or `:stop_backend` flow.
- Assert `run.bat` retains CRLF line endings and has no UTF-8 BOM.
- Run the launcher regression test directly with PowerShell.
- Run focused omfx backend-session and return-to-menu lifecycle tests.
- Run formatting checks for any touched Rust workspace; no Rust production change is expected.

## Success Criteria

Using `run.bat`, the player can start a game, return to the title before or after `GameStart`, and immediately start another game. The second game runs in a newly spawned backend with tick and round state beginning from the new session and with the newly selected map and difficulty.
