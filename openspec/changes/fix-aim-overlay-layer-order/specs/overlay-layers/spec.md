## ADDED Requirements

### Requirement: Overlay reports its actual highest draw layer
The common aiming and communication overlay SHALL return at least its highest emitted draw layer, including early exits after partial drawing. Inactive overlay SHALL not invent extra layers. Later sibling geometry SHALL consistently composite above earlier overlay text.

#### Scenario: Communication wheel and later foreground
- WHEN a real ping or emote wheel paints center text above its ring
- THEN production OnPaint reports that text layer
- AND a later opaque sibling covers all center pixels
- AND retiring input removes the wheel and its extra layer requirement
