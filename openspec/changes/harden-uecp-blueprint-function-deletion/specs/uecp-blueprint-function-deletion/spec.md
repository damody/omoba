## ADDED Requirements

### Requirement: Function deletion classifies graph health before mutation

UECP `delete_function` SHALL inspect the target Blueprint function graph before mutating it. The inspection SHALL distinguish a normal function graph from a corrupt function graph, including at least missing `UK2Node_FunctionEntry`, invalid graph ownership, null or invalid nodes, duplicate entry nodes, or a graph that is not present in the target Blueprint `FunctionGraphs` list.

#### Scenario: Normal function graph uses normal deletion path
- **WHEN** `blueprint(action="delete_function")` targets a valid Blueprint function graph with a function entry root node
- **THEN** UECP MUST use the normal deletion path
- **AND** the response MUST identify the deletion path as normal
- **AND** the function MUST no longer appear in `get_blueprint_functions`

#### Scenario: Missing entry root is detected before deletion
- **WHEN** `blueprint(action="delete_function")` targets a function graph without a `UK2Node_FunctionEntry`
- **THEN** UECP MUST classify the graph as corrupt before calling the normal high-level graph removal path
- **AND** the response MUST include a corrupt reason for the missing entry root

#### Scenario: Force does not bypass corrupt graph classification
- **WHEN** `blueprint(action="delete_function", force=true)` targets a corrupt function graph
- **THEN** UECP MUST still use the corrupt graph deletion handling
- **AND** `force=true` MUST only bypass the non-trivial-node refusal for normal function graphs

### Requirement: Corrupt function graph deletion uses fallback handling

UECP `delete_function` SHALL support a fallback deletion path for corrupt function graphs. The fallback path MUST remove the target function graph from the Blueprint function list without requiring the corrupt graph to compile, reconstruct, or pass normal function entry validation.

#### Scenario: Corrupt helper function is removed
- **WHEN** `blueprint(action="delete_function")` targets a half-created helper function graph that is missing its function entry root
- **THEN** UECP MUST remove that function from the Blueprint
- **AND** subsequent `get_blueprint_functions` MUST NOT list that function name
- **AND** UECP MUST keep the Blueprint asset loadable

#### Scenario: Fallback avoids unsafe high-level cleanup
- **WHEN** a function graph is classified as corrupt
- **THEN** UECP MUST NOT route the deletion through a high-level editor cleanup path that requires a valid function entry/root node
- **AND** UECP MUST mark the Blueprint modified or structurally modified only through a path that does not hang on the corrupt graph

#### Scenario: Fallback reports follow-up warnings
- **WHEN** corrupt graph fallback deletion succeeds
- **THEN** the response MUST include a warning that the Blueprint may still require validation or compile because unrelated graph errors may remain
- **AND** the response MUST NOT claim the whole Blueprint compiled successfully unless compile was actually run and succeeded

### Requirement: Delete function responses are structured and actionable

UECP `delete_function` SHALL return structured JSON for single and batch calls. The result SHALL include the function name, deletion status, deletion path, whether corrupt fallback was used, corrupt reasons when present, and warnings when follow-up validation is recommended.

#### Scenario: Single delete reports deletion path
- **WHEN** a single `delete_function` call succeeds
- **THEN** the JSON response MUST include `success`, `deleted`, `function_name`, `deletion_path`, `corrupt`, and `warnings`

#### Scenario: Batch delete reports per-item result
- **WHEN** `delete_functions` deletes multiple functions and one item is corrupt
- **THEN** each batch item MUST report its own function name, success/failure, deletion path, and corrupt reasons
- **AND** one corrupt item MUST NOT erase diagnostic detail for other items

#### Scenario: Failed delete preserves diagnostic detail
- **WHEN** `delete_function` cannot safely delete the requested function
- **THEN** the response MUST include a clear error
- **AND** if graph health inspection ran, the response MUST include the detected health state or corrupt reasons

### Requirement: UECP endpoint survives delete function timeout

UECP SHALL bound waiting for game-thread tool execution so a single `delete_function` call cannot indefinitely block the MCP/UECP server worker. If a timeout occurs, UECP MUST return a timeout response that states whether the operation may still complete, and the endpoint MUST remain able to answer at least health/read-only requests when the editor game thread is not permanently wedged.

#### Scenario: Delete function handler exceeds timeout
- **WHEN** `delete_function` execution exceeds the configured UECP tool timeout
- **THEN** UECP MUST return a timeout error instead of waiting forever
- **AND** the response MUST indicate the operation state is unknown if the game-thread task had already been dispatched

#### Scenario: Endpoint responds after tool timeout
- **WHEN** a `delete_function` call times out but the Unreal Editor process remains alive and the game thread is not permanently blocked
- **THEN** a subsequent read-only UECP request such as `get_blueprint_functions` or `ping` MUST receive a response

#### Scenario: Destructive retry is guarded after uncertain timeout
- **WHEN** a destructive tool times out with operation state unknown
- **THEN** UECP MUST prevent immediate unsafe re-entry into the same destructive operation until the prior task is known complete or the subsystem reports a safe state

### Requirement: Blueprint graph mutation is bounded and root-safe

UECP SHALL apply bounded game-thread waiting and busy/unknown retry protection to Blueprint graph mutating tools, including graph build, clear, node delete/remove, pin connect/disconnect/default edits, and Blueprint compile. Function graph clear and node deletion SHALL preserve function root/terminator nodes.

#### Scenario: Build graph timeout does not allow immediate mutation retry
- **WHEN** `build_blueprint_graph` exceeds the configured UECP tool timeout
- **THEN** UECP MUST return a timeout response with `operation_may_still_complete=true`
- **AND** a subsequent Blueprint mutating request MUST be rejected as busy until the prior operation is known complete

#### Scenario: Function graph clear preserves entry and return nodes
- **WHEN** `clear_before_build` or `clear_blueprint_graph` targets a valid function graph
- **THEN** UECP MUST NOT delete `UK2Node_FunctionEntry`
- **AND** UECP MUST NOT delete `UK2Node_FunctionResult`

#### Scenario: Corrupt function graph clear is refused
- **WHEN** `clear_before_build` or `clear_blueprint_graph` targets a function graph with zero or multiple `UK2Node_FunctionEntry` nodes
- **THEN** UECP MUST refuse to clear or rebuild that graph
- **AND** the response MUST identify the graph as corrupt and recommend `delete_function` fallback followed by recreation

#### Scenario: Delete node refuses protected function roots
- **WHEN** `delete_nodes`, `remove_node`, or `remove_nodes` targets a function entry or function result node
- **THEN** UECP MUST refuse that node deletion
- **AND** the response MUST report the protected node block instead of deleting the root/terminator

#### Scenario: Ping does not require game thread dispatch
- **WHEN** the UECP bridge receives `ping` or `health`
- **THEN** it SHOULD respond without dispatching a task to the Unreal game thread
- **AND** the response SHOULD distinguish bridge availability from Blueprint graph task completion

### Requirement: Regression coverage includes corrupt Blueprint function graphs

The implementation SHALL include regression coverage for corrupt Blueprint function graph deletion. The coverage SHALL create or use a Blueprint function graph missing its function entry root node, delete it through UECP, and verify the endpoint remains usable afterward.

#### Scenario: Missing root graph smoke passes
- **WHEN** the regression smoke creates a Blueprint function graph without a function entry root node and calls `delete_function`
- **THEN** the function MUST be removed
- **AND** the Blueprint MUST remain loadable
- **AND** a follow-up UECP read request MUST succeed

#### Scenario: Existing BP_PlayerController failure mode is represented
- **WHEN** the regression coverage models the observed `BP_PlayerController` failure mode with a half-created helper or priority function graph
- **THEN** `delete_function` MUST not hang the UECP endpoint
- **AND** the response MUST identify whether normal or corrupt fallback deletion was used
