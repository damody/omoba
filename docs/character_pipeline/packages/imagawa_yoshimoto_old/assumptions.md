# B15_old 今川義元（舊版）推定與還原紀錄

原作 commit f966720，路徑相對 `game/dota_addons/nobu/scripts/`。`resource/addon_*.txt` 沒有任何 `B15*_old` 的名稱或說明，四招的名稱取 KV 檔裡的中文註解，說明依 KV 與腳本還原。

## 槽位
- `vscripts/addon_game_mode.lua:294` `heromap_skill.B15["11"] = "WERT"`；`vscripts/items/Addon_Items/tofar_goback.lua:96-140` `choose_11` 拔掉全部技能後依序 `AddAbility("B15W_old")`…`("B15T_old")`。
- `addon_game_mode.lua:397` `heromap_autoskill.B15["11"] = ""`，沒有自動學習；`vscripts/library/events/eventforpichero.lua:163-171` 的 8 級自動學習只找本體 `B15D`，舊版不適用。所以舊版沒有 D 槽，多重箭是 T 槽大招 `B15T_old`。

## 各招
1. **B15W_old 巨人之臂力**（`npc/B_Unified/B15.txt:107` 註解名；KV 107-195）
   - 說明依 `modifier_b15w` Properties（174-175：BASEATTACK_BONUSDAMAGE `%attack_bonus`、STATS_STRENGTH_BONUS `%power`）與 ApplyModifier Duration 40（147）還原。
   - 舊版 OnSpellStart（130-149）沒有 RunScript，所以沒有本體的模型放大與淨化；modifier 沒有 States，沒有魔法免疫。
   - 英文名 Giant's Strength 為自譯。
2. **B15E_old 烈焰箭**（註解 331；KV 331-428；腳本 27-37、57-72、87-101）
   - 扣魔 `B15E`（87-101）讀本招 `mana_cost` 10/12/14/16；命中 `B15E_attack3`（57-72）讀 `B15E_old` 的 `damage_bonus`，建築乘 0.2。
   - 死亡時關閉開關（FC-104／D13）照本體 B15E 的修正寫法，屬設計修正而非原作行為（v3.lua quirks `fixed`）。
3. **B15R_old 巨人之外皮**（註解 537；KV 537-563）
   - 只有 `block_B15R_old` 的 PHYSICAL_CONSTANT_BLOCK 40/80/120/160（551、560），沒有本體的力量加成。
4. **B15T_old 多重箭**（註解 642；KV 642-719；腳本 133-211）
   - 腳本與本體 B15D 共用 `B15D_SplitShotLaunch`／`B15D_SplitShotDamage`；數值讀本招 KV（arrow_count 2/4/5、range 800、projectile_speed 1800），目標類型與旗標讀本招 KV（HERO|BASIC、MAGIC_IMMUNE_ENEMIES）。
   - 傷害比例 0.9、幻象 0.27、武將 0.5 來自腳本 195-202，說明照本體寫法呈現。
   - 烈焰箭連動（157-160、203-207）：原作條件是 `B15E_old` 有等級且 `caster.nobuorb1 == "B15E_1"`。產生器不支援對其他技能呼叫 `orb_slot_free`（`fn:orb_slot_free of another ability`），v3 以「身上有 `b15e_old_orb`」近似。差異：法球道具（item_mana_thief 等）覆寫法球槽期間，原作箭不帶火傷，v3 仍帶；死亡後開關仍亮的原作狀態在 FC-104 修正下不再出現。
   - 火傷數值以 `$b15_old_flame_arrow.damage` 讀烈焰箭的等級（原作 `b15e:GetLevel()`）。
