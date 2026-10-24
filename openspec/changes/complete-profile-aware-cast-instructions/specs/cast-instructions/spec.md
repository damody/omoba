## ADDED Requirements

### Requirement: Current input help matches active bindings
World aiming help SHALL describe active current primary/secondary confirm and cancel chords, preserve intrinsic right-click cancellation, and name the actual held key for release confirmation. Missing/inactive bindings SHALL not be advertised as available. It SHALL remain read-only and use existing scope/lifecycle rules.

#### Scenario: Rebound confirmation
- WHEN confirm and escape keys are rebound with modifiers
- THEN current help shows those chords and actual owner matching accepts the same confirmation chords
- AND removed default or inactive bindings are not advertised
