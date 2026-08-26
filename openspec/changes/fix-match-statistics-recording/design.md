## Context

個人資料頁從 `omb/player_profile.json` 讀取 `games_played`、`wins`、`highest_wave` 與 `total_kills`。目前這四欄和 KP 一起在 backend 的 `game/end` handler 更新，但實際本機 session 由 `omfx` 管理：前端 replica 一旦從 snapshot 判定勝負，就會停止 sim runner、lockstep client，接著 kill 並 wait backend child。實際 log 顯示前端到達 40/40 時 backend 仍在第 23 關，因此 backend 尚未看到 `game/end` 就被關閉。

此外，中途返回標題與關閉應用程式沒有 `game/end`，現有架構本來就無法計入「玩過的遊戲」。同一份 profile 又同時被前端 UI 與 backend knowledge system 使用，必須避免兩端同時寫相同戰績欄位。

## Goals / Non-Goals

**Goals:**

- 每個成功進入 gameplay 的本機 session 在勝利、失敗或退出時恰好結算一次。
- 勝利增加場次與勝場；失敗、返回標題、Ctrl+Escape、應用程式 shutdown 增加場次但不增加勝場。
- 以玩家實際看到的 replica snapshot 更新最高回合與擊殺數。
- 保留舊 profile JSON 相容性、KP 與 hero knowledge 行為。
- profile 寫入失敗不得阻止 session teardown，也不得破壞原檔。

**Non-Goals:**

- 實作目前仍為 placeholder 的當前版本、CHIMPS、Deflation 或使用塔數統計。
- 中途退出發放 KP。
- 將本機 JSON profile 改成帳號服務。
- 強制前端 replica 與 backend 進度相同。

## Decisions

### 1. 前端是 match statistics 的唯一 mutation owner

`omfx` 建立並結束本機 session，也擁有玩家看到的終局狀態，因此由它在 teardown 統一結算。本機與 external-backend 模式皆遵守相同 ownership。Backend 的 game-end handler只發 KP，不再修改四個戰績欄位。

沒有選擇等待 backend game-end acknowledgement，因為目前兩端可能相差多個 round，等待不能保證得到與玩家畫面一致的結果，還會增加 protocol 與 timeout 狀態。沒有選擇 append-only journal，因為單機 JSON profile 尚不需要額外 recovery service。

### 2. 使用顯式 per-session tracker 保證 at-most-once

前端 tracker 保存：session 是否成功開始、是否已結算、最高觀察回合、最新 match kill count 與 `Victory`／`Defeat`／`Abandoned` result。只有成功 `mark_in_game` 後才設為 started；啟動過程失敗不計局。

Terminal snapshot 只設定 result，真正寫入集中在 `shutdown_game_session`。若 teardown 時尚無 result，指定為 `Abandoned`。結算旗標在嘗試落盤前先設定，讓重複 shutdown、terminal snapshot 加自動 teardown 或 Drop path 都不會重複加一。寫入失敗採 at-most-once，不在同一 process 靜默重試而冒雙計數風險。

### 3. Kill count 由 authoritative-replica snapshot 複製

在共用 render snapshot 加入唯讀 `match_kills`，由 ECS `MatchKillCounter` 複製。前端每次接受新 snapshot 時更新 tracker。這個欄位不回寫 simulation，也不加入 visual effect drain。

沒有選擇從 `removed_entity_ids` 推導，因為該集合同時包含擊殺、漏怪、projectile 與其他 despawn，無法正確分類。

### 4. Backend 先停止，frontend 再 merge profile

Teardown 依序停止 lockstep／sim runner、kill 並 wait owned backend，之後重新讀取 profile 再合併戰績。若 backend 已開始同步 KP 寫入，wait 讓該寫入先完成；frontend merge 會保留最新 `total_kp`、`spent_kp`、unlock 與 enabled 欄位。

External backend 不由前端關閉，但 backend 已不再寫戰績；frontend 仍以本機 session tracker 結算一次。

### 5. Profile 使用安全替換與 saturating arithmetic

舊 JSON 缺少戰績欄位時沿用 serde/default 0。Merge 對 counter 使用 saturating add，最高回合使用 max。寫入先完成 serialization，再在同目錄建立 temporary file，以 platform-safe replace 更新正式檔。失敗時保留原 profile 並記錄 result、path 與 error，teardown 繼續。

## Risks / Trade-offs

- [應用程式被作業系統強制終止，teardown 沒有執行] → 本 change 保證正常 UI exit／session shutdown，不宣稱可攔截 crash 或 kill -9；後續才需要 journal recovery。
- [Backend KP 與 frontend profile 同時寫入] → owned backend 先 kill/wait，再由 frontend reload-and-merge；backend 不再寫戰績。
- [External backend 同時寫 KP] → 戰績 ownership 仍唯一，但 KP 可能由遠端服務管理；frontend merge 必須保留所有未知 JSON 欄位，縮小覆寫範圍。
- [Snapshot schema 增欄影響 determinism] → 欄位只由既有 ECS counter 複製並測試 extraction 不修改 world。
- [落盤失敗造成少算一局] → log 明確報錯且保留原檔；選擇 at-most-once 避免同一 session 重複累計。

## Migration Plan

1. 擴充 snapshot 與 sim runner payload，加入 `match_kills` 及無 mutation 測試。
2. 新增獨立 profile merge／safe replace helper 與 legacy JSON、saturation、failure tests。
3. 導入 frontend session tracker，接上 start、snapshot、terminal detection 與全部 teardown path。
4. 移除兩個 backend game-end handler 的戰績 mutation，只保留 KP。
5. 執行 `omoba-core`、`omfx`、`omb` tests 與 focused lifecycle integration test。

Rollback 必須一起回復 frontend ownership 與 backend mutation；不可同時讓兩端寫戰績。

## Open Questions

無。退出視為 played 但非 win、退出不發 KP、mode-specific rows 不在本 change，均已確認。
