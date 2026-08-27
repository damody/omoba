## MODIFIED Requirements

### Requirement: server authoritative dispatcher runs at 120Hz

omb authoritative game loop SHALL 以120Hz執行`State::tick()`，讓gameplay input apply、host-side pending input drain、script dispatch、outcome processing與team frame cadence對齊。以tick數表示real-time interval的constants SHALL 依120Hz重新換算，以保留原本秒數語意。

每個secure authoritative tick SHALL 在Team 1與Team 2 frame都成功進入reliable outbound queue後才完成。Queue backpressure可以延長該tick，但 SHALL 記錄deadline miss；持續超過configured watchdog時 SHALL 安全終止match，不得丟frame或降級legacy protocol。

#### Scenario: server TPS constant is 120
- **WHEN** 檢查`omb/src/main.rs`
- **THEN** authoritative loop的`TPS`為120
- **AND** `Clock::new`使用`1.0 / TPS`

#### Scenario: second-based intervals keep same wall-clock duration
- **WHEN** 檢查state hash、snapshot與visibility diff intervals
- **THEN** state hash interval仍代表約10秒
- **AND** snapshot interval仍代表約30秒
- **AND** visibility diff interval仍代表原本設計的wall-clock cadence，而不是因120Hz變成4倍頻繁

#### Scenario: Outbound backpressure延長tick但不遺失frame
- **WHEN** secure match的outbound queue暫時滿載
- **THEN** authoritative tick等待Team 1與Team 2 frame可靠入queue
- **AND** deadline metric反映實際延遲
- **AND** team sequence沒有缺口
