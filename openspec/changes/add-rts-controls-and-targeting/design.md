## Context

現有 lockstep schema 已有 `MoveTo` 與 `AttackTarget`，`player_input_tick` 也有 routing scaffold；`MoveTo` 目前主要寫入 `PendingMoveQueue`，再由後續 drain 寫入 `MoveTarget`。英雄移動 tick 已有 blocked/collision 檢查，但行為仍是「直線推進、卡住就嘗試 X/Y」，還不是完整的路徑規劃。

塔的攻擊選目標目前主要來自腳本側 `query_nearest_enemy` 與 host 端附近敵人排序，玩家無法替單座塔切換 first/last/nearest/farthest/highest-health/lowest-health。因為塔腳本 DLL 與 host 使用 `abi_stable` FFI，新增目標策略時要避免把複雜 Rust 型別放進 `script-abi`，並維持 scripts workspace 與 omb workspace 同 rustc/toolchain 建置。

## Goals / Non-Goals

**Goals:**

- 讓英雄支援 RTS/MOBA 常用操作：右鍵移動、右鍵指定攻擊、`A` 點地 attack-move。
- 讓英雄移動指令使用後端權威尋路，避開 `BlockedRegions`、塔/建築碰撞與其他不可穿越碰撞體。
- 讓塔可由玩家設定單座 target priority，並由後端 authoritative 套用於攻擊選目標。
- 所有新 gameplay input 都走 `PlayerInput` lockstep pipeline，失敗以 log/rejection 呈現，不送 bespoke ack。
- 透過 snapshot expose 目前命令/路徑與塔策略，使 omfx UI 與 render feedback 不需自行推測權威狀態。

**Non-Goals:**

- 不在本變更重做整個單位 AI 或多選編隊系統。
- 不新增 fog-of-war、視野限制或敵我陣營規則的完整重構；合法敵人判斷沿用現有 faction/team/hostile 查詢邏輯。
- 不要求塔腳本自帶不同策略 UI；UI 由 omfx 選中塔面板提供，策略值由 host component/snapshot 管理。
- 不要求第一版尋路達到大型 RTS navmesh 品質；先提供 deterministic、可測、能避開現有 blocked/collision 的路徑產生與跟隨。

## Decisions

1. **新增 `HeroCommandQueue`/`HeroPath` component，而不是把所有狀態塞進 `MoveTarget`。**

   `MoveTarget` 適合單點直線移動，但 attack-move、指定攻擊與 Shift queued commands 需要保存 command kind、目標 entity、目的地、目前 waypoint、acquire cooldown、完成條件、queue order 與回退行為。新增 component 可避免讓 `MoveTarget` 承擔過多語意；`MoveTarget` 可保留作為低階「下一個 waypoint」或相容層。

   Alternatives considered：直接擴充 `MoveTarget`。這會讓既有 summon/hero move tick 混入攻擊語意，也較難 snapshot 命令狀態。

2. **尋路在 sim 端 deterministic 計算，前端只送 target point/entity。**

   omfx 的 hit-test 只負責判斷玩家點到地面或 entity，並送出 lockstep input。omb/omoba-core local replica 在相同 tick 對相同 world state 做 path planning，確保後端與前端 lockstep sim 不分歧。路徑計算使用固定點座標或固定排序規則，避免浮點 tie-break 造成不一致。

   Alternatives considered：前端先算路徑再送 waypoints。這會讓 wire payload 變大，也讓 client pathfinding bug 直接變成 gameplay state。

3. **ECS tick 決策階段保持唯讀，只輸出 outcome/command results。**

   本變更新增或修改的 hot-path ECS tick，包括 hero command decision、path planning/following decision、attack-move acquisition、tower target priority selection 與 target query，SHALL 盡量採 read storage/resource 輸入並產生 `Outcome`、pending command result 或等效集中套用資料。實際 component/resource 寫入、entity lifecycle、attack state transition、damage/projectile spawn 與 priority/component 更新，SHALL 在既有 outcome processing 或明確的 serial apply phase 中集中執行。這個邊界讓候選搜尋、排序與決策能平行化，並降低 specs borrow conflict。

   Tick pipeline SHALL be explicit:

   `PlayerInput drain -> read-only decision pass -> deterministic result sort -> serial outcome/apply phase -> snapshot extraction`

   Snapshot extraction 只讀取已套用後的 deterministic state，並遵守既有 render-only queue drain invariant。

   Alternatives considered：在每個 tick system 中直接 write storage 套用結果。這較容易完成小功能，但會把查詢、排序與 mutation 綁在一起，阻礙 `ParJoin`/平行 query，也讓 deterministic ordering 與測試邊界更難維持。

4. **Attack-move、指定攻擊與 Shift queue 使用同一套 hero command system。**

   `AttackMove { destination }` 在移動中週期性掃描攻擊範圍內合法敵人；找到敵人時發起普攻，該次攻擊進入 backswing 後，attack-move 的自動續走或重新索敵不得中斷 backswing lockout。目標失效、離開條件或 backswing 完成後，英雄再繼續原目的地。`AttackTarget { target_id }` 優先追擊指定 entity，若目標死亡、不可見/非法或無法解析，指令被拒絕或清除並保留目前位置。

   Player input payload SHALL carry a queue modifier, e.g. `queued: bool` or equivalent, for `MoveTo`、`AttackMove` 與 `AttackTarget`。`queued=true` 代表將命令 append 到 queue tail，不中斷目前正在執行的命令。Hero command queue maximum length SHALL be 16 commands, including active command and queued tail；append 超過上限 SHALL be rejected and logged。

   Any accepted non-append gameplay action SHALL clear the hero command queue before applying that action. This includes non-Shift move/attack inputs and other accepted gameplay actions such as ability, item, tower action, or start-round inputs. Pure UI selection or panel hover state is not gameplay input and does not clear queue by itself.

   Queue advancement SHALL happen only when current command reaches its completion condition: `MoveTo`/`AttackMove` 到達目的地，`AttackTarget` 的指定 target 死亡、失效或被判定不可繼續追擊。Queued command append SHALL NOT cancel current windup or backswing；只有 non-append replace gameplay action 可依既有 attack cancellation contract 取消 windup。例：按住 `Shift` 右鍵點怪物 A，再右鍵點地面 B，會先執行 `AttackTarget(A)`，A 死亡或失效後再執行 `MoveTo(B)`。

   `AttackTarget` 的追擊沒有時間上限，但有距離上限：若英雄為追擊指定目標造成的偏移超過目前有效攻擊距離的 0.5 倍，該 `AttackTarget` command SHALL be abandoned and the queue advances。實作可用 command 開始追擊時的 hero position 或等效 deterministic leash anchor 計算偏移。

   Alternatives considered：把 attack-move 寫在 input handler 中即時轉成 AttackTarget。這會讓 acquire 時機與後續移動狀態不清楚，也無法自然處理途中遇敵。

5. **塔 target priority 存在 `Tower` component，scripts 透過 host query 取得已排序結果。**

   新增 `TowerTargetPriority` enum，以 serde-friendly 小型值儲存在 `Tower`。host 查詢候選敵人時依 priority deterministic 排序，再提供給 tower attack path。若既有 tower script 呼叫 `query_nearest_enemy`，可先把該 API 的排序行為改成「依 caller tower 的 priority 回傳最佳敵人」，並保留函式名稱相容；若未來需要，另加 `query_enemy_by_priority`。

   Alternatives considered：讓每個 tower script 自行排序。這會重複邏輯、增加 DLL API 面積，也讓玩家設定與腳本實作容易不一致。

6. **first/last 讀取 creep 預先更新的終點距離排名，其他策略以即時屬性定義。**

   Creep movement/path tick SHALL 在 creep component 或相鄰 component 上維護 deterministic 的 path ordering data，例如「距離終點剩餘長度」與可比較的 rank key。`first` 選擇 remaining distance 最小或 rank 最前的敵人；`last` 選擇 remaining distance 最大或 rank 最後的敵人。塔 target query 只需要對範圍內候選讀取這個已更新欄位，不得在每座塔選目標時重新掃描整條 path geometry 來計算進度。`nearest/farthest` 以 tower 到候選敵人的距離排序，`highest-health/lowest-health` 以目前 HP 排序。所有 tie-break SHALL 使用 entity id 升冪，避免不同 storage iteration order 影響結果。

   Alternatives considered：在塔查詢時即時計算每個 creep 的 path progress，或用 spawn 時間當 first/last。即時計算會讓 1000 塔 × 1000 creep 場景重複做路徑幾何工作；spawn 時間不等於離終點遠近，遇到 slow、stun 或多路徑時會錯。將終點距離/rank 寫在 creep state 上，能讓塔在拿到範圍候選後快速比較，也符合 TD 玩家對 first/last 的直覺。

7. **snapshot expose tower priority 與 hero command/path render state。**

   `EntityRenderData` 對 tower 增加 `target_priority`，對 hero 增加 optional `command_ext` 或等效欄位，包含 command kind、目標 entity、destination 與 active waypoints。omfx selection panel 與 command feedback 只讀 snapshot，不做 optimistic 狀態覆寫。

## Risks / Trade-offs

- **[Risk] deterministic pathfinding 使用 f32 或 unordered collection 會造成前後端分歧** → 尋路核心使用 fixed-point/grid key 與明確排序；測試包含同 seed pin-hash 與同輸入雙世界比較。
- **[Risk] 尋路每 tick 重算造成 TD_STRESS 負擔** → 只在新命令、blocked topology 變更或 waypoint 失敗時重算；每 tick 只跟隨已算出的 waypoint。
- **[Risk] tick 決策階段直接 mutation，導致 borrow conflict 並阻礙平行化** → 新增的搜尋/排序/決策系統保持唯讀，輸出 `Outcome` 或 command result；集中 apply phase 才寫 storage/resource，pipeline 固定為 input drain、read-only decision、deterministic sort、serial apply、snapshot extraction。
- **[Risk] scripted tower 仍直接呼叫 nearest API，導致策略看似無效** → host query API 先在相容函式內套用 tower component priority，並新增測試覆蓋 `query_nearest_enemy` 在不同 priority 下的結果。
- **[Risk] first/last 排名欄位 stale，導致塔選錯目標** → creep/path movement tick SHALL 在 creep 位置或 path index 更新的同 tick 更新 remaining-distance/rank；塔選目標只讀該欄位，並以測試覆蓋 slow/stun/多路徑情境。
- **[Risk] first/last 在非 TD 或沒有終點距離排名的敵人上不明確** → 對沒有 ranking data 的候選，first/last 使用 deterministic fallback：以距離與 entity id 排序，並在 spec 明確定義。
- **[Risk] attack-move 自動續走讓英雄取消後搖，造成攻擊節奏異常** → hero command tick SHALL 區分玩家新輸入與 attack-move 自動狀態轉移；自動續走/重新索敵等待 backswing 完成後才執行。
- **[Risk] Shift queued commands 與即時取代命令語意混淆** → input schema SHALL 明確攜帶 queue modifier；`queued=true` append 且 queue 上限 16；任一 accepted non-append gameplay action 清除 queue；snapshot expose queue head/tail 摘要供 UI 驗證。
- **[Risk] UI 點擊優先序互相干擾** → omfx hit-test 順序固定為 HUD/面板控制 > 技能/物品控制 > entity hit > ground command，新增測試或 guard 避免塔策略選單點擊落到地圖命令。

## Migration Plan

1. 擴充 schema 與 shared enums：加入 `AttackMove`、`SetTowerTargetPriority`、hero command queue modifier 與 snapshot fields；保留既有 `MoveTo`、`AttackTarget` 欄位編號與語意。
2. 在 omoba-core/omb 實作 pending queue、drain entry points、hero command queue/path component 與塔 priority component default。
3. 讓現有 tower spawn path 將 priority 預設為 `first`；舊存檔/deserialize 缺欄位時使用 `first`。
4. 更新 tower target query 與 scripted tower host adapter，使既有 tower scripts 不需一次性重寫。
5. 更新 omfx input 與 UI：A 點地、右鍵點敵/點地、塔策略 segmented/menu 控制。
6. 加入單元、lockstep、snapshot 與 UI smoke 測試；確認 `cargo test --manifest-path omb/Cargo.toml -p omobab` 與相關 scripts test 可通過。
