# Unreal 双陣營 IPC adapter 驗收（2026-10-03）

## 計畫

1. 核對独立 player/team 身分與 Rust → C ABI → UE 設定。
2. 使用真實 server、兩個外部 runtime、兩個 Unreal -game client 驗證安全投影與移動；不是 runtime 自動注入輸入。
3. 修正實測問題，保存失敗 evidence 與防錯紀錄，再跑 Rust、完整 UE build、Editor repeat 與 PIE regression。
4. 僅依真實驗收勾選 OpenSpec 4.2；整體 MOBA、LAN、完整重連與 UI 仍另有待辦。

## 已做決定

- C ABI 升至 v3，新增 explicit `team_id`。player 7/team 2 真實 TCP handshake 測試證明不再以玩家 ID 猜隊伍；舊 ABI／IPC 零身分拒絕。network protocol v2 與 presentation IPC v3 是不同版本，不混改。
- launcher 非 skip-build 一律增量建 server/runtime；UE 明確傳隊伍，native registry 路徑不必使用舊 creep Blueprint。既有 Blueprint 與 Saika 相容 handlers 未刪除。
- bridge 一次最多取得 256 driver updates，只投影最後完整安全 snapshot；removal 與終局 input results 保留，reset 淘汰舊 view。
- fog 平面改為已載入 Cube 壓薄，避免第一個遊戲 frame 額外載入 Plane。
- smoke 以兩邊 UE 發送紀錄及 safe replica 真實位移做 bounded condition polling；接上 IPC、LoadMap、活著或 FORWARDED 都不是成功。
- demo fog cache 僅在同隊伍、完全相同安全英雄中心座標時命中；1 raw unit 位移亦失效，空 view／隊伍變更不沿用舊 fog。僅快取目前靜態 demo occluder 計算，未来動態遮蔽物必須加入 cache identity。

## 失敗證據與解讀

所有 runs 位於 `target/interactive-runs/`，保留 stdout、runtime evidence 與 `unreal-ipc-smoke-report.json`。

| run | 結果／修正 |
| --- | --- |
| interactive-ue-1791012372 | 缺少預期 abslog；改讀 host 必定捕獲 stdout |
| interactive-ue-1791012553 | 25 秒不足以覆蓋相機定位後 +28 秒 demo 移動；並發現無界 snapshot 重工 |
| interactive-ue-1791012933 | 50 秒失敗；第一個 frame 等 Plane 準備，改重用 Cube |
| interactive-ue-1791013302 | Plane 等待消失，仍沒有在 50 秒內發送移動；改條件等待 |
| interactive-ue-1791013521 | 两邊 UE 已發送，120 秒仍無位移；server late-input 原 tick 3138/current 5769/late 2632，修正靜止英雄 fog 重算 |

`replica lag=0` 只比較已接收到的 tick，不代表追上 server；佇列深度 1024 和 server late-input 是更直接證據。未降低權威輸入期限來掩蓋落後，沒有更改共享 Unreal 引擎。

## 驗證狀態

- bridge：36 unit passed、2 TD integration passed、1 需要外部 KCP 的 integration ignored。
- runtime：31 library + 2 binary unit passed，含 fog cache 與未快取結果相等、exact position/team/reset 失效。
- full UE build session 52197 exit 0，Editor MCP ready。
- fog cache 後真實 run `interactive-ue-1791013831` exit 0，兩隊各 15 observed frame、own-only view；UE 發出的 move 在 safe replica 改變座標。沒有使用 `--scripted-move-tick` 注入。
- 保存日誌加強復驗亦 PASS：隊伍 1 UE (-1320,-1100) → (-1313,-1098)，隊伍 2 UE (1320,1100) → (1317,1099)。6 個純解析測試通過，額外 observation report 不覆寫原 live report。launcher 也已加入這項必要斷言與 partial JSON retry。
- full build 80262 曾 exit 1：bridge stage 成功，共享 Unreal Renderer DLL 被另一專案鎖住（E037）。外部程序自行結束後 full build **3355 exit 0**，Editor PID 52212、MCP ready；沒有停止別的專案。
- 加強版 live run **1791014362 exit 0**，兩隊各 15 frame、own-only view。隊伍 1 UE (-1320,-1100) → (-1317,-1099)，隊伍 2 UE (1320,1100) → (1313,1098)；兩邊 safe replica 也改變座標。所有五個 child processes 自行清理，active session 登記已移除。
- session **90909 exit 0**：同一 Editor 兩輪各 7/7，failed/skipped/not_run=0；PIE native_mesh_rendered=true、remembered_ghost_rendered=true、memory counts=[1,0]，自行啟動的 PIE 已停止。
- Lua module tests、6 個 observation scenarios、生成器 `--check`（11 files／13 Lua inputs）、diff whitespace check 通過。
- OpenSpec 4.2 完成，總計 15/30。4.1、4.3、4.4、MOBA 規則及完整 UI／LAN 仍未完成。

## 重現指令

固定 Lua 入口：`tools/lua/lua.exe scripts/run_2player_ue.lua`。
smoke 環境：`OMOBA_RELEASE=0`、`OMOBA_UE_SMOKE_SECONDS=120`；只有確認 UE build 成功且沒有 C++ 變更時才設 `OMOBA_SKIP_UE_BUILD=1`。
保存真實 evidence 的加強復驗：`tools/lua/lua.exe scripts/tests/ue_two_team_observation_test.lua target/interactive-runs/interactive-ue-1791013831`。

外部鎖定核對：PID 20476 屬於 `C:/portable/OpenKoikatsu/OpenKoikatsu.uproject` 的 CrowdRuntimeStable benchmark，不是 omoba child process，未停止／修改。其自行結束後已完成 full build → 新 launcher 移動 smoke → Editor repeat → PIE regression。

## 下一階段

依 OpenSpec 未完成清單繼續 4.1 的正式 MOBA 輸入／投影、4.3 跨 renderer instance 的一次性 cue 契約與 4.4 舊玩法路徑隔離。当前 run_ue 仍是 TD 相容入口，雙 Unreal launcher 已使用外部 runtime；本輪不把整個前端預設模式遷移冒充完成。通用英雄事件、MOBA 規則、完整 UI、LAN、動態 fog occluder 與效能門檻仍需實作驗收。

完整歷史與預防規則见 `unreal-moba-error-register.md` E028–E037。
