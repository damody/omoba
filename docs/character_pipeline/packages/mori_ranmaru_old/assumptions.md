# A31_old 森蘭丸（舊版）說明還原假設

原作 `resource/addon_tchinese.txt` 沒有任何 `A31*_old` 條目；只有 `resource/addon_japanese.txt` 有舊版名稱與說明。以下說明依 KV 與腳本還原。

- **豪雷（A31W_old）**：名稱取自 KV 註解 `// 森蘭丸 [豪雷]`（npc/A_Oda/A31.txt:470）與日文說明。日文說明寫「持續 4 秒、最多 180/300/420/540」（四次 × abilityDamage）；腳本 `A31_Mori_Ranmaru.lua:364-384` 實際是施放後一秒起每秒一次、共十一次；D22 照設計意圖修成四次（`tick_count` = duration ÷ tick_interval，quirk 改 fixed）。說明裡的「英雄與一般單位」依 KV `AbilityUnitTargetType HERO | BASIC`。
- **瞬步（A31E_old）**：名稱取自 KV 註解（A31.txt:565）。腳本與本體共用 `A31E`（A31_Mori_Ranmaru.lua:7-29），說明沿用本體寫法，數值換成 `max_blink_range` 500/600/700/800。
- **嵐矢（A31R_old）**：名稱取自 KV 註解（A31.txt:635）。行為與本體相同（datadriven TrackingProjectile + ActOnTargets），說明沿用本體。日文說明寫射程 600，KV `AbilityCastRange` 是 800，照 KV。
- **五月雨（A31T_old）**：名稱取自 KV 註解（A31.txt:754）。純被動分箭（A31_Mori_Ranmaru.lua:389-431、225-240），說明沿用本體被動段落。日文說明寫 3/4/6 支箭，KV `arrow_count` 是 3/5/7，照 KV。
