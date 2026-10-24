# 設計

營地additional_objective_buffs缺欄空、最多4項；連同原objective_buff都是隊伍獎勵，同map同ID政策／scope一致、隊伍與個人各最多32種以符合既有lease cap。每營地不得重複同ID，避免單次死亡誤疊兩層。

隊伍duration_seconds=0表示永久，必須死亡保留且不可個人傳承；個人Buff仍限1..3600秒。TeamObjectiveLease expires改為Option，None是真正永久，有限期限仍Some。BuffStore沿既有raw i64::MAX永久標記，tick不扣此值；snapshot既有-1／ABI永久flag／Unreal持續生效顯示沿用，不用巨大有限秒數模擬。

南史詩保留原120秒護甲Buff，另授予原創永久傳承，每層+5攻擊、最多4層，死亡／重生保留，對局結束／新局清理。所有效果仍為單一BuffStore entry乘權威層數，規則與Option期限納入digest。
