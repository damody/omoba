## ADDED Requirements

### Requirement: 地圖以合法且穩定的資料描述視野遮蔽物

系統 SHALL 接受預設為空的 `VisionTrees` 與 `VisionOccluderPolygons` 地圖欄位。每個遮蔽物 SHALL 具有同地圖唯一且非零的 `StableId`；樹木半徑 SHALL 大於零；多邊形 SHALL 至少三點、不得有重複相鄰點、自交或零面積。非法資料 MUST 使 map load 明確失敗，不得靜默修正。

#### Scenario: 舊地圖沒有遮蔽物欄位
- **WHEN** 載入未宣告 `VisionTrees` 或 `VisionOccluderPolygons` 的既有地圖
- **THEN** runtime 建立空的 vision occluder set
- **AND** 可見性結果與加入功能前相同

#### Scenario: 非法多邊形被拒絕
- **WHEN** 地圖包含自交、零面積或重複相鄰頂點的視野多邊形
- **THEN** map load 回傳指出 `StableId` 與原因的錯誤
- **AND** 該地圖不得開始 simulation

### Requirement: 固定點 LOS 正確判定圓形樹木遮蔽

server SHALL 以固定點線段最近點與平方距離判定 source-target LOS 是否先撞到樹木圓。切線接觸 SHALL 算遮擋；位於 target 後方的樹 SHALL 不影響結果；source 位於樹內時 SHALL 忽略該樹；target 位於樹內時 SHALL 視為遮擋。

#### Scenario: 樹後目標被遮擋
- **WHEN** source、樹圓與 target 依序位於同一條 LOS，且線段穿過或切到樹圓
- **THEN** 該 source 不得揭露 target

#### Scenario: 目標後方樹木不遮擋
- **WHEN** 樹圓只與 source 到 target 延長線相交且位於 target 後方
- **THEN** 該樹不影響 target 可見性

### Requirement: 固定點 LOS 正確判定不規則地形遮蔽

server SHALL 以固定點線段相交與 point-in-polygon 支援順時針、逆時針、凸與凹簡單多邊形。LOS 穿邊、碰頂點或沿邊重疊 SHALL 算遮擋；target 位於內部或邊界 SHALL 視為遮擋；source 位於內部或邊界時 SHALL 忽略該多邊形。

#### Scenario: 凹多邊形阻擋穿越邊界的 LOS
- **WHEN** source-target 線段進入合法凹多邊形
- **THEN** 該 source 不得揭露 target
- **AND** 反轉多邊形 winding 後結果相同

#### Scenario: LOS 通過凹口但未碰邊界
- **WHEN** source-target 線段位於凹多邊形的外部凹口且未與任何邊相交
- **THEN** 該多邊形不遮擋 target

### Requirement: Wave B 依隊伍與多視野來源產生權威結果

Wave B SHALL 從單一 immutable committed view 讀取 entity、vision source 與穩定排序的 occluders。候選 entity 只要被任一合法己方 source 在距離、方向、detection 與 LOS 上同時通過 SHALL 揭露；只有所有來源皆失敗時 SHALL 不可見。`Public`、`ForceShow`、`ForceHide`、`ServerOnly` 與 owner-team 規則 SHALL 保持既有優先序。

#### Scenario: 任一來源無遮擋即揭露
- **WHEN** team 有兩個可偵測 target 的視野來源，其中一條 LOS 被樹遮擋而另一條暢通
- **THEN** target 出現在該 team 的 disclosed set

#### Scenario: 所有來源受阻時 Forget
- **WHEN** 前一 tick 可見的 target 在本 tick 對該 team 所有合法來源都受阻
- **THEN** server 產生一次 canonical `Forget`
- **AND** 後續 team bytes 不含 target 的 canonical ID、位置或狀態

### Requirement: 遮蔽結果跨平行排程保持確定性

source、entity、occluder 與 transition SHALL 使用 canonical order。不同 Rayon worker 完成順序 MUST 產生完全相同的 visible set、Reveal/Forget 順序、team hash 與 encoded frame bytes。幾何運算 overflow 或無法安全判定時 MUST fail closed 為不可見並產生限流 diagnostic。

#### Scenario: 重排平行完成順序
- **WHEN** 同一 committed world 以不同 team/source worker completion order 重算 Wave B
- **THEN** 每隊 transition、hash 與 encoded bytes 逐 byte 相同

#### Scenario: 幾何計算無法安全完成
- **WHEN** checked fixed-point 中間值超出可安全表示範圍
- **THEN** 該 LOS 結果為 blocked
- **AND** server 不會向玩家揭露該候選 entity

### Requirement: Replica team observer 驗算實際送出資料

同 process 的另一個 thread SHALL 各自以指定 team 的 observer replica 消費實際送出佇列內容。Observer SHALL 依收到的 Reveal、Update 與 Forget 維護 replica 並比較預期 team hash/bytes；不得讀 canonical world 作為答案捷徑。

#### Scenario: 兩隊遮蔽結果各自驗算
- **WHEN** 同一 target 對 team 1 可見、對 team 2 被地形遮擋
- **THEN** team 1 與 team 2 的實際送出佇列內容不同
- **AND** 兩個 replica team observer 都通過各自的驗算

### Requirement: 遮蔽壓力場景符合效能上限

在 100 個普通單位、兩位額外英雄、至少 64 棵樹、三個多邊形與 120 Hz 的固定 demo 場景中，Wave B p99 SHALL 不高於相同 entity/source 配置且無遮蔽物 baseline 的兩倍，整體 server tick SHALL 不持續超過 8.33 ms。若未達標，最佳化 MUST 保持逐項可見性結果與 encoded bytes 不變。

#### Scenario: 執行固定遮蔽 benchmark
- **WHEN** 以相同 build profile 分別量測無遮蔽 baseline 與固定遮蔽壓力場景
- **THEN** 遮蔽場景符合 Wave B p99 與 server tick 上限
- **AND** benchmark 記錄場景數量、樣本數、profile 與量測結果
