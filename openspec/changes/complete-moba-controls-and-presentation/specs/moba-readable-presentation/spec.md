## ADDED Requirements

### Requirement: 可讀的正式MOBA場景
系統 SHALL 提供完成視覺的一張三路地圖，含lane／river／jungle／阻擋邊界、可區分hero／creep／tower／base、team色與shadow／lighting／materials。正式主體不得以debug cube、checker或漂浮HUD文字作最終呈現。

#### Scenario: 固定視點識別
- **WHEN** 在出生點、lane推進與多人交戰固定camera視點觀察
- **THEN** 可區分控制hero、敵友、creep、tower、base及可走／阻擋邊界；裝飾不遮重要目標，視覺collision與權威規則一致

### Requirement: 動畫、VFX與音效回饋
系統 SHALL 依權威cue／合法frame呈現idle／move／attack／cast／hit／death／respawn與recall、projectile／area／buff，搭配有界、可設定音量的UI／combat／ambient音效，避免隱藏來源資訊洩漏。

#### Scenario: 敵人隱藏與大量effects
- **WHEN** enemy失去視野或同時產生多個skill／attack效果
- **THEN** hidden actor與source效果按契約清除／隱藏；效果使用pool／節流／LOD，不靠per-entity無界UI掃描

#### Scenario: 正式Buff特效拒絕時仍維持容量上限
- **WHEN** 正式native Buff特效因共享池滿載、隱藏、失效目標或非法狀態無法配置
- **THEN** 不以額外legacy文字元件繞過原池上限，不為拒絕狀態新增ownership物件；目前有效的持續狀態可在容量回復後重新配置，既有自訂handler與明確選用的legacy呈現保留原契約

#### Scenario: 舊entity的Buff事件不影響目前occupant
- **WHEN** 單位物件或visual key重用後，default Buff added／state／removed／refreshed／updated回呼收到不同entity id或generation的payload
- **THEN** 在任何native pool、自訂追蹤、文字或debug視覺操作前拒絕，不移除目前有效handle、不改目前文字；同identity的有效事件仍可正常建立、更新與退休

#### Scenario: 已有native Buff刷新終止或被拒絕
- **WHEN** 目前entity已追蹤native Buff，而refresh／update收到zero remaining、無有效pool或pool拒絕目前狀態
- **THEN** 同步退休renderer與native ownership，不留下失效handle、不落入legacy分配或自動重試；合法有限值與無限期狀態仍保留，自訂handler與明確legacy路徑保持原契約

#### Scenario: 目前回城狀態在容量回復後補上圈框
- **WHEN** 有效回城的持續圈框曾被共享池拒絕，後續仍有效的動畫快照抵達且容量恢復
- **THEN** 補上恰好一個圈框，已成功呈現後不重啟age；池仍滿時以常數時間跳過補配，取消／隱藏／死亡／來源重設後不復活舊待配狀態，不重播oneshot或增加網路重試

### Requirement: 視覺證據與效能門檻
系統 MUST 保存before／after固定視點PNG、必要錄影、resolution／DPI／camera／quality設定與實際render capture來源；效能沿原固定基準，不提高門檻。

#### Scenario: 畫面驗收
- **WHEN** 宣稱場景／HUD視覺改善完成
- **THEN** 提供相同視點與尺寸可比較的actual viewport證據，確認游標／字體／fog／VFX可讀；fixture或局部採樣不冒稱整場固定效能PASS

### Scenario: 戰鬥特效保持動畫快照時序
- **WHEN** 同entity已接受新的回城／暫停動畫，稍後收到較舊動畫快照
- **THEN** 不得取消目前圈框、復活舊回城或改變目前暫停狀態，即使其tick仍高於最後生命值frame
- **AND** 同tick合法更新可取消回城／解除暫停，隱藏顯示保留動畫時序，新presentation epoch或角色身分重設可接受新時間軸較低tick
- **AND** 不新增timer、網路重試或逐entity掃描，不將受控快照回歸冒稱權威ACK

### Scenario: 骨架動畫拒絕落後目前生命值frame的快照
- **WHEN** 角色已接受較新的有效生命值frame，之後收到落後該frame的動畫快照
- **THEN** 不得以該過期動畫改變目前姿勢或暫停狀態，即使其tick高於上一筆動畫
- **AND** 位於目前frame的動畫及同tick合法更新仍可切換姿勢／暫停，新epoch與角色重用能接受新的時間軸
- **AND** 由共用動畫actor既有frame時序判斷，不新增timer、掃描或網路重試

### Scenario: 生命值音效基準隔離角色與呈現來源
- **WHEN** 角色生命值音效入口收到外來entity、generation、presentation epoch或非所屬world／bridge的音效元件
- **THEN** 在更新任何音效基準、owner、scope與時鐘前拒絕，不得以目前角色身分產生死亡／恢復cue
- **AND** 外來future tick不抑制後續目前有效frame，新epoch較低tick先建立baseline，不假重生；目前恢復及死亡提示維持可用
- **AND** admission／caption回歸與實際device PCM輸出分別驗證，不新增timer或網路重試

### Scenario: 移動與生命值音效只接受目前綁定來源的目前frame
- **WHEN** 收到外來音效元件，或落後／超前目前audio presentation frame的移動與生命值狀態
- **THEN** 在更新基準前拒絕，不得吃掉合法起步、重播持續移動起步或污染HP clock
- **AND** 生命值與移動入口共用精確epoch／tick判斷；移動元件必須是角色目前綁定元件及同world／bridge owner
- **AND** 合法目前狀態與同tick轉換維持可用，不新增timer、掃描、重送或權威資料
