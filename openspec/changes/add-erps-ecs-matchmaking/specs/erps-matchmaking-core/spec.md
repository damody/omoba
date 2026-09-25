## ADDED Requirements

### Requirement: ERPS 使用獨立 Specs ECS authority
ERPS SHALL 在獨立 process 的單一 Specs `World` 保存 player、party、ticket、proposal、match 與 game server authority state。gRPC handler MUST 透過 bounded command queue 提交 mutation，MUST NOT 直接修改 `World`。

#### Scenario: RPC mutation 於 tick 邊界套用
- **WHEN** 多個 RPC task 同時提交 party 與 queue mutation
- **THEN** ERPS 依 deterministic command order 在 tick 邊界套用 mutation，且 RPC task 不直接取得 ECS mutable storage

### Requirement: 候選搜尋可平行且 commit deterministic
ERPS SHALL 依 region、mode 與 Elo bucket 將候選搜尋分 shard 平行執行，並 SHALL 於單一 commit 階段原子重驗 ticket claim、party revision 與 player state。相同設定、seed 與命令序列 MUST 產生相同 proposal、roster 與 placement。

#### Scenario: 相鄰 bucket 不重複使用 ticket
- **WHEN** 同一 ticket 同時出現在 owner shard 與相鄰 bucket halo 的可行候選中
- **THEN** commit 最多接受一個包含該 ticket 的 proposal，其他候選完整失敗且不產生半完成 match

#### Scenario: Worker 排程不改變結果
- **WHEN** 使用不同 worker thread 數執行相同 seed 與命令序列
- **THEN** proposal、roster、team assignment 與 stable ID tie-break 結果完全相同

### Requirement: 支援三種 party-safe 配對模式
ERPS SHALL 支援 1v1、5v5 與八人自由混戰。1v1 MUST 只接受單人 ticket；5v5 MUST 接受 1～5 人 party 且組成兩支恰好 5 人隊伍；八人自由混戰 MUST 接受 1～4 人 party 且湊滿恰好 8 人，進場後 MUST 產生八個單人 team。

#### Scenario: 5v5 party 不拆分
- **WHEN** bounded matching 選出多個 party 組成 5v5 候選
- **THEN** 每個 party 的所有成員都位於同一支五人隊伍，且沒有成員被省略、複製或移到另一隊

#### Scenario: 八人 party 一起入場但各自成隊
- **WHEN** 八人自由混戰由大小不超過 4 的多個 party 湊滿 8 人
- **THEN** 所有 party 成員都在同一 match roster 中，且 roster 包含八個各一人的 team

### Requirement: Elo 搜尋範圍隨等待時間擴張
ERPS SHALL 對每種模式使用獨立 Elo，預設起始值為 1000，並 SHALL 依設定的階梯週期擴大 ticket 搜尋範圍直到設定上限。Party effective rating SHALL 包含平均 rating、party size adjustment 與 internal spread adjustment；超過 `max_party_rating_spread` 的 ranked party MUST 被拒絕。

#### Scenario: 長時間等待擴大候選範圍
- **WHEN** ticket 跨過一個設定的等待階梯且尚未達最大範圍
- **THEN** 下一次候選 snapshot 使用擴張後的 Elo 範圍且不超過最大值

### Requirement: 匹配品質兼顧等待與平衡
1v1 SHALL 優先相容範圍內 Elo 差最小的雙方。5v5 SHALL 以 bounded search 評分最久等待、兩隊 effective Elo 差與隊內離散；前 60 秒兩隊 party 人數多重集合 MUST 完全相同。任一參與 ticket 等待至少 60 秒後 MAY 接受不同結構，但 2／3／4／5 人 party MUST 分別加上 +5／+10／+20／+30 優勢 Elo，1 人 party 加 0；結構不同時，優勢較低隊伍的原始平均 Elo MUST 至少高出兩隊依成員人數加權的優勢差。八人模式 SHALL 評分全場 Elo range、離散與最久等待。所有同分情況 MUST 使用 stable ID tie-break。

#### Scenario: 60 秒內只配對相同 5v5 結構
- **WHEN** 5v5 候選所有 ticket 等待未滿 60 秒，且隊伍由 `4+1` 或 `2+2+1` 組成
- **THEN** `4+1` 只能對上 `4+1`，`2+2+1` 只能對上 `2+2+1`；party 順序不影響判定

#### Scenario: 60 秒後跨結構需補足組隊優勢
- **WHEN** 任一 ticket 等待至少 60 秒，`4+1` 與 `2+2+1` 可組成 5v5 候選
- **THEN** `2+2+1` 隊伍的原始平均 Elo 至少比 `4+1` 高 12 分，且兩隊 effective Elo 仍符合搜尋範圍

### Requirement: 每模式更新 Elo rating
ERPS SHALL 以可設定 K-factor 更新 1v1 與 5v5 rating。八人自由混戰 SHALL 將每名玩家與其他七名玩家依最終名次做 pairwise 勝、平、負比較，彙總後 MUST 套用單場最大變動限制；原 party MUST NOT 共享自由混戰結果。

#### Scenario: 5v5 依每位玩家已完成場次選擇 K-factor
- **WHEN** 同一場 5v5 同時包含未滿新手場次門檻與已達門檻的玩家，game server 回報合法的全隊勝負
- **THEN** 每位玩家皆以對手隊伍原始平均 Elo 計算期望分數，但分別使用自己所屬的新手或已定級 K-factor 更新分數；結算後各自的完成場次加一

#### Scenario: 八人同名次視為平手
- **WHEN** 八人自由混戰結果包含兩名同名次玩家
- **THEN** 兩人的相互 pairwise 結果為平手，且每人的最終 rating delta 不超過設定上限
