# Tower Ability Button Tooltip Implementation Plan

**Goal:** Make every tower active-ability button display only a large ability name, with complete ability details and live state shown in a hover tooltip.

**Architecture:** Add authored total duration to the existing render-only `TowerActiveAbilitySnapshot`, then derive button text and tooltip copy through pure helpers in `omfx`. Keep all cast input, slot binding, pagination, cooldown overlays, and backend authority unchanged. Remove the tower ability bar's image-specific presentation because the approved button surface is text-only.

**Tech Stack:** Rust 1.95.0, Fyrox UI, Cargo tests, `omoba-core` render snapshots, `omfx` submodule

---

### Task 1: Expose authored duration in the render snapshot

**Files:**
- Modify: `omoba-core/src/runtime/native/snapshot.rs`
- Test: `omoba-core/src/runtime/native/snapshot.rs`

**Step 1: Add a failing snapshot test**

In the existing `snapshot.rs` test module, import `TowerActiveAbilityState` and add:

```rust
#[test]
fn tower_active_ability_snapshot_exposes_total_duration() {
    let registry = TowerUpgradeRegistry::new();
    let mut tower = Tower::new();
    tower.active_ability = Some(TowerActiveAbilityState::ready("arty_fire_at_will"));

    let snapshot = tower_active_ability_snapshot(Some(&tower), 1, &registry)
        .expect("live tower with unlocked ability should be rendered");

    assert_eq!(snapshot.duration_total, 3.0);
}
```

**Step 2: Run the focused test and verify RED**

```powershell
cargo test --manifest-path omoba-core/Cargo.toml tower_active_ability_snapshot_exposes_total_duration
```

Expected: compilation fails because `TowerActiveAbilitySnapshot` has no `duration_total` field.

**Step 3: Add the minimal snapshot field**

Add this field after `cooldown_total`:

```rust
pub duration_total: f32,
```

Populate it in `tower_active_ability_snapshot` from the already-resolved definition:

```rust
duration_total: def.duration.to_f32_for_render(),
```

This is render-only data. Do not alter ability state, upgrade metadata, serialization protocol, or cooldown scheduling.

**Step 4: Verify GREEN and snapshot regressions**

```powershell
cargo test --manifest-path omoba-core/Cargo.toml tower_active_ability_snapshot_exposes_total_duration
cargo test --manifest-path omoba-core/Cargo.toml runtime::native::snapshot::tests
```

Expected: the new test and all snapshot tests pass.

### Task 2: Build pure button and tooltip presentation models

**Files:**
- Modify: `omfx/game/src/native.rs`
- Test: `omfx/game/src/native.rs`

**Step 1: Update the test fixture for the new snapshot field**

In `ability_entity`, add:

```rust
duration_total: 5.0,
```

This is a compatibility edit required before the frontend tests can compile against Task 1.

**Step 2: Add failing button-label tests**

Add focused tests that express the approved button surface:

```rust
#[test]
fn ability_bar_button_contains_only_the_ability_name() {
    assert_eq!(ability_bar_button_text("甜點狂歡"), "甜點狂歡");
}

#[test]
fn ability_bar_button_wraps_a_long_name_into_two_complete_lines() {
    let text = ability_bar_button_text("超級猴子粉絲俱樂部");

    assert_eq!(text.lines().count(), 2);
    assert_eq!(text.replace('\n', ""), "超級猴子粉絲俱樂部");
}
```

**Step 3: Add failing tooltip-model tests**

Replace the existing minimal tooltip assertion with tests covering ready, active, cooling, invalid data, and rejection matching. Build the item through `ability_bar_items_with_names`, then assert that the model description contains the stable information rather than incidental whitespace:

```rust
#[test]
fn ability_bar_ready_tooltip_contains_authored_details_and_shortcut() {
    let entities = vec![ability_entity(
        1,
        0,
        "tower_cake_splash",
        "甜點狂歡",
        "party.png",
        0.0,
    )];
    let names = HashMap::from([("tower_cake_splash".to_string(), "蛋糕濺射塔".to_string())]);
    let item = ability_bar_items_with_names(&entities, &names, 7, 0, 0.0).remove(0);
    let tooltip = ability_bar_tooltip_model(Some(&item), None).unwrap();

    assert_eq!(tooltip.title, "甜點狂歡");
    assert!(tooltip.description.contains("蛋糕濺射塔"));
    assert!(tooltip.description.contains("description"));
    assert!(tooltip.description.contains("10.0"));
    assert!(tooltip.description.contains("5.0"));
    assert!(tooltip.description.contains("準備完成"));
    assert!(tooltip.description.contains("1"));
}
```

Add separate tests that mutate the fixture snapshot to prove:

- `active_remaining = 2.5` produces `施放中：剩餘 2.5 秒`.
- `cooldown_remaining = 7.5` produces `冷卻中：剩餘 7.5 秒`.
- zero duration produces `瞬發`.
- empty description produces `沒有可用的技能描述。`.
- `NaN`, infinity, and negative timing values render as `0.0`, never `NaN`, `inf`, or a negative duration.
- a matching `AbilityBarRejection` adds `上次施放失敗：<reason>`.
- a different `AbilityBarKey` does not add rejection text.
- `ability_bar_tooltip_model(None, ...)` returns `None`.

**Step 4: Run the focused tests and verify RED**

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx ability_bar_button_
cargo test --manifest-path omfx/Cargo.toml -p omfx ability_bar_tooltip_
```

Expected: compilation or assertions fail because the new button helper, duration field on `AbilityBarItem`, and enriched tooltip signature do not exist.

**Step 5: Implement safe timing normalization**

Add a pure helper:

```rust
fn non_negative_finite_seconds(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
```

Use it while building `AbilityBarItem` for `cooldown_total`, `duration_total`, `cooldown_remaining`, and `active_remaining`. Add `duration_total: f32` to `AbilityBarItem`.

**Step 6: Implement text-only button copy**

Add `ability_bar_button_text(name: &str) -> String` with these rules:

- Trim surrounding whitespace.
- Use `未命名技能` when empty.
- Return short names unchanged.
- For names longer than six characters, split the complete character sequence near its midpoint into exactly two balanced lines.
- Never add tower name, shortcut, state, rejection, or icon fallback text.

Do not truncate the ability name.

**Step 7: Implement the enriched tooltip model**

Change the helper signature to:

```rust
fn ability_bar_tooltip_model(
    item: Option<&AbilityBarItem>,
    rejection: Option<&AbilityBarRejection>,
) -> Option<AbilityBarTooltipModel>
```

Build localized description lines for tower, authored description/fallback, cooldown, duration/instant, current visual state, shortcut, and a rejection only when `rejection.key == item.key`. Continue using `ability_name` as the title.

**Step 8: Verify the pure models are GREEN**

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx ability_bar_button_
cargo test --manifest-path omfx/Cargo.toml -p omfx ability_bar_tooltip_
```

Expected: every new model test passes.

### Task 3: Wire the text-only Fyrox widgets

**Files:**
- Modify: `omfx/game/src/native.rs`
- Test: `omfx/game/src/native.rs`

**Step 1: Remove obsolete tower-bar icon presentation**

Delete the tower-bar-only `AbilityBarIcon`, `AbilityBarTextureKind`, `ability_bar_texture_kind`, and `resolved_ability_bar_icon` code. Remove `icon` and `fallback_icon` from `AbilityBarItem`, remove `ui_tower_ability_bar_icons` and `tower_ability_bar_cached_icon` from `Game`, and remove their constructor/reset/update logic.

Delete or replace the three obsolete tests that assert tower ability bar icon fallback behavior. Do not remove the separate hero ability icon cache or hero HUD icons.

**Step 2: Make the slot visibly name-only**

In the tower ability bar widget constructor:

- Do not create an image node for each tower ability slot.
- Increase the slot text font from `15.0` to `22.0`.
- Keep horizontal and vertical centering.
- Enable word wrapping.
- Use the full slot width and height for text.

In `update_tower_ability_bar_ui`, replace the multi-line formatted string with:

```rust
let text = ability_bar_button_text(&item.ability_name);
```

Retain existing background, cooldown overlay, active overlay, hit rect, slot binding, and enabled-state logic.

**Step 3: Connect the live rejection-aware tooltip**

Build the tooltip with:

```rust
let tooltip = ability_bar_tooltip_model(
    hovered,
    self.tower_ability_bar_rejection.visible.as_ref(),
);
```

Because `AbilityBarItem` is rebuilt from elapsed simulation time each update, the formatted active/cooldown text changes in the tooltip model and invalidates the existing cached model while the pointer remains stationary.

Expand the existing tooltip background and description widget to fit all lines, and update the window-edge clamp and vertical offset using the new dimensions. Keep the tooltip hidden when no slot is hovered.

**Step 4: Run the focused ability-bar regression suite**

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx ability_bar_
```

Expected: button, tooltip, cast, slot, pagination, rejection, cooldown, and ownership tests pass.

**Step 5: Commit the `omfx` submodule change**

```powershell
git -C omfx status --short
git -C omfx diff --check
git -C omfx add -- game/src/native.rs
git -C omfx commit -m "feat: simplify tower ability buttons"
```

Only `game/src/native.rs` should be included in the submodule commit.

### Task 4: Verify and integrate the complete change

**Files:**
- Verify: `omoba-core/src/runtime/native/snapshot.rs`
- Verify: `omfx/game/src/native.rs`
- Update submodule pointer: `omfx`

**Step 1: Run formatting checks for touched Rust files**

```powershell
rustfmt --edition 2021 --check omoba-core/src/runtime/native/snapshot.rs
rustfmt --edition 2021 --check omfx/game/src/native.rs
```

Expected: both touched files pass.

**Step 2: Run complete relevant test suites**

```powershell
cargo test --manifest-path omoba-core/Cargo.toml
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib
```

Expected: both suites pass. The existing `protoc not found` fallback warning is acceptable; test failures are not.

**Step 3: Inspect submodule and root diffs**

```powershell
git -C omfx status --short
git status --short
git diff --check
git diff --submodule=log -- omoba-core/src/runtime/native/snapshot.rs omfx
```

Expected: `omfx` itself is clean; root shows the `omoba-core` snapshot edit, the new `omfx` pointer, and the pre-existing untracked `omfue` entry only. Do not stage or modify `omfue`.

**Step 4: Commit root integration**

```powershell
git add -- omoba-core/src/runtime/native/snapshot.rs omfx
git diff --cached --check
git commit -m "feat: show tower ability details on hover"
```

Do not push unless the user explicitly requests it.

**Step 5: Verify final commit state**

```powershell
git status --short
git -C omfx status --short
git log --oneline -5
git -C omfx log --oneline -3
```

Expected: only the pre-existing `omfue` entry remains in root status, both implementation repositories contain their intended commits, and no unrelated files were included.
