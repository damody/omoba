# 全遊戲盤點：個人野區Buff與擊殺傳承

接續17系統盤點；本輪補野區獎勵欠缺的個人持有與傳承，而非改動既有史詩全隊Buff。

- Lua新增personal_buff及transfer_on_kill政策，個人上限一層、傳承必須死亡丟失；同map同ID政策與scope一致。缺欄兼容、生成、catalog、hash與重建守門完整。
- 四個一般營地指定原創120秒力量／守衛Buff，分別+10攻擊／護甲，只有有玩家歸因的主體致死會授予。沒有改寫英雄／營地名稱判斷到runtime。
- 權威玩家lease沿英雄／召喚／投射物來源歸因，有效敵對擊殺成功結算後傳承剩餘期限，既有同類Buff取較晚期限，不重設完整時間、不分給隊友。
- 匿名／強制死亡只清理、同生命去重、死去擊殺者不收Buff、pause／非Playing不傳承；到期、重生資格、結束清理及digest沿權威。
- 既有Buff snapshot／戰鬥屬性與Unreal消費管線沿用ABI33，新Buff使用native-only內容；沒有新增傳承公告或專用UI。

集中驗收包含真實營地、英雄與召喚歸因、剩餘期限合併／到期、匿名死亡、正式60Hz generated技能取得與native DeathTick清理。最後數字詳同日期buff-transfer-final-acceptance.md，錯誤T01另記。

仍缺永久史詩stack、小兵光環、野怪分裂、經濟差補償、更多公告、持續引導／特殊幾何／動畫、完整召喚複本／回滾、塔皮與美術。原MVP33/44／11外部ACK／LAN／DPI／效能待驗維持。
