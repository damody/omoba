## ADDED Requirements

### Requirement: Geometry-driven detached HUD layout
The HUD SHALL fit measured Slate geometry even without a game-world tick, preserve the shared DPI normalization policy and update after detached resize/content changes. The callback SHALL be weak and SHALL not mutate layout during arrange/paint. Existing live viewport behavior and memoization SHALL remain covered.
