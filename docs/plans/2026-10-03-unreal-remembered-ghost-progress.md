# Unreal 最後已知位置：原生安全記憶呈現

## 本輪計畫

1. 只解碼 IPC 的 server-sanitized remembered payload，透過獨立 C ABI 陣列接到 Unreal。
2. 使用原生 instanced marker，不建立可攻擊的 live unit，也不需要手寫角色 Blueprint。
3. 分清完整視野與控制 frame，驗證 Hide／Forget／ResetView／斷線／重連。
4. 測 Rust、完整 Unreal build、同一 Editor 兩輪 automation 與 PIE 渲染，記錄錯誤後修正重跑。

## 問題與決定

- **C ABI 邊界**：增加 `OmRememberedGhost` 與 header 陣列，ABI 由 1 升至 2；DLL、header 與 OmGame 由同一建置入口同步產生。IPC 本身維持 v3，沒有新增網路 protocol。ghost epoch 保留完整 u64，不轉為 live entity 的 u32 generation。
- **資料來源**：只能取已核准的 `sanitized_presentation`；不查 hidden world 的當前位置。不合法長度、零身分或與 live ID 衝突的記憶不呈現。ABI 只攜帶 ID、epoch、位置與 kind，不含 owner、HP、技能、動畫或攻擊範圍。
- **呈現與輸入分離**：`RememberedGhosts` 使用原生 Cube 的獨立 ISMC，無碰撞、無 overlap、無導航影響、無影子，沒有 live actor／血條／選取能力。它是中性的最後位置 fallback 標記，不是假裝已完成英雄美術剪影。正式輸入查找使用最新完整安全視野的 live ID／generation，ghost-only 視野即使 frame ring 忙碌也拒絕攻擊目標。
- **熱路徑**：位置未改變就不 ClearInstances／AddInstance；不採用只有 hash 的比較。正常發布 frame 以 ownership transfer 移動 Vec，忙碌時取回原 FrameBuild，避免複製每個正常 frame 或重複附加控制紀錄。
- **生命周期**：driver 保存最近已核准的完整 snapshot，局部 Hide 只移除匹配身分並加入該事件核准的記憶，Forget 只移除匹配 epoch 記憶；舊 Forget 不刪新記憶。完整空 view 清除 live actors 和 markers；只有記憶的斷線也必須發 reset。重連可直接使用 latest snapshot，不需等待新 sim tick。
- **控制 frame**：新增 `presentation_snapshot`，input result 不清除記憶、地圖／fog 或 live actor 清單。尚未被 renderer 取得的 removal records 不能觸發第二個空 view 覆蓋剛發布的完整視野。真正 reset 忙碌時獨立重試。
- **停止清理**：StopRuntime 銷毀本 bridge 管理的 live／pool actors 並清除 markers，不能只清空參照而讓舊 actor 留在場景。
- **畫面驗收**：新增僅存在於 OmEditor 的 `AOmAutomationMemoryProbe`，不啟動、不 tick、不停止共享 runtime；MCP 使用它把 synthetic safe frame 送進正式 ProcessFrame。fixture 不是完整兩隊玩法驗收，也不納入 packaged gameplay API。

## 驗證指令與結果

```text
cargo test --manifest-path omfue/bridge/Cargo.toml
tools\lua\lua.exe scripts\build_ue_moba.lua --full --ue-root D:\UE5.8
tools\lua\lua.exe scripts\ue_native_visual_smoke.lua
tools\lua\lua.exe scripts\ue_pie_smoke.lua
git -C omfue diff --check
```

- Rust bridge：33/33 unit tests、2/2 TD single-player integration tests 通過；1 個需外部 KCP backend 的 integration test 維持 ignored，沒有宣稱已跑 LAN。
- 新增 Rust 驗證：malformed memory fail closed、獨立 frozen memory／u64 epoch、slot ownership、忙碌重試／不可攻擊 ghost、未 ACK removals 不覆蓋 view、局部 lifecycle 保留其他 live、stale Forget、ResetView，以及真實 TCP ghost-only 斷線／重連兩輪相同位置（不啟動 sim thread）。
- 完整 build 最終 session 84528 exit 0：Rust bridge staging、OmGameEditor、MCP ready 通過；Editor PID 91580。先前失敗及修復見 E025。
- Editor automation：最終 session 28134 exit 0，同一 Editor 兩輪皆 7/7，failed／skipped／not_run=0；涵蓋 native art／animation、舊 Blueprint 相容、projectile history、synthetic dispatch、remembered ghost。
- PIE：最終 session 28134 exit 0，17 個 sequence steps、PASS、zero failed／could-not-tell assertions；native_mesh_rendered=true、remembered_ghost_rendered=true、counts=[1,0]，最終 pie_running=false。透過正式 component 渲染 memory marker，先擷取實際 frame 再檢查 `WasRecentlyRendered`。native hero 與缺圖 fallback 保持原驗收；自行啟動的 PIE 在成功或失敗時皆停止。
- Lua syntax、diff whitespace 檢查通過；既有 dependency／dead-code／換行警告仍存在，不宣稱已清除。

可重建的非版控報告：`omfue/Saved/McpAutomation/NativeVisual/report.json`、`pie-smoke-report.json`，畫面 `pie-remembered-ghost.png`。Saved、DLL、EXE 等不提交。

## 未完成與下一步

OpenSpec 維持 14/30，4.3 不勾選。此輪補上獨立 memory 呈現，不代表完成跨新 renderer instance 的所有一次性 cue 消費契約。下一步先驗證實際兩隊 filtered world 的 Unreal adapter／移動，再補跨新 instance cue replay gate；之後處理單一 MOBA 模擬路徑、對局規則與通用 UI。現有 PIE memory fixture 不冒充「兩台玩家完整對局」。
