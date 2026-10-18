# 公開模板呈現識別

Selective lockstep protocol 9 新增 `0x464f4715`，不改 renderer C ABI。內容為版本 `1`、一個 byte 的字串長度、1–96 bytes ASCII 英數字／底線模板名稱；不接受多餘 bytes、空名稱、非 ASCII 或路徑符號。

Wave B 只在實體的既有公開 baseline 中放置此欄位，名稱取既有非空 `ScriptUnitTag.unit_id`，沒有腳本標籤則取 `Unit.id`。這個欄位不會插入 `ScriptUnitTag`，不載入腳本、不執行 callback、不公開 Unit 的私有目標、計時或 AI 資料。仍由原有 team／vision／scope 過濾整個實體。

Filtered replica 僅保存與驗證呈現識別，不將它 materialize 為玩法元件。Bridge 僅解碼已公開 bytes；若與已公開腳本名稱衝突，拒絕該列。塔／主堡的種類使用既有有界 `0x464f4710` 結構角色，而不憑字串前綴或挪用別的模板。未登錄名稱的 catalog ID 仍為零，不能冒充現有英雄或小兵。

本次同時修正既有 ID 衝突：戰鬥關係保持 `0x464f4713`，回城 channel 狀態移至 `0x464f4716`；ward body 保持 `0x464f4714`。secure allowlist 建立時必須保有全部 schema 的唯一性，重複立即失敗。Protocol 8 peer 拒絕連線，不能將新版 bytes 交給舊解碼器。

驗證來源：`.ai-collab/world-identity-rust-v2/summary.json` 與各原始 log。先前兩輪真失敗保留於 E897／E898，不能以新結果覆蓋。模型匯入與正式場景呈現另外驗證。
