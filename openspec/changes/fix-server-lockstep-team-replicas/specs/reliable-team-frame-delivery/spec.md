## ADDED Requirements

### Requirement: Authoritative tick使用可靠阻塞enqueue
Authoritative tick SHALL 成功把Team 1與Team 2 encoded frame放入broadcaster-owned bounded queue後才能完成。實作 MUST NOT 忽略`try_send`失敗或靜默跳過team sequence。

#### Scenario: Queue暫時滿載
- **WHEN** reliable outbound queue暫時沒有容量
- **THEN** authoritative tick阻塞直到frame成功進入queue
- **AND** frame sequence保持連續且沒有frame被丟棄

### Requirement: Blocking邊界不等待下游完成
Blocking enqueue SHALL 只等待queue取得frame ownership，不得等待socket transmission、玩家ACK、observer simulation或hash comparison。

#### Scenario: Observer執行較慢
- **WHEN** Team 1 observer step耗時高於一個tick但outbound queue仍有容量
- **THEN** authoritative enqueue不等待Team 1 hash結果
- **AND** network broadcaster仍可送出已入queue的frame

### Requirement: Outbound watchdog安全終止
Queue backpressure持續超過configured watchdog時，server SHALL 安全終止secure match。Server MUST NOT 丟棄frame、跳過sequence或runtime downgrade至legacy protocol。預設watchdog SHALL 為5秒。

#### Scenario: Queue持續滿載超過watchdog
- **WHEN** outbound queue連續5秒無法接收下一個required team frame
- **THEN** server記錄deadline miss與safe termination reason
- **AND** secure match結束而不是繼續缺幀執行

### Requirement: 兩隊frame都是tick完成條件
每個authoritative tick SHALL 為Team 1與Team 2建立並可靠enqueue frame。任一required team frame未入queue時，該tick不得宣告delivery commit完成。

#### Scenario: Team 2 enqueue晚於Team 1
- **WHEN** Team 1 frame已入queue但Team 2 frame仍因backpressure等待
- **THEN** authoritative tick保持未完成
- **AND** Team 1 frame不得導致server跳到下一個authoritative tick

