# 2026-10-22 通用召喚生成最終驗收

OpenSpec `complete-moba-summon-spawning` 5/5 完成。本輪接續整體17範圍盤點，實際修復原先無效的召喚生成入口。

## 本輪實作

- Generated／runtime Lua summon catalog 決定 HP、傷害、移速、射程、攻擊週期、防禦與碰撞半徑；不依英雄名稱選擇原生生成分支。
- 返回真實 generation-aware handle，deferred outcome 提交完整元件、ScriptUnitTag、Spawn hook、隊伍與 PlayerOwner。平行 tick 有生成需求時按固定排序重跑分配；失敗施法、panic 或未提交 adapter drop 回收預留。
- 固定點生命期、零秒永久、pause凍結；到期走 EntityRemoved、移除 BuffStore 並退出實體；到期不算擊殺。MoveTarget 使用內容移速、rooted／碰撞規則，命令進行中不被自主 chase 覆寫。
- 召喚索敵可查活著的敵對英雄／野怪／單位，牆體LOS過濾、距離及完整entity定序；Saika實際腳本不再使用舊硬編碼移速。召喚生命期／主人／屬性／位置／朝向／模板與命令進入 replay digest。

## 集中驗收

RustV3 **1692通過／10既有ignored**：template60、core846、codegen57、base250、backend190/1、bridge127/1、client147/8、ABI15；另內容生成成功。9個工作均exit0並退休。新增4項core測試；正式60Hz技能輸入實際生成兩名援軍、繼承主人／隊伍、發射真投射物並造成傷害、pause凍結與到期退休，包含在base250內，不重複加總。

Lua三契約成功。最終UE5.8 build-v2、backend-v2成功；UBT **3.65秒**。WorldV2 **33成功**（32 Success＋1 SuccessWithWarnings／4 warnings，0 Failed／NotRun）；4筆preview-world清理警告保留。兩map各74影格／6片段；已目視兩張輸出，為owned公開場景fixture，並非新真對局中的召喚畫面／ACK。

**3346來源／資產**在最終接受前後的集合、SHA、size、mtime一致。Current build與接受前staging／DLL／import library另驗。**77份原始證據**逐byte／SHA封存，獨立複核成功；含失敗與superseded輸出。三分鐘監測11次，UTC最大間隔181秒；另持續讀取最新log／CPU。**21個owned workflow worker與21個watcher**均退休，全程health monitor也已退出。

## 保留範圍

全遊戲尚未等同成熟競技MOBA。優先補通用施法排程／中斷、已退休投射物來源快照、召喚落點避障／跟隨／更多控制策略及可見性複本完整生命期接受；其後為永久目標stack／傳承／小兵光環、營地分裂、特殊技能幾何、塔皮／公告、內容與專用美術。

原MVP維持 **33/44／11外部待驗**。未新增連線對局、未重設配額、未commit／push，omfx不維護。本輪未重跑完整209 native／458 IPC／69 Blueprint／五PCM，不混入新數字。錯誤S01–S12與失敗CheckV1／CoreCheckV2、V3／RustV1、V2保留。

證據：`docs/acceptance/2026-10-22-summon-spawn/index.json`。盤點：`2026-10-21-whole-game-completeness-audit.md`及`2026-10-22-summon-spawn-audit.md`。
