# ERPS 固定負載驗證

- 日期：2026-08-28
- command：`erps-load-test --players 100000 --seed 1163022419 --workers 1`
- 平台：Windows x86_64，Rust 1.95.0，release
- 結果：PASS
- 玩家：100,000
- parties：58,834
- matches：22,500
- unmatched：0
- invariant failures：0
- elapsed：33 ms（本數字只代表 in-process correctness runner，不代表網路 SLA）
- throughput：2,956,332 players/s

傳輸生命週期由 `core/tests/grpc_e2e.rs` 驗證；`omb/tests/erps_adapter.rs` 另外驗證真正的 adapter register/heartbeat/launch/ready。
