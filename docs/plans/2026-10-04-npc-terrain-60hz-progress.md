# NPC 公開地形避障／60Hz 進度

## 本輪計畫與決策

- [x] 從英雄 planner 抽出共用 static_next_waypoint，保持 grid64／margin8／最大96格、BTreeMap 與固定鄰居次序。
- [x] 三路 creep 追擊與回兵線、野怪追擊與回位接 static_step_toward／static_advance_route。直線可通不做 BFS；受阻才搜尋，每次實際位移另做整段 swept-circle。
- [x] 保留 map_id=None 的單路算法與 TD；不改 Lua 地形允許範圍，不新增角色 C++／Blueprint graph。
- [x] 共用 planner 60Hz 往返、route cursor、阻擋終點及零／負 budget 測試。
- [x] 正式 ECS 三 seed 各900 tick，5400雙隊 filtered steps：兵線／野怪實際繞薄牆、兵線越過、野怪越過後回位 Heal、每tick雙authority digest／canonical team hash／零repair／private input隔離。
- [x] 完整 base_content 與跨 workspace 回歸封關。
- [x] 真實 KCP 三路60Hz、Unreal build／DLL stage SHA 獨立核對。
- [x] OpenSpec／diff check，更新最後結果。

## 已確認結果

- base_content：108 passed／0 failed，204.58秒；包含三路完整 lifecycle、野區正式攻擊與既有英雄地形測試。
- core：339 passed。
- server：156 passed／1 benchmark ignored。
- client runtime：61 passed／8 opt-in ignored，integration3 passed。
- Unreal bridge：53 passed／1 opt-in ignored，integration2 passed／1 opt-in ignored。
- 新 NPC ECS 測試：seed1／42／539365380，各900tick／1800雙隊steps，全通過（22.08秒總測試耗時，不是tick wall-clock benchmark）。
- codegen --check：11 files／16 Lua inputs，content_hash=3b296ff5bdbcd7d9，未新增每角色C++或BP。
- OpenSpec strict validate 與本輪 scoped diff check 通過。

## 真實三路 KCP／Unreal 建置

執行固定 Lua 入口 `scripts/run_moba_runtime_smoke.lua`，mode=three_lane、fps=60、port base58321，run1791114001。報告位於 `target/interactive-runs/moba-runtime-1791114001/moba-runtime-smoke-report.json`，success=true／cleanup_verified=true／tick_rate_hz=60。兩隊 MoveTo 已執行，報告各9 checkpoints、最後1080。

獨立讀完整保存 three-way-checkpoints.jsonl：team1 PASS14行／9 unique ticks，team2 PASS11行／9 unique ticks，皆最後1080、pre/post parity全部true、零FAIL。UNVERIFIED2232／2171行不是通過證據，不宣稱網路每tick全部hash驗證。server53752／runtime25768／37432正常清理，另以CIM exact PID查詢皆不存在。報告SHA-256：

`3d0bf5885bdb9490da29f7671b0b69d7b0ca75c718f7ef4e73d78bae39dc40d5`

最後 `scripts/build_ue_moba.lua --build-only`：OmGameEditor Win64 Development Result:Succeeded；`--verify-staged-only` 另跑通過，沒有開Editor／PIE。獨立讀檔SHA確認：

- bridge build／UE staged：`d6f3e9171a0749e9c4fe1dd615c9a138ccbc788cb7cd4571e01c665f58338ba6`
- script debug build／scripts根目錄／UE staged三份：`4305bcd7eb1fdabedd9d248c7ff340be190fca23e28dd91b9a1a9d7079f52c3b`

KCP驗證的是目前合法Lua三路地圖、普通玩家移動／filtered network流程；**不是**新障礙fixture現場跨牆／野怪受擊測試。未改omb/game.toml、未更新legacy omb/scripts release DLL，沒有本輪release headless宣稱。

## 問題與修正

最初 route wrapper 只看當前微小 proposed step，繞障後又回牆邊，x=55.78未抵達。改先檢查完整當前 waypoint 路徑，受阻就繼續共用 planner；必須驗證抵達，不只驗證不穿牆。記於 E130。

## 邊界與未完成

- NPC 現階段使用20-unit靜態導航 envelope，只查公開 polygon，未新增動態單位碰撞／navmesh／完整可达性證明。
- 攻擊命中／視野遮蔽仍沿既有規則，本輪沒有新增牆阻擋普攻或把collision polygon自動當vision occluder。
- 新測試以明確 fixture 放置英雄、初始正常 Outcome 正傷害建立野怪仇恨，再用正式 MoveTo 撤退。牆置於兵線／camp leash 內，**不是** Lua 生成器允許這種 map 的證據，也不是網路玩家首次攻擊測試。
- filtered ECS 不取得 MobaMatch／aggro／route cursor；NPC 沿既有 PreStepMovement 和正常 committed vitals 投影。
- Lua 仍禁止地形擋住兵線100-unit corridor、五人出生與 camp leash＋100；不把單組fixture當作所有複雜迷宮均可達。
- 英雄既有同格／超跨度拒絕與 closest-partial 搜尋行為不變；不可達 NPC 停留而不穿牆，尚未引入卡住timeout或任意地形接受。
- 仍缺 Unreal 三路／野區／地形 layout、完整建築層次、五位置Bot／100場批次與整體UI驗收。OpenSpec5.4不勾選；60Hz deterministic測試不代表畫面60FPS。
