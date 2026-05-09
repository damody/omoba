## Context

TD 模式目前的 UI 在 `omfx/game/src/lib.rs` 以多個 `TextBuilder` 節點呈現，買塔、選中塔、`[SELL]` 與三路升級都排在右側。互動仍靠 `td_tower_button_rects`、`td_sell_button_rect`、`td_upgrade_button_rects`、`start_round_button_rect` 做手動 hit-test，並送出既有 `TowerPlace`、`TowerSell`、`TowerUpgrade`、`StartRound` lockstep input。

Bloons TD 6 的資訊分工更清楚：右側永遠是買塔與回合控制；左側只在選中塔時打開，放塔資訊、升級路線與出售。這次設計採同樣的操作模型，但使用本專案自己的素材與文字，不複製原作美術。

技能列已經有可用的圖片模式：用 `TextureResource::load_from_memory` 載入 PNG，設定 `CompressionOptions::NoCompression`，再用 `ImageBuilder` 顯示。TD 左右面板可以沿用這個模式支援透明 PNG 圖示、卡片底圖與控制按鈕。

## Goals / Non-Goals

**Goals:**

- 將 TD UI 改成 BTD 風格左右分欄：左側選中塔/升級/出售，右側買塔/開始/暫停。
- 右側買塔使用圖示格子或雙欄網格，顯示透明塔圖、價格、鎖定/可買狀態。
- 左側選中塔面板顯示大塔圖、塔名、三路升級卡、出售金額與目前 gold。
- 所有買塔、升級、出售、開始、暫停控制都能顯示透明 PNG；缺圖時仍有文字 fallback。
- 保留既有輸入行為、快捷鍵與 lockstep gameplay routing，只替換呈現層與 hit-test rect 的版面。
- 避免 per-frame 建立/刪除 UI node；延續 create-once、hide-offscreen、update-position/text/texture 模式。

**Non-Goals:**

- 不新增或修改 `proto/game.proto` 欄位。
- 不改變塔價格、退款公式、升級規則、路線等級上限或後端驗證。
- 不把 UI 系統全面搬到 `eui`，本次先在既有 Fyrox UI 節點中完成。
- 不使用或仿製 Bloons TD 6 的受版權保護素材，只借鑑「左右分欄操作模型」。

## Decisions

- 採左右兩個固定邊欄，而不是單一右側側欄。理由是買塔與升級/出售是兩種不同情境，分到左右能和 BTD 操作節奏一致，也避免選中塔時把買塔清單往下擠。替代方案是右側 accordion，但資訊仍混在同一側。
- 左側面板只在 `selected_tower_entity` 存在時顯示；未選塔時完全隱藏或只保留窄邊框。理由是左側面板是 context panel，玩家不選塔時不該佔用地圖視野。替代方案是永遠顯示空面板，但會浪費畫面。
- 右側面板永遠顯示買塔網格與底部 Start/Pause 控制。理由是買塔與開始回合是 TD 模式常駐操作，和 BTD 一樣固定在右側容易形成肌肉記憶。
- 採用 Fyrox UI `ImageBuilder` + `TextBuilder` 混合節點，而不是新增自製 renderer。理由是技能圖示已有成功模式，可直接支援 PNG alpha，實作風險低。
- 新增小型前端 asset loader/helper，集中處理 `omfx/data/td_ui/...`、`data/td_ui/...`、`../data/td_ui/...` 與 `exe_dir/data/td_ui/...` 的候選路徑。理由是避免把技能圖示載入邏輯複製多份，並讓 repo root、`omfx` 工作目錄與打包執行都能找到圖片。
- 用 `unit_id` 與固定用途命名圖檔，例如 `tower_dart.png`、`tower_dart_p1.png`、`sell.png`、`start_round.png`、`pause.png`、`panel_left.png`、`panel_right.png`。理由是現有 snapshot 已有 `tower_kind` 與 `td_template_order`，足夠解析前端資源。
- 建立面板與卡片 handle 結構取代裸 `Vec<Handle<Text>>` 的塔按鈕狀態，例如買塔格保留背景圖、塔圖、價格文字、快捷鍵文字與 rect；升級卡保留底圖、路線圖、名稱、等級、價格與 rect。
- 保留手動 hit-test rect，但 rect 對齊新的卡片 bounds。理由是目前輸入流程穩定，且點擊後仍走相同 lockstep input；UI 點擊優先序不應改變。
- 左側選中塔面板與升級卡使用 `selected_tower_entity`、`network_entities`、`td_templates`、`td_upgrade_defs`。理由是這些資料已由 snapshot-backed mirror 驅動，能保持與權威 sim state 一致。

## UI Layout Mockup

SVG 示意圖位於 `openspec/changes/bloons-style-right-sidebar-ui/ui-layout.svg`。它不是最終美術稿，而是 implementation 時的版面契約：左側選中塔升級/出售、右側買塔與回合控制，互動 rect 都以卡片 bounds 為準。

```text
1920x1080 viewport
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ top HUD: hearts/lives, gold, round, settings                                                                  │
├────────────── LEFT CONTEXT PANEL ───────────────┬──────────────────────── MAP / PATH ───────────────────────┬──────── RIGHT SHOP PANEL ────────┤
│ only visible when a tower is selected            │                                                            │ always visible                    │
│ ┌──────────────────────────────────────────────┐ │                                                            │ ┌──────────────────────────────┐ │
│ │ selected tower name + close                 │ │                                                            │ │ START / PAUSE / PLAY controls│ │
│ │ large transparent tower art                 │ │                                                            │ └──────────────────────────────┘ │
│ │ current levels / range / damage summary     │ │                                                            │ ┌──────────────────────────────┐ │
│ └──────────────────────────────────────────────┘ │                                                            │ │ tower shop title/category     │ │
│ ┌──────────────────────────────────────────────┐ │                                                            │ ├──────────────┬───────────────┤ │
│ │ Upgrade path 1 card: icon + name + $ cost   │ │                                                            │ │ tower icon    │ tower icon     │ │
│ └──────────────────────────────────────────────┘ │                                                            │ │ $200          │ $650           │ │
│ ┌──────────────────────────────────────────────┐ │                                                            │ ├──────────────┼───────────────┤ │
│ │ Upgrade path 2 card: icon + name + $ cost   │ │                                                            │ │ tower icon    │ tower icon     │ │
│ └──────────────────────────────────────────────┘ │                                                            │ │ $400          │ $400           │ │
│ ┌──────────────────────────────────────────────┐ │                                                            │ └──────────────┴───────────────┘ │
│ │ Upgrade path 3 card: icon + name + $ cost   │ │                                                            │ bottom: large start/pause button  │
│ └──────────────────────────────────────────────┘ │                                                            │ and optional speed/pause icons     │
│ bottom: current gold + large SELL button        │                                                            │                                    │
└─────────────────────────────────────────────────┴────────────────────────────────────────────────────────────┴────────────────────────────────────┘
```

版面規則：

- 左側選中塔面板寬度目標 300-340 px；未選塔時整組 hidden，`td_sell_button_rect` 與 `td_upgrade_button_rects` 移到螢幕外。
- 右側買塔面板寬度目標 260-320 px；買塔格子用 2 欄網格，格子約 104-128 px 寬、120-150 px 高。
- Start/Pause/Play 控制固定在右側，優先放右上或右下，不能放到左側選中塔面板。
- 左側升級三路是垂直堆疊的大卡片，右側買塔是網格小卡片；兩者視覺語意要明顯不同。
- 出售是左側底部的大型橘/紅色按鈕，顯示退款金額；目前 gold 可放在出售按鈕左側或上方。
- 所有圖示都要支援 alpha，透明區域直接露出面板背景或卡片背景。
- Hit-test rect 以每張卡片背景 bounds 為準，不能只包文字或 icon。

## Risks / Trade-offs

- [Risk] 左右兩側都佔寬度，地圖可視區變窄。→ Mitigation：左側只在選中塔時顯示；右側寬度在小視窗降級，並保留地圖中心操作區。
- [Risk] 多個圖文節點可能增加 draw calls 與 UI update 成本。→ Mitigation：只在初始化或資料數量變化時建節點；每幀只更新 position、必要 text 與少量 texture，避免 stress 場景中 create/remove。
- [Risk] PNG 圖片缺漏會讓 UI 變空白。→ Mitigation：所有圖片節點都必須允許 `None` texture 或 fallback placeholder，文字資訊仍完整可用。
- [Risk] 新卡片點擊區若沒有對齊視覺，玩家會覺得按鈕失準。→ Mitigation：hit-test rect 以卡片背景位置與尺寸為唯一來源，文字與圖片只跟隨該 rect 定位。
- [Risk] 類 Bloons TD 6 方向容易過度複製特定商業美術。→ Mitigation：只借鑑「左右分欄、塔商店網格、選中塔升級面板、回合控制位置」的 UX 模式，不複製原作素材或商標元素。

## Migration Plan

- 先新增 TD UI asset loader 與 placeholder 圖片命名規則，不改 gameplay data。
- 建立右側常駐 shop/control 面板，遷移買塔按鈕與 Start Round/Pause UI 的定位。
- 建立左側 selected tower 面板，遷移出售與三路升級 UI 的定位。
- 保留舊文字內容作為 fallback，圖片或背景不存在時仍顯示可操作文字。
- 驗證 `run.bat` 下 TD_1 可買塔、選塔、賣塔、升級與 start round；若新 UI 出問題，可回退到既有純文字節點邏輯。

## Open Questions

- 最終美術圖檔是否要由專案自帶 placeholder 先上，或等實作後再替換成正式素材？
- 右側買塔格子要固定 2 欄，還是依視窗高度切成 1 欄/2 欄？本設計以 2 欄為主，小視窗可退化成 1 欄。
- 暫停/播放目前若只有前端 UI 還沒有 lockstep action，第一版是否先做視覺 placeholder 與 disabled 狀態？
