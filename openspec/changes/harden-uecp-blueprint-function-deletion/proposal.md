## Why

UECP 的 `blueprint(action="delete_function")` 目前在遇到破損 Blueprint function graph 時可能卡住 Unreal Editor game thread，導致 MCP/UECP endpoint timeout，甚至後續連線變成 timeout 或 connection refused。這次 `BP_PlayerController` 的 `RecordSubmitBoolStatus` 與 `SetSelectedTowerPriorityFirstFromUI` 都顯示：刪除本身不是危險點，危險點是缺少 function entry/root node 或半建立 graph 時仍走一般 `FBlueprintEditorUtils::RemoveGraph` 流程。

## What Changes

- 強化 UECP `delete_function`，刪除前檢查 target function graph 是否缺少 function entry/root node、graph ownership 異常、節點狀態破損或與 Blueprint function list 不一致。
- 對破損 function graph 加入 fallback deletion path：避免依賴會 hang 的高階 editor cleanup，改用受控的低階 graph removal、參照清理、mark modified/save-ready 狀態與清楚 diagnostic。
- 將 `delete_function` 的回傳 JSON 擴充為可觀測結果，包含 deletion path、corrupt graph reason、deleted flag、warnings，以及是否跳過 compile/structural compile。
- 強化 UECP dispatch/game-thread 執行保護，讓單次 tool handler timeout 或卡住時不會拖死整個 MCP/UECP endpoint。
- 將保護範圍擴大到 Blueprint destructive graph mutation：`build_blueprint_graph`、`clear_blueprint_graph`、`delete_nodes`、`remove_node(s)`、`connect_pins`、`set_pin_default`、`compile_blueprint` 等 mutating tools 需要 bounded wait / busy guard；health/ping 不需 game thread。
- 修正 function graph 清空/刪 node 行為，保證 `UK2Node_FunctionEntry` / `UK2Node_FunctionResult` 不會被 `clear_before_build`、`clear_blueprint_graph`、`delete_nodes` 或 `remove_node(s)` 刪掉；若 function graph 已缺 entry root，清空/重建必須拒絕並要求走 corrupt function fallback。
- 加入針對破損 function graph 的 regression test 或 editor automation/smoke，覆蓋缺 function entry root node、半建立 helper function、重試刪除與 endpoint 後續可讀性。
- 更新 tool docs，要求 agent 在刪除 function 前可先查詢/診斷 function graph health，並說明破損 graph 的安全刪除行為。

## Capabilities

### New Capabilities
- `uecp-blueprint-function-deletion`: 定義 UECP Blueprint function deletion 的安全契約，包含正常 function、破損 function graph、fallback deletion、逾時隔離與 endpoint 存活性。

### Modified Capabilities
- 無。

## Impact

- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Source\UECPTools\Private\Tools\BlueprintDeletionTools.cpp`: `HandleDeleteFunctionWithForce` 需要新增 graph health check、corrupt graph fallback、structured JSON result 與避免高階 API hang 的刪除路徑。
- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Source\UECPTools\Public\Tools\BlueprintDeletionTools.h`: 可能需要新增內部 helper 或公開診斷 API，視實作切分而定。
- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Source\UECPMCPBridge\Private\MCPDispatchHelpers.cpp`: `RunOnGameThread` 或同等 dispatch helper 需要避免無界等待，至少讓 endpoint 能對卡住或逾時的工具回報錯誤並繼續處理後續連線。
- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Source\UECPMCPBridge\Private\MCP_EditorSubsystem.cpp`: destructive tool dispatch 可能需要 per-request timeout/health state，避免單一 destructive command 讓 server worker 停在等待。
- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Source\UECPMCPBridge\Private\Dispatch\MCPDispatch_FastPath.cpp`: `ping` / `health` 應可不依賴 game thread 回應，方便 timeout 後判斷 bridge 是否仍活著。
- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Source\UECPTools\Private\Tools\BlueprintGraphTools.cpp`: `clear_before_build`、`clear_blueprint_graph`、`remove_node(s)` 需要保護 function root/terminator 並拒絕覆蓋 corrupt function graph。
- `D:\omoba\omfue\Plugins\BpGeneratorUltimate\Content\Python\tool_docs.md`: 更新 `delete_function` 行為與建議 workflow。
- UE automation 或 smoke script：新增可重現 corrupt function graph 的測試資產或程式化建構，驗證刪除後 Blueprint 可再讀取，UECP endpoint 仍回應。
