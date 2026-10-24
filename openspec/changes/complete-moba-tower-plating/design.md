# 設計

SingleLaneConfig optional TowerPlatingRules，預設None保留既有fixture；正式server與role plan啟用原創training數值：5層、840秒、每層125本地金、半徑1200、每破層+20護甲。驗證plate1..16、expiry1..3600、gold<=1m、radius1..10000、armor<=1000。

只接受tracked lane outer-depth0塔，綁定entity generation生命。首次正傷記錄max HP raw，lost=floor((maxHP-currentHP)*plates/maxHP)，i128中間值避免溢位，最高lost永不減少。匿名／同隊／無附近英雄也消耗門檻；治療、重複Outcome不能重領。跨多層依每一層先均分捨餘再乘層數，不能因合併傷害而多領捨餘。

沿既有jungle_credit英雄／召喚／NPC隊伍歸因與building_rewards::credit；僅附近存活同side英雄分金，不加擊殺／助攻／XP。正傷commit hook在原lethal拆塔獎勵前處理，與拆塔／首塔金獨立。BuffStore tower_plating提供lost*armor固定點加固，HP0／退役／到期／結束清除；不改基礎護甲。ledger與規則加入digest。公開Buff snapshot／catalog與既有owner economy HUD延用；專用塔血條分段與美術仍留盤點。
