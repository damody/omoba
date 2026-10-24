# Next concrete single-PC investigation: communication wheel coordinate space

Aim-layer V3 establishes the real production Slate overlay's highest paint layer and sibling compositing. It does not establish pointer-to-wheel placement at a nonzero desktop window position.

During V2 visual review, the fixture armed its gesture from a cached root `LocalToAbsolute(center)`, but the actual wheel center did not align with the separate root-centered blue panel. The original image is preserved at `omfue/Saved/Automation/AimLayerWorldV2/aim-layer/ordering.png` and in the aim-layer acceptance history. That observation could arise from fixture geometry timing or desktop/window/paint coordinate differences. It is not proof that actual gameplay pointer routing is broken, and it must not be presented as fixed.

V3 deliberately uses finite origin zero and existing production edge clamping to isolate layer compositing. It checks the entire later foreground pane and shows an unoccluded reference wheel. This avoids false visual acceptance, while leaving coordinate-space behavior as a separate concrete question.

Next batch should mount the actual production viewport/owned pointer route at explicitly nonzero window positions, sample the gesture screen origin and emitted center geometry in the same frame, and verify expected placement away from edge clamping at multiple logical scales. Preserve scope, user/pointer ownership and retirement behavior. If a real mismatch is demonstrated, repair a shared coordinate conversion rather than adding fixed offsets or a fixture-only compensation. Synthetic logical-scale tests do not replace physical OS DPI or controller acceptance.

Use the existing actual-exit Lua launcher, explicit immutable parent baseline from the final aim-layer index, source freeze and original failure evidence. Do not rerun already accepted aim-layer work merely to continue the automation.


Presentation investigation completed: see 2026-10-24-paint-space-final-acceptance.md and final paint-space index. Engine conventions confirmed; shared conversion fixes all common aiming/communication desktop points. Actual moved-window/render-scale center measurements pass. This does not establish live owner-context admission/network ACK/physical OS DPI.
