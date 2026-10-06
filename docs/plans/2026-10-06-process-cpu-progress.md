# Process CPU progress (2026-10-06)

Self-contained module only. No world, hash, ABI, or wire changes. `pub mod process_cpu` is intentionally not added here; Codex wires the module and runs tests separately.

Primary follow-up: module is now exported and integrated into opt-in fixed-step diagnostics. Actual native FFI / arithmetic review completed; `cargo test --manifest-path omoba-core/Cargo.toml comp::process_cpu::tests --lib` ran 7 tests, all passed, 487 filtered, including Windows current-process query. Runtime compiled-content-only cargo check and release build passed. Three pre-existing td_rounds dead_code warnings remain. The author's "not compiled/tests not executed" statement below describes the handoff, not the current accepted result. No 50 ms performance-pass claim.

Primary concrete fixture correction after real sampling: valid CPU delta can be zero, so query-sanity no longer assumes the first current-process total must be positive. Availability remains Some versus None; 100 ns encoding does not guarantee a short step produces a nonzero delta. No production API change.

## API

```rust
pub fn process_cpu_ns() -> Option<u128>;
pub fn process_cpu_delta(start: Option<u128>, end: Option<u128>) -> Option<u128>;
```

- `process_cpu_ns`: current-process kernel + user CPU time in nanoseconds, or `None`.
- `process_cpu_delta`: `end - start` when both samples exist and `end >= start`. Equal samples return `Some(0)`. Missing sample or `end < start` returns `None` (`checked_sub`).

Private helper (not part of the public API): `filetime_pair_to_ns(kernel: u64, user: u64) -> Option<u128>` sums the two 100 ns counters with `checked_add` and scales with `u128 * 100`. `u64` overflow returns `None`.

## Files

| Path | Action |
|---|---|
| `omoba-core/src/comp/process_cpu.rs` | Created. Module was absent. |
| `docs/plans/2026-10-06-process-cpu-progress.md` | Created. |
| Any other file | Not edited. |

`omoba-core/src/lib.rs` already gates `pub mod comp` with `not(target_arch = "wasm32")`. This change does not modify that gate. Until `pub mod process_cpu;` exists under `comp`, this file is not part of the crate graph.

## Platform behavior

Windows (`cfg(windows)` only):

- Raw `kernel32` FFI, no new dependency. Symbols: `GetCurrentProcess`, `GetProcessTimes` (`extern "system"`, `link_name`).
- `GetCurrentProcess` supplies the pseudo-handle (`-1`). It is not closed.
- `GetProcessTimes` out-params are creation, exit, kernel, user `FILETIME` (`repr(C)`, low dword then high dword). Only kernel and user are read, and only after a non-zero `BOOL`.
- Each `FILETIME` becomes a `u64` counter: `(high << 32) | low`. Kernel + user use `u64::checked_add`. The sum is `u128` multiplied by 100 (100 ns units to nanoseconds).
- `BOOL == 0` returns `None`. Outputs are not read on failure. `GetLastError` is not consulted.
- Checked counter-sum overflow returns `None`.

Every other target, including wasm (where `comp` is not built): `process_cpu_ns` returns `None`. No fallback clock, no `libc` clock, no wall clock.

The value is process-aggregate CPU across all threads (kernel plus user). It may exceed wall-clock time. The module does not subtract it from wall time and does not report wait time.

## Tests in the module

Pure (all targets):

- `delta_positive`
- `delta_zero`
- `delta_regression_is_none`
- `delta_missing_is_none`
- `delta_large_counters` (`u64::MAX` boundary and `u128::MAX` neighborhood, including regression)
- `filetime_pair_scales_by_100_and_rejects_overflow`

Windows only, read-only: `current_process_query_readonly_sanity` calls `process_cpu_ns` twice on the current process. It expects `Some`, non-zero, a multiple of 100, and a non-decreasing second sample. It does not change priority, handles, or threads.

Non-Windows: `process_cpu_ns_unsupported_platform_is_none`.

## Errors

No authoring error. The module was not compiled. Tests were not executed. This note does not claim that tests passed or that `cargo test` was run.

Not done here, on purpose: `pub mod process_cpu` in `omoba-core/src/comp/mod.rs`, crate build, test run, commit, push.
