## Context

`bloons-style-right-sidebar-ui` 將 TD UI 改成左右面板後，需要大量圖片：右側買塔格、左側選中塔大圖、三路升級圖示、出售、Start/Pause、面板背景與卡片背景。第一版實作曾把 placeholder 放在 `omfx/data/td_ui/`，但這會把 content art 綁在前端專案，企劃無法從 scripts content mod 中直接維護。

本專案的塔、英雄、技能與升級行為由 `scripts/base_content` 提供，TD UI 圖片也應跟著 content mod 走。前端和主程式只負責讀取與顯示，不應把正式 content 圖片內建在前端。這組 TD UI 的美術主題是「甜點戰爭」：糖果、餅乾、蛋糕、奶油、巧克力、冰淇淋與玩具感武器組成可愛但有戰鬥感的塔防介面。

## Goals / Non-Goals

**Goals:**

- 將 TD UI 圖片權威來源放到 `scripts/base_content/assets/td_ui/`。
- 每個 UI 圖片用途都提供唯一檔名與預設 PNG，讓企劃知道「這個檔就是這個位置」。
- 所有預設圖與提示詞 SHALL 採「甜點戰爭」主題，避免回到一般猴子/軍事塔防素材。
- 提供 `README.md` 或 manifest，列出檔名、用途、建議尺寸與替換規則。
- `omfx` texture loader 優先讀 scripts asset 目錄，再視需要 fallback 到 staged/cwd/exe 旁路徑。
- 不讓 `omfx/data/td_ui/` 成為權威來源；若保留只能是 dev fallback 或舊資源相容。
- 缺圖時不 panic，但應 log 或以明確 fallback 顯示，方便診斷。

**Non-Goals:**

- 不把 PNG bytes 透過 `omb-script-abi` FFI 傳輸。
- 不修改 lockstep protocol 或 snapshot schema 來承載圖片內容。
- 不在這個 change 中設計最終商業美術，只提供可替換 placeholder。
- 不要求 release packaging 一次到位；但 dev run 必須能從 repo 讀到 scripts assets。

## Decisions

- 使用 `scripts/base_content/assets/td_ui/` 作為權威資源目錄。理由是 base_content 已是塔與升級內容的 mod，圖像命名能直接對應 `tower_*` 與 upgrade path。替代方案是 `omfx/data/td_ui/`，但會讓內容資源被前端專案擁有，不利企劃維護。
- 圖片檔名採用途唯一，不共用同一張 `default.png`。每個 slot 都要有自己的 placeholder，例如 `panel_left.png`、`shop_card_selected.png`、`tower_dart.png`、`tower_dart_p1.png`。理由是企劃看到檔名就能知道替換位置。替代方案是多個位置共用 fallback 圖，但會回到「不知道哪邊放圖」的問題。
- 使用一份文字 manifest `scripts/base_content/assets/td_ui/README.md` 作為人類可讀契約。第一版不新增二進位 manifest API，避免牽動 ABI。未來若需要工具化，可再新增 `td_ui_assets.toml`。
- `omfx` loader 的搜尋順序以 scripts 為優先：`scripts/base_content/assets/td_ui/<file>`、`../scripts/base_content/assets/td_ui/<file>`、`<repo>/scripts/base_content/assets/td_ui/<file>`、`<exe_dir>/scripts/base_content/assets/td_ui/<file>`，最後才允許舊的 `data/td_ui/<file>` fallback。
- placeholder PNG 要帶透明區域、可讀標籤與明確外框，不只是空白透明圖。理由是第一次啟動時就能看出圖掛在哪個 UI 位置、圖片範圍在哪裡，也能測 alpha。
- 實作上應清理或停止使用 `omfx/data/td_ui/` 生成的 placeholder，避免企劃替換錯目錄。

## Asset Filename Contract

第一版必須至少提供以下檔案：

- `panel_left.png`: 左側 selected tower panel 背景。
- `panel_right.png`: 右側 shop/control panel 背景。
- `shop_card.png`: 一般買塔格背景。
- `shop_card_selected.png`: 已選買塔格背景。
- `shop_card_locked.png`: 錢不夠或鎖定狀態買塔格背景。
- `tower_fallback.png`: 未知 tower kind 的 fallback 圖。
- `tower_dart.png`: Dart tower 買塔格與選中塔圖。
- `tower_bomb.png`: Bomb tower 買塔格與選中塔圖。
- `tower_tack.png`: Tack tower 買塔格與選中塔圖。
- `tower_ice.png`: Ice tower 買塔格與選中塔圖。
- `upgrade_p1.png`、`upgrade_p2.png`、`upgrade_p3.png`: 共用三路升級 fallback 圖。
- `tower_dart_p1.png`、`tower_dart_p2.png`、`tower_dart_p3.png`: Dart tower 專屬升級路線圖。
- `tower_bomb_p1.png`、`tower_bomb_p2.png`、`tower_bomb_p3.png`: Bomb tower 專屬升級路線圖。
- `tower_tack_p1.png`、`tower_tack_p2.png`、`tower_tack_p3.png`: Tack tower 專屬升級路線圖。
- `tower_ice_p1.png`、`tower_ice_p2.png`、`tower_ice_p3.png`: Ice tower 專屬升級路線圖。
- `sell.png`: 左側出售按鈕圖。
- `start_round.png`: 右側開始回合圖。
- `pause.png`: 右側暫停 placeholder 圖。

## Risks / Trade-offs

- [Risk] 前端從 scripts 讀檔會受 working directory 影響。→ Mitigation：loader 使用多候選路徑，並保留清楚 log。
- [Risk] scripts asset 不是 Rust crate 編譯輸出的一部分，release packaging 可能漏拷。→ Mitigation：dev 階段直接從 repo 讀；後續 packaging task 再明確 stage `scripts/base_content/assets`。
- [Risk] 每個 slot 都有 placeholder 會增加檔案數。→ Mitigation：這正是讓企劃可替換的成本，且 PNG 數量固定、體積小。
- [Risk] 舊 `omfx/data/td_ui` 與新 scripts 目錄同時存在會讓替換混亂。→ Mitigation：實作時移除或降級舊目錄，README 明確標示 scripts 目錄才是權威來源。

## Migration Plan

- 新增 `scripts/base_content/assets/td_ui/`，放入所有預設 PNG 與 README。
- 調整 `omfx` TD UI loader，使 scripts asset 目錄優先。
- 將目前前端 `omfx/data/td_ui/` 的 placeholder 移除或標示為非權威 fallback。
- 執行 `cargo check --manifest-path omfx/Cargo.toml` 驗證前端編譯。
- 手動跑 TD_1，確認每個 placeholder 都能顯示且替換檔案會生效。

## Open Questions

- release build 是否要把 `scripts/base_content/assets/td_ui/` stage 到 `omb/scripts/base_content_assets/` 或跟 exe 同層？第一版先支援 repo dev 路徑。
