## Purpose

定義 Unreal 前端如何安全顯示 MOBA 對局與提交玩家操作，讓玩法狀態由 Rust 對局系統持有而 Unreal 保持可替換的畫面層。

## ADDED Requirements

### Requirement: 權威動畫暫停狀態
正式攻擊動畫 SHALL 使用後端明確發布的 paused 狀態，不從快照差異推測私人 Buff。公開狀態 SHALL 不包含控制來源或目標；原生播放器 SHALL 先依權威進度定位再停止插值，解除後由目前進度恢復。舊 selective wire4／C ABI13 SHALL 明確拒絕，不能忽略新暫停語意。

#### Scenario: 暈眩與解除
- **WHEN** 權威攻擊進度因暈眩凍結，之後控制效果解除
- **THEN** 公開 sequence／phase／progress 在暈眩期間保持且 paused=true，播放器凍結游標；解除後 paused=false 與進度正常更新，不發布 Buff ID 或來源。

### Requirement: 原生動畫重設與重複狀態安全
原生英雄 SHALL 在 presentation reset 清除舊動畫資產、action 與游標，不只停止播放。重複套用相同狀態 SHALL 不恢復已完成的非循環片段；已載入素材 SHALL 不重複同步載入。首次播放 SHALL 檢查實際素材及裁切區間，非法或空區間使用既有 fallback。

#### Scenario: 重設或替換較短素材
- **WHEN** actor 身分重設，或新素材使既有片段裁切為空區間
- **THEN** 舊 action 與 asset SHALL 清除，空區間 SHALL 不播放，後續有效片段仍可經通用路徑恢復。

### Requirement: 通用播放倍率不得覆寫權威相位時間
原生英雄 SHALL 使用 Lua 生成的 default_play_rate 與目前倍率播放一般動畫；非法倍率 SHALL 個別以 1 取代，結果保持有限且有界。合法權威攻擊相位 SHALL 以後端解析 duration 同步，不能再乘美術或 state 倍率改變相位時間，Impact 暫停保持為零播放速度。

#### Scenario: 高倍率與權威相位同時存在
- **WHEN** 作者指定非單位播放倍率，且快照提供合法 attack phase duration
- **THEN** 一般動畫 SHALL 使用有界合併倍率，該 attack phase SHALL 仍依權威 duration 定位與插值，不提前或延後由後端定義的命中時間。

### Requirement: 原生動畫缺片段的通用策略
原生英雄 SHALL 使用 Lua 宣告生成的 fallback policy。找不到可表示目前基本狀態的片段時，UseGenericState SHALL 嘗試通用 attack／idle；UseReferencePose／HideVisual SHALL 不以另一種 action 代替，並清除先前動畫資產與 action 狀態。無可用通用片段時 SHALL 使用參考姿勢，HideVisual SHALL 隱藏原生模型，不改變 actor 的視野隔離。

#### Scenario: 缺片段後恢復有效動畫
- **WHEN** 先前狀態因缺少片段套用 fallback，後續快照有可播放的合法片段
- **THEN** 原生模型 SHALL 恢復顯示並播放目前片段，不保留先前 action；重複缺失快照 SHALL 不重複載入失敗素材。

### Requirement: 正式攻擊動畫使用權威解析時間
正式 MOBA SHALL 以實際攻擊流程解析出的前搖／後搖發布目前公開狀態，不由 renderer 使用基礎攻速猜測。狀態 SHALL 只有序號、階段、已過時間與時長，不攜帶私人 Buff 來源或攻擊目標，也不得合成命中／暴擊。

#### Scenario: 目前狀態清除過期攻擊
- **WHEN** 可見英雄的權威攻擊狀態切換為待機
- **THEN** bridge SHALL 停止前一段攻擊動畫，不重播 legacy FX；更新只允許已有公開 entity／component，非法 payload 與舊 selective wire 3 SHALL 拒絕。

### Requirement: 安全呈現與輸入
Unreal 前端 SHALL 只顯示分配給本地玩家隊伍的資料，並將玩家操作交給 Rust client runtime 驗證與轉送。

#### Scenario: 編譯 catalog 缺值或錯配
- **WHEN** 本地bridge的catalog header／generation非法，或內容／介面雜湊缺值、格式錯誤、越界或不同於編譯registry
- **THEN** Unreal SHALL 拒絕啟動或停止本地bridge呈現，在frame處理及ACK之前阻擋，不將空signature當相容、不只warning後繼續、不重啟後端對局；新runtime清舊catalog／frame游標，每次成功取得catalog均需驗證

#### Scenario: 非法 frame 不得部分更新或消費
- **WHEN** frame header尺寸／ABI不相容、snapshot旗標非法，或任一正count沒有storage（含optional fog grid的cells）
- **THEN** Unreal SHALL 在actor／HUD／地圖／效果變更之前拒收，釋放lease且不得ACK或推進已處理sequence；保留上一份合法呈現，允許後續合法frame恢復，不重啟後端；零count可保留storage，不以任意entity上限限制高負載場景

#### Scenario: 呈現 ACK 可重試而不重播
- **WHEN** 合法frame已套用，但bridge拒絕本次MarkFrameConsumed
- **THEN** Unreal SHALL 在下次取得同frame的有效lease時只重試ACK，不重複套用呈現；每次仍驗證shape與frame／lease sequence，非法或未套用成功不得ACK，成功duplicate不重ACK，停止runtime重設消費狀態且每次取得均釋放lease

#### Scenario: 霧幾何改變或恢復
- **WHEN** 公開視野圓或遮擋幾何改變、scale改變，或無視野後恢复相同origin
- **THEN** Unreal SHALL 依完整公開幾何精確判斷是否重建，不只比較數量與整格origin；無視野或非法幾何清快取，成功建立並顯示mask後才commit，polygon span越界須在frame更新前拒絕，不以frame sequence強制每次重建

#### Scenario: Route 與 polygon 範圍整批准入
- **WHEN** 任一公開route或polygon的point start＋count超出各自point storage容量
- **THEN** Unreal SHALL 以共用寬整數範圍檢查在任何呈現更新前拒收整個frame，不局部跳過後更新其餘資料或ACK；合法尾端空範圍允許，不用任意point上限替代真實storage邊界

#### Scenario: Frame 文字參照越界或長度縮窄
- **WHEN** 任一非空frame文字ref超出string table，或ref／FX view長度超出Unreal converter表示容量，或非空FX view缺少storage
- **THEN** Unreal SHALL 在任何呈現更新及ACK前拒收整個frame，以寬整數驗邊界，不以空字串fallback或縮窄轉換放行；零長度保持無文字語意且不要求NUL結尾，lease仍負責storage生命週期

#### Scenario: 隱藏敵人
- **WHEN** 敵方單位離開本地隊伍視野
- **THEN** Unreal 清除或更新該單位的即時呈現，且不能取得其隱藏即時狀態

### Requirement: 未知內容的通用 fallback
系統 SHALL 在未提供內容身分時使用通用原生 fallback，不依照 entity kind 指定另一個英雄或小兵內容；catalog 未命中 SHALL 維持未知身分，而明確披露的合法 ID SHALL 保持原值。

#### Scenario: 未知內容不冒用其他角色
- **WHEN** 可見單位沒有合法內容 ID 或其 ID 不在 catalog
- **THEN** 畫面使用通用 fallback，不冒用預設英雄或 practice dummy 的資產與內容身分

#### Scenario: 同一 replica 的內容綁定變更
- **WHEN** 可見 replica 的內容綁定由未知變為已知或由已知變為未知
- **THEN** 依當前公開內容使用對應類別，替換時先取得新 actor 再回收舊 actor，失敗保留原 actor；保留同身分的一次性去重，不把切換當作 gameplay 重生

#### Scenario: 回收 actor 用於新身分
- **WHEN** 已回收 actor 從物件池用於新的 replica 或 disclosure generation
- **THEN** 清除舊身分的插值基準、tracked effects 與原生動畫 instance，再由當前公開 frame 建立呈現

### Requirement: 持續效果的狀態恢復
原生呈現端 SHALL 從當前已披露的 active buff baseline 恢復持續視覺，不重播歷史 Added／Refreshed 一次性效果。狀態 SHALL 綁定 live replica 與 disclosure generation，未知或非法記錄不能建立持續效果。

#### Scenario: 權威安全持續效果來源
- **WHEN** 正式對局發布同隊、有玩家 owner 且當前 Disclosed 英雄的 Buff 視覺狀態
- **THEN** 僅發布已註冊 ID 與剩餘時間，不發布任意 payload、來源或動態內部名稱；每 tick 更新與完整空集合清除，缺合法 baseline 的 replica 不接受注入，新線路版本拒絕舊協定

#### Scenario: 宣告式效果的公開身分
- **WHEN** Lua 已宣告且通過共用生成驗證的持續效果使用私有 source key
- **THEN** 由編譯 lookup 映射公開 Buff ID，同 ID 的有效來源合併最大剩餘時間、無限期優先，來源移除後重新計算；不公開私有 key／caster／payload，未宣告或不匹配的來源不映射

#### Scenario: Actor 重建後恢復既有效果
- **WHEN** actor 重建且當前安全快照仍提供合法 active buff state
- **THEN** 以原 visual instance key 恢復持續呈現，重複快照不重複建立 component

#### Scenario: 完整空 baseline 與控制 frame
- **WHEN** 完整快照已無效果，或收到不包含 baseline 的 control-only frame
- **THEN** 前者退役舊持續呈現，後者保留既有狀態，不能把缺值當成空集合

#### Scenario: 自己英雄的完整 HUD 清單
- **WHEN** 合法完整快照包含自己英雄的多個 Buff，或之後變成空列表、英雄離開完整快照、畫面停止或重置
- **THEN** HUD 以整份 owner-local 清單取代，精確匹配身分與世代，不被隊友單筆事件覆寫；空列表或停止／重置清除，control-only 保留，永久效果不顯示成零秒且不顯示任意內部 payload

### Requirement: 通用原生動畫狀態選擇
原生英雄 SHALL 從安全快照的 locomotion／overlay 恢復動畫，以建置期 Lua 配置生成的 state-to-slot binding 選擇既有片段；不得加入角色專屬判斷或重播歷史 toggle。生成階段 SHALL 拒絕未知目標、非法名稱與 FName 大小寫等價的重複來源名稱。

#### Scenario: 持續姿態與移動變體
- **WHEN** 當前快照有合法 overlay 或明確 locomotion variant
- **THEN** 明確變體優先於站姿預設，攻擊優先於兩者；走路缺片段使用普通移動，不選站姿，效果移除恢復正常待機變體

#### Scenario: 正式可見單位的移動觀測
- **WHEN** presentation-only快照對同一live ID與disclosure generation提供兩次有限且存活的已披露位置
- **THEN** 原生移動動畫可依當前位置變化選擇walk／stand，不讀private命令或隱藏目的地；重複tick不當成停止，較舊tick不連接旅程

#### Scenario: 移動觀測的生命週期
- **WHEN** 單位首次或再次披露、世代更換、死亡或非法座標，或發生retirement／ResetView／停止／重啟
- **THEN** 不使用舊生命／隱藏期間的位置推測移動，歷史僅保存目前live披露集合；control-only不產生新移動觀測，embedded／TD原動畫來源不被取代

#### Scenario: 缺少非必要動畫資產
- **WHEN** 宣告片段缺失或 skeleton 不相容
- **THEN** 快取失敗片段並同 frame 有界回退到可用普通動畫，不逐 frame 重載錯誤資產、不影響權威 gameplay

#### Scenario: 動畫相位時脈
- **WHEN** driver 已協商支援的60／90／120Hz或沒有合法時脈
- **THEN** 前者依實際 game time 計算相位與待機週期、impact一個tick，過期或未到攻擊不維持 action；後者不猜120Hz或從快照頻率推斷時間

#### Scenario: 從攻擊相位恢復片段游標
- **WHEN** 安全快照提供合法攻擊身分、相位、進度與時間，且生成片段包含合法命中點
- **THEN** 前搖在片段起點至命中點定位、命中停在命中點、後搖在命中點至終點定位；重建不重播已過前搖，普通快照與相位回呼共用路徑，游標不觸發動畫通知或 gameplay

#### Scenario: 未配置或非法攻擊片段
- **WHEN** 舊片段未提供命中點，或呈現相位／進度／時間非法，或實際匯入片段不足以容納命中點
- **THEN** 不推測命中位置，保留舊播放相容；缺值不同於零秒命中點，非法資料不修改計算游標。Lua非法命中點於兩邊生成階段拒絕並指出英雄／片段

### Requirement: 真正投射物命中的安全呈現
系統 SHALL 僅從權威實際 ProjectileHit 產生獨立命中 cue，不把移除、逾時或傷害事件當作命中。cue SHALL 只依賴當隊 visible 且 Disclosed 的受擊者 replica，沿共用保留／ACK／退役管線傳送，不公開隱藏攻擊者、投射物種類或推測座標。

#### Scenario: 可見受擊者的命中
- **WHEN** 權威產生實際命中且受擊者對本隊 visible、Disclosed 且仍 live
- **THEN** 原生畫面以獨立 target-local impact fallback 呈現，不把它當作傷害或彈道

#### Scenario: 逾時與不可披露受擊者
- **WHEN** 投射物只逾時移除，或受擊者已隱藏、記憶、忘記或退休
- **THEN** 不產生或退役該命中 cue，不能補推任何隱藏位置

### Requirement: 權威成功施法 metadata
系統 SHALL 只從成功施法事實產生一次性技能 cue。已知技能等級須捕捉於該次 invocation，不得由稍後 HUD、renderer snapshot 或輸入接受結果推測；只對目前可見且具有有效 replica mapping 的施法者發布。呈現 payload SHALL 使用明確版本與精確長度，未知版本或非法資料不得派發，未提供的等級 SHALL 保持未知。

#### Scenario: 通用技能 C ABI 投影
- **WHEN** bridge發布通用技能或宣告式Buff生命週期資料供Unreal消費
- **THEN** producer與consumer SHALL 共用OmAbilityProjection／event.projection，不建立角色專屬副本；C ABI16 SHALL 明確拒絕舊版，原事件身分／等級／披露gate與單位換算保持，通用命名不得恢復legacy自動事件派發

#### Scenario: 施法後等級改變
- **WHEN** 技能以等級 3 成功施放後，英雄的技能等級變成 4
- **THEN** 原一次性施法 cue 仍攜帶等級 3，保留與 ACK 不改寫該事件

#### Scenario: 技能等級越界
- **WHEN** 技能的正等級超過 handler 型別範圍或技能宣告上限
- **THEN** 施法在 handler 執行與資源變更前拒絕，不溢位、不發布成功 cue；合法施法的成本、handler 與 cue 使用同一次等級解析

#### Scenario: 舊 cue 不含等級
- **WHEN** 消費端接收到合法的舊版施法 cue
- **THEN** 保持等級未知，不自行補成 1，也不產生未公開的目標、座標或切換狀態

### Requirement: 安全施法者位移結果
系統 SHALL 從成功 handler 提交的施法者自身位移結果取得可選終點，不得用請求點或稍後 renderer 位置替代。事件 SHALL 明確區分位置存在與缺值，並只向來源同隊、目前可見且有有效 replica mapping 的呈現端公開此歷史終點。

#### Scenario: 世界原點與可見對手
- **WHEN** 英雄成功位移到世界原點，且兩隊均可見該施法者
- **THEN** 同隊 cue 明確保留零座標，對手 cue 不包含歷史位移終點，兩者仍遵守既有一次性身分／ACK／epoch規則

### Requirement: 安全範圍效果事件
系統 SHALL 在完整效果計畫與資源准入成功後，才從已解析的範圍中心、半徑與呈現時間產生獨立area cue；不把施法請求、施法者位移或匿名ordinal當作範圍效果來源。事件 SHALL 綁定真正caster並沿既有一次性保留／ACK／epoch規則，只向來源同隊且具live disclosed caster的呈現端公開。

#### Scenario: 合法空範圍施法
- **WHEN** 範圍技能合法施放但區域內沒有可命中的敵人
- **THEN** 同隊仍取得合法範圍呈現結果，敵人身分或隱藏命中清單不會出現在cue中

#### Scenario: 範圍施法遭拒絕
- **WHEN** 完整效果計畫、射程或額外資源成本未通過
- **THEN** 不發布成功area cue，且area不會被舊embedded adapter偽裝成另一個cast

### Requirement: 完整對局介面
Unreal 前端 SHALL 提供選角、移動、普攻、四技能、商店、裝備、HUD、小地圖、計分板與勝負畫面所需的操作與呈現。

#### Scenario: 原生守住與佇列語意
- **WHEN** configured owner 在 HUD／選角未攔截且 runtime 已啟動並連線時按 H 或 Shift+H
- **THEN** 原生 controller SHALL 從精確 chord 取得立即／佇列參數，共用 NoTarget HoldPosition 正式輸入，不從實體 Shift 取樣、不使用 player-one fallback；非法 owner 或不可用 runtime 不送出，queued 提交不當作權威已執行

#### Scenario: 自己英雄的權威護盾餘量
- **WHEN** 完整安全快照提供 configured owner 的護盾餘量，之後受傷、耗盡、到期或死亡
- **THEN** 共用原生 HUD SHALL 使用具名有界 Q10 狀態更新餘量，不讀私人 Buff payload 或從商品初始量猜測；死亡／缺英雄為零，完整缺 owner 或停止清除，control-only 保留，舊線路／IPC／C ABI 版本明確拒絕

#### Scenario: 購買物品
- **WHEN** 玩家在商店提交購買操作
- **THEN** Unreal 顯示後端接受或拒絕的結果及更新後的金錢與裝備

#### Scenario: 自己物品的主動狀態與冷卻
- **WHEN** 完整安全快照提供自己的六格物品與權威冷卻
- **THEN** 原生 HUD SHALL 結合生成 metadata 顯示主動／被動、剩餘與總冷卻；未知 ID、非法冷卻、死亡或不可用英雄不得標 ready，不能因不在商店就禁止標合法主動物品；完整缺 owner 或停止清空六格，control-only 保留，不從任意 Buff payload 推測盾量

#### Scenario: 原生六格主動物品快捷鍵
- **WHEN** configured owner 在 HUD／選角未攔截時按1–6，且有完整自己的可用主動物品 baseline
- **THEN** controller SHALL 沿唯一原生binding與共用入口提交合法槽位 NoTarget ItemUse，不使用player one fallback或要求商店距離；空／被動／未知／冷卻／owner錯配／斷線與不完整baseline不得送出，失敗清除輸入身分，重新連線前不得沿用已清除的ready，queued不得當作效果成功或預扣冷卻

#### Scenario: Owned 技能與回城安全准入
- **WHEN** 本地玩家要求施法、升級技能或回城
- **THEN** 通用owned入口 SHALL 驗證runtime已啟動且Connected，失敗清除baseline與舊輸入身分；只有完整owner HUD可建立ready，control-only不得恢復已清ready；施法冷卻必須finite、非負、剩餘不超總量且恰零，不把NaN／負值當ready，回城明確使用configured owner，技能點數／效果仍由Rust權威判定

#### Scenario: 斷線 HUD 與輸入一致
- **WHEN** runtime未啟動或診斷未Connected，且畫面曾取得owned HUD資料
- **THEN** Unreal SHALL 共用清除baseline並通知Hero／四技能／六物品／Economy／完整Buff列表為不可用，持續失效不得每tick重複發布，retained frame不得恢復owned HUD；世界呈現與frame消費保持原管線，停止runtime使用相同清除入口且可強制通知，新連線需合法baseline恢復；技能／物品／商店／小地圖即時連線失敗亦通知同入口，仍連線的非法單次操作只拒絕輸入而不清HUD

#### Scenario: 原生攻擊移動操作
- **WHEN** 玩家在有效遊戲視窗游標位置按 A 或 Shift+A，且 HUD／選角介面未攔截輸入
- **THEN** 通用原生 controller SHALL 透過既有 Rust 權威管線提交 configured owner 的攻擊移動，Shift+A 保留佇列語意；視窗外或非法投影不得提交，不依賴英雄專用 graph

### Requirement: 畫面重連
Unreal 前端 SHALL 在 renderer 重新連接時取得最新安全狀態，清理舊 actor 與一次性效果，且不重啟對局。

#### Scenario: 呈現生命週期清理一致
- **WHEN** 本地runtime停止、WorldBridge結束或建立新的runtime handle
- **THEN** Unreal SHALL 使用共用重設入口清除技能／回城ready、四技能HUD與槽位資料、runtime／HUD快取、地圖遮擋與fog、actor／route／ghost／terrain、frame消費及cue歷史；同handle重複Start不得視為新生命週期或清除待ACK狀態，清理不得重啟後端對局

#### Scenario: 退役後重複收集一次性事件
- **WHEN** 同一 view epoch 的事件已被有效 ACK 消費或因披露失效退役，之後又收集到相同 ID
- **THEN** runtime SHALL 不重新准入該事件；去重歷史須有界且淘汰後仍拒絕過舊 ID，重連後晚到且不晚於恢復基準的事件不得重播，新 view epoch 或重設使用新去重 domain

#### Scenario: PIE 畫面重啟
- **WHEN** Unreal renderer 斷線後重新連接仍在進行的對局
- **THEN** 畫面恢復最新可見狀態且不重播已消費的一次性技能效果
