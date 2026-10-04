# Unreal renderer 明確消費回報

## 本輪計畫與決定

續做 OpenSpec build-unreal-rust-moba-framework／4.3。先實作 lease → 原始 IPC snapshot 身分、跨 socket 隔離與測試，再 tests → build/stage → Editor/PIE → 雙 client 驗收。整體仍為 15/30；正式 cue ID、所有 effects/audio 投影與跨 renderer 一次性效果契約尚未封關，4.3 不勾選。

- 問題：本機 sequence 與 envelope.sequence 不同。決定：Rust 內部保存 (connection ID, snapshot sequence)，不改 C ABI v3 結構；新增 export 並由 UE 載入時強制檢查。
- 問題：收到／釋放 frame 不等於完成呈現。決定：WorldBridge 在 ProcessFrame 返回後呼叫 MarkFrameConsumed，再 ReleaseFrame；此契約表示呈現程式已處理，不宣稱 GPU 已完成 rasterization。
- 問題：舊 lease 延遲到新 socket。決定：每次連線取得不重用代次，writer 丟棄其他代次；每輪最多 256 ACK，合併最高序號；同一 slot mark 冪等。
- 問題：控制 frame 或 lifecycle 不是完整新 snapshot。決定：它們沒有 ACK token；不能用控制序號抬高 runtime cursor。
- 問題：驗收只證明畫面與移動，未證明 runtime 接收 ACK。決定：runtime 每連線首次有效消費留下 player/team/sequence 日誌，dual smoke 要求兩隊證據並寫進報告；每 frame 不新增日誌。

## 已驗證

- Bridge 37 unit＋2 TD integration passed，1 外部 KCP ignored。
- Runtime 33 library＋2 binary passed。
- 真實 TCP／C ABI：原始 IPC 97、不處理即 release 無 ACK、錯序號／released lease 拒絕、duplicate/control 不重送。
- 真實 TCP 重連：上一條 socket token 999 被丟棄，新 socket 正確只回報自身 snapshot 1。
- Busy ring retry／snapshot coalescing 保留 token；Lua stage gate 3 情境與 observation 6 情境通過；root／omfue diff whitespace 通過。

## Unreal 驗收

完整 build session 49694 exit 0，OmGameEditor 14 actions Succeeded；Editor PID 89572、MCP HTTP 30000 ready。header 含 om_mark_frame_consumed；built／staged DLL SHA-256 = 5ec882ff08ada09fd2f05987e59b3279ebfc6d33d96ff06511d3b2d177d52ee4，所有 bridge tests 都在 stage 前執行。

- Session 1562：同一 Editor 兩輪各 7/7 passed、0 failed；RememberedGhost fixture 仍有既有 transient world DestroyActor warning，不把它隱藏成 warning-free。
- Session 99441：PIE success、native_mesh_rendered=true、remembered_ghost_rendered=true，測試自有 PIE 已 stop。
- Session 80235：真實 5-process dual run interactive-ue-1791016176 exit 0，報告 success=true；兩隊 own_only_observed=true，UE 與 replica 均位移。
- Team 1 首次消費 snapshot 1090、15 個 observation frames；replica tick 7000，safe tick 7001。
- Team 2 首次消費 snapshot 1772、16 個 observation frames；replica tick 7046，safe tick 7047。
- runtime 首次 ACK 日誌證明回報抵達外部 runtime，UE consumption failed／runtime unsent snapshot error 搜尋無結果。launcher cleanup 完成，active-ue-session.json 不存在。
- Lua module tests passed；codegen --check 11 files／13 inputs、content hash de9c7fcfc98d6479。

原始證據：target/interactive-runs/interactive-ue-1791016176/unreal-ipc-smoke-report.json、該 run logs；omfue/Saved/McpAutomation/NativeVisual/report.json、pie-smoke-report.json。這些是本機驗收產物，不提交建置／log 暫存。

錯誤與防重犯規則見 unreal-moba-error-register.md／E042–E045。Consumed 目前仍不負責刪除所有效果；正式 snapshot builder 的 effects/audio 尚未完整映射、角色專屬 C++ 遷移仍待後續。下一段先建立正式 cue 身分與事件分類（一次性 vs 持續狀態），再接入通用 dispatch，避免套用 projectile key 吞掉合法 buff/script 更新。
