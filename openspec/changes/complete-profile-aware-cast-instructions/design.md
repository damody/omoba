# Design

Read-only shared native helper queries current WorldCombat/Pressed primary/secondary bindings and IsChordBindingActive, exactly as owner input matching. Default profile fallback matches existing owner fallback. QuickWithIndicator names actual held key or generic release; Normal lists both confirmation chords or explicit unbound; Quick/invalid modes do not invent a confirmation. Cancel lists intrinsic right-click plus active EscapePriority/CancelAim chords, deduplicated. Existing wrapping/text-scale/owner overlay lifecycle retained. No command, profile persistence, scope gates or model changes.

Actual owner matching tests modifiers/primary/secondary/removal/retirement, helper inactive/missing/release/change cases and mounted Slate capture with full bounds assertion. Freeze4native sources, UE/full88native/3460source/59images. Use explicit immutable batch JSON and preflight parent binding before any expensive readers, avoiding broad version substitutions.
