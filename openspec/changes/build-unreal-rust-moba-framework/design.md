## Context

完整架構與階段驗收見 `docs/superpowers/specs/2026-09-25-unreal-rust-moba-framework-design.md`。目前 `omoba-client-runtime` 已有 filtered replica 與 localhost protobuf IPC，但呈現包含示範用資料；`omfue/bridge` 仍有自己的玩法 world 與直連路徑。`omfue/codegen` 可從 Lua 產生 C++，但包含 Saika 專屬分支。`BpGeneratorUltimate` 已在專案中提供 Editor MCP 通道與資產、Blueprint、PIE 工具。工作樹目前另有未提交變更，實作須逐檔保留並整合。

## Goals / Non-Goals

**Goals:**
- 一個權威 Rust 對局核心，同時支援單機與 LAN。
- Unreal 只負責輸入與呈現，英雄內容由 Lua/Rust 與生成器驅動。
- 資產與 PIE 操作能透過 BpGeneratorUltimate 重跑與驗證。

**Non-Goals:**
- 首版不加入帳號、排位、配對與商業營運系統。
- 不把 Editor MCP 當成遊戲執行期依賴。

## Decisions

1. 採用 `omb` → `omoba-client-runtime` → localhost IPC → `omfue`。替代方案是在 `omfue/bridge` 延續獨立 world；這會複製視野、同步與重連邏輯，因此只作過渡相容路徑。
2. Lua 是靜態資料與可組合 effect 的來源；特殊行為由 `base_content.dll` 中的 Rust handler 執行。替代方案是任意 Lua 轉譯 Rust；現有架構沒有此能力，也難維持確定性。
3. 擴充現有 `omfue/codegen` 生成薄 Unreal 類別、registry 與資產配方；移除角色專屬生成分支。替代方案是逐英雄 Blueprint/C++，無法符合內容製作目標。
4. BpGeneratorUltimate 只在 Editor 啟動時套用配方與驗證 PIE；正式遊戲由 IPC 驅動。替代方案是把 MCP 放進遊戲執行流程，會造成 Editor 耦合。
5. 先修復建置與內容版本檢查，再打通 IPC，之後實作 MOBA 規則與呈現。每階段都以可執行證據封關。

## Risks / Trade-offs

- 現有未提交的 `omfue` 與啟動工具改動可能與新路徑重疊 → 先讀取 diff，避免覆蓋；優先實作獨立生成器與測試，再逐點整合。
- IPC 目前部分欄位是示範投影 → 先明確標記並加入正式 MOBA schema 與視野測試。
- 腳本 DLL 與 Rust host 需相同 rustc → 建置入口固定 toolchain，先建 DLL 再 stage，啟動時檢查版本。
- Editor MCP 可能未啟動 → 保留資產配方與待處理報告，不能將未驗證資產標成成功。

## Migration Plan

1. 建立可重跑的生成器檢查與完整建置流程，保留原 TD 啟動。
2. 為 Unreal 新增 IPC 呈現路徑並用雙隊視野測試驗證；待功能相同後隔離舊玩法 driver。
3. 將英雄與資產生成改為共用內容模型，分角色遷移，最後移除 Saika 特例。
4. 逐步交付 headless MOBA 規則、Bot、三路地圖與 Unreal UI，通過各階段驗收後作為預設 MOBA 啟動模式。

回退方式為保留既有 TD 模式與其啟動參數；新 MOBA 模式在版本或資產驗證失敗時不啟動對局。
