# Pregame Menu Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Chinese, reference-inspired pregame menu flow: home screen, difficulty selection, map selection, then session launch.

**Architecture:** Keep `scripts/base_content/assets/pregame_ui/catalog.json` as the content source and keep session launch behavior in `omfx/game/src/native.rs`. Change `PregameRuntime` so difficulty is selected before map, then replace the generic button grid with state-specific Fyrox UI layouts using the existing pregame UI node pool.

**Tech Stack:** Rust 1.95.0, Fyrox UI, serde JSON catalog parsing, existing `omfx` native tests.

---

## File Structure

- Modify `scripts/base_content/assets/pregame_ui/catalog.json`: localize base menu, map, and difficulty copy to Traditional Chinese and make `開始` navigate to difficulty selection.
- Modify `omfx/game/src/pregame.rs`: update fallback Chinese copy and runtime flow so `SelectDifficulty` navigates to map selection, while `SelectMap` starts the selected session.
- Modify `omfx/game/src/native.rs`: add pregame visual element roles, build enough pooled border/text nodes, calculate per-state layouts, and update tests for the new flow.

### Task 1: Runtime Flow And Chinese Catalog

**Files:**
- Modify: `scripts/base_content/assets/pregame_ui/catalog.json`
- Modify: `omfx/game/src/pregame.rs`

- [ ] **Step 1: Write the failing runtime tests**

Add these tests to the existing `#[cfg(test)] mod tests` in `omfx/game/src/pregame.rs`:

```rust
    #[test]
    fn difficulty_first_flow_starts_after_map_selection() {
        let mut runtime = PregameRuntime::new_for_menu(PregameCatalog::fallback());

        assert_eq!(
            runtime.dispatch(&PregameAction::Navigate {
                target: "difficulty_select".to_string()
            }),
            None
        );
        assert!(matches!(runtime.state, PregameState::DifficultySelect));

        assert_eq!(
            runtime.dispatch(&PregameAction::SelectDifficulty {
                difficulty_id: "easy".to_string()
            }),
            None
        );
        assert!(matches!(runtime.state, PregameState::MapSelect));
        assert_eq!(
            runtime.selected_difficulty.as_ref().map(|entry| entry.id.as_str()),
            Some("easy")
        );

        let selection = runtime
            .dispatch(&PregameAction::SelectMap {
                map_id: "td_1".to_string()
            })
            .expect("map selection starts the session");

        assert!(matches!(runtime.state, PregameState::StartingSession));
        assert_eq!(selection.map.id, "td_1");
        assert_eq!(selection.difficulty.id, "easy");
    }

    #[test]
    fn back_from_map_select_preserves_difficulty_screen_only() {
        let mut runtime = PregameRuntime::new_for_menu(PregameCatalog::fallback());
        runtime.dispatch(&PregameAction::Navigate {
            target: "difficulty_select".to_string(),
        });
        runtime.dispatch(&PregameAction::SelectDifficulty {
            difficulty_id: "easy".to_string(),
        });
        runtime.selected_map = Some(runtime.catalog.enabled_maps()[0].clone());

        runtime.dispatch(&PregameAction::Back);

        assert!(matches!(runtime.state, PregameState::DifficultySelect));
        assert!(runtime.selected_map.is_none());
        assert_eq!(
            runtime.selected_difficulty.as_ref().map(|entry| entry.id.as_str()),
            Some("easy")
        );
    }

    #[test]
    fn fallback_catalog_uses_chinese_pregame_labels() {
        let catalog = PregameCatalog::fallback();

        assert_eq!(catalog.screen("main_menu").unwrap().title, "Omoba 塔防");
        assert!(catalog
            .screen("main_menu")
            .unwrap()
            .widgets
            .iter()
            .any(|widget| widget.label == "開始"));
        assert_eq!(catalog.difficulty("easy").unwrap().label, "簡單");
        assert_eq!(catalog.difficulty("medium").unwrap().label, "中級");
        assert_eq!(catalog.difficulty("hard").unwrap().label, "困難");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests::difficulty_first_flow_starts_after_map_selection pregame::tests::back_from_map_select_preserves_difficulty_screen_only pregame::tests::fallback_catalog_uses_chinese_pregame_labels
```

Expected: tests fail because `difficulty_select` navigation is not handled, `SelectDifficulty` currently starts selection, and fallback labels are English.

- [ ] **Step 3: Update runtime flow**

In `PregameRuntime::dispatch` in `omfx/game/src/pregame.rs`, add `difficulty_select` navigation and change `SelectMap` / `SelectDifficulty` behavior to:

```rust
            PregameAction::Navigate { target } if target == "difficulty_select" => {
                self.selected_map = None;
                self.selected_difficulty = None;
                self.state = PregameState::DifficultySelect;
                None
            }
            PregameAction::SelectMap { map_id } => {
                let Some(map) = self.catalog.map(map_id) else {
                    return None;
                };
                if !map.is_playable() {
                    return None;
                }
                self.selected_map = Some(map.clone());
                self.start_selection()
            }
            PregameAction::SelectDifficulty { difficulty_id } => {
                let Some(difficulty) = self.catalog.difficulty(difficulty_id) else {
                    return None;
                };
                if !difficulty.enabled {
                    return None;
                }
                self.selected_difficulty = Some(difficulty.clone());
                self.selected_map = None;
                self.state = PregameState::MapSelect;
                None
            }
```

Update `PregameAction::Back` cases to:

```rust
                    PregameState::MapSelect => {
                        self.selected_map = None;
                        self.state = PregameState::DifficultySelect;
                    }
                    PregameState::DifficultySelect => {
                        self.selected_map = None;
                        self.selected_difficulty = None;
                        self.state = PregameState::MainMenu;
                    }
```

- [ ] **Step 4: Localize fallback catalog**

In `PregameCatalog::fallback()`, update visible strings:

```rust
title: "Omoba 塔防".into(),
subtitle: "選擇難度與地圖，準備守住路線".into(),
label: "開始".into(),
description: "選擇難度".into(),
target: "difficulty_select".into(),
label: "設定".into(),
description: "即將開放".into(),
title: "選擇地圖".into(),
label: "返回".into(),
title: "選擇難度".into(),
subtitle: "先選難度，再挑戰地圖".into(),
label: "綠野路口".into(),
description: "預設塔防流程的小型路線".into(),
reward: "100 金幣".into(),
label: "簡單".into(),
description: "放鬆波次與寬裕經濟".into(),
reward: "1x 獎勵".into(),
label: "中級".into(),
description: "標準平衡挑戰".into(),
reward: "1.25x 獎勵".into(),
label: "困難".into(),
description: "經濟更緊，失誤空間更少".into(),
reward: "1.5x 獎勵".into(),
```

- [ ] **Step 5: Localize base catalog**

Update `scripts/base_content/assets/pregame_ui/catalog.json` with these content changes:

```json
{
  "id": "main_menu",
  "title": "Omoba 塔防",
  "subtitle": "準備守住路線，選擇難度與地圖後開始。",
  "widgets": [
    {
      "id": "start",
      "label": "開始",
      "description": "選擇難度",
      "action": { "kind": "Navigate", "target": "difficulty_select" }
    },
    {
      "id": "heroes",
      "label": "英雄",
      "description": "即將開放"
    },
    {
      "id": "settings",
      "label": "設定",
      "description": "即將開放"
    }
  ]
}
```

Also localize screen titles, map labels/descriptions/rewards, and difficulty labels/descriptions/rewards to the same Chinese copy as fallback.

- [ ] **Step 6: Run task tests**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests
```

Expected: all `pregame::tests` pass.

### Task 2: Native Button Model Tests

**Files:**
- Modify: `omfx/game/src/native.rs`

- [ ] **Step 1: Update failing native tests for the new flow**

In `pregame_button_model_is_catalog_driven_for_each_screen`, update expectations:

```rust
        let main = game.current_pregame_buttons();
        assert!(main.iter().any(|(label, _, active, action)| {
            label == "開始"
                && *active
                && matches!(action, pregame::PregameAction::Navigate { target } if target == "difficulty_select")
        }));

        game.pregame_runtime.state = pregame::PregameState::DifficultySelect;
        let difficulties = game.current_pregame_buttons();
        assert!(matches!(difficulties[0].3, pregame::PregameAction::Back));
        assert_eq!(difficulties[0].0, "返回");
        assert!(difficulties.iter().any(|(label, _, active, action)| {
            label == "簡單"
                && *active
                && matches!(action, pregame::PregameAction::SelectDifficulty { difficulty_id } if difficulty_id == "easy")
        }));

        game.pregame_runtime.selected_difficulty = Some(
            game.pregame_runtime
                .catalog
                .difficulty("easy")
                .unwrap()
                .clone(),
        );
        game.pregame_runtime.state = pregame::PregameState::MapSelect;
        let maps = game.current_pregame_buttons();
        assert!(matches!(maps[0].3, pregame::PregameAction::Back));
        assert_eq!(maps[0].0, "返回");
        assert!(maps.iter().any(|(label, _, active, action)| {
            label == "綠野路口"
                && *active
                && matches!(action, pregame::PregameAction::SelectMap { map_id } if map_id == "td_1")
        }));
```

Update `pregame_click_dispatch_consumes_menu_input_without_starting_session` to click `difficulty_select` and expect `DifficultySelect`.

- [ ] **Step 2: Run native tests to verify failure**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib native::tests::pregame_button_model_is_catalog_driven_for_each_screen native::tests::pregame_click_dispatch_consumes_menu_input_without_starting_session
```

Expected: tests fail until `current_pregame_buttons()` returns the new Chinese labels and difficulty-first actions.

- [ ] **Step 3: Update `current_pregame_buttons()`**

In `omfx/game/src/native.rs`, change hard-coded English back/status labels:

```rust
("返回".to_string(), String::new(), true, pregame::PregameAction::Back)
("啟動中...".to_string(), "請稍候".to_string(), false, pregame::PregameAction::NoOp)
("返回選單".to_string(), String::new(), true, pregame::PregameAction::Back)
```

Keep catalog-driven labels for main widgets, maps, and difficulties.

- [ ] **Step 4: Run task tests**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib native::tests::pregame_button_model_is_catalog_driven_for_each_screen native::tests::pregame_click_dispatch_consumes_menu_input_without_starting_session
```

Expected: both tests pass.

### Task 3: State-Specific Pregame Layout

**Files:**
- Modify: `omfx/game/src/native.rs`

- [ ] **Step 1: Add visual roles to pregame UI nodes**

Change `PregameButtonUi` to:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PregameVisualRole {
    Button,
    Decoration,
}

#[derive(Debug, Default)]
struct PregameButtonUi {
    bg: Handle<UiNode>,
    text: Handle<Text>,
    role: PregameVisualRole,
}
```

Implement:

```rust
impl Default for PregameVisualRole {
    fn default() -> Self {
        Self::Button
    }
}
```

- [ ] **Step 2: Increase the node pool**

In initialization, replace `for _ in 0..12` with:

```rust
        for _ in 0..32 {
```

Create `PregameButtonUi { bg, text, role: PregameVisualRole::Button }`.

- [ ] **Step 3: Add layout helper functions**

Add helper functions near `pregame_button_text`:

```rust
fn pregame_ref_rect(window_size: Vector2<f32>, x: f32, y: f32, w: f32, h: f32) -> UiRect {
    let scale = (window_size.x / 2048.0).min(window_size.y / 1152.0).max(0.01);
    let content_w = 2048.0 * scale;
    let content_h = 1152.0 * scale;
    UiRect {
        x: (window_size.x - content_w) * 0.5 + x * scale,
        y: (window_size.y - content_h) * 0.5 + y * scale,
        w: w * scale,
        h: h * scale,
    }
}

fn pregame_button_label(label: &str, description: &str, active: bool) -> String {
    let mut lines = vec![label.trim().to_string()];
    if !description.trim().is_empty() {
        lines.push(description.trim().to_string());
    }
    if !active {
        lines.push("鎖定".to_string());
    }
    lines.join("\n")
}
```

- [ ] **Step 4: Add send helper methods**

Add methods on `impl Game` near `update_pregame_ui`:

```rust
    fn place_pregame_node(
        &mut self,
        ui: &mut UserInterface,
        index: &mut usize,
        rect: UiRect,
        text: String,
        active: bool,
        action: pregame::PregameAction,
        role: PregameVisualRole,
    ) {
        if *index >= self.ui_pregame.buttons.len() {
            return;
        }
        let node = &mut self.ui_pregame.buttons[*index];
        node.role = role;
        ui.send(node.bg, WidgetMessage::DesiredPosition(rect.pos()));
        ui.send(node.bg, WidgetMessage::Width(rect.w));
        ui.send(node.bg, WidgetMessage::Height(rect.h));
        ui.send(
            node.text,
            WidgetMessage::DesiredPosition(Vector2::new(rect.x + 8.0, rect.y + 4.0)),
        );
        ui.send(node.text, WidgetMessage::Width((rect.w - 16.0).max(1.0)));
        ui.send(node.text, WidgetMessage::Height((rect.h - 8.0).max(1.0)));
        ui.send(node.text, TextMessage::Text(text));
        if active {
            self.pregame_button_rects.push((rect, action));
        }
        *index += 1;
    }
```

- [ ] **Step 5: Split `update_pregame_ui()` by state**

Replace the shared grid block after background/title/status setup with a state dispatch:

```rust
        self.pregame_button_rects.clear();
        let mut node_index = 0;
        match self.pregame_runtime.state {
            pregame::PregameState::MainMenu => {
                self.layout_pregame_home(ui, &mut node_index);
            }
            pregame::PregameState::DifficultySelect => {
                self.layout_pregame_difficulty(ui, &mut node_index);
            }
            pregame::PregameState::MapSelect => {
                self.layout_pregame_maps(ui, &mut node_index);
            }
            pregame::PregameState::StartingSession | pregame::PregameState::SessionEnded => {
                for (label, description, active, action) in self.current_pregame_buttons() {
                    let rect = pregame_ref_rect(self.window_size, 744.0, 520.0, 560.0, 120.0);
                    self.place_pregame_node(
                        ui,
                        &mut node_index,
                        rect,
                        pregame_button_label(&label, &description, active),
                        active,
                        action,
                        PregameVisualRole::Button,
                    );
                }
            }
            pregame::PregameState::InGame => {}
        }
        self.hide_unused_pregame_nodes(ui, node_index);
```

- [ ] **Step 6: Implement home layout**

Add `layout_pregame_home()` that places:

```rust
// left side utility circles
("設定", 28.0, 185.0, 96.0, 96.0)
("任務", 28.0, 325.0, 96.0, 96.0)
("商店", 28.0, 465.0, 96.0, 96.0)
// bottom buttons from catalog, with start centered and larger
```

Use the catalog start widget action for the green center button. Disabled items use `PregameAction::NoOp` and are not clickable.

- [ ] **Step 7: Implement difficulty layout**

Add `layout_pregame_difficulty()` with a `返回` button at reference rect `(36, 28, 96, 96)` and three difficulty card rects:

```rust
[
    (560.0, 430.0, 280.0, 190.0),
    (884.0, 390.0, 280.0, 190.0),
    (1208.0, 430.0, 280.0, 190.0),
]
```

Each card text is `"{label}\n獎勵：{reward}"` if reward exists, otherwise `label`.

- [ ] **Step 8: Implement map layout**

Add `layout_pregame_maps()` with a `返回` button at `(36, 28, 96, 96)`, up to six map card rects:

```rust
[
    (370.0, 135.0, 380.0, 170.0),
    (834.0, 135.0, 380.0, 170.0),
    (1298.0, 135.0, 380.0, 170.0),
    (370.0, 470.0, 380.0, 170.0),
    (834.0, 470.0, 380.0, 170.0),
    (1298.0, 470.0, 380.0, 170.0),
]
```

Add visual-only category tabs at the bottom with text `新手`, `中級`, `高級`, `專家`.

- [ ] **Step 9: Hide unused nodes**

Add:

```rust
    fn hide_unused_pregame_nodes(&mut self, ui: &mut UserInterface, used: usize) {
        for button in self.ui_pregame.buttons.iter_mut().skip(used) {
            button.role = PregameVisualRole::Button;
            ui.send(
                button.bg,
                WidgetMessage::DesiredPosition(Vector2::new(UI_HIDDEN_POS, UI_HIDDEN_POS)),
            );
            ui.send(
                button.text,
                WidgetMessage::DesiredPosition(Vector2::new(UI_HIDDEN_POS, UI_HIDDEN_POS)),
            );
            ui.send(button.text, TextMessage::Text(String::new()));
        }
    }
```

- [ ] **Step 10: Run native tests**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib native::tests::pregame_button_model_is_catalog_driven_for_each_screen native::tests::pregame_click_dispatch_consumes_menu_input_without_starting_session
```

Expected: tests pass.

### Task 4: Verification

**Files:**
- Test only.

- [ ] **Step 1: Run pregame tests**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests native::tests::pregame_button_model_is_catalog_driven_for_each_screen native::tests::pregame_click_dispatch_consumes_menu_input_without_starting_session
```

Expected: all named tests pass.

- [ ] **Step 2: Run broader omfx lib tests if time permits**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib
```

Expected: pass, or document unrelated pre-existing failures with exact failing test names.

- [ ] **Step 3: Optional visual run**

Run from repo root:

```powershell
.\run.bat
```

Expected: native window opens to the Chinese pregame home screen. Clicking `開始` opens difficulty selection; choosing a difficulty opens map selection; choosing a playable map starts gameplay.
