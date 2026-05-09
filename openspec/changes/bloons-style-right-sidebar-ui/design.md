## Context

TD 模式目前的 UI 在 `omfx/game/src/lib.rs` 以多個 `TextBuilder` 節點呈現，買塔、選中塔、`[SELL]` 與三路升級都排在右側。互動仍靠 `td_tower_button_rects`、`td_sell_button_rect`、`td_upgrade_button_rects`、`start_round_button_rect` 做手動 hit-test，並送出既有 `TowerPlace`、`TowerSell`、`TowerUpgrade`、`StartRound` lockstep input。

Bloons TD 6 的資訊分工更清楚：右側永遠是買塔與回合控制；選中塔時打開獨立 context panel，放塔資訊、升級路線與出售。這次設計採同樣的操作模型，但使用本專案自己的素材與文字，不複製原作美術；context panel 需要依選中塔在畫面中的位置自動換邊，避免遮住塔本體。

技能列已經有可用的圖片模式：用 `TextureResource::load_from_memory` 載入 PNG，設定 `CompressionOptions::NoCompression`，再用 `ImageBuilder` 顯示。TD 左右面板可以沿用這個模式支援透明 PNG 圖示、卡片底圖與控制按鈕。

## Goals / Non-Goals

**Goals:**

- 將 TD UI 改成 BTD 風格分區：右側固定買塔/開始/暫停，選中塔/升級/出售使用可依塔位置自動換邊的 context panel。
- 實作版面必須對齊 `ui-layout.svg` 的 `1920x1080` primary reference bounds；圖片素材可替換，但面板、卡片與互動區的位置語意不可漂移。
- 右側買塔使用圖示格子或雙欄網格，顯示透明塔圖、價格、鎖定/可買狀態；shop 區必須是可捲動 viewport，內容容量至少 12 個塔卡，避免只能放 4 個塔卡。
- 選中塔面板顯示大塔圖、塔名、三路升級卡、出售金額與目前 gold；塔在畫面左半邊時，面板顯示在右側商店欄左緣。
- 所有買塔、升級、出售、開始、暫停控制都能顯示透明 PNG；缺圖時仍有文字 fallback。
- 保留既有輸入行為、快捷鍵與 lockstep gameplay routing，只替換呈現層與 hit-test rect 的版面。
- 避免 per-frame 建立/刪除 UI node；延續 create-once、hide-offscreen、update-position/text/texture 模式。

**Non-Goals:**

- 不新增或修改 `proto/game.proto` 欄位。
- 不改變塔價格、退款公式、升級規則、路線等級上限或後端驗證。
- 不把 UI 系統全面搬到 `eui`，本次先在既有 Fyrox UI 節點中完成。
- 不使用或仿製 Bloons TD 6 的受版權保護素材，只借鑑「左右分欄操作模型」。

## Decisions

- 採固定右側買塔欄 + 選中塔 context panel，而不是把所有資訊塞進單一右側 accordion。理由是買塔與升級/出售是兩種不同情境，分離能和 BTD 操作節奏一致，也避免選中塔時把買塔清單往下擠。
- 選中塔面板只在 `selected_tower_entity` 存在時顯示；未選塔時完全隱藏或只保留窄邊框。面板錨點依選中塔的 screen-space x 決定：塔在畫面左半邊時，面板貼在右側 shop/control panel 左緣；塔在畫面右半邊時，面板使用左側錨點。理由是 context panel 不該遮住玩家剛點選的塔、射程圈或升級回饋。替代方案是永遠顯示左側空面板，但會在左半邊塔場景擋住塔。
- 右側面板永遠顯示買塔網格與底部 Start/Pause 控制；買塔網格在中段 viewport 內垂直捲動，Start/Pause 固定在底部不跟著捲。理由是買塔與開始回合是 TD 模式常駐操作，和 BTD 一樣固定在右側容易形成肌肉記憶，同時 12 種以上塔不應擠壓底部控制。
- 採用 Fyrox UI `ImageBuilder` + `TextBuilder` 混合節點，而不是新增自製 renderer。理由是技能圖示已有成功模式，可直接支援 PNG alpha，實作風險低。
- 新增小型前端 asset loader/helper，集中處理 `omfx/data/td_ui/...`、`data/td_ui/...`、`../data/td_ui/...` 與 `exe_dir/data/td_ui/...` 的候選路徑。理由是避免把技能圖示載入邏輯複製多份，並讓 repo root、`omfx` 工作目錄與打包執行都能找到圖片。
- 用 `unit_id` 與固定用途命名圖檔，例如 `tower_dart.png`、`tower_dart_p1.png`、`sell.png`、`start_round.png`、`pause.png`、`panel_left.png`、`panel_right.png`。理由是現有 snapshot 已有 `tower_kind` 與 `td_template_order`，足夠解析前端資源。
- 建立面板與卡片 handle 結構取代裸 `Vec<Handle<Text>>` 的塔按鈕狀態，例如買塔格保留背景圖、塔圖、價格文字、content-space rect 與 viewport-clipped hit-test rect；快捷鍵/名稱可保留於資料或 tooltip，但不得疊在卡片內；升級卡保留底圖、路線圖、名稱、等級、價格與 rect。
- 保留手動 hit-test rect，但 rect 對齊新的卡片 bounds。理由是目前輸入流程穩定，且點擊後仍走相同 lockstep input；UI 點擊優先序不應改變。
- 選中塔 context panel 與升級卡使用 `selected_tower_entity`、`network_entities`、`td_templates`、`td_upgrade_defs`。理由是這些資料已由 snapshot-backed mirror 驅動，能保持與權威 sim state 一致。
- `ui-layout.svg` 是 layout contract，不只是視覺參考。理由是目前截圖已出現升級卡退化成小 icon/價格、右側控制與商店區沒有依 SVG 對齊的問題；實作必須以 SVG 的面板、卡片、scrollbar、viewport 與按鈕 bounds 作為 hit-test 與定位來源，再按 `1920x1080` reference 比例縮放。

## UI Layout Mockup

SVG 示意圖位於 `openspec/changes/bloons-style-right-sidebar-ui/ui-layout.svg`。它不是最終美術稿，而是 implementation 時的版面契約：可換邊的選中塔升級/出售 context panel、右側買塔與回合控制，互動 rect 都以卡片 bounds 為準。

參考座標使用 SVG 的 `1920x1080 viewBox`；這是主要設計與驗收目標。實作在其他 16:9 視窗時 SHALL 由 `1920x1080` 等比例縮放主要 bounds。非 16:9 視窗可保留高度優先或寬度優先的安全縮放，但仍 SHALL 維持 context panel、右側 shop/control panel、中央地圖、卡片順序與控制位置的相對關係。

```text
1920x1080 reference viewport
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ top HUD centered above map                                                                                     │
├──── CONTEXT PANEL LEFT ANCHOR ────┬──────────────────────────── MAP / PATH ───────────────────────┬──── RIGHT SHOP/CONTROL PANEL ────┤
│ x=24 y=45 w=426 h=990             │                                                               │ x=1479 y=0 w=405 h=1080          │
│ right anchor: 1053,45 426x990     │                                                               │ buy button/title: 1506,33 351x96 │
│ selected tower card: 63,158 348x255│                                                              │ shop title: 1503,124 357x40      │
│ path cards, left-anchor reference:│                                                               │ shop viewport: 1500,170 360x745  │
│   P1 57,480 357x117               │                                                               │   2 columns, 12-card content     │
│   P2 57,615 357x117               │                                                               │   right scrollbar inside panel   │
│   P3 57,750 357x117               │                                                               │   cards clipped to viewport      │
│ gold/refund: 57,915 177x78        │                                                               │ start/pause: bottom row          │
│ sell: 255,900 159x99              │                                                               │   1508,938 162x111               │
│                                   │                                                               │   1692,938 162x111               │
└─────────────────────────────────────────────────┴────────────────────────────────────────────────────────────┴────────────────────────────────────┘
```

版面規則：

- `1920x1080` 是 primary target；實作應先計算 reference-to-window scale，再以同一份 reference bounds 更新視覺節點與 hit-test rect。
- 選中塔 context panel 有兩個錨點：左側錨點是 `x=24 y=45 w=426 h=990`；右側錨點是 `x=1053 y=45 w=426 h=990`，右緣貼齊 shop/control panel 左緣 `x=1479`。未選塔時整組 hidden，`td_sell_button_rect` 與 `td_upgrade_button_rects` 移到螢幕外。
- 當選中塔 screen-space x 在 1920 reference 下小於 `960` 時使用右側錨點；大於或等於 `960` 時使用左側錨點。非 1920 視窗應用相同的縮放後半寬判斷。
- 塔資訊卡在左錨點 reference 中是 `x=63 y=158 w=348 h=255`；塔圖、塔名與資訊文字都必須在此卡片內，不應浮在面板外。使用右錨點時，panel 內部元素的 x 位置加上 `1029` 的 anchor delta。
- 三路升級是 context panel 內三張橫向大卡，左錨點 reference bounds 分別是 `57,480 357x117`、`57,615 357x117`、`57,750 357x117`；使用右錨點時 x 位置同樣加上 `1029`。不能只顯示小 upgrade icon 加漂浮價格。
- 出售區固定在 context panel 底部，退款/目前 gold 使用左錨點 reference `57,915 177x78` 區塊，SELL 使用 `255,900 159x99` 大按鈕；使用右錨點時 x 位置加上 `1029`。
- 右側 shop/control panel 在 1920x1080 參考圖中是 `x=1479 y=0 w=405 h=1080`，應貼近右緣並覆蓋全高。
- 右側買塔格子用 2 欄可捲動網格。shop viewport 參考 bounds 是 `1500,170 360x745`，上方只保留 title / buy header，下方保留固定 Start/Pause；viewport 右側顯示 scrollbar track 與 thumb。內容高度 SHALL 依塔數計算，至少能容納 12 個塔卡（2 欄 x 6 列）而不丟失節點；超過可見高度時以 scroll offset 顯示。買塔卡參考尺寸以 `158x160` 為主，兩欄與列間距應緊貼；卡內只顯示大塔圖與底部價格，不在卡上疊名稱、快捷鍵文字或 `SHOP CARD` / `SELECT` 等素材內嵌英文；價格放在卡片原本的底部文字區。Start 按鈕只顯示圖示，不額外疊 `開始 1/5` 文字。
- 買塔卡片在 content-space 中排列，hit-test 必須先套用 scroll offset 並裁切到 viewport；被捲出 viewport 的卡片不得可點擊。小視窗才可降級 1 欄，但仍必須保留至少 12 張卡的可捲動內容容量。
- Start/Pause/Play 控制固定在右側底部，參考 bounds 是 `1508,938 162x111` 與 `1692,938 162x111`；不能放到 context panel，也不能壓在買塔格或地圖底部技能列上。
- Scrollbar 只控制右側買塔 viewport，Start/Pause/Play、金錢/生命 HUD 與選中塔 context panel 都不得跟著右側 shop scroll 位移。
- Context panel 升級三路是垂直堆疊的大卡片，右側買塔是網格小卡片；兩者視覺語意要明顯不同。
- 出售是 context panel 底部的大型橘/紅色按鈕，顯示退款金額；目前 gold 可放在出售按鈕左側或上方。
- 所有圖示都要支援 alpha，透明區域直接露出面板背景或卡片背景。
- Hit-test rect 以每張卡片背景 bounds 為準，不能只包文字或 icon。

## Risks / Trade-offs

- [Risk] 右側 shop 與可換邊 context panel 都佔寬度，地圖可視區變窄。→ Mitigation：context panel 只在選中塔時顯示，且依選中塔所在半邊換到相反側；右側寬度在小視窗降級，並保留地圖中心操作區。
- [Risk] 多個圖文節點可能增加 draw calls 與 UI update 成本。→ Mitigation：只在初始化或資料數量變化時建節點；每幀只更新 position、必要 text 與少量 texture，避免 stress 場景中 create/remove。
- [Risk] scroll offset 與 hit-test rect 不一致會造成捲出畫面的塔仍可點。→ Mitigation：買塔卡 hit-test rect 一律由 content-space bounds 減去 scroll offset 後與 viewport 做 intersection；不可見卡片 rect 設為螢幕外或 0 尺寸。
- [Risk] PNG 圖片缺漏會讓 UI 變空白。→ Mitigation：所有圖片節點都必須允許 `None` texture 或 fallback placeholder，文字資訊仍完整可用。
- [Risk] 新卡片點擊區若沒有對齊視覺，玩家會覺得按鈕失準。→ Mitigation：hit-test rect 以卡片背景位置與尺寸為唯一來源，文字與圖片只跟隨該 rect 定位。
- [Risk] 類 Bloons TD 6 方向容易過度複製特定商業美術。→ Mitigation：只借鑑「左右分欄、塔商店網格、選中塔升級面板、回合控制位置」的 UX 模式，不複製原作素材或商標元素。

## Migration Plan

- 先新增 TD UI asset loader 與 placeholder 圖片命名規則，不改 gameplay data。
- 建立右側常駐 shop/control 面板，遷移買塔按鈕與 Start Round/Pause UI 的定位。
- 將右側買塔區改成可捲動 viewport，支援至少 12 張塔卡的內容容量與 scrollbar。
- 建立可換邊 selected tower context panel，遷移出售與三路升級 UI 的定位。
- 保留舊文字內容作為 fallback，圖片或背景不存在時仍顯示可操作文字。
- 驗證 `run.bat` 下 TD_1 可買塔、選塔、賣塔、升級與 start round；若新 UI 出問題，可回退到既有純文字節點邏輯。

## Open Questions

- 最終美術圖檔是否要由專案自帶 placeholder 先上，或等實作後再替換成正式素材？
- 右側買塔格子要固定 2 欄，還是依視窗高度切成 1 欄/2 欄？本設計以 2 欄為主，小視窗可退化成 1 欄，但內容容量仍至少 12 張並以 scrollbar 捲動。
- 暫停/播放目前若只有前端 UI 還沒有 lockstep action，第一版是否先做視覺 placeholder 與 disabled 狀態？
