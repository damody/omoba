# omfue 與 omfx 完整功能同步設計

日期：2026-07-12

## 背景

`omfue` 的玩家功能停留在約 2026 年 5 月的單機 TD、基礎 HUD、RTS 操作與 bridge 整合；`omfx` 在 6–7 月新增了完整賽前流程、九張地圖與預覽、難度篩選、遊戲速度、自動開波、BTD6 風格快捷鍵、音效、解析度與新版 Settings。`run_ue.bat` 目前又預設以 `TD_1` 直接啟動，未進入最新版賽前流程。

本變更要讓 `omfue` 在功能、操作流程、版面與視覺上盡量一比一對齊目前的 `omfx`，並建立可重複的共享設定與生成流程，避免兩個前端日後各自維護相同資料。

## 目標

- `run_ue.bat` 預設進入完整賽前主選單，而不是直接啟動 `TD_1`。
- UE 前端完整支援 `omfx` 現有的玩家可見功能：九圖選擇與預覽、難度、Settings、快捷鍵重綁、音效、解析度、遊戲速度、自動開波及新版遊戲 HUD。
- UE 的主要畫面布局與視覺盡量一比一對齊 `omfx`。
- 將可共享的設定放在前端之外，並擴充 `omoba-core`、`omoba-sim`、`omoba-template-ids` 與 `scripts`；不新增 crate 或頂層模組。
- 讓 `omfx` 與 `omfue` 從相同來源取得 catalog、穩定 ID、預設快捷鍵、設定 schema、布局 token 與 session config。
- 保留單機、networked、smoke test 及直接啟動指定場次的除錯能力。

## 非目標

- 不把 Fyrox widget 程式直接轉譯或嵌入 Unreal。
- 不新增第五個共享 crate 或新的頂層模組。
- 不強迫兩個引擎共享 widget hierarchy、材質實作或引擎特有的畫質選項。
- 不在本次工作中重構與功能同步無關的 backend 或遊戲內容。

## 架構與責任邊界

### `omoba-core`

擴充現有模組，承載不依賴 Fyrox 或 Unreal 的前端共用契約：

- 賽前狀態機及 action/state 型別。
- 設定 schema、schema version、migration 與預設值。
- 快捷鍵 action、預設綁定、序列化格式與衝突驗證。
- 地圖、難度、Settings 與 UI layout/theme manifest 的共用資料型別。
- 共用資料的解析、驗證與 round-trip API。

`omfx` 與 `omfue/bridge` 直接依賴同一套 API，不各自複製狀態轉移或設定解析邏輯。

### `omoba-sim`

擴充現有模組，承載會影響模擬結果的共用規則：

- 難度資料到 session/simulation config 的轉換。
- 遊戲速度允許值與切換規則。
- 自動開波判斷與每個 idle round 只送出一次的規則。
- 地圖、story 與難度組合的有效性驗證。

兩個前端不得各自重新解讀這些模擬參數。

### `omoba-template-ids`

擴充既有穩定 ID 定義與生成輸出，涵蓋：

- 地圖、story 與難度。
- 塔、英雄、技能及 buff。
- 快捷鍵 action 與共用 Settings key。
- UI screen、widget role、音效 cue 與資產 key。

generator 與兩個前端都使用相同 ID。未知 ID、重複 ID 或失效 reference 必須在生成或測試階段失敗。

### `scripts`

`scripts` 是玩家可調內容與視覺設定的主要來源，擴充現有 assets 與生成流程，承載：

- `scripts/base_content/assets/pregame_ui/catalog.json` 中的九圖、難度、預覽圖及 story mapping。
- 共用 UI theme/layout tokens，包括 1920×1080 reference rectangles、顏色、字體 token、文字、圖片與 responsive 規則。
- 音樂、UI、放塔及其他音效 cue 對應。
- 預設快捷鍵與 Settings 選項清單。
- 共享資料版本與 freshness metadata。

生成流程使用 `omoba-template-ids` 檢查穩定 ID，使用 `omoba-core` 檢查 schema 與狀態資料，並使用 `omoba-sim` 檢查模擬設定。它輸出或 staging：

- `omfx` 可載入的共享資料。
- `omfue/bridge` 可載入的 manifest。
- UE 需要的 generated header、圖片、音效與 cook/staging manifest。
- 輸入檔 hash、schema version 與生成器版本，供 freshness check 使用。

### 前端專屬責任

`omfx` 只保留 Fyrox widget、音訊與視窗 API 等引擎實作。`omfue` 只保留 UMG、Blueprint/C++、UE 音訊、解析度與視窗 API 等引擎實作。

兩者共享資料、布局規格與行為契約，但不共享引擎專屬 widget hierarchy。無法合理共用的 RHI、UE-only 畫質或 Fyrox-only 選項，放在設定 schema 的 frontend-specific 區段。

## 建置與生成資料流

1. `scripts` generator 讀取 catalog、theme/layout、音效、快捷鍵及 Settings 外部設定。
2. `omoba-template-ids` 驗證穩定 ID 與 reference。
3. `omoba-core` 驗證 schema、預設值和狀態資料。
4. `omoba-sim` 驗證難度、story、速度與自動開波參數。
5. generator 原子地更新兩前端的生成輸出及 freshness metadata。
6. `run.bat` 與 `run_ue.bat` 使用相同 metadata 判斷是否需要重新生成或 staging。

生成輸出是衍生物，不是第二份手工資料來源。任何會同時影響兩個前端的變更，必須先修改外部來源或上述四個既有共用模組，再重新生成。

## 執行資料流

1. `run_ue.bat` 預設啟動 menu runtime，不設定固定 `OMB_STORY=TD_1` 作為正常入口。
2. UE 從 bridge 取得共享設定描述的主選單、Settings、難度與地圖狀態。
3. 玩家操作由 UE 轉為 `omoba-core` 定義的 action。
4. `omfue/bridge` 執行共用狀態轉移並回傳權威 UI state；UE widget 不自行維護另一套賽前狀態機。
5. 玩家選定地圖與難度後，bridge 使用 `omoba-sim` 產生共用 session config。
6. 單機模式用該 config 啟動本機模擬；networked 模式把相同選擇交給 backend。
7. 遊戲速度、自動開波、快捷鍵及返回選單也走相同 action/state contract。

## 設定持久化

- `omfx` 與 `omfue` 預設讀寫同一份使用者設定檔。
- 共用設定具有明確 schema version，包含音量、解析度、視窗模式、快捷鍵及其他共用 Settings。
- frontend-specific 區段保存無法跨引擎共用的選項。
- 首次載入新版設定時，相容匯入舊版 `omfx` 的 `data/hotkeys.json`；匯入成功後按新版 schema 儲存。
- 單一欄位損壞時只回復該欄位預設值並記錄 diagnostics；整份檔案無法解析時備份損壞檔，再建立預設設定。
- 寫入使用 temporary file 加 atomic replace，避免中途終止造成設定損壞。

## UI 與玩家功能

UE 以 `omfx` 的 1920×1080 reference layout 重建畫面，透過 anchors、scale box 與 safe-zone 規則支援其他解析度。位置、尺寸、顏色、字體 token、圖片、文字和音效映射來自外部 theme/layout 設定。

完整同步範圍：

- 主選單與最新版 Settings。
- 難度選擇、依難度篩選的九張地圖、地圖預覽與開始遊戲流程。
- 最新遊戲 HUD、塔商店、選中塔面板、三路升級、sell 與 target priority 操作。
- BTD6 風格快捷鍵與 F1 重綁面板。
- 遊戲速度切換與自動開波。
- BGM、按鈕、放塔及 `omfx` 已有的其他玩家音效。
- 解析度、視窗／全螢幕模式與音量設定。
- 暫停、返回主選單與場次結束流程。
- 現有 hero、ability、buff、placement preview、entity overlay、RTS camera 與 diagnostics。

圖片與音效優先直接共享 `scripts` 下的來源，由 generator staging 成兩個引擎需要的格式。UE widget hierarchy、材質、動畫和 hover transition 維持 UE 原生實作，但必須以 `omfx` reference screenshot 做逐畫面比較。

## `run_ue.bat` 行為

- 預設啟動完整賽前選單。
- 不再以固定 `TD_1` 取代正常賽前選擇。
- build/freshness 流程涵蓋共享設定生成、bridge、DLL、UE 資產 staging 與 cook freshness。
- 保留 `--single-player`、`--networked`、`--editor`、build 與 smoke modes。
- 增加顯式 direct-session 除錯入口，可指定 story 與難度；只有使用該參數時才跳過賽前選單。
- 所有必要生成物在啟動前都需存在且版本一致，否則先重建；重建失敗即停止。
- `.bat` 持續使用 CRLF 與 UTF-8 no BOM，並檢查缺少參數值、未知參數及依賴工具。

## 錯誤處理

- generator 遇到重複 ID、未知 story、缺少必要圖片或音效、無效快捷鍵、非法難度或 layout schema 錯誤時直接失敗，訊息包含來源檔與欄位。
- 關鍵 catalog、manifest 或 schema version 不一致時禁止使用過期資料啟動。
- 非關鍵視覺資產缺失時使用明確 placeholder，並在 diagnostics 顯示資產 key。
- session 啟動失敗時返回地圖選擇畫面，保留玩家選擇並顯示原因。
- 音效裝置初始化失敗時停用音效但允許遊戲繼續。
- 解析度或視窗模式套用失敗時回退到上一個已知可用設定；若沒有，使用安全的 1280×720 windowed 設定。
- bridge ABI 必須保留 version/size 檢查，新增資料採向後相容的尾端欄位或明確提升 ABI version。

## 測試策略

### Rust 與 generator

- `omoba-template-ids`：ID 唯一性、reference 完整性與生成穩定性。
- `omoba-core`：賽前狀態機、設定 schema、migration、快捷鍵衝突、序列化 round-trip。
- `omoba-sim`：九張地圖與各自難度的 session config、速度和自動開波規則。
- `scripts`：generator golden tests、缺失資產、無效設定、freshness hash 與兩前端輸出一致性。
- `omfx`：抽取共用邏輯後的 regression tests，證明行為與抽取前一致。
- `omfue/bridge`：ABI projection、action dispatch、設定持久化、單機與 networked session。

### Unreal 與啟動流程

- Unreal automation 覆蓋主選單、難度、九圖、Settings、快捷鍵、HUD、音效 cue 與解析度切換。
- `run_ue.bat --build-only` 驗證完整生成及 staging。
- headless smoke 驗證 menu runtime 與 bridge 啟動。
- direct-session smoke 驗證指定 story/difficulty 的除錯入口。
- networked smoke 驗證前端選擇能正確送至 backend。
- 固定 1920×1080、1280×720 與一個 ultrawide 尺寸做 UI screenshot regression。

## 完成標準

- `run_ue.bat` 無參數時進入完整賽前主選單。
- 九張地圖、難度、預覽、Settings、快捷鍵重綁、音效、解析度、速度及自動開波均可實際操作。
- 主要選單與遊戲 HUD 在 reference resolution 下和 `omfx` 無明顯版面或視覺偏差。
- `omfx` 與 `omfue` 從相同來源產生一致的 catalog、穩定 ID、預設快捷鍵、Settings schema、layout tokens 與 session config。
- 可共享資料不再分別硬編碼於兩個前端。
- 相關 Rust tests、generator tests、bridge tests、Unreal automation 與 smoke tests 全部通過。
- 生成輸入改變時 freshness check 能可靠觸發重建；輸入未變時不做不必要的完整重建。

## 實作順序約束

1. 先建立 characterization tests，固定目前 `omfx` 行為與 shipped catalog。
2. 依序擴充 `omoba-template-ids`、`omoba-core`、`omoba-sim` 與 `scripts` 生成流程。
3. 將 `omfx` 切換到共用契約，確認 regression tests 通過。
4. 擴充 `omfue/bridge` ABI 與 UE native presentation。
5. 重建 UMG/Blueprint 畫面並完成視覺驗收。
6. 最後調整 `run_ue.bat` 的預設入口、freshness、staging 與 smoke tests。

這個順序確保共享來源先穩定，再讓兩個前端依序接入，避免以 UE 的臨時複製資料反過來成為新的資料來源。
