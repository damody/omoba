# Pregame Menu Redesign Design

## Goal

Redesign the native `omfx` pregame flow to match the three provided reference screens:

- A colorful full-screen home menu with Chinese labels, top resource/profile UI, side utility buttons, and a large bottom-center `開始` button.
- A Chinese difficulty selection overlay with three large cards: `簡單`, `中級`, `困難`.
- A Chinese map selection overlay with a 2x3 card grid, page indicators, and bottom map-category tabs.

The implementation should stay compatible with the existing script-owned `scripts/base_content/assets/pregame_ui/catalog.json` content model.

## Flow

The user-facing flow becomes:

1. `MainMenu`: home screen.
2. Click `開始`.
3. `DifficultySelect`: choose difficulty.
4. `MapSelect`: choose map.
5. Start the session with the selected difficulty and map.

This reverses the current map-first flow. The runtime keeps `selected_difficulty` when moving from difficulty to map, and starts only after a playable map is selected.

## Content And Chinese Copy

`catalog.json` remains the authoritative source for labels, descriptions, map metadata, and difficulty metadata. The base catalog and frontend fallback catalog will be localized to Traditional Chinese.

Initial copy:

- Main title: `Omoba 塔防`
- Main `開始` button: `開始`
- Other visible home items: `英雄`, `知識`, `設定`, `任務`, `商店`
- Difficulty labels: `簡單`, `中級`, `困難`
- Back label: `返回`
- Map examples: current playable catalog entries receive Chinese labels and descriptions. Locked/developer maps use Chinese locked text.

## Layout

`omfx/game/src/native.rs` will replace the single generic pregame button grid with state-specific layout helpers:

- `MainMenu`: full-screen stylized background using Fyrox UI borders/text. It approximates the island scene with sky, foliage, huts/statue shapes, side circular buttons, top resource/profile strips, and a bottom navigation row. The large green `開始` button is the primary clickable action.
- `DifficultySelect`: full-screen dark teal overlay over the same background style. Three large cards sit horizontally on wide screens and stack or shrink on narrow screens. Each card shows a circular portrait placeholder, Chinese label, reward line, and remains clickable only when enabled.
- `MapSelect`: full-screen dark overlay with a 2x3 responsive map-card grid. Existing catalog maps are shown first. If fewer than six maps exist, only available maps are shown; locked maps appear disabled. Page dots and category tabs are visual-only for this change unless the catalog later grows enough maps to require real paging.
- `StartingSession` and `SessionEnded`: use simple Chinese status states consistent with the new palette.

The current `PregameButtonUi` pool can be reused for clickable UI rectangles and text, but the update path should calculate rectangles per state rather than using one shared grid algorithm.

## Behavior

- `開始` navigates from `MainMenu` to `DifficultySelect`.
- Choosing a difficulty stores `selected_difficulty` and navigates to `MapSelect`.
- Choosing a playable map stores `selected_map` and starts the session.
- `返回` from `DifficultySelect` returns to `MainMenu` and clears difficulty/map.
- `返回` from `MapSelect` returns to `DifficultySelect` and clears map only.
- Disabled/locked entries are visible but not clickable.

## Tests

Update existing pregame unit tests in `omfx/game/src/native.rs` and `omfx/game/src/pregame.rs` to cover:

- Catalog/fallback Chinese labels parse and remain active.
- `開始 -> DifficultySelect -> MapSelect -> StartingSession` flow.
- Back behavior from each pregame screen.
- `default_session_selection()` still chooses the first playable map and first enabled difficulty for legacy autostart.

Manual verification should run the `omfx` test target and, if feasible in the local environment, launch `run.bat` to inspect the native menu.
