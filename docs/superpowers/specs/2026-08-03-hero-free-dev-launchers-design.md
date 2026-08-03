# Hero-Free Dev Launchers Design

## Goal

`run.bat` and `run_10000.bat` must start TD sessions without creating any Hero entities. Other launchers and callers must retain the existing campaign-driven hero initialization unless they explicitly opt into the same behavior.

## Selected Approach

Introduce the process environment flag `OMB_NO_HEROES=1`, following the existing `OMB_NO_CREEPS=1` development-override pattern.

- `run.bat` sets `OMB_NO_HEROES=1` before starting the frontend session.
- `run_10000.bat` also sets `OMB_NO_HEROES=1` before calling `run.bat`, making its intended behavior explicit even though the called launcher sets the same value.
- `omoba-core` checks the flag at the campaign hero creation boundary. When the value is exactly `1`, hero creation returns before creating entities or enqueueing hero spawn events.
- Any other value, or an absent variable, preserves current behavior.

This keeps the mission data unchanged and scopes the override to the requested launchers. It also ensures the frontend local simulation and the launcher-owned backend inherit the same value, preserving lockstep initialization.

## Runtime Behavior

When `OMB_NO_HEROES=1`:

- No entity receives the `Hero` component during campaign initialization.
- No hero `ScriptEvent::Spawn` is queued.
- No hero receives `PlayerOwner`, `Gold`, inventory, combat, collision, or script components.
- Creep waves, towers authored in the map, lives, and other TD resources initialize normally.
- The runtime emits one informational diagnostic explaining that hero creation was skipped.

The current economy and tower input handlers locate the requesting player's Hero entity. Therefore hero-free sessions cannot manually build, upgrade, or sell towers. This limitation is accepted for these two launchers and is not addressed by this change.

## Files and Boundaries

- `run.bat`: set the opt-in environment flag while retaining CRLF and UTF-8 without BOM.
- `run_10000.bat`: set the same flag alongside the starting-gold override, retaining CRLF and UTF-8 without BOM.
- `omoba-core/src/runtime/native/initialization.rs`: centralize the exact flag check at `create_campaign_heroes`.
- Focused initialization tests: inject the resolved policy into the creation boundary so tests do not mutate process-global environment variables, then prove enabled and default behavior.

No mission Lua, hero templates, frontend selection UI, or other root launcher is changed.

## Verification

1. A focused initialization test with `OMB_NO_HEROES=1` produces zero Hero entities and no hero spawn event.
2. A control test without the flag preserves the current TD hero count.
3. Relevant `omoba-core` and backend tests pass.
4. Both batch files contain only CRLF line endings and no UTF-8 BOM.
5. A `cmd.exe` launcher smoke reaches the first freshness step without the `'M' is not recognized` parsing error; the smoke may stop before launching the interactive frontend.

## Non-Goals

- Moving player gold or ownership out of Hero entities.
- Supporting tower input in hero-free sessions.
- Removing heroes from mission data.
- Changing `run_2player.bat` or `run_ue.bat`.
- Changing hero selection or hero knowledge UI.
