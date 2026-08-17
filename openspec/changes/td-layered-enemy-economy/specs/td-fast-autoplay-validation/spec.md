## ADDED Requirements

### Requirement: Reference player 只使用正式 PlayerInput

Headless `AutoplayController` SHALL 只透過正式 `PlayerInput` 提交 tower placement、upgrade、target priority、active ability、sell 與 start-round actions。它 MUST NOT 直接修改 cash、lives、enemy HP、entity existence、round index 或 cooldown，也不得使用 debug spawn、instant kill、invulnerability 或 test-only combat outcome。

#### Scenario: 自動建塔與開波
- **WHEN**reference policy 決定購買 tower 並開始下一回合
- **THEN**兩個動作都進入與玩家相同的 input queue、validation 與 authoritative apply path
- **AND**report 可追溯 input id、target tick 與結果

#### Scenario: Strategy 缺錢
- **WHEN**policy 想要的 upgrade 目前 unaffordable
- **THEN**controller 等待或選擇另一個合法正式 action
- **AND**不得增加 cash 或繞過 affordability validation

### Requirement: 1–100 full run 使用 uncapped 15 Hz coarse profile

完整自動測試 SHALL 在 `TD_GREEN_CROSSROADS`、固定 seed、heroes disabled、knowledge disabled 的條件下，以 `dt = 66.667ms` 的 15 Hz coarse profile 從 round 1 執行至 round 100。runner SHALL uncapped 且不得 deliberate sleep；wall-clock ticks per second 或倍速 MUST NOT 成為 pass condition。

#### Scenario: 普通硬體低於 240 coarse ticks per second
- **WHEN**machine 無法維持 240 coarse ticks／wall-second，但 simulation progress 持續前進
- **THEN**test 繼續執行直到完成或 simulation watchdog 失敗
- **AND**不得只因 wall-clock throughput 低於 16×而失敗

#### Scenario: Reference run 完成 round 100
- **WHEN**controller 使用正式七塔四階內容完成 full run
- **THEN**round 1–100 各開始與結束一次
- **AND**round 100 final enemy 由正式 combat path 生成並被擊破
- **AND**result 為 victory、lives 大於零且 cash 非負

### Requirement: Coarse tick 不漏掉 elapsed-time events

支援 coarse profile 的 spawn、attack、pulse、DoT、Regrow、cooldown 與 buff 系統 SHALL 在單 tick 依 deterministic order drain 所有到期 occurrences、保留 fractional remainder，並套用 content-derived bounded limit。Creep 與 projectile movement SHALL 使用 swept segment 處理跨 checkpoint、終點與 collision。

#### Scenario: 單一 coarse tick 有多個 spawn 到期
- **WHEN**`66.667ms` interval 內有多個 enemy spawn time
- **THEN**全部 due enemies 依 authored order 生成
- **AND**沒有 spawn 被延後到錯誤 round 或永久遺失

#### Scenario: Projectile 跨過窄目標
- **WHEN**projectile 的 coarse-step segment 從目標一側移到另一側
- **THEN**swept collision 偵測 segment 與 hit shape 的交會
- **AND**命中順序以 deterministic distance／entity tie-break 決定

#### Scenario: Drain 超過安全上限
- **WHEN**invalid content 使單 tick due occurrences 超過 validated bound
- **THEN**run 以明確 error 終止
- **AND**error 包含 system、entity／content id、dt 與 occurrence count

### Requirement: Full run 在相同 coarse profile 可精確重播

同一 seed、content generation、reference policy 與 coarse tick profile SHALL 連續執行兩次，並產生相同 final state hash、per-round end tick、cash、lives、tower builds、enemy accounting 與 economy ledger digest。

#### Scenario: 兩次 coarse run 比較
- **WHEN**test harness 以相同輸入完成兩次 1–100 run
- **THEN**所有指定 deterministic artifacts 完全相同
- **AND**任何差異都回報第一個 divergent round 與欄位

### Requirement: Production 120 Hz 由 focused milestones 保護

正式 120 Hz simulation SHALL 有 focused tests 覆蓋 early rounds 與 24、28、40、60、80、90、100 的代表 threat，包含 Camo、damage immunity、Regrow、Fortified、MOAB-class、leak 與 economy。15 Hz 與 120 Hz 只比較 invariant totals 與 legal outcomes，不要求 tick、target sequence 或完整 hash 相同。

#### Scenario: Cross-rate invariant comparison
- **WHEN**同一 milestone fixture 分別以 15 Hz 與 120 Hz 執行
- **THEN**spawn counts、authored layer cash totals、property legality 與 conservation invariants 相同
- **AND**完成 tick 與 target sequence 可以不同

#### Scenario: Production cadence 未被修改
- **WHEN**正常 backend 與 local replica 啟動 standard TD session
- **THEN**authoritative cadence 仍為 120 Hz、`dt = 1/120s`
- **AND**coarse test profile 不會由 production config 或玩家 speed input 選到

### Requirement: Autoplay failure report 可定位首次錯誤

Full run 失敗時 SHALL 在 `target/td-autoplay/` 寫入未追蹤 report，包含 seed、tick profile、round、tick、cash、lives、tower build、remaining enemies、recent outcomes、rejected inputs、ledger summary／digest、state hash、entity peak 與 watchdog state。Report writing failure MUST NOT 隱藏原始 simulation failure。

#### Scenario: Reference strategy 在 round 90 失敗
- **WHEN**lives 在 round 90 歸零
- **THEN**test 失敗並產生包含 round 90 首次 defeat context 的 report
- **AND**report path 位於 `target/td-autoplay/`

#### Scenario: 成功 run 不產生大型 trace
- **WHEN**1–100 test 成功
- **THEN**test 可輸出 compact summary
- **AND**不得把大型 report、DLL、log 或 trace 加入 git tracked files

