## Why

本機前端在自己的 replica 判定勝負後會立即關閉 backend，但目前戰績只在 backend 收到 `game/end` 時寫入；兩端進度不同時，backend 會在落盤前被終止，導致勝利、失敗與中途退出都沒有正確累計。戰績必須改由實際擁有 session lifecycle 與本機 profile 的前端結算，才能保證每個已開始的 session 恰好記錄一次。

## What Changes

- 新增前端 per-session 戰績 tracker，保存是否已開始、是否已結算、最高回合、擊殺數與勝利／失敗／退出結果。
- 勝利增加 `games_played` 與 `wins`；失敗或中途退出只增加 `games_played`，三者都更新 `highest_wave` 與 `total_kills`。
- 所有 session teardown 路徑在停止 backend 後統一執行一次本機 profile merge，並以安全替換方式保存 `omb/player_profile.json`。
- 將 `MatchKillCounter` 以唯讀欄位帶入 render snapshot，避免以前端 entity removal 猜測擊殺。
- backend 的 game-end handler 保留 KP 發放，但停止修改四個戰績欄位，消除雙重計數與並行覆寫來源。
- 保持舊 profile JSON 的 serde default 相容；啟動失敗不計局，中途退出不發 KP。

## Capabilities

### New Capabilities

- `local-match-statistics`: 定義本機 session 的勝利、失敗、退出結算規則、一次性保證、snapshot 擊殺資料與 profile 持久化契約。

### Modified Capabilities

無。

## Impact

- `omfx/game/src/native.rs`：session lifecycle、terminal result、profile merge 與 UI cache refresh。
- `omfx/game/src/sim_runner.rs` 與 `omoba-core` snapshot：傳遞唯讀 match kill count。
- `omb/src/state/core.rs`、`omb/src/state/resource_management.rs` 與 knowledge profile helper：移除 backend 戰績 mutation，保留 KP。
- `omb/player_profile.json` schema 不新增必要欄位；既有 optional/default 欄位會在首次結算後正規化寫回。
- 不改變 protocol wire schema、對局模擬結果、英雄知識加成或中途退出的 KP 規則。
