# Design

The shared resolution normalizer publishes its measured geometry during Slate Tick through an optional delegate and updates its own relative child scale. CommandBar binds a weak UObject callback to the existing layout calculation factored into one helper also used by NativeTick. Existing signature caching avoids repeated heavy layout work; no per-entity operations or OS-DPI override. No mutation during arrange or paint.

Exercise real worldless UOmCommandBarWidget in a standalone SWindow at multiple sizes, inspect combat/minimap/shop bounds and resized recovery, retain current first-kill strip and capture corrected HUD. Keep existing actual game-viewport DPI acceptance. Check source consistency and inherit unchanged Rust/backend only with exact SHA proof.
