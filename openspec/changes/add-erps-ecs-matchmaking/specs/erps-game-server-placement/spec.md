## ADDED Requirements

### Requirement: Game server 動態註冊能力與 generation
Game server SHALL 透過 gRPC 註冊 stable server ID、generation、endpoint、region、supported modes、capacity、per-mode costs、`max_instances` 與現有 instances。`max_instances` MUST 位於 1～100；ERPS MUST 以 server／class policy 限制回報上限，舊 generation 訊息 MUST NOT 覆蓋新程序狀態。

#### Scenario: 舊 heartbeat 不覆蓋新註冊
- **WHEN** 同一 server ID 的新 generation 已註冊後收到舊 generation heartbeat
- **THEN** ERPS 忽略舊訊息且不修改新 generation 的 capacity 或 health

### Requirement: Heartbeat 與 reconcile 控制 server 健康
Game server SHALL 透過雙向 control stream 定期 heartbeat 並於註冊／重連時 reconcile instances。ERPS SHALL 在設定的 missed-heartbeat 門檻後停止新 placement，並於失聯門檻後釋放未確認 reservation；running match SHALL 標記 `ServerLost` 並通知 client。

#### Scenario: 失聯 server 不接新 match
- **WHEN** server 超過健康 heartbeat 門檻
- **THEN** placement 不再選擇該 server，且其 running match 不會被自動遷移

### Requirement: Placement 同時遵守容量與 instance 上限
每個模式 SHALL 使用 server 回報且 policy 核准的 capacity cost。Placement MUST 同時滿足 region、supported mode、健康狀態、`capacity_used + mode_cost <= capacity_total` 與 `running_or_reserved_instances < max_instances`。

#### Scenario: 容量足夠但 instance 已滿
- **WHEN** server 尚有 capacity units 但 running／reserved instances 已達 `max_instances`
- **THEN** ERPS 不得在該 server 建立新 reservation

### Requirement: Ready check 前不占用正式容量
ERPS SHALL 在 ready check 期間只做 soft feasibility check，MUST NOT 建立正式 reservation。全員接受後 SHALL 原子選擇 server 並扣除 reservation；暫時無容量時 SHALL 進入有期限 placement waiting。

#### Scenario: 未接受 proposal 不占容量
- **WHEN** proposal 處於 `AwaitingAccept`
- **THEN** 所有 server 的 authoritative capacity 與 reserved instance count 都不因該 proposal 改變

### Requirement: Game instance Ready 後才通知 client
Launch lifecycle SHALL 依序經過 `Reserved`、`Accepted`、`Ready`、`Running` 與 `Finished`。ERPS MUST 只在 game server 回報 `Ready` 並提供 endpoint 與 connection token 後發布 client match result。

#### Scenario: Accepted 尚未可連線
- **WHEN** game server 已接受 launch command 但尚未回報 instance `Ready`
- **THEN** client 不會收到可連線 match result，reservation 保持受控且受 ready timeout 約束

### Requirement: Launch failure 可安全回收與重試
Game server reject、ready timeout 或 placement timeout SHALL 釋放完整 reservation，MUST NOT 超配或遺失 ticket。ERPS SHALL 優先重試其他合法 server；無法配置時 SHALL 讓玩家無懲罰回 queue並保留等待時間。

#### Scenario: Reject 後容量完全歸還
- **WHEN** game server reject 一個已建立 reservation 的 launch
- **THEN** 該 reservation 的 capacity units 與 instance count 完整歸還，且玩家信用分不變

