# 史詩目標隊伍增益：最終本機驗收

OpenSpec add-moba-objective-team-buffs **6/6完成**。先完成功能接線，再集中驗收。

## 實作

營地主體資料新增optional objective_buff：Buff ID、期限、攻擊／護甲加成及lost_on_death；模板驗證範圍與非tombstone catalog引用，編譯至共用常數，兩張map共用同一資料轉換。附屬怪不繼承主體政策，沒有營地名稱特判。

正式主體正傷害的首次致死歸因，在Death結果提交後授予擊殺英雄的隊伍。匿名／強制Death、重複死亡及附屬怪不授予。同隊同Buff刷新期限、不疊加，兩隊獨立。死亡失去政策僅授予當下存活英雄、死亡移除資格；死後保留政策包含死亡隊員，重生依原期限剩餘時間恢復。pause凍結、精確deadline過期、結束清除、新world無殘留；有界lease／期限／資格入digest。

實際加成進入正常BuffStore／UnitStats／snapshot與原生Buff catalog：史詩之力180秒+20攻擊／+10護甲，死亡失去；守護祝福120秒+15護甲，死後保留原期限。這是本專案資料平衡，不宣稱LOL特定版本。此類team lease效果不做獨立dispel；未包含永久stack、傳承、小兵光環、非英雄來源歸因與營地分裂。

## 最終驗收

Rust **1676通過／10既有ignored**：template60、core832、codegen57、base248、backend190/1、bridge127/1、client147/8、ABI15，另generation成功；9工作均exit0／退休。Lua三契約成功。UE5.8 build-v1、backend-v1成功，UBT **51.42秒**；World **33成功**（32 Success＋1 SuccessWithWarnings／4既有preview-world清理警告，0 Failed／NotRun）。

**3344來源／資產**的集合、SHA、size、mtime接受前後一致；DLL與import library另依接受前staging／current build核對。**39份原始證據**逐byte及SHA封存並獨立複核；三分鐘監測8次，UTC最大間隔181秒；8個owned worker及8個watcher均退休，全程health monitor也已退出。

## 接受界線與剩餘缺口

新增資料測試驗證兩map各兩個objective政策、數值邊界與未知catalog；ECS驗證隊伍授予、實際攻擊／護甲、refresh／不疊加、兩隊獨立、死亡政策、重生剩餘時間、匿名／強制／附屬死亡、pause／deadline／terminal／新world。新增60Hz生成技能擊殺測試，確認己方Buff／敵方無Buff、實際+20攻擊與snapshot payload／剩餘時間。

World新增CompiledCatalogContract、OwnedBuffListSnapshot、BuffEffectLifecycle三項；兩map各74影格／6片段，已目視本輪overview與layered HUD。場景像素是synthetic compiled-map fixture；新增Buff以正式snapshot與原生清單契約驗證，不冒稱已取得真對局增益hover截圖／ACK。本輪未重跑前輪整套209 native／458 viewport IPC／69 Blueprint／五PCM，不混為本輪結果。

未新增連線對局、重設配額、commit／push或維護omfx。完整MVP仍33/44、11外部ACK／LAN／OS DPI／固定效能待驗。仍缺通用cast scheduling／interrupt、永久objective stack／傳承／小兵光環、非英雄來源歸因、營地分裂、塔皮／第一塔與特殊技能幾何等。OpenSpec CLI不可用，未宣稱CLI validate。

[17系統盤點](2026-10-21-whole-game-completeness-audit.md) · [錯誤O01–O04](2026-10-21-objective-buffs-errors.md) · [原始封存](../acceptance/2026-10-21-objective-buffs/index.json)
