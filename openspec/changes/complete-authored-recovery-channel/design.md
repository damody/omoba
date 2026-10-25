# Design and batch plan

1. Preserve sealed summon-guidance inputs and all native generated directory files before editing.
2. Add a closed exclusive self recovery-channel effect with rank keys for pulse amount/duration/interval. Validate representable bounded values and exact pulse divisibility; channeled/no-target/active-or-ultimate only.
3. Publish closed HostAbilityExecution in a serde-defaulted AbilityDef.host_execution JSON field, keeping the full GameWorld stable method table unchanged. EffectSpec/effects_preview remain descriptive and never execute gameplay. Generic direct script invocation fails closed; native dispatch executes the explicit host-owned declaration rather than calling a no-op script.
4. Stage an internal outcome, admit it in the existing atomic cast transaction before mana/cooldown commitment, then start the shared recovery clock. Recheck exact owner/life/phase/position/control; reject mixed arbitrary mutations in a channel transaction.
5. Reuse command/damage interruption. Cancel queued channeled casts on later accepted commands, including same-frame Stop, using authoritative cast metadata rather than names. Preserve pause, root/non-moving casts, healing reduction and exact-generation behavior.
6. Convert apprentice_mend authoring to that effect. Add schema/default/error tests, real authored formal60Hz resource/pulse/interruption/replay checks and native metadata/presentation checks. Consolidate builds and source consistency after functionality is implemented.
7. Preserve original failures, actual exit/API reaps/retirement and reviewed visual evidence; update tasks/errors/audit/handoff. No true network match, quota actions, commits or historical frontend work.
