## ADDED Requirements

### Requirement: Attack cancellation uses impact as the commit point

The backend attack scheduler SHALL treat the attack impact event as the authoritative commit point for a basic attack. Accepted move commands and accepted skill commands SHALL be able to cancel an attack during windup before the impact event. A windup-canceled attack SHALL NOT apply damage, spawn a projectile, produce a hit outcome, roll or report a critical hit, or consume any impact-side effect for that attack sequence.

If an accepted move or skill command arrives after the impact event while the unit is in attack backswing, the command MAY cancel the remaining backswing animation or backswing lockout, but the already committed attack result SHALL remain valid. The backend SHALL NOT roll back damage, projectile spawn, hit outcome, critical result, or cooldown accounting that committed at impact.

#### Scenario: Move command cancels windup before damage
- **WHEN** a hero starts attack windup and receives an accepted move command before the attack impact event
- **THEN** the backend cancels that attack sequence before impact
- **AND** no damage, projectile, hit outcome, or critical result is produced for that attack sequence
- **AND** omfx can transition the hero out of the attack animation before the hit frame

#### Scenario: Skill command cancels windup before damage
- **WHEN** a hero starts attack windup and receives an accepted skill command before the attack impact event
- **THEN** the backend cancels that attack sequence before impact
- **AND** the skill command may proceed according to existing skill validation rules
- **AND** the canceled attack does not apply damage or spawn a projectile

#### Scenario: Move command during backswing preserves committed attack
- **WHEN** a hero attack reaches the impact event and then receives an accepted move command during backswing
- **THEN** the backend may interrupt the remaining backswing
- **AND** the attack damage, projectile, hit outcome, or critical result that committed at impact remains valid
- **AND** the move command does not roll back the attack result

#### Scenario: Skill command during backswing preserves committed attack
- **WHEN** a hero attack reaches the impact event and then receives an accepted skill command during backswing
- **THEN** the backend may interrupt the remaining backswing
- **AND** the skill command may proceed according to existing skill validation rules
- **AND** the attack result that committed at impact remains valid

### Requirement: Attack cancel cues keep frontend animation aligned

The backend SHALL provide render-only cues that let omfx distinguish a windup-canceled attack from a backswing-interrupted attack. Windup cancel cues SHALL include at minimum entity id, entity generation, attack sequence id, and cancellation phase. Backswing interrupt cues MAY reuse state/cancel cues, but SHALL indicate that impact already committed. These cues SHALL NOT affect gameplay state hashing.

#### Scenario: Frontend receives windup cancel cue
- **WHEN** an attack sequence is canceled before impact
- **THEN** a render-only cancel cue is available to omfx
- **AND** omfx stops or blends out the attack windup animation
- **AND** omfx does not show the attack hit frame, critical hit animation, projectile fire, or damage-only impact effect for that canceled sequence

#### Scenario: Frontend receives backswing interrupt after impact
- **WHEN** an attack sequence is interrupted after impact during backswing
- **THEN** omfx may stop or blend out the remaining backswing animation
- **AND** omfx preserves any hit, projectile, recoil, critical, or impact visual that already corresponded to the committed impact event

#### Scenario: Cancel cues are render-only
- **WHEN** `extract_snapshot` drains attack cancel cues into `SimWorldSnapshot`
- **THEN** source queues are emptied using the render-only queue pattern
- **AND** gameplay components, damage state, projectile state, cooldown state, and deterministic hashes are not mutated by the drain

### Requirement: Attack animation phases align with authoritative timing

omfx SHALL play attack animations in three visual phases that match backend attack timing: animation windup before impact, animation hit frame at impact, and animation backswing after impact. For model-backed heroes, the metadata-defined attack or critical impact tick SHALL be retimed to align with the attack phase cue impact offset.

#### Scenario: Attack animation hit frame aligns with impact
- **WHEN** omfx receives an attack phase cue with windup duration and backswing duration
- **THEN** omfx starts the attack animation during windup
- **AND** omfx aligns the animation hit frame with the authoritative impact event
- **AND** omfx plays or skips backswing according to later authoritative cancel/interruption cues

#### Scenario: Windup cancel prevents hit visuals
- **WHEN** omfx started an attack animation from a windup cue and later receives a pre-impact cancel cue for the same attack sequence
- **THEN** omfx cancels the animation before the hit frame
- **AND** omfx does not show damage, projectile, critical, or hit-confirm visuals for that sequence

#### Scenario: Backswing cancel preserves hit visuals
- **WHEN** omfx started an attack animation and the authoritative impact event has already occurred
- **THEN** a later backswing interrupt may stop the remaining backswing animation
- **AND** omfx does not remove the already displayed hit or projectile visuals tied to the impact event
