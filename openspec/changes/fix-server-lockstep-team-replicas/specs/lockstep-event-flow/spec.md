## ADDED Requirements

### Requirement: Team frame經由單一可靠送出邊界
Secure V2 team frame SHALL 先可靠進入broadcaster-owned outbound queue，再由broadcaster把同一份encoded bytes分送給team-bound network sessions與對應server observer。Observer bootstrap與frame delivery不得依賴任一玩家session成功接收。

#### Scenario: 玩家未連線時observer仍收到frame
- **WHEN** match已啟動Team 1 observer但沒有Team 1玩家session
- **THEN** Team 1 encoded frame仍進入可靠outbound queue
- **AND** Team 1 observer持續收到並驗證相同frame

#### Scenario: Network與observer使用相同bytes
- **WHEN** broadcaster處理一個Team 2 frame
- **THEN** Team 2 sessions與Team 2 observer收到源自同一encoded `Arc<[u8]>`的內容
- **AND** observer不重新encode projector frame

