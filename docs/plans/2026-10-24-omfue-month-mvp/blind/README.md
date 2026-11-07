# 盲測輸入包（G5）

盲測對象兩位，都還沒做過：

| 目錄 | 來源英雄 | 檔案 |
|---|---|---|
| `C15/` | C15 玉子 | `source-data.md` |
| `B29/` | B29 村上義清 | `source-data.md` |

這兩位在 D4（`source-index/roster.md`）就已保留給盲測，不得換人。

## 給盲測 agent 的任務

你只能使用兩份資料：

1. `docs/plans/2026-10-24-omfue-month-mvp/hero-authoring-guide.md`：製作流程與規則。
2. 本目錄下該英雄的 `source-data.md`：來源原文與數值，由 `scripts/hero_source_index.lua export` 產生，只有來源事實。

**不要**參考以下檔案：
- `source-index/` 底下的分析檔案：`candidate-requirements.lua`、`d3-requirements.md`、`candidates-draft.md`、`roster.md`。
- 其他 session 的工作紀錄。

能力狀態查 `docs/plans/2026-10-24-omfue-month-mvp/capability-registry.md`（只有能力登錄，沒有任何英雄的分析），或直接讀程式碼。`hero_author validate` 會自動用完整登錄交叉檢查狀態，你不需要打開它。

交付：
1. 在 `docs/character_pipeline/packages/<hero_id>/` 寫出完整 package：source、character、abilities、capabilities、presentation、assumptions、acceptance，以及 `rust/`。
2. `tools\lua\lua.exe scripts\hero_author.lua validate <package>` 通過。
3. 每個 `rust_handler` 都附單元測試；正式英雄另需 parity soak。兩者都依 guide 的「測試」章節，在自己的 shadow 樹跑過。
4. `acceptance.md` 寫出每招的實機觀察點。
5. 來源要求、但引擎目前做不到的機制，標為 `needs_framework`，並在 `assumptions.md` 說明缺什麼。不可改寫成較簡單的技能，也不可刪招。**禁止修改共用引擎（omoba-core、script-abi、UE）來支援本英雄**；需要新能力時，回報缺口即可。
6. 只用 current 版，不可混入 `_old` 技能。來源 vscript 缺失時，依 KV 與描述推定，並寫進 assumptions。

結果會依 README §8 G5 判定：從原始描述產生、至少一位需要自訂 Rust handler、不得修改共用引擎、不支援的機制如實記為缺口。
