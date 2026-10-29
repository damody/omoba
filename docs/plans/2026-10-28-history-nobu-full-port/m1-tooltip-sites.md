# M1 工作線 C：UE 顯示說明的位置與 TooltipRenderer 接線

盤點日期：2026-10-28（omfue live）。路徑相對於 `omfue/Plugins/OmRuntime/Source/`。

## 渲染器

| 檔案 | 內容 |
|---|---|
| `OmRuntime/Public/OmTooltipRenderer.h` | 模板解析，以及把 values 代入成 runs `{ 文字, 加粗, 傷害類型 }`；rich markup。規則和網站相同，由共用向量 `docs/character_pipeline/schema/v3-text-render-vectors.lua` 驗證 |
| `OmRuntime/Public/OmTooltipCatalog.h` | 讀 compiled catalog 的 v3 區塊（bridge 匯出的 `om_v3_content_catalog_json`，執行期才查找符號）；`Describe()` 產生標題、資訊列（可附即時值）、屬性行與說明 |
| `OmRuntime/Public/OmTooltipRichStyle.h` | rich text 樣式：OmB（目前等級）、OmTitle、OmLabel、OmDt〈類型〉［B］ |
| `OmRuntime/Public/SOmHudToolTip.h` | 新增 `RichText` 屬性；有值就用 SRichTextBlock，沒有就照舊顯示純文字 |
| `OmEditor/Private/OmTooltipRendererTests.cpp` | `Om.Runtime.TooltipRendererVectors`（共用向量）、`Om.Runtime.TooltipCatalog` |

- **語言**：`FOmTooltipCatalog::CurrentLanguage()` 只有在明確以英文執行時（`-culture=en`，或遊戲設定檔 `[Internationalization] Culture`）用 en，其餘用 zh_TW；該語言沒有文字時退回 zh_TW。遊戲沒有自己的語言設定，引擎預設的 en 不算（E201）。
- **過渡期**：catalog 裡沒有 v3 text 的條目，或 bridge 沒有匯出 catalog 時，一律照舊顯示 description。

## 顯示位置

| # | 位置 | 現在的來源 | 接線狀態 |
|---|---|---|---|
| 1 | 技能 tooltip（技能卡、技能圖示）`UOmCommandBarWidget::BuildAbilityTooltip` | codegen `OmVisualRegistry` 的 `AbilityDefinition.description`，數字寫死在文字裡；資訊列在 C++ 組 | **已接，實機驗證**（`target/tooltip/evidence/`）：有 v3 時改用 rich 文件，等級數列以目前等級加粗；冷卻與耗魔附 bridge 的即時值（不同時才顯示「目前」）；目標、施放狀態、升級條件、按鍵說明照舊附在後面 |
| 2 | 技能升級預覽 `BuildAbilityUpgradeTooltip` | codegen RankBases＋EffectSummary（Rust 組好的中文） | **已接**：有 v3 時只留升級條件；各等級數值由 #1 的說明顯示 |
| 3 | 技能格純文字 tooltip `SetAbilitySlot` | C++ 組名稱、等級、冷卻 | 不改：只有名稱與狀態，沒有說明 |
| 4 | 道具說明 `BuildItemDescription`（#5～#8 共用） | `SourceDescription`（原作文字，E155），或 C++ 依數值欄位組字串 | **已接**：有 v3 時回傳屬性行、說明、資訊列的純文字；搜尋、效果區塊、tooltip 都跟著改 |
| 5 | 商店購買卡 tooltip `BuildShopBuyCardTooltip` | #4 加名稱、價格、配方 | 經 #4（純文字；道具只有一個等級，不需要加粗） |
| 6 | 商店背包、出售格 tooltip | #4 | 經 #4 |
| 7 | 商店詳細欄 `EffectBlocks` | #4，依「被動：」「主動：」等前綴切成區塊 | 經 #4；v3 說明沿用同樣的前綴 |
| 8 | 道具欄 tooltip `ItemSlotTips` | #4 | 經 #4 |
| 9 | 召喚師技能 `BuildSummonerTooltip` | `moba_summoners.lua` 的 description | 未接：catalog 還沒有召喚師技能的 v3 資料 |
| 10 | 守衛／飾品 `BuildWardTooltip` | `moba_trinkets.lua` 的 description | 未接：同上 |
| 11 | 道具名稱 | bridge 的 `MOBA_ITEM_CATALOG` | 不改 |
| 12 | 狀態列表 `FormatBuffList` | 只有 buff 名稱，沒有說明 | 未接：狀態的 v3 text 進 catalog 後再加 |
| 13 | 選角 `UOmHeroSelectionWidget` | 只有名稱與稱號，沒有說明 | 未接：沒有說明可以顯示 |
| 14 | 選取單位 | 名稱與數值 | 不改 |
| 15 | 回城 tooltip | 寫死的中文 | 不改（介面字串，不是內容） |

## 2026-10-29 更新

- #13 選角：**已接**。右側可捲動說明欄顯示英雄各招的 v3 說明（`FOmTooltipCatalog::DescribeHero`）；選角時由 bridge subsystem 的 `EnsureBridgeLoaded()` 載入。
- #12 狀態圖示：**已接**。依 BuffStore id 對應 v3 狀態（`runtime_id`），完整清單附說明（E204）。
- 提示框上限隨字級放大（`SOmHudToolTip::ScaledLimit`），1080p 不再截掉長說明。720p、1080p 截圖在 `target/tooltip/evidence/`。
