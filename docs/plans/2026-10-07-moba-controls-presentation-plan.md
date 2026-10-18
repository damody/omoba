# MOBA完整操作與畫面補完計畫

2026-10-09 最新進度：OpenSpec父項8／44，3.8召喚師整合已完成。最新獨立證據 `.ai-collab/owned-summoner-accepted-v1/summary.json`：ordinary build-v3 nocache PASS、native-v2 exact63Success（1筆警告保留）、實際viewport-v2 158cases／69intents／51stages與清理全過，原小地圖121／58／36前綴比對成功。真人authority ACK／整套畫面與跨DPI／PIE／LAN及完整最終驗收仍待後續。Grok4.7非fast正在實作HUD可讀性／顯示偏好；後續道具地點／單位效果先補正式權威目標模型，再做快捷鍵全流程。下方各批6／44、7／44與舊ABI敘述保留其歷史時間點，不代表最新狀態。

## 目標與參照

以玩家可實際操作的LoL式MOBA為基準：鏡頭與小地圖可用、A點地板與修飾键完整、技能／道具／HUD狀態清楚、場景與戰鬥可讀。全部對局內基本操作列入實作；Riot客戶端／排名／社交平台不在此遊戲範圍。以原創資產製作，不複製LoL內容。

Riot官方參照（2026-10-07查詢）：[遊戲基礎](https://www.leagueoflegends.com/en-us/how-to-play/)說明英雄能力、法術與裝備的對局分類；[Hotkeys FAQ](https://support.riotgames.com/en-us/league-of-legends/performance/hotkeys-keybindings-faq)確認快捷鍵可設定；[WASD開發說明](https://www.leagueoflegends.com/en-gb/news/dev/dev-wasd-controls-are-on-the-way/)保留點擊控制並介紹替代控制／動態camera。以下是本專案預設決策，不声稱所有鍵位都是LoL當前版本固定預設。

2026-10-08重新核對官方文件補齊細節：[Hotkeys FAQ](https://support.riotgames.com/en-us/league-of-legends/performance/hotkeys-keybindings-faq)列F9切換edge-scroll、F12截圖、Enter開聊天、滑鼠1–5改鍵；[Smart Ping](https://support.riotgames.com/en-us/league-of-legends/gameplay/smart-ping/)列G一般／V警戒，Alt或Ctrl＋左鍵拖曳選擇並鬆開送出／右鍵取消，世界與小地圖共用，八種語意及objective投票；[WASD FAQ](https://support.riotgames.com/en-us/league-of-legends/gameplay/league-of-legends-keyboard-wasd-input-faq)列獨立英雄／全域profile、動態鏡頭、Scout Ahead、45度方向與對局輸入切換鎖定。這些亦納入下列批次，不把功能名稱或未綁定列當實作完成。

## 真正已找到的缺口

### 當前實作進度（2026-10-08）

最新證據補充：召喚師持續呈現 85 項測試、ABI18／IPC6 的 133 項測試及各三項編譯檢查通過；可信配裝交接 v4 通過 28 項 Rust 與 51 項 Lua 檢查，使用實際 debug／release 設定工具驗證兩次 launch.prepare。UE 兩槽 payload／專屬提交接口及 31 筆圖示 registry 的正式 refresh 與普通 NoEngineChanges gate 成功，D3D12 scoped42 Success／0Fail／0Warning／0NotRun，三張 production PNG 已開圖。共用施法／D/F 模式與響應式 HUD 候選經獨立審查、Primary 修正後已 promotion，尚待下一批建置驗證；Grok 4.7 Fast run-muytxfno-wz1dww 接 D/F 實體按鍵、設定與卡片。新權威對局0，父項仍6／44。下方較早增量段落保留其歷史批次，不代表目前 ABI 或新 source 已驗收。

最新接續：召喚師 wire／authority drain／owner-team bootstrap與持續fixed-step projection 已經獨立v4驗證通過：core40、Stop10、backend3、client1及兩個all-targets check。v1-v3失敗證據與修正記錄E411／E414／E415均保留。D/F跨ABI與HUD尚未完成，3.8仍不勾選。Tab hold／toggle、F12真正viewport PNG與Ctrl+F真FPS／RTT source已完成，普通local NoEngineChanges v2無action cache建置進行中，未以舊111+69報告代替本批驗收。整套仍6/44完成，新Authority game為0，既有10/10不重設。

已完成並驗證的增量：小地圖左鍵／拖曳、滾輪與Space／Y／F1／方向鍵原生接線、每玩家版本化profile與原子存檔、可改鍵設定介面與鏡頭偏好。最新偏好fixture exact12Success、設定原生Slate圖1400×1578；上次完整回歸83native＋69Blueprint通過，但新增中鍵／F2–F5／S的新source仍待重新編譯，不用舊報告代替本批验收。

正在整合：Grok4.7 Fast修正中鍵投影拖曳與同隊視角，Primary補exact local Slate pointer／focus、modal source token、中央HeroHUD公開資料接線及失效清空；另由Grok4.7 Fast做Stop權威語意、command protocol與ABI一致更新，Primary接通用S/H可改鍵輸入層。全部仍依原44項逐項驗收，不宣稱完整LoL操作已完成。

2026-10-08 接續：Stop 六個 Rust case 已全部通過（33 次執行，含重複 filter／lib/bin），ABI17 三個精確 gate 通過。當前 UE source 正在正式 refresh 建置；新 camera／S／H 與改鍵 mouse preview 須用新 DLL 驗收。Grok 施法候選已審查及修正 fixture，live QWER 接線 packet 已備妥，仍不勾選技能操作完成。

最新：Grok4.7 Fast `run-muyazqnr-n2vq4d` 完成正式 QWER owner／typed bridge／settings，同 thread `run-muycey4r-72mrsm` 修復兩個編譯錯誤及空 controller gate。Normal NoEngineChanges 22.83s 成功。第一批 scoped native 中13個 pure cast sequences Success，但 camera fixture 的 abstract UObject／viewport Outer ensure 使 process exit3；已按真實 UE ownership 修 fixture，保留失敗證據，需新 run-id 重跑，尚不算整批驗收。

五位實際英雄的原創頭像及八個前排／遊俠技能圖已 promotion。新增版本化 HUD artwork registry，29個內容 ID 對應五張頭像與24個技能／相容 alias；texture由 widget UObject保存且每檔只載一次，未知 ID 清舊圖，不借其他角色圖。最新 source 已接 native controller 的真正 HeroHud.ContentId，並公開 Blueprint 可呼叫的身份入口；此接口不能替代既有 Blueprint actual viewport 驗收。HUD圖片、registry與camera fixture正在正式重編，需實際渲染及完整 native／69 Blueprint回歸。

| 使用者問題 | Source證據 | 處理 |
|---|---|---|
| 小地圖左鍵沒移視野 | SOmMinimap.cpp13–22只在RightMouseButton呼叫MoveRequested；left直接Handled | 增加local camera focus／drag delegate；不送hero移動 |
| wheel不縮放 | CameraPawn.cpp71用named axis；DefaultInput.ini只有AxisConfig、無AxisMappings，預設EnhancedInput | 完整實際input binding與local viewport／pawn ownership驗證 |
| 即使zoom接好仍太慢 | WorldBridge首hero fit約90000cm，固定wheel step450cm | 有界且依目前尺度有感的zoom／pan，保持正常camera pitch与range |
| A不是A點地板 | PlayerController.cpp46直接AttackMoveNow | 分離AttackMoveAim與AttackMoveClick、左鍵confirm與cancel |
| helper成功但實際操作失效 | 相機測試直接AddZoomInput；minimap舊測試要求left只被消費 | 加入Slate／viewport真事件与actualcamera／ACK證據 |
| 控制器／HUD接線不確定 | BP_PlayerController驗收只證明APlayerController父類；native快捷鍵在AOmPlayerController | local-player adapter兼容兩者，不強改既有uasset父類 |

Grok read-only audit `run-muxzys74-heed23`／thread8826a5c0-2d9a-4a23-9d2d-93dc09c0cd21，166s／13calls／USD0.43520544；Primary獨立核對前四項source。實際possession、Blueprint mouse接線與選用mesh仍待live evidence，不亂標已修復。此表是2026-10-07實作前基線。

## 完整action矩陣

附加快捷鍵已細化為[實際事件驗收矩陣](2026-10-08-moba-control-event-matrix.md)，逐列列出 A 點地板、精確點擊、英雄限定、修飾鍵與其餘對局操作的事件及可觀察結果。矩陣列為契約，未跑的不算通過。

2026-10-08 最新增量驗證：`controls-recall-pointer-native-v12` 實際D3D12 scoped34 Success／0Fail／0Warning／0NotRun；中鍵使用真local viewport、painted hit-test geometry與Deproject ray，驗實際camera位移／鬆開／失焦／UI遮擋。Recall／六物品快捷鍵、HUD hover guard、QWER狀態機與原創HUD registry／saved profile hotkey也在同批通過。兩張production widget PNG已開圖檢視，無server／無match fixture不當作正式地圖。完整105 native及69既有Blueprint正在驗收。DF隔離模型9unit通過，正式data/runtime foundation由Grok4.7Fast整合中，world／transport／FFI／HUD仍pending；plain/BP共享HUD缺口已完成source audit，下一批遷移，不把native-only畫面當所有入口已完成。

完整回歸已terminal PASS：`controls-cast-recall-hud-native-v13-full` 為105native（97Success＋8warning-success，14warnings，failed0/notRun0）及69Blueprint（errors0/warnings0/failedLoads0），兩程序exit0。一般NoEngineChanges與當批stageSHA通過；原105native固定NullRHI因painted input contract失敗，正式入口原生預設D3D12，Blueprint仍NullRHI。2.5鏡頭快捷鍵增量已驗收，7.2完整整套功能final gate仍待後續未完成actions整合，D/F的新source未編譯不借此通過背書。

召喚師 foundation 已完成獨立驗證：Grok `run-muyfk2e6-37757p`，20m26s／28model calls／USD1.5011136；Lua-only catalog、同一 CONTENT_CATALOG_DATA_HASH、不可變 cast plan／effect-first cooldown commit及落點規則經Primary審查。內容2、生成資料1、核心12項 unit 全過，新content test warning已修正重跑；後續真正ECS adapter與trusted roster loadouts也完成20core/world＋10Stop實際測試，並納入正式UE bridge／base stage。ABI17／presentation5／command2維持，D/F命令、network projection、FFI／HUD仍pending，3.8保持未完成。F9已在controls-edge-scroll-native-v1實際34項D3D12驗收通過，含保存、改鍵、modal、Shift負例及保持Space原release。

| 類別 | 預設操作方案 | 必須出現的行為 |
|---|---|---|
| 世界move／attack | 右鍵地板／合法敵人 | MoveTo／AttackTarget各一次，合法ACK、click cue |
| 攻擊移動 | A→左鍵；Shift+A立即action，可改鍵 | range／aim cursor、confirm一次AttackMove；Esc／右鍵取消 |
| 停止／保持 | S／H | Stop與Hold權威語意不同；取消適當命令／自動攻擊行為 |
| 目標限制 | `~`位置鍵預設只攻擊英雄，可設hold／toggle | 不攻擊minion／tower；不得讀hidden enemy target |
| 普攻設定 | attack-move-on-cursor、auto attack、range display | 依profile選合法目標、range提示与停用自動攻擊 |
| QWER | normal／quick／indicator可逐slot選 | aim shape、release／confirm、拒絕原因与cooldown |
| 修飾鍵 | Ctrl+QWER升級；Alt+QWER自我施法 | 點數／合法target／rank權威驗證，不double-submit |
| 強制施法模式 | Shift+QWER normal／indicator override可設定；Alt+item self-use | 與queue modifier分開，對point／target／self分類一致处理 |
| 召喚師／回城 | D、F、B | 獨立DF slot cooldown；回城progress與被打斷回饋 |
| 道具／飾品 | 新profile item 1/2/3/5/6/7、ward4；舊1–6保留設定並提示migration | 六item slots與ward各自單一action；point／target／self use，charges／視野／cooldown權威化 |
| 鏡頭移動 | edge、方向鍵、middle drag、F9 edge切換 | map clamp、UI focus guard、scale有感、切換狀態可見且可存 |
| 鏡頭focus | Space按住、Y鎖定、F1自身／F2–F5隊友 | 明確restore／lock與可見team資料，不改hero位置 |
| 鏡頭zoom | wheel上拉近、下拉遠 | 真effective camera變動，有界平滑；shop scroll不zoom |
| 小地圖 | 左鍵／drag視野，右鍵move，Shift右鍵queue | focus與command分開、camera框更新、DPI命中一致 |
| 商店 | P為主，相容既有shop鍵；Esc | search／category／recipe／purchase／sell／合法undo与receipt |
| 資訊 | Tab按住或toggle、C詳細stats | scoreboard／stat／target資訊、只有允許資料 |
| 溝通 | G一般／V警戒、Alt或Ctrl＋左鍵ping輪盤、Enter team、Shift+Enter all | 八種語意、拖曳鬆開送出／右鍵取消、world與minimap共用、objective投票；team隔離／mute／limit，chat焦點吞gameplay |
| 額外可設定action | retreat ping、attack-only-click、move-only-click、click hero／team portrait focus | 預設可未綁定，但設定中有可執行action，不用假快捷鍵占位 |
| 表情／訊息 | T原創emote wheel、Ctrl+1–5原創角色互動、Z聊天歷史 | 有權威／team合法傳送與本地回饋，缺資產不假裝已有動作 |
| 診斷／擷取 | Ctrl+F FPS／RTT，F12截圖 | 實際統計與durable viewport PNG，關閉debug overlay可回正常HUD |
| 視覺回饋 | hover、選取、傷害／治療、死亡／重生 | tooltip／輪廓／range／VFX／音效與實際狀態一致 |
| 設定 | Esc選單、改鍵、HUD／minimap大小、色盲、音量、HUD／血條／名稱顯示切換 | 可持久化／衝突提示／restore、focus loss回收；顯示切換不揭露hidden資訊 |
| 替代控制 | 可選WASD，獨立英雄／全域profile | 復用權威command、明確與QWER／camera键衝突；動態鏡頭／Scout Ahead／45度方向，match rules決定輸入切換鎖定與重連持續 |

Ward鍵與item4不能都採4：正式新profile預設物品採1/2/3/5/6/7、飾品4；保留舊1–6 profile並在migration顯示差異，不默默改玩家設定。Shift queue為既有專案擴充，不假稱LoL預設。

### 附加快捷鍵的完整語意

右鍵取消瞄準與一般右鍵操作需分開仲裁：取消不提交原先瞄準的 AttackMove／Cast；若該事件仍是合法世界右鍵，再由普通路徑送一次移動／指定攻擊。Esc 只取消；HUD／modal／無效 pointer 不轉送世界命令。延遲原鍵 release 不得重新提交舊技能。

每個action都要出現在可改鍵設定，可設主／次鍵與精確Ctrl／Alt／Shift組合；實際尚未接通的action不以可用按鈕占位。預設未綁定的附加action也要有可執行入口、說明、衝突檢查與保存。每個英雄可覆寫全域profile，還原該英雄或全部設定分開處理。鍵盤／滑鼠鍵使用相同action仲裁，失焦、重連、換owner後清除按住與瞄準狀態。

| action群 | 實作細節與互不替代的行為 |
|---|---|
| A點地板 | A顯示射程與待確認游標，世界左鍵只送一次AttackMove；點HUD不確認；Esc／右鍵取消 |
| 立即攻擊移動 | 獨立AttackMoveClick，預設Shift+A；不用第二次左鍵；不偷偷變成Shift queue |
| 精確點擊 | MoveOnlyClick只移動；AttackOnlyClick只有游標合法目標才攻擊，點空地不走；均可另綁鍵 |
| 只選英雄 | hold／toggle選項，約束直接目標攻擊與指定單位技能；AttackMove的自動尋敵是獨立設定，不合併成同一功能 |
| 攻擊移動選敵 | 游標附近／英雄附近的選敵基準可保存；只能用當前允許公開且可攻擊的目標 |
| 技能修飾鍵 | Ability／D-F／Item／Trinket每個可用slot的普通／快速／帶指示器及可改鍵override；合法自身形狀另有自用action，Ctrl升級僅Ability；正常／鬆開施法與取消逐一驗證 |
| 道具／DF／ward | 自身／地點／單位三類目標；道具含快速／普通／自用與六格改鍵；DF獨立兩格；ward獨立charges與冷卻，不借技能index |
| 介面快捷鍵 | scoreboard hold／toggle、詳細屬性、目標資訊、商店、聊天歷史、FPS／RTT、截圖、HUD顯示與隊友portrait互動 |
| 溝通附加鍵 | 一般／警戒／撤退、語意輪盤、HUD狀態Alt點擊、team／all chat、快捷訊息、mute／音效設定、原創表情與角色互動 |
| 鏡頭附加鍵 | 自身／各隊友focus、hold center／lock toggle、上下左右／中鍵drag／edge toggle、zoom、minimap focus／drag、可選WASD獨立profile |

小地圖與隊友頭像也要有明確仲裁：增加可保存的小地圖左右位置、允許小地圖移動與小地圖施法偏好。沒有瞄準時左鍵查看／拖曳；允許施法時只把合法小地圖座標映射到共用地點目標，照常驗證射程、terrain、slot 與公開資料，不能把 UI 像素當世界座標。預設維持既有左鍵查看／右鍵移動及施法不確認，開啟偏好才允許新路徑；ping、modal、取消和拖曳捕捉優先序必須可重現。隊友頭像除鏡頭 focus 外，應能為內容允許的友方指定單位技能提供實際公開目標；死亡／hidden／source 退役不得沿用舊 identity。這些是本專案完整對局控制的待實作契約，不宣稱都是 LoL 固定預設。

LoL操作參考採[Riot快捷鍵說明](https://support.riotgames.com/en-us/league-of-legends/performance/hotkeys-keybindings-faq)與[Riot 2026 WASD／英雄個別設定說明](https://www.leagueoflegends.com/en-us/news/dev/dev-wasds-ranked-release/)。上表是本專案明確設計contract，預設Shift+A是我們的配置，不把不同action互相混用或聲稱所有鍵位等同LoL。

2026-10-08再次核對後，替代控制亦包含官方提到的無障礙游標方向／速度改鍵、搖桿映射WASD，以及Select Object／Interact／Auto Attack／Cast Ability拆分輸入。必要移動鍵未綁必須顯示具體錯誤並保留原合法profile；允許共用鍵的語意由當前上下文仲裁，不能重複送指令。這些仍為待實作契約，併入3.10／1.4及事件矩陣，不當作已可用功能。

### 2026-10-08 目前實際驗證狀態

召喚師wire／fixed-step projection v4：core40、Stop10、backend3、client1及兩項all-targets check全過；D/F的presentation／ABI／UE仍待整合。Tab／F12／CtrlF build-v5普通NoEngineChanges33.46s通過，native-v3 exact41Success／0Fail／0Warning／0NotRun，實際F12含shop/settings成功保存，三張production圖及settings真F12均已開圖。保留v1/v2失敗，詳E417/E418；C詳細stats仍缺，4.5父項不勾選，不使用舊full-v7替本增量背書。Grok4.7Fast run-muypxsdl-c6gaa0接持續D/F資料呈現批次。維持6／44父項完成；每批只驗當前實際功能，所有項目整合後再執行完整最終驗收，authority10／10不重設。

## 實作批次與完成條件

1. **P0鏡頭／小地圖／input owner**：left／drag／zoom／Space／Y、camera框、world與HUD focus。完成條件是actualviewport操作有效，單次wheel有可見縮放，拖曳不被follow拉回。
2. **P1完整戰鬥輸入**：A點地板、右鍵move／attack、S/H、target-only、cast modes／修飾鍵、道具與DF／ward的缺少權威能力。每個動作要有原command ID／ACK、合法target、取消／失焦負測試。
3. **P2 HUD／商店／資訊／溝通**：重新設計版面与icons／hotkey／tooltip／cooldown，scoreboard、stats、ping、chat与設定；空白UI不遮世界。
4. **P3正式畫面vertical slice**：完成一張三路地圖，辨識度清楚的hero／creep／tower／base、材質／lighting／shadow、animation／VFX／audio。固定視點before-after證據，保留fog安全与效能限制。
5. **P4完整操作驗收**：native＋既有Blueprint、1080p／1440p与100%／150%DPI、windowed／fullscreen／PIE、focus loss／disconnect／death／respawn／restart；正式build／stageSHA／exact71+增量native／69Blueprint。真整場／LAN／fixed12缺證仍pending，舊10/10不重設。

順序先解玩家不能操作，再改善外觀；每批有source、回歸、實際圖像與限制。不得把新增函式或合成marker當實作成功，也不為讓20min內終局改勝負規則。

焦點細節也分action：聊天／modal吞鍵，但hover一般HUD時S/H/B、自我施法、升級仍應可用；不能用commandbar大矩形封鎖全部快捷鍵。需要world target的move／attack／cast才受pointer命中與target合法性限制。

完整可執行規格位於 `openspec/changes/complete-moba-controls-and-presentation/`：proposal／design／五個capability specs／tasks。此文件與tasks共同維護，完成一項才勾選，原計畫27/31保持獨立。

### 2026-10-08 F9與召喚師權威元件增量

F9正式refresh與普通NoEngineChanges／stage實測PASS。controls-edge-scroll-native-v1 D3D12為34 Success／0Fail／0Warning／0NotRun，已驗真InputStack開關、保存、改F10、舊F9失效、Shift／modal負例，以及Space按住跨F9保持與原release。已開實際HUD與設定PNG；常駐商店列表／整體版面仍需P2重設。2.4還包含完整UI wheel與pointer矩陣，維持未勾選，不以F9單項推定全部成功。

D/F增加production native/summoner_world.rs，正式match配置使用明確summoner_loadouts；未配置維持不可用，不借QWER或hardcode配備。setup在spawn前一次驗完未知player、未知catalog與重複spell；狀態在MobaHeroSlot跨entity generation保留。實際Pos/CProperty與cooldown在同一ECS借用交易commit；失敗不改HP／位置／cooldown／command，成功blink復用公開terrain/body collision、既有attack取消與Recall中斷，self heal不清movement。Playing部分delta且非pause才扣兩slot冷卻。summoner-world-validation-v2實際20測試（core12＋world8）與Stop回歸10測試全過；僅三個既有template build warnings，新增模組無warning。測試含碰牆半徑／越牆、root／silence、死亡／phase／pause／ownership、實際delete+maintain+entity replacement後不可重用冷卻。此為元件驗證，未啟動server或新authority game；wire協議、server admission、投影、ABI／UE及positive ACK仍未完成，3.8維持未勾選。

### 2026-10-08 共用HUD／鏡頭框／商店驗收與後續批次

Grok4.7Fast共用HUD修正run-muyk2igf-3imudy（918s／84model calls／USD4.2460288）經Primary獨立審查、編譯與真D3D12回歸。修正widget實際GameInstance outer／player context、AddToViewport持有、controller替換與stale source epoch、selection focus和真viewport fixture。Primary另外補鏡頭投影輪廓、商店P／改鍵／Esc焦點還原、scroll不zoom、搜尋與真正主動／被動型別分類。scoped-v5 exact41Success／0warning，三張2800×1578 HUD／shop與1400×1578 settings PNG全部開圖；它們是無server／無match的production widgets，不當作正式三路地圖畫面。

普通build-v12為24.38s／exit0；完整controls-common-hud-shop-native-v7-full有111native（103Success＋8warning-success／14既有warnings／failed0／notRun0）及69Blueprint（errors0／warnings0／failedLoads0），两程序exit0。此前native-v1／v3 crash、v2的33／8、v4的40／1與full-v6的110／1全部保留；錯誤E401–E409記錄原因與修法。2.6鏡頭框增量已勾選，全plan目前6／44；同步fixture的公開spring-arm tick不能代替2.7／7.3跨Engine frame／PIE與完整UI矩陣。

下一批専用D/F wire／deterministic phase／合法owner-team投影交由Grok4.7Fast run-muyl8o4n-h4hzd3／thread60f93f74-ba5b-4e3a-ba9a-433e7f185ce9處理，仍未建置／驗收／接ABI或UE。Primary同步只修改Unreal的Tab戰績按住／切換、atomic profile保存／設定、改鍵與release／focus／handoff／retire；已新增真InputStack測試source，尚未編譯，不使用full-v7舊報告背書。C詳細stats、Ctrl+F、F12等4.5其他項目仍待後续。維持既有authority10／10，不新開整場或重設計數。

## Summoner persistent presentation accepted subset, 2026-10-08

Grok4.7Fast run-muyq8jrn-qjgwhl (636s/67calls/USD3.34591416), same-thread compile repair run-muyqqf26-5et83b (92s/12calls/USD0.76158504); Primary reviewed actual extractor, capability gating, retained dead-owner state, manifest admission and committed batch transactions. v1 failed compiler guard clone, preserved. v2 eight checks passed; Primary then shared canonical validation across final committed batches and tested clear/equip in both fact orders. Fresh v3 core42/client-summoner3/HUD11/presentation28/bridge1 =85 tests, plus bridge/backend/client all-targets checks passed. Formal Lua prost fallback regeneration passed; diff contains the actual new proto messages/tags/comments only. Root/omfue HEAD unchanged; zero authority games, quota10/10. Only persistent Rust/protobuf subset accepted. ABI17/presentation5 and Unreal D/F still pending; full44task plan remains6checked, final full native/BP deferred until all increments.

### Summoner ABI18／IPC6 accepted increment

Grok4.7Fast run-muyrgsv2-03jzex／thread ed0e830f-f005-42e3-8bc3-6da86625c830:505s,49calls,USD1.96489672. Primary reviewed dedicated slot17 command, C/Rust36-byte row layout, retained frame strings/dead-owner equipment, capability/shape/owner/cooldown rejection before input id. v1 preserved actual old-version assertion failure E424; Primary corrected literal6. Fresh v2:89bridge＋16input＋28presentation=133tests and3all-targets checks passed, zeroauthoritygames. C++header smoke updated but not yet compiled; Unreal payload／D/F owner/cards／trusted selection still pending, parent6/44 unchanged.

### 2026-10-08 攻擊選敵與DF增量

共用純策略19測試及core all-targets通過：英雄／游標選敵、合法性、世代／穩定排序、精確raw距離、禁止先截十個目標、auto-attack及只選英雄的獨立語意。實際native消費端及偏好transport尚未完成。DF owner/HUD卡片本輪49 native全過、八張PNG已開圖；仍缺連線權威positive ACK及最後整套驗收，不勾完成3.8。

### 2026-10-08 共用選敵消費端驗證

Grok4.7 Fast接入hero_command_tick停步／追擊及hero_tick出手，共用trusted HP／faction／世代／已提交視野判定、局部hero／creep／tower查詢與raw距離。consumer-validation-v4共61專項測試及core all-targets通過。已排除先取十個導致漏敵、strict float邊界漏選與非法明確目標在selector內轉打其他敵人。偏好目前只供注入測試，權威owner command／持久化／UI啟用尚待後續，UE staged bridge亦待更新；3.3／3.5仍不勾完成。

共用選敵後續E458：修正游標攻擊移動與閒置自動普攻的獨立性，新增純策略與實際hero_tick回歸；consumer-validation-v5最新63測試及all-targets通過。v4的61測試屬前一source cutoff。


商店交易狀態當前範圍接受（2026-10-11）：.ai-collab/shop-transaction-accepted-v1/summary.json。ABI25雙ID關聯、有界pending與最新提交狀態、舊pending保留／歷史隔離、可見前景及400x300放大文字已驗。ordinary no-cache shop-transaction-build-v9成功20.27秒；fresh shop-transaction-native-v5八native全Success，保留FrameContract一筆刻意負例warning。四張當次狀態original及九張owned原圖已檢視；fresh shop-transaction-viewport-v3真正314cases／128intents／123stages、304／125／115 ordered prefix及cleanup通過。Rust259／六checks／九capture-dependent ignored及codegen19／header／MSVC layout明示沿用，以826來源不變與UE-only delta綁定；metadata四SHA／art24SHA／trinket package與stage副本再驗。E730–E750原失敗保留。父項8／44與4.4完整結算不勾完，無新權威對局／ACK／完整世界驗收；立即接Grok4.7非fast隊伍溝通基礎，完整驗收最後。


隊伍溝通共享基礎範圍接受（2026-10-11）：.ai-collab/team-communication-foundation-accepted-v1/summary.json。fresh13不同Rust測試／四個maintained all-target checks／零ignored及重複，827來源驗證後不變。實際MobaMatch名冊／TeamIdentityState揭露、Unicode與時鐘限流／有界history與vote、真World Hero/Inventory/owner/generation facts、room/run mute與獨立容量已驗。E751首版12成功但producer缺口與修復證據保留；實際warning另記audit/review，不宣稱零warning。未配置tag/schema、未接authenticated transport或UE；ABI25／metadata／原美術／staged副本明示保留並再驗，未新跑UE。無authority game／ACK，5.1與父項8/44不勾完；立即接Grok4.7非fast傳輸批次，完整驗收最後。


2026-10-11 Codex直接接手（不再派發子代理）：authenticated Rust communication transport範圍接受。team-transport-direct-accepted-v1/summary.json綁定42不同測試、四個maintained all-target checks、零ignored/重複；E757 fixture與E758 backend Fetch型別失敗及原log保留，後端修復後fresh check通過。ABI25與codegen-v3為既有紀錄，未新啟用UE/未跑authority game/無ACK。父項8/44保持，下一步直接實作canonical原創互動與獨立HUD status，再接public Unreal input/received widgets。


2026-10-11 Codex直接實作接受：communication-content-status-accepted-v1/summary.json。fresh49不同Rust測試／四個maintained all-target checks／零ignored或重複；官方fallback／19generate-check／20Lua inputs通過。固定ID五種原創角色互動與八種emote、單一ledger／nonce／mute／clock生命期、真World能力與道具／public KDA獨立status已驗。protocol/schema2、ABI25，無optional media匯入、無UE新啟用／authority game／ACK／完整驗收。E759原失敗保留，修復後unsupported測試明確拒絕舊版1與未支援版。5.1文字所列ping語意／transport／合法target座標／限流靜音與隔離契約已由已接受Rust證據滿足，故僅勾5.1（9/44）；5.2–5.4真UE入口與畫面仍pending。立即直接接public Unreal communication API，無子代理。
