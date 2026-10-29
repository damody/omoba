# 完整還原 history_nobu：技能與內容系統重構計畫

建立日期：2026-10-28（Asia/Taipei）。狀態：執行中。2026-10-29 16:22 依使用者指示全部停止，交接見 [handover-2026-10-29.md](handover-2026-10-29.md)。

> **2026-10-29 架構變更（D15～D16）**：取消執行期的 behavior IR 直譯器。v3 文件改在建置期**產生 Rust 原始碼**，交給 rustc 編譯，換取極致性能與編譯器檢查。另外改成**每位英雄一個 DLL、可熱加載**，讓編譯更快，開發時也能像腳本一樣在 runtime 驗證。設計見 [m3-rust-codegen-design.md](m3-rust-codegen-design.md)。[m3-ir-design.md](m3-ir-design.md) 與 [ir-blockers.md](ir-blockers.md) 只保留作歷史紀錄。

**目標**：原作 nobu 所有英雄的 current 版技能、所有商店道具，都在 omoba 裡照原作還原。同時把技能系統重構成「資料驅動、數值單一來源」。

**本計畫的文件**

| 文件 | 內容 |
|---|---|
| 本文件 | 決策、架構、工作線、里程碑、風險 |
| [behavior-model-v1.md](behavior-model-v1.md) | 自有技能 API：規則層、攻擊管線、技能、狀態、動作、選取、運算式 |
| [value-single-source.md](value-single-source.md) | 數值單一來源：玩法、遊戲內說明、網站共用一份數值 |
| [behavior-model-v0.md](behavior-model-v0.md)、[census/](census/) | 試點普查的草案與結果 |

原作：`C:\nobu\history_nobu`（Dota 2 addon，commit `f966720`）。舊官網：[damody/nobugg_official](https://github.com/damody/nobugg_official)，本機副本在 `target/nobugg_src`。遊戲常數見 [history-nobu-analysis.md](../2026-10-24-omfue-month-mvp/source-index/history-nobu-analysis.md)。

---

## 目前進度（2026-10-29 16:30，全部停止時）

| 里程碑 | 狀態 |
|---|---|
| M0 schema v3 | ✓ v3.0 凍結；之後依普查擴充到 v3.3（`docs/character_pipeline/schema/v3.md`），加上 `host` 欄位與法術增幅規則 |
| M1 數值單一來源 | ✓ 資料線與 UE 都已放回 live：技能欄、商店、選角改用 v3 說明，狀態說明與狀態外觀也已完成（E204、E218） |
| M8 網站 | ✓ 全量 512 頁（97 位英雄、156 件道具，繁中與英文；未上架英雄有標示；改版說明自動產生） |
| M3 執行器 | **改方向（D15）**。目前 live 上有 IR 直譯器（`omoba-core/src/runtime/native/ir.rs`），由它執行雜賀孫市 B01、阿市 C17、稻姬 A02，以及伊達政宗 B04 的兩招（E212、E219、E224）。之後改成由產生的 Rust 執行，再移除直譯器（§5 M3'） |
| M5 前置 | conformance：1303 通過、25 已知差異 (b)、124 待修 (c)、837 unsupported（E227）。harness 與重播比對工具不依賴 IR，可以沿用 |
| M2 規則層 | ✓ 兩批都已放回 live（E193、E198），之後補上 E221、E226；10 位英雄 parity soak 0 mismatch |
| M6／M7 前置 | 9 批呈現覆蓋報告、缺圖清單；商店價錢、配方、屬性改由 v3 values 產生（E222） |
| M4 全量普查 | ✓ 97 位英雄＋全部商店道具共 105 份 v3 文件，全部通過驗證（含跨文件登錄表）；614 筆中 598 筆可表達（97.4%）、0 筆需要 hook、6 筆待實測；出處比對 4057 筆一致。詳見 [census-v3/RESULTS.md](census-v3/RESULTS.md) |
| 遊戲模式 | E187 原作塔、兵、野怪數值與成長規則已放回；quick 局長中位 7.5～8 分、UE 實機 7:32 自然結束；剩中路加兵、兵營破後追加波、建築回血 |

## 1. 為什麼要重構

1. **規模。**
   - 原作有 97 位英雄、444 招 current 版技能（含子技能），背後是 1,405 個腳本函式、約 3.9 萬行；另有 173 個道具、約 1.1 萬行道具腳本。
   - 現在每招技能都寫成一個 Rust handler。10 位英雄花了大約兩天。
   - 照這個方式做剩下的英雄，每位都會各自重做一套小框架，而且很難維持一致。
2. **內容模型是「一個功能一個欄位」。** `AbilityDefinition` 不斷長出專用欄位：`projectile`、`charges`、`summon`、`attack_orb`、`channel`、`zone_lifetime`……每多一種機制就要改 content-model、codegen、bridge。
3. **數值寫在很多地方。** package 的 `description` 是寫死數字的長句，`effects` 又寫一次，Rust 註解再抄一次。原作本身也是 KV、說明、腳本三份不同步。使用者要求數值只寫一次（見 [value-single-source.md](value-single-source.md)）。
4. **試點證明資料驅動可行。** 最難的 8 位英雄 43 筆技能、25 個道具，用 v0 詞彙寫出來，只有 4 筆需要 Rust hook。v1 補上概念後可以再消除 3 筆。其餘 184 個缺口都是詞彙不夠，已整理進 v1。

## 2. 已確定的決策

| # | 決策 | 來源 |
|---|---|---|
| D1 | 自有 API，不追求與 Dota 相容。Dota API 只用來理解原作 | 使用者 2026-10-28 |
| D2 | 只還原 current 版技能；`_old`、`_20` 不做 | 使用者 2026-10-28 |
| D3 | 不允許兩人選同一位英雄（維持 E164） | 使用者 2026-10-28 |
| D4 | 全量普查用子 agent，結果回到主對話彙整 | 使用者 2026-10-28 |
| D5 | 數值單一來源：玩法、遊戲內說明、網站共用一份 `values`；說明文字裡不能有數字 | 使用者 2026-10-28 |
| D6 | 攻擊管線分前搖、發出、後搖三段；命中與未命中是判定事件 | 使用者 2026-10-28 |
| D7 | 以資料為主，少數例外寫 Rust hook，每個 hook 都要寫明理由。**門檻（使用者 2026-10-30）：有 2 招以上類似的機制就做成共用（v3 詞彙＋產生器＋`ability_kit`）；跟其他招完全不同的才寫 hook。** hook 由 AI 實作產生的 trait 方法，只能用產生的常數與 `ability_kit`，v3 仍要寫行為描述或預期結果供 conformance 比對（m3-rust-codegen-design §2.4） | 試點結果；使用者 2026-10-30 |
| D8 | 遊戲執行期沒有 Lua；所有數值用 Fixed64。~~建置期把資料編譯成 IR~~（已由 D15 取代） | AGENTS.md compiled-content-only |
| D9 | 全遊戲共用的結算規則集中在規則層，技能資料不可以各自重做 | 試點結果 |
| D10 | 以漸進取代的方式遷移：新舊並存，逐招切換，用重播證明行為不變 | 本計畫 |
| D11 | 網站只做繁體中文與英文；banpick、預覽影片、勝率統計等工具先不做 | 使用者 2026-10-28 |
| D12 | 核心程式（schema、規則層、執行器、UE、網站）由固定的 session 負責；普查用子 agent | 使用者 2026-10-28 |
| D13 | 原作 bug 預設照實際行為；跟 tick 頻率有關的照設計意圖。**[fix-candidates.md](fix-candidates.md) 的 134 項全部要修**（使用者 2026-10-29）。修法照各條的建議欄，要改 v3 文件或規則層；每一條都要有測試，證明修好之後的行為 | 使用者 2026-10-29 |
| D14 | 2026-10-28 起無人值守 72 小時執行本計畫，主 session 每 10 分鐘檢查一次。**2026-10-29 16:22 使用者指示全部停止**；同日晚間使用者決定**恢復無人值守與 10 分鐘例行檢查**，目標是把遊戲做完 | 使用者 2026-10-28、2026-10-29 |
| D15 | **不用執行期 IR。** v3 文件在建置期由 Lua 編譯器產生 Rust 原始碼，經 rustc 編譯後原生執行。數值、狀態 id、StatKey、cue id 都產生成具型別的常數或列舉，寫錯在編譯期就會被擋下。少數例外寫手寫 Rust hook（D7 不變）。現有的 IR 直譯器只做過渡，逐招換成產生的 Rust 後就刪除 | 使用者 2026-10-29：「用 rust 就是要極致的性能跟編譯器的檢查能力」 |
| D16 | **每位英雄一個 DLL，可熱加載。** 改一位英雄只重編那一個 DLL；開發用的 server 不必重開就能載入新版，接著馬上跑該英雄的情境測試，像腳本一樣在 runtime 驗證。正式對局在開局時固定每個 DLL 的 hash，對局中不熱加載 | 使用者 2026-10-29 |
| D17 | 缺的美術（頭像、技能圖示、cue 參考圖）用 Grok 生成。沿用既有流程：Grok Build CLI 的 `image_gen`，經 `scripts/item_icon_import.lua`／`ui_texture_import.lua` 匯入，每次生成都記到 `asset-ledger.md` | 使用者 2026-10-29 |

## 3. 目標架構

```text
                原作 KV＋腳本（唯讀、釘選 commit）
                        │  普查：子 agent 讀原作 → 寫 v3 package
                        ▼
 docs/character_pipeline/packages/<hero>/      ← 唯一的作者資料
   values（數值）＋ text（說明模板）＋ behavior（v1 行為）＋ hooks（少數）
   ＋ source／parity／assumptions（出處與原作 bug）
                        │  建置期 Lua：驗證 → 求衍生值 → 產生 Rust 原始碼 ＋ content
                        ▼
 產生的 Rust（每位英雄一個 crate，不手改）＋ 手寫 hook
   │  rustc 編譯（型別檢查；數值是 Fixed64 常數）
   ▼
 hero_<id>.dll × 97、items.dll、base_content.dll（塔、兵、野怪等共用單位）
   ├─▶ omoba-core host：規則層 ＋ 共用元件（ability_kit）；依 DLL 註冊技能、可熱加載
   │      └─ authority／team replica 載入同一組 DLL（hash 相同）決定性執行
 compiled content catalog（JSON，content hash；只有 values、text、呈現資料，沒有行為）
   ├─▶ UE（bridge 內嵌 catalog）：通用 TooltipRenderer、技能欄、指示器、狀態圖示、cue
   ├─▶ 網站產生器（Lua）：靜態 HTML，含英雄、技能、道具、商店、合成樹、改版說明
   └─▶ bot、測試、文件
```

### 3.1 各層的責任

| 層 | 放在哪裡 | 內容 |
|---|---|---|
| 作者資料 | `docs/character_pipeline/packages/`；共用規則數值在 `scripts/lua_data/templates/game_rules.lua` | 只有資料；數字只出現在 `values` |
| 編譯器 | `scripts/hero_author/`、`docs/character_pipeline/tools/`（Lua，建置期） | schema 驗證、運算式型別檢查、單一來源檢查、**Rust 產生器**、出處比對 |
| 產生的 Rust | `scripts/heroes/<hero>/`（每位英雄一個 cdylib crate）、`scripts/items/` | 每招一個 `AbilityScript` 實作，數值、狀態、StatKey 都是產生的常數或列舉。由產生器輸出，不手改 |
| content model | `omoba-content-model`、`omoba-template-ids` | `values`（Fixed64）、`text`、呈現資料。不放行為。取代現在的專用欄位 |
| 共用元件 | `scripts/ability_kit/`（每個英雄 DLL 都靜態連結）；`omoba-core/src/runtime/native/` | 產生的程式碼呼叫的函式庫：選取、排程器（delay／repeat／periodic）、狀態實例、投射物、位移、光環。規則層（傷害、攻擊、緩速、魔抗、狀態抗性、治療）留在 host。重用現有元件，例如 ability_projectile、forced_motion、reveal_zone、BuffStore |
| hook | 跟所屬英雄放在同一個 crate | 手寫的 Rust，只讀產生的數值常數 |
| DLL 載入 | `omb`／omoba-core 的 script host | 依 manifest 載入各英雄 DLL；開發模式可熱加載；開局時固定每個 DLL 的 hash |
| 呈現 | `omfue` | 通用 UI，不為個別英雄加 C++ 特判 |
| 網站 | `scripts/build_site.lua`；產物在 `target/site/` | 只讀 compiled catalog |

### 3.2 必須先想清楚的設計約束

1. **replica 的可見性。**
   - 執行器在 authority 和每個隊伍的 replica 上都要跑出相同結果。但 replica 看不到敵方的隱藏狀態，這正是 E168、E175 那類 mismatch 的來源。
   - 每一種狀態與變數都要宣告揭露政策：`public`（隨單位揭露）、`team`（只有己方知道）、`derived`（只揭露結算結果，例如屬性總值）。
   - 規則是：**會影響對方 replica 決策的判定，只能讀雙方都有的狀態。** M2 要先做一個 spike 驗證這條規則。
2. **效能。**
   - 技能是原生 Rust，沒有直譯迴圈。產生的程式碼不能在執行期查字串，熱路徑上也不能配置記憶體；事件訂閱表在載入時就建好。
   - host 與英雄 DLL 之間是 abi_stable 的 FFI 邊界，呼叫一次的成本固定。「每單位、每 tick」的工作放在 host 的共用元件裡批次處理，不要每個 tick 跨 FFI 呼叫每個單位。
   - 每個 tick 只處理有事件或排程的單位。
   - 以 `run_10000.bat` 量測，不能退步超過既有預算。
3. **熱加載的決定性（D16）。**
   - 技能狀態一律放在 host（BuffStore、排程器、狀態變數），DLL 本身不保存狀態。這樣換掉程式碼也不會遺失狀態。
   - abi_stable 不支援卸載 library。熱加載時，把新版複製成新檔名再載入，切換註冊表；舊版留在記憶體裡（開發用可以接受）。
   - authority 與 replica 的每個 DLL hash 都要相同；hash 記進重播檔與 lockstep 握手，不一致就拒絕開局（fail closed）。
   - 正式對局中不熱加載。開發用 server 只在 tick 之間切換，並在重播紀錄裡記下切換的 tick。
4. **決定性。** 數值一律在建置期轉成 Fixed64 常數；亂數走每 tick 的決定性 seed；selector 依 entity id 排序；排程與狀態變數都進 hash。
5. **協定版本。** 新增權威事實或呈現欄位時，依既有規則升 lockstep、presentation、CABI 版本；舊版要 fail closed。
6. **擴充方式。** schema 有版本號。改 schema 時附遷移腳本，一次把所有 package 升級，不讓新舊格式長期並存。

## 4. 工作線

每條工作線有固定的負責範圍，避免平行修改同一批檔案。所有 live 變更都經 `land_to_live` 放回。

| 工作線 | 範圍（獨佔的檔案） | 內容 |
|---|---|---|
| **A 資料與工具** | `docs/character_pipeline/schema`、`tools`、`scripts/hero_author`、`omoba-content-model` | schema v3、驗證器、單一來源檢查、Rust 產生器、出處比對、遷移腳本 |
| **B 規則層與執行器** | `omoba-core/src/runtime/native`、`scripts/ability_kit`、script host 的 DLL 載入（依模組再細分負責人） | 規則層、攻擊與傷害管線、共用元件（狀態實例、事件、排程器、選取）、每位英雄一個 DLL 的載入與熱加載 |
| **C 呈現** | `omfue` | TooltipRenderer、動態技能欄（換技能、子技能、重施放）、向量施法輸入、狀態圖示、cue 目錄 |
| **D 普查** | 只寫 `docs/character_pipeline/packages/<hero>` 與 `census/` | 子 agent 平行處理，每個 agent 負責 4～5 位英雄或 20～25 個道具 |
| **E 網站** | `scripts/build_site.lua`、`site/` 版型 | 從 catalog 產生靜態網站 |
| **F 遷移與量產** | 各英雄的 package 與 hook | 舊英雄改成資料；新英雄分批上架與驗收 |

遊戲模式（小兵、野怪、魔王）、地圖（DMX 解析）、美術素材，在本計畫之外平行進行。

## 5. 里程碑

沒有日曆，依相依關係推進。每個里程碑結束時，遊戲都要能正常開局（`check_live_consistent` 通過，實機 ≤10 分鐘一局）。

| 里程碑 | 工作線 | 內容 | 結束條件 |
|---|---|---|---|
| **M0 定案** | A | v1 定案；schema v3 規格與驗證器，包含 values、text、behavior、hooks；用 v3 改寫 8 位試點英雄與 25 個道具 | 驗證器通過；每一筆的狀態都是可表達、需要 hook 或待確認；schema 凍結為 v3.0 |
| **M1 數值單一來源打通** | A、C、E | 已有的 10 位英雄與已上架道具改用 values＋text；UE 改用 TooltipRenderer；網站產生器 MVP；出處比對 | 這些英雄的說明裡沒有數字；網站、遊戲內說明、結算三處數字一致（自動測試）；網站顯示 content hash |
| **M2 規則層與管線** | B | 傷害四段管線、攻擊三段管線、魔免、魔抗池、緩速、狀態抗性、減療、偽隨機；揭露政策 spike | 規則層單元測試；現有 10 位英雄重播 parity 0 mismatch；`run_10000` 效能不退步 |
| ~~M3 執行器核心~~ | A、B | ~~IR 編譯器與直譯器~~。已完成 B01、C17、A02、B04 兩招；由 M3' 取代 | — |
| **M3' Rust 產生器與每英雄一個 DLL** | A、B | ① 英雄 DLL 拆分與熱加載（script host 依 manifest 載入，開局時固定 hash）；② `ability_kit` 共用元件；③ Lua 產生器：v3 → Rust；④ 先把目前由 IR 執行的 B01、C17、A02，以及 B04 的兩招改成產生的 Rust，再刪掉 `ir.rs`、`ir_switch`、`ir_probe` | 這幾招產生的 Rust 與 Rust handler 的重播每 tick 狀態 hash 相同；改一位英雄只重編那一個 DLL，熱加載後馬上跑該英雄的情境；`run_10000` 不退步；live 不再有 IR 直譯器 |
| **M4 全量普查** | D | 剩下 79 位英雄的 current 版、其餘約 150 個商店道具，直接寫成 v3 package。M0 之後就可以開始，與 M1～M3 平行 | 97 位英雄與全部商店道具都有 v3 package 並通過驗證；統計每個 action、狀態、規則、hook 的使用次數 |
| **M5 舊英雄全部遷移** | F | 已有的 10 位英雄全部改成由產生的 Rust 執行，各自一個 DLL；刪除 content 的專用欄位與對應的手寫 handler | 10 位重播 hash 相同、實機逐招 PASS；舊欄位移除 |
| **M6 量產** | A、B、F | 依 M4 的使用次數，補齊產生器的寫法與 `ability_kit` 元件；英雄每批 8～10 位上架 | 每批：編譯、測試（每招每個 rank，加上原作 quirks 推導的情境）、實機逐招 PASS；進正式選角目錄 |
| **M7 道具全量** | B、F | 全部商店道具改成產生的 Rust（`items.dll`） | 商店全部上架，或明確標示 deferred 與原因 |
| **FC 原作 bug 全修** | A、B | [fix-candidates.md](fix-candidates.md) 134 項全部修正（D13）。改資料的跟著該英雄的批次做，改規則層的由 B 統一做 | 每一項都有測試證明修好之後的行為；fix-candidates 逐條標記完成 |
| **美術補齊** | C | [m6-missing-art.md](m6-missing-art.md) 的頭像、技能圖示、cue 參考圖用 Grok 生成（D17） | 清單逐項有圖；asset-ledger 有紀錄 |
| **M8 網站與文件完成** | E、A | 完整網站、自動改版說明、作者手冊改寫 | 舊官網的英雄、技能、道具、商店頁都有對應；文件自動產生 |

**關鍵路徑**：M0 → M2 → M3' → M5 → M6。M1、M4、M8 可以平行。M4 越早完成，M6 的優先序就越早確定。

## 6. 驗證方式

| 層次 | 驗證什麼 | 怎麼做 |
|---|---|---|
| 出處 | 我們的數值與原作一致 | `source = "kv:..."` 的值，匯入時與釘選的原作 KV 比對；不一致必須寫理由 |
| 單一來源 | 說明、網站、結算三處一致 | 驗證器檢查（見 value-single-source §5），加上 UE 與網站產物的快照測試 |
| 元件 | 執行器每個 action、狀態、selector、規則 | conformance 單元測試 |
| 行為 | 每招、每個 rank | 由普查的 flow 與 quirks 推導測試情境：正常、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性 |
| 遷移 | 改成產生的 Rust 後行為不變 | 重播既有實機局的 accepted inputs，比對每 tick 的狀態 hash |
| 編譯期 | 引用錯數值、狀態、屬性 | 產生的程式碼只能引用產生的常數與列舉，寫錯 rustc 就編不過；產生的 crate 開 `#![deny(warnings)]` |
| 熱加載 | 改一位英雄後馬上驗證 | 只重編 `hero_<id>.dll`，開發用 server 熱加載，接著跑該英雄的 conformance 情境；換版前後同一個情境的結果一致 |
| parity | authority 與 replica 一致 | 既有 parity soak，加上每批英雄的 role-bot soak |
| 實機 | 真的能玩 | 每批英雄開一局 ≤10 分鐘，`check_hero_live_acceptance` 逐招判定 |

注意：行為測試的期望值可以引用 `values`，但「數值本身對不對」由出處比對負責。這兩層不能互相代替。

## 7. 風險

| 風險 | 影響 | 對策 |
|---|---|---|
| 規模（97 位英雄、173 個道具） | 拖很久或品質不一 | 先打通流程（M0～M3）再量產；普查與量產依機制分群；每批有固定驗收 |
| replica 可見性分歧 | 對局中止（E168、E175 類） | §3.2 第 1 點的揭露政策；M2 spike；每批 parity soak |
| 執行器效能 | 高負載場景退步 | 原生 Rust；每單位、每 tick 的工作在 host 批次處理，不逐單位跨 FFI；`run_10000` 當 gate |
| 編譯時間 | 444 招＋173 道具的產生程式碼讓 base_content 變很慢 | 每位英雄一個 crate，只重編改過的那一個（E206 的教訓：只產生有改的 package，但相依的要一起重建）；共用的 `ability_kit` 少改 |
| DLL 數量與熱加載 | 載入很多 DLL；abi_stable 不能卸載；hash 不一致造成 desync | 開局時固定 hash，並寫進重播檔與握手；開發模式才熱加載，舊版留在記憶體；狀態都在 host |
| schema 反覆修改 | 已寫好的 package 要重寫 | M0 凍結 v3.0；之後只做向後相容的擴充；不相容的修改附遷移腳本 |
| 子 agent 普查品質 | 規格錯誤 | 驗證器、出處比對、抽樣交叉審查（另一個 agent 盲讀原作比對）、信心標記 |
| 原作 bug 的取捨 | 還原結果有爭議 | §9 的政策；修正候選清單由使用者逐條決定 |
| 平行修改衝突 | live 半成品、覆蓋別人的修改（E143） | 工作線獨佔範圍；land_to_live；一次只有一個 schema 負責人 |
| 與既有 session 衝突 | 規則層的修改和進行中的工作重疊 | 法術增幅、比賽長度兩個 session 收尾後再啟動 M2 |

## 8. 成本估計

- **普查**：試點每位英雄約 85k token，每個道具約 14k token。剩下 79 位英雄約 6.7M、約 150 個道具約 2.1M，合計約 9M token。用 v3 驗證器與範本之後應該會下降。
- **執行器與規則層**：M2、M3 是新的核心程式，加上測試，規模接近現有 omoba-core native 模組的一大部分。這是最需要謹慎設計的部分，建議由固定的 session 負責，不用子 agent 分散進行。
- **量產**：主要成本是每批的實機驗收與修 bug。執行器元件齊全之後，每位英雄的成本應該遠低於目前的手寫方式。

## 9. 需要使用者決定的事

2026-10-28 已回覆：網站只做繁中與英文、不做其他工具（D11）；核心程式用固定 session（D12）。

2026-10-29 已回覆：
- 原作 bug 134 項全部修（D13）；
- 缺圖用 Grok 生成（D17）；
- `target/_to-delete-2026-10-28`（515G）要刪，已刪除；
- 目前的變更要 commit；
- 不用 IR，改成產生 Rust（D15）；每位英雄一個 DLL，可熱加載（D16）。

以下保留原始問題供參考。

1. **原作 bug 的還原政策**（v1 §11）：
   - 預設照實際行為；
   - 明顯違反設計意圖的，列入修正候選清單給你逐條決定；
   - 跟 tick 頻率有關的行為照設計意圖。
   這樣可以嗎？
2. **網站**：
   - 要支援哪些語言？舊站有 zh-TW、zh-CN、en、jp 四種。
   - banpick 工具、英雄預覽影片、勝率統計要不要做？
3. **工作方式**：M2、M3 這類核心程式，是否用固定的 session 負責（可以點開來看）？普查照 D4 用子 agent。

## 10. 試點普查結果（2026-10-28）

| 範圍 | 筆數 | 缺口 | hook | 信心（高／中／低） |
|---|---|---|---|---|
| 英雄 B22、C06、C07、C01、A21、A13、B11、A11 | 43 | 121 | 2 | 22／20／1 |
| 道具 25 個（以 deferred、needs_framework 為主） | 25 | 63 | 2 | 13／12／0 |

缺口已全部整理進 v1：

| 缺口 | v1 位置 |
|---|---|
| 全域規則：魔免、攻擊者死亡不造成傷害、狀態抗性、自訂緩速、減療、魔抗池、分身倍率 | §1 |
| 攻擊管線：三段、命中與未命中事件、法球、以技能觸發攻擊、單次攻擊紀錄 | §2 |
| 條件與數值運算式、事件參數、執行 context | §7 |
| 狀態：運算子、顯示層數與數值層數、`vars`、`scope`、吸收、次數、驅散篩選、逐屬性取最高 | §4 |
| 實體引用、單位模板 | §8、§9 |
| 經濟、背包、樹木 | §5 |
| 空間：向量施法、路徑投射物、點位產生器、地標、`prefer` | §5、§6 |
| 技能：tag、多組共用冷卻、限時換技能、施法前攔截、等級連動 | §3 |

## 11. 原作規模

| 類別 | 數量 |
|---|---|
| 英雄 | 97 位；`herolist.txt` 開放 75 位；已移植 10 位 |
| 技能 | 861 個條目；current 版 444 招（含子技能） |
| 技能腳本 | 121 個檔、39,184 行；不同的 RunScript 函式 1,405 個 |
| 道具 | 173 個；商店 156～177 項；腳本 103 個檔、11,466 行 |
| 單位 | 235 個 |
| 呈現 | 粒子 3,834 個、音效 1,799 個、模型 fbx 329 個＋dmx 575 個 |
