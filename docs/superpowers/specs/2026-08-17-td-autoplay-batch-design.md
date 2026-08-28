# TD 1–100 自動測試批次檔設計

## 目標

提供一個 Windows 批次檔，讓開發者以單一命令執行一次完整的 TD 第 1 至 100 關自動測試。批次檔必須先確保 release 版 script DLL 已建置，再執行既有的精準整合測試。

## 位置與介面

- 檔案位置：`scripts/test_td_1_to_100.lua`
- 呼叫方式：從任意工作目錄執行該批次檔，不接受額外參數。
- 不新增根目錄 `.bat`，維持根目錄保留腳本限制。
- 批次檔使用 CRLF 行尾。

## 執行流程

1. 以 `%~dp0` 解析批次檔位置並切換至 repository 根目錄。
2. 執行 `cargo build --manifest-path scripts\Cargo.toml -p base_content --release`，產生測試需要的 release script DLL。
3. 建置成功後，執行 `cargo test --manifest-path omoba-core\Cargo.toml --test td_autoplay_100 layered_td_coarse_autoplay_completes_rounds_1_to_100 -- --nocapture`。
4. 保留 Cargo 的原始輸出，並將最後一個命令的 exit code 傳回呼叫端。

## 錯誤處理

- repository 根目錄切換失敗時立即結束並回傳非零狀態。
- script DLL 建置失敗時不執行 autoplay 測試。
- autoplay 測試失敗時保留既有 failure report 行為，報告位於 `omoba-core/target/td-autoplay/failure.txt`。
- 不使用 `pause`，避免 CI 或非互動終端被阻塞。

## 驗證

- 檢查檔案為 CRLF。
- 從 repository 根目錄以 `D:\code\omoba\tools\lua\lua.exe scripts\test_td_1_to_100.lua` 執行。
- 確認完整 1–100 關測試通過且批次檔回傳 exit code 0。
- 確認沒有將 DLL、`target/`、log 或 failure report 加入版本控制。
