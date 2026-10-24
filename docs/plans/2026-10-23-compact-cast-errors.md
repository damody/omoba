# E1527 — compact cast readout

## Root cause and prevention

Previously accepted native screenshots show a 38x65 cast row: 施法 <0.1s breaks across several lines. Initial construction uses WrapTextAt(36) with AllowPerCharacterWrapping, and legacy retained composition reapplies icon-width wrapping. Existing tests checked that wrapped desired size fitted arranged height, which cannot detect this readability defect.

Fix both layout paths with compact active/terminal labels and unwrapped measured card width; preserve full status in tooltip and user font size. Regression now compares actual glyph-measured single-line height in a real viewport across five states, three text scales and two window sizes. Existing cast lifecycle and cooldown checks remain. No build/test failure has occurred in this batch yet.

## WorldV1 runner identity substitution

Copying the previous runner with an unrestricted PublicChannel → CompactCast substitution changed the existing Om.Runtime.PublicChannelDuration test ID to nonexistent Om.Runtime.CompactCastDuration, and renamed its fixture flag/path. The actual cast-time regression passed, but this run cannot count as complete native acceptance. Keep its runner, raw report/logs/available captures and false receipts; do not overwrite the root. Retire/reap WorldV1 and its upstream-gated source reader, then run a fresh WorldV2 with the exact existing test identity and fixture flag restored. All three formal native inputs remain frozen and unchanged; no rebuild is needed for runner-only repair. Prevention: restrict batch substitutions to paths and assert the original native test ID, CLI fixture key and exported ABI file before launch.

## Accepted closure

UE5.8 build-v1/staged DLL/import SHA passed. WorldV2 executed75:74success+1existingwarningcase,0failed/0notrun. Source guard verified3440 inputs; accepted Rust1844passed/0failed/9existingignored and backend results inherited after exact SHA proof of3437 unchanged prior inputs plus3 native deltas; no new full Rust suite is claimed. Archive contains360 independently reverified evidence files and37 PNGs. Health5 samples/max171 seconds. Four successful final workers and two failed historical workers have creation-token-bound retired receipts; all launchers, inheritance reader and the single170-second health monitor were reaped. CIM found no owned worker remaining.

6 changed/new images actually viewed; 31 SHA-identical accepted images inherited. Actual cast crop is40x24, previously38x65; <0.1s remains one line. Actual full1280x720 viewport shows cooldown20, cast progress and hotkey intact. No font shrinking or local cast countdown.
