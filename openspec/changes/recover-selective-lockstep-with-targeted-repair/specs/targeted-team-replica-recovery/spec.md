## ADDED Requirements

### Requirement: 可恢復的 replica 錯誤不得直接終止 client

Client runtime 遇到 `UnknownEntity`、可恢復的 disclosure epoch 差異、hash mismatch 或無法補齊的 sequence gap 時 SHALL 保留最後已提交狀態並進入 `AwaitingRepair`，不得關閉 renderer、runtime 或連線。

#### Scenario: 增量幀引用未知單位

- **WHEN** team frame 在套用期間引用不存在的 replica entity
- **THEN** client 保留失敗前的 world、replica tick 與 sequence
- **AND** client 送出一次 repair report 並等待 server 回覆
- **AND** renderer 繼續顯示最後一個安全狀態

### Requirement: Frame apply 必須是原子的

Selective replica SHALL 以 staging 狀態套用完整的 `pre-step`、`step` 與 `post-step`，只有全部成功才提交 world、tick、sequence、authority revision 與 memory directives。

#### Scenario: post-step repair 失敗

- **WHEN** `pre-step` 成功但 `post-step` 回傳可恢復錯誤
- **THEN** live replica 不包含該幀任何部分變更
- **AND** 修復完成後可以重試同一 team sequence

### Requirement: Server 決定最小安全修復

Server SHALL 將 repair report 視為不可信提示，並依 authoritative team projection、當下 visibility、disclosure epoch 與 allowlist 選擇 `ComponentRepair`、`EntityReplace`／dependency bundle 或 filtered rebase。Client SHALL NOT 指定 canonical entity 或要求任意 component。

#### Scenario: 單一可見 component 不一致

- **WHEN** server 證明一個仍公開 entity 只有 allowlist component 不一致
- **THEN** server 只送該 component 的 `ComponentRepair`
- **AND** response 不包含其他 entity 或不可公開 component

#### Scenario: 回報視野外 replica ID

- **WHEN** client 回報 server 無法在該 team 當下 disclosure 中驗證的 replica ID
- **THEN** server 不回傳該 entity 的 baseline、存在性或 canonical identity
- **AND** server 使用 generic denial 或 filtered rebase 恢復 session

### Requirement: 修復必須逐級升級且有界

Authority recovery SHALL 追蹤 request ID、失敗 sequence、嘗試次數與 progress token。沒有進展的 repair MUST 升級至 entity replacement，再升級至 filtered rebase；只有安全驗證失敗或恢復上限耗盡才終止該 client。

#### Scenario: ComponentRepair 沒有收斂

- **WHEN** 同一 sequence 套用 component repair 後仍產生相同差異
- **THEN** server 不重複無限傳送相同 repair
- **AND** server 升級至下一個恢復層級

### Requirement: Projector 不得建立已知無效的 entity lifecycle 組合

Team projector SHALL 移除同幀 Hide／Forget 後仍指向該 entity 的 repair／replace。Reveal baseline 已代表 post-tick 狀態時，client SHALL NOT 在同一 tick 再套用該 entity 的 Movement public event。

#### Scenario: Reveal 與 Movement 發生在同一 tick

- **WHEN** entity 進入視野且同一 frame 包含其 Movement public event
- **THEN** replica 使用 reveal safe baseline 作為該 tick 結果
- **AND** replica 不會對尚未加入 Specs mirror 的 entity 套用 Movement event
- **AND** observer 與 external runtime 都不會回傳 `UnknownEntity`

### Requirement: 修復診斷不得洩漏 canonical identity

Server-side 診斷 SHALL 記錄 team、session、request ID、tick、sequence、phase、operation、replica ID、epoch、revision、tier、bytes 與結果。Client-visible 訊息與 log MUST NOT 包含 canonical ID 或其他 team 的 disclosure 資料。

#### Scenario: 記錄 UnknownEntity

- **WHEN** replica 在任一 apply phase 回傳 `UnknownEntity`
- **THEN** server log 能指出失敗 phase、operation 與 opaque replica ID
- **AND** client log 不包含 canonical ID
