# A04_old 竹中重治（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有 `A04*_old` 招式說明，只有 modifier_A04D_old「冰雪爆」、modifier_A04D_old_debuff「冰冷」、modifier_A04R_old_debuff「冰霜」的狀態名稱（1736-1743 行）。六招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`），招式名取 KV 註解（`npc/A_Oda/A04.txt:945,1048,1206,1316,1367,1679`）。

1. **槽位**：`heromap_skill["A04"]["11"] = "WERDFT"`、`heromap_autoskill["A04"]["11"] = "F"`（`vscripts/addon_game_mode.lua:246,349`）；`tofar_goback.lua:105-133` 依序 AddAbility 六招並自動學 F。A04D_old 不能學，由 `vscripts/library/events/eventforpichero.lua:131-142` 在 18 級時設為一級，寫成 auto_learn + requires.hero_level。
2. **式神．水（A04W_old）**：說明依 `npc/A_Oda/A04.txt:1049-1204` 與 `vscripts/heroes/A_Oda/A04_Takenaka_Juji.lua:416-423,612-625` 還原。bonus_attack、bonus_def 沒有被讀取，加成是寫死的 10 攻擊、1 護甲。
3. **凍結術（A04E_old）**：KV 缺 OnAbilityPhaseStart，共用的 FireEffect_IcePath 讀到 nil 的 ability.forwardVector 而報錯（`A04_Takenaka_Juji.lua:9-22`），原作整招沒有效果。D22 比照 D13 照設計意圖修：補上前搖開始時記下面向（同本體 A04E），沿面向放出冰道，非建築敵人暈眩 1/1.5/2/2.5 秒並受 150/200/250/300 魔法傷害（A04E_old 自己的 modifier_A04E）。說明照修正後的行為寫。
4. **冰霜攻擊（A04R_old）**：說明依 `npc/A_Oda/A04.txt:1317-1365` 與 `A04_Takenaka_Juji.lua:627-633` 還原：被動，命中非建築時降攻速 30、移速 30%。
5. **冰雪爆（A04D_old）**：說明依 `npc/A_Oda/A04.txt:946-1046` 與 `A04_Takenaka_Juji.lua:554-603` 還原：10 秒內每 0.1 秒在周圍 600 隨機位置爆炸。落點的角度以整數度數當弧度使用，照設計意圖寫成均勻方向。
6. **孤高的軍師（A04F_old）**：說明依 `npc/A_Oda/A04.txt:1680-1710` 與 `A04_Takenaka_Juji.lua:680-703` 還原：法球槽空著時，普攻命中非英雄額外 50% 物理傷害。
7. **吹雪（A04T_old）**：說明依 `npc/A_Oda/A04.txt:1368-1677` 與共用的 `A04T`（`A04_Takenaka_Juji.lua:467-548`）還原，結構同本體：每 0.7 秒一跳的 damage_scepter，緩速 5 秒（KV slow_duration 6/7/8 被腳本覆寫）。OnChannelFinish 呼叫的 A04T_End 不存在。
