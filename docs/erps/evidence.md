# ERPS capability-to-test evidence

以下 42 個 OpenSpec scenario 均對應到可重跑的自動化測試或固定負載證據。測試名稱以 `cargo test --workspace --all-features` 的輸出為準。

## erps-party-ready-credit（9）

| Scenario | Evidence |
|---|---|
| CJK 房名通過驗證 | `party::tests::cjk_valid_special_invalid`、Windows/Linux C E2E 的 party rename |
| 特殊符號房名遭拒 | `party::tests::cjk_valid_special_invalid` |
| 過期 revision 不覆蓋 roster | `party::tests::revision_and_freeze_are_enforced` |
| 全員同意後進入 placement | `proposal::tests::all_accept_before_placement`、`grpc_e2e.rs` |
| 舊接受訊息不接受新場次 | `ready_check::deadline_is_exclusive_and_late_accept_is_stale` |
| 未回應者停止配對 | `grpc::runtime_timeout_penalizes_only_missing_player_and_requeues_innocent_at_original_age` 同時驗證雙方依序收到具原因／credit／eligibility 的 cancellation 與更新後 party state，讓 client UI 停止 proposal 倒數；Rust/C mapping tests 保留停權截止時間 |
| Launch failure 不扣信用分 | `credit::tests::infrastructure_never_penalizes`、placement retry test |
| Party 失敗成員阻止自動重排 | `ready_check::mixed_party_failure_preserves_unaffected_solo` |
| Grace period 內重連恢復狀態 | `grpc::runtime_disconnect_grace_preserves_then_cancels_queue`；ready-timeout lifecycle test 另驗證 `GetState` 還原 profile、queue mode、region、proposal ID 與 authoritative deadline |

## erps-matchmaking-core（8）

| Scenario | Evidence |
|---|---|
| RPC mutation 於 tick 邊界套用 | `grpc::bounded_actor_runs_commands`、`grpc::one_ecs_snapshot_commits_many_disjoint_proposals` |
| 相鄰 bucket 不重複使用 ticket | `claim_races::halo_duplicate_can_only_commit_once` |
| Worker 排程不改變結果 | `deterministic_replay::worker_counts_produce_same_candidates`、`grpc::identical_rpc_sequences_produce_identical_domain_ids` |
| 5v5 party 不拆分 | `mode_properties::modes_build_exact_rosters` |
| 八人 party 一起入場但各自成隊 | `mode_properties::ffa_never_emits_wrong_team_count` |
| 長時間等待擴大候選範圍 | `matching::bucket::tests::expansion_clamps` |
| Party 結構限制隨等待放寬 | `matching::dispatcher::older_candidate_can_outrank_soft_party_structure_penalty` |
| 八人同名次視為平手 | `rating::tests::ffa_ties_are_draws_and_clamped` |

## erps-load-validation（7）

| Scenario | Evidence |
|---|---|
| 隨機 party 組合不破壞 roster | `mode_properties.rs` property tests、`load_test::generated_party_limits_hold` |
| Cancel 與 candidate commit 競爭 | `claim_races::cancellation_wins_revalidation_race` |
| C client 收到 ready match | Windows clang 與 Linux gcc `tests/c_smoke/e2e.c`：`C_E2E_PASS` |
| 預設大規模測試完成 | `docs/erps/load-test.md` 固定 100,000-player PASS |
| gRPC 模式納入傳輸成本 | 1,000 clients／669 parties／225 matches 的 `transport.path=grpc-loopback` PASS；runner 逐場檢查 launch mode、無遺漏／重複／外來玩家、team shape、5v5 party 不拆與 MatchResult ACK，負向測試 `load_test::grpc_launch_checker_rejects_split_party_and_foreign_roster` 證明錯誤 roster 會讓 run 失敗；`grpc_transport_uses_heterogeneous_servers_in_every_region` 以 `tw`／`us`／`eu` 九台不同 capacity／cost／instance limit 的 server，證明三個 region 都由真實 control stream 承接 match |
| 超配立即使測試失敗 | `load_test::over_capacity_checker_returns_minimal_server_diagnostic`、`placement_properties.rs` |
| Baseline 比較具有環境資訊 | `load_test::baseline_comparison_rejects_different_environment_or_settings` 與 CLI `--output/--baseline` 實測 |

## erps-game-server-placement（6）

| Scenario | Evidence |
|---|---|
| 舊 heartbeat 不覆蓋新註冊 | `server::stale_generation_cannot_overwrite_or_heartbeat_replacement`；`server::newer_generation_cannot_discard_authoritative_instances` 驗證新 generation 不得以空 snapshot 清除既有 running／reserved ledger、容量與 failure score，也不得注入未知 instance |
| 失聯 server 不接新 match | `server::lost_running_instance_is_marked_and_never_presented_as_retryable` |
| 容量足夠但 instance 已滿 | `placement::instance_limit_and_release`、`placement_properties.rs` |
| 未接受 proposal 不占容量 | `proposal::all_accept_before_placement` |
| Accepted 尚未可連線 | `grpc_e2e.rs` 先送 Accepted、確認無事件，再送 Ready；`grpc::reported_instance_state_requires_the_ordered_lifecycle` 與 `grpc::authoritative_match_result_updates_elo_and_releases_reservation` 拒絕跳過 Accepted、缺少 endpoint/token 與倒退狀態，重複 Ready 保持冪等 |
| Reject 後容量完全歸還 | `placement::reject_releases_full_reservation_and_retries_another_server` |

額外的 `grpc::launch_uses_common_ticket_region_and_never_cross_region_server` 防止跨 region placement；`grpc::authoritative_match_result_updates_elo_and_releases_reservation` 同時驗證非 owner server 不得送 LaunchResult、重複 roster entry不得更新 Elo／釋放容量，且 instance 尚未進入 `Running` 時即使 roster 正確也不得提前結算或改動 rating／capacity；production TLS test 驗證未受 trusted server class 約束的 server 會被拒絕；`server::reconcile_preserves_authoritative_cost_and_rejects_unknown_instance` 驗證 reconnect ledger、禁止 snapshot 偽造 Finished 並保留容量，完成只能由 Running instance 經可靠的 MatchResult/ACK 流程結算。註冊與 reconcile 邊界也拒絕重複 instance ID，註冊容量以 checked addition 防止 overflow。omb adapter 在 Accepted/Ready 握手完成前遮蔽 heartbeat snapshot，並略過 Finished/ServerLost snapshot，避免與控制事件及 MatchResult 發生排序競態；整合 E2E 連續五次通過。

## erps-grpc-services（7）

| Scenario | Evidence |
|---|---|
| omb 透過服務註冊 | `omb/tests/erps_adapter.rs` |
| 重送 enqueue 不建立重複 ticket | `grpc::repeated_enqueue_request_id_returns_same_ticket_without_duplicate` |
| 重連後 reconcile instance | `server::reconcile_preserves_authoritative_cost_and_rejects_unknown_instance`、`server::same_generation_registration_is_idempotent_and_preserves_ledger`、`grpc::stale_control_generation_cannot_replace_sender_or_mutate_instance`；Ready/Running snapshot 強制攜帶非空 endpoint/token，control update 會同步兩欄，缺失時保持原權威狀態並拒絕，未知 match ID 或與 reservation 不同的 cost 也會明確失敗；match result 以 ACK、bounded idempotency cache 與跨重連 outbox 可靠重送；Rust event stream 使用 bounded exponential reconnect backoff |
| 不相容 major 被拒絕 | `grpc::api_major_is_rejected_and_minor_is_forward_compatible`、`grpc::admin_snapshot_rejects_incompatible_api_major`；matchmaking、game-server、admin 的所有 request envelope 均攜帶並驗證版本 |
| Production 拒絕未驗證身分 | `grpc::production_tls_handshake_and_token_identity_are_enforced`；同一測試以公開 Rust SDK、系統信任根設定與自訂 CA PEM 完成真實 TLS handshake，C ABI 及 `omb` 共用相同 trust policy；`config::tests::shipped_production_example_is_tls_only_and_has_trusted_server_classes` 直接解析隨附的 production example，確認禁用 plaintext、TLS certificate/key 成對，且三種異質 server class 的容量、1～100 instance 上限與各模式成本均合法 |
| 慢 client 不造成無界記憶體 | `grpc::slow_consumer_is_terminated_and_get_state_recovers_latest_state` |
| 關閉期間不接受新 ticket | `grpc::graceful_shutdown_transition_dynamically_rejects_new_mutations`；`grpc::grpc_health_reports_every_public_service` 透過真實 health RPC 驗證 Matchmaking、GameServer、Admin 都回報 SERVING，drain 僅先停止新 matchmaking，保留 game-server 結算 grace period |

## erps-client-sdks（5）

| Scenario | Evidence |
|---|---|
| Rust client 完成配對流程 | `grpc_e2e.rs`、`client::matched_event_preserves_roster`、`client::state_and_cancellation_preserve_player_facing_credit_details`、1,000-player gRPC run；MatchResult test 驗證結算後主動發布新 Elo/credit；`erps-test-fixture` 搭配公開 `party_to_match` example 實跑兩名 client enqueue／accept／相同 ready match，且範例不記錄 connection token |
| 遊戲主迴圈 poll ready proposal | Windows/Linux `e2e.c` 驗證公開 `ERPS_EVENT_*` 常數、queue mode、authoritative deadline 與 party member Elo/credit accessors；C bridge 對 Rust event stream 的 transient reconnect error 保持消費與重試，只有本地 bounded queue overflow 或真正結束才停止 |
| Event 由唯一 release API 釋放 | `client-ffi::mapped_values_have_no_cross_allocator_ownership`、`client-ffi::exported_value_guard_contains_panics`、`client-ffi::failed_output_calls_clear_caller_pointers`、`e2e.c` 的 roster accessor/release 驗證；create／poll 失敗或無事件時 out-pointer 會先清空 |
| 並行送出 cancel 與狀態查詢 | `e2e.c` 兩個 native threads 同時呼叫 command，另驗證第二 poll thread 得到 `ERPS_THREAD_MISUSE` |
| 真實 C compiler 連結 SDK | Windows x64 clang DLL/import library 與 WSL Linux x86_64 gcc shared-object 均以平台隔離 target 依 `client-ffi/README.md` 重建並執行 smoke；完整網路 E2E 均為 `C_E2E_PASS` |

所有 generated/network 邊界只傳 stable opaque IDs，不暴露 Specs `Entity`。敏感欄位 redaction、queue high-watermark 與 observability independence 由 metrics tests 驗證；`grpc::admin_metrics_exposes_counters_high_watermarks_and_latency` 驗證 standalone admin API 可讀取 counters、high-watermark、Elo quality 與 bounded latency percentiles；`load_test::memory_high_watermark_comes_from_the_operating_system` 驗證 load report 記憶體高水位取自 OS process RSS 取樣，而非 entity `size_of` 推估；`placement::releasing_finished_instance_does_not_charge_another_match` 防止 Finished/MatchResult 雙重釋放錯扣其他 instance 容量。Repository artifact 檢查不追蹤 `target`、DLL、EXE、PDB、SO、log 或 trace。低信用玩家的有限期停權邊界由 `grpc::credit_suspension_expires_at_a_defined_deadline` 驗證，期限後可重新排隊並透過完成對局恢復信用。
