## Why

目前三程序 selective lockstep 與戰爭迷霧只在 loopback近零延遲下完成驗證，尚未證明 secure join、team frame、input與 disclosure transition在一般 20～100 ms RTT及 delay-induced reorder下仍安全收斂。需要可重現且不解碼 gameplay payload的 UDP delay層，才能把網路延遲問題與遊戲邏輯問題分開量測。

## What Changes

- 新增 Rust `omoba-netem-proxy`，以每隊獨立 client-facing與upstream UDP socket代理 authoritative KCP流量。
- 新增20格 RTT直方圖、35%～65%非對稱上下行拆分、固定seed與可重播profile。
- 新增`ordered-delay`與`natural-reorder`兩種排程模式，以及固定packets／bytes queue budget與fail-closed watchdog。
- 擴充三程序launcher與evidence流程，記錄delay histogram、release lateness、reorder、queue high-watermark與profile timeline。
- 新增延遲環境下的secure join、replica sequence、MoveTo、hidden target、Reveal／Hide／Forget／LastKnown、sentinel與visual gate。
- 所有production與測試資產完成後，集中執行profile smoke、5分鐘矩陣及30分鐘soak。

## Capabilities

### New Capabilities

- `deterministic-rtt-delay-proxy`: 定義雙隊UDP route、20格RTT分佈、上下行拆分、排序模式、queue budget、watchdog與可重播evidence。
- `delayed-selective-lockstep-validation`: 定義20～100 ms RTT下secure session、filtered replica、input、戰爭迷霧安全與blocking verdict的端到端要求。

### Modified Capabilities

無。

## Impact

- 新增獨立Rust proxy crate與binary，但不改變authoritative server協定或信任邊界。
- `run_2player.bat`、既有PowerShell launcher、manifest、comparison與evidence目錄需要支援proxy process與delay profile。
- `omoba-core`或新crate會新增純delay model、deterministic sampler、priority queue及統計型別。
- `omoba-client-runtime`與`omfx`不取得hidden資料，也不新增client-side prediction。
- Root仍只保留既有四個`.bat`，不新增根目錄shell script。
