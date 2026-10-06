# 2026-10-06 移除未使用的逐 step 報告歷史

## Primary 接受紀錄

Grok `run-muwacisn-dq61gc` / thread `98965d83-a59a-428b-85fa-5d8b9be25f39` completed 1m46s，follower exit0。primary 已讀實際 diff、所有引用及 checkpoint 呼叫點，独立 runtime replica_stage 1 passed／87 filtered、compiled-only binary check exit0；接受這個限定刪除，不接受效能已修復。metrics input22186/cached1315840/full1338026/output4902/reasoning2308/total1342928，8 model calls／8 turns，reported USD0.24877936，API duration unavailable。下方「未改error/tasks/.ai-collab」指worker範圍；primary另整合E323與collector11/11。

root HEAD 仍是 `c5211e8311cdca867841fcc1494899634b9d17b1` / `master`。沒有 commit、push、rebase、reset、clean。既有 dirty / untracked 保留。沒有重做 replica stage 診斷，沒有對局、simulation、UE、release 或 DLL。

這只刪掉可證明沒有讀者的無界 `BTreeMap`。不宣稱 50 ms 尖峰根因，也不宣稱效能通過。`OM_PERF`、十二門檻、wire、ABI、內容與 hash 都沒改。error register / tasks / scripts / `.ai-collab` 未改；若要登記，由 primary 整合 E323。任務未勾。

## 引用搜尋

全庫 `pre_repair_reports` 只有三處，全在 `omoba-client-runtime/src/replica_host.rs`，刪除前：

- 欄位：`pre_repair_reports: BTreeMap<(u64, u64, u64), ReplicaApplyReport>`
- `bootstrap`：`pre_repair_reports: BTreeMap::new()`
- `Applied`：`self.pre_repair_reports.insert((replica_tick, team_sequence, authority_revision), report.clone())`

沒有讀取、清除、迭代或對外 getter。刪除後同一搜尋為 0。`BTreeMap` 在該檔沒有其他用途，所以 `use std::collections::BTreeMap` 一併刪除。

checkpoint / evidence 不讀這份 map：

- `main.rs` `apply_ready_frame` 使用 `apply_encoded_frame_profiled` 回傳的 `ReplicaApplyReport`。
- `evidence.record_checkpoint(&report)` 寫 tick、sequence、authority、pre/post hash。
- `checkpoint_queue.enqueue` 使用同一筆 report 的 pre/post/encoded SHA。

這些呼叫點本批沒有改。

## 本批刪除

相對上一批工作樹，`replica_host.rs` 只少了上述 import、欄位、初始值與 `insert` / `report.clone()`。仍保留：

- `ReplicaApplyReport` 與其六個欄位
- `Ok(Some(report))`，含 encoded SHA、pre/post hash、tick、sequence、authority
- `expected_team_sequence` 更新
- fog ingest
- stage profile / `host_finalize` 計時（不再把無界 insert 算進去）

沒有另設 history cap，也沒有把 checkpoint 改成默默丟棄。

## 驗證

```
cargo test --manifest-path omoba-client-runtime/Cargo.toml replica_stage --lib
```

exit 0。1 passed / 0 failed / 87 filtered out：

- `replica_host::map_contract_tests::replica_stage_host_finalize_keeps_only_an_applied_profile`

```
cargo check --manifest-path omoba-client-runtime/Cargo.toml --bin omoba-client-runtime --features compiled-content-only
```

exit 0。`Finished dev profile`。

警告只有既有 `omoba-template-ids` build script：`validate_layer_catalog`、`REGROW`、`FORTIFIED` unused。不是本批檔案。

## Coverage limitation

`replica_stage` 這筆測試只打 `commit_stage_profile`，不 `bootstrap`、不載 DLL、不呼叫 `apply_encoded_frame`。因此它證明 host 模組在刪除後仍可編譯，以及 profile 接口沒壞，但沒有直接執行報告生成或已刪除的 map。報告生成路徑要載 script DLL 才能 bootstrap `ReplicaHost`；依本批限制不為它新增玩法或公開 API。bin check 只證明 `main` 仍能接回傳的 report，不是實機 checkpoint。
