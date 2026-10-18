# MOBA 結算後下一局：受管理的對局生命期

日期：2026-10-18。依可玩 MVP 計畫繼續，Codex 自行實作。本文是下一批設計與工作清單，未勾項目不代表已接線。

## 現況與問題

正式 role launcher 每次產生新設定、擁有 authority／各玩家 Rust runtime／Unreal `-game` 程序，結束時按原始 process creation identity 清理。選角收據與輸出禁止覆寫，HUD／input owner 退休後不可復活。已完成公開勝負面板與一次性離場，並有 native 真 Slate 點擊證據；`run_ue.bat --moba` 已轉入正式選角入口。目前沒有遊戲內下一局操作，不能把重播舊 finalized receipt、重新 StartRuntime 或直接重跑舊命令當成下一局。

## 架構與邊界

使用現有「每場獨立程序」架構，增加跨場的 Lua session coordinator。它持有原始作者配方／使用者選項，監督每一場 role workflow；新的場次由既有正式工廠重新選角、準備與啟動。每場 UE process／world／controller／GameInstance 與 replica 均為新物件，故不復活退休元件，也不必在遊戲中修改 immutable command line。

這不是遊戲狀態的重置按鈕。必須具有明確使用者準備操作、所有真人的準備狀態、舊場完整清理確認，以及新輸出／新選角／新啟動。錯誤、關窗、崩潰與逾時不能自動開新場。

協調能力採 provider 契約。第一個 provider 使用正式 host plan 的所有真人都受本機管理的模式，依 plan 的玩家清單支援 1..10 人，禁止寫死 player1、兩位玩家或某個英雄。dedicated、remote-client、部分本機席位的 LAN plan、prepare-only 與 bounded result automation 保留既有語義，不宣告未提供的下一局能力。後續遠端 room provider 可沿同一 UI／session 契約實作，不把本機私有檔案當成 LAN 協議。

## 使用者流程

1. 正式入口進入真正選角、對局。只有已安裝有效協調能力的當前場次才提供「再玩一局」。
2. 自己的當前 controller／註冊 LocalPlayer／HUD bar／player／source epoch 及公開 Finished 成立後，點按寫出一次準備意圖；不發 gameplay command 或聲稱權威 ACK。
3. 結算面板顯示「等待其他玩家…」，保留離場。準備不能重複提交；玩家選擇離場會先寫取消意圖。
4. 所有真人準備且沒有取消，coordinator 才發出本場離場通知。UE 沿已驗證的 source／input／HUD 清理及正式 QuitGame 路徑退出；native 測試只注入最後 OS quit 邊界。
5. coordinator 等待原 role workflow 成功結束、每個原 owned process 確認退休，再次檢查取消；之後才分配新目錄、從原作者配方開新選角。
6. 任何成員離場／失聯、清理失敗或取消競態使下一局取消；不能偷偷移除真人席位或強行重開。沒有能力的入口仍可正常離場，再由正式入口另開新局。

## 私有控制契約

- 每個 session／generation 有獨立且排他建立的目錄，包含版本化 manifest；每位受管理玩家只有自己的一份能力描述、一次性 ready／cancel 與 resolution 路徑。
- 描述綁定 session identity、generation、player、當前 catalog identity 與原始 launcher ownership；不將檔案內容提供的 PID 或執行路徑當成可停止／可啟動目標。
- UE 只在已解析的能力範圍讀寫有界 JSON；要求 absolute、正規化且位於本代私有目錄。未知版本／字段、越界值、錯代／錯玩家／重複／既有輸出均拒絕，不猜測修復。
- ready 與 cancel 以不覆寫方式建立；取消在關場前及新場分配前均有優先權，避免「準備後立刻離開」仍開下一場的競態。coordinator 是 resolution 的唯一寫入者。
- 無能力時不做 frame polling。準備後只由 GameInstance 的有界低頻 ticker 讀本場 resolution，結束／退休後移除；不新增每 entity IO、controller 搜尋或反射函式呼叫。
- 畫面只顯示簡短中文狀態；內部路徑、nonce、command、IPC／ABI／authority 詞不進產品文字。錯誤記到受管理 run 的 MD 與專案錯誤 register，不記私密能力內容。

## 實作順序

- [x] A. 純 Lua 協調狀態機與嚴格 manifest／request 契約：準備、取消、全員準備、舊場退出、清理確認、新代分配，禁止錯代與重播。managed-session-policy-v1 真246斷言與兩來源SHA前後一致；涵蓋1..10真人、取消競態、崩潰／清理失敗及Windows u32退出碼。尚無file adapter或GUI，不是下一局整體完成。
- [x] B. 正式 role workflow capability 接線與 coordinator：沿現有 Windows Job 及 owned-record API，保存原作者參數、每代新輸出、全部清理後才呼叫新場工廠；help／prepare／remote 語義保持。
- [x] C. GameInstance subsystem 能力解析、一次性準備／取消、resolution ticker；重用已存在的 controller＋player＋source epoch 離場資格，GUI callback 不能自行分配下一場。
- [x] D. 結算面板真正的「再玩一局」／等待／失敗狀態與離場取消，透明區域及 repeat／foreign／retired bar 拒絕。
- [x] E. root MOBA 入口、操作手冊與明確能力說明；不新增 root bat、不重新開歷史 omfx、不手改 engine/cache。
- [x] F. 當前功能驗證：Lua 模型／owned neutral child 的生命期、native 真 Slate 準備與退出、錯代／取消競態／崩潰／清理失敗、兩位以上人數決定性與重複；各批 source SHA／ordinary build／必要操作回歸及原圖。範圍明確限下列證據，不代表新權威對局。

## 驗證限制

本批範圍接受：.ai-collab/playable-mvp-managed-next-accepted-v1/summary.json。ordinary managed-next-build-v4成功（NoEngineChanges／local／nocache，UBT11.66秒），native-v2明確5項Success／0warning／exit0，包括真Slate下一局點擊、既有離場、HUD epoch隔離、responsive及新增取消生命期。91來源SHA從build到final一致。純模型347、真private Job控制檔88、跨代不可變快照50、正式role owner中性程序45斷言／28個原始lifetime退出、既有role10／entry39與根bat help通過。原圖v1四種1280／400與text1／2已逐張直接檢視，v2一般版與400 text2另直接檢視；按鈕垂直全寬，不縮字。v1/v2 build失敗與E1150–E1155保留。

補充互通證據：managed-native-json-v1實際讀取兩組由UE JSON writer產生的ready／retired檔，production Lua policy嚴格解析與同scope／epoch確認共15斷言成功；沒有注入process exit，因此兩組都不能retire_round或commit新代。這只驗跨語言檔案格式與防止把synthetic quit當真退出，不冒稱額外權威或完整重開。

正式root已走run_moba_session_ue，coordinator同process重用原role；每代新bootstrap與新選角／match目錄，原作者JSON及base TOML以SHA驗證，拒絕變更／外來session／roster／generation。UE能力由有界UTF-8 hex locator讀取，只接受實際同名private Job會員及kill-on-close，固定兄弟檔案create-new；ready後4Hz ticker，abort優先。只有協調離場且三項實際退休完成才發retired通知；GUI不能配置新程序，只有coordinator能在原Child真退出碼與全清場後分配新代。

測試邊界：正式role測試使用真native Child lease和中性Lua參與者，其退休內容是synthetic；coordinator測試注入role工廠；native用synthetic公開Finished與coordinator回覆，只有最末OS QuitGame替換成callback。沒有新authority場次、真正跨局authority ACK、LAN room或MVP完整驗收；這些仍留主計畫P3及父項pending，不因本批六項勾選而聲稱已取得。

平台調查補充：目前process.wait_owned只能證明原始lifetime已退出，不提供保留handle的退出碼；不能把PID消失當作exit0。因此B先補同步的native owned run-to-files能力，在同一Child handle取得birth identity、通知啟動並等待真退出碼；Lua waiter只負責流程與寫scope收據。role/coordinator同時監督waiter與實際子程序，只有相符identity的退出收據及完整清理可進下一代；waiter／Lua owner崩潰使整個受管理workflow失敗並由既有private Windows Job退休後代。沿用現有spawn行為，不引入PowerShell、常駐服務或脫離job的背景程序。

B1 實作方式：native同步run把stdout／stderr直接寫本場新檔，在持有的Child handle上取得原始birth identity，create-new發出ready通知，wait後回傳完整u32退出碼。Lua waiter在private Windows Job內阻塞等結果並寫退出收據；父流程同時監督waiter與實際child，startup未完成、waiter失聯、收據缺失／錯identity均失敗，不補造exit0。既有spawn／wait不改語義。以固定Lua中性child的exit0／3／0xC0000005、ready時仍活著、重複檔不覆寫與錯birth收據驗證，不啟動UE或權威對局。

B1平台範圍已驗（owned-run-platform-v3）：原生25tests／production工具build成功，真中性程序50斷言通過，三個完整退出碼與原始lifetime清理、ready仍活著、嚴格birth／exe／schema、既有檔不覆寫、發布後writer退出的最後讀取競態、缺private workflow零輸出均驗明；九來源SHA前後一致。E1146–E1148原失敗保留。B的coordinator／role接線、C與D的GUI仍未完成，這不是下一局端到端驗收。

B2 接線決定：coordinator保持同一個受private Job監督的Lua owner，直接重用正式role workflow函式，每場新options／output；不額外啟動另一個role supervisor。模型的retire條件改成真workflow成功boolean、完整清理及各UE的原生exit0，避免為不存在的role子程序捏造exit0。UE用共同production Lua waiter／native Child lease；原本未受管理的role入口保持單場語義。moba-config補只讀--catalog-scope並在configuration report提供真正compiled三hash，managed路徑在選角前驗明，不從目錄名或範例猜雜湊。

B3 離場確認補強：all-ready之後，正常exit0本身仍不足以區分玩家手動關窗。只有處理本代coordinator通知、完成真input／HUD／runtime退休後，UE才create-new寫retired.json；須匹配原ready的source epoch與同session／generation／player／三hash，且三項退休旗標為true。parent同時驗明此通知與原Child真exit0，才可記錄受協調的離場；缺通知、崩潰、取消或未清場均取消下一局。這不是權威command ACK。resolution與後續abort使用不同append-only檔，避免取消競態要求覆寫已發布通知；最後分配新代前再讀取消。

目前權威對局額度 10/10 已用完，不重設、不另用新名稱繞過。可實作並用純狀態機、真受管理的中性程序、native synthetic 公開 Finished 和最後 OS quit 邊界 callback 驗證控制流程；這些不能稱為真正下一局 authority ACK、LAN room 或完整對局驗收。完整 MVP 最後驗收仍按主計畫記錄取得與未取得證據。
