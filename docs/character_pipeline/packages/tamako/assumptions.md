# 玉子：假設與阻塞

來源：`damody/history_nobu@f966720`，C15 current 版。原作腳本 `S/vscripts/heroes/C_Neutral/C15.lua` 在來源 repo；每招的原作行為與移植差異見 `parity.md`，結構化版本在 `source.lua` 的 assumptions。

## 阻塞（needs_framework）

- **魔法免疫狀態與目標旗標**：W 亂射、E 爆裂箭原作不打魔免；T 可對魔免施放。引擎沒有魔免狀態。
- **施法者自動攻擊目標**（`order_owner_attack`）：`new_C15T` 對施法者下佇列攻擊目標指令（`C15.lua:98-106`）；GameWorld 沒有設定施法者指令的入口。
- **英雄跨生命的腳本計數**：被動 `caster.C15T` 不因死亡重置（`C15.lua:114-119`）；本遊戲計數放在 buff，死亡即清除。
- W 亂射可閃避（`Dodgeable 1`）由通用能力 projectile_dodge 處理（w1）。

## 已用通用能力補上

- **爆裂箭火焰**：`DamageZoneSpec` 線形獨立區域（end／grow／linger）。火焰從前方 200 到 2600、約 3333／秒鋪出，每段 15 秒。
- **一般視野**（reveal_zone `detect=false`）：爆裂箭每段 200、12 秒的施法者隊伍視野（AddFOWViewer），不揭露隱形。
- **飛行追擊召喚物**（base_content `summons::chaser`）：致命箭矢直線飛向鎖定目標（移速 900），貼身（射程 58＝近戰 10＋雙方 hull 24）每秒攻擊；目標死亡後取 900 內最近敵人。

## 假設

- 距離單位：1 個來源距離單位等於 1 個本遊戲世界單位。
- 版本：只用 current 版。
- 不對應的欄位：欄 4 `magical_resistance`、欄 5 `C15_precache`，D 欄留空。
- 基礎數值：由 hero_source_stats 以 `C15_2.txt` 為準（E157）。
- 亂射：追蹤彈 1500、存活 3 秒（本遊戲取值）、最多 32 個目標、只鎖已揭露的敵人、空地照常施放。
- 爆裂箭：火焰半寬 170（KV `arrow_width`）；每段 15 秒（`duration 12` 未使用）；施法距離 1350 只影響指向；多段重疊只一個灼燒（原作是否多份無法判定）。
- 精準之光：輸出傷害 % 以 `BaseDamageOutgoingPercentage`；敏捷 5/10/15/20 送 `StatKey::StatsAgilityBonus`，由三圍 settle 換算。
- 百箭齊發：箭矢傷害採 KV 50/100/150；主動 10 支在目標周圍 300 每 36°；被動在目標位置；6/12/18 級解鎖；改鎖半徑 900（Dota 單位 AI，非隨機）。
- 魔免（原作設定，E170；通用魔免已接上，E179／E184：C15T `pierces_magic_immunity`，可對魔免敵人施放並暈眩；C15W／E 的魔法傷害歸零、減益被擋）：C15W 不打魔免（ENEMIES_NO，ActOnTargets 無旗標）；C15E 火焰 ActOnTargets 無旗標（不打魔免）；C15R 被動自身；C15T 可對魔免施放（SPELL_IMMUNITY_ENEMIES_YES、MAGIC_IMMUNE_ENEMIES），箭矢普攻為物理。
