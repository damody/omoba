# 編譯地圖共用契約進度

## 計畫與決策

- [x] 收斂compiled map到BlockedRegions轉換，權威初始化與KCP初始bootstrap共用，不依three_lane_training名稱分支。
- [x] runtime bootstrap與正式RuntimeReady共用validate_metadata：present compiled ID／hash必須伴隨唯一schema1地形，canonical wire bytes完全相同；無compiled identity保留legacy契約。
- [x] 解碼任何公開地形前，以剩餘bytes檢查region／point count，禁止依不可信count直接reserve巨大容量。
- [x] 新測試涵蓋非shipped map名稱、正负座標、兩個矩形、空地形、缺失／重复／錯schema／修改內容、u32::MAX數量與bootstrap載入script前拒絕。
- [x] 完整回歸與最新build／stage。
- [x] 正式三路60Hz回歸、保存最後結果。

碰撞幾何與視野遮蔽物不是同一契約，這次不改FOG_2TEAM_DEMO或把terrain送進vision polygon。靜態地圖可由Lua新增／調整並重建peers，runtime不接受與compiled catalog不同的動態地形；未來動態地形需要獨立版本化contract，不能偷偷修改BlockedRegions。validator只在bootstrap／ready跑，不增加每entity每tick轉換。

本輪是後续地形／營地通用呈現的資料一致性前提，不是美術或建築完整驗收，不勾選整項5.4／6.1／6.2。既有dirty worktree保留，不commit／push、不清理target或他人程序。

## 錯誤紀錄

詳見unreal-moba-error-register.md E132。使用本輪重新建置與執行結果，不沿用上一輪DLL SHA或把上輪雙UE證據視為本輪結果。

## 已完成回歸

- core343 passed（最後format後再次343）；base_content108 passed，227.93秒。
- runtime62 passed／8 opt-in ignored＋integration3；新增錯配bootstrap拒絕與既有真實TCP晚連線／重連保留ready通過。
- server156 passed／1 benchmark ignored；bridge54 passed／1 opt-in ignored＋integration2／1 opt-in ignored。
- codegen --check：11 files／16 Lua inputs，content_hash=3b296ff5bdbcd7d9；OpenSpec strict與scoped diff check通過。
- 最後OmGameEditor增量Result:Succeeded（2.20秒），格式整理後重新build／stage，不使用首輪舊SHA。
- 獨立stage bridge SHA-256 `2d2e1906061930d72f9f5067fcd3039949bba9bfae4f44cbe3ea1e06ac96d5c5`；script debug target／scripts根目錄／UE Binaries三份SHA-256一致：`719264841d5193532b18d202c6c609e262c5e61d87ed6f5cef557882f7f4b60f`。

## 本輪真實雙 Unreal 60Hz

`target/interactive-runs/interactive-ue-1791116233/unreal-ipc-smoke-report.json`：success=true、cleanup_verified=true、three_lane／60Hz／debug／D3D11，port58501。報告SHA-256 `f452379d3ae4bb4ecaaadf0811a2f89ed715ed38d44529e9fc087edc1b8f46d8`。

- 雙UE route0／1／2點數4／2／4，兩native minimap routes=3／owner=1；own-only HUD與economy皆觀測到。
- UE已呈現位移team1(120,0)→(105,0)、team2(2280,0)→(2275,0)；raw replica也已移動。不同觀測時間不混成同tick數值，不聲稱抵達目标或像素同tick精確。
- 消費快照sequence887／1423，safe tick3171／3168。
- 保存server/three-way-checkpoints.jsonl獨立讀回：team1 41 PASS rows／27 unique ticks、team2 43／27，均至3240，零FAIL且所有PASS pre/post parity=true。UNVERIFIED紀錄保留，不冒充全部網路tick有驗證。
- Win32_Process另查報告五PID105848／87096／6512／74732／98016皆不存在；正常退出後stage再獨立驗證仍為2d2e19…。
- 沒有新增每map／英雄C++、Blueprint graph或關掉地形碰撞。沒有本輪MCP／PIE／PNG／LAN／稳定60FPS／營地地形美術完整驗收，20/30維持，不勾選完整5.4／6.1／6.2。
