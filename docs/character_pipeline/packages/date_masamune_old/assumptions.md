# B04_old 伊達政宗（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B04*_old` 說明，五招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/B_Unified/B04.txt:442,573,669,733,766`）。

1. **槽位**：`heromap_skill["B04"]["11"] = "WERDT"`、autoskill 空（`vscripts/addon_game_mode.lua:283,386`）；`tofar_goback.lua` 的 choose_11 依序 AddAbility W、E、R、D、attribute_bonusx、T。W/E/R/T → Q/W/E/R，D 留在 D。
2. **踏火無痕（B04E_old）不能學**：KV 有 `DOTA_ABILITY_BEHAVIOR_NOT_LEARNABLE`（`B04.txt:581`），火焰刀的 OnUpgrade `B04W_old_OnUpgrade` 把它設成同等級（`vscripts/heroes/B_Unified/B04.lua:225-233`）。寫成 `learnable = false`、`level_link = "b04_old_flame_blade"`，但仍放在 W 槽（它是實際出生就有、可施放的一般技能，不是換技能的子技能）。
3. **火焰刀（B04W_old）**：說明依 `B04.txt:443-571` 還原：直線穿透衝擊波，200～500 魔法傷害，飛行時提供視野。
4. **踏火無痕（B04E_old）**：說明依 `B04.txt:574-667` 與 `B04.lua:235-270` 還原：以 knockback 把自己推到施放點（速度 1500），期間 75 範圍光環給敵人每 0.1 秒一次 50～110 魔法傷害的 modifier。光環殘留時間 KV 沒寫，推定 Dota 預設 0.5 秒（`aura_linger`，hidden）。
5. **火焰強襲（B04R_old）**：說明依 `B04.txt:670-731` 與 `B04.lua:88-138`（和本體共用 B04E_OnSpellStart）還原；說明文字沿用本體 B04E 的寫法。
6. **靈活加成（B04D_old）**：說明依 `B04.txt:734-764` 還原：被動加敏捷 8～32。KV 沒寫 MaxLevel，採一般技能預設四級。
7. **火繩銃（B04T_old）**：說明依 `B04.txt:767-897` 與 `B04.lua:140-154,272-283` 還原：45 秒遠程、射程 +572/672/772、基礎攻擊 +90/130/170、必中；普攻發射時 87% 暈眩 0.1～0.3 秒（英雄與小兵、非魔免）。KV 沒寫 RequiredLevel，採 6/12/18（同本體 B04T）。
