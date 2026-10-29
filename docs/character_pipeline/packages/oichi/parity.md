# H04 阿市（C17）原作對照與移植結果

原作：`damody/history_nobu@f966720`，路徑相對 `game/dota_addons/nobu/scripts`（以下 `S/`）。

- 技能 KV：`S/npc/C_Neutral/C17.txt`（C17W 47-160、C17E 162-243、C17R 245-319、C17D 321-389、C17T 391-475）。
- 技能腳本：`S/vscripts/heroes/C_Neutral/C17.lua`（udg_C05W_stun 1-5、new_C17D 40-85、new_C17D_DMG 87-108、C17W_Enemy 134-210、C17W 212-222、C17W_Friend 225-265、C17T 282-323）。
- 共用：`S/vscripts/amhc_library/amhc.lua` AMHC:Damage 535-584（攻擊者或受害者已死不傷害，540-542）。

以 hero-port-8 對照卡（`target/hero-port-8/cards/oichi.md`）為底。

## Q c17_light_chain（C17W 光鍊罰則）

### 原作行為對照卡

- 觸發：`UNIT_TARGET`（51），只對英雄（52）、TEAM_BOTH（53）；敵方走 C17W_Enemy、同隊走 C17W_Friend（217-221）。
- 冷卻 13（58）、魔耗 140（59）、距離 600（60）、前搖 0.1（57）。
- 對敵（134-210）：3 秒 Timer 後，`r = min(CalcDistanceBetweenEntityOBB, 1000)`；魔法傷害 `r × dmg_special`（0.35/0.52/0.69/0.86，152）；定身 `((1000−r)/200+1) × udg_C05W_stun[等級]`（0.3/0.5/0.7/0.9，1-5、187-191）＝貼身 1.8/3.0/4.2/5.4、1000 外 0.3/0.5/0.7/0.9。modifier_C17W（87-116）：ROOTED、IsPurgable、OnCreated 另 5 魔法傷害（109-114）。
- 施法者 3 秒內死亡：AMHC:Damage 不傷害，但 194 行仍套定身（＋5）。Timer 不能驅散。
- 對友：modifier_C17W2 受到傷害 −30/40/50/60%，3 秒（117-125、236）。
- 雙方視野：每 0.2 秒對兩隊在施法者與目標位置開 100 視野 0.3 秒，持續 3.3 秒（149-160、239-250）。

### 移植結果

- 已照原作改（通用能力 tether 擴充）：
  - 距離上限 750 → 1000，傷害 = 350..860 × r/1000（等價 r × dmg_special）。
  - 定身改仿射公式：`root_far + (root − root_far) × (1 − f)`，root＝1.8/3.0/4.2/5.4、root_far＝0.3/0.5/0.7/0.9。已驗證原作公式在 r∈[0,1000] 對 r 線性（`tether::tests::far_root_floor_matches_the_source_affine_formula`）。
  - 定身時附帶 5 點魔法傷害（`root_damage`）。
  - 施法者死亡：略過距離傷害，仍定身＋5（`root_after_source_death`）；死亡施法者沒有位置時以 1000 外計。
  - 距離改邊到邊：中心距離減兩者 CollisionRadius（英雄約 30），近似 OBB；判斷理由：引擎每個單位都有碰撞半徑，Dota 英雄 hull 約 24，兩者量級相同。
  - 只對英雄（拒絕小兵、建築、自己）。
  - 驅散：tether buff 原本就不在 cleanse 範圍（payload 只有 tether key），補測試確認；之後的定身可驅散，與原作一致。
- 保留：對友減傷、前搖、冷卻、魔耗、距離。
- 已實作（E185）：光鍊期間兩隊各有跟著阿市與目標的 100 一般視野 3.3 秒（reveal_zone `teams: all`＋`follow`）。原作每 0.2 秒在當下位置開 0.3 秒，等價於連續跟隨；單位消失時視野跟著結束。
- 不確定：能否以自己為目標（原始碼無法判定）；維持拒絕。

## W c17_victory_blessing（C17E 祈勝光輝）

### 原作行為對照卡

- `UNIT_TARGET`、友方 HERO＋BASIC＋BUILDING（166-168）；modifier_C17E 16 秒（201-213）：DAMAGEOUTGOING +7.5/15/22.5/30%、護甲 +5、回血 5/10/15/20；可驅散；重複施放刷新。
- 冷卻 12（170）、魔耗 130/150/180/210（171）、未寫施法距離（tooltip 全區域）、前搖 0.1（169）。

### 移植結果

- 不變（已一致）：數值、16 秒、可對建築、全圖（施法距離 10000，假設 global_range）。

## E c17_divine_strike（C17R 神助一擊）

### 原作行為對照卡

- `POINT | AOE`；敵方 HERO＋BASIC，腳本排除建築（33）；200 範圍魔法 200/300/400/500（311、316）。
- 冷卻 7/6/5/4（255）、魔耗 100/110/120/130（256）、距離 600/700/800/1000（257）、前搖 0.25（254）。

### 移植結果

- 已照原作改：冷卻 7×4 → 7/6/5/4；魔耗 80/100/120/140 → 100/110/120/130。
- 保留：傷害、半徑、距離、前搖、排除建築。

## R c17_aurora_guard（C17T 極光守護）

### 原作行為對照卡

- `POINT | AOE`、ULTIMATE；TEAM_BOTH HERO＋BASIC、Flags INVULNERABLE；腳本排除建築（309）。
- 冷卻 90（404）、魔耗 200/250/300（405）、距離 1050（406）、半徑 800（407）、前搖 1（402）；6/12/18。
- 腳本（282-323）：敵方 `dmg2 = maxHP × hpx/100`，`hpx` 不存在 → 0，所以固定造成 650/700/750 魔法（299、310-316）；同隊（含自己）`SetHealth(目前生命 + maxHP × regen/100)`，regen 40/45/50（318-319），不經治療事件。
- 前搖期間兩隊看得到 800 範圍提示（OnAbilityPhaseStart 267-280）。

### 移植結果

- 已照原作改（**依腳本實際行為，與 tooltip 不同**）：
  - 移除最大生命 20/25/30% 分支與 `damage_max_hp_pct`，敵人只受固定 650/700/750（tooltip 寫取高，腳本讀不存在的 `hpx` 永不成立）。
  - 友軍回復改「直接加生命」：ally_area_restore `direct` 選項（不經 received-healing 倍率，上限最大生命）。
  - 友軍排除建築：ally_area_restore `exclude_buildings`。
- 保留：前搖、冷卻、魔耗、距離、半徑、回復比例 40/45/50%（假設重複 "05" 兩者都保留，`aurora_heal_kv`）。
- needs_framework：前搖期間對敵隊公開地面範圍提示（若 cast_windup 尚未對敵方公開地面目標；本次未驗證）。

## extra1 c17_devotion（C17D 自我奉獻）

### 原作行為對照卡

- `PASSIVE | NOT_LEARNABLE`、MaxLevel 1；16 版 autoskill "D"（addon_game_mode.lua:436），開場學會。
- modifier_C17D OnAbilityExecuted → new_C17D（40-85）：切換技與 item_logging／item_tpscroll／item_flash_shoes／item_flash_ring／item_samurai_cloth 不觸發（49-54）；其餘技能與物品：800 內友方 HERO＋BASIC（含自己、不含建築，57-60）`SetMana(+4% 最大法力)`、`Heal(5% 最大生命)`（74-77）。
- OnTakeDamage → new_C17D_DMG（87-108）：條件 `caster.damagetype == 1` 永不成立（賦值已註解，eventfordamage.lua:21-28），沒有效果。

### 移植結果

- 已照原作改：
  - **依腳本實際行為，與 tooltip 不同**：移除物理格擋 20（tooltip 寫 −20，腳本條件永不成立）。被動 buff 只剩學會標記。
  - 物品主動使用也觸發（通用能力 `ScriptEvent::ItemUsed`），依原作排除切換技與 5 種物品（`owner_cast.rs`）。
  - 範圍排除建築（ally_area_restore `exclude_buildings`）。
- 保留：5% 生命（經治療政策，對應 Heal）、4% 法力、800、出生自動學會。
- 不確定：小兵法力（引擎只回有 mana_pool 的英雄）。
