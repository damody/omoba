2026-10-12 compiled公開地圖renderer範圍接受：`.ai-collab/compiled-map-accepted-v1/summary.json`。Codex自行實作，無子代理。Rust export-v1真1test經正式公開ABI26發布兩compiled地圖roads／terrain／grid；ordinary NoEngineChanges／NoUBA／nocache build-v7成功14.60秒，fresh native-v6真1Success零warning，fresh Blueprint-v3真69且原packages SHA／mtime不變。17來源及export9來源凍結核對、stage再驗；14張1024×768當輪original PNG逐張直接檢視。真asset編譯／render fence、idle asset及真camera owner／projection檢查皆通過；全景與近照共用公開區域＋實際actor bounds的八角projection有界fit，基地屋頂完整。E980–E988及失敗原logs／v4、v5雖native pass但畫面拒絕證據保留。

兩圖目前輸出的公開geometry相同，22單位擺放及all-visible fog明示synthetic；未匯出layered tower實際配置，不當成真對局、fog變化、完整世界或OS DPI驗收。地圖平面外黑底及基地小量陰影鋸齒仍為畫面限制。neutral Rust/codegen producer明示只沿用未變檔；driver cfg(test)與WorldUnitVisualScaleTests optional hook已改，未冒稱所有neutral來源不變。父項12/44、quota10/10保持；接剩餘操作接線，完整驗收最後。

2026-10-12 Codex自行實作（無子代理）接續：HUD E975已按hud-overlay-accepted-v1獨立接受，ordinary build-v1、三native零warning、16paint組合、selection12/2/1及cleanup、69Blueprint packages不變；六新原圖逐張檢視。最大字級400需既有combat scroll，不冒稱完整409／DPI驗收。

兩張compiled地圖範圍進行中：Rust driver::compiled_map_fixture真正經moba_map_layout::resolve、apply_public_map_paths、RuntimeInner::publish_snapshot匯出ABI26 roads/terrain/fog-grid，compiled-map-export-v1真1Rust test成功，重複發布與empty清理驗過；all-visible cells／22單位擺放明示synthetic，無server／simulation／authority ACK。CPP OmCompiledMapSceneCapture由WorldBridgePresentationScale的optional參數啟用，ownerlessPreview沿正式MinimapModel explicit team1+WorldPresentationActor，不放寬production team guard。道路與terrain經真bridge，camp模型經公開camp id→generated registry查Creep，不用座標／順序另抄資料。

E980–E988均已記register。build-v1 C4458失敗後修；native-v1 camera manager缺漏、v2/3首球距離地圖太小皆真Fail保留。v4真pass但原圖缺編譯建築／T-pose，未接受；補asset compile/render fence與synthetic stand/idle後v5真pass／Blueprint-v2真69，但spawn圖主體裁切仍拒絕。已把全景八角actual-projection有界二分fit共用到每張近照，以公開區域地面＋真actor bounds算視點，compiled-map-validation-v7凍結17來源，ordinary compiled-map-build-v7執行中。下一步消費build terminal，fresh compiled-map-native-v6，逐張檢視14當輪original PNG，再fresh Blueprint-v3、source hashes／stage核對後才記accepted。run-native-subset.lua新增--compiled-map-fixture=絕對JSON路徑，要求WorldBridgePresentationScale；輸出compiled-maps/<map id>/{overview,spawn,lane-0..2,camp-0..1}.png，1024×768。

當前未accepted compiled-map範圍；父項12/44、authority quota10/10保持。接下來仍要補操作手冊列的只選英雄／autoattack偏好／個別英雄profile／可選WASD及既有跨DPI/PIE/真ACK限制；完整父項驗收最後。無新Git reset/pull/commit/push、無omfx工作。

# 完整公開地圖呈現：接續計畫

Codex 自行實作，不使用子代理。此文件是待執行計畫，不能當作完成或驗收證據。先完成公開單位選取的 full409 回歸與來源核對，再進入本範圍。保留 authority quota 10/10，不啟動新權威對局；完整父項驗收放最後。

1. 從實際 compiled `MOBA_MAPS` 經 `moba_map_layout::resolve`、bridge `apply_public_map_paths` 及 `FrameBuild` 投影取得兩張正式地圖的公開 routes／terrain／bounds，避免在 C++ 測試另抄 top/mid/bottom 座標。檢查首入、來源替換、空資料與重複發布；不匯出隱藏單位或 camp 私有狀態。
2. 建立可重現的完整地圖 renderer fixture，使用正式 WorldBridge、WorldPresentationActor、MapRouteActor、公開障礙與目前已接受的六英雄／塔基地／小兵／守衛視覺。合成單位要明示 synthetic，與真實公開地圖幾何分開標示；不稱為實際對局或 ACK。
3. 用正式 camera 的有效 component transform，依實際 bounds、viewport aspect 與深度限制做 fit。保存兩圖全景、出生點、三路、守衛區近照；原始 GPU 圖逐張檢查地面／道路／障礙、單位相對比例、隊伍色、陰影與裁切。畫面問題修正式共用呈現，不調假世界位置或只美化截圖。
4. 如發現 ground、terrain、routes 的來源／尺度／team 限制不一致，修共用生命週期與有界重建。維持 NoCollision／不影響 navigation，沿用合法素材，不能改權威地形去迎合視覺。相同幾何零重建、來源退休完整清理、超限／非法資料拒絕要有實際驗證。
5. 普通 NoEngineChanges／NoUBA／nocache 建置，受影響 Rust／native、當輪原圖與 69 既有 Blueprint 相容驗證；錯誤與拒絕的舊圖保留到 MD。兩張全景成功也不自動勾完跨 DPI／PIE／效能或所有 44 項。

已定位來源：`omoba-template-ids/src/lib.rs`、`omoba-template-ids/build.rs`、`scripts/lua_data/templates/moba_maps.lua`、`omoba-core/src/runtime/moba_map_layout.rs`、`omfue/bridge/src/driver.rs`、`omfue/bridge/src/projection.rs`、`omfue/bridge/src/lib.rs`、`OmRuntime/Private/OmWorldPresentationActor.cpp`、`OmEditor/Private/OmWorldUnitVisualScaleTests.cpp`。這些是查閱入口，尚未在本範圍修改。

## 已定位的具體實作路徑

橋接層已用 `driver::presentation_snapshot_to_frame_for_owner` 接收 `TeamPresentationSnapshot`，再由 `apply_public_map_paths` 根據已解析 compiled map 與 HUD 的 `lane_length_raw` 安裝 routes；`ProjectionState::project_snapshot` 產生 `OmMapRouteEntry`／point/string table；`RuntimeInner::publish_snapshot` 將 `PresentationExtras.public_map` 的 terrain 放入真正 frame-owned `OmTerrainRect`。可在 driver 的 test 子模組驗證並匯出這些正式 ABI frame 的幾何，避免為渲染驗收新增 production ABI 或改 schema。

公開地面覆蓋應沿既有 `FogGridGeometry::from_map`，而非另一份 C++ bounds 計算。fixture 的全可見 cells 必須明示 synthetic，只用來檢視公開幾何；霧的正式語意另以既有來源／hidden／retirement測試驗證。此方法不啟動 server、對局或固定步 simulation，也不取 camp 私有戰鬥計時。

匯出的 fixture JSON 必須有版本、map id、compiled map catalog hash、ABI version、明確 synthetic 標記、公開 routes／terrain／grid，並嚴格驗長度、finite值與容量。預設 Rust test 只驗資料與正式投影；只有固定 Lua 工作流提供明確 evidence 目錄時才保存新檔，採 create-new 防止覆寫舊證據。C++ optional capture test 必須先校驗這些資訊後才建立世界；受控單位的擺放應標示合成，不宣稱與真對局 spawn／塔層位置相同。

目前地圖 public routes 的 ABI string 為 `route_0` 等投影名稱，不是 authored `top` 字串；驗證應比對正式投影與原 waypoint 順序，不能因名稱不同硬改生成結果。ground 的 team1/2 條件與正式 `FogGridSnapshot::from_presentation` 一致，先前唯讀審查已確認，不應單獨放寬 renderer 來支援未協商 team。
