# Errors and prevention

- Final read-only provenance audit guessed native/tick/script_tick.rs; that file does not exist. No source/build mutation occurred. Stop synthesizing paths from system names: discover the actual file with rg --files before a targeted read/search. The source snapshot and acceptance inputs remain unchanged.

- Read-only search passed a Windows wildcard directly as an rg file argument (os123). No mutation occurred. Use directory plus --glob, or rg --files discovery.
- GameWorld stable method table is full (documented world.rs). Use an explicit typed PathSpec variant in the already shared projectile ABI; rebuild both consumers together instead of appending an unsupported virtual method.

- Edit V1 failed before writing game_processor.rs: a broad match also transformed the test constructor in memory, so its later exact match failed. Earlier files were already written. V2 applies only that still-original file with owner-variable vs owner-field discrimination; then completes docs. Original failed helper preserved; no blind rerun or source rollback.

- RustV1 failed:341 base tests passed/1 new formal failed at mode3 actual HP10148740 vs expected10147840 raw. Core1007 passed. Fixture zeroed Unit.base_armor but target is Hero, whose authority raw settlement reads CProperty.def_physic. Remaining armor mitigated 900raw damage. After actual Windows1 receipt, owner/waiter retirement and API94946 reap, V2 changes ONLY fixture CProperty.def_physic=0. Prevent recurrence: fixtures must set the authority defense selected by target type; never change damage implementation or weaken HP expectations to hide mitigation. Preserve V1 source copies/logs/receipt, require11unchangedruntimefiles+1fixturedelta and completefreshV2.
