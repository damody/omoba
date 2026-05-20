## Context

UECP 的 `delete_function` 目前由 `BlueprintDeletionTools::HandleDeleteFunctionWithForce` 處理。流程會載入 Blueprint、在 `TargetBlueprint->FunctionGraphs` 找同名 graph、做簡單 non-trivial node 防呆，接著建立 `FScopedTransaction`，呼叫 `FBlueprintEditorUtils::RemoveGraph(TargetBlueprint, FunctionGraph)`，最後 `MarkBlueprintAsStructurallyModified`。

這條路徑對正常 function graph 可用，但對半建立或破損 graph 風險很高。這次 `BP_PlayerController` 的 `RecordSubmitBoolStatus` 與 `SetSelectedTowerPriorityFirstFromUI` 都已確認仍存在，且 graph 曾處於 compile error 狀態；其中 `SetSelectedTowerPriorityFirstFromUI` 缺 function entry root node。一般 editor deletion/structural compile path 在這種 graph 上可能進入長時間等待或卡死，導致 MCP server worker 停在 `RunOnGameThread` / task wait，UECP endpoint 也就無法服務後續 request。

現有 `MCPDispatch::RunOnGameThread` 會等待 saving/GC 清空、投遞到 game thread，然後用 `FTaskGraphInterface::Get().WaitUntilTaskCompletes(Task)` 無限等待。`TryDispatcherFallback` 也用 `Task->Wait()` 等待 dispatcher handler。這代表任何單一工具如果卡住，server worker thread 就會卡住，後續連線會 timeout 或 connection refused-like。

## Goals / Non-Goals

**Goals:**

- 讓 `delete_function` 對正常 function graph 維持既有行為與防呆。
- 讓 `delete_function` 在 graph 缺 `UK2Node_FunctionEntry`、缺 root、owner 異常或 function list 不一致時，走可預期的 corrupt graph fallback。
- 避免 fallback path 觸發容易在破損 graph 上卡住的高階 editor cleanup 或 compile。
- 回傳 structured JSON，讓 agent 能知道是否使用 fallback、破損原因與後續是否需要 compile/validate。
- 讓 MCP/UECP dispatch 對 game-thread task 有有界等待，避免單一工具卡住 server worker。
- 讓 Blueprint graph mutating tools 都進入相同的有界等待與 busy/unknown 防護，不只 `delete_function`。
- 防止 `clear_before_build`、`clear_blueprint_graph`、`delete_nodes`、`remove_node(s)` 刪除 function graph 的 entry/root/return terminator。
- 讓 `ping` / `health` 不需要 game thread，至少能區分 socket bridge 還活著但 game thread 忙碌，或整個 listener 已停止。
- 加入可重現 regression smoke，確保刪除破損 function 後 Blueprint 可讀，UECP endpoint 仍可回應。
- 所有編譯驗證都使用 UE/UBT 增量編譯；除非使用者明確要求，不執行 clean、full rebuild、重新產生整個 solution 或會重編全專案的流程。

**Non-Goals:**

- 不嘗試修復 `BP_PlayerController` 的 gameplay Blueprint 邏輯或節點誤連，例如 Tick 裡 status node 被解析成 `StartRoundFromUI`。
- 不刪除整個 Blueprint asset。
- 不在 background thread 直接操作 UE UObject 或 Blueprint graph；所有 UObject mutation 仍在 game thread。
- 不承諾可以中止已經在 game thread 內部無限卡住的 Unreal API。防護重點是 preflight 避免進入已知危險 path，dispatch timeout 則避免 server worker 無界等待。
- 不把驗證流程設計成完整 UE 專案重建；本 change 的驗證只需要受影響 plugin/module 的增量編譯與針對性 smoke。
- 不在這個 change 中重寫所有 Blueprint graph edit 為完整 async job queue；先用 bounded wait / busy guard / root-preserving preflight 收斂風險。

## Decisions

1. `delete_function` 先做 graph health classification，再決定刪除路徑。

   新增內部 helper，例如 `ClassifyFunctionGraphForDeletion(UBlueprint*, UEdGraph*, FFunctionGraphDeleteHealth&)`。分類至少檢查：

   - `FunctionGraph` 是否存在於 `TargetBlueprint->FunctionGraphs`。
   - `FunctionGraph->GetOuter()` 或 typed outer 是否屬於 target Blueprint/package。
   - 是否有 `UK2Node_FunctionEntry`。
   - 是否有多個 function entry。
   - graph 名稱是否與 requested `FunctionName` 一致。
   - node array 是否含 invalid/null node。
   - 是否為 interface、macro、ubergraph 或 inherited/native function，避免誤走 function deletion。

   分類結果包含 `Normal`、`MissingEntryRoot`、`InvalidOwnership`、`MalformedNodes`、`NotFunctionGraph` 等 reason。正常 graph 才使用既有 high-level deletion；corrupt graph 走 fallback。

   Alternative considered: 直接對所有 function graph 使用低階 removal。拒絕原因是正常 graph 應保留 Unreal editor 的 transaction、reference cleanup 與 structural modification 行為，避免破壞既有工作流。

2. Corrupt graph fallback 使用受控低階 removal，並避免立即 structural compile。

   fallback path 在 game thread 內執行，做最小必要 mutation：

   - 建立 `FScopedTransaction`，transaction label 明確標示 corrupt delete。
   - `TargetBlueprint->Modify()`、`FunctionGraph->Modify()`。
   - 從 `TargetBlueprint->FunctionGraphs` 移除該 graph。
   - 必要時呼叫 graph/node break links 或 rename 到 transient/deleted outer，讓 editor 不再從 Blueprint function list 找到它。
   - 清理與該 function name 對應的 metadata/entry points，僅使用對 corrupt graph 不會重新走完整 graph validation 的 API。
   - 使用 `FBlueprintEditorUtils::MarkBlueprintAsModified(TargetBlueprint)`；只有在確定安全時才做 `MarkBlueprintAsStructurallyModified`。

   fallback 回傳 warning，提示使用者下一步可執行 `validate_blueprint` 或 `compile_blueprint`。若 compile 仍失敗，錯誤應歸屬於 Blueprint 其他破損節點，而不是 deletion endpoint 卡死。

   Alternative considered: 先新增 missing `FunctionEntry` 再走 `RemoveGraph`。這可能修復部分 case，但也可能改變破損 graph 的語意或觸發 reconstruction/compile path；第一版只在刪除前做最小 mutation，不主動重建 graph。

3. `force=true` 只覆蓋「含有較多 non-trivial nodes」防呆，不覆蓋 corrupt graph 安全路徑。

   `force=true` 代表使用者接受刪除正常 function 的內容，不代表要使用高風險 editor cleanup。若 graph 被分類為 corrupt，仍必須走 fallback，並在 JSON 裡回報 `forced` 與 `fallback_used`。

   Alternative considered: `force=true` 一律使用既有 `RemoveGraph`。拒絕原因是這正是會讓破損 graph timeout 的危險路徑。

4. `delete_function` 回傳 structured JSON。

   成功時至少回傳：

   - `success`
   - `deleted`
   - `function_name`
   - `blueprint_path`
   - `deletion_path`，例如 `normal_remove_graph` 或 `corrupt_low_level_remove`
   - `corrupt`
   - `corrupt_reasons`
   - `warnings`

   失敗時透過既有 `OutError` 也保留 JSON body，讓 `BuildToolResponse` 可合併 error。batch delete 每個 item 應保留相同欄位，避免只剩文字錯誤。

   Alternative considered: 只修 hang、不改回傳格式。拒絕原因是 agent 需要知道這次是否走 fallback，才能決定是否接著 compile、validate 或停手。

5. MCP dispatch 增加 bounded wait helper。

   新增 helper，例如 `RunOnGameThreadWithTimeout` 或擴充 `RunOnGameThread`，以短間隔輪詢 task completion，超過 per-tool timeout 後回傳 timeout JSON，並讓 server worker 繼續處理後續 request。timeout 必須記錄 command type、elapsed、是否已投遞到 game thread，以及 task 是否仍可能在背景完成。

   對 destructive/write tools 使用較保守 timeout，預設可從 config 讀取；若未設定，採用明確常數，例如 30 秒。對 known long-running import/build 類工具可後續另設 whitelist，但 `delete_function` 不應長時間執行。

   Alternative considered: 只依賴 Python MCP client timeout。拒絕原因是 client timeout 後 server worker 仍可能卡在 `WaitUntilTaskCompletes`，下一次 request 仍無法處理。

6. timeout 後進入 tool-busy/uncertain state，而不是宣稱 mutation 一定失敗。

   Unreal task 一旦投遞到 game thread，C++ 層不能安全取消。若 bounded wait timeout 發生，回傳應明確標示 `state:"unknown"` 或 `operation_may_still_complete:true`。在 task 完成前，後續 destructive tools 可回報 busy，read-only health/ping 則仍可回應，避免重入同一 Blueprint 造成更多破損。

   Alternative considered: timeout 後直接重試同一 destructive action。拒絕原因是可能造成同一 graph 被重複 mutation。

7. Blueprint graph mutation 與 destructive delete 共用 busy guard。

   `build_blueprint_graph`、`place_node`、`connect_pins`、`set_pin_default`、`clear_blueprint_graph`、`delete_nodes`、`remove_node(s)`、`compile_blueprint` 等工具都可能修改 Blueprint 或觸發 editor graph validation。它們應使用 bounded wait，timeout 後回報 `operation_may_still_complete=true`，並在前一個 mutating operation 未確認完成時拒絕下一個 mutating operation。這個 guard 不等同 UI confirmation；不必把所有 write tools 都當成「需要人工 destructive confirm」，但 server 層要避免未知狀態下重入。

8. Function graph clear/remove 必須保 root。

   `clear_before_build` 與 `clear_blueprint_graph` 對 function graph 執行前，必須確認 graph 有且只有一個 `UK2Node_FunctionEntry`。若缺 entry root 或有多個 entry，這已經是 corrupt graph，清空/重建只會放大破損，應拒絕並要求先用 `delete_function` corrupt fallback 移除後重建。刪節點 API 必須拒刪 `UK2Node_FunctionEntry` / `UK2Node_FunctionResult`；刪 temporary inspection node 可以繼續，但不能碰 function root/terminator。

## Risks / Trade-offs

- [Risk] 低階 removal 漏清 editor reference，造成 Blueprint 面板顯示 stale function。→ Mitigation: fallback 後執行 function list 查詢與 `get_blueprint_functions` smoke；必要時補上安全 metadata 清理。
- [Risk] `MarkBlueprintAsModified` 不觸發完整 structural refresh，導致編譯前狀態不同步。→ Mitigation: fallback 回傳 warning，並在刪除後做輕量 refresh/validate；只有安全 case 才 structural modified。
- [Risk] game thread 已經被 Unreal API 卡死時，bounded wait 無法讓 editor 恢復。→ Mitigation: preflight 避免呼叫已知危險 API；timeout 訊息明確說明 operation state unknown，建議重啟 editor 前不要重試 destructive action。
- [Risk] timeout 後 task 仍完成，agent 誤判失敗。→ Mitigation: 回傳 `operation_may_still_complete`，後續 workflow 先用 read-only `get_blueprint_functions` 查狀態。
- [Risk] graph health classification 漏掉某種破損型態。→ Mitigation: classification 預設保守；任何缺 entry/root、invalid node、owner 異常都走 fallback，不把未知破損交給 high-level path。
- [Risk] batch delete 中一個破損 function 影響其他項目。→ Mitigation: batch 每項獨立分類、獨立結果；遇到 timeout 或 uncertain state 後停止後續 destructive item。

## Migration Plan

1. 新增 graph health classification helper 與 JSON result builder。
2. 將 `HandleDeleteFunctionWithForce` 拆成 normal path 與 corrupt fallback path。
3. 更新 `HandleDeleteFunctionFromArgs` 與 batch 回傳，保留 per-item `deletion_path` 與 `corrupt_reasons`。
4. 在 MCP dispatch helper 加入 bounded wait，先套用 `delete_function` 或 destructive dispatcher path，再視測試結果擴到其他 game-thread write tool。
5. 新增 corrupt function graph smoke：程式化建立缺 `UK2Node_FunctionEntry` 的 function graph 或使用測試 Blueprint fixture，呼叫 `delete_function`，再呼叫 `get_blueprint_functions`/`validate_blueprint`/`ping`。
6. 更新 `tool_docs.md` 的 `delete_function` 說明。
7. 編譯驗證採用增量 UBT/plugin/module build：不加 clean/rebuild 參數，不刪 `Intermediate`/`Binaries`，不重新產生整個 solution；若環境只能 full rebuild，先記錄限制並跳過編譯驗證。
8. 在 UE 5.7 editor smoke 中對 `BP_PlayerController` 類型資產驗證 endpoint 不會因破損 graph 刪除而失去回應。

Rollback strategy: 若 fallback deletion 對特定 Blueprint 造成 stale metadata，先保留 graph health detection 與 timeout protection，將 corrupt graph delete 改為拒絕並回傳 diagnostic，不再走 high-level `RemoveGraph`。

## Open Questions

- UE 5.7 對 function graph 低階 removal 後是否需要額外呼叫特定 refresh API 才能讓 My Blueprint 面板立即更新，需要由 editor smoke 確認。
- 是否要新增獨立 `inspect_function_graph` 或在既有 `get_blueprint_functions` 中加入 health 欄位，方便 agent 在刪除前先查詢。
- destructive tool timeout 的預設值應固定在程式碼中，還是沿用 UECP settings/config。建議先用 config with default，避免未來長操作共用同一值。
