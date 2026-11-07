# B13_old 百地三太夫（舊版）推定紀錄

原作舊版五招（B13D_old、B13W_old、B13E_old、B13R_old、B13T_old）在 `resource/addon_tchinese.txt` 都沒有說明，招式名取自 KV 註解，說明依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`（history_nobu f966720）。

## 槽位與學習

- 依 `vscripts/items/Addon_Items/tofar_goback.lua:96-140` 的 choose_11 與 `vscripts/addon_game_mode.lua:292`（heromap_skill B13 ["11"] = "WERDT"）依序加 B13W_old、B13E_old、B13R_old、B13D_old、B13T_old，W/E/R/T → Q/W/E/R，D 維持。舊版沒有縮地。
- `addon_game_mode.lua:395`（heromap_autoskill ["11"] = "D"）讓 B13D_old 出生即 1 級 → `hero.auto_learn`。`B13.lua:369-371` 的 B13T_old OnUpgrade 把它設為大招等級＋1 → `on_upgrade` 的 `set_ability_level`。

## 各招說明的依據

- **背刺（B13D_old）**：名稱取自 `npc/B_Unified/B13.txt:430` 的註解「D old 背刺」。效果依 `B13.lua:151-176`：只比較雙方面向的 yaw 差（|差| < 90 或 > 270，等同面向夾角小於 90 度），不看站位；目標不是建築才造成 B13D_old_damage 純粹傷害。說明寫成「與自己面向大致相同（背對三太夫）」。
- **忍法．遁地（B13W_old）**：名稱取自 `B13.txt:468` 的註解「old忍法．遁地」。效果依 `B13.txt:492-509` 與 `B13.lua:178-188`：隱形、繳械、魔免、定身，加固定生命與法力回復；再施放取消。說明特別寫出「無法移動」（ROOTED），與本體可移動、可縮地不同。英文名沿用本體的 Earth Hiding。
- **忍法．催眠（B13E_old）**：名稱取自 `B13.txt:526` 的註解。效果依 `B13.txt:566-602` 與 `B13.lua:315-342`：沉睡 B13E_old_duration；0.1 秒後若仍在沉睡，加 1 秒無敵（modifier_B13E_old_invulnerable）；受傷醒來並暈眩 0.1 秒。
- **忍具．隱雷（B13R_old）**：名稱取自 `B13.txt:604` 的註解「R old 忍具．隱雷」。效果依 `B13.txt:604-699` 與 `B13.lua:190-264`：在腳下放隱形地雷（生命 B13R_old_hp），敵方英雄進入 B13R_old_radius_detector 時爆炸，對 B13R_old_radius_explosion 內非魔免敵方 HERO|BASIC 造成物理傷害。無上限、無存在時間。英文名 Hidden Mine 為自訂。
- **忍法密傳．暴風（B13T_old）**：名稱取自 `B13.txt:699` 的註解。效果依 `B13.txt:700-826` 與 `B13.lua:267-313`：與本體相同的結構，數值不同（7 秒、每秒 400/600/800、緩速 60/70/80% 與 120/240/360 攻速）；緩速改走 dummy.lua Slow 系統。

## 與原作的差異或推定

1. **催眠的醒來對象（照本體 FC-015）**：原作 `B13.lua:334-342` 叫醒的是 `caster.B13E_old_target`（最後一次催眠的目標），不是受傷的單位。本體已依 D13 修正為受傷者自己醒來，舊版沿用同一修正（policy `fixed`），寬限旗標也照本體記在被催眠的目標身上（`port_adaptation`）。若舊版要完全照原作，需要另議。
2. **舊版沉睡不帶 tag `b13e_sleep`**：原作 `vscripts/items/Addon_Items/item_flame_armor.lua:10` 只跳過 `modifier_B13E_sleep`（本體），所以舊版沉睡中的目標會被灼燒光環叫醒。資料照原作，不加這個 tag。
3. **地雷同幀啟動**：原作 `active_delay = 0` 以 Timers 0 秒（下一幀）掛偵測光環（`B13.lua:205-209`），資料在 on_cast 同幀掛上。
4. **地雷多人同時觸發**：原作光環在同一刻可能對多名英雄各觸發一次爆炸；資料推定只爆一次（同前野 A36E），需實測。
5. **暴風跳數**：`B13.lua:280-310` 以浮點 `count += 0.1` 累加、`count > 7.1` 停止，以 Lua 5.4 實算為 71 跳（第一跳在 0 秒），寫成 `tick_count = 71`。
6. **暴風光環殘留**：KV 沒寫，照本體暫定 Dota datadriven 預設 0.5 秒（`assumed:`）。
7. **dispel 推定**：KV 沒寫 IsPurgable 的 datadriven modifier（遁地、沉睡、無敵、暴風光環）暫定 `basic`；Lua 的 modifier_B13D 寫 `none`；引擎 modifier_stunned 寫 `strong`。照本體。
