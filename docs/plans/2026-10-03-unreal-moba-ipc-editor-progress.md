# Unreal MOBA 接軌與 Editor 驗收（2026-10-03）

## 已執行計畫與決定

1. 持續 `build-unreal-rust-moba-framework`，先補 4.1 的輸入與 IPC 缺口，再檢查 4.4 模擬隔離，接續 Editor 驗收。工作樹既有變更保留，不提交或推送。
2. 發現 `select!` 的 outgoing 分支會取消已消耗 TCP 前綴的 read future。改為持續的接收／送出 loop，僅在整條連線結束時取消讀取。實際 loopback 測試刻意拆開輸入封包，中間送出重要結果與新快照，確認輸入仍完整抵達。
3. 握手成功後取得單一 renderer lease；第二連線被拒絕且不改變原連線狀態，原連線離開後可重連並取得最新畫面。這避免兩個 renderer 競爭重要結果佇列。
4. client runtime 驗證四個技能槽、六個物品槽及 Fixed32 座標範圍，溢位明確回報 `INVALID_POSITION`，不再截斷。測試使用實際 `TeamViewProjector` 與 `SelectiveReplicaRuntime` fixture，覆蓋錯玩家、過期 view、未揭露自己的英雄及目標、secure reference 與合法邊界。
5. IPC 位址優先於單機／LAN 模式。renderer 不要求本地 gameplay DLL/story，也不建立 simulation thread；實際 socket 測試兩種模式的握手、filtered frame、移動及閒置停止。舊 TD 單機 smoke 仍通過。
6. Unreal bridge 先前在本地排入操作時就發布已套用 ACK，並忽略外部 runtime 結果。IPC 模式現在只發布 `APPLIED_TO_PRESENTATION` 或拒絕結果；`FORWARDED`／`SERVER_ACCEPTED` 維持 pending。writer 保留原 bridge input ID 作 renderer request ID，runtime 將伺服器的 secure request ID 對回 renderer ID；拒絕清除 pending，接受保留至實際套用。重要結果遇到 frame ring 滿時保留重試，且不覆蓋 HUD 的 round/lives。C ABI 拒絕暫映射 `InvalidArgument`，詳細錯誤字串仍待呈現。
7. 插件已換為 UE 5.8 預編譯版 BpGeneratorUltimate 2.0.6；舊 source patch 因 `ImportTools.cpp` 不存在而阻擋建置。建置入口辨識 descriptor、precompiled rules 與 DLL 後允許建置，並明確要求另做動畫匯入驗證；未知或不完整套件仍停止。完整 DLL、codegen、bridge、OmGame 建置通過，Editor 及 HTTP MCP 健康檢查通過。
8. 新增 `scripts/ue_mcp.lua`，處理本機 bearer token、動態工具探索、多行 SSE 與失敗結果，保存工具回應供驗收；token 不放進 argv、stdout 或報告。Lua JSON 模組加上明確空物件標記，以區別 `{}` 與 `[]`，既有陣列行為保留。
9. 新增 `scripts/ue_pie_smoke.lua`，讀取 PIE、生成 native `OmHeroTrainingLuminary`、斷言位置、收集日誌與截圖並停止自啟動 PIE。已重跑通過，斷言回報 `PASS`；3.3 完成，整體進度 13/30。詳見 `openspec/changes/build-unreal-rust-moba-framework/evidence/editor-mcp-smoke-2026-10-03.md`。

## 動畫匯入問題與處置

本輪也補上 4.3 的第一段：renderer 以 `(render_id, disclosure_epoch)` 追蹤可見單位，Hide／Forget 只移除相同世代，ResetView／空快照／斷線則清理先前單位；移除與輸入結果持續合併到後續 frame，直到 Unreal 取得包含它們的 lease，避免新快照覆蓋尚未讀取的重要訊息。IPC 送出端也跳過已被最新 snapshot 涵蓋的舊 lifecycle／snapshot，保留待處理輸入結果。測試覆蓋舊世代拒絕、空快照清理、重要回報重試與重連狀態不倒退。cue 去重及真正雙隊 UE 畫面驗收仍未完成，4.3 保持待處理。

- `import_skeletal_mesh` 在 `/Game/OmGenerated/Heroes/saika_magoichi` 成功建立模型，查詢得到 40 個 bones、2 個材質槽。
- `import_animation` 回傳 `asset_type:AnimSequence`，但 `class:SkeletalMesh`；`get_asset_summary` 亦確認產生了 mesh。錯誤測試資產位於 `/Game/OmGenerated/Heroes/saika_magoichi/Animations/b01_ani_stand`，未綁到正式英雄動畫，保留供診斷。
- 已啟用插件內建 Import 擴充，查詢其 Interchange 工具。公開 `import_asset_file` 只有來源、目標、名稱與覆寫參數，尚無指定既有 Skeleton 的參數；目前不足以保證正確的動畫配方。
- 第一版 `ue_mcp.lua` 對錯誤主要資產做 class 驗證並拒絕，避免誤認成功。接著發現匯入器另外產生未保存的 `_Anim`；重啟後 Restore Packages 提示僅列出本輪測試的 `b01_ani_stand_Anim`，選擇 Skip Restore 並保留 Saved autosave，以重新從來源驗證。最終入口改為選取 `_Anim`，透過 `get_anim_sequence_info` 驗證真實 AnimSequence、正長度與 frame 數，保存動畫及額外產物，再以 `get_asset_dependencies` 驗證指定 Skeleton 的完整資產路徑。
- 已驗證 idle（81 frames、2.67 秒）與 move（24 frames、0.77 秒）。move 使用最終 Lua 入口成功匯入、保存且依賴 `/Game/OmGenerated/Heroes/saika_magoichi/saika_magoichi_Skeleton`。報告同時列出主要 mesh／PhysicsAsset，未把額外產物假裝不存在。預編譯插件的回傳問題已由 Lua 端安全處理；完整配方批次套用、材質綁定與來源 hash 重跑驗收仍未完成，3.1 保持待處理。

## 驗證

- client runtime：28 項 library、2 項 main 測試通過。
- Unreal bridge：28 項單元測試、2 項 TD 單機 smoke 通過；需執行中的 KCP 後端測試維持 ignored。
- 固定 Lua module tests 通過；OpenSpec strict validation 通過。
- 完整 Unreal 建置及 Editor native hero PIE smoke 通過。這次 PIE 仍使用既有 TD 單機測試場景，不能宣稱完成 MOBA 對局或效能驗收。

## 後續順序

### 最後重建的共用引擎阻塞

本輪前段完整建置與 PIE 已通過；後段為 staging 最終 bridge 再跑完整建置時，共用引擎的 `UnrealEditor-NetCore.dll`／`UnrealEditor-Engine.dll` 被另一個 `C:\portable\OpenKoikatsu\OpenKoikatsu.uproject` 的 `UnrealEditor-Cmd.exe` 鎖住。UBA 重試後 linker 回報 `LNK1104`；再次啟動本專案 Editor 回報 `0xc0000135`。這次建置因此不能記為通過。另一個專案的程序未停止，未還原或覆寫其引擎來源變更；待共用引擎可重新連結後，需要重跑完整建置及 PIE。最新 Rust 測試仍通過，bridge 可獨立編譯及 staging，不代表 Editor 此刻可用。

### 剩餘工作

上述共用引擎阻塞已在本輪排除：另一個專案的命令列程序自行結束後，再跑 `scripts/build_ue_moba.lua --full --ue-root D:\UE5.8`，缺少的 DLL 重新連結成功，完整建置、Editor 啟動與 MCP 健康檢查皆通過。沒有停止另一個專案。最終 bridge 與 plugin staging 的 SHA-256 相同：`6ba6d45bec59feadfc65c730b64ff3405dea898f20784bc125c131cb5d53369e`。最後 Editor PID 為 22516。

解除上述僅含測試動畫的還原提示後，最終 `ue_pie_smoke.lua` 再次完整通過；Editor 保持開啟、PIE 已停止，報告 `success:true`。後續新增的動畫驗證入口也已對 move 動畫實測通過。

1. 完成 4.1 的正式 MOBA 輸入／安全投影及 4.2 雙隊端到端測試，補齊 C ABI 的結果碼呈現。
2. 完成 4.3 的 Hide／Forget／ResetView、cue 去重及畫面重連狀態清理。
3. 完成正式啟動路徑的 4.4 驗收，再整合 5.x 對局生命週期、兵線、建築、經濟與 Bot。
4. 3.1 動畫型別與完整資產配方已完成；接續通用 native skeletal 呈現、必要 Blueprint／UMG 與完整畫面驗收。

## 完整資產配方與重跑驗收

10. 接續資產計畫：新增純 Lua planner、MCP 批次套用及實際 Editor 重跑驗收。決定把本輪配方隔離到 `RecipeV1`，保留既有 Blueprint 與前段診斷資產。來源先做全量 preflight，source SHA-256 與操作參數納入 ledger；同來源動畫只匯入一次，失敗步驟可接續，未擁有的既有資產拒絕覆寫。
11. 材質工具無法從 `saika_magoichi_mat` 猜用途，改為明確 `type:BaseColor`；工具不允許重複建立材質，因此 owned pending 材質採讀回、驗證與保存後完成，不刪除重建。材質摘要格式與 mesh 不同，改用 `validate_material` 真實型別／shader 診斷，要求 valid、零 issue 並核對 Texture2D 依賴。
12. Saika 的單一 BaseColor atlas 套到生成 mesh 的兩個材質槽，保存後讀回完整 object path 與依賴。所有操作只針對本輪 ledger 擁有的資產；既有 `/Game/RustBP` 未改寫。native actor 目前仍是 fallback，不能把匯入完成說成畫面已接上。
13. Planner 測試、dry-run、完整 10 個唯一資產工作及實際 Editor 驗收均通過。7 個動畫槽共用 5 支動畫；重跑零 import、零材質 binding 變更，22 個保存套件 SHA-256 全數不變。3.1 完成，總進度更新為 14/30。驗收詳見 `openspec/changes/build-unreal-rust-moba-framework/evidence/editor-asset-recipe-2026-10-03.md`。
