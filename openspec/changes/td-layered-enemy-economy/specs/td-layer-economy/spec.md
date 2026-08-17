## ADDED Requirements

### Requirement: TD cash 與 pop count 逐層結算

每個被合法移除的 TD layer SHALL 依 catalog cash value credit damage source 的 `PlayerOwner`，並把該 layer 計入 source tower 的 pop count。單次 hit 移除多層時 SHALL 依 resolution plan 精確加總，不得只在最終 entity death 結算一次。

#### Scenario: 單一 hit 移除多層
- **WHEN**owner 1 的塔以一次 hit 移除三個有 cash 的 layers
- **THEN**owner 1 cash 增加三層 cash 的總和
- **AND**該塔 pop count 增加三
- **AND**ledger 記錄相同 layer count 與 amount

#### Scenario: Immune hit 不給 cash
- **WHEN**hit 被 current layer immunity 完全阻擋
- **THEN**player cash 與 tower pop count 不變
- **AND**ledger 不得出現 layer income entry

#### Scenario: 無 owner source 不創造 cash
- **WHEN**合法 environment 或 owner-less source 移除 layer
- **THEN**layer transition 仍正常發生
- **AND**任何 player cash 都不增加
- **AND**ledger diagnostic totals 記錄 unattributed layer cash

### Requirement: Generated TD enemy 不使用通用 bounty fallback

所有帶 `TdLayerState` 的 enemy SHALL 只透過 layer cash 結算，不得使用 `creep_bounty_from_template` 的通用 10 金或 legacy final-death bounty。非 TD creep SHALL 保留既有 bounty。

#### Scenario: 最底層 TD enemy death
- **WHEN**`td_btd_*` enemy 的最後 layer 被移除
- **THEN**cash 只等於被移除 layer 的 authored cash
- **AND**不額外 credit 10 金或 legacy template bounty

#### Scenario: MOBA bounty 保持不變
- **WHEN**owner hero 擊殺沒有 `TdLayerState` 的 `melee_minion`
- **THEN**既有 proximity／owner bounty 與 experience 路徑保持不變

### Requirement: Round bonus 與 layer cash 分離

Round completion SHALL 只 credit `TdEconomyRules.round_bonus(round)`；該 bonus MUST 與當回合 layer cash 分開 author、分開記錄。代表整回合 pop income 的 legacy table MUST NOT 同時作為 clear bonus 使用。

#### Scenario: Round clear 對帳
- **WHEN**某回合所有 enemy 都被擊破並進入 idle
- **THEN**ending cash 等於 starting cash 加 layer income 加 round bonus 減 purchases／upgrades 加 sales
- **AND**round bonus ledger category 只出現一次

#### Scenario: Leak 後仍完成回合
- **WHEN**部分 enemy leaked、其餘 enemy 被擊破且回合完成
- **THEN**只為實際 popped layers credit layer cash
- **AND**依 rule 發放一次 round bonus
- **AND**leaked layers 不產生 pop income

### Requirement: 所有 TD balance mutation 經過 deterministic ledger

`PlayerEconomy` 的 TD initialize、layer credit、round bonus、tower place、upgrade、sell SHALL 透過單一 ledger mutation boundary。每筆 entry SHALL 包含 deterministic tick、serial、player id、category、signed amount、resulting balance 與 stable digest input。Debit MUST 先驗證完整 command，失敗 command 不得留下 entry 或部分 mutation。

#### Scenario: Successful tower purchase
- **WHEN**player 有足夠 cash 且 placement validation 全部通過
- **THEN**ledger 記錄一筆 tower purchase debit
- **AND**resulting balance 與 `PlayerEconomy` 相同

#### Scenario: Rejected purchase 不污染 ledger
- **WHEN**placement 因道路重疊或 cash 不足被拒絕
- **THEN**cash 不變
- **AND**ledger totals、digest 與 recent entries 都不變

#### Scenario: Replica ledger digest 一致
- **WHEN**backend 與 local replica 以同一 tick profile 套用相同 inputs 與 outcomes
- **THEN**每回合 ledger digest 與 player balances 相同

### Requirement: Production ledger bounded 且 test 可完整觀察

Production SHALL 保存 per-player/category cumulative totals、rolling digest 與 bounded recent-entry ring，不得因 100 回合 layer 數量無界成長。Integration test observer SHALL 能從相同 mutation point 收集完整 entry stream，不得另建不同計算公式。

#### Scenario: Recent ring 達到容量
- **WHEN**ledger entries 超過 production recent ring capacity
- **THEN**最舊 recent entry 被移除
- **AND**cumulative totals、digest 與 current balance 不變

#### Scenario: Test observer 完整對帳
- **WHEN**1–100 headless run 啟用 full ledger observer
- **THEN**observer 收到每個成功 mutation entry
- **AND**依 entries 重播所得 ending balances 與 runtime 完全一致

### Requirement: Sellback 使用一致的 rule ratio

TD sell refund SHALL 以 `TdEconomyRules.sellback_ratio` 對 base tower spend 與所有 upgrade spend 的總和一致計算，使用 deterministic rounding。不得再對 base 與 upgrade 使用不同 hard-coded percentage。

#### Scenario: Upgraded tower sellback
- **WHEN**tower 的 base 與 upgrades 總支出為 `S` 且 ratio 為 `R`
- **THEN**refund 等於規定 deterministic rounding 的 `S * R`
- **AND**ledger 只記錄一次 sell credit

