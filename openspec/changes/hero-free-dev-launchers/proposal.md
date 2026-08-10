## Why

一般與 10000 金幣開發啟動流程目前會依 TD campaign 資料自動建立 Hero entities，無法提供純塔防／無英雄的本機測試環境。現在需要一個只由指定 launcher 啟用的明確開關，避免修改共用 mission 資料或影響其他啟動方式。

## What Changes

- 新增 `OMB_NO_HEROES=1` runtime opt-in，於 campaign 初始化英雄的唯一邊界跳過 Hero entity 與 hero spawn event 建立。
- `run.bat` 與 `run_10000.bat` 明確設定該旗標；其他 launcher 不變。
- 保留未設定旗標時的既有英雄生成行為。
- 新增初始化 policy 與 hero-free 結果測試，並驗證兩個 batch 檔維持 CRLF、UTF-8 without BOM。
- 記錄 hero-free session 因目前經濟與塔操作綁定 Hero entity，而無法手動建塔、升塔或賣塔的已接受限制。

## Capabilities

### New Capabilities

- `hero-free-dev-launchers`: 定義指定 Windows dev launcher 如何以一致的環境旗標啟動零 Hero entity 的 TD session，以及未啟用時的相容行為。

### Modified Capabilities

無。

## Impact

- 影響 `run.bat`、`run_10000.bat` 與 `omoba-core/src/runtime/native/initialization.rs`。
- 新增 process environment contract：`OMB_NO_HEROES=1`；非 `1` 值與未設定維持現況。
- frontend local simulation 與 launcher-owned backend 必須繼承相同環境值，以維持 lockstep 初始狀態一致。
- 不變更 Lua mission、hero template、protocol、ABI、其他 launcher 或前端選角 UI。
