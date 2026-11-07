# H06 淺井長政（B08）原作對照與移植結果

原作：`damody/history_nobu@f966720`，路徑相對 `game/dota_addons/nobu/scripts`（以下 `S/`）。

- 技能 KV：`S/npc/B_Unified/B08.txt`（B08D 14-140、B08W 143-287、B08E 421-528、B08R 621-678、B08T 746-875）。
- 技能腳本：`S/vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua`（B08W 7-11、B08D_Copy 13-43、B08D_old 45-108、B08E_Action 110-171、B08R_SE 190-235、B08R_Action 250-278）。
- 共用：`S/vscripts/amhc_library/amhc.lua` AMHC:Damage 535-584。

以 hero-port-8 對照卡（`target/hero-port-8/cards/asai_nagamasa.md`）為底。

## Q b08_fang_slash（B08W 一刀．牙）

### 原作行為對照卡

- 觸發：`DIRECTIONAL | POINT`（B08.txt:147）。OnSpellStart：LinearProjectile（193-208），RunScript `B08W` 對自己套 `modifier_B08W`（腳本 7-11）。
- 目標：敵方 BASIC＋HERO（不含建築），穿透（DeleteOnHit 0），TargetFlags NONE。
- 數值：魔法傷害 200/300/400/500（286）；速度 1500（268）；StartRadius＝EndRadius＝175（198、200、273）；固定飛行 1200（278）；視野 175（283）。
- 冷卻 7/6/5/4（159）、魔耗 100/120/140/160（160）、施法距離 1100（161）。
- 前搖 0.1（155）；施放後 `modifier_B08W` 0.1 秒 `MODIFIER_STATE_COMMAND_RESTRICTED`（253-261）。

### 移植結果

- 已照原作改：前搖 0.2 → 0.1；施法距離 1100 與飛行距離 1200 分開（`projectile_distance` extra，`b08_fang.rs` 不再用施法距離當飛行距離）；投射物半徑 87.5 → 175（Dota 的值是半徑）；施放後自身 `command_restricted` 0.1 秒（通用能力，見下）。
- 保留：傷害、冷卻、魔耗、速度、穿透、不打建築。
- needs_framework：
  - ~~投射物飛行視野~~ 已實作（E185）：刀牙帶 `projectile_vision` 175，飛行中施法者隊伍有跟隨投射物的一般視野（ability_projectile `with_vision`）。
  - 魔法免疫（SPELL_IMMUNITY_ENEMIES_NO、TargetFlags NONE）：引擎沒有魔免狀態。

## W b08_flash_cut（B08E 一刀．閃瞬）

### 原作行為對照卡

- 觸發：`UNIT_TARGET | IGNORE_BACKSWING`（425）；OnSpellStart → `B08E_Action`（450-468）。
- 目標：敵方 HERO＋BASIC，FOW_VISIBLE（426-428）。
- 數值：B08E_Damage 100/200/300/400（524）、暈眩 0.3（519）、燒魔 B08E_mana 100/160/220/280（514）。
- 冷卻 13（436）、魔耗 100/120/140/160（437）、距離 1100（438）、前搖 0.1（434）。
- 腳本依序（110-171）：ProjectileDodge（111）→ phased 0.1＋`SetAbsOrigin(目標位置)`（118-119）→ 非魔免時暈眩 0.3（121-123，魔免直接 return 124）→ 魔法傷害（125-126）→ 燒魔 `min(目標法力, B08E_mana)`（128-145）→ 另造成等於實際燒掉法力的魔法傷害（163-169）→ 自身 `modifier_B08E2` 0.1 秒 COMMAND_RESTRICTED（170、499-507）。

### 移植結果

- 已照原作改：前搖 0.3 → 0.1；落點由「目標前方 100」改為目標所在位置（host blink_landing，preferred＝anchor＝目標位置；對施法者身體不可走時留在原地）；燒魔附帶「實際燒掉量」的魔法傷害（mana_drain 封包 `damage` 選項，host 結算）；結尾自身 `command_restricted` 0.1 秒；結算順序改為原作順序（暈眩 → 傷害 → 燒魔 → 燒魔傷害）。
- 保留：傷害、暈眩、燒魔量、冷卻、魔耗、距離、需可見。
- 照原作改：施放第一步 ProjectileDodge（111）：送出通用躲避封包，追蹤施法者、可閃避的技能投射物與普攻投射物作廢（`every_rank_...` 測試斷言第一個 outcome 是躲避）。
- needs_framework：魔免分支（魔免目標只瞬移、其餘略過）——引擎沒有魔免狀態。

## E b08_living_soul（B08R 活心．魂）

### 原作行為對照卡

- 觸發：`PASSIVE`（625）；`modifier_B08R` 永久攻速 20/35/50/65（648、675）；OnAbilityExecuted → `B08R_Action`（636-644），對技能與物品都觸發。
- 排除：切換技（252）、item_logging／item_tpscroll／item_samurai_cloth（253-255）。
- `modifier_B08R_buff`（651-668）：PREATTACK_BONUS_DAMAGE 15、INCOMING_DAMAGE_PERCENTAGE −4、Duration 8，固定值，不乘 StackCount。
- 層數：首次 SetStackCount(1)（262-266）；之後 +1，上限 3+level＝4/5/6/7（260、267-277），每次觸發刷新 8 秒。層數只顯示。
- `B08R_SE` 的 damage_absorb 不存在（194），只有粒子。

### 移植結果

- 已照原作改：攻擊 +15、受到傷害 −4% 改成單一 buff（`b08_living_soul_buff`），每次觸發刷新 8 秒，數值不隨層數疊加；層數改成不帶數值的標記 buff（上限 4/5/6/7）；物品主動使用也觸發（通用能力 `ScriptEvent::ItemUsed`），依原作排除切換技與 logging／tpscroll／samurai_cloth（`owner_cast.rs`）。
- 保留：永久攻速 20/35/50/65、8 秒。
- 不確定（沿用對照卡建議）：264、268 行 `hCaster` 未定義；依字面意圖每次觸發都刷新。
- 純表現不做：B08R_SE 粒子。

## R b08_sky_fang（B08T 一刀．空牙）

### 原作行為對照卡

- 觸發：`DIRECTIONAL | POINT`（753）；LinearProjectile（787-801），穿透，魔法 800/1200/1600（874）。
- 速度 3000（854）、StartRadius＝EndRadius 500（792、794、859、864）、距離 3000（869）、ProvidesVision 0（800）。
- 冷卻 100/85/70（752）、魔耗 200（770）、施法距離 3000（766）、前搖 0.2（764）；RequiredLevel 6、間隔 6（761-763）；沒有 COMMAND_RESTRICTED。

### 移植結果

- 已照原作改：投射物半徑 250 → 500；飛行距離改讀 `projectile_distance` 3000（與施法距離同值）。
- 保留：傷害、冷卻、魔耗、前搖、距離、速度、穿透、6/12/18。
- needs_framework：魔免（同 Q）。`HasFrontalCone 1` 在起訖半徑相同時視為無影響。

## extra1 b08_endless_fang（B08D 一刀．連牙斷空）

### 原作行為對照卡

- 觸發：`UNIT_TARGET | NOT_LEARNABLE`（18）、MaxLevel 1；OnSpellStart → `B08D_old`（54-77）；OnProjectileHitUnit 魔法 100、穿透（78-92）。
- 目標：敵方 HERO＋BASIC。冷卻 150（29）、魔耗 300（30）、距離 1100（31）、前搖 0.3（26）。
- `B08D_old`（45-108）：ProjectileDodge（46）；雙方隊伍在目標原位置 1000 視野 3 秒（62-63）；施法者 invulnerable 2.2、施法者 stunned 2.0、目標 stunned 2.2（64-66）；
  i＝1..18：0.1·i 秒施法者 `SetAbsOrigin(pos[i])`＋殘影（75-78），pos[i]＝目標原位置＋700·(cos 40i°, sin 40i°)（68-70）；0.1+0.1·i 秒從 pos[i] 朝中心發射直線投射物：速度 2000、距離 1100、半徑 175（79-100）；1.9 秒回到目標原位置並 phased 0.1（104-107）。
- 被暈在原地的目標 18 顆全中＝1800；直線上其他敵人也受傷。

### 移植結果

- 已照原作改（取代原本「立即 1800 單體傷害」）：
  - 施放時：目標暈眩 2.2；施法者暈眩 2.0；施法者 2.2 秒不受傷害（`absolute_no_damage_*`）。
  - 英雄 UnitScript on_tick 依施法者標記 buff 驅動 19 步序列：第 n 步（0.1·n 秒，第 19 步＝1.9 秒）先從 pos[n−1] 發射 ability_projectile Linear（瞄準目標原位置、速度 2000、半徑 175、飛行 1100、穿透、每個敵人 100 魔法），再瞬移到 pos[n]；第 19 步回到目標原位置。同一 tick 內先發射再瞬移，所以每顆刀牙都從自己的環上位置出發，不需要新增「指定發射起點」能力。
  - 序列狀態（時鐘與中心座標）放在標記 buff 的剩餘時間（GameWorld 方法表已滿，腳本不能讀 payload；中心座標以與時鐘 buff 的剩餘時間差編碼，同批加入、同步倒數，差值精確不變），authority 與施法方隊伍 replica 跑同一個 on_tick。
- 保留：冷卻、魔耗、距離、前搖、16 級 auto_learn（原作學會途徑不明，假設 `endless_fang_learn`）。
- 照原作改：開頭 ProjectileDodge（46）：施放時送出通用躲避封包（`the_cast_dodges_first_...` 測試）。
- needs_framework：
  - 施法者無敵的「不可被選取」部分：BuffId::Invulnerable 沒有消費者；若只在 authority 排除選取，敵方隊伍 replica 沒有這個 buff（BuffStore 不揭露）會預測出不同的施法結果，需要先有可揭露給敵隊的無敵狀態。目前只做不受傷害。
  - ~~雙方隊伍區域視野~~ 已實作（E185）：施放時在目標原位置對兩隊各開 1000、3 秒的一般視野（reveal_zone `teams: all`，不顯形）。
  - 刀牙飛行視野（vision_radius 未定義，原作實際無視野）：不需要。
  - 殘影 B08D_SE、標題：純表現。
  - 終點 phased 0.1：本遊戲沒有 phased（單位碰撞穿越）狀態；終點直接 set_pos 到目標原位置。
