## ADDED Requirements

### Requirement: authoritative visible cast preparation
Visible heroes SHALL show a preparation pose driven only by a valid current authority windup and its fixed-point remaining time.

#### Scenario: cancellation and release
- WHEN movement, control, target retirement, death or recall cancels a windup
- THEN preparation is removed and no release is fabricated.
- WHEN a cast commits successfully
- THEN the existing release cue starts independently of the preparation clip.

#### Scenario: disclosure and lifetime
- WHEN a source, generation or complete status disappears
- THEN old preparation cannot restart from a stale packet.
- WHEN an ongoing preparation is freshly disclosed
- THEN its current authority phase may resume without replaying its beginning.
