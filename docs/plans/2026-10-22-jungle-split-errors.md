# Jungle splitting errors and prevention

J01: Camp death currently receives `&World`; spawning directly there would violate existing ECS ownership. Queue offspring definitions and corpse positions, consume only during the mutable camp tick.

J02: Counting only live entities would schedule respawn during deferred spawning. Pending offspring explicitly prevent the full-clear deadline.

J03: Recursive content must bound lifetime bodies, depth and cumulative offsets before generation. No unbounded runtime recursion or camp-name behavior switches.

J04: The temporary Lua exact-edit helper initially used `[[...]]` around Rust nested array expressions containing `]]`. Lua rejected parsing before any edit ran. Switched those literals to `[=[...]=]`; both failed invocations made no production changes. Use delimiter levels that cannot occur inside embedded source.

J05: V1 Rust/UE/backend compilation reported E0425: the recursive module helper could not access `valid_id`, previously a local closure in `validate_moba_maps`. Waited for the entire V1 Rust/UE/backend/contracts cohort and watchers to retire, then promoted the unchanged identifier policy to one shared module function. Also made the new test's hash import explicit before retry. Original failure logs are retained.

J06: V2 core had 893 passes and one failure: the new corpse-relative movement/reset test exercised movement fact emission in a minimal fixture lacking ObservableFactBuffer. Production StateInitializer already creates this resource; no production event emission was weakened. Waited until all V2 consumers/watchers retired, then inserted the default buffer into camp_world and reran the full V3 cohort. V2 base264, UE and backend successes are preserved separately. An exact patch using the wrong import-qualified VisionOccluderSet line was rejected before mutation; reapplied against the observed Outcome line.
