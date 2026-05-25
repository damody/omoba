## ADDED Requirements

### Requirement: pregame UI content is owned by scripts content mod

Pregame UI screen data, map cards, difficulty cards, display text, image slots, and controlled interaction actions SHALL be owned by scripts content mod data. `omfx` SHALL NOT treat hard-coded Rust tables as the canonical source for pregame menu, map select, or difficulty select content.

#### Scenario: base_content provides canonical pregame catalog

- **WHEN** `scripts/base_content` is available
- **THEN** it provides a pregame UI catalog for main menu, map select, and difficulty select
- **AND** `omfx` loads the catalog before rendering pregame UI
- **AND** the catalog is the canonical source for map ids, story/runtime identifiers, difficulty ids, labels, image slots, and button actions

#### Scenario: frontend fallback is not canonical

- **WHEN** `omfx` contains fallback placeholder UI data for developer safety
- **THEN** that fallback is used only when scripts content is missing or invalid
- **AND** logs clearly identify that fallback data is being used
- **AND** docs do not tell content authors to edit `omfx` source for normal pregame UI changes

### Requirement: pregame actions are declarative and whitelisted

Scripts pregame UI content SHALL declare interaction behavior using a bounded action model. `omfx` SHALL execute only known action ids and SHALL NOT execute arbitrary script-provided frontend code.

#### Scenario: known action changes screen state

- **WHEN** a catalog button declares a known action such as `Navigate`, `Back`, `SelectMap`, `SelectDifficulty`, `StartSession`, or `NoOp`
- **THEN** `omfx` executes the corresponding frontend state transition or session action
- **AND** no arbitrary script code runs in the frontend process

#### Scenario: unknown action is safe

- **WHEN** a catalog entry declares an unknown or malformed action
- **THEN** `omfx` treats that entry as disabled or no-op
- **AND** logs enough information to diagnose the bad action
- **AND** the frontend does not panic

### Requirement: maps and difficulties come from scripts catalog

The selected map and difficulty used to start a game session SHALL come from the loaded scripts pregame catalog. Each enabled map entry SHALL include a stable `map_id` and story/runtime identifier. Each enabled difficulty entry SHALL include a stable `difficulty_id` and backend-visible config value.

#### Scenario: map selection uses catalog story id

- **WHEN** the player selects an enabled map card
- **THEN** `omfx` stores the `map_id` and story/runtime identifier from the scripts catalog
- **AND** the stored story/runtime identifier is used for both backend launch config and local `sim_runner` scene config

#### Scenario: difficulty selection uses catalog difficulty id

- **WHEN** the player selects an enabled difficulty card
- **THEN** `omfx` stores the `difficulty_id` and backend-visible difficulty config from the scripts catalog
- **AND** that difficulty config is included in session startup metadata

### Requirement: pregame assets use script-owned replaceable paths

Pregame UI image slots SHALL reference assets in scripts content mod paths or staged equivalents. `omfx` SHALL prioritize scripts assets over frontend-local placeholders when loading pregame background, map preview, difficulty icon, button, and badge images.

#### Scenario: scripts asset wins for pregame image

- **WHEN** a pregame catalog image slot references a scripts asset path
- **THEN** `omfx` loads that scripts asset or its staged equivalent
- **AND** a frontend-local placeholder with the same logical slot does not override it

#### Scenario: missing pregame asset falls back without panic

- **WHEN** a pregame catalog image path is missing
- **THEN** `omfx` uses a diagnostic placeholder for that slot
- **AND** logs the missing asset path
- **AND** the pregame flow remains usable if the required action data is valid

### Requirement: mod replacement preserves frontend safety

A mod SHALL be able to replace pregame UI catalog content without recompiling `omfx`, provided it follows the catalog schema and uses supported action ids. Invalid mod content SHALL fail safely.

#### Scenario: mod changes menu data without frontend rebuild

- **WHEN** a mod replaces the scripts pregame catalog with valid map, difficulty, text, image, and action entries
- **THEN** restarting `omfx` or reloading content causes the pregame UI to reflect the mod data
- **AND** no `omfx` source code change is required

#### Scenario: invalid mod content does not start a broken session

- **WHEN** a mod defines an enabled map without a story/runtime identifier or an enabled difficulty without a difficulty id
- **THEN** `omfx` disables the affected entry or rejects session start
- **AND** logs the validation failure
- **AND** no backend session is launched from incomplete config
