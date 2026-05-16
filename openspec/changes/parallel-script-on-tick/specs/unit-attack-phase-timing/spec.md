## MODIFIED Requirements

### Requirement: existing projectile and damage outcomes occur at impact event

The impact event SHALL reuse existing attack outcome semantics. Current projectile creation or damage outcomes, including `Outcome::ProjectileLine2`, `Outcome::ProjectileDirectional`, script-driven `spawn_projectile_ex`, direct damage outcomes, and related `Outcome::UpdateAttack` / `asd_count` cooldown accounting, SHALL be scheduled so their gameplay effect occurs at the authoritative impact event point.

For scripted `on_tick`, attack-phase state updates and impact-side effects SHALL be recorded as deterministic script outcomes during compute, then applied by the script outcome apply stage. The apply stage SHALL preserve the same authoritative impact timing as the serial implementation and SHALL NOT make projectile spawn, damage, cooldown accounting, or attack phase cues visible before the intended impact boundary.

#### Scenario: existing projectile outcome is delayed to impact

- **WHEN** a tower attack would currently emit `Outcome::ProjectileLine2` when cooldown is ready
- **THEN** the new attack scheduler starts windup when cooldown is ready
- **AND** emits the projectile outcome at the impact event point
- **AND** does not treat impact as a duration

#### Scenario: cooldown accounting still covers the whole interval

- **WHEN** an attack with windup and backswing completes
- **THEN** `asd_count` / cooldown accounting represents the full effective attack interval
- **AND** adding windup/backswing does not allow an extra attack before the interval is complete

#### Scenario: scripted tower attack effects are applied from outcomes

- **WHEN** a scripted tower reaches its impact step during parallel `on_tick`
- **THEN** script compute records projectile, damage, facing, `asd_count`, and render cue changes as ordered outcomes
- **AND** deterministic apply writes those effects to ECS at the same tick boundary
- **AND** the effect remains aligned with the authoritative impact event
