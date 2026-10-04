# 單路 MOBA 閉環：5.1／5.2 驗收

## 結果與邊界

已完成 opt-in 單路權威玩法與可重跑的 headless 對局。OpenSpec 由 15/30 推進至 17/30；完整 Unreal + Rust 框架仍未完成。

這輪沒有修改 Unreal 資產、Blueprint 或 C++，沒有啟動新 Editor／PIE、重建 bridge 或重用舊 UE 報告當成新玩法證據。既有 `run_ue.bat`／TD 啟動未改成此模式。`omb::State` 已接 begin／commit hook，但現有正常 Story 不會自行安裝 MobaMatch；正式 server 選擇此模式、filtered replica 同步與 Unreal 可玩流程是下一輪工作。

## 問題與決定

1. 現有 Creep 是 TD 沿路／漏怪邏輯，不會形成對抗兵線。新增 opt-in lane AI，使用同一正式 ECS 的 Unit／Creep／Faction／Pos／CProperty；不建立第二套玩具 world，也不更改 TD path。兩隊定時各生成三隻兵，每隊一塔一基地。
2. 英雄死亡直接刪除，沒有重生。提交 Death 時先保存 Hero／Inventory／Gold，按 fixed elapsed 安排出生；保留等級／技能／裝備，重生換 entity generation，移除舊 buffs、移動／攻擊狀態，死亡期間 cooldown 繼續前進。HP／傷害使用 Lua generated stats 與等級成長。
3. 基地不能只靠 Bot 自律不攻擊。傷害提交邊界檢查敵塔先決條件；直接技能／攻擊也不能繞過。雙基地同 tick 死亡算 draw；結束只發一個 game.end，後續 step 不再修改玩法。
4. Bot 不能直接改 ECS 血量或跳過輸入驗证。Push／Guard fixture 提交與網路玩家相同的 PlayerInput，經 pending routing、命令、四技能 handler 與正式結算。單路為無遮擋／無 stealth 測試幾何，非五位置或完整視野 Bot。
5. 首場 4500 ticks 沒結束；實際診斷為塔已倒、基地滿血，Push 一直 AttackMove 清兵。修正目標策略：合法範圍內優先建築，敵塔倒下後 AttackTarget 基地；相同命令不重送，避免打斷前搖。沒有強制扣基地 HP／設定胜方或延長時間上限來通過。
6. hero_tick 依 wall-clock 50ms 決定是否攻擊，平行 outcomes 次序也未固定。移除負載 gate，以 fixed delta 與 entity-order reduce 執行，同距離目標加 ID／generation tie-break。
7. lane 攻擊首版直接送 Damage Outcome，沒走護甲計算。改成既有 DamageInstance::new_attack → damage_tick → Outcome／DirectCombat facts。SimulationDriver 修正 script rng_seed 為 MasterSeed，逐 tick drain／驗證 facts，防止整場累積未讀事件。
8. MOBA 測試模式只允許 roster 的移動、攻擊、技能、升技與物品操作；拒絕 TD StartRound、建塔、debug 與調速等輸入。暫停由權威 host 的 GamePause 控制，沒有把玩家 TD 暫停權限帶進來。
9. Replay digest 包含 lane 狀態／計時器、英雄進度／cooldown、命令、HP／屬性、裝備／Gold、Facing、MoveTarget、在途 projectile 與未結算 Outcome；JSON map canonical 排序。這是單路診斷雜湊，不冒充 selective-replica wire hash 或完整 future-mode world hash。

每次錯誤與防重犯規則集中在 `unreal-moba-error-register.md` 的 E059–E063。

## 主要實作

- `omoba-core/src/runtime/native/moba_match.rs`：SingleLaneConfig、bootstrap、生命周期、兵塔 AI、基地防護、Bot formal input、replay digest。
- `omoba-core/src/runtime/native/simulation_driver.rs`：headless begin／commit、正確 script seed、committed facts 回傳。
- `omoba-core/src/runtime/native/game_processor.rs`：死亡保存、duplicate death 防重、死亡英雄不能施法、傷害防護。
- `omoba-core/src/runtime/native/tick/hero_tick.rs`：決定性戰鬥排序與不依賴 wall-clock。
- `omb/src/state/core.rs`：同一權威 tick 的 opt-in hooks；沒有變更既有 Story 選擇。
- `omb/src/bin/moba_headless.rs`：載入真實 DLL、跑至勝負、逐 tick 重播並輸出 report。
- `scripts/run_moba_headless.lua`：固定 Lua 5.4 建置／執行入口，任何階段失敗即回傳非零。
- `scripts/base_content/src/single_lane_match_tests.rs`：十個新增驗收；test-only kcp feature 不引入 script ABI 的重依賴。

## 最終驗證

完整指令：

```text
D:\code\omoba\tools\lua\lua.exe scripts/run_moba_headless.lua
D:\code\omoba\tools\lua\lua.exe scripts/run_moba_headless.lua --profile 120 --report D:/code/omoba/omb/target/moba-headless/report-120hz.json
cargo test --manifest-path scripts/Cargo.toml -p base_content --lib --quiet -- --test-threads=1
cargo test --manifest-path omoba-core/Cargo.toml --lib --quiet
cargo test --manifest-path omb/Cargo.toml -p omobab --lib --quiet
cargo check --manifest-path omoba-core/Cargo.toml --lib --no-default-features --quiet
```

| 實際 DLL 對局 | 15Hz 測試 profile | 120Hz 正式 profile |
| --- | --- | --- |
| seed／winner | 1／team 0 | 1／team 0 |
| finish tick | 3155 | 15480 |
| 遊戲秒數 | 210.3330078125 | 129 |
| 波數 | 27 | 16 |
| 死亡／重生（team 0,1） | [11,1]／[11,1] | [7,3]／[7,2] |
| 四技能正式輸入數 | [26,23,3,14] | [13,13,2,8] |
| committed combat facts | 864 | 953 |
| game.end 次數 | 1 | 1 |
| 逐 tick replay 通過 | 3155/3155 | 15480/15480 |

15Hz digest：`17bdfe4dda55ab48910f68dbf8d68fba763515e3f3612aa5f04297426a798f21`。

120Hz digest：`cdec51179cbffe68a80726649a7b61f8d82cc4ed6161bb0ab38f47d0498f300b`。

不同 profile 的離散戰鬥結果並不宣稱一致；逐 tick replay 在各自相同 profile／版本／種子／輸入下驗證。正式遊戲使用 120Hz，15Hz 只供 headless 測試。

報告位於 `omb/target/moba-headless/report.json` 與 `report-120hz.json`，包含所有 tick digests，按專案規則不提交 target 產物。採用實際 release `scripts/target/release/base_content.dll`，不是只用測試內 manifest。

- scripts/base_content：59 passed（含十個新增單路測試）。
- omoba-core：296 passed。
- omobab library：135 passed，1 既有 ignored。
- core 無 default features 編譯通過。
- 新增測試涵蓋正式四技能 HP 結算、護甲／戰鬥 facts、基地先決條件、死亡與單次重生、warmup／暫停、結束凍結、同 tick draw、非法 config atomic 拒絕、缺 script 拒絕、非 roster／TD 控制拒絕、完整 Bot match 與逐 tick replay。
- 既有 td_rounds dead-code 與 backend protoc fallback 警告仍存在，不宣稱零警告。

## 接下來的明確驗收

1. 將單路模式接入正式 server 選擇／啟動與 filtered projection；Unreal 不建立第二份 gameplay world，收斂 4.1／4.4。
2. native 地圖、英雄／兵／塔／基地、四招與生命／重生／勝負 HUD，驗收 Unreal 玩家實際從開局玩到結束（6.1／6.2 的最小閉環）。
3. 第二英雄維持只改 Lua／Rust／美術，重跑生成與 Editor 配方；用完整對局證明，不只 codegen check（6.3）。
4. 再加 5.3 經濟／商店、5.4 三路野區、5.5 完整 Bot、LAN／reconnect 與效能驗收。未完成項目仍留在 checklist，不以 headless 閉環冒充完整框架。
