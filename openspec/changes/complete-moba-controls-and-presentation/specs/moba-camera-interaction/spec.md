## ADDED Requirements

### Requirement: 小地圖左鍵操作鏡頭
系統 SHALL 在既有預設與非施法瞄準操作，以小地圖左鍵按下及拖曳定位本local player的有效鏡頭，復用安全map bounds與DPI座標轉換，不提交gameplay command。預設右鍵與Shift右鍵維持MoveTo／queued MoveTo；玩家可另存允許小地圖移動／地點施法與左右位置偏好。

#### Scenario: 左鍵與右鍵各有單一效果
- **WHEN** 玩家左鍵點擊小地圖有效地點，再於相同地點右鍵
- **THEN** 左鍵只改camera focus，右鍵只提交一次權威移動；左鍵不產生MoveTo，右鍵不改camera

#### Scenario: 拖曳、迷霧與游標釋放
- **WHEN** 玩家拖曳經過迷霧或邊界，或拖曳途中失去焦點
- **THEN** camera clamp至合法界線；不新增隱藏敵方資訊；focus loss釋放capture且停止drag

### Requirement: 小地圖顯示與施法偏好
系統 SHALL 保存小地圖左右位置與允許移動／地點施法的獨立偏好，依目前owned geometry轉換合法座標，與camera／ping／modal／aim共用事件仲裁。

#### Scenario: 小地圖偏好與施法仲裁
- **WHEN** 玩家切換小地圖位置、移動／施法偏好並重新進入，或在合法地點瞄準時點小地圖
- **THEN** 版面與命中共用當前 geometry；只有開啟對應偏好才把合法 map point 送共用能力瞄準／權威驗證，不把 UI 像素當世界座標；ping／modal／取消／capture 不重複觸發 camera、move 或 cast

### Requirement: 有效鏡頭滾輪縮放
系統 SHALL 將實際viewport wheel事件送到該local player的有效camera，向前拉近、向後拉遠，有界、平滑，縮放step依目前尺度可感知。不得依賴未定義的named axis或未持有的pawn。

#### Scenario: 初始化後實際縮放
- **WHEN** 首個owned hero frame完成camera fit後，游標在world區域向前及向後各滾一格
- **THEN** 實際render camera的projection／arm length朝正確方向改變，不只改helper欄位；範圍與pitch保持合法

#### Scenario: UI消費wheel
- **WHEN** 游標在商店捲動清單上滾輪
- **THEN** 只捲動清單，camera不縮放；透明HUD空白區不攔截world wheel

### Requirement: 鏡頭模式、平移與回英雄
系統 SHALL 支援Free、LockedHero、TemporaryFocus與MinimapInspect，依profile提供edge pan、方向鍵／middle drag、Space回hero、Y鎖定與F1–F5隊友focus。focus只用允許的owner／team資料。

#### Scenario: 鎖定下檢視小地圖
- **WHEN** LockedHero玩家按住小地圖左鍵檢視其他地點，然後放開
- **THEN** 拖曳期間不被follow每frame拉回，release回復鎖定；Free模式release保留檢視點

#### Scenario: 暫時回英雄與焦點切換
- **WHEN** Free模式玩家按住Space，放開或失去viewport焦點
- **THEN** 按住跟隨owner hero，放開回先前模式／視點，focus loss取消held state；聊天輸入不觸發快捷鍵

#### Scenario: 邊緣捲動切換
- **WHEN** 玩家使用可改鍵的F9動作切換edge pan
- **THEN** 當前狀態可見且持久化；停用時邊緣游標不移camera，方向鍵與合法中鍵拖曳仍依profile運作

### Requirement: 小地圖視窗範圍框
系統 SHALL 由實際render camera與map平面交點生成viewport輪廓，隨pan／zoom更新，使用同一map transform，不能用固定裝飾矩形。

#### Scenario: 縮放改變範圍框
- **WHEN** 玩家拉近、拉遠或拖曳鏡頭
- **THEN** 小地圖範圍框面積／位置反映實際視窗；無效intersection安全隱藏輪廓
