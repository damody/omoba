# Unreal 計分板畫面 60Hz 驗收

## 本輪計畫

1. 沿用雙隊正式 60Hz KCP／runtime／Unreal 路徑。
2. 原生 Slate 面板實際 viewport layout 就緒後保存兩隊 PNG 與顯示資料。
3. 對照 authority wire、IPC 與 UI 相同 tick；核對 UI 後120 ticks三方hash與 owned程序清理。

## 決策與實作

- 新增 `OMOBA_UE_SCOREBOARD_SMOKE=1`（須 bounded single lane）與非Shipping `-om-scoreboard-smoke`；正常啟動不截圖或增加測試輸入。
- `SetScoreboardState` 只節流文字更新，layout檢查獨立重試；Playing／tick至少120、在viewport、真實panel可見且cached geometry非零才能記錄 `OM_SCOREBOARD_UI` 與截圖。
- launcher 要求兩隊身分、完整固定1v1名單、PNG存在、每隊至少6個PASS rows與UI後120 ticks checkpoint。此fixture只驗1v1，不把player ID等於team的測試條件帶進正式runtime或UI。
- `real_unreal_scoreboard_capture` 逐tick讀原始TeamTickFrame建立公開board，要求兩隊同tick一致，逐筆比較IPC並精確核對UI記錄的所有五個欄位。snapshot tick明確減1，不使用最近樣本容差。
- 初始零KDA畫面僅證明正式多人資料／layout接通，不宣稱擊殺更新、十人完整實戰或真人按鍵。

## 已通過的基礎驗證

- Unreal build-only 編譯成功，ABI9產物仍一致；Lua觀測器新增9項正反向及既有31項情境通過，launcher語法通過。
- runtime 59 lib tests通過，6個外部capture tests預設忽略；本輪新增verifier需對完成後的正式證據另行執行。
- scoped diff whitespace檢查通過。問題與防重犯規則記在E114。

## 正式驗收

`interactive-ue-1791079800` 正式release 60Hz成功，report success／cleanup_verified皆true。兩队UI tick1162／1632，三方hash各54 PASS rows至3240，UI後超過120 ticks；server＋雙runtime＋雙Unreal五程序74940／47800／34636／81564／36436均獨立inspect確認退出。

- `real_unreal_scoreboard_capture` 另行執行通過：team1 3402／team2 3384（共6786）筆IPC快照逐tick對authority公開資料完整一致，UI記錄tick與IPC五欄位精確一致。
- 兩張 `scoreboard-ui/team-1.png`、`team-2.png` 已以view_image實際檢視：1280×720兩隊名單、標題及0／0／0可讀未裁切。相機與美術仍為既有fixture，不算三路正式畫面。
- 截圖時shader準備中，FPS21／23；Observed60／60Hz僅代表對局rate，不能宣稱UE已穩定60FPS。截圖是request後下一次viewport draw，不提供GPU fence或像素與同tick原子快照契約；值未變的初始名單可對照，但不能拿它替代動態擊殺畫面的逐frame驗證。
- 新增證據不需要重新產生角色C++或Blueprint graph；只修改共用Widget和固定Lua驗收工具。

## 工具修復

- 收尾full build的 `compile-1791080124` gate失敗：nested Lua host同秒／同random暫存request名稱碰撞，安全拒絕overwrite，不是Blueprint編譯錯誤。
- host改用LuaFileSystem的原子mkdir保留獨立owned目錄，最多1000次retry；讀回後只刪自身request／response與空目錄，不碰其他程序或舊檔。強制父子相同time/random回歸與Lua module tests通過。
- nested test順便揭露relative script root推導錯誤，改path.repo_root；兩個失敗及patch context拒絕均記E114。
- 同一owned Editor的 `compile-1791080223/report.json` 11BP gate成功，`NativeVisual/report.json`兩輪各15/15通過。原失敗report保留，不宣稱首次full build已通過。
- 串行PIE成功；MCP保存dirty assets後正常關閉owned Editor93264，未強制關閉其他專案。最後build-only通過，stage SHA-256 `39921be7cc7da92074f3ee5e6b0e16c6025d64b0511eaccec6262161bc47ea60` 與bridge產物一致；codegen 11檔／15Lua輸入 content hash仍 `de9c7fcfc98d6479`。OpenSpec strict validation與scoped diff whitespace檢查通過。

OpenSpec全項仍19／30，6.2選角到結算與十人畫面未完成。
