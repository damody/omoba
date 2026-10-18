# Owned retirement progress — 2026-10-07

## Symptom recorded from the selection run

Selection-only Unreal fixture reached ready/select/lock/finalize after about 3 minutes, past the old 45s bound. The exact process identity query then failed with access denied on natural process exit. Report: elapsed 242016ms, success false. Original PID 31232 was absent, and the workflow supervisor had retired descendants.

Source already settled a failed query with a zero-duration `WaitForSingleObject` on the retained handle. That check still observed the process as alive (`WAIT_TIMEOUT`), so the access-denied query error was returned unchanged. The runtime binary was already fresh before this change.

## Change

`live_owned_identity` still calls `settle_owned_query`. The pure helper contract is unchanged: success returns the value and does not call the probe; a signalled probe returns `None`; a live probe returns the original query error; a probe error stays an error.

Only the Windows failure probe changed. After the identity query fails, it waits up to `OWNED_QUERY_RETIREMENT_SETTLE_MS` (1000) on the same retained handle.

- `WAIT_OBJECT_0` (0) → `None` (retired)
- `WAIT_TIMEOUT` (258) → original query error, including access denied
- `WAIT_FAILED` and any other code → error, not retirement

Successful queries still take the zero-duration path only. `inspect_owned`, `stop_owned`, and `close_window_owned` still require the exact handle, creation token, and executable match before any stop or close. This does not reopen by PID or name, does not treat access denied on a live process as death, and does not change frontend auth.

## Verification actually run

`cargo test --manifest-path tools/lua-host/Cargo.toml owned_query_race -- --nocapture`

- Compile: `test` profile finished in 18.65s
- `running 9 tests`, result `ok. 9 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.02s`
- `immediate_exact_handle_signal_elapsed_ms=0` and the failed query settled to `None`
- `delayed_exact_handle_signal_elapsed_ms=251 result=Ok(None)` on this process's own `findstr.exe` handle, stdin closed after 250ms
- `live_timeout_elapsed_ms=1001 error=cannot query exact process identity: Access is denied. (os error 5)` and the own child was still running
- `wait_failure_elapsed_ms=0 error=access denied; cannot settle lifetime: cannot query owned lifetime: 控制代碼無效。 (os error 6)`
- `successful_live_identity_elapsed_ms=0 identity={"alive":true,"created_unix_seconds":1791362710,"creation_token":"134358363105129055","path":"C:\\Windows\\System32\\findstr.exe","pid":44736}`
- `exited_own_child_elapsed_ms=0 settled=Ok(None)` through `live_owned_identity` on the spawn handle of our own `cmd.exe /c exit 0` after `Child::wait`

`cargo build --manifest-path tools/lua-host/Cargo.toml`

- `Finished dev profile [unoptimized + debuginfo] target(s) in 7.74s`
- `D:\omoba\tools\lua-host\target\debug\omoba-lua-host.exe`, length 2241024, last write 2026/10/7 16:45:29

## Left for primary review

No selection-fixture rerun, no Unreal Editor or game build, and no commit. The 1000ms bound is only large enough to cover handle rundown after the query has already failed; a process that is still alive at 1000ms still surfaces the original query error.

## Primary independent review

Grok 4.7 Fast job `run-muxuxdyh-15vd2j`, thread `421a994b-811d-4517-8762-1d3c20736b90`, completed 352s / 12 calls / USD0.44945552. Primary inspected the source diff and independently ran the native regression: 9 passed, 0 failed, 1.03s; delayed signal 252ms, live query failure 1006ms, successful identity 0ms. `process_owned_wait_test.lua` passed 10/10; `process_identity_live_test.lua --run-owned-fixture` passed exact-token rejection and owned-child cleanup. Diff whitespace check passed. Real Unreal selection-only v2 is being checked separately; these tests alone do not accept the full selection fixture.

Final actual acceptance: `ue_selection_startup_validation.lua --output target/role-ue-runs/machine-selection-startup-budget-v2 --ue-root D:/UE5.8 --graphics d3d12 --profile release --editor-ready-timeout-seconds 3600 --selection-timeout-seconds 3600` exited 0. Report success=true, cleanup_verified=true, elapsed150453ms, three native selection callbacks, original renderer retired. Zero authority matches. Source contract and actual current feature accepted; no claim that the 1000ms wait was exercised in this successful run, nor that full natural match/LAN/performance passed.
