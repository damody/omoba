# H01 雜賀孫市：原作對照與移植結果

原作：`history_nobu@f966720b0956d002099d72aabec7ddc94b16d4bf`。`S/` = `game/dota_addons/nobu/scripts/`。完整分析見 hero-port-8 對照卡 `saika_magoichi.md`。

## Q b01_sniper_mode（B01W 狙擊模式）

### 原作行為對照卡
- 觸發：`IMMEDIATE | NO_TARGET`（`S/npc/B_Unified/B01.txt:19`），OnSpellStart 掛 `modifier_B01W`（:27-49）；腳本 `S/vscripts/heroes/B_Unified/B01.lua:B01W:3-28`、`B01W_end:30-39` 只處理粒子。
- 目標：自身。
- 數值：`bonus_atk` 0/10/20/30（BASEATTACK_BONUSDAMAGE，:57、:77）；`bonus_atk_rng` 200/350/500/700（:56、:82）；`bonus_sight_rng` 1000 夜晚視野（:58、:87）。
- 持續 12（:92）；CD 22（:20）；魔耗 100（:21）；前搖未寫（0）；無傷害。
- 特殊：`B01W` 讀不存在的 `keys.modifier_one`，只影響粒子（腳本缺陷，以實際行為：buff 由 KV 掛上）。

### 移植結果
- 一致：攻擊 +0/10/20/30（BaseAttackBonusDamage）、射程 +200/350/500/700、12 秒、CD 22、魔耗 100。
- 不適用：夜晚視野（無日夜循環）；動作替換、粒子、音效（純表現）。

## W b01_saika_corps（B01E 雜賀眾）

### 原作行為對照卡
- 觸發：`POINT`（B01.txt:319），施法距離 500（:323），前搖 0.15（:322），魔耗 35（:329），CD 0（:326）。OnUpgrade → `B01.lua:B01E:41-59`；OnSpellStart → `B01E_CHECK:99-149`；扣次數在 `S/vscripts/heroes/modifier_charges.lua:32-43`。
- 目標：地面點；`CreateUnitByName(..., true)` 自動 FindClearSpace（:105）。
- 數值：`count` 2/3/4/5（:372，同時是儲存上限與同時存在上限）；補充 15 秒（:377）；首次學會 2 次（B01.lua:45-47）。
- 召喚物 B01W_UNIT（`S/npc/npc_units_custom_load.txt:4720-4798`）：生命 500、護甲 1、魔抗 0、AttackRate 1.4、射程 725、索敵 800、移速 280、視野 1400/1100、賞金 20 金 5 經驗、無單位碰撞（for_no_collision）；存活 180（B01.lua:106）；攻擊 50＋5×英雄等級（:142-143）；持 great_sword_of_disease 生命 ×1.5（:113-115）；玩家可操控（:108）；1000 內沒有友方 B01 時 −90% 輸出（`dummy.lua:passive_b01w_unit:722-736`）。
- 狀態機：升級時記下 stack、以新上限重建 modifier（B01.lua:49-59）→ 升級不補次數、未滿時補充計時重新開始（modifier_charges.lua:15-22）；超過 count 時 ForceKill 最早生成者（B01.lua:116-140，`B01E_units` 檔案層級表，判定為「每位孫市」）。

### 移植結果
- 照原作改：召喚物生命 500、護甲 1、射程 725、攻擊間隔 1.4、存活 180、攻擊 55＋5×(等級−1)＝50＋5×等級、碰撞 0、視野 1400（取日間）。
- 新通用能力：每位主人同時存在上限＋替換最早生成者（content `summon.owner_cap` 2/3/4/5 → host `summon_owner_cap`，被替換者直接離場、不給賞金）；charges 升級政策 keep_count_restart（content `custom.charges_rank_up_keep_count` → host `ChargeRankUp::KeepCountRestart`）。首次學會滿 2 次與原作一致。
- 落點：host `summon_navigation::placement` 在落點附近 16 圈（最遠 512）找可放置點，對應 FindClearSpace；全部放不下才拒絕（不扣次數、不扣魔）。
- 保留：召喚物以 immobile 定點（原地待命、攻擊 725 射程內敵人）代替「可移動、玩家可操控」——玩家操控召喚物是 needs_framework（G-3）。定點 AI 的子彈速度是 host emplacement 的 900（原作 3000），索敵範圍等於射程 725（原作 800）。
- needs_framework：
  - 玩家直接操控召喚物（選取、移動、攻擊指令），G-3 未完成；完成後才能給移速 280。
  - 遠離孫市 −90% 輸出：需要「召喚物距主人超過半徑時套 buff」的 settle，而且要只用揭露資料判定（敵隊 replica 看不到迷霧中的孫市，無法一致決定是否減傷，E158/E168 原則），不是小幅擴充。
  - 賞金 20 金 5 經驗：正式 MOBA 的擊殺結算（`moba_match` lane／jungle 路徑）沒有召喚物分支，需新增召喚模板賞金欄位（template-ids schema）與結算路徑及經驗分配規則。
  - great_sword_of_disease 生命 ×1.5：道具旗標影響技能尚無 hook。
- 不適用：`cast` 0.3（腳本不讀）、phased_dummy、粒子音效。

## E b01_rain_iron_cannon（B01R 雨鐵炮）

### 原作行為對照卡
- 觸發：`PASSIVE | ATTACK`（B01.txt:223），OnAttackLanded → `B01.lua:B01R:308-345`（有效定義）。只在命中觸發。
- 目標：普攻目標；排除 `_G.EXCLUDE_TARGET_NAME`（`dummy.lua:331-374`：魔王、武將 com_general、兵營 com_soldiercamp、EARTH_WALL、地雷、dummy）。
- 數值：英雄 10/18/24/30%、建築 10/15/20/25%、其他 50/70/90/110%（:251-261），`%attack_damage × per / 100` 物理（:333-335）。

### 移植結果
- 一致：三類百分比、物理另一筆傷害、只在真正普攻命中觸發。
- 排除名單：本遊戲沒有魔王、武將、地雷、dummy 對應單位；兵營 com_soldiercamp（npc_dota_building，各路軍營）最接近的是 `moba_inhibitor`。但 lane structure 沒有 ScriptUnitTag（`unit_id_of` 回 None），GameWorld 方法表已滿，要做「單位模板旗標查詢」需新增模板欄位與 world 查詢介面 → 列 needs_framework。影響：本遊戲 inhibitor 會吃到建築 10～25% 加成，原作兵營不會。
- 不確定（保留）：`%attack_damage` 是否含暴擊；目前用命中當下 `get_final_atk`，沒有暴擊時相同。

## R b01_three_stage（B01T 三段）

### 原作行為對照卡
- 觸發：`IMMEDIATE | NO_TARGET`、ULTIMATE（B01.txt:461-463），掛 `modifier_B01T`（:481-489）。
- 數值：持續 7/11/13（:532）；`per_att` 200（BASEDAMAGEOUTGOING_PERCENTAGE，:513、:537）；CD 135/123/111（:465）；魔耗 300/350/400（:466）；前搖 0.1（:464）；6/12/18 級（:467-468）。

### 移植結果
- 一致：持續、CD、魔耗、前搖、解鎖等級、+200%。
- 條件性差異「只乘白字」：Dota 的 BASEDAMAGEOUTGOING_PERCENTAGE 只乘基礎（白字）攻擊；引擎 `unit_stats.rs final_atk` 以 `(base + PreattackBonusDamage + BaseAttackBonusDamage) × (1 + pct)` 計算，也乘了綠字 `PreattackBonusDamage`。孫市自己的技能只給 BaseAttackBonusDamage（白字），所以只有在其他來源（道具或他人技能）給 PreattackBonusDamage 時才會不同；本次不改全域公式（會影響所有英雄），記錄為條件性差異。
