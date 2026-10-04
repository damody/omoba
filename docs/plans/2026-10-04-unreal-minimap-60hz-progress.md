# 原生安全小地圖：60Hz 部分完成

## 決策與資料鏈

1. `OmCommandBarWidget` 的 MAP placeholder 改為通用 `SOmMinimap`。單一 Slate primitive 畫路線／小方塊，不產生每個單位的 UObject/widget，不需要角色 C++ 或 Blueprint graph。
2. 只讀 Rust 安全 `presentation_snapshot` 的 live entities 和公開 routes，租約內複製值。沒有 actor 位置反查、完整世界重建或 hidden movement。控制 frame 保留完整 view；完整快照替換標記，所以 Hide／Forget／Reset 不殘留舊敵人；Stop／EndPlay 清空。
3. 正式 IPC 原先沒有 paths。權威從 SingleLaneConfig 發布 AllPlayers lane-length HUD metric；runtime 只解碼 team0 的公開 metric，Q10 2000～1000000 backend units，未知或越界為0。MobaHudPresentation additive tag8；bridge 經 owner／HUD 驗證後，轉為 (0,0) 到 (length,0) 公開單路端點。C ABI6 不變；未知舊runtime不硬編碼地圖。
4. Unreal 由公開路線 extent 建 square schematic、10% padding，等比例 letterbox、Y軸翻轉。無有效路線／NaN／溢位索引／退化範圍 fail closed；單位不影響邊界，超出示意邊界不顯示、不夾成邊緣假位置。
5. 第一版不画 frozen memory（不能冒充live enemy）、fog tiles，不提供map click／目標命中。顏色是自己／其他有owner玩家／中立，不代表五人隊伍辨色。這不是正式可行走map bounds；現有測試move到(-400,-400)會超出X最小-240而不顯示該marker，已知限制，不擴張邊界偷讀單位。

## 驗證

- Rust1.95.0：core322、base_content73（含完整filtered死亡／重生／終局96.70s）、runtime49+3、bridge47 passed。runtime3個capture和bridge1個capture是opt-in ignored，本段未執行。新增公開地圖資料／錯team／錯owner／非法長度測試。
- 完整建置96915 exit0，content identity de9c7fcfc98d6479，staged bridge SHA `e9085c7115541e12f7bf1a64cdd0d5ebf3a8d1a8f40f274c84f80a44b59e657f`；最後stage-only仍一致。
- Editor57864同session兩輪各10/10；新 `Om.Runtime.NativeMinimap` 覆蓋轉換、generation、Y inversion、letterbox、out-of-bounds、Hide、重複snapshot、控制frame、memory非live、NaN、route range溢位、reset、Stop、HitTestInvisible。synthetic EditorPreview有no-context DestroyActor warnings，沒有error，不宣稱零warning。
- 串行PIE35587 exit0／pie-after=false。截圖 `omfue/Saved/McpAutomation/pie-smoke.png` 已檢視，路線／標記呈現且bar/shop不溢位；它是legacy fixture，不是正式雙UE或效能證明，截圖HUD顯示3FPS。
- 第一run `interactive-ue-1791057171`：success／cleanup=true、雙native map／HUD／movement／ACK；保存hash56／57 PASS至3480，零FAIL。checkpoint早於最後safe3551，所以加上post-input hash gate再驗證。
- 最終run `interactive-ue-1791057377`：正式server＋2 filtered runtime＋2 UE -game，release、network60Hz、presentation30Hz、UE t.MaxFPS60。launcher70848 exit0、success／cleanup=true；雙native_minimap_observed=true、owned marker、固定bounds=(-240,-1440,2640,1440)、one route。雙HUD／economy／own-only rendered movement／Consumed通過；safe3500／3506，Consumed1045／1825，UE觀察21／15 frames。三方pre/post hash60／59 PASS至3600，後於移動觀察，零FAIL。
- 最終五PID82452／37072／73216／45844／85308已CIM查無；前一run五PID亦查無。Editor已正常Save＋QUIT並退出。沒有任意清理其他專案程序。
- protobuf受版控fallback與vendored-protoc生成差異只有tag8三行，已狹義同步，不覆寫既有dirty內容。
- fallback同步後再次release server build8656 exit0（38.39s、明確走no-protoc fallback），針對變更檔的diff --check通過；既有LF/CRLF警告未大範圍改寫。最終CIM再次確認Editor與五owned遊戲程序皆查無。
- hash數字只計週期性checkpoint：最終原檔team1 PASS60／UNVERIFIED7257、team2 PASS59／UNVERIFIED7258、FAIL0。非checkpoint列沒有expected而UNVERIFIED，不是每tick都已被authority hash證實；不把它們算成PASS。

## 重現

使用固定Lua執行 `scripts/build_ue_moba.lua --full` → `scripts/ue_native_visual_smoke.lua` 完成兩輪 → `scripts/ue_pie_smoke.lua`，順序不可重疊。正常退出Editor後，設定 OMOBA_UE_MINIMAP_SMOKE=1、OMOBA_UE_SMOKE_SECONDS=90、OMOBA_UE_STEP_FPS=60、OMOBA_RELEASE=1、OMOBA_SKIP_UE_BUILD=1，再執行 `scripts/run_2player_ue.lua --single-lane`。正式後端不可跳過新Rust建置；最終run僅因前一run已重建同版本才設OMOBA_SKIP_BUILD=1。

證據：`target/interactive-runs/interactive-ue-1791057377/unreal-ipc-smoke-report.json`、logs/ue-p1/2.stdout.log、server/three-way-checkpoints.jsonl；Editor報告 `omfue/Saved/McpAutomation/NativeVisual/report.json` 與 pie-smoke-report.json。暫存產物不提交。

## 剩餘

按OpenSpec實作流程，此為6.2的部分進展，總數仍17/30，不勾選整項。完整選角→結算、計分板、真實map bounds／fog／memory／互動小地圖、五人隊伍與三路、LAN／renderer重連和完整效能封關仍待後續。60Hz是network成功，不是所有frame time已達60FPS。

失敗與防重犯：`unreal-moba-error-register.md` E098；首次bridge owner-test失敗保留，修正後重跑通過，沒有跳過斷言。
