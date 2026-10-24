# 共用固定 Lua 驗收入口

後續批次使用 `tools\lua\lua.exe scripts/run_owned_acceptance.lua <spec.json>`。不再使用歷史 Saved `wall-guidance-launch.lua` 的日誌字串成功判斷；已封存檔案保持原樣。

```json
{
  "schema": "omoba-owned-acceptance/1",
  "script": "omfue/Saved/BuildTemp/example-validation.lua",
  "args": [],
  "output": "omfue/Saved/Automation/ExampleValidationV1",
  "summary": "omfue/Saved/Automation/ExampleValidationV1/result.json",
  "watch": ["omfue/Saved/Automation/ExampleValidationV1/engine.log"]
}
```

`output` 必須是尚不存在的專用資料夾。`summary` 可省略；提供時必須在啟動前不存在，最後產生至多 1 MiB 的 JSON 且 `success` 為 true。子工作寫入自己的 `result.json`，不要覆寫啟動器的 `receipt.json`。`args` 原樣傳給固定 Lua，`watch` 最多 64 個明確檔案路徑，只記錄大小和修改時間。沒有 shell fallback。

若既有 runner 會自行建立並要求自己的工作目錄不存在，啟動器的 `output` 請使用另一個專用目錄，例如 `omfue/Saved/Acceptance/Batch/world-v1`；`summary`／`watch` 仍指向 runner 的 `omfue/Saved/Automation/BatchWorldV1`。啟動器會先建立自身 output，不要因此破壞 runner 的新證據目錄檢查。

成功同時要求原始 PID／建立 token／執行檔一致的 Windows 退出碼為 0，以及指定的最終摘要成功。即使中途日誌有 `exit=0` 或先寫出成功摘要，最後錯誤仍失敗。CLI 本身回傳 0 或 1。保留 `process/spawn.json`、`ready.json`、`exit.json`、`identities.json`、原始 stdout/stderr、最終 receipt 與 health；失敗另有 errors.md。只清理自己記錄的原始程序身分。

等待期間每輪最多等待 30 秒，進度採樣門檻 140 秒，正常最大間隔 170 秒。這是進度證據，不代表日誌沒變就自動終止長工作；外部檢查仍須比較 CPU、工作日誌與目前階段。呼叫端需等 CLI 真正結束並核對原始退出證據，不能把 receipt 的存在當成整個上層 workflow 已退休。

批次另行凍結所有正式輸入、驗證來源一致性及封存結果。啟動器只綁定直接 runner 的雜湊，不宣稱替代整個來源快照。實際引擎畫面、authority ACK 與網路對局證據仍須各自驗證。
