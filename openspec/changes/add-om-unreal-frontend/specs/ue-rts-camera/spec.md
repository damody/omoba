## ADDED Requirements

### Requirement: Custom RTS camera controller
`Om UE frontend` SHALL provide a custom RTS camera controller for the UE frontend. The camera SHALL support mouse-edge panning and mouse-wheel zoom so the player can inspect the map without changing deterministic gameplay state. Camera movement SHALL be local presentation state only and SHALL NOT submit bridge gameplay inputs.

#### Scenario: Mouse at viewport edge pans camera
- **WHEN** the player moves the mouse cursor inside a configured edge-scroll band near the viewport edge
- **THEN** the RTS camera MUST pan continuously in the corresponding world direction
- **AND** diagonal panning MUST work when the cursor is near a corner
- **AND** pan speed MUST be configurable
- **AND** the camera MUST stop edge panning when the cursor leaves the viewport, the window is unfocused, or UI captures the pointer

#### Scenario: Mouse wheel zooms camera
- **WHEN** the player scrolls the mouse wheel
- **THEN** the RTS camera MUST zoom in or out
- **AND** zoom distance or FOV MUST be clamped between configurable min and max values
- **AND** zoom speed/smoothing MUST be configurable
- **AND** zoom MUST preserve a stable look-at point or equivalent RTS-style focus behavior

#### Scenario: Camera is bounded to playable map area
- **WHEN** the camera pans or zooms
- **THEN** the camera MUST remain within configured map bounds or bounds derived from available map route/path data when enabled
- **AND** invalid or missing bounds MUST fall back to safe default limits rather than allowing NaN/invalid transforms

#### Scenario: Camera settings are designer-configurable
- **WHEN** a designer configures RTS camera defaults in UE settings, Blueprint defaults, or a placed camera actor
- **THEN** edge band size, pan speed, zoom limits, zoom speed, pitch/yaw defaults, smoothing, and bounds behavior MUST be editable
- **AND** changes MUST NOT require editing generated C++ code

#### Scenario: Camera input does not conflict with gameplay input
- **WHEN** the player uses edge scroll or mouse wheel zoom
- **THEN** the camera controller MUST consume only camera navigation input
- **AND** it MUST NOT generate move, attack, ability, tower, or other gameplay input events
- **AND** gameplay input generator/UI guard behavior MUST remain responsible for gameplay commands

### Requirement: RTS camera verification
The implementation SHALL include verification for edge-scroll panning, mouse-wheel zoom, clamping, UI capture behavior, and isolation from gameplay input submission.

#### Scenario: Synthetic camera input smoke covers pan and zoom
- **WHEN** automated smoke or tests inject cursor positions near each viewport edge and mouse wheel deltas
- **THEN** the camera transform MUST move or zoom in the expected direction
- **AND** camera values MUST remain finite and within configured bounds

#### Scenario: UI capture suppresses edge panning
- **WHEN** UI captures the mouse pointer or marks the pointer event as consumed
- **THEN** edge-scroll camera movement MUST not run for that input frame
- **AND** no gameplay input event MUST be submitted as a side effect
