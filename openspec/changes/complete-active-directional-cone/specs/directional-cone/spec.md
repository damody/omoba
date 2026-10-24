## ADDED Requirements

### Requirement: Data-driven active directional sector
An instant point active/ultimate ability MAY declare one exclusive cone_damage effect with rank-complete damage, positive bounded cast range and half angle in [1/1024,pi/2]. Authority SHALL quantize half-angle with the shared Q10/4096-turn sector function and publish those integer ticks for presentation.

#### Scenario: Short valid direction reaches full range
- WHEN a living caster requests a distinct in-range ground point
- THEN each living enemy in the source-centered directed sector is damaged once in canonical entity order
- AND the sector extends to the full rank range and includes its boundaries
- AND invalid direction/range or an excessive resolved plan rejects before mutation
- AND original point-circle abilities keep their geometry and IDs

### Requirement: Common directional guidance
Native point aiming SHALL use disclosed rank cone ticks to show a source-centered sector and missing-direction guidance. Invalid/replaced/retired state SHALL clear the cached geometry. Tooltip SHALL describe source, angle and direction requirement. Guidance SHALL remain read-only; actual fixed-point authority decides hits.

#### Scenario: Zero ground direction
- WHEN the selected point equals the source ground position
- THEN preview requests a direction without inventing a facing
- AND no sector outline remains
- AND authority rejects the cast without damage or cooldown/resource consumption
