# Restartable Game Session Lifecycle Design

## Problem

Returning from an active game drops the frontend lockstep and simulation handles, then stops the backend. The lockstep worker can still be blocked for up to ten seconds in `join_lockstep`, however, because dropping its handle does not cancel or join its thread. Starting another game during that window creates a second client with the same player id. The stale client and replacement then race to join, and the backend can reject the replacement as a duplicate player.

The observed log sequence confirms this: a session is returned from before receiving `GameStart`, a new session starts, and the old worker later reports `join_lockstep timed out ... duplicate player_id=1` at the original ten-second deadline.

## Goals

- Returning to the menu deterministically stops all frontend workers before another session may start.
- A lockstep worker blocked in connection, retry backoff, join, or receive exits promptly when cancelled.
- The simulation worker is joined after its input channels are closed.
- Per-session frontend state is reset so the next game cannot reuse stale tick, input, selection, or transient UI state.
- Add regression coverage for promptly stopping a lockstep client while it is waiting for the server handshake.
- Do not modify `omfue`.

## Design

### Explicit lockstep cancellation

`LockstepClientHandle` will own a cancellation sender and its worker `JoinHandle`. Its consuming `shutdown` method will signal cancellation and join the worker thread. The asynchronous client loop will observe cancellation during every potentially long wait:

- reconnect backoff;
- KCP connection;
- the ten-second `join_lockstep` handshake;
- the connected receive loop.

Cancellation is terminal and must not emit a disconnected/retry event. Normal network failures retain the existing reconnect behavior.

### Deterministic simulation shutdown

`SimRunnerHandle::shutdown` will consume the handle, close the master-seed and tick-input senders, and join the simulation thread. Both the pre-GameStart wait and active tick loop already exit when their channels disconnect; joining makes that exit a lifecycle guarantee instead of a detached background operation.

### Session shutdown order

`Game::shutdown_game_session` will take and stop the lockstep worker first, then take and stop the simulation worker, then terminate and wait for the owned backend process. This prevents an old client from surviving into the replacement backend's lifetime.

After workers and backend are stopped, the method will reset connection status and all session-scoped tick, input, selection, drag, ability feedback, and auto-start debounce state needed by the next session. Returning to the title will continue to delegate menu-state reset to `PregameRuntime::return_to_menu`.

### Failure handling

Worker thread panics during shutdown will be logged and will not prevent the remaining session resources from being stopped. Starting a session remains prohibited while any session handle is present.

## Testing

- Add a lockstep lifecycle regression test using a local TCP/KCP test endpoint that accepts connection progress but never completes `GameStart`; cancellation must finish well below the existing ten-second join timeout.
- Add simulation lifecycle tests for shutdown before `GameStart` and, where practical, after startup.
- Extend native session reset tests to assert session-scoped state is cleared when returning.
- Run focused `omfx-game` tests and formatting/lint checks appropriate to the touched crate.

