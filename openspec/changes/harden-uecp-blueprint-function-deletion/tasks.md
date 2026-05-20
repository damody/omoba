## 1. Delete Function Graph Health

- [x] 1.1 在 `BlueprintDeletionTools.cpp` 新增 function graph health result 型別，記錄 `Normal` / corrupt reasons / warnings / deletion path。
- [x] 1.2 實作 `delete_function` preflight 檢查，確認 graph 位於 `TargetBlueprint->FunctionGraphs`、owner 正確、名稱符合 requested function、node list 可安全遍歷。
- [x] 1.3 在 preflight 中辨識缺少 `UK2Node_FunctionEntry`、多個 entry root、invalid/null node、非 function graph 或 inherited/native function 等狀態。
- [x] 1.4 保留正常 graph 的 non-trivial node 防呆，並確認 `force=true` 只覆蓋這個防呆，不覆蓋 corrupt classification。

## 2. Safe Corrupt Graph Deletion

- [x] 2.1 將 `HandleDeleteFunctionWithForce` 拆成 normal deletion path 與 corrupt fallback deletion path。
- [x] 2.2 normal path 維持既有 `FBlueprintEditorUtils::RemoveGraph` 行為，但只允許 graph health 為 normal 時使用。
- [x] 2.3 corrupt fallback path 從 `TargetBlueprint->FunctionGraphs` 低階移除 graph，並避免觸發需要有效 function entry/root 的高階 cleanup。
- [x] 2.4 fallback path 正確呼叫 `Modify()`、清理 graph links/owner 狀態，並以不會 hang 的方式 mark Blueprint modified。
- [x] 2.5 刪除後重新查詢 function list，確認 target function 不再出現；失敗時回傳可診斷錯誤。

## 3. Structured Results

- [x] 3.1 為 single `delete_function` 回傳 JSON，包含 `success`、`deleted`、`function_name`、`blueprint_path`、`deletion_path`、`corrupt`、`corrupt_reasons`、`warnings`。
- [x] 3.2 更新 `HandleDeleteFunctionFromArgs` batch path，讓每個 item 保留 deletion path、corrupt reasons、warnings 與 failure diagnostic。
- [x] 3.3 確認 `OutError` 與 `OutJsonString` 在成功、拒絕、fallback、batch partial failure 時都能被 `BuildToolResponse` 正確合併。
- [x] 3.4 更新 `tool_docs.md` 的 `delete_function` 說明，加入 corrupt graph fallback、timeout 後先 read-only 查狀態、以及避免重試 destructive action 的建議。

## 4. UECP Dispatch Timeout Protection

- [x] 4.1 在 `MCPDispatchHelpers.cpp` 新增有界等待 helper，避免 `WaitUntilTaskCompletes` 或 `Task->Wait()` 無限阻塞 server worker。
- [x] 4.2 將 `delete_function` 或 destructive write-tool dispatch 接到 bounded wait path，逾時時回傳 timeout JSON 與 `operation_may_still_complete`。
- [x] 4.3 加入 timeout 後的 busy/uncertain state guard，避免同一 destructive operation 在前一個 task 未確認完成前立即重入。
- [ ] 4.4 確認 read-only request 或 health/ping 在 editor game thread 未永久卡死時仍能得到回應。

## 5. Regression Coverage

- [x] 5.1 新增 UE editor automation 或 smoke helper，程式化建立缺 `UK2Node_FunctionEntry` 的 Blueprint function graph fixture。
- [ ] 5.2 用 UECP `delete_function` 刪除該 corrupt function，驗證回傳 `corrupt=true` 並使用 fallback deletion path。
- [ ] 5.3 刪除後呼叫 `get_blueprint_functions`，確認 function 不再存在且 Blueprint asset 仍可讀。
- [ ] 5.4 加入 endpoint survival smoke：刪除 corrupt function 後再呼叫 read-only UECP request，確認不 timeout。
- [x] 5.5 覆蓋正常 function graph deletion，確認仍使用 normal path 且既有 force/non-trivial node 行為不回歸。

## 6. Verification

- [x] 6.1 執行 UECP plugin/module C++ 增量編譯，確認 `BpGeneratorUltimate` 受影響 modules 可編譯；不得執行 clean、full rebuild、刪除 `Intermediate`/`Binaries` 或重新產生整個 solution。
- [ ] 6.2 執行新增的 corrupt function graph automation/smoke。
- [ ] 6.3 對 `/Game/RustBP/System/BP_PlayerController` 先用 read-only 查詢確認 `RecordSubmitBoolStatus` 與 `SetSelectedTowerPriorityFirstFromUI` 狀態，再只在需要時用修正後 `delete_function` 驗證安全刪除。
- [ ] 6.4 執行一次 UECP endpoint health/read-only smoke，確認 destructive timeout 或 fallback 後 endpoint 仍可讀。
- [x] 6.5 記錄未執行的 UE editor smoke 或 compile 限制，包含 Unreal Engine path、editor 是否已啟動、是否只能 full rebuild、以及是否需要手動重啟 editor；若只能 full rebuild，跳過並說明原因。

### Verification Notes

- 2026-05-18 17:01 +08:00: Ran incremental UBT compile only:
  `D:\UE5.7\Engine\Build\BatchFiles\Build.bat OmGameEditor Win64 Development -Project=D:\omoba\omfue\om.uproject -WaitMutex`.
  Result: succeeded; UECPMCPBridge and UECPTools compiled and linked. No clean/rebuild, no `Intermediate`/`Binaries` deletion, no solution regeneration.
- UE editor state during smoke attempt: `UnrealEditor.exe` process was still alive and responding, but `D:\omoba\omfue\Saved\Logs\om.log` showed `MCP Server: Stopped.` and UECP modules shutdown at 2026-05-18 16:51:52 +08:00. No `9877-9886` listener was active, so UECP read-only and corrupt graph smoke requests could not be executed in this session.
- Engine path used for verification: `D:\UE5.7`. A manual editor restart/reload of the freshly built plugin is required before running tasks 4.4, 5.2, 5.3, 5.4, 6.2, 6.3, and 6.4.

## 7. Blueprint Graph Mutation Hardening

- [x] 7.1 將 `build_blueprint_graph`、`place_node`、`connect_pins`、`set_pin_default`、`remove_node(s)`、`clear_blueprint_graph`、`compile_blueprint` 等 Blueprint mutating tools 納入 bounded wait / busy guard，不只保護 `delete_function`。
- [x] 7.2 新增 `ping` / `health` fast path，不需 game thread dispatch 即可回應 bridge 是否仍活著。
- [x] 7.3 `clear_before_build` 與 `clear_blueprint_graph` 在 function graph 缺 `UK2Node_FunctionEntry` 或有多個 entry 時拒絕執行，要求使用 corrupt `delete_function` fallback 後重建。
- [x] 7.4 `clear_before_build` 與 `clear_blueprint_graph` 永遠保留 `UK2Node_FunctionEntry` 與 `UK2Node_FunctionResult`。
- [x] 7.5 `delete_nodes`、`remove_node`、`remove_nodes` 拒刪 `UK2Node_FunctionEntry` / `UK2Node_FunctionResult`，回傳 protected-node diagnostic。
- [x] 7.6 執行 UECP plugin/module C++ 增量編譯，確認 Blueprint graph mutation hardening 可編譯；不得 clean/rebuild。
- [ ] 7.7 Editor bridge 恢復後 smoke：`ping`、`health`、function graph clear protected-node refusal、delete temporary body node、read-only follow-up。

### Blueprint Graph Mutation Verification Notes

- 2026-05-19 17:02 +08:00: Ran incremental UBT compile only:
  `D:\UE5.7\Engine\Build\BatchFiles\Build.bat OmGameEditor Win64 Development -Project=D:\omoba\omfue\om.uproject -WaitMutex`.
  Result: succeeded; UBT compiled 10 actions and linked UECPMCPBridge / UECPTools. No clean/rebuild, no `Intermediate`/`Binaries` deletion, no solution regeneration.
- Runtime smoke for 7.7 still requires restarting/reloading UnrealEditor so the freshly built UECP DLLs are active.
