# Match Statistics Recording Design

## Problem

The profile screen reads local statistics from `omb/player_profile.json`, but a
match is currently recorded only when the backend observes a `game/end`
runtime event. The native frontend owns the local session lifecycle and shuts
the backend down as soon as its local replica reaches a terminal snapshot.
Observed logs show the frontend completing round 40 while the backend was still
on round 23, so the backend was killed before it could persist the match.

Consequently victory and defeat can both leave `games_played`, `wins`,
`highest_wave`, and `total_kills` unchanged.

## Scope

This change makes completed and abandoned local sessions update the four real
statistics already displayed by the profile screen:

- games played;
- games won;
- highest round;
- enemies killed.

The existing placeholder rows for current-version high round, CHIMPS,
Deflation, and towers used remain outside this change. Knowledge-point balance,
unlocking, and combat bonuses retain their existing behavior. Abandoning a
match records statistics but grants no end-of-match KP.

## Ownership Decision

The native frontend becomes the single owner of local match-statistic
settlement because it:

- creates and tears down the local session;
- knows whether teardown followed victory, defeat, or a user exit;
- displays the local profile;
- remains alive after the backend process has been stopped.

The backend continues to award KP for authoritative victory or defeat events,
but no longer mutates match statistics. This prevents an official game-end
event from being counted once by the backend and again during frontend
teardown.

## Session State

The frontend keeps a small per-session tracker with:

- whether gameplay started successfully;
- whether statistics have already been settled;
- the highest observed round;
- the latest authoritative-replica match kill count;
- an optional terminal result: victory, defeat, or abandoned.

The tracker starts only after the session has successfully entered the in-game
state. A failed launch does not count as a played game. It is reset only after
settlement or after discarding a session that never started.

`MatchKillCounter` is added to the render snapshot as read-only match metadata.
It does not alter simulation state or participate in rendering. This avoids
trying to infer kills from removed entity IDs, which also contain leaks and
other removals.

## Settlement Rules

Every successfully started session is settled exactly once:

| End condition | games_played | wins | highest_wave | total_kills |
|---|---:|---:|---|---|
| Victory | +1 | +1 | max with highest observed round | add match kills |
| Defeat | +1 | unchanged | max with highest observed round | add match kills |
| Return to title / Ctrl+Escape / application shutdown | +1 | unchanged | max with highest observed round | add match kills |
| Startup failure before entering gameplay | unchanged | unchanged | unchanged | unchanged |

Repeated teardown calls, duplicate terminal snapshots, and a terminal snapshot
followed by automatic teardown must not create another settlement.

## Data Flow

1. A successful session start initializes the tracker.
2. Each accepted render snapshot updates highest round and match kills.
3. A terminal snapshot records victory or defeat but does not write immediately.
4. Any teardown path assigns `abandoned` when no terminal result exists.
5. Lockstep and simulation workers stop.
6. The owned backend process is killed and waited for, allowing any already
   running KP write to finish first.
7. The frontend reloads the latest profile, merges one statistics settlement,
   and persists it.
8. The frontend refreshes its profile cache and resets session state.

External backends are not killed by the frontend, but local statistics are
still settled once by the frontend. Backend match-statistic mutation is removed
for both owned and external modes so ownership remains unambiguous.

## Persistence and Errors

Profile parsing keeps the existing serde defaults so old files without the new
statistics remain valid. Arithmetic uses saturating increments.

The profile update serializes the complete merged profile before touching the
destination, writes a same-directory temporary file, and replaces the profile
with a platform-safe atomic replacement. If serialization or replacement
fails, the previous profile remains readable and the failure is logged with the
session result and target path. Teardown must continue even when persistence
fails.

The implementation does not silently create a second settlement in the same
process after a write error. This favors at-most-once counting over a possible
double count; the error remains visible in logs.

## Tests

Unit tests cover:

- victory increments played and wins once;
- defeat increments only played;
- abandoned sessions increment only played;
- victory followed by teardown is not double-counted;
- repeated teardown is not double-counted;
- startup failure is not counted;
- highest round is monotonic;
- match kills are added with saturation;
- legacy JSON without statistic fields is upgraded on write;
- an existing KP update is preserved when statistics are merged;
- a failed profile replacement leaves the previous file valid.

Snapshot tests verify that `MatchKillCounter` is copied without mutation.
Backend tests verify that game-end handling awards KP without changing the four
statistics. A focused frontend lifecycle test covers the exact sequence that
previously failed: terminal local snapshot, backend shutdown, profile update,
and profile cache reload.

## Non-Goals

- Counting application launches as matches.
- Granting KP for abandoned matches.
- Implementing per-mode high-round rows.
- Replacing the local JSON profile with an account service.
- Reconciling a frontend replica and backend that have progressed to different
  rounds; the player-visible local session is the statistic source for this
  local profile.
