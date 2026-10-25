# 敵方魔力削減：最終單機驗收

OpenSpec `complete-enemy-mana-drain` 3/3完成。

共用封閉 DrainManaEnemy 效果支援指定存活敵方。既有光束保留80/125/170/215法術傷害，新增20/30/40/50魔力削減；原消耗／冷卻／射程600與穩定ID維持。內容在複製後調整，學徒光束維持原效果。削減量最多目前魔力；空魔／沒有魔力池扣0，不阻止原有傷害。削減獨立於HP／護盾／治療，不是施法者自行耗魔，不觸發目標SpentMana回呼。效果套用仍重查存活來源與目標；不對已退休或死亡生命期扣魔。

ManaPool::drain使用整數受限扣除並保留自然回魔餘數。Rust authoring與固定Lua生成器驗證逐rank正值[1/1024,1000000]、unit敵方陣營及正射程≤10000；缺值、非法數字、錯誤形狀、敵我衝突拒絕。共用targeted-effect helper讓護盾與敵方扣魔沿用相同來源generation／生命／隊伍／範圍檢查，沒有英雄名稱特判。封閉內部封包沿用既有add_stat_buff ABI，managed serial admission綁定實際caster；傳輸duration必須恰為1，封包永不存入BuffStore。沒有新增ABI slot或wire欄位。

整個generic計畫先preflight，host再於序列提交前檢查封包與後續sibling，非法計畫不留下付款／冷卻／效果。公開基礎提示由封閉effects生成「敵方魔力削減20（最多目前魔力；無魔力池不生效）」與下一級30，與法術HP傷害分開；沒有把資源削減偽裝成legacy HP傷害preview。實際native fixture跑四級metadata與widget，新的Slate截圖包含光束目前→下一級和前批友方護盾回歸。

定向11項Rust與22項固定Lua契約通過，包含任意名稱／宣告順序、封包型別／額度／來源／duration、空魔／無Hero或無池、敵我／死亡／raw1越界／舊generation、後續無效sibling與回呼數量；抽出的護盾守門重跑。Production60Hz正式四級×空魔／少量／充足魔力，驗不足施法成本、護盾全吸收下仍削魔、自然回魔精確餘數、冷卻不能重扣、pause與paired replay。

FreshRust九套 **2121通過／0失敗／9既有ignored**。正式backend／moba-config、UE5.8、DLL/import staging與離線catalog成功；native4／0測試warnings。實際原生Slate截圖已目視檢查並綁定SHA；僅為共用tooltip接受，不宣稱完整遊戲viewport／實體OS DPI。

**3494來源／資產一致**：3472未變／19變更／3新增。集中驗收前後hash、set、size、mtime一致；282份證據獨立重核。原始工作API實際退出、程序退休與健康間隔≤180秒保留；錯誤／根因／預防見本批errors MD。

Catalog data hash `af094e8b04f748b0`，identity `b01c80b5babb0fa3`，class surface `7c09b85b21eea4de`。ABI36／presentation12／selective14維持。

未新增真網路對局／quota reset／commit／push／omfx工作或終止無關程序；MVP33/44與11外部項仍待。

![實際魔力削減提示](D:/omoba/docs/acceptance/2026-10-25-mana-drain/native/capture/rank-effects.png)
