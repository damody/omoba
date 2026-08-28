## MODIFIED Requirements

### Requirement: `omoba-core::runtime` provides the mandatory local lockstep replica boundary

前後端共用的deterministic simulation primitives SHALL 位於`omoba-core::runtime`，且 SHALL 是mandatory public contract。`omb` SHALL 使用它執行authoritative simulation與兩隊server-local observer；`omoba-client-runtime` SHALL 使用它執行filtered replica。Secure fog模式的native `omfx` SHALL NOT 建立或step Specs world、載入script DLL或直接消費此gameplay runtime，只能透過presentation IPC取得render-safe資料。Dependency direction SHALL 是`omb -> omoba-core`、`omoba-client-runtime -> omoba-core`及`omfx -> shared presentation schema`，不得存在`omfx -> omb`。

#### Scenario: Server與external runtime共用runtime entrypoints
- **WHEN** 檢查authoritative、observer與external client runtime implementation
- **THEN** world initialization、tick phases、script dispatch、outcome processing與hash使用`omoba-core::runtime`共用entrypoints
- **AND** secure renderer不呼叫這些gameplay entrypoints

#### Scenario: Renderer-only omfx沒有replica ownership
- **WHEN** 以secure renderer-only mode建置及啟動omfx
- **THEN** omfx不建立`SelectiveReplicaRuntime`或`SpecsDisclosedWorldStepper`
- **AND** omfx不載入`base_content.dll`或連線authoritative KCP
- **AND** 不新增`omfx -> omb`dependency edge

## ADDED Requirements

### Requirement: Secure replica allowlist只有一個production來源

`omoba-core` SHALL 提供唯一component與resource allowlist API；server projector、server observer及兩個external runtime MUST 呼叫該API。Production consumer MUST NOT 維護局部schema ID set。

#### Scenario: Source guard阻止重複allowlist
- **WHEN** source guard掃描projector、observer、client runtime與secure renderer
- **THEN** 只存在`omoba-core`共用allowlist定義
- **AND** 所有consumer對相同schema集合的contract test通過
