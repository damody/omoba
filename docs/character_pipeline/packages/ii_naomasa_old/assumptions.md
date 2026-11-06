# A06_old 井伊直政（舊版）假設

原作沒有舊版技能的繁中說明（`resource/addon_tchinese.txt` 只有本體），只有日文 `resource/addon_japanese.txt:1103-1110`。四招的說明都依 KV 與腳本還原。

- **赤鬼之力（A06W_old）**：名稱取 KV 註解 `npc/A_Oda/A06.txt:202`。效果依 `npc/A_Oda/A06.txt:225-292`（modifier_A06Wx：移速 %、BASE_ATTACK_TIME_CONSTANT）與 `vscripts/heroes/A_Oda/A06_Ii_Naomasa.lua:27-53`（受傷時來源追加 original_damage × A06W_dmg 純粹傷害）。日文說明寫攻速 200～500%、移速 5～20%、追加 50% 真實傷害，與 KV 不符，不採用。
- **赤炎斬（A06E_old）**：名稱取 KV 註解 `npc/A_Oda/A06.txt:415`（日文「赤炎斬り」）。暈眩、傷害依 `npc/A_Oda/A06.txt:437-494`；擊飛依 `A06_Ii_Naomasa.lua:55-64`。擊飛距離約 1250、時間約 3 秒是用 `vscripts/physics.lua:911-942`、`1376-1391` 的預設摩擦 0.05／幀與停止門檻 20/30 推算，沒有實測。
- **戰魂（A06R_old）**：名稱取 KV 註解 `npc/A_Oda/A06.txt:562`（日文「炎駆けの戦魂」）。效果依 `npc/A_Oda/A06.txt:564-600`。
- **赤鬼之怒（A06T_old）**：名稱取 KV 註解 `npc/A_Oda/A06.txt:745`。效果依 `A06_Ii_Naomasa.lua:131-157`（75% 機率、失敗後下一次必中、半徑 450、純粹傷害、魔免減半）與 KV dmg `npc/A_Oda/A06.txt:786-792`。日文說明的範圍 550、傷害 170/250/340 不採用。
- 英文名稱為本檔自譯。
