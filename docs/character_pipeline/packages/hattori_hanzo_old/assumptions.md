# A13 服部半藏（舊版）推定紀錄

四招原作都沒有繁中說明（source-catalog 的 tooltip 為空）；名稱取自 KV 註解，說明依 KV 與腳本還原。

- a13_old_wind_step（A13W_old）：名稱取自 npc/A_Oda/A13.txt:160「忍法．風行步」。效果依 A13.txt:182-269 與 A13_Hattori_Hanzo.lua:160-181（A13D_End 讀 A13W_old 自己的 A13D_Damage）。隱形時間由 ApplyModifier 的 Duration 20 決定（modifier 的 %duration 不存在）。
- a13_old_chain_sickle（A13E_old）：名稱取自 A13.txt:415「忍法．勾鎖鐮」。A13_Hattori_Hanzo.lua:500 `A13E_old = A13E`，行為同本體；傷害類型依 A13E_old KV 的 PURE（A13.txt:423）。路徑拉回需要手寫 hook（同本體）。
- a13_old_shadow_clone（A13R_old）：名稱取自 A13.txt:624「忍法．影分身之術」。效果依 A13_Hattori_Hanzo.lua:333-437；自身 0.5 秒暈眩依 A13.txt:657-674（modifier_A13W 套在 CASTER）。幻象 30% 傷害依 :399（-70）與 :404（A13W_attack × 0.01）。
- a13_old_hidden_slash（A13T_old）：名稱取自 A13.txt:909「忍法密傳．隱斬」。效果依 A13_Hattori_Hanzo.lua:890-1021；飛刃幾何（目標點左方 700 起、朝右方飛 1400、沿面向 ±400）由 :903-935 推算；「左／右」依 v3 rotate 逆時針為正的推定。
