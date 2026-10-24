# E1529 — channel native animation

## Gap and prevention

Accepted channel world VFX and progress still leave the skeletal actor in normal locomotion. Use shared semantic channel bindings and existing clips under configured fallback; exact cursor comes from authoritative elapsed fraction, not a local animation timer. Full opaque keys must remain valid. World scope must be checked before native epoch reset to avoid foreign old epochs mutating current state. Death/respawn, recall and preparation retain priority; hide/rebind/reset retire cached poses. Avoid hero/item names or treating preparation as sustained channel gameplay.

Native character captures require a verified live channel animation and actual mesh refresh before capture. E1528's rejected T-pose remains preserved in the prior evidence; do not inherit a character screenshot after changing its active pose. This batch uses path-only runner replacement and preserves exact native test IDs/fixture flags per E1527.

## NativeWorldV1 actual fixture epoch collision

ChannelNativeAnimation failed the first-state assertion for the second hero. The first hero completed all lifecycle tests and three pose PNGs, advancing the shared world epoch from100 to101 and its tick to4. The second hero started at epoch100+H=101/tick1, so the new world scope gate correctly rejected the backwards tick. This is a fixture scope collision, not a reason to weaken admission. Preserve the complete raw77-test cohort and all original images plus four inputs-v1 before repair. Wait/reap WorldV1 and its upstream-gated source reader, then modify only the new test: reserve distinct per-hero epoch ranges (100+H*10), log hero/epoch/tick before admission, rebuild and run freshWorldV2. All three other native inputs must remain unchanged. Prevention: multi-actor lifecycle fixtures must allocate disjoint world epoch ranges when each actor advances the shared scope during testing.

## Accepted closure

UE5.8 buildV2/staged DLL/import SHA passed. WorldV2 executed77:76success+1existingwarningcase,0failed/0notrun. Source guard verified3442 inputs; full accepted Rust1844passed/0failed/9existingignored and backend result inherited after SHA proof3438 unchanged prior inputs+3 native changes+1 addition; no full Rust rerun claimed. Archive contains397 independently reverified files; 44 final accepted PNGs, with failed historical captures also retained. Health5 samples/max171 seconds. Four final workers and four historical workers (2successful/2failed) have creation-token-bound retired receipts. All launchers, two inheritance readers and the single170-second health monitor were reaped; CIM found no owned build/test/source worker remaining.

11 new/changed images actually viewed; 33 accepted prior SHA-identical images inherited. Three actual512x512 early/mid/late pose images and the combined turquoise-ring/character image show sampled channel posture. Existing sustained asset movement is subtle; exact normalized cursors are asserted on two real skeletons. No bespoke animation assets or new channel gameplay are claimed.

NativeWorldV1 correctly failed the second-skeleton first-state assertion because the fixture reused epoch101/tick1 after the first skeleton advanced that shared world epoch to tick4. Original raw77-test run/report/all available images, false World/source receipts and4 exact inputs-v1 remain archived. Fixture-only repair reserves100+H*10 epoch ranges and logs initial scope; all3 other native sources unchanged. Final buildV2/fresh fullWorldV2 use inputs-v2. Both real skeletons completed with initial epochs100 and110. E1529 records root cause and prevention; production admission was not weakened.
