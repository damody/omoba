## Context

目前 `run_script_dispatch()` 會先收集所有 `ScriptUnitTag` entity，再用單一 `WorldAdapter` 串行呼叫每個 script 的 `UnitScript::on_tick`。`WorldAdapter` 持有大量 `WriteStorage` / `Write<Resource>`，script API 例如 `set_pos`、`set_facing`、`set_asd_count`、`spawn_projectile_ex`、`deal_damage_splash` 會即時修改 ECS 或 queue。

這個設計簡單且 deterministic，但在 TD_STRESS 1000 座 scripted tower 同步進入 ready/impact 的 tick，`script_on_tick` 會形成單一核心瓶頸。要平行化，核心限制不是 Rayon 本身，而是 script 執行期間的 shared mutable ECS borrow 與副作用排序。

## Goals / Non-Goals

**Goals:**

- 讓 `UnitScript::on_tick` 的主要 target selection、attack phase progression、script decision compute 可以平行執行。
- 將 script 造成的 ECS mutation 改成 deterministic `Outcome` / `ScriptOutcome`，由固定順序 apply 階段寫回。
- 保持 authoritative backend 與 omfx local sim_runner 在同一 TickBatch 序列下 deterministic。
- 保持現有 `base_content` script 高層邏輯盡量不變，避免一次重寫所有 content。
- 增加 profiling，能看出 parallel compute、outcome apply、各 script id 的耗時與 ready count。

**Non-Goals:**

- 不在本 change 解決所有 script event hook 的平行化；`on_spawn`、`on_attack_hit`、`death` 等事件可先維持 deterministic serial drain。
- 不改變 lockstep protocol、TickBatch 格式或 state hash protocol。
- 不以犧牲 determinism 的方式使用 wall-clock RNG、unordered HashMap iteration 或非固定 merge order。
- 不把所有 ECS systems 改為 outcome-driven；範圍限於 script `on_tick` 及其必要副作用。

## Decisions

### Decision: 以 read-only view + command buffer 平行執行 `on_tick`

`ParallelWorldAdapter` 提供與 `GameWorld` 相容的 host implementation，但在 `on_tick` 階段不直接寫 ECS。讀取 API 從 pre-borrowed `ReadStorage` / immutable resources 取得資料；寫入 API 只 append 到 thread-local command buffer。

替代方案是直接對 ECS storage 加鎖讓 script 平行寫入，但會造成高 contention，且 mutation interleaving 會破壞 deterministic ordering。

### Decision: command merge 使用 stable entity order

`run_script_dispatch_parallel_on_tick()` 先用 deterministic order 建立 tagged list，例如依 `Entity::id()` / generation / 原本 join 輸出排序。每個 entity 的 script command buffer 平行產生後，依 tagged list 順序 merge 成全域 `Vec<Outcome>`。

替代方案是每個 Rayon task 直接 push 到 shared `Mutex<Vec<_>>`，但 push 順序取決於 thread scheduling，不能用於 lockstep。

### Decision: script mutation 以新增 outcome 覆蓋，而不是在 script 內 `World` 即時 mutation

新增或擴充 outcome 類型，涵蓋 `ScriptSetPos`、`ScriptSetFacing`、`ScriptSetAsdCount`、`ScriptSpawnProjectile`、`ScriptDealDamage`、`ScriptDealDamageSplash`、`ScriptEmitAttackPhaseFx`、`ScriptEmitTowerFireFx`、`ScriptEmitExplosionFx`、`ScriptBuffMutation` 等必要副作用。Apply 階段集中在 `process_outcomes()` 或 script-specific apply helper 中，維持單執行緒 deterministic 寫回。

替代方案是讓 script 回傳 domain-specific high-level action，例如 `TowerAttackIntent`，但現有 script API 已包含多種通用 mutation；一次改成 intent model 會讓 content rewrite 太大。

### Decision: 保留 attack phase 語意，將 `asd_count` 寫回延後到 outcome apply

Scripted tower 的 `advance_attack_phase()` 目前會透過 `w.set_asd_count()` 即時更新 cooldown。平行版本中，script 讀到的是 tick 開始時的 `asd_count`，更新結果以 outcome 記錄，並於同 tick 的 post-script apply 寫回。由於每個 `on_tick` 只處理自己的 entity，正常不會有多 script 同時寫同一 tower cooldown。

替代方案是將 attack phase 完全移回 host system，但 scripted tower 仍有自訂 target、projectile、splash、flag 邏輯；短期會造成雙軌行為。

### Decision: RNG 以 entity/tick/op deterministic seed 提供

Parallel adapter 不使用共享 mutable RNG。`rand_unit()` 應基於 `MasterSeed`、`Tick`、entity handle、script-local op counter 或固定 op kind 產生 deterministic value，避免 thread 執行順序影響 RNG stream。

替代方案是延用單一 `Pcg64Mcg` 並加鎖，但 lock order 會跟 thread scheduling 綁定，不適合 deterministic simulation。

## Risks / Trade-offs

- [Risk] outcome apply 變大，可能把成本從 `script_on_tick` 移到 serial apply。→ Mitigation：先量測 `script_compute` 與 `script_outcome_apply`，並將可批次處理的 outcome 合併 apply。
- [Risk] script 讀取 tick-start snapshot，無法看到同一 tick 前一個 script 的即時 mutation。→ Mitigation：定義 `on_tick` 平行模式的語意為同 tick read snapshot；必要的跨 entity 互動透過下一個 deterministic apply boundary 生效。
- [Risk] 現有 script 依賴即時 `set_asd_count` 後再 `get_asd_count`。→ Mitigation：command buffer 提供 per-entity overlay，讓同一 script invocation 內可以 read-your-writes，但不暴露給其他 script。
- [Risk] script ABI 變更會要求 host 與 DLL 同步重建。→ Mitigation：優先只替換 host-side adapter 行為，不改 trait method；若必須新增 ABI 型別，納入 scripts workspace 與 omb workspace 雙 build 驗證。
- [Risk] omb 與 omoba-core mirror implementation 漏改。→ Mitigation：以 `omoba-core` 為 source of truth，`omb/src/scripting/*` 同步最小修改並跑 `cargo check --manifest-path omb/Cargo.toml -p omobab`。

## Migration Plan

- Phase 1：加入 script outcome 類型與 serial adapter command buffer，先保持 `on_tick` 串行但不即時寫 ECS，驗證行為等價。
- Phase 2：加入 read-only parallel adapter 與 deterministic merge，啟用 `on_tick` 平行 compute。
- Phase 3：擴充 profiling 與 tests，確認 TD_STRESS 下 `script_on_tick` spike 下降，且 `script_outcome_apply` 沒形成新瓶頸。
- Rollback：保留 feature flag 或 local constant 可切回 serial `WorldAdapter` path，方便比對 state hash 與 bisect。

## Open Questions

- `on_attack_hit` 等 event hooks 是否要在後續 change 套用同樣 command-buffer 模式。
- `deal_damage_splash` 的 target set 是否應在 compute 階段決定，或 apply 階段依 apply-time snapshot 查詢；為 determinism 與可推理性，初步偏向 compute 階段決定 target list 並寫入 outcome。
