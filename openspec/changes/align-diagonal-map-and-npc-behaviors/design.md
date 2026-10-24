# 設計

所有路線共用起終點；基地與出生沿 mid 路線內向取樣，不再假設水平軸。三路每方三層塔；16 個中立營地包含雙方鏡射野區、河道與大型中立目標。現有塔解鎖鏈不變。

護主狀態只存在 authority，依攻擊者 entity generation＋受害者 player 保存（最多 roster²），含受害者生命、事件位置與2.5秒期限，納入 deterministic hash。普攻 windup／成功的指定單位技能對小兵及塔觸發；其他實際英雄傷害只觸發塔。小兵與塔有獨立期限，AOE不延長小兵護主；同一攻擊者傷害不同英雄不互相抹除。受害事件須在防守單位500範圍，攻擊者亦須在索敵／可見範圍；攻擊者死亡、過期或離開範圍解除。已在攻擊塔的小兵繼續拆塔。一般目標保留到無效或護主插隊；沒有目前有效目標時優先小兵、英雄、塔、基地，再以距離和entity key排序。塔只攻擊英雄／小兵。

野怪使用既有硬 leash 狀態機：受擊追擊、失去有效目標或越界後返回，返回期間不可受擊／重新索敵，抵達營地經共用 Heal 路徑恢復，擊殺後依 catalog 重生。不是 patience 軟重置機制。

Idle／AttackMove不主動取得HostileNeutral野怪，明確AttackTarget仍可攻擊。常規／河道營地leash700，大型1000；公開地形允許位於leash內，由共用NPC導航繞行，營地本體20半徑禁止被牆占用。單一營地目前仍為單一中立單位；大型目標沒有宣稱包含LOL完整增益、巢穴或分裂機制。

參考：Riot 13.10 minion Tunnel Vision：https://www.leagueoflegends.com/en-gb/news/game-updates/patch-13-10-notes/；Riot 26.10移除因敵方攻擊友方小兵而觸發的護主訊號：https://www.leagueoflegends.com/en-us/news/game-updates/league-of-legends-patch-26-10-notes/

正式投影的2次冪padding buckets擴充至64KiB且維持有限上限，涵蓋完整5v5／三層塔／兵線出生burst。KCP使用既有stream framing，沒有放寬通訊tag的個別payload界線；需驗32／64KiB真framing roundtrip、超限拒絕及正式State::tick。

參考：Riot 12.22 patch notes 說明 leash／reset 的概念（歷史版本，不用作現版數值依據）：https://www.leagueoflegends.com/en-us/news/game-updates/patch-12-22-notes/

## 兵線編成

共享地圖需宣告 minion_wave（身分、每波數量、出現週期、HP、傷害、射程、移速）及波間隔。正式 RoleBotMatchPlan 採用這份編成；直接建立 SingleLaneConfig 的隔離 fixture 保留可選的簡化兵線。三路每方常規 3 近戰＋3 遠程，每第三波增加 1 砲車。數值是本專案可調整的訓練設定，不宣稱等於現版 LOL。Lane AI 使用 Unit 的實際射程與傷害，而非角色類型中的固定常數；各種兵有可解析的共享 world_units 呈現身分。
