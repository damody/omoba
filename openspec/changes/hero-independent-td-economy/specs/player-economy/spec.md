## ADDED Requirements

### Requirement: TD 玩家帳戶獨立於 Hero entity 初始化

系統 SHALL 在 TD mode 初始化 player 1 與 player 2 的 `PlayerEconomy` 帳戶，並使用 resolved `TdDifficultyConfig.starting_gold`；帳戶初始化 SHALL NOT 依賴 Hero entity、`Gold` component 或 `OMB_NO_HEROES` 的值。

#### Scenario: 零英雄仍有預設起始金錢
- **WHEN** TD_1 使用 `OMB_NO_HEROES=1` 與 expert 預設設定初始化
- **THEN** ECS 中 Hero entity 數量為零
- **AND** player 1 與 player 2 的帳戶餘額皆為 650

#### Scenario: run_10000 覆寫起始金錢
- **WHEN** resolved TD config 的 `starting_gold` 為 10,000
- **THEN** player 1 與 player 2 的帳戶餘額皆為 10,000

### Requirement: TD 塔交易使用玩家帳戶

建塔與升級 SHALL 從 requesting `player_id` 的 `PlayerEconomy` 帳戶扣款，出售 SHALL 將退款加入同一帳戶。所有 validation SHALL 在 mutation 前完成；失敗交易 SHALL NOT 改變任何玩家餘額。

#### Scenario: 無英雄玩家成功建塔
- **WHEN** 無 Hero entity 的 player 1 帳戶餘額足以支付合法塔的 cost
- **THEN** 建塔成功
- **AND** player 1 餘額精確減少該 cost
- **AND** player 2 餘額不變

#### Scenario: 餘額不足拒絕交易
- **WHEN** requesting player 帳戶餘額小於建塔或升級 cost
- **THEN** 命令回傳 insufficient gold 錯誤
- **AND** 所有帳戶、tower entities 與 upgrade levels 均不變

#### Scenario: 缺少帳戶拒絕交易
- **WHEN** requesting `player_id` 不存在於 `PlayerEconomy`
- **THEN** 命令回傳 missing player economy account 診斷
- **AND** 不建立、升級、移除 tower 或改變任何餘額

#### Scenario: 出售退款回到 owner 帳戶
- **WHEN** player 1 出售自己擁有的合法 tower
- **THEN** tower 經既有 removal outcome 移除
- **AND** 既有退款公式的金額加入 player 1 帳戶
- **AND** 其他玩家帳戶不變

### Requirement: TD 獎勵寫入玩家帳戶

TD round income SHALL credit 所有已初始化帳戶；TD creep bounty SHALL 僅在 damage source 可解析出 `PlayerOwner` 時 credit 該 player。加款 SHALL 使用 saturating arithmetic，且非 TD Hero bounty 與 experience 行為 SHALL 維持不變。

#### Scenario: 零英雄完成回合仍取得收入
- **WHEN** 無 Hero entity 的 TD round N 完成且 Bloons income table 為該回合提供 amount
- **THEN** 每個已初始化玩家帳戶增加 amount

#### Scenario: owned tower 擊破獎勵歸 owner
- **WHEN** player 2 擁有的 tower 擊破具有正 bounty 的 TD creep
- **THEN** player 2 帳戶增加該 bounty
- **AND** player 1 帳戶不變

#### Scenario: 無法歸屬的 TD 擊破不猜測玩家
- **WHEN** TD creep death 的 damage source 不存在或沒有 `PlayerOwner`
- **THEN** 任一玩家帳戶均不因該 bounty 改變

### Requirement: 玩家經濟參與 deterministic state hash

authoritative state hash SHALL 包含依 `player_id` 排序的所有玩家餘額。相同帳戶集合與餘額 SHALL 產生相同 hash，不受帳戶插入順序影響。

#### Scenario: 餘額改變會改變 hash
- **WHEN** 兩個其他狀態相同的 World 僅有一個玩家餘額不同
- **THEN** `compute_state_hash` 結果不同

#### Scenario: 插入順序不影響 hash
- **WHEN** 兩個 World 以相反順序插入相同 `(player_id, balance)` 帳戶
- **THEN** `compute_state_hash` 結果相同
