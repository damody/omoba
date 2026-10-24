# 設計

營地新增optional personal_buff，沿現有Buff規則型別，max_stacks必須1，transfer_on_kill缺欄false。傳承必須lost_on_death=true；team objective_buff不可開啟傳承。每張map同Buff ID的政策與個人／隊伍scope必須一致，避免兩條同步路徑互相覆蓋。

個人lease以(player,buff ID)索引，固定上限320，authority expires與rule進入digest。真實營地主體死亡且有player歸因才授予，原創力量／守衛Buff各120秒、+10攻擊／護甲，四個一般營地配置。未啟用營地保持原樣。

有效敵方英雄擊殺且普通金成功結算後，從受害者移除可傳承lease，僅交給仍活著的擊殺者，保持原expires；已有同政策Buff取較晚expires，不重設完整duration。死亡來源匿名／強制則只清理，不傳承；同生命lethal_pending去重。歸因沿既有英雄／召喚／投射物鏈。正常死亡／重生、pause、到期／結束與新對局清理完整。既有Buff／屬性管線到Unreal，ABI33維持。
