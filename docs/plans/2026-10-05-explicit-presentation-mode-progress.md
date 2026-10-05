# 明確呈現模式與 gameplay 隔離

## 本批計畫與決策

1. 接續 4.4，不重新跑 LAN／雙 UE／完整驗收；先修正可能回落舊玩法的共用入口。
2. 原 bridge 用 presentation_address 是否非空推測模式。PRESENTATION_IPC 已設定但空位址時，has_driver_config／spawn_runtime_driver 可能改選舊 single-player 或 selective gameplay；改保存 explicit presentation_only，再由該模式決定派發／輸入回覆／driver。
3. C ABI runtime create 先驗所有 string view，再拒絕呈現模式的空／純空白位址，out_runtime 維持 null。內部 driver 再驗相同界線，也拒絕有 IPC 位址卻未宣告呈現模式。即使提供有效 legacy DLL／Story／server，也不得取代 intent。
4. Unreal 的 EOmRuntimeMode append PresentationIpc，不改舊 enum 值；加入 config PresentationAddress（CLI→env→settings）。OM_RUNTIME_MODE=presentation-ipc／-om-presentation-only 為明確意圖，CLI 呈現旗標優先於 legacy 單機／連線旗標。有效舊 IPC 位址設定仍自動相容，但明確呈現模式缺位址不代入 KCP endpoint。
5. 正式 Lua 雙前端啟動器使用 presentation-ipc env 與 -om-presentation-only；日誌輸出實際呈現模式。Rust C ABI11 不變，舊 TD／未指定呈現的流程保留，不依英雄、Story 或地圖名稱判斷模式。

## 當前功能確認

- 新 C ABI 缺位址拒絕與 driver intent 拒絕測試各 1/1：涵蓋兩種 local_mode、空／純空白、有效 legacy inputs 不能 fallback。
- 既有真實 localhost IPC 測試 1/1：兩種 single_player flag 都 sim_thread=None，握手／安全 frame／移動／停止正常。
- IPC 身分與舊 ABI 拒絕、terminal result／busy ring、legacy single-player config 各 1/1。共六個指定測試通過，未重跑整套。
- Lua launcher loadfile 語法檢查通過。
- 原 Editor102680 的 executable／project command line 已核對；project-scoped stop 回報 matching_editors=1／force_terminated=true，另確認原 PID 不存在。
- 正常 build-only53273 exit0，OmGameEditor 12 actions／Succeeded，含新的 UHT enum／config 與 OmRuntimeBridgeSubsystem。既有入口核對 staged bridge SHA `9a5d7856e46883bf5d18466999eb86fb3f68434f441e55055dfa211abe5bf0ad`，C header unchanged／ABI11。
- 未追加完整 MCP gate／PIE／雙 UE／LAN／效能驗收；本批建置與六個指定測試已足以確認目前隔離功能。新 Editor14716 啟動成功並保留開啟，未額外等待 MCP，不冒充 readiness。

## 剩餘範圍

- 這是明確模式與禁止 fallback 的架構實作，不是兩台 LAN 或完整單機 MOBA 啟動流程驗收。run_ue.lua 舊 TD 入口未改成新 MOBA launcher。
- runtime-driver 編譯 feature 與舊 TD gameplay 實作仍共存；正式呈現分支不生成 world／sim thread，但此批不宣稱 DLL 已移除全部 legacy code。
- 完整 4.4 留最後驗收，20/30 不變。不新增角色 C++／Blueprint graph，不提交或清理使用者變更。
