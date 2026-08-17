## ADDED Requirements

### Requirement: TD enemy layer graph 是 authoritative content

每個 shipped TD enemy archetype SHALL 由 generated content 提供穩定 id、目前層 HP、移動速度、ordered children、layer cash、remaining leak value、property flags 與 damage compatibility metadata。Layer graph MUST 為無 cycle 的有向圖，所有 child reference MUST 存在，數值 MUST 在可表示且非負的範圍內。

#### Scenario: Valid layer catalog 通過 codegen
- **WHEN** base layers 與 modifier variants 的 child references 都存在且 graph 無 cycle
- **THEN** `omoba-template-ids` 產生可由 runtime 查詢的 TD layer metadata
- **AND** generated Rust 與 runtime Lua content snapshot 保留相同 declaration order 與數值

#### Scenario: Invalid child reference 立即失敗
- **WHEN** 某 layer 宣告不存在的 child id
- **THEN** codegen 或 runtime Lua validation 在 gameplay 前失敗
- **AND** error message 包含 parent layer id、child id 與欄位名稱

#### Scenario: Cyclic graph 被拒絕
- **WHEN** layer graph 直接或間接回到已在 ancestor chain 的 layer
- **THEN** content validation 失敗
- **AND** error message 包含完整 cycle path

### Requirement: TD creep 保存目前 layer 與 property state

TD creep SHALL 保存 optional `TdLayerState`，至少包含目前 archetype、Camo／Regrow／Fortified／MOAB-class properties、Regrow ceiling／timer 與 deterministic spawn lineage。`CProperty.hp/mhp` 對該 entity SHALL 只表示目前 layer HP，不得再表示整棵 graph 的 flattened effective HP。

#### Scenario: TD enemy spawn 建立 layer state
- **WHEN** round 產生一個 Camo Regrow enemy
- **THEN** entity 的 `TdLayerState` 包含 base archetype、Camo 與 Regrow
- **AND**目前 HP／max HP 等於 current layer HP

#### Scenario: Non-TD creep 保持 legacy state
- **WHEN** MOBA story 產生沒有 TD layer metadata 的 creep
- **THEN**該 entity 的 `TdLayerState` 為 `None`
- **AND**既有 HP、armor、magic resistance、bounty 與 AI 行為不變

### Requirement: Layer damage 以 deterministic resolution plan commit

TD layer damage SHALL 先由不修改 ECS 的純 resolver 計算 ordered resolution plan，再以固定順序更新原 entity、建立 surviving children、記錄 pop attribution 與 cash。單 child transition SHALL 重用原 entity；branch transition SHALL 依 authored child order 與 deterministic spawn serial 建立 children。

#### Scenario: Overkill 穿透多個單 child layers
- **WHEN**一次合法 hit 的 damage 超過目前 layer 與後續數個單 child layers 的 HP
- **THEN** resolver 依序記錄所有被移除 layers
- **AND**只將最後仍存活的 layer 寫回原 entity
- **AND**中間 layer 不會被建立成 transient entity

#### Scenario: Branch 只建立存活 children
- **WHEN**被擊破 layer 產生多個 children，且剩餘 damage 足以繼續擊破其中一部分
- **THEN** resolver 依 authored order 分配與消耗 damage
- **AND**commit 只建立該 hit 結束後仍存活的 children
- **AND**相同 seed 與 input 產生相同 entity ids、spawn lineage 與 outcome order

#### Scenario: Exact layer damage 轉入 children
- **WHEN** damage 恰好移除目前 layer 且該 layer 有 children
- **THEN**目前 layer 被記為 popped
- **AND**children 以完整 authored HP 進入 world
- **AND**沒有負 HP 或重複 death outcome

### Requirement: Regrow 與 Fortified 有明確繼承規則

Regrow enemy SHALL 依 fixed simulation time 與 authored interval 向 Regrow ceiling 恢復，且不得超過該 ceiling。Fortified SHALL 只套用於 catalog 標示 eligible 的 layers，並只傳遞給 eligible children。移除 property 的效果 SHALL 立即影響後續 transition。

#### Scenario: Regrow 恢復上一層
- **WHEN** Regrow enemy 已被剝到低於 ceiling 且 timer 到期
- **THEN**它恢復一個合法 parent layer
- **AND**timer 保留 deterministic remainder
- **AND**它不超過 authored ceiling

#### Scenario: 單一 coarse tick 到期多次
- **WHEN** `dt = 66.667ms` 內跨過多個合法 Regrow interval
- **THEN**runtime 依時間順序消耗所有到期 occurrence，直到 ceiling 或 bounded drain limit
- **AND**結果不會因 driver wall-clock throughput 改變

#### Scenario: Fortified 傳遞到 eligible child
- **WHEN** Fortified layer 被擊破且 child 支援 Fortified
- **THEN**child 保留 Fortified 並使用 authored Fortified HP 規則
- **AND**不支援 Fortified 的 child 不會非法取得該 property

### Requirement: 漏怪依剩餘 layer graph 扣命一次

TD creep 到達終點時 SHALL 依目前 layer state 的 remaining leak value 扣除 player lives，並只產生一次 leak outcome。Lives SHALL saturate at zero；同一 entity MUST NOT 同時被 popped 與 leaked。

#### Scenario: 部分剝層後漏怪
- **WHEN**高層 enemy 已被移除部分 layers 後到達終點
- **THEN**扣除目前剩餘 graph 對應的 leak value
- **AND**扣命少於未受傷完整 archetype 的 leak value

#### Scenario: 大型 enemy 漏怪
- **WHEN** MOAB-class enemy 以多層剩餘內容到達終點
- **THEN**扣除值大於普通最底層 enemy 的一命
- **AND**該 entity 與尚未 materialize 的 child value 不會重複扣除

### Requirement: Snapshot 暴露穩定的 TD layer 資訊

Authoritative snapshot SHALL 對 TD creep 暴露 current layer id、property flags、current／max layer HP 與 remaining leak value；這些欄位 SHALL 由 authoritative state 複製，snapshot drain MUST NOT 修改 gameplay。

#### Scenario: Camo Fortified snapshot
- **WHEN** snapshot 包含仍存活的 Camo Fortified TD creep
- **THEN**該 entity render data 包含 current layer id、Camo 與 Fortified flags
- **AND**snapshot extraction 前後 deterministic gameplay state 相同

