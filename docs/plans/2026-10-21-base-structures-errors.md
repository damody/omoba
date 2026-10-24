# 基地攻防實作與錯誤紀錄

本輪不新增權威連線對局，不重設配額；正式流程使用固定 Lua 5.4，UE D:/UE5.8。

## B01 建築生命週期缺口
交接正式基地僅由線塔開放，沒有水晶／雙塔／超級兵。新增可選共享規則與完整退休／重生狀態，避免在傷害入口、Bot及視覺各自猜測。

## 監測
開始實作時沒有背景建置或測試等待。後續長時間程序以受擁有權驗證的 Lua watcher 每180秒記錄檔案進度，並檢查子程序的實際案例／測試輸出；無變動時先判定是否內層輸出緩衝。

## B02 查找與fixture欄位
初次查找誤把 compiled_map_fixture.rs 放在 driver/ 子目錄；由 `rg --files --no-ignore omfue/bridge/src` 查明實際位於 src/。新增fixture時先寫了不存在的 Finished.reason，檢查enum後立即改為現有tick，未送交編譯。後續須先確認現有型別及檔案位置。第一輪core僅編譯成功、篩選0測試，不能算功能驗收，已新增真實生命週期及兵線測試。

10:13 三分鐘檢查：首輪編譯已正常結束，沒有等待中的程序；正在新增死亡管線／重生／兵線回歸。

## B03 watcher 不存在檔案
舊 Saved watcher 直接呼叫 path.attributes，對尚未由 buffered cargo runner 寫出的 progress.json 拋錯。修正本輪 Saved 通用watcher先 exists 再 attributes，保留失敗輸出，新v2監測已啟動；同時監測實際release deps目錄。Rust工作本身未中斷。

## B04 Windows rg glob 與工具清單
讀取既有backend helper先猜了不存在的completeness檔名，後由rg檔案清單定位diagonal-backend-build.lua；統計日誌曾直接給rg一個含星號的Windows路徑，未展開。改用 `rg --glob '*.stdout.log' 'test result:' <directory>`。均為查找錯誤，未改動產品或接受失敗結果。

10:18:39 監測正常；10:19:31確認release246項全數通過（130.90秒），模板58／核心790通過，後端回歸開始。三路完整生命週期包含新基地鏈，最末種子20399tick成功結束。

## B05 模板測試feature與數字更正
上一行模板58數字為誤讀預期：RustV1一般模式實際為10＋23＋9＝42，未啟用runtime-lua-content，因此未執行新authoring驗證。已在對話更正並增加獨立帶feature的完整lib測試，最後以實際log計數。曾再次在查找helper時把星號交給rg路徑，後改成--glob或rg --files。

## B06 UE C4458 區域名稱遮蔽
正式UE建置v1在OmCompiledMapSceneCapture.cpp:169失敗：區域FString Id遮蔽FMap.Id，UE提升為編譯錯誤。改為StructureId，保留base-structures-build-v1.log，必須v2成功後才開始場景與完整本機接受，不接受失敗建置。新增結構格式也同步升級scene/4，避免舊fixture誤入新讀取器。

10:21:47 Rust整批完成，全部exit0；10:23確認owned140、local69及跨map ID Lua專項通過。模板Lua feature獨立lib58成功。UE v1失敗已定位並修正，沒有卡住程序。

## B07 真畫面揭露基地模型重疊
WorldV1雖27項通過，但實際game-viewport-spawn.png顯示nexus offset140、水晶260、內塔400與主堡尺寸重疊。拒絕把自動通過當視覺接受。共享規則新增minimum_spacing，正式採250，雙塔400、水晶650、線塔1500/1200/900；塔邊界改依全部實際route弧長整數保守計算，移除水平side scalar假設。舊v1來源指紋及場景保留為已取代；需新Rust模擬、建置、場景和來源cohort。另將真正生成的lane_super加入native骨架／動畫／高度／生命週期測試。

後續設計核對發現只把線塔前移會讓對向外塔過近，最終改為整體共享幾何2倍（4800邊長）、線塔2200/1600/1000，雙塔400、水晶650；戰鬥射程／速度／模型大小不縮放。三路中路總長約6788，外塔之間保留約2388空間。所有lane、terrain、視覺surface及camp中心共同縮放；member offsets與leash保留遊戲單位。正式RoleBotMatchPlan已依map生成lane_length；測試fixture補用相同來源。

## B08 通訊邊界fixture寫死舊map大小
RustV2模板58成功；核心789通過／1失敗，communication測試仍把2401當作越界。產品本身已依共享geometry正確接受新地圖，fixture改成讀實際waypoint上界並測edge+1，保留負座標與全部舊拒絕斷言。V2失敗保留，不冒稱790全過。後續Rust runner使用即時stdout/stderr及擁有權exit receipt，且整批收集所有suite結果。

## B09 技能隔離fixture進入新塔射程
RustV3實際資料與核心／模板已通過，內容模擬發現多個原本用(0,2500)隔離塔／營地的技能fixture，如今位於新地圖線塔2200附近，HP預期多差90點（真塔攻擊），不是技能數值錯誤。改為共用依全部compiled geometry上界加2000的隔離origin，施法點、移動目標與cue中心皆同源，原HP／魔力／冷卻／命中斷言保留，不關閉塔AI或放寬預期。先等待V3全部讀取完成，再修改fixture並重跑完整內容測試。

10:37:01 三分鐘檢查：即時log持續前進，三種子authored5v5已完成4500tick與雙副本；10:38前兩種子完整三路生命週期各16493tick完成，較舊擁擠配置20399tick縮短，死亡由10降為4。WorldV2的27項成功且實際基地畫面已檢視，拒絕舊WorldV1重疊畫面作最終接受。

## B10 fixture固定點型別與後端舊大小斷言
BaseV4共用origin重構首次編譯有兩個cue tuple的i64／i32型別不符；原數字literal可自行推導，改函數後須對兩處公開固定點tuple明確i64::from，wire Vec2I仍保留i32。編譯失敗log保留。RustV3其餘全部跑完，後端188通過／1失敗／1ignored，失敗同樣是fixture硬寫2400，改從compiled map讀lane_length；正式config已正確生成4800。bridge127、client147、ABI15成功；V3整體失敗不列作整批成功，最後須採通過的各suite與重跑內容／後端。

## B11 隔離origin的自訂牆也必須同源
BaseV5原七個技能HP／魔力測試已恢復，新增暴露三個dash／combat navigation fixture自訂牆仍位於y2400..2600，而演員已移至新隔離區。必須把三處牆改成由同一origin及相對半徑建立，不能刪掉碰撞或修改拒絕預期。V5失敗保留；V6先跑dash及navigation專項，再跑完整246項。後端V4已189通過／1既有ignored；UE最後v4正式build／current script DLL staging成功。

10:46穿牆4項與導航1項專項成功；10:49:04完整BaseV6的246項成功（165.51秒），無失敗。最終主要Rust suites共1572通過／10既有ignored，三種子完整三路及三種子5v5雙副本均成功。WorldV3的27項成功；最後3339來源／資產基準完成，正式backend-v2 build／staging成功。開始完整UE本機接受，來源凍結。

## B12 過早讀取尚未建立的階段歷史
owned viewport剛啟動時直接讀control-history.jsonl，檔案尚未建立；實際UE log已顯示armed-a-rmb與目前budget，程序正常。後續先檢查檔案存在，並用同源.ue.log的OM_OWNED_TIMING確認目前案例；不把缺少尚未寫出的檔案誤判為停滯。搜尋失敗訊息也改為精確Result={Fail，避免合法測試名稱含Failure被當成失敗。

10:54 native-combined確認206項：198成功＋8帶警告成功、0失敗／未跑；owned foundation開始，監測持續。

## B13 小地圖實際游標取樣與理想投影的精度差
最終local-v1的206 native成功，但foundation在minimap-normal-point失敗：正式wire點y_raw2046與理想投影點約2048不同。初判舊硬編座標，查明AimMinimap先Project／Unproject理想local pixel，再轉Slate absolute，沒有將float screen round trip納入expected；擴大地圖會放大這個誤差。改以正式CommandBar.TrySampleMinimapPoint(實際Absolute)取得PendingSamplePoint，再保留原一raw quantum wire誤差上限與射程／widget路徑檢查，增加DescribePoint細節。沒有放寬誤差、調整點或改產品施法座標。失敗local-v1與錄影保留，須重新建置並完整local-v2接受；先前source-v3不可冒充最後來源。查找中含星號路徑的rg亦改回目錄＋--glob。

## B14 非角色計畫的一般三路入口漏接authored兵線
完成client檢查時發現ServerConfig.single_lane_config的無MATCH_ROLE_PLAN_JSON分支沒有設定authored_minion_waves與地圖wave_interval，因此直接設定three_lane的玩家會沿用legacy二隻兵／12秒，水晶雖存在但不產生新超級兵。RoleBotMatchPlan分支本來已正確。等待完整local-v2與其3339來源一致檢查成功後，僅修改omb/src/config/server_config.rs，接上共用map編成與波次，新增跨全部map的人類roster／legacy保留回歸。這是正式接線修復，不用局部force flag或只改fixture。

UE本機接受的publisher來自omoba-client-runtime，不消費omobab server_config。以兩個來源cohort與精確transition證明：UI／共享3338檔未變，唯一後端檔另做完整190項與正式authority／moba-config建置。不能把最後後端修改冒稱UE cohort期間全3339未變，也不為單獨後端設定修正重跑無關UI冒充其權威ACK。查找時曾猜錯moba_owned_viewport_fixture.lua，已由require及rg清單定位moba_renderer_fixture.lua。

11:17:24完整local-v2成功：206 native（198成功＋8帶警告、12 warnings）、458cases／222intents／157stages／13cursor、69Blueprint零compiler errors/warnings／load failures、五真PCM（一警告）；11:19:38其3339來源／資產完整前後一致。之後才進行此單一後端檔修改，兩段證據分別封存。

## B15 短程序已結束後才附加watcher
backend-v5完整190項在15.52秒完成，晚附加watcher時原PID已退休，因此validate_owned正確拒絕。保留拒絕輸出；正式run_owned_to_files的exit receipt已證實exit0／retired，沒有超過三分鐘的等待或程序洩漏。後續短流程先讀完成狀態，已完成者不再附加watcher。

11:21確認後端190項成功／1既有ignored，新無角色計畫的跨map兵線回歸通過，最終主要Rust合計1573通過／10既有ignored；開始正式backend-v3建置與staging。

11:24:38最終source-v5共3339檔前後一致；唯一後端配置的source transition已核對，3338個UI／共享等檔案未變。backend-v3建置成功，最終封存13份byte-identical JSON與Rust／工作流SHA-256／size，index.success=true。完整UE local-v2的180秒監測6個樣本與owned程序退休確認。報告及交接更新；所有新範圍6/6完成，舊MVP33/44及外部缺證保留。
