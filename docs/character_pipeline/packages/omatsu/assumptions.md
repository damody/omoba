# H03 阿松：假設與阻塞

來源：`damody/history_nobu@f966720`，A34 current 版（`S/npc/A_Oda/A34.txt`：A34W 13-144、A34E 147-330、A34R 333-506、A34T 509-664；腳本 `S/vscripts/heroes/A_Oda/A34_Matsu.lua:1-193`；`S` = `game/dota_addons/nobu/scripts`）。結構化版本在 `source.lua`，逐招對照在 `parity.md`。

## 阻塞

無。

## 假設

- 基礎數值：由來源單位檔 `A34_2.txt` 決定（E157），package 不再撰寫。
- 只使用 current 版，不混入 `_old`、`_20` 變體（`addon_game_mode.lua:174,276`）。
- 冰山之襲（A34.txt:201-236）：先對指定目標 aoe_damage 100，再對 335 內每個敵人（含指定目標）AbilityDamage 100/200/300/400；目標與範圍都不含建築。攻速減速是攻速常數 −20 點（slow_self，`npc_abilities_custom_load.txt:6016-6024`）。
- 冰霜之息：
  - 命中半徑隨飛行距離由 150 線性增到 500（A34.txt:470-475）；表現層寬度仍是起始半徑。
  - 施法距離 800（358）、飛行距離 1200（480）分開；host 對方向點檢查 800。
  - 持續傷害每 1 秒 50/60/70/80（ThinkInterval 1，427-445），共 5 跳，含到期那一刻（見 `source.lua` frost_breath_dot_ticks）。
- 急速冷卻：
  - 6/12/18 級解鎖（524-525）。
  - 主動的冷卻上限、回魔、魔抗減免照 A34_Matsu.lua:10-123；魔抗以點數加總表達（`source.lua` magic_resist_points），不實作 70 上限。
  - 急速冷卻自身的施放不回魔（`source.lua` rapid_cooling_self_mana）。
- 魔免（原作設定，E170；引擎接上通用魔免時依此）：A34W 不打魔免（ENEMIES_NO）；A34E 主目標與範圍不打魔免（ENEMIES_NO）；A34R 技能欄位有 MAGIC_IMMUNE_ENEMIES，但投射物 TargetFlags NONE，實際不命中魔免；A34T 自身。全域規則：魔法傷害打魔免歸零（eventfordamage.lua:30-32）。
