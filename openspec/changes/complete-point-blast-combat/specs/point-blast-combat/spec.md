## ADDED Requirements
### Requirement: Strict point-circle execution
The executor MUST reject invalid rank/JSON/scalars/point/source/range before any outcomes. Alive enemy center points inside or tangent to the circle MUST receive at most one canonical damage/control pair. Ally, dead and one-raw-outside targets MUST be excluded. Only the center is constrained by cast range; area victims may extend beyond it.
### Requirement: Authored geometry
Special-handler PointCircle MUST reference a complete bounded per-rank radius key and MUST NOT conflict with executable effects. It MUST remain non-movement and publish the same current rank radius through existing Unreal aiming/tooltip data.
### Requirement: Resource rollback
Failed formal casts MUST preserve mana/cooldown and produce no damage/control. Valid casts MUST debit once and apply instantaneous effects once, with canonical public control duration and explicit FIRE damage profile preserved.