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

### Requirement: 視覺證據與效能門檻
系統 MUST 保存before／after固定視點PNG、必要錄影、resolution／DPI／camera／quality設定與實際render capture來源；效能沿原固定基準，不提高門檻。

#### Scenario: 畫面驗收
- **WHEN** 宣稱場景／HUD視覺改善完成
- **THEN** 提供相同視點與尺寸可比較的actual viewport證據，確認游標／字體／fog／VFX可讀；fixture或局部採樣不冒稱整場固定效能PASS
