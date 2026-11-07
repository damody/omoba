# B32 上杉謙信（2.0）uesugi_kenshin_v20 假設紀錄

原作 `*_20` 招式沒有技能說明；只有 `modifier_B32D_20` 的 tooltip（`resource/addon_tchinese.txt:2200-2201`，UTF-16 轉碼後確認）。
KV 的 `*_20` 區段只有 `// 20版` 一行註解，沒有招式名稱；名稱沿用本體同招（神斬、神閃、神陣、毘沙門天），D 槽取 tooltip「軍神．越後之龍」。
說明依 KV 與腳本還原。

1. **B32W_20 神斬**：KV `npc/B_Unified/B32.txt:1377-1496`、腳本 `B32_Uesugi_Kenshin.lua:2-16`。與本體同流程；可指定魔免（TargetFlags）。
2. **B32E_20 神閃**：KV `B32.txt:1499-1642`、腳本 `B32_Uesugi_Kenshin.lua:37-96,169-191`（用本體的 FireEffect_IcePath）。沿用本體 FC-056 修正。
3. **B32R_20 神陣**：KV `B32.txt:1645-1699`、腳本 `B32_Uesugi_Kenshin.lua:194-241`（與本體共用）。
4. **B32D_20 軍神．越後之龍**：KV `B32.txt:1240-1375`、腳本 `B32_Uesugi_Kenshin.lua:276-315`。tooltip 寫 +20，KV 是 +50，照 KV。
   移速攻速寫在 Properties（不經緩速池）。等級跟大招（B32T_20_upgrade）。
5. **B32T_20 毘沙門天**：KV `B32.txt:1702-1844`、腳本 `vscripts/AON_Cleave.lua:1-15`。`AON_Cleave_B32x` 雖名為分裂，實際只對主目標追加
   20% 物理傷害；KV CleaveRadius 10 推定是刻意縮成單體，照原作寫成追加傷害（不視為 bug）。
