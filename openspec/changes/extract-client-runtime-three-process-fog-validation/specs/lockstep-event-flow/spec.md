## ADDED Requirements

### Requirement: External runtime與server observer消費相同team frame語意

對每個team sequence，server SHALL 將同一份正式encoded team frame語意提供給對應external runtime與server-local observer。Observer MUST 經過與wire decode等價的邊界，不得直接讀canonical world補齊未投影component。兩隊可同時完成，arrival/completion order MUST NOT 影響frame bytes、authority或repair decision。

#### Scenario: Observer不能取得額外canonical資料
- **WHEN** Team 1 frame未揭露某Team 2 component
- **THEN** Team 1 observer與external runtime都無法從frame取得該component
- **AND** observer hash不得以canonical world補值

### Requirement: Team outbound queue滿載時阻塞authoritative tick

Authoritative server送往兩隊的secure team frame queue SHALL 是bounded。任一必要team frame無法安全入列時，server MUST 阻塞authoritative tick，而不得丟棄frame、跳過sequence、只讓另一隊前進或降級傳輸。阻塞與解除 SHALL 產生不含hidden payload的diagnostic。

#### Scenario: Queue滿載不造成sequence gap
- **WHEN** Team 1 outbound queue已滿且下一個authoritative frame必須送出
- **THEN** authoritative tick等待queue可用
- **AND** Team 1與Team 2都不跳過該tick sequence

### Requirement: Secure flow不夾帶legacy global state

Secure team session SHALL 只接收filtered bootstrap、team frames、必要control與server correction。它 MUST NOT 接收global snapshot、legacy完整`TickBatch`或可推回hidden entity的render event；協商或runtime錯誤時 MUST 終止或安全rebase，不得降級。

#### Scenario: Capability downgrade fail closed
- **WHEN** peer無法維持secure V2 selective capability
- **THEN** server終止該secure session或要求相容reconnect
- **AND** 不送出global snapshot或legacy完整world事件
