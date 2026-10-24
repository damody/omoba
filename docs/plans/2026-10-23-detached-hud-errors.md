# Detached HUD errors and prevention

## E1532 — investigation

Known accepted screenshots exposed a cropped standalone first-kill HUD. The combat column starts at480 logical pixels, while full content needs a measured reflow. NativeTick normally invokes that reflow, but a worldless UUserWidget hosted in an SWindow does not reliably receive a world tick. The Slate normalizer already measures/arranges actual geometry; publish it during Slate Tick through a weak callback to the same memoized layout helper. Do not special-case screenshot filenames or manually position individual HUD elements. No build/test error has occurred yet. Preserve original failure evidence before repairs.

## E1532.1 — ambiguous edit context caught before freeze

An identical Slate.AddWindow/tick statement existed in multiple tests. The first edit matched an unrelated fixture instead of the first-kill fixture; immediate line-context inspection caught it. Reverted that unrelated change and anchored the intended edit on its unique window title. No reader/build/test had started. Prevention: use unique surrounding context for repeated fixture statements and inspect the exact changed locations before source freeze.

## E1532.2 — retained private font override was not an interface preference

BuildV1 succeeded. Raw NativeWorldV1 executed all80 but ObjectiveAnnouncementBanner failed its unchanged36px user-font assertion in two cases. That fixture called the private ApplyReadableFonts helper directly; the new measured Slate update correctly read default interface TextScale1 and replaced the unsupported private override. DetachedHudLayout passed all3 worldless bounds/resize cases, and actual first-kill capture showed the crop corrected. The failed full80 report/all available49 images, false World/source receipts, successful build/source-capture receipts,5 original inputs and full3445 source baseline remain preserved. Visual helper refused the failed World receipt; no failed run was accepted. A subsequent summary read found no success-only summary, as expected for a rejected run.

After all build/test/source readers retired and launchers were reaped, preserved the original announcement fixture with SHA binding to the failed full baseline. Fixture-only repair uses SetInterfacePreviewForAutomation with the intended TextScale, matching the existing actual-interface acceptance path. All5 prior inputs unchanged; the36px assertion is retained. Freeze6 final inputs-v2, rerun exact inherited-source proof and fresh build/full80 native run. Prevention: express interface choices through the maintained profile/preview API; a private visual helper call is not persistent preference state and must not disable real layout updates.

Pre-execution V2 sealer audit caught a broad version substitution also renaming the immutable parent baseline and single V1 health/done files. Restored those references before any seal run. Native readers/source inputs unaffected. Prevention: version only current batch workers/roots; keep inherited baseline identities and the single monitor's original version fixed.

## Final validation

BuildV2/staged DLL/import SHA and fullWorldV2 passed:80 executed,79success+1existingwarningcase,0failed/0notrun,warnings4. Source check verified3445 inputs; exact inherited Rust/backend proof3439 unchanged+5 native deltas+1 native addition. No new full Rust suite claimed (prior1849passed/0failed/9existingignored). Twelve final changed/new images actually viewed,37 accepted SHA-identical images inherited. Actual standalone first-kill crop closed; before/after retained. Every final and historical build/test/source worker retired/reaped before sealing. The failed V1 report and49 available images remain separate from the49 final accepted images.
