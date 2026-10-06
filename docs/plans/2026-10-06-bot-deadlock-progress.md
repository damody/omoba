# 通用執行預算與目標無進展診斷（2026-10-06）

本批取代舊第 4 批的退讓重試。不改 Bot 策略、遊戲規則或成功條件，不 commit。

## 決定

1. `--max-game-seconds` 預設 600，`--stall-game-seconds` 預設 300。兩者都必須是 60..=3600 的明確整數，stall 不得超過 max。profile 乘上秒數使用 checked multiplication。時間只是 headless 執行預算。
2. 自然 `Finished`、恰好一個 `game.end`、combat facts 大於 0、完整 replay digest 一致，仍然是唯一成功條件。plan-only 不是對局。逾時與 objective stall 都是失敗，不能把逾時寫成 success。
3. 目標進展只看公開塔／基地的真實 HP 下降、退休，或層替換。waves、英雄移動、擊殺、營地 respawn 與私有命令都不算，也不注入 world，不把命令送回 Bot。開始後連續 stall 秒沒有這類進展才失敗，原因裡帶最近一次 progress tick。樣本最多 10 筆，保留最近間隔與最後一筆，不做每 frame 全量快照。
4. plan-only、成功、失敗都用 create_new。既有報告或配對診斷（成功 JSON、失敗 JSON、非法 JSON）在開跑前拒絕，不刪歷史。單場 Lua 預設新目錄；明示 `--report` 原樣使用但拒絕重複與已存在輸出。批次仍用自己的新目錄。
5. 批次可傳上述預算，預設仍是 600／300，並寫進 batch summary。verify 必須與請求預算一致；較長預算的終局不能通過較短請求。60Hz、十 Bot、guard、replay 閘門保持。

## 錯誤

- 先前把 seed1 的 600 秒逾時當成可能死局。`omb/target/moba-headless-repair-seed1/attempt2.failure-samples.json` 與 `attempt3.failure-samples.json` 的頂層 `samples` 仍顯示後期塔退休與基地 HP 下降。詳見 `unreal-moba-error-register.md` E307。
- 本批 Lua 測試第一次把 seed 1 fixture 拿去對請求 seed 4，預算斷言沒跑到。Rust `write_failure` 初次編譯 E0521。兩者都已修正後重跑，不是遊戲失敗。

## 驗證

- `cargo test --manifest-path omb/Cargo.toml -p omobab --bin moba-headless --features compiled-content-only -- diagnostic_tests`：6 passed，0 failed。
- `D:\code\omoba\tools\lua\lua.exe scripts\tests\moba_headless_batch_test.lua`：9 passed，沒有啟動對局。
- `cargo build --manifest-path omb/Cargo.toml -p omobab --bin moba-headless --features compiled-content-only --release`：exit 0，43.84 秒。只改 headless；`rustc 1.95.0`，沿用既有 `scripts/target/release/base_content.dll`（2026-10-06 10:13，640512 bytes），沒有另建 DLL。
- 一場真實 seed1，exit 0，wall 136.10 秒：

```text
omb\target\release\moba-headless.exe --role-plan omb\target\moba-headless-batches\1791252707-1\role-plan.json --profile 60 --seed 1 --scripts-dir scripts\target\release --max-game-seconds 1800 --stall-game-seconds 300 --report omb\target\moba-headless-budget-seed1\result.json
```

報告 `omb/target/moba-headless-budget-seed1/result.json`：success true、match_played true、profile_hz 60、seed 1、map `three_lane_training`、bot_mode `committed_role_plan`、defender_policy guard、winner_team 0、finish_tick 62951、game_seconds 1049.1826171875、end_events 1、committed_combat_facts 10210、replay_verified_ticks 62951、final_digest `80eede71fccbfd9ebf27d54e91775d11238f68b0a02c08a5ff18fe7a1de6bd94`、last_progress_tick 62951、max_game_seconds 1800、stall_game_seconds 300、max_ticks 108000、stall_ticks 18000。沒有 failure paired 檔。stdout：`MOBA headless PASS: winner=Some(0), tick=62951, waves=131, replay=62951 ticks`。

這是 1800 秒預算內的自然終局，不是 600 秒批次通過，也不是 100 場。`git diff --check` 對本批檔案 exit 0。HEAD 仍是 root `c5211e8311cdca867841fcc1494899634b9d17b1`、omb `578f475f97520127d1cc7cc930e7d7786f1cea0f`、omfue `1e29a27c0f58ff798f82fa13f592a53a967f78ac`。

## 剩餘

- 預設批次仍是 600／300。seed1 的自然終局在 tick 62951，超過 36000，所以預設 600 秒批次仍不會把這一場算成功。若進度持續，該失敗應是 execution budget，不是 objective stall；本批沒有重跑 600 秒。
- 100 場、Unreal、LAN、stage 都沒做。Bot 既有 dirty 保留，沒有改策略。
- OpenSpec 5.5 不勾。

## 語意邊界（不重跑對局）

- 目標槽存在改為 generation 非零。ID 0、generation 1 的 HP 下降與退休算進展；generation 0（含 dead tuple，以及 ID 非零但 generation 0 的 HP 雜訊）不算；同一 ID 換新 generation 算層替換。有界與 stall 測試保留。
- 報告新增 `winner_side`（內部 side）。`winner_team` 依 `config.teams` 映射，與 `moba_match.rs` game.end 相同；draw 為 null；side 2 回錯誤且不寫報告。正式 `[1, 2]` 不會再把 side 0 寫成 team 0。既有 `omb/target/moba-headless-budget-seed1/result.json` 仍是 winner_team 0，沒有改寫。
- Lua verify 拒絕 team 0 與不在 role plan 的 `winner_team`；兩隊與真正 draw 允許。其他 60Hz／十 Bot／guard／replay／預算閘門仍在。fixture 補 team 1／2。
- 覆寫測試不再 `remove_dir_all`。唯一 `create_dir` 預留，結束時只刪本次具名檔，再 `remove_dir` 空目錄。
- 驗證：diagnostic_tests 7 passed、0 failed（編譯 5.05 秒，測試 0.02 秒）；Lua batch 10 passed；release host exit 0，11.01 秒，`rustc 1.95.0`，未重建 DLL。未跑 seed1。詳見 E311。OpenSpec 5.5 不勾。
