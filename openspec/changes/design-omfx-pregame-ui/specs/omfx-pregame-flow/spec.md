## ADDED Requirements

### Requirement: omfx starts in pregame menu

`omfx` native frontend SHALL start in a pregame menu state before any gameplay session is created. In this state, `omfx` SHALL NOT start the lockstep client, SHALL NOT spawn `sim_runner`, and SHALL NOT start or connect to an `omobab` backend process.

#### Scenario: menu-only startup does not contact backend

- **WHEN** the native `omfx` executable starts without an active game session
- **THEN** the first visible state is the main menu
- **AND** no `lockstep_client` thread is spawned
- **AND** no `sim_runner` thread is spawned
- **AND** no session-owned backend process is running

#### Scenario: legacy gameplay is hidden before session start

- **WHEN** the player is on the main menu, map select, or difficulty select screen
- **THEN** the existing gameplay HUD and render interaction layer are not active
- **AND** gameplay map clicks, tower placement, ability hotkeys, and `StartRound` input are not submitted

### Requirement: main menu routes to map select

The pregame main menu SHALL present a player-visible start control in a bright TD lobby-style screen. Activating the start control SHALL transition to map selection without creating a backend session.

#### Scenario: clicking start opens map selection

- **WHEN** the player clicks the main menu start button
- **THEN** `omfx` transitions to the map select screen
- **AND** no backend process is started yet

#### Scenario: main menu supports decorative non-gameplay controls

- **WHEN** the main menu displays non-start controls such as character, shop, settings, or side buttons
- **THEN** those controls do not submit gameplay `PlayerInput`
- **AND** unsupported controls may show disabled, placeholder, or no-op behavior without leaving the main menu flow

### Requirement: map select chooses one playable map

The map select screen SHALL show a selectable set of map cards loaded from script-owned pregame UI content. Each playable map card SHALL map to a stable `map_id` and backend story/runtime identifier used when the session starts. The screen SHALL provide a back action returning to the main menu.

#### Scenario: selecting a map advances to difficulty selection

- **WHEN** the player selects an enabled map card
- **THEN** `omfx` stores that map selection
- **AND** transitions to the difficulty select screen
- **AND** no backend process is started yet

#### Scenario: map select back returns to main menu

- **WHEN** the player activates the map select back button
- **THEN** `omfx` discards any transient map selection for this flow
- **AND** returns to the main menu
- **AND** no backend process is started

#### Scenario: locked or disabled map cannot start a session

- **WHEN** the player clicks a locked or disabled map card
- **THEN** `omfx` remains on the map select screen
- **AND** no difficulty screen is opened for that map
- **AND** no backend process is started

### Requirement: difficulty select starts the selected session

The difficulty select screen SHALL show difficulty choices loaded from script-owned pregame UI content. The default `base_content` catalog SHALL include at least easy, medium, and hard choices. Selecting a difficulty SHALL create a game session using the previously selected map and the selected difficulty, then transition through a loading state into the existing gameplay screen.

#### Scenario: selecting difficulty starts backend session

- **WHEN** the player has selected a map
- **AND** the player selects a difficulty
- **THEN** `omfx` creates a session config containing the selected `map_id`, backend story/runtime identifier, and difficulty id
- **AND** the config values originate from the loaded scripts pregame catalog
- **AND** `omfx` transitions to a session loading state
- **AND** the session-owned backend lifecycle is started for that config

#### Scenario: difficulty back returns to map select

- **WHEN** the player activates the difficulty select back button
- **THEN** `omfx` returns to map select
- **AND** preserves the ability to choose a different map
- **AND** no backend process is started by the back action

#### Scenario: session start failure is recoverable

- **WHEN** backend startup or initial lockstep connection fails during session loading
- **THEN** `omfx` stops any partially created session resources
- **AND** shows a recoverable error or returns to difficulty select
- **AND** the player can retry without restarting the executable

### Requirement: existing gameplay starts only after session creation

The existing gameplay renderer, lockstep client, local sim runner, HUD, and gameplay input routing SHALL become active only after a selected map/difficulty session has successfully started.

#### Scenario: entering gameplay initializes current runtime

- **WHEN** a selected session reaches the connected gameplay state
- **THEN** `omfx` starts the lockstep client for that session
- **AND** starts `sim_runner` with the same map/story config as the backend
- **AND** displays the existing gameplay screen currently used during development

#### Scenario: gameplay config matches selected map

- **WHEN** the player starts a session from a map card
- **THEN** backend startup config and local `sim_runner` scene config use the same story/runtime identifier
- **AND** logs include the selected `map_id`, story/runtime identifier, difficulty id, and session id

### Requirement: game end returns to pregame and tears down session

When a game session ends by victory, defeat, player exit, or plugin shutdown, `omfx` SHALL stop gameplay runtime resources, close the session-owned backend, and return to a pregame state unless the executable is shutting down.

#### Scenario: player exits gameplay to menu

- **WHEN** the player activates a return-to-menu or exit-session action during gameplay
- **THEN** `omfx` stops gameplay input routing
- **AND** drops the lockstep client and `sim_runner`
- **AND** shuts down the session-owned backend process
- **AND** returns to the pregame main menu or session result screen

#### Scenario: game over shuts down backend

- **WHEN** `omfx` observes the authoritative game-over condition for the active session
- **THEN** `omfx` shows the session end/result state
- **AND** shuts down the session-owned backend process before starting another session

#### Scenario: plugin deinit cleans active session

- **WHEN** the `omfx` plugin is deinitialized while a session is loading or in gameplay
- **THEN** session shutdown runs exactly once
- **AND** no session-owned backend process is intentionally left running
