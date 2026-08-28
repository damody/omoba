## ADDED Requirements

### Requirement: 每隊使用獨立外部 replica runtime

系統 SHALL 為 Team 1與Team 2各啟動一個獨立`omoba-client-runtime` OS process。每個process SHALL 各自持有secure V2 session、filtered Specs world、ScriptRegistry、dispatcher、replica ID map、remembered cache、pending queue及mutable runtime state，且 MUST NOT 與另一隊共享這些物件。

#### Scenario: 兩隊runtime使用不同PID與world
- **WHEN** 啟動雙隊secure session
- **THEN** Team 1與Team 2 runtime PID不同
- **AND** 兩者各自建立並持續step自己的filtered Specs world

### Requirement: Bootstrap與tick皆fail closed

Runtime SHALL 綁定固定player/team並以共用allowlist建立空filtered world，再從`TeamGameStart`與Reveal baseline建立entity。Wrong team、wrong epoch、unknown schema、sequence gap、hidden target或secure V2 downgrade MUST 拒絕整個frame並要求安全replay/rebase；不得套用部分frame或回退global snapshot。

#### Scenario: Sequence gap不會部分套用
- **WHEN** runtime收到跳號的`TeamTickFrame`
- **THEN** 該frame的transition、input及effect都不套用
- **AND** session進入安全replay/rebase流程

### Requirement: Runtime使用共用deterministic pipeline

Runtime SHALL 在PreStep套用Reveal、Hide、Forget與dependency closure，再注入accepted input/effect，以`global_seed + tick`建立tick-local RNG，執行共用phases並計算pre-repair hash。Server correction SHALL 擁有最終權威，但runtime MUST 先記錄原始divergence才套repair/replace/rebase。

#### Scenario: 故意修改後由server修復
- **WHEN** test-only fault修改Team 1已揭露component
- **THEN** Team 1 pre-repair hash先回報mismatch
- **AND** server correction使Team 1重新收斂
- **AND** 該checkpoint不被記為原始parity pass

### Requirement: Renderer生命週期不擁有replica

Renderer斷線時runtime SHALL 在bounded grace內繼續接收及step team frames；renderer重連 SHALL 從latest presentation恢復且不得觸發server filtered rebootstrap。Server斷線時runtime MUST 停止simulation並回報安全終止狀態，不得自行推算未收到的世界。

#### Scenario: Renderer重啟不重設world
- **WHEN** Team 1 renderer退出後重新啟動
- **THEN** Team 1 runtime PID與replica tick持續前進
- **AND** 重連renderer取得latest presentation而不建立第二個world
