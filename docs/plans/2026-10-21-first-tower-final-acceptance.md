# 第一塔與分層建築獎勵：最終本機驗收

OpenSpec add-moba-first-tower-reward **5/5完成**，先完成兩項功能接線，最後集中驗收。

## 實作

BuildingRewardRules新增first_tower bounty與8個optional tower_layers override，全部有界、setup生成前原子驗證。training第一塔額外local pool150，外塔100 team／250 local、內塔125／300、高地塔150／350；legacy default零獎勵。層級讀取lane_tower_layers持久depth，外塔退休不重編號，無塔名／地圖座標特判。這是本專案平衡，不宣稱LOL特定版本。

整場首塔使用常數空間pending／settled ledger：最早tick優先、同tick依generation-packed entity identity定序；兩隊同tick拆塔不受結果插入順序影響。只計lane Tower，其他建築維持原規則。匿名／非法來源與強制退休consume但不發獎勵，重複致死／Heal不能重領。pause保留候選不提交，後tick的小entity不覆蓋更早tick。

額外金在finish_moba_match_tick既有owner economy發布前一次提交；local資格取致死當刻，當tick死亡者仍從死亡錢包取得已賺到的金，重生保留。原拆塔bounty與首塔bonus獨立、飽和；policy／pending／settled進digest，新world reset。沒有新增首塔公告或塔皮。

## 最終驗收

Rust **1681通過／10既有ignored**：template60、core837、codegen57、base248、backend190/1、bridge127/1、client147/8、ABI15；另generation成功，9工作均exit0／退休。Lua三契約成功。UE5.8 build-v1／backend-v1成功，UBT **3.35秒**。World **33成功**（32 Success＋1 SuccessWithWarnings／4 warnings，0 Failed／NotRun）。

**3345來源／資產**的集合、SHA、size、mtime接受前後一致；DLL與import library依current build／接受前staging另驗。**38份原始證據**逐byte及SHA封存、獨立複核。三分鐘監測5次，UTC最大間隔181秒；7個owned worker與7個watcher均退休，全程health monitor已退出。

## 接受範圍與剩餘缺口

新增ECS測試覆蓋同tick雙方拆塔的兩種結果排列、一次提交、pause／死亡錢包／重生、匿名／強制摧毀、零政策、實際map持久depth與邊界驗證。既有60Hz生成技能拆塔測試擴充：施法者一般金＋首塔金共500，遠處隊友100，且同step OwnerEconomy已帶出500，不增加英雄擊殺／助攻。

本輪World33包含catalog／owned Buff清單／lifecycle及原場景回歸；兩map各74影格／6片段，已目視overview與layered HUD。場景像素為synthetic fixture，首塔效果以正式ECS／60Hz owner economy驗證，不冒稱有新首塔公告或真對局ACK。前輪整套209 native／458 viewport IPC／69 Blueprint／五PCM未重跑，不混為本輪數字。

完整MVP仍33/44，11項外部ACK／LAN／實際OS DPI／固定效能待驗；仍缺通用cast scheduling／interrupt、永久objective stack／傳承／小兵光環、非英雄野怪來源歸因、營地分裂、塔皮／目標公告及特殊技能幾何。未新增連線對局、重設配額、commit／push或維護omfx。OpenSpec CLI不可用，未宣稱CLI validate。

[17系統盤點](2026-10-21-whole-game-completeness-audit.md) · [錯誤T01–T02](2026-10-21-first-tower-errors.md) · [原始封存](../acceptance/2026-10-21-first-tower/index.json)
