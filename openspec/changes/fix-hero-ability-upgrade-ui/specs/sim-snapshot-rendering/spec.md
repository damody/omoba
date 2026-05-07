## ADDED Requirements

### Requirement: 永久 buff 剩餘時間顯示為無限

omfx hero panel SHALL 將永久或 toggle 型 buff 顯示為 `∞`，而不是顯示 permanent duration sentinel 換算出的巨大秒數。`SimWorldSnapshot` 的 hero buff projection SHALL 將 permanent sentinel 或等效超大 remaining duration 正規化為 `remaining_secs == -1.0`，前端 SHALL 依既有規則把負值顯示為 `∞`。

#### Scenario: Passive permanent buff displays infinity

- **WHEN** hero 具有被動技能產生的永久 buff，且 backend BuffStore 使用 permanent sentinel duration
- **THEN** snapshot 中該 buff 的 `remaining_secs == -1.0`
- **AND** omfx hero panel 顯示該 buff 剩餘 `∞`
- **AND** omfx 不顯示 `2097147.9秒` 或其他巨大秒數

#### Scenario: Finite buff still counts down

- **WHEN** hero 具有一般有限時間 buff，且 snapshot 回報正的 remaining seconds
- **THEN** omfx 仍以秒數顯示並在 frame 間本地遞減

### Requirement: 從快照狀態渲染可點擊技能升級按鈕

omfx 的技能 HUD SHALL 在目前可升級的每個技能圖示上方渲染三角升級按鈕，位置 SHALL 類似 LoL 的升級提示，而不是覆蓋在技能圖示內部。當本地英雄快照具有 `skill_points > 0`、該欄位有綁定的技能 id，且目前技能等級低於該技能 metadata 中的最高等級時，該欄位即為可升級。

按鈕 SHALL 由以快照為依據的英雄狀態與技能 metadata 推導。omfx SHALL NOT 在送出升級輸入時透過 optimistic 地扣除技能點來隱藏按鈕；按鈕可見性 SHALL 在權威快照值變更時更新。

按鈕可見時 SHALL 有對應的滑鼠 hit-test 區域。點擊按鈕 SHALL 送出對應欄位的 lockstep `UpgradeAbility` input，且 SHALL NOT 同時觸發技能施放、放塔、選塔、移動或其他地圖點擊行為。若三角按鈕 hit-test 與技能圖示 hit-test 皆可能命中，三角按鈕 SHALL 優先處理。

#### Scenario: 可升級技能顯示三角按鈕

- **WHEN** 英雄快照回報 `skill_points > 0`、技能欄位 1 有綁定技能，且其目前等級低於最高等級
- **THEN** 欄位 1 的技能圖示上方會顯示三角升級按鈕
- **AND** 按鈕會跟著圖示上方定位，而不是覆蓋在圖示內部、tooltip 或英雄狀態文字中
- **AND** 按鈕有可命中的滑鼠點擊區域

#### Scenario: 不可升級技能隱藏按鈕

- **WHEN** 英雄沒有技能點、欄位沒有綁定技能，或技能已達最高等級
- **THEN** 對應的技能圖示不會顯示三角升級按鈕
- **AND** 對應的按鈕 hit-test 區域不會接受點擊

#### Scenario: 點擊三角按鈕送出升級輸入

- **WHEN** 技能欄位 2 顯示三角升級按鈕，且玩家左鍵點擊該按鈕
- **THEN** omfx 送出 `UpgradeAbility` input，且 `ability_index == 2`
- **AND** 該次點擊不會被後續技能圖示施法、地圖或 TD 點擊邏輯再次處理

#### Scenario: 三角按鈕位於圖示上方

- **WHEN** 技能欄位可升級且 HUD layout 更新
- **THEN** 三角升級按鈕的可見位置位於技能圖示上方
- **AND** 技能圖示本體仍保留可點擊施法區域

#### Scenario: 按鈕跟隨權威升級結果

- **WHEN** 玩家送出技能升級輸入，而目前快照仍回報舊的技能點與技能等級值
- **THEN** omfx 會根據該目前快照維持按鈕可見性與可點擊性
- **AND** 當後續快照回報已扣除的技能點或已提升的技能等級後，會根據新的權威值重新計算按鈕可見性
