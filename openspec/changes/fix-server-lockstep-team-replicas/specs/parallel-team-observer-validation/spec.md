## ADDED Requirements

### Requirement: Team 1與Team 2使用獨立validation thread
Server SHALL 為固定Team 1與Team 2各建立一條validation thread。每條thread SHALL 獨占該隊Specs world、script runtime、RNG、input queue與replica mapping。

#### Scenario: 兩隊同時執行相同tick
- **WHEN** Team 1與Team 2都收到replica tick T
- **THEN** 兩條worker可以同時執行tick T
- **AND** 任一worker不需要等待另一worker完成才開始

#### Scenario: 單隊失敗不刪除另一隊world
- **WHEN** Team 1 worker需要rebootstrap
- **THEN** Team 2 worker繼續處理自己的連續frame
- **AND** Team 2 world、sequence與coverage不被重設

### Requirement: 跨隊完成順序不得影響結果
Observer coordinator SHALL 以`team_id`、`replica_tick`與`team_sequence`關聯report，不得依channel arrival order修改authoritative state、frame bytes或repair decision。

#### Scenario: 反轉worker完成順序
- **WHEN** 測試讓Team 2先完成tick T，再改成Team 1先完成tick T
- **THEN** 兩次run產生相同expected hash、repair decision與後續encoded frames

### Requirement: Observer消費實際送出的encoded bytes
Network broadcaster SHALL 把相同的encoded `Arc<[u8]>`分送給team sessions與對應team validation worker。Observer SHALL decode wire bytes，不得直接讀取projector內部frame或authoritative Specs world。

#### Scenario: Wire corruption可由observer發現
- **WHEN** fault fixture在broadcaster與observer之間破壞encoded frame
- **THEN** observer拒絕該frame並記錄coverage或decode failure
- **AND** 不會因projector內部資料仍正確而誤判通過

### Requirement: Observer coverage必須誠實記錄
Worker queue overflow、decode failure、sequence gap或rebootstrap期間的sequence range SHALL 標記為unverified，且 MUST NOT 計入verified frame count。

#### Scenario: 單隊queue overflow
- **WHEN** Team 1 validation queue滿載但Team 2正常
- **THEN** Team 1記錄精確coverage gap並進入filtered rebootstrap
- **AND** Team 2 verified sequence持續前進

### Requirement: Server authority recovery
Observer mismatch SHALL 回報first divergent tick、team、sequence、expected hash、observed pre-repair hash與安全component path。Server SHALL 決定repair、replace、rebase或safe termination。

#### Scenario: Repair後重新驗證
- **WHEN** server對單component mismatch送出較新authority revision的repair
- **THEN** 對應team worker套用server值
- **AND** 下一checkpoint重新計算pre-repair hash

