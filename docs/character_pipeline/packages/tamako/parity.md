# 玉子（C15，current）原作對照與移植結果

原作：`D:\omoba\target\source\history_nobu`（`damody/history_nobu@f966720`）。以下 `S/` 代表 `game/dota_addons/nobu/scripts/`。

| 類別 | 檔案:行 | 內容 |
|---|---|---|
| 技能 KV | `S/npc/C_Neutral/C15.txt:17-122` | C15W 亂射 |
| 技能 KV | `S/npc/C_Neutral/C15.txt:124-272` | C15E 爆裂箭 |
| 技能 KV | `S/npc/C_Neutral/C15.txt:278-308` | C15R 精準之光 |
| 技能 KV | `S/npc/C_Neutral/C15.txt:310-467` | C15T 百箭齊發 |
| 單位 KV | `S/npc/C_Neutral/C15_2.txt:4-98` | Ability1-6＝C15W／C15E／C15R／magical_resistance／C15_precache／C15T；只掛 current 版 |
| 腳本 | `S/vscripts/heroes/C_Neutral/C15.lua:1-48`、`50-107`、`110-136`、`138-152` | new_C15E、new_C15T、C15T_OnAttackLanded、new_C15T_attack／end |
| 單位 | `S/npc/npc_units_custom_load.txt:5187-5259` | C15T_dummy（致命箭矢） |
| 系統技能 | `S/npc/npc_abilities_custom_load.txt:5597-5651` | majia／majia_vison |

`_old` 版（`C15.txt:472-798`、`C15.lua:156-215`）不採用。

---

## Q c15_scatter_shot（C15W 亂射）

### 原作行為對照卡

- 觸發：`POINT | AOE | IGNORE_BACKSWING`（`C15.txt:23`）；`ActOnTargets` 以落點為中心半徑 275（114），對每個敵方 `HERO | BASIC`（24-25、66-67）發一顆 `TrackingProjectile`（速度 1500、`Dodgeable 1`、不給視野，71-80）。命中：先 `Stun` 0.6/1.2/1.8/2.4（92-96、119）再 75/150/225/300 魔法（97-102、33）。
- 冷卻 18/15/12/9（39）；魔耗 140/160/180/200（42）；距離 700（43）；前搖 0.3（36）。
- 不含建築、不打魔免（`SPELL_IMMUNITY_ENEMIES_NO`，27）；追蹤彈無存活上限。

### 移植結果

- 數值全部與原作相同，本 worker 未改 handler。
- 可閃避由通用能力 projectile_dodge 處理（w1）。
- 保留：箭存活 3 秒（引擎 Homing 必須有 lifetime，4500 遠大於射程）；最多 32 個目標（每來源投射物上限）；只鎖已揭露的敵人（team replica 決定性）。
- needs_framework：魔法免疫狀態與目標旗標。

---

## W c15_burst_arrow（C15E 爆裂箭）

### 原作行為對照卡

- 觸發：`DIRECTIONAL | POINT | IGNORE_BACKSWING`（`C15.txt:130`）；`OnSpellStart`（153-184）：純視覺 `LinearProjectile`（速度 1666、距離 2000、不命中，165-178）＋`RunScript new_C15E`。
- `new_C15E`（`C15.lua:1-48`）：從施法者位置沿游標方向，`Timers` 每 0.03 秒共 25 次（22-23、41），每次先前進 100（26）再放 `npc_dummy_unit`（29）＋`modifier_C15E_2`（32），並對施法者隊伍 `AddFOWViewer` 半徑 200、12 秒（34）；dummy 15 秒後 `ForceKill`（36-40）。結果：前方 200、300…2600 共 25 段。`duration` 讀取被註解（12-13）。
- `modifier_C15E_2`（187-219）：15 秒、每 0.1 秒對半徑 170 內敵方 `HERO | BASIC` 套 `modifier_C15E_3`（1.01 秒、每 0.25 秒 25/30/35/40 魔法，220-235）。
- 冷卻 22/20/18/16（138）；魔耗 150/170/190/210（141）；距離 1350（144）；前搖 0.2（134）。

### 移植結果

- 照原作改：
  - 火焰持續 12 → **15** 秒（`damage_zone_duration`）。
  - 火焰中心從前方 **200** 到 **2600**（`damage_zone_offset` 200、`damage_zone_length` 2400），鋪設速度 100/0.03≈**3333**／秒（`damage_zone_grow_speed`，不再用箭速 1666）（`every_rank_lays_fire_from_200_to_2600_ahead_for_15_seconds`）。
  - 施法距離 2000 → **1350**（只影響指向）。
  - 每段火焰給施法者隊伍半徑 200、12 秒的一般視野（reveal_zone `detect=false`，不揭露隱形，對應 AddFOWViewer）；在施放時開出 25 個，第 k 段持續 12＋0.03(k+1) 秒使結束時間一致（`every_segment_gives_the_caster_team_plain_sight_for_12_seconds`）。
- 保留：
  - 同一施法者的多段火焰只維持一個灼燒（不同 dummy 的 `modifier_C15E_3` 是否各自一份無法由腳本判定）。
  - 視野開始時間最多比原作早 0.75 秒（全部在施放時開）。
- needs_framework：魔法免疫狀態與目標旗標（原作不打魔免）。

---

## E c15_precise_light（C15R 精準之光）

### 原作行為對照卡

- 觸發：`PASSIVE`（`C15.txt:282`），`modifier_C15R` `Passive 1`（285-287）。
- `DAMAGEOUTGOING_PERCENTAGE` 10/18/26/34（290、300）；`STATS_AGILITY_BONUS` 5/10/15/20（291、305）；玉子主屬性敏捷（`C15_2.txt:52`）。

### 移植結果

- 照原作改：敏捷改送 `StatKey::StatsAgilityBonus` 5/10/15/20（package `custom.agility_bonus` → extras `custom_agility_bonus`），由 omoba-core 三圍 settle 換算，刪掉手算的攻擊／攻速／護甲（`every_rank_grants_outgoing_damage_and_source_agility_and_replaces_the_previous_rank`）。
- 保留：輸出傷害 % 以 `BaseDamageOutgoingPercentage`（只作用於普攻）。

---

## R c15_hundred_arrows（C15T 百箭齊發）

### 原作行為對照卡

- 觸發：主動 `UNIT_TARGET | IGNORE_BACKSWING`（`C15.txt:316`），敵方 `HERO | BASIC`、可對魔免施放（317-321）；被動 `modifier_C15T_trigger` `OnAttackLanded`（367-379）。
- `new_C15T`（`C15.lua:50-107`）：先 `modifier_C15T_2`（暈眩 1/1.5/2、OnCreated 5 魔法，70、416-434、449）；迴圈 10 次 `deg += 36`，在**目標位置**半徑 300 產生 `C15T_dummy` 並下攻擊目標指令（68-89）；對**施法者**下佇列攻擊目標指令（98-106）。
- `C15T_OnAttackLanded`（110-136）：`caster.C15T` 每次命中 +1，達 8 歸零並在**目標位置**產生 1 支（118-133），不暈眩；計數為英雄 Lua 欄位，不因死亡重置。
- `C15T_dummy`（`npc_units_custom_load.txt:5187-5259`）：飛行、移速 900、近戰射程 10、`AttackRate 1`、`AttackAcquisitionRange 900`、視野 400、`DOTA_HULL_SIZE_HERO`；`majia_vison` 無碰撞、不可選取；`modifier_C15T` 無敵、攻擊 +50/100/150、10 秒（380-413、454-459）。
- 冷卻 140/115/90（328）；魔耗 150（331）；距離 700（334）；前搖 0.3（325）；6/12/18 級。

### 移植結果

- 照原作改：
  - 主動 10 支箭出生在目標周圍半徑 300、每 36°（第 k 支在 36k°）（`each_rank_stuns_and_releases_ten_arrows_assigned_to_the_target`、`ring_points_circle_the_target_every_36_degrees`）。
  - 被動箭出生在目標位置（`the_eighth_landed_attack_releases_one_arrow_on_the_victim`）。
  - 箭矢可移動：模板 `immobile` 取消、移速 900、射程 58（原作近戰 10＋雙方 DOTA_HULL_SIZE_HERO 24；host 以中心距離判定普攻射程）、視野 400、HP 10，由新的通用召喚物 AI `summons::chaser`（飛行直線追擊、忽略地形與單位碰撞、進入射程才攻擊、首擊即時）驅動（`arrows_chase_with_900_acquisition_and_a_hero_hull_melee_range`、`summons::chaser::tests::flies_straight_and_attacks_on_contact`）。
  - 改鎖半徑 700 → **900**（AttackAcquisitionRange），刪除「隨機」敘述：目標死亡後取 900 內最近的已揭露敵人（原作由 Dota 單位 AI 決定）。
- 保留：
  - 箭矢索敵沿用 host 的 `summon_target_allowed`（需揭露、視線不被牆擋），原作飛行單位的索敵細節不在腳本內。
  - 玩家對召喚物下指令的通用機制未對箭矢關閉（原作不可選取）。
- needs_framework：
  - 施法者自動攻擊目標（`order_owner_attack`：GameWorld 沒有設定施法者指令的入口）。
  - 被動計數跨死亡（英雄跨生命的腳本計數欄位；現在計數放在 buff，死亡清除）。
  - 魔法免疫狀態與目標旗標（原作可對魔免施放）。
