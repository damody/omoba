# 指定單位技能射程提示錯誤與決策

- P01：盤點猜測reinforcements檔名與WorldBridge所在module，已依rg --files定位實際來源。WorldBridge在OmGenerated但為既有手寫橋接檔，codegen只生成內容類別；後續只用已定位路徑。
- D01：現有ground point不可當unit center；不掃描Actor、不維護全entity位置cache，直接讀目前PublishedDirectTargets，按owner／source與generation守門。
- 工具：沿既有OpenSpec手動spec-driven artifacts流程，CLI未安裝；依使用者授權自主實作。
- T01：實作後檢查繪製路徑發現，entity預覽尚未指向單位時，新source／target共同查詢會回false，連原本施法者射程圈也被一起隱藏。需要把「可顯示施法者圈」與「可計算單位目標距離」分開：有完整公開pair時用新source，尚無目標時保留既有owner source圈；不使用cursor替代單位。等待當前Rust／UE World／來源capture與watcher全部退休後再修繪製並重做UE／World／來源接受。
- T02：WorldV1只找到45個舊測試，新UnitCastRange缺席，完整案例守門正確拒絕；UE build成功不代表新增.cpp已納入。UE5.8 BuildConfiguration.cs說明UBT makefile會保留來源集合，當前CLI明確支援-NoUBTMakefiles。正式Lua ordinary／bootstrap與既有restart旗標資料統一加該選項，不刪cache、不修改引擎或全域設定；加入Lua規劃契約與restart九測試。V1的缺席report／log／receipt保留，不當成功接受。
- P02：追查建置入口時猜測ue_source_engine_bootstrap.lua，實際require為ue_engine_bootstrap；已讀build_ue_moba import定位。無來源誤改。
- T03：修復UBT來源發現後BuildV2真的編譯到新測試，C2248發現WorldUnitsToCm為protected，C2039發現Frame欄位為sequence而非seq。沿既有測試使用UPROPERTY反射設定fixture縮放，並使用實際CABI欄位；不改正式存取權限或協定。等待RustV2全批退出才修新CPP，已通過且未改動的Rust來源維持RustV2結果，重驗UE／World與最終來源。
- T04：WorldV2新增查詢測試失敗；WorldV3分離direct與反射後確認direct成功、實體半徑3500且完整參數契約正確，唯owner ProcessEvent回false。UE5.8 Actor.cpp:1488指出未初始化Actor的測試World會略過ProcessEvent。使用CoreUObject提供的FEditorScriptExecutionGuard限定於原生測試scope，不改正式可見性／owner／generation守門，不在產品加CallInEditor，也不繞過後端。V2／V3失敗原始report保留。
- P03：查UHT產物時先用omfue/Intermediate猜路徑；實際產物在plugin Intermediate。改用rg --files --no-ignore定位；不修改生成產物。
- P04：最終封存守門發現BuildV5對應的staged-v5.json尚未產生，封存在讀取必要metadata時停止，未產生成功index。補跑既有只讀staging helper，逐項核對當前七DLL／import library與BuildV5原始log的SHA；不改來源或二進位、不重跑已通過功能。StageV3亦保留作先前建置記錄。
