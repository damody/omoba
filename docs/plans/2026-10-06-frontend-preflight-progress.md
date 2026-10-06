# 共用 Unreal 前端啟動檢查

## 問題與決定

遠端私人邀請選角新增後，只解析 UnrealEditor 路徑就開視窗，漏掉正式對局入口已有的 Unreal BuildId／模組 DLL 存在性與 staged bridge／base_content 副本雜湊檢查。這會讓遠端使用舊產物時先進入 UI 才失敗。

採通用 `moba_frontend_preflight.verify`：先以指定引擎檢查本專案二進位，再以固定 Lua 執行既有 `build_ue_moba.lua --verify-staged-only`。只有明確 exit_code=0 才成功；缺值或失敗都拒絕，不建置、不 stage、不改 BuildId、不切換引擎或 profile。

正式對局的前後兩次 stage gate 改用此入口；遠端選角在建立輸出目錄與 spawn renderer 前使用同一入口。原先的 scripts/moba_stage_contract.lua 維持 debug frontend 產物副本契約，並非新的 Rust profile 相容性證明；Rust moba-config 仍按指定 profile 解析。Dedicated server-only 不引入 Unreal 相依。

局部測試用依賴注入：檢查順序、指定引擎轉送、固定 Lua／verify-only 參數、成功及失敗 exit、拒絕時不建立遠端輸出／不啟動 renderer。這不是實機 BuildId／DLL 重新驗收或完整遊戲相容性驗收。

## 本輪證據

固定 Lua 的共用 gate 5、遠端選角 10、launch contract 8、既有 binary preflight 15、role launch 8、dedicated server 7，共 53 組通過。role／dedicated 部分使用既有已編譯 Rust 設定工具，其餘為 fixture／注入程序。沒有啟動 Unreal、遊戲、場次模擬、真實 LAN，也沒有 Cargo／UBT 建置。入口語法與 git diff --check 另行確認。完整六項仍維持待驗收，25/31，不以 mock 勾選。

Grok job `run-muwi643t-m7crhy`／thread `84673de8-6bbb-45f6-95ee-ca6cd221d050` 正常 write delegation 有文字進度，但停在讀取呼叫端階段，2分8秒沒有產出兩個指定檔案而由 primary 取消。stop 指令 exit1／taskkill128 表示 bridge PID 已不存在，不把該訊息單獨當清理成功；follower terminal cancelled、三個 tracked PID null，OS 查原42964／45864均不存在後才由 primary 實作。沒有 Grok 補丁可接受，成本／API 時間／根因未知，未更動全域 Grok／MCP／登入設定。

E334 保存漏接檢查及本輪失誤：曾猜讀不存在的 test_moba_launch_workflow.lua，改以 rg --files／rg 尋找真正 test_moba_launch_contract.lua，不新增重複測試入口。Lua 的新行括號呼叫可能黏到前一個回傳值，接線使用具名 local verify_frontend 再呼叫，避免此語法陷阱。所有實作確認通過，HEAD02757與原 dirty 保留；未 commit／push／omfx 維護。
