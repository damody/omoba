# 建築獎勵契約

- 正式MOBA plan啟用角色化且有界的獎勵；legacy零設定保持不發錢。
- 原生與script正傷害共用第一次致死ledger，重複／Heal／匿名後補來源不能重複領。
- 敵方英雄、小兵與權威召喚物可觸發隊伍獎勵；匿名、友方、中立不發。
- team gold每位正式隊員獨立領取，包含死亡錢包；local pool依當刻存活距離資格等分，不因roster順序給餘數。
- 非Playing、pause、非建築、純退休不發。建築respawn新生命可以再次領。
- 規則與生命ledger進入deterministic digest；不新增公開敏感狀態或假ACK。
- 新兵按暫停可凍結的match elapsed成長，精確階段邊界包含；HP／damage各自有cap、floor及溢位保護，舊兵不重算，所有composition共用。
