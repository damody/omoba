# 單路 MOBA 正式 server／filtered runtime 整合

## 本輪決定

1. 保留 Story／TD 預設；以 server.MATCH_GAMEPLAY_MODE="single_lane" 選用正式單路模式，限定 secure V2、team 1/2 各一名已認證玩家。
2. 使用 State 既有權威 World，不額外啟動另一份 MOBA 世界，也不生成 Story 場上實體。server ECS Tick 必須與 transport local_tick 一致。
3. side 0/1 與 authenticated wire team 1/2 分離；英雄、兵、塔、基地使用真正 VisionSource／OwnerTeam。缺席的既有公開實體遵守原 remember policy 退休，不洩漏隱藏死亡。
4. NPC AI 僅在權威端執行。兵線 PreStepMovement 在 replica gameplay 前套用，NPC 傷害以可見目標的 sanitized external result 結算。
5. 公開 Warmup／Playing／Finished 的 delta、elapsed、phase；暫停／暖機時 client 同樣不推進玩法。沒有改成 warmup=0 或全圖可見。
6. 可見敵英雄只發布 MovementPriority 布林狀態與最終可見姿態；不發布敌方目的地、輸入、命令 target ID。filtered World 有靜態 script registry，但没有權威 MobaMatch resource 或隱藏實體。
7. 正式入口拒絕 TD tower／StartRound／speed 等單路不支援控制。固定 Lua 測試比較 authority、observer、external runtime 的 pre/post-repair hash，不把 client 相互一致或 repair 後恢復當成功。

## 已驗證

- `cargo test --manifest-path omoba-core/Cargo.toml --lib`：299 passed。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib`：62 passed。
- 900 ticks 真實 Warmup／host Pause／兵線傷害，以及 2400 ticks 雙英雄移動，所有 checkpoint 對照權威 hash 通過。
- 空 filtered World 有塔、升級、技能 registries 且零實體；MovementPriority 只發布可見 actor 的一 byte 布林值，隱藏來源不發布。
- core `--no-default-features` 編譯通過；root／omb CRLF-aware diff check 通過。
- backend lib 138 passed／1 既有 ignored；client runtime 38 lib＋2 bin passed。
- 固定 Lua 真實 KCP：`target/interactive-runs/moba-runtime-1791026421/moba-runtime-smoke-report.json` success=true、cleanup_verified=true。team 1 safe tick 1991、16 checkpoints（最後 1920）；team 2 safe tick 1099、7 checkpoints（最後 1080）。三方 pre/post hash 與 observer／external frame hash 全相同，沒有把 mismatch 關掉。
- 前一成功 run `1791026264` 亦保存；失敗 run 不刪除、不重寫 success。E064–E070 記錄各原因與修正。

## 啟動與下一段驗收

所有入口使用固定 Lua，不新增根目錄 bat 或 PowerShell fallback：

```text
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_runtime_smoke.lua
D:\code\omoba\tools\lua\lua.exe scripts\run_moba_headless.lua --profile 120
D:\code\omoba\tools\lua\lua.exe scripts\run_2player_ue.lua --single-lane
```

最後一個入口已接入本次獨立 server 配置；沒有 --single-lane 時維持原 FOG／Story 流程。它不是已通過完整 Unreal 對局的宣告。

OpenSpec 維持 17/30；4.1／4.4 仍需四技能／死亡重生／終局在正式安全 IPC 的完整測試，以及 Unreal 單份玩家模擬驗收。6.2 的選角、HUD、商店、小地圖、計分板與結算 UI 未完成。三路、經濟、五位置 Bot、LAN 與美術替換完整驗收仍未完成。優先把單路 HUD／四技能／重生／結算接成一個完整 Unreal 垂直切片，不回到僅 FOG 移動成功的迴圈。

## 最新重建／重播

- 固定 Lua `run_moba_headless.lua --profile 120` 在最終碼 exit 0；實際 release script DLL，seed 1，15324 ticks／16 波／909 combat facts，四槽施放 [13,13,2,8]，死亡 [7,3]、重生 [7,2]，side 0（wire team 1）勝，game.end 一次，15324 ticks 全部 replay 一致。
- 固定 Lua `build_ue_moba.lua --full --ue-root D:/UE5.8` exit 0：生成 11 檔／13 Lua inputs、content hash de9c7fcfc98d6479，OmGameEditor 編譯成功。bridge staged SHA-256 `d5a78c82185905b61c692b7a4945c2152c30073818583a7df989cd9ca6733e38` 一致。
- 專案 restart 只關閉符合 omfue/om.uproject 的舊 Editor PID 68560；正常關閉逾時後工具完成該 PID tree 的 force close，不終止其他專案 Editor。新 Editor PID 20264；BpGeneratorUltimate HTTP MCP 30000 ready。
- 同一 Editor 兩輪原生／Blueprint 相容性測試各 7/7 passed；PIE native／remembered ghost／截圖／停止清理 smoke success=true。報告位於 `omfue/Saved/McpAutomation/NativeVisual/report.json` 和 `omfue/Saved/McpAutomation/pie-smoke-report.json`。
- 正式单路五程序入口：`interactive-ue-1791026989/unreal-ipc-smoke-report.json` success=true、gameplay_mode=single_lane。雙 UE 各觀察 15 個 presentation frames、own-only 起始視野與實際 rendered movement；consumed snapshot sequence 1105／1545，雙隊 safe tick 5321。UE 自行發出移動，不由 test runtime 代送。
- 該 run 完整三方 checkpoint records：team 1／2 各 46 筆 PASS、零 FAIL（以換行完成的 records 檢查，不把 kill 時未完成尾行當成證據）。這不是全場終局／所有技能覆蓋。
- 精確 PID 清理核對：server 84124、runtime 75864／9768、UE game windows 97440／58572 均已退出。Editor PID 20264 保留供 MCP 編輯，PIE 已停止；未關閉其他 Unreal 專案。
- 最終 bridge stage gate 仍為上述 SHA-256。沒有 commit／push 或清除既有使用者改動。

## 下一個實作順序

1. 四槽技能的正式 UE input/result 與 filtered gameplay 結算測試，不只 headless skill casts。
2. 從同一份安全 projection 提供英雄 HP／MP、冷卻、單路 phase／時間、死亡重生與終局 HUD；不讓 renderer 重建權威規則。
3. 確認 UE 從暖機玩到結算後，再補經濟／商店／三路，不先擴張新的傳輸或 reconnect 架構。
4. 以實際 content／地形替換目前 presentation 的 FOG fixture 視覺資料；可見 actor 安全性由 server 已驗證，但視覺遮擋與正式單路地形不能宣稱已驗收。

## 後續里程碑更新

上述順序中的 owner HUD 安全鏈與四技能雙隊 filtered 結算已補強，最新證據見 [單路 HUD 進度](2026-10-03-single-lane-hud-progress.md)。最新正式双 UE run 1791030915 通過 owner HUD／移動／consumed，三方 56／73 PASS、零 FAIL；這不包含由 UE 四技能操作玩到終局。

下一段優先驗證 UE 正式四槽輸入／結果、死亡重生與終局 HUD；current mana／經濟／選角／商店／小地圖／計分板、正式地形與 LAN 仍待實作或驗收。OpenSpec 維持 17/30，不把本輪部分里程碑勾成整項完成。
