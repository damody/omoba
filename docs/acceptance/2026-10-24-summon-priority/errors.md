# Summon movement priority errors

SP01 discovery: literal Windows wildcard paths ability_runtime* and hero_command*/hero_summon* were passed to rg; these are invalid/unavailable paths. Subsequent guessed headless_world.rs and native/filtered_specs.rs paths were absent. No production mutation occurred. Use rg --files on the known containing directory, then exact returned paths or --glob; never infer a filename from a module alias. This repeats the previously recorded discovery mistake and must not be treated as a gameplay failure.

Identified gameplay gap: summon_tick advances active MoveTarget before the summon script executes; parallel adapter advance_with_collision had root guarding but no host MoveTarget guard. A disclosed enemy in aggro range could add a second lateral chase step while a host destination remained active. Fix at the shared adapter boundary, not the named gunner script.
