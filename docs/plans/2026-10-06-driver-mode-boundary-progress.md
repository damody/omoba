# Unreal bridge 的啟動模式邊界

## 計畫與決定

1. 檢查4.4的舊 gameplay 隔離；既有 explicit IPC intent 已禁止缺位址時 fallback，不重做這項功能。
2. SinglePlayer 是啟動拓樸，與 IPC 同時設定有既定用途；不得把合法組合改成互斥，也不能用 story 名稱猜 MOBA 模式。舊 campaign 不含正式 MOBA match 設定。
3. 新增私有閉集 DriverMode，啟動前唯一解析 Presentation／LocalTd／NetworkTd。速率初值與執行緒建立只使用同一結果，match 明列三個分支。
4. 發現 C ABI 的 IPC 非零身分限制未涵蓋內部直接 driver 呼叫，補在共用 driver 啟動前；非法設定在 channels／workers 配置前拒絕。
5. IPC 不需要本機 story／script DLL，且始終只有呈現執行緒；TD 保持本機 story／DLL 與網路位址的既有檢查。

## 當前功能確認

以下三項指定測試各1 passed：

```text
cargo test --manifest-path omfue/bridge/Cargo.toml driver_mode_is_explicit_and_validated_before_startup -- --nocapture
cargo test --manifest-path omfue/bridge/Cargo.toml ipc_modes_receive_filtered_frame_send_move_and_stop_while_idle -- --nocapture
cargo test --manifest-path omfue/bridge/Cargo.toml presentation_intent_cannot_fall_back_to_legacy_gameplay -- --nocapture
```

- 新矩陣檢查兩種拓樸、三種模式、零玩家／隊伍、無本機資料的 IPC、非法混用端點及 TD 缺 story／DLL。
- IPC 測試實際建立 localhost TCP 收送：握手、filtered frame、Move 輸入、idle stop；兩種拓樸都斷言 sim_thread 為空。
- fallback 測試保留：即使提供可用舊輸入，缺 IPC 位址仍拒絕；未明確設定模式的 IPC 端點也拒絕。

## 防錯與驗收界線

E270 記錄內部入口驗證缺口與查詢猜錯路徑，沒有新增編譯失敗。先列檔案再讀，不能把檔案查詢失敗當建置失敗。既有 dead_code warnings 未在本輪擴大處理。

本輪無 Lua 內容／VM、ABI／wire／IPC 協定、Unreal C++ 變更；未部署 release binary，不重啟 Editor。三项只證明當前 driver 功能，完整單機／LAN／對局仍留最後，4.4保持未勾選，總進度21/31。
