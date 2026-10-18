# 可改鍵游標與搖桿操作

最新當前功能接受：`.ai-collab/playable-mvp-portrait-accepted-v1/summary.json`。RTS相機明確固定垂直戰鬥尺度，避免繼承MaintainX時極窄視窗看穿地平線；三種玩家axis／四比例／兩有效縮放共24組與兩map十二當次原圖已驗。普通建置13.38秒、153來源一致；fresh195 native全成功（12 warning保留）、full458操作／222 wire／157 stage／13筆游標觀察、69 Blueprint零compiler／load診斷且SHA／mtime不變、三PCM與全部清理成功。詳見`2026-10-18-camera-portrait-projection.md`；任務核對見`2026-10-18-mvp-functional-task-audit.md`，目前24／44，完整世界／語音與跨環境等20項仍待辦。無子代理、新權威對局或ACK。以下較早接受與進度保留歷史。

最新整合驗收已含公開地圖鏡頭／地表覆蓋：`playable-mvp-camera-coverage-accepted-v1`。fresh195 native／full458操作／222 wire／157 stage、13筆游標觀察、69 Blueprint與三PCM全部成功，153來源及69資產SHA／mtime一致。新鏡頭邊界未破壞本篇的游標／搖桿與小地圖確認；詳見`2026-10-18-camera-map-coverage.md`。以下194沿用與focused接受資料保留原始歷史。

完整本機驗收已成功：`.ai-collab/playable-mvp-local-accepted-v1/summary.json`。普通建置34.53秒、153來源一致，fresh完整458案例／222 wire／157 stage及13筆Slate游標觀察成功；194 native、69 Blueprint零compiler／load診斷與三PCM經嚴格來源／資產重驗明示沿用。商店提示scope、模式狀態合成及跨情境pending／戰鬥設定銜接都已驗正確。詳見 `2026-10-18-playable-mvp-local-acceptance.md`；以下獨立接受與中間進度僅作歷史，並非目前仍在執行。

最終本批狀態（2026-10-18）：已接受 `.ai-collab/playable-mvp-accessible-input-accepted-v1/summary.json`。ordinary no-cache build-v8 17.45秒成功；native-v3真25 Success／0 warning；owned-v3 focused accessibility 真14 cases／7 wire intents／5 stages與publisher／editor／profile／listener清理成功。13筆真Slate cursor位置觀察嚴格重驗，150個唯一來源於建置／native／owned前後一致。800×600新控制區及當次正式設定圖共兩張original已直接檢視；短標頭自然取寬，長內容仍可捲動。schema9 additive profile與既有Direction路徑，無Rust／ABI改版；前批Rust套件明示沿用，維護closure中的Rust／ABI輸入比對不變。沒有實體控制器測試或新權威對局／ACK，不冒稱完整MVP驗收。`playable-mvp-final-v1` 正在執行192個動態盤點native、完整owned與既有Blueprint最終本機驗收。

下列中間進度與失敗記錄保留作歷程；當前接受狀態以上段為準。

本批玩家操作：在操作設定的「游標與搖桿」區，先設定四向游標鍵，再開啟鍵盤游標；缺少任何方向或與啟用操作衝突時，整次修改拒絕且保留原設定。可選加速與精細鍵、每秒游標速度、死區，以及左右搖桿軸映射。預設右搖桿控制游標，左搖桿英雄移動僅在已公開的鍵盤移動模式生效。開啟功能、換設定、失焦或來源換代後，使用的軸須先實際回中立；另一未動軸不阻擋純水平／垂直操作。確認與選取可分別改鍵；右鍵保留取消。速度與輸入偏好依目前全域／模式／英雄層保存；畫面縮放、色盲與音量全域共用。

2026-10-18 第一輪原生結果：accessible-input-native-v1 真25項22成功／3失敗，150來源前後不變。PointerModel／AnalogPointer／GamepadDirection／MinimapConfirmPolicy及既有方向、命令／小地圖回歸通過；Profile發現scope洩漏並修正，另兩舊斷言隨正式新axis／物品目標語意更新。錯誤E1218–E1220完整保留。新修訂以accessible-input-source-v5凍結150來源，ordinary accessible-input-build-v6執行中，尚未接受。

小地圖確認接線已實作、尚待原生與實際事件驗證：既有 widget 唯一路徑加入 owner-qualified optional confirm 查詢，其他玩家事件不改變 capture；預設左鍵 camera／右鍵取消與移動保持原語意，非預設確認鍵可依法選地點，舊左鍵不冒認已改掉的 confirm。profile 保留右鍵取消，不接受配置成確認而產生互斥歧義。不可只因 World 路徑已改就宣稱全域改鍵完成。

2026-10-18 本輪：五項 AccessibleInput 原生回歸已編寫，新增 owned accessibility phase 14 cases／7 wire intents／5 producer stages；full 原444項順序不變，現在宣告458／222／157，尚未 fresh full 執行。新增案例使用真 Slate user 鍵盤／pointer／analog events，游標位置另記 accessible_input witness；synthetic renderer fixture 不代表實體控制器或權威對局。Lua artifact contract 127項實跑成功。151來源已擷取 accessible-input-source-v3，refresh 建置 accessible-input-build-v3 執行中，未接受。

實際事件計畫：在既有 owned synthetic renderer fixture 增 focused accessibility phase，完整 full inventory 的既有444項順序保留，新增案例附在尾端。覆蓋四向／加速／分像素／真viewport clamp、單軸省略零事件、焦點與editable guard、滑鼠／鍵盤確認與共用選取鍵、gamepad方向 Begin／Heartbeat／End及返回預設；以目前正式角色／FFI／wire記錄驗證，沒有新權威對局或正 ACK。新 phase／案例／producer stage／Lua oracle 一起更新，完整驗收仍最後執行，不把純模型成功代替實際事件。

2026-10-18逐軸復歸修訂：依真 XInput 的事件省略規則，FOmAnalogAxisSampler 讓每個映射軸各自觀察中立後才貢獻，未知軸保持封鎖；純水平輸入可獨立啟動，縱軸的零值事件不必出現。cursor／方向映射採一致逐軸死區；profile／owner／焦點換代清 sampler，非法數值不當作中立。已加 sparse-axis model 回歸，待 native 實跑。首輪 ordinary accessible-input-build-v1 140.54 秒因 EngineChange 未完成，149來源前後一致，見 E1209／E1210；進正式 refresh，不修改 Engine 衍生檔或 cache。

承接 task 3.10。先完成物品名稱／按鈕字級這批的凍結建置與原生回歸，再修改本批來源。下列為待實作計畫，不是完成紀錄。

1. 在既有 profile catalog 增加可選四向游標鍵、游標速度與搖桿偏好；沿正式序列化、匯入驗證、衝突檢查、英雄／模式覆寫與設定 UI。預設保持目前滑鼠操作，缺欄位採預設，非法有限值／缺少必要移動配置原子拒絕。
2. 游標只在該 local Slate user 的有效遊戲窗口移動，限制於實際 viewport 幾何，以每秒速度與有界 delta 計算，使用 Slate 每使用者 cursor 更新，不能操控其他窗口。失焦、換來源、退休和設定修訂均清掉持續輸入，舊按住必須回中立才重啟。
3. 搖桿英雄移動接現有 Direction 命令及 gesture identity、heartbeat、旋轉與權威準入；不新增本地移動預測。數位鍵與搖桿共用仲裁，反向／斜向／回中立必須可重現，不能因重複 axis event 無界送命令。
4. 選取、世界物件互動、普攻與確認施法各自可配置；共用鍵只在互斥上下文合法，不得一鍵同時選取與施法。HUD／聊天／modal 優先，游標移動不得繞過世界命令 hit-test。
5. 先驗純輸入模型及 profile roundtrip／非法匯入，再驗真 Slate user、按鍵／axis、焦點轉移、來源世代、改鍵、模態與退休。普通 UE 建置及針對回歸成功後才勾選功能；沒有實體搖桿或權威對局的測項須明示範圍。

來源盤點：FOmAttackMoveAimProcessor 已統一 owner preprocessor，但 Tick 為空、未接 analog；FOmDirectionalInputModel 已有來源／press identity、heartbeat、旋轉與 neutral 生命期，應擴充而非另送一條 Move。SelectUnit／WorldCommand／AttackOnlyClick 已各自存在；PointerConfirm 雖可設定，HandleAimPointerDown／MakeCastPointerSample 仍以實體 LeftMouseButton 判斷，沒有完整吃改鍵，先修此共用確認契約再接無障礙游標。SDK FAnalogCursor 的每使用者 UpdateCursorPosition 可重用；其預設 HandleKeyDown 會強制把 gamepad Accept 轉左鍵，不能直接啟用而繞過專案設定。

另發現操作設定的物品說明仍宣稱目前所有物品作用於自身，與正式 point_nova／unit_hex 不符；此靜態說明須改為依裝備目標型態／施法模式的一般說明，不另造即時配裝資料。

實作進度（尚未接受）：PointerConfirm 已接真鍵盤／滑鼠共用確認與配對去重，移除寫死左鍵；新增 OmAccessiblePointerModel 的有界速度、分像素累加、方向／加速／精細主次鍵合併與中立閘。schema9 加可選四向／速度 actions、左右搖桿軸與五項可儲存偏好，缺四向／缺軸／活動衝突／非法數字拒絕。Owner 沿 Slate 每使用者 cursor 更新，限制真 viewport，區分本人的 viewport／HUD／設定焦點與外部窗口，文字輸入只封鍵盤游標；搖桿英雄方向擴充既有 directional model／press identity／heartbeat，無 Rust 協定改版。失焦／來源／設定換代清掉軸樣本，橫縱兩軸均有當前觀察且回中立才恢復。設定 UI 已有開關、速度／死區、軸映射與主次游標改鍵，也修正物品目標說明。四項新 model／profile tests 待普通建置後真 native 執行，實際 viewport 新確認與搖桿矩陣仍待接入 owned fixture；不由純模型成功代替。
