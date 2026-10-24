## ADDED Requirements

### Requirement: Desktop input points map through the current paint frame
Shared aiming and communication overlays SHALL transform desktop points using the current paint frame window translation before inverse geometry. They SHALL reject invalid/singular conversions and retain existing ownership/input gates.

#### Scenario: Window moves away from desktop origin
- WHEN a pointer-event desktop origin arms a production wheel and the window is at a nonzero desktop position
- THEN the actual painted wheel center matches that origin away from edge clamping
- AND the same holds after window movement and render scaling
- AND retired state paints no wheel
