# A33_old 本多正信（舊版）說明還原假設

原作所有語言檔都沒有 `A33*_old` 條目；名稱取自 KV 註解，說明依 KV 與腳本還原。

- **妖火（A33W_old）**：KV 註解 `//妖火`（npc/A_Oda/A33.txt:350）。效果是本體大招「傷害加深」的同一個 modifier（名稱同為 `modifier_A33T`）：減甲、提供視野、不能隱形 15 秒，施放後 0.2 秒自動攻擊（A33.lua:121-140）。英文名 Demon Fire 為自譯。
- **軀體腐爛（A33E_old）**：KV 註解 `//軀體腐爛`（A33.txt:415）。一秒後最大生命百分比魔法傷害與砍樹依 `A33E_old_DelayedAction`（A33.lua:154-180）；施放時在正信位置開雙方視野（A33.lua:150-151）屬推定的「實際行為」，說明不寫。
- **腐化靈光（A33R_old）**：KV 註解 `// 腐化靈光`（A33.txt:474），與本體同名；800 與 400 兩層光環依 KV `modifier_A33R_aura`、`modifier_A33R_aura2`（A33.txt:486-536）。
- **黑暗召喚（A33T_old）**：KV 註解 `//黑暗召喚`（A33.txt:539），與本體 Q 同名；骷髏數、緩速光環依 `A33T_old_OnSpellStart`（A33.lua:183-206）與 KV（A33.txt:560-628）。骷髏名稱「黑暗骷髏」為自訂（單位 `A33T_old_SKELETON` 沒有名稱字串）。
- `required_level` 6/12/18 是 Dota 大招預設（KV 沒寫）。
