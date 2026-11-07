# A11 齋藤道三（舊版）推定紀錄

原作只有 A11E_old 有繁中說明（resource 檔 DOTA_Tooltip_Ability_A11E_old，第 2589-2590 行）；其餘三招沒有說明，名稱取自 KV 註解，說明依 KV 與腳本還原。

- a11_old_vortex（A11W_old）：名稱取自 npc/A_Oda/A11.txt:596 註解「妖術．漩渦」。效果依 A11.lua:5-58（A11W()，舊版 A11W_duration = 10，所以十跳）。
- a11_old_viper_chain（A11E_old）：KV 定義兩次（A11.txt:177、:649，註解「妖術．冥鎖」與「妖術．腹蛇之鎖」）。照本專案「重複鍵後者生效」採 :649；原作說明（每秒 135、2.0~3.5 秒）對應 :177。兩份都執行 A11.lua:60-121 的 A11E()，第 86 行因舊版沒有 modifier_A11E_increase_int 報錯中斷；實際效果只剩 KV 的 datadriven 暈眩、施放傷害與每 1.05 秒的傷害（A11.txt:666-751）。A11E() 的目的地原作是世界原點；D22 照本體 FC-116 改成 lerp_point(目標, 道三, distance_percentage)，舊版該值為 0，目的地即目標原地，被暈眩涵蓋，資料只寫暈眩與傷害。說明的「中斷施法不會提早解除暈眩」是依 RemoveModifierByName 只移除同名第一個 modifier（先加上的 Lua 那個）推定。
- a11_old_cunning（A11R_old）：名稱取自 A11.txt:769「老奸巨猾」；只有 MODIFIER_PROPERTY_STATS_INTELLECT_BONUS（A11.txt:780-801）。
- a11_old_evil_wind（A11T_old）：名稱取自 A11.txt:805「禁術．妖風」；效果依 A11.lua:260-271 與 A11.txt:835-885。RequiredLevel 未寫，推定終極技預設 6/12/18。光環原本掛在 dummy 上，資料改掛在道三身上（全地圖半徑，位置無關）。
