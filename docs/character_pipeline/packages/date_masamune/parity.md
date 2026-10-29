# H02 伊達政宗：原作對照與移植結果

原作：`history_nobu@f966720b0956d002099d72aabec7ddc94b16d4bf`。`S/` = `game/dota_addons/nobu/scripts/`。完整分析見 hero-port-8 對照卡 `date_masamune.md`。

## Q b04_flame_trail_dash（B04W 踏火無痕）

### 原作行為對照卡
- 觸發：`POINT | DIRECTIONAL | IGNORE_BACKSWING`（`S/npc/B_Unified/B04.txt:55`）；OnSpellStart → `S/vscripts/heroes/B_Unified/B04.lua:B04W_Start:3-50`（ProjectileDodge、knockback 衝刺、光環）；光環觸發 → `B04W_OnTrigger:52-70`；命中 → `B04W_OnProjectileHitUnit:72-86`。
- 目標：光環 ENEMY、HERO | BASIC（:129-130），不含建築；預設不選魔免。
- 數值：傷害 100/200/300/400 魔法（:58、:63）；衝刺速度 3000（:74）；光環半徑 300（:79）；火焰速度 speed × 0.5 = 1500（B04.lua:65）、dodgeable（:63）。
- CD 21/19/17/15（:61）；魔耗 80/100/120/140（:62）；距離 900/1000/1100/1200（:65）；前搖 0.01（:66）。
- 特殊：每個進入光環的敵人在當下由施法者位置發射一發追蹤火焰；沒有目標上限、沒有壽命、沒有隨機。

### 移植結果
- 照原作改：火焰速度 1500。
- 引擎上限（記錄於 assumptions）：目標上限 32（host 每來源存活投射物上限）、火焰壽命 10 秒（G-1 投射物必須有壽命，上限 30）。
- 一致：CD、魔耗、距離、前搖、傷害、魔法、衝刺速度、觸發半徑、排除建築。
- needs_framework：衝刺途中逐 tick 觸發並從當下位置發射。現行在施法時依核准路徑決定目標、火焰從起點送出。要做成通用需 forced_motion Dash 每 tick 回報位置並對新進入半徑的敵人回呼腳本，且要以揭露資料選目標維持 replica 一致——不是小幅擴充，本次未做。
- 由 w1 處理（本 worker 未做）：施放時 ProjectileDodge、火焰 dodgeable。
- needs_framework：魔法免疫狀態與選取規則（引擎無魔免）。
- 保留：零位移拒絕施放、撞牆截斷、只鎖定己方可見敵人（E163 replica 一致性）。

## W b04_fire_blast（B04E 火焰強襲）

### 原作行為對照卡
- 觸發：`POINT | AOE`（B04.txt:177）；OnSpellStart → `B04.lua:B04E_OnSpellStart:88-138`。
- 目標：`FindUnitsInRadius` ENEMY、HERO | BASIC（:178-179；B04.lua:96-104），不含建築。
- 數值：傷害 150/250/350/450 魔法（:185）；半徑 500（:199）；暈眩 0.5/0.8/1.1/1.4（:204）；CD 15（:183）；魔耗 140/155/170/185（:184）；距離 700（:187）；前搖 0.3（:188）。
- 特殊：先暈眩再傷害（B04.lua:119-123）；砍樹（:137）。

### 移植結果
- 照原作改：排除建築（`point_blast` 新增 `buildings` 參數，預設 true 不變；本招用 `without_buildings()`）。新增測試 `enemy_buildings_inside_the_blast_are_never_damaged_or_stunned`、`point_blast_buildings_are_hit_by_default_and_skipped_without_buildings`。
- 一致：數值。
- 不適用：砍樹（本遊戲無可破壞樹木）、粒子音效。needs_framework：魔免。

## E b04_flame_swiftness（B04R 炎之迅捷）

### 原作行為對照卡
- 觸發：`PASSIVE`（B04.txt:247），`modifier_B04R_passive`（:260-269）。
- 數值：`add_agi` 10/20/30/40，`MODIFIER_PROPERTY_STATS_AGILITY_BONUS`（:254、:267）；B04 主屬性敏捷（B04_2.txt:47）。
- 升級以新等級數值取代，不疊加；永久。

### 移植結果
- 照原作改：payload 改為 `StatKey::StatsAgilityBonus` 10/20/30/40（content `custom.agility_bonus`），交給 host attributes settle 依 attribute_rules 換算護甲、攻速、主屬性攻擊；技能不再自行換算。同一 buff id，升級取代、死亡保留。
- 換算係數是否對齊原作（0.2 護甲／1.5 攻速）由全域 attribute_rules 決定（另有人處理，本次不改）。

## R b04_tanegashima（B04T 種子島）

### 原作行為對照卡
- 觸發：`NO_TARGET`、ULTIMATE（B04.txt:278-281）；OnSpellStart 掛 `modifier_B04T`（:347-370）；modifier `OnAttack`（:406-420，datadriven OnAttack＝普攻發射時）→ `B04.lua:B04T_OnAttack:157-211`；子彈命中 → `B04T_OnHit:213-221`。
- 數值：射程 +672（:296，128＋672＝800）；持續 28/32/36（:301）；基礎攻擊 +50/70/90（:306）；暈眩 38%（:311，`stun_chance >= RandomInt(1,100)`）0.2 秒（:316）；子彈 35/50/65 魔法（:321）；CD 110（:284）；魔耗 150（:285）；前搖 0.1（:288）；RequiredLevel 未寫。
- 狀態機：發射時擲暈眩，只對 `IsHero() or IsCreep()`，不看命中（B04.lua:171-175）；沿施法者→目標方向射直線穿透子彈（距離 1200、寬 100、速度 2000、ENEMY HERO+BASIC，:177-196）；兩次獨立 `RandomInt(0,100) <= 25` 各在 0.05／0.1 秒後追加一發（:199-210）。

### 移植結果
- 照原作改：
  - 射程加成 672（英雄基礎射程已是原作 128，最終 800）。
  - 暈眩：host 攻擊暈眩新增選項 `attack_stun_on_launch`（發射時直接對目標套用，不看命中）、`attack_stun_skip_buildings`（排除建築），本招 payload 兩者皆開；預設（其他來源）維持舊行為。
  - 直線子彈：host 新增普攻發射 hook（`AbilityScript::on_attack_launch`，英雄每次普攻投射物發射時輪詢已學技能、帶當前 rank 與目標），本招在帶 `b04_tanegashima` buff 時以 G-1 Linear 投射物射出（35/50/65 魔法、1200、半徑 100、速度 2000、穿透 max_hits 64、不打建築）。
  - 追加子彈：G-1 投射物新增 `delay`（0～1 秒），延遲期間停在原發射點與方向、不移動不命中；兩次獨立擲骰，機率以 `rand_unit`（1024 等分）取 264/1024 ≈ 25.8%（原作 26/101 ≈ 25.7%）。
- 解鎖等級：改 6/12/18。KV 沒寫；history_nobu 明確寫出的組合是 RequiredLevel 6／LevelsBetweenUpgrades 6，同 kit 的 A02T、B01T 也是 6/12/18，現代 Dota 預設亦同。
- 一致：持續、CD、魔耗、前搖、基礎攻擊加成；遠程切換以射程加成代替（d3 attack_range_switch）。
- 不移植：`projectile_table` 全域覆寫（B04.lua:177，兩位 B04T 同時攻擊時延遲子彈用錯方向，判定為缺陷）。
- needs_framework：子彈與暈眩的魔免檢查（引擎無魔免）。
- 呈現：延遲子彈在等待的 3～6 tick 內會以靜止投射物出現在發射點（launch fact 在建立時送出）。
