# Pregame UI Catalog

`catalog.json` is the script-owned source for the native `omfx` pregame flow.
Content mods should edit or replace this catalog and its assets instead of
hard-coding map, difficulty, label, or action data in `omfx`.

Supported widget action kinds:

- `Navigate` with `target`
- `Back`
- `SelectMap` with `map_id`
- `SelectDifficulty` with `difficulty_id`
- `StartSession`
- `NoOp`

Each enabled map must provide a stable `id` and `story` or `runtime` value.
That story/runtime value is used for both backend launch metadata and the local
`sim_runner` scene path. Each enabled difficulty must provide an `id`; `config`
is passed to the backend as `OMB_DIFFICULTY` and defaults to the difficulty id.

Invalid or unknown actions are treated as `NoOp` and logged by `omfx`. Missing
image paths are also logged, but the menu remains usable when action data is
valid.
