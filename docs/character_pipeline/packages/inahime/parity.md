# H05 稻姬：原作對照與移植結果

原作：`history_nobu@f966720b0956d002099d72aabec7ddc94b16d4bf`。`S/` = `game/dota_addons/nobu/scripts/`。完整分析見 hero-port-8 對照卡 `inahime.md`。

## Q a02_venom_arrow（A02W 蠍毒箭）

### 原作行為對照卡
- 觸發：`TOGGLE`（`S/npc/A_Oda/A02.txt:39`），無 CD、無開關魔耗；`modifier_A02W`（:113-139）法球：OnAttackStart → `S/vscripts/heroes/A_Oda/A02.lua:A02W_OnAttackStart:22-47`，OnAttackLanded → `A02W_OnOrbImpact:49-69`。
- 目標：ENEMY、HERO | BASIC（:40-41）；腳本另排除建築與魔免（A02.lua:42、54）。
- 數值：傷害 30/50/70/90 魔法（:45）；減益 4 秒（:52）；攻速 −9/−13/−17/−21（:57）、移速 −9/−13/−17/−21（:62）；每次攻擊 10 法力（:67）。
- 狀態機：攻擊開始先檢查法力 < 10 → 關閉並 return（:36-39）；目標不是建築／魔免才扣 10（:42-43），否則呼叫 `PerformAttack`（:45）；命中時對非建築非魔免目標套 debuff＋魔法傷害（:54-67）。減益經 `dummy.lua:Slow:816-858` 寫入 `as_slow`／`ms_slow`，`slow_self`（:866-902；npc_abilities_custom_load.txt:5987-6026）結算：移速依來源乘法疊加，攻速取最負者並以 ATTACKSPEED_BONUS_CONSTANT 點數套用。

### 移植結果（host attack_orb 通用修正）
- 照原作改：
  - 目標是建築：不扣魔、不附加傷害與減益（法力不足時仍先關閉）。
  - 攻速減益改為攻速常數點 −9/−13/−17/−21（content `attack_orb.attack_speed_slow_constant` → `AttackSpeedBonusConstant` 負值，aggregation family `slow_self_attack_speed` 只取最強來源）。沒有宣告此欄位的法球維持舊的 `attack_speed_multiplier`。
  - 扣魔時點改到攻擊開始：英雄攻擊前搖開始（`AttackPhaseCue`）時結算扣魔／關閉／建築判定，同一 attack_seq 的發射沿用結果；前搖被取消也已扣魔（與原作相同），下一次攻擊開始再扣。沒有前搖記錄就發射的攻擊在發射時扣（舊行為）。攻擊開始後才關閉開關，該次不附加（原作 modifier 已移除）。
- 移速減益照現有介面送出（緩速疊加規則由 w4 處理）。
- needs_framework：打建築時的 `PerformAttack(target,false,false,false,false,true,false,false)`（A02.lua:45）：腳本主動補一次不觸發法球的普攻，引擎沒有「腳本發動一次普攻」API；實際效果（是否等於打建築多一發）也無法從腳本確認。
- needs_framework：魔免判定（引擎無魔免）。
- 不適用：C14T 例外、ms_unslow 緩速抗性（目標方能力）。
- 已知邊界：開關在前搖中才打開的那一下，原作 OnAttackLanded 仍會附加效果但不扣魔；本實作在發射時補扣並附加（等同舊行為）。

## W a02_wind_prayer（A02E 風之祈）

### 原作行為對照卡
- `NO_TARGET | IMMEDIATE`（A02.txt:189）；`A02.lua:A02E_OnSpellStart:71-84` 回血 100/200/300/400（:199）；`modifier_A02E_buff` 攻速點 +40/70/100/130（:204、:270）10 秒（:209）；CD 15/14/13/12（:191）；魔耗 130/150/170/190（:192）；前搖 0.1（:190）。

### 移植結果
- 一致，未改動。動作替換、粒子不適用（純表現）。

## E a02_wind_drift（A02R 遊風行身）

### 原作行為對照卡
- `PASSIVE`（A02.txt:301）；`modifier_A02R_passive`：輸出 +6/9/12/15%（DAMAGEOUTGOING_PERCENTAGE，:308、:335）、魔抗 +9/18/27/36（:313、:336）。

### 移植結果
- 一致，未改動（輸出加成只放大普攻，與 Dota 慣例相同；魔抗乘法疊加）。

## R a02_shadow_meld（A02T 融影術）

### 原作行為對照卡
- `NO_TARGET | IMMEDIATE`（A02.txt:349）；`modifier_A02T` 持續 20/22/24（:358）、BAT 0.9（:363、:428）、移速 +60（:368、:429）、隱形＋無單位碰撞（:420-424），攻擊施法不破隱；CD 140/130/120（:350）；魔耗 200/225/250（:351）；解鎖未寫（取 6/12/18）。

### 移植結果
- 一致，未改動（BAT 0.9 以攻速乘數 ×2 等效，基礎 1.8 時相同）。
