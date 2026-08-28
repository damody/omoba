## Context

目前受版控工作流分散在 7 個 Batch、11 個 PowerShell、11 個 Python 與 1 個 shell 腳本。它們共同處理 Cargo freshness、artifact staging、跨程序啟停、UDP control、JSON evidence、角色工具與測試，但引用、錯誤語意及平台假設不一致。專案已提供 Lua 5.4.8：`D:\code\omoba\tools\lua\lua.exe`，且根目錄規範要求保留四個 `.bat` 入口。

## Goals / Non-Goals

**Goals:**

- 所有現行腳本邏輯統一由固定 Lua runtime 執行。
- 保留四個根目錄 launcher 的雙擊體驗、參數、環境變數與 exit code。
- 以小型、可測試的 Lua 模組統一 JSON、process、path、hash、UDP、time 與 evidence 行為。
- 移除已被取代的 PowerShell、Python、shell 與額外 Batch 腳本。
- 保持 server、兩個 team runtime、netem、renderer、角色 pipeline 與 build workflow 行為相容。

**Non-Goals:**

- 不改 gameplay、lockstep、fog、netem profile 或 evidence verdict 語意。
- 不把 Python-based AI／Blender／ComfyUI 本身重寫成 Lua；只把 repository 自有 orchestration script 改成 Lua。
- 不建立一般用途 package manager 或完整作業系統抽象層。
- 不改寫純歷史文件中的舊指令，除非該文件仍被當成現行操作入口。

## Decisions

### 保留四個薄 Batch wrapper

`run.bat`、`run_10000.bat`、`run_2player.bat`、`run_ue.bat` 只定位 repository、呼叫固定 `lua.exe`、原樣轉送 `%*` 並 `exit /b` 回傳結果。這符合既有 Windows 雙擊入口及 `AGENTS.md`，同時確保實際分支都在 Lua。完全移除 Batch 會破壞既有操作方式；繼續讓 Batch 承擔流程則無法真正統一。

### 共用模組保持小而明確

`tools/lua/lib/` 分成 args、path、process、json、udp、hash、time、evidence。每個模組只暴露腳本實際需要的 API。大型單檔雖較快移植，但難以讓 5.6 Terra 安全修改，也容易使 quoting 或 cleanup 修正影響無關工作流。

### 缺少的原生能力使用 Rust helper

Lua 5.4 標準庫沒有 UDP、背景程序 handle 與 SHA-256。可由一個小型 Rust helper 提供具版本的 JSON stdin/stdout protocol；Lua 負責 orchestration 與資料契約。不得退回 PowerShell 或 Python，因為那會保留本次要消除的 runtime 依賴。能安全使用 `cmd.exe` 內建命令的簡單操作不必經 helper。

### 以行為契約逐支遷移

每支舊腳本先記錄參數、環境、輸出、exit code、寫入目標、子程序與清理順序，再實作同名用途 Lua replacement。呼叫端更新完成並有對應 smoke 後才移除舊檔。這比機械翻譯語法更能避免 PowerShell pipeline、Python JSON 或 shell quoting 的隱性行為遺失。

### 測試集中在最後

實作期間只做必要的靜態閱讀與檔案存在性確認；所有 Lua syntax、module、launcher、process、netem、fog、character tooling 與 Cargo regression 測試集中到最後階段。若最終測試發現問題，修正後只重跑受影響測試，再跑完整 gate。

## Risks / Trade-offs

- [Lua 標準庫缺乏平台 API] → 使用窄介面的 Rust helper，protocol 版本化並對錯誤 fail-closed。
- [Windows quoting 改變命令語意] → 集中在 `process.lua`，加入空白、引號、Unicode 與 metacharacter 測試。
- [程序清理誤殺其他 session] → PID 與 executable path 雙重驗證，禁止全域同名終止。
- [一次遷移 30 支腳本範圍大] → 依依賴層級遷移，tasks 拆成單一檔案或單一小責任。
- [Python 工具包含第三方 Python runtime 操作] → Lua 只取代 orchestration，仍允許明確啟動外部 venv Python、Blender 或 ComfyUI。
- [歷史文件大量出現舊名稱] → 只更新現行入口與機器可執行引用，保留 archive 的歷史真實性。

## Migration Plan

1. 建立 Lua module loader、共用模組與 Rust helper protocol。
2. 轉換低依賴 generator、validator、selective-lockstep 工具。
3. 轉換 process、memory、screenshot、manifest 與 evidence helpers。
4. 轉換 netem scenario、matrix、control 與 lifecycle。
5. 轉換 launcher 主流程，再把四個 `.bat` 縮成 wrapper。
6. 更新現行呼叫端與文件，移除舊腳本。
7. 最後集中執行完整測試與工作樹檢查。

Rollback 可還原本 change 的 Lua、helper、wrapper 與呼叫端變更；不得回退 gameplay 或先前 selective-lockstep 實作。

## Open Questions

無。平台能力不足時固定選擇 Rust helper；現行文件與歷史 archive 的邊界依本設計處理。
