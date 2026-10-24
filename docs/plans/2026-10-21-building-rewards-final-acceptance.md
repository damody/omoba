# 建築獎勵與兵線成長：最終本機驗收

OpenSpec add-moba-building-rewards **6/6完成**。先完成兩組功能，再集中驗收。

## 實作

新增BuildingRewardRules：一般塔、兵營、主堡塔與基地可各自設定隊伍每人金與附近英雄獎池。正式人類／role bot plan啟用共用training平衡，legacy default零獎勵。原生Damage與checked direct damage接相同首次致死ledger；匿名／友方／中立先退休不發獎勵，Heal／重複致死不重領，純Death不發。敵方隊伍英雄、小兵及合法Faction召喚物可觸發；附近存活非pending lethal英雄依包含邊界的半徑floor等分，餘數丟棄。全隊獎勵包括死亡等待重生錢包，Gold飽和，重生建築新生命可再結算；政策與生命旗標入digest。

新增MinionGrowthRules：正式plan每180模擬秒，新兵HP +10%、攻擊 +5%，各自cap +200%／+100%；只在spawn套用，舊兵不變，近戰／遠程／砲車與super composition共用。pause凍結時間與spawn；整數floor、極端值飽和、setup生成前驗證、政策入digest。這些是本專案可配置平衡，不宣稱特定LOL版本數值。

## 最終驗收

Rust **1670通過／10既有ignored**：template59、core828、codegen57、base247、backend190/1、bridge127/1、client147/8、ABI15；另codegen generation成功，9工作均exit0／退休。Lua三契約成功。UE5.8正式build-v1與backend-v1成功，UBT **3.39秒**。World **30項成功**（29 Success＋1 SuccessWithWarnings／4既有preview-world警告，0 Failed／NotRun），兩map各74影格／6片段。

接受前後 **3343來源／資產**的集合、SHA、size與mtime一致；DLL／import library依current build與接受前staging另驗。**41份原始證據檔**逐byte與SHA封存後獨立複核，含全部Rust log、建置、receipt、盤點與錯誤MD。三分鐘監測8次，UTC最大間隔181秒；10個owned worker與10個watcher均退休，另本輪全程health monitor已退出。

## 接受範圍與剩餘缺口

原生／script正式傷害、角色配置、匿名／重複、局部距離、死亡錢包、建築重生與暫停以ECS驗證；新增60Hz生成技能摧毀塔測試，施法者350金、遠方隊友100金且不增加英雄擊殺／助攻。兵線成長驗證精確時間邊界、cap、overflow、新舊兵與pause。

本輪沒有修改C++／公開schema；新World實際視窗圖已目視，圖片為synthetic compiled-map fixture，不是新獎勵HUD的真對局截圖。本輪未重跑整套209 native／458 viewport IPC／69 Blueprint／五PCM，這些仍引用前輪獨立封存，不混為本輪結果。未新增連線對局、重設配額、commit／push或維護omfx。

全遊戲仍有通用施法排程／中斷、史詩野怪效果／分裂、塔皮／第一塔獎勵、特殊技能幾何與專用美術等缺口；原MVP仍33/44、11項外部ACK／LAN／OS DPI／固定效能待驗。OpenSpec CLI不可用，未宣稱CLI validate。

[全遊戲17系統盤點](2026-10-21-whole-game-completeness-audit.md) · [錯誤B01–B06](2026-10-21-building-rewards-errors.md) · [原始證據](../acceptance/2026-10-21-building-rewards/index.json)
