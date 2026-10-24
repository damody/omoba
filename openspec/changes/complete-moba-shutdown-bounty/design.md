# 設計

Lua economy中的可選kill_streak_bounty政策缺省關閉，正式server選用generated政策；門檻、首額、每殺增額與cap均有界驗證及重建保護。狀態屬玩家roster而非entity ID；只在有效敵對英雄致死且獎勵入帳後增加連殺，任何英雄死亡清除受害者連殺，重複致死不重付。既有召喚／投射物歸因與死亡錢包共用，助攻獎勵不變。

新增公開score metric及protobuf欄位，缺欄舊snapshot為零，typed bridge與C ABI同步，Unreal計分板顯示額外賞金。狀態與政策納入digest。不能由renderer冷卻／總擊殺數推斷連殺。
