## 1. Schema 與 shared data model

- [x] 1.1 擴充 `proto/game.proto`，新增 `AttackMove`、`SetTowerTargetPriority` input action，並為 `MoveTo`、`AttackMove`、`AttackTarget` 加入 queued/append flag，保留既有欄位編號相容性
- [x] 1.2 重新產生或更新 `omoba-core` / `omfx` 使用的 prost generated code，確認 `PlayerInput` enum 可編譯
- [x] 1.3 在 shared runtime data model 新增 `TowerTargetPriority` enum，支援 serde/default 與 deterministic ordering
- [x] 1.4 在 `Tower` component 新增 `target_priority` 欄位，舊資料缺欄位時使用 default priority
- [x] 1.5 在 `Creep` component 或相鄰 deterministic component 新增 path ranking data，表示離終點剩餘距離或等效 first/last rank key
- [x] 1.6 新增 hero command queue/path component 或等效資料結構，表示 `MoveTo`、`AttackMove`、`AttackTarget`、destination、target entity、queued order、completion condition、queue limit 16 與 waypoints

## 2. Lockstep input routing

- [x] 2.1 在 `player_input_tick` 新增 `AttackMove` arm，log player id、tick、raw target position 與 queued flag，並推入 pending hero command queue
- [x] 2.2 將既有 `AttackTarget` arm 從 log-only 行為改為推入 pending hero command queue 或呼叫 shared `GameProcessor` entry point，保留 queued flag
- [x] 2.3 新增 `SetTowerTargetPriority` arm，log tower id 與 priority，並推入 pending tower command queue
- [x] 2.4 實作 pending queue drain entry points，驗證 player hero、target entity、tower ownership、priority 合法性與 hero command queue 上限 16
- [x] 2.5 確保所有新 input rejection 只 warning log，不 panic、不 disconnect、不送 bespoke ack

## 3. 英雄移動、尋路與攻擊命令

- [x] 3.1 定義 hero command decision result / outcome 資料結構，讓 hero command tick 的搜尋、排序與決策階段保持 read-only
- [x] 3.2 實作 deterministic path planner，使用 fixed/grid keys 與明確 tie-break，避開 `BlockedRegions` 與不可穿越 collision
- [x] 3.3 將 `MoveTo` / `AttackMove` / `AttackTarget` drain 改為建立 hero command；queued=false 時覆蓋目前 command 並清空 queue，queued=true 時 append queue tail，append 超過 16 時拒絕
- [x] 3.4 實作 hero command queue advancement，依 `MoveTo` 抵達、`AttackMove` 抵達、`AttackTarget` 目標死亡/失效/不可追擊等 completion condition 推進下一個命令，並處理 queued command 執行前失效時 skip/repath
- [x] 3.5 更新 `hero_move_tick` 或新增 hero command tick，使英雄跟隨 waypoint，卡住時可 deterministic 重算或拒絕
- [x] 3.6 實作 `AttackMove` acquisition：移動中依有效攻擊範圍尋找合法敵人，目標失效後等待 backswing 完成再回到原目的地，決策階段只輸出 command result/outcome
- [x] 3.7 實作 `AttackTarget` 追擊：目標在範圍外時尋路靠近，進入範圍後發起普攻，追擊偏移超過 effective attack range 的 0.5 倍時放棄，mutation 由集中 apply phase 套用
- [x] 3.8 串接既有 attack windup cancellation，讓 accepted non-append gameplay input 在 impact 前取消攻擊並清空 queue，但 queued append command 與 attack-move 自動續走/重新索敵不得取消 windup 或 backswing
- [x] 3.9 實作所有 accepted non-append gameplay inputs 清空 hero command queue，包含 ability、item、tower action 與 start round；`NoOp` 與純 UI selection 不清 queue

## 4. 塔 target priority

- [x] 4.1 實作 tower priority sorting helper，支援 `first`、`last`、`nearest`、`farthest`、`highest-health`、`lowest-health`，新塔與舊資料缺欄位 default 為 `first`
- [x] 4.2 在 creep/path movement tick 更新 creep path ranking data，確保 slow、stun、checkpoint 推進與多路徑情境下排名正確
- [x] 4.3 定義 first/last 讀取 creep path ranking data 與非 TD fallback，所有 tie-break 使用 entity id 升冪
- [x] 4.4 定義 tower target-selection result / outcome，讓範圍候選搜尋與 priority 排序保持 read-only
- [x] 4.5 更新 host tower target query，使 scripted towers 透過既有 query API 也遵守 Tower priority
- [x] 4.6 更新非 scripted / host-side tower attack path，確保同樣使用 priority helper 選目標，且 mutation 由集中 apply phase 套用
- [x] 4.7 實作 `SetTowerTargetPriority` 的 ownership/permission validation 與 component 更新

## 5. Snapshot 與 omfx UI/render

- [x] 5.1 擴充 `SimWorldSnapshot` tower entity data，expose `target_priority`
- [x] 5.2 擴充 `SimWorldSnapshot` hero entity data，expose command kind、destination、target entity、queued command count/summary、queue limit 16 與 next/active waypoints
- [x] 5.3 更新 omfx snapshot-backed `network_entities` / hero mirror，讀取 tower priority 與 hero command/path render data
- [x] 5.4 在 omfx input handling 實作 `A` targeting mode：合法地面左鍵送出 `AttackMove`，並依 `Shift` 狀態設定 queued flag
- [x] 5.5 在 omfx right-click handling 區分 enemy entity 與 ground：enemy 送 `AttackTarget`，ground 送 `MoveTo`，並依 `Shift` 狀態設定 queued flag
- [x] 5.6 在 tower panel 新增 target priority 控制，點擊送出 `SetTowerTargetPriority`
- [x] 5.7 固定 hit-test 優先序，確保 HUD/技能/item/tower panel 點擊不會同時送地圖命令
- [x] 5.8 依 snapshot 顯示英雄命令/path feedback 與塔目前 priority，不使用 optimistic state 作為權威顯示

## 6. Tests 與驗證

- [x] 6.1 新增 `PlayerInput` routing tests，覆蓋 `AttackMove`、`AttackTarget` 與 `SetTowerTargetPriority` 不再是 log-only
- [x] 6.2 新增 pathfinding tests，覆蓋 blocked region 繞路、不可達目的地處理與 deterministic tie-break
- [x] 6.3 新增 hero command tests，覆蓋 attack-move 途中 acquire、目標死亡後等待 backswing 再續走、自動重新索敵不取消後搖、指定攻擊非法目標拒絕、Shift queued AttackTarget→MoveTo 順序執行、queue 上限 16、queued command 執行前失效 skip/repath、非 Shift MoveTo/AttackMove/AttackTarget 覆蓋目前命令並清空 queue、accepted ability/item/tower/start-round 清空 queue
- [x] 6.4 新增 read-only decision tests 或 grep guards，確認新增 hero/tower decision hot path 不直接寫 gameplay storages，只輸出 outcome/command result
- [x] 6.5 新增 tower priority tests，逐一覆蓋 six strategies、creep path ranking data 更新、first/last 不重掃 path geometry 與 entity id tie-break
- [x] 6.6 新增 snapshot tests，確認 tower priority 與 hero command/path data expose 且 extraction 不 mutate gameplay state
- [x] 6.7 新增 omfx input/unit tests 或 smoke guards，確認 A 模式、右鍵點敵/點地與 tower panel hit-test 不互相觸發
- [x] 6.8 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab` 與 `cargo test --manifest-path scripts/Cargo.toml -p base_content`
