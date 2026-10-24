# 施法與整組召喚結算：最終驗收

完成 `complete-cast-settlement-transactions`。先前腳本成功後就扣費與發布成功提示，逐隻召喚結算仍可能拒絕後段單位。現在受支援召喚技能先檢查整組，再一起建立單位、結算法力與冷卻及發布成功提示；拒絕不生成、不扣費、不發SpentMana或成功提示。

共用 native `ScriptCastTransaction` 保存施法者、rank、陣營／可選玩家歸屬、預期法力、效果與成功提示。正式受管理施法及 legacy 召喚施法走同一結算入口，重新檢查生命、控制、phase、法力、rank及冷卻。整組召喚使用確定性的友方占位預檢、每隻不可變 stats 引用與獨佔World提交，最多128隻／512個召喚封套效果；普通大範圍技能不受這個512上限，也不掃描召喚占位。沒有英雄／營地名稱特判或ABI修改。

所有腳本法力回呼（含非施法與on_tick）改用 `ScriptManaCommit`：保存每個受影響實體的原始與最終法力池，先確認所有原始狀態再寫入，通知維持原始呼叫順序。這避免前一筆失敗後，後續絕對法力寫入仍夾帶其費用；任一法力擁有者不符時整批法力與通知都不寫入。採保守拒絕，不用退款補丁或猜測重算；新輸入／後續呼叫可依新狀態重試。

## 集中驗收

- 最終V3 Rust **1523成功、零失敗、9既有ignored**：core936、base_content301、bridge135＋1ignored、client151＋8ignored。不是所有workspace全面重跑。全base使用release，測試180.65秒。
- 新增6個真ECS測試：完整提交／重送與serde round-trip、12種後段或施法者拒絕、後續施法法力依賴、整組占位與確定性、非施法回呼不攜帶失敗費用、多擁有者法力原子接受與通知順序。
- 新增正式輸入＋60Hz腳本階段fixture，在handler成功後、結算前改變地形，確認整組拒絕保留法力／冷卻且無生成；成功對照為4隻。此測試明確是階段fixture，不冒稱完整driver時間推進。另全套實際Production60Hz driver的四rank召喚、pause生命期、法力、戰鬥與雙複本逐tick回放成功。
- 三個既有提示／rank／順序測試增加「結算前沒有managed成功提示」檢查，再經真process_outcomes檢查原本rank、policy、身份、冷卻與同tick唯一ordinal。唯一順序fixture按每次派發結算，使用其實際最大法力280。
- 正式backend及moba-config建置成功；**UE5.8建置、DLL部署與import library SHA一致**。
- **66項原生回歸成功，零Failed／NotRun**；保留4個既有RuntimePresentationReset warning。World另一次真正compiled-map export屬橋接測試的重執行，不重複增加1523計數。
- **29PNG**：25張與前批已審閱像素雜湊完全相同，回城、分層地圖、首殺及召喚姿勢4張差異已目視。地圖04:32與KDA保持正確；既有首殺圖下／右HUD裁切及部分英雄／focus名牌重疊仍保留限制。圖片證明呈現回歸；新權威行為由ECS與正式輸入測試證明。
- **3423來源／資產** exactset、SHA、size、mtime前後一致；**153份證據**逐byte與SHA獨立校驗成功。6最終worker、2歷史worker及watcher全部退休/reap；同一170秒dynamic health timer涵蓋三輪Rust與後續建置／原生／來源階段，13次採樣、最大171秒。timer亦已退休/reap，最終CIM沒有本輪Lua／Cargo／rustc／cl／link／Unreal／base測試程序。

## 原始失敗與修正

V1 core934成功；base298成功／3失敗，原因是舊測試在結算前便要求managed成功提示。保留7份凍結來源及全部失敗日誌；所有reader退休後才加強fixture並補上非施法法力依賴。V2完整1523成功／9ignored，作為快路徑優化前歷史；其後才發現一般施法不必建立全場占位，退休全部V2 reader、保留原檔後加上早退，以V3重跑全範圍。最終來源證據不混用V1／V2輸入。詳見[錯誤與預防](2026-10-23-cast-transactions-errors.md)與E1510。

## 範圍與接續

現有作者召喚陣形及施法者法力／冷卻／朝向是整組接受或拒絕。任意混合地形或生命變更的召喚副作用尚未支援，會整筆拒絕；沒有宣稱任意腳本效果都可回滾。顯式native raw ScriptSetMana保留相容用途，正式腳本法力交易不走此旁路。

一般持續引導／脈衝區域、更多原創技能內容、完整combat replica接受與11外部父項仍pending。MVP33/44不變；下一批優先具體channels／zones，不重做已完成目錄、名牌或整組召喚驗收。48h原期限2026-10-25 00:33:38 Taipei不變。沒有新真網路對局、quota reset、commit/push或omfx工作。

證據：[index.json](../acceptance/2026-10-23-cast-transactions/index.json)。OpenSpec tasks 4/4完成。
