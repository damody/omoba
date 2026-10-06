## Purpose

定義原創三路 MOBA 對局的權威行為，讓單機與區網玩家在相同規則下完成一場有英雄、兵線、野區、建築與勝負結算的遊戲。

## ADDED Requirements

### Requirement: 權威對局生命週期
系統 SHALL 由後端決定選角、開局、進行、勝利與結算，並對所有玩家發布一致的對局階段。

#### Scenario: 摧毀基地
- **WHEN** 一隊的基地在合法進攻後被摧毀
- **THEN** 後端結束對局並公布獲勝隊伍，停止接受改變對局的輸入

### Requirement: 三路與經濟規則
系統 SHALL 支援三路兵線、防禦塔、基地、野區、英雄等級、金錢、物品、死亡與重生，並以固定輸入與種子產生可重現結果。

#### Scenario: 固定種子重播
- **WHEN** 同一版本以相同種子與輸入序列執行對局
- **THEN** 每個驗證節點的權威狀態雜湊相同

#### Scenario: 同導航格內的薄牆
- **WHEN** 合法起點與終點落在同一導航格，直線被公開地形阻擋，但在既有搜尋範圍內存在繞行路徑
- **THEN** 共用規劃器 SHALL 將精確終點作獨立連線接入有界搜尋，不因格身分相同直接拒絕；所有路段仍檢查碰撞半徑與swept地形，牆內終點或無合法連線不得傳送或穿牆

#### Scenario: 有界搜尋未找到完整路徑
- **WHEN** 終點本身被占據，或在既有搜尋範圍內只能抵達較近的格而不能連到實際終點
- **THEN** 共用規劃器 SHALL 返回失敗，不以closest cell偽裝成功；英雄沿原命令佇列處理失敗與後續合法命令，NPC保持原停留語意，不傳送、不放寬碰撞或把有限搜尋失敗宣稱全域不可達

#### Scenario: 限時物品移速效果與未支援效果
- **WHEN** 共用物品執行器收到合法限時衝刺，或非法／尚未支援的主動效果
- **THEN** 衝刺 SHALL 使用限時標準 Buff 且不改寫基礎移速，同來源刷新不疊加、不同來源使用最強family聚合；非法或未支援效果 SHALL 在效果／冷卻／命令變更前拒絕，不以日誌代替真正效果

#### Scenario: 正式生成主動商品
- **WHEN** 玩家購買並使用Lua來源編譯生成的合法主動商品
- **THEN** 權威60Hz管線 SHALL 從正式registry扣款、入六格背包、套用對應共用效果與冷卻，不需替換fixture或英雄專屬程式；護盾不回血、衝刺不改基礎移速、回魔事件僅記物品實際回復量，其他owner不受影響，冷卻拒絕不得重設或刷新效果

#### Scenario: 物品回復受管理魔力
- **WHEN** 玩家使用合法回魔物品，且自己的英雄有既有受管理魔力池
- **THEN** 共用執行器 SHALL 先驗證有限正量與當前容量，在完整預檢後回復到容量上限並發布實際正回復量；缺池或非法資料不得建立新池、耗冷卻或清命令，滿魔合法使用可耗冷卻但不得發布零增加事件

#### Scenario: 限時物品減傷
- **WHEN** 玩家合法使用有限比例與時間的減傷道具
- **THEN** 執行器 SHALL 提交限時負 DamageTakenBonus，沿共用傷害 packet 結算；同item刷新不疊加，不同item只採同群組最強效果，強效果到期後保留仍有效的弱效果，全部到期恢復原結算，非法資料不得耗冷卻或清命令

#### Scenario: 下一次普攻一次性增傷
- **WHEN** 英雄已合法準備一次性普攻增傷，之後建立正常普攻投射物
- **THEN** 共用執行器 SHALL 在合法 launch 時消耗一次並加到唯一真正 physical packet，視覺副本維持零傷害；命中率落空消耗且傷害為零，非法 launch 保留，同來源或跨道具重複準備只保留最強值，不永久改基礎攻擊力

#### Scenario: 真正限時護盾
- **WHEN** 玩家合法使用護盾物品並受到有效傷害
- **THEN** 共用執行器 SHALL 保持HP不因護盾使用而增加，正常非TD-layer packet先結算modifier再吸收、直接傷害亦吸收，溢出才扣HP；全吸收仍中斷回城，護盾耗盡或到期清除，重複授予只保留最強餘量，非法量／時間不得耗冷卻或清命令

#### Scenario: 舊文字物品入口與正式模式隔離
- **WHEN** 舊文字物品請求抵達後端
- **THEN** 正式 MOBA SHALL 在公開及私有物品入口拒絕請求；非 MOBA 相容模式 SHALL 只在六格整數槽位與精確唯一名稱、非零唯一 owner、存活英雄成立時委派共用執行器，不退回第一英雄、不重複效果邏輯、不將名字當認證，拒絕時不得改效果／冷卻或發布 completed

### Requirement: Bot 使用相同規則

系統 SHALL 讓 Bot 經由正式遊戲輸入行動，且僅使用其隊伍可獲得的對局資訊。

#### Scenario: 普通攻擊追擊目標未找到完整路徑
- **WHEN** Bot依原角色排序或focus選出目前披露但在本人普通攻擊射程外的目標，而公開地形與本人半徑下未找到完整追擊路徑
- **THEN** Bot SHALL 只在本次決策排除該攻擊候選，保留原披露供塔前警戒及兵線觀測，以原角色規則最多8候選內選其他合法目標，全失敗正式Hold且去重；不得永久黑名單或讀敵方私有狀態，射程內普通攻擊不新增LOS或導航需求，技能政策不套普通追擊導航，恢復地形重新判斷

#### Scenario: 五位置低血撤退未找到完整路徑
- **WHEN** 任一位置 Bot 需在目前披露威脅下撤回基地，而本人碰撞半徑與公開地形下的共用有界導航未找到完整路徑
- **THEN** Bot SHALL 不提交無法執行的撤退或改回進攻，保留合法自保技能優先；無自保行動時使用去重的正式 HoldPosition 取消舊攻擊，地形恢復後重新判斷並以普通 MoveTo 撤退，不使用私有敵方資訊、永久黑名單或傳送；定身的既有等待語意不變

#### Scenario: Bot 主動物品與可配置門檻
- **WHEN** Bot 出裝宣告合法 active_use，且自己已持有可用主動物品
- **THEN** 共用決策 SHALL 僅從自己的資源與目前當隊披露威脅產生 NoTarget ItemUse，不直接修改效果或冷卻；非法／非零冷卻、已有對應效果、定身衝刺及前搖增傷不得重送，省略政策保持舊行為，最後由正式權威管線提交

#### Scenario: Carry 本人已準備普攻增傷
- **WHEN** Carry 已合法準備下一次普攻增傷，並判斷目前披露的兵線目標
- **THEN** Bot SHALL 與正常投射物共用 outgoing modifier 後加上增傷的唯讀公式，不在觀測階段消耗效果、不讀對手增傷或隱藏目標；仍需合法命中率與已披露入傷倍率，真正消耗只發生於合法普攻 launch，不保證未來 impact 尾刀

#### Scenario: Bot 物品與普攻命令交接
- **WHEN** Bot 同時有例行回魔需求與目前披露威脅下的低血防禦需求，或已在普通攻擊前搖
- **THEN** 共用決策 SHALL 依效果種類選擇護盾／減傷／逃生先於回魔；例行回魔及下一擊準備 SHALL 在前搖延後，緊急防禦仍可打斷，槽位只作同類穩定排序，不改玩家或權威ItemUse取消規則

#### Scenario: 視野外敵人
- **WHEN** 敵人離開 Bot 隊伍的視野
- **THEN** Bot 不得以該敵人的即時隱藏狀態決定技能或攻擊目標

#### Scenario: Bot 位移候選被公開地形阻擋
- **WHEN** ApproachEnemyPoint的合法披露候選與本人之間有公開地形阻擋
- **THEN** Bot SHALL 使用共用swept地形檢查與本人碰撞半徑排除該候選，沿原排序找其他合法候選，全受阻繼續下一作者政策；不得以繞行導航放行直線位移，不讀hidden dynamic或預扣資源，不將該旅行限制套到普通範圍技能，地形改變後重新判斷並由權威handler最終驗證

#### Scenario: Jungle 的局部建築推進
- **WHEN** Jungle 沒有既有協防或可見野怪focus，且附近有目前披露的合法敵方建築
- **THEN** Bot SHALL 使用共用攻城選擇及正式輸入，塔需存活當隊披露兵支援、無支援依共用範圍守住，基地需披露為可攻擊；鎖定／隱藏／死亡或超距不得選取，不新增無協防條件的單挑或搶兵政策，沒有合法攻城機會才回公開camp巡邏

#### Scenario: Jungle 接近已披露攻城機會
- **WHEN** Jungle沒有協防／附近野怪或局部戰鬥／塔前等待行動，且current披露有合法遠處建築
- **THEN** Bot SHALL 以距離及canonical ID穩定排序選擇可攻擊基地或當前活同隊兵支援塔，提交普通MoveTo接近再沿共用AttackTarget；不得從hidden baseline產生新目的地或以AttackMove途中自動取得兵線目標，相同旅行不重送，無機會回公開營地巡邏

#### Scenario: 攻城旅行候選未找到完整路徑
- **WHEN** 合法披露候選在本人半徑與公開地形下未通過共用有界導航
- **THEN** Bot SHALL 在距離／canonical ID排序後最多8個候選內依序選第一個找到完整路徑者，無結果回原營地巡邏；不得用hidden候選、partial path或傳送替代，不作永久黑名單，地形改變後重新判斷，且局部focus／Hold優先不被旅行查詢覆寫

#### Scenario: 營地巡邏目的地未找到完整路徑
- **WHEN** Jungle最終決定巡邏且preferred公開營地點不在抵達範圍或未通過共用有界導航
- **THEN** Bot SHALL 沿preferred起的公開環狀順序，跳過已抵達點並最多查8次選第一個完整路徑，無結果使用持續HoldPosition且不重送；地形恢復重新判斷，不建永久黑名單、不讀private camp state或传送，原局部focus與攻城優先，命令接受不得當作同tick移動完成

#### Scenario: 兵線推進目的地未找到完整路徑
- **WHEN** Top／Mid／Carry／Support 最終決定沿公開兵線推進，但目前目的地未通過本人半徑與公開地形的共用有界導航
- **THEN** Bot SHALL 從preferred開始只依兵線前進順序、跳過已抵達點並最多查8次，選第一個完整路徑；不得環狀回頭或繼續保留不可達命令，全失敗用正式HoldPosition且去重，地形恢復重新判斷，不建立永久黑名單、不讀隱藏敵方狀態；既有戰鬥／塔前等待／護送優先不被新旅行查詢覆寫

#### Scenario: 護送披露位置未找到完整路徑
- **WHEN** Support最終決定護送目前已披露的Carry，而該目的地未通過本人半徑及公開地形的共用有界導航
- **THEN** Bot SHALL 以正式去重HoldPosition取消舊追隨，不改回進攻、不從隊友未披露的新位置猜替代點；地形恢復重新普通MoveTo護送，既有協防／自保優先保持，已經通過候選導航的攻城旅行不得在提交分支重複查詢而超過8次預算

### Requirement: 單機與區網一致
系統 SHALL 讓單機與區網模式共用權威玩法規則及輸入驗證。

#### Scenario: 區網選角區分本機與遠端席位
- **WHEN** 主機以interactive-selection、明確selection-bind與local-player啟動共享選角
- **THEN** 工作流 SHALL 只為本機已宣告真人席位開renderer，保留全體席位於同一Rust房間；預設仍loopback，服務ready地址必須精確符合選定interface與合法port，遠端使用自己的private invitation而不得替其他席位鎖定
- **AND** 遠端選角入口 SHALL 只啟動本機選角renderer，取得shared-room／owner／catalog一致的terminal receipt後結束；取消／逾時不得自動lock或啟動gameplay、不得停止remote host或洩露invitation token，正式對局仍使用host發布的final plan

#### Scenario: 選角與對局共用前端部署檢查
- **WHEN** 正式對局或私人邀請選角需要啟動本機 Unreal 前端
- **THEN** 工作流 SHALL 共用指定引擎的本專案 BuildId／模組 DLL 與既有 staged 副本雜湊檢查；選角檢查必須在建立該入口輸出與開 renderer 前完成，唯讀子程序必須明確 exit_code=0，不以成功文字取代成功狀態，不自動重建／stage／改 BuildId 或切換 profile
- **AND** 不需要本機前端的 dedicated server-only SHALL 保持不依賴 Unreal 檢查

#### Scenario: 已准入玩家取得主機完成後配方
- **WHEN** 唯一選角房間已由全部真人鎖定並明確finalize
- **THEN** Rust SHALL 在已綁定玩家的回覆提供同一份只讀final plan，尚未finalized不得提早分發；host的一次性take與成功finalize mutation仍各一次，read／EOF／stale／rebind不得重建開局權限或修改revision
- **AND** 遠端 SHALL 在terminal receipt的身分／catalog／locked roster與本人真人席位核對後，以指定profile compiled工具驗證完全相同plan才保存本機match-plan.json；缺plan或變動拒絕，不由seats拼接、不自動鎖定房間或啟動遊戲，正式對局沿既有明確connect入口

#### Scenario: 無本機 renderer 的專用權威主機
- **WHEN** 部署者明確選擇 server-only 角色
- **THEN** 工作流 SHALL 沿同一 recipe、Rust 選角鎖定、compiled-content-only 內容與60Hz設定，只建置／部署並啟動 Rust 主機，不要求 Unreal／bridge／client runtime；完整真人席位保留供遠端加入，本機 client 數為零
- **AND** server-only SHALL 拒絕 connect、本機席位、互動選角與原生結算畫面擷取；生命週期等待原始主機程序退役，不因沒有 renderer 而立刻關閉主機，只清理本次原始 executable／creation-token 身分的程序

#### Scenario: 啟動工具 profile 與禁止隱含建置
- **WHEN** 主機、遠端client、專用主機或prepare-only需要Rust選角鎖定與設定預檢
- **THEN** 工具 SHALL 執行指定debug／release的已編譯moba-config.exe，不以cargo run暗中建置或切換profile；缺檔應在建立輸出前拒絕並提供明確建置指令，程序失敗或缺exit code不得靠stdout通過JSON驗證

#### Scenario: 單機開局
- **WHEN** 玩家啟動單機對局
- **THEN** 玩家與九名 Bot 進入同一套權威對局流程
