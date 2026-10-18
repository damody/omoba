# 原生角色呈現：資產、組裝與驗證

## 執行計畫

1. 保留來源 FBX、貼圖、CC0 授權及 archive SHA；正式來源使用可移動的 clone-relative manifest。
2. 匯入獨立 skeleton，啟用 FBX scene unit conversion；不能把相同骨骼名稱視為相同 rest pose。
3. 原生 IK characterization、chain mapping、target pose alignment，轉換到各自目標 skeleton；以獨立 Editor 重新載入並確認 zeroSave。
4. 匯入 color／linear normal／mask 貼圖，建立材質接線、角色串流與 mipmap 政策，檢查實際 shader resource。
5. 服裝缺少的頭部以權重子樹擷取到 Head bone 局部座標；驗證輸出幾何、UV、法線與 material slots，來源 mesh 不改寫。
6. 使用 typed `ue.native_visual` 資產路徑、材質槽、動畫與骨骼附掛設定。舊 RecipeV1 與既有 Blueprint override 保持可用。
7. 實際角色載入、動畫、骨骼附掛、變形、可見性、退役與重生檢查完成後，再進入場景／光照／固定視點驗收。

## 已有證據

- 來源匯入：`.ai-collab/character-source-import-accepted-v1/summary.json`。
- 重定向：`.ai-collab/character-retarget-accepted-v1/summary.json`；六角色、258 clips、276 packages、37 契約案例、六次真正 native preflight 拒絕。
- 材質目前：`character-material-import-v4` 首次成功保存28 textures與九個材質；第二次獨立 Editor reload／zeroSave已通過。完整 scope 尚待修正後 native 拒絕案例及來源綁定。
- typed codegen：46個 library tests 全過，包含兩個新增顯式資產設定測試。Runtime／rigid extractor 已通過 `character-presentation-build-v2` ordinary NoEngineChanges／NoUBA／nocache。

## 資產與限制

角色與服裝來自 Quaternius 官方免費 Standard 包，是第三方 CC0 資產。28張貼圖 bytes 與兩份 included licenses 位於 `omfue/SourceArt/Characters/MaterialsV1`，來源 manifest 保留 archive member、archive SHA 與每張貼圖 SHA。新增角色不共用錯誤 rest-pose skeleton；附掛不引入每 frame 資產載入、掃描或 UI 工作。

資料與 shader 驗證不等於完成畫面。父計畫仍12/44；6.2／6.4不能僅憑匯入、重定向或編譯通過勾完。仍須頭部實際輸出、角色映射、實際動作與固定視點變形檢查。沒有新權威對局或 ACK；歷史 omfx 不在維護／建置／驗收範圍。

## 已記錄失敗

### 2026-10-12 實際播放比例修正

`character-assembly-native-v1` 實際播放發現六角色身體只有1.327–1.841公分，附掛頭部也縮小；因此 V1 retarget 的資料檢查不可當作播放變形成功。E844 保留原失敗與 PNG。新的 RetargetV2 依目標 reference hierarchy scale 從公分 component pose 重建 local tracks，包含 parent scale 下的 child translation 換算；保留 V1 原 packages，不使用 actor scale 常數補救。

普通 `character-reference-scale-build-v1` 已 exit0（NoEngineChanges／NoUBA／nocache）。`character-reference-scale-male-v1` 真正轉換46 packages／43 clips，第二個獨立 Editor zeroSave；全六角色轉換與真正 CPU skin／場景 PNG 驗證接續中，尚未接受組裝範圍。

錯誤 register 保留 E827–E835：EngineChange gate、UE5.8 SharedString、實際動畫 warnings、Lua path API、RHI module、commandlet rendering、非同步 texture GammaSpace assertion，以及未等本次 build 完成即啟動 Editor 的 DLL sharing violation。每次失敗使用獨立 evidence run，不覆寫為成功。


2026-10-11 材質當前範圍接受：`.ai-collab/character-material-accepted-v1/summary.json`。28第三方CC0來源貼圖／兩份included licenses、九材質、37 packages；真正shader resource／graph／normal與ORM channel／color space／角色streaming與mipmap／2048政策已驗。fresh independent Editor重載zeroSave，30 Lua契約與修正後六次native preflight拒絕全過，37 package SHA／mtime不變。ordinary NoEngineChanges／NoUBA／nocache build-v3成功；E830–E835全部原失敗保留。仍不是實際角色組裝／變形／世界畫面接受，父項12/44保持，無新權威對局或ACK。


2026-10-12 角色組裝基礎範圍接受：`.ai-collab/character-assembly-accepted-v1/summary.json`。六個實際生成hero／六獨立skeleton、258 RetargetV4 clips／276 packages、兩個公分頭部附掛、37材質packages；實際idle／move／attack CPU skin比例、12設定clip解析、shader map、貼圖residency與12角色＋4來源診斷original PNG已驗並逐張審查。ordinary NoEngineChanges／NoUBA／nocache build成功，fresh兩native Success／0測試warnings；40＋26 Lua契約、八次retarget與六次rigid native preflight拒絕，333／59 packages SHA與mtime不變。V1–V3比例與程序失敗、E841–E850保留；正式引用V4，來源raw動畫root校正由明確reference clip推導。仍須兩基底服裝／武器、隊伍色／輪廓、cast／hit／death／respawn／recall生命期與世界畫面，6.2／6.4及父項12/44不勾完，無新權威對局／ACK。
