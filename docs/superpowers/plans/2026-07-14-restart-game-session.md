# Restartable Game Session Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make returning to the title screen terminate the old lockstep/simulation session deterministically so pressing Start can immediately launch a clean game.

**Architecture:** Add explicit cancellation and consuming shutdown methods to the two frontend worker handles. `Game::shutdown_game_session` will take and join both workers before stopping the backend, then reset session-scoped client state before returning to pregame.

**Tech Stack:** Rust 1.95.0, Tokio `watch`/`select!`, crossbeam-channel, Fyrox, Cargo tests.

## Global Constraints

- Keep host and script builds on Rust 1.95.0.
- Do not modify `omfue`.
- Preserve normal lockstep reconnect behavior for genuine network failures.
- Cancellation must finish without waiting for the existing ten-second GameStart timeout.
- Use TDD: observe each focused regression test fail before implementation.

---

### Task 1: Cancellable Lockstep Worker

**Files:**
- Modify: `omfx/game/Cargo.toml`
- Modify: `omfx/game/src/lockstep_client.rs`

**Interfaces:**
- Produces: `LockstepClientHandle::shutdown(self)` which signals cancellation and joins the worker.
- Produces: `cancel_or(&mut watch::Receiver<bool>, future) -> Option<future::Output>` used at every long asynchronous wait.

- [ ] **Step 1: Add a failing cancellation regression test**

Add a `#[cfg(test)]` module in `lockstep_client.rs` that runs a pending future through the cancellation helper, signals cancellation from another thread, and asserts completion in under one second:

```rust
#[test]
fn cancellation_interrupts_a_pending_handshake() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(25));
        cancel_tx.send(true).unwrap();
    });
    let started = Instant::now();
    let result = runtime.block_on(cancel_or(
        &mut cancel_rx,
        std::future::pending::<()>(),
    ));
    assert!(result.is_none());
    assert!(started.elapsed() < Duration::from_secs(1));
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx cancellation_interrupts_a_pending_handshake`

Expected: compilation fails because `tokio::sync::watch`, `enable_time`, and `cancel_or` are not available yet.

- [ ] **Step 3: Enable Tokio time/sync and implement cancellation**

Change the native Tokio features in `omfx/game/Cargo.toml`:

```toml
tokio = { version = "1.0", features = ["rt-multi-thread", "sync", "time"] }
```

Add a watch sender to `LockstepClientHandle`, pass its receiver into `run_client`, and implement:

```rust
async fn cancel_or<F>(cancel: &mut watch::Receiver<bool>, future: F) -> Option<F::Output>
where
    F: Future,
{
    if *cancel.borrow() {
        return None;
    }
    tokio::select! {
        biased;
        changed = cancel.changed() => {
            let _ = changed;
            None
        }
        output = future => Some(output),
    }
}

impl LockstepClientHandle {
    pub fn shutdown(self) {
        let LockstepClientHandle { cancel_tx, input_tx, _thread, .. } = self;
        let _ = cancel_tx.send(true);
        drop(input_tx);
        if _thread.join().is_err() {
            log::error!("lockstep-client worker panicked during shutdown");
        }
    }
}
```

Wrap reconnect backoff, `KcpClient::connect`, `timeout(... join_lockstep ...)`, and the inner receive timeout with `cancel_or`; return immediately on `None` without emitting a reconnect event.

- [ ] **Step 4: Run focused lockstep tests and verify GREEN**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx lockstep_client::tests`

Expected: all lockstep client tests pass, including the pending-handshake cancellation test.

- [ ] **Step 5: Commit the lockstep lifecycle change in the `omfx` repository**

```powershell
git -C omfx add game/Cargo.toml game/src/lockstep_client.rs
git -C omfx commit -m "fix: cancel lockstep worker during session shutdown"
```

### Task 2: Joinable Simulation Worker

**Files:**
- Modify: `omfx/game/src/sim_runner.rs`

**Interfaces:**
- Produces: `SimRunnerHandle::shutdown(self)` which closes both input channels and joins the worker.
- Consumes: existing pre-start and tick-loop channel-disconnection exits.

- [ ] **Step 1: Add a failing pre-GameStart shutdown test**

Add a test that constructs the handle from in-module parts, starts a worker waiting on `master_seed_rx`, calls `shutdown`, and asserts the join completes:

```rust
#[test]
fn shutdown_joins_runner_waiting_for_game_start() {
    let (master_seed_tx, master_seed_rx) = unbounded();
    let (tick_input_tx, _tick_input_rx) = unbounded();
    let thread = std::thread::spawn(move || {
        assert!(master_seed_rx.recv().is_err());
    });
    let handle = SimRunnerHandle {
        state: Arc::new(Mutex::new(SimWorldSnapshot::default())),
        tick_input_tx,
        diagnostics: Arc::new(Mutex::new(SimRunnerDiagnostics::default())),
        master_seed_tx,
        _thread: thread,
    };
    handle.shutdown();
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx shutdown_joins_runner_waiting_for_game_start`

Expected: compilation fails because `shutdown` and the test constructor do not exist.

- [ ] **Step 3: Implement consuming simulation shutdown**

Implement shutdown by destructuring the handle, dropping both senders before joining, and logging a worker panic:

```rust
impl SimRunnerHandle {
    pub fn shutdown(self) {
        let SimRunnerHandle {
            tick_input_tx,
            master_seed_tx,
            _thread,
            ..
        } = self;
        drop(tick_input_tx);
        drop(master_seed_tx);
        if _thread.join().is_err() {
            log::error!("sim_runner worker panicked during shutdown");
        }
    }
}
```

- [ ] **Step 4: Run simulation tests and verify GREEN**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx sim_runner::tests`

Expected: all simulation runner tests pass.

- [ ] **Step 5: Commit the simulation lifecycle change in the `omfx` repository**

```powershell
git -C omfx add game/src/sim_runner.rs
git -C omfx commit -m "fix: join simulation worker on session shutdown"
```

### Task 3: Deterministic Game Session Reset

**Files:**
- Modify: `omfx/game/src/native.rs`

**Interfaces:**
- Consumes: `LockstepClientHandle::shutdown(self)` from Task 1.
- Consumes: `SimRunnerHandle::shutdown(self)` from Task 2.
- Produces: `Game::reset_session_state(&mut self)` for state that must never cross session boundaries.

- [ ] **Step 1: Extend the existing return-button test with stale state**

Seed representative stale state before clicking Return, then assert reset values:

```rust
game.current_sim_tick = 42;
game.current_round = 7;
game.round_is_running = true;
game.is_game_paused = true;
game.game_speed_multiplier = 2;
game.td_auto_start_sent_for_idle_round = true;
game.is_td_mode = true;
game.td_camera_configured = true;
game.tower_ability_bar_snapshot_at = Some(Instant::now());

assert!(game.handle_in_game_return_click(Vector2::new(20.0, 20.0)));
assert_eq!(game.current_sim_tick, 0);
assert_eq!(game.current_round, 0);
assert!(!game.round_is_running);
assert!(!game.is_game_paused);
assert_eq!(game.game_speed_multiplier, 1);
assert!(!game.td_auto_start_sent_for_idle_round);
assert!(!game.is_td_mode);
assert!(!game.td_camera_configured);
assert!(game.tower_ability_bar_snapshot_at.is_none());
```

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx in_game_return_button_click_returns_to_main_menu`

Expected: assertions fail for state currently retained across sessions.

- [ ] **Step 3: Stop workers in order and centralize state reset**

Update shutdown ordering:

```rust
if let Some(lockstep) = self.lockstep_handle.take() {
    lockstep.shutdown();
}
if let Some(sim_runner) = self.sim_runner_handle.take() {
    sim_runner.shutdown();
}
if let Some(mut backend) = self.backend_session.take() {
    backend.shutdown();
}
self.reset_session_state();
```

`reset_session_state` must clear tick timing, pending input counters/meter, round/pause/speed state, tower selections and shop dragging, TD camera flags, auto-start debounce, current ability-bar snapshot/items/rejection/interaction transient state, and Lua/session diagnostics. Preserve user preferences such as `td_auto_start_enabled`, sound volume, hotkeys, and display settings.

- [ ] **Step 4: Run native and full crate tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx in_game_return_button_click_returns_to_main_menu
cargo test --manifest-path omfx/Cargo.toml -p omfx
```

Expected: focused reset test and all `omfx` crate tests pass.

- [ ] **Step 5: Commit the game lifecycle integration in the `omfx` repository**

```powershell
git -C omfx add game/src/native.rs
git -C omfx commit -m "fix: reset game session before returning to title"
```

### Task 4: Verification and Monorepo Integration

**Files:**
- Modify: `omfx` submodule pointer in root repository
- Modify: `docs/superpowers/plans/2026-07-14-restart-game-session.md` only for checked task boxes if desired

**Interfaces:**
- Consumes: completed Tasks 1–3.
- Produces: a root commit pointing at the verified `omfx` fix.

- [ ] **Step 1: Format and check the touched crate**

Run:

```powershell
cargo fmt --manifest-path omfx/Cargo.toml --all -- --check
cargo check --manifest-path omfx/Cargo.toml -p omfx
```

Expected: both commands exit 0 with no formatting diff or compiler errors.

- [ ] **Step 2: Run the full frontend test suite**

Run: `cargo test --manifest-path omfx/Cargo.toml -p omfx`

Expected: all tests pass with zero failures.

- [ ] **Step 3: Inspect scope and confirm `omfue` was untouched**

Run:

```powershell
git -C omfx status --short
git status --short
git diff --submodule=log
```

Expected: `omfx` is clean; root shows only the intended `omfx` pointer update plus the pre-existing `? omfue` status.

- [ ] **Step 4: Commit the verified submodule pointer**

```powershell
git add omfx docs/superpowers/plans/2026-07-14-restart-game-session.md
git commit -m "fix: allow restarting game after returning to title"
```

- [ ] **Step 5: Report manual verification path**

Launch with `run.bat`, start any TD map, return to title before and after GameStart, then immediately start another map. Expected: the replacement session receives GameStart without a duplicate-player timeout and begins at tick/round zero.
