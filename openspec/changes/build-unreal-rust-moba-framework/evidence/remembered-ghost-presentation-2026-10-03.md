# 原生最後位置標記驗收

## 驗收範圍

完成獨立 frozen memory 的 Rust→C ABI v2→UE ISMC 接線；不將 memory 放入 live entity、HP／技能 overlay 或可攻擊目標。完整視野與控制 frame 分開，局部 lifecycle 不清掉其他 live entity，ghost-only disconnect 發出完整 reset，frame ring 忙碌時保留最新安全 view。

## 可重現結果

- `cargo test --manifest-path omfue/bridge/Cargo.toml`：33 unit tests 與 2 TD integration tests passed；1 外部 KCP integration test ignored。
- 新 TCP 測試確實開啟 loopback socket，ghost-only 斷線／重連兩輪相同 frozen position，沒有啟動 simulation thread。
- `tools\lua\lua.exe scripts\build_ue_moba.lua --full --ue-root D:\UE5.8`：最終 session 84528 exit 0，Editor PID 91580，MCP HTTP ready 30000。
- `tools\lua\lua.exe scripts\ue_native_visual_smoke.lua`：同一 Editor 兩輪各 7/7，zero failed／skipped／not_run。新 marker test 檢查不可碰撞、不可 overlap／導航、無 live actor、正確厘米位置、無變更不重建、control 不清除、空 view 清除、reconnect restore、live Reveal 優先。
- `tools\lua\lua.exe scripts\ue_pie_smoke.lua`：17 steps PASS；native_mesh_rendered=true、remembered_ghost_rendered=true、counts=[1,0]、zero failed／could-not-tell assertions，最終 pie_running=false。fixture 存在於 OmEditor，不啟動或停止共享 runtime。
- `git diff --check`、`git -C omfue diff --check`、Lua syntax checks passed；既有 warnings 保留。

報告與 screenshots 在 `omfue/Saved/McpAutomation/`，可由上述固定 Lua 入口重建，不提交 Saved／DLL／EXE。決策與錯誤細節：`docs/plans/2026-10-03-unreal-remembered-ghost-progress.md`、`docs/plans/unreal-moba-error-register.md` E024–E027。

## 不宣稱完成的範圍

PIE 使用原生 synthetic memory fixture，不是正式兩隊 IPC 對局。未驗收跨新 renderer instance 的所有一次性 cue 消費、LAN 對局或英雄美術剪影，因此 tasks 4.2／4.3／6.1 不勾選；整體維持 14/30。
