# 真實 60Hz 三玩家網路擊殺／助攻

## 計畫與決定

延續 OpenSpec `build-unreal-rust-moba-framework` 的 5.3，補足前輪只有移動與 NPC 死亡的缺口。三個外部 filtered runtime 連到正式 KCP authority；test-mode＋OMOBA_COMBAT_SMOKE 才啟用 fixture。以披露資料提交一般 RendererInput MoveTo／AttackTarget，沿既有 intent queue／身分與目標驗證／KCP／PlayerInput／權威 ECS，不注入 HP、Damage、Gold、kill 或 assist。

三名英雄在 tick 360 移至單路中點、Y=900，遠離兵線。同隊 player 1／3 各一次普攻命令，player 2 只移動。擊殺者不寫死，但同隊總 kills=1、assists=1，victim deaths=1；結算後至少240 ticks，三名玩家各至少兩個獨立 post-kill parity checkpoints。

這是 runtime renderer-intent injection，不是 TCP renderer／Unreal 客戶端／實體鍵鼠驗收。

## 發現與修正

- 1791077835 建置失敗：MoveToIntent沒有queued。依實際schema移除，不新增fallback；尚無遊戲程序。
- 1791077851 普攻都到達authority，但只有player3造成擊殺，player1沒有助攻。hero_tick將windup／impact包在方向距離平方>0.01內：重合目標永遠不能出手。改零／近零方向保留當前facing，照常啟動攻擊；非零方向及所有敵我／HP／範圍／轉向驗證維持。新增方向回歸，保持重合站位重跑，不改站位避開bug。
- 錯誤、查找失誤與防重犯規則記錄於 `unreal-moba-error-register.md` E112。失敗report／capture保留，未重寫。

## 真實證據

- 修正後1791078015成功：tick1018，player1 0/0/1、player2 0/1/0、player3 1/0/0。原wire對IPC共4010 snapshots及逐筆金錢驗證通過。
- 最終1791078192成功且cleanup_verified=true：tick1019同樣分數；safe1279。雙隊各10 checkpoints至1200、第三玩家獨立9至1200；三人各2 post-kill checkpoints，沒有FAIL或修復後才一致。
- 最終原始wire／IPC verifier：player1 1356筆／tick1529，player2 1253筆／tick1524，player3 1220筆／tick1519，共3829筆。score每筆對照snapshot tick-1原frame，核對owner/team及所有metric隊伍隔離。
- 兩名攻擊者原始input_id=2各一次action_kind=3正式acceptance且actor／target存在，victim無攻擊；capture金錢每筆等於active整秒收入加Lua300 kill／100 assist，不重付，死亡後score與economy仍持久。
- 三次實際smoke共12個owned PID均清理且由固定Lua process.inspect獨立核對不存在；最終17624／64600／3536／31036。

## 建置與回歸

- core lib330（新增重合方向回歸）、base_content84、server lib154（1 ignored）、runtime56（5 opt-in ignored）＋main3、bridge51（1 opt-in ignored）＋legacy integration2（1 ignored）通過。server其它integration／doc ignored不算已執行。Fyrox check --tests通過。
- TD integration `td_autoplay_100` 三項通過，228.45秒；1–100波無畫面與observed完整對照（round ticks／ledger／state hash／cash／lives一致）。使用測試既有scripts/target/release內容DLL及本輪編譯core，未把它稱為MOBA／Unreal效能基線。
- Unreal build-only通過，ABI8未變、codegen --check通過（11 files／15 Lua inputs，hash de9c7fcfc98d6479）；同步更新bridge與base_content DLL。staged bridge SHA-256：72abf9a082f029205438e46aa182ed4729dd4a114f31c99b99ff37d13f3249f2。
- 本輪沒有啟動Editor、MCP、PIE或Unreal game client；前輪14/14 automation不當作這轮執行結果。

## 重現入口

```text
OMOBA_ROSTER_SMOKE=1
OMOBA_COMBAT_SMOKE=1
tools/lua/lua.exe scripts/run_moba_runtime_smoke.lua

OMOBA_SCORE_CAPTURE_ROOT=D:/code/omoba/target/interactive-runs/moba-runtime-1791078192
cargo test --manifest-path omoba-client-runtime/Cargo.toml real_three_player_owner_score_capture -- --ignored --nocapture
```

環境變數由呼叫環境設定；不新增PowerShell／Python工作流。capture未版控，預設ignored測試不假造測試來源。

## 剩餘

本輪封關網路擊殺／助攻及私有HUD／金錢管線，不是完整5.3或6.2。下一步實測多人Unreal KDA畫面與安全公開計分板，再處理完整選角／三路／Bot／LAN與效能驗收。整體維持19/30，不把上述局部測試勾選成完整框架。
