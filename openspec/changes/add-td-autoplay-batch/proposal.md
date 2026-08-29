## Why

目前完整 TD 第 1 至 100 關自動測試只能手動輸入多段 Cargo 指令，且容易漏掉 release script DLL 建置。需要一個使用 repository 固定 Lua runtime 的單命令入口，讓開發者可重複執行正確測試流程。

## What Changes

- 新增 `scripts/test_td_1_to_100.lua`，由任意工作目錄皆可透過固定 `D:\code\omoba\tools\lua\lua.exe` 啟動。
- 先建置 release `base_content` script DLL，成功後才執行既有單次 1–100 關整合測試。
- 保留 Cargo 輸出並正確傳遞失敗 exit code，不以 `pause` 阻塞非互動環境。
- 不新增 Batch、PowerShell、Python 或 shell fallback，並維持建置產物排除規則。

## Capabilities

### New Capabilities

- `td-autoplay-batch-runner`: 規範固定 Lua runtime 單命令執行 TD 1–100 關自動測試的流程、錯誤傳遞與產物邊界。能力 ID 為既有名稱，實際入口已遷移至 Lua。

### Modified Capabilities

無。

## Impact

- 新增 `scripts/test_td_1_to_100.lua`。
- 使用既有 `scripts/Cargo.toml`、`omoba-core/Cargo.toml` 與 `td_autoplay_100` integration test。
- 不變更 runtime、遊戲內容、公開 API 或第三方相依套件。
