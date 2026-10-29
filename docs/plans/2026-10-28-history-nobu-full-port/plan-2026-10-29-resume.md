# 恢復後的執行計畫（2026-10-29 晚間）

使用者指示「繼續把遊戲做完」，恢復無人值守與 10 分鐘例行檢查（README D14）。本文件是恢復後的完整計畫；前因見 [handover-2026-10-29.md](handover-2026-10-29.md)，M3' 設計見 [m3-rust-codegen-design.md](m3-rust-codegen-design.md)。

## 0. 已完成的恢復前置

| 項目 | 結果 |
|---|---|
| 現場 | 沒有遊戲／建置程序，`target/locks/` 沒有鎖 |
| commit | `omb` `c84fdfe`、`omfue` `ea4e11b` 已 push，主 repo 與遠端一致 |
| live | `check_live_consistent --ubt` 通過（codegen、cargo、server、bridge、UBT） |
| 清理 | 30 個可刪目錄已刪，釋出 178G；證據目錄（`live-*`、`accept-*`、log）保留 |
| 決定 | `D:\tmp_fix.lua` 保留；產生的 Rust 不進版控 |

## 1. 「做完」的定義

1. 97 位英雄全部上架，每位一個 `hero_<id>.dll`，技能由 v3 產生的 Rust 執行，沒有 IR 直譯器、沒有手寫 handler（hook 除外）。
2. 商店道具全部可用（`items.dll`），價錢、配方、說明都來自 v3 values。
3. 原作 bug 134 項（fix-candidates）全部修好，每項有測試。
4. 遊戲模式完整：塔、兵、野怪營地、樹、中路加兵、兵營被破後追加的兵、建築回血。
5. UE 呈現完整：技能欄、說明框、商店、合成樹、狀態圖示、cue；缺圖用 Grok 補齊並記入 asset-ledger。
6. 1 位玩家＋9 個電腦的實機局（`run_ue.bat`）能在 10 分鐘內自然結束、0 mismatch；`run_10000` 不退步。
7. 網站與作者手冊更新到最終版。

## 2. 工作線與 session

| 線 | session | 內容 |
|---|---|---|
| 主 session（本 session） | — | 調度、10 分鐘例行檢查、審查每次 land、記錄 main-session-log |
| **B 規則層與執行器** | `m2-rules-layer` | M3' ①②④、`ability_kit`、host 端 DLL 載入與熱加載、規則層 bug 修正 |
| **A schema／編譯器** | `m0-schema-v3` | M3' ③ 產生器 `v3_rust.lua`、`v3_codegen_coverage.lua`、各批的產生器寫法 |
| **C UE 呈現** | `e188-spell-amp` | M7 道具呈現收尾、各批 presentation、狀態外觀、Grok 美術匯入 |
| **遊戲模式** | `e187-match-length` | 野怪營地、樹、加兵、建築回血 |
| **conformance** | `hero-port-8-mismatch` | 重跑 E226 後的 124 筆 (c)、下一批類別；比對對象改成產生的 Rust |

規則照舊：在影子改、`land_to_live` 放回；實機局排隊等機器鎖、10 分鐘內結束；每個子 agent 用自己的暫存目錄。

## 3. 階段與順序

### 階段 1：M3' 基礎（關鍵路徑，B＋A）

| 步驟 | 內容 | 驗收 |
|---|---|---|
| ① DLL 拆分 | spike 確認 abi_stable 同型別能載入多個 root module（若不行，改成每個 DLL 匯出不同 root 型別或自建 `libloading` 入口）；稻姬拆成 `hero_a02.dll`；launch 依 manifest 複製整組 DLL；握手與重播檔記錄每個 DLL hash | 實機局 ≤10 分鐘正常結束；hash 不一致拒絕開局 |
| ② 熱加載 | `scripts/hero_hot_reload.lua --hero <id>`、`dev-hot` profile、server／replica 在 tick 之間切換 | 改稻姬一個數值，熱加載後情境結果跟著變；換版前的重播每 tick 相同 |
| ③ 產生器 | `v3_rust.lua` 產生 values、statuses、技能、manifest；`ability_kit` 第一批；`.gitignore` 排除 `src/generated/` | 產生的 crate 在 `deny(warnings)` 下能編譯 |
| ④ 換掉 IR | B01、C17、A02、B04 兩招改成產生的 Rust，與 handler 比對重播；之後刪 `ir.rs`（core、content-model）、`ir_switch.rs`、`tools/ir_probe`、catalog 的 `ir` 欄位 | 5 局重播每 tick 相同；`run_10000` 不退步；live 沒有直譯器 |

同時平行：C 的 M7 道具呈現收尾、遊戲模式的野怪營地與樹、conformance 重跑隱形施法者情境（雜賀眾、三段擊 r3、B06T r3、B29T、C15T）與稻姬毒箭判定修正。

### 階段 2：M5 已上架英雄（B＋A＋conformance）

- 其餘已上架英雄逐位改成產生的 Rust，各自一個 DLL；每位都和原 handler 比對重播。
- 全部換完後刪掉手寫 handler 與 `runtime_id` 對照。
- 跟著該英雄做它在 fix-candidates 裡改資料的 bug。

### 階段 3：M6 其餘 87 位英雄（9 批，照 m6-waves）

每批的固定流程：

1. A／B 補這批需要的產生器寫法與 `ability_kit` 元件（第 1 批：投射物、引導、擊退／拉扯、換技能、全場共用狀態；第 3 批：光環、召喚、地面區域、隱形、分身；第 5 批：充能、召喚物 AI；第 6 批：位移；第 8 批：偷技能；第 9 批：法球、連鎖彈跳）。
2. 產生該批英雄的 crate，修該批的 fix-candidates。
3. conformance 跑該批情境；C 依 `m6-batchN-presentation.md` 接呈現、匯入 Grok 美術。
4. teamfight stress＋一局 10 分鐘實機局，0 mismatch 才算該批完成。

### 階段 4：M7 道具全量

商店道具依解鎖順序改成產生的 Rust（`items.dll`）；還不能用的標「尚未開放」，直到全部開放。

### 階段 5：遊戲模式收尾

中路加兵、兵營被破後追加的兵、建築回血；調整 bot 與 fixture，讓 lifecycle 測試在時限內打完。

### 階段 6：最終驗收與 M8 收尾

- 規則層 bug 全部修完（fix-candidates 134 項都有測試）。
- 全量 conformance、`run_10000`、`run_ue.bat` 10 人局。
- 網站重建、作者手冊改寫、改版說明。

## 4. 例行檢查（每 10 分鐘）

1. 列出 linked session 的狀態；閒置或卡住的，依計畫給下一步或替它做決定。
2. 看 `target/locks/` 與實機局歷史，確認沒有超過 10 分鐘的局、沒有異常中止沒處理。
3. 有新的 land 時，跑 `check_live_consistent`；失敗就通知負責的 session。
4. 進度寫進 `main-session-log.md`；錯誤寫進 `errors.md`（先查最大編號）。
