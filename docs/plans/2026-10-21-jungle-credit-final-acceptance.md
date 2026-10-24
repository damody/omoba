# 野怪非英雄來源歸因與仇恨：最終本機驗收

OpenSpec complete-moba-jungle-source-credit **5/5完成**。工作跨2026-10-21／22，檔名沿用起始日；先完成接線，再集中驗收。

## 實作

新增權威JungleCredit resolver：來源完整entity須仍有效，Faction須對應正式隊伍；PlayerOwner須同側roster，主人死亡不改變玩家身份。無PlayerOwner時最多追8個SummonedUnit generation-aware連結，各節點須有效且同隊，cycle／超限／stale／未知或跨隊owner拒絕；無owner的合法隊伍單位只給team side，不猜個人領錢者。

第一個正傷害致死把player／side保存在camp body，正式Death一次消費。受控來源個人Gold／XP進live或dead wallet，無主來源不給個人金／XP；史詩主體按side授予正常objective team lease。來源在致死後退休不丟credit，匿名／中立／非法來源先退休本生命ledger，Heal／後補來源／重複Death不能重領。新side隨既有jungle replay material納入digest。

非英雄合法隊伍單位可吸引camp仇恨、被實際攻擊、追擊與回頭；目標退休後沿原patience／soft-hard return，原LOS與攻擊週期不改。ownership追溯只在致死時執行，有界且不配置記憶體，不新增per-entity每frame掃描。

## 最終驗收

Rust **1687通過／10既有ignored**：template60、core842、codegen57、base249、backend190/1、bridge127/1、client147/8、ABI15，另generation成功；9工作均exit0／退休。Lua三契約成功。UE5.8 build-v1與backend-v1成功，UBT **3.43秒**；World **33成功**（31 Success＋2 SuccessWithWarnings／5 warnings，0 Failed／NotRun），4筆preview-world清理與1筆背景HTTP timeout保留。

**3346來源／資產**的集合、SHA、size、mtime接受前後一致；DLL與import library依current build／接受前staging另驗。**39份原始證據**逐byte與SHA封存、獨立複核。三分鐘監測7次，UTC最大間隔181秒；8個owned worker及8個watcher均退休，全程health monitor已退出。

## 接受範圍與新查明缺口

ECS測試涵蓋8-link／cycle／stale generation／未知和跨隊owner、死亡主人收150金／300 XP（升級後level3／80）、persistent增益重生恢復、來源致死後退休、重複死亡、無主team-only效果、匿名／非法來源不能後補重領，以及實際camp對非英雄攻擊／退休回頭。新增60Hz正式結果佇列測試同時驗owned與unowned來源的Gold／XP與Buff隔離。

重要新缺口：ParallelWorldAdapter::spawn_summoned_unit仍回EntityHandle::INVALID。本輪不實作召喚生成，ECS測試建立ownership fixture不等於召喚技能可用。該stub的生成／ownership／duration／退休接線列下一個優先實作。已退休來源且缺少source snapshot的投射物、無持久PlayerOwner又指向已退休主人的鏈仍拒絕，後續需生成時歸屬快照。

World33與兩map各74影格／6片段成功，已目視本輪overview與layered HUD；像素是synthetic map fixture，新歸因以正式ECS／60Hz結果驗證。前輪209 native／458 viewport IPC／69 Blueprint／五PCM未重跑不混算。原MVP文件仍33/44、11外部ACK／LAN／OS DPI／效能待驗；新增召喚stub另列實作缺口，不把勾選數當全部英雄技能成功。未新增連線對局、重設配額、commit／push或維護omfx。OpenSpec CLI不可用，未宣稱CLI validate。

[17系統盤點](2026-10-21-whole-game-completeness-audit.md) · [錯誤J01–J07](2026-10-21-jungle-credit-errors.md) · [原始封存](../acceptance/2026-10-21-jungle-credit/index.json)
