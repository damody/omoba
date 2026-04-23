# Map Editor Wave Mode Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 為 `map_editor` 新增全螢幕 Wave 編輯模式（`ViewMode::Waves`）— 三欄式（Wave 列表 / Timeline / Inspector），可視覺化編輯 `CreepWaveJD` 結構，含拖曳 Time、右鍵選單、zoom、Add/Dup/Del wave/detail/spawn。

**Architecture:** 純前端編輯器擴充，不改 schema、後端、IO。新增 4 個 panel 檔，擴充 `Selection` 與 `AppState`。沿用既有 `begin_edit(tag)` undo 機制。Timeline 用 `paint_*` API 自繪。

**Tech Stack:** Rust 2021、`eui` (本地 crate)、`serde_json`、本專案 `schema.rs::CreepWaveData`

**設計來源：** `docs/plans/2026-04-23-map-editor-wave-mode-design.md`

**測試策略：**
- 純資料 mutation 函式（add_wave / duplicate_wave / drag_time math） → 寫單元測試（仿 `schema.rs::tests`）
- UI 渲染 / 事件 → 手動驗證（`cargo run -- D:/omoba/omb/Story/TD_1`），每 task 含驗證清單

**前置作業（執行此 plan 前）：**
- 在 worktree 中執行（`git worktree add ../omoba-wave-editor`）；本機已在 master，subagent-driven 模式可直接 in-place 但建議先 stash 子模組變更
- 確認 `cd D:/omoba/map_editor && cargo build` 目前通過

---

## Phase P1：骨架（讓 Waves 模式按鈕能切換、三欄空版面顯示）

### Task 1: 擴充 `Selection` enum 與 `ViewMode`

**Files:**
- Modify: `D:/omoba/map_editor/src/app.rs:30` (ViewMode enum)
- Modify: `D:/omoba/map_editor/src/app.rs:7` (Selection enum)

**Step 1: 在 `ViewMode` 加 `Waves` 變體**

修改 `app.rs:44-48`：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Map,
    Entities,
    Waves,
}
```

**Step 2: 在 `Selection` 加 3 個 wave 變體**

修改 `app.rs:7-20`，在 `Enemy(usize)` 後加：

```rust
    /// 選中整波（Inspector 顯示 Name / StartTime / + Detail）
    Wave(usize),
    /// (wave_idx, detail_idx) — 選中 wave 內某條 lane
    WaveDetail(usize, usize),
    /// (wave, detail, spawn) — 選中 timeline 上的某顆 spawn 圓
    WaveSpawn(usize, usize, usize),
```

**Step 3: 編譯**

```bash
cd D:/omoba/map_editor && cargo build 2>&1 | tail -20
```
Expected: 編譯通過（可能有 unused warning，OK）

**Step 4: Commit**

```bash
cd D:/omoba/map_editor && git add src/app.rs && git commit -m "feat(map_editor): add ViewMode::Waves and Wave* Selection variants"
```

---

### Task 2: 新增 `WaveEditState` 結構並掛入 `AppState`

**Files:**
- Modify: `D:/omoba/map_editor/src/app.rs`

**Step 1: 在 `app.rs` 末尾加 `WaveEditState` 與相關型別**

```rust
// ── Wave 編輯模式專屬狀態 ─────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveZoom {
    Fit,
    Fixed(f32),
}

impl Default for WaveZoom {
    fn default() -> Self { WaveZoom::Fit }
}

#[derive(Debug, Clone, Copy)]
pub struct SpawnDrag {
    pub sel: (usize, usize, usize),
    pub start_mouse_x: f32,
    pub orig_time: f32,
    pub batch_after: bool,
}

#[derive(Debug, Clone)]
pub enum CtxMenu {
    Empty {
        wave: usize,
        detail: usize,
        time: f32,
        screen_pos: (f32, f32),
    },
    Spawn {
        sel: (usize, usize, usize),
        screen_pos: (f32, f32),
    },
}

#[derive(Debug, Clone, Default)]
pub struct WaveEditState {
    pub selected_wave: Option<usize>,
    pub zoom_mode: WaveZoom,
    pub scroll_x: f32,
    pub drag: Option<SpawnDrag>,
    pub context_menu: Option<CtxMenu>,
    pub last_inserted_creep: Option<String>,
    /// 二次點擊確認刪除：(wave_idx, 第一次點擊時間)
    pub pending_delete_wave: Option<(usize, std::time::Instant)>,
}
```

**Step 2: 在 `AppState` struct 加欄位**

在 `app.rs` `AppState` struct 內 `pub undo: UndoStack,` 之前加：

```rust
    pub wave_edit: WaveEditState,
```

**Step 3: 在 `Default for AppState` 加初始化**

在 `undo: UndoStack::new()` 前加：

```rust
            wave_edit: WaveEditState::default(),
```

**Step 4: 編譯確認**

```bash
cd D:/omoba/map_editor && cargo build 2>&1 | tail -10
```
Expected: 通過

**Step 5: Commit**

```bash
git add src/app.rs && git commit -m "feat(map_editor): add WaveEditState for wave editor UI state"
```

---

### Task 3: 加 Waves 切換按鈕到 toolbar

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/toolbar.rs:155`

**Step 1: 在 Entities 按鈕後加 Waves 按鈕**

在 `toolbar.rs:157` （Entities button 結尾的 `}` 之後）新增：

```rust
            {
                let br = Rect::new(x, row.y, cell_w, row.h);
                x += cell_w + TOOLBAR_CELL_GAP;
                let style = if app.view_mode == ViewMode::Waves {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Ghost
                };
                if ui.button("Waves").rect(br).style(style).draw() {
                    app.view_mode = ViewMode::Waves;
                }
            }
```

**Step 2: 編譯**

```bash
cd D:/omoba/map_editor && cargo build 2>&1 | tail -5
```

**Step 3: 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：toolbar 看到第三顆 `Waves` 按鈕，點擊後變 Primary 高亮（畫面內容暫不變，正常）

**Step 4: Commit**

```bash
git add src/panels/toolbar.rs && git commit -m "feat(map_editor): add Waves toggle button in toolbar"
```

---

### Task 4: 在 `style.rs` 加 Wave 模式專用尺寸常數

**Files:**
- Modify: `D:/omoba/map_editor/src/style.rs`

**Step 1: 在 `style.rs` 末尾加常數**

```rust
// ---- Wave 編輯模式 ----
const BASE_WAVE_LIST_W: f32 = 150.0;
const BASE_WAVE_LANE_H: f32 = 36.0;
const BASE_WAVE_HEADER_H: f32 = 28.0;
const BASE_WAVE_RULER_H: f32 = 18.0;

pub const WAVE_LIST_W: f32 = BASE_WAVE_LIST_W * UI_SCALE;
pub const WAVE_LANE_H: f32 = BASE_WAVE_LANE_H * UI_SCALE;
pub const WAVE_HEADER_H: f32 = BASE_WAVE_HEADER_H * UI_SCALE;
pub const WAVE_RULER_H: f32 = BASE_WAVE_RULER_H * UI_SCALE;
pub const WAVE_DOT_R: f32 = 9.0 * UI_SCALE;
pub const WAVE_PX_PER_SEC_DEFAULT: f32 = 50.0;
pub const WAVE_PX_PER_SEC_MIN: f32 = 10.0;
pub const WAVE_PX_PER_SEC_MAX: f32 = 500.0;
```

**Step 2: 編譯**

```bash
cd D:/omoba/map_editor && cargo build 2>&1 | tail -5
```

**Step 3: Commit**

```bash
git add src/style.rs && git commit -m "feat(map_editor): add Wave mode style constants"
```

---

### Task 5: 新增 4 個 wave panel 檔案（空骨架）並改 `panels/mod.rs`

**Files:**
- Create: `D:/omoba/map_editor/src/panels/wave_list.rs`
- Create: `D:/omoba/map_editor/src/panels/wave_timeline.rs`
- Create: `D:/omoba/map_editor/src/panels/wave_inspector.rs`
- Modify: `D:/omoba/map_editor/src/panels/mod.rs`
- Modify: `D:/omoba/map_editor/src/panels/waves.rs` (整個改寫)

**Step 1: 改 `panels/mod.rs`**

```rust
pub mod toolbar;
pub mod templates;
pub mod inspector;
pub mod waves;
pub mod wave_list;
pub mod wave_timeline;
pub mod wave_inspector;
```

**Step 2: 建立 `wave_list.rs`（暫時空 panel）**

```rust
use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::style::{FS_LABEL, LH_LABEL};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);
        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Wave List").font_size(FS_LABEL).height(LH_LABEL).draw();
            let _ = app;
        });
    });
}
```

**Step 3: 建立 `wave_timeline.rs`（暫時空 panel）**

```rust
use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::style::{FS_LABEL, LH_LABEL};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let bg = eui::rgba(0.10, 0.11, 0.12, 1.0);
        let r = ui.content_rect();
        ui.paint_filled_rect(r, bg, 0.0);
        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Wave Timeline").font_size(FS_LABEL).height(LH_LABEL).draw();
            let _ = app;
        });
    });
}
```

**Step 4: 建立 `wave_inspector.rs`（暫時空 panel）**

```rust
use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::style::{FS_LABEL, LH_LABEL};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);
        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Wave Inspector").font_size(FS_LABEL).height(LH_LABEL).draw();
            let _ = app;
        });
    });
}
```

**Step 5: 改寫 `panels/waves.rs` 為三欄分派**

整個替換 `waves.rs` 的內容：

```rust
use eui::quick::ui::UI;
use eui::Rect;

use crate::app::AppState;
use crate::style::WAVE_LIST_W;
use crate::panels::{wave_list, wave_timeline, wave_inspector};

/// Waves 模式三欄分派：左 wave 列表｜中 timeline｜右 inspector
pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    let list_w = WAVE_LIST_W;
    let inspector_w = app.inspector_w.max(crate::style::INSPECTOR_MIN_W);
    let timeline_w = (rect.w - list_w - inspector_w).max(100.0);

    let list_rect = Rect::new(rect.x, rect.y, list_w, rect.h);
    let timeline_rect = Rect::new(rect.x + list_w, rect.y, timeline_w, rect.h);
    let inspector_rect = Rect::new(rect.x + list_w + timeline_w, rect.y, inspector_w, rect.h);

    wave_list::draw(ui, list_rect, app);
    wave_timeline::draw(ui, timeline_rect, app);
    wave_inspector::draw(ui, inspector_rect, app);
}
```

**Step 6: 編譯**

```bash
cd D:/omoba/map_editor && cargo build 2>&1 | tail -10
```

**Step 7: Commit**

```bash
git add src/panels/ && git commit -m "feat(map_editor): scaffold wave_list/wave_timeline/wave_inspector panels"
```

---

### Task 6: `main.rs` 加 Waves 模式 layout 分支

**Files:**
- Modify: `D:/omoba/map_editor/src/main.rs:120-167`

**Step 1: 在現有 layout 區塊外包一層 `match state.view_mode`**

找到 `main.rs:163-167`（原本的 toolbar / templates / inspector / waves / canvas draw 呼叫），改成：

```rust
        panels::toolbar::draw(ui, toolbar_rect, &mut state);

        match state.view_mode {
            crate::app::ViewMode::Waves => {
                let body_rect = Rect::new(
                    content.x,
                    content.y + toolbar_h,
                    content.w,
                    (content.h - toolbar_h).max(0.0),
                );
                panels::waves::draw(ui, body_rect, &mut state);
            }
            _ => {
                panels::templates::draw(ui, templates_rect, &mut state);
                panels::inspector::draw(ui, inspector_rect, &mut state);
                panels::waves::draw(ui, waves_rect, &mut state);
                canvas::draw(ui, canvas_rect, &mut state);
            }
        }
```

注意：splitter 拖拉邏輯（`main.rs:170-200` 區塊）僅在 `_` 分支才有效，要把它包進 `if state.view_mode != ViewMode::Waves { ... }` 或維持原狀（splitter rect 在 Waves 模式下不在畫面上，無效點擊不會發生）。先採後者，後續若有 bug 再包。

**Step 2: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證清單：
- 點 `Map` 按鈕 → 看到原本的編輯器（templates/canvas/inspector/底部 waves）
- 點 `Waves` 按鈕 → 中央切到三欄式空畫面，可看到三個標題：「Wave List」、「Wave Timeline」、「Wave Inspector」
- 點 `Map` 按回 → 復原

**Step 3: Commit**

```bash
git add src/main.rs && git commit -m "feat(map_editor): branch layout for ViewMode::Waves full-screen"
```

---

## Phase P2：顯示（Wave 列表可選 + Timeline 看到 spawn）

### Task 7: Wave List — 列出所有 wave、可選中、選中切 `Selection::Wave`

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_list.rs`

**Step 1: 改寫 `wave_list.rs`**

```rust
use eui::quick::ui::UI;
use eui::{ButtonStyle, Rect};

use crate::app::{AppState, Selection};
use crate::style::{FS_LABEL, FS_SUBHEAD, LH_LABEL};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);
        let inner = eui::quick::ui::inset(&r, 8.0, 8.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Waves").font_size(FS_SUBHEAD).height(LH_LABEL).draw();
            ui.spacer(4.0);

            let waves = app.map.CreepWave.clone();
            for (i, w) in waves.iter().enumerate() {
                let total: usize = w.Detail.iter().map(|d| d.Creeps.len()).sum();
                let is_selected = app.wave_edit.selected_wave == Some(i);
                let style = if is_selected { ButtonStyle::Primary } else { ButtonStyle::Ghost };
                let label = format!("{}  t={:.1}s  x{}", w.Name, w.StartTime, total);
                if ui.button(&label).style(style).draw() {
                    app.wave_edit.selected_wave = Some(i);
                    app.selection = Selection::Wave(i);
                }
                ui.spacer(2.0);
            }

            if waves.is_empty() {
                ui.label("(無 wave)").font_size(FS_LABEL).draw();
            }
        });
    });
}
```

**Step 2: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：切到 Waves 模式 → 左欄看到 W01~W05 共 5 顆按鈕 → 點 W02 → 變 Primary 高亮

**Step 3: Commit**

```bash
git add src/panels/wave_list.rs && git commit -m "feat(map_editor): wave_list shows waves and updates selection on click"
```

---

### Task 8: Timeline — 顯示時間刻度 + lane 背景（無 spawn dot）

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`

**Step 1: 改寫 `wave_timeline.rs` 為基本繪製**

```rust
use eui::quick::ui::UI;
use eui::{Rect, TextAlign};

use crate::app::{AppState, WaveZoom};
use crate::style::{
    FS_CAPTION, FS_LABEL, FS_SUBHEAD, LH_LABEL, WAVE_HEADER_H, WAVE_LANE_H,
    WAVE_PX_PER_SEC_DEFAULT, WAVE_RULER_H,
};

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let bg = eui::rgba(0.10, 0.11, 0.12, 1.0);
        let r = ui.content_rect();
        ui.paint_filled_rect(r, bg, 0.0);

        let Some(w_idx) = app.wave_edit.selected_wave else {
            // 無選中時顯示提示
            let text_color = eui::rgba(0.6, 0.6, 0.6, 1.0);
            ui.ctx().paint_text(r, "(請從左側選擇一個 Wave)",
                FS_LABEL, text_color, TextAlign::Center);
            return;
        };
        if w_idx >= app.map.CreepWave.len() {
            return;
        }
        let wave = app.map.CreepWave[w_idx].clone();

        // 計算 px_per_sec
        let total_sec = wave.Detail.iter()
            .flat_map(|d| d.Creeps.iter().map(|c| c.Time))
            .fold(0.0_f32, f32::max)
            + 0.5;
        let total_sec = total_sec.max(1.0);
        let px_per_sec = match app.wave_edit.zoom_mode {
            WaveZoom::Fit => (r.w - 16.0) / total_sec,
            WaveZoom::Fixed(s) => s,
        };

        // 標題列：Wave 名稱 + 模式按鈕（Fit/Fixed）
        let header = Rect::new(r.x, r.y, r.w, WAVE_HEADER_H);
        let title = format!("{}  StartTime={:.1}s", wave.Name, wave.StartTime);
        let muted = ui.theme().muted_text;
        ui.ctx().paint_text(header, &title, FS_SUBHEAD, muted, TextAlign::Left);

        // ruler：每秒一條豎線 + 數字
        let ruler_y = r.y + WAVE_HEADER_H;
        let ruler_rect = Rect::new(r.x + 8.0, ruler_y, r.w - 16.0, WAVE_RULER_H);
        let ruler_color = eui::rgba(0.25, 0.27, 0.30, 1.0);
        ui.paint_filled_rect(ruler_rect, ruler_color, 0.0);
        let scroll_x = app.wave_edit.scroll_x;
        let max_visible_sec = ((r.w - 16.0) / px_per_sec).ceil() as i32 + 1;
        for s in 0..max_visible_sec {
            let cx = ruler_rect.x + s as f32 * px_per_sec - scroll_x;
            if cx < ruler_rect.x || cx > ruler_rect.x + ruler_rect.w {
                continue;
            }
            let line = Rect::new(cx - 0.5, ruler_y, 1.0, WAVE_RULER_H);
            ui.paint_filled_rect(line, eui::rgba(0.5, 0.5, 0.5, 1.0), 0.0);
            let lbl = Rect::new(cx + 2.0, ruler_y, 30.0, WAVE_RULER_H);
            ui.ctx().paint_text(lbl, &format!("{}s", s), FS_CAPTION,
                eui::rgba(0.7, 0.7, 0.7, 1.0), TextAlign::Left);
        }

        // lane 區塊
        let lanes_y = ruler_y + WAVE_RULER_H + 4.0;
        for (di, detail) in wave.Detail.iter().enumerate() {
            let ly = lanes_y + di as f32 * (WAVE_LANE_H + 2.0);
            let lane_rect = Rect::new(r.x + 8.0, ly, r.w - 16.0, WAVE_LANE_H);
            let zebra = if di % 2 == 0 {
                eui::rgba(0.16, 0.17, 0.19, 1.0)
            } else {
                eui::rgba(0.13, 0.14, 0.16, 1.0)
            };
            ui.paint_filled_rect(lane_rect, zebra, 4.0);

            // lane header（左邊路徑名）
            let header_rect = Rect::new(lane_rect.x + 6.0, lane_rect.y + 4.0,
                                         120.0, WAVE_LANE_H - 8.0);
            ui.ctx().paint_text(header_rect, &detail.Path, FS_LABEL,
                eui::rgba(0.85, 0.85, 0.85, 1.0), TextAlign::Left);
        }

        let _ = app; // suppress unused for now
    });
}
```

**Step 2: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：Waves 模式選 W01 → 中央顯示「W01  StartTime=0.0s」標題、時間刻度 0s 1s 2s ...、一條 lane 顯示 `td_main` 字樣

**Step 3: Commit**

```bash
git add src/panels/wave_timeline.rs src/app.rs && git commit -m "feat(map_editor): wave_timeline shows ruler and lane backgrounds"
```

---

### Task 9: Timeline — 加 spawn dot 繪製（圓 + 字母 + 顏色）

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`

**Step 1: 在 `wave_timeline.rs` 加 helper 函式（檔案頂部 use 之後）**

```rust
use crate::style::WAVE_DOT_R;

/// 由 creep_name hash 決定顏色（固定 8 色 palette）
fn creep_color(name: &str) -> eui::GfxColor {
    const PALETTE: [(f32, f32, f32); 8] = [
        (0.30, 0.78, 0.45),   // 綠
        (0.85, 0.30, 0.30),   // 紅
        (0.30, 0.55, 0.85),   // 藍
        (0.95, 0.75, 0.20),   // 黃
        (0.75, 0.40, 0.85),   // 紫
        (0.40, 0.80, 0.80),   // 青
        (0.95, 0.55, 0.25),   // 橙
        (0.65, 0.65, 0.70),   // 灰
    ];
    let mut h: u32 = 5381;
    for b in name.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u32); }
    let (r, g, b) = PALETTE[(h as usize) % PALETTE.len()];
    eui::rgba(r, g, b, 1.0)
}

fn creep_letter(name: &str) -> String {
    name.chars()
        .skip_while(|c| !c.is_ascii_alphabetic())
        .take(1)
        .collect::<String>()
        .to_uppercase()
}
```

**Step 2: 在 lane 繪製迴圈內，於 lane header 之後加 spawn dot 繪製**

替換原本 lane header 那段，改成：

```rust
            // lane header
            let header_rect = Rect::new(lane_rect.x + 6.0, lane_rect.y + 4.0,
                                         100.0, WAVE_LANE_H - 8.0);
            ui.ctx().paint_text(header_rect, &detail.Path, FS_LABEL,
                eui::rgba(0.85, 0.85, 0.85, 1.0), TextAlign::Left);

            // spawn dots
            let lane_origin_x = lane_rect.x + 110.0;  // header 後預留
            let cy = lane_rect.y + lane_rect.h * 0.5;
            for (si, spawn) in detail.Creeps.iter().enumerate() {
                let cx = lane_origin_x + spawn.Time * px_per_sec - scroll_x;
                if cx < lane_origin_x - WAVE_DOT_R || cx > lane_rect.x + lane_rect.w + WAVE_DOT_R {
                    continue;
                }
                let dot_rect = Rect::new(cx - WAVE_DOT_R, cy - WAVE_DOT_R,
                                          WAVE_DOT_R * 2.0, WAVE_DOT_R * 2.0);
                let color = creep_color(&spawn.Creep);
                // 用 paint_filled_rect with corner radius 模擬圓
                ui.paint_filled_rect(dot_rect, color, WAVE_DOT_R);

                // 字母
                let letter = creep_letter(&spawn.Creep);
                ui.ctx().paint_text(dot_rect, &letter, FS_LABEL,
                    eui::rgba(1.0, 1.0, 1.0, 1.0), TextAlign::Center);

                // 若 selected → 黃描邊
                if let crate::app::Selection::WaveSpawn(ws, ds, ss) = app.selection {
                    if ws == w_idx && ds == di && ss == si {
                        // 簡單方式：再畫一圈稍大的透明 + 邊框（用 filled rect 不是真 stroke，先簡化）
                        let outline_r = WAVE_DOT_R + 2.0;
                        let outline = Rect::new(cx - outline_r, cy - outline_r,
                                                outline_r * 2.0, outline_r * 2.0);
                        ui.paint_filled_rect(outline,
                            eui::rgba(1.0, 0.9, 0.2, 0.4), outline_r);
                    }
                }
            }
```

注意：`lane_origin_x` 改變後，hit-test 要用同樣的座標換算。

**Step 3: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：選 W01 → lane 上看到 8 個綠色圓點，內含 "T"（td_basic 第一個英文字 "t" 大寫）。選 W02 → 看到綠（B/T 雜在 td_basic）+ 紅或其他色（td_tough）

**Step 4: Commit**

```bash
git add src/panels/wave_timeline.rs && git commit -m "feat(map_editor): render spawn dots with color palette and letter labels"
```

---

## Phase P3：Inspector 表單（純表單編輯，無 timeline 互動）

### Task 10: Wave Inspector — `Selection::Wave` 分支表單

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_inspector.rs`

**Step 1: 改寫 `wave_inspector.rs`**

```rust
use eui::quick::ui::UI;
use eui::Rect;

use crate::app::{AppState, Selection};
use crate::style::{
    FS_BODY, FS_FIELD_LABEL, FS_FIELD_VALUE, FS_HEAD, H_FIELD,
    LH_FIELD_LABEL, LH_HEAD,
};

fn input_str(ui: &mut UI, label: &str, v: &mut String) -> bool {
    ui.input(label, v)
        .label_font_size(FS_FIELD_LABEL)
        .label_height(LH_FIELD_LABEL)
        .height(H_FIELD)
        .value_font_size(FS_FIELD_VALUE)
        .draw()
}

fn input_f32(ui: &mut UI, label: &str, v: &mut f32) -> bool {
    let mut s = format!("{:.2}", v);
    let changed = input_str(ui, label, &mut s);
    if changed {
        if let Ok(parsed) = s.trim().parse::<f32>() {
            *v = parsed;
            return true;
        }
    }
    false
}

pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    ui.scope(rect, |ctx| {
        let mut ui = UI::new(ctx);
        let panel_color = ui.theme().panel;
        let r = ui.content_rect();
        ui.paint_filled_rect(r, panel_color, 0.0);
        let inner = eui::quick::ui::inset(&r, 10.0, 10.0);
        ui.scope(inner, |ctx| {
            let mut ui = UI::new(ctx);
            ui.label("Wave Inspector").font_size(FS_HEAD).height(LH_HEAD).draw();
            ui.spacer(6.0);

            match app.selection {
                Selection::Wave(w) => draw_wave(&mut ui, app, w),
                Selection::WaveDetail(w, d) => draw_detail(&mut ui, app, w, d),
                Selection::WaveSpawn(w, d, s) => draw_spawn(&mut ui, app, w, d, s),
                _ => {
                    ui.label("(請選中 Wave / Detail / Spawn)").font_size(FS_BODY).draw();
                }
            }
        });
    });
}

fn draw_wave(ui: &mut UI, app: &mut AppState, w: usize) {
    if w >= app.map.CreepWave.len() { return; }
    let mut name = app.map.CreepWave[w].Name.clone();
    let mut start_time = app.map.CreepWave[w].StartTime;

    if input_str(ui, "Name", &mut name) {
        app.begin_edit(None);
        app.map.CreepWave[w].Name = name;
        app.dirty = true;
    }
    if input_f32(ui, "StartTime", &mut start_time) {
        app.begin_edit(Some(&format!("wave_starttime_{}", w)));
        app.map.CreepWave[w].StartTime = start_time;
        app.dirty = true;
    }
}

fn draw_detail(ui: &mut UI, app: &mut AppState, w: usize, d: usize) {
    if w >= app.map.CreepWave.len() { return; }
    if d >= app.map.CreepWave[w].Detail.len() { return; }
    let mut path = app.map.CreepWave[w].Detail[d].Path.clone();
    if input_str(ui, "Path", &mut path) {
        app.begin_edit(None);
        app.map.CreepWave[w].Detail[d].Path = path;
        app.dirty = true;
    }
    let count = app.map.CreepWave[w].Detail[d].Creeps.len();
    ui.label(&format!("Spawns: {}", count)).font_size(FS_BODY).draw();
}

fn draw_spawn(ui: &mut UI, app: &mut AppState, w: usize, d: usize, s: usize) {
    if w >= app.map.CreepWave.len() { return; }
    if d >= app.map.CreepWave[w].Detail.len() { return; }
    if s >= app.map.CreepWave[w].Detail[d].Creeps.len() { return; }
    let mut time = app.map.CreepWave[w].Detail[d].Creeps[s].Time;
    let mut creep = app.map.CreepWave[w].Detail[d].Creeps[s].Creep.clone();
    if input_f32(ui, "Time (s)", &mut time) {
        app.begin_edit(Some(&format!("wave_spawn_time_{}_{}_{}", w, d, s)));
        app.map.CreepWave[w].Detail[d].Creeps[s].Time = time.max(0.0);
        app.dirty = true;
    }
    if input_str(ui, "Creep", &mut creep) {
        app.begin_edit(None);
        app.map.CreepWave[w].Detail[d].Creeps[s].Creep = creep;
        app.dirty = true;
    }
}
```

**Step 2: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：點 W01 → 右側顯示 Name "W01" + StartTime "0.00"。改 Name 為 "W01_test" → 左欄列表跟著變。Ctrl+Z 還原。

**Step 3: Commit**

```bash
git add src/panels/wave_inspector.rs && git commit -m "feat(map_editor): wave_inspector forms for Wave/Detail/Spawn selections"
```

---

## Phase P4：Timeline 互動（點擊選中、拖曳改 Time、右鍵選單、Zoom）

### Task 11: Timeline — 點擊 spawn dot 與 lane 空白進行選中

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`

**Step 1: 在 spawn dot 繪製迴圈內加 click hit-test**

需要拿到 mouse_x / mouse_y 與 click event。eui 提供 `ui.ctx().input()` 取得 input 狀態（檢查 `inspector.rs` 是否有先例）。

在 `wave_timeline::draw` 函式內，於畫完所有 dot 之前先記錄 hit 候選，loop 結束後處理 click。完整改動：

於 lane 迴圈外加 mut 變數：

```rust
let mut hit_spawn: Option<(usize, usize, usize)> = None;
let mut hit_lane: Option<(usize, usize)> = None;
let input = ui.ctx().input_snapshot();  // 假設有此 API；若無，改用 ui.ctx().input() 或記錄至 app.prev_mouse_screen
let (mx, my) = (input.mouse_x, input.mouse_y);
let mouse_clicked = input.mouse_pressed_left;  // 確認 eui API 名稱
```

於 spawn dot 繪製內加：

```rust
                let dx = mx - cx;
                let dy = my - cy;
                if dx * dx + dy * dy <= WAVE_DOT_R * WAVE_DOT_R {
                    hit_spawn = Some((w_idx, di, si));
                }
```

於 lane 迴圈內 spawn loop 結束後加 lane click：

```rust
            if hit_spawn.is_none() && lane_rect.contains(mx, my) {
                hit_lane = Some((w_idx, di));
            }
```

於 wave 迴圈外（draw 函式末尾）：

```rust
if mouse_clicked {
    if let Some((w, d, s)) = hit_spawn {
        app.selection = Selection::WaveSpawn(w, d, s);
    } else if let Some((w, d)) = hit_lane {
        app.selection = Selection::WaveDetail(w, d);
    }
}
```

**Step 2: 確認 eui input API**

如果 `input_snapshot()` 不存在，先 grep：

```bash
grep -rn "fn input\b\|input_snapshot\|mouse_pressed" D:/omoba/eui/src/ | head -20
```

依實際 API 調整（可能是 `ui.ctx().input()` 回傳 borrow）。

**Step 3: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 點 spawn dot → 該 dot 出現黃色描邊，右側 Inspector 顯示 Time/Creep
- 點 lane 空白 → 右側 Inspector 顯示 Path 欄位

**Step 4: Commit**

```bash
git add src/panels/wave_timeline.rs && git commit -m "feat(map_editor): click select on timeline (spawn dot / lane)"
```

---

### Task 12: Timeline — 拖曳 spawn 改 Time（含 batch_after for Shift）

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`
- Create: `D:/omoba/map_editor/tests/wave_drag.rs` (單元測試)

**Step 1: 寫單元測試 — drag math**

建立 `D:/omoba/map_editor/tests/wave_drag.rs`：

```rust
use map_editor::schema::*;

fn build_wave(times: &[f32]) -> CreepWaveJD {
    CreepWaveJD {
        Name: "W".into(),
        StartTime: 0.0,
        Detail: vec![DetailJD {
            Path: "p".into(),
            Creeps: times.iter().map(|t| CreepsJD {
                Time: *t,
                Creep: "c".into(),
            }).collect(),
        }],
    }
}

#[test]
fn drag_single_spawn_changes_time_only_for_that_index() {
    let mut w = build_wave(&[0.0, 1.0, 2.0, 3.0]);
    // simulate drag of idx=1 from t=1.0 to t=1.5 (delta = +0.5)
    let orig = w.Detail[0].Creeps[1].Time;
    let new_time = (orig + 0.5).max(0.0);
    w.Detail[0].Creeps[1].Time = new_time;
    assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
    assert_eq!(w.Detail[0].Creeps[1].Time, 1.5);
    assert_eq!(w.Detail[0].Creeps[2].Time, 2.0);
}

#[test]
fn drag_batch_after_shifts_subsequent() {
    let mut w = build_wave(&[0.0, 1.0, 2.0, 3.0]);
    let delta = 0.5_f32;
    let from = 1;
    for s in from..w.Detail[0].Creeps.len() {
        w.Detail[0].Creeps[s].Time = (w.Detail[0].Creeps[s].Time + delta).max(0.0);
    }
    assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
    assert_eq!(w.Detail[0].Creeps[1].Time, 1.5);
    assert_eq!(w.Detail[0].Creeps[2].Time, 2.5);
    assert_eq!(w.Detail[0].Creeps[3].Time, 3.5);
}

#[test]
fn drag_clamps_negative_time_to_zero() {
    let mut w = build_wave(&[1.0]);
    let delta = -5.0_f32;
    w.Detail[0].Creeps[0].Time = (w.Detail[0].Creeps[0].Time + delta).max(0.0);
    assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
}
```

注意：需要在 `D:/omoba/map_editor/Cargo.toml` 確認 `[[bin]]`/`[lib]` 支援 — 若沒 lib，要把 schema 暴露為 `pub mod`。先檢查：

```bash
cat D:/omoba/map_editor/Cargo.toml
```

若只有 `[[bin]]`，要新增 `[lib] name = "map_editor" path = "src/lib.rs"` 並建立 `src/lib.rs` `pub mod schema; ...`。或改用 `#[path = "../src/schema.rs"] mod schema;` 避開 lib 設置。**簡化做法：把測試寫在 `src/wave_ops.rs` 用 `#[cfg(test)] mod tests`**，下述步驟改為此。

**Step 1 (修正): 改在 `src/wave_ops.rs` 寫測試**

建立 `D:/omoba/map_editor/src/wave_ops.rs`：

```rust
//! Pure data mutation helpers for wave editing (testable without UI).
use crate::schema::{CreepWaveJD, DetailJD, CreepsJD};

/// 單一 spawn drag：改某 spawn 的 Time（clamp to 0）
pub fn drag_spawn_time(wave: &mut CreepWaveJD, d: usize, s: usize, new_time: f32) {
    if let Some(detail) = wave.Detail.get_mut(d) {
        if let Some(spawn) = detail.Creeps.get_mut(s) {
            spawn.Time = new_time.max(0.0);
        }
    }
}

/// 批次 drag：將某 detail 從 index `from` 起的所有 spawn 整體位移 delta（clamp to 0）
pub fn drag_spawn_time_batch(wave: &mut CreepWaveJD, d: usize, from: usize, delta: f32) {
    if let Some(detail) = wave.Detail.get_mut(d) {
        for s in from..detail.Creeps.len() {
            let nt = detail.Creeps[s].Time + delta;
            detail.Creeps[s].Time = nt.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_wave(times: &[f32]) -> CreepWaveJD {
        CreepWaveJD {
            Name: "W".into(),
            StartTime: 0.0,
            Detail: vec![DetailJD {
                Path: "p".into(),
                Creeps: times.iter().map(|t| CreepsJD {
                    Time: *t,
                    Creep: "c".into(),
                }).collect(),
            }],
        }
    }

    #[test]
    fn drag_single_changes_only_target() {
        let mut w = build_wave(&[0.0, 1.0, 2.0]);
        drag_spawn_time(&mut w, 0, 1, 1.5);
        assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
        assert_eq!(w.Detail[0].Creeps[1].Time, 1.5);
        assert_eq!(w.Detail[0].Creeps[2].Time, 2.0);
    }

    #[test]
    fn drag_clamps_negative_to_zero() {
        let mut w = build_wave(&[1.0]);
        drag_spawn_time(&mut w, 0, 0, -5.0);
        assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
    }

    #[test]
    fn drag_batch_shifts_from_index() {
        let mut w = build_wave(&[0.0, 1.0, 2.0, 3.0]);
        drag_spawn_time_batch(&mut w, 0, 1, 0.5);
        assert_eq!(w.Detail[0].Creeps[0].Time, 0.0);
        assert_eq!(w.Detail[0].Creeps[1].Time, 1.5);
        assert_eq!(w.Detail[0].Creeps[2].Time, 2.5);
        assert_eq!(w.Detail[0].Creeps[3].Time, 3.5);
    }
}
```

**Step 2: 在 `main.rs` 加 `mod wave_ops;`**

```bash
sed -i '/mod undo;/a mod wave_ops;' D:/omoba/map_editor/src/main.rs
```
或手動編輯。

**Step 3: 跑測試（應通過）**

```bash
cd D:/omoba/map_editor && cargo test wave_ops 2>&1 | tail -20
```
Expected: 3 passed

**Step 4: 在 `wave_timeline.rs` 接上 drag 邏輯**

在 spawn dot 繪製內：

```rust
                // 滑鼠按下且 hit dot → 開始 drag
                if input.mouse_pressed_left
                    && (mx - cx).powi(2) + (my - cy).powi(2) <= WAVE_DOT_R * WAVE_DOT_R
                    && app.wave_edit.drag.is_none()
                {
                    let shift = input.shift_held; // 確認 eui 欄位名
                    app.wave_edit.drag = Some(crate::app::SpawnDrag {
                        sel: (w_idx, di, si),
                        start_mouse_x: mx,
                        orig_time: spawn.Time,
                        batch_after: shift,
                    });
                    app.begin_edit(Some("wave_drag_time"));
                }
```

於 draw 函式末尾處理 drag move/up：

```rust
if let Some(drag) = app.wave_edit.drag {
    let new_time = drag.orig_time + (mx - drag.start_mouse_x) / px_per_sec;
    let (w, d, s) = drag.sel;
    if drag.batch_after {
        // 計算 delta：從 orig 到 new
        let delta = new_time - drag.orig_time;
        // 把 from=s 之後的所有 spawn 都從 orig 基準位移
        // 簡化：每幀重置成 orig + delta
        // 注意此處需要儲存 orig_times，不然多次 frame 累加會漂移
        // 為簡單起見，改成單次 drag = 該 spawn 之後同 detail 的所有 time 都 += delta（每幀重新基於 orig）
        // → 需要 SpawnDrag 加 orig_times: Vec<f32> 欄位（修正下方）
    } else {
        crate::wave_ops::drag_spawn_time(&mut app.map.CreepWave[w], d, s, new_time);
    }
    app.dirty = true;
    if !input.mouse_held_left {
        app.wave_edit.drag = None;
    }
}
```

**注意**：batch_after 模式為了避免每幀累加漂移，需要在 `SpawnDrag` 加 `orig_times: Vec<f32>` 欄位記錄 drag 開始時 from..end 的所有 Time。修正：

回到 `app.rs` 加：

```rust
pub struct SpawnDrag {
    pub sel: (usize, usize, usize),
    pub start_mouse_x: f32,
    pub orig_time: f32,
    pub batch_after: bool,
    pub orig_times: Vec<f32>,  // 從 sel.2 起的原始 times（batch 才用）
}
```

drag 開始時填：

```rust
let orig_times: Vec<f32> = app.map.CreepWave[w_idx].Detail[di].Creeps[si..]
    .iter().map(|c| c.Time).collect();
app.wave_edit.drag = Some(SpawnDrag { ..., orig_times });
```

drag 期間：

```rust
let delta = new_time - drag.orig_time;
if drag.batch_after {
    for (offset, ot) in drag.orig_times.iter().enumerate() {
        let target_idx = s + offset;
        if let Some(spawn) = app.map.CreepWave[w].Detail[d].Creeps.get_mut(target_idx) {
            spawn.Time = (ot + delta).max(0.0);
        }
    }
} else {
    crate::wave_ops::drag_spawn_time(&mut app.map.CreepWave[w], d, s, new_time);
}
```

**Step 5: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo build && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 拖曳 W01 第 5 個 dot → 跟著滑鼠移動
- 鬆開 → Time 固定，左欄列表 spawn 數量不變但 Time 已改
- Ctrl+Z → 還原到 drag 前
- Shift+拖曳 → 該 dot 與後面所有 dot 一起平移

**Step 6: Commit**

```bash
git add src/wave_ops.rs src/app.rs src/main.rs src/panels/wave_timeline.rs && \
  git commit -m "feat(map_editor): drag spawn dots to change Time (with Shift batch)"
```

---

### Task 13: Timeline — 右鍵選單（lane 空白插入 / spawn 操作）

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`

**Step 1: 在 hit-test 區段加右鍵偵測**

```rust
let mouse_right_clicked = input.mouse_pressed_right;  // 確認 eui API 名

// 在 spawn dot loop 內：
if mouse_right_clicked && hit_spawn == Some((w_idx, di, si)) {
    app.wave_edit.context_menu = Some(CtxMenu::Spawn {
        sel: (w_idx, di, si),
        screen_pos: (mx, my),
    });
}

// 在 lane loop 結束後（lane 空白右鍵）：
if mouse_right_clicked && hit_lane == Some((w_idx, di)) && hit_spawn.is_none() {
    let click_time = (mx - lane_origin_x + scroll_x) / px_per_sec;
    app.wave_edit.context_menu = Some(CtxMenu::Empty {
        wave: w_idx,
        detail: di,
        time: click_time.max(0.0),
        screen_pos: (mx, my),
    });
}
```

**Step 2: 在 draw 函式末尾畫 ctx menu**

```rust
if let Some(menu) = app.wave_edit.context_menu.clone() {
    let (sx, sy) = match &menu {
        CtxMenu::Empty { screen_pos, .. } => *screen_pos,
        CtxMenu::Spawn { screen_pos, .. } => *screen_pos,
    };
    let menu_w = 200.0_f32;
    let item_h = 28.0_f32;
    let items: Vec<String> = match &menu {
        CtxMenu::Empty { time, .. } => {
            // 列出所有 creep type 作為「在此插入 X」
            app.map.Creep.iter()
                .map(|c| format!("➕ 插入 {} @ {:.1}s", c.Name, time))
                .collect()
        }
        CtxMenu::Spawn { .. } => {
            vec!["🗑 刪除".into(), "📋 複製到 +1s".into(), "✏ 改 Creep …".into()]
        }
    };
    let menu_h = items.len() as f32 * item_h;
    let menu_rect = Rect::new(sx, sy, menu_w, menu_h);
    ui.paint_filled_rect(menu_rect, eui::rgba(0.18, 0.19, 0.21, 0.98), 4.0);

    let mut clicked: Option<usize> = None;
    for (i, label) in items.iter().enumerate() {
        let item_rect = Rect::new(sx, sy + i as f32 * item_h, menu_w, item_h);
        let hover = item_rect.contains(mx, my);
        if hover {
            ui.paint_filled_rect(item_rect, eui::rgba(0.30, 0.55, 0.85, 0.5), 0.0);
        }
        ui.ctx().paint_text(item_rect, label, FS_LABEL,
            eui::rgba(0.95, 0.95, 0.95, 1.0), TextAlign::Left);
        if hover && input.mouse_pressed_left {
            clicked = Some(i);
        }
    }

    // 點選 → 執行
    if let Some(i) = clicked {
        match menu {
            CtxMenu::Empty { wave, detail, time, .. } => {
                if let Some(creep) = app.map.Creep.get(i).map(|c| c.Name.clone()) {
                    app.begin_edit(None);
                    app.map.CreepWave[wave].Detail[detail].Creeps.push(
                        crate::schema::CreepsJD { Time: time, Creep: creep.clone() }
                    );
                    app.wave_edit.last_inserted_creep = Some(creep);
                    app.dirty = true;
                }
            }
            CtxMenu::Spawn { sel, .. } => {
                let (w, d, s) = sel;
                match i {
                    0 => { // 刪除
                        app.begin_edit(None);
                        app.map.CreepWave[w].Detail[d].Creeps.remove(s);
                        app.selection = Selection::WaveDetail(w, d);
                        app.dirty = true;
                    }
                    1 => { // 複製到 +1s
                        let mut copy = app.map.CreepWave[w].Detail[d].Creeps[s].clone();
                        copy.Time += 1.0;
                        app.begin_edit(None);
                        app.map.CreepWave[w].Detail[d].Creeps.insert(s + 1, copy);
                        app.dirty = true;
                    }
                    2 => { /* 改 Creep — 簡化為先標記，UI 之後做 */ }
                    _ => {}
                }
            }
        }
        app.wave_edit.context_menu = None;
    } else if input.mouse_pressed_left || input.key_escape {
        // 點外面或 ESC → 關閉
        app.wave_edit.context_menu = None;
    }
}
```

**Step 3: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 右鍵 lane 空白 → 浮現選單列出 td_basic / td_tough → 點 td_basic → 新 spawn dot 出現
- 右鍵 spawn → 選單顯示 刪除/複製 → 點刪除 → dot 消失
- 右鍵後點空白處 → 選單關閉

**Step 4: Commit**

```bash
git add src/panels/wave_timeline.rs && git commit -m "feat(map_editor): right-click context menu for insert/delete/duplicate spawn"
```

---

### Task 14: Timeline — Fixed zoom + Ctrl+滾輪縮放 + 水平 scroll

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`

**Step 1: 在標題列右側加 [Fit] [Fixed] 按鈕**

於 header 繪製後加：

```rust
let btn_w = 50.0;
let fit_rect = Rect::new(r.x + r.w - 2.0 * btn_w - 8.0, r.y + 2.0, btn_w, WAVE_HEADER_H - 4.0);
let fixed_rect = Rect::new(r.x + r.w - btn_w - 4.0, r.y + 2.0, btn_w, WAVE_HEADER_H - 4.0);
let active = eui::rgba(0.30, 0.55, 0.85, 1.0);
let inactive = eui::rgba(0.20, 0.22, 0.25, 1.0);
let (fit_c, fixed_c) = match app.wave_edit.zoom_mode {
    WaveZoom::Fit => (active, inactive),
    WaveZoom::Fixed(_) => (inactive, active),
};
ui.paint_filled_rect(fit_rect, fit_c, 4.0);
ui.paint_filled_rect(fixed_rect, fixed_c, 4.0);
ui.ctx().paint_text(fit_rect, "Fit", FS_CAPTION, eui::rgba(1.0,1.0,1.0,1.0), TextAlign::Center);
ui.ctx().paint_text(fixed_rect, "Fixed", FS_CAPTION, eui::rgba(1.0,1.0,1.0,1.0), TextAlign::Center);
if input.mouse_pressed_left && fit_rect.contains(mx, my) {
    app.wave_edit.zoom_mode = WaveZoom::Fit;
    app.wave_edit.scroll_x = 0.0;
}
if input.mouse_pressed_left && fixed_rect.contains(mx, my) {
    if matches!(app.wave_edit.zoom_mode, WaveZoom::Fit) {
        app.wave_edit.zoom_mode = WaveZoom::Fixed(WAVE_PX_PER_SEC_DEFAULT);
    }
}
```

**Step 2: 加 Ctrl+滾輪縮放**

```rust
if input.ctrl_held && input.scroll_y.abs() > 0.0 && r.contains(mx, my) {
    if let WaveZoom::Fixed(s) = app.wave_edit.zoom_mode {
        let factor = if input.scroll_y > 0.0 { 1.2 } else { 1.0 / 1.2 };
        let ns = (s * factor).clamp(WAVE_PX_PER_SEC_MIN, WAVE_PX_PER_SEC_MAX);
        app.wave_edit.zoom_mode = WaveZoom::Fixed(ns);
    } else {
        // 在 Fit 模式按 Ctrl+滾輪 → 自動切到 Fixed
        app.wave_edit.zoom_mode = WaveZoom::Fixed(WAVE_PX_PER_SEC_DEFAULT);
    }
}
```

**Step 3: 加水平 scroll（中鍵拖曳或 Shift+滾輪）**

最簡實作：Shift+滾輪 = 水平 scroll

```rust
if input.shift_held && input.scroll_y.abs() > 0.0 && r.contains(mx, my) {
    app.wave_edit.scroll_x = (app.wave_edit.scroll_x - input.scroll_y * 30.0).max(0.0);
}
```

**Step 4: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 點 Fixed → spawn 變密／變稀（依預設 50px/s 與該 wave 長度而定）
- Ctrl+滾輪向上 → 放大
- Shift+滾輪 → 水平卷動

**Step 5: Commit**

```bash
git add src/panels/wave_timeline.rs && git commit -m "feat(map_editor): Fit/Fixed zoom toggle + Ctrl+wheel zoom + Shift+wheel scroll"
```

---

## Phase P5：打磨（Add/Dup/Del wave、空狀態、校驗）

### Task 15: Wave List 加 Add / Duplicate / Delete 按鈕

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_list.rs`
- Modify: `D:/omoba/map_editor/src/wave_ops.rs` (加 helper + 測試)

**Step 1: 在 `wave_ops.rs` 加 helper 與測試**

```rust
use crate::schema::{CreepWaveData, CreepWaveJD, DetailJD};

/// 新增一個 wave，預設名稱 W{N+1:02}，自動含一個用 first_path 的 Detail
pub fn add_wave(map: &mut CreepWaveData) -> usize {
    let n = map.CreepWave.len();
    let name = format!("W{:02}", n + 1);
    let path = map.Path.first().map(|p| p.Name.clone()).unwrap_or_default();
    map.CreepWave.push(CreepWaveJD {
        Name: name,
        StartTime: 0.0,
        Detail: vec![DetailJD { Path: path, Creeps: vec![] }],
    });
    n // new index
}

/// 深拷貝指定 wave，名稱加 `_copy` 尾碼（碰撞遞增）
pub fn duplicate_wave(map: &mut CreepWaveData, idx: usize) -> Option<usize> {
    let src = map.CreepWave.get(idx)?.clone();
    let mut name = format!("{}_copy", src.Name);
    let mut k = 2;
    while map.CreepWave.iter().any(|w| w.Name == name) {
        name = format!("{}_copy{}", src.Name, k);
        k += 1;
    }
    let mut new = src;
    new.Name = name;
    let new_idx = map.CreepWave.len();
    map.CreepWave.push(new);
    Some(new_idx)
}

pub fn delete_wave(map: &mut CreepWaveData, idx: usize) -> bool {
    if idx < map.CreepWave.len() {
        map.CreepWave.remove(idx);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests_wave_crud {
    use super::*;
    use crate::schema::{PathJD};

    fn empty_map_with_path() -> CreepWaveData {
        let mut m = CreepWaveData::default();
        m.Path.push(PathJD { Name: "p0".into(), Points: vec![] });
        m
    }

    #[test]
    fn add_wave_uses_first_path_and_increments_name() {
        let mut m = empty_map_with_path();
        let i = add_wave(&mut m);
        assert_eq!(i, 0);
        assert_eq!(m.CreepWave[0].Name, "W01");
        assert_eq!(m.CreepWave[0].Detail[0].Path, "p0");

        add_wave(&mut m);
        assert_eq!(m.CreepWave[1].Name, "W02");
    }

    #[test]
    fn duplicate_appends_copy_with_collision_handling() {
        let mut m = empty_map_with_path();
        add_wave(&mut m);
        let i1 = duplicate_wave(&mut m, 0).unwrap();
        assert_eq!(m.CreepWave[i1].Name, "W01_copy");
        let i2 = duplicate_wave(&mut m, 0).unwrap();
        assert_eq!(m.CreepWave[i2].Name, "W01_copy2");
    }

    #[test]
    fn delete_removes_and_returns_true() {
        let mut m = empty_map_with_path();
        add_wave(&mut m);
        assert!(delete_wave(&mut m, 0));
        assert!(m.CreepWave.is_empty());
        assert!(!delete_wave(&mut m, 0));
    }
}
```

**Step 2: 跑測試**

```bash
cd D:/omoba/map_editor && cargo test wave_ops 2>&1 | tail -20
```
Expected: 6 passed

**Step 3: 在 `wave_list.rs` 加底部按鈕**

於 `for (i, w)` 迴圈外（panel 末尾）加：

```rust
            ui.spacer(12.0);
            if ui.button("+ Add Wave").primary().draw() {
                app.begin_edit(None);
                let new_idx = crate::wave_ops::add_wave(&mut app.map);
                app.wave_edit.selected_wave = Some(new_idx);
                app.selection = Selection::Wave(new_idx);
                app.dirty = true;
            }
            ui.spacer(2.0);
            if let Some(sel) = app.wave_edit.selected_wave {
                if ui.button("Duplicate").secondary().draw() {
                    app.begin_edit(None);
                    if let Some(new_idx) = crate::wave_ops::duplicate_wave(&mut app.map, sel) {
                        app.wave_edit.selected_wave = Some(new_idx);
                        app.selection = Selection::Wave(new_idx);
                        app.dirty = true;
                    }
                }
                ui.spacer(2.0);
                // 二次點擊確認
                let confirming = matches!(
                    app.wave_edit.pending_delete_wave,
                    Some((i, t)) if i == sel && t.elapsed().as_secs() < 5
                );
                let label = if confirming { "再點一次刪除" } else { "Delete" };
                let style = if confirming { eui::ButtonStyle::Primary } else { eui::ButtonStyle::Ghost };
                if ui.button(label).style(style).draw() {
                    if confirming {
                        app.begin_edit(None);
                        crate::wave_ops::delete_wave(&mut app.map, sel);
                        app.wave_edit.selected_wave = None;
                        app.selection = Selection::None;
                        app.wave_edit.pending_delete_wave = None;
                        app.dirty = true;
                    } else {
                        app.wave_edit.pending_delete_wave = Some((sel, std::time::Instant::now()));
                    }
                }
            }
```

**Step 4: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 點 + Add Wave → 列表多出 W06，自動選中
- 點 Duplicate → 多出 W06_copy
- 點 Delete → 變紅「再點一次刪除」→ 5 秒內再點 → 刪除；超過 5 秒 → 重置

**Step 5: Commit**

```bash
git add src/wave_ops.rs src/panels/wave_list.rs && \
  git commit -m "feat(map_editor): Add/Duplicate/Delete wave buttons with confirm-twice for delete"
```

---

### Task 16: Wave Inspector — Add/Delete Detail 與 Add/Delete Spawn 按鈕

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_inspector.rs`

**Step 1: 於 `draw_wave` 末尾加 `+ Detail` 按鈕**

```rust
    ui.spacer(8.0);
    if ui.button("+ Detail").secondary().draw() {
        app.begin_edit(None);
        let path = app.map.Path.first().map(|p| p.Name.clone()).unwrap_or_default();
        app.map.CreepWave[w].Detail.push(crate::schema::DetailJD {
            Path: path, Creeps: vec![],
        });
        app.dirty = true;
    }
```

**Step 2: 於 `draw_detail` 末尾加 `+ Spawn` / `Delete Detail`**

```rust
    ui.spacer(8.0);
    if ui.button("+ Spawn").secondary().draw() {
        let creep = app.wave_edit.last_inserted_creep.clone()
            .or_else(|| app.map.Creep.first().map(|c| c.Name.clone()))
            .unwrap_or_default();
        app.begin_edit(None);
        let next_t = app.map.CreepWave[w].Detail[d].Creeps
            .iter().map(|c| c.Time).fold(0.0_f32, f32::max) + 1.0;
        app.map.CreepWave[w].Detail[d].Creeps.push(
            crate::schema::CreepsJD { Time: next_t, Creep: creep.clone() }
        );
        app.wave_edit.last_inserted_creep = Some(creep);
        app.dirty = true;
    }
    ui.spacer(4.0);
    if ui.button("Delete Detail").ghost().draw() {
        app.begin_edit(None);
        app.map.CreepWave[w].Detail.remove(d);
        app.selection = Selection::Wave(w);
        app.dirty = true;
    }
```

**Step 3: 於 `draw_spawn` 末尾加 `Delete Spawn`**

```rust
    ui.spacer(8.0);
    if ui.button("Delete Spawn").ghost().draw() {
        app.begin_edit(None);
        app.map.CreepWave[w].Detail[d].Creeps.remove(s);
        app.selection = Selection::WaveDetail(w, d);
        app.dirty = true;
    }
```

**Step 4: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 選 W01 → 右側 + Detail → lane 多出第 2 條
- 選新 lane → + Spawn → dot 出現
- Delete Detail → lane 消失

**Step 5: Commit**

```bash
git add src/panels/wave_inspector.rs && \
  git commit -m "feat(map_editor): Inspector Add/Delete buttons for Detail and Spawn"
```

---

### Task 17: 邊界處理 — 空狀態提示與 path/creep 名校驗紅字

**Files:**
- Modify: `D:/omoba/map_editor/src/panels/wave_timeline.rs`

**Step 1: lane header 顯示「(not found)」紅字**

於 lane header 繪製處，先檢查 path 是否存在於 `map.Path`：

```rust
            let path_exists = app.map.Path.iter().any(|p| p.Name == detail.Path);
            let header_color = if path_exists {
                eui::rgba(0.85, 0.85, 0.85, 1.0)
            } else {
                eui::rgba(0.95, 0.40, 0.40, 1.0)
            };
            let header_text = if path_exists {
                detail.Path.clone()
            } else {
                format!("{} (not found)", detail.Path)
            };
            ui.ctx().paint_text(header_rect, &header_text, FS_LABEL,
                header_color, TextAlign::Left);
```

**Step 2: spawn dot 用 `?` 表示 creep_name 不在 `map.Creep`**

於 spawn dot 內：

```rust
                let creep_exists = app.map.Creep.iter().any(|c| c.Name == spawn.Creep);
                let letter = if creep_exists {
                    creep_letter(&spawn.Creep)
                } else {
                    "?".to_string()
                };
                let color = if creep_exists {
                    creep_color(&spawn.Creep)
                } else {
                    eui::rgba(0.5, 0.5, 0.5, 0.7)
                };
```

**Step 3: 空 wave list 提示**

於 `wave_list.rs` if `waves.is_empty()` 區塊改加 `+ Add Wave` 按鈕（已在 Task 15）。確認此情境正常。

**Step 4: `map.Path` 為空時 + Add Wave 行為**

於 `wave_list.rs` 的 + Add Wave 按鈕加保護（呼叫 `add_wave` 已會自動填空字串，配合 Step 1 的紅字提示，無需阻擋）。

**Step 5: 編譯 + 手動驗證**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```
驗證：
- 選 W01，把右側 Detail Path 改成 `xxx` → lane header 變紅「xxx (not found)」
- 把某 spawn 的 Creep 改成 `nope` → 圓變灰、字母變 ?

**Step 6: Commit**

```bash
git add src/panels/wave_timeline.rs && \
  git commit -m "feat(map_editor): red-text validation for missing path/creep references"
```

---

### Task 18: 最終驗證 + 文件更新

**Files:**
- Modify: `D:/omoba/CLAUDE.md` (若需新增說明)

**Step 1: 跑全測試**

```bash
cd D:/omoba/map_editor && cargo test 2>&1 | tail -30
```
Expected: 全綠

**Step 2: End-to-end 手動驗證 checklist**

```bash
cd D:/omoba/map_editor && cargo run -- D:/omoba/omb/Story/TD_1
```

逐項驗：
- [ ] 點 toolbar `Waves` → 進三欄模式
- [ ] 左欄選 W01~W05 → 中央 timeline 對應更新
- [ ] Fit / Fixed 切換 → 縮放正確
- [ ] Ctrl+滾輪 → 放大縮小（Fixed 模式下）
- [ ] Shift+滾輪 → 水平卷動
- [ ] 點 spawn → 選中黃描邊 + 右側 Inspector 顯示 Time/Creep
- [ ] 拖曳 spawn → Time 即時更新；鬆開 commit
- [ ] Ctrl+Z → 還原拖曳
- [ ] Ctrl+Y / Ctrl+Shift+Z → 重做
- [ ] 右鍵 spawn → 刪除 / 複製 +1s
- [ ] 右鍵 lane 空白 → 插入指定 creep
- [ ] + Add Wave → 自動取名 W06、選中
- [ ] Duplicate → W01_copy
- [ ] Delete → 二次確認
- [ ] + Detail / Delete Detail → lane 數量變化
- [ ] 改 Path 為不存在 → 紅字
- [ ] 改 Creep 為不存在 → 圓變灰 + ?
- [ ] Save → 開 `D:/omoba/omb/Story/TD_1/map.json` 確認 JSON 變更正確
- [ ] 切回 Map 模式 → 原本編輯器正常運作（regression check）

**Step 3: 更新 CLAUDE.md（可選）**

若項目目錄有需要記錄的 instruction，在 `D:/omoba/CLAUDE.md` 加：

```markdown
## map_editor

- Waves 模式編輯波次：toolbar 點 `Waves` 切換
- Wave/Detail/Spawn 三層 Selection 對應左欄、lane、spawn dot
```

**Step 4: 最終 commit + 推到 remote（若有）**

```bash
cd D:/omoba && git status
# 確認 map_editor 子模組（若是）或路徑變更，做最後一個整合 commit
git add -A
git commit -m "feat(map_editor): wave editor mode complete (P1-P5)"
```

---

## 執行注意事項

1. **eui API 假設**：plan 內多處假設 `input.mouse_pressed_left` / `input.scroll_y` / `input.shift_held` 等欄位名稱。實際執行時若 eui 不同，要 grep `D:/omoba/eui/src/` 找正確命名並調整。
2. **`paint_filled_rect` with corner radius 模擬圓**：若視覺不夠圓滑，後續可在 eui 加 `paint_filled_circle`。
3. **`begin_edit` 的時機**：每次 mutate 之前呼叫，並在 frame 結束時 `state.undo.end_group()`（已在 main.rs 處理）。
4. **canvas 模式 regression**：每個 task 後快速切回 Map 模式按一下，確認原功能不壞。
5. **字型限制**：UI 中文文字依靠 Windows 字型（msjh.ttc），按鈕內 emoji（🗑/➕）若不支援會顯示框框，可換純文字（"刪除"/"+"）。

## 已知技術風險

- 右鍵選單與 dropdown 並存的事件吃單可能要調試（同一 frame 內 mouse_pressed 被多處消費）
- timeline 大量 spawn（>200）時 `paint_filled_rect` 可能變慢，先做不優化
- Shift+drag batch 模式下 undo group tag 用 `"wave_drag_time"` 全 wave 共享，多次拖不同 spawn 之間 `end_group` 要正確切割（依靠 `mouse_up` 後下一幀 `key_redo`/`key_undo`/`mouse_pressed` 觸發 end_group）
