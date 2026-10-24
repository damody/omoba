## ADDED Requirements

### Requirement: 對局HUD資訊與狀態
系統 SHALL 呈現portrait、HP／合法資源、等級／經驗、QWER、D/F、item／trinket、gold／shop、team status、時間／比分、公開objectives、Tab scoreboard、C詳細屬性與死亡／回城／勝負狀態。資訊只來自允許的frame與權威回覆。

#### Scenario: 狀態更新與隱藏資訊
- **WHEN** hero受傷、升級、施法、死亡、重生或敵人離開視野
- **THEN** HUD與cooldown／disabled reason及team顯示一致更新；不呈現敵人隱藏位置／裝備／cooldown或假mana

### Requirement: 隊友頭像操作
系統 SHALL 讓隊友頭像依當前操作狀態提供鏡頭focus或內容允許的友方指定單位目標，使用公開且當前有效的owner／entity identity。

#### Scenario: 隊友頭像選擇與施法
- **WHEN** 玩家一般點隊友頭像，或正在瞄準內容允許的友方指定單位技能時點頭像
- **THEN** 一般點擊依設定 focus；瞄準只用當前公開且合法友方 identity 送同一施法路徑；死亡、hidden、owner 交接或 entity generation 改變時拒絕舊目標

### Requirement: hover、範圍與操作回饋
系統 SHALL 對技能／道具／stats／scoreboard提供可讀tooltip，展示key、作用、消耗、cooldown、rank与不可用原因。移動／攻擊／施法／回城／購物有可區分且有界的feedback，權威拒絕不播成功效果。

#### Scenario: 操作被拒絕
- **WHEN** 玩家在cooldown、缺錢、距離不符或disconnect時操作
- **THEN** 顯示對應原因並保留正確狀態，不只沉默地失敗，也不畫成功cue

#### Scenario: 閱讀提示失效時退休自己的焦點
- **WHEN** 長提示已取得閱讀焦點，而HUD範圍、來源、文字或可用client變更使提示失效
- **THEN** 擁有該提示的host在正常更新時關閉並移除舊提示，僅在焦點仍屬該提示時歸還原焦點；不依賴全域tooltip更新順序，不關閉其他host的提示，返回原範圍不復活舊提示

### Requirement: 商店與overlay操作
系統 SHALL 支援P或既有相容鍵開關商店、搜尋／分類、說明、purchase／sell／undo在權威支持的範圍內、recipe／upgrade提示，Tab按住／可設定toggle scoreboard，Esc取消aim後關閉overlay再開設定。

#### Scenario: 商店與Esc優先序
- **WHEN** 玩家在商店搜尋框輸入、捲動與按Esc
- **THEN** 字元不觸發gameplay；wheel不縮放camera；Esc先清除／關閉当前互動層；purchase／undo依權威receipt更新

#### Scenario: Runtime停止同步退休記分板操作
- **WHEN** 玩家按住記分板時直接停止或重建Runtime，且尚未執行下一個world tick
- **THEN** 輸入owner立即清除所有記分板hold槽位與按鍵，HUD同步關閉記分板請求；舊release不恢復舊狀態，新Runtime接受fresh press，冪等Start保留目前hold，控制profile不變

#### Scenario: Runtime停止退休HUD已揭露身分
- **WHEN** HUD已接受合法bound source的公開英雄投影後Runtime停止或重建，即使來源尚未選用且沒有world tick
- **THEN** HUD自己的scope回呼同步清除已揭露頭像／名稱及身分快取；新Runtime無身分的死亡快照不能繼承舊身分，同一Runtime內的死亡及冪等Start仍保留當前已揭露身分

#### Scenario: Runtime停止同步退休整組HUD投影
- **WHEN** Runtime停止或重建，即使bound source未選用且沒有world tick
- **THEN** widget自己的scope回呼清除舊結算modal、回城時鐘與文字、英雄資源／技能／召喚師、選取／Buff／小地圖、商店操作結果與公開記分板；舊modal不能封鎖新Runtime，fresh投影可重新使用同一widget，冪等Start保留live投影及使用者控制設定

### Requirement: 可讀版面與縮放
系統 SHALL 提供HUD與minimap scale、色盲team palette、字體可讀性與tooltip屏幕clamp，支援1920×1080／2560×1440及100%／150% DPI。

#### Scenario: DPI與極端tooltip位置
- **WHEN** 玩家變更DPI／window size／HUD scale，並hover畫面邊緣物件
- **THEN** HUD無重疊裁切，minimap命中與paint共用座標，tooltip保持可見，空白區可操作世界
