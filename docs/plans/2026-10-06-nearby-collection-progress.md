# nearby collection 分配收斂（iteration 11）

## 這次只證明什麼

四條 `par_join` 先前對每一列配置 `vec![ent]` 與 `vec![*pos]`，再 append 進 worker accumulator。`collect_entity_pos_rows` 改為把 `(Entity, Pos)` 直接 push 進 per-worker `Vec`，`sorted_f32_index_items` 仍先串 Unit 再串 Creep（英雄／塔各一組），用 `xy_f32`，再 `sort_by_key((id, generation))` 後 `rebuild_from`。塔的雙重 `is_dirty`、Rayon、profiling span 名稱與 multiset（同一實體可同時在 Unit 與 Creep）沒有改。

這只移除靜態可見的 O(entity-count) 暫時一元素 `Vec`。上一輪實際 60Hz 配對跑、每邊 50 windows／3000 steps：p1 outer 93.7459ms／dispatcher 93.0694ms；p2 outer 102.1183ms／dispatcher 101.7626ms；ProcessCPU delta valid=0。該證據只說明牆鐘落在 dispatcher bucket，不證明這些配置造成尖峰，也不構成 50ms 通過或根因結論。沒有改 `dispatch`／`dispatch_seq`、worker 數、優先權、門檻、診斷或 hash gate。

## 檔案

- `omoba-core/src/runtime/native/tick/nearby_tick.rs`（唯一程式變更；`git diff --stat`：144 insertions, 117 deletions）
- 本檔

## 測試

```bat
cargo test --manifest-path omoba-core/Cargo.toml nearby_collection --lib
```

實際選中 3，不是 0。`495 filtered out`。3 passed，0 failed，exit 0，約 0.00s。名稱：

- `runtime::native::tick::nearby_tick::nearby_collection::empty_rows_stay_empty_through_sort_and_index`
- `runtime::native::tick::nearby_tick::nearby_collection::parallel_many_rows_match_serial_pairs_after_same_sort`
- `runtime::native::tick::nearby_tick::nearby_collection::duplicate_generation_order_is_kept_in_creep_index_count`

編譯警告不在本檔。`omoba-template-ids` build script 既有 3 則 dead_code：`validate_layer_catalog`、`REGROW`、`FORTIFIED`。`omoba-core` 這次 test 編譯沒有再印 warning。未跑遊戲、模擬或 Unreal。

## 下一個排程邊界（只讀，未改這些檔）

`dispatcher_ns` 是 `DeterministicGameplayPhase::Dispatcher`。`filtered_specs.rs` 的 `fixed_step_impl` 用同一個 `Instant` 包住整段 `SystemDispatcher::run_systems`。

`run_systems` 先在呼叫緒 `run_now::<player_input_tick::Sys>`，再 `Dispatcher::dispatch`。本機 shred 0.16.1（`omoba-core/Cargo.lock`）`Dispatcher::dispatch` 呼叫 `SendDispatcher::dispatch`，parallel feature 下進入 `dispatch_par`：`ThreadPool::install` 擋住呼叫緒，裡面是

```rust
for stage in stages {
    stage.execute(world);
}
```

`Stage::execute` 是 `groups.par_iter_mut().for_each`，group 之間平行、group 內 `run_now` 依序，`for_each` 返回前 join 完全部 group。下一個 stage 不能提前開始。

`Job::run`（`omoba-core/src/comp/ecs.rs`）只量 `T::run` 前後，寫入 `TickProfile::record_system`。它不含 `install` 裡 stage 迴圈的 join 等待。系統時間加總因此不是 dispatcher 牆鐘。

靜態讀 `StagesBuilder`（未 dump 執行期 stage 表）：預設 `RunningTime::Average`。`improves_balance` 在該 stage 只有一個衝突 group 時，`max == old`，再加 Average 不會更平衡，`insertion_target` 落到 `NewStage`。

- `nearby` 寫 `Tower` 與 `Searcher`，讀 `Pos`
- `player` 同樣寫 `Tower` 與 `Searcher`，讀 `Pos`，資源衝突，獨立 stage
- `demo_patrol` 寫 `Pos`，與前兩者衝突，獨立 stage
- `projectile` 依賴 `nearby_sys`、`player_sys`、`demo_patrol_sys`。`find_conflict` 在依賴尚未落在目前或更早 stage、且剩餘依賴多於一個時回 `Conflict::Multiple`。它也寫 `Pos`，不會因 balance 併進 `demo_patrol` 的 group

nearby 自己的 `par_join` 返回之後，下一個具體阻塞點就是這個 stage join：nearby 的 stage 結束後才跑 player 的 stage；projectile 的 stage 要等 nearby、player、demo_patrol 三段 join 都返回。這三段仍在同一個 `dispatcher_ns` 牆鐘裡。這是排程邊界，不是 93ms 根因。ProcessCPU delta valid=0 也沒有把這段牆鐘分成 CPU 或等待；本地 `process_cpu_delta` 缺樣本時是 `None`（不是用 0 表示等待），`OM_FIXED_STEP` 的 `cpu_not` 標成 `wait_time_or_phase_attribution`。
