## ADDED Requirements

### Requirement: 已開始的本機 session 必須結算一次

每個成功進入 gameplay 的本機 session SHALL 在勝利、失敗或使用者退出時恰好更新一次 profile 戰績。重複 terminal snapshot、重複 shutdown 或 terminal detection 後的自動 teardown MUST NOT 重複結算。尚未成功進入 gameplay 的啟動失敗 MUST NOT 計為一場。

#### Scenario: 勝利後自動 teardown
- **WHEN**已開始的 session 收到 victory terminal snapshot，下一個 frame 自動 teardown
- **THEN**`games_played` 增加一且 `wins` 增加一
- **AND**teardown 不會再次增加任何戰績

#### Scenario: 失敗後自動 teardown
- **WHEN**已開始的 session 因 lives 歸零收到 defeat terminal snapshot
- **THEN**`games_played` 增加一且 `wins` 不變

#### Scenario: 中途退出
- **WHEN**已開始的 session 透過返回標題、Ctrl+Escape 或正常應用程式 shutdown 結束，且沒有 terminal result
- **THEN**session 以 `Abandoned` 結算，`games_played` 增加一且 `wins` 不變
- **AND**不發放 end-of-match KP

#### Scenario: 啟動失敗
- **WHEN**backend、lockstep 或 sim runner 啟動失敗，session 尚未進入 gameplay
- **THEN**profile 戰績完全不變

### Requirement: 結算必須保存最高回合與擊殺數

Frontend SHALL 保存 session 期間最高觀察 round 與最新 `MatchKillCounter` snapshot。結算時 `highest_wave` SHALL 取既有值與 session peak 的最大值，`total_kills` SHALL saturating-add 該 session kill count。Removed entity 或 leak MUST NOT 被前端自行推導為 kill。

#### Scenario: 失敗前到達更高回合
- **WHEN**profile 的 `highest_wave` 為 12，session 在 round 28 失敗
- **THEN**結算後 `highest_wave` 為 28

#### Scenario: 退出未突破紀錄
- **WHEN**profile 的 `highest_wave` 為 40，session 在 round 17 退出
- **THEN**結算後 `highest_wave` 仍為 40

#### Scenario: 擊殺數來自 snapshot
- **WHEN**terminal 前最新 snapshot 的 `MatchKillCounter` 為 321
- **THEN**`total_kills` 增加 321
- **AND**frontend 不以 `removed_entity_ids` 數量替代該值

### Requirement: Match kill snapshot 必須唯讀且 deterministic

共用 render snapshot SHALL 暴露 ECS `MatchKillCounter` 的目前值。Snapshot extraction MUST NOT 清除、增加或以其他方式修改 counter，backend 與 local replica 對相同 world state SHALL 取得相同值。

#### Scenario: Snapshot 複製 kill counter
- **WHEN**world 的 `MatchKillCounter` 為 47 並執行 snapshot extraction
- **THEN**snapshot 的 match kill count 為 47
- **AND**world counter 在 extraction 後仍為 47

### Requirement: Frontend 必須是戰績唯一寫入者

Frontend SHALL 在 session teardown 結算四個戰績欄位。Backend game-end handling MAY 發放 KP，但 MUST NOT 修改 `games_played`、`wins`、`highest_wave` 或 `total_kills`。Frontend SHALL 在 owned backend 結束並完成 wait 後重新載入 profile，再合併戰績，以保留已完成的 KP 或 knowledge 更新。

#### Scenario: Backend 已寫入 KP
- **WHEN**backend 在 shutdown 前增加 `total_kp`，frontend 隨後結算勝利
- **THEN**保存後的 profile 同時包含新 KP、增加一場與增加一勝

#### Scenario: Backend 收到 game end
- **WHEN**backend 收到 victory 或 defeat `game/end`
- **THEN**它依既有規則處理 KP
- **AND**四個戰績欄位保持不變

### Requirement: Profile merge 必須相容且不破壞原檔

Frontend SHALL 接受缺少戰績欄位的 legacy JSON，將缺少值視為零，並保留既有及未知欄位。所有 counter arithmetic MUST saturate。Persistence SHALL 在完整 serialization 成功後使用同目錄 temporary file 與 platform-safe replacement；寫入或 replacement 失敗 MUST 保留原 profile 可讀，且不得阻止 teardown。

#### Scenario: Legacy profile 首次結算
- **WHEN**profile 只有 `total_kp`、`spent_kp` 與 `unlocked_nodes`
- **THEN**一次 defeat 結算後 `games_played` 為一、`wins` 為零
- **AND**原有 knowledge 欄位保持不變

#### Scenario: Counter overflow
- **WHEN**`games_played` 或 `total_kills` 已接近 `u32::MAX`
- **THEN**結算結果 saturate 在 `u32::MAX`，不 overflow 或 wrap

#### Scenario: Replacement 失敗
- **WHEN**temporary profile 已序列化，但正式檔 replacement 回傳 error
- **THEN**原 profile 仍是合法可讀 JSON
- **AND**session teardown 繼續並記錄包含 result、path 與 error 的 log
