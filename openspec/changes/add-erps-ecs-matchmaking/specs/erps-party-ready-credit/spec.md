## ADDED Requirements

### Requirement: Party 使用安全邀請與受限顯示名稱
ERPS SHALL 以短效 invite token 授權加入 party。Party name SHALL 只作顯示、不要求唯一，並 MUST 在 NFC normalization 後包含 1～24 個 Unicode letter／number 字元；CJK SHALL 合法，空白、標點、emoji、控制字元、零寬字元與 combining-only 字元 MUST 被拒絕。

#### Scenario: CJK 房名通過驗證
- **WHEN** leader 建立名稱為 `台灣第一隊5` 的 party
- **THEN** ERPS 接受正規化後的名稱，且其他玩家仍必須使用有效 invite token 才能加入

#### Scenario: 特殊符號房名遭拒
- **WHEN** leader 提交包含空白、底線、井字號或 emoji 的 party name
- **THEN** ERPS 拒絕 mutation 且 party revision 與既有名稱保持不變

### Requirement: Party mutation 使用 leader 權限與 revision
ERPS SHALL 只允許 leader 執行 enqueue、cancel、kick、rename 與 invite mutation，並 SHALL 使用 party revision 防止並行覆蓋。排隊、ready check 或 match 期間 MUST 禁止更名與 roster mutation。

#### Scenario: 過期 revision 不覆蓋 roster
- **WHEN** leader 以過期 party revision 提交 kick 或 rename
- **THEN** ERPS 回報 revision conflict 且不修改 party

### Requirement: 所有玩家個別完成 ready check
配對候選 SHALL 先進入 `AwaitingAccept`。每位玩家 MUST 在 server 提供的 authoritative deadline 前個別 `AcceptMatch`；預設期限為 15 秒。Leader MUST NOT 代替 party 成員同意。

#### Scenario: 全員同意後進入 placement
- **WHEN** proposal 的每名玩家都在 deadline 前接受
- **THEN** proposal 原子轉為 `AwaitingPlacement`，並且在此之前沒有正式容量 reservation

### Requirement: Ready 回覆冪等且綁定 proposal
`AcceptMatch` 與 `RejectMatch` MUST 包含 `proposal_id` 與 `request_id`。相同 request MUST 冪等；已取消或過期 proposal 的延遲回覆 MUST NOT 影響目前 ticket 或新 proposal。

#### Scenario: 舊接受訊息不接受新場次
- **WHEN** client 在第一個 proposal 取消後送達其延遲 `AcceptMatch`
- **THEN** ERPS 回報 stale proposal 且不變更新 proposal 的接受狀態

### Requirement: 拒絕與逾時套用個人信用處分
ERPS SHALL 維護 0～100、預設 100 的個人信用分。主動拒絕預設扣 2 分，期限內未回應預設扣 5 分；低於 60 MUST 暫停排隊。ERPS 或 game server 基礎設施失敗 MUST NOT 扣除任何玩家信用分。

#### Scenario: 未回應者停止配對
- **WHEN** proposal deadline 到期且一名玩家未回應
- **THEN** 該玩家 ticket 停止配對、信用分依 timeout policy 扣除，已同意玩家不被扣分

#### Scenario: Launch failure 不扣信用分
- **WHEN** 全員接受後 game server reject 或 instance ready timeout
- **THEN** 所有玩家信用分保持不變

### Requirement: Proposal 取消保留無責任玩家等待權益
Proposal 因拒絕或逾時取消時，已同意的單人 ticket與未受失敗成員影響的完整 party SHALL 保留原 `enqueued_at` 自動回 queue。包含拒絕／未回應成員的 party MUST 保留 roster 並進入 `NotReady`，MUST NOT 自動踢人或修改 roster。

#### Scenario: Party 失敗成員阻止自動重排
- **WHEN** party 一名成員拒絕而其他成員已接受
- **THEN** 整個 party 進入 `NotReady`，leader 必須移除失敗成員或等其恢復資格後才能再次 enqueue

### Requirement: Client 斷線具有 grace period
ERPS SHALL 在 client 斷線後保留預設 30 秒 grace period。期間 proposal、match 與 party MUST 可透過重連及 `GetState` 對帳；grace period 到期後未匹配 ticket SHALL 被取消，leader 斷線 MUST NOT 立即解散 party。

#### Scenario: Grace period 內重連恢復狀態
- **WHEN** client 在斷線後 30 秒內以有效 session 身分重連
- **THEN** ERPS 保留其 party、ticket 與待處理 proposal state 並允許 SDK 對帳

