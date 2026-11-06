# A07_v20 本多忠勝（2.0）假設

原作 `resource/addon_tchinese.txt`、`addon_english.txt`、`addon_japanese.txt`、`addon_schinese.txt` 都沒有任何 `A07*_20` 的技能名稱或說明。招式名取自 KV 檔的註解，說明全部依 KV 與腳本還原。英文名稱除舊版已有的譯名外為本檔自譯。

- **毫髮無傷（A07W_20）**：名稱取 `npc/A_Oda/A07.txt:1288` 註解。物理免疫依 `npc/A_Oda/A07.txt:1345-1353`（INCOMING_PHYSICAL −300），持續 2～5 秒依 `npc/A_Oda/A07.txt:1386`；魔法、純粹傷害取消並回復 30%（沿用本體 FC-025，致死傷害也擋得住） 依 `vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:27-46,123-130`。和舊版的差別是沒有 −40% 移速。假設 `ApplyDataDrivenModifier` 能套上 Lua modifier `modifier_A07W`，和本體、舊版的推定相同。
- **赤雷擊（A07E_20）**：名稱取 `npc/A_Oda/A07.txt:1405` 註解。無目標、以自身為圓心 525 範圍的魔法傷害與 75% 緩速依 `npc/A_Oda/A07.txt:1450-1514,1523-1543`。和暴雷擊共用冷卻依 `vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:364-380`。說明的「以巨槍重擊地面」是依動作 ACT_DOTA_ATTACK 與粒子還原的描述。
- **暴雷擊（A07F_20）**：名稱取 `npc/A_Oda/A07.txt:1549` 註解。單體暈眩加魔法傷害依 `npc/A_Oda/A07.txt:1595-1657`（與舊版 A07E_old 相同的 modifier）。英文名 Raging Thunder Strike 為本檔自譯。
- **碎擊槍（A07R_20）**：名稱取 `npc/A_Oda/A07.txt:1662` 註解。擴散依 `vscripts/AON_Cleave.lua:131-163`：50%、戰國最強期間 100%，半徑讀 KV `radius`。原作暴擊沒有效果（讀不到 `crit_persent`，`vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:81-121`）；依 D22 照設計意圖修正，倍率取 KV 第二個 `CleavePercent` 140～200（`npc/A_Oda/A07.txt:1756`），保底沿用本體 FC-046。說明照修正後寫。
- **戰國最強．啟動（A07T_20）**：KV 沒有註解，沿用舊版與本體的中文名。效果依 `npc/A_Oda/A07.txt:1840-1930` 的 modifier_A07T：魔免、移速 +60、力量 +15/25/35、BAT 0.8。`A07T_Transform` 找不到 A07W 而中斷（`A07_Honda_Tadakatsu.lua:256-265`），原作因此沒有本體的重置冷卻與連發；依 D22 照設計意圖補上：免費以目前等級施放毫髮無傷與赤雷擊（不耗法力、兩招冷卻不變）、解除敵方暈眩。說明照修正後寫。
- **雷擊切換（A07D_20）**：KV 註解寫「碎擊槍」（複製殘留），名稱「雷擊切換」／Thunder Switch 是本檔自擬。效果依 `vscripts/heroes/A_Oda/A07_Honda_Tadakatsu.lua:332-344`（SwapAbilities 赤雷擊與暴雷擊）。
