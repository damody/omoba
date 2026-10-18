# 可讀且有界的HUD工具提示

被動加成接線後執行。SDK盤點確認：Unreal CoreStyle的ToolTip.Font固定8pt，預設WrapWidth為1000，直接ToolTipText沒有套用玩家文字設定。不能修改engine style或全域CVar作臨時解。

- [x] 建立共用Slate tooltip元件，動態字級沿既有可讀字體規則，依真正 owner viewport normalized allotment 推導寬高；不猜 OS DPI。長內容有界可捲動，不縮字。
- [x] 共用背包／商店道具、技能、召喚師、守衛提示；弱owner／退休與商店revision隔離，不捕捉裸UObject作延後回呼。
- [x] disabled卡真hover；提示wheel／keyboard／repeat不送世界操作，焦點可還原。
- [x] 真Slate hover／paint bounds／scale、400／800與text1／2首尾、Esc／pointer leave／退休／弱來源GC。實際 OS DPI 1.75；額外DPI只為model，不冒稱完整OS矩陣。

範圍接受：playable-mvp-readable-tooltips-accepted-v2；ordinary build-v10 no-cache／refresh=false，native-v5 exact8 Success／0warning，103來源SHA一致，8張當次原圖逐張檢視。E1163–E1173保留；不新增對局／ACK，非完整MVP驗收。下一批玩家商店交易文字。
