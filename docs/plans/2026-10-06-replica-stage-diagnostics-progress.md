# 2026-10-06 client step 分段診斷

root HEAD 仍是 `c5211e8311cdca867841fcc1494899634b9d17b1` / `master`。沒有 commit、push、rebase、reset、clean。既有 dirty / untracked 未還原。第 5 批指令沒有重跑。沒有對局、full suite、release、DLL stage、UE 或 simulation。

效能沒有修復。先前 52.654 / 51.626 ms 對 50 ms 上限的失敗原因仍然未知。本批只加可選 wall-clock 分段，不改成功條件、不扣未知成本、不擴大門檻、不挑樣本。`OM_PERF` / `OM_PERF_ENABLE` 與既有 12 門檻沒有改。`omoba-core/src/comp/perf_window.rs` 與 `scripts/moba_performance_report.lua` 未編輯。6.5 未勾，留給 Codex 獨立審查。error register 未改；若審查要記，由 primary 整合 E322。

## 行為

具名、不重疊的 phase：

1. `decode`
2. `staging`：preflight、pre-step、injection、baseline staging
3. `fixed_step`
4. `pre_repair`：restore、allowlist、pre-repair hash
5. `post_step_repair`
6. `post_repair_hash`
7. `host_finalize`：ReplicaHost fog、SHA、report、expected sequence

進入 staging 前的 sequence / tick / epoch 檢查，以及 post-repair hash 之後的 expected tick / sequence 更新，不另命名，留在 `residual_ns`。residual 是未歸屬 wall time，不是 CPU 原因。

未量測 caller 走原來的 `apply_encoded_frame` / `apply_frame`，使用 `NoopReplicaStageClock`，不呼叫 `Instant::now`。量測路徑才讀時鐘。時間不進 world resource、canonical hash、wire、ABI 或 catalog。操作順序、結果、hash 與錯誤型別保持原樣。

`main` 的 outer timer 仍包住整次 `apply_encoded_frame_profiled`，停表後才組 sample 與格式化。只有 `Applied` 進窗口。duplicate 丟棄 profile 且不採樣。stalled / error 在 host 清掉 profile，`?` 在採樣前返回，不洩漏上一筆。每 60 筆成功 sample 打一行獨立 `OM_REPLICA_STAGE`。窗口只留 outer 最慢的那一筆（平手留第一次）的 tick、sequence、全部 phase、outer、residual；不是各 phase 各自最大值。phase sum 大於 outer，或 residual 與 outer 對不上，拒絕且不推進窗口。

phase clock 讀取發生在既有 outer timer 裡面，所以正式 `OM_PERF` 的 `replica_step` 會含這幾次 `Instant::now`。格式化與 log 在停表之後，沒有從樣本扣除。

## API

- `SelectiveReplicaRuntime::apply_encoded_frame`：簽章與結果不變。
- `SelectiveReplicaRuntime::apply_encoded_frame_profiled`：同樣的 `Result<FrameApplyResult, ReplicaRuntimeError>`。只有 `Applied` 留下 profile。
- `SelectiveReplicaRuntime::take_stage_profile`：取走並清空。
- `ReplicaHost::apply_encoded_frame`：簽章不變，不讀時鐘。
- `ReplicaHost::apply_encoded_frame_profiled`：同樣的 `Result<Option<ReplicaApplyReport>, ClientRuntimeError>`，成功時把 `host_finalize` 寫進同一筆 durations。
- `ReplicaHost::take_stage_profile`
- `commit_stage_profile(measured, applied, core, host_finalize_ns)`：非量測或非 Applied 一律 `None`。
- `ReplicaStageWindow::new(player, team)`：容量 60。`record` 在第 60 筆回 `Ok(Some(line))`。
- `ReplicaStageSample::from_durations`：phase sum 超過 outer 則 `Err`。
- log 前綴 `OM_REPLICA_STAGE`，`v=1`，`unit=ns`，`clock=wall`，`scope=apply_encoded_frame`，含 player、team、samples、該筆 tick / sequence、outer、residual、七個 `*_ns`。

## 檔案

- 新增 `omoba-core/src/comp/replica_stage.rs`
- `omoba-core/src/comp/mod.rs`：只加 `pub mod replica_stage`
- `omoba-core/src/runtime/selective_replica.rs`
- `omoba-client-runtime/src/replica_host.rs`
- `omoba-client-runtime/src/main.rs`：`apply_ready_frame` 成功路徑
- 本檔

## 驗證

filter：`replica_stage`。下列是去掉本批 `unused_mut` 之後的最終重跑；第一次同樣 6 與 1 通過，當時多了 6 個 unused_mut，已刪除。

```
cargo test --manifest-path omoba-core/Cargo.toml replica_stage
```

exit 0。lib 6 passed，0 failed，475 filtered out：

- `comp::replica_stage::tests::replica_stage_window_is_bounded_to_one_slowest_sample`
- `comp::replica_stage::tests::replica_stage_closed_line_uses_every_phase_of_the_same_outer_peak`
- `comp::replica_stage::tests::replica_stage_tie_keeps_the_first_sample_and_reset_drops_it`
- `comp::replica_stage::tests::replica_stage_invalid_phase_sum_is_rejected_without_advancing`
- `runtime::selective_replica::replica_stage_tests::replica_stage_measured_and_unmeasured_apply_share_result_and_hash`
- `runtime::selective_replica::replica_stage_tests::replica_stage_duplicate_stall_and_error_drop_stale_profile`（`NoopDisclosedWorldStepper`，未載 DLL）

同一指令還執行 `tests/td_autoplay_100.rs`：0 passed，3 filtered out。那是 filter 沒有命中該 integration harness，不是本批測試為空。lib 的 6 筆是實際測試。

```
cargo test --manifest-path omoba-client-runtime/Cargo.toml replica_stage --lib
```

exit 0。1 passed，0 failed，87 filtered out：

- `replica_host::map_contract_tests::replica_stage_host_finalize_keeps_only_an_applied_profile`

此測試不 bootstrap、不載 script DLL。duplicate / error 的 Noop 行為在 core 那一筆。

```
cargo check --manifest-path omoba-client-runtime/Cargo.toml --bin omoba-client-runtime --features compiled-content-only
```

exit 0。`Finished dev profile`。

警告只有既有 `omoba-template-ids` build script：`validate_layer_catalog`、`REGROW`、`FORTIFIED` unused。不是本批檔案。git 對 `main.rs`、`replica_host.rs`、`selective_replica.rs` 顯示 working-copy LF→CRLF 提示，沒有改行尾。

## 剩餘

- 52 ms 失敗原因未知，不能說已修復，也不能勾 6.5。
- 沒有實機 log，尚未看到真實 `OM_REPLICA_STAGE` 行。
- `OM_PERF` 門檻與定義沒變；profiled apply 的時鐘讀取會計入既有 outer，沒有扣除。
- Primary 已完成實際diff審查及獨立core6/runtime1/compiled-only binary check，均exit0；接受本批診斷功能，不接受效能修復聲明。既有三個td_rounds dead-code警告保留。沒有真實對局採樣，沒有改原十二門檻。
