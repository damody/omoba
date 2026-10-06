# Unreal compiled-only reload 政策（2026-10-06）

## 問題與計畫

接續build-unreal-rust-moba-framework。Rust正式feature禁止runtime Lua，但Unreal仍可依舊設定送ENABLE_LUA_RELOAD旗標、每0.25秒遞迴掃Lua作者檔；WorldBridge手動reload在被拒絕之前清ClassCache及LastAttackStates。不能依賴Rust拒絕來掩蓋前端副作用與掃描成本。

## 實作與決策

- 移除正式Unreal的TickLuaContentWatcher呼叫、scanner／debounce方法及全部watcher私有狀態，遊戲不再掃描作者Lua目錄。
- runtime config不再送OM_RUNTIME_FLAG_ENABLE_LUA_RELOAD；舊reload／watcher設定欄位保留Config與反射名稱，標示deprecated並移除EditAnywhere，不再影響執行路徑。
- 保留WorldBridge及Subsystem的公開RequestRuntimeLuaReload名稱供既有資產引用，重設OutStatus、回Disabled與具體生成／編譯指引，一律false。不呼叫DLL reload，不清ClassCache／LastAttackStates、不啟動runtime；Subsystem只更新LastBridgeError診斷。
- 不移除Rust authoring工具或C ABI相容export，不變更Lua建置流程；內容仍Lua作者資料→生成→Rust／Unreal C++編譯執行。
- 新CompiledContentReloadPolicy native案例涵蓋舊OutStatus清除、兩入口Disabled與一致指引、拒絕保留baseline actor、不建立runtime；加入既有scoped Lua runner。

## 局部確認與限制

- UE限定OmRuntime＋OmGenerated＋OmEditor：14 actions、Result: Succeeded、exit0；Saved/Logs/compiled-content-reload-policy.log。
- 固定Lua runner loadfile及兩repo whitespace通過；搜尋Source內scanner與DLL reload呼叫、reload flag設定均無命中（rg exit1是預期零結果，不是建置失敗）。
- 新native斷言僅編譯未執行，遵守E285不重試Cmd啟動器；未PIE／stage／完整驗收。既有plugin dependency warnings保持，沒有新編譯失敗。
- ABI16／wire6／IPC5與生成hash未改。整體21/31；6.0既有compiled-only實作補強，不勾完整2.2b或6.4。見E288。
