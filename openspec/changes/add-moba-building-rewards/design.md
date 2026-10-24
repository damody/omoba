# 設計

BuildingRewardRules含Tower／Inhibitor／NexusTower／Base四種bounty及local radius，全部有界、setup生成entity前驗證、納入replay digest。Default零獎勵保留舊隔離fixture；正式server plan明確啟用training平衡。數值不宣稱對應任何特定LOL版本。

兩個正傷害入口接共用record_moba_building_damage。追蹤LaneUnit.last_hit_retired作為每生命有界ledger，先退休再驗證來源，避免匿名／友方／Heal／重複致死製造第二次獎勵。純Death不發錢。來源須是權威Faction的對方正式隊伍，可由小兵／召喚物摧毀；無合法來源不發獎勵。

隊伍每人金包含死亡等待重生的正式錢包。local pool只分給致死當刻半徑內存活、非pending lethal的己方英雄，邊界包含，floor等分、餘數丟棄，無recipient不發local pool。既有Gold飽和、owner投影及respawn沿用。重生建築由新LaneUnit開始新ledger；不保留無界歷史集合。暫停、warmup、finished不發。

MinionGrowthRules以秒間隔、每階段HP／damage百分比與兩個獨立cap配置。整數計算floor，負時間視為0，極端時間／數值飽和，invalid policy setup前拒絕。正式training每180模擬秒+10%HP／+5%damage，上限+200%／+100%，不是LOL特定版本數值。只在spawn_wave套用Unit.current/max HP、base_damage與CProperty HP，移速／射程／舊兵不變。規則入digest，pause不推進elapsed或spawn。受壓制兵線的super composition先決定，再共用同一成長。
