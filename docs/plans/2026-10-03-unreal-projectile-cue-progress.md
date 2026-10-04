# Unreal 投射物呈現去重

## 本輪計畫

1. 核對 retained FX 與 renderer frame 身分，先補明確可驗證的投射物去重缺口。
2. 讓 UE dispatch 使用有界歷史，不讓同一事件在多個新 frame 重播。
3. 測試事件世代／instance／tick 身分、容量與過期 replay，並測真正的 synthetic frame dispatch。
4. 完整建置、同一 Editor 兩輪測試、更新診斷與防錯紀錄。

## 設計決定

- `FOmPresentationCueHistory` 是呈現工具，不持有玩法 world，不計算傷害。
- 原生事件鍵排除 frame sequence，保留 source generation；相同 tick 的不同 instance 不互相吞掉。
- 找不到 actor 時不消費，讓仍保留的事件可在 source 進入呈現後播放。
- 4096 ticks 的歷史與 16384 key 容量皆有上限。過期 key 即使已逐出，仍由高水位拒絕，避免老事件重新播放。
- 超量拒絕新增 cue，diagnostics 有 `bProjectileCueHistoryOverflowed`；不逐出有效 key 來製造重播。
- `DispatchedProjectileCueCount` 提供真實 dispatch 診斷與 synthetic integration assertion。
- 只在明確 StopRuntime 重設；actor 回收／Hide 不重設歷史。

## 實作與驗收

已新增 `OmPresentationCueHistory.h`，接入 `AOmWorldBridgeActor::DispatchFxCues` projectile branch，擴充 `WorldBridgeSyntheticFrameSmoke` 並新增 `ProjectileCueHistory`。

`scripts/ue_native_visual_smoke.lua` 每輪由 4 項增為 6 項，且維持同一 Editor 連跑兩輪：native hero、animation、Saika compatibility、Blueprint surface、cue history、synthetic dispatch。

- `tools\lua\lua.exe scripts\build_ue_moba.lua --full --ue-root D:\UE5.8`：session 22346 exit 0，OmGameEditor 編譯、Rust bridge staging 與 MCP ready 通過，Editor PID 72232。
- `tools\lua\lua.exe scripts\ue_native_visual_smoke.lua`：同一 Editor 兩輪各 6/6 passed，failed／skipped／not_run 皆 0。
- `tools\lua\lua.exe scripts\ue_pie_smoke.lua`：native 模型實際 render 與 fallback smoke 通過，自行啟動的 PIE 已停止。
- `git -C omfue diff --check` 與 Lua syntax check 通過；既有換行與建置 dependency 警告不宣稱已消除。

報告保留於 `omfue/Saved/McpAutomation/NativeVisual/report.json`（兩輪 results）、`pie-smoke-report.json`。Saved 產物不提交；上述指令可重新產生驗收。

## 尚未完成

整體維持 14/30；4.3 不勾選。此輪完成的是相同 WorldBridge instance 內 retained projectile 的去重，不是所有技能 cue 或跨新 renderer instance 的消費契約。安全記憶仍需獨立 ghost 呈現通道、不可選取限制與畫面驗收；不能把 frozen ghost 加入 live entity 清單當作完成。
