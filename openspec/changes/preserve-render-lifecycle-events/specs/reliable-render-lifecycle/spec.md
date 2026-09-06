## ADDED Requirements

### Requirement: 生命週期事件使用不可丟棄通道
Client runtime MUST 將 `Hide`、`Forget` 與 `ResetView` 送入保序且不可覆蓋的 lifecycle 通道，並 MUST NOT 依賴 latest snapshot 傳遞這些事件。

#### Scenario: 最新狀態覆蓋舊狀態
- **WHEN** renderer 尚未消費一份含 lifecycle event 的資料，而後續多份狀態 snapshot 抵達
- **THEN** 系統可以合併狀態 snapshot，但 MUST 依序保留並交付所有 lifecycle event

#### Scenario: lifecycle queue 滿載
- **WHEN** lifecycle queue 達到容量上限
- **THEN** producer MUST 等待 queue 可用或結束已中斷的 session，且 MUST NOT 丟棄或覆蓋既有事件

### Requirement: 事件保留 replica 安全識別資訊
每個 entity lifecycle event MUST 攜帶 `replica_id` 與 `disclosure_epoch`，每個 batch MUST 攜帶 `view_epoch`，且玩家端資料 MUST NOT 包含 canonical entity ID。

#### Scenario: Forget 經過 IPC
- **WHEN** team replica 產生 disclosure epoch 非零的 `Forget`
- **THEN** renderer 收到的事件 MUST 保留相同的 `replica_id` 與 `disclosure_epoch`

### Requirement: Renderer 依序且冪等地套用 lifecycle
Renderer MUST 依事件順序套用 lifecycle，且重複事件 MUST 不會建立額外 deterministic 或 remembered presentation。

#### Scenario: 同一 Forget 重複抵達
- **WHEN** renderer 對同一 `(replica_id, disclosure_epoch)` 套用兩次 `Forget`
- **THEN** 該 deterministic identity MUST 保持移除，且不得產生額外 scene node

#### Scenario: Hide 後收到晚到的舊狀態
- **WHEN** renderer 先套用 `Hide`，之後較舊 snapshot 才抵達且仍包含已退休的 replica identity
- **THEN** renderer MUST 忽略該 identity，並 MUST NOT 復活 deterministic presentation

#### Scenario: Hide 後重新 Reveal
- **WHEN** 同一 canonical entity 重新進入視野，沿用 replica ID 並提高 disclosure epoch
- **THEN** renderer MUST 保留新 disclosure 的 deterministic presentation、移除同 replica ID 的舊 remembered presentation，且舊 disclosure identity MUST 維持關閉

#### Scenario: Forget 後收到晚到的舊狀態
- **WHEN** renderer 先套用 `Forget`，之後較舊 snapshot 才抵達且仍包含已退休的 replica identity
- **THEN** renderer MUST 忽略該 identity，並 MUST NOT 復活 deterministic 或 remembered presentation

### Requirement: View epoch 切換清除舊視圖
Runtime MUST 在建立新 renderer 視圖或切換 view epoch 時送出 `ResetView`，renderer MUST 在接受新 epoch 資料前清除舊 epoch 的 presentation 狀態。

#### Scenario: Renderer 重新建立視圖
- **WHEN** renderer 連線建立新的 view epoch
- **THEN** renderer MUST 清除前一 epoch 的 deterministic、remembered 與 retired 狀態，且 MUST 忽略之後抵達的舊 epoch lifecycle event

### Requirement: 慢速 Renderer 不得產生視野分身
系統 MUST 在 renderer 消費速度低於 presentation 產生速度時維持 lifecycle 正確性。

#### Scenario: 敵方英雄反覆跨越視野邊界
- **WHEN** renderer 被刻意延遲消費，且同一敵方英雄反覆 Reveal 與 Forget
- **THEN** 每個 Forget MUST 最終移除其 replica identity，任一玩家在 renderer 中 MUST 最多只有一個 deterministic hero presentation
