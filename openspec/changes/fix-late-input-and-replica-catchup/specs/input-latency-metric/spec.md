## ADDED Requirements

### Requirement: Late input 與 replica catch-up 可被量測

Input latency diagnostics SHALL顯示server input outcome、late-by ticks、client received/applied tick lag、inbound backlog、catch-up batch size與checkpoint queue depth。Diagnostics SHALL只存在transport或runtime外圍，不得進入deterministic gameplay state。

#### Scenario: Server retarget late MoveTo
- **WHEN**server將late MoveTo retarget到下一tick
- **THEN**log或metric包含input ID、original/effective/current tick與late-by ticks
- **AND**client最終能從applied input metadata配對該input

#### Scenario: Runtime 進入 catch-up
- **WHEN**client inbound queue含多個連續 TeamTickFrame
- **THEN**runtime記錄batch處理數、耗時與批次後剩餘lag
- **AND**正常無backlog時不產生每tick warning spam

#### Scenario: Input 超過一秒仍未套用
- **WHEN**input超過grace且沒有出現在authoritative applied metadata
- **THEN**server與client diagnostics能判斷是RejectedLate或最後已知phase
- **AND**HUD與log不得只顯示舊的低p99而隱藏pending input
