## ADDED Requirements

### Requirement: 塔選目標決策保持唯讀並輸出 outcome
塔 target priority 的候選搜尋、ranking 讀取與目標選擇階段 SHALL 保持 read-only，讀取 Tower、Creep、Position、HP、path ranking data、攻擊範圍與 faction/hostility state 後，輸出 deterministic target-selection result、`Outcome` 或等效集中套用資料。該階段 MUST NOT 直接改寫 Tower attack state、projectile state、damage state、entity lifecycle 或其他 gameplay storages。

實際 attack windup start、projectile spawn、damage application、tower facing update 或其他 mutation SHALL 由既有 outcome processing 或明確 serial apply phase 執行，並依 deterministic ordering 套用。

#### Scenario: tower priority query is read-only
- **WHEN** tower target priority system 對範圍內候選套用 `first`、`last`、`nearest`、`farthest`、`highest-health` 或 `lowest-health`
- **THEN** candidate query 與排序只讀取 gameplay state
- **AND** 選出的目標以 deterministic result 或 `Outcome` 輸出
- **AND** 不在 query/sort hot path 直接寫入 Tower、TAttack、Projectile 或 CProperty storage

#### Scenario: tower decisions can be applied in fixed order
- **WHEN** 多座 tower 在同一 tick 平行選目標
- **THEN** 每個 target-selection result 具有 deterministic apply key
- **AND** serial apply phase 依固定順序套用 attack/facing/outcome mutation

### Requirement: 塔具有可持久化的 target priority
每座 Tower entity SHALL 具有 target priority 狀態。合法值 SHALL 包含 `first`、`last`、`nearest`、`farthest`、`highest-health`、`lowest-health`。新建塔 SHALL 使用 `first` 作為 deterministic default priority；若舊資料或反序列化資料缺少該欄位，SHALL 使用 `first`。

#### Scenario: 新塔有預設策略
- **WHEN** 玩家放置一座新塔
- **THEN** 該 Tower component 的 target priority 為 `first`
- **AND** 下一個 snapshot expose 相同 priority 給 omfx

#### Scenario: 缺欄位資料使用 default
- **WHEN** deserialize 舊 Tower data 且缺少 target priority 欄位
- **THEN** Tower 使用 `first` priority
- **AND** sim 不 panic

### Requirement: 玩家可設定選中塔的 target priority
omfx tower panel SHALL 顯示選中塔目前的 target priority，並提供可點擊控制讓玩家切換 `first`、`last`、`nearest`、`farthest`、`highest-health`、`lowest-health`。點擊策略控制 SHALL 送出 lockstep `SetTowerTargetPriority` input，包含 tower entity id 與 priority。

omb SHALL 驗證 tower entity 存在、是 Tower、由送出玩家擁有或符合現有可操作權限，且 priority 合法。驗證成功後 SHALL 更新該 Tower 的 target priority；失敗時 SHALL log warning 且不 panic。

#### Scenario: 玩家將塔設定為血最少
- **WHEN** 玩家選中自己的一座塔並點擊 `lowest-health`
- **THEN** omfx 送出 `SetTowerTargetPriority { tower_entity_id, priority: lowest-health }`
- **AND** omb 驗證權限後更新該 Tower priority
- **AND** 下一個 snapshot 顯示該塔 priority 為 `lowest-health`

#### Scenario: 玩家不能設定別人的塔
- **WHEN** player A 對 player B 擁有的 tower 送出 `SetTowerTargetPriority`
- **THEN** omb 拒絕該 input 並 log warning
- **AND** 該 Tower priority 保持不變

### Requirement: 塔依 target priority 選擇攻擊目標
塔在攻擊選目標時 SHALL 從攻擊範圍內的合法敵人候選集合中，依自身 target priority 選擇目標。`nearest` SHALL 選擇距離塔最近的候選；`farthest` SHALL 選擇距離塔最遠的候選；`highest-health` SHALL 選擇目前 HP 最高的候選；`lowest-health` SHALL 選擇目前 HP 最低的候選。

TD creep SHALL expose deterministic path ranking data for target selection，至少包含可比較的「離終點剩餘距離」或等效 rank key。該資料 SHALL 由 creep/path movement 系統在 creep 位置、path checkpoint index 或 path 狀態改變時更新，並儲存在 creep component 或相鄰 deterministic component 上。

`first` SHALL 選擇離終點剩餘距離最小或 rank 最前的候選；`last` SHALL 選擇離終點剩餘距離最大或 rank 最後的候選。塔選目標時 SHALL 對範圍內候選直接讀取該 ranking data，不得為每座塔候選即時重掃整條 path geometry 來計算 first/last。若候選沒有 path ranking data，first/last SHALL 使用 deterministic fallback。所有策略的 tie-break SHALL 使用 entity id 升冪，確保相同 state 下結果穩定。

#### Scenario: nearest 選最近敵人
- **WHEN** 一座塔 priority 為 `nearest`，範圍內有兩個合法敵人
- **THEN** 塔選擇距離較近的敵人作為攻擊目標

#### Scenario: highest-health 選血最多敵人
- **WHEN** 一座塔 priority 為 `highest-health`，範圍內有多個合法敵人且 HP 不同
- **THEN** 塔選擇目前 HP 最高的敵人作為攻擊目標

#### Scenario: creep 更新離終點排名
- **WHEN** TD creep 沿 path 移動並更新 checkpoint progress
- **THEN** creep 的 deterministic path ranking data 也在同 tick 更新
- **AND** ranking data 可用來比較哪個 creep 離終點更近

#### Scenario: first 使用離終點排名
- **WHEN** 一座塔 priority 為 `first`，範圍內有多個 TD creep 位於同一路徑不同進度
- **THEN** 塔直接讀取候選 creep 的 path ranking data
- **AND** 塔選擇離終點剩餘距離最小或 rank 最前的 creep

#### Scenario: last 使用離終點排名
- **WHEN** 一座塔 priority 為 `last`，範圍內有多個 TD creep 位於同一路徑不同進度
- **THEN** 塔直接讀取候選 creep 的 path ranking data
- **AND** 塔選擇離終點剩餘距離最大或 rank 最後的 creep

#### Scenario: first last 不重掃 path geometry
- **WHEN** tower target query 已取得範圍內 creep 候選
- **THEN** `first` 與 `last` selection 使用候選 creep 上的 ranking data
- **AND** 不會對每個 tower candidate 重新掃描整條 path checkpoint geometry

#### Scenario: tie-break 使用 entity id
- **WHEN** 一座塔 priority 為 `lowest-health`，範圍內兩個敵人的 HP 相同
- **THEN** 塔選擇 entity id 較小的敵人

### Requirement: 腳本塔查詢遵守塔的 priority
既有 tower scripts 透過 host `GameWorld` query API 取得攻擊目標時，host SHALL 套用呼叫者 Tower entity 的 target priority。若 API 名稱仍為 `query_nearest_enemy`，其回傳結果 SHALL 對 Tower caller 表示「依目前 priority 選出的最佳敵人」，以維持既有 scripts 相容。

#### Scenario: Dart tower script 遵守 priority
- **WHEN** `tower_dart` 的 Tower priority 為 `farthest`
- **AND** script 呼叫既有 enemy query API
- **THEN** host 回傳範圍內距離最遠的合法敵人
- **AND** dart tower 對該敵人發起攻擊

#### Scenario: 非 tower caller 保持相容
- **WHEN** 非 Tower entity 呼叫既有 `query_nearest_enemy`
- **THEN** host 保持既有 nearest 行為或使用明確 documented fallback
- **AND** 不因缺少 Tower priority 而 panic

### Requirement: target priority 透過 snapshot expose
`SimWorldSnapshot` 的 tower-facing entity data SHALL expose 每座塔目前 target priority。omfx SHALL 從 snapshot-backed tower mirror 顯示該值，並在權威 snapshot 更新後刷新面板狀態；不得只依賴本地 optimistic selection state。

#### Scenario: tower panel 顯示權威策略
- **WHEN** 玩家選中一座 priority 為 `last` 的塔
- **THEN** omfx tower panel 顯示 `last` 為目前策略
- **AND** 顯示值來自 snapshot-backed tower mirror

#### Scenario: 策略更新後面板刷新
- **WHEN** 玩家送出策略變更 input 且後端套用成功
- **THEN** 下一個 snapshot expose 新 priority
- **AND** omfx tower panel 更新為新 priority
