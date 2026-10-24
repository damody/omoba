# Control item feedback errors and prevention

## E1531 — repeated read-only path discovery mistakes

Early investigation guessed the WorldBridgeActor source under OmRuntime rather than its actual maintained OmGenerated path, and guessed bridge/include/om_bridge.h. Several searches incorrectly put Windows wildcards in path arguments (OmCastPreview*, OmPublicUnit*, omoba*). These reads/searches failed without mutation. Discover exact paths using `rg --files`; use `-g` globs on an existing directory, never wildcards in Windows filesystem arguments. Actual WorldBridgeActor is a maintained native source in OmGenerated; codegen does not rewrite this actor. Do not conflate that directory with the four generated catalog/hash outputs.

At the initial freeze no build/test failure had occurred. Subsequent failures and repairs are recorded below. Preserve failure evidence before repair and never overwrite versioned roots.

## E1531.1 — new native fixture omitted its test friend

BuildV1 failed with MSVC C2248 because FOmControlItemTargetFeedbackTest used the same private automation setup/lifecycle as existing FOmItemCastFeedbackTest without its own WITH_DEV_AUTOMATION_TESTS friend declaration. All8 exact inputs-v1, failed raw build-v1.log, false retired creation-token receipt and successful initial source inheritance proof are preserved. No native test run started. Build launcher was reaped and the inheritance reader retired/reaped before the repair.

Repair: one additional friend declaration inside the existing test-only guard; private access remains private in product builds. All7 other frozen inputs stay byte-identical. Freeze inputs-v2, rerun exact inheritance proof and fresh BuildV2 before native acceptance. Prevention: when reusing a private native fixture, inspect existing friend declarations before adding a new test class; never use a public-access workaround or weaken production admission to fix fixture compilation.

## Final validation

Fresh BuildV2 passed; staged DLL/import hashes matched. NativeWorldV1 completed79:78success+1existingwarningcase,0failed/0notrun,warnings4. Full source check verified3444 inputs. Exact inheritance proof binds3436 unchanged prior inputs+7 maintained native deltas+1 native test addition to accepted Rust1849passed/0failed/9existingignored and backend; no new full Rust rerun claimed. Nine new/changed images actually viewed and37 prior SHA-identical images inherited. Failed BuildV1 and both exact source cohorts remain preserved. All final and historical build/test/source workers retired/reaped before sealing.
