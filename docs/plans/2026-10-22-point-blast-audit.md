# 全遊戲盤點：正式圓形爆發技能

接續位移提示後，盤點到兩個手寫技能的實際玩法缺口：flame_blade／flame_assault缺一致rank／JSON／數值／來源／目標／射程守門，錯誤target部分回ROk，且前端真圓形半徑未發布。新PointBlast共用執行器先完整admit再提交，兩wrapper僅供資料鍵／profile／可選控制參數，不在共用機制寫英雄或技能名稱特判。

bounded fixed-point source／center，精確i128 circle；落點中心受range限制，圓內敵人可超出caster射程；alive／HP正值、相切、one-raw外、友軍／死亡排除，canonical排序去重。damage kind／profile明確，optional stun／root／silence為closed canonical集合。保留一次instant damage／cue與既有rank數值；非法RErr交還host退魔力／CD／outcomes。這不是一般outcome整組atomic rollback。

PointCircle typed metadata與真handler共享radius key；兩生成器校驗完整rank及邊界，與executable effects避免雙重幾何。共用movement classification改explicit SweptPath，圓形技能root下仍可施放。前端沿既有rank payload→owner circle／tooltip顯示100／500半徑，不靠特效大小推測。炎刃preview改circle、炎襲補control metadata，描述改與當前instant圓形玩法一致。

新增共用真adapter／handler／metadata、兩技能四rank正式60Hz、boundary／invalid rollback／root非位移案例及native當前半徑／tooltip接受。formal共用既有isolated/no-regen fixture，不複製另一套對局設定。集中結果以final-acceptance與index為準，健康170秒採樣。

整體盤點仍沿用2026-10-21-whole-game-completeness-audit.md：地圖斜向、野區／塔／兵線既有回歸；持續引導、定時衝刺／殘留、更完整特殊幾何、combat replica生命期、召喚整組commit原子回滾、更多英雄道具召喚師、美術音效仍待完成。MVP33/44與11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending。不新增真對局，不維護omfx。