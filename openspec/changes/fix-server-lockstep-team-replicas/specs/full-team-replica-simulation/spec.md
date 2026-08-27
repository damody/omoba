## ADDED Requirements

### Requirement: 固定兩隊各自持有filtered Specs world
Secure match SHALL 在match建立時建立Team 1與Team 2兩份獨立Specs world，且每份world SHALL 只包含該隊已disclosed的gameplay entity與resource。

#### Scenario: 沒有玩家連線仍建立兩隊world
- **WHEN** server建立一場新的secure match且尚無玩家session
- **THEN** Team 1與Team 2 filtered Specs world都已完成bootstrap
- **AND** 兩份world不依賴玩家成功接收`TeamGameStart`

#### Scenario: Hidden entity不進入另一隊world
- **WHEN** canonical entity只對Team 1 disclosed
- **THEN** Team 1 world包含對應replica-local entity
- **AND** Team 2 world的entity storage、resource與render memory都不含該entity gameplay state

### Requirement: Replica執行完整deterministic gameplay phases
Production team replica SHALL 使用與authoritative server相同順序的deterministic gameplay phase runner，包含Specs systems、pending queue drains、outcome boundaries、tower ability、script dispatch與creep wave；production path MUST NOT 使用`NoopDisclosedWorldStepper`。

#### Scenario: 移動由replica simulation產生
- **WHEN** disclosed hero收到已接受的`MoveTo`輸入
- **THEN** team replica透過共用phase runner更新hero位置
- **AND** steady-state frame不需要component repair才能得到相同位置

#### Scenario: Production observer沒有Noop stepper
- **WHEN** secure match建立server team observer
- **THEN** worker建構真正的Specs disclosed-world stepper
- **AND** `NoopDisclosedWorldStepper`只允許存在於明確test fixture

### Requirement: Visibility transition在固定phase改變simulation membership
Replica SHALL 在`PreStep`依effective tick套用Reveal、Hide與Forget。Remembered presentation SHALL 存在於simulation之外，且不得參與hash、碰撞、targeting或script query。

#### Scenario: Reveal entity參與當前tick
- **WHEN** `RevealEntity.effective_tick`等於replica目前tick
- **THEN** baseline與dependency在gameplay step前建立完成
- **AND** entity可以參與該tick的deterministic systems

#### Scenario: Forget entity不再參與當前tick
- **WHEN** `ForgetEntity.effective_tick`等於replica目前tick
- **THEN** entity在gameplay step前從Specs world移除
- **AND** retired replica ID不能再被輸入或transition使用

### Requirement: Hidden dependency以external effect跨界
需要未disclosed dependency才能決定的gameplay結果 SHALL 由authoritative server投影為sanitized external effect，team replica MUST NOT 建立hidden surrogate entity或取得canonical identity。

#### Scenario: Hidden attacker傷害visible hero
- **WHEN** hidden attacker在authoritative world傷害Team 1已disclosed hero
- **THEN** Team 1 frame只包含套用傷害所需的sanitized external effect
- **AND** frame不包含attacker canonical ID、replica ID或hidden position

### Requirement: Deterministic parity在authority correction前判定
Replica SHALL 在local step完成且套用PostStep correction之前計算`pre_repair_observed_hash`。Server expected hash與observed hash衝突時 SHALL 先記錄divergence，再由server選擇repair、replace或rebase。

#### Scenario: 故意改變replica component可被偵測
- **WHEN** 測試在checkpoint前故意修改replica hero位置
- **THEN** `pre_repair_observed_hash`與server expected hash不同
- **AND** 後續repair不能把該checkpoint改記為parity pass

### Requirement: Steady-state frame不以repair代替simulation
Server SHALL NOT 對每個可見entity每tick主動產生component repair。`ComponentRepair`、`EntityReplace`與filtered rebase SHALL 只用於bootstrap以外的明確authority recovery。

#### Scenario: 穩定移動不含主動repair
- **WHEN** disclosed hero連續移動且server與replica沒有hash mismatch
- **THEN** steady-state frame不含該hero的位置`ComponentRepair`
- **AND** server與replica pre-repair hash保持一致

