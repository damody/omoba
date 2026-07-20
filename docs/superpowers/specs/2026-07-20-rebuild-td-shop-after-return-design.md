# Rebuild TD Shop After Return Design

## Problem

Returning from gameplay enters `update_pregame_ui`, which calls `hide_gameplay_ui_for_pregame`. That method clears `td_tower_button_rects` but keeps the dynamically created `ui_td_tower_cards`. When a second game starts on another map, the new snapshot repopulates `td_template_order`. The shop layout sees that enough card nodes already exist, skips its card-creation loop, and then writes `td_tower_button_rects[0]` while the vector is empty. The frontend panics at `game/src/native.rs:7419` and the executor exits.

Runtime evidence confirms that the second backend starts, accepts the lockstep player, publishes tick zero, and initializes the new map before the frontend panic. The crash is therefore a TD shop UI lifecycle bug, not a backend or transport failure.

## Goals

- Remove all dynamically created TD tower card nodes when returning to pregame.
- Clear the card handles and hit rectangles as one lifecycle operation.
- Let the existing layout code rebuild cards from the second session's tower templates.
- Support selecting a different map and difficulty after returning.
- Preserve the existing session-owned backend lifecycle and other pregame UI nodes.

## Design

Add a focused `clear_td_tower_shop_cards` method on `Game`. It receives the active `UserInterface`, drains `ui_td_tower_cards`, and sends `WidgetMessage::Remove` for each card's background, icon, shortcut text, name text, and price text. It then clears `td_tower_button_rects`.

`hide_gameplay_ui_for_pregame` will call this method instead of clearing only the hit rectangles. The method is naturally idempotent: `update_pregame_ui` runs every pregame frame, but after the first drain subsequent calls have no nodes to remove.

On the next game, `td_template_order` is populated from the new map's authoritative snapshot. Because `ui_td_tower_cards` is empty, the existing `while self.ui_td_tower_cards.len() < n` loop recreates every card and pushes one matching hit rectangle per card before indexed layout writes occur.

## Failure Handling

UI removal uses Fyrox messages, matching existing node-removal patterns. Repeated pregame updates are safe because the card vector is drained exactly once. No fallback indexing or silent bounds suppression will be added; the card and hit-rectangle lifetimes will remain coupled by construction.

## Testing

- Add a regression test that creates real TD card UI nodes, seeds matching hit rectangles, invokes the cleanup method, and verifies both state vectors are empty.
- Process the UI removal messages and verify the removed handles are no longer valid.
- Simulate a subsequent card rebuild and verify the card and hit-rectangle counts match before indexed access.
- Run the focused regression test and the full `omfx` crate suite.
- Re-run the manual `run.bat` path: start one map, return, select a different map, and enter the second game without executor exit.

## Scope

- Modify only the omfx frontend submodule for the production fix and tests.
- Do not change backend lifecycle, lockstep protocol, map data, or `run.bat`.
- Do not modify the user's existing `omoba-core/src/runtime/native/game_processor.rs` work.
