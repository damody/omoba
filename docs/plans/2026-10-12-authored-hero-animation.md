# 既有角色動畫補接

## 最新證據（尚待材質畫面修正）

refresh-v2 成功，fresh ordinary build-v6 至 v12 均真成功（最新17.29秒，refresh false）。import-v8 首次4 imported／4 saved，第二次4 verified／0 saved；原RecipeV1與新V2 SHA／mtime不變，全部 root frames scale 對原ref通過。contract-v2 八種原生錯誤計畫零寫入拒絕，所有package不變。產生器47測試通過。native-v4 17 Success／0 test warnings，20張原PNG存在；已直接看六張Saika，但外衣atlas錯誤，未視覺接受。E880記錄來源第三材質純色語意，正在補通用section材質指定與自有純色shader。父計畫仍12/44，未勾6.4。

由 Codex 直接實作，無子代理。原創 pooled combat VFX 已於 native-combat-vfx-accepted-v1 接受當前範圍。父計畫仍 12/44，6.4 未完成。

## 計畫

1. 讀取實際 binary FBX AnimationStack 時序，不推測施法／死亡動畫長度。
2. 在 Lua 模板接入既有 cast、cast2、die、run2，生成正式 C++ 路徑及 clip binding。回城使用現有持續姿勢。
3. 原生 commandlet 新增尚缺動畫，不覆寫或接管既有 RecipeV1。所有來源與目標 preflight、來源 SHA256／資產簽章／目標骨架／實際長度驗證，骨架 ref pose／parents 全量比對。
4. 兩次獨立 commandlet，第二次零匯入／零儲存，全部原 package 與新動畫 SHA／mtime 不變。
5. 受支援來源九張 D3D12 原始 PNG、CPU skin 幾何與真公開 cast／death／recall／sniper_walk 狀態驗證；未提供 hit／respawn／emote 的原始動畫不冒稱已完成，後續依原計畫補足。

## 已完成的前置驗證

fbx_animation_metadata_test.lua 真讀 11 來源綁定，LocalStart／LocalStop 與 Lua 的 duration／timeline offset 一致；四種損壞輸入拒絕。ue_asset_recipe_test.lua 14 工作項規劃通過，om_codegen 正式生成 19 files／21 Lua inputs。

原 MCP 匯入 saika-authored-import-v1 因插件不支援 check_asset_exists 失敗，原證據與 E867 保留；沒有新動畫寫入，原 Editor 已驗證清理。轉用原生 commandlet，不繞過簽章或所有權檢查。

## 尚未接受的原生結果與修正

saika-authored-build-v2／v3 ordinary nocache 成功，v1 EngineChange 與 refresh-v1 C++ 型別失敗保留。native-import-v1 兩次匯入機制通過（4 saved → 0 saved），八原生負例拒絕且原檔不變；這些不是動畫品質接受。native-v1 16 項中 15 Success／1 Fail，native-v2 單項仍 Fail；新增四片段根骨比例 2.54，既有五片段 1.0，sniper_move CPU 範圍超過 500cm。原圖與證據不改。首張 shader 暖機已修正，衣物部分 checker 尚須定位。

V2 方案：從既有目標 mesh 的 UFbxSkeletalMeshImportData 繼承真實座標與單位設定，完整精度簽章含 mesh 路徑；新資產只寫 AuthoredAnimationsV2，不覆寫錯誤 V1。產生器支援保留 recipe mesh／material／transform 的動畫 slot 覆蓋，正負例共用同一組驗證。原生測試新增 root scale 對 reference 的嚴格比對，保留既有幾何上限。接 fresh build-v4、兩次 import-v2、八原生拒絕與 16 項受影響測試／原始 PNG。E870／E871 記錄實際失敗，尚無當前角色接受。

build-v4 ordinary 真成功（38.14秒，refresh false），import-v2 在 preflight 拒絕且零寫入：既有 mesh 實際為 Interchange。依真實 provenance 補兩種 importer 分支；Interchange 複製原 generic pipeline，骨架固定、其他資產匯入全關閉，mesh package SHA 納入設定簽章。當前 translator 與原記錄的可編輯設定須一致，否則明確拒絕。動畫全部 root frames 在 save 前比對 ref scale。完整產生器47測試、實際FBX11綁定與4損壞輸入、recipe規劃通過。build-v5 真遭 FailedDueToEngineChange（12.14秒），maintained refresh-v2 執行中，之後 fresh ordinary 才是正式編譯證據。錯誤E872／E873保留。


2026-10-12 既有Saika動畫與來源材質當前範圍接受：`.ai-collab/authored-hero-accepted-v1/summary.json`。ordinary NoEngineChanges／NoUBA／nocache build-v2成功7.95秒；fresh17 native Success／零test warnings，48產生器測試。四個新增片段首輪4 saved、次輪4 verified／0 saved，逐幀root與原skeleton檢查；八原生拒絕及原package SHA／mtime不變。純色cloth首輪1 saved、次輪0 saved，七錯誤計畫零寫入拒絕，九張當輪original PNG直接逐圖檢視；11 combat PNG與已審原圖SHA完全相同，明示沿用。外衣已依來源第三純色材質修正，未改原RecipeV1；attack／critical共用來源，death圖為中間姿勢。E867–E882原失敗保留。缺少專用hit／respawn／emote來源及服裝／世界與完整驗收，父項12/44、6.4保持pending，無新權威對局／ACK。
