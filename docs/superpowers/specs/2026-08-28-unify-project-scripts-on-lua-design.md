# 專案腳本統一為 Lua 設計

## 目標

將 repository 內受版控的批次、PowerShell、Python 與 shell 腳本統一移植為 Lua 5.4，固定使用 `D:\code\omoba\tools\lua\lua.exe`。保留專案規定的四個根目錄 `.bat` 作為可雙擊的薄入口，但所有建置、程序、網路、測試與證據邏輯都由 Lua 執行。

## 範圍

基準盤點包含 7 個 `.bat`、11 個 `.ps1`、11 個 `.py` 與 1 個 `.sh`。實作 SHALL：

- 保留 `run.bat`、`run_10000.bat`、`run_2player.bat`、`run_ue.bat`，並將其縮減為呼叫固定 Lua runtime 的薄入口。
- 將其他受版控的 `.bat`、`.ps1`、`.py`、`.sh` 功能移植成 `.lua` 後移除舊檔。
- 更新程式碼、測試、文件與 OpenSpec 中仍會實際執行舊腳本的呼叫路徑；歷史歸檔文件只在其指令仍是現行操作入口時更新，避免改寫歷史敘述。
- 保留既有 CLI 參數、環境變數、exit code、輸出格式、產物路徑、程序生命週期與安全檢查。
- 不改變 gameplay、lockstep、戰爭迷霧、netem 延遲模型或證據判定語意。

## 架構

### Lua runtime 與入口

所有 launcher 明確呼叫 `D:\code\omoba\tools\lua\lua.exe`，不得依賴 PATH 裡的其他 Lua。四個根目錄 `.bat` 只負責定位 repository、轉送原始參數、呼叫相應 Lua 主程式並回傳 exit code；不得保留業務流程分支。

主要 Lua 程式放在 `scripts/` 或原工具所在目錄，檔名沿用原用途。例如 `scripts/run_client_delay_scenario.lua` 取代同名 PowerShell 腳本。工具專屬腳本保留在 `docs/character_pipeline/tools/`、`docs/tools/` 與 `tools/selective_lockstep/`，避免把互不相關的責任集中到單一檔案。

### 共用模組

建立 `tools/lua/lib/`，模組保持單一責任：

- `args.lua`：Windows 參數解析、旗標與型別驗證。
- `path.lua`：路徑正規化、repository root、引號與檔案操作。
- `process.lua`：同步命令、背景程序、PID、等待、終止與 executable 身分驗證。
- `json.lua`：目前腳本所需的 JSON encode/decode，不接受靜默資料遺失。
- `udp.lua`：netem control datagram。
- `hash.lua`：透過 Windows 系統工具計算 SHA-256，並驗證結果格式。
- `time.lua`：UTC timestamp、deadline 與 polling。
- `evidence.lua`：JSONL、manifest、verdict 與不可覆寫規則。

Lua 5.4 標準庫不直接提供 process、UDP 或 SHA-256。Windows 平台採用明確且可驗證的系統命令或專案既有 executable；不得透過 PowerShell 或 Python 回退。若 UDP 無法只靠標準 Lua 完成，新增小型 Rust helper binary，Lua 仍是工作流與資料語意的唯一腳本層。

## 行為相容性

每支 Lua replacement 要建立輸入／輸出契約表，至少包含：參數、環境變數、成功 exit code、失敗 exit code、寫入檔案、啟動程序與清理順序。轉換時維持：

- build freshness 與 artifact staging 判斷。
- server、team runtime、netem proxy、renderer 的啟動及反向停止順序。
- PID 必須同時匹配預期 executable，禁止廣泛終止同名程序。
- evidence run ID 不可覆寫、Team 1／Team 2 路由隔離、直方圖與 profile control。
- Windows 批次入口使用 CRLF、UTF-8 無 BOM。
- Linux 專用診斷功能由 Lua 依平台執行等價工具；Windows launcher 不會誤走 Linux 分支。

## 錯誤處理與安全性

- 所有外部命令必須保留原始 exit code，錯誤訊息包含用途與目標，不得只回傳通用失敗。
- 路徑含空白時必須安全引用；不得把未驗證輸入拼成破壞性命令。
- 刪除或終止前先解析精確路徑或 PID，並驗證位於預期 workspace／對應 executable。
- 背景程序啟動失敗時，已啟動的同 session 子程序依反向順序清理。
- JSON schema、20-bin 權重、PID manifest 與 evidence sentinel 採 fail-closed。

## 遷移順序

1. 建立 Lua 共用模組與測試 harness。
2. 轉換低依賴的 generator、validator 與 selective-lockstep 工具。
3. 轉換 process、memory、screenshot、manifest 與 evidence helpers。
4. 轉換 netem scenario、matrix、control 與 lifecycle。
5. 轉換四個主要 launcher 的實際邏輯，最後縮減根目錄 `.bat`。
6. 更新全部現行呼叫端、AGENTS.md 與操作文件。
7. 移除已被替代的舊腳本。
8. 所有功能完成後才集中執行完整測試。

## 測試策略

測試集中在實作最後，避免每完成一小步就跑完整 suite。最終驗證包含：

- 對所有 `.lua` 執行 `loadfile` 語法檢查。
- 共用模組的參數、JSON、路徑、命令引用與錯誤傳播測試。
- 掃描受版控檔案，確認除四個薄 `.bat` 外沒有 `.ps1`、`.py`、`.sh` 或額外 `.bat`。
- 驗證四個 `.bat` 僅啟動指定 Lua、轉送參數與回傳 exit code，且維持 CRLF／無 BOM。
- 執行 generator、validator、selective-lockstep 與 TD autoplay 腳本 smoke。
- 執行 server + Team 1 runtime + Team 2 runtime 的 headless 測試。
- 執行 netem custom 20-bin、ordered-delay、natural-reorder 與 profile control 測試。
- 執行視野隔離、Reveal／Hide／Forget、移動、rebase 與 evidence comparison。
- 執行受影響 Cargo workspaces 的 regression tests 與 `git diff --check`。

## 完成條件

- 四個根目錄薄 `.bat` 之外，現行專案工作流不再依賴 PowerShell、Python、shell 或額外批次腳本。
- 所有 Lua 腳本固定由 `D:\code\omoba\tools\lua\lua.exe` 執行。
- 原 CLI、環境變數、產物與 exit code 相容，文件可由一般開發者直接照做。
- 完整測試全部通過；任何平台限制均以明確錯誤呈現，不得靜默降級。
