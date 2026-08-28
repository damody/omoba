## ADDED Requirements

### Requirement: 所有secure session流量經delay proxy
Netem scenario SHALL 讓join、bootstrap、team frame、hash report、input及recovery的所有client／server UDP datagram經過指定team route，且 MUST 不修改production KCP與secure session timeout。

#### Scenario: 延遲下完成secure bootstrap
- **WHEN** 兩個runtime在任一合法20～100 ms profile下啟動
- **THEN** 兩隊都完成secure join、team binding、filtered bootstrap及ready marker

#### Scenario: Production timeout保持不變
- **WHEN** launcher按最大RTT延長ready evidence等待時間
- **THEN** KCP與secure session使用與direct模式相同的production timeout設定

### Requirement: Lag期間只推進完整filtered frame
Client runtime SHALL 只在完整frame barrier接受連續team sequence後推進filtered Specs world；lag或亂序期間 MUST 不輸出部分simulation frame或預測未收到的server資訊。

#### Scenario: 暫時亂序後恢復
- **WHEN** natural-reorder使後一個UDP datagram先抵達但KCP最終恢復完整frame
- **THEN** runtime依team sequence恰好套用每個frame一次，且沒有永久gap或duplicate apply

#### Scenario: Renderer等待新frame
- **WHEN** runtime尚未收到下一個完整team frame
- **THEN** renderer最多保留最後一個安全presentation，不得自行推進英雄或敵方狀態

### Requirement: Disclosure transition不因亂序倒退
系統 SHALL 維持單調disclosure epoch與永久retired replica ID規則，較舊Reveal MUST 不得在Hide或Forget後恢復entity。

#### Scenario: 舊Reveal晚於Hide到達
- **WHEN** transport亂序使包含較舊epoch Reveal的資料晚於已接受Hide
- **THEN** runtime拒絕epoch倒退且entity維持hidden

#### Scenario: 舊Reveal晚於Forget到達
- **WHEN** replica ID已被Forget並永久退休
- **THEN** 任何較舊資料不得重新建立該replica ID或target lookup

### Requirement: 延遲下server-authoritative input
合法MoveTo SHALL 經proxy送到server並只在server acceptance指定的authoritative tick影響owning hero；hidden或stale target input SHALL 被拒絕，且server結果 MUST 優先於runtime預期。

#### Scenario: MoveTo round trip
- **WHEN** renderer送出合法MoveTo且RTT介於20～100 ms
- **THEN** server接受input、runtime收到對應結果，英雄只在排定tick移動且renderer沒有optimistic simulation

#### Scenario: Hidden target仍被拒絕
- **WHEN** client在延遲期間提交已離開當前disclosure epoch的target
- **THEN** server拒絕input且結果安全回傳，不洩漏target最新狀態

### Requirement: 三方checkpoint在延遲下收斂
Server expected、server observer與external runtime SHALL 依checkpoint key配對，不得依report arrival order配對；短暫network lag MUST 不被誤判為simulation divergence。

#### Scenario: External hash較晚抵達
- **WHEN** external runtime hash report因proxy delay晚於另一隊或server observer
- **THEN** server保留pending checkpoint並在相同key到齊後比較

#### Scenario: Blocking checkpoint完成
- **WHEN** scenario結束前所有必要report都已抵達
- **THEN** 每個checkpoint保留pre-repair診斷且三方post-repair hash收斂

### Requirement: 延遲不得破壞戰爭迷霧安全邊界
每隊packet、filtered world、runtime memory、presentation、renderer memory及玩家可見log SHALL 不包含對方hidden sentinel或未揭露canonical資訊。

#### Scenario: 非對稱視野延遲驗證
- **WHEN** 兩隊在不同delay stream下觀看同一場戰局
- **THEN** 各隊只顯示自身當前安全視野，兩隊screenshot hash不同且對方sentinel掃描為零命中

#### Scenario: Hide與Forget延遲抵達
- **WHEN** entity離開視野且對應transition經proxy延遲
- **THEN** client不預測其未來狀態，transition接受後下一個presentation依policy隱藏或移除entity

### Requirement: Delay scenario blocking verdict
Comparison工具 SHALL 僅在process、delay、sequence、checkpoint、input、fog、sentinel、queue及evidence gate全部通過時輸出PASS。

#### Scenario: Natural-reorder缺少實際亂序
- **WHEN** `natural-reorder` run的reordered datagram count為零
- **THEN** verdict為`UNVERIFIED`而不是PASS

#### Scenario: 任一安全gate失敗
- **WHEN** 出現wrong-team、wrong-epoch、permanent gap、duplicate apply、unintended rebase、sentinel hit或queue overflow
- **THEN** verdict為FAIL並指出個別blocking gate

### Requirement: 完整profile矩陣與soak
系統 SHALL 在所有實作與測試資產完成後，執行每profile 15秒smoke、5分鐘ordered／natural-reorder矩陣、非對稱team isolation、visual檢查及30分鐘profile切換soak。

#### Scenario: 三十分鐘soak通過
- **WHEN** scenario依序切換low、middle、high、bimodal及low profile並運行30分鐘
- **THEN** 兩隊保持0 permanent gap、0 unintended rebase、0 sentinel hit、queue在budget內且所有checkpoint完成

#### Scenario: 完整矩陣有失敗
- **WHEN** 任一profile或模式未通過
- **THEN** 變更不得標記完成，修正後必須重跑受影響scenario並最後重跑全部blocking gate
