# A32_old 瀧川一益（舊版）說明還原假設

原作所有語言檔都沒有 `A32*_old` 條目，KV 也沒有名稱註解；名稱與說明依 KV、腳本與本體同類技能推定。

- **忍法．怨靈飛翔斬（A32W_old）**：圖示 `dota2/a32_w_old` 與本體 A32W 相同系列，行為是本體一段斬去掉二段與馬甲排除（A32.lua:402-466），沿用本體名稱。
- **忍法．放逐（A32E_old）**：KV modifier 名稱 `modifier_A32E_old_Banish` 與註解「放逐」（A32.txt:603-604）；行為與本體幽魄刀主動的虛空相同（isvoid、繳械、增傷、緩速），名稱取「放逐」。
- **忍術．幽魄刀（A32R_old）**：只剩暴擊被動（A32.txt:683-720），modifier 名稱 `modifier_A32R_old_Crit` 與本體幽魄刀的暴擊同系列，沿用本體名稱。
- **忍術．斬魂刀（A32T_old）**：切換技，開啟時設 `caster.attackvoid`（A32.lua:493-512），與本體斬魂刀（A32F）同一套「物理傷害改魔法」機制，沿用其名稱；加攻、耗魔與範圍傷害依 KV `modifier_A32T_old`（A32.txt:827-845）與 `A32T_old_OnIntervalThink`（A32.lua:119-139）。
- `required_level` 6/12/18 是 Dota 大招預設（KV 沒寫）。
