# MOBA 正式技能升級：60Hz 增量驗收

## 計畫與決定

1. 使用既有 UpgradeAbility，不建立第二套技能系統。Rust 權威要求 Playing、未暫停、存活且屬於正式 roster，槽位合法、有 SP、rank 合法且未達 Lua max_level；缺 metadata 不以預設 5 級放行 MOBA。非單路舊行為保留。
2. 保留 vertical slice 四招初始 rank1／SP0。Lua 目前沒有每級英雄等級門檻，不硬編英雄聯盟門檻、不用 R 名称判 ultimate。之後需資料化解鎖規則与 UI。
3. 可見敵人升級不能靠公開敵方 input 重演。新增 CommittedAbilityRanks(kind25) 四槽 little-endian i32，僅可見 actor；filtered 先驗全部 rank／compiled max_level／空槽，再原子更新 Hero，不扣 SP、不重置 CD、不用 ComponentRepair。
4. RendererInput tag19 AbilityUpgradeIntent → Rust input bridge → 原 PlayerInput UpgradeAbility。IPC 驗 owner、epoch、slot、非零 request；同 runtime 1024 筆有界 request 去重，重送／衝突不再配置 authority input ID。不是跨 runtime 重連 journal，也不宣稱混版兼容。
5. Unreal 共用 WorldBridge 的 SubmitOwnedAbilityUpgrade 與 Controller Ctrl+Q/W/E/R 分支，只送 configured owner／slot，不預測 rank／扣點。原 QWER 施法保持；Ctrl 分支不再同時施法。沒有新增每英雄 C++ 或 Blueprint graph。
6. 真實 KCP fixture 只經普通 RendererInput 使用戰鬥賺到的 SP，無注入 XP／傷害／rank。第二輪刻意同一 request 送兩次，wire verifier 要求只接受一次。

## 已驗證

- core333、base_content92、server154（1 opt-in ignored）、runtime60（6 opt-in ignored）、bridge52（1 opt-in ignored）全部通過；Fyrox `cargo check -p omfx --tests` 通過。base 包含正式升級／SP不足連續輸入／死亡重生保存、11種非法條件不改 Hero、60Hz40ticks／80雙隊steps逐步 canonical hash／zero repairs。
- rank fact 原子／絕對值／重送冪等測試保留 cooldown 與 SP；可見／隱藏 actor 投影測試不洩漏敵方輸入。
- 第一輪 KCP run1791091143：3247筆 wire／IPC核對；player1 tick1015提交→1017 rank2 SP0，player3 997→999。每人原 upgrade input3一次，victim0次。四 PID39872／63404／38096／36064另外 inspect 全部退出。
- 最終去重 KCP run1791091387：3950筆 snapshots（1381／1334／1235），rank／XP／SP守恆／Gold／KDA／公開scoreboard精確核對原 wire。player1 tick1000提交→1009 rank2 SP0，player3 1015→1025；各同 request 送兩次但 accepted upgrade input3各一次，敵 input 不披露，actor有值／target無值／payload確實 slot0 upgrade。兩隊各10 checkpoints、player3獨立9 checkpoints至1200，各2個 post-kill parity；四 owned PID66556／95928／87736／61572另驗退出。
- 最終 OmGameEditor full build成功，Unreal Editor73092曾啟動，MCP readiness與11個既有BP compile全部通過；報告 `target/blueprint-validation-runs/compile-1791091510/report.json`。
- staged bridge SHA256 `e08d2abc6abe37e0b06b54ab9be90c695cbd8ef47df68887cd9a728c581b381c`；codegen11files／15Lua inputs／content_hash de9c7fcfc98d6479 check通過。C ABI未改，新 input intent 的 protobuf source 與既有 generated fallback 同步。

## 限制與下一步

- 本輪尚無 Unreal Ctrl鍵操作、升級後HUD畫面、升級後技能傷害／冷卻數值的實戰驗收。新增 GameplayInputSurface reflection斷言已編譯，但未宣稱 automation執行通過。
- full workflow完成後 Editor73092退出，om.log記錄正常shutdown；後續MCP list curl7因此未連線，不能推論插件崩潰，也不以舊automation代替本輪結果。
- 正式 Lua rank解鎖英雄等級規則、UI可升級提示／不可升級理由、跨runtime重連request保障仍需補齊。OpenSpec 5.3、4.1與6.2保持未勾選，整體19/30，不將此次增量當完整MOBA框架完成。
- 60Hz指權威／filtered simulation profile，不是GPU render FPS或大規模效能達標；120Hz依使用者要求不作本輪驗收目標。
- 問題及避免方式記於 `docs/plans/unreal-moba-error-register.md` E118；未commit、push或清理使用者工作樹。
