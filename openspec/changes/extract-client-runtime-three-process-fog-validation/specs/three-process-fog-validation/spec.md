## ADDED Requirements

### Requirement: 三process與五process拓撲可重複啟動

Headless安全驗證 SHALL 啟動一個authoritative server及Team 1、Team 2兩個外部runtime；視覺驗證 SHALL 再啟動兩個renderer-only omfx。Launcher MUST 記錄PID、executable SHA-256、port、player/team binding與ready狀態，且只清理由本次run驗證過的PID。

#### Scenario: 五個process彼此獨立
- **WHEN** 執行視覺驗證模式
- **THEN** server、兩個runtime與兩個renderer共有五個不同PID
- **AND** renderer退出不會停止server或任一runtime

### Requirement: Demo重現完整戰爭迷霧行為

驗證 SHALL 使用`FOG_2TEAM_DEMO`的100個普通單位與另外兩名英雄、圓形視野、10×10 fog grid、至少16個patrol unit、Forget、LastKnown、tree circle與polygon occlusion。人工右鍵與scripted MoveTo SHALL 走相同renderer→runtime→server路徑；己方英雄永遠可見且可移動。

#### Scenario: 非對稱視野產生不同畫面
- **WHEN** 兩名英雄位於不同視野與遮擋區域
- **THEN** Team 1與Team 2的visible entity集合及畫面不同
- **AND** 離開視野的Forget單位消失，LastKnown只留下sanitized ghost

### Requirement: Runtime random sentinel證明資料隔離

每次run SHALL 由server產生不同128-bit Team 1/2 sentinel並注入test-only hidden fixture。驗證 MUST 掃描各session raw/decoded packet、filtered world、runtime process memory、presentation、renderer process memory及玩家可見log。任何另一隊hidden sentinel命中、dump失敗或證據缺失 MUST 使verdict為FAIL或UNVERIFIED。

#### Scenario: Team 1所有邊界都不含Team 2 sentinel
- **WHEN** Team 2 sentinel仍在Team 1視野外
- **THEN** Team 1 packet、world、runtime memory、presentation、renderer memory與log掃描皆為零命中
- **AND** scan evidence記錄PID、binary hash、canary hash、工具與排除理由

### Requirement: 每隊保存pre-repair分歧並執行三方post-repair收斂驗算

Server SHALL以兩條獨立observer thread同時處理兩隊，並以`(team_id, replica_tick, team_sequence, authority_revision)`保存三方pre-repair hash及比較server expected、server observer與external runtime三方post-repair hash。Completion order MUST NOT改變authoritative state、encoded bytes或repair decision。

#### Scenario: 無fault時三方一致
- **WHEN** checkpoint coverage完整且未注入fault
- **THEN** 每隊保留pre-repair結果，且三個post-repair hash完全相同
- **AND** 任一缺report、coverage gap或worker crash不會被標記PASS

### Requirement: Evidence產生單一blocking verdict

每次run SHALL 產生manifest、canonical與filtered timeline、packet/memory/presentation scan、disclosure matrix、checkpoint hash、lifecycle、同步截圖及`verdict.json`。只有所有blocking gate通過時verdict才可為PASS。

#### Scenario: 漏拍截圖不能通過
- **WHEN** 預定tick的任一隊同步截圖不存在
- **THEN** comparison輸出非PASS verdict
- **AND** 明確列出缺少的artifact
