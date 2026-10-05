# Lua role recipe → Unreal launcher（2026-10-05）

## Plan and decisions

1. Reuse the production role-plan compiler and server decoder; no second Lua gameplay validator, no copied two-player smoke scenario.
2. Fixed Lua host uses the full TOML parser. Replace complete selected field values (especially AUTH tables), preserve unrelated sections and values. Source TOML and Lua are never overwritten. Formatting/comments in the generated copy need not match the source.
3. New unique run directory receives absolute content paths, immutable recipe JSON, human-only authorization, 60 Hz and normal speed. Bot membership never creates a client process.
4. One server plus one runtime and presentation-only Unreal per human. IPC ports are assigned by ordinal, not team/ID, supporting multiple humans on the same team. Profile is explicit; no release-to-debug fallback.
5. Runtime listener readiness is insufficient: require the admitted player/team replica ready line before Unreal starts. Starting an Editor process is not renderer readiness, stable 60 FPS, or match acceptance.
6. Reuse build_ue_moba.lua --build-only and its restart/build guard, then build matching-profile server/runtime and verify bridge staging. No full MCP/Blueprint/smoke acceptance in this increment, no automatic termination of another Editor.
7. Cleanup only owned PIDs, executable-identity checked, in reverse order; wait for exit and propagate errors. Persist each spawned PID immediately. Lifecycle/error records distinguish cleanup failure from successful launch. No prior-session sweep.

## Entry points

```text
tools/lua/lua.exe scripts/run_moba_role_ue.lua --prepare-only
tools/lua/lua.exe scripts/run_moba_role_ue.lua
tools/lua/lua.exe scripts/run_moba_role_ue.lua --recipe scripts/lua_data/moba_single_player.lua --profile debug --ue-root D:/UE5.8
tools/lua/lua.exe scripts/test_moba_role_launch.lua
```

The normal command builds and starts interactive processes; not executed in this increment. --prepare-only builds/runs only the configuration preflight, without loading a DLL, constructing World, opening game/IPC sockets or spawning server/runtime/Unreal. --no-build is an explicit advanced opt-out, not a fallback; staging is still checked.

Generated launch-plan.json is prepared-not-launched. lifecycle.json is process-lifecycle-only, not gameplay evidence. Generated files remain below target/ and are not committed. Changing Lua bot=false selects humans without Unreal/C++ edits. The example story is FOG_2TEAM_DEMO, the role plan owns three_lane_training.

## Current-function evidence

- cargo test --manifest-path tools/lua-host/Cargo.toml toml_sections: 2/2 passed. Complete TOML parsing preserves multiline strings, arrays and unrelated values; rejects invalid source/update documents and scalar path collisions; authentication field replacement removes stale entries.
- Real fixed-Lua --prepare-only passed: one human, nine Bots, 60 Hz. This also compiled and executed the new moba-config binary through the production Setting decoder.
- Lua functionality tests: 5/5 passed. Real production preflight covers one human, same-team two humans, ten Bots and an invalid recipe; source game.toml bytes unchanged. Argument/readiness checks and mocked lifecycle cover reverse stop/wait and surfaced cleanup failure with errors.md. No actual game processes were spawned by these tests.

## Remaining

The launcher implementation is connected, but actual gameplay, renderer IPC readiness, nine-Bot skills/match lifecycle and complete acceptance are not proven by configuration or mock tests. OpenSpec remains 20/30; 5.5 stays unchecked (three hero archetypes/100 matches also remain). Final acceptance follows completion of the outstanding framework work, rather than repeating it for each increment.
