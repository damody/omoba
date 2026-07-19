# Rebuild TD Shop After Return Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent the frontend from panicking when a player returns to the title and starts a different map by deleting dynamic TD shop cards and rebuilding them for the next session.

**Architecture:** Couple dynamic tower-card UI nodes and their hit rectangles behind one cleanup method. Pregame UI cleanup drains and removes every dynamic card node, leaving both vectors empty so the existing second-session layout path recreates matching cards and hit rectangles from the new snapshot.

**Tech Stack:** Rust 1.95.0, Fyrox UI 1.0.1, Cargo tests

---

## File Structure

- Modify `omfx/game/src/native.rs`: add the TD shop cleanup method, call it from pregame cleanup, and add focused regression coverage in the existing native test module.
- Modify the root `omfx` submodule pointer after the verified submodule commit.
- Preserve the user's existing `omoba-core/src/runtime/native/game_processor.rs` modification.

### Task 1: Reproduce and Fix Dynamic TD Shop Cleanup

**Files:**
- Modify: `omfx/game/src/native.rs:12471-12519`
- Test: `omfx/game/src/native.rs` in `input_latency_tests`

- [ ] **Step 1: Add the failing cleanup regression test**

Add this test beside `session_render_reset_releases_entity_slot_state`:

```rust
#[test]
fn returning_to_pregame_removes_dynamic_td_shop_cards() {
    let mut game = Game::default();
    let mut ui = UserInterface::new(Default::default());
    let bg = WidgetBuilder::new().build(&mut ui.build_ctx());
    let icon = WidgetBuilder::new().build(&mut ui.build_ctx());
    let key_text = TextBuilder::new(WidgetBuilder::new()).build(&mut ui.build_ctx());
    let name_text = TextBuilder::new(WidgetBuilder::new()).build(&mut ui.build_ctx());
    let price_text = TextBuilder::new(WidgetBuilder::new()).build(&mut ui.build_ctx());
    let handles: [Handle<UiNode>; 5] = [
        bg,
        icon,
        key_text.transmute(),
        name_text.transmute(),
        price_text.transmute(),
    ];
    game.ui_td_tower_cards.push(TdTowerShopCard {
        bg,
        icon,
        key_text,
        name_text,
        price_text,
    });
    game.td_tower_button_rects.push((1.0, 2.0, 3.0, 4.0));

    game.clear_td_tower_shop_cards(&mut ui);
    while ui.poll_message().is_some() {}

    assert!(game.ui_td_tower_cards.is_empty());
    assert!(game.td_tower_button_rects.is_empty());
    assert_eq!(
        game.ui_td_tower_cards.len(),
        game.td_tower_button_rects.len()
    );
    for handle in handles {
        assert!(!ui.nodes().is_valid_handle(handle));
    }
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx returning_to_pregame_removes_dynamic_td_shop_cards
```

Expected: compilation fails because `Game::clear_td_tower_shop_cards` does not exist.

- [ ] **Step 3: Implement complete card-node cleanup**

Add this method immediately before `hide_gameplay_ui_for_pregame`:

```rust
fn clear_td_tower_shop_cards(&mut self, ui: &mut UserInterface) {
    for card in self.ui_td_tower_cards.drain(..) {
        for node in [card.bg, card.icon] {
            ui.send(node, WidgetMessage::Remove);
        }
        for text in [card.key_text, card.name_text, card.price_text] {
            ui.send(text, WidgetMessage::Remove);
        }
    }
    self.td_tower_button_rects.clear();
}
```

In `hide_gameplay_ui_for_pregame`, replace:

```rust
self.td_tower_button_rects.clear();
```

with:

```rust
self.clear_td_tower_shop_cards(ui);
```

- [ ] **Step 4: Run the focused test and verify GREEN**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx returning_to_pregame_removes_dynamic_td_shop_cards
```

Expected: one test passes with zero failures.

- [ ] **Step 5: Run existing restart lifecycle regressions**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx in_game_return_button_click_returns_to_main_menu
cargo test --manifest-path omfx/Cargo.toml -p omfx session_render_reset_releases_entity_slot_state
```

Expected: both focused tests pass.

- [ ] **Step 6: Commit the omfx fix**

```powershell
git -C omfx add game/src/native.rs
git -C omfx commit -m "fix: rebuild TD shop cards after returning"
```

### Task 2: Verify and Integrate the Frontend Fix

**Files:**
- Verify: `omfx/game/src/native.rs`
- Modify: root `omfx` submodule pointer

- [ ] **Step 1: Run formatting verification**

Run:

```powershell
cargo fmt --manifest-path omfx/Cargo.toml --all -- --check
```

Expected: command exits zero with no formatting diff.

- [ ] **Step 2: Run the complete omfx test suite**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx
```

Expected: all omfx unit and doc tests pass with zero failures.

- [ ] **Step 3: Inspect scope**

Run:

```powershell
git -C omfx status --short
git status --short
git diff --submodule=log -- omfx
```

Expected: the omfx repository is clean; root shows the intended omfx pointer update, the user's pre-existing `omoba-core/src/runtime/native/game_processor.rs` modification, and the pre-existing `? omfue` state.

- [ ] **Step 4: Commit the submodule pointer**

```powershell
git add omfx
git commit -m "fix: rebuild TD shop after returning"
```

- [ ] **Step 5: Manual smoke path**

Run `run.bat`, select one TD map, return to the title after entering gameplay, select a different map, and start again. Expected: the second backend initializes, the TD shop cards are rebuilt, and `executor.exe` remains open without the `native.rs:7419` bounds panic.
