# 角色來源匯入與後續呈現

2026-10-11，自行實作，未使用子代理。父計畫目前12/44；此文件的來源匯入完成不代表6.2或6.4完成。

`scripts/import_character_sources.lua` 使用固定Lua與受監督的Unreal commandlet，正式來源在`omfue/SourceArt/Characters/QuaterniusV1`。包含兩個全身、四套衣裝、一套43動畫庫，以及三份原包CC0授權；來源副本與原包解出的FBX SHA一致，durable manifest使用clone-relative路徑。每個來源獨立匯入自己的Skeleton，不按骨頭名稱強行共用。

目前接受證據是`.ai-collab/character-source-import-accepted-v1/summary.json`：

- ordinary NoEngineChanges／NoUBA／nocache build-v3成功9.64秒。
- actual import-v2：七個來源各65骨頭、43動畫，共57套件；第二個獨立Editor驗證七來源、零匯入／零保存，全部套件SHA與mtime不變。
- 標準FBX scene unit conversion已啟用，身體高約178–182公分，衣裝高約153–187公分。初次錯誤單位V1保留，只使用V2接續工作。
- 54個自足Lua契約測試通過。四個真正native gate將錯誤放在最後一筆，前面合法probe仍完全零寫入，57個既有套件不變，probe來源已退休。
- 首次匯入234筆warning保留，包括bind-pose重建與缺少smoothing groups。第二次0warning不能抹去來源問題；法線、材質與變形仍須實際畫面驗證。

來源匯入不包含完成的角色材質、衣裝組裝、IK retarget、template映射或實景接受。正式`source-registry.json`由成功的雙Editor驗證輸出，使用資產路徑與套件SHA，供後續工具核對；不依賴歷史Saved測試報告推測資產。

接續順序：

1. 透過Unreal原生IK Rig／Retargeter controller建立source與target chains、FBIK及target pose對齊，從獨立動畫骨架產生每個target自己的動畫。
2. 檢查所有輸出Skeleton、動畫時間／keys、finite transforms與有界尺寸，再以第二個Editor零寫入重驗。
3. 做有來源的材質與頭部／衣裝組裝，維護可覆寫且經驗證的template呈現欄位，確認不同角色可辨識。
4. 用實際viewport驗idle／move／attack／cast／hit／death／respawn／recall，再接完整場景驗收。

相關問題與修正保留於`unreal-moba-error-register.md`的E823–E826。


2026-10-11 Codex獨立實作：原生角色動畫重定向範圍接受，`.ai-collab/character-retarget-accepted-v1/summary.json`。六角色各21 chains／43 clips／131885 finite transforms／39 moving clips；276 packages，兩次fresh native重載均zeroSave。37 Lua契約及六次真正native preflight拒絕通過，333來源與輸出package SHA／mtime不變；ordinary NoEngineChanges／NoUBA／nocache build-v3成功。E827–E829原失敗與實際warnings保留。不代表角色外觀／服裝頭部／材質／實景變形完成，6.2／6.4及父項12/44保持pending；無新權威對局／ACK，繼續材質與角色組裝。
