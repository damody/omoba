## MODIFIED Requirements

### Requirement: PlayerInput 端到端流程

Secure fog模式的gameplay input SHALL 遵循：renderer把UI事件編成presentation IPC input；`omoba-client-runtime`驗證player owner、格式、disclosure epoch與target membership；runtime透過secure lockstep session送server；`omb`排入target tick並在`player_input_tick`做authoritative驗證與ECS處理；accepted input隨team frame回到runtime並由filtered world於排定tick套用；renderer從後續presentation觀察結果。Runtime rejection與server rejection SHALL 可區分、MUST NOT panic，且server結果永遠優先。

#### Scenario: 右鍵移動走完整外部runtime路徑
- **WHEN** Team 1玩家在renderer右鍵點擊合法位置
- **THEN** omfx送出MoveTo IPC message而不直接修改英雄
- **AND** Team 1 runtime轉送secure input
- **AND** server在authoritative tick套用後，Team 1 filtered replica與presentation顯示己方英雄移動

#### Scenario: 嘗試控制敵方英雄被拒絕
- **WHEN** Team 1 renderer送出owner為Team 2英雄的input
- **THEN** runtime拒絕且不洩漏hidden target資料
- **AND** server仍會拒絕任何繞過runtime的請求

## ADDED Requirements

### Requirement: Input處理不受presentation backpressure阻塞

Renderer input SHALL 使用獨立低延遲ordered channel進入runtime，runtime到server的secure input SHALL 不等待presentation snapshot或renderer consumed acknowledgement。Presentation滿載只能丟棄舊snapshot，MUST NOT 丟棄已接受的gameplay input或critical result。

#### Scenario: Renderer落後時MoveTo仍準時送出
- **WHEN** presentation latest-wins slot持續被覆寫
- **AND** 玩家送出MoveTo
- **THEN** input仍立即進入runtime與server target-tick排程
- **AND** critical acceptance/rejection不被snapshot覆蓋
