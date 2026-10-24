## ADDED Requirements

### Requirement: Content driven minion aura
Team objective rewards MAY define a bounded minion aura. Missing metadata SHALL preserve legacy behavior; invalid, conflicting, unknown or personal aura policies SHALL be rejected.

### Requirement: Authority proximity lifecycle
Only living eligible heroes SHALL emit to allied living lane creeps inside the inclusive radius. Overlap and reward stacks SHALL NOT multiply one aura. Spawn, departure, hero death, expiration, pause and match end SHALL respect authority lifecycle and deterministic state. Buffs SHALL use normal stats and public snapshot paths.

### Requirement: Applied combat statistics
Lane attack packets and DamageInstance defenses SHALL aggregate existing BuffStore modifiers through UnitStats. Acceptance SHALL verify actual authority attack packets and final HP damage, in addition to public snapshot metadata.
