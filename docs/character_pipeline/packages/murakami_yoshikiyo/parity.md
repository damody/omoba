# 村上義清（B29，current）原作對照與移植結果

原作：`D:\omoba\target\source\history_nobu`（`damody/history_nobu@f966720`）。以下 `S/` 代表 `game/dota_addons/nobu/scripts/`。

| 類別 | 檔案:行 | 內容 |
|---|---|---|
| 技能 KV | `S/npc/B_Unified/B29.txt:27-83` | B29W 動地斧 |
| 技能 KV | `S/npc/B_Unified/B29.txt:87-309` | B29E 震地斧 |
| 技能 KV | `S/npc/B_Unified/B29.txt:311-391` | B29R 士氣提升 |
| 技能 KV | `S/npc/B_Unified/B29.txt:394-483` | B29T 大地震撼 |
| 單位 KV | `S/npc/B_Unified/B29_2.txt:4-87` | Ability1-6＝B29W／B29E／B29R／magical_resistance／B29_precache／B29T；只掛 current 版 |
| 腳本 | `S/vscripts/heroes/B_Unified/B29.lua:6-31`、`32-82`、`85-120`、`122-177`、`241-280` | B29W、B29E、B29R（死碼）、B29T、B29T_Effect |
| 共用腳本 | `S/vscripts/library/common/dummy.lua:816-902`、`S/npc/npc_abilities_custom_load.txt:5987-6026` | 緩速 `Slow`／`ReturnSpeed`／`slow_self_passive` |

`_old` 版（`B29.txt:487-873`、`B29.lua:179-378`）不採用。

---

## Q b29_ground_slam（B29W 動地斧）

### 原作行為對照卡

- 觸發：`NO_TARGET`（`B29.txt:32`），`OnSpellStart` → `B29W_OnSpellStart`（57-64）。
- 目標：施法者位置半徑 **550**（`B29.lua:15` 寫死；`AOERadius 500` 只是指示圈，41）；敵方 `HERO | BASIC`、旗標 0（不打魔免，33-36）；`not IsBuilding()`（25）。
- 數值：傷害 150/250/350/450 魔法（47）；暈眩 1/1.3/1.6/1.9（52）。
- 冷卻 14（38）；魔耗 100/120/140/160（39）；無施法距離。
- 前搖 0.3（37）；後搖 `ACT_DOTA_CAST_ABILITY_2`（40）。
- 狀態機：先 `modifier_B29W`（暈眩，68-81）再 `ApplyDamage`（26-27）。

### 移植結果

- 照原作改：作用半徑 500 → **550**（指示圈也用 550）；測試改為 550 在內、551 在外（`every_rank_damages_and_stuns_each_enemy_within_550_once`）。
- 保留：先傷害後暈眩（同一結算，結果相同）。
- needs_framework：魔法免疫狀態與目標旗標（原作不打魔免敵人；引擎沒有魔免）。

---

## W b29_quake_axe（B29E 震地斧）

### 原作行為對照卡

- 觸發：`POINT | DIRECTIONAL | AOE`（`B29.txt:95`），無施法距離；`OnSpellStart`（141-178）：`B29E_OnSpellStart`、`DelayedAction 0.25` 粒子、`ApplyModifier modifier_B29E_aura`（169-177）。
- `B29E_OnSpellStart`（`B29.lua:32-82`）：目標點超過 300 截到 300（38-42）；`modifier_knockback` 固定 `knockback_duration` 0.15 秒、高度 200（45-57）；`modifier_phased` 0.25 秒（62）；`Timers` 0.2 秒後（63）以施法者**當下**位置半徑 500（69）對敵方 `HERO | BASIC` 造成 200 魔法（78）並套 `modifier_B29E_debuff`（79）。
- `modifier_B29E_debuff`（182-211）：8 秒，`Slow` name `B29E`、`ms_slow` −18/−22/−26/−30（118）。
- 緩速規則（`dummy.lua:816-902`）：`ms_slow[name]` 同名覆蓋，`ms_unslow` 取最大值削弱；`slow_self_passive` 每 0.1 秒把不同名緩速乘法疊加（875-882）；攻速減速取最強（887-901）。
- `modifier_B29E_aura`（212-287）：8 秒、每 0.5 秒；`OnCreated` 在施放瞬間（施法者尚未位移，起跳點）對 800 內敵人套 `modifier_B29E` 1 秒，之後每次 think 以施法者位置刷新；`modifier_B29E` 護甲 −4/−7/−10/−13（113、295）。
- 冷卻 20/18/16/14（101）；魔耗 105/125/145/165（102）；前搖 0.15（106）。

### 移植結果

- 照原作改：
  - 跳躍固定 0.15 秒落地：`swept_dash::motion_packet_for(…, dash_duration=0.15)`，速度＝距離／0.15（`every_leap_lands_after_the_fixed_0_15_seconds`）。
  - 猛擊改為施放後 0.2 秒、以施法者當下位置為中心：通用能力 `delayed_effect`（caster buff 到期 → `QuakeAxeHandler::on_delayed_effect` → `slam`）（`the_slam_resolves_around_the_casters_current_position_with_the_rank_slow`）。
  - 靈光首次套用改以起跳點為中心（`every_rank_leaps_at_most_300_and_the_aura_starts_at_the_take_off_point`）。
  - 緩速疊加：通用規則（omoba-core `BuffStore` `generic_ability_slow`：不同來源 1−Π(1−sᵢ)、同名只算一筆、`slow_resistance` 削弱），本招 buff id `generic_slow:b29_quake_axe:<caster>`。
- 保留：
  - 跳躍受地形碰撞截斷（`modifier_knockback` 拋物線是否越過地形，腳本無法判定）。
  - `modifier_phased` 未另外實作（forced_motion 位移不受單位碰撞阻擋）。
  - 小兵也會被減速（原作只對掛 `slow_self` 的單位生效，兵線模板未逐一確認）。
  - 施法者在 0.2 秒內死亡時猛擊取消（`delayed_effect` 排程放在施法者 buff，死亡清除；原作 Timer 照常結算）。
- needs_framework：魔法免疫狀態與目標旗標。

---

## E b29_morale_boost（B29R 士氣提升）

### 原作行為對照卡

- 觸發：`PASSIVE`（`B29.txt:315`）；`OnSpellStart` → `B29R_OnSpellStart`（`B29.lua:85-120`）永遠不觸發，死碼。
- `modifier_B29R_aura`（357-368）：半徑 600、友方 `HERO | BASIC`、含自身；`modifier_B29R_passive` `DAMAGEOUTGOING_PERCENTAGE` 7/14/21/28（354、383）。
- `modifier_B29R_str`（369-376）：`STATS_STRENGTH_BONUS` 4/8/12/16（388）。
- 光環離開後殘留：Dota 光環預設 0.5 秒（KV 未寫）。

### 移植結果

- 照原作改：
  - 力量改送 `StatKey::StatsStrengthBonus` 4/8/12/16（package `custom.strength_bonus` → extras `custom_strength_bonus`），由 omoba-core 三圍 settle 換算，刪掉手算的生命／回復／攻擊（`learning_grants_source_strength_and_replaces_the_previous_rank`）。E168：技能力量只以 `__attributes` HealthBonus 套用，不寫入揭露的 `attribute_hp`。
  - 光環改用通用光環發射器 `aura_emitter`（E173，與道具光環同一機制）：英雄 tick 每 **0.25** 秒在自身刷新 0.5 秒的發射器，核心每次 settle 把 `b29_morale` 刷新到 600 內友軍（含自身、不含建築），離開後殘留 AURA_LINGER **0.5** 秒（原本 0.75）（`the_aura_rearms_one_emitter_per_refresh_with_the_rank_bonus`）。
  - 收到者 buff id 改為不含施法者的 `b29_morale`：Dota 同名光環不疊加，兩位村上只算一份（原本依施法者疊加）。
- 保留：輸出傷害 % 只作用於普攻（`BaseDamageOutgoingPercentage`；該版 Dota 是否也作用於技能傷害不確定）。

---

## R b29_earth_tremor（B29T 大地震撼）

### 原作行為對照卡

- 觸發：`AOE | POINT | CHANNELLED`（`B29.txt:398`），引導 4.5/5.2/5.9 秒（413、475）；`OnSpellStart` → `B29T`（428-441）。
- `B29T`（`B29.lua:136-177`）：施法者位置對雙方開 100／6 秒視野（147-148）；`sk_radius = B29T_Radius*0.5`＝350/400/450（161）；Timer 自 0.1 秒起每 0.1 秒：施法者位置雙方 100／0.5 秒視野（164-165）、施法者隊伍在施法點 `sk_radius+100`／0.5 秒視野（166）、`B29T_Effect(point + RandomVector(RandomInt(10, sk_radius)), 45)`（167），先打再檢查 `IsChanneling` 與上限 50/60/70（169-175）。
- `B29T_Effect`（241-280）：0.45 秒後（246）砍 340 內樹（249）；340 內敵方 `ALL`、旗標 `MAGIC_IMMUNE_ENEMIES`（250-258）；非建築 45 魔法＋`modifier_stunned` 0.2（266-267）；建築 13.5 魔法、不暈眩（268-269）。
- 未使用：`AbilityDamage 60`、`B29T_Duration` 0.2/0.3/0.4。
- 冷卻 100（409）；魔耗 200/225/250（410）；距離 1100（411）；前搖 0.3（408）；6/12/18 級。

### 移植結果

- 照原作改：
  - 每擊 45 魔法、暈眩 0.2（各等級）；建築受 30%（`45×0.3`，Q10）不暈眩（`a_landing_strike_hits_340_with_45_and_a_0_2_stun_and_buildings_take_30_percent`）。
  - 打擊點距施法點 [10, 350/400/450]，距離均勻、方向均勻的決定性雜湊（`strike_points_stay_10_to_half_the_radius_from_the_cast_point_and_spread_out`）；`shape_radius` 700/800/900 只作指示圈。
  - 打擊次數 45/52/59（引導時間／0.1）。**45／46 的判斷**：t＝4.5 秒那一擊產生後若同幀 `IsChanneling()` 仍為真會再多一擊（46），取決於 Dota 引導結束與 Timer 的同幀順序，腳本無法判定；採不延長引導的 45/52/59（`ffi_declares_a_host_point_channel_with_the_source_channel_time`）。
  - 每擊產生後 0.45 秒才落下，引導結束或中斷後已出手的仍落下：通用能力 `delayed_effect`（pulse 排程 `delayed_effect:b29_earth_tremor:<擊序>`，`on_delayed_effect` → `land`）（`every_pulse_schedules_its_strike_0_45_seconds_later_and_refreshes_the_cast_point_sight`）。
  - 施法點己方視野：每擊對施法者隊伍開半徑 spread＋100、0.5 秒的一般視野（reveal_zone `detect=false`，不揭露隱形）。
- 保留／差異：
  - 中斷時原作會在中斷後的下一個 Timer 多出一擊；host channel 中斷即停。
  - 施法者死亡時尚未落下的打擊取消（排程隨死亡清除；原作 Timer 照常）。
  - 砍樹：本遊戲沒有可破壞樹木，不適用。
- needs_framework：
  - ~~為敵方隊伍開視野~~ 已實作（E185）：第一個 pulse 在施法者位置對兩隊開 100／6 秒，之後每個 pulse 開 0.5 秒（reveal_zone `teams: all`）。
  - 魔免：原作打魔免（旗標 `MAGIC_IMMUNE_ENEMIES`），引擎沒有魔免，已等同包含。
