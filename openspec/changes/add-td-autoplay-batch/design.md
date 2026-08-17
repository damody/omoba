## Context

完整 TD 1–100 關測試已存在於 `omoba-core/tests/td_autoplay_100.rs`，但執行前必須先建置 `scripts/target/release/base_content.dll`。專案根目錄只允許既定四個 `.bat`，因此新的專用測試入口必須放在 `scripts/`，並遵循 Windows `cmd.exe` 的 CRLF 要求。

## Goals / Non-Goals

**Goals:**

- 提供單次、無作弊的 1–100 關 autoplay 測試入口。
- 自動建置測試所需的 release script DLL。
- 從任意工作目錄正確定位 repository 根目錄。
- 保留 Cargo 輸出並準確傳遞失敗狀態。

**Non-Goals:**

- 不執行 deterministic replay 的第二次完整模擬。
- 不變更 autoplay 策略、runtime 或 balance。
- 不把建置產物或失敗報告加入版本控制。

## Decisions

1. 批次檔放在 `scripts/test_td_1_to_100.bat`。這符合根目錄腳本限制，也讓用途可由路徑辨識；不採用新增根目錄入口。
2. 使用 `%~dp0` 搭配 `pushd` 取得 repository 根目錄。這比依賴呼叫端目前目錄穩定，並可在結束前以 `popd` 還原目錄。
3. 先執行 `cargo build --manifest-path scripts\Cargo.toml -p base_content --release`，再執行指定 integration test。這比假設既有 DLL 新鮮可靠，且避免執行會跑兩次的 example。
4. 每個失敗點保存 `%errorlevel%`，完成 `popd` 後以相同狀態結束。不加入 `pause`，讓腳本適用於互動終端與 CI。
5. 檔案使用無 BOM UTF-8 與 CRLF，避免 Windows `cmd.exe` 解讀行首錯誤。

## Risks / Trade-offs

- [每次皆檢查 release build，首次執行時間較長] → Cargo 增量建置會重用未變更產物，換取 DLL 新鮮度保證。
- [完整 1–100 測試約需兩分鐘] → 保留即時 `--nocapture` 輸出，讓使用者能看到進度與錯誤。
- [測試失敗可能產生 report] → report 保持在已忽略的 `omoba-core/target/td-autoplay/`，不得加入提交。
