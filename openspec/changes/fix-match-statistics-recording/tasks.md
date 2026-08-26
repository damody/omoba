## 1. Snapshot 戰績資料

- [x] 1.1 在共用 render snapshot schema 加入 `match_kills`，從 `MatchKillCounter` 複製並保持舊 snapshot consumer 相容。
- [x] 1.2 將 `match_kills` 傳入 `omfx` sim runner snapshot，補 extraction 不修改 ECS counter 與相同 state 值一致測試。

## 2. Profile 結算與安全持久化

- [x] 2.1 在 `omfx` 建立獨立 match statistics profile helper，支援 legacy/default 欄位、未知欄位保留、saturating add 與 monotonic highest round。
- [x] 2.2 實作同目錄 temporary file 加 platform-safe replacement；錯誤包含 result/path/cause，且失敗不破壞原 JSON。
- [x] 2.3 補 victory、defeat、abandoned、legacy JSON、KP 保留、overflow 與 replacement failure unit tests。

## 3. Frontend session lifecycle

- [x] 3.1 新增 per-session tracker，僅在成功進入 gameplay 後標記 started，並從 snapshot 更新 peak round 與 match kills。
- [x] 3.2 在 terminal snapshot 記錄 `Victory`／`Defeat`，所有其他正常 teardown 將未結束 session 分類為 `Abandoned`。
- [x] 3.3 將一次性結算接到 `shutdown_game_session`：先停止 runner／lockstep 與 owned backend，wait 後 reload-merge-save profile，再 refresh UI cache。
- [x] 3.4 補 duplicate terminal、terminal 加 teardown、重複 shutdown、返回標題、Ctrl+Escape、application shutdown 與 startup failure tests。

## 4. Backend ownership 收斂

- [x] 4.1 將 `omb` 兩條 game-end handling path 共用化，保留 victory／defeat KP 發放但移除四個戰績 mutation 與 kill-counter reset。
- [x] 4.2 補 backend regression tests，驗證 game end 只改 KP、victory bonus 正確且 profile 戰績保持不變。

## 5. 驗證與收尾

- [x] 5.1 執行 `cargo test --manifest-path omoba-core/Cargo.toml`、`cargo test --manifest-path omb/Cargo.toml -p omobab` 與相關 `omfx` package tests。
- [x] 5.2 執行 focused lifecycle reproduction，驗證前端先結束並關閉落後 backend 時仍只記一場，勝敗與退出分支皆正確。
- [x] 5.3 執行 `git diff --check` 與 `openspec validate fix-match-statistics-recording --strict`，確認沒有提交 profile temporary file、log 或 build artifact。
