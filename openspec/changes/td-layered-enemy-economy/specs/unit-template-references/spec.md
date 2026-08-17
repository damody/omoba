## ADDED Requirements

### Requirement: TD layer metadata 在 generated 與 runtime Lua 模式語意一致

`omoba-template-ids` SHALL 從 canonical Lua builders 產生 dependency-light TD layer catalog 與 lookup API。Explicit runtime Lua content mode SHALL 產生相同 layer ids、ordered children、HP、properties、damage masks、cash 與 leak values，並套用相同 graph validation。

#### Scenario: Generated 與 runtime Lua catalog parity
- **WHEN**同一份 `scripts/lua_data` 分別經 build-time codegen 與 runtime Lua loader 載入
- **THEN**TD layer catalog 的 stable serialization／digest 相同
- **AND**所有 round enemy references 解析到相同 archetype 與 modifier properties

#### Scenario: Invalid layer metadata 在兩條路徑都被拒絕
- **WHEN**Lua builder 宣告 cyclic child graph 或 unknown damage bit
- **THEN**build-time codegen 與 runtime Lua validation 都在 gameplay 前失敗
- **AND**兩者 error 都包含 offending layer id

## MODIFIED Requirements

### Requirement: Creep templates are the canonical unit definitions

Generated Rust template data SHALL contain the authoritative creep definition for every creep id referenced by shipped story maps. Each active non-TD creep template SHALL define `id`, `display_name`, `hp`, `armor`, `magic_resistance`, `damage`, `attack_range`, `move_speed`, `enemy_type`, `ai_type`, `exp_reward`, and `gold_reward`.

Each generated TD round enemy SHALL resolve to authoritative TD layer metadata containing stable layer id, display label, current-layer HP, move speed, ordered children, layer cash, remaining leak value, property flags, and damage compatibility mask. TD variants MAY be derived from a canonical base layer plus validated Camo／Regrow／Fortified modifiers and MUST NOT require duplicated full creep templates for every modifier combination.

Story maps SHALL NOT override these fields with map-local creep stats, layer graphs, damage masks, cash, leak values, properties, or labels.

#### Scenario: Every shipped map creep resolves to a generated template

- **WHEN** all generated map data is scanned for `Creep[].Name` values
- **THEN** every non-TD value resolves through `omoba_template_ids::creep_by_name`
- **AND** every resolved non-TD id has `omoba_template_ids::creep_stats(id)` data
- **AND** every generated TD round enemy resolves through the TD layer lookup API
- **AND** every resolved TD archetype has validated layer, property, cash, leak, and damage compatibility metadata

#### Scenario: TD_STRESS uses the generated catalog value

- **WHEN** the `td_stress` creep template is loaded from generated Rust data
- **THEN** its `display_name` is `壓測怪`
- **AND** its `hp` is `10000.0`
- **AND** its `move_speed` is `100.0`
- **AND** it remains a non-layered stress template unless its source explicitly declares TD layer metadata

#### Scenario: TD modifier variant avoids duplicated flattened stats

- **WHEN**codegen resolves a Camo Regrow variant used by a TD round
- **THEN**variant references the canonical base layer and validated modifiers
- **AND**runtime receives Camo／Regrow state without synthesizing a flattened effective-HP creep template

### Requirement: Creep emitters resolve stats from generated templates

`omoba-core` runtime initialization SHALL build non-TD `CreepEmiter` values by resolving template ids through `omoba_template_ids` display and stats APIs. The non-TD emitter's label, HP, max HP, move speed, physical defense, and magic defense SHALL come from resolved generated template data.

TD round initialization SHALL resolve every enemy through the generated TD layer lookup API and SHALL attach complete `TdLayerState`／layer metadata when spawning. It MUST NOT construct `td_btd_*` emitters by flattening effective HP or discard Camo／Regrow／Fortified properties. Missing or stat-less non-TD templates and missing or invalid TD layer references MUST fail before gameplay with an error that includes the referenced id.

#### Scenario: TD_STRESS emitter uses generated template stats

- **WHEN** `TD_STRESS` initializes creep emitters from generated story data
- **THEN** the `td_stress` emitter has label `壓測怪`
- **AND** its HP and max HP are built from generated template `hp = 10000.0`
- **AND** its move speed is built from generated template `move_speed = 100.0`

#### Scenario: Missing template reference fails clearly

- **WHEN** a map declares `Creep[].Name: "missing_creep_template"`
- **THEN** story initialization fails before gameplay starts
- **AND** the error message includes `missing_creep_template`

#### Scenario: TD enemy retains generated properties

- **WHEN**round initialization spawns a Camo Regrow Fortified TD archetype
- **THEN**entity current-layer HP 與 move speed 來自 generated TD layer metadata
- **AND**`TdLayerState` 同時保留 Camo、Regrow 與合法 Fortified state
- **AND**runtime 不會以 `effective_hp` 建立替代 emitter
