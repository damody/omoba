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

## 1000 玩家逐秒 Elo 整合測試

執行：

```powershell
cargo test --manifest-path erps/Cargo.toml -p erps paced_1000_players_match_nearby_elo_and_settle_after_ten_seconds -- --nocapture
```

測試以固定 seed 產生 1000 名玩家與分散的起始 Elo，打亂入列順序，每個「模擬秒」最多放入 17 人。玩家收到 1v1 proposal 後個別同意，game server 依序回報 accepted、ready；第 10 個模擬秒再回報勝負。模擬時鐘讓此測試約 1 秒完成，不需真的等待 70 秒；它測的是時序與結算邏輯，不是 gRPC 的實際到達速率。初始 Elo 分成十個相隔 100 分的群組，每組隨機散布 11 分；搜尋上限設為 20 分，跨群錯配會使測試失敗。

2026-09-25 Windows 驗證：`ERPS_1000_PASS players=1000 arrivals_per_second=17 matches=500 settled_after_seconds=10 max_elo_gap=10 mean_elo_gap=2.70 peak_instances=87`。測試逐場比對獨立 Elo 公式、玩家 profile、場次計數與信用分；驗證每人只結算一次、所有容量在結束時釋放，並確認下一次排隊的 ECS 搜尋範圍與 bucket 使用更新後的 rating。

## 5v5 隨機組隊與 60 秒規則

執行：

```powershell
cargo test --manifest-path erps/Cargo.toml -p erps --test mode_properties five_v_five
cargo test --manifest-path erps/Cargo.toml -p erps five_v_five_live_queue_releases_structure_after_sixty_seconds
```

固定案例驗證 `4+1` 只配 `4+1`、`2+2+1` 只配 `2+2+1`；property test 隨機取五人 party 分割，驗證未滿 60 秒的所有成功候選都保持相同結構、完整十人且不拆 party。Live ECS 測試從實際 ticket 入列時間計算 59／60 秒邊界。跨結構例子中，`4+1` 的隊伍平均優勢為 16 分，`2+2+1` 為 4 分，因此後者原始平均 Elo 至少高 12 分才能配對。

2026-09-25 以更新後規則重跑 100,000 玩家、seed 42 的 release core workload：1 與 8 workers 均為 `PASS`、65,895 parties、22,500 matches、0 unmatched、0 invariant failures，logical digest 均為 `13361627642972436328`。舊版報告的 digest 與效能數字屬於先前「結構軟性懲罰」規則，不能與這次結果當成相同 scenario 比較；新版 report settings 已記錄 60 秒門檻與 +5／+10／+20／+30 優勢。

## 10000 玩家、12000 秒持續組隊模擬

執行：

```powershell
cargo test --manifest-path erps/Cargo.toml -p erps --test paced_party_12000 -- --nocapture
```

固定 seed 建立 10000 名玩家的可重用池；每個模擬秒恰好釋放 17 名目前未在排隊或對戰的玩家。以輪替的閒置玩家為起點，再挑選評分相近的玩家湊滿十人，隨機選擇 `5`、`4+1`、`3+2` 或 `2+2+1` party 結構；兩隊使用相同結構，並交給正式的 5v5 candidate builder 選隊。所有玩家自動同意並進入模擬對戰；開戰滿 10 個模擬秒才隨機決定勝隊與更新 Elo，結算後回到玩家池。每場均檢查 party 不拆、十人唯一、兩隊五人、雙方結構鏡像、隊伍原始平均 Elo 差不超過 50、勝隊分數上升、敗隊下降，並將每人的新分數與依其完成場次選擇 K-factor 的獨立 Elo 公式比較。再次參賽時會讀取該玩家最新分數。這是 in-process 核心長跑測試，不量測 gRPC／真實 game server 網路延遲；時間以邏輯時鐘推進，沒有實際 sleep，因此比 10 ms 對應 1 秒的方案更快。

2026-09-25 Windows 執行結果：`ERPS_PARTY_12000_PASS`；12000 個模擬秒、204000 次入列、20400 場配對及結算、204000 次逐人 Elo 更新。每位玩家完成 13～36 場，172459 次再配對使用已改變的 Elo；最大／平均兩隊原始平均 Elo 差為 24／0.08，最高同時對戰 17 場，四種 party 結構分別出現 5092／5066／5217／5025 場。測試本體耗時約 691 ms（不含編譯）。
