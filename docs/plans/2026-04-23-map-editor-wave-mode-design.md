# Map Editor — Wave 編輯模式設計

日期：2026-04-23
範圍：`map_editor/` — 為 TD 模式（與既有 MOBA 模式共用）增加波次（CreepWave）的編輯能力。

## Context

`map_editor` 目前只能編輯地圖元素（CheckPoint、Tower、BlockedRegion、Structure 與 Hero/Enemy/Creep 模板），波次（`map.json` 的 `CreepWave`）只在底部 30~50px 的 `panels/waves.rs` 唯讀顯示一行 `[名稱 t=時間 x總數]`。

新增的 TD 模式（見 `~/.claude/plans/bloons-td6-image-squishy-flurry.md`）讓波次成為核心玩法資料：玩家按 Start Round 才放整波 creep。原作者編輯波次時必須手改 JSON，沒 schema 校驗、易打錯 path/creep 名。本設計目標是讓波次可以在編輯器內視覺化編輯。

`schema.rs` 的 `CreepWaveJD { Name, StartTime, Detail[] }` / `DetailJD { Path, Creeps[] }` / `CreepsJD { Time, Creep }` 已完備，無須改 schema 或後端。

## 設計決策

| 決策 | 結果 | 理由 |
|---|---|---|
| UI 形式 | 全螢幕 Wave 模式（新增 `ViewMode::Waves`） | 波次編輯資料量大，底部展開或塞 Inspector 都嫌擠 |
| 內部佈局 | 三欄式：左 Wave 列表｜中 Timeline｜右 Inspector | 平衡瀏覽、視覺、編輯三需求 |
| Timeline 互動 | 點擊選中 + 拖曳改 Time + 右鍵選單 | 常用操作直接在 timeline 完成，避免來回切右側表單 |
| 縮放 | Fit 與 Fixed(px/sec) 兩模式並存，Fixed 支援 Ctrl+滾輪 | 短波長波都好用 |
| Creep 視覺 | 圓點 + 第一個字母 + 顏色由 hash 映射 | 高密度也讀得出，hover 顯示完整名 |
| 多 Detail（多 path） | 每條 Detail 一個橫向 lane（垂直堆疊） | 可看跨 path 同步出怪，與 schema 一致 |
| 地圖預覽 | 不顯示 | YAGNI；要看路徑切回 Map 模式 |
| Undo | 沿用既有 `begin_edit(tag)` 機制 | 拖曳合併、CRUD 各次 push |

## 資料模型擴充

### `app.rs`

```rust
pub enum ViewMode {
    Map,
    Entities,
    Waves,    // ← 新增
}

pub enum Selection {
    // ... 既有 ...
    Wave(usize),                    // wave_idx
    WaveDetail(usize, usize),       // (wave_idx, detail_idx)
    WaveSpawn(usize, usize, usize), // (wave, detail, spawn)
}

pub struct WaveEditState {
    pub selected_wave: Option<usize>,
    pub zoom_mode: WaveZoom,
    pub scroll_x: f32,
    pub drag: Option<SpawnDrag>,
    pub context_menu: Option<CtxMenu>,
    pub last_inserted_creep: Option<String>,
    pub pending_delete_wave: Option<(usize, std::time::Instant)>, // 二次點擊確認
}

pub enum WaveZoom { Fit, Fixed(f32) }   // Fixed 預設 50.0 px/sec
pub struct SpawnDrag {
    pub sel: (usize, usize, usize),
    pub start_mouse_x: f32,
    pub orig_time: f32,
    pub batch_after: bool,    // Shift 按住 = 平移後續所有
}
pub enum CtxMenu {
    Empty { wave: usize, detail: usize, time: f32, screen_pos: (f32, f32) },
    Spawn { sel: (usize, usize, usize), screen_pos: (f32, f32) },
}
```

`AppState` 新增欄位：`wave_edit: WaveEditState`。

### Undo 策略

| 操作 | tag |
|---|---|
| Add / Duplicate / Delete wave | `None` |
| Add / Delete Detail | `None` |
| Add / Delete spawn | `None` |
| 拖曳 spawn Time（連續） | `Some("wave_drag_time")` |
| Inspector slider StartTime / Time | `Some("wave_starttime_<wave>")` / `Some("wave_spawn_time_<w>_<d>_<s>")` |
| Inspector 改 Creep / Path 下拉 | `None` |

每次 mutate 後 `app.dirty = true`。`apply_snapshot` 不必修改。

## 佈局與繪製

### Layout 分支（`main.rs`）

```rust
match state.view_mode {
    ViewMode::Waves => {
        // 跳過 templates/canvas/inspector/底部 waves 的擺位
        let toolbar_rect = Rect::new(content.x, content.y, content.w, TOOLBAR_H);
        let body_rect = Rect::new(content.x, content.y + TOOLBAR_H, content.w, content.h - TOOLBAR_H);
        panels::toolbar::draw(ui, toolbar_rect, &mut state);
        panels::waves::draw(ui, body_rect, &mut state);
    }
    _ => {
        // 維持現狀
    }
}
```

### Style 常數（`style.rs`）

```rust
pub const WAVE_LIST_W:    f32 = 150.0;
pub const WAVE_LANE_H:    f32 = 36.0;
pub const WAVE_HEADER_H:  f32 = 28.0;
pub const WAVE_RULER_H:   f32 = 18.0;
pub const WAVE_DOT_R:     f32 = 9.0;
pub const WAVE_PX_PER_SEC_DEFAULT: f32 = 50.0;
```

### 三欄分派（`panels/waves.rs`）

```rust
pub fn draw(ui: &mut UI, rect: Rect, app: &mut AppState) {
    let list_w = WAVE_LIST_W;
    let inspector_w = app.inspector_w;
    let list_rect = Rect::new(rect.x, rect.y, list_w, rect.h);
    let timeline_rect = Rect::new(rect.x + list_w, rect.y,
                                  rect.w - list_w - inspector_w, rect.h);
    let inspector_rect = Rect::new(rect.x + rect.w - inspector_w, rect.y, inspector_w, rect.h);
    wave_list::draw(ui, list_rect, app);
    wave_timeline::draw(ui, timeline_rect, app);
    wave_inspector::draw(ui, inspector_rect, app);
}
```

### Timeline 繪製順序（每幀）

1. 算 `viz_w = timeline_rect.w`；計算 `px_per_sec`：
   - `Fit`：`viz_w / wave_total_seconds`（wave_total = 該波最後一 spawn 的 Time + 0.5s buffer，最小 1s）
   - `Fixed(s)`：直接用 s
2. 畫頂部時間刻度（每秒一豎線 + 數字），>8 條才顯示半秒以避免雜亂
3. 對每條 `Detail`（lane）：
   - lane 背景條（zebra striping）
   - 對每 spawn：算 `cx = lane.x + spawn.Time * px_per_sec - scroll_x`
   - 畫圓（顏色 = `palette[hash(creep_name) % 8]`）
   - 圓內白色字母（creep_name 第一字符大寫）
   - 若 selected → 畫黃色描邊
4. Hover 顯示 tooltip：`td_basic @ 3.5s`
5. 拖曳中：被拖 spawn 跟 mouse_x，其他半透明

### Wave List（左欄）

每筆按鈕 `[W01  4.8s  x8]`（StartTime + spawn 總數）。底部三顆按鈕：
- `+ Add Wave`：建新 wave，自動加 1 個 Detail（用 `map.Path[0].Name`）
- `Duplicate`：deep copy + `_copy` 尾碼（碰撞遞增）
- `Delete`：第一次點變紅顯示「再點一次刪除」，5 秒重置

### Wave Inspector（右欄）

依 Selection 分支顯示對應表單（仿 `inspector.rs::draw_*` 結構）：

- `Wave(w)` → `Name` input、`StartTime` slider、`+ Detail` 按鈕
- `WaveDetail(w,d)` → `Path` 下拉（`map.Path` 名）、`+ 加 spawn` 按鈕、`刪除整 lane` 按鈕
- `WaveSpawn(w,d,s)` → `Time` slider、`Creep` 下拉（`map.Creep` 名）、`刪除` 按鈕

## 互動細節

### 點擊選中

- spawn 圓 → `Selection::WaveSpawn(w,d,s)`
- lane 空白 → `Selection::WaveDetail(w,d)`
- 頂部 wave 標題列 → `Selection::Wave(w)`
- 左欄 wave → `selected_wave = Some(w)`，且 `Selection::Wave(w)`

### 拖曳改 Time

- `mouse_down` 在 spawn 上 → 記 `SpawnDrag { sel, start_mouse_x, orig_time }`，`begin_edit(Some("wave_drag_time"))`
- `mouse_move` → `new_time = max(0, orig_time + (mx - start_mouse_x) / px_per_sec)`，即時寫回，set `dirty`
- `mouse_up` → 清空 drag；不排序（保留作者順序）
- Shift 按住拖曳 → `batch_after = true`，同 lane 後面 spawn 一起平移
- 出 lane 不取消，數值就讓它變大

### 右鍵選單

- 右鍵 spawn → `CtxMenu::Spawn`：`刪除` / `複製此 spawn 在 +1s 處` / `改 Creep ▼`
- 右鍵 lane 空白 → `CtxMenu::Empty { time }`：「在 X.XXs 插入 spawn ▼」
- 關閉條件：點外面、ESC、選中項
- 用 eui 既有 dropdown / popup 元件

### Zoom 控制

- 標題列右側兩顆按鈕：`[Fit]` / `[Fixed]`
- `Fixed` 時顯示底部水平 scrollbar
- `Ctrl+滾輪` 改 px_per_sec（×1.2 / ÷1.2，clamp [10, 500]）
- 切回 `Fit` 自動 `scroll_x = 0`

### 鍵盤快捷

- `Delete` → 刪選中 spawn / detail / wave
- `Ctrl+D` → Duplicate 選中 wave
- `Ctrl+Z` undo / `Ctrl+Y` 或 `Ctrl+Shift+Z` redo（eui 已內建，自動含括 wave 操作）

## 預設值與邊界情況

| 操作 | 預設行為 |
|---|---|
| `+ Add Wave` | `Name="W{N+1:02}"`、`StartTime=0.0`、自動加 1 Detail（用 `map.Path[0].Name` 或空字串） |
| `Duplicate Wave` | 深拷貝、`Name` 尾碼 `_copy` |
| `+ Detail` | `Path = map.Path[0].Name`、`Creeps = []` |
| 右鍵插入 spawn | `Time = 點擊處時間`、`Creep = last_inserted_creep` 或 `map.Creep[0].Name` |
| 拖曳 Time | 下限 0.0、無上限 |
| 空 wave list 的 `Delete` | 按鈕 disabled |

### 邊界情況

- `map.Path` 為空 → `+ Add Wave` 仍可建（path 用空字串），lane header 紅字「(無 path 對應)」
- `map.Creep` 為空 → 右鍵插入彈窗 disable，提示「先到 Map 模式新增 Creep 模板」
- Path / Creep 名打錯或被刪 → header / spawn 紅字「(not found)」，但保留 input 不破壞
- MOBA 模式相容 → schema 一致，編輯器不分模式

## 檔案總清單

| 檔案 | 動作 | 大致行數 |
|---|---|---|
| `map_editor/src/app.rs` | 加 `ViewMode::Waves`、3 個 `Selection::Wave*`、`WaveEditState` | +60 |
| `map_editor/src/main.rs` | layout 分支 | +20 |
| `map_editor/src/style.rs` | 加 `WAVE_*` 常數 | +6 |
| `map_editor/src/panels/toolbar.rs` | 加 `Waves` 按鈕 | +12 |
| `map_editor/src/panels/waves.rs` | 改寫：唯讀 → 三欄分派 | ~80 |
| `map_editor/src/panels/wave_list.rs` | 新增 | ~120 |
| `map_editor/src/panels/wave_timeline.rs` | 新增（繪製、拖曳、右鍵、zoom） | ~350 |
| `map_editor/src/panels/wave_inspector.rs` | 新增（依 selection 分支表單） | ~180 |
| `map_editor/src/panels/mod.rs` | 加 3 個 `pub mod` | +3 |

總計 +830 行。

## 分期落地

1. **P1 骨架**（半天）：`ViewMode::Waves` 切換 + 三欄空版面 + Wave 列表 + 點擊選 wave
2. **P2 顯示**（半天）：時間刻度 + lane + spawn 圓點繪製 + 顏色／字母 + Fit zoom
3. **P3 Inspector 編輯**（半天）：右欄三類 selection 的表單（純 click & form 編輯）
4. **P4 Timeline 互動**（1 天）：點擊選中、拖曳 Time、右鍵選單、Fixed zoom + 滾輪縮放 + 水平 scroll
5. **P5 打磨**（半天）：Add/Dup/Del wave、Shift-drag 批次、空狀態提示、Path/Creep 校驗紅字

每個 P 結束都能跑、能存檔、可以實際編一波看效果。

## 驗證

1. **編譯**：`cargo build` 於 `map_editor/`
2. **載地圖**：`cargo run -- D:/omoba/omb/Story/TD_1`
3. **切換**：toolbar 點 `Waves` → 整個中央切到三欄編輯
4. **編輯**：左欄選 W01 → 中央顯示 8 個 spawn 圓 → 點某個 → 右側 Inspector 出 Time/Creep 表單 → 改數值看 timeline 同步更新
5. **拖曳**：拖第 5 個 spawn → Time 即時變化 → 鬆開 → Ctrl+Z 還原
6. **新增**：右鍵 lane 空白 → 選 td_tough → 新 spawn 出現
7. **存檔**：Ctrl+S → 開 `TD_1/map.json` 確認 JSON 變更正確

## 已知風險

- **eui dropdown 在 popup 場景** — 右鍵選單裡放下拉，需驗證浮動位置 paint 與事件接收。fallback：右鍵直接顯示一張選單列表，不嵌套 dropdown
- **timeline hit test** — 多 lane 重疊到 splitter 邊界時 mouse_y 歸屬要對
- **大波次效能** — 1 波 200+ spawn 可能變慢；先做不優化，超過再 batch draw
