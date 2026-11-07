# B15 今川義元（2.0）imagawa_yoshimoto_v20 假設紀錄

原作 `*_20` 招式沒有任何語系的技能說明（`resource/addon_*.txt` 轉碼後查無 `B15*_20`）。名稱取自 KV 註解，說明依 KV 與腳本還原。

1. **B15D_20 百步穿楊**：名稱取自 `npc/B_Unified/B15.txt:721`。說明依 KV `B15.txt:722-746`（Passive_B15D_20 OnAttackLanded）與
   `vscripts/heroes/B_Unified/B15_Imagawa_Yoshimoto.lua:421-430`（非建築 10% 暈眩 0.1 秒）。英文名為意譯。
2. **B15W_20 巨人之臂力**：名稱取自 `B15.txt:748`。說明依 KV `B15.txt:749-836`（ApplyModifier Duration 40、attack_bonus、power）；
   2.0 版沒有 RunScript，不放大模型、不淨化。
3. **B15E_20 烈焰箭**：名稱取自 `B15.txt:838`。說明依 KV `B15.txt:839-977` 與腳本 `B15_Imagawa_Yoshimoto.lua:15-24,74-101`。
   modifier_searing_arrow 的 OnAttackLanded 呼叫 `B15E_attack2`，它讀 `caster.owner.nobuorb1`；英雄與 B15T_20 幻象都沒有 `owner`
   欄位（全專案只有 `B15T_create_illusion:224`、A04、A27、B01 與道具召喚物會設），所以 nil 取欄位報錯，加傷從未發生。
   依主 session 決定（原作 bug 照設計意圖修，比照 D13）寫成 fixed：本體命中加 damage_bonus（以法球槽判斷），幻象命中在本體開著時加 damage_bonus2；
   B15E_attack2 沒有建築折減，照它不折減（推定設計意圖）。幻象判斷本體用 self.owner（create_illusion 的擁有者）。
4. **B15R_20 巨人之外皮**：名稱取自 `B15.txt:979`。說明依 KV `B15.txt:980-1018`（力量、物理格擋，與本體同數值）。
5. **B15T_20 東海道一弓取**：名稱取自 `B15.txt:1020`。說明依 KV `B15.txt:1021-1083` 與腳本 `B15_Imagawa_Yoshimoto.lua:326-419`。
   英文名為意譯。幻象魔法傷害倍率沿用本體 B15 的處理（只取 illusion_damage 0.3，不再乘 −80%），待實測。
   幻象到期是否觸發 modifier_B15T_20_count 的 OnDeath：推定會（modifier_illusion 到期即死亡），否則計數會卡在 10。
