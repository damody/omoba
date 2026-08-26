## ADDED Requirements

### Requirement: Team-shared visibility authority

omb SHALL 以 team 為 visibility authorization 與 projection 單位。同一 team 的 `VisionSource` SHALL 合併成 shared team view；player session SHALL 只能接收其綁定 team 的 view。Client viewport、camera position 與 player name cache MUST NOT 作為 gameplay visibility authority。

#### Scenario: 隊友視野共享

- **WHEN** team A 的 unit A1 看見 enemy E，而同隊 player A2 的 unit 未直接看見 E
- **THEN** E 進入 team A 的 resolved visibility set
- **AND** A1 與 A2 的 session 都能在相同 effective tick 收到 E 的 disclosure
- **AND** 其他 team 不會因 team A 的 vision source 收到 E

#### Scenario: viewport 不改變 gameplay visibility

- **WHEN** player 移動或縮放 client camera，但 authoritative unit 與 vision source 都未變更
- **THEN** server resolved team visibility 不變
- **AND** 不產生 reveal、hide 或 forget transition

### Requirement: Deterministic visibility rule resolution

omb SHALL 以 fixed-point、tick-based 規則解析 `ReplicationScope`、`VisionSource`、`StealthProfile`、`VisibilityOverride` 與 `RememberPolicy`。Resolution order SHALL 為 `ServerOnly` deny、force-hide、`Public`/force-show、`OwnerTeam`、`TeamVision` geometry/detection；同 priority tie SHALL 依 stable rule ID 決定，不得依賴 insertion order、wall clock 或 floating-point nondeterminism。

#### Scenario: Override precedence deterministic

- **WHEN** 同一 entity 同時具有未過期 force-hide 與較低 priority force-show
- **THEN** entity 對目標 team 保持 hidden
- **AND** 不同 thread scheduling 或 insertion order 產生相同結果

#### Scenario: Override 於指定 tick 過期

- **WHEN** force-show 的 `expires_tick == T`
- **THEN** 規則在 T 之前有效
- **AND** 從 T 開始只依剩餘 deterministic visibility rule resolve

### Requirement: Scheduled visibility transition

Raw visibility change SHALL 先進入 candidate state。Default `visibility_commit_delay_ticks` SHALL 為 3，允許範圍 2–4。Candidate 到期且條件仍成立時，omb SHALL 在 effective tick 從當下 authoritative state 擷取 fresh baseline 並 commit transition；條件提前失效時 SHALL cancel candidate。

#### Scenario: Reveal 使用 effective tick fresh baseline

- **WHEN** entity 在 tick T 首次滿足 team A visibility，且持續到 `T + D`
- **THEN** server 在 `T + D` commit `RevealEntity`
- **AND** baseline 反映 `T + D` 的 position、component revision 與 safe dependency
- **AND** baseline 不使用 tick T 的 stale state

#### Scenario: 短暫可見不觸發 reveal

- **WHEN** entity 在 candidate 到期前再次 hidden
- **THEN** server cancel reveal candidate
- **AND** team stream 不包含該 entity 的 identity 或 baseline

### Requirement: Remembered presentation 不參與 simulation

`RememberPolicy` SHALL 支援 `Forget`、`LastKnown` 與 registered custom presentation。Hide 後的 remembered data SHALL 已去敏感化並只存在 render cache，MUST NOT 參與 simulation、targeting、collision、input validation 或 team hash。Entity 在 fog 中死亡時，remembered record MUST NOT 因 server-only death 自動消失。

#### Scenario: LastKnown ghost 不影響 gameplay

- **WHEN** disclosed enemy 以 `LastKnown` policy 進入 fog
- **THEN** client deterministic world 移除該 entity
- **AND** renderer 可顯示最後已知 presentation
- **AND** player 無法以該 record 作為 target input

#### Scenario: Fog death 不洩漏

- **WHEN** remembered enemy 在 hidden 狀態死亡，且 death 尚未成為 team-known event
- **THEN** remembered presentation 不自動消失
- **AND** team frame 不透露該 death

### Requirement: Cross-visibility projection policy

每個 gameplay system 與 script-visible action SHALL 聲明 visible-visible、hidden-visible、visible-hidden、hidden-hidden projection policy。只有 deterministic evaluation 所需 dependency 全部可安全 disclosure 時 client 才能 local simulate；否則 server SHALL 產生 sanitized external effect。缺少 policy SHALL 是 blocking integration error。

#### Scenario: Hidden attacker 傷害 visible target

- **WHEN** hidden attacker 在 authoritative world 傷害 team A 已 disclosed target
- **THEN** team A frame 包含 target、amount、damage class、effective tick 與允許的 attribution
- **AND** frame 不包含 hidden attacker replica ID、canonical ID 或 position

#### Scenario: 缺少 projection policy 被拒絕

- **WHEN** content 或 gameplay action 沒有完整四象限 projection policy
- **THEN** startup/content validation 失敗並指出 policy ID
- **AND** secure match 不會以預設 full disclosure 繼續
