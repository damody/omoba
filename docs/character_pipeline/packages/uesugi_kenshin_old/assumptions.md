# B32_old 上杉謙信（舊版）假設

原作 `resource/addon_tchinese.txt`（UTF-16）沒有 `B32*_old` 的技能說明，只有 modifier 名稱：`modifier_B32D_old_aura`「靈氣．軍神」、`modifier_B32R_old`「神烈」、`modifier_B32T_old`「毘沙門天」（2206-2231 行）。五招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取自這些 modifier 名稱，W、E 沿用本體的神斬、神閃。

1. **槽位**：`heromap_skill["B32"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:311`），choose_11 逐一 `AddAbility("B32?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）。`heromap_autoskill["B32"]["11"] = "D"`（414 行），出生自動學 B32D_old（PASSIVE | NOT_LEARNABLE、MaxLevel 1），寫進 `hero.auto_learn`。
2. **靈氣．軍神（B32D_old）**：說明依 `npc/B_Unified/B32.txt:173-216` 還原。自身 +7 再加光環 +2。光環對魔免友軍照樣生效、殘留 0.5 秒，都是推定。
3. **神斬（B32W_old）**：說明依 `B32.txt:349-480` 與 `vscripts/heroes/B_Unified/B32_Uesugi_Kenshin.lua:2-16` 還原。和本體共用腳本 B32W。可以指定魔免（356 行）。推定 datadriven ApplyModifier 對魔免目標照樣暈眩。
4. **神閃（B32E_old）**：說明依 `B32.txt:632-783` 與 `B32_Uesugi_Kenshin.lua:99-191` 還原。舊版用 FireEffect_IcePath2：冰道長度寫死 1200，沿面向排列五個判定點。施放點近時冰道也照樣延伸 1200。
5. **神烈（B32R_old）**：說明依 `B32.txt:841-949` 還原。`damage` 與 `damage2` 都用鍵名 `"01"`。推定兩者都生效（同 B08R_old）。`%damage` 同時用在三個屬性，所以拆成三個數值，source 相同。CANNOT_MISS DISABLED 不表達（no_effect）。
6. **毘沙門天（B32T_old）**：說明依 `B32.txt:1087-1238` 與 `vscripts/AON_Cleave.lua:42-65` 還原。推定 %attack_damage 是減免前的攻擊傷害，同 A28TE_old。CleaveAttack 的 CleavePercent 0 沒有效果。
