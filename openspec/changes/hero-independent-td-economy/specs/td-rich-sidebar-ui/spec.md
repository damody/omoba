## ADDED Requirements

### Requirement: TD HUD 與商店使用本機玩家 snapshot 金錢

omfx TD UI SHALL 在 snapshot 含有 `local_player_id` 的玩家金錢時更新既有金錢顯示與 affordability state，且 SHALL NOT 以 Hero entity 存在作為更新條件。Hero metadata、生命、技能與 entity selection SHALL 仍只由真正的 Hero snapshot 更新。

#### Scenario: 零英雄顯示本機玩家金錢
- **WHEN** snapshot 沒有 Hero entity，但 player 1 金錢為 650 且 `local_player_id == 1`
- **THEN** TD HUD 顯示 `$650`
- **AND** cost 不高於 650 的 tower 顯示為可購買
- **AND** UI 不建立 Hero entity id 或假 Hero selection

#### Scenario: 多玩家只顯示本機帳戶
- **WHEN** snapshot 中 player 1 金錢為 650、player 2 金錢為 10,000，且 `local_player_id == 2`
- **THEN** TD HUD 與 shop affordability 使用 10,000
- **AND** 不使用 player 1 的餘額

#### Scenario: snapshot 缺少本機帳戶時保留安全狀態
- **WHEN** snapshot 不含 `local_player_id` 對應帳戶
- **THEN** omfx 不以另一玩家帳戶覆寫本機金錢
- **AND** 不 panic 或建立假 Hero 狀態
