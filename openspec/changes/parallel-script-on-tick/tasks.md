## 1. Baseline 與保護網

- [ ] 1.1 補上目前 serial `run_script_dispatch()` 的 behavior tests，涵蓋 `set_asd_count`、`spawn_projectile_ex`、`deal_damage_splash`、`set_facing` 與 FX queue 的結果。
- [ ] 1.2 加入 deterministic replay 測試，固定 master seed 與同一批 TickBatch，確認重跑兩次 state hash / outcome order 一致。
- [x] 1.3 在 `TickProfile` 或 sim_runner profile 中拆出 `script_compute`、`script_outcome_apply`、ready `tagged_count` 與 per-script-id timing 欄位。

## 2. Script Outcome Model

- [x] 2.1 在 `omoba-core/src/runtime/native/comp/outcome.rs` 新增或擴充 script mutation outcome，覆蓋 pos、facing、asd_count、projectile spawn、damage、splash damage、buff/stat mutation 與 render-only FX/cue。
- [x] 2.2 在 `process_outcomes()` 新增 deterministic apply handlers，集中寫回 ECS component storage、resource queue 與 LazyUpdate entity spawn。
- [x] 2.3 同步 `omb/src/comp/game_processor.rs` 的 outcome kind / apply mirror，避免 backend 與 core runtime 分歧。
- [x] 2.4 為 script outcome apply 補上單元測試，確認 apply 順序固定且結果與既有即時 mutation 語意一致。

## 3. Command Buffer Adapter

- [x] 3.1 建立 serial command-buffer `GameWorld` adapter，先讓 `on_tick` 仍串行執行，但所有 mutation API 改為寫入 invocation-local command buffer。
- [x] 3.2 在 adapter 內實作 read-your-writes overlay，至少支援同一 invocation 內的 `set_asd_count` / `get_asd_count`、`set_pos` / `get_pos`、`set_facing` / `get_facing`。
- [x] 3.3 將 command buffer 轉成 ordered script outcomes，並在 `run_script_dispatch()` 後接到 existing outcome pipeline。
- [x] 3.4 驗證 `scripts/base_content/src/towers/*` 與 `summons/saika_gunner.rs` 不需改寫即可通過 serial command-buffer path。

## 4. Parallel on_tick Execution

- [x] 4.1 將 ready tagged entity list 排成 deterministic order，明確使用 stable entity identity，不依賴 worker completion order。
- [x] 4.2 建立 read-only parallel adapter 所需的 immutable storage/resource view，移除 `on_tick` compute 階段對 shared `WriteStorage` / `Write<Resource>` 的需求。
- [x] 4.3 使用 Rayon parallel iterator 執行 eligible `UnitScript::on_tick`，每個 invocation 產生自己的 command buffer 與 timing data。
- [x] 4.4 依 deterministic tagged list 順序 merge command buffers 到全域 outcome stream。
- [x] 4.5 保留 feature flag 或 local constant 切換 serial command-buffer path 與 parallel path，方便 regression bisect。

## 5. Deterministic RNG

- [x] 5.1 將 script `rand_unit()` 改為不依賴共享 mutable RNG call order，輸入包含 master seed、tick、entity identity 與 invocation-local op index。
- [x] 5.2 補測試確認不同 Rayon scheduling 下，同一 script invocation 的 RNG 結果一致。

## 6. Attack Phase 與 Scripted Tower 驗證

- [ ] 6.1 確認 scripted tower 的 `advance_attack_phase()`、`start_attack_windup()`、impact projectile/damage、`asd_count` 寫回都透過 outcomes 且 timing 不提前。
- [ ] 6.2 補測試確認 scripted tower 在 ready、charging、impact 三種狀態下 outcome 與 serial 版本等價。
- [ ] 6.3 確認 `AttackPhaseFxQueue`、`TowerFireFxQueue`、`ExplosionFxQueue` drain 仍是 render-only，不影響 gameplay hash。

## 7. Verification

- [x] 7.1 執行 `cargo check --manifest-path omoba-core/Cargo.toml`。
- [x] 7.2 執行 `cargo check --manifest-path scripts/Cargo.toml -p base_content`，確認 script ABI / content 編譯。
- [x] 7.3 執行 `cargo check --manifest-path omb/Cargo.toml -p omobab`。
- [x] 7.4 執行 `cargo check --manifest-path omfx/Cargo.toml -p executor`。
- [ ] 7.5 跑 TD_STRESS profiling，比對 `script_on_tick` / `script_compute` spike、`script_outcome_apply`、`sim_runner_profile queue_len/max_queue_len` 與 `omfx_frame_slo p99/max`。
