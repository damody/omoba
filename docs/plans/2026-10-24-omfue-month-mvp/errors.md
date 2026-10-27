# 錯誤紀錄與防止重犯

每個錯誤記：現象、根因、通用修正、以後如何避免。所有 session 共用這個檔案；新紀錄加在最上面，只追加，不改寫別人的紀錄。 編號撞號時，後寫入者改用目前最大號 +1。

## E153 商店收尾：背包列被切、✓ 蓋到價格、設定面板字級沒跟畫面高度（2026-10-27，session-ward）

- **現象**（E151／E152 之後留下的三個小問題）：
  1. 720p 與 1080p 的商店底部列，背包六格換到第二行，被主框底邊切掉一部分。
  2. 1080p 字變大後，已擁有道具的 ✓ 蓋住價格的百位數字。
  3. 設定面板的字級仍只用使用者的 TextScale，跟 HUD 依畫面高度縮放的字級不一致。
- **根因**：
  1. 背包格是預設樣式的 SButton，樣式本身左右各多 13px padding，每格寬度約是圖示的兩倍，所以那一行放不下而換行。底部列又是參考高度 55·S 固定不變，換行後的第二行就被切掉。
  2. ✓ 是掛在卡片 overlay 左下角，底部 padding 寫死 16px。價格文字一旦比 16px 高，✓ 就會落在價格那一行。
  3. `UOmControlSettingsWidget::UpdateFontCache` 沒有乘上 E152 的高度係數。
- **通用修正**：
  1. 背包格改用無 padding 的 `GlyphButtonStyle`，720p、1080p、1440p 都能放在同一行。底部列的高度改為「參考高度與實際所需高度取大者」，道具格區相應讓出空間，所以即使字更大、換行，也不會被切。
  2. ✓ 改放在圖示本身的 overlay 左下角，無論字多大都在圖示上，不會碰到價格。
  3. 設定面板字級改為「使用者 TextScale × `ViewportTextFactor`（自己的 viewport 高度）」，和 HUD 共用同一個內文大小規則。
- **測試**：`Om.Runtime.ShopLayout` 在 720p、1080p、1440p 下檢查：
  - 背包 6 格都完整落在主框內。
  - 每一個可見的 ✓ 底邊都在價格頂邊以上，而且至少有一個 ✓ 顯示。
  - 2000×1013 參考比例測試（含底部列 3% 以內）仍然通過。
  - 影子專案：商店系列、ResponsiveHudLayout、PlayableMvpHud 共 10 項 Success。前一輪 Om.Runtime. 全量（含設定面板改動）只有兩項模型測試失敗，與這次改動無關。
  - 截圖在 `target/session-ward/fix4-shots/shop-720p-all.png` 和 `shop-1080p-all.png`。
- **避免**：
  - 小圖示按鈕不要使用預設 ButtonStyle，它會加上看不到的 padding。
  - 固定高度的區塊，要容得下內容可能換行之後的高度。
  - 角標要掛在它標示的那個元素上，不要用固定 padding 去估位置。

## E152 HUD 內文在 1080p 只有 11px：字級完全不隨畫面高度變化（2026-10-27，session-ward）

- **現象**：商店和整個 HUD 的內文在 1080p 只有 11px（參考圖約 15px），看起來偏小（商店子 agent 留下的缺口 2）。
- **根因**：`FOmHudReadableText` 的字級只由使用者的 TextScale 決定（內文 11、標題 14、技能格 12），刻意不跟 viewport 連動。HUD 會抵消 1080 以下的 DPI 縮小，所以 720p 和 1080p 都是 11 個實體 px；1440p 以上才會隨 UE DPI 曲線放大。
- **通用修正**：
  - 新增 `FOmHudReadableText::ViewportTextFactor(HUD 高度)`：720 以下為 1，1080 以上為 14/11，中間線性內插。
  - `UOmCommandBarWidget::RefreshClusterScales` 以「使用者 TextScale × 這個係數」套用字級（仍受 ScaleLower／ScaleUpper 限制）。
  - 結果：1080p 內文 14px、標題 18px；720p 維持 11px（空間不夠放大）；1440p 的 logical 高度是 1080，同樣是 14，再由 UE DPI 曲線放大。所有 HUD 尺寸本來就由 `AppliedTextScale` 推導，會一起跟著變。
  - 設定面板也已在 E153 改為同樣的規則。
- **測試**：
  - 新增 `Om.Runtime.HudTextViewportFactor`：600、720 為 11；1080、1440 為 14；900 介於兩者之間；1080 的標題為 18；沒有高度時為 1。
  - 鎖定字級的測試改為依實際套用的值計算：`ResponsiveHudLayout` 的「Preview text scale is applied」改成預期 TextScale × 係數；選單按鈕字級改用 `GetAppliedTextScale()`。
  - `PlayableMvpHud` 的 hotbar 上限從固定 160 改成 160 + 最多 20（1080p 時字變大後為 165）。
  - 影子專案 Om.Runtime. 全量：除了這兩項之外，只有模型相關的 NativeCharacterAssembly、NativeCreepPresentation 失敗（Tripo 模型那條線，與字級無關）。修正後 13 項 HUD／商店相關測試全部 Success。
- **避免**：決定可讀字級時，要同時考慮實體像素和畫面高度；改了字級規則，要用 `FOmHudReadableText` 的函式在測試裡推導預期值，不要寫死數字。

## E151 實機 720p 商店退回直式堆疊版：隊友卡和提示行佔掉帶狀區（2026-10-27，session-ward）

- **現象**：實機 720p 開商店時，HUD 標頭（比分列、4 張隊友卡、回城、操作提示）到指令列之間只剩約 315px，S 約 0.45，低於 `FOmShopSpec::MinScale` 0.59，於是改用舊的直式堆疊商店（商店子 agent 留下的缺口 1）。
- **根因**：商店帶狀區的上緣是整個標頭欄的底（`HeaderBottom`），隊友卡在 720p 就佔掉約 80px。
- **通用修正**：
  - 商店開著時，隊友卡（`OmTeamHudViewport`）暫時收起，關閉後恢復，所以商店永遠不會蓋住隊友卡。
  - 帶狀區上緣改用標頭頂端的固定列（狀態列、比分列與右上角的設定按鈕，`ShopTopStripBottom`）。標頭較下方的回城、提示行在畫面左側，而商店主框在 720p 從 x 約 168（快速購買欄）、y 約 88 開始，實際上不會重疊。
  - 結果：720p 的 S 約 0.68，使用參考排版（兩欄、快速購買欄、合成樹）。
- **測試**：`Om.Runtime.ShopLayout` 加上實機標頭（4 張隊友卡加兩行操作提示）。檢查項目：
  - 720p、1080p、1440p 都使用兩欄商店。
  - 商店開著時隊友卡收起，關閉後恢復。
  - 截圖在 `target/session-ward/font3-shots/shop-720p-all.png`。
- **後續**：背包列被切、✓ 蓋到價格，已在 E153 修正。
- **避免**：要擠進帶狀區的 modal，應該先收起它會蓋到、而且暫時不需要的 HUD 元件；不要只靠縮小 modal。

## E150 商店第三輪（依參考圖量測）：換解析度後詳細欄沿用舊尺寸；1080p 底部列換行，autopilot 找不到背包格；patch 工具把 LF 寫進 CRLF 標頭（2026-10-27，session-shop）

- **現象**：
  - 版面測試在 720p 失敗：`upgrade row above the tree`。進階道具列量到 50px 高（應約 23），疊到合成樹上。
  - 1080p 實機局（live-5）裡，autopilot 兩位玩家的 `shop_sell_select` 都是 `widget OmShopSell.0 not visible within 6.0s`。截圖顯示底部列的交易狀態文字佔掉寬度，背包六格被擠到第二行，而第二行在框外被切掉。
  - 改完商店後第一次實機局（live-3、live-4）在啟動時就被擋下：`launch binaries carry different content: authority content=3ced… renderer bridge=…`。
  - 用 Lua 區塊替換工具（`target/session-shop/sub.lua`）修改 `OmCommandBarWidget.h` 後，這個 CRLF 標頭出現了 LF 行。用 Lua 長字串 `[[...]]` 包 C++ 時，遇到 `]]]` 會提前結束字串。
- **根因**：
  - 詳細欄（進階道具、合成樹）是在選取道具時依「當下的縮放與模式」建好的，視窗大小改變不會重建。測試從 1080p 換成 720p 時，留著的是另一種尺寸（compact）建出來的列。
  - 參考圖的底部列只有出售、復原、金幣，我們多了狀態文字和背包六格；縮放約 0.9 時，一行放不下。
  - 別的 session 改了 lua_data，伺服器的 content hash 和影子專案的 bridge 不一致（和 E149 同一類）。
  - 替換工具把新內容原樣寫入，沒有沿用檔案的行尾。
- **通用修正**：
  - `SOmShopFrame::Tick` 偵測 fit 的模式或縮放改變（取到千分位），改變時呼叫 `OnLayoutChanged`，清掉詳細欄和快速購買欄的 signature 後重建。詳細欄的 signature 也加入縮放與模式。
  - 參考圖版面的底部列固定一行：出售、復原、金幣、背包六格（20px×S）。交易狀態顯示在分頁列的狀態欄，復原的原因寫在按鈕 tooltip。compact 堆疊模式維持原本的狀態列順序（Responsive 的 narrow 測試要求捲動後同時看得到復原按鈕和原因）。
  - autopilot 的快速購買步驟在 compact 商店（沒有快速購買欄）時直接略過，不會讓整段商店腳本逾時。
  - 實機局改成不帶 `--no-build`，讓伺服器重建到目前的 content；影子專案改用 `resync2.sh` 同步，流程是先 sync、再 3-way merge、最後 stage，所以 stage 失敗也不會丟掉 session 的修改。
  - 替換後一律用 `crlf.lua` 正規化行尾，並比對 CR 數量；Lua 內嵌 C++ 改用 `[==[ ... ]==]`。
- **驗證**：影子專案 d3d12 跑 Shop*／ResponsiveHudLayout／PlayableMvpHud／HudAssetRegistry／LegacyTdInputGuard，12/12 通過；nullrhi 10/10 通過。2000×1013 的參考比例檢查中，10 個區塊的尺寸誤差與位置誤差都 ≤ 1.6%（表格在 `shop-layout-spec.md`）。
- **避免**：
  - 依幾何建出來的 Slate 列（尺寸寫在建構當下），要在 layout key 改變時重建，不能只在資料改變時重建。
  - 照參考圖固定一行的列，要先算出最小縮放時的總寬；放不下的內容移到 tooltip 或其他區域，不能讓它換行到框外。
  - 改了 lua_data 或 bridge 之後的實機局，不要帶 `--no-build`。

## E149 lua_data 放回 live 後，release omobab 沒有重建，實機局被 content hash 檢查擋下（2026-10-27，session-tripo）

- **現象**：野怪的 lua_data 在 15:55 經 `land_to_live` 放回，main 在 15:57 建好 live 的 UE bridge。之後跑實機局，launch 拒絕啟動，訊息是 `authority content=3ced…, renderer bridge=884d…`。
- **根因**：`land_to_live` 和 `check_live_consistent` 只對候選樹跑 `cargo check`，不會重建 `omb/target/release/omobab.exe`。這個 exe 停在 14:54，內嵌的是舊的 content hash。
- **通用修正**：沒有改程式。launch preflight（E125）已經正確擋下，訊息也指出要重建哪一個。照著重建 release 的 omobab、client runtime 和 base_content 之後就能啟動。
- **以後**：放回 lua_data 之後要跑實機局，先重建 release 的三個 Rust 產物再 launch。不要懷疑 bridge，bridge 是在 live 建置時跟著重建的。

## E148 斜視角加 FOV 20° 之後，浮動數字飄到名牌上方 100–250 px：大小、行距、上升距離都是世界單位（2026-10-27，session-ward）

- **現象**：鏡頭改成 -56°、FOV 20° 並拉遠（E140／E147）之後，`target/spotlight-camera-fov20b/date_masamune/keyframes/02_slot0_enemy_effect.png` 裡的「-400／-110.2」字變大，離名牌和單位太遠，兩行之間的間距也很大。
- **根因**：`UOmCombatVisualSubsystem` 的文字大小（`WorldSize` 28／32）、堆疊行距（`HealthTextRowHeight` 28）、上升速度（`HealthTextRisePerSecond` 45/s）都是世界單位，換算到畫面上的像素取決於鏡頭距離和 FOV。窄 FOV、遠距離時每公分佔的像素變多，所以整個堆疊被放大、推高。避開名牌的 `FOmFloatingTextClearance::Lift` 本來就是用畫面像素計算的，不是原因。
- **通用修正**：
  - 改用畫面空間的常數，以 1080 px 高為基準，隨 viewport 高度等比例縮放：字高 26 px（暴擊 30 px）、行距 30 px、上升 36 px/s。
  - 每個 frame 由實際投影量出數字原點沿 screen-up 每單位佔幾個像素（`PixelsPerUnitAlong`，100 單位探針），再由純函式 `FloatingTextLayout` 換算成 `WorldSize` 和偏移量。所以任何鏡頭距離、FOV、解析度下看起來都一樣。沒有本機鏡頭時，退回原本的世界單位乘以單位縮放。
  - 堆疊時判斷「列是否被占用」的比例改用像素常數（rise/row 的比例維持一致）。
- **測試**：新增 `Om.Runtime.FloatingTextScreenSpace`。
  - 情境：用 UE 實際的 view-projection（`UGameplayStatics::GetViewProjectionMatrix` 加 `FSceneView::ProjectWorldToScreen`），測舊俯視鏡頭（-80°、FOV 90、距離 2000）和斜視角（-56°、FOV 20、距離 9000），解析度 1920×1080 和 1280×720。
  - 斷言：第 0、2 列、age 0 和 0.5 s 時，離原點的像素距離與預期誤差 ≤ 1 px；字高誤差 ≤ 0.5 px；暴擊字較大；沒有鏡頭時退回世界單位。
  - 影子專案結果：FloatingTextScreenSpace、FloatingTextClearance、PublicHealthTextLifecycle、PresentationCueSlots、NameplateAboveModelOnScreen 全部 Success。
- **目視確認，第二輪修正**：
  - 用 live binaries 錄 spotlight（`target/spotlight-ft148-live2/date_masamune/keyframes/02_slot0_enemy_effect.png`）：數字已經貼在名牌上方，但字偏小，而且「-400」和「-110.2」疊在一起。
  - 疊字的原因：每個數字停在自己生成時的世界位置。date 衝刺後，新的數字生成點在畫面上往上移，用世界單位算出來的列就壓到舊的數字上；各數字自己記錄的名牌 lift 也會各自不同。
  - 修正：
    - 有本機鏡頭時，生成時在畫面上選列：從該擁有者目前的名牌 lift 開始，取最低一個與所有存活數字的畫面位置相距一整列的列。
    - Tick 時同一擁有者共用最大的 lift。
    - 字放大到 32 px（暴擊 38 px），行距 32 px。
  - 影子結果（`target/spotlight-ft148-shadow/date_masamune/keyframes/02_slot0_enemy_effect.png`）：三個數字等距排列、不重疊，最新的一個在名牌上方約 40 px（含上升）。四項相關測試都 Success。
- **避免**：疊在畫面上的 HUD 元素（數字、名牌、提示），大小和距離都要用畫面像素定義，由實際投影換算成世界單位；不要寫死世界單位，否則鏡頭參數一改就會跑掉。

## E147 鏡頭改用 20° 窄 FOV：縮放距離改成「參考鏡頭距離」、霧要依多出的距離補償；autopilot 買失敗後商店面板沒關，擋住所有地面點擊（2026-10-27，camera-oblique）

- **需求**：使用者要求把 FOV 從 90° 降到 20°、把鏡頭拉遠來減少廣角變形，同時維持差不多的地面視野。距離要從 FOV 推算，不能寫死。
- **通用修正（相機）**：
  - `AOmRtsCameraPawn` 的 FOV 改由程式定義：`DefaultFieldOfViewDegrees = 20`，在 constructor、`OnConstruction` 和 `PostInitializeComponents` 套用。level 上放置的 pawn 載入時不會重跑 construction，所以 `PostInitializeComponents` 也要套用。
  - 所有縮放值都改成「參考鏡頭（90°）距離」，包括 `DesiredZoomDistance`、上下限、滾輪步距、地圖 fit 和 control profile。實際 arm 長度是 `GetArmLengthForZoom(zoom) = zoom × tan(45°)/tan(FOV/2)`，20° 時約為 5.67 倍。上下限、滾輪步距和地圖 fit 因此全部等比例調整，玩家存好的 profile 也不需要遷移。
  - 霧／大氣：Main.umap 的 ExponentialHeightFog 和 SkyAtmosphere 會依相機距離累積霧，相機拉遠後畫面整片泛白。pawn 改成把霧的 `StartDistance`、`FogCutoffDistance`、`VolumetricFogStartDistance` 和大氣的 `AerialPerspectiveStartDepth` 往後推「arm − arm/鏡頭倍率」。只在 arm 長度變化超過 1% 時更新；fog 元件每 2 秒最多掃描一次；EndPlay 時還原。
  - 已確認不需要改的部分：
    - 陰影：VSM 已開啟（`r.Shadow.Virtual.Enable=1`），clipmap 的範圍足以涵蓋 4～5 km。
    - far clip：UE 的 perspective 投影用無限遠的 far plane。
    - 名牌：screen-space widget，大小固定。
    - 浮動數字：world-space，但地面覆蓋範圍不變時，焦點附近的投影大小也不變。
    - 滑鼠射線和小地圖視錐框：都是逐像素 deproject 到地面，跟 FOV 無關。
- **潛在 bug（小地圖）**：`FOmMinimapModel::Unproject` 的像素邊界檢查沒有容許捨入誤差。arm 變長後地圖範圍數值變大，點在小地圖邊角的點擊會被拒絕（MinimapCameraOrientation 的 corner click 失敗）。已加上 `1+1e-9` 的容差，和既有的世界座標邊界容差一致。
- **autopilot bug**：
  - 現象：FOV 20 第一局，autopilot 在英雄剛離開商店範圍時開了商店。`OmShopBuy.auto` 3 秒內沒出現，`UiQueue.Reset()` 連後面的 `shop_close` 一起丟掉，面板一直開著蓋住畫面中央。之後的地面點擊都被正式的指標分類器判定為 HUD，出現 82 次 `projection_failed` 和 533 次「鏡頭回到自己」，英雄卡在出生點約 20000 個 tick。
  - 修正：`FUiStep` 新增 `bCleanup`，`shop_close` 標記為清理步驟。腳本中某一步失敗時，只保留清理步驟，其餘捨棄。
- **測試修正**：`OmCompiledMapSceneCapture`、`CameraGroundCoverage`、`WorldUnitVisualScale` 原本斷言 `TargetArmLength == zoom`，改成 `GetArmLengthForZoom(zoom)`。`RtsCameraObliqueDefault` 新增斷言：FOV 等於 20°，且 `arm × tan(FOV/2)` 等於 `zoom × tan(45°)`。
- **驗證**：
  - 影子專案 `target/ue-shadow-camera` 中，19 個相機相關測試全部通過，包含 Camera*、Minimap*、NameplateAboveModelOnScreen、WorldSurfacePresentation、WorldUnitVisualScale、FloatingTextClearance、RtsCameraObliqueDefault。
  - spotlight `target/spotlight-camera-fov20b`：8 張關鍵影格，構圖不變，邊緣角色不再傾斜，霧補償後的顏色和 FOV 90 時一致。
  - 對局 `target/camera-fov20-match3`：quick map 在 600 秒內分出勝負（隊伍 2 勝）。鏡頭為 `[-56,0]`，autopilot 送出 251 個指令。
  - 相機高度和 FOV 90 時的比例約為 5.6，符合等覆蓋範圍的推算。
- **附帶發現**：影子專案剛重新 stage 後的第一局，UE 的啟動和 shader 暖機比平常多約 60～95 秒，600 秒的時間上限因此被吃掉。快取熱了之後，第三局就在時限內打完。
- **放回 live**：這些改動是在 E143 的 `land_to_live.lua` 流程出現之前直接寫進 live 的。事後用 `check_live_consistent.lua --ubt` 確認 live 一致（codegen、cargo、server、bridge、UBT 都通過）。之後的相機改動改走 land_to_live。
- **避免**：
  - 改 FOV 時要一起處理 arm 長度、隨距離累積的霧／大氣，以及所有把 arm 長度當成 zoom 的斷言。
  - autopilot 的 UI 腳本凡是會打開面板的，都要有清理步驟；某一步失敗時不能把關閉面板的步驟也丟掉。

## E146 Tripo 兵線：共用 lua_data 改到一半，live codegen 失敗；`--sync` 把影子裡未放回的 C++ 覆蓋掉；陣營材質只支援靜態模型（2026-10-27，Tripo 兵線）

- **現象**：
  - 14:29 左右起，live 的 `om_codegen` 對共用 `scripts/lua_data` 失敗，所有 `build_ue_moba`／`stage_ue_shadow` 都被擋下（omfx-decouple 回報）。
    - 先是 `invalid type: map, expected a sequence`：小兵 profile 裡空的 `attachments={}` 被序列化成 map。
    - 修掉後又遇到 `unit 'lane_melee' invalid static native visual metadata`：Lua 已經給「會動的小兵」設了 `ally_material_sections`，但 live codegen 只接受靜態模型使用它。
  - 為了重新 stage bridge 跑 `stage_ue_shadow --sync`，把影子專案裡還沒放回 live 的 C++（陣營材質延伸到會動的模型、姿勢檢查擴充）覆蓋回 live 版本。
- **根因**：
  - `scripts/lua_data` 和 codegen 一樣是 live 輸入，我卻把它當成只影響自己影子專案的設定在改。
  - `--sync` 的定義就是「影子 = live」，不會保留影子專案自己的改動。
- **修正**：
  - 當下：共用 lua_data 退到 live codegen 支援的版本（小兵先換 Tripo 身體，`team_colored={}`），live codegen 恢復正常。
  - 驗證流程改成三份副本：
    - codegen 用 `target/session-assets/codegen`（Cargo 路徑多一層 `../`，CARGO_TARGET_DIR 獨立）；
    - Lua 用 `target/session-assets/lua_data`；
    - C++ 用影子專案，並把改動寫成 patch 腳本（scratchpad `patch_relation.lua`），`--sync` 後重新套用。
    - 驗證通過後，codegen、C++ 和 Lua 同一批放回 live，再用 live 跑一次 codegen，以及 `stage_ue_shadow --sync --build`，確認一致。
  - 功能面：
    - 陣營材質改成 `AOmUnitActor` 的通用機制：`GetRelationMaterialMesh()` 靜態模型回傳 `NativeBodyMesh`，`AOmAnimatedUnitActor` 回傳 `NativeHeroMesh`；`InitializeNativePresentation` 結束時套用；陣營未知時還原成記住的原始材質。
    - codegen 抽出 `generated_relation_material_sections`，靜態和會動的模型都可以用（測試共 90 項）。
  - `Om.Runtime.NativeHeroPoseSanity` 改成列舉所有用 TripoV1 身體的 `AOmAnimatedUnitActor`，不再只看 OmHero 類別。現在 13 個都通過：8 位英雄、近戰／遠程／攻城／超級兵、測試用 probe。
- **驗證**：
  - 影子專案 `WorldUnitVisualScale`（新增兵線陣營斷言與截圖）、`NativeHeroPoseSanity` 通過。
  - 實機局 `target/minions-match-3`：友方小兵藍衣、拿長槍的足輕。
  - 遠鏡頭兩隊同框：`.ai-collab/structures/world_fit_ally_enemy_minions.png`。
  - 放回 live 後，live codegen 測試 90 項通過，`stage_ue_shadow --sync --build` 通過。
- **避免**：
  - `scripts/lua_data` 也是 live；改 lua_data 時，先在副本 content root 驗證。
  - 影子專案有還沒放回 live 的改動時，不要跑 `--sync`，或先把改動寫成可以重套的 patch。

## E145 量測局的 client 連到另一個 session 的 server：預設 port 衝突，server 綁定失敗，launcher 沒發現就啟動 client（2026-10-27，P0 #5）

- **現象**：第一次 10 英雄量測局（perf-10hero-1）在 `runtime-p1 ready timeout` 失敗。runtime 的 log 顯示 `Connected to KCP server at 127.0.0.1:57061`，卻始終沒有 ready。
- **根因**：
  - 所有 session 的 split 都預設用 port 57061。另一個 session 14:49 啟動的 server（PID 59796）已經佔著這個 port，我的 server 綁定失敗並結束（`Failed to bind KCP listener on 0.0.0.0:57061`）。
  - `moba_role_launch.launch` 啟動 server 後沒有確認它真的綁好 port，就繼續啟動 client。結果 client 連到別人那局的 server，等一個永遠不會來的 ready。
- **通用修正**：
  - `moba_role_launch.wait_server_bound`：啟動 server 後先等 `KCP listener bound on` 這行。
    - server 結束或出現 `Failed to bind` 時，立即報錯，附上原因並建議換 `--port`。
    - 舊版 server 沒有這行 log：出現 `Starting KCP server` 後 5 秒內沒失敗，就視為已綁定。
    - dedicated server 模式也一樣處理。
  - omb `kcp_transport.rs` 綁定成功後加一行 `info!("KCP listener bound on {}")`，以 `land_to_live` 放回（E143 的第一次正式使用）。
  - renderer 的 content 一致性檢查（E125）從 prepare 移到 launch，在實際要執行的 UE 執行副本上檢查 bridge；沒有 authority content hash 的手寫計畫不檢查。這樣 prepare-only 測試就不會受當下 live 前端狀態影響。
- **驗證**：
  - `test_moba_role_launch.lua` 11/11、`test_moba_dedicated_server.lua` 7 組通過。共用的測試 fixture 讓假的子程序輸出綁定成功那行。
  - `moba_role_finish_observer_test.lua` 失敗在 `unknown launch mode`：測試的計畫沒有 mode，與本次修改無關。
- **避免**：
  - launcher 啟動任何程序後，要等它明確回報就緒才啟動依賴它的程序；不能只看程序已經 spawn。
  - 同時跑多個 split 時，每個 session 用各自的 `--port`（spotlight 用 57461，量測用 57261）。

## E144 1080p、10 英雄的 frame time p95／p99：UE 端原本只有 60 幀視窗的平均與最大值，無法得到分位數（2026-10-27，P0 #5）

- **現象**：P0 #5 要求 1080p、10 英雄的 p95／p99 frame time。但 renderer 的 `OM_PERF` 每 60 幀只輸出一次 mean／max（而且標明 `not_a_statistic: p95`），既有工具都算不出分位數。
- **通用修正**：
  - **UE**（`OmPresentationPerf.h/.cpp`、`OmWorldBridgeActor`）：
    - 新增 `FOmPresentationPerfHistogram`：0.25 ms 一格、上限 250 ms，另有一格溢位。
    - 每 600 幀、以及 presentation 結束時輸出 `OM_PERF_HIST`：從 presentation 開始累積的完整直方圖（稀疏的 bucket:count），加上 p50／p95／p99 和 max。
    - 指標有兩個：game_frame_interval（game thread DeltaSeconds）和 presentation_work（Om presentation tick 本身的工作量）。
    - 已在影子專案建置並通過 `Om.Generated.PresentationPerfWindow`，再以 3-way merge 合併回 live。
  - **報告**：`scripts/moba_frame_time_report.lua UE_LOG [--warmup-seconds N] [--json OUT]`。
    - 取暖機結束前的最後一行和最後一行相減，得到只含量測區間的分佈。
    - 量測條件（OS、CPU、GPU、解析度、RHI、是否離屏、幀率上限）一律從同一份 log 讀出。
    - 分位數取 bucket 上緣，最多偏高 0.25 ms；超過 250 ms 的只回報「>250」，不填一個捏造的數字。
  - **啟動選項**：`--frame-cap N`（`moba_role_launch`／`run_moba_split_local`）。預設 60，也就是玩家拿到的 `t.MaxFPS 60`；0 表示不限，用來量實際成本與餘裕。
- **量測結果**（i9-12900KS、RTX 4070、Windows 11、1920x1080、d3d11、離屏、真實地圖 `moba_single_player.lua`：1 個 autopilot 席＋9 bot，共 10 英雄；扣除前 60 秒暖機）：

  | 條件 | 樣本／時長 | p50 | p95 | p99 | 平均 | >250 ms |
  |---|---|---|---|---|---|---|
  | 上限 60（出貨設定，perf-10hero-2） | 17400 幀／295 s | 16.75 ms | 16.75 ms | 17.00 ms | 59.0 fps | 1 |
  | 不限（perf-10hero-uncapped） | 109800 幀／520 s | 3.75 ms | 7.50 ms | 48.00 ms | 211 fps | 1 |

  - presentation_work（Om 自己的 tick）：上限 60 時 p50 2.00、p95 3.25、p99 5.00 ms；不限時 p50 0.25、p95 2.25、p99 19.25 ms。
  - 解讀：出貨設定下 p99 仍維持 60 fps；不限幀率時 p95 有 2 倍以上餘裕。不過約 1% 的幀會出現 20～50 ms 的尖峰，而且 presentation_work 的 p99 同步升高，表示尖峰主要來自 Om presentation 的突發工作（推測是大量快照套用或 HUD 重建），值得後續分析。量測區間內各有 1 幀超過 250 ms。
- **量測條件的限制**（必須一併閱讀）：
  - 離屏（`-RenderOffScreen`，E114 規定自動化不得佔用使用者桌面）沒有 swap chain present、沒有 DWM 合成、沒有 vsync 等待，所以真實視窗的 frame time 會比較高。真實視窗的數字要由使用者在不使用桌面時，以 `OM_OWN_DESKTOP=1`、同樣的指令再量一次。
  - renderer 是 editor 的 `-game` 模式（從影子專案的執行副本啟動），不是 Shipping 打包版。
  - 上限 60 那次和另一個 session 14:49 啟動的對局部分重疊，CPU 被分走；不限那次沒有確認有沒有其他對局同時在跑。
  - 長時間 soak（2 小時）的取捨仍需使用者決定，見 next-month.md P0 #5。
- **驗證**：`moba_frame_time_report_test.lua` 14 項通過；UE 單元測試 `Om.Generated.PresentationPerfWindow` 通過（影子建置）。
- **避免**：效能數字一律附上量測條件（硬體、解析度、RHI、是否離屏、幀率上限、是否有其他負載）。有幀率上限時，frame time 只能證明「達到上限」，要量成本和餘裕必須另外跑不限幀率的一局。

## E143 直接改 live、留下半成品已經第五次：需要一個會先驗證、再一次寫入 live 的落地流程（2026-10-27，omfx-decouple）

- **現象**：
  - E076（stuck recover 落地時沒編譯 UE）、E107／E109（載入畫面在 live 改到一半，其他 session 編譯失敗）、E139 都是同一類問題。
  - 2026-10-27 14:29 也發生一次：共用 lua_data 的小兵陣營材質先放進 live，但 live 的 codegen 還不支援，`om_codegen` 失敗（`invalid type: map, expected a sequence`）。這段期間所有前端建置都失敗，server／runtime 的 content（eef2…）也和 bridge（3072…）分開了。
- **根因**：lua_data、codegen、omfue C++、Rust crate 都是其他 session 正在建置的 live 輸入，但是否改好再放回只靠自律，沒有任何機制在寫入前確認「live 加上這批變更」仍能 codegen 和編譯，也沒有防止只放回一部分。
- **通用修正**：
  - `scripts/land_to_live.lua --change SRC=LIVE[@BASE] ... [--manifest FILE.json] [--checks auto|...] [--dry-run]`：
    - 在 build lock 保護下計算每個檔案的結果：有 BASE 時，用 git merge-file 3-way merge 到目前的 live 檔；有衝突就整批不放回。
    - 在 `target/land/tree` 鏡像 Rust 與 Lua 原始碼（scripts、omoba-*、specs、omfue/codegen、omfue/bridge，不含 target），套上這批變更。需要 UBT 時，另外把 `target/land/shadow` 從 live 同步，再套上 omfue 的變更。
    - 依變更涉及的路徑自動決定檢查：
      - 永遠跑 om_codegen，而且讀的是候選樹的 lua_data，不會寫入 live 的 OmGenerated；
      - Rust、lua_data 或 codegen 有變更時，跑 cargo check base_content；
      - bridge 或 client runtime 有變更時，跑 cargo check bridge，並重產 header；
      - omfue、lua_data 或 codegen 有變更時，跑影子專案的 UBT。
    - cargo 使用獨立的 `CARGO_TARGET_DIR=target/land/cargo-target`，不會鎖到或改寫任何人的 target。
    - 全部通過、而且檢查期間沒有任何 live 檔被別人改動，才一次寫入所有檔案；否則一個都不寫，並印出原因，結果也寫進 `target/land/last.json`。
  - `scripts/check_live_consistent.lua [--ubt]`：同樣的檢查，但不帶任何變更，任何人隨時可以跑，確認 live 目前能否 codegen 和編譯。
  - AGENTS.md 核心注意事項加入規則：live 輸入一律經 land_to_live 放回。
- **驗證**：
  - `scripts/tests/land_to_live_test.lua` 26 項，涵蓋：
    - 路徑決定要跑哪些檢查、拒絕 repo 外與 target 內的路徑；
    - 檢查失敗時 live 不變，並回報原因；
    - 通過時才寫入，codegen 讀候選樹，live 的 OmGenerated 不被寫入；
    - 3-way merge 保留雙方修改，衝突時整批拒絕；
    - 多檔要嘛全部放回、要嘛全部不放回，UBT 失敗時全部不放回；
    - 檢查期間 live 被改動時拒絕；
    - dry run、manifest、參數解析；check_live_consistent 不帶變更。
  - 第一次實際執行 check_live_consistent 就失敗：base_content 的 build.rs 會執行 repo 內的 `tools/lua/lua.exe`，但候選樹沒有這個目錄。補上 `tools/lua` 後通過：codegen、cargo check base_content、omobab、bridge 都通過。
  - 第一次正式放回：E145 的 kcp「綁定成功」log（omb/src/transport/kcp_transport.rs）帶 BASE 經 3-way merge，codegen、cargo check base_content、omobab 通過後寫入 live（15:22）。
- **避免**：
  - 不要直接編輯 live 的 lua_data、codegen、omfue 原始碼或 Rust crate，改在影子或副本完成後用 land_to_live 放回。
  - 覺得 live 壞了，先跑 check_live_consistent 確認，不要猜。

## E142 商店改版第二輪：快速購買欄的價格被切掉；影子專案重新同步會蓋掉 session 自己的修改；Edit 工具把混合行尾檔的單一 CRLF 改成 LF（2026-10-27，session-shop）

- **現象**：
  - 商店框左側的快速購買欄（寬 64）裡，「1,450」只畫出「1,45」。測試量到價格文字實際寬 22、需要 28（720p）。
  - 影子專案 `stage_ue_shadow.lua --sync` 第二次執行時，鏡像把 session 已修改的 13 個檔案換回 live 版（大小不同就覆蓋）。同一次同步的 content codegen 又因為別的 session 改到一半的內容失敗（`unit 'lane_melee' invalid static native visual metadata`），腳本中斷，原本接在後面的 3-way merge 沒有執行。
  - 第一次影子建置時，OmGenerated 編譯失敗（`NativeStaticAllyMaterials` 未宣告）：live 的 codegen 已經輸出新欄位，`OmUnitActor.h` 卻是在同步之後才由別的 session 落地。
  - `OmAutoPilot.cpp` 幾乎全是 LF，只有第 2 行是 CRLF。用 Edit 工具修改後，這一行被改成 LF，diff 會多出一行無關的變更。另外，我又對 CRLF 的 `ue_run_snapshot_test.lua` 用了 `sed -i`，整個檔案變成 LF（E107 的老問題）。發現後已用位元組安全的 Lua 改回 CRLF。
  - 最後一輪測試時，`ue_run_snapshot.prepare` 連續失敗 8 次，訊息是 `live omfue binaries kept changing`。實際原因是 `target/ue-run/7f504b8c87152aba` 只剩一個 omfue/Saved／Intermediate 的空殼，裡面沒有 ready.txt：這個副本在 renderer 啟動時被其他 session 的 prune 改名移走（那次 UE 回報 `module 'OmEditor' could not be found`），UE 隨後又在原路徑重建了 Saved。之後每次 prepare 的 rename 都撞上這個殘留目錄，只好重試，最後回報錯誤。
- **根因**：
  - 預設的 `SButton` 樣式（CoreStyle "Button"）左右共佔約 24px padding，放進 56px 寬的欄位後，內容只剩 22px。
  - 影子同步的設計是「影子＝live 的鏡像」，不會保留 session 自己的修改；codegen 讀的是 live 當下的狀態，可能剛好是別人改到一半的版本。
  - Edit 工具會把整個檔案統一成一種行尾。
- **修正**：
  - 快速購買卡改用無 padding 的 `GlyphButtonStyle`；欄內價格不加千分位，並且置中。`Om.Runtime.ShopLayout` 新增斷言「Quick price is not clipped」（實際寬度 ≥ 需要寬度），720p 和 1080p 都會檢查。
  - session 工作流程：同步前先備份自己改過的檔案（`target/session-shop/mine`），同步後以 `git merge-file`（mine／base／live）合回，base 改成這次的 live 版。codegen 失敗時，照樣手動執行合併；影子沿用上一次成功的 OmGenerated，仍可建置與測試。
  - 修改混合行尾的檔案後，比對 CR 數量。這次已用位元組安全的 Lua 把第 2 行的 CRLF 復原。
  - `ue_run_snapshot.prepare`：目標副本目錄存在但沒有 ready.txt 時，視為殘留，先用 `remove_tree` 清掉（不會進入 junction），再 rename。`ue_run_snapshot_test.lua` 新增這個情境（35 checks）。
  - 驗收：影子專案 d3d12 與 nullrhi 都跑過 Shop*／ResponsiveHudLayout／PlayableMvpHud，全部通過；兩場 ≤10 分鐘的離屏實機局（`target/session-shop/live-1`、`live-2`），購買、選取、出售、復原、快速購買都被權威端接受。之後以 3-way merge 放回 live，沒有衝突。live 尚未重新建置。
- **避免**：
  - 窄欄（< 80px）裡的按鈕，一律使用無 padding 的樣式，並在測試中斷言文字沒有被切掉，不能只看截圖。
  - 影子專案裡的修改，在重新同步前一定要先備份；同步腳本失敗時，先檢查自己的檔案是否已被覆蓋，再決定下一步。
  - 用 Edit 工具改檔之後，比對檔案的 CR 數量和 base 是否一致。

## E141 IPC 失敗在 `input-tap-w`：中斷的測試把玩家自己的鍵盤 W 改成 F6 留在 profile 裡（2026-10-27，session-ward）

- **現象**：live 建置後單獨跑 IPC（live-build2-ipc-2），失敗在 `owner control wire sequence incomplete: input-tap-w`。鍵盤方案已經切換成功（`input_keyboard` 有 ack），但按 W 沒有送出任何方向意圖。
- **根因，跟 E135 的預設鍵位修改無關**：
  - 使用者本人的 `omfue/Saved/Controls/1/profile.json` 裡，鍵盤 patch 的 MoveForward 是 F6。
  - IPC 的 `input-rebind`（case 9）會把 MoveForward 改成 F6，等到 `input-restore-binding`（case 11）才改回 W。
  - 上一輪 `live-build2-null-1-ipc` 在 13:10 執行途中被中止，F6 就留在了玩家自己的設定檔裡。
  - 之後每一輪 IPC，以及使用者本人用鍵盤方案時，按 W 都沒有反應。
  - 更根本的問題：自動化程序（IPC、本機驗收、autopilot、spotlight、shader warmup）讀寫的都是玩家自己的 `Saved/Controls/<id>`。另外有十多個測試自己組這個路徑做備份和還原，所以任何中途中止都會改壞使用者的鍵位。
- **通用修正**：
  - runtime 新增 `UOmPlayerActionOwnerComponent::ControlProfilePath(PlayerId)` 作為唯一的路徑來源。有 `-om-control-profile-dir=<dir>` 時改用 `<dir>/<id>/profile.json`。`PrepareProfilePath` 和所有會擷取或寫入 profile 的測試（ActionOwner、CameraInteraction、OwnedViewportIpc、ShopUndoUi、PieActionOwner、PlayerHudOwner）都改用它。
  - `ue_desktop.isolate_controls(list, dir)`：每個自動化 UE 啟動都要帶自己的 profile 目錄，空值會被拒絕。已套用到 ue_local_acceptance（`<work>/controls`）、moba_owned_viewport_ipc（`<evidence>/controls`）、moba_role_launch 的 autopilot／spotlight 席位（`<output>/controls-p<id>`）和 ue_shader_warmup（移除從其他席位複製來的旗標，改用自己的目錄）。真人玩家的啟動入口（run_ue、run_2player_ue、真人席位）照舊使用玩家自己的 profile。
  - 使用者的 profile 改之前先備份到 `target/session-ward/backup/Controls-1-profile-before-e139.json`，然後只還原被測試改壞的那一列（鍵盤 MoveForward 改回 W），其他自訂鍵位都沒動。
  - `moba_owned_viewport_ipc.lua` 本身也會在開始時備份玩家的 `Saved/Controls/1/profile.json`，結束時（即使是被中止）寫回去。兩次 IPC 被中止的時間點，玩家 profile 的 mtime 都剛好更新。現在改成只處理這一輪 evidence 底下的隔離 profile，使用者在跑測試期間自己改的鍵位，不會再被寫回的舊檔覆蓋（IPC contract 140 cases 通過）。
- **驗證**：
  - Lua：`ue_desktop_test` 6、IPC contract 140、local acceptance contract 69、role_launch、shader warmup 全部通過。
  - 影子專案：17 項 profile 相關測試（ControlProfile*、ActionOwner*、CameraInteraction*、ShopUndo*、PlayerHud*、ManagedNextMatch）都 Success。
  - IPC `e139-ipc-2`（只有 ue_desktop 隔離，還沒有 C++ 的路徑覆寫）：整輪 `OwnedViewportIpc` Success。
  - 完整修正後（影子專案含 `ControlProfilePath`，加上 runner 不再備份或寫回玩家 profile）重跑 IPC `e141-ipc-4`：整輪 `OwnedViewportIpc` Success（632 s），玩家的 `Saved/Controls/1/profile.json` mtime 沒有變。中間兩次超過 15 分鐘的 deadline（e140-ipc-1、e141-ipc-2），是因為其他 session 的對局和編譯同時在跑，每秒處理的 frame 從 67.7 降到 51.7，不是回歸。
- **注意**：
  - live 前端重建之前，舊 binaries 會忽略 `-om-control-profile-dir`，autopilot 對局仍會寫入 `Saved/Controls/1`（14:18 另一個 session 的對局就是這樣）。
  - lua_data 或 codegen 正在被其他 session 修改時，影子 stage 會失敗；這時只要稍後重跑即可。
- **避免**：
  - 自動化程序絕不能讀寫使用者的設定（控制、畫面、音量等）。設定路徑只從一個 helper 取得，自動化入口一律帶隔離目錄。
  - 會改鍵位的測試，要假設自己隨時可能被中止。

## E139 Tripo 建築：靜態模型沒有匯入流程、沒有陣營配色；`WorldUnitVisualScale` 把 saika 當紅色英雄；Tripo 帳號點數被別處用掉 1500（2026-10-27，Tripo 建築）

- **現象**：
  - 要把塔、主堡、抑制塔換成 Tripo 模型，但角色匯入流程只收有骨架的模型，`WorldModelsV1` 也只收 CC0 的 OBJ；靜態模型也沒辦法依陣營換色。
  - live 建置後，`Om.Runtime.WorldUnitVisualScale` 的「Original red hero is readable」失敗（紅色像素只有 64，門檻 200）。
  - 打開 Tripo 時點數是 5595，上次結束時是 7095；資產列表多了家具類模型。這不是本 session 花的。
- **根因**：
  - 匯入管線只為角色設計。
  - 那支測試和 E130 一樣，把 saika 預設成舊的紅色 b01 外觀；Tripo 版 saika 是黑甲，只有頭巾和圍巾是紅的。
- **通用修正**：
  - **靜態模型來源集 `tripo_static_v1`**：`omfue/SourceArt/Structures/TripoStaticV1/<id>/`。
    - 收件腳本加上 `--static`，並斷言模型沒有骨頭。
    - 匯入 commandlet 新增 `bStatic` 來源集：FBX 匯入成 StaticMesh（合併網格、不產生碰撞），只能產生一個 StaticMesh 加上材質和貼圖，回報高度和三角面數。
    - Lua 的 validate／verify 同步支援，契約測試從 76 項增加到 89 項。
  - **陣營配色**：
    - 上色工具改成通用：一個模型可以宣告多個變體 `recolors`，並支援多個來源集。靜態模型的變體放在 `TripoStaticVariantsV1/<id>/`，同一目錄可以放多個變體。
    - runtime 新增 `NativeStaticAllyMaterials`／`NativeStaticEnemyMaterials`，依單位相對本機玩家的陣營（Self 算友方；陣營未知時用模型原本的材質）切換材質，陣營改變時跟著換。
    - codegen 新增 `ally_material_sections`／`enemy_material_sections`，只有靜態模型可以用（會動的模型會被拒絕），測試共 90 項。
    - 三個建築的米白瓦片和旗幟（飽和度 ≤ 0.22、亮度 ≥ 0.62）在友方改成藍（色相 215），在敵方改成紅（色相 0），木頭和石頭不動。
  - **測試**：
    - `WorldUnitVisualScale` 在現有的建築檢查裡加上陣營切換：敵方顯示紅、自己和友方顯示藍、陣營未知時還原原本材質。
    - 「紅色英雄」改成量測英雄輪廓：畫面中心附近和地面顏色差異大、又不是回城光環的像素數，門檻 150，saika 實測 433。紅色像素數仍會記錄。
- **驗證**：
  - 影子建置後，`WorldUnitVisualScale`、`NativeHeroPoseSanity`、`NativeHeroPresentation` 全部通過。
  - 實機局 `target/structures-match-2`：友方建築用新模型、藍色屋頂，尺寸沿用原本的 240／260／110／210 world units。
  - 敵方紅色的遊戲內截圖沒拍到：autopilot 的英雄沒推到敵方建築；spotlight 把英雄放在敵塔旁會被塔打死，放在射程外，敵塔又不在鏡頭裡。紅色只由自動化測試和貼圖對照確認。
- **流程錯誤（live 半成品約 34 分鐘）**：
  - codegen（`omfue/codegen/src/lib.rs`）是直接在 live 修改的，13:28 就進了 live；但 runtime 標頭 `OmUnitActor.h` 和實作是先在影子專案寫完、建置後，14:02 才放回 live。
  - 中間這段時間，任何從 live 做 stage 建置的人（ward、lol-45）都在 `OmContentClasses.cpp` 編到 C2065。
  - 14:02 之後，從 live `--sync --build` 的影子建置已確認通過。
  - **規則**：codegen 會產生 C++ 程式碼，屬於 C++ 改動的一部分，要和 runtime 標頭一起在影子專案建置通過後，同一批放回 live（E076、E107、E109）。這次 codegen 是先改 live 再複製到影子，順序反了。
- **避免**：
  - 換掉某個單位的外觀後，搜尋測試裡用到該單位的地方（例如 `AOmHeroSaikaMagoichi`）是否假設了舊外觀，改成量測可見度或依來源檢查。
  - Tripo 帳號是共用的，每次開始前先記下點數，結束時核對。

## E140 主視角是正上方俯視：`BP_RtsCameraPawn` 覆寫了 pitch 和 SpringArm 旋轉（2026-10-27，camera-oblique）

- **現象**：實機對局截圖是正上方俯視，autopilot trace 的 `camera_pitch_yaw` 一直是 `[-90,0]`。但 C++ 的 `AOmRtsCameraPawn::CameraPitchDegrees` 預設是 -60，小地圖在沒有量測基底時的 fallback（`GetDefault<AOmRtsCameraPawn>()`）也假設是這個值。
- **根因**：
  - 正式使用的 `/Game/RustBP/System/BP_RtsCameraPawn` 覆寫了 `CameraPitchDegrees`，也覆寫了 SpringArm 元件模板的 `RelativeRotation`，兩個值都是 -90。owned Tick 每個 frame 都會用 BP 的 pitch 重設 arm。
  - Tick 在沒有 input owner 時會提早 return，所以 arm 會一直停在模板的 -90。
  - 測試用的 `EditorPreview` world 沒有初始化 actors，`PostInitializeComponents` 不會被呼叫，因此修正不能放在那裡。
- **通用修正**：
  - 鏡頭方向改成由程式定義，不再是 reflected UPROPERTY：`DefaultCameraPitchDegrees = -56`（LoL 式斜視，yaw 固定為 0）。BP 存的舊值在載入時會被忽略。要在執行期改方向，只能用 `SetCameraOrientation`。
  - 新增 `OnConstruction`，在 BP 元件模板和 construction script 之後把 arm 設成程式定義的方向。
  - 新增 `Om.Runtime.RtsCameraObliqueDefault`：載入正式 BP 並 spawn，檢查實際 view pitch 在 -60～-45 之間、yaw 為 0、畫面中心是焦點。
  - 已確認滑鼠射線、小地圖視錐框（逐角 deproject 到地面）、小地圖方向基底、名牌（E133 的 ScreenUp）、迷霧（世界空間格子）都不依賴 pitch；地圖 clamp 夾的是焦點，焦點仍在畫面中心。
- **驗證**：
  - 影子專案 `target/ue-shadow-camera` 中，RtsCameraObliqueDefault、NameplateAboveModelOnScreen、MinimapCameraOrientation、MinimapCameraFootprintClip、MinimapCameraRoutedInput、OwnedCameraInputStack、FloatingTextClearance 全部通過。
  - quick map 的 split autopilot 對局 `target/camera-oblique-match` 在 600 秒內分出勝負（隊伍 2 勝）。trace 的 122 筆 `camera_pitch_yaw` 都是 `[-56,0]`。截圖中塔有側面、名牌在模型上方、小地圖視錐框是梯形。
  - omfx-decouple 建議的 spotlight 取景檢查：`record_hero_spotlight.lua --hero date_masamune --no-build`（輸出在 `target/spotlight-camera-oblique`），8 張關鍵影格都沒有跳過。角色從斜上方看得到全身，名牌和浮動數字的位置都正確。
- **附帶發現**：
  - 第一次 sync 時，`omfue/codegen` 已經會產生 `NativeStaticAllyMaterials`／`NativeStaticEnemyMaterials`，但當時影子專案的標頭還沒有這兩個成員，UBT 報 C2065。Tripo session 表示對應的 OmUnitActor.h／.cpp 在 14:02 已進 live。這次只在自己的影子專案刪掉那幾行產生碼，沒有動 live。
  - 之後重新 sync 時，`templates.lua` 的輸出格式錯誤，codegen 失敗（invalid type: map, expected a sequence），應該是其他 session 正在修改 lua_data。
- **避免**：
  - 會影響全遊戲呈現的相機參數不要開放給 BP 覆寫；需要調整時改程式常數並加測試。
  - 驗收鏡頭時要看實機 trace 的 `camera_pitch_yaw`，不能只看 C++ 預設值。
  - 用 `EditorPreview` world 測試 actor 初始化時，記得 `PostInitializeComponents` 不會被呼叫。

## E138 打包版完全不顯示血量文字和浮動數字：`UnlitText` 只用 runtime 路徑載入，沒被 cook（2026-10-27，session-ward）

- **現象**：next-month P2 #20 一直留著「UnlitText 載入失敗」的警告。打包版的 log 出現 `SkipPackage: /Engine/EngineMaterials/UnlitText … does not exist`，接著是 `Public health text requires Engine UnlitText material`。`ShowFloatingText` 因此直接 return false，打包版看不到任何傷害或治療數字。之前紀錄寫「名牌照常顯示」，把這個問題的嚴重性看輕了。
- **根因**：`UOmCombatVisualSubsystem` 只用 `LoadObject(TEXT("/Engine/EngineMaterials/UnlitText…"))` 在執行期載入。沒有任何 cook 進去的 package 參考它，所以 cooker 不會帶上它。
  - 第一版修正把它加進 `DefaultEngine.ini [Engine.StartupPackages]`，結果無效：用 `UnrealPak -List` 檢查 package-shipping-5 的 utoc，裡面沒有 UnlitText，連引擎自己列在 StartupPackages 的 `DefaultTextMaterialTranslucent` 也沒有。這一版已撤回。
  - 但被原生 constructor 用 `ConstructorHelpers` 載入的引擎資產都有被 cook，例如 `DefaultTextMaterialOpaque`（來自 TextRenderComponent）、`Widget3DPassThrough_Translucent`（來自 WidgetComponent），以及我們自己的 `BasicShapes/*`。
- **通用修正**：
  - `UOmCombatVisualSubsystem` 新增 constructor，用 `ConstructorHelpers::FObjectFinder` 參考 UnlitText；runtime 的 `LoadObject` 流程不變。已在影子專案建置並通過 PublicHealthTextLifecycle、FloatingTextClearance、PresentationCueSlots，並已 3-way 合併進 live（omfx-decouple 確認沒有衝突）。
  - `ue_cook_coverage_test.lua` 新增規則：原始碼裡所有只用路徑載入的 `/Engine/...` 資產（`LoadObject`、`FSoftObjectPath`），都必須另外有 constructor 參考；引擎自己的 constructor 已經載入的資產列在白名單裡，並註明是哪個 class。修正前跑這個測試會抓到 UnlitText，修正後通過。
- **驗證（live 重建後）**：Development 打包 `target/session-ward/package-dev-e138`（50 個檔案）。`UnrealPak -List` 列出 `Engine/Content/EngineMaterials/UnlitText.uasset`（98849 bytes）。用打包版跑 6 分鐘冒煙對局（`target/mvp-a3/pkg-e138-smoke`，p1 date_masamune）：兩個 client 都沒有 UnlitText 的 SkipPackage 或 warning；`p1/autopilot-p1-shot-004-combat-text.png`、`shot-006` 可以看到浮動的回復數字（+2、+6、+6.1）。
- **profiler DLL**：`aqProf.dll`、`VtuneApi*.dll` 的 `Failed to load (126)` 是 UE 在 Windows 上探測選用 profiler 的正常訊息，不影響執行，不需要處理。
- **避免**：只在執行期用路徑載入的資產（不論 `/Game` 或 `/Engine`），一定要同時有 cook 的來源。`/Game` 用 DirectoriesToAlwaysCook；`/Engine` 用 constructor 參考。兩種都由 cook coverage 測試把關。

## E137 音訊驗收用字串比對 UE 回報的錄音路徑，執行副本經 junction 回報的路徑對不上；修改 lua-host 原始碼讓所有 Lua 流程失敗（2026-10-27，omfx-decouple）

- **現象一**：`live-build2-null-1` 第 11 批 `Om.Runtime.OriginalAudioOutput` 在 `verify_original_audio_output.lua:48` 失敗，錯誤為 `unexpected recording path or duplicate`。
- **根因一**：
  - E128 把音訊驗收改成從執行副本啟動 UE。UE 在 `output.json` 回報的錄音檔路徑是透過副本的 junction：`D:/omoba/target/ue-run/0d7646eb996e1827/omfue/Saved/Automation/OriginalAudio/.../routed-mix.wav`。
  - 驗收拿這個字串和 live 路徑 `omfue/Saved/...` 做字串比對；兩者指向同一個檔案，字串卻不同。
- **修正一**：
  - 新增 `ue_run_snapshot.live_path(root, file)`。副本中經 junction 連到 live 的項目都維持原本的相對位置，所以把 `target/ue-run/<key>/` 去掉，就是 live 的真實路徑。
  - 副本自己複製的 binaries（Binaries、Intermediate、外掛 Binaries）沒有對應的 live 檔案，原樣傳回；副本以外的路徑也原樣傳回。
  - 音訊驗收比對前先換成 live 路徑。
  - 盤點其他字串比對：`moba_owned_viewport_ipc` 和 `moba_renderer_fixture` 的 `same_path` 只用來判斷入口腳本，`ue_local_acceptance` 的報告欄位不比對 UE 回報的路徑。目前只有音訊驗收需要改。
- **現象二（本次修正途中由我造成）**：我為了加一個解析 junction 的 `real_path` 操作而修改 lua-host 原始碼，之後所有 Lua 流程都停在 `host.lua:42: failed to build omoba-lua-host`，我自己的 spotlight 重錄也在 omatsu 中斷。其他 session 的流程同一時段也可能受到影響。
- **根因二**：
  - `host.lua` 發現原始碼比 exe 新，就執行 `cargo build`；但其他流程的長駐 host（12:51、12:56 啟動的 workflow supervisor）正從 `tools/lua-host/target/debug/omoba-lua-host.exe` 執行，exe 被鎖（os error 5），建置失敗後直接 assert。
  - 這和 E125 是同一類問題：直接執行 build 輸出的檔案。
- **修正二**：
  - `host.lua` 一律從 `tools/lua-host/target/run/<size>-<mtime>/` 的副本執行 host，build 輸出因此永遠不會被鎖，下次原始碼更新就能順利重建。
  - 如果重建失敗但先前已有建好的 host，沿用舊的並提示，不再讓所有流程失敗；只有用到新操作時才會失敗。
  - `real_path` 已改用純 Lua 的 `live_path` 取代，所以從 lua-host 移除，沒有留下沒用到的程式碼。
  - 那兩個仍從 target/debug 執行的 supervisor 結束後，下一次呼叫就會重建並改用副本。
- **驗證**：
  - `ue_run_snapshot_test.lua` 增加到 32 項：Saved 換回 live、大小寫與分隔符號、binaries 不換、live 路徑不變。
  - host 在 exe 被鎖時實測：印出 `rebuild failed; using the previous host build` 後照常回應，副本建在 `target/run/2680832-1793073858`。
  - 單獨重跑音訊驗收：`verify_original_audio_output.lua --run-id e137-audio-1`（live binaries，從執行副本啟動）通過，5 段實際的 realtime mixer PCM 全部驗證成功，exit 0。
- **避免**：
  - 從執行副本啟動的 UE 回報的路徑，比對前一律用 `live_path` 換成 live 路徑。
  - 任何會被長駐程序執行的工具（lua-host、遊戲 exe）都要從副本執行，build 失敗時要能沿用既有版本；修改共用工具的原始碼前，要先確認不會讓正在執行的流程失敗。

## E136 影子專案同步沿用 live 的 mtime，UBT 沒有重編較早落地的改動，連結失敗（2026-10-27，session-ward）

- **現象**：影子專案 `--sync --build` 後連結失敗，錯誤是 `LNK2019 UOmCombatVisualSubsystem::RegisterNameplateUnit`。這個函式在 live 和影子的 .cpp 裡都有定義。
- **根因**：`stage_ue_shadow.lua` 的 `copy_file` 會把複本的 mtime 設成來源的 mtime；mirror 也用「mtime 不同」判斷是否複製。
  - 另一個 session 12:13 在 live 改了 `OmCombatVisualSubsystem.cpp`，影子專案 12:2x 才同步進來。
  - 複本的 mtime 是 12:13，比影子專案上次建置的 .obj 還舊，所以 UBT 認為不用重編，保留了舊的 obj。
  - 而 `OmUnitActor.cpp` 是新近修改，有重編，開始呼叫這個新函式，連結就找不到。
- **通用修正**：
  - `copy_file` 把複本的 mtime 設成 `max(來源 mtime, 現在)`，不會落在過去。
  - mirror 改成「沒有複本、大小不同，或來源比複本新」時才複製，所以已經同步過的複本不會每次重抄。
  - `ue_shadow_stage_test.lua` 新增情境：來源時間比影子專案還舊時仍會複製，複本時間 ≥ 同步當下，再同步一次不會重抄（39 checks）。
  - 既有影子專案（session-ward 的 ueproj）裡今天同步進來的檔案已經手動 touch，讓 UBT 重新編譯。
- **避免**：
  - 給建置工具吃的鏡像不能保留來源的 mtime。UBT、cargo 這類工具只看「輸入是否比輸出新」，而別的 session 的改動，時間可能早於你上次建置。
  - 看到「定義明明在、卻 unresolved external」的錯誤時，先懷疑 .obj 是舊的。

## E135 鍵盤移動方案從舊設定檔載入時，Stop 和額外技能都用 X，整份控制設定被拒（2026-10-27，session-ward）

- **現象**：live 建置後的 OwnedViewportIpc（`live-tripo-null-4-ipc`）出現 `OM_CONTROL_PROFILE player=1 code=duplicate-chord message=Stop primary (pressed) and CastExtra primary (pressed) both use X in worldCombat`，整份設定檔被當成錯誤，只能用記憶體裡的預設值。
- **預設鍵位本身沒有衝突**（照 LoL 慣例）：
  - 點擊移動方案（scheme 0）：Stop = S，額外技能 CastExtra = X。
  - 鍵盤移動方案（scheme 1）：S 已經是「向後移動」，所以 Stop = X，額外技能預設不綁定，由玩家自己選。
- **根因**：`FOmControlProfileSet::ImportJson` 讀 `om.player-control-profile-set` v1 時，各方案的 patch 原樣保留。
  - 額外技能欄位出現之前存的檔案，global 層沒有任何 `*Extra` 列。global 匯入時的遷移（`ReleaseOccupiedDefault`）因此補上 CastExtra = X。
  - 但鍵盤 patch 是舊的，裡面有 Stop = X，卻沒有「CastExtra 不綁定」這一列。
  - 鍵盤方案於是繼承了 global 新補的 X，跟 Stop 撞鍵，Validate 失敗。
- **通用修正**（`OmControlProfileSet.cpp` 的 `SeedNewKeyboardActions`）：
  - 判斷依據：存檔裡 global 層完全沒出現過的 action，代表它是存檔之後才新增的，鍵盤 patch 對它也不可能有設定。
  - 這類 action 在鍵盤 patch 裡改用工廠預設的鍵盤對應補齊。
  - 玩家在舊檔裡自己改過的鍵盤設定保持不變。
  - 以後再新增 action 也適用，不用一個一個寫遷移。
- **測試**：新增 `Om.Runtime.ControlProfileSetUniqueChords`。
  - 工廠預設的兩個方案都不能有 duplicate-chord，並逐項檢查鍵位：點擊方案 Stop=S、CastExtra=X；鍵盤方案 Stop=X、向後=S、CastExtra 不綁定。
  - 模擬額外技能欄位出現前存的檔案（去掉所有 `*Extra` 列），必須能載入、不撞鍵、保留玩家原本的鍵盤設定，而且能完整 round-trip。
- **驗證**（全部 live 程式碼同步進影子專案並重編，`-d3d12 -RenderOffScreen`，log 在 `target/mvp-a3/ue-tests-ui4.log`）：HudHintFollowsRoutedPointer、NameplateAboveModelOnScreen、ReadableHudToolTip、ManagedNextMatch、ManagedNextMatchCancellation、ControlProfileSet、ControlProfileSetPersistence、ControlProfileSetUniqueChords、PublicHealthTextLifecycle 全部 Success。nullrhi（`ue-tests-ui2-null.log`）：ManagedNextMatch 以 warning 跳過畫面段並通過，HudHint 以 warning 跳過。
- **避免**：
  - 新增任何會補預設鍵位的 action 時，鍵盤方案必須透過 `SeedKeyboard` 的保留表處理，不能只改 global 的遷移。
  - 任何可能衝突的預設值，都要讓 `ControlProfileSetUniqueChords` 涵蓋。

## E134 `ManagedNextMatch` 的「真實指標點到下一局按鈕」在 nullrhi 下根本無法成立，E127 的推測是錯的（2026-10-27，session-ward）

- **現象**：`Om.Runtime.ManagedNextMatch` 失敗在 `Actual pointer hits next-match action`。
  - live-tripo-null-1 只加 `-NullRHI` 時失敗，live-tripo-null-4 加了 `-NullRHI -RenderOffScreen` 也一樣失敗。
  - 所以不是使用者桌面干擾。
- **更正 E127**：
  - 查遍 `Saved/Automation/LocalAcceptance/*/native-*-engine.log`，這項測試只在 d3d12 下通過過：10-21 的十多次全部是 `-d3d12`，今天的 `offscreen-rhi-1` 是 `-d3d12 -RenderOffScreen`。
  - 它在 nullrhi 下從來沒有通過過。E127 說「先前在 nullrhi 下 302/302 都通過過」，是把 d3d12 套件誤認成 nullrhi。
  - E127 給全部套件加 `-RenderOffScreen` 這個修正本身仍然正確，但它不是這個失敗的原因。
- **根因**：`-NullRHI` 下 `FApp::CanEverRender()` 為 false，`FSlateApplication::PrivateDrawWindows` 會直接 return（`Slate.SkipWidgetDrawingInHeadlessMode`）。
  - 所有 widget 的幾何都是 0，hit-test grid 是空的，所以任何點擊判定都不可能成立。
  - 結果面板的 `Contains` 檢查也因為全是 0 而「空洞地」通過了。
  - 並沒有任何元件擋住按鈕：
    - 浮動數字、名牌：world-space 或 viewport 層，不在結果面板的 widget 樹裡。
    - 合併後的 tooltip：測試裡沒有開 hint。
    - loading 畫面：這個 fixture 沒有掛。
  - 實際上，同一份 live 程式碼在 d3d12 off-screen 下這項測試會通過（見本條驗證）。
- **通用修正**（`OmPlayerHudOwnerTests.cpp`）：
  - 沒有 rendering RHI 時，結果面板幾何、hit-test 和 Slate 點擊這幾段都跳過並發出 warning。
  - ready 請求改從按鈕自己用的 session gate（`RequestReadyNextMatch`）送出，所以 Job、檔案和 owner 退役流程在 nullrhi 下仍然有驗到。
  - 有 RHI 時，點擊判定改用 `OmLocateHit`（只看該視窗自己的 grid）。失敗訊息會帶出實際點到的 widget，以後如果真的有東西擋住，可以直接看出是誰。
- **驗證**（全部 live 程式碼同步進影子專案並重編，`-d3d12 -RenderOffScreen`，log 在 `target/mvp-a3/ue-tests-ui4.log`）：HudHintFollowsRoutedPointer、NameplateAboveModelOnScreen、ReadableHudToolTip、ManagedNextMatch、ManagedNextMatchCancellation、ControlProfileSet、ControlProfileSetPersistence、ControlProfileSetUniqueChords、PublicHealthTextLifecycle 全部 Success。nullrhi（`ue-tests-ui2-null.log`）：ManagedNextMatch 以 warning 跳過畫面段並通過，HudHint 以 warning 跳過。
- **避免**：只要測試會碰到 widget 幾何、hit-test 或截圖，第一步就要分清楚有沒有 rendering RHI（`OmSkipWithoutRendering` 或 `FApp::CanEverRender()`）。不要在 nullrhi 下斷言畫面上的東西。

## E133 名牌蓋住 Tripo 英雄上半身：偏移量沿世界 Z，俯視鏡頭下這剛好是視線方向（2026-10-27，session-ward）

- **現象**：一般對局和 spotlight 裡，名牌疊在 Tripo 英雄的胸口和頭上。
- **根因**：`AOmUnitActor::GetNameplateWorldAnchor` 把錨點放在 bounds 原點，再沿世界 Z 往上抬。
  - MOBA 俯視鏡頭的 pitch 約 -80°，世界 Z 幾乎就是視線方向，抬得再高，投影到畫面上也只移動一點點。
  - 模型越高（Tripo 身體），名牌就越深入模型裡。
- **通用修正**：錨點改沿鏡頭的 screen-up（`OmScreenFacing::ScreenUp`，本機視角）。
  - 位移量 = bounds 在 screen-up 方向的實際半高 `|Ex·Ux|+|Ey·Uy|+|Ez·Uz|` 再加 12 cm。
  - 高度由模型 bounds 推出，不寫死。任何 yaw、俯視或側視都適用。沒有本機視角時退回世界 Z。
- **測試**：新增 `Om.Runtime.NameplateAboveModelOnScreen`。
  - 情境：實際生成的英雄，俯視 yaw 0/90/37 加上側視。
  - 斷言：錨點在 screen-up 上位於模型頂端 +12 cm（±0.1），在 screen-right 上居中。
- **驗證**（全部 live 程式碼同步進影子專案並重編，`-d3d12 -RenderOffScreen`，log 在 `target/mvp-a3/ue-tests-ui4.log`）：HudHintFollowsRoutedPointer、NameplateAboveModelOnScreen、ReadableHudToolTip、ManagedNextMatch、ManagedNextMatchCancellation、ControlProfileSet、ControlProfileSetPersistence、ControlProfileSetUniqueChords、PublicHealthTextLifecycle 全部 Success。nullrhi（`ue-tests-ui2-null.log`）：ManagedNextMatch 以 warning 跳過畫面段並通過，HudHint 以 warning 跳過。
- **避免**：任何放在模型「上方」的畫面元素（名牌、血條、提示），都要用畫面方向和模型 bounds 計算，不要用世界 Z 加常數。

## E132 回城提示卡在左上角不關：hint 的來源元件是用 OS 游標找的（2026-10-27，session-ward）

- **現象**：autopilot 或 off-screen 測試 hover 回城按鈕後，提示出現在左上角而且一直不關。真實玩家在 app 失焦時也會殘留。
- **根因**：
  - `SOmHudToolTip::OnOpening` 用 `GetCursorPos()`（OS 游標）去找開出 hint 的來源元件。autopilot 和 off-screen 的合成輸入只送 Slate pointer event，不會移動 OS 游標，所以找不到來源。
  - 找不到來源時，關閉與否完全交給引擎的 `UpdateToolTip`。但引擎只在 app active 或 OS 游標在 Slate 視窗上時才檢查，所以 hint 永遠不關。
  - 位置也是用 OS 游標算的：off-screen 下游標是 (0,0)，提示就跑到左上角。
  - 補充：cursor user 的 `GetCursorPosition()` 回傳的也是平台游標，所以不能拿它當「Slate 實際送出的 pointer 位置」。
- **通用修正**：
  - 來源改從 Slate 最後一次送出 pointer event 的路徑取得（`FSlateUser::GetLastWidgetsUnderCursor()`，也就是引擎 tooltip 更新本身用的路徑）。真實或合成的 pointer 都適用，找不到時才退回 OS 游標。
  - `SOmHudToolTipHost::Tick`：來源和 hover anchor 都不再被 hover 時，就關閉 hint（可閱讀捲動的 hint 在 hover 時除外）。不依賴引擎是否檢查。
  - 定位：OS 游標不在來源元件上時（合成輸入），hint 開在來源元件旁邊。
- **測試**：新增 `Om.Runtime.HudHintFollowsRoutedPointer`（需要 RHI，加 `-RenderOffScreen`，只移動 null cursor）。
  - 合成 pointer 情境：hint 開在來源旁邊，不在游標或視窗角落；pointer 離開後就關閉。
  - 真實游標情境：hint 開在游標旁邊；離開後就關閉。
- **驗證**（全部 live 程式碼同步進影子專案並重編，`-d3d12 -RenderOffScreen`，log 在 `target/mvp-a3/ue-tests-ui4.log`）：HudHintFollowsRoutedPointer、NameplateAboveModelOnScreen、ReadableHudToolTip、ManagedNextMatch、ManagedNextMatchCancellation、ControlProfileSet、ControlProfileSetPersistence、ControlProfileSetUniqueChords、PublicHealthTextLifecycle 全部 Success。nullrhi（`ue-tests-ui2-null.log`）：ManagedNextMatch 以 warning 跳過畫面段並通過，HudHint 以 warning 跳過。
- **實戰證據**：tripo-heroes 的 split run（`target/tripo-live-match-3/p1/autopilot-p1-shot-007-combat-text.png` 等）中，提示框就在回城[B] 按鈕正下方，從第 4～5 張截圖開始一直到比賽結束都沒關。
- **避免**：HUD 互動邏輯只能依賴 Slate 實際送出的 event 和 hover 狀態，不要依賴 OS 游標位置。autopilot 從不移動使用者的游標（E114）。

## E131 owned-viewport IPC 的 fixture 一直沒連上：bridge 拒絕 content 較新的 fixture，preflight 沒擋（2026-10-27，live-tripo-null-4）

- **現象**：`live-tripo-null-4` 的 IPC 階段失敗：`fixture frame was not ready: connected=0 fps=0 catalog=1 frame=33 player=1 hero=0 phase=0`。時間點緊接在 E128 改成從副本執行 fixture 之後，因此一度懷疑是副本造成的。
- **根因**：
  - 不是副本造成的。fixture 正常啟動，startup.json 完整，也是從 `target/run-bin` 的副本執行。
  - UE 的 `ue.stderr.log` 不斷出現 `om presentation runtime-ready rejected: runtime content=30724da96c6528cc renderer content=53083805ccf07bf7`。
  - `moba_owned_viewport_ipc` 每次執行都會用目前的 lua_data 重建 fixture（`fixture.build()`）；live bridge 卻只在前端建置時更新（11:33）。之後 lua_data（Tripo 英雄）又改了，所以兩邊內容不一致。
  - 這和 E125 是同一類問題，但 E125 的 content 檢查只接在 `moba_role_launch`，IPC 路徑沒有，結果只能等到 frame timeout，才得到一個看不出原因的錯誤。
- **通用修正**：
  - `moba_owned_viewport_ipc.lua` 在 fixture 啟動後、UE 啟動前，比對 fixture 的 `startup.content_hash` 與執行副本中 bridge 內嵌的 `OM_CONTENT_CATALOG_DATA_HASH`（`run_artifacts.content_hash`）。
  - 不一致時立即報錯，並說明解法：重建前端，或使用相符的影子建置。
  - 兩邊的 hash 都會寫進報告（`live_content_hash`、`bridge_content_hash`）。
- **驗證**：
  - `owned_viewport_ipc_contract_test.lua` 140 cases 通過。
  - 用 `stage_ue_shadow.lua --sync --build` 依目前的 lua_data 重建影子專案，bridge 的 marker 為 `30724da9…`。
  - `OM_UE_RUN_BINARIES=影子` 單獨執行 `moba_owned_viewport_ipc.lua --run-id e130-ipc-1`：`Om.Runtime.OwnedViewportIpc` 成功，success=true、failed=0、not_run=0，沒有任何 rejected，fixture 也是從 `target/run-bin` 執行。
  - live 的 IPC 要等 live 前端用目前的 lua_data 重建（`build_ue_moba`）後才會通過；在那之前，新的 preflight 會直接報出原因。
- **避免**：
  - 凡是會在執行期比對 content hash 的兩端（server、runtime、fixture 對 bridge），每一個啟動入口都要在啟動 UE 前比對 marker，不能只接在其中一條路徑。
  - 看到 `entities=0` 或 `connected=0` 時，先查 UE stderr 裡的 `runtime-ready rejected`。

## E130 saika 換成 Tripo 身體後，`Om.Generated.NativeHeroPresentation` 仍要求舊 recipe 材質（2026-10-27，Tripo 材質 namespace）

- **現象**：live 建置後的 nullrhi 全量測試（`omfue/Saved/Automation/LocalAcceptance/live-tripo-null-1`，native-001）失敗：`Expected 'Material uses recipe namespace' to be true.`。這次 run 裡英雄相關只有這一個斷言失敗（native-005 的 ManagedNextMatch 是 E127）。
- **根因**：這支測試和 E121 的 `AuthoredHeroAnimationLifecycle` 一樣，寫死了 saika 只會有舊 b01 recipe 外觀（`RecipeV1/Materials/M_saika_magoichi`）。Tripo 身體用的是 FBX 內嵌貼圖匯入的材質（`TripoV1/<hero>/tripo_mat_*`），阿市則用上色變體（`TripoVariantsV1/oichi/`）。
- **判斷：Tripo 材質是合理的例外，不改匯入流程**：
  - RecipeV1 材質是 `ue_apply_asset_recipe.lua` 透過 MCP，用舊模型的貼圖為每位英雄各做一個「貼圖→材質」，沒有共同的 master material、outline 或遠鏡頭可讀性處理。本質和 Tripo FBX 內嵌的材質相同。
  - MaterialsV1（`character_materials.lua`）是 CC0 Quaternius 人形專用的貼圖組。
  - 讓 Tripo 改走 recipe 管線不會多得到任何東西，反而會多出一份重複的貼圖。
  - 遠鏡頭的配色問題，已經改由 manifest 的 `recolor` 變體在建置期處理（阿市，見 ledger）。
- **通用修正**：
  - 測試依身體來源檢查各自的 namespace，而不是放寬：舊 recipe 身體照舊要求 RecipeV1；`TripoV1/<hero>/` 身體只接受自己匯入的材質（`TripoV1/<hero>/`）或宣告的變體（`TripoVariantsV1/<hero>/`）。
  - `Om.Runtime.NativeHeroPoseSanity` 也對所有 Tripo 英雄做同樣的檢查，並記錄每個 slot 的材質路徑（8/8：阿市是 `oichi_mat_sakura`，其餘是各自的 `tripo_mat_*`）。
- **驗證**：影子專案建置後，`NativeHeroPresentation`、`NativeHeroPoseSanity`、`AuthoredHeroAnimationLifecycle` 都 Success（`.ai-collab/tripo-material-ns-1`）。兩個檔案確認與影子一致後放回 live（CRLF）。
- **避免**：更換某位英雄的身體來源時，用 `grep` 找出寫死該英雄資產路徑的測試（`RecipeV1`、`M_<hero>`、`b01_`），一起改成依來源檢查。

## E129 英雄和敵人重疊時，傷害數字被另一個單位的名牌擋住（2026-10-27，英雄 spotlight）

- **現象**：Tripo 版 spotlight 中，伊達 Q（踏火無痕）衝到敵人身上、村上突進時，「-510.2」「-325」被疊在一起的兩個名牌（敵人的在上，自己的在下）蓋住。
- **根因**：
  - 名牌是 UMG widget，永遠畫在世界空間的 `UTextRenderComponent` 數字之上，調整繪製層級無法解決。
  - E123 的 `ClearNameplate` 只避開數字所屬單位自己的名牌（`Owner->GetNameplateDesktopBounds`），而且只看數字中心點的 X 是否落在名牌範圍內，沒考慮數字的寬度。
  - 兩個單位重疊時，數字被推到自己名牌上緣，正好落在另一個單位的名牌底下。
- **通用修正**（一般對局和 spotlight 共用同一條路徑）：
  - 新增 `OmRuntime/Public/OmFloatingTextClearance.h`：純螢幕空間計算 `Lift(Base, HalfWidth, HalfHeight, Plates, Current)`，逐輪把數字推到所有仍重疊的名牌之上，疊在一起的名牌會一層層越過，最多 16 輪。數字的寬度也列入判斷，名牌順序不影響結果，已經上推的距離不會變小。
  - `UOmCombatVisualSubsystem` 新增 `RegisterNameplateUnit`；`AOmUnitActor::EnsureNameplateWidget` 建立名牌時會註冊（只加 4 行）。
  - 每個 tick 只有在有數字顯示時，才把可見名牌轉成 viewport 像素矩形（`GatherNameplatePixels`，畫面外的略過，失效的單位移除），再交給 `ClearNameplate` 一起避讓。沒有數字時不做任何事，不影響壓力場景的熱路徑。
- **驗證**：
  - 新測試 `Om.Runtime.FloatingTextClearance`：
    - 先重現舊行為：只避開自己的名牌時，數字仍在敵人名牌底下；
    - 新行為下數字清開所有名牌，名牌順序不影響結果，數字的邊緣碰到名牌也會上推；
    - 遠處的名牌、上方很遠的名牌都不影響數字；已經上推的距離不會變小；無效矩形和 NaN 都不影響結果；
    - 15 層疊在一起的名牌全部清開。
  - 第一版的重疊判斷含邊界（`>=`），結果把剛好上推到邊距外的位置也算成重疊；改成嚴格比較後才通過。
  - 影子建置中，`FloatingTextClearance`、`PublicHealthTextLifecycle`、`HeroNameplatePlacement` 都通過。
  - 影子 binaries 重錄伊達與村上：伊達 Q 的「-400／-110.2」、村上的「-200／-102.6」都在兩個疊在一起的名牌之上。
  - 原始碼以 3-way merge 合併回 live；live 在這段期間沒有變動，合併後與影子建置完全相同。live binaries 尚未用 UBT 重建，會在下一次 `build_ue_moba` 生效。
- **避免**：世界空間的提示文字要避開螢幕空間 UI 時，必須考慮所有可見的 UI 矩形，不能只看自己的；同時要用文字的實際寬度判斷，而不是只看中心點。

## E128 E124 漏掉的入口：ue_local_acceptance、原始音訊驗收仍檢查共用 base_content；renderer fixture 直接執行 target 裡的 exe（2026-10-27，omfx-decouple）

- **現象**：`live-tripo-null-2` 的 `ue_local_acceptance.lua` 又因為「staged base_content differs from current build」被擋下。
- **根因**：E124 只改了 `moba_role_launch` 的路徑。下面這些入口仍有舊問題：
  - `ue_local_acceptance.lua` 第 49 行仍呼叫 `moba_stage_contract.verify(bindings(root))`，檢查共用的 `scripts/base_content.dll` 和 plugin 副本是否等於最新的 debug build。
  - `verify_original_audio_output.lua` 除了同樣的檢查，還直接用 live `omfue/om.uproject` 啟動 UE，違反 E116。
  - `moba_renderer_fixture.lua`（owned-viewport IPC 用）直接執行 `omoba-client-runtime/target/debug/omoba-renderer-fixture.exe`，執行期間鎖住它（E125 的同一類問題）。
- **通用修正**：
  - **ue_local_acceptance**：
    - 改用 `ue_bridge_stage.verify_launch`，只比對 bridge 和它的 stage record。
    - base_content 用 `run_artifacts.stage_base_content` 複製到 `target/ue-local-acceptance/<run_id>/`。
    - UE 執行副本用 `ue_run_snapshot.prepare(root, nil, {base_content=...})` 載入這份。
  - **verify_original_audio_output**：同上；UE 改從執行副本（`snapshot.uproject` / `project_dir`）啟動。
  - **moba_renderer_fixture**：
    - 來源改走 `run_artifacts.rust_binary`（遵守 `OM_RUST_RUN_BINARIES`）；
    - `spawn_fixture` 先用 `stage_binaries` 複製到 `target/run-bin/<key>/`，再從副本啟動。
  - **全面盤點**：
    - 仍用 `moba_stage_contract.bindings` 的只剩建置流程（`build_ue_moba` build 模式、`hero_author` 的 build 驗證），它們檢查的是剛 stage 的東西，屬於正確用法。
    - `run_ue.lua` 的舊 TD 模式（非 MOBA）仍讀共用 `scripts/`，MOBA 模式已轉到 `run_moba_session_ue`。
    - `run_2player*.lua`、`run_moba_headless_batch.lua`、`run_moba_runtime_smoke.lua` 直接讀 `target/<profile>`，但不做共用副本檢查。這些列為後續工作。
- **驗證**：
  - `ue_local_acceptance_contract_test.lua` 69 cases、`owned_viewport_ipc_contract_test.lua` 140 cases 通過。
  - `renderer_spawn_lifetime_test.lua --run-spawn-lifetime` 實際啟動 fixture 並通過，fixture 是從 `target/run-bin` 的副本執行。
  - 所有修改的 Lua 檔都能載入。
- **避免**：新的驗收或啟動入口一律走 `ue_bridge_stage.verify_launch`、`run_artifacts.stage_base_content` / `stage_binaries`、`ue_run_snapshot`；不可再用 `moba_stage_contract.bindings` 做 launch 前檢查。

## E127 nullrhi 全量測試的真實游標點擊判定受使用者桌面影響，`ManagedNextMatch` 偶發失敗（2026-10-27，main）

- **現象**：live 建置後跑 nullrhi 全量 Om.（live-tripo-null-1），`Om.Runtime.ManagedNextMatch` 失敗在 `Actual pointer hits next-match action`。同一項測試先前在 nullrhi 下 302/302 都通過過。
- **根因**：E114 只對 d3d12 套件加了 `-RenderOffScreen`。只加 `-NullRHI` 時，UE 仍使用 Windows platform application，`LocateWindowUnderMouse` 會先問 OS「目前游標下是哪個視窗」，所以結果取決於使用者桌面上開著什麼。測試碼裡還有 20 多處直接用真實桌面做點擊判定，沒有改用 `OmLocateHit`。
- **通用修正**：`ue_local_acceptance` 對所有套件（含 nullrhi）都加 `-RenderOffScreen`，改用 null platform application，`GetWindowUnderCursor` 一律回空，判定退回 Slate 自己的視窗順序，結果固定，也不會碰到使用者桌面。不逐一修改測試，也沒有把未編譯的 C++ 放進 live（E076）。
- **避免**：任何自動化 UE 程序，不論 RHI，都要經過 `ue_desktop.append`。之後新寫的點擊測試一律用 `OmLocateHit`。
- **更正（2026-10-27，session-ward，見 E134）**：`ManagedNextMatch` 失敗的原因不是使用者桌面。這項測試從來沒有在 nullrhi 下通過過（先前通過的都是 d3d12）。nullrhi 下 Slate 不會繪製，hit-test grid 是空的；加 `-RenderOffScreen` 也一樣會失敗。本條的 `-RenderOffScreen` 修正本身仍然正確。

## E126 其餘 7 位 Tripo 英雄：離線截圖畫出上一個姿勢；omatsu 的 idle 不合格；Tripo 頁面設定會被重設（2026-10-27，Tripo 英雄批次）

- **現象**：
  - `Om.Runtime.NativeHeroPoseSanity` 和 `Om.Editor.PoseCapture` 的截圖，常常畫出上一支動畫，甚至是更早的姿勢。量測數據顯示 attack 和 idle 差 0.3–0.6 個身高，但兩張圖一模一樣；loop 的 move 有時正確。
  - omatsu 沒通過英雄姿勢檢查：`Spell_Simple_Idle_Loop` 有一隻手腕高過肩膀（+0.016），attack 和 idle 只差 0.025。
  - Tripo studio：每次進頁面，AI 模型都會回到 H3.1（40 點），綁定的骨骼預設也會回到 Mixamo。有一次按「生成」沒有送出（沒扣點），重新上傳後才成功。
  - 多英雄同框的遊戲內錄影，和標準的 saika spotlight 一樣，都停在 loading 4/6、`entities=0`。原因是 E124 提到的 bridge（10:56）和 client runtime（10:28 release）不是同一批建置。runtime exe 正被其他 session 的 run 使用，沒辦法重建。已解決：E125 的流程改成每局從 exe 副本執行之後，重建 omobab 和 client runtime（release），用 `stage_ue_shadow.lua --sync --build` 重新 stage 影子專案，lineup 就錄成功了（`target/spotlight-lineup/saika_magoichi/`）。
- **根因**：
  - 不在 world tick 裡時，`RefreshBoneTransforms` 只更新可編輯的骨架緩衝，渲染端還是舊的 skinning。`SendAllEndOfFrameUpdates`、`MarkRenderDynamicDataDirty` 都不夠。saika 專用的 authored 測試之所以截得正確，是因為它會讀 `GetCPUSkinnedVertices`，順帶把姿勢推到渲染端。
  - 動畫庫的施法 idle 本來就舉著手，用在 idle 上就分不出施法和待機。
  - Tripo 只記住幾何與紋理設定（2K、PBR、拓撲、面數），不記 AI 模型版本和骨骼預設。
- **通用修正**：
  - `Om.Editor.PoseCapture` 在 FK 之後呼叫 `GetCPUSkinnedVertices`，截圖和量測現在一致。`NativeHeroPoseSanity` 拿掉截圖功能，只做量測，避免產生會誤導的證據；圖改由 PoseCapture 產生，一律加一支 A_TPose 當暖身。
  - omatsu 的 idle 改成 Idle_Loop。這是英雄姿勢檢查第一次實際擋下不合格的選擇。
  - Tripo 操作守則（寫在共用筆記 `tripo-hero-pipeline.md`）：每次生成前確認按鈕上是 25 點（H3.0），綁骨前確認預設是 UE5 Mannequin；按下生成後確認網址換成新的任務 ID、點數有扣，否則重新上傳。
- **驗證**：
  - 8 位的 retarget：facing 都是 180，參考姿勢誤差 3–5°，最大 36°（saika 的 Pistol_Aim_Down）。
  - `NativeHeroPoseSanity` 8/8 通過。
  - PoseCapture 截圖 `.ai-collab/tripo-pose-2/all_heroes_idle_move_attack.png`（側面：idle／跑步／attack）。
- **避免**：
  - 離線渲染動畫姿勢時，要用和截圖同一條渲染路徑驗證「換一支 clip 畫面就會變」，不能只看檔案大小或 hash 不同。
  - 要在網站上花錢的設定，每次送出前都看按鈕上的價格。

## E125 每局直接執行 target 裡的 omobab／client runtime，exe 被鎖住，其他 session 只能重建一半；bridge 拒絕 content hash 不同的 runtime，畫面一直是 entities=0（2026-10-27，omfx-decouple）

- **現象**：E124 的 smoke（10:57、11:06）、ward 與 tripo-heroes 的 run 都卡在 loading 4/6。UE log 一直是 `OM_PRESENTATION tick=1 entities=0`，runtime 不斷記錄 `renderer IPC disconnected: unexpected end of file`。重建 runtime 也失敗（`存取被拒 os error 5`）。
- **根因**：
  - UE 的 stderr 有 `om presentation runtime-ready rejected: runtime ... content=3c37d7b4ef769133 renderer ... content=62f50a931e14ae4f`。bridge 會比對 runtime ready 帶來的 `CONTENT_CATALOG_DATA_HASH`，不同就斷線；runtime 隨即重連，結果無限循環，畫面上看不到任何錯誤。
  - 為什麼會不同：每一局都直接執行 `omb/target/release/omobab.exe` 和 `omoba-client-runtime/target/release/omoba-client-runtime.exe`，執行中 exe 被鎖。lua_data 改動後，其他 session 的 cargo 只重建得了 moba-config 和 bridge，omobab 和 runtime 複製失敗，仍停在舊內容。launch-plan 的 `content_hash` 是 62f5…，跑起來的 runtime 卻是 3c37…。
  - preflight 沒有比對各個 binary 帶的內容版本。protocol 版本（`PRESENTATION_PROTOCOL_VERSION=14`）兩邊相同，所以那一層也查不出來。
- **通用修正**：
  - **exe 改從快取副本執行**：`run_artifacts.stage_binaries` 把 server 與 client runtime 複製到 `target/run-bin/<key>/`，再從副本啟動。
    - key 依來源的名稱、大小、時間產生，重建後會產生新副本，已在跑的局繼續用舊的。重用時會 touch `ready.txt`，最近用過的最晚被清。
    - 只保留 6 份，清理前先改名，使用中的 Windows 拒絕改名，因此不會被刪。
    - `target/<profile>` 底下的 exe 從此不被任何一局鎖住，cargo 隨時可以重建。
    - 接在 `moba_role_launch.prepare`（`use_run_binaries`），所以 run_moba_role_ue、split、spotlight、dedicated server 都會走這條路。
  - **指定 binaries 來源**：`OM_RUST_RUN_BINARIES=<cargo target 目錄>`（例如 session 自己的 `CARGO_TARGET_DIR`）指定所有 crate 的來源，比照 `OM_UE_RUN_BINARIES`；沒設定時才用各 crate 的 `target`。
    - 解析入口是 `run_artifacts.rust_target`／`rust_binary`。
    - 跟著這個變數的有：omobab、client runtime、moba-config（`moba_config_command`）、`moba_server_build.verify`、run 的 base_content 來源，以及 `ue_bridge_stage.sources`（影子 stage 的 bridge／base_content）。
    - 之前 exe 路徑寫死在 `moba_role_launch`，session 自建的 target 用不上。
  - **content marker 與 preflight**：
    - `omoba-template-ids/build.rs` 產生 `CONTENT_CATALOG_MARKER = "OM_CONTENT_CATALOG_DATA_HASH=<hash>;"`。omobab、client runtime 啟動時會 log 它，bridge 在 `om_runtime_create` 時 log 它（`black_box`），確保這段字串留在 binary 裡。header 不變，UE 不必重建。
    - `run_artifacts.content_hash` 直接掃描 binary 讀出 hash。
    - prepare 時，`verify_content` 拿 server、runtime，以及將要執行的 UE 專案（`OM_UE_RUN_BINARIES` 或 live）的 om_bridge.dll，和 authority 的 `content_hash` 比對；不一致就在啟動前報錯，並列出每個 binary 的 hash。
    - E125 之前建置的 binary 沒有 marker，只提示，不擋下。
- **驗證**：
  - `run_artifacts_test.lua` 增加到 40 項：快取、重建產生新副本、已開始的局保留副本、清理、marker 跨讀取邊界、不一致時拒絕、沒有 marker 只提示。
  - `test_moba_role_launch.lua` 11/11（斷言 server 與 runtime 都從 `target/run-bin` 執行）；`test_moba_dedicated_server.lua` 7 組；launch contract 9/9。
  - 實機 smoke：
    - 其他 session 的舊流程程序釋放 release exe 後，重建 omobab、client runtime、base_content，並用 `stage_ue_shadow.lua` 重新 stage 影子專案。三者的 marker 都是 `62f50a931e14ae4f`。
    - date_masamune spotlight（run-20261027-112837，離屏，影子 binaries）錄製成功，8 個 keyframe 都有，影片 1920x1080。
    - server 與 runtime 都從 `target/run-bin/<key>/` 執行；UE 達到 `tick=5379 entities=23 heroes=3`，沒有出現任何 `runtime-ready rejected`。
- **避免**：
  - 啟動遊戲程序一律從 run 副本執行（`run_artifacts`），不可直接執行 `target/<profile>` 裡的 exe。
  - 會在執行期比對內容或協定的兩端，launch 前就要用 binary 裡的 marker 比對；不能只靠執行期斷線來表示不一致，因為那在畫面上看不出來。
  - 尚未接上的入口：`moba_owned_viewport_ipc`、`run_moba_headless_batch.lua`、`run_ue.lua`、`run_2player*.lua`。

## E124 共用的 scripts/base_content.dll 與影子專案的 bridge／base_content 副本，被其他 session 的重建弄成過期，launch preflight 反覆擋下（2026-10-27，omfx-decouple）

- **現象**：E002、E106 之後仍反覆發生。某個 session 重建 debug base_content（`cargo build`、`cargo test`、build_ue_bridge 都會），共用的 `scripts/base_content.dll` 和 plugin 裡的副本就和 `scripts/target/debug` 不一致。結果：
  - `moba_server_build.verify` 擋下 server；
  - `build_ue_moba --verify-staged-only` 擋下 launcher；
  - 影子專案（E116）要靠各 session 自己的腳本手動 `cp`（ward 的 `ueproj.sh`、session-g1 的 `shadow_stage.lua`）；
  - bridge 被重建也一樣：今天 live plugin 的 om_bridge.dll（08:01 stage）比共用 build（10:29）舊，live launch 被擋。
- **根因**：
  - 所有 launch 共用同一份 stage 副本，preflight 檢查的是「副本 == 共用 target 裡最新的 build」，而那份 build 任何 session 都會改寫。這是在檢查新舊，不是在檢查這次 run 自己是否一致。
  - server 和 runtime 直接載入共用副本，副本被換掉時兩邊也可能不一致。
- **通用修正**：
  - **base_content 每個 run 各自一份**：
    - `scripts/run_artifacts.lua`：launch 的 prepare 從 `scripts/target/{debug,release}` 取最新的 build（同時間時取本次建置的 profile），複製到 `<run>/artifacts/scripts/base_content.dll`，並寫 `artifacts/base_content.json`（profile、來源、SHA-256）。複製期間來源被重建（大小或時間改變）會重試，三次後拒絕。
    - `moba_role_launch` 的 game.toml `SCRIPTS_DIR`／`DLL_PATH`、server 與 client runtime 的 `OMB_SCRIPTS_DIR`／`OMB_DLL_PATH` 都指向這份副本；載入器讀取的是 `OMB_SCRIPTS_DIR` 目錄，裡面只有這一個 DLL。
    - 同一台機器上的同一局必須載入相同內容：`run_moba_split_local` 的 client 加上 `--base-content <authority 的 run 副本>`，複製 authority 的那份，不另外取最新 build。
    - `ue_run_snapshot.prepare(root, deps, {base_content=...})`：執行副本用 run 的 DLL 取代 plugin 裡 stage 的那份，並把 SHA-256 混進 key，所以內容不同的 run 不會共用副本。
    - `moba_server_build.verify` 只要求有 build 存在，不再檢查共用副本。
  - **bridge 改成檢查 stage record**：
    - 新增 `scripts/ue_bridge_stage.lua`：stage header、lib、bridge、base_content，並在 `Plugins/OmRuntime/Binaries/Win64/om_bridge.stage.json` 寫下這組檔案的 SHA-256。`build_ue_bridge.lua` 已改用它。
    - launch 檢查（`verify_launch`）只確認專案裡的 DLL 和 header 是當初一起 stage 的；若共用 build 比較新，只提示「不是最新」，不擋下。
    - 沒有 record 的舊專案沿用舊規則；live 下次 `build_ue_moba` 後就會有 record。
    - 建置（build mode）仍然檢查剛 stage 進 live 的兩份副本。
  - **影子專案統一入口**：`tools\lua\lua.exe scripts\stage_ue_shadow.lua --project DIR [--sync] [--build]`。
    - `--sync`：只複製有變動的原始碼，刪除 live 已刪除的檔案；新影子專案會一次性帶入 live 的 Binaries 和 Intermediate/Build，讓 UBT 從增量開始。
    - 預設動作：codegen 輸出到影子專案；建置 bridge 和 debug base_content；cbindgen；stage 並寫 record。不會寫入 live。
    - `--build`：對影子專案跑 UBT（`-WaitMutex`）。
- **驗證**：
  - 新增 `scripts/tests/run_artifacts_test.lua`（27 項）、`scripts/tests/ue_shadow_stage_test.lua`（35 項）。
  - 更新並通過：`test_moba_server_build.lua`（5 組）、`test_moba_role_launch.lua`（11/11，斷言 server／runtime／game.toml 都指向 run 副本）、`tests/ue_bridge_stage_test.lua`（12 cases，verify-only 不再檢查共用 base_content）、`tests/run_moba_split_local_test.lua`、`test_moba_launch_contract.lua`（9/9）、`tests/ue_run_snapshot_test.lua`（27 項）。
  - 實際用 `stage_ue_shadow.lua` stage `target/session-g1/ue_shadow` 並寫入 record；`OM_UE_RUN_BINARIES=影子專案` 執行 `--verify-staged-only` 通過。
  - 實機 spotlight（date_masamune，影子 binaries，離屏，run-20261027-105729）：
    - 程序模組清單顯示 server（omobab）與 client runtime 都載入各自 `<run>/server|p1/artifacts/scripts/base_content.dll`，不是 `scripts\base_content.dll`；
    - 兩份副本與 UE 執行副本 `target/ue-run/359f6ed7cd481530` 內 plugin 的 base_content 都是 SHA-256 `585427ec…`；
    - 同時間另一個 session 的 spotlight-lineup run 也已走這條路徑。
    - presentation-only 的 UE 本身不載入 base_content。
    - 這兩次 smoke（10:57、11:06）都沒有錄到影片。runtime 正常 tick，但 UE 每次連上 presentation IPC 就斷線（runtime 記 1125 次 `renderer IPC disconnected: unexpected end of file`），畫面停在 `tick=1 entities=0`，loading 4/6 逾時。
      - 同時間另一個 session 的 spotlight-lineup run（快照 b3a0…）也是一樣的情況。
      - 10:36 成功錄影後唯一改變的是 bridge：影子專案從共用 target 重新 stage 了其他 session 10:29～10:56 間重建的 om_bridge.dll（c7268d9e…；header 沒變），runtime exe 仍是 10:28 版。
      - 想重建 runtime 做對照，但 exe 正被其他 run 使用（os error 5），對照未完成。
      - 研判是 bridge 和 runtime 不是同一批建置，與 per-run base_content 無關，列為後續工作。也就是說，「stage 最新的 bridge」需要搭配同一批建置的 client runtime。
  - `test_moba_network_launch.lua` 第 56 行失敗，原因是期望值 57062 沒有跟上 presentation port = port + player_id（57067），與本修正無關；它的 prepare 已產生 run 副本，`moba-config --config` 也接受了。
- **避免**：
  - launch 的檢查只回答「這次 run 自己是否一致」，不要拿共用 target 裡最新的 build 當基準。
  - 新的 launcher 要經過 `run_artifacts.stage_base_content`，不可讀 `scripts/base_content.dll`。
  - 影子專案不要手動複製 DLL，用 `stage_ue_shadow.lua`。
  - 尚未接上的舊入口（`run_ue.lua`、`run_2player*.lua`、`run_moba_headless_batch.lua`、`run_moba_runtime_smoke.lua`）仍直接讀共用副本或 target，列為後續工作。

## E123 浮動血量數字疊成一團，被誤認為「直排」：回血每 tick 一個、堆疊行號依數量計算、上飄跨行、名牌避讓逐行計算（2026-10-27，英雄 spotlight）

- **現象**：saika spotlight 的 08 影格中，敵方替身頭上有一團看似直排的「+0.1」。tamako 的燃燒傷害「-2.7／-2.1」也疊在一起，「-357.5／+0.9」在名牌上緣重疊。
- **調查**：
  - 連續放大 4 格後確認：每個數字都是橫向，和同一格的「-357.5」方向一致。E111 的 OmScreenFacing 在 spotlight 鏡頭下有生效，鏡頭沒有 roll，也沒有使用其他 view。
  - 「‡8:1」其實是 3 到 4 個「+0.1」上下重疊。
  - 回血和傷害走同一條路徑：整個專案只有 `OmCombatVisualSubsystem` 建立浮動數字。
- **根因**（`OmCombatVisualSubsystem`，一般對局和 spotlight 都受影響）：
  1. 回血幾乎每個 tick 都送一筆很小的回復，每筆各冒一個數字、各存在 1 秒。
  2. 堆疊行號取「同一單位目前有幾個數字」：較舊的數字消失後，新數字會重用仍被佔用的行。
  3. 行距 28，但上飄速度 45／秒，舊數字會飄進下一行，和在那一行新建的數字重疊。
  4. `ClearNameplate` 以每個數字自己那一行的位置計算上推量：第 0 行推得多、第 1 行推得少，兩者剛好停在名牌上緣同一個位置。
- **通用修正**：
  1. 小於 5 HP 的回復按單位累積，累積到 5 或從第一筆起滿 2 秒才合併成一個「+N」。傷害、暴擊、閃避和大額治療照舊即時顯示。到期的合併數字在老化迴圈之後才顯示，否則一次 Tick 的 DeltaSeconds 會讓它當場過期。`IsTickable` 也納入「有累積中的回復」。
  2. 新數字取「和每個存活數字目前高度（行號＋已上飄的行數）都至少差一行」的最低行。所有數字上飄速度相同，建立時隔開，整個存活期間都會隔開。
  3. 名牌避讓改以堆疊底部（`State.Origin`）計算，整疊數字一起上移，行距不變。
  4. 測試 `Om.Runtime.PublicHealthTextLifecycle` 新增斷言：
     - 30 個 0.1 合併成一個「+3」，累積到門檻時立即顯示；
     - 傷害不會被合併；
     - 每個數字各占一行，空出的行會被重用；
     - 0.3 秒後的燃燒數字會避開已經上飄的那一行。
     
     它和 `HeroNameplatePlacement` 都在影子 binaries 離屏 d3d12 下通過。
- **結果**：8 位 spotlight 重錄後，tamako 的燃燒數字等距排開，saika 的回血合併成一個「+0.8」，傷害數字橫向且不重疊。
- **過程中的錯誤**：
  - 第一版測試在一次 Tick 了 2.1 秒，合併數字當場過期而沒顯示，因此改了顯示順序。
  - 第二版測試沒考慮門檻後剩下的 1.0 還在累積。
  - 第一次修好重疊後，名牌避讓又讓底部兩行重疊，抽看影格才發現。
- **避免**：判斷「方向錯誤」前，先放大連續幾格，看是單一字形旋轉還是多個字重疊。凡是逐筆產生的高頻呈現（regen、DoT），都要有門檻或聚合；堆疊類的 UI 要用物件目前的實際位置挑空位，不能用數量計算；任何後處理的位移都要整組一起套用。

## E122 Tripo saika 雙臂穿過身體像殭屍：來源動畫庫和 Tripo 骨架朝向相反，auto-align 依世界方向把手臂拗向另一側（2026-10-27，Tripo saika 試點）

- **現象**：main 檢查 E121 的截圖，發現 idle 和 attack 都是兩臂往前平伸；attack 和 idle 幾乎一樣。E121 的直立檢查（Idle 963‰）沒有擋下來。
- **根因**：
  - 用新增的 `Om.Editor.PoseCapture` 以 FK 量測關節：retarget 後的 A_TPose，左臂從 X +11（左肩）伸到 X −17，右臂也一樣反過來，手臂穿過胸口。
  - 原因是朝向：CC0 動畫庫的參考姿勢腳趾朝 −Y，Tripo 骨架朝 +Y，兩邊左右命名都正確，只差 180°。`AutoAlignAllBones(Target)` 把目標骨頭對齊到來源的世界方向，所以每條肢體都被拗到身體另一側。
  - 直立度只看垂直範圍，手臂水平地往錯的方向伸，它照樣算「直立」。
  - 另一方面，來源的 Pistol_Idle_Loop 本來就是雙手持槍前舉（來源手腕 Z 1.34–1.40，肩膀 1.42，往前 0.36 m），Pistol_Shoot 和它幾乎一樣。saika 的步槍背在背上、手上沒有槍，所以這個 idle 看起來像殭屍，attack 也看不出差別。
- **通用修正**：
  - `OmCharacterRetargetCommandlet` 從參考姿勢的腳→腳趾（沒有腳趾就用頭→臉）算出來源和目標的朝向，取差值並四捨五入成 90° 的倍數（`FacingDegrees`）。
    - 不為 0 時：對來源 root 設 retarget pose 偏移（local delta `Ref⁻¹·Yaw·Ref`），每支來源評估動畫的 root 也乘上同一個 Yaw，然後才 auto-align。
    - 簽章加上 `facing:<deg>|`，結果回報 `source_facing_correction_deg`。facing 為 0 時簽章不變，CC0 六個 rig 的既有輸出照常 verified。
  - 姿勢忠實度 `pose_error_deg`：每支動畫在第一格和中間格，比較來源和目標「retarget root → 左右手、左右腳、頭」的方向夾角，取最大值。Lua 端要求每支 ≤ 60°、reference clip ≤ 20°，另外驗證 facing 只能是 0／±90／180，且簽章一致（測試 51 項，含 folded_limbs、fractional_facing）。
  - 實測結果：
    - saika 修正後是 5–18°（Pistol_Aim_Down 36°）。
    - CC0 回歸（male_base 等 6 個）是 3–16°。
    - 修正前手臂穿身體，接近 180°。
  - 新增 `Om.Runtime.NativeHeroPoseSanity`：每個用 TripoV1 身體的英雄都要通過兩項檢查。
    - idle 時兩手腕至少低於肩膀 5% 身高。
    - attack 和 idle 的手腕或頭位移至少 8% 身高。
    - saika 實測：手腕低於肩膀 0.26／0.25，attack 差異 0.385。
  - saika 的 idle 改成 Idle_Loop。
  - `Om.Editor.PoseCapture`（`-om-pose-capture-mesh/clips/dir`）可對任何網格加動畫輸出正面／側面 PNG，以及 FK 關節座標。
- **避免**：
  - 新的骨架來源先比對朝向（腳趾方向），再看 retarget 結果。
  - 判斷 retarget 好壞用「和來源比較的方向誤差」，不要只看直立度或有沒有動作。
  - 選 idle／attack 動畫時，要看英雄手上實際拿什麼；動畫庫的手槍姿勢不適合空手或背槍的身體。

## E121 Tripo 試點：retarget 驗證不檢查姿勢是否直立；俯視錄影誤判成躺倒；authored 英雄測試寫死舊 recipe；直接改了 live C++（2026-10-27，Tripo saika 試點）

- **現象**：
  - saika_magoichi 換成 Tripo 骨架後，spotlight 俯視（-60°）畫面看起來像躺在地上、雙手攤開。retarget 結果 46 物件、39 支有動作，全部「通過」，但驗證只檢查數值有限和有沒有動作，就算姿勢整個倒下也會通過。
  - `Om.Runtime.AuthoredHeroAnimationLifecycle` 寫死舊 b01 recipe 的材質路徑（M_SaikaCloth／RecipeV1）、`cast_secondary` 片段，並且要求 `EndSeconds > 0`。換成 Tripo 外觀後，8 個 slot 全部失敗。
  - 第一次 spotlight 啟動時被 `moba_stage_contract` 擋下：影子專案的 `om_bridge.dll`／`base_content.dll` 比目前的建置舊。
  - 本 session 一度直接 Edit live 的 `OmCharacterRetargetCommandlet.cpp`（還沒編譯就寫進 live 樹，違反 E102／E107），幾分鐘內用影子副本還原。
- **根因**：
  - retarget 驗證缺少「直立」量測。俯視畫面裡，雙手前舉的 Pistol_Idle 看起來像攤開，無法判斷姿勢。
  - 測試假設 saika 只會有一種外觀。
  - 影子專案的 Binaries 只由 UBT 建置，Rust bridge 與 script DLL 要另外 stage。
- **通用修正**：
  - `OmCharacterRetargetCommandlet` 為每支 clip 回報 `upright_permille`（第 0 格骨架垂直範圍 ÷ 參考姿勢垂直範圍）。`character_retarget.verify_result` 要求每支 clip 都有這個值、reference clip ≥ 900，且至少一半的 clip ≥ 700（測試加上 missing_upright、lying_library）。saika 實測：Idle 963、Pistol_Shoot 963、Crouch 約 500、Swim_Fwd 361，結論是站著的，不是躺倒。
  - 判斷姿勢改用 `Om.Runtime.AuthoredHeroAnimationLifecycle` 的側面截圖（`-om-authored-hero-capture=<dir>`），不用俯視錄影。這支測試改成通用版：網格在 `/Game/OmGenerated/Heroes/` 底下才套用舊 recipe 的期望；其他外觀改成要求每個 section 用自己匯入的材質，`EndSeconds` 為 0 視為整段，沒有 `cast_secondary` 就跳過。
  - 影子專案啟動前，從 `omfue/bridge/target/debug/om_bridge.dll` 和 `scripts/target/debug/base_content.dll` 複製到影子的 `Plugins/OmRuntime/Binaries/Win64`（只寫影子）。
- **驗證**：影子 UBT 建置（`-NoHotReloadFromIDE`）成功；retarget 兩次（verified／saved 0）通過新門檻；`AuthoredHeroAnimationLifecycle` 在影子執行副本 Success；`character_retarget_test.lua` 46 項。兩個 C++ 檔確認與影子完全相同後才複製回 live。
- **避免**：
  - 改 C++ 一律先改影子、建置通過再複製回 live。
  - 判斷模型姿勢、朝向、比例，用側面截圖；俯視畫面只能用來看辨識度。
  - 英雄尺寸不用手調：hero 由 `ConfigureWorldPresentationScale` 依網格邊界正規化到 120 world units。

## E120 離屏 renderer 忽略 -ResX/-ResY，spotlight 只錄到 888×500；stage 檢查不認 OM_UE_RUN_BINARIES（2026-10-27，英雄 spotlight）

- **現象**：
  - E114 改成 `-RenderOffScreen` 之後，spotlight 錄到的 `frames.jsonl` 每一格都是 888×500，ffmpeg 把它放大成 1080p，畫面糊掉。
  - 設定 `OM_UE_RUN_BINARIES` 用影子 binaries 跑時，client 的 preflight（`build_ue_moba --verify-staged-only`）一律回報「staged base_content differs」。
- **根因**：
  - 離屏時 UE 改用 null platform application，`-ResX/-ResY` 不會套到 viewport。
  - `moba_stage_contract.bindings` 寫死檢查 live 的 `omfue/Plugins/OmRuntime/Binaries`，但 snapshot 實際複製的是 `OM_UE_RUN_BINARIES` 指定的專案。
- **通用修正**：
  - `moba_role_launch` 對離屏的 autopilot／spotlight 座位加上 `-ForceRes`，並在 `-ExecCmds` 補上 `r.SetRes <寬>x<高>w`（尺寸取自 `--resolution`）。實際錄到的影格因此變成 1920×1080。
  - `bindings(root, project)` 預設檢查 `OM_UE_RUN_BINARIES` 指定的專案，沒設時才看 live omfue。`build_ue_moba` 在建置模式下明確檢查 live（它剛 stage 進 live）。`test_moba_launch_contract` 新增一項測試。
- **結果**：從影子 binaries（含 ward 的 E111 OmScreenFacing）重錄 8 位，全部 1920×1080，傷害數字為橫向。
- **避免**：換渲染模式（離屏、null RHI、遠端）時，第一次跑要先核對實際影格尺寸，不能只看參數有沒有帶上。任何「替代 binaries」機制都要讓 snapshot、preflight 與 stage 檢查讀同一個來源。

## E119 owned-viewport IPC 的 item-cooldown 斷言寫死「4.0」，HUD 早就改顯示「4秒」；alt-r-aid-self 偶發沒送出（2026-10-27，IPC 驗收）

- **現象**：
  - E115 修好之後，`manafix-ipc-3` 失敗在 `item-cooldown item HUD did not match the owner rows: kind=2 texts=守護符 4秒 [1] | ...`。冷卻值本身是 4.0，row 也正確，只有文字不符。
  - `manafix-ipc-2` 失敗在 `alt-r-aid-self produced no AbilityCast intent`：按 Alt+R 後，log 既沒有 OM_CAST 也沒有 OM_CAST_REJECT。
- **根因**：
  - item-cooldown：e397732（10-24）讓道具格的冷卻改用 `FOmDurationText::Seconds(v, true)`，整數秒會去掉「.0」，所以顯示成「4秒」或「[冷卻 4/4秒]」。`OmOwnedViewportIpcTests.cpp` 有四個地方仍用 `Contains(TEXT("4.0"))`；10-21 之後沒有跑過完整一輪 IPC，所以一直沒發現。
  - alt-r-aid-self：沒能重現。影子專案在 `HandleCastChord`／`HandleCastKeyUp`／`CombatAim.Press` 加了診斷 log 後，連跑三輪（ipc-3、4、5），這個案例每次都過：SelfCastR 的 chord 每次都看到 `hold=0`，modifier 也比對成功。「上一個案例的 R 鍵卡在按住」的推測沒有證據支持，所以沒有改產品程式碼。
- **通用修正**：
  - 測試新增 `ShowsItemCooldown(Text, Seconds)`，用 HUD 同一個 formatter 產生預期字串，同時接受精簡版「<值>秒」和完整版「冷卻 <值>/」。四處 `"4.0"` 斷言都改成呼叫它，數值本身仍然用 `Near(..., 4.f)` 檢查。
  - 影子建置一律帶 `-NoHotReloadFromIDE`（見 E118）。
- **驗證**：影子專案 `target/session-mana/ueproj` 以 UBT 建置（只多了診斷 log 和這次的測試修正）。帶 `OM_UE_RUN_BINARIES` 跑的 `manafix-ipc-4`（full，`-RenderOffScreen`）整輪通過：`success=true`、`Om.Runtime.OwnedViewportIpc` 沒有錯誤。`manafix-ipc-5` 在 Selection 階段撞到 860 秒的總時限；editor 啟動花了約 4 分鐘（當時機器負載高），在那之前所有案例都通過。live 只放回測試原始碼，沒有建置 live。
- **避免**：
  - 驗收斷言要檢查顯示文字時，用產品自己的 formatter 產生預期值，不要寫死格式。
  - 改 HUD 文字格式的 commit，要同時搜尋 IPC／automation 測試裡寫死的字串。
  - 偶發失敗要先加診斷 log 重跑幾輪取得證據，有證據才改產品程式碼。

## E118 影子專案的 UBT 建置被使用者 editor 的 Live Coding 擋下；角色管線只認 Quaternius 一套來源（2026-10-27，英雄 3D 模型）

- **現象**：
  - 在 `target/session-assets/ueproj/omfue` 影子專案用 `Build.bat OmGameEditor ... -Project=<影子>` 建置，UBT 直接回 `Unable to build while Live Coding is active`（使用者的 editor 開著 live 專案）。同一個指令十分鐘前還能建置。
  - 要替 8 位英雄匯入 Tripo 生成的骨架模型，但 `OmCharacterSourceImport`／`OmCharacterRetarget` 把來源目錄（QuaterniusV1）、輸出目錄與 registry 都寫死，也不接受帶材質的 FBX；`import_character_sources`／`retarget_characters` 還是直接用 live `omfue/om.uproject` 啟動（違反 E116）。
  - 用一個非 UE 標準骨架的 FBX 試跑時，UE 的 auto characterization 失敗（`native auto characterization/FBIK failed`），retarget 整個中止。
- **根因**：
  - UBT `HotReload.CheckForLiveCodingSessionActive` 以 engine 的 `UnrealEditor.exe` 判斷 Live Coding session；只要有任何 editor 開著 Live Coding，同一個 engine 的所有 editor target（包括影子專案）都會被擋，與影子專案自己的 Intermediate 無關。
  - 角色管線當初只為一套 CC0 rig 設計，沒有「來源集」的概念。
  - auto characterization 只認得 UE 已知的骨架樣板。
- **通用修正**：
  - 影子建置加 UBT 參數 `-NoHotReloadFromIDE`（`bAllowHotReloadFromIDE=false`，只關掉這個檢查；影子專案有自己的 Intermediate／Binaries，不會改到 live coding 載入中的 obj）。
  - 來源集：C++ `FSourceSet`／`FTargetSet` 與 Lua `character_source_import.sets`、`character_retarget.target_sets`。plan 沒有 `source_set`／`target_set` 時行為與 registry 完全不變（CC0 回歸：影子內從零 retarget male_base，46 物件、39 支有動作）；`tripo_v1` 讀 `SourceArt/Characters/TripoV1/<hero>/{<hero>.fbx,manifest.json}`，匯入嵌入材質／貼圖（各上限 16），輸出 `/Game/Omoba/Characters/TripoV1` 與 `TripoRetargetV1`。retarget 簽章帶「目標 FBX 的 sha256」而不是整份 registry 的 sha，新增英雄不會讓舊英雄的輸出變成 stale。
  - manifest 可選 `retarget = {root, chains=[{name,start,end}]}`：auto characterization 不認得的骨架改用明確的 chain（名稱必須是來源庫的 auto chain 名稱，才能 exact mapping；只做 FK）。chain 定義也寫進簽章。影子試跑：非標準 40 骨 rig 用 10 條明確 chain，46 物件、39 支有動作，第二次執行 verified／saved 0。
  - 兩個 driver 改用 `ue_run_snapshot.prepare`，plan 的來源路徑指向副本自己的 `SourceArt`。
  - `scripts/tripo_character_intake.lua`：下載檔只讀不執行，複製到 `<hero>/<hero>.fbx`，manifest 寫入 Tripo 任務、選項、點數與 Grok 參考圖 sha256。FBX 的 LimbNode 數只是估計（UE 會把中間的 Null 節點也算成骨頭：試跑 27 vs 實際 40），以第一次匯入回報的實際骨數為準（`--bones`）。
- **驗證**：`character_source_import_test.lua` 76 項、`character_retarget_test.lua` 44 項；影子 UBT 建置成功；影子專案自己的 Content（非 junction）跑完匯入兩次與 retarget 兩次。
- **避免**：
  - 影子建置一律帶 `-NoHotReloadFromIDE`；看到 Live Coding 錯誤不要重試等待，也不要叫使用者關 editor。
  - 新增一類美術來源時，先把「來源目錄、輸出目錄、registry、ownership 簽章」做成可列舉的集合，不要複製一份 commandlet。

## E117 影子執行副本的 UE 啟動時卡在 UBT QueryTargets；每份副本都重新暖 shader（2026-10-27，main）

- **現象**：ward 的 float-text-2 從影子執行副本啟動 shader warmup 後，log 從 08:38 起一直停在 `Launching UnrealBuildTool... -Mode=QueryTargets -Project=target/ue-run/<key>/omfue/om.uproject`，6 分鐘以上沒有進展；同時 launcher 印出 `Warming Unreal shaders ... (cold DDC)`。
- **根因**：
  - E116 的副本沒有 `Intermediate/`。UE editor 讀不到 `Intermediate/TargetInfo.json`，就呼叫 UBT `-Mode=QueryTargets`；UBT 的全域 mutex 被其他 session 的影子建置佔住，所以只能一直等。
  - `ue_shader_warmup.stamp_path` 用專案路徑當 key。副本路徑每次重建都不同，雖然 DDC 透過 junction 共用，仍被判定為冷。
- **通用修正**：
  - `ue_run_snapshot` 新增 `seed_target_info`：新建和重用的副本都會從 live `omfue/Intermediate/TargetInfo.json` 補一份，時間戳記為當下，除非之後 `*.Target.cs` 有變動，否則快取一直有效。
  - `ue_shader_warmup.stamp_path` 把 `target/ue-run/<key>/omfue` 對應回 live `omfue`。
  - 測試：`ue_run_snapshot_test.lua`（27 項）、`ue_shader_warmup_test.lua`。
- **避免**：讓 UE 從另一個專案目錄執行時，要列出 UE 啟動時會讀寫的專案狀態（Intermediate 的快取、DDC、Saved），逐一確認不會觸發建置工具或冷快取；不能只看 Binaries。

## E116 自動化 UE 從 live omfue 啟動，鎖住 DLL，使用者在 Visual Studio 編譯失敗（LNK1104）（2026-10-27，main）

- **現象**：使用者在 VS 建置 `OmGameEditor` 時，OmRuntime、OmEditor、OmGenerated 與 32 個 UECP DLL 全部連結失敗（`LNK1104`、UBA 的 `being used by another process - UnrealEditor-Cmd.exe`）。占用者是子 agent 正在跑的 owned-viewport IPC 驗收。E114 的離屏只解決搶視窗，沒解決鎖檔。
- **根因**：所有自動化（Om. 測試、IPC 驗收、autopilot／spotlight 實機局、英雄選擇）都用 `omfue/om.uproject` 啟動 UE，模組 DLL 從 live `Binaries` 載入並被映射；只要有一個在跑，任何人都無法重新連結。
- **通用修正**：
  - `scripts/ue_run_snapshot.lua`：在 `target/ue-run/<key>/` 建立執行副本。只複製 `omfue/Binaries` 與各外掛 `Binaries`（不含 pdb、exp、lib、exe，約 100 MB，3 秒）；`Content`、`Config`、`Saved`、外掛 `Source` 與 repo 其他頂層目錄都用 junction 連到 live，因此 `ProjectDir()/..` 照常找到 `scripts`、`omb`，證據仍寫進 live `Saved`。
  - `<key>` 由 live binaries 的名稱、大小、時間算出，重建後自動換新副本；複製期間 binaries 變動就重試。
  - 只保留最近 3 份。清理時先改名，使用中的副本 Windows 拒絕改名，因此會略過；刪除時只移除 junction 本身，絕不進入 live 樹。
  - `OM_UE_RUN_BINARIES=<影子專案目錄>`（或 `prepare(root, nil, {binaries=...})`）：副本改從 session 自己的影子建置複製 Binaries，Content／Saved 仍是 live。使用者的 editor 開著 live 專案、live DLL 無法重建時，session 用這個測試自己的建置，不必等 live。
  - lua-host 新增 `junction_create`、`junction_inspect`、`junction_remove`：建立目錄符號連結需要權限，junction 不需要。
  - 已接上 `ue_local_acceptance`、`moba_owned_viewport_ipc`、`moba_role_launch`（實機 renderer 與英雄選擇）。
- **驗證**：`scripts/tests/ue_run_snapshot_test.lua`（20 項）；實際從副本執行 `Om.Runtime.ShopModel`，log 確認 Om 與 UECP 模組都從 `target/ue-run/...` 載入；執行中的副本改名會得到 `Permission denied`。
- **避免**：任何 agent 手動啟動 `UnrealEditor(-Cmd)` 都必須用 `require('ue_run_snapshot').prepare(root).uproject`，不可直接用 `omfue/om.uproject`。只有 `build_ue_moba` 與資產 commandlet（要寫 live Content）可以碰 live 專案。

## E115 owned-viewport IPC 的 q-mana-refuses：法力不足的 Q 沒有在前端被拒，直接送上 wire（2026-10-27，IPC 驗收）

- **現象**：`ue_local_acceptance.lua --native-rhi d3d12` 最後的 IPC 階段（run `offscreen-rhi-1-ipc`）失敗：`q-mana-refuses refusal was not Owned ability mana is short (accepted=1 id=49 error=)`。fixture 的 `owner_short_mana` 已經把法力設成 lumen_bolt rank 1 成本 −1，ack 也正常。
- **根因**：3ad758b1（10-24）改成「價格要由 authority 證明」：`moba_hud::effective_mana_cost` 只在快照有同一 tick 的 `MANA_COST_TICK`、對上 ability／rank 的 `MANA_COST_BINDING[slot]` 和 `MANA_COST_RAW[slot]`（team-scoped HUD 事件，由 `moba_match::emit_owner_mana_costs` 發出）時，才給 `mana_cost_state` 1／2；否則是 0（價格未知）。`renderer_fixture.rs` 的 `to_snapshot` 從沒發過這三種事件，所以每個 slot 都是 0，`AOmWorldBridgeActor` 只在 state==2 時拒絕，就把 Q 放行了。fixture 執行檔本身不舊（IPC 腳本每次都會 `fixture.build()`），fixture 的單元測試只檢查 mana 數值、沒檢查 `mana_cost_state`，所以沒抓到。
- **通用修正**：`omoba-client-runtime/src/renderer_fixture.rs` 新增 `owner_mana_cost_events`，每個快照依 authority 同樣的格式發出 owner 的 tick／binding／cost（team = OWNER_TEAM_ID，tick = 快照 replica_tick，cost = 編譯後 rank 成本；owner 不在、死亡或沒有法力池時 binding 0、cost −1）。fixture 測試加上 `owner_short_mana` → `(2, cost)`、`owner_cooldown`（滿法力）→ `(1, cost)`。
- **驗證**：fixture lib 測試 50/50、`owned_viewport_ipc_contract_test.lua` 140 cases 通過。IPC 重跑 `manafix-ipc-2`（full，`-RenderOffScreen`）：q-mana-refuses 得到 `Owned ability mana is short`，之後 q-cooldown-refuses、q-far-refuses、apprentice、point、死亡／替換／移除／epoch、r-aid-ally 都通過。整輪仍然沒過：卡在後面的 `alt-r-aid-self`，Alt+R 沒有送出 SelfCast（log 沒有 OM_CAST 或 OM_CAST_REJECT，10-21 的 foundation run 有 `label=SelfCast input=54`）。法力狀態只會在 bridge 拒絕，而且會留下 OM_CAST log，所以研判跟這次修正無關，另外追查。後續見 E119：`manafix-ipc-4` 整輪全綠。
- **避免**：HUD／前端改成只信任 authority 發布的新事件時，同一個 commit 要同步讓 renderer fixture 發出這些事件，並在 fixture 測試直接斷言前端實際讀的欄位（例如 `mana_cost_state`），不要只斷言原始數值。

## E114 自動化測試與 autopilot 局的 UE 視窗搶走使用者桌面：使用者開著 editor 卻無法操作（2026-10-27，main）

- **現象**：使用者開 Unreal Editor 時，各 session 的 RHI 全量測試與 autopilot／spotlight 實機局不斷在桌面開出視窗。`OmRequireInteractiveDesktop` 還會 `HACK_ForceToFront` 並用 `SetCursorPos` 把真實游標移到測試點，使用者完全無法控制 editor。使用者 editor 的 log 也收到其他測試程序的 `LogAutomationWorker: Received FindWorkersMessage`。
- **根因**：所有自動化 UE 程序都用真實桌面（Windows platform application）開視窗；「互動桌面」測試預設就會搶前景、移動游標。沒有任何一處區分「使用者的桌面」與「自動化可用的桌面」。
- **通用修正**：
  - `scripts/ue_desktop.lua`：自動化啟動的 UE（`ue_local_acceptance` 的 d3d12 套件、`moba_owned_viewport_ipc`、`moba_role_launch` 的 autopilot／spotlight 座位）預設加 `-RenderOffScreen`。UE 改用 null platform application 與無 swapchain 的 viewport（D3D11／D3D12 都支援），不在桌面出現視窗、不搶前景、不動游標；backbuffer 截圖與錄影照常。
  - `OM_OWN_DESKTOP=1` 才改傳 `-om-own-desktop`，保留可見視窗。
  - `OmHeadlessAutomation.h` 新增 `OmOwnsDesktop()`：只有 `-om-own-desktop` 且不是離屏時，`OmRequireInteractiveDesktop` 才會把視窗拉到前面、移動游標；否則只驗證視窗自己的 hit-test grid，並警告略過真實桌面路由。
- **避免**：任何 agent 手動下 `UnrealEditor(-Cmd)` 跑 RHI 測試或實機 autopilot 局，都必須帶 `-RenderOffScreen`，除非使用者明確同意讓出桌面。不要寫會移動真實游標或強制前景的測試，除非用 `OmOwnsDesktop()` 擋住。

## E113 spotlight --all 錄完的英雄 renderer 沒退場，8 位錄完時機器上同時有 8 個 UE＋8 個 omobab；法力不足被拒的施放被當成成功（2026-10-27，英雄 spotlight）

- **現象**：
  - `record_hero_spotlight.lua --all` 執行期間（07:02–07:15），每位英雄錄完後，它的 UnrealEditor 遊戲視窗和 omobab 仍然留著；到 07:14 時，機器上共有 8 個 UE 和 8 個 omobab。
  - 預覽的 trace 顯示，omatsu E、oichi R、inahime R、asai W／R、murakami W／R、tamako W 都被 authority 以 `NOT_ENOUGH_MANA`（code 6）拒絕，spotlight 卻把它們算成已施放。
  - saika 的 E 被跳過，原因是敵方替身已經被召喚物和強化普攻打死。
- **根因**：
  - 8 位英雄都在同一個 `supervise_workflow` 裡依序執行。split runner 只停掉它直接啟動的 lua 角色啟動器；UE 和 runtime 是啟動器的子孫程序，要等整個 workflow 結束、job 被回收時才會一起關掉，所以每位英雄的程序都留到最後。
  - 一招示範連放 4 到 5 個技能，法力撐不到最後。spotlight 只看「有沒有送出去」，沒看 authority 的結果。
  - 替身 8 倍血量不夠。
- **通用修正**：
  - `--all` 改成每位英雄各開一個子程序（`record_hero_spotlight.lua --hero X`），每個都有自己的受監督 workflow，子程序結束時那位英雄的 server、runtime、UE 一定會被回收。每位結束後再用 `assert_retired` 依 owned-processes 紀錄（含 creation_token）確認沒有任何程序還活著，然後才開始下一位。每位英雄也用自己的 port（57461 + 10×序號）。
  - spotlight 會讀取 authority 的施放結果：被拒的示範會在等待時間內重試，超過時間才記為 `spotlight_refused`／skipped，並附上拒絕碼。
  - showcase 的 recipe 改為 `mana_enabled=false`（spotlight 的主題不是資源管理）、`start_level=16`（所有大招都學得到，包括 asai 16 級的 extra 招）、`hold_hp_permille=30000`。
- **避免**：
  - 在同一個 workflow 裡連續跑多次實機時，每次結束都要確認它的整棵程序樹都已退場，不能只停直接啟動的子程序。
  - 判斷「施放成功」一律看 authority 的 ack 和效果，不能看「送出」。

## E112 商店改版：合成道具直接購買缺材料必被拒、舊 TD P 鍵在 MOBA 仍送塔優先指令、商店版面依第一幀估算而錯位（2026-10-27，shop-v3）

- **現象**：
  - 權威端 `buy_item` 要求合成道具的每個直接材料都已在背包，否則回 MissingComponent；舊商店只能直接買整件，缺材料時玩家點了必定失敗。
  - 舊 `RustBP/System/BP_PlayerController` 的 TD 按鍵圖在 MOBA 仍輪詢 P（塔優先）等鍵，送出 SetTowerTargetPriority 等 TD 指令（E100 待辦）。
  - 新商店的 roomy／compact 判斷與 body 高度一開始用「模型估算的 frame」，第一幀 header／指令列帶狀區還沒量到，`ShopUndoProjectionAndPin` 首幀量不到復原按鈕；實機 720p 帶狀區只有約 290px，門檻 320 讓 1280 寬的商店退回直排版，格子被擠到畫面外。
  - 工作副本用 `sed -i` 刪行，Git Bash 的 sed 把整個 CRLF 檔改成 LF（當場用 unix2dos 復原，未進 live）。
- **根因**：把「直接購買」當成「依配方購買」；TD 藍圖輸入沒有依模式停用；版面依賴上一幀量測值而不是 Slate 本身的排版。
- **通用修正**：
  - `FOmShopModel`（OmShopModel.h/.cpp）只讀目錄資料算出階級、分類、配方樹、進階道具、扣除已持有材料的價格與「下一件該買什麼」。左鍵仍是原本的單一權威購買（不改 wire 合約）；右鍵、詳細欄「購買」、佇列都送 `NextPurchaseStep`，每一步都是一筆正常的權威交易。
  - `AOmWorldBridgeActor::CanSubmitInputEvent` 在權威模式為 MOBA 時拒絕 PlaceTower／UpgradeTower／SellTower／SetTowerTargetPriority／StartRound（不送 wire），TD 模式不受影響；`Om.Runtime.LegacyTdInputGuard` 驗證。
  - 版面改由 Slate 自己決定：外層 `OmShopScroll` 的 body slot 用 `FillContentSize`，roomy 時 split 只要求最小高度、由 slot 撐滿；footer 放在 split 內（roomy 在底部、compact 在最上面）；compact 門檻依文字縮放調整（寬 640×max(1,0.75s)、高 200×s）。
  - 行刪除改用 Lua 以位元組讀寫（scratchpad 的 dellines／movelines），不再對 repo 檔案用 `sed -i`。
- **避免**：送任何交易前先確認權威端的前置條件（材料、範圍、模式）；UI 版面不要用上一幀量測決定結構，交給 Slate 的 slot 規則；改 CRLF 檔只用 Edit 或二進位安全的 Lua。

## E111 浮動傷害數字被轉成 90 度直排，而且有殘影（2026-10-27，session-ward）

- **現象**：spotlight keyframe（`target/spotlight/saika_magoichi/keyframes/06_slot2_passive_effect.png`）與遊戲中，「-147.9」都是直排，旁邊還有疊影。
- **根因**（與 E097 同類：寫死 world 軸，沒有從主攝影機實際的視角推導）：
  - `OmCombatVisualSubsystem` 用 `(CameraLocation-Location).Rotation()` 讓文字面向攝影機。這只決定 X 軸，roll 一律是 0。MOBA 攝影機接近正俯視，pitch 約 90 度時 yaw 只由一個極小的水平分量決定，文字的上方因此變成任意的水平方向，畫面上就成了旋轉的直排。
  - 數字上浮與同一單位的多個數字疊放，都是沿 world Z 偏移。Z 在俯視時就是視線方向，所以數字既不會在畫面上上浮，疊放的數字也全部重疊，看起來就是殘影。
- **通用修正**：
  - 新增 `OmScreenFacing`（OmRuntime）。`TextRotation` 以 `MakeFromXZ(-camera forward, camera up)` 計算旋轉。UTextRenderComponent 的字形在 YZ 平面，閱讀方向是 -Y，所以這樣 -Y 正好是畫面右方，文字由左到右、不鏡像。`ScreenUp` 回傳攝影機的 up 向量。
  - 浮動數字每一 frame 依當下的視角重設朝向，上浮與疊放都沿畫面上方；疊放層數另外記錄，不再寫死在 Origin 裡。
  - 舊版 buff 文字（原始 VFX 關閉時，`AOmUnitActor::Tick`）也改用同一個朝向。
- **其他 world-space 文字檢查**：名牌是 screen-space widget，ping 是投影到螢幕位置，表情不用 world 文字，三者都沒有這個問題。
- **測試**：`Om.Runtime.PublicHealthTextLifecycle` 新增 yaw 0／90／37（pitch -80）的檢查：閱讀方向沿畫面右方、基線水平（與畫面上方的內積小於 0.01）、文字直立、面向攝影機、沿畫面上方上浮。
- **實機證據**：float-text-4（quick map，lane_v0，影子建置加 `OM_UE_RUN_BINARIES`，離屏執行）。`p1/autopilot-p1-shot-004-combat-text.png` 的「-118.8」與 `p6/autopilot-p6-shot-001-combat-text.png` 的「+2」都是水平、可讀、沒有殘影，位置在名牌上方。全量 Om. RHI 302/302、-nullrhi 302/302。
- **附帶修改**：autopilot 新增「combat-text」截圖，每局最多 3 張、間隔 20 秒，只在浮動文字落在本機 viewport 內、且載入畫面已關閉時才拍（`HasHealthTextOnScreen`、`IsMatchLoadingScreenShown`），之後的驗收可以直接拿到傷害數字的畫面。
- **避免**：
  - world-space 文字或 billboard 一律以攝影機的 forward 與 up 建出完整的旋轉，不要只用方向向量 `Rotation()`。
  - 位移動畫要沿畫面座標軸，不要沿 world 軸。

## E110 第一支 spotlight 影片拍到錯的地方：鏡頭沒對準英雄、和別人的比賽撞 port、替身被字幕擋住（2026-10-27，英雄 spotlight）

- **現象**：
  1. hold 席位被移出 bot 指派後，server 拒絕啟動，訊息是「human authentication and bot controllers must partition the match roster」。
  2. 第一支 saika 影片整段都拍著基地地面，鏡頭沒有移動。對自己施放的 Q、R 成功了，但 W、E 回報「target is outside the visible world area」，最後被跳過。
  3. 重錄時 server 綁不到 57061（os error 10048），因為另一個 session 的 omobab 正在用這個 port；我的 client 還連上了那個 server。
  4. 敵方替身被放在 +x 方向，畫面上是正下方，剛好被字幕擋住，被動展示的普攻點擊失敗。
  5. keyframes 資料夾裡混著上一輪留下的舊圖。
- **根因**：
  1. server 把名單分成「人類」與「bot 控制者」兩組，hold 席位必須算在控制者裡。
  2. RTS 鏡頭並不是 controller 的 pawn。`Cast<AOmRtsCameraPawn>(PC->GetPawn())` 拿到的是 null，於是沒有對準英雄、也沒有拉近，而且沒有任何錯誤訊息；我又寫死了一個絕對縮放值 1100。
  3. 所有 split 局預設都用 57061，錄影卻沒有自己專屬的 port；而且「等其他程序結束」的檢查和實際啟動之間還有時間差。
  4. 擺放位置時沒有先確認地圖座標和畫面方向的對應。
  5. 編碼前沒有先清空輸出資料夾。
  6. 另外，`--no-build` 加上「server 用舊的 release omobab」導致修正沒有生效。正確順序是：先讓 server 角色建置，再 `build_ue_moba --build-only` 重新 stage，最後才用 `--no-build` 錄影。
- **通用修正**：
  1. hold 席位保留 bot 指派，但在 `role_bot_inputs` 裡依比賽設定（`ShowcaseConfig::holds`）略過，不產生任何輸入。測試有對照組：同一批席位不設 hold 時會做出決策。
  2. 鏡頭改用 `AOmRtsCameraPawn::ResolveForController`；縮放改成目前距離 × 0.6；每一招開始前重新對準英雄；拿不到鏡頭就以 `camera_unavailable` 結束，不會錄出錯誤的畫面。
  3. `record_hero_spotlight.lua` 改用專屬的 port 57461（可用 `--port` 指定，但不允許 57061）。
  4. 替身改放在英雄左右兩側（y 方向），間距 280，並避開河道草叢。
  5. 編碼前先清空 keyframes。
- **結果**：saika_magoichi 四招全部施放，0 次跳過，785 格，0 格掉幀，影片約 26 秒，H.264 1920×1080 30fps。
- **避免**：需要點擊畫面的自動化，在錄影或驗收前一定要先截一張圖確認構圖，不能只看 ack 結果。取得遊戲物件失敗時要明確報錯並中止，不能默默略過。共用機器上的實機程序都要用自己專屬的 port。

## E109 載入畫面第二輪又在 live 樹改到一半，其他 session 編譯失敗；`TryGetStringField` 把 JSON 數字當成字串（2026-10-27，match loading screen）

- **現象**：
  - 加 MoviePlayer 地圖載入階段時，我直接在 live `omfue` 改 `OmMatchLoading*`、新增 `SOmMatchLoadingScreen`／`OmMatchLoadingSubsystem`，還沒改完呼叫端與 Build.cs，ward 和 spotlight 的 build_ue_moba 因此失敗（違反 E102／E107）。
  - 新測試 `MatchLoadingRoster`「Non-text display name rejected」在 nullrhi 與 RHI 都失敗：`"display_name":7` 被接受。
- **根因**：
  - E107 寫了「先在工作區編譯」，但當時沒有能單獨編譯的工作副本，於是又回到 live 改。
  - UE 的 `FJsonObject::TryGetStringField` 會把數字、布林轉成字串回傳 true，不能拿來驗證型別。
- **通用修正**：
  - live 樹先還原為已提交且能編譯的版本，半成品移到 `target/session-loading/work`。
  - 建立獨立的影子專案 `target/session-loading/shadow/omfue`（複製 `om.uproject`、`Config`、`Source`、`Plugins/OmRuntime/Source`，影子 uproject 停用 BpGeneratorUltimate），用它的 `-Project=` 編譯 OmGameEditor（含 OmEditor 測試）。它有自己的 Intermediate／Binaries，不會碰到 live 的 DLL 或 staging。影子編譯通過後，等 `target/locks/build_ue_moba` 釋放才複製進 live，接著立刻跑 build_ue_moba。
  - 型別驗證改用 `HasTypedField<EJson::String>`，再 `TryGetStringField`。
- **避免**：
  - 改既有 UE 模組（尤其是 Build.cs 依賴或跨檔 API）時，一律先在影子專案編譯，再著陸；著陸前檢查 live 檔案在複製影子之後沒有被別人改過（只差自己的改動）。
  - 驗證 JSON 欄位型別時不要依賴 `TryGet*Field` 的回傳值，用 `HasTypedField<EJson::...>`。

## E108 RHI 的真實命中測試依桌面狀態失敗：UECP 1.7.3 的置頂歡迎視窗擋在測試點上（2026-10-27，session-ward）

- **現象**：
  - 自 2026-10-26 23:05 起，每一輪 RHI 全量 Om. 都固定失敗 10 項：PieActionOwnerMatrix、ControlRuntimePersistence、MinimapPreferences、OwnedItemCards、ReadableHudToolTip、ItemIconPresentation，以及商店 4 項。21:02 那一輪全部通過。
  - 失敗的都是用 `FSlateApplication::LocateWindowUnderMouse` 做的真實命中測試（例如「Actual PIE world center hit-test」、「painted hit path」、「under the pointer」、「Actual hover presents …」）。
  - 在沒有 editor、Claude 視窗最小化的情況下仍會重現。
- **根因**：
  - `LocateWindowUnderMouse` 會先向 OS 詢問「真實游標底下是哪個視窗」，再依 Slate 的視窗順序 fallback，所以結果取決於桌面上的前景視窗、真實游標位置與視窗堆疊順序。
  - 加上診斷後確認：點位被導到 `'Ultimate Engine Copilot'` SWindow。這是使用者 21:42 換上的 UECP 1.7.3 每次啟動都會跳出的歡迎／公告視窗，`IsTopmostWindow(true)`，700×620 置中，直到在外掛 UI 按下 dismiss 為止。21:02 通過、23:05 失敗，中間的變化就是這次換外掛；沒有證據顯示是程式回歸。
  - 背景程序無法取得前景，`HACK_ForceToFront` 也拿不到。
- **通用修正**（`OmEditor/Private/OmHeadlessAutomation.h`）：
  - `OmLocateInWindow`：只把目標視窗列為候選，查詢該視窗自己的 hit-test grid。結果是確定的，與 OS 前景和真實游標無關。
  - `OmRequireInteractiveDesktop`：先 ForceToFront／BringToFront，再把真實游標移到測試點；仍然拿不到時，AddWarning「desktop not interactive: <原因>; the point routed to '<視窗>'」。
  - `OmLocateHit`：一律回傳視窗 grid 的路徑，由呼叫端驗證。桌面可以互動時，真實路徑也必須命中同一個 leaf widget，否則判為錯誤，所以回歸不會被蓋掉。
- **套用範圍**：6 個非商店測試。
  - 點擊在桌面不能互動時，改沿著 grid 路徑 Route*。
  - tooltip 一律驗證 hovered path 最內層的 tooltip 是該卡片的 tooltip，以及說明文字；真實 popup 的呈現與截圖只在桌面可以互動時執行。
  - 商店 4 項交給商店改版的子 agent 套用。
- **證據**：
  - 6 項全數通過。
  - 移開游標後，MinimapPreferences 的真實路徑通過且與 grid 一致；OwnedItemCards 多數卡片點擊的真實路徑也通過。
  - 只有落在 UECP 歡迎視窗範圍內的點（PIE 中心、部分卡片）被跳過。
  - 完整一輪都走真實路徑，需要使用者在 UECP 外掛中按下 dismiss，關掉歡迎視窗（設定存在外掛的全域 config，不由 session 修改）。
- **避免**：
  - 「actual」命中測試一律經過 `OmLocateHit`，不要直接呼叫 `LocateWindowUnderMouse(…GetInteractiveTopLevelWindows())`。
  - 安裝或更新 editor 外掛後，先確認是否會跳出置頂視窗，再跑 RHI。

## E107 載入畫面新檔著陸後才第一次編譯：兩個編譯錯誤短暫弄壞 live 樹；`sed -i` 把 CRLF 改成 LF；繞過 build_ue_moba 直接呼叫 Build.bat（2026-10-26，match loading screen）

- **現象**：
  - 外掛換版期間 build_ue_moba 無法執行，我把 `OmMatchLoading*` 新檔放進 live `omfue` 後，直接用 `Build.bat OmGameEditor` 編譯，第一次失敗兩處：UE 5.8 的 `FJsonObject::Values` key 是 `UE::TSharedString`，沒有 `IsNumeric/Contains`；迴圈變數 `Slot` 遮蔽 `UUserWidget::Slot`（C4458 在專案設定為 error）。約 3 分鐘內修正並編譯成功，但這段時間其他 session 的 UE 建置會失敗。
  - 用 Git Bash 的 `sed -i` 修改 CRLF 檔案後，整個檔案變成 LF（`file` 顯示 `with CRLF, LF` 或無 CRLF）。
  - 直接呼叫 `Build.bat` 不經過 build_ue_moba，沒有 E106 的建置鎖，可能與其他 session 的建置互相干擾。
- **根因**：違反 E102 的「先在工作區編譯再著陸」；UE 5.8 JSON API 與舊版不同卻沒有先查標頭；Windows 上的 GNU sed 會正規化行尾；為了繞過暫時壞掉的 build_ue_moba 改走 UBT。
- **通用修正**：修正兩處編譯錯誤（`FString Key(Pair.Key.ToView())`、改名 `SpellSlot`）；以 Lua 二進位讀寫把 5 個新檔統一成 CRLF；之後一律用 build_ue_moba／consistent_build（E106 建置鎖）。
- **避免**：
  - 新 UE 檔案先放工作區，等 build_ue_moba 可用、編譯通過才著陸；build_ue_moba 壞掉時等待，不改走 Build.bat。
  - 讀取 `FJsonObject::Values` 的 key 時用 `ToView()` 轉成 `FString`。UUserWidget／UWidget 子類別裡不要用 `Slot` 當區域變數名。
  - CRLF 檔案只用 Edit 工具或 Lua `rb/wb` 改寫，不用 `sed -i`；改完用 `file` 確認行尾。

## E106 多個 session 同時跑 build_ue_moba，staged base_content.dll 互相覆蓋，checksum 驗證失敗（2026-10-26）

- **現象**：錄影 session 的 build_ue_moba，和載入畫面子 agent 的 consistent_build 同時進行；staged `scripts/base_content.dll` 的 checksum 驗證失敗，錄影建置因此中止。
- **根因**：UBT 有全機 mutex，但 build_ue_moba 前後的 cargo 建置與 DLL／bridge staging 沒有任何互斥，兩個流程交錯寫入同一份 staged 產物。
- **通用修正**：新增 `scripts/build_lock.lua`，build_ue_moba 在 `supervise_workflow` 之後（只有 supervised 子程序持有）取得 repo 層級的鎖 `target/locks/build_ue_moba`（mkdir 是原子操作；owner.txt 記錄持有者）。其他執行會排隊等待並印出持有者；超過 3 小時視為失效鎖並接手；鎖用 to-be-closed 變數持有，不論成功或錯誤都會釋放。consistent_build、hero_author、各 session 只要經過 build_ue_moba 都會自動排隊。測試：`scripts/tests/build_lock_test.lua`。
- **避免**：只要多個程序會寫同一份產物，就要有互斥機制；不能只靠「請別的 session 先不要跑」的口頭協調。

## E104 外掛換成 vfx_mcp 附的 BpGeneratorUltimate_1.7.3 後，build_ue_moba 卡在寫死的外掛路徑（2026-10-26）

- **現象**：使用者在處理 UECP 授權時，把 omfue/Plugins/BpGeneratorUltimate（2.0.6）換成 vfx_mcp 工具包附的 BpGeneratorUltimate_1.7.3。`scripts/ensure_bpgu_animation_import.lua` 寫死讀取 `omfue/Plugins/BpGeneratorUltimate/BpGeneratorUltimate.uplugin`，所以 build_ue_moba 在編譯前就失敗了。UE 本身是依 descriptor 名稱載入外掛，所以 OmGameEditor 照樣能編譯。
- **通用修正**：改成在 `omfue/Plugins/*` 底下搜尋 `BpGeneratorUltimate.uplugin`：找不到或找到兩份都直接失敗。只接受 EngineVersion 5.8.0。如果是 `"Installed": true` 的預編譯版本，UBT 不會重建，source patch 也沒有作用，所以只記錄版本，不修改外掛；只有 source 版本才套用動畫匯入的 patch。不再寫死 2.0.6。
- **避免**：第三方外掛依 descriptor 名稱尋找，不要依資料夾名稱，也不要寫死版本號；不要修改使用者正在設定的外掛。

## E103 VFX runtime 未經編譯就著陸，使用者用 VS 編譯 OmGameEditor 時失敗（2026-10-26，技能特效）

- **現象**：OmAbilityVfx 著陸後，`build_ue_moba --build-only` 在外掛檢查階段（BpGeneratorUltimate 被換掉）就失敗了，C++ 根本沒有編譯，我卻把它當成「外掛問題，與我無關」，沒有先撤回自己的改動。之後使用者在 VS 編譯時遇到三個錯誤：
  - `OmAbilityVfxCatalog.h` 不存在：codegen 輸出新增的檔案只會在 build 流程中生成。
  - `OmUnitActor.cpp` 的區域變數 `Layers` 遮蔽了 `AActor::Layers`（C4458，同 E067）。
  - `OmAbilityVfxTests.cpp` include 了 `NiagaraComponent.h`，但 OmEditor 沒有 Niagara 依賴。
  - 主 session 已修正：重跑 codegen、改名為 `VfxLayers`、在 OmEditor.Build.cs 加上 Niagara。
- **根因**：
  - 把「建置流程失敗」當成「改動已驗證」，在編譯前就著陸，而且失敗後沒有撤回。
  - 新增 codegen 輸出時，沒有同時把生成檔產生出來。
  - 新的 include 沒有對照模組依賴。
- **通用修正**：
  - UE 改動先放在工作區，至少要用 `Build.bat OmGameEditor` 或 `build_ue_moba --build-only` 編譯通過才著陸。
  - 建置因其他原因跑不到編譯時，改動留在工作區，不著陸。
  - 新增 codegen 檔案時，同一步重跑 content codegen。
  - 新增 include 時，檢查該模組的 Build.cs。
- **避免**：
  - 區域變數不要用 UE 基底類別已有的成員名稱（`Layers`、`Owner`、`Player`、`World`、`Tags`…）。
  - 著陸清單裡的每個檔案都要確認被編譯過。

## E102 著陸閘門沒有擋住：檢查失敗後仍然著陸，未編譯的 UE 原始檔放進 live 樹（2026-10-26，英雄 spotlight）

- **現象**：
  - 我把 `RoleBotMatchPlan` 的新欄位 `showcase` 著陸時，同一行指令先跑 `cargo check`，再無條件執行 land3。check 已經回報 `base_content` 測試裡有 12 個 struct literal 缺 `showcase`，卻照樣著陸。live 的 `base_content` 測試因此約 2 分鐘無法編譯，補上欄位後才恢復。
  - 新的 UE 檔案（`OmSpotlight.h/.cpp`、`OmSpotlightTests.cpp`）在還沒編譯前，我就直接寫進 live 的 `omfue`。UBT 會編譯目錄裡所有原始檔，其他 session（ward、main）的 UE build 可能被我未驗證的程式碼弄壞。發現後已移到 `target/session-g1/ue_ws/new`，等編譯通過再著陸。
- **根因**：把「驗證」和「著陸」寫在同一行、用 `;` 連接，沒有依驗證結果決定要不要著陸。另外把「新檔案沒有合併衝突」誤當成「新檔案可以直接放進 live」，忽略了 UBT 會自動收錄目錄裡的檔案。
- **通用修正**：著陸一律是獨立的一步，只在前一步的 exit code 為 0 時才執行。UE 新檔案先放在工作區，`build_ue_moba --build-only` 通過後才複製進 `omfue`。修改既有 UE 檔案時，用同一套 base／work 工作區，最後 3-way 合併。
- **避免**：給 struct 新增公開欄位前，先用 `rg "Type *\{"` 列出整個 monorepo（`omoba-core`、`omb`、`scripts/base_content`、`omb/tests`）所有的 literal 一起補上，並在著陸前對每個受影響的 crate 跑 `cargo check --tests`。任何「檢查 → 著陸」的指令都要以 exit code 串接（`&&` 或分開執行），不能用 `;`。
- **再犯（23:10）**：為了確認 spotlight 修正能編譯，我直接跑 UBT `Build.bat OmGameEditor`，但當時有其他 session 的 `UnrealEditor-Cmd` 自動化測試正在執行。我只把 tasklist 印出來，沒有拿它當閘門，結果 link 遇到 DLL 被鎖（LNK1104）；編譯本身已由別人的建置完成，所以沒有造成損害。規則：任何 UBT 或 build_ue_moba 都要用「`tasklist` 裡沒有 UnrealEditor」當前置條件，條件不成立就不執行（例如 `while tasklist | grep -qi UnrealEditor; do sleep 15; done && ...`），不能只是印出來看。

## E105（原誤標為 E102） shadow 樹裡的 junction 指回 live，對 shadow 子目錄 `rm -rf` 會刪掉 live 檔案（2026-10-26，技能特效）

- **現象**：
  - 為了讓 shadow 的 codegen 用到最新依賴，執行了 `rm -rf $SHADOW/omoba-sim/src && cp -r omoba-sim/src …`。
  - live 的 `omoba-sim/src` 10 個檔案因此被刪除，cp 也失敗。
  - 約 3 分鐘後以 `git checkout -- omoba-sim/src` 還原。還原前已比對過：所有 session shadow／blind tree 的副本（最新一份 21:15）與 HEAD 只差 CRLF，所以 live 沒有未提交的修改遺失。
- **根因**：
  - `target/session-ward/shadow/` 下有多個 junction 直接指回 live：`omoba-sim`、`docs`、`tools`、`specs`、`log4rs`、`omfx`、`eui`、`data` 等，`scripts` 也是。
  - 對 junction 內的路徑做刪除，實際刪的是 live。
- **通用修正**：
  - 先用 `cmd /c dir /AL <shadow>` 列出 junction。
  - junction 底下一律不刪除也不覆寫；需要較新的版本時，直接使用 live（junction 本來就是 live）。
  - 只有非 junction 的實體目錄才同步。
- **防範規則**（主 session 決定）：shadow 不得用 junction 或符號連結指向 live 樹；要同步只能複製。現有 session-ward shadow 裡的 junction 視為待移除，在移除之前，對 shadow 內任何路徑做刪除或覆寫前，都必須先用 `dir /AL` 確認。
- **避免**：
  - 對 shadow／worktree 之類的樹執行 `rm -rf` 前，必須先確認目標不是 reparse point。
  - 刪除前先確認 live 是否還有對應的版本控制副本可以復原。

## E101 vfx_mcp 工具包對不上專案安裝的 UECP 2.0.6：port／API 不同，且外掛未啟用授權，第 7 次寫入起全部被擋（2026-10-26，技能特效）

- **現象**：
  - `ref/vfx_mcp/Tools/uecp.py health` 回報 30000–30009 都連不到。
  - 換到正確 port 之後，`vfx_build.py` 呼叫 `niagara`／`material`／`asset_management`／`python_tools` 都回「Unknown command type」。
  - 補上相容層後，又出現兩個擋寫入的檢查：
    - 每次開 editor 後，第一個寫入指令被拒：「editor was restarted」。
    - 第 7 個寫入起全部被拒：「MCP runtime gate blocked … telemetry cache is not currently valid」，但 `original_response` 其實是成功的。
- **根因**：
  - 工具包是對 UECP 1.7.3 寫的：port 30000、有 session token、有 umbrella 工具。`omfue/Plugins/BpGeneratorUltimate` 裝的是 2.0.6：port 9877、沒有 token、只有扁平工具，也沒有 `create_asset`／ParticleMaterial 這個型別。
  - 2.0.6 要求每次開 editor 後先收到一個唯讀指令，才開放寫入。
  - 外掛有線上授權驗證（`FTelemetryManager`）。尚未收到驗證伺服器回應時只放行前 6 次寫入；`%LOCALAPPDATA%\UltimateCoPilot\settings.ini` 沒有授權 token，所以之後的寫入全部被擋。
- **通用修正**（只改 `ref/vfx_mcp/Tools/uecp.py`；工具包 README 允許通用化修正）：
  - port 自動偵測：30000 之後再試 9877–9886；沒有 token 檔時用 `none`。
  - umbrella 呼叫若伺服器沒列出該工具，就轉成同名的扁平工具 `action`。
  - `create_asset` 依型別對應到 `create_niagara_system`／`create_material`。ParticleMaterial 改用 `execute_python` 加 `MaterialEditingLibrary` 建出同一張圖。
  - init 時先送唯讀的 `ping`。
  - `delete_asset` 遇到 destructive-op 確認時，改用 `EditorAssetLibrary` 刪除。
  - 授權檢查**不繞過**：這是外掛的授權機制，必須由使用者在 editor 的 UECP 設定中登入或啟用授權。
- **決策**（2026-10-26）：
  - 使用者表示「UECP 授權 我來處理 你先做別的事」。
  - 方案 B（自研 Niagara 產生器）暫停。授權啟用前只做不需要 UECP 的部分：runtime 接線、VFXSpecs 規格（沿用 vfx_mcp 的 VFX_SPEC 格式）、Grok 素材。
  - 授權啟用後回到 vfx_mcp 流程。
  - 21:42 外掛被換成 1.7.3（使用者本人操作）；1.7.3 一樣有 FTelemetryManager 線上授權檢查。
- **另記**：`aivfx_editor.py` 已複製到 `omfue/Content/Python/`。`vfx_build.py` 的預覽與擷取 helper 是從「目前開啟專案」的 Content/Python 載入的，沒有別的放法。這不是 repo 的工作流程腳本，不會被 `scripts/` 呼叫。
- **避免**：
  - 用外部工具包前，先比對外掛版本、port 與工具清單（`uecp.py list`）。
  - 必須用帳號或授權啟用的外部工具，要先確認啟用狀態再排進自動化流程。
  - 不要用重開 editor 的方式重置免費次數。


## E100 舊 TD Blueprint 的 PrintString（「Tower priority rejected」）漏到 MOBA 玩家畫面，商店 modal 也蓋到指令列（2026-10-26，shop-grid-1）

- **現象**：
  - 實機截圖左上方隊友卡 F2 上浮著青色英文「Tower priority rejected」。
  - 商店 modal 是固定 620×480 置中，在 1280×720 下底邊蓋到指令列。
  - 每張卡片都有「合」，包括沒有配方的基礎道具，畫面很雜。
- **根因**：
  - 舊 TD 的 `RustBP/System/BP_PlayerController.uasset` 把 **P** 綁成 tower priority；MOBA 也用 P 開商店。autopilot 或玩家按 P 時，TD 指令被拒絕，Blueprint 用 PrintString 把 debug 文字印到畫面上。PrintString 是 UE 給開發者看的 on-screen message，不是玩家提示。
  - 商店 frame 只依整個 viewport 做 fit，不知道 HUD 頂部列和指令列的位置。
  - 配方按鈕沒有依道具是否有素材來決定是否顯示。
- **通用修正**：
  - **debug 訊息**：玩家 renderer（`!GIsEditor`）在 `AOmPlayerController::BeginPlay` 關閉 `bEnableOnScreenDebugMessagesDisplay`，EndPlay 時還原；`-om-dev-screen-messages` 保留給本機除錯。editor 與 automation 不受影響。
  - **商店位置**：新增 `FOmHudModalFit::ShopFrameInBand`，商店 frame 放在 header（含隊友卡）下緣與指令列上緣之間（`GetShopBandInsets`），格子區在內部捲動；空間太窄時退回原本的 fit。ResponsiveHud 在 1280×720 與 1920×1080 檢查商店不蓋到指令列與隊友卡。
  - **配方標記**：「合」只出現在有 recipe components 的卡片上，ShopIconGrid 已補上「無配方的卡片沒有合成按鈕」。
- **待辦**：舊 Blueprint 的 P 綁定仍會在 MOBA 送出 TD 指令（被拒絕、沒有影響），要在 Editor 修改 BP_PlayerController，讓 MOBA 不啟用 TD 的按鍵。
- **避免**：
  - 玩家看得到的文字只能來自 HUD 的在地化提示；開發用的 print 一律不得出現在玩家畫面。
  - modal 尺寸必須依實際可用空間（扣掉 HUD 保留區）計算，不能只依 viewport。

## E099 商店改成 LoL 式圖示格後，換行寬度晚一個 frame 才更新，窄視窗的配方按鈕定位到捲動區之外（2026-10-26，shop-grid）

- **需求**（使用者原話）：「購買欄位應該要像 LOL 那樣只放圖跟價錢，滑鼠 hover 上去才顯示詳細內容」。
- **現象**：
  - 改用 `SWrapBox.UseAllottedSize(true)` 排列卡片後，ResponsiveHudLayout 的「Narrow production shop opens the long recipe」失敗：按鈕 Y=443，在捲動區 top=497 之外。
  - ShopUndoProjectionAndPin 的「narrow large text」也失敗。
- **根因**：
  - `SWrapBox` 只在 Tick 時把 PreferredSize 換成實際分配的寬度，所以第一次 prepass 沿用舊的換行寬度；捲動與點擊位置都用了晚一個 frame 的 geometry。
  - 另有兩個測試仍斷言名稱顯示在列上，這是舊設計的假設。
- **通用修正**：
  - **卡片排版**：
    - 卡片的 PreferredSize 改綁 `MeasureShopForegroundWrap()`（扣掉捲軸空間），也就是 SOmShopFrame 本身 fit 出來的內寬，在 prepass 就已正確，`UseAllottedSize(false)`。
    - 卡片只顯示 icon 與總價。價錢顏色：可買是金色、金幣不足是紅色、商店目前不能交易是灰色（`ShopBuyAffordability`）。
    - 配方改成卡角的「合」。
  - **hover 內容**：tooltip 使用 `BuildShopBuyCardTooltip`，內容包含名稱、價錢、描述、合成素材與合成費，以及目前不能買的原因。
  - **相容性**：所有 tag（`OmShopBuy.<id>`／`OmShopIcon`／`OmShopRecipe`／`OmShopSell`）都保留，autopilot 與既有測試不受影響。
  - **測試**：新增 `Om.Runtime.ShopIconGrid`；舊測試改為檢查 tooltip 有名稱、卡面沒有名稱。
- **避免**：
  - 需要在 prepass 就決定 geometry 的 Slate 容器（捲動、點擊、fit 測試），不要依賴 `UseAllottedSize`，而要綁定已知的 fit 寬度。
  - 改 UI 版型時，同時檢查哪些測試把舊的文字位置當成功能本身；功能本身指的是資訊仍然可以取得，不是寫在哪裡。

## E098 補師型 autopilot 死太多：危險記憶太短、後退時仍施法、跟兵線用路點近似、點不到就 attack-move（2026-10-26，regress-50-oichi）

- **現象**：build-50 的回歸局 regress-50-oichi（oichi vs omatsu，quick map，9.7 分），補師型的 oichi 死了 11 次（7 次死於 omatsu，其餘死於小兵），其他英雄都在 0～4 次。
- **根因**（從 trace 的 `health`／`decision`／`submit` 逐一找出）：
  1. heavy fire 的記憶太短：2 秒窗口內沒有新的傷害，就被當成安全。一輪集火之後，下一個決策就恢復施法，`cast_w_self`（victory_blessing）在兵線正前方施放；施法會讓英雄停下來，下一輪集火接著打到。
  2. 兵線 aggro 的後退與避開英雄的後退仍允許施法；被動的 pre-fight buff 讓她留在已經拒絕的交戰裡。channel／tether 只會被緊急撤退打斷。
  3. 「跟著自家兵線走」允許英雄領先自家小兵一個路點；lane 上沒有自家小兵時則完全沒有限制，因此她會在轉角先撞上敵方兵線。
  4. 判斷「是否在 lane 上」以「距最近路點 ≤700」計算。quick map 的路段很長，在路段中間交戰的小兵離所有路點都很遠，因此被判成「lane 上沒有自家兵」，英雄被卡在基地塔附近（support-survival-1：兩位英雄 10 分鐘內幾乎沒有交戰，比賽撞到 time box）。
  5. 兵線接近時的後退一律退一個路點；在長路段上可能一路退回基地。又因為它與「跟著自家兵線走」互相矛盾，英雄會來回走而不交戰（support-survival-3：omatsu 整場 0 次攻擊、Q／W 沒有施放）。
  6. 攻擊目標點不到時改用 attack-move 到最近的可見點，會走進敵方兵線。support-survival-4 中，7 次死亡有 5 次在死前 40 行內出現 `attack_move_clamped`。
  7. 地圖邊緣的 lane 在 HUD 底下，可見的直線步長很短，撤退因此很慢（omatsu 從後面追上）。
- **通用修正**（OmAutoPilot，以卡片的 ally/self 屬性與威脅判斷，不針對特定英雄）：
  - 一輪重擊之後 3 秒內維持撤退（`FSurvival::bUnderFire`）。
  - 施法許可：撤退且附近有敵人時完全不施法；撤退但附近沒有敵人時，只允許 ally/self 卡（`CastPermission`：None／AllyOnly／All），補師在回家途中仍會治療。施法後立刻重新下移動指令。
  - channel（包含 tether）在需要撤退時就放棄；短的施法前搖仍然只會被緊急撤退打斷。
  - 兵線位置改用 route polyline 投影（段索引 + 比例，`RouteProgressOf`）。英雄只有在 lane 上有自家小兵位於自己前方時才前進；lane 上沒有自家兵時，不超過自家最前面的塔。
  - 只有在沒有自家小兵位於英雄前方時，才因為看到敵方兵線而後退。
  - 所有戰術性後退（兵線 aggro、拒絕交戰的英雄、等待兵線、無護送的塔、寡不敵眾）改為沿 route 後退 450（`StepBack`），並保留 lane 進度；生存撤退仍退兩個路點。
  - 攻擊目標點不到時，recentre 一次；仍然點不到就放棄那次攻擊，不再 attack-move。
  - 地圖邊緣的可見步長小於 300 時，改點最接近路點的可見點。
- **回歸測試**（`Om.Runtime.AutoPilotSurvival`）：danger hold、CastPermission 三種情況、後退時不施法、跟在兵線後面（領先一個路點仍算在前方）、長路段中間的小兵、空 lane 停在自家最前面的塔、看到兵線時跟隨或後退、StepBack 固定距離且可跨路點。`build_ue_moba --build-only` 成功，`Om.Runtime.AutoPilot*` 5/5 通過。
- **實機**（quick map，oichi vs omatsu，600 秒 time box，`target/mvp-a3/support-survival-*`）：
  - 修正前 regress-50：oichi 11 死、omatsu 4 死，9.7 分。
  - 中間版本：survival-1（oichi 1、omatsu 1）撞到 time box，原因是根因 4；survival-2（0／1，7.7 分）victory_blessing 沒有施放，因此加入 AllyOnly；survival-3（0／1，5.1 分）omatsu 的 Q／W 沒有施放，原因是根因 5。
  - 最終版本：
    - survival-4：oichi 7、omatsu 1，9.5 分，PASS；
    - 加入「不再 attack-move」後的 survival-5：oichi 5、omatsu 0，撞到 time box，PASS；
    - survival-6：oichi 3、omatsu 1，8.2 分自然結束，PASS 11/11，0 mismatch。
  - 結論：oichi 的平均死亡次數從 11 降到 3～7，survival-6 落在其他英雄的 0～4 範圍內。仍有變異：死亡多發生在比賽後期，敵方 super minion 推進到我方基地的時候。
- **風險（未解決）**：quick map 的比賽長度落在 5～10 分，survival-1 與 survival-5 撞到 600 秒 time box（修正前的 regress-50 也已經是 9.7 分）。英雄變得比較謹慎之後，推塔節奏變慢；比賽長度要另外調整（地圖或推進規則），不應該靠讓 autopilot 冒險來縮短。
- **其他**：建置期間，別的 session 正在修改 OmRtsCameraPawn 相機欄位的存取權限，OmMinimapModel.cpp 與 OmCameraInteractionTests.cpp 曾暫時無法編譯。等檔案 5 分鐘沒有變動之後再建置，就成功了；那些檔案沒有被動到。
- **避免**：
  - 生存規則要用「最後一個指令」與「施法是否讓英雄停下」來檢查，不能只看決策結果。
  - 任何「點不到就換一種指令」的備援，都不能讓英雄朝敵人移動。
  - 「在 lane 上」與「前後」要以路線本身（polyline）計算，不能用路點近似。

## E097 小地圖左上／右下與主畫面相反：點右下卻移到左上（2026-10-26，omfue 小地圖方位）

- **現象**：使用者回報「地圖左上跟右下跟實際位置是相反的，我點右下他就移動到左上」。兩邊主堡（左下、右上）與中路對角線看起來正確，但上路／下路、野區與點擊位置沿主對角線鏡射。
- **根因**：`FOmMinimapModel::Project/Unproject` 固定把 backend +X 畫成小地圖右、+Y 畫成上。主畫面的 RTS 攝影機（`AOmRtsCameraPawn`，pitch -60、yaw 0）看向 +X，所以螢幕上方是 world/backend +X、螢幕右方是 +Y（UE 左手座標，backend 與 world 同軸只差 `WorldUnitsToCm`）。兩者差一個轉置（沿主對角線鏡射），對角線上的主堡不變，左上與右下互換。繪圖（地形、路線、野區、霧、單位、ping、視野框）與輸入（左鍵鏡頭跳轉、右鍵移動、Alt 點 ping、地面施法、autopilot `minimap_move`）全部走同一組 Project/Unproject，所以一起錯。
- **通用修正**：
  - `FOmMinimapStatePayload` 新增 `ScreenRight`／`ScreenUp`（backend 單位的主畫面螢幕右／上在地面上的方向）；Project/Unproject 改為依此基底投影（以邊界中心為原點，旋轉後半徑決定縮放與 letterbox），未設定時退回 `AOmRtsCameraPawn` 預設 yaw 的基底（`ViewBasisFromYaw`），不寫死翻轉。
  - `ViewBasisFromProjection`：由主視角的 inverse view-projection，把螢幕中心與其右、上偏移點反投影到地面求方向，正交化並吸附軸向雜訊。`UOmCommandBarWidget::NativeTick` 每 tick 在算視野框時一併量測，交給 `SOmMinimap::SetViewBasis`；`SOmMinimap::SetState` 會把量到的基底蓋到每個新 frame payload，基底實際改變時才結束拖曳 capture。
  - `TrySampleMinimapPoint`（ping、鍵盤地面施法）改用 widget 的 state，與繪圖同一個基底。
  - `SOmMinimap` 的矩形（地形、霧、野區）改用 `ProjectRect`（四角投影後取外框），不再假設 (min.x,max.y) 是左上。
  - `AOmRtsCameraPawn` 新增 `GetCameraPitchDegrees/GetCameraYawDegrees/SetCameraOrientation`。
- **驗證**：
  - 新測試 `Om.Runtime.MinimapCameraOrientation`：以真實 camera pawn 的 camera component 建 view-projection，yaw 0／90／37 各自驗證「主畫面四個象限的地面點畫在小地圖同一象限」「點小地圖四角後，目標在主畫面同一方向」、round trip、軸向 yaw 下世界邊界角精確對到小地圖角、未設定基底與 yaw 0 量測一致、widget 保留量測基底。
  - 修正原本把錯誤方位寫死的期望：`NativeMinimapInput`（letterbox 尺寸改 300x132）、`MinimapTerrain`、`NativeMinimap`、`MinimapPreferences` 的邊角案例（`OmCameraInteractionTests`）。
  - `build_ue_moba.lua --build-only` 成功；headless `Om.` 全套 292/292 通過。
  - 實機 `target/mvp-a3/minimap-orient-1`（saika_magoichi vs date_masamune，lane_v0，150 秒）：p1 `autopilot-p1-shot-002/003` 英雄從己方主堡沿螢幕向右的下路前進，小地圖視野框從左下角沿下緣向右移；p6 `autopilot-p6-shot-002` 英雄在地圖右緣的直路上、主堡在上方，小地圖視野框在右緣靠右上角。方向與主畫面一致。
- **避免**：任何「world→2D UI」的投影（小地圖、指示器、預覽）都要從主攝影機的實際螢幕基底推導，不要假設 backend +Y 是螢幕上方；測試要至少有一個以真實攝影機投影比對象限的案例，而不是只測 round trip（round trip 在鏡射時也會通過）。

## E096 quick map 比賽縮短後英雄到不了 6 級：地圖新增 hero_xp_scale（2026-10-26，quick map 成長）

- **現象**：E094 讓 autopilot 打得更穩之後，quick map（three_lane_quick_training）的比賽縮短到約 4～5.5 分鐘。英雄在比賽結束前到不了 6 級，split59 與 after-12 的 R 都是 INCONCLUSIVE_RUN_SHORT；after-12 的 asai 在 4.0 分時只有 4 級。
- **需求**：quick map 的目的是在 10 分鐘內驗完整套技能。英雄成長要跟上比賽長度，但不能動到金錢與其他規則，也不能影響正式地圖。
- **通用修正**：
  - 地圖 template 新增可選的 `hero_xp_scale`：範圍 0.1～10，必須是 0.001 的整數倍；runtime 以整數 permille 套用，維持 deterministic。
  - lua_content.rs 負責驗證；build.rs 產生 `moba_map_hero_xp_scale_permille(id)` 與 `MobaMapConst::hero_xp_scale_permille()`。不在 `MobaMapConst` 加欄位，避免改到各處的 literal。
  - `SingleLaneConfig.hero_xp_scale_permille`（預設 1000，驗證 100..=10000）。三個入口都接上，各自有測試，避免重犯 E055 只接一處的錯：server_config（human three-lane）、bots/plan.rs（role plan）、moba_headless `--map`。
  - 套用點只有兩處，涵蓋所有英雄經驗值：`credit_moba_player`（擊殺、助攻、野怪，含死亡錢包）與小兵附近共享 XP（先乘總額，再依人數 floor 均分）。金錢不變。
  - state hash 只在 scale≠1000 時加入，未縮放地圖的既有 hash 不變。
  - replica 端的等級與經驗值來自 authority 的 CommittedProgression，自己不計算，所以不需要另外傳值。
  - quick 設 2.5。依據：4.0 分鐘時最慢的人類席只有約 364 XP（4 級），6 級需要 744，至少約 2.0x，留餘裕取 2.5。
- **驗證**：
  - template-ids 測試通過，quick 2500、正式地圖 1000、未知地圖 1000；core 1176、omb 全過；base_content 的 XP 倍率測試涵蓋 1／2 人分享、0.5x／0.1x、金錢不變、範圍驗證。
  - 新增 `role_bot_quick_map_scaled_xp_keeps_team_replicas_identical`（quick map、10 bot、7200 ticks），0 mismatch。
  - 實機 quick-xp-1（build-50，inahime vs asai，`target/mvp-a3/quick-xp-1`）：7.5 分自然結束，inahime 11 級、asai 14 級。兩個 R 都有施放（shadow_meld cast_ok=4、sky_fang cast_ok=6），check_hero_live_acceptance PASS，0 mismatch、0 termination。endless_fang 要 16 級才自動學，10 分鐘內本來就到不了，屬於預期。
- **未驗證**：lua_content 的驗證單元測試需要 `runtime-lua-content` feature，目前 runtime_content.rs:767 有別人改到一半的 `[u8; 5]` loadout 編譯錯誤，所以沒辦法跑；build.rs 已用同一套驗證擋過 quick 的 2.5。
- **避免**：調整 live check 的「時間預算」時，要同時檢查英雄成長是否趕得上，例如 6 級 R 所需的累積 XP；不要改金錢或怪物數值來間接達成。

## E095 windup 的 origin 取自 replica 預測的 tick 中途位置，權威位置校正後被判成「移動」而取消，己方 replica 少了冷卻（2026-10-26，autopilot-strength-after-9）

- **現象**：autopilot-strength-after-9（quick map，p1 inahime 對 p6 asai_nagamasa）tick 7560 出現 team 2 observer mismatch：asai 的 HERO 元件中，權威端有 `b08_fang_slash` 冷卻 6929/7168，replica 沒有。log 中這次施放只有權威端的 `[scripting] cast ok`，replica 那行不見了。
- **重現**：用 E086 的 `replay_accepted_inputs` 重播，第一次就重現同類問題：tick 6504 team 1 replica 少了 inahime 的 `a02_wind_prayer` 冷卻。暫時加診斷後確認：replica 在 6502 以 `pos≠origin` 取消 windup。replica 的 origin 是 (671757, 3901392)，位置與權威端 origin 都是 (678794, 3902891)。
- **根因**：
  - 有 `cast_time` 的技能（W 0.1 秒、Q 0.2 秒）會先進 windup。`Windup.origin` 取的是 script dispatch 當下（tick 中途）的施法者位置，之後每 tick 只要 `pos≠origin` 就取消。
  - 權威端在 dispatch 前，英雄已經走完本 tick 的一步；己方 replica 的位置是預測值，晚了一步（約 7 單位，和它只能預測的單位碰撞）。step 後 PostStep `Movement` fact 把位置校正為權威值，下一 tick 就被判定「windup 期間移動」而取消。replica 沒有放出技能、也沒有開始冷卻，權威端則正常放出。
  - 和 E071／E083 同類：拿預測的 tick 中途狀態，去比對之後的已提交狀態。
- **通用修正**：`cast_windup::anchor_started_windups`：windup 的 origin 定義成「開始那個 tick 結束時的已提交位置」，兩端用同一條規則。權威端在 `finish_moba_match_tick` 的 `synchronize_preparation` 之前呼叫，replica 在 `apply_authoritative_movement_outcomes` 之後呼叫。每個 windup 只錨定一次（`Windup.anchored`），之後的實際位移仍然會取消施放。
- **驗證**：
  - 修正後完整重播通過：3605 筆輸入、26018 tick，從 6480 起逐 tick 比對，全部一致。
  - 新測試 `windup_parity::a_windup_started_where_the_replica_mispredicted_the_casters_pose_releases_on_every_replica`（inahime W、asai Q；施放的那個 tick 讓權威端位置和 replica 預測差 7 單位）：修正前 team 1 HERO 有 49 列不一致，修正後兩隊逐 tick 一致。
  - `cast_windup::windup_origin_is_the_committed_pose_of_its_start_tick` 也驗證：只錨定開始那個 tick，之後被位移仍然會取消。
  - release base_content 539、omoba-core 1176、omobab 196、client-runtime 167 全數通過；著陸後 `build_ue_moba --build-only` 成功（重新編譯 omoba-core／base_content，bridge 和 base_content stage SHA 驗證通過）。尚待主 session 用 consistent build 重跑 inahime 對 asai。
- **過程中的錯誤**：
  - 初版只用 `last_tick==tick` 判斷「開始的那個 tick」。但 `last_tick` 每次 advance 都會更新，結果每 tick 都重新錨定，位移取消規則等於失效。寫單元測試時發現，改成一次性的 `anchored` 旗標，並重跑重播。
  - 第一版 parity soak（兩人互相走近、同時施放）在沒有修正時也通過，抓不到 bug，已換成上面這個能確定重現的測試。
- **避免**：任何「在某個 tick 擷取、之後拿來比對」的狀態（windup origin、通道錨點、計時起點），在 replica 上都要以該 tick 的已提交值為準，不能用 tick 中途的預測值。新增這類狀態時，要用「權威端在擷取當下的值與 replica 預測不同」的確定性測試驗證；只靠 soak 碰運氣不夠。單元測試一定要包含「之後的 tick 不該再被修改」的反向斷言。

## E094 autopilot 人類席死太多：回城站著被打死、決策被 recentre 迴圈餓死、比自己兵線先走進敵方兵線（2026-10-26，next-month #21）

- **現象**：220 次 autopilot 死亡中有 68% 死於小兵，約一半在回城後 10 秒內死亡。同 quick map、同英雄的基準局：base-1（oichi vs tamako）8 死、5.0 分；base-2（inahime vs asai）11 死、6.0 分。
- **根因（逐一從 trace 查到）**：
  1. 回城在 35% 血量、只要 1200 內沒有敵方英雄就按。結果在敵方小兵中站著讀條，被打斷後 `bRecallIssued` 一直不會重置；回城剛按下時 HUD 計時器還沒出現，下一個決策的攻擊指令就把回城取消了。
  2. 撤退門檻固定 45%，而且必須有威脅才撤。小兵一發約 72，四隻 caster 一輪就打掉 550 血英雄的一半，門檻來不及觸發。
  3. **決策飢餓**：地圖邊緣的 bounded camera 無法把英雄置中，英雄落在下方 HUD 底下（可點擊區只到畫面高度 80%）。點擊點看不到就按 camera_focus_self 然後 return，一直重複，最長連續 26 次（約 20 秒）站著不動挨打。基準局就已經存在。
  4. 被畫面裁短的走路點走到之後，同一決策還要等 ReissueSeconds（2 秒）才會再點，每一步之間都停頓。
  5. 英雄走得比自己的兵線快，先在 lane 轉角遇到敵方兵線；敵兵進入視野時已經在開火，第一輪就打掉 227～344 血。
  6. 面對射程較長的敵方英雄（tamako）仍站在對方射程內打小兵；撤退期間的施法前搖／channel 會被移動取消。
- **通用修正**（OmAutoPilot，純函式 `AssessThreat`／`RetreatThreshold`／`ShouldRecall`／`Decide(…, FSurvival)`，只使用玩家可見資訊：自己 HUD 血量歷史、已揭露單位）：
  - 威脅評估：敵方英雄數、以 max hp 加權的小兵（super 算 4）、我方兵與英雄、無護送的敵塔射程、我方塔、「自己是哪些敵兵最近目標」的 aggro 權重、視野內的敵方兵線。
  - 撤退：門檻 0.25 + 0.2×敵方英雄 + 0.05×小兵權重，扣掉我方陪同，夾在 0.2～0.85；另外觸發撤退的情況：2 秒內失血達 20% max hp（heavy fire）、剩餘血量撐不過 2.5 個窗口（burst）、低於 40% 起進入回家補血、補到 85% 才回線（遲滯）、站進無護送的敵塔射程。
  - 回城：低於 45%（或正在補血）、2 秒內沒有受傷、不在家、1400 內沒有敵方英雄、900 內沒有敵兵／召喚物、不在敵塔射程 +300 內，才按回城。按下後等到 HUD 出現計時器（最多 1.5 秒）；計時器結束後可再按；burst 時放棄讀條。
  - 兵線紀律：自己是 ≥2 權重敵兵的最近目標時往後退一個路點；視野內有敵方兵線而身邊沒有自己的兵時不前進；這條 lane 上的我方兵都在後面時原地等兵線（`Kind=None`）；在我方塔下可以正常防守。
  - 換血：對方明顯比較健康（自己低於 80%，且對方血量比例高出 0.15 以上）、1 打 2、或在敵塔下時不開打；拒絕交戰的英雄若在其射程 +150 內，自己先退出射程。
  - 施法：撤退中不施法（升級照常）；已經開始的前搖／channel 只會被緊急撤退（burst 或低於 40%）打斷。
  - 輸入：recentre 1.5 秒內不重複按。點不到時，把投影像素夾回可點擊區後 deproject 到地面，點最近的可見點（攻擊則改用 attack-move）；走到被裁短的點後立刻點下一步；站在目的點上就不再點擊。
  - 新增 `health` trace（受傷時每秒一筆：血量、近期傷害、威脅數字、決策）。
  - 另外修正 `run_moba_split_local.lua` 的 all_finished：UE 正在寫入 trace 時讀檔會遇到 Permission denied，現在視為「還沒結束」，不會中斷整個 run（after-2 就是這樣斷掉的）。
- **驗證**：
  - `build_ue_moba --build-only` 成功；`Om.Runtime.AutoPilot*` 5/5 通過，新增 `Om.Runtime.AutoPilotSurvival`（威脅門檻、super 權重、burst／heavy fire／urgent、補血、兵線 aggro、視野兵線、跟隨我方兵線、塔、換血、回城安全條件）。
  - 實機（quick map、600 秒 time box，`target/mvp-a3/autopilot-strength-*`）：
    - after-11（oichi vs tamako）2 死（基準 8）、4.7 分；
    - after-12（inahime vs asai）3 死（基準 11）、4.0 分；
    - 合計 19 → 5（-74%）。兩局都自然結束、0 mismatch，連續 recentre 的最長紀錄從 26 降到 ≤4。
  - check_hero_live_acceptance：after-12 為 PASS／INCONCLUSIVE，未學到的 R 與基準局相同。after-11 中 oichi 的 R 在比賽結束前 6 秒才學到，沒有機會施放，屬於比賽太短，其餘 10 項 PASS。
- **過程中的錯誤**：
  - after-1 只加門檻與回城，結果 10 死、8.7 分，反而變差。只調門檻沒有用，必須先從 trace 找出「第一次大量失血時英雄在做什麼」。
  - after-3 以 `&` 起跑，以為已經被 shell 殺掉，又起 after-4，結果撞同一個 port。實機局一律用 run_in_background 起跑，並以 PID 確認是否還活著。
- **另案**：after-9 有 1 次 observer mismatch：team 2 replica 的 asai 沒有 `b08_fang_slash` 冷卻（authority 6929／7168）。與 autopilot 規則無關，交給 replica cast 結算追查。
- **避免**：autopilot 的問題要先量測「決策有沒有真的送出」（連續 recentre、同一決策被抑制），再調戰術參數；任何點不到的情況都要有不靠相機置中的出路。

## E093 沒有正式模型的 authored 英雄在實機上顯示成灰色方塊（2026-10-26，split58-portraits）

- **現象**：omatsu、oichi、inahime、asai_nagamasa、murakami_yoshikiyo、tamako 的生成 class 沒有 `NativeMeshAsset`，所以 `AOmUnitActor` 退回 diagnostic cube，實機看起來是方塊。例如 `split58-portraits/p1/autopilot-p1-shot-001-facing.png`。
- **根因**：codegen 只有在英雄本身寫了 `ue.native_visual.mesh_asset` 或舊式 render model 時，才會產生 skeletal 呈現。hero_author 產生的英雄沒有任何美術欄位，而且也沒有共用的人形預設可以套用。
- **通用修正**：
  - **fallback 規則**：`templates/moba_shared_hero_fallback.lua` 讓所有沒有 mesh 的英雄改用共用人形。人形從訓練英雄現有的 Quaternius rig 中，依英雄 id 的穩定 hash 挑選，帶 idle／move／attack／cast／hit／death 動畫，並沿用 auto forward 對齊；`heroes.lua` 在最後套用這條規則。
  - **區分正式美術**：
    - codegen 對 `shared_fallback` 產生 `NativeSharedFallbackMesh = NativeMeshAsset`。
    - `HasAuthoredNativeMesh()` 只有在 mesh 和 fallback 不同時才回傳 true，另外新增 `UsesSharedFallbackVisual()`。
    - Editor Blueprint 只要指定不同的 mesh，就會被視為正式美術；ResolveEditorArtClass 的優先順序不變。
  - **PublicHeroCombatRoster**：fallback 英雄同樣會跑完整的生命週期，但仍以 content-gap warning 列出。
  - **實機驗證**：`target/mvp-a3/fallback-humanoid-1`（oichi 對 tamako，150 秒 time box）。
    - 截圖中是人形（male_ranger），不是方塊。
    - facing_probe 顯示 actor yaw 與移動方向一致，mesh relative yaw 90 抵銷 skeleton forward −90。
    - move、attack、cast 都有被接受。
- **避免**：
  - 「沒有內容」的呈現預設值必須是可以玩、能夠辨識的共用資產（人形、會動），不能是除錯用的方塊。
  - 驗收時要分開列出「正式美術」和「共用 fallback」，讓還沒換模型的英雄持續被報告出來。

## E092 九位英雄頭像仍是共用 placeholder；Grok 生頭像常把人物畫成「畫中畫」（2026-10-26，hero portraits）

- **現象**：
  - `hud-assets-v1.json` 中 training_frost_mage、training_channeler、omatsu、oichi、inahime、asai_nagamasa、training_grappler、murakami_yoshikiyo、tamako 九列都是 `hero-placeholder-v1.png`（`placeholder=true`），指令列、隊友卡、計分板與選角都只顯示灰色占位圖。
  - 用 Grok Build（`grok --prompt-file`，headless，image_gen）生頭像時，20 次輸出中退回 11 次：
    - 人物被放進內嵌畫框或面板，下方還有一段底色或地板：training_frost_mage、murakami_yoshikiyo，以及 oichi 第 3、4、5 次。
    - 背景兩側出現較暗或較淺的直條，等於多了一塊面板：oichi 第 2、8 次。
    - 人物像一塊方形剪影浮在空白背景上：oichi 第 6 次。
    - 弓身橫過臉部，肩甲上還有像文字的紋樣：inahime 第 1 次。
    - 臉太小或低頭，HUD 尺寸下看不清楚：tamako 第 1 次、oichi 第 1 次。
  - oichi 越是在 prompt 裡寫「no rectangle／no panel／no vertical strips」，越常畫出面板。
- **根因**：
  - 「宮廷貴婦肖像」這類主題容易讓模型畫成掛在牆上的肖像畫。
  - 負面描述反而把「矩形」「面板」這些概念帶進畫面。
- **通用修正**：
  - 退回後重生時，改用正面描述構圖：袖子和肩膀被左、右、下三邊裁掉，花瓣和光點一路飄出每個邊緣，臉放大並位於中上方。
  - 拿掉會越寫越糟的負面字眼。
  - 必要時同時跑兩個候選，挑沒有框的那張。
  - 每次退回的原因都寫進 provenance 的 `rejected_attempts`。
- **驗證**：
  - 九張圖都直接看過，沒有框、沒有文字、主題正確。
  - 每張都是 1254×1254 RGB PNG，尺寸與既有頭像相同。
  - `hud-assets-v1.json` 有 18 張 authored 頭像，placeholder 頭像為 0。
  - `ue_hud_portrait_coverage_test` 共 57 checks，新增「真實 catalog 0 placeholder 頭像」檢查。
  - `build_ue_moba --build-only` 成功。
  - UE 測試 `Om.Runtime.HudAssetRegistry`、`HudAbilityIconAtlas`、`ResponsiveHudLayout`、`PlayableMvpHud`、`PlayerStatsPanel`、`ShopUndoProjectionAndPin` 全部 Success。
- **以後如何避免**：
  - 頭像 prompt 一開始就寫「主體越過畫面邊緣」的正面構圖，不要只靠 no-frame 這類負面詞。
  - 每張輸出都要親自看過，不能只看 grok 回報。

## E091 44 個英雄技能沒有 HUD 圖示，技能格空白；Grok 生圖常帶內框或符文字（2026-10-26，ability icons）

- **現象**：
  - `hud-assets-v1.json` 只有 26 筆 ability（舊 atlas），catalog 中 18 位英雄的 62 個技能裡有 44 個沒有對應列，`RefreshAbilityIcon` 找不到描述就畫 NoDrawType，技能格是空的。
  - sync／coverage 只檢查頭像，沒檢查技能，所以新英雄缺圖不會有任何警告。
  - 用 Grok Build（`grok --prompt-file`，headless，image_gen）生圖時，約三分之一的輸出不能用：畫面中有內嵌方框或較淺的畫布邊（a02_wind_drift、a02_venom_arrow、b04_flame_swiftness、c17_light_chain、lumen_repulse×2、b08_fang_slash×2），或者斧面、盾面、光束、塔基上有像字母的符文（b29_ground_slam、b29_quake_axe、c17_devotion、lumen_sentry×2）。另外 c17_divine_strike 左右有邊界，lumen_venom_orb 的斑紋像一張臉。總共退回 15 次，重生之後全部通過目視檢查。
  - 我用 Lua 改 `OmItemIconImportCommandlet.cpp` 時統一寫成 CRLF，但這個檔案的屬性是 `-text`、工作樹是 LF，結果 diff 變成整檔 1474 行；改回 LF 後只剩 30 行。
- **根因**：
  - 技能圖示只有 atlas 路徑，沒有像道具那樣的「每個 PNG 對應一個 UTexture2D 再發布 resource」的管線。commandlet 和 runtime registry 只接受 Items／Trinkets 兩個 root。
  - prompt 中的「icon」「vignette」「edges」會讓模型畫成卡片。
  - 改檔工具沒有保留原本的行尾。
- **通用修正**：
  - `ui_texture_import.lua` 新增 `ability` kind：
    - schema 是 `omoba.ability-icon-assets.v1`。
    - catalog id 取 `templates/abilities.lua` 的 include 位置，等於生成的 ability numeric id，commandlet 會驗證。
    - manifest 可以只涵蓋部分技能。
    - 每個 asset 都必須有 prompt、visual_review、tool。
  - `publish_registry_resource` 對 ability 會把 placeholder 列轉成 1×1 的 packaged 列，缺列時補上；如果原本是 atlas 美術，則拒絕覆蓋。
  - commandlet 與 `OmUiTextureImport.h` 新增 `/Game/Omoba/UI/Abilities`。`OmHudAssetRegistry` 只有帶 resource 的 ability 才當成 packaged 處理。
  - `ue_hud_portrait_coverage` 新增 `validate_abilities`／`ability_placeholder_entries`／`prune_ability_placeholders`。sync 與 `require_ready` 會讓每個英雄裝備的技能都有一列。缺圖時補上 `ability-placeholder-v1.png`（Grok 生成的中性灰晶），並印出 `WARNING`，不會再留白。
  - prompt 改成「full-frame cropped digital painting… outermost pixels same deep navy」，退回重生時再加上「single background layer」以及「no engravings/glyphs」。
- **驗證**：
  - Lua 測試：`ue_hud_portrait_coverage_test` 56 checks（含真實 catalog 0 placeholder）、`ui_texture_import_test`、`item_icon_import_test`、`ue_bridge_stage_test` 全部通過。
  - 匯入：`import_ui_textures --kind ability`，第一次匯入 44 個、存檔 44 個；第二次驗證 44 個、存檔 0 個；registry 發布 44 個。
  - `build_ue_moba --build-only` 成功，結果是「62 equipped abilities, 44 packaged, 0 placeholder」。
  - UE automation：`Om.Editor.AbilityIconResources`、`UiTextureImport`、`ItemIconResources`、`Om.Runtime.HudAssetRegistry`、`HudAbilityIconAtlas` 全部 Success。
- **避免**：
  - 新英雄的技能要先過 coverage。看到 WARNING placeholder，就要補 `Abilities/Source/<id>.png` 與 manifest，再跑 `import_ui_textures --kind ability`。
  - 每一張生成圖都要目視檢查有沒有內框和類文字符號。
  - 用腳本改檔前要先看 `git ls-files --eol`，保留原本的行尾。

## E090 range 為 0 的跳躍技沒有公布可達距離，指示器的圓畫在跳不到的位置，autopilot 的觸及距離也算錯（2026-10-26，next-month #15a）

- **現象**：E088 讓 `b29_quake_axe` 公布 500 的 area，但它的 `targeting.range` 是 0。實際是「朝指定方向最多跳 300（受地形截斷），再砸落點 500 內」。所以指示器的 500 圓畫在游標位置（可能跳不到）；autopilot 的 CastReach 只用 500，漏掉 300～800 之間的目標。
- **確認 handler**：No2_quake_axe 以 `swept_dash::plan(..., spec.leap)` 把位移夾在 `dash_max_distance`（300）內，落點才是砸地中心。超過 300 的點擊仍然跳滿 300，不會被拒絕。
- **不能直接把 range 設成 300**：windup 的目標檢查會以 range 拒絕更遠的點（OUT_OF_RANGE），等於改變 gameplay。
- **通用修正**：
  - content model 新增 presentation-only 的 `targeting_reach_key`：只允許 point 目標、每級 range 都是 0、per-rank 正值的 extras key。preview 每級多一個 `clamp_range`。
  - hero_author：range 0 的 point／direction 技能，如果 effects 有帶 `max_distance` 的 dash，就以該 dash 的 extras key（`dash_max_distance` 或 `dash_N_max_distance`）宣告 reach。`authored_heroes.lua` 回填 quake_axe 一行。
  - codegen 產生 `ClampRanges`，bridge 寫入 `PublishedClampRangeCm`。
  - 指示器：多畫一圈 reach，area 圓的中心夾在 reach 內（`ClampedPointAreaCenter`）。
  - autopilot：`CastReach` = reach + area radius（range-free 的跳躍：落點夾在 reach 內，效果再往外延伸）；有公布 range 時仍以 range 為準。
  - 掃描結果：所有 package 中，只有 `b29_quake_axe` 是「range 0 且有位移」。asai 的 flash_cut（1100）、date 的 flame_trail_dash（900～1200）都已用 range 公布位移距離；oichi 的 max_distance 是 tether，不是位移。
- **驗證**：
  - content-model 76、om_codegen 87、hero_author 12、template-ids 全部通過。
  - `build_ue_moba --build-only` 成功（quake_axe 每級公布 ClampRanges 300）。
  - `Om.Runtime` 240 個全部通過，包含新增的 AutoPilotAbilityRule（800 內施放、801 不施放）與 PointAreaPreview（reach 夾住落點）。
- **避免**：「range 0」在 package 裡代表「施放不受距離限制」，不是「沒有幾何」。會移動施法者的技能，要把位移距離公布為呈現用 reach；不要改 gameplay range，除非 handler 本來就拒絕更遠的點。

## E089 rematch 實機局用了標準地圖而且沒有 time box，round-1 打了 11 分鐘還沒結束（2026-10-26，rematch-1）

- **現象**：
  - 我啟動 managed rematch 時用了 `--recipe scripts/lua_data/moba_two_player.lua`，對應 `three_lane_training` 標準地圖，要 30 分鐘以上。
  - round-1 從 11:30 打到 11:41（tick 53000+）都沒有結束，違反「實機局 ≤10 分鐘」的規則。
  - 已用 TaskStop 停止我自己的背景 workflow。workflow Job 會一併回收它的 owned processes（server、runtime-p1/p6、unreal-p1/p6），結束後逐一以 PID 確認都已退出。
- **根因**：
  - 實機 split 都用 `moba_two_player_quick.lua`（`three_lane_quick_training`），並以 `--max-seconds 600` 設定時限；我改用 managed 入口時沒有沿用這兩點。
  - managed round 原本只能無限等待：finish timeout 和 managed bootstrap 互斥，而且 finish timeout 的語意是「時限到前沒看到結果就判失敗」，不是 time box。
- **通用修正**：
  - **time box 選項**：role workflow 新增 `--round-time-box-seconds 1..7200`。時限到了會寫 `round-time-box.json`（verdict 為 inconclusive），以 owned-process 紀錄回收本局，而且不再提供下一局。
  - **coordinator**：
    - 非最後一局逾時時，`completed.json` 的 verdict 寫 `inconclusive` 並停止 session。
    - bounded 最後一局本身就以 time box（`--managed-final-round-seconds`）作為預定結束。
  - **入口檢查**：`run_moba_session_ue.lua` 遇到 `--autopilot` 卻沒有 `--round-time-box-seconds` 時直接拒絕。
  - **驗收**：`check_managed_rematch.lua` 對 inconclusive 回報 INCONCLUSIVE（exit 2），永遠不會判 PASS。
  - 重跑時改用 quick recipe，並設定 600 秒 time box：`target/mvp-a3/rematch-1/session` 於 11:42:58 開始、11:51:54 結束（共 9 分鐘）。round-1 約 5.5 分鐘打完，round-2 在 180 秒時依預定結束，check_managed_rematch 判定 PASS。
- **避免**：
  - 任何實機入口都必須先確認地圖是 live-check map（quick），或者有 time box，兩者至少要有一個。自動化入口改為強制要求 time box。
  - 換一個新入口之前，先比對既有實機腳本的 recipe 與時限（例如 split 的 `moba_two_player_quick.lua` 和 `--max-seconds 600`），不要直接用預設值。

## E088 rust_handler 技能的 targeting.shape 沒有進入 HUD 卡片，autopilot 把 quake_axe 當成 range 0 的方向技（2026-10-26，split52）

- **現象**：
  - murakami 的 `b29_quake_axe`（direction、range 0、circle 500）卡片上的 `PublishedPointAreaRadiusCm` 是 0，E082 的 CastReach 拿不到 AreaRadius，只在 100 內找目標，整局沒有施放 W。
  - 另外 14 個 rust_handler 技能的 circle／line 也沒有公布。
  - data-driven 的 `lumen_flare`（reveal_area 450）與 `lumen_tremor`（point channel 半徑 340）同樣是 0。
- **根因**：
  - `ability_aim_preview` 只從 gameplay 契約推導範圍：`aim_geometry`、area／line／cone effect、linear projectile。
  - hero_author 只把 package 的 `targeting.shape` 攤平成 `extras.shape_radius`，沒有任何契約引用這個 key，line 的 `width` 連 extras 都沒有產生。
  - preview 也漏掉 `RevealArea` 與 point-anchor channel 的半徑。
  - 不能改用 `aim_geometry`：PointCircle／SweptPath 帶有 gameplay 語意（range>0、instant、`requires_movement`、zone／channel／projectile 互斥），而 direction 技的 range 是 0。
- **通用修正**：
  - **content model**：
    - 新增 presentation-only 的 `targeting_shape`（Circle／Line／Cone，每級正值並驗證上限）。
    - 只有 point 目標、而且沒有任何 gameplay 範圍時才填入 preview，並標記 `shape_declared_only`。
    - 遇到 gameplay 範圍時，一律以 gameplay 為準。
    - `RevealArea` 與 point channel 的半徑改為 gameplay point area。
  - **hero_author**：package shape 轉成 `targeting_shape`，並產生 `shape_radius`／`shape_width`／`shape_half_angle`（角度轉成弧度）extras，未知的 shape kind 直接拒絕。`authored_heroes.lua` 以 generator 本身的 `ability_entry` 回填，只補上這 15 行 shape 欄位。
  - **codegen 與 UE**：
    - codegen 產生 `bShapeDeclaredOnly`。
    - bridge 寫入 `bPublishedShapeDeclaredOnly`。
    - tooltip 對 declared 的 line／cone 標示「技能自訂判定」，不再套用 host 的「直線光束」／「遇地形停止」說法。
    - 沒有公布仍然是 0。
    - quake_axe 公布 500 之後，`bDirectional=false`，autopilot 改用 point area。
- **避免**：
  - package 的每個 authored 欄位都要有明確的消費者，或者明確標示只是說明文字；只攤平成 extras 而沒有任何 key 被引用，就等於丟棄。
  - 新增 effect／delivery 時，同時檢查 `ability_aim_preview` 是否涵蓋它的範圍。
  - 已知限制：direction 技的實際落點受跳躍距離限制（quake_axe 是 300），但 range 沒有公布，所以指示器的圓畫在游標位置。

## E087 英雄在泉水回血又被持續傷害打到時，team replica 保留扣血後的舊 HP（2026-10-26，split48／split52）

- **現象**：
  - split48 出現 4 次只差 HP 的英雄 PROPERTY mismatch。
  - split52 在 tick 12600、12720 出現同樣問題，rebase 後仍不一致，比賽被 observer-mismatch-after-rebase 終止。
  - 兩隊 replica 算出的錯誤值相同，差值約為 1 個 tick 的泉水回血（2040 raw）。
- **重現**：用 E086 的紀錄 `replay_accepted_inputs` 重播 split52，soak tick 12133 team 1 不一致：murakami 站在泉水旁，滿血，同時被 tamako 的 W 灼燒。authority 的 HP 是 759.67，replica 是 757.68。第一次重播就重現，重播工具端到端驗證成功。
- **根因**：
  - authority 每個 tick 送兩種 HP 事實：`CommittedVitals`（process_outcomes 中，剛扣血後）與 `CommittedEquipmentStats`（finish 中，泉水回血等 authority-only 結算之後，是 tick 結尾的絕對值）。
  - replica 的 `apply_disclosed_events` 卻先套用 `CommittedEquipmentStats`，再在事件迴圈套 `CommittedVitals`。結果剛扣血後的 HP 蓋掉了 tick 結尾的值。
  - 只有「同一 tick 既受傷又被 authority-only 結算改 HP」才會出錯，所以只在泉水附近被持續傷害時發生。這兩位盲測英雄的 W 灼燒與泉水剛好湊在一起。
- **通用修正**：`apply_committed_economy_and_equipment` 移到 `apply_disclosed_events` 的最後（在 CommittedVitals 與 PROP effect 之後），讓 tick 結尾的快照最後生效。split52 完整重播通過：1830 筆輸入、13609 tick，每 120 tick 比對，從 11000 起逐 tick 比對，全部一致。
- **附帶**：
  - `soak_match` 現在會把 PROPERTY 解成 [hp, max_hp, move, armor, magic]，不再只印長度。
  - 同一個 Tick error（例如比賽終止後的每個 tick）只記第一次，之後在第 2、4、8……次記摘要，結束時記總次數。
- **避免**：
  - 同一欄位有多個 committed fact 時，replica 的套用順序必須依「誰最晚」決定，tick 結尾的絕對快照永遠最後套用。
  - 實機 mismatch 一律先用 accepted-inputs 重播重現，再下結論。

## E086 實機 mismatch 無法離線重現：server log 沒有完整記錄被接受的輸入（2026-10-26，split48 後續）

- **現象**：split48 的英雄 HP mismatch，在 soak 裡用同一份 match plan 跑 37000 tick（含 rebase）都沒有重現。實機的 autopilot 輸入（AttackTarget、point cast 的目標點、商店購買與賣出）在 `player_input_tick` log 裡不完整：point cast 沒有座標，購買也沒有記錄，所以無法重播同一局。
- **根因**：除錯工具缺口。權威端最終消費的輸入（人類與 role bot 合併、fog 過濾、shop undo pin、item use seal 之後）從來沒有完整落地。
- **通用修正**：
  - `omb/src/state/accepted_input_log.rs`：設定 `OMOBA_ACCEPTED_INPUTS=<path>` 後，server 會在 `PendingPlayerInputs` 填好的那一刻，每個有輸入的 tick 寫一行 JSONL：ECS tick、local tick、比賽的 `elapsed_raw`（step 前），以及每筆 `(player, PlayerInput)` 的 prost 位元組（hex）。header 記錄 step_fps 與 seed。每行寫完即 flush，server 被殺也保留已完成的 tick；沒有輸入的 tick 不寫。
  - 啟用方式：`moba_role_launch.lua --accepted-inputs FILE`。`run_moba_split_local.lua` 預設為 server 寫 `<server output>/accepted-inputs.jsonl`，`--no-accepted-input-log` 可關閉。
  - 重播：`replay_accepted_inputs_keeps_team_replicas_identical`，設定 `SOAK_ACCEPTED_INPUTS=<jsonl>` 與 `SOAK_ROLE_PLAN=<match-plan.json>` 後以 dual replica 重播同一局。以 `elapsed_raw` 對齊，因為 server 的 ECS tick 從 lobby 開始算，soak world 不是。比較間隔預設 120（`SOAK_REPLAY_EVERY`），`SOAK_EVERY_TICK_FROM` 可縮小範圍。
- **避免**：
  - 實機驗收局一律保留可重播的權威輸入；log 只是摘要，不能拿來重建狀態。
  - 新增會進入 `PendingPlayerInputs` 的輸入種類時，不必改紀錄器（整個 PlayerInput 原樣存下），但要確認 soak 的 `admit` 會接受它。

## E085 百箭齊發被召喚交易規則整招拒絕，箭矢從未出現；kit soak 沒驗證每招真的生效（2026-10-26，split48）

- **現象**：
  - split48（murakami 對 tamako）出現 4 次只差 HP 的 observer mismatch：兩隊 replica 錯得一樣。
  - 追查時發現 tamako 的 R 在實機上從沒生效：handler 回報 cast ok，結算整筆拒絕，所以一支箭矢都沒放出。
  - 讓 kit soak 真的放出箭矢之後，在建立 shadow 的那棵樹上，tick 784 重現 replica 不一致：10 支箭矢的 `asd_count` 在 replica 多走一個 tick。原因是箭矢射的野怪營地 hard reset，在 tick 開頭被移除；replica 的箭矢 script 依舊有的狀態，還照常累加。
- **根因**：
  1. 施放結算對「有生成召喚物的交易」只接受 spawn、資源與施法者自身的效果，連「對自己剛生成的召喚物上 buff」與「所有 spawn 之後的效果」也拒絕。R 的 handler 單元測試只檢查 outcome、沒經過 settlement，所以沒抓到。
  2. 箭矢 asd_count 的分岔和 E083 同類：replica 會跑的召喚物 script，讀到的是只在 authority 改變的狀態（這裡是野怪 AI 在 tick 開頭的 reset），結果只是預測。E083 已讓所有帶 TAttack 的實體送 PostStep `CommittedAttack`，replica 在 step 後採用權威值；在含 E083 的樹上這個 soak 通過。我曾試過把野怪 AI 移到 tick 結尾，會改變所有營地時序，既然 E083 已用 committed fact 修正，就不採用。
- **通用修正**：
  - 召喚交易允許：對本筆交易生成的召喚物上 buff，以及位於最後一個 spawn 之後、目標不是施法者的 buff 或傷害（AddBuff、RawDamage、Damage、ScriptDirectDamage）。兩者都在擺放規劃完成後結算，不影響規劃依據的狀態。spawn 之間插入的效果，以及作用在施法者身上的效果（施法者的生命與資源撐起整筆交易），仍然拒絕。已補 `summon_casts_admit_their_summons_setup_and_trailing_effects_but_nothing_between_siblings`。
  - 測試：
    - tamako 的 package parity soak 斷言 burn／arrows／stun 都真的發生。
    - 新增 murakami＋tamako 的 role-bot 全場 soak，可用 `SOAK_ROLE_PLAN=<live 的 match-plan.json>` 重播實機計畫（map、roster、seed 規則）。
    - `soak_match` 新增 `SOAK_REBASE_EVERY=N`，模擬 observer rebase。
- **split48 尚未直接重現的部分**：只差 HP 的英雄 mismatch，用 split48 的完整計畫跑 37000 tick（含 rebase）都沒有重現。實機 autopilot 的輸入（AttackTarget、point cast 座標）server log 沒有完整記錄，無法重播。依 E083 的分析，最可能是 replica 端預測、而英雄 HP 沒被即時校正的同類問題。請在含 E083 與本修正的 build 重跑 split48 確認。
- **避免**：
  - 召喚類 handler 要用經過 settlement 的 soak 驗證，不能只看 outcome。
  - 新 kit 的 soak 要斷言每一招真的生效，否則 parity 在什麼都沒發生時也會通過。
  - 只在 authority 改變的狀態，若會被 replica 端 script 讀到，必須有 PostStep committed fact 校正（E083）。

## E084 同步 shadow 時 tar 保留了舊的 mtime，cargo 拿過期產物編譯，測試結果不可信（2026-10-26，omfx-decouple）

- **現象**：修 E083 前，我用 `tar` 把 live 的原始碼複製到 shadow。之後 client-runtime 在 omoba-core 內編譯失敗，報 `SUMMON_TARGET_BUFF` 不存在、`SummonStats` 沒有 `invulnerable` 欄位，但兩者在原始碼裡都在。
- **根因**：`tar -x` 保留了原檔的修改時間，而這個時間比 target 目錄裡過期的產物還舊。cargo 依 mtime 判斷要不要重編，所以把舊的 `omb-script-abi`／`omoba-template-ids` 產物當成最新。同一輪先跑完的測試也可能混用了舊程式碼。
- **通用修正**：同步完成後，對所有同步過來的原始碼 `touch`，強迫 cargo 重新判斷；然後整輪重跑測試，結果才算數（E083 的數據就是重跑後的）。
- **避免**：任何保留時間戳記的複製（`tar`、`cp -p`、`robocopy` 預設）寫進共用 target 的工作區之前，都要先更新 mtime。如果 cargo 報「原始碼裡明明存在的符號找不到」，先懷疑是過期產物，不要先改程式碼。

## E083 召喚物的攻擊計時器只在敵人進入射程時累加，己方 replica 依預測的敵人位置判斷，ATTACK 不一致（2026-10-26，split50）

- **現象**：split50-heroes（build-40，saika_magoichi 對 date_masamune）tick 9000 出現 team 1 observer mismatch：entity 422、canonical 0xb0000005f、kind=summon（雜賀眾 gunner），schema `0x464f4706` ATTACK。權威端 asd_count=973，replica 是 188，其他欄位相同；只發生一次，rebase 後修回。
- **重現**：`summon_facing_parity::a_turret_attack_clock_matches_in_every_replica_when_an_enemy_crosses_its_range_edge`。`lumen_sentry` 先在敵隊看不到的地方閒置 120 tick，接著敵方英雄走進視野，每 20 tick 跨一次砲塔射程邊緣。修正前 team 1 的 ATTACK 有 228 個 tick 不一致；修正後兩隊逐 tick、逐元件都相同。
- **根因**：
  - 召喚物 script（`saika_gunner`、generic `Emplacement`）只有在 `query_nearest_enemy` 找到射程內的敵人時，才會累加 `asd_count`。閒置時不會無限累加（推測不成立），揭露時也會從 baseline 帶入權威值，目標篩選也已經依揭露集合過濾。
  - 問題在於己方 replica 會跑同一段 script，但它收不到敵方玩家的輸入，敵人在本 tick 的位置只是預測值。敵人在射程邊緣附近時，權威端判定「在射程內」的 tick 和 replica 不同，計時器因此分岔。split50 差了約 46 tick（785 raw）。和 E071 的 FACING 問題同類。
  - 英雄的攻擊計時器每 tick 都有 PostStep `CommittedAttack` fact 校正；召喚物和其他攻擊者沒有。
- **通用修正**：`finish_moba_match_tick` 改成對每個帶 `TAttack` 的實體（英雄以外，目前是所有召喚物）送出 PostStep `CommittedAttack`（elapsed、sequence、phase，HERO_ABILITY 可見性政策），英雄的發送也改用同一個 `emit_committed_attack`。看得到該單位的 replica 在 step 後採用權威值，自己的預測不再影響揭露結果。MOBA 的小兵、野怪、塔用 `attack_remaining`，不帶 `TAttack`，也不會在 replica 跑攻擊 AI，所以不受影響。
- **驗證**：release base_content 537（含 saika／gunner parity soak）、omoba-core 1174、omobab 195、client-runtime 167 全數通過；著陸後 `build_ue_moba --build-only` 成功（重新編譯 omoba-core／base_content，bridge 和 base_content stage SHA 驗證通過）。尚待主 session 重跑 split50 同配置。
- **避免**：replica 端會跑的任何邏輯，只要讀到敵方單位本 tick 的位置（射程判定、索敵、轉向、計時），結果都只是預測。這類狀態要有 PostStep 已提交 fact 校正，新增會被揭露的狀態時，要一併確認它的已提交 fact 涵蓋所有實體種類，不能只做英雄。

## E082 沒有施放距離的範圍技能，autopilot 只在 100 內找目標，整局不施放（2026-10-26，split47）

- **現象**：murakami_yoshikiyo 的 W `b29_quake_axe`（跳躍最多 300 後，對落點 500 內造成傷害）學到 2 級，但 autopilot 從沒發出 cast_w，cast_ok 是 0。
- **根因**：package 的 `targeting.range` 是 0（效果以自身為中心），所以 HUD 不公布 range。`ChooseCast` 的觸及距離公式是 `max(Range, 100)`，只會在 100 內找目標；這張卡有 area radius，又不算 directional，還必須能點到目標。
- **通用修正**：新增 `FOmAutoPilot::CastReach`：沒有 range 但有 `PublishedPointAreaRadiusCm` 的卡片，以半徑（換算成世界單位）作為觸及距離；有 range 的卡片仍照 range；兩者都沒有時維持 100。`FSlotCard` 新增 `AreaRadius`。回歸測試寫在 `Om.Runtime.AutoPilotAbilityRule`。
- **避免**：判斷「能不能打到」時，要把卡片公布的所有幾何資訊（range、area radius、swept width、cone）都納入，不能只看 range；range 為 0 的意思是「以自身為中心」，不是「射程 0」。

## E081 英雄驗收局太早結束，大招還沒學就被判 FAIL（2026-10-26，split47）

- **現象**：split47（murakami_yoshikiyo 對 tamako，quick map）在 3 分 11 秒、tick 11442 就結束：一路兵線推到底，nexus 和基地接連倒下。兩位英雄只升到約 4 級，R（需要 6 級）的 learn 是 0，`check_hero_live_acceptance` 判兩個 R 都是 FAIL。同一張地圖的 split46 打了 6.5 分鐘，split43 打了 10 分鐘，局長差異很大。
- **根因**：判定工具只區分「10 分鐘內到不了的 auto_learn」和其他技能，沒有用「這一局英雄實際升到幾級」來判斷。局太短時，「沒學過」會被當成英雄或 autopilot 的錯誤。
- **通用修正**：autopilot 的 `match_finished` 事件加上 `hero_level`（取自 HUD 的 `level`）。判定工具讀這個值：技能沒學過、而且 rank 1 需要的等級高於英雄最終等級時，判為 `INCONCLUSIVE_RUN_SHORT`，整體判定為 INCONCLUSIVE（exit 2，需要重跑），不算 PASS 也不算 FAIL。等級已經達到卻沒學，仍然判 FAIL；trace 裡沒有等級時，維持嚴格判定。`check_hero_live_acceptance_test.lua` 新增測試。
- **避免**：驗收工具要先確認「這一局有沒有能力證明這件事」，再判對錯；證據不足時要明確標成無法判定，不能當成失敗或通過。
- **補充（2026-10-26，split-dev-4-smoke-2）**：英雄在比賽結束前陣亡時，HUD 的 level 是 0，`match_finished` 因此記成 `hero_level:0`。現在改成記錄整局看過的最高等級（`MaxHeroLevelSeen`）。以後狀態要取「整局累積值」時，不能只拿結束當下的 HUD 快照。

## E080 出生 rank 與重生從不執行 on_learn，被動永遠沒套上；autopilot 只升 Q，unit／方向技能目標不在畫面內時靜默卡住（2026-10-26，split45）

- **現象**：split45（inahime 對 asai_nagamasa）的 check_hero_live_acceptance FAIL。
  - 兩個 E 被動 learn=0。
  - asai 的 W flash_cut 學會了但從沒施放；R sky_fang 也沒施放。
  - 升級順序是 Q×3 → W → R → W，E 一次都沒升。
- **根因**：
  1. `generate.lua birth_rank` 讓第一級需求 ≤1 的技能出生就是 rank 1，所以 Q、W、E 出生都是 1 級。`apply_moba_birth_loadout` 只寫入 `ability_levels`，不送 `SkillLearn`，於是 `on_learn` 從沒執行：被動 stat buff 與 rank 標記都不存在。另外英雄死亡時 `remove_all_for` 清掉身上所有 buff，重生是新 entity，只還原 rank、不重新 learn，所以連用技能點學到的被動也會在第一次死亡後消失。auto_learn 的出生 rank 也一樣：只寫 log，不執行 hook。
  2. ChooseUpgrade 的「每個基礎技能先學一級」只看 level==0，但出生已經是 1 級，所以永遠走到第三個 loop，而那個 loop 依 Q、W、E 順序選第一個，結果只深升 Q。
  3. 俯視鏡頭在 1280×720 下可點擊的世界區域，大約只有上下 −270～+460、左右 ±800 單位。ChooseCast 選射程內最近的敵人，但 W（1100）、R（3000）的目標大多在畫面外。TryAbilities 投影失敗時直接 return false，不寫 trace、不退避，下一次決策又選同一招，於是這招一直卡住，也擋住後面的技能。
- **通用修正**：
  - server：`relearn_life_ranks` 在 AbilityUpgrades 階段（authority 與每個 replica 相同），對每一段存活的英雄生命，把持有的每個 rank 送一次 `SkillLearn`。以每個 world 各自的 `LifeRanksLearned` entity 集合記錄「這段生命已學過」；重生是新 entity，所以會自動重學。所有 on_learn 都以固定 buff id 冪等。第一版把標記做成隱藏的永久 buff，結果違反「單位身上沒有 buff」的測試不變式，已改掉。
  - server：HealthBonus／ExtraHealthBonus 之前只被 regen 用來放寬上限（`mhp + bonus`），沒有寫進 `CProperty.mhp`。所以 HP 會回到超過公開的最大 HP，team replica 因此拒收 baseline（`hp > max_hp` → MalformedBaseline），murakami E 出生就學會後，3 個 windup 測試與 kit soak 都失敗。改成 `settle_health_bonus`：比照 item 與升級成長，以 delta 寫進 `mhp`；bonus 增加時 HP 同步增加，減少時 HP 夾在上限內；regen 的上限改為 `mhp`。
  - UE：ChooseUpgrade 先學 D／R、再學未學的基礎技能，然後深升 rank 最低的基礎技能（同 rank 依 Q、W、E）。ChooseCast 加入 Clickable 判斷，只挑可點擊的目標，沒有就跳過這招、讓後面的技能用。沒有 area radius 的 point 技能（直線、錐形、位移）視為方向技能：目標在畫面外時，用 FarthestVisibleToward 朝目標瞄準。真的無法派送時，該 slot 退避 1 秒。OmAutoPilotTests 加了回歸測試。
- **避免**：
  - 新增的 stat key 要走完所有讀取端（disclosure、HUD、heal、regen）；不要只在單一系統放寬上限。
  - 凡是「學會時」的效果，都要在每一段生命開始時能重建，不能只靠升級事件。
  - 測試資料不要預先放入 rank 標記，否則會遮住 learn 漏掉的問題（murakami 的 soak 就是這樣）。
  - autopilot 規則要用「出生就有 rank」的英雄與「目標在畫面外」的情境做測試。

## E079 全量 `Om.` 改用 `-nullrhi` 執行後，9 個測試失敗、1 個測試讓 editor 當掉：測試寫死了會變動的常數，或沒有說明自己需要 RHI（2026-10-26，build-37）

- **現象**：`ue-tests-37.log` 有 86 個 Success、9 個 Fail，之後 editor 在 `AuthoredHeroAnimationLifecycle` 的 `GetCPUSkinnedVertices` assert（MeshObject）當掉。之前的實機驗收使用 `-d3d12 -NoSound`，只跑篩選過的測試。
- **根因**（分成三類）：
  - **寫死會變動的常數**：
    - CABI 38 的 ability slot 從 4 個變成 5 個，但 HUD 測試仍寫 5/3/10/14，tooltip 測試仍把 slot 4 當成無效。
    - Summon 測試寫死 `abi_version==37`。
    - Saika 測試寫死舊 ability id，內容改名為 `b01_*` 後就找不到。
  - **測試需要 RHI 卻沒有說明**：`-nullrhi` 下 Slate 會略過 widget 的繪製與 tick（`Slate.SkipWidgetDrawingInHeadlessMode`），所以 viewport geometry、像素與 CPU skin 都不存在。PreciseWorldCommand.Route／Rebind 的 RMB 判定、AimOverlayLayers 的像素檢查因此失敗；AuthoredHeroAnimation 則直接 assert。這和 Ctrl 滑鼠鎖定無關，因為測試 controller 是一般的 APlayerController。
  - **產品 bug**：`AllowAudioPlayback(false)` 的 world 透過 `GetAudioDevice()` 借用主音訊裝置，仍然會建立 voice。之前只在 `-NoSound` 下跑，所以沒被發現。
- **通用修正**：
  - **常數改用來源推導**：slot 數一律從 `OmAbilitySlots::Count` 推導；ABI 版本用 `OM_ABI_VERSION`；ability id 從 `GetGeneratedHeroMetadata().AbilityIds` 取得。
  - **headless 明確跳過**：新增 `OmEditor/Private/OmHeadlessAutomation.h` 的 `OmSkipWithoutRendering(*this, what)`，需要繪製或 geometry 的測試在 headless 下以 warning 跳過。main 已經在 CPU skin 測試加上 `FApp::CanEverRender()` 判斷。
  - **音訊**：`OmAudioPresentationComponent` 在 world 不允許音訊播放時不建立 voice。
- **避免**：
  - 測試裡的數量、版本、內容 id 都要引用產生它們的常數或 metadata，不能寫死字面值。
  - 新增的 UE 測試如果需要 RHI（geometry、像素、screenshot、skin），開頭就要呼叫 `OmSkipWithoutRendering`。
  - 宣稱「全量通過」時，要寫出執行模式（`-nullrhi` 或 `-d3d12`，以及有沒有 `-NoSound`）。兩種模式至少各跑過一次，才算涵蓋所有測試。
- **第二輪（`ue-tests-38.log`）**：109 個 Success、4 個 Fail，之後 editor 在 `CastWindupTooltip` 當掉，原因是讀取 `Metadata.CastTimes[0]` 時越界（陣列是空的）。
  - **根因**：authored 重製把 Saika 改成 `b01_*`、Date 改成 `b04_*`，同時移除了舊能力。`GetGeneratedAbilityMetadata("saika_reinforcements"/"fire_dash"/...)` 因此回傳空的 metadata。
  - **content gap**（要不要補回，由內容決策者決定）：
    - 目前沒有任何 authored 能力使用 facing formation，也沒有任何能力會被傷害中斷。
    - point-area 的數量從 12 降到 4，`bRequiresMovement` 從 2 降到 1。
  - **通用修正**：
    - 新增 `OmEditor/Private/OmAuthoredContentQuery.h`：`OmFindAuthoredHeroAbility` 會掃描整個 catalog，依「功能」尋找能力，而不是依 id。
    - 找不到時以 `OmRequireAuthoredHeroAbility` 發出 warning（content gap），widget／preview 改用固定的 published spec 驗證。
    - 只綁某個 authored id 的內容測試，改用 `OmRequireAuthoredHeroAbilityId` 檢查。
    - 需要 geometry 的 CameraProjectedDrag、CastInstructionBindings、CastTimeModifiers 後半段、ReadableHudToolTip 改呼叫 `OmSkipWithoutRendering`。
  - **避免**：內容測試不得寫死英雄的 ability id 或 authored 數值（半徑 100、寬 150、0.5 秒等），要從 catalog 依功能取得。內容重製時，同時檢查「哪些前端功能失去了 authored 來源」，並回報給內容負責人。
- **收斂結果（`ue-tests-38.log`，全量 `Om.`，`-nullrhi -unattended`，沒有 `-NoSound`）**：
  - 共跑完 286 個測試，`TEST COMPLETE`，沒有 crash。
  - 285 個 Success，1 個 Fail：`AutoPilotAbilityRule`，來自 04:18–04:20 另一個 session 正在修改的 OmAutoPilot，不屬於本項。
  - 43 個 render 段落以 headless warning 跳過；7 個測試因缺少 harness 以 warning 跳過（fixture／private workflow Job）；4 個功能是 content gap。
  - **headless 跳過的通用規則**：
    - `OmSkipWithoutRendering`：需要 geometry、像素、render target、texture platform data、PIE 視窗或 skinned bounds 的測試或段落使用。
    - `OmSkipWithoutHarness`：需要外部 fixture 或 Job 的測試使用。
    - 跳過一律以 warning 呈現，不能用 return false 讓整輪失敗，也不能用 Success 冒充已驗證。
- **RHI 輪（`ue-tests-39-rhi.log`，`-d3d12 -NoSound -unattended`）**：
  - 第一次跑：282 個 Success、4 個 Fail。
  - **ItemIconPresentation、OwnedItemCards**：冷卻文字已改成整數省略 `.0`，測試仍寫死 `"4.0"`／`%.1f秒`。改用 `FOmDurationText::Seconds` 產生預期字串。
  - **PublicHeroCombatRoster**：新 authored 的 6 位英雄（omatsu、oichi、inahime、asai_nagamasa、murakami_yoshikiyo、tamako）沒有 `NativeMeshAsset`，屬於 content gap。新增 `AOmAnimatedUnitActor::HasAuthoredNativeMesh()`，沒有 mesh 的英雄以 warning 跳過。
  - **WorldUnitVisualScale**：
    - 這不是 RHI 問題。我先前把它歸類為 headless 並加上跳過，是誤判，已撤回。
    - 根因是 generated 骨架 mesh 會先啟動 native presentation，讓實際呈現的 mesh 從 BP 的 `BodyMesh` 換成 `NativeHeroMesh`。測試在啟動前就取得 mesh 指標，而且寫死高度 70，但生成值是 65。
    - 新增 `AOmUnitActor::GetPresentationMeshForAutomation()`。測試改成先 `InitializeNativePresentation`，再量測實際呈現的 mesh，高度讀取 `PresentationHeightWorldUnits`。
  - **最終結果**：`-d3d12 -NoSound` 與 `-nullrhi` 兩種模式都是 286/286 Success，`TEST COMPLETE. EXIT CODE: 0`。7 個測試因缺少 harness 跳過，content gap 分別是 6 個（RHI）與 4 個（headless）。
  - **避免**：歸類成「需要 RHI」之前，先用 RHI 跑一次確認失敗確實消失。斷言訊息要帶出實際值，例如類別、量測值與期望值。

## E078 perf after 局漏了 --autopilot，玩家英雄閒置，數據不能和基準比（2026-10-26，split42）

- **現象**：split42-perf-after 的 tick 平均 6.09 ms，看起來比基準 split37 的 13.21 ms 好很多。但 p1/p6 資料夾沒有 autopilot trace，比賽在 391 秒結束後，run 也沒有提前退場，一直等到 600 秒時限。
- **根因**：重組指令時只照抄了 recipe、hero 和 max-seconds，漏掉 `--autopilot lane_v0`。少了這個參數，兩個玩家英雄全場站著，hero_command 負載偏低，而且 `all_finished` 偵測不到結束（它依賴 autopilot trace）。
- **通用修正**：`summarize_moba_perf.lua` 檢查每個玩家的 autopilot trace，缺少時在 verdict 後面標註 `no autopilot trace ... not comparable with autopilot runs`，閒置局不會再被誤當成對照組。正式 after 局用 split43 重跑。
- **避免**：before/after 對照局要從基準 run 的 `launch-plan.json` 抄完整參數（包含 `--autopilot`），不要憑記憶重組。

## E077 rejoin 後串流停住：舊 KCP socket 持續重送，netem proxy 的 endpoint 來回翻轉（2026-10-26，split41）

- **現象**：team 2 斷線 45 秒後，client 在 03:06:15 rejoin 成功（tick 10821），但 replica 只前進 1 tick 就停住，30 秒後又 link lost。下一次 rejoin 出現 `KCP read error: the offset to copy is not contained in the decompressed buffer`，之後 server 以 `missing or stale resume token` 拒絕。
- **根因**：client-runtime 判定 link lost 時沒有關閉舊的 KCP socket，舊 socket 持續重送。proxy 每個 team 只有一個 upstream socket，rebinding 依「最後發話者」決定 s2c 送往哪一端，結果在新舊 socket 之間每 100ms 左右翻轉（proxy.stdout.log）。新連線收不到大部分 s2c，兩條 KCP 串流的資料混在一起。
- **通用修正**：交給 omfx-decouple 處理。(1) proxy 改成依 client endpoint 各配一個 upstream socket（NAT 語意）。(2) client 在 rejoin 前先丟棄舊的 KCP socket 與 session。(3) resume token 輪替要和最後一次被接受的 rejoin 一致。
- **已修正（omfx-decouple，2026-10-26 03:42 著陸）**：
  1. **proxy NAT 語意**：`omoba-netem-proxy/src/route.rs` 依 client endpoint 各配一個 upstream socket。第一個 endpoint 用設定的 `upstream_bind`，之後的 endpoint 用同介面的臨時 port。s2c 只送回收到它的那條 mapping 的 endpoint，不再依「最後發話者」翻轉。閒置超過 `NAT_IDLE_TIMEOUT`（30 秒）的 mapping 會被回收，並關閉它的 upstream socket；mapping 被回收後，延遲中的封包和遲到的回覆都會丟棄。outage 仍以 team 為單位，涵蓋該 team 的所有 endpoint。evidence 新增 `nat_flows`（endpoint、upstream、opened_ms、reclaimed_ms、雙向封包數），`endpoint_changes` 也多了 `upstream` 欄位。測試 `each_client_endpoint_gets_its_own_upstream_mapping_and_idle_mappings_are_reclaimed` 取代舊的 rebinding 測試。
  2. **舊 socket 釋放**：`KcpClient` 持有 reader 和 ping task 的 `AbortHandle`，Drop 時會 abort，讓 KCP stream 的兩個 split half 都被釋放（tokio_kcp 的 Drop 會 close session），舊 socket 就不會再重送。`CheckpointWriter` 的 Drop 會 abort 寫入 task，避免卡在死連線上的回報繼續持有 writer。`run_session` 以值接收 session，結束時整個 session 一起被丟棄；rejoin 嘗試逾時時，`timeout` 會丟棄還沒建完的連線。
  3. **resume token 兩段式輪替**：`LockstepState` 每個玩家保存 `confirmed` 和 `pending` 兩個 token。新發的 token 先放在 pending，client 在下一次 join 出示它時才升為 confirmed（較舊的 token 同時失效）。在這之前，confirmed 仍然有效，所以「server 已接受 join、但 TeamGameStart 因逾時沒送到 client」時，client 可以用手上的舊 token 重試。新增測試 `rejoin_whose_new_token_was_lost_can_retry_with_the_previous_token`。
  - 驗證：proxy 18+3、omb lockstep 57、omb kcp 54、core kcp 14、client-runtime 167 全數通過。
  - **實機驗證：split44 PASS**（split44-rejoin，build-38，有 autopilot）。`check_netem_outage` 8/8 PASS；link lost 1 次、rejoin 1 次後串流持續，mismatch 0、terminated 0。endpoint change 只有 2 次（split41 有上百次）。`nat_flows` 中兩個舊 endpoint 分別在 194103 和 204108 ms 被回收（outage 在 179161 ms 結束）。
- **避免**：「某條連線失效」時，一定要先釋放它的 socket 和狀態，再建立替代連線。測試用 proxy 要模擬真實的位址語意（一個 endpoint 對應一個 upstream），不能讓多個來源共用一條路由。

## E076 E072 的 stuck recover 著陸時沒有編譯 UE：const Decision 被改寫，巢狀 FProgressWatch 未匯出（2026-10-26，I1）

- **現象**：murakami 的 hero_author build 先後出現兩個錯誤。第一個是 `OmAutoPilot.cpp(1626)` 的 C2678，對 `const FVector2D` 賦值。第二個是 OmEditor 連結時的 LNK2019，找不到 `FOmAutoPilot::FProgressWatch::NoteMove` 和 `IsStuck`。
- **根因**：stuck recover 會改寫 `Decision.RouteIndex` 和 `Decision.Point`，但 `Decision` 宣告成 const。另外，MSVC 的 `class OMRUNTIME_API` 不會匯出巢狀 struct 的非 inline 成員，跨模組的測試因此連結失敗。這兩個問題都是著陸前沒有跑 UE 編譯造成的。
- **通用修正**：`Decision` 改成可變。`FProgressWatch` 加上 `OMRUNTIME_API`。
- **避免**：OmRuntime 的 public 巢狀型別只要被 OmEditor 測試呼叫，就要自己加上 `OMRUNTIME_API`。UE 原始碼的修改，著陸前至少要跑過一次 `build_ue_moba.lua --build-only` 或 consistent build，不能只跑 Rust 或 Lua 測試。

## E075 失敗的 hero_author build 留下孤兒佔位頭像，之後每次 UE build 都卡在 HUD 同步（2026-10-26，I1/I2）

- **現象**：murakami_yoshikiyo 的 `hero_author build --execute` 在 build_ue 失敗並回滾；接著 tamako 的 build 和單獨的 `build_ue_moba.lua --build-only` 都在 `sync_ue_hud_manifest_labels.lua:19` 失敗，訊息是 `HUD asset refers to unknown content: portrait:murakami_yoshikiyo`。
- **根因**：build_ue 的 HUD 同步階段會替新英雄把佔位頭像寫進 `hud-assets-v1.json`，但 hero_author 的回滾清單只涵蓋 templates 和 DLL。回滾後英雄從目錄消失，佔位條目卻留著，同步工具又把未知條目一律視為錯誤。
- **通用修正**：`ue_hud_portrait_coverage.prune_placeholders` 把 `placeholder=true`、但英雄已不在目錄裡的頭像視為衍生資料移除；正式美術條目指向未知英雄時仍然失敗。`hero_author/build.lua` 的 protected 清單也加入 `hud-assets-v1.json`。`ue_hud_portrait_coverage_test.lua` 新增兩項檢查。
- **避免**：只要建置鏈會寫入「從目錄衍生、而且會累積」的檔案，就要列入交易的回滾清單，同步工具本身也要能從來源重新推導。不能假設失敗的建置不會留下東西。

## E074 移動目標在地形裡時，整個命令被靜默丟棄，英雄停在原地（2026-10-26，split38）

- **現象**：split38 中 oichi（p1）最後約 22 秒卡在 (760.6, 2559.9) 不動。autopilot 每 2 秒送一次 move_route，ack 都是 accepted，但 server 記錄 `hero_command: rejecting unreachable MoveTo ... target=(845.7,3351.1)`；其他英雄對 (629.8, 3351.1) 和 (2958.7, 4401.7) 也被拒。
- **根因**：
  1. 這些目標都在 jungle wall 裡面：(845.7, 3351.1)、(629.8, 3351.1) 在 `jungle_wall_3_0`（400–1000 × 3300–3420）內，(2958.7, 4401.7) 在 `jungle_wall_1_3` 內，任何英雄半徑都被擋。reachability 判定本身沒錯，是點擊或路線點落在地形裡。
  2. 不可到達的 MoveTo／AttackMove 在 `hero_command` 被整個丟棄，沒有任何通知；ack 早在 admission 時就已經回 accepted，所以 client 以為英雄正在移動。
- **通用修正**：
  - 新增 `static_nearest_reachable`，採用 MOBA 的慣例：目標不可到達時，走向最接近目標的可達格。
    - 目標太遠時，先沿直線拉回可搜尋的範圍。
    - 用同一套 FIFO 與鄰格順序把有界搜尋跑完，距離相同時保留搜尋順序中最早的格，因此權威端與 replica 結果一致。
  - `drain_pending_moves` 在 admission 時解析 MoveTo／AttackMove 的目標：可到達就照舊；不可到達就換成最近可達點，並記錄 log；如果英雄已經無法再接近，就以新的拒絕碼 `UNREACHABLE` 寫入 owner command outcome（ack-result 的 reason）。
  - `PendingHeroCommand` 新增 `admitted_correlation`，讓這個拒絕對應到原本的輸入。
- **驗證**：
  - 單元測試：牆內目標會走到牆外最近點，結果是確定的；可到達的目標維持原樣；太遠的目標會被拉回；被完全圍住時回傳 None，由呼叫端以 UNREACHABLE 拒絕。
  - parity soak `move_orders_into_terrain_walk_to_nearest_reachable_point_on_every_replica`：英雄真的朝牆內目標移動（不再丟棄命令），兩隊 replica 逐 tick 一致。
- **避免**：權威端拒絕已 ack 的命令時，一律寫入 command outcome 並附原因碼；對「點到不可到達的地方」採用最近可達點的通用規則，不要靜默丟棄。autopilot 的路線點也應該避開地形（quick map 的 lane 路線點與 jungle wall 重疊，可另外檢查地圖資料）。

## E073 伺服器 tick 熱點：每個英雄每 tick 對整個 BFS 區域重算地形幾何，另外每 tick 深拷貝 512 tick 的可見度歷史（2026-10-26，J5 perf）

- **現象**：split37（乾淨時段的基準局）伺服器 tick 平均 13.21 ms、p99 視窗 35.37 ms、55.1 Hz。分段計時顯示 dispatcher 佔 64%，其中 `hero_command` system 每 tick 16.8 ms，其他 system 都在 0.05 ms 以下。
- **根因**：
  1. `static_next_waypoint` 每 tick 為每個移動中的英雄跑一次有界 BFS（最多 113×113 格）。每展開一格，都對所有地形多邊形做約 16 次 `path_hits_regions`，每次都重新量化浮點座標並配置記憶體；parent 用 `BTreeMap`。這些「格子是否被擋、格心到格心的邊是否被擋」只取決於靜態地形，卻每 tick 重算。
  2. `build_input_validation_snapshot` 每 tick 為兩隊各複製一次完整的 512 tick 可見度歷史（每 tick 一個 BTreeSet）；`run_team_projection_after_wave_b` 開頭也 clone 整個 `TeamVisibilityRuntime`，連同這些歷史一起複製。
- **通用修正**（結果完全相同）：
  1. `TerrainIndex`：每次 system run 量化一次地形，`hits` 與 `path_hits_regions` 回傳完全相同。
  2. `NavMemo`（`hero_command` 的 resource，跨 tick 保留）：快取「格子是否被擋」與「格心到格心的邊是否被擋」（以半徑為 key）。地形指紋改變時清空，並設有上限。碰到精確起點或終點的邊仍然即時計算。
  3. BFS 的 parent 改用密集陣列；FIFO 與鄰格順序完全不變。
  4. 可見度歷史改為 `Arc<BTreeSet>`（`SharedVisibleSet`）：push 後內容不再改變，snapshot 與 clone 只複製指標。
- **驗證**：
  - `memoized_planner_matches_reference_on_every_compiled_map`：所有已編譯地圖、每張 600 個查詢 × 3 種半徑，新舊 planner（舊版逐字保留作為對照）結果完全一致，包含共用 memo 與新 memo 兩種情況。
  - `terrain_index_matches_path_hits_regions_and_memo_follows_terrain_changes`：地形改變時 memo 不會沿用舊答案。
  - release 微基準：每個查詢 1.236 ms → 0.080 ms（15.5 倍）。
  - parity soak（scripted_motion、gunner、移動進牆）與完整套件通過。前後的實機數據記在 perf-report.md §6。
- **避免**：每 tick 對每個單位都要跑的搜尋，只取決於靜態資料的部分要快取，並以資料指紋失效；不可變的歷史資料用共享指標，不要整份複製。新增 per-entity 的 system 前，先用 `summarize_moba_perf.lua` 的 system_time 確認成本。

## E072 autopilot 的「畫面可見截點」落在牆內，被誤認成 lane 路線點不可到達（2026-10-26，E070 追查）

- **現象**：E070 被拒的目標 (845.7,3351.1) 位於 quick map 的 `jungle_wall_3_0` 內（縮放後是 (400,3300)-(1000,3420)），而不是 lane 路線點。公開路線就等於 lane waypoints，三張 MOBA 地圖的所有路線點與相鄰路段，用英雄半徑檢查都可以通過。
- **根因**：`FarthestVisibleToward` 沿著英雄到 waypoint 的直線，取畫面上看得到的最遠點當點擊點，這條直線會穿過叢林牆。之後連續被拒也沒有改走其他路徑，因為 autopilot 只看 ack。
- **通用修正**：
  - 地圖驗證：
    - `lua_content` 的錯誤訊息會標出 map、lane、segment 與座標。
    - 新增 `moba_map_layout::every_compiled_lane_route_is_reachable_by_a_hero`，用 runtime 的 `path_hits_regions`／`static_next_waypoint` 檢查每個點與每個路段，並包含刻意放牆的負向測試。
  - autopilot：
    - 點擊點用 `ClearOfWalls` 從 frame 的 `terrain_rects` 往英雄方向退回牆外。
    - `FProgressWatch` 判定卡住：收到拒絕，或 3 秒內送出 2 次以上 move 但位移 <40，就算卡住。
    - 卡住時跳到下一個路線點，並用 `DetourPoint` 取 ±45/90/135° 的繞行點（左右交替），同時寫入 trace `{"event":"stuck_recover"...}`。
    - 測試是 `Om.Runtime.AutoPilotStuckRecover`。
- **避免**：
  - 追「哪個點不可到達」之前，先確認這個點的來源：路線資料，還是 client 推導出來的點。
  - 任何由 client 推導的移動目標（clamp、偏移、取樣）都要經過地形檢查。
  - 「沒有進展就改道」是 autopilot 的通用規則，不能只在特定 map 或英雄上處理。

## E071 召喚物在 script dispatch 中轉向敵人，己方 replica 用的是預測位置，造成 FACING 不一致（2026-10-26，split39）

- **現象**：split39 tick 6120，team 2 observer mismatch：entity 368 `DISCLOSED_FACING 0x464f4707`，canonical 0x600000020，kind=summon。這是 p6 saika 的定點 gunner，權威端面向 2989，replica 是 2895。發生在斷線前約 20 秒，和 rejoin 無關。
- **重現**：`summon_facing_parity::a_turret_tracking_a_zigzagging_enemy_faces_the_same_in_every_replica`。`lumen_sentry` 定點砲塔追蹤每 5 tick 換一次方向的敵方英雄，修正前 240 tick 中有 239 tick 不一致。
- **根因**：
  - 定點召喚物（generic `Emplacement`、`saika_gunner`）在 `on_tick`（script dispatch，位於本 tick 移動之後）呼叫 `set_facing(atan2(target - pos))`。
  - 己方 replica 也會跑同一段 script。但敵方玩家的輸入不會送給這一隊，敵人在本 tick 的實際位置要等 step 結束後套用 Movement fact 才知道，所以 replica 是對著預測（舊方向）的位置轉向。
  - 英雄的面向一直由 PostStep 已提交姿勢的 fact 校正，召喚物沒有這個 fact。
- **通用修正**：`finish_moba_match_tick` 對每個召喚物送出 PostStep `Movement` fact（位置＋facing_ticks，MOVEMENT 可見性政策），與英雄的已提交姿勢相同。看得到該召喚物的 replica 在 step 後採用權威值，自己的預測不再影響揭露結果。
- **避免**：replica 會執行的 script 或系統，只要讀取**敵方**單位在本 tick 的位置（轉向、選目標、距離判定）並寫入會被揭露的元件，就必須讓權威值透過 fact 回到 replica（或證明只讀到開始時的位置）。新增定點或自動瞄準類的召喚物、塔或 ward 時，要用「敵人每幾個 tick 換方向」的 parity 測試驗證。

## E070 不可到達的 MoveTo 被靜默丟棄，英雄卡在原地（2026-10-26，split38）

- **現象**：oichi 最後約 22 秒停在 (760.6,2559.9)。autopilot 每 2 秒送一次 move_route，ack 都是 accepted，但 server 出現 `hero_command: rejecting unreachable MoveTo ... target=(845.7,3351.1)`。同一個公開 lane 路線點，其他英雄也被拒。
- **根因（待 omfx-decouple 確認）**：一是 lane 路線點在 quick map 上被判為不可到達；二是 hero_command 對不可到達的目標整個拒絕，而且沒有回報。輸入層的 ACK 已經是 accepted，client 不知道指令失效，於是重複送同一個指令。
- **通用修正方向**：移動到朝目標方向最近的可達點（權威端與 replica 用同樣的規則），完全不能移動時要回可見的拒絕碼。地圖的公開路線點必須保證可達，這一點應加入地圖驗證。
- **避免**：「ACK accepted」不等於指令已生效（E023 的延伸）。實機判定除了 ack，也要看英雄位置是否真的有進展；之後可以讓 autopilot 在沒有進展時改走其他路徑，並把這件事寫進 trace。

## E069 共用樹直接改了 handler，package 的 rust 檔沒同步；從 package 重建就會編譯失敗（2026-10-26，session-h02）

- **現象**：live 的 `ability_projectile::Spec` 加了 `forced` 欄位，`scripts/base_content` 裡 4 個 handler 也補了 `forced:None`。但 `packages/{asai_nagamasa,date_masamune,omatsu}/rust/` 的同名檔沒改，tamako 的新 handler 也沒有這個欄位。只要用 `build --execute` 從 package 重建這些英雄，就會缺欄位編譯失敗，或把 live 的修正洗掉。shadow6 是在加 `forced` 之前取的，所以 shadow 測試看不出來。
- **根因**：package 才是英雄 handler 的來源，但共用 struct 加欄位時，只修了已安裝的副本。
- **處理**：4 個 package 檔從 live 同步（diff 確認只差 `forced:None`），tamako No1 補上這個欄位。著陸改成「把 anchored spec 重播到從 live 新取的 shadow7」，再和舊 shadow 比對：凡是不同之處，都必須是 live 自己的新修改。
- **避免**：替 host struct 加欄位時，同一次修改要 grep `docs/character_pipeline/packages/*/rust`，並更新所有 struct literal。著陸前一律從 live 取新 shadow 重播並測試，不拿舊 shadow 的測試結果當作 live 的結果。
- **差點犯的錯**：重播腳本把 spec 的路徑換算成相對路徑時，先比對 live 根目錄 `D:/omoba/`。但 shadow 本身也在 `D:/omoba/target/...` 底下，所以 shadow6 的檔案被算成 `target/session-h02/shadow6/...`；如果直接 `--apply`，就會寫進舊 shadow，而不是 live。dry run 的輸出列出這些路徑，才發現。修正：一律先比對最長、最具體的前綴；著陸前一定先 dry run，確認每個輸出路徑都是 repo 內的相對路徑。
- **02:43 的著陸**：dry run 與 `--apply` 寫在同一個指令裡（`&&`），dry run 列出 3 個新的 DIFFERS 檔案，指令仍然直接著陸。事後確認這 3 個檔案都只是 live 自己的新修改（E070 不可達點、lane 邊診斷、召喚物姿勢 fact），與這批修改不重疊。以後 dry run 與 apply 分成兩個指令，先看完 DIFFERS 清單再 apply。

## E068 伺服器 tick 超出 60 Hz 預算，但正式路徑沒有分段計時；效能量測與建置同時進行而被污染（2026-10-26，J5）

- **現象**：`scripts/summarize_moba_perf.lua` 彙總 split19、27～32、35、36。未污染的局伺服器 tick compute 平均 9.3～12.5 ms（門檻 8 ms），p99 視窗平均 22～27 ms（60 Hz 預算 16.7 ms）。split36 平均 17.75 ms、47.7 Hz，看起來像新英雄造成的回歸。
- **根因**：
  1. split36 從頭到尾都與 consistent-build-35 的 UE 編譯同時進行（由 00:47 的 omfue 著陸觸發，持續到 01:13）；split35 開頭與 package-shipping-3 重疊。這兩局量到的是機器的競爭，不是遊戲本身。
  2. 正式 MOBA tick 只輸出 `tick_compute` 總數。`TickProfile` 的 phase／system 分段（run、dispatch、outcomes、前 13 個 system）只在舊路徑記錄，五局 log 中出現 0 次，因此無法指出熱點。
- **處理**：
  - 新增可重複執行的彙總工具：以 sum/count 計算真實平均，對視窗平均取 nearest-rank p99，算出實際 tick rate；UE 卡頓扣除 60 秒暖機；判定 PASS／FAIL。附 fixture 測試。
  - 結果寫進 perf-report.md，污染的局標為不採計。
  - **待做的通用修正（omb）**：正式 tick 依 phase 輸出 OM_PERF（dispatcher、script dispatch、process_outcomes、wave B、每隊 projection、observer validation），由同一個工具彙總。
- **避免**：效能驗收局要排在沒有任何建置（consistent_build、package、UE 測試）的時段，並且在 perf-report 記錄同時進行的建置；看到單局回歸時，先對照建置 log 的時間，再判斷是不是程式的問題。

## E067 UE 區域變數遮蔽 APlayerController／AActor 成員，C4458 被當成錯誤（2026-10-26，滑鼠鎖定）

- **現象**：OmPlayerController.cpp 新增的游標鎖定程式在 build-35 前編譯失敗。原因是區域變數 `Player`（ULocalPlayer*）與 `Owner`，遮蔽了 `APlayerController::Player` 與 `AActor::Owner`，觸發 C4458；UE 會把這個 warning 當成 error。
- **根因**：這台機器不能編譯 UE，寫程式時只參考同檔案的其他函式；沒有注意到在 actor 子類別的成員函式中，常見名詞剛好就是繼承來的成員。
- **通用修正**：主 session 把它們改名為 `LocalViewer`／`ActionOwner`。
- **以後如何避免**：在 `AActor`／`APawn`／`APlayerController`／`UActorComponent` 的成員函式中，不要用 `Player`、`Owner`、`Controller`、`World`、`Pawn`、`Instigator`、`InputComponent`、`RootComponent` 這類名稱當區域變數、參數或 lambda capture，改用具體名稱（`LocalViewer`、`ActionOwner`、`Client`）。著陸 UE 程式前要逐一檢查新加的區域變數名稱。

## E066 盲測 I2 發現：guide 宣稱支援的能力，工具其實沒有接上（2026-10-26，B29 村上義清）

- **現象**：盲測者照 hero-authoring-guide 做 B29，遇到以下問題：
  - 引導技：guide 說支援，但生成器從不輸出 content `channel`，只能改用沒寫在文件裡的 `build_ability_ffi_with_execution`。
  - pulse channel 規定 cast_time 0，guide 沒寫，B29T 原本 0.3 秒的前搖因此被拿掉。
  - `needs_framework` 技能的 handler 會被 rust_install 一併安裝而編譯失敗。
  - soak 只能貼進共用測試檔。
  - soak 範本讓兩位英雄從不碰面，什麼都沒發生也會通過。
  - stats 沒有最大生命，guide 裡的捨入規則也和範例不一致。
- **根因**：guide 依「引擎有這個能力」撰寫，沒有逐項用「只看 guide 做一次」驗證工具鏈。每個能力只在共用引擎或手寫英雄驗過，作者流程（package → 生成 → 安裝 → 測試）沒有端到端走過。
- **通用修正**：
  - **引導**：package `timing.channel` 生成 content `channel`，template-ids 匯出 `PulseChannelConst`，`build_ability_ffi` 自動帶成 host execution。引導可以有前搖（content 與 host 都放寬到 [0,60] 秒），沒有宣告式 effect 時由 handler 的 `on_channel_pulse` 出波。
  - **工具**：rust_install 依 slice 過濾；package 可攜帶 `rust/parity_soak.rs`（test-only），並開放 soak helper 與 `soak_report` 計數。shadow 每次都會更新驗證工具。
  - **能力**：stats 新增 `max_hp_bonus`；新增通用 `ally_area_buff` 封包（友方光環、範圍增益）。
  - **文件**：guide 補上以上各項、`query_enemies_in_range` 含建築、`scripts/text_patch.lua`，以及 soak 有效性檢查。
- **避免**：guide 每宣稱一項能力，都要附一個「只用 guide 寫法」做出來的 package 範例或工具測試。盲測回饋要逐條轉成工具測試，不能只改文件。

## E065 Shipping 仍會執行測試自動化開關，且發佈包內含 1.8 GB 除錯符號（2026-10-26，Shipping 驗證）

- **現象**（package-shipping-3，Shipping 打包成功，32 個檔案）：
  - autopilot 確實沒有編入：exe 中找不到 `om-autopilot=` 與 `OmAutoPilot`。
  - 但 exe 仍含有 `om-shop-smoke`、`om-recall-smoke`、`om-presentation-smoke` 等開關，其中 `om-presentation-smoke` 區塊會自動送出移動輸入。也就是說，可以從命令列讓發佈版自動操作、寫出截圖檔。
  - archive 內有 1.85 GB 的 `OmGame-Win64-Shipping.pdb`。
- **根因**：
  - 測試開關散落在 4 個檔案中，共 34 處，各自直接呼叫 `FParse`，只有一部分包在 `#if !UE_BUILD_SHIPPING` 裡，沒有統一的規則。
  - 打包時沒有排除除錯資訊。
- **通用修正**：
  - 新增 `OmRuntime/Public/OmTestSwitches.h`，提供 `OmTestSwitch`、`OmTestSwitchValue`，在 Shipping 一律回傳 false。所有 `-om-*-smoke` 與 `-om-*-screenshot=` 都改走這兩個函式；連線與選角等產品開關照舊直接用 `FParse`。
  - 新增 `scripts/tests/ue_test_switch_guard_test.lua`：掃描所有 runtime 原始碼（不含 OmEditor），有任何 smoke／screenshot 開關繞過 helper 就失敗。修正前它會在共用樹上抓到上述位置。
  - `package_ue_moba.lua` 在 Shipping 加上 `-nodebuginfo`，Development 保留符號以便診斷。
- **避免**：新增測試用的命令列開關時，一律使用 `OmTestSwitch`／`OmTestSwitchValue`，不要靠零散的 guard；Shipping 驗收時檢查 exe 內是否還有測試開關字串（UTF-16）。

## E064 Shipping 打包失敗是假警報：建置期間別的 session 改了 UHT 標頭，GENERATED_BODY 行號對不上（2026-10-26，Shipping 驗證）

- **現象**：`package_ue_moba.lua --config Shipping`（23:54 開始，37 分鐘）以 UAT exit 6 失敗。`OmUiEventTypes.h` 從第 596 行起，每一個 `GENERATED_BODY()` 都報 C4430（遺漏類型規範）。
- **根因**：
  - 不是 Shipping 的 guard 問題。這個標頭在 00:13:45 被別的 session 插入了 3 行（未 commit 的工作），時間點落在 UHT 產生程式碼之後、模組編譯之前。
  - `GENERATED_BODY` 依賴行號，所以插入點之後的每個 USTRUCT 都對不上。
  - `consistent_build.lua` 本來就有「等安靜、比對指紋、變動就重建」的機制，但打包腳本沒有。
- **通用修正**：`package_ue_moba.lua` 改用 consistent_build 的指紋與安靜視窗：
  - 每一輪開始前先等輸入靜止 `START_QUIET_SECONDS`。
  - 建置前後比對指紋。若期間輸入有變動，這一輪的結果（失敗或成功）都不算數：失敗視為別人正在改，成功則代表產物已過期。改用新的 `archive-N` 目錄重跑，最多 4 輪，仍在變動就明確報錯。
  - package-report 升到 schema 2，記錄每一輪的 archive、log、exit code 與 `sources_changed`。
  - 測試涵蓋「改動中失敗後重試成功」與「持續變動會放棄、不回報過期產物」。
- **第二次（package-shipping-2）**：4 秒就以 UAT exit 10（`Failed (ConflictingInstance)`）失敗，原因是別的 session 的 UnrealBuildTool 正持有全機唯一的 UBT mutex。新增 `consistent_build.ubt_busy(result)`，打包與 consistent build 共用：碰到 mutex 被占用時等待 `BUSY_WAIT_MS` 後重試同一輪，不消耗輪數，最多 `MAX_BUSY_RETRIES` 次，並補上對應測試。
- **避免**：任何讀共用原始碼的長時間建置（打包、consistent build、UE 測試）都要走同一套指紋與安靜視窗；看到 GENERATED_BODY 或 UHT 相關錯誤時，先查標頭的 mtime 是否落在建置期間，再判斷是不是真的編譯錯誤。

## E063 用程序名稱強制結束測試，可能誤殺其他 session 的 base_content 測試（2026-10-25 23:3x，session-h02）

- **現象**：停止自己的背景 cargo test 後，又用 `tasklist | grep base_content- | taskkill /F` 清除殘留測試程序。比對條件只有名稱，所有 `base_content-<hash>.exe` 都會被結束，包括其他 session 用自己 CARGO_TARGET_DIR 跑的測試。
- **根因**：測試程序名稱只由 crate 名稱和 hash 組成，看不出屬於哪個 session 或 target 目錄。用名稱篩選，等於對共用機器上的所有同名程序下手。
- **通用修正**：只用 TaskStop 停止自己啟動的背景工作；若真的有殘留，只結束由自己的 target 目錄啟動的程序。例如 `wmic process where "ExecutablePath like '%session-h02%'"`，或比對 `tasklist /v` 的映像路徑，絕不只比對名稱。
- **避免**：不得用程序名稱結束 cargo、rustc 或測試程序（另見 ward 結束 cargo 造成 consistent-build 重試）。發生誤殺時要通知受影響的 session 重跑。

## E062 著陸時把 shadow 副本整檔覆蓋到共用樹，洗掉別人在取 shadow 之後的修改（2026-10-25，extra_action_slot 23:07 著陸）

- **現象**：主 session 在 22:55 對 OmRuntimeBridgeSubsystem.cpp（`OutStatus.Error = SessionTermination.InputRefusal()`）與 OmPlayerHudOwnerComponent.cpp（`GetStatusNotice()`）加的 link-stall 修改，在我 23:07 著陸後消失，build-32 的 `Om.Runtime.MatchTerminationRefusesInput` 失敗（Expected "Connection stalled"）。
- **根因**：shadow 是約 1 小時前從共用樹複製的。著陸時我直接 `cp shadow live`，雖然先看過「共用樹與 shadow 的差異行數」，但沒有區分「我改的行」和「別人在我複製之後改的行」：「-3 +6」被我當成自己的修改。整檔覆蓋就把別人的新修改洗掉了。
- **通用修正**：`target/session-ward/land.sh`：
  - `snap`：開工時把共用樹檔案同時複製到 shadow 和 base（我修改的基準）。
  - `land`：用 `git merge-file -p live base shadow` 做 3-way merge。沒有衝突才寫回共用樹，有衝突就停下、不寫入。著陸後把 base 與 shadow 更新為著陸後的版本，作為下一次的基準。
  - 新檔只有在共用樹沒有同名檔案時才複製。
  - 已在 CRLF 檔驗證可以保留行尾。
- **避免**：任何著陸都只能透過 3-way merge，禁止 `cp shadow → live`。每個要修改的檔案都要先 `snap`，不能直接拿舊 shadow 來改。

## E061 用 taskkill /IM cargo.exe 停自己的測試，連帶砍掉其他 session 的 cargo；Git Bash 的 sed -i 把 CRLF 檔改成 LF（2026-10-25，extra_action_slot）

- **現象**：（1）為了停掉自己的背景回歸測試，執行了 `taskkill /F /IM cargo.exe`，結果機器上所有 cargo 程序都被結束，包括其他 session 正在跑的 build／test。（2）用 `sed -i` 修改 CRLF 的 UE header（OmPlayerActionOwnerComponent.h），整個檔案被改成 LF；同一批 perl 替換因為比對 \r\n 而沒有套上，陣列少了一個初始值。另外，shadow 中幾個沒有修改的檔案比實際樹舊，導致 template-ids 測試誤報失敗。
- **根因**：（1）taskkill 依映像名稱比對，不是依父程序，共用機器上沒有「只砍自己的」效果。（2）Git Bash 的 GNU sed 在這個環境會吃掉 \r。（3）shadow 只在開工時同步一次，其他 session 之後的修改沒有進來。
- **通用修正**：背景工作一律用 TaskStop（以 task id 停止），不再用 taskkill／pkill 依名稱結束。CRLF 檔案只用 perl -pi（逐行，不在 pattern 中寫死 \n）或 Edit 工具修改，改完用 `file` 比對實際樹與 shadow 的行尾；不要用 grep $'\r'，它在這個環境看不到 \r。著陸前列出 shadow 與實際樹所有不同的檔案，凡是不在「我修改的檔案」清單中的，一律先從實際樹同步，再跑測試。
- **避免**：見上。測試失敗時，先確認失敗的檔案是不是自己改過的；不是的話，先懷疑 shadow 過期。

## E060 斷線沒有任何存活判定：client 不發 STALLED／READY、超時也不下判定，UE 在世界凍結時仍顯示已連線；proxy 被硬殺時不寫證據（2026-10-25，J1 斷線）

- **現象**（J1 斷線前的程式碼盤點）：
  - `RuntimeSessionState` 定義了 STALLED、SERVER_DISCONNECTED，但 client runtime 只會發 TERMINATED。單隊 UDP 全斷時，KCP 保持串流、恢復後依序補齊，但斷線期間畫面凍結，UE 仍顯示已連線，沒有任何提示。
  - 沒有任何容忍上限：伺服器端寫入被 KCP send window 卡住時會無限期等待（`write_all` 擋住整個 select，連讀取都停），也沒有 client 端的超時判定。
  - 另外，split runner 用 `stop_owned` 硬殺 netem proxy，所以 proxy 沒有寫出路由統計與 `proxy-evidence.json`（split31 的 proxy.stdout 只有 ready 一行）。
- **根因**：
  - 存活判定沒有實作：lockstep 依賴 KCP 的可靠傳輸，斷線期間「沒有新 frame」與「遊戲暫停」在 client 端無從分辨。
  - proxy 只有在收到 control `shutdown` 時才會寫出證據，runner 卻沒有走這條路。
- **通用修正**：
  1. `omoba_core::lockstep_timing::{LINK_STALL_AFTER=2 s, LINK_LOST_AFTER=30 s}` 是兩端共用的容忍值。
  2. client runtime 新增 `link_liveness::LinkLiveness`，以 replica 進度判定存活：
     - 每 250 ms 檢查一次。2 s 沒有進度就發 `STALLED`（`link_stalled`）；恢復後第一個進度發 `READY`（`link_recovered`）。
     - 30 s 仍無進度，發 `SERVER_DISCONNECTED`（`link_lost`），並以 `ShutdownReason::ServerDisconnected` 結束。
     - 都會寫入 evidence network event（`link-stalled`／`link-recovered`／`link-lost`）。
     - 為了承載這些 reason，`session_state_envelope` 改為直接帶完整的 safe_reason；TERMINATED 的 reason 仍是 `match_terminated:<class>`。
  3. 伺服器 `handle_client`：
     - 讀到任何 client 流量就更新 `last_client_activity`。
     - 送 frame 前，若 client 已沉默超過容忍值，就結束 session 並記錄 `KCP link lost ... no client traffic`。
     - 寫入以相同容忍值設 timeout，被卡住時同樣結束 session 並記錄 `outbound blocked`，不再無限期等待。
  4. netem proxy 新增排程的單隊斷線：
     - `--outage team=N,at_ms=…,duration_ms=…` 可重複指定，同隊的時間窗不可重疊，最多 64 個。
     - 斷線期間雙向全部丟棄，包含已經排進延遲佇列的 datagram。
     - 預設錨點是 proxy ready；收到 control `anchor` 後改以當下為錨點，但在任何窗開始後就不能再移動。
     - route 在時間窗邊界醒來，記錄實際的開始與結束時間，並印出 `outage start/end/summary`。
     - evidence 升到 schema 2：新增 `outages`（排程、錨點、實際起訖、雙向丟包數），各方向新增 `lost_datagrams`（loss_ppm 造成的丟包）。停止時印出 `netem-proxy stopped status=…`。
  5. `run_moba_split_local.lua`：
     - 新增 `--netem-outage TEAM:AT_S:DUR_S`（可重複）。
     - 所有人類 client 都進場後才送 `anchor`。
     - 結束時先用 control `shutdown` 優雅停止 proxy（最多等 7 s），再硬停。
     - `send_netem_control.lua` 支援 `anchor`。
  6. 斷線期間送出、被伺服器以 late 拒絕的輸入，原本 client 永遠收不到結果（`InputSubmit` 的拒絕只寫 log，pending request 不會清除）。新增 `link_liveness::InputSettleDeadlines`：一般輸入（不含 shop／undo，它們走 receipt 查詢）的目標 tick 加上 `INPUT_SETTLE_DEADLINE_TICKS`（2×LOCKSTEP_TPS，大於伺服器 1 秒的 late grace）後仍未結算，就對 renderer 發 `INPUT_EXPIRED`。
- **測試**：
  - proxy 共 21 項。新增 `outage::tests` ×3、config 的 outage 解析與重疊拒絕、整合測試 `team_outage_drops_both_directions_only_inside_its_window`（真 UDP route，窗內不轉送、窗外恢復，實際起訖時間誤差在 100 ms 內）。
  - `link_liveness` ×2；`run_moba_split_local_test.lua` 的 outage 參數。
- **未修（記錄為已知限制）**：
  - 超過 30 s 判定斷線後沒有重新加入的流程：client 結束，hero 留在場上不動。
  - KCP 指數退避可能讓恢復後再多凍結數秒（kcp-0.5.3 nodelay 每次 RTO ×1.5、沒有上限），要看實機數據。
  - UE 端：bridge 目前把 STALLED 映射為 Joining，畫面顯示「正在加入對戰…」，應改成「連線中斷，等待恢復」並暫停操作；這屬於 omfue，交給主 session。
- **實機結果（split33，build-32）**：`check_netem_outage` 除 1 筆 mismatch 外全部 PASS。兩次 10 秒斷線都是 stalled→recovered；恢復花了 14.5 秒與 11.25 秒，第二次恢復後又 stalled 5.75 秒，符合 KCP 退避的預期。40 秒斷線時，p6 恰好判定 link lost 一次，server 也判定 link lost；沒有 terminated。
  - 唯一的 mismatch：team 2、tick 6000（team 1 恢復的那一刻），p6 剛生成的雜賀眾 gunner 的 ATTACK 元件不一致，rebase 已恢復。
  - 定點 gunner 加上 owner 對視野邊緣目標下指令的 parity 情境（`gunner_acquisition_with_owner_attack_orders_on_vision_edge_keeps_replicas_identical`）重現不出來。
  - 因此加強診斷：projector 保留最近 4 個 checkpoint 的完整 expected 元件；`ObserverMismatch` 帶出 replica 的元件內容。mismatch log 每列改為 `authority=<該 checkpoint 的確切內容> replica=<replica 內容>`，下次就能直接看出是哪個欄位。
- **避免**：任何依賴「對方會一直送資料」的網路路徑，都要有兩端一致的存活判定和明確的超時結果；量測工具必須在優雅停止時寫出證據，runner 也必須走優雅停止的路徑。

## E059 hero_author 產生器：append 切片缺中間槽時 rank 錯位；extra1 被動被輸出成孤兒內容（2026-10-25，H04 阿市）

- **現象**：H04 `validate` 回報 `generated ability c17_devotion missing from template ids`。追查時另外發現：append 英雄的切片若少了中間的槽（例如只建 W／E），`hero.abilities` 會被壓實成 `{W, E}`，但 `moba_loadout.ranks` 仍按原本槽位 `{0, r2, r3, 0}`，rank 與技能錯位。
- **根因**：`package_entries` 對所有技能都產生 ability entry，包括沒有 loadout 槽的 `extra1..3`；validate 卻要求所有 package 技能都要出現在 template ids。壓實 `list` 時沒有同步壓實 `ranks`。
- **通用修正**：
  - 沒有 `SLOT_INDEX` 的 action（extra 槽）在 G-3 `extra_action_slot` 完成前一律不輸出，列入 skipped。
  - validate 只檢查實際產生的 entries。
  - append 英雄壓實技能列表時，rank 跟著技能一起移動。
  - 新增測試 `append slices keep ranks with their abilities and never emit extra-slot abilities`。
- **避免**：任何把稀疏槽位轉成密集陣列的地方，要同時轉換所有平行陣列（abilities／ranks／cues）。新增 action 種類時，要確認產生器、validate 與 registry 三處採用同一個「哪些技能會被輸出」的規則。

## E058 hero_author build 對「新增英雄」寫不進 Rust 模組：目錄不存在；工具測試又依賴真實樹已建置的內容（2026-10-25，H03 阿松）

- **現象**：`shadow_apply omatsu <slice>` 回報 `write failed: .../heroes/A34_omatsu/No1_frost_bolt.rs: No such file or directory`；同時 `hero_author_test.lua` 在乾淨的修改前就已失敗（protected 數 5≠6、`abilities outside the slice are not registered`）。
- **根因**：H01／H02 都是 `update_existing`，Rust 模組目錄本來就在，`build.execute` 只會寫檔、不建目錄。第一個 `append` 英雄（A34_omatsu）才踩到。測試 fixture 從真實 `scripts/lua_data/templates` 複製，主 session 建置 H01／H02 後，真實樹的 `authored_heroes.lua` 已含 saika 完整 kit 與 summons hook，fixture 不再是「尚未建置」狀態。
- **通用修正**：`build.execute` 寫檔前補建缺少的上層目錄並記錄；回滾時刪除本次建立的目錄（最深者先刪）。`cli.default_deps` 提供 `rmdir`。fixture 一律以空的 `generate.render({})` 覆寫 authored 檔；dry-run 測試改為檢查「基礎檔案＋所有規劃寫入都受保護」，不寫死數量。新增測試 `a hero module directory that does not exist yet is created and removed on rollback`。
- **避免**：產生器／安裝器要同時測 update_existing 與 append 兩條路徑。工具測試的 fixture 不得依賴真實樹的建置狀態；從真實樹複製時，要把會被工具寫入的檔案重設為初始狀態。

## E057 分路小兵與野怪在 UE 一律面向世界 +X：權威端從未建立或更新它們的 Facing（2026-10-25，creep facing）

- **現象**：英雄的面向在 E040 之後已經正確，但分路小兵（lane_melee／caster／siege／super）和野怪不論往哪裡走、攻擊誰，畫面上都朝同一個方向。
- **根因**：UE 的 actor yaw 直接取自 frame 的 `facing_rad`，bridge 從 replica 的 `DISCLOSED_FACING` 讀取，沒有時預設為 0。`MobaMatch::spawn_wave` 和 `jungle::spawn_body` 建立的 NPC 沒有 `Facing` component，lane／camp 移動只改 `Pos`，movement fact 也送 `facing_ticks: None`。mesh 對齊沒有問題：這些 class 和已驗證的英雄使用同一組 QuaterniusV2 骨架，自動對齊全部開啟。
- **通用修正**：`moba_match::face_toward` 只有一條規則：可見正面＝這一步的位移方向，或正在攻擊的目標方向。分路小兵出生時面向下一個 waypoint，野怪出生時帶 `Facing`。面向改變時 PreStepMovement fact 帶上 `facing_ticks`，replica 經既有路徑寫回 specs 並納入 hash。新增 Rust 測試，以及 UE Editor 測試 `Om.Generated.EveryUnitClassFacesActorForward`。詳見 work-creep-facing.md。
- **避免**：新增任何會移動或攻擊的單位種類時，要確認它有 `Facing`，而且每個移動／攻擊路徑都經過 `face_toward`（或等效規則），並送出帶 facing 的 movement fact。測試要用有轉彎的地圖（three_lane_training）：直線路線上，固定面向剛好等於移動方向，測試會誤判通過（本次 mutation 驗證就踩到）。

## E056 權威端執行、replica 卻看不到的指令：回城取消前搖，以及目標在同一 tick 離開視野（2026-10-25，split17／18／19／20）

- **現象**：以下 observer mismatch 都靠 rebase 恢復。log 只有 digest，看不出是哪個 entity、哪個欄位。
  - split17 tick 33600：team 2，replica 2703，ATTACK `0x464f4706`。
  - split18 tick 23400：team 1，replica 14，HERO `0x464f4705`。
  - split19 tick 10800：FACING `0x464f4707`。
  - split19 tick 128280：PROPERTY `0x464f4702`。
  - split20 tick 20160：ATTACK。
- **重現**：`live_soak_replica_parity::role_bot_match_keeps_team_replicas_identical` 改用 split17 的 10 人 role bot 對局（`moba_role_match.lua`，p6 為 saika_magoichi，所有玩家都是 bot，p6 另外學技能並放定點 gunner）。tick 1799 時，team 2 的 replica 已經用掉一次 `b01_saika_corps` charge，權威端仍是 2 次。
- **根因 1（回城）**：p6 的 bot 在施法前搖期間（tick 1750）回城。權威端 `AbilityWindups::advance` 用 `MobaMatch::is_recalling` 取消前搖；filtered replica 沒有 `MobaMatch`，這個條件永遠是 false，所以照常放出技能、扣 charge、生成 gunner。`hero_tick` 的回城中停止攻擊（攻擊序列，正是 live 不一致的元件）和 `hero_command_tick` 的回城中停止方向移動，也是同一類只在權威端判斷的條件。
- **根因 2（同 frame 失去目標，E050 列為殘留風險）**：用同一個 soak 跑 18900 tick，在 tick 18862 重現 split18。
  - 玩家 4 對敵方小兵施放 `lumen_bolt`。權威端准入時，目標仍在視野內，施法成功、冷卻開始。
  - 但小兵在同一 tick 結束時離開視野，frame 18862 的 pre_step 先 Forget 它，projector 因此丟掉這筆輸入，replica 沒有施法。
  - attack（攻擊目標）、facing（轉向目標）、hero（冷卻）都會因為同一機制分歧。
- **通用修正**：
  1. `cast_windup::hero_recalling` 成為唯一的回城判斷：權威端讀 `MobaMatch`，replica 讀 `DisclosedRecallingOwners`。後者由 `SpecsDisclosedWorldStepper` 每個 frame 從隊伍專屬的 recall-active HUD fact 寫入，與既有清除指令的來源相同。前搖取消、攻擊序列、方向移動三處都改用它。
  2. 伺服器輸入准入抽成 omoba-core 的 `sanitize_accepted_action`、`resolve_accepted_target`；omb 與 parity harness 共用同一份邏輯（harness 依序套用 lane gate、item owner seal、sanitizer、霧戰規則）。
  3. soak 泛化為 `soak_match(config, ticks, compare_every, prepare, inputs)`，可以直接用 role bot recipe 跑整場對局。
  4. 指令目標 grace：`TeamVisibilityState` 新增 `sighted`（真實視野）和 `input_grace`（本 tick 已准入輸入的目標）。wave B 會讓目前已揭露的 grace 目標多留一個 frame，replica 因此能解析並執行同一筆輸入。准入規則改為 `TeamProjectionRuntime::input_target_admissible`，要求目標同時已揭露且在 `sighted` 內，所以 grace 不會連鎖延長視野，也不會把隱藏單位揭露出來。
  5. mismatch log 每一列附上 `canonical=… kind=hero|summon|creep|tower|other|dead current=<該隊視角目前的元件內容，截斷 320 bytes>`，下次可以直接判斷是哪個 entity、哪個欄位。
- **驗證**：10 人 bot 對局。修正前在 tick 1799（回城）和 18862（失去目標）失敗；修正後跑完整場。另有單元測試 `input_grace_keeps_a_targeted_entity_one_frame_without_extending_sight`、`replica_without_moba_match_cancels_windup_on_its_disclosed_recall`。
- **避免**：gameplay 判斷如果讀的是只有權威端才有的資源（`MobaMatch`、host lifecycle），而 replica 也會跑同一段程式，必須提供 replica 端的揭露來源，或證明該段只在權威端執行。新增這類條件時，用 `soak_match` 加上 role bot 跑至少 12000 tick。
- **未解（交給 split22 的新診斷）**：「英雄擊殺野怪後 1～2 秒出現 attack／hero 揭露不一致」的 live 規律（split17／18／20／21），在 harness 裡重現不出來。已確認 `level_one_heroes_kill_neutral_camps_keep_team_replicas_identical` 確實擊殺 28 隻野怪，完整 130000 tick 的 bot 對局也沒有出現，即使關掉 grace 也一樣。剩下的差異在 omb 自己的 tick 管線、UE client 與 rebase。新的 mismatch log 會帶出 canonical、kind 與當下內容，用來定位。
- **後續（主 session，2026-10-25）**：split27（build-29，含 E056／E057，quick map，p6 saika，10 分鐘時限）整場 observer mismatch 0、terminated 0、late input 0，期間有野怪擊殺、召喚與回城。先前 7～9 分鐘出現的 attack／hero 揭露 mismatch 沒有再發生；仍在多跑幾局確認穩定。

## E055 地圖欄位只接在一條設定路徑上：角色配方對局沒有吃到 map tower_hp／base_hp（2026-10-25，quick 測試地圖）

- **現象**：split25 使用 three_lane_quick_training（tower_hp=600、base_hp=800，generated const 正確），match_status 的塔與主堡 max_hp 仍是 1200／1600。
- **根因**：MOBA 對局的 SingleLaneConfig 有三個建立點：server_config（MATCH_MAP_ID）、角色配方 `RoleBotMatchPlan::compile`（bots/plan.rs）、`moba_headless --map`。新欄位只加在第一個，而 role recipe 對局走的是第二個。
- **通用修正**：plan.rs 與 moba_headless 都改從 map 取 tower_hp／base_hp。新增測試 `role_bot_plan_takes_structure_hit_points_from_the_compiled_map`，同時驗正式地圖（1200／1600）與 quick 地圖。
- **避免**：新增地圖欄位時，先用 `rg "lane_length: *Fixed64::from_i32\(map"` 找出所有從 map 建立 config 的地方，一次接齊；並用實際跑的 match_status 確認值有生效。

## E054 打包版沒有地面材質、英雄模型和道具圖示：只用軟路徑載入的資產沒被 cook（2026-10-25，J6）

- **現象**：用 `--packaged-game` 跑 split22 能連線並開打，但地面是藍色背景加灰色、英雄只剩名牌。ue-p1.log 載入失敗：M_grass 1166 次、female_peasant／male_peasant／male_ranger，以及 UI Items 的圖示。
- **根因**：生成的內容類別（OmContentClasses.cpp 有 724 處）與 world surface 都是在執行時用 `FSoftObjectPath(TEXT("/Game/Omoba/..."))` 載入，沒有任何硬引用；DefaultGame.ini 只把 Audio 兩個目錄列入 cook，其餘全被 cooker 略過。Editor 是直接讀 uasset，所以看不出問題。
- **通用修正**：DirectoriesToAlwaysCook 加入 `/Game/Omoba` 與 `/Game/OmGenerated`。新增 `scripts/tests/ue_cook_coverage_test.lua`：掃描 plugin 原始碼中所有軟路徑的 `/Game/<root>`，任何一個沒被 cook 設定涵蓋就失敗（已確認拿掉 `/Game/Omoba` 時會失敗）。
- **避免**：只在 Editor 驗證的畫面不能代表打包版；每次打包後都要實際啟動，並在 log 檢查 "Failed to load"。新增軟路徑內容根目錄時，要讓上面這個測試保持通過。

## E053 G-2 引導讓 replica 拒絕己方施法，且敵方暈眩從未送到己方 replica，split16 observer mismatch 後終止（2026-10-25，split16）

- **現象**：split16 第一次 `lumen_tremor` 施放（tick 3706）後 14 tick，observer 在施法者身上 mismatch（entity=1，schema 0x464f4705 DISCLOSED_HERO）；第二次施放同樣出問題，rebase 後再次失敗，最後終止。log 中 `AbilityCast ok` 出現兩次，`AbilityChannel start` 卻只有一次。
- **根因**：
  1. 引導的准入（`stage_ability_channel`、`ability_channel::live`）與開始（`begin_ability_channel` 的 `write_resource::<MobaMatch>()`）都要求 `MobaMatch`。replica 沒有 `MobaMatch`，所以拒絕了自己隊伍的引導施法，不扣魔力也不進冷卻，從第 0 tick 起和權威分歧。
  2. 通用缺口：敵方造成的 stun／silence／root 只存在權威的 BuffStore，己方 replica 永遠看不到。被暈的英雄施法時，權威拒絕、replica 准入，冷卻分歧。tremor 每 0.25 秒暈一次，讓這個問題幾乎必然發生。
  3. replica 的 specs BuffStore 不在 bootstrap 中，rebase 會丟失引導與控制狀態。
- **通用修正**：replica 不需要 `MobaMatch` 就能用相同的施法者狀態准入並開始引導。新增 team-private fact `OwnerControl`（`native/owner_control.rs`）：每個 outcome batch 後送出己方英雄的控制剩餘與引導狀態，replica 在同一個 batch 後套用，並由權威決定引導何時結束。新的 local life（bootstrap、rebase、reveal）從 owner-only 的 `buff_visual_state` baseline 播種。詳見 work-G2-channel-desync.md。
- **避免**：新的 host 生命週期（引導、讀條、任何 replica 會在 gameplay 中讀取的 buff）一定要加入 `channel_replica_parity`，逐 tick 比對 hash 與 replica specs BuffStore，並包含 rebase 中途與被敵方控制的情境。不能只看 hash：BuffStore 不在 hash 內，要用 `status_rows` 這類直接比對，並做 mutation 驗證（關掉修正後測試必須失敗）。

## E052 Windows 上任一 KCP peer 離開，就讓所有連線每次停 1 秒（WSAECONNRESET 10054）（2026-10-25，split18）

- **現象**：split18 跑到 30 分鐘時，omb-mcp 的 match_status 一直 timeout；server log 每秒都出現 `udp.recv_from failed ... (os error 10054)`，共 148 次。也可能是之前大量出現「rejected late InputSubmit」的原因。
- **根因**：所有 KCP session 共用 server 的同一個 UDP socket。Windows 收到某個已離開 peer（短命的 mcp 查詢、關掉的 renderer）的 ICMP port unreachable 後，下一次 `recv_from` 會回 WSAECONNRESET；tokio_kcp 的 listener 遇到錯誤就 `sleep(1s)`，整台 server 的收包因此停住。server 還會繼續對舊 session 送資料，於是每秒重複發生。
- **通用修正**：omb `bind_kcp_listener` 改為自己綁 std UdpSocket，在 Windows 上以 `WSAIoctl(SIO_UDP_CONNRESET, FALSE)` 關閉這個行為，再用 `KcpListener::from_socket` 建立 listener。新增測試 `udp_connreset_tests`：送到已關閉的 port 之後，同一個 socket 仍能收到其他 peer 的封包。netem proxy 本來就會略過 10054。
- **避免**：Windows 上所有共用的 UDP server socket 都要關閉 SIO_UDP_CONNRESET；第三方 listener 只要有「出錯就 sleep」的迴圈，都要確認錯誤來源不會由單一 peer 觸發。

## E051 MOBA 按 W／E／R 送錯欄位：舊 TD 藍圖輸入圖繞過 owner component（2026-10-25，split15）

- **現象**：兩位玩家按 R，server 每次都收到 ability_index=2 且沒帶目標，index 3 從未出現；W→0、E→1。UE log 只有 Q 有 `OM_CAST ... label=Cast`，另有大量 `[BP_PlayerController_C_0] Ability R accepted`。
- **根因**：`Content/RustBP/System/BP_PlayerController` 留著 TD 時期的 "Move attack tower ability keys" 圖，用 WasInputKeyJustPressed 輪詢 W/E/R/T，當成欄位 0～3 呼叫 `SubmitCastAbilityNoTarget`。輪詢不受 C++ 是否已消耗按鍵影響，所以真人按 R 也會中。它的本地 accepted 還讓 autopilot 以為已送出，略過了確認點擊，正確的 C++ 施法因此從未送出。
- **通用修正**：新增 `OmOwnedInput::FOwnerDispatchScope`（OmOwnedInputDispatch.h）。owned MOBA input 啟用時（bOwnedAbilityHudReady），AOmWorldBridgeActor 的舊 Blueprint 入口 SubmitCastAbilityNoTarget、SubmitAttackMoveFromScreen、SubmitUpgradeAbility 只接受 owner component 在 scope 內的呼叫，其餘一律拒絕。TD 模式不受影響。函式仍保持 BlueprintCallable，reflection 測試不變。
- **避免**：MOBA 輸入只能有 UOmPlayerActionOwnerComponent 一個來源。查「指令送錯」時，先在 UE log 找 `[BP_` 開頭的訊息，確認有沒有第二個來源。藍圖裡殘留的舊圖之後可在 Editor 刪除（.uasset 是二進位，不能用文字修改）。

## E050 同 tick 的隱藏效果改變施法准入、指令可指定隊伍看不到的目標，split13 在 42.5 分鐘 observer-mismatch-after-rebase 後終止（2026-10-25，split13）

- **現象**：split13 跑到 42.5 分鐘（tick 152999）時，observer validation 連續兩次 mismatch，比對在 rebase staging 期間又失敗，伺服器以 `observer-mismatch-after-rebase` 安全終止。log 只有 hash 不一致，看不出是哪個 entity 或 component；client 沒收到任何通知，畫面直接卡住。
- **根因**（用新增的 `live_soak_replica_parity` soak 重現：兩隊英雄對打、道具主動技、魔力吸取、lumen_mend、死亡與重生，每 tick 比對兩隊 replica）：
  1. **同一批結算中，前面的施法改變後面施法的准入。** 兩名英雄同 tick 互放 `lumen_bolt`（帶魔力吸取）。權威先結算玩家 1，吸取讓玩家 2 的魔力低於 `expected_mana` 快照，玩家 2 的施法因此被 `cast_settlement::admit` 拒絕；玩家 2 隊伍的 replica 看不到玩家 1 的隱藏輸入，照常准入並預測冷卻，兩邊從此分歧。控制、致死傷害等任何同 tick 隱藏效果都屬於同一類問題。
  2. **指令可以指定隊伍看不到的 entity。** projector 會丟掉目標無法對應到隊伍 replica id 的 accepted input，但權威端照樣執行。例如無目標技能（`lumen_touch`）帶著過期的敵方 id、對霧中單位攻擊或使用道具，replica 都看不到這筆輸入。這也是一個霧戰作弊管道。
- **通用修正**：
  1. `game_processor::process_outcomes` 新增批次預先准入：每批結算開始時，依序以批次開頭的世界狀態准入所有 `ScriptCastTransaction`，並立即套用該施法者自己的魔力 commit（`cast_settlement::apply_caster_mana`），之後才結算任何效果。召喚擺放（`plan_summons`）也在同一個批次開頭世界判定，失敗就不准入、不扣魔力。結果包成 `Outcome::ScriptCastPreAdmitted`；批次中途才產生的 transaction 走同一個 `pre_admit`。
  2. `TeamProjectionRuntime::target_disclosed(team, canonical)` 成為輸入目標的唯一霧戰規則。omb `build_canonical_accepted_inputs_with_fog` 會把目標未揭露的輸入連同原始輸入一起丟棄，權威端不再執行 replica 看不到的指令。
  3. 診斷：projector 每個 hash checkpoint 保存 component digest（最多 128 個 checkpoint）；`ObserverMismatch` 帶上 replica 的 digest。伺服器以 `observer mismatch team=… replica_tick=…` 記錄前 16 筆不一致的 `entity／schema／digest`（只有 hash，沒有內容，不洩漏霧中資料）。
  4. 終止通知：安全終止時送一次 `LockstepFrame::MatchTerminated`，KCP tag `0x36`（`[u64 BE last_safe_sequence][u8 len][ascii reason ≤64]`）。client runtime 會發布 `RuntimeSessionState::Terminated`（`match_terminated:<class>`）並以 `ShutdownReason::MatchTerminated` 結束；舊 client 遇到未知 tag 只會 warn。
- **殘留風險**：輸入被接受的那個 tick 結束時，如果目標剛好離開視野（同一 frame 的 pre_step 做 Forget/Hide），projector 仍會丟掉該輸入，可能觸發一次 FilteredRebase。要徹底修正，需要讓 replica 在套用 pre_step 的 hide 之前先注入輸入（協定層變更），列為後續工作。
- **避免**：新增會在同一批結算中改變他人資源或狀態的效果時，用 `live_soak_replica_parity::soak` 加入同 tick 互相施放的情境；准入檢查只能讀批次開頭的狀態。新的 input 種類如果帶 entity 目標，必須經過 `target_disclosed`。

## E049 autopilot 只對英雄施法、技能固定先點 R 和 Q，導致新英雄的技能完全沒被實測（2026-10-25，G-2 實機）

- **現象**：split14 用 training_channeler 跑了 7 分鐘，W（lumen_tether）從未學習，R（lumen_tremor）沒有施放過；trace 中沒有任何 cast intent。
- **根因**：ChooseUpgrade 固定照 R→Q→W→E 的順序點技能，導致 W 一直沒學；ChooseCast 只瞄準可見的敵方英雄，分路上沒碰到敵方英雄時就一直不施法。
- **通用修正**：升級順序改為 R 有點數就點，其他技能先各學一級再加深；施法改為兩階段，第一階段找敵方英雄，找不到且法力至少 70% 時才對小兵施法。AutoPilotAbilityRule 測試補上兩種情況。
- **避免**：驗收新技能時，先從 trace 確認每個 slot 都有 learn 與 cast 紀錄，再談效果；coverage 不能只看 intent 總數。

## E048 ABI 升版沒有同步所有 pin：header smoke 停在 35，integration 測試缺 ABI36 欄位（2026-10-25，reject-reason）

- **現象**：ABI 升到 37 時發現 `omfue/bridge/smoke/om_bridge_header_smoke.cpp` 還在 `static_assert(OM_ABI_VERSION == 35)`；`omfue/bridge/tests/runtime_driver_smoke.rs` 自從 ABI36 加了 `OmInputCommand.summon_command` 之後就編不過（`missing field summon_command`）。`cargo test --lib` 會通過，所以一直沒被發現。
- **根因**：升 ABI 時只改了 lib 測試裡的 pin；smoke 不在 cargo 的編譯範圍，integration test 也沒有被一般驗證流程執行。
- **通用修正**：本次把 smoke 更新到 37，並加上 `OmAppliedInput` 的 layout 斷言。integration test 的缺欄位屬於 ABI36 的範圍，這裡沒有修，留給負責 summon ABI 的人處理。
- **避免**：升 ABI 時用 `rg "ABI3[0-9]|OM_ABI_VERSION"` 找出所有 pin（Rust 單元測試、smoke、UE Editor 測試），並至少跑一次 `cargo test --manifest-path omfue/bridge/Cargo.toml`（包含 integration test），不能只跑 `--lib`。
- **後續（主 session，2026-10-25）**：`runtime_driver_smoke.rs` 的三個 `OmInputCommand` 補上 `summon_command: Default::default()`，`cargo test --test runtime_driver_smoke` 結果為 2 passed、1 ignored。

## E047 Editor 測試直接讀 protected UPROPERTY，UE 建置時出現 C2248（2026-10-25，E040 後續）

- **現象**：consistent-build-18 在 OmEditorAutomationTests.cpp 失敗，原因是 `Hero->bAutoAlignMeshForward` 是 protected。
- **修正**：和同一個測試讀 NativeMeshTransform 的方式一致，改用 `FindFProperty<FBoolProperty>` 讀取。
- **避免**：在 AOmAnimatedUnitActor 的 protected 區段新增 UPROPERTY 時，測試一律透過 reflection 讀取；改了 UE 程式碼後，不要只跑 Rust 測試就當作已驗證。

## E046 replica 會替 host-only 生命週期回報假的施放拒絕（2026-10-25，G-2／ack-result）

- **現象**：設計 G-2 時發現，owner replica 沒有 MobaMatch，recovery／pulse channel 一定 stage 失敗；如果照原本做法把 dispatch 後段的失敗寫進 journal，權威端接受的施放會在 client 被回報成 CAST_REJECTED。
- **根因**：「replica 與權威跑同一段程式」只在兩邊有同一份狀態時成立。host-only 生命週期與隱藏資訊不在 replica 上。
- **通用修正**：非權威 world 只記錄靠 owner 自己英雄狀態判定的碼（`command_outcome::owner_state_code`）；其餘拒絕碼只由權威 world 記錄。
- **避免**：client 端推論權威結果時，先列出每個判定用到哪些狀態，確認 replica 擁有其中每一項；缺任何一項就寧可漏報，也不能誤報。

## E045 GameWorld 的 sabi_trait 方法表已滿，新增方法讓整個 scripts workspace 編不過（2026-10-25，G-2）

- **現象**：在 `GameWorld` 新增 `channel_pulse` 後出現 `proc-macro derive panicked: attempt to shift left with overflow`，base_content 整個編不過；其他 session 在共用樹上被擋住。
- **根因**：abi_stable 的 sabi_trait 方法表有上限，GameWorld 已經用滿（`TowerActiveAbilityAccess` 的註解寫過這件事）。我是直接在共用樹上嘗試，沒有先在隔離副本驗證。
- **通用修正**：撤掉 GameWorld 的修改；pulse context 改用 `AbilityScript::on_channel_pulse` 的參數傳入，預設實作會轉呼叫 `on_tick`。
- **避免**：ABI 層的改動先在 shadow 樹編譯通過再著陸；GameWorld 不要再加方法，改用 hook 參數、另開窄的 sabi_trait，或保留 payload key。

## E044 技能測試依賴英雄目前的技能欄（2026-10-25，H01）

- **現象**：H01 換掉雜賀孫市的技能欄後，17 個舊技能的正式測試失敗（以 slot index 施放 saika_reinforcements 等）。
- **根因**：測試用「英雄名＋slot 編號」指定技能，等於把英雄內容寫死在技能測試裡。
- **通用修正**：新增 `legacy_world`／`restore_legacy_kit`：測舊技能的測試明確指定手寫 kit（技能欄、出生 rank、移除被替換技能的等級與次數）；`pin_legacy_loadout` 也走同一條路。新 kit 與通用測試仍使用內容的 kit。
- **避免**：技能測試要明確宣告它需要的 loadout，不依賴某位英雄目前的內容。

## E043 build 寫入的檔案沒有全部納入回滾保護（2026-10-25，E2／H01）

- **現象**：dry run 會寫 `templates/summons.lua`，回滾清單卻沒有它；build 失敗時 hook 會殘留。
- **根因**：回滾清單是手寫的固定清單，新增 hook 時忘了同步。
- **通用修正**：回滾清單改由 writes 自動聯集，凡是寫入的檔案都受保護；測試的快照也加入 summons.lua。
- **避免**：受保護的集合要由實際寫入的檔案推導，不要另外維護一份清單。

## E042 測試在施法前搖結束前就檢查結果（2026-10-25，H01）

- **現象**：雜賀眾的正式測試判定「沒有召喚、沒有扣次數」，看起來像施放失敗。
- **根因**：B01E 的 AbilityCastPoint 是 0.15 秒；權威端要過了前搖才結算，測試卻在下一個 tick 就斷言。
- **通用修正**：測試在 `cast_skill` 之後先推進 `WINDUP_TICKS`，再斷言。
- **避免**：有 cast_time 的技能，測試要從 content 的 cast_time 推算等待的 tick 數。

## E041 假設 MOBA 英雄帶有 `hero_<id>` script tag（2026-10-25，H01）

- **現象**：雨鐵炮把真實 MOBA 敵方英雄判成部隊，加成取 50% 而不是 10%。
- **根因**：只看了 story 初始化（`initialization.rs`）的命名規則。MOBA 的 `spawn_hero` 不掛 ScriptUnitTag，`unit_id_of` 因此回傳 None。
- **通用修正**：adapter 的 `unit_id_of` 在沒有 tag 時回傳 Hero 元件的模板 id；分類同時接受舊的 `hero_` 前綴。用正式 MOBA 世界的真實英雄、小兵、塔做每個 rank 的驗證。
- **避免**：依賴 host 身分的規則，一定要用正式對局世界的實體測一次，不要只用測試 fixture 建出來的 body。
## E040 角色面向偏 90°：模型正面沒有對齊 actor 的 +X（2026-10-25，C／D 視覺）

- **現象**：facing_probe 顯示 actor／mesh yaw 等於移動方向（95° 對 90°），畫面上角色卻是側身移動。female_peasant 是 Mannequin 骨架，腳尖（ball_l／ball_r）朝 -Y。
- **根因**：codegen 把 `render.yaw_offset_deg`（Fyrox 座標系的舊值）直接當成 UE yaw；其他英雄的 yaw 預設是 0，也沒有人檢查匯入模型實際朝哪個方向。
- **通用修正**：`AOmAnimatedUnitActor::ComputeSkeletalForwardYaw` 從 reference pose 計算腳到腳尖（toe／ball）的平均方向；`bAutoAlignMeshForward`（預設 true）載入時把這個方向轉到 actor +X。只有 Unreal 端明確寫了 `ue.native_visual.yaw_deg` 才視為人工覆寫，codegen 會輸出 false。autopilot 的 facing_probe 也共用同一個函式。
- **避免**：跨引擎的座標值（Fyrox yaw／pitch）不能直接沿用；朝向要從資產本身推導，或由 UE 端明確設定。

## E039 新增 launch 步驟（shader warmup）讓 `test_moba_role_launch.lua` 失敗（2026-10-25，A3）

- **現象**：fake process 測試裡的 pid 順序多了一個 warmup renderer，`stopped=='2,1'` 的斷言失敗。
- **根因**：warmup 寫死在 launch 裡，plan 無法表達要不要執行，測試也就沒辦法略過。
- **通用修正**：`plan.shader_warmup` 由 prepare 設為 true，launch 依 plan 決定是否執行；fake renderer 測試把它設成 false，並斷言預設值是 true。
- **避免**：每在 launch 加一個步驟，就要跑 `scripts/test_moba_role_launch.lua`。行為要用 plan 欄位控制，不要只寫在程式流程裡。

## E038 bash heredoc 寫 Lua 時 `[/\]` 變成 `[/\]`，出現 invalid escape（2026-10-25，工具）

- **現象**：用 `<<'EOF'` 寫出的 patch 腳本第一行在 Lua 裡解析失敗。
- **修正／避免**：要在字串裡放反斜線時，改用 `string.char(92)`：`local dir=(arg[0]:gsub(string.char(92),"/")):match("^(.*)/")`。

## E037 實機 desync：`observer-mismatch-after-rebase`，伊達政宗施放 b04_flame_trail_dash 後整局終止（2026-10-25，G-1／G-2 修正）

- **現象**：split9 第 5520 tick，`secure match terminated ... reason=observer-mismatch-after-rebase`，client log 沒有錯誤。
- **重現**：`single_lane_match_tests::scripted_motion_replica_parity`。權威 world 與兩隊 filtered replica 同步前進，施放 frost bolt（追蹤）、frost lance（直線）、sentry（charges＋召喚）、b04_flame_trail_dash（windup dash＋G-1 追蹤火焰），每個 tick 比對 `canonical_team_hash`。修正前有 552 筆元件不一致。
- **根因有三個，都屬於同一類：揭露給 replica 的權威狀態，replica 沒辦法照著重算出來。**
  1. **G-1 投射物**：飛行時沒有發 `Movement` fact，replica 的位置停在原地；另外 flight 元件每一步都在改 heading，目標失效時 motion 也會從 2 變 3，這些都沒有對應事件。
  2. **windup 施放**：filtered replica 沒有 `MobaMatch`，所以 `run_script_dispatch` 的 `enforce_learned_ranks` 是 false，整段 windup 被略過。replica 在輸入當下就施放，冷卻比權威早一個 tick。這是既有缺口，H02 是第一個帶 cast_time 的 MOBA 技能，才把它暴露出來。
  3. **G-2 charges**：敵方英雄消耗次數時沒有 committed fact，敵隊 replica 的 Hero JSON 不會更新。
- **通用修正**：
  1. 投射物每一步都發 PROJECTILE policy 的 `Movement` fact（與舊投射物相同）；flight 元件改成 v2，生命期內不變（version 2、motion、speed、radius）。
  2. 新增 `cast_windup::FormalCastRules` marker，由 filtered replica world 插入。dispatch 在有 `MobaMatch` 或有 marker 時，都套用相同的 windup、施放中拒絕、rank 規則；只跟 `MobaMatch` 有關的 phase 與 pause 檢查仍然加上保護。
  3. 新增 `FactKind::CommittedCharge = 39`：結算 `ScriptConsumeCharge` 時發出，內容是該 slot 的完整狀態；擁有隊伍自行預測，所以會被過濾掉；replica 收到後寫回 Hero JSON。selective 協定升到 18。
- **避免**：任何新的權威狀態，只要會進 disclosed baseline（位置、元件、Hero JSON 欄位），合併前都必須通過 team replica parity 測試：權威與兩隊 replica 逐 tick 比對 hash。replica 無法預測的變化，一律要有對應的 committed／Movement fact，或者乾脆不揭露。

## E036 package 的 lifecycle 宣稱「只鎖定可見敵人」，handler 卻沒有做到（2026-10-25，H02）

- **現象**：`b04_flame_trail_dash` 的 `lifecycle.vision_lost` 寫「火焰只鎖定發射當下可見的存活敵人」，但 handler 以沒有 disclosure 過濾的 `query_enemies_in_range` 選目標。
- **根因**：寫規格時沒有核對 script ABI 是否有 disclosure 過濾的查詢（沒有，而且 sabi 方法表已滿）。驗收情境也沒有涵蓋「霧中目標」。
- **通用修正**：規格中每一條可見性政策，都要對應一個 dual-replica 驗收情境（authority＋team-filtered replica，hash 一致）。這次已寫 harness（side-session-notes 的 h02-windup-dual-replica-repro.rs）；目標過濾待 G-1 的 homing admission 或 disclosure 查詢完成後補上。
- **避免**：package 的 lifecycle／visibility 敘述不是說明文字，是要被測試的行為；寫進去之前先確認引擎有對應能力，沒有就標 needs_framework。
## E033 權威拒絕的施放／升級仍回給 renderer「accepted」（2026-10-25，ack-result）

- **現象**：server log 有 `AbilityCast failed ... still on cooldown`，renderer 與 autopilot 收到的卻是 `accepted / APPLIED_TO_PRESENTATION`，只能事後從 HUD 沒變化推論 `no_effect`。
- **根因**：`settled_inputs` 把「權威把輸入排進這一 tick」直接當成結果；cast／upgrade 的拒絕只寫 log，而且拒絕點（admission、dispatch 的多個 `continue`、handler `RErr`、transaction admission）都沒有帶 input id。
- **通用修正**：`command_outcome::OwnerCommandOutcomes` 依 input id 記拒絕碼；每個拒絕點都回傳碼。權威端和 owner replica 跑同一段程式，client 從 replica 讀出拒絕碼，把結果改成 `accepted=false` 加原因碼。協定不變。
- **避免**：ACK、排程、套用、拒絕是不同狀態，結果碼要分開。任何會把指令丟掉的 `continue` 或 `Err`，都必須帶出可回報的原因。

## E032 召喚物 AI 測試永遠找不到目標：fixture 沒有跑隊伍視野 wave（2026-10-25，G-3）

- **現象**：定點砲台的 `asd_count` 一直停在 ready，敵人始終不扣血，畫面上看起來像 AI 沒有被呼叫。
- **根因**：`query_nearest_enemy` 對召喚物要求目標已經揭露給該隊（`DisclosureContext`）。base_content 的單線 fixture driver 不會自動跑 `run_committed_visibility_wave_b`，所以新建立的敵人從未被揭露。這是正確的 fail-closed 行為，不是 AI 的 bug。
- **通用修正**：測試在每個 step 前呼叫 `run_committed_visibility_wave_b(&mut w, driver.tick(), 0)`，與既有的 summon focus 測試相同。
- **避免**：凡是要靠視野選目標的測試（召喚物、塔、bot），先確認 fixture 會跑視野 wave；遇到「AI 沒反應」時，先用小型診斷測試印出計數器與揭露狀態，再判斷是不是邏輯錯。

## E031 Lua 的空表 `{}` 序列化成 JSON object，build.rs 的 `effects` 解析失敗（2026-10-25，G-3）

- **現象**：`omoba-template-ids` 的 build script panic：`buff visual sources: invalid abilities: invalid type: map, expected a sequence`。
- **根因**：在只有召喚投遞、沒有 effects 的技能上寫了 `effects={}`。Lua 無法區分空陣列與空表，轉成 JSON 時變成 `{}`（與 E015 同源）。
- **通用修正**：選填的陣列欄位直接省略不寫，由 serde default 補空 Vec。生成器（`gen_hero_registry.lua`）在本地自行補 `effects={}`，不要寫進內容資料。
- **避免**：內容資料裡永遠不要出現空表字面值；需要「沒有」時就省略欄位。


## E030 測試預期與幾何不符：把觸發範圍端點外的單位當成不會被鎖定（2026-10-25，H02）

- **現象**：衝刺被薄牆截斷後，牆後 210 的單位被火焰鎖定，測試判失敗。
- **根因**：觸發區是以 300 為半徑的膠囊形，終點外仍有半圓。來源的 aura 也不受牆阻擋，所以程式行為正確，是測試預期寫錯。
- **通用修正**：把測試的牆後單位放在 900（超出終點半圓），並在測試中寫明原因。
- **避免**：幾何測試的邊界案例要先按定義算出期望值，不要憑直覺寫。

## E029 預設 moba_loadout 讓大招出生就是 rank 1，Lua 驗證沒有攔下（2026-10-25，H02）

- **現象**：shadow 中 `omoba-template-ids` 的 build.rs 失敗：`invalid MOBA birth rank 1 for 'b04_tanegashima'`。`hero_author validate` 卻回報 OK。
- **根因**：`MobaLoadout` 預設 ranks=[1,1,1,1]，而來源大招要 6 級才能學。validate 只跑 `gen_hero_registry`，沒有實作 `omoba-content-model` 的 `validate_moba_loadout`。
- **通用修正**：package 可選 `moba_loadout`，未指定時由 `required_hero_level` 推導出生 rank；generator 依技能欄寫入；hook 只替換被建置的欄位；validate 加 Lua 版的 loadout 規則；schema 也檢查出生 rank。都有測試。
- **避免**：生成器只要依賴 Rust 端的驗證規則，就在 Lua 端複製同一條規則並測試，不要等 cargo 才失敗。

## E028 斷言本遊戲沒有三圍（2026-10-25，D3／H01／H02）

- **現象**：D3 與兩份 package 寫「本遊戲沒有三圍」。實際上 `HeroDefinition` 有 strength／agility／intelligence 欄位，`comp/hero.rs` 也有舊屬性公式。
- **根因**：只看了 MOBA 數值來源，沒有查模板 schema 就下結論。
- **通用修正**：改正措辭：欄位存在，但 MOBA 路徑使用 authored 數值，舊公式與 `StatsAgilityBonus` 沒有消費者（已 grep 確認）。換算規則不變。
- **避免**：寫「沒有 X」之前，先 grep schema 與消費端；要分清「欄位不存在」和「欄位存在但沒有效果」。

## E027 shadow 樹重新同步時蓋掉自己的修改（2026-10-25，H02）

- **現象**：為了在 BUILD_FREEZE 期間編譯，把 crate 複製到 `target/session-h02/shadow`。重新同步時，`swept_dash.rs`、`self_transform.rs`、`mod.rs` 的 shadow 修改被 repo 版本覆蓋。
- **根因**：同步規則是「大小不同就複製」，把「shadow 端有修改」誤判成「repo 端較新」。
- **通用修正**：同步只補缺少的檔案；要更新的目錄必須明確列為 refresh_dir；所有修改都寫成可重放的 patch spec，被覆蓋時可以重放。
- **避免**：任何鏡像或同步工具都要先定義哪一邊是真相；預設不覆蓋已存在的目的檔。
## E026 i128::div_ceil 在 Rust 1.95 仍是 unstable（2026-10-25，G-1）

- **現象**：base_content 編譯失敗，錯誤為 `E0658 use of unstable library feature int_roundings`。
- **根因**：直覺以為 `div_ceil` 對所有整數型別都已 stable，實際上 signed／i128 版本在固定的 1.95.0 toolchain 仍是 unstable。
- **通用修正**：正數向上取整改寫成 `(a + b - 1) / b`。
- **避免**：toolchain 固定在 1.95.0，用到較新的整數 API 時先在自己的 CARGO_TARGET_DIR 跑一次 `cargo check`，不要在共用 checkout 留下不能編譯的狀態。

## E025 在 Git Bash 用 `sed -i` 改 CRLF 檔，整檔行尾被改成 LF，多行比對也失敗（2026-10-25，G-1）

- **現象**：`perl -0pi`／`sed` 以 `\n` 比對多行時在 CRLF 檔上悄悄不生效；`sed -i` 單行替換後，整個檔案的行尾變成 LF（`file` 不再顯示 CRLF）。另外 `lua.exe -e` 沒有 `arg` 表，用它寫的臨時改檔腳本直接失敗。
- **根因**：大部分 Rust／Lua 原始檔是 CRLF；msys 的 sed 以文字模式改寫，會丟掉 `\r`。git 的 autocrlf 讓 diff 看不出來，但檔案確實被改寫了。
- **通用修正**：多行或結構性修改一律用 Edit 工具（它保留 CRLF）；`sed` 只用在單行替換，事後用 `file` 檢查行尾。
- **避免**：先用 `file <path>` 看行尾，CRLF 檔不要用 heredoc 或 `\n` 比對做修改（與 E010 同源）。

## E024 定點時間的測試期望值用理想秒數推算，120 Hz 時差一個 tick（2026-10-25，G-1）

- **現象**：近距命中的 tick 期望值 `ceil(290×120/1200)=29`，實際是 30。
- **根因**：每 tick 的 `DeltaTime` 是把 1 秒的 1024 raw 做整數分割（120 Hz 時每 tick 是 8 或 9 raw），累積時間比理想值慢，命中時間要用 raw 時間表推算。
- **通用修正**：期望值改用 `(1..).find(|t| (t*1024/hz) * speed >= distance*1024)`，以實際的 raw 累積時間計算。
- **避免**：凡是「第幾 tick 發生」的斷言，一律用 `fixed_raw_for_tick` 的整數時間表推算，不用浮點秒數乘以 Hz。

## E023 協定升版時，漏改 base_content 測試裡寫死的 selective 版本（2026-10-25，G-1）

- **現象**：base_content 的 `public_range_formal_60hz_authority_filtered_snapshots_pause_expiry_and_replay` 失敗，錯誤為 `left: 16, right: 15`；這個測試對 bootstrap 寫死了 `assert_eq!(start.protocol_version,15)`。
- **根因**：升版時只 grep 了常數名稱 `SELECTIVE_LOCKSTEP_PROTOCOL_VERSION, 15`，而且只搜 omoba-core、client-runtime、omb，沒有搜 `scripts/base_content`，也沒搜「欄位 == 數字」這種寫法。
- **通用修正**：解凍後把該測試改成 16。bridge 的 `omfue/bridge/src/lib.rs:6171`（presentation 12）留給主 session 修改。
- **避免**：升版前先跑 `rg -n "protocol_version\W*,?\s*1[0-9]|PROTOCOL_VERSION\W*,?\s*1[0-9]" omoba-core omoba-client-runtime omb/src scripts/base_content omfue/bridge`，列出所有寫死的版本後一次改完；升版要在 BUILD_FREEZE 之外的時段進行，並先通知主 session（E018）。


## E017 owner ward envelope 用 frame tick 封裝 live bytes，client 持續判 NONCANONICAL（2026-10-25，ward）

- **現象**：B+C+C 對局中 client runtime 一加入就持續記錄 `owner ward envelope rejected: NONCANONICAL_OWNER_WARD`，一小時 6834 次，ward 投影一直被清掉。
- **根因**：server 每 step 重建 store 的 owner bytes（含 `elapsed_raw`），但 continuing dispatch 用觸發它的 team frame `replica_tick` 封裝；projection barrier 未到時同一個 frame tick 會重送，於是同一個 (tick, revision) 出現多種 bytes。recovery（`max(store tick, 上次 tick)`）和 rebase dispatch 也是 tick 與 bytes 來源不同。client 的「同 tick+revision 必同 bytes」規則是對的。
- **通用修正**：新增 `seal_stored_owner_ward_envelope`，envelope tick 一律取 bytes 所屬 store 的 bootstrap tick；omb 的 continuing、rebase、recovery 三條路徑都改用它的 tick。所有拒絕分支改成可區分的 `NONCANONICAL_OWNER_WARD_<規則>`。
- **避免**：簽章或排序用的 tick 必須和被封裝的資料來自同一份快照；不要拿「觸發送出的事件」的 tick 去封裝另一份資料。驗證函式每個拒絕分支都要有獨立原因碼，否則 log 無法分辨。

## E016 資料檔沙盒環境是空表，package 內的 `pairs` 無法呼叫（2026-10-25，E2）

- **現象**：`hero_author.lua validate` 載入 package 時報 `attempt to call a nil value (global 'pairs')`。
- **根因**：以 `loadfile(file, 't', {})` 載入會執行的資料建構函式，沒有提供任何純函式。
- **通用修正**：`cli.safe_env()` 只放純函式（pairs、ipairs、string、table、math…），不放 io／os／require／load。
- **避免**：沙盒要區分「純資料 table」與「會執行的 builder」；後者給白名單環境，並用真實 package 跑一次。

## E015 序列化把有洞的整數 key 表當成陣列，資料被丟掉（2026-10-25，E2）

- **現象**：`{[1]=q,[3]=e,[4]=r}` 被寫成 `{ q }`；build 後孫市只有 Q 欄被替換。
- **根因**：`is_array` 只檢查 key 不超過 `#t`；有洞時 `#t` 可能是 4，`ipairs` 卻在洞前停止。
- **通用修正**：`is_array` 改為 key 數量必須等於 `#t`，否則一律輸出 `[i] = v`；加有洞表的 round-trip 測試。catalog hash 未變。
- **避免**：不要用 `#t` 判斷「是不是序列」；生成器的 round-trip 測試要包含有洞與稀疏表。

## E014 D3 把被動技能映射成「不佔欄位」的 passive action（2026-10-25，D3／E2）

- **現象**：C17、A04 各有 5 個來源技能（含 D 被動），D3 卻判定不需要額外槽；設計 `build` 的欄位映射時才發現。
- **根因**：沿用 Dota「被動不佔熱鍵」的直覺，沒有先查本遊戲的 loadout：`moba_match.rs` 與 `omoba-content-model` 限制最多 4 個技能，被動也佔一格。
- **通用修正**：移除 `passive` action；來源 W/E/R/T → Q/W/E/R，D/F → extra1～3，且 extra 必須伴隨 `extra_action_slot` 機制（`requirements.lua` 與 `package_v2.lua` 都檢查）。D3 重新生成，roster 的 G-3 需求加上 H04。
- **避免**：建能力映射前先確認目標引擎的硬限制（欄位數、上限），不要沿用來源遊戲的規則。
## E010 用 shell heredoc／patch 字串寫 Lua 時反斜線被吃掉，且 CRLF 檔整段比對失敗（2026-10-25，E1）

- **現象**：替 `docs/character_pipeline/tools/validate_package.lua` 打補丁後，第 6 行的 `"[/\]"` 變成 `"[/\]"`，檔案無法載入；第一次補丁則因檔案是 CRLF 而找不到任何多行原字串。
- **根因**：補丁內容經 shell 傳遞時反斜線跳脫被處理了一層；多行比對字串是 LF，目標檔是 CRLF。
- **通用修正**：改 CRLF 檔前先 `file` 確認行尾；補丁時先把 `\r\n` 正規化成 `\n`，改完再還原。含反斜線的程式行用精確 Edit 修改。
- **避免**：補丁後立刻執行被改的檔案（這次是 v1 package 驗證），不要只看補丁腳本回報 ok。
## E035 UE 開發版的 debug exec 鍵（F1 線框、F2 unlit…）和遊戲熱鍵衝突（2026-10-25，C1）

- **現象**：autopilot 截圖裡整個世界變成青色、洋紅色的線框；CameraFocusSelf 綁在 F1，隊友卡片用 F2～F5。
- **根因**：引擎 `BaseInput.ini` 的 `[/Script/Engine.PlayerInput] +DebugExecBindings` 在非 shipping 版預設把 F1 綁成 `viewmode wireframe`、F2 unlit、F3 lit、F4 lit_detaillighting、F5 shadercomplexity、F9 截圖，還有 Ctrl+Tab 等。**真人玩家**在開發版按這些鍵也會切換檢視模式。
- **通用修正**：`omfue/Config/DefaultInput.ini` 加上 `[/Script/Engine.PlayerInput] !DebugExecBindings=ClearArray`，遊戲按鍵完全由 control profile 管理。
- **避免**：遊戲預設綁定要對照引擎的保留鍵（DebugExecBindings、console 鍵）；這類衝突要用截圖或真人試玩才看得到，單元測試抓不到。

## E034 自訂啟動參數含 `seconds=`，所有 renderer 在 120 秒後自己退出（2026-10-25，B3）

- **現象**：split8 的 p1 在 2 分鐘時正常關閉，log 顯示 `RequestExit(... FEngineLoop::Tick.Benchmarking)`。
- **根因**：引擎用 `FParse::Value(CommandLine, "SECONDS=", MaxTickTime)` 解析參數，而 `FParse::Value` 是不分大小寫的**子字串**比對。新增的 `-om-autopilot-screenshot-seconds=120` 剛好包含 `seconds=120`，引擎就把它當成「120 秒後結束」。
- **通用修正**：參數改名為 `-om-autopilot-shot-interval=`；`moba_role_launch.assert_engine_safe_args` 檢查所有 `-om-*` 參數，不得含 `seconds=`、`benchmark`、`fps=`、`deterministic`、`fixedtimestep` 這類引擎會用子字串解析的保留字。prepare 時強制檢查，也有測試。
- **避免**：替 UE 新增命令列參數時，先在引擎原始碼裡搜尋 `FParse::Value(...TEXT("<名稱尾段>")`；專案參數名稱不要以常見單字結尾。

## E028 三個 writer 同時直接編輯共用樹，一致性建置永遠等不到穩定（2026-10-25，協作）

- **現象**：連續 4 輪都是「sources changed during pass N」，最後以「another writer is active」失敗；其中一輪還編到半成品 `stage_ability_channel`。
- **根因**：ward（channel）、omfx-decouple（toggle orb）、source-index（H03）同時在共用樹上逐步修改 omoba-core 和 Lua。每一筆修改都會打斷一輪約 15 分鐘的建置；Lua 一改，codegen 輸出也跟著變。
- **通用修正**：(1) `consistent_build` 每一輪開始前，先等輸入靜止 3 分鐘（最多等 30 分鐘），輪數上限提高到 8；(2) 所有 writer 改用著陸流程：在 shadow 樹完成、編譯並測試通過後，一次套回共用樹，著陸之間至少隔 3 分鐘。
- **避免**：多個 writer 共用一個工作樹時，要讓變更以批次方式出現，不能一筆一筆持續流入；建置要找空檔開始，而不是硬撞。

## E027 一致性建置遇到別的 session 改到一半、暫時編不過的共用原始碼（2026-10-25，建置）

- **現象**：`consistent_build` 在 omoba-core 失敗：`ChannelPulse` 缺 `elapsed`、`Outcome::ScriptAbilityChannel` 的 match 沒有處理完。這是 ward session 正在做的 G-2 channel。
- **根因**：共用 checkout 裡，其他 writer 的半成品隨時可能暫時編不過；原本的腳本只處理「原始碼在建置期間變動」，沒處理「建置開始時就是壞的」。
- **通用修正**：編譯失敗時，如果輸入檔在最近 10 分鐘內被改過，就判定為 writer 正在編輯：等 2 分鐘後重試整組，最多 15 次。原始碼已經靜止卻仍失敗，才當成真的錯誤回報。測試涵蓋三種情況（立即失敗、等待後成功、持續損壞）。另外請其他 session 在自己的 shadow 副本完成整段修改後再一次套回共用樹。
- **避免**：共用工作樹的建置要能分辨「正在編輯」和「真的壞掉」，不能把別人的半成品當成自己的錯誤。

## E026 patch 腳本的 Lua 長字串被程式碼裡的 `]]` 提早結束（2026-10-25，J1）

- **現象**：patch 腳本解析失敗：`'}' expected near '='`。
- **根因**：要替換的程式碼裡有 `per_team[teams[id]]`，`]]` 結束了 `[[ ... ]]` 長字串。
- **通用修正**：patch 腳本的長字串一律使用 `[==[ ... ]==]`；套用前先 `loadfile` 檢查語法（patchlib 每個檔案只在所有修改都成功時才寫入，所以失敗不會留下半套修改）。
- **避免**：產生包含任意程式碼的 Lua 字串時，預設使用 level-2 長括號；同時參考 E010（不要用 shell heredoc 傳含特殊字元的內容）。

## E025 BUILD_FREEZE 忘了刪，平行 session 空等（2026-10-25，協作）

- **現象**：兩個 session 回報「等 freeze 解除才能編輯」，但主 session 早就不需要它了。
- **根因**：凍結檔是手動建立、手動刪除的約定，沒有任何機制保證會被刪掉；訊息又是排隊送達，等到被讀時早已過期。
- **通用修正**：廢除 `target/BUILD_FREEZE`；建置競爭一律交給 `consistent_build.lua` 的原始碼指紋處理（E018），它不需要別人配合。
- **避免**：跨 session 協作不要依賴「要記得清除」的標記；優先使用自己能檢查和修正的機制。

## E024 合成按鍵沒有真的按下 modifier，Ctrl+R 被當成 R；還誤判成遊戲 bug（2026-10-25，B2）

- **現象**：autopilot 的「升級 R」一直 `no_effect`，R 的冷卻卻開始跑。server log 有 pid=1 的 `CastAbility ability_index=3`（`lumen_mend` cast ok），沒有 `UpgradeAbility`。主 session 先把它當成「ACK 卻沒套用」的遊戲 bug，轉交給 ward session，之後才撤回。
- **根因**：合成 `FKeyEvent` 時只在 modifier 欄位帶了 Ctrl，沒有送出 LeftControl 的按下事件；`UPlayerInput` 依自己記錄的 key state 判斷 Ctrl。而且它是在下一個 tick 才評估綁定，同一幀內按下又放開的鍵，在評估時早已放開。
- **通用修正**：先送 modifier 的按下事件，再送主鍵；所有鍵按住 0.12 秒（好幾個 tick）才依相反順序放開。滑鼠點擊時，modifier 也一樣按住。autopilot 結束時強制放開所有鍵。
- **避免**：在指控遊戲端之前，先比對 server log「實際收到的是什麼指令」；合成輸入要模擬真人的完整按鍵時序，不能只填事件欄位。

## E023 autopilot 把 ACK 當成「權威已套用」，升級指令狂送 266 次（2026-10-25，B2）

- **現象**：`upgrade_r` 每次都收到 `accepted, status 0`，HUD 的等級卻一直是 0。server log 完全沒有 pid=1 的 `UpgradeAbility`（Bot 的升級正常），也就是指令被 ACK 卻沒有套用（已轉交 ward-canonical session 追根因，見 work-upgrade-ack.md）。
- **根因（autopilot 端）**：trace 欄位叫 `authority_result`，誤導成「權威已處理」；效果驗證每次重新選擇時都覆寫了待驗證狀態，期限永遠到不了，於是不斷重送。
- **通用修正**：欄位改名為 `ack_result`。技能類動作一律做效果驗證：升級要看到等級上升、施法要看到冷卻開始，記錄 `effect_applied`／`no_effect`。驗證進行中不再選新的技能動作；沒有效果就讓該 slot 退避 15 秒，並寫出 `no_effect` 事件。
- **避免**：ACK、submit sent、權威已套用是三件不同的事，命名和驗收都要分開；任何「成功」都要看狀態有沒有真的改變。

## E022 autopilot 要求「英雄在畫面上」才行動，英雄在地圖角落時永遠不動（2026-10-25，B1）

- **現象**：p6（team 2）的英雄停在泉水（4716, 4716），autopilot 狂按 `camera_focus_self` 279 次，整局沒有移動過。
- **根因**：鏡頭受地圖邊界限制，角落的英雄會落在畫面邊緣的排除區內；「英雄必須可見」這個前置條件永遠不成立。
- **通用修正**：移除英雄可見性的前置條件，只要求點擊點可見：沿著英雄到目標的線段，找最遠的可見點；整條線都看不到才重新對準鏡頭。
- **避免**：autopilot 的前置條件要和真人實際能做到的事一致（真人只需要點得到目標），不要多加人為限制。

## E021 合成鍵盤的 intent 當下就結算，但 InputComponent 綁定要到下一個 PlayerTick 才送出（2026-10-25，B1）

- **現象**：Stop、Hold、A、Shift+A 全部是 `input layer submitted nothing`；滑鼠的右鍵移動卻正常。
- **根因**：Slate 的滑鼠事件經 PreGame preprocessor 同步送出；鍵盤事件經 `UPlayerInput` 排隊，綁定在下一個 PlayerTick 才觸發，submission 晚一幀才到。「按鍵加確認」也一樣，armed 狀態是在後面的幀才出現。
- **通用修正**：intent 改成非同步：送出後保持開啟 0.75 秒或直到收到 submission；arm 之後等確認進入 armed 狀態的那一幀，才送出確認點擊。修正後 stop、hold、attack_move、attack_move_queued 都由權威端 accepted。
- **避免**：判定「輸入有沒有生效」時，要對照真實引擎的事件時序；不要用同步呼叫的回傳值去推論非同步的路徑。

## E020 新增英雄沒有頭像，UE 建置被頭像覆蓋檢查擋下（2026-10-25，A3）

- **現象**：`build_ue_moba.lua` 在「synchronize original HUD content names」失敗：`missing portrait for authoring hero: training_frost_mage`。這是 G-1 為了投射物 demo 新增的英雄。
- **根因**：覆蓋檢查要求每位 authoring hero 都在 `hud-assets-v1.json` 有一張專屬頭像；新英雄沒有美術就無法建置。這和計畫「缺美術有明確 fallback，缺玩法才阻擋」的契約相反，以後每位生成英雄都會卡在這裡。
- **通用修正**：`ue_hud_portrait_coverage.placeholder_entries` 替缺頭像的英雄補上共用的 `hero-placeholder-v1.png`（取自 F2 的 `hero_default_portrait.png`，hash 相同），並標 `placeholder=true`；只有明確標示的 placeholder 可以共用檔案，authored tile 仍然一張圖一個英雄。sync 會印出補了哪些英雄。測試 33 checks。
- **避免**：美術資產缺漏一律走明確的 fallback 並留紀錄，不要讓它變成建置錯誤；玩法缺漏才 fail closed。

## E019 client runtime 從來沒有送出 content hash，正式路徑的 renderer 一律被拒絕（2026-10-25，A3）

- **現象**：logger 裝上後看到 `om presentation runtime-ready rejected: runtime ... content= renderer ... content=9a30688c0fbfd351`。UE 永遠只拿到空 frame。
- **根因**：`RuntimeReadyPresentation.content_hash` 只有在帶 `--content-hash` 時才會填，但沒有任何 launcher 傳這個參數；bridge（88bc1d1 之後）把空值當成內容不符。過去的單機合成驗收沒有走到這條路。
- **通用修正**：`omoba-client-runtime/src/config.rs` 預設使用 binary 自己編譯進去的 `omoba_template_ids::CONTENT_CATALOG_DATA_HASH`；明確傳入的 `--content-hash` 只做相等斷言，不符就 fail closed。加了單元測試。
- **避免**：身分類欄位（content／protocol／catalog）一律由 binary 自己的編譯常數提供，不靠 CLI 字串；新增 fail-closed 檢查時，要同時確認正式 launcher 真的會送出該欄位。

## E018 共用 checkout 中，其他 session 的協定升版落在兩次 cargo 建置之間（2026-10-25，A3）

- **現象**：server join 一直回 `SHOP_CATALOG_REQUIRES_SECURE_PLAYER`；runtime 顯示 `protocol=16`，server 卻是 15。
- **根因**：先建 omobab、後建 client runtime；中間 G-1 session 把 `SELECTIVE_LOCKSTEP_PROTOCOL_VERSION` 改成 16。發給其他 session 的訊息要等它們這一輪結束才會送達，「請暫停編輯」不能即時生效。
- **通用修正**：新增 `scripts/consistent_build.lua`。每一輪建置 omobab、moba-config、client runtime 與 UE bridge／game，建置前後各取一次原始碼指紋（所有輸入檔的 size＋mtime）；指紋有變就整組重建，最多 4 輪，仍不穩定就報「另有 writer」。另外加了 `target/BUILD_FREEZE` 約定，給能讀到的 session 參考。
- **避免**：實機對局前一律用 `consistent_build.lua` 建置，不要分別手動 cargo build；看到協定或 join 錯誤，先比對各 binary 的版本常數。

## E013 bridge 被拒絕後零延遲重連（2026-10-25，A3）

- **現象**：presentation 連線被拒絕後，23 萬次以上的 `renderer IPC disconnected: unexpected end of file`，CPU 被白白燒掉。
- **根因**：driver 的 connect 失敗會 sleep 250ms，但連上後因契約不符而 `break` 的路徑沒有退避。
- **通用修正**：每次連線結束（不論原因）都退避 250ms 再重連（`omfue/bridge/src/driver.rs`）。
- **避免**：任何重連迴圈，所有出口都要有退避，不能只顧到 connect 失敗那條。

## E012 bridge DLL 沒有 logger，所有拒絕原因被吞掉（2026-10-25，A3）

- **現象**：UE 只收到 `tick=1 entities=0` 的空 frame，UE log 和 stderr 都看不到原因。
- **根因**：bridge 用 `log::warn!`／`error!`，卻從來沒有安裝 logger，全部變成 no-op；有些拒絕分支甚至連 log 都沒寫，直接 `break`。
- **通用修正**：新增 `omfue/bridge/src/bridge_log.rs`，在 `om_runtime_create` 安裝 stderr logger（`OM_BRIDGE_LOG` 控制等級，預設 warn）；launcher 本來就會把 renderer 的 stderr 存到 `unreal-pN.stderr.log`。所有靜默斷線分支補上帶參數值的原因。
- **避免**：跨 DLL／跨程序的元件，第一天就要確認 log 真的有輸出到某個 session 檔；每個 fail-closed 分支都要留下原因。

## E009 用 sed 改含 pattern 字元的程式行（2026-10-25，D1）

- **現象**：`sed` 替換 `gmatch("[^%z]+")` 時，把替換字串本身塞進結果，產生壞掉的 Lua。
- **根因**：sed 替換端的 `&`／`\0` 與 Lua pattern 字元衝突。
- **通用修正**：含 `& \ [ ] %` 的修改一律用精確字串 Edit。
- **避免**：sed 只做不含上述字元的簡單替換；改完立刻跑相關測試。

## E008 共用 checkout 的檔案被其他 session 改過（2026-10-25，D1）

- **現象**：對 `scripts/test_lua_tooling.lua` 的 Edit 找不到原字串。
- **根因**：另一個 session 同時在檔尾新增了未提交段落。
- **通用修正**：先 `git diff` 看他人變更，把自己的掛點加在其後，不改他人行。
- **避免**：碰共用檔前先看 diff；Edit 失敗時不要整檔覆寫繞過。

## E007 不受信任來源有孤立 token，整段 key/value 錯位（2026-10-25，D1）

- **現象**：`npc/B_Unified/B18_2.txt` 第 31 行孤立的 `"0"` 讓之後所有欄位 key/value 對調。
- **根因**：KV 只按 token 兩兩配對時，一個缺 key 會讓後續全部位移。
- **通用修正**：來源格式一行一對；key 後若換行才出現字串，記 `orphan_token`，key 視為缺值、不吃下一行（`scripts/hero_source/kv.lua`）。來源不改，只出診斷。
- **避免**：解析不受信任資料要能在錯誤點局部恢復並回報，不能沉默產生錯位資料。

## E006 序列化 Lua 保留字當 key（2026-10-25，D1）

- **現象**：生成的 `source-catalog.lua` 無法 `load`（`'(' expected near '='`）。
- **根因**：`function = "..."` 是保留字，不能當裸 key。
- **通用修正**：serializer 對保留字改用 `["function"]`，並有 round-trip 測試。
- **避免**：產生 Lua 資料檔後，在同一步 `load` 回來驗證。

## E005 生成產物依賴 `pairs()` 順序（2026-10-25，D1）

- **現象**：同一來源連跑兩次 `index`，`source-catalog.lua` 的 hash 不同。
- **根因**：分類器用 `pairs()` 走集合，evidence 順序隨 hash 種子變動。
- **通用修正**：凡是會進輸出的集合一律先排序；測試加「同輸入建兩次，序列化結果相同」。
- **避免**：寫生成器時以「重跑 hash 相同」作為完成條件。

## E004 Lua pattern 字元類別放多位元組字元（2026-10-25，D1）

- **現象**：從 `mod.rs` 首行抽中文名得到亂碼；`"降低[^。，]-移動速度"` 會誤配。
- **根因**：Lua pattern 以 byte 為單位，`[^（]`、`[^。，]` 排除的是這些 UTF-8 字元的個別 byte，連帶排除大量其他中文字。
- **通用修正**：多位元組分隔一律用 `string.find(s, sep, 1, true)`；分類改成先按標點切子句，再以純文字有序配對（`classify.clauses`／`match_clause`）。
- **避免**：含中文的 pattern 只能出現在字面序列，不得放進 `[...]`。

## E003 冷 shader cache 讓同機雙 UE 卡在第 0 幀超過 45 分鐘（2026-10-25，A3）

- **現象**：B＋C＋C 的兩個 `UnrealEditor -game` 雙方都完成 admit，但一直停在第 0 幀，log 全是 `LogShaderCompilers`。
- **根因**：DDC 是冷的；每個 UE 帶 `-corelimit=6`（host budget），全機只有 6 個 ShaderCompileWorker；兩個實例重複編譯同一批 shader。
- **修正方向**：launcher 在啟動多個 renderer 前，先確認 shader／DDC 已暖好；沒暖好時由單一實例、不加 corelimit 先完成編譯，再啟動其他 renderer。
- **避免**：每日迭代與效能量測前先暖 DDC；不要把冷快取的時間當成載入效能。

## E002 同機 B＋C＋C 的 base_content DLL profile 衝突（2026-10-25，A3）

- **現象**：`--server-only --no-build` 兩次都失敗，錯誤是 `staged base_content differs from current build`。三份 DLL 的 hash 其實一致。
- **根因**：`build_ue_moba.lua` 放上去的是 debug 版 `scripts/base_content.dll`，client 端 preflight 也檢查 debug；但 `moba_server_build.verify` 只拿 release 版比對。同一台機器上兩邊不可能同時通過。
- **通用修正**：`moba_server_build.verify` 接受「任一 profile 的最新建置」，stale copy 仍拒絕（`scripts/test_moba_server_build.lua` 有測試）。
- **避免**：看到 stage 不一致，先比對 hash 和「是拿哪個 profile 比」，不要先假設是平行 session 造成的。這次第一次判斷就錯了。

## E001 同機多個 `--connect` 綁同一個 presentation port（2026-10-25，A2）

- **現象**：讀程式碼確認，`moba_role_launch.prepare` 用 `options.port+ordinal`，ordinal 每次呼叫都從 1 開始。
- **通用修正**：改由 player ID 推導（`M.presentation_address`），超出 65535 就失敗；新增 `--host-simulations`／`--host-renderers`，讓同機的多次呼叫共用 CPU 預算。
- **避免**：跨 process 的資源（port、目錄、CPU）一律由全域唯一的 ID 推導，不用每次呼叫內的序號。
