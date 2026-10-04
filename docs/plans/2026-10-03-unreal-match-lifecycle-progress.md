# 單路完整對局與 filtered lifecycle 進度

## 計畫與決定

1. 先讓正式 input 的完整單路對局逐 tick 對照雙隊 disclosed hash，包含死亡、重生、XP、勝負與終局凍結。
2. 結算邊界明確化：可見 progression／普攻 clock 與單路 Damage 使用權威結算；不公開敵人私有 target、不裝 MobaMatch、不使用 ComponentRepair。owner script／cooldown 必須正式重演。
3. 真實雙 Unreal opt-in 四槽驗收後推進對局，只提交正常 input；測試器不得修改 HP、強迫勝負或取得 hidden world。要求兩隊 Playing 死亡正倒數、不同 hero generation 身分重生、同 winner 與終局後 120 ticks 三方 hash。
4. 重新建置所有 Rust consumers／script DLL／bridge／Unreal，再做 MCP 與雙程序驗收。錯誤保留於防錯紀錄 E080、E081。

## 已驗證

- `single_lane_full_match_filtered_replay_covers_respawn_and_finished_freeze`：1618 ticks、3234 steps，Finished winner Some(0) at tick 1604；death／respawn [7,1]，終局 15 ticks 玩法凍結，所有雙隊 canonical hash 完整相等，零 ComponentRepair。
- 新增正式 120Hz `single_lane_production_full_match_filtered_lifecycle`：10698 ticks、21394 steps，Finished winner Some(0) at tick 10684，death／respawn [7,1]，相同完整 hash／終局凍結要求通過；兩種 profile 的測試均改要求兩隊死亡與重生，不只任一隊。
- core 304；base_content 最新全套 64；client-runtime 41 lib＋3 bin；omobab 139 pass／1 ignored；bridge 41 pass／1 ignored。Lua observation：movement 6／ability 7／parity 3／lifecycle 8 scenarios 通過。
- Unreal full build 通過，codegen content hash de9c7fcfc98d6479；bridge stage SHA-256 b4315ec4166e9b5eb10a34a6ac98d74773efc3b150e40cbda19a45a19697e67c。

## 真實 Unreal 驗收

- 新 Editor PID 19428：NativeVisual 同 Editor 兩輪各 7/7 通過。第一輪真實雙 Unreal run 1791035563 的四槽／死亡重生通過但無終局，240 秒 timeout；失敗與五程序清理保留於 E082。補上可見進攻技能後再測，未先宣稱通過。
- 最新 Editor `Om.Runtime.GameplayInputSurface` 1/1、PIE native smoke 與 stop confirmation 通過；第二輪真實雙 Unreal run 1791036154 使用相同 240 秒門檻。
- run 1791036154 **success=true**：兩隊四槽各 4/4，正值冷卻與獨立 input ID Applied；team 1 deaths／respawns 6／6，team 2 3／2（終局時仍死亡，不繼續重生）；HUD Finished 分別觀測 tick 18425／18421，均 winner_team=1。
- gate 雙隊三方 pre/post hash 169／203 PASS、零 FAIL、last_tick 均 18600，超過最後 Finished +120。UNVERIFIED 不當 PASS；完整檢查 expected／external pre/post 與 observer frame hash，不只比較前後自己。
- 新工具 `tools/lua/lua.exe scripts/tests/ue_match_lifecycle_acceptance.lua target/interactive-runs/interactive-ue-1791036154` 重新讀取原始日誌與全部 checkpoint 嚴格驗證後，保存精簡可版控證據至 `openspec/changes/build-unreal-rust-moba-framework/evidence/unreal-match-lifecycle/interactive-ue-1791036154.json`。原始失敗與成功 run 都保留。
- 清理後的全檔重驗為 170／204 PASS、零 FAIL、last_tick 18720（比 gate 多最後一個 checkpoint）；與 gate 的 169／203 不混淆。保存工具亦已負向測試：拒絕失敗 run 1791035563，且不產生其 success evidence。
- 驗收後 Get-Process 核對 server／兩 runtimes／兩 game instances 均退出，只剩 Editor PID 19428；stage SHA-256 再核對一致。root／omfue scoped diff --check 通過。IPC disconnect 10054 是有序關閉 renderer 後的預期紀錄，不當玩法失敗。

## 重跑入口與限制

以 PowerShell 只設定入口 env（工作流仍由固定 Lua 執行）：`OMOBA_RELEASE=0`、`OMOBA_SKIP_BUILD=1`、`OMOBA_SKIP_UE_BUILD=1`、`OMOBA_UE_SMOKE_SECONDS=240`、`OMOBA_UE_ABILITY_SMOKE=1`、`OMOBA_UE_MATCH_SMOKE=1`；執行 `tools/lua/lua.exe scripts/run_2player_ue.lua --single-lane`。skip build 僅適用已完成全量建置且 stage hash 相同；改源碼後先 `scripts/build_ue_moba.lua --full --ue-root D:/UE5.8`，server／runtime 亦需新建置。一般啟動不設定 smoke env。

- Headless Push／Guard 是 fixture Bot，不是五位置正式視野 Bot；Unreal opt-in API smoke 不是物理 QWER 按鍵測試。
- 選角、商店、金錢／裝備、三路／野區、正式 Bot／三英雄／100 場、兩台 LAN／完整 UI 尚未完成；OpenSpec 維持 17/30，不以此里程碑勾選整項。
