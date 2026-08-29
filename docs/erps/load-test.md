# ERPS 固定負載驗證

- 日期：2026-08-28
- commands：`erps-load-test --players 100000 --seed 42 --workers 1`、同 seed `--workers 8`、縮小 `--grpc --players 1000 --workers 8`
- 平台：Windows x86_64，Rust 1.95.0，release
- 結果：PASS
- 玩家：100,000
- parties：65,775
- matches：22,500
- unmatched：0
- invariant failures：0
- workers=1（OS memory sampling）：elapsed 1,286 ms；throughput 77,722 players/s
- workers=8（OS memory sampling）：elapsed 1,653 ms；throughput 60,470 players/s
- logical digest（1 worker 與 8 workers 相同）：`16351624370020315097`
- ready rate：0.9940；average Elo quality key：107.8852
- retries/rejections/timeouts/cancellations/server cycles：135/112/23/90/22
- peak capacity utilization：0.0001478；OS-sampled RSS high-watermark：24,223,744 bytes（1 worker）、24,961,024 bytes（8 workers）
- p50/p95/p99（8 workers）：24/53/79 µs

`--grpc --players 1000` 實際建立 1,000 個 client、669 個 party，以三種模式完成 225 場 proposal/accept/launch/ready/result，transport elapsed 72,747 ms；最新 120-player regression 另完成 76 parties／27 matches／27 acknowledged result commits，transport elapsed 2,924 ms 且 invariant failure 為 0。Transport fleet 在 `tw`、`us`、`eu` 各註冊三台能力不同且 mode 專用的 server：1v1 為 capacity/cost/max-instances `8/1/2`、5v5 為 `20/5/4`、FFA8 為 `12/4/3`；報告為 `fleet_servers=9`、`servers_used=7`、`regions_used=3`。gRPC runner 會逐場驗證 launch mode、完整且無重複的 roster、各模式 team shape、5v5 party 不拆，並等待對應 MatchResult ACK；control stream 中間出現其他 ACK 不會被誤判為 Launch。Windows clang 與 WSL Linux gcc 使用平台隔離的 release target，以公開 C header/dynamic library 跑 invite/join/leave/rename、雙執行緒 command、poll misuse、state reconciliation、ready match 與 roster accessors，兩者皆為 `C_E2E_PASS`。`omb/tests/erps_adapter.rs` 驗證真正的 adapter register/heartbeat/accepted/ready/reconcile。

2026-08-29 完成交付稽核重跑：release 模式以相同 `--players 100000 --seed 42` 分別執行 workers=1/8，兩者皆為 `PASS`、22,500 matches、0 unmatched、0 invariant failures，logical digest 仍同為 `16351624370020315097`；加入真實 OS resident-memory sampling 後 elapsed 為 1,286/1,653 ms。`memory_high_watermark_bytes` 在開始、ECS materialization 後、每 256 場及結束時讀取 OS process RSS 並取最大值，不再使用 entity 數量乘 `size_of` 的估算；Windows/Linux 共用安全的 `sysinfo` API，核心仍維持 `forbid(unsafe_code)`。CLI 現提供 `--help`，且 `--workers 0` 會明確失敗，避免實際使用一個 worker卻在 baseline 報告記錄為零。
