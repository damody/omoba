# 設計

BuildingRewardRules新增first_tower bounty及8個optional tower_layers override。每個bounty的team／local pool皆0–1,000,000，setup原子驗證；缺layer override回落tower。來源取lane_tower_layers中的持久depth，不因外塔退休後重新編號，也不依名稱／座標。

首塔ledger為常數空間pending／settled兩個claim，記tick、generation-packed entity、credited side及致死當刻附近player set。Playing且非pause的Tower第一次正傷害致死觀測候選；無合法來源也consume，純強制退休記無credit。其他建築不參與。tick結算前依(tick,entity)取最小，避免佇列順序改變同tick勝者；pause保留原候選，後tick不能覆蓋更早候選。

結算在finish_moba_match_tick owner economy發布之前，額外bounty只發一次；local資格取致死當刻，當tick死亡者仍透過正式死亡錢包收款，重生保留。base bounty沿原首次致死規則即時提交；兩種獎勵獨立且飽和。policy／pending／settled全進digest，換world reset。沒有首塔UI公告或塔皮機制。

training第一塔額外local pool150；外塔沿100 team／250 local，內塔125／300，高地塔150／350。legacy default皆零／無override。數值為專案可配置平衡，不宣稱LOL特定版本。
