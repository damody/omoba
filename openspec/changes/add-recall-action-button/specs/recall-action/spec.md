## ADDED Requirements

### Requirement: Discoverable recall action uses the existing authority path
The native HUD SHALL provide a clickable recall action using the existing cursor-free recall command path. The label and tooltip SHALL reflect current bindings. Active channel, dead, ended, unknown, malformed, non-MOBA and retired states SHALL NOT present ready. Cleared scope SHALL remove the action. Disabled clicks SHALL be consumed. The action SHALL NOT infer completion from local time or invent transport acceptance.

#### Scenario: Current snapshot changes the same live button
- **WHEN** a current alive playing MOBA hero has a valid recall snapshot with zero remaining
- **THEN** the recall action is enabled
- **WHEN** a subsequent snapshot has positive recall remaining
- **THEN** the action is disabled and displays 回城中
- **WHEN** the current presentation changes to TD or waiting, or the input scope retires
- **THEN** the retained MOBA snapshot cannot expose or enable the action

#### Scenario: Slate verifies actual state transitions
- **WHEN** the test updates recall state, shop scope or presentation mode
- **THEN** it SHALL pump a mounted Slate window before reading registered widget attributes
- **AND** a hidden action SHALL have a collapsed root and reject activation through the model guard
- **AND** a visible unavailable action SHALL have an actual disabled button
