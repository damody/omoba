# Active Task 3 Report — Deterministic Tower Ability Scheduler

## Result

Added deterministic Fixed64 state and queueing for tower active abilities.

- `TowerActiveAbilityState` owns cooldown, active-window, pulse timing, pulse
  charges, a monotonic non-zero activation serial, and the next pulse index.
- `activate` rejects casts while cooling down and advances the state-owned
  activation serial only for accepted casts.
- `advance` saturates cooldown/window timers, treats zero or negative `dt` as
  paused, and emits at most one pulse opportunity per call.
- `acknowledge_pulse(false)` preserves the charge and index while the consumed
  interval prevents same-tick retry spin; the next interval retries it.
- `tick_tower_abilities` advances ECS towers and queues deterministic records;
  script dispatch and acknowledgement remain deliberately outside Task 3.
- `PendingTowerAbilityPulseQueue` is installed with the other runtime queues.

The existing `Tower.ultimate_cooldown` remains untouched and independent from
the new active-ability state.

## TDD Evidence

All commands ran from
`D:\code\omoba\.worktrees\three-tower-active-abilities`.

### Scheduler/state API RED

`cargo test --manifest-path omoba-core/Cargo.toml tower_ability_tick`
failed to compile on the intentionally missing `TowerActiveAbilityState`,
`PendingTowerAbilityPulseQueue`, `tick_tower_abilities`, and
`Tower.active_ability` APIs.

### Scheduler/state GREEN

After the minimal state, queue, ECS wrapper, and initialization were added, the
same focused command passed all original 9 tests covering ready, activation,
duplicate rejection, zero-dt pause, uneven exact pulses, negative ack, expiry,
cooldown saturation, and ECS queue records.

### State-owned serial RED/GREEN

- The new four-argument progression test first failed to compile because
  `activate` still accepted a caller-provided fifth serial argument.
- After moving serial generation into the state, the focused suite passed and
  asserted accepted casts produce serials 1 then 2.
- The wrap test then failed with actual serial 0 after `u32::MAX`; the minimal
  fix skips reserved zero and produces 1.

Final focused result: 11 passed, 0 failed.

## Verification

- `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_tick` —
  11 passed, 0 failed.
- `cargo check --manifest-path omoba-core/Cargo.toml` — passed.
- Scoped `rustfmt --check` with `skip_children=true` for all Task 3 Rust files
  — passed.
- `git diff --check` — passed (only expected line-ending notices).
- Full `cargo test --manifest-path omoba-core/Cargo.toml` — 134 passed and one
  pre-existing unrelated fixture failure; see Concerns.

## Commit

Commit subject: `feat(core): add deterministic tower ability scheduler`.

## Self-review

- All timing math stays in `Fixed64`; no floating-point conversion is used.
- Cooldown starts at accepted cast time and advances concurrently with the
  active duration.
- A due opportunity subtracts exactly one interval, so large/uneven ticks do
  not emit multiple records from one call.
- Only a consumed acknowledgement decrements the charge and advances the
  zero-based pulse index. A false acknowledgement cannot spin in the same tick.
- Activation serial is authoritative state, increments only after validation,
  uses wrapping arithmetic, and reserves zero for never-activated state.
- Queue records copy entity, ability ID, activation serial, and pulse index;
  they do not dispatch scripts or reuse the old automatic-proc cooldown.
- `next_pulse_index` is serialized with a default so reconnect/older snapshots
  remain compatible while providing stable zero-based script indices.

## Concerns

- The repository-wide core suite retains the pre-existing unrelated
  `runtime::native::tick::creep_wave::tests::td_wave_clear_awards_btd_easy_round_income_to_heroes`
  failure because its test world does not insert `PendingDebugCreepSpawnQueue`.
- Full-project `cargo fmt --check` also reports unrelated pre-existing format
  drift in files outside Task 3; scoped checks for every Task 3 Rust file pass.
- Task 4/5 integration must drain/dispatch queued records before advancing the
  same tower again and feed the script result to `acknowledge_pulse`.

---

## Review Follow-up — Oversized `dt` Backlog

### Root cause

The first implementation removed only one interval from `pulse_accumulator`
per `advance` call. Once that same oversized call saturated
`active_remaining` to zero, later calls bypassed accumulator processing, so
the remaining crossed intervals were stranded permanently.

### RED

Added regressions for:

- a 3-second call over a 0.5-second interval quantizing six due pulses and
  draining all six after the active window reaches zero;
- one call crossing multiple intervals while emitting only one opportunity;
- an outstanding opportunity suppressing duplicate emission before ack;
- false ack consuming its due attempt without consuming the charge, followed
  by backlog retry and final unused-charge expiry;
- window expiry before any interval expiring all unused charges.

`cargo test --manifest-path omoba-core/Cargo.toml tower_ability_tick` failed to
compile on the intentionally missing `pending_due` and
`opportunity_outstanding` state.

### GREEN

Added serde-defaulted bounded state fields, reset them during activation, and
quantized crossings with a single `i128` division/remainder operation. New due
attempts are capped by `pulses_remaining - pending_due`; no elapsed-time loop
is used. Focused result: 15 passed, 0 failed.

Ack semantics now are:

- `true`: clears the outstanding marker, consumes one pending due attempt and
  one charge, then advances the successful pulse index;
- `false`: clears the outstanding marker and consumes one pending due attempt,
  but retains charge/index for another already-due or future interval;
- once the active window and due backlog are both empty, any unused charge
  expires.

### Follow-up verification

- Scoped `rustfmt --check` — passed.
- `cargo check --manifest-path omoba-core/Cargo.toml` — passed.
- Focused scheduler suite — 15 passed, 0 failed.
- `git diff --check` — passed with expected line-ending notices only.
- Full core suite — 138 passed; the same pre-existing unrelated creep-wave
  fixture failed because it omits `PendingDebugCreepSpawnQueue`.

### Follow-up self-review

- `pending_due <= pulses_remaining` is maintained: quantization caps additions,
  true ack decrements both, false ack decrements only pending due, and expiry
  zeroes charges only when pending/outstanding are empty.
- `opportunity_outstanding` prevents duplicate records without dropping newly
  elapsed interval crossings; crossings can continue accumulating bounded due
  state until dispatch acknowledges the outstanding record.
- Active duration contributes only its saturating remaining portion of `dt`,
  while cooldown consumes the full positive `dt` concurrently.
- Older serialized states default both new fields to zero/false; activation
  explicitly clears them for reuse.

Follow-up commit subject: `fix(core): preserve tower pulse backlog`.

---

## Re-review Follow-up — Transient Outstanding Marker

### Root cause and RED

`opportunity_outstanding` described a transient queue record, but the first
backlog fix serialized it with the durable tower state. The queue itself is not
serialized, so restoring `true` without its record prevented every later
`advance` from re-emitting that pending due opportunity.

The serde round-trip regression created a state with two pending due attempts
and one outstanding opportunity. It failed because the encoded JSON still
contained `opportunity_outstanding`; deserialization would restore the deadlock.

### GREEN

Marked `opportunity_outstanding` with `#[serde(skip, default)]`. Durable
`pending_due` remains serialized, while reload always resets the transient
marker to false. The next positive `advance` re-emits pulse index 0, and a true
ack drains exactly one due attempt and one charge.

An additional compatibility regression removes both backlog fields from an
encoded payload and confirms deserialization defaults them to zero/false.

### Verification and self-review

- Focused scheduler suite: 17 passed, 0 failed.
- The serialized form no longer contains the transient marker.
- `pending_due` remains durable, so a lost queue record is reconstructible.
- Old payloads without either field retain serde-default compatibility.
- Activation still explicitly resets both durable backlog and transient marker.

Commit subject: `fix(core): recover tower pulse after reload`.
