## ADDED Requirements

### Requirement: 完整可持久化操作設定
系統 SHALL 提供action profile與可搜尋的設定介面，支援按鍵／滑鼠與Ctrl／Alt／Shift修飾鍵、衝突提示、restore defaults、camera／HUD／音量設定及版本化儲存。預設對局action矩陣以計畫文件為準。

profile MUST 包含normal-cast override、self-use item、attack-only／move-only-click、portrait focus、FPS／RTT切換與viewport screenshot等額外可設定action。可設定action沒有預設鍵時仍須有可執行語意，不用空handler占位。

系統 SHALL 以同一domain／slot／cast-variant仲裁Ability、Summoner、Item、Trinket可用槽的普通／快速／指示器與可改鍵override。自用僅限內容允許自身目標的shape，升級僅限支援rank的Ability；不以D/F索引冒充QWER或向不支援的shape提交假自用效果。

#### Scenario: 改鍵與重啟
- **WHEN** 玩家重新綁定一個action並重新開啟遊戲
- **THEN** 新鍵位仍有效、舊鍵不再觸發該action，單次事件沒有重複提交；錯誤設定保留診斷並使用合法預設

### Requirement: 獨立WASD操作profile
系統 SHALL 提供可選WASD與點擊控制的獨立英雄／全域設定，點擊維持預設；WASD復用權威命令、具有有界發送頻率、相同focus與死亡／disconnect回收。動態camera、Scout Ahead、45度方向為可設定action與偏好，不能佔用另一profile的QWER／camera鍵而重複執行。

#### Scenario: 對局模式鎖定與重連
- **WHEN** 玩家在match rules允許的時間切換輸入模式，或鎖定後重連
- **THEN** 切換原子停用舊profile並啟用新profile；權威規則決定鎖定時間，鎖定後不能藉重連繞過；owner／hero改變不套用錯誤角色設定

系統 SHALL 支援無障礙游標方向／速度改鍵與搖桿映射WASD，並提供選取、物件互動、普攻、確認施法的獨立輸入設定。允許同鍵時依上下文仲裁；必要移動鍵未綁不得安裝非法profile。

#### Scenario: 無障礙輸入與必要鍵驗證
- **WHEN** 玩家使用方向輸入移動游標、失去焦點，或保存缺少必要移動鍵的設定
- **THEN** 游標使用相同公開目標／UI判定且失焦停止；非法設定顯示錯誤並保留原profile；共用鍵不重複提交命令

### Requirement: A確認式攻擊移動與直接攻擊移動
系統 SHALL 提供AttackMoveAim與AttackMoveClick獨立action。預設A進入指示模式，左鍵有效地板確認一次AttackMove；右鍵／Esc取消。直接action預設Shift+A且可改鍵。既有queue modifier SHALL 與直接action分離。

#### Scenario: A點地板
- **WHEN** 玩家按A，再左鍵點地板
- **THEN** 顯示攻擊範圍／游標、提交一次AttackMove並退出aim；沒有額外MoveTo或第二次AttackMove

#### Scenario: 取消或點HUD
- **WHEN** aim期間按Esc／右鍵、失去焦點，或左鍵點HUD
- **THEN** 取消不提交原本瞄準的AttackMove／Cast；Esc只取消。右鍵取消後若仍是合法world事件，可由獨立普通右鍵路徑提交一次移動／指定攻擊，不重複；HUD事件由UI處理，不誤確認世界攻擊

### Requirement: 移動、目標攻擊、停止與保持
系統 SHALL 提供右鍵地板MoveTo、合法可見敵人AttackTarget、S Stop、H Hold、attack-move-on-cursor與只攻擊英雄modifier／toggle；權威區分Stop與Hold的行為，不以同一action假裝兩者。

右鍵持續按住更新目標時 MUST 節流並依pointer有效地點產生合法命令；離開viewport／進入互動UI／focus loss停止重複提交，不能每render frame無界送命令。

#### Scenario: 目標選擇與拒絕
- **WHEN** 玩家對可見合法目標右鍵，或在只攻擊英雄模式點非hero目標
- **THEN** 前者取得原command ID／ACK；後者不攻擊非法目標且有明確feedback，不能從hidden actor取得target

### Requirement: 精確點擊與攻擊偏好
系統 MUST 分別實作 MoveOnlyClick 與 AttackOnlyClick：前者於合法世界地點只送移動，即使游標下是敵人；後者只有游標下合法公開敵人時送指定攻擊，點空地不移動。英雄限定 hold／toggle 僅約束直接目標，不能隱式改寫 AttackMove 自動尋敵。關閉 auto attack SHALL 只停用 idle 尋敵，不停用明確 AttackMove／AttackTarget。

#### Scenario: 精確點擊與附加修飾鍵
- **WHEN** 玩家使用精確主／次鍵與 Ctrl／Alt／Shift 組合執行 MoveOnlyClick 或 AttackOnlyClick，或以額外未配置修飾鍵、repeat、失焦後舊release嘗試
- **THEN** 合法單次事件只執行對應指令；非法組合／舊release不提交；AttackOnlyClick 點空地不產生 MoveTo；改鍵後 HUD 標籤與持久化一致

#### Scenario: 攻擊偏好與大量鄰近單位
- **WHEN** 玩家切換游標／英雄附近選敵或 auto attack，附近有十個以上盟友／死亡單位、同距離敵人或射程邊界目標
- **THEN** 權威停止移動與攻擊選擇共用合法可見性、固定點距離及穩定 identity 規則；明確目標不被鄰兵擠掉；auto attack 關閉不阻止明確攻擊

### Requirement: 完整施法快捷鍵與生命週期
系統 SHALL 支援QWER normal／quick／quick-with-indicator、Ctrl+QWER升級、Alt+QWER自我施法、B回城、六個道具slot（新profile預設1/2/3/5/6/7）、D/F召喚師能力與獨立飾品／ward action（新profile預設4）。舊1–6物品profile保留合法設定並提示migration，ward與item不能共用一鍵而雙提交。新能力與ward MUST 有權威slot／cooldown／charges／target與視野生命周期，不能借用QWER或僅本機產生效果。

#### Scenario: 指示式施法
- **WHEN** 玩家按住indicator-mode技能，移動游標後放開或按Esc
- **THEN** 按住顯示合法range／shape；release只提交一次，Esc不提交，無效target／cooldown有明確原因

#### Scenario: 召喚師技能與插眼
- **WHEN** 玩家使用合法D/F或ward，之後再次在cooldown／無charges時嘗試
- **THEN** 合法action由權威產生結果與owner狀態；後者拒絕且沒有本機假效果；ward資料遵守team／fog揭露

### Requirement: 焦點與input owner仲裁
系統 SHALL 對native與既有Blueprint local controller使用單一可回收的action owner，支援PIE進出／重連／死亡／重生／新owner frame。文字輸入、modal與互動HUD優先，不得讓transparent overlay遮住世界操作。

系統 MUST 依action區分pointer target需求；游標hover非文字HUD時，Stop／Hold／Recall／SelfCast／Upgrade仍可操作，不能用整塊HUD矩形封鎖全部gameplay鍵。point／target命令不得透過HUD空白猜world target。

#### Scenario: 聊天與重連
- **WHEN** 玩家在聊天輸入QWER／Space，或按住action時發生disconnect／PIE stop
- **THEN** 文字不送gameplay／camera；失效input context與held state回收，新連線重新註冊一次，沒有stale pointer或duplicate binding
