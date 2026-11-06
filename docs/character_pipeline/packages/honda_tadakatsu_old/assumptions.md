# A07_old 本多忠勝（舊版）假設

原作沒有舊版技能的繁中技能說明，只有日文 `resource/addon_japanese.txt:1122-1129`，以及赤雷擊的 modifier 名稱（`resource/addon_tchinese.txt` 的 `DOTA_Tooltip_modifier_A07E_old`）。說明依 KV 與腳本還原。名稱取 KV 註解。

- **毫髮無傷（A07W_old）**：名稱取 `npc/A_Oda/A07.txt:289`。物理免疫與減速 40% 依 `npc/A_Oda/A07.txt:345-353`；魔法、純粹傷害回寫生命並回復 30% 依 `vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46,123-130`。這裡假設 `ApplyDataDrivenModifier` 能套上 Lua modifier `modifier_A07W`，和本體 A07W 的推定相同。
- **赤雷擊（A07E_old）**：名稱取 `addon_tchinese.txt` 的 modifier 名稱與 KV 註解。暈眩與傷害依 `npc/A_Oda/A07.txt:719-818`。英文名取 `addon_english.txt:1630`。
- **碎擊槍（A07R_old）**：名稱取 `npc/A_Oda/A07.txt:498`。機率 31/101、連續 4 次沒暴擊下一次必中（FC-046 比照本體修）、倍率 120～180% 依 `A07_Honda_Tadakatsu.lua:81-121` 與 KV crit_persent。日文說明寫 36% 與 1.4～2 倍，不採用。
- **戰國最強．啟動（A07T_old）**：名稱取日文「戦国最強．起動」與本體的中文用字。效果依 `npc/A_Oda/A07.txt:1084-1288` 的 modifier_A07T 與 `vscripts/AON_Cleave.lua:165-197`。`A07T_Transform` 原作找不到 A07W 而中斷；D22 比照 D13 照本體意圖修：免費施放已學的毫髮無傷、對射程內最近的敵人免費施放赤雷擊（舊版是單體指向技，推定）、解除敵方暈眩、0.3 秒後放大體型，冷卻不變，見 v3.lua 的 quirk。日文說明的持續 15/20/25、2.5 倍攻速、+60 攻擊力不採用。
- 英文名稱除赤雷擊外為本檔自譯。
