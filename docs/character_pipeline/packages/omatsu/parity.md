# A34 阿松（omatsu）原作對照與移植結果

`S` = `game/dota_addons/nobu/scripts`（history_nobu）。腳本 = `S/vscripts/heroes/A_Oda/A34_Matsu.lua`。

## Q a34_frost_bolt（A34W 寒霜彈）

### 原作行為對照卡
- 出處：`S/npc/A_Oda/A34.txt:13-144`；腳本 CD_reduce 52-78、A34T_OnHit 10-40、A34T_debuff_OnDestroy 42-50。
- 觸發：UNIT_TARGET；OnSpellStart：CD_reduce → TrackingProjectile（速度 2000，`Dodgeable 1`，61-64）。
- 目標：敵方 HERO | BASIC | BUILDING，不含魔免旗標。
- 數值：傷害 100/200/300/400（141）；結凍 1/1.5/2/2.5 秒（136、modifier STUNNED 99-110）；`Stun %stun_duration` 未定義＝無效果（83）。
- 冷卻 14、魔耗 100/125/150/175、距離 600、前搖 0.25；後搖未寫。
- 命中順序：DeleteOnHit → 結凍 → Damage（魔法）→ A34T_OnHit。
- 特殊：可閃避；A34T 主動期間命中疊魔抗減免（見 R）。

### 移植結果
- 照原作：追蹤彈加 `dodgeable`（host `ability_projectile::with_dodgeable`），目標呼叫躲避時失效不結算；加 `hit_stack`（急速冷卻主動 buff 存在時，命中當下疊魔抗減免）。
- 保留：投射物 radius 16、lifetime 3（引擎 envelope 必填；6000 距離遠大於 600 射程）；先扣血後結凍的順序差異不影響結果。
- 被動 −2 秒冷卻：host `cast_cooldown_refund`，已一致。
- needs_framework：無。

## W a34_iceberg_strike（A34E 冰山之襲）

### 原作行為對照卡
- 出處：`S/npc/A_Oda/A34.txt:147-330`；腳本 CD_reduce、A34T_OnHit；緩速 `dummy.lua` Slow 816-858、slow_self_passive 866-902、`npc_abilities_custom_load.txt` slow_self 5987-6026。
- 觸發：UNIT_TARGET | AOE；OnSpellStart（178-237）：CD_reduce → 對指定目標 Damage `aoe_damage` 100 → ActOnTargets（半徑 335，含目標）：緩速 → Damage `AbilityDamage` 100/200/300/400 → A34T_OnHit。
- 目標：敵方 HERO | BASIC（不含建築）。
- 數值：移速 −30%、攻速常數 −20 點（ATTACKSPEED_BONUS_CONSTANT）、5 秒；半徑 335。
- 冷卻 7、魔耗 125/145/165/185、距離 700、前搖 0.1（160）。

### 移植結果
- 照原作改：前搖 0.25 → 0.1；指定目標先受 100、範圍內每人（含指定目標）受 100/200/300/400（指定目標合計 200/300/400/500）；目標與範圍排除建築；攻速減速改為 `attack_speed_bonus_constant` −20 點（buff id `a34_iceberg_attack_slow`，同原作 `as_slow["A34E"]` 單一 key）；主動期間每個被擊中者送一次 hit_stack 封包。
- 保留：移速緩速用現有 generic_ability_slow 介面（多來源乘法合成由 w4 處理）；攻速多來源「只取最強」也屬 w4 的緩速聚合。
- needs_framework：無。

## E a34_frost_breath（A34R 冰霜之息）

### 原作行為對照卡
- 出處：`S/npc/A_Oda/A34.txt:333-506`。
- 觸發：DIRECTIONAL | POINT；OnSpellStart：CD_reduce → LinearProjectile（381-397）：start_radius 150、end_radius 500（470-475，HasFrontalCone）、FixedDistance 1200、速度 1250。
- 命中（404-424）：穿透；modifier_A34R_datadriven（Duration 5）→ Damage AbilityDamage 100/175/250/325 → A34T_OnHit；modifier ThinkInterval 1 每次 Damage 50/60/70/80（427-445）。
- 目標：敵方 BASIC | HERO（投射物 TargetFlags NONE）。
- 冷卻 10/9/8/7、魔耗 100/120/140/160、施法距離 800、前搖 0.1。

### 移植結果
- 照原作改：前搖 0.1；levels.range 800（施法距離），飛行距離另由 `projectile_distance` 1200；命中半徑由 150 隨已飛距離線性增到 500（host `with_end_radius`）；DoT 改離散 1 秒一跳（host `with_dot_interval`），5 跳（第 1～5 秒，含到期那一刻；理由見 source.lua `frost_breath_dot_ticks`）；主動期間命中疊魔抗減免。
- 保留：表現層投射物寬度維持起始半徑（揭露 flight schema 不變）；host 會拒絕 800 外的方向點（Dota 是否走近不確定）。
- needs_framework：無。

## R a34_rapid_cooling（A34T 急速冷卻）

### 原作行為對照卡
- 出處：`S/npc/A_Oda/A34.txt:509-664`；腳本 A34T_OnCreated 80-92、OnDestroy 94-98、OnIntervalThink 100-113、OnUpgrade 115-118、Mana_regen 120-123、A34T_OnHit 10-40、debuff OnDestroy 42-50。
- 被動：最大法力 +300/400/500（653）；W/E/R 互減 2 秒冷卻（CD_reduce）。
- 主動 22 秒：每 0.1 秒把 A34W/E/R 剩餘冷卻壓到 ≤ GetCooldown(-1)×(1−40/50/60%)，含已在跑的；OnAbilityExecuted 回 100/150/200 法力；W/E/R 命中者疊 modifier_A34T_debuff（3 秒、上限 3 層、每層 −5/10/15 魔抗點；未滿層時刷新，滿層不刷新；到期一次還原）。
- 冷卻 100/90/80、魔耗 100/200/300、前搖 0、6/12/18 級。

### 移植結果
- 照原作改：移除 `cooldown_percentage`（只影響新冷卻）；主動 buff 改帶 host `cooldown_cap`（每 tick 壓剩餘冷卻，含既有冷卻，只限非大招技能）與 `hit_stack` 規則（MagicalResistanceBonus −5/10/15 每層、3 層、3 秒、滿層不刷新、只對英雄）；新增 hero UnitScript `on_skill_cast`，主動期間每施放技能回魔（rank 由同時掛上的標記 buff 判斷）。
- 保留：魔抗以點數加總表達，不實作 70 上限；A34T 自己的施放不回魔；物品使用是否觸發 on_skill_cast 由 w2 處理；think 0.1 秒改為每 tick（更頻繁，結果相同或更精確）。
- needs_framework：無。
