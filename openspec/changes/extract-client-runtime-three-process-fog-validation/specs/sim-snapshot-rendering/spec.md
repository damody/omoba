## MODIFIED Requirements

### Requirement: `SimWorldSnapshot` structure 與 read-only-except-queues invariant

`omoba-core::runtime::SimWorldSnapshot` SHALL 包含presentation projection所需的render-facing state，包括tick、entities、paths、removed entity ids、round data、lives、blocked regions、explosions、ability definitions、tower templates與tower upgrade definitions。Snapshot entity SHALL 保留optional hero extension、optional tower upgrade levels與render-safe fixed-point conversion。

Secure fog模式中，只有`omoba-client-runtime`可從filtered Specs world擷取此state並轉成版本化presentation protobuf；omfx SHALL consume presentation contract，而不是直接讀取`SimWorldSnapshot`、Specs storage或`omobab`。Extraction MUST 只看見該team已揭露的entity與合法remembered資料，且 MUST NOT 以canonical entity ID對renderer識別entity。

`extract_snapshot` SHALL 將sim ECS world視為read-only，唯一例外是以`std::mem::take(&mut q.pending)`drain既有producer-consumer render queues。它 MUST NOT write gameplay components、create entities、delete entities或修改其他resources。Boundary values SHALL 使用project fixed-point helper轉成render `f32`。

#### Scenario: Secure snapshot來源是filtered runtime
- **WHEN** Team 1 runtime在tick N發布presentation
- **THEN** entity state只來自Team 1 filtered world及sanitized remembered cache
- **AND** omfx不直接lock或讀取simulation snapshot

#### Scenario: Extraction維持read-only invariant
- **WHEN** 擷取filtered snapshot與presentation
- **THEN** 唯一simulation writes是`RemovedEntitiesQueue`、`ExplosionFxQueue`、`TowerFireFxQueue`與`AttackPhaseFxQueue`的`mem::take` drains
- **AND** extraction不建立、刪除或修改gameplay component

#### Scenario: omoba-sim determinism tests維持通過
- **WHEN** 執行`cargo test --manifest-path D:/code/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** determinism suite與pin-hash tests通過

#### Scenario: EntityRemoved在同tick boundary刪除
- **WHEN** system push `Outcome::EntityRemoved { entity: e }`
- **THEN** `process_outcomes`將ID寫入`RemovedEntitiesQueue`並刪除entity
- **AND** `world.maintain()`後entity不再alive且state hash不再包含它

## ADDED Requirements

### Requirement: Fog transition同步清理render state

Reveal SHALL 在effective tick建立目前baseline；Forget SHALL 同tick移除Specs entity、target lookup與render entity，舊replica ID永久失效；LastKnown SHALL 只產生不參與Specs、collision、targeting、scripts或team hash的sanitized ghost。Hidden entity後續移動或死亡 MUST NOT 更新ghost。

#### Scenario: Forget後畫面與world同時移除
- **WHEN** 敵方單位在tick N套用Forget
- **THEN** tick N presentation包含對應removed render ID
- **AND** filtered world與target lookup不再包含該entity

#### Scenario: LastKnown不洩漏霧中變化
- **WHEN** LastKnown敵人在霧中移動或死亡
- **THEN** renderer仍只看到離開視野時的sanitized ghost
- **AND** ghost不影響replica hash
