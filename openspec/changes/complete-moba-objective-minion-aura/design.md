# 設計

MobaObjectiveBuffConst新增optional minion_aura（buff_id／radius／attack_bonus／armor_bonus），舊內容缺欄None。僅允許隊伍政策；radius1..3000、bonus0..1000且非全零，光環ID需註冊、不可與任何英雄獎勵ID衝突，同ID政策一致。canonical內容hash與生成自然包含新欄位。

由權威team lease的有效／存活／eligible英雄位置發射，只作用state.units中的同side存活LaneRole::Creep。半徑以固定點平方距離判斷、邊界包含；同ID不因英雄數或史詩層數重複。BuffStore以單instance提供攻擊／護甲；只在membership或payload改變時寫入，離開／死亡／到期／結束清除。沒有光環emitter時不掃兵線。

新生成波次在第一個決策前接光環，英雄退休即撤回其影響；pause凍結。state保存所擁有的(target generation, buff ID)集合並納入digest，避免清除其他buff。北史詩沿原180秒死亡遺失獎勵，增加900半徑、+15攻擊／+10護甲的攻城祝福。

實作盤點發現MOBA兵線攻擊與DamageInstance防禦原本直接讀基礎值，未聚合BuffStore。沿既有UnitStats補上兵線物傷、Unit／Hero護甲與魔抗，保留原有固定點結算、旗標、暴擊與閃避；以正式60Hz實際AI發射與HP減傷驗證，不能只查聚合getter。
