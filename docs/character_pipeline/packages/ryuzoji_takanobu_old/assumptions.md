# C13_old 龍造寺隆信（舊版）假設

舊版五招都有原作繁中說明（`resource/addon_tchinese.txt:3762-3771`）；資料以腳本實際行為為準，下列為說明與腳本不一致或需要推定的地方（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C13"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:329`），`heromap_autoskill["C13"]["11"] = "D"`（432 行），C13D_old 出生一級。
2. **風之鎖（C13D_old）**：說明寫造成 200 傷害，但 `vscripts/heroes/C_Neutral/C13.lua:182-187` 只套狀態，`AbilityDamage 200`（`npc/C_Neutral/C13.txt:408`）沒有使用。依 D22 照說明補上 200 魔法傷害（類型取 AbilityUnitDamageType MAGICAL）。
3. **血腥狂風（C13W_old）**：召喚本體 C13R 同一種龍捲風單位，模板另取 id `c13w_old_wind`（召喚列待工作線 D 重產生）。「隨英雄等級成長」即 damage + 等級 × 5（`C13.lua:199-200`）。
4. **龍捲波（C13E_old）**：與本體 C13E 完全相同。
5. **力量強化（C13R_old）**：被動力量。
6. **龍捲之漩渦（C13T_old）**：說明寫結束後吹飛，但 Timer 第 134 跳才吹飛，30Hz 下要約 4.4 秒，超過 4 秒引導，實際永遠不吹飛（`C13.lua:218-306`）。依 D22 改成完整引導結束時吹飛 300 內敵人；被打斷時不吹飛（推定）。KV 沒寫 RequiredLevel，照 Dota 大招預設 6/12/18。
