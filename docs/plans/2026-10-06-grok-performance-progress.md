# 2026-10-06 Grok 效能量測進度

本批只加正式路徑上的低負擔視窗量測，以及可重用的 Lua 報告收集器。沒有跑一場正式對局，沒有填實測數字，沒有門檻常數。這不是 6.5 完成，也不能代替後續驗收。

量測不是模擬輸入，不進 state hash。取樣不含等待。平均是 `floor(sum / samples)`，不是 p95，輸出裡沒有 p95 欄位。

## 格式

每一段先有一筆啟用契約，視窗關閉時再有一筆摘要。兩者都是單行 JSON，整數，不出現 NaN。

- 啟用：`OM_PERF_ENABLE {json}`
- 摘要：`OM_PERF {json}`

`OM_PERF ` 後面有空白，所以不會把 `OM_PERF_ENABLE` 當成摘要。日誌前綴可以留著，收集器從標記開始解析。

共通欄位：`v=1`、`component`、`metric`、`scope`、`unit`、`samples`、`count`（等於 samples）、`sum`、`max`、`mean`、`mean_definition=integer_floor_sum_over_samples`、`not_a_statistic=p95`、`window_duration_ns`。需要身分時加 `player_id`、`team_id`，IPC 再加 `connection`。

| 收集器段落 | component | metric | scope | unit | 這段是什麼 |
| --- | --- | --- | --- | --- | --- |
| `server_tick_compute` | `server` | `tick_compute` | `state_tick_excluding_transport_send` | `ns` | 成功結束的 `State::tick` 牆鐘，減去每一次成功的 `reliable_send_with_watchdog`。不是 debug 的 run+dispatch+outcomes 局部和，也不是含 scheduler sleep 的整個行程牆鐘。 |
| `client_replica_step` | `client_runtime` | `replica_step` | `apply_encoded_frame` | `ns` | `apply_encoded_frame` 返回當下停止。不含 evidence IO、後續 presentation、KCP。 |
| `presentation_ipc_send` | `presentation_ipc` | `wire_bytes` | `localhost_tcp_length_prefixed_frame` | `byte` | 單一連線成功寫出的 frame，含 4-byte 長度前綴。 |
| `presentation_ipc_receive` | `presentation_ipc` | `wire_bytes` | `localhost_tcp_length_prefixed_frame` | `byte` | 同一連線成功讀完並接受的 frame，含長度前綴。失敗、截斷、未接受的 payload 不計。 |
| `unreal_game_frame_interval` | `unreal` | `game_frame_interval` | `game_thread_delta_seconds` | `ns` | 遊戲執行緒 `Tick` 的 `DeltaSeconds`。不是 GPU，也不是 render thread。 |
| `unreal_presentation_work` | `unreal` | `presentation_work` | `actor_tick_fplatformtime` | `ns` | 同一個 actor 呈現工作的 `FPlatformTime` 區間。不是幀間隔。 |

IPC 摘要另有 `direction`（`send` 或 `receive`）、`messages`、`bytes`（等於 sum）、`rate_bytes_per_s`、`rate_definition=integer_floor_bytes_per_window_second`、`rate_is_not=frame_interval`。速率是 `floor(bytes * 1000000000 / window_duration_ns)`。視窗時長為 0 時速率是 JSON `null`，收集器把該段標成未驗證，不補 0。幀間隔與呈現工作沒有 `rate_bytes_per_s`。server 與 client 時間摘要也沒有。

UE 幀間隔另有 `interval_is=game_thread_delta_seconds`、`interval_is_not=gpu_or_render_thread`。呈現工作另有 `work_is=actor_tick_fplatformtime`、`work_is_not=gpu_or_render_thread`。

視窗長度是 60。未滿 60 不印摘要。印出的 `samples` 在正式路徑上是 60；收集器接受任何內部一致、samples 至少為 1 的視窗，但不接受 0、非有限值、錯 scope 或錯 unit。

server 啟用契約例子：

```text
OM_PERF_ENABLE {"v":1,"component":"server","metric":"tick_compute","scope":"state_tick_excluding_transport_send","level":"info","window_samples":60,"excludes":"scheduler_sleep,reliable_send_timeout,warmup,paused,finished","hash_input":false}
```

## 開啟方式

沒有新的環境變數、cvar 或 feature。沿用現有 logger。

- Server、client runtime、presentation IPC：`log::info!`。`scripts/moba_role_launch.lua` 已經把 `RUST_LOG` 設成 `info`，正式 role launch 會看到這些行。debug 的 phase 摘要仍是 `log::debug!`，不會混進這份正式摘要。
- Unreal：`LogOmWorldBridge` 的 `Display`。一般 Unreal 日誌看得到，不需要另開統計 cvar。
- Server 只在該 tick 開始前已經有 `MobaMatch`，而且 `begin_moba_match_tick` 回傳 active 時取樣。warmup、暫停、結束，以及 tick 中途失敗，都不進入視窗。
- Client 在 runtime 進入 session loop 前印一次啟用契約。只有 `apply_encoded_frame` 成功且回傳 `Some` 才計一筆。
- IPC 在 renderer handshake 驗證並取得連線租約後印一次啟用契約。每個連線自己的 meter，斷線後不留跨測試的累積。
- UE 只在 presentation runtime 已啟動、catalog 可用、連線為 Connected、世界未暫停時計。啟動、暫停或斷線造成的間隔不記；恢復後的第一個 `DeltaSeconds` 也不記。`Stop`、尚未啟動時的 `Start`，以及新的 runtime handle，會清兩個視窗與啟用標記。同一個 handle 上重複 `Start` 不清窗。

若把 info 或 Display 濾掉，收集器看不到啟用契約，結果是 `unverified`，不是成功。

## 收集器

純解析模組是 `scripts/moba_performance_report.lua`。CLI 只轉呼叫它：

```bat
D:\code\omoba\tools\lua\lua.exe scripts\collect_moba_performance.lua --server SERVER_STDERR.log --runtime RUNTIME_STDERR.log --unreal UE.log --out SUMMARY.json
```

有外部基線時才加 `--baseline`。本批不附基線檔，也不把任何數字寫死：

```bat
D:\code\omoba\tools\lua\lua.exe scripts\collect_moba_performance.lua --server SERVER_STDERR.log --runtime RUNTIME_STDERR.log --unreal UE.log --baseline BASELINE.json --out SUMMARY.json
```

`BASELINE.json` 必須由之後的正式對局量測產生。形狀如下，`limit` 要換成那個外部整數；`statistic` 只能是 `mean` 或 `max`，`p95` 會變成 error。`wire_bytes` 必須帶 `direction`，否則 send 與 receive 無法分辨。

```json
{
  "comparisons": [
    {
      "component": "server",
      "metric": "tick_compute",
      "statistic": "max",
      "op": "lte",
      "limit": "<external integer from a later measured match>",
      "unit": "ns"
    }
  ]
}
```

stdout 與 `--out` 都是一份 JSON。一定含 `source_paths`、`metric_definitions`、`segments`、`thresholds`、`errors`、`missing`、`status`。

- `success`：六段都有、四個啟用契約都在正確檔案、數值與 scope 合法。exit 0。沒有 baseline 時 `threshold_result` 是 `not_evaluated`。client、IPC send、IPC receive 與兩個 UE 指標必須是同一個 `player_id` 與 `team_id`。各 component 的啟用契約必須與該段摘要的身分一致。每條有樣本的 IPC connection 都要有相同身分的啟用契約；同一玩家重連多條 connection 可以。server 沒有 player，不參與這次比對。
- `unverified`：缺任一段、缺啟用契約，或 IPC 視窗時長為 0 而沒有速率。exit 2。剩下的身分仍然要一致，否則升為 error。
- `error`：NaN、無限大、空視窗、非整數、mean 不是 `floor(sum/samples)`、`ceil(sum/samples) > max`、錯 scope、錯 unit、p95、幀間隔帶了 byte rate、IPC 帶了 frame interval、段出現在錯誤檔案、同一段混了 player 或 team、跨 component 的 player/team 不一致、啟用契約與摘要身分不符、IPC connection 沒有對應啟用、baseline 失敗、baseline 要求 p95、`comparisons` 是空的或不是 array、`limit` 不是非負精確整數。exit 1。

`limit` 必須是非負的精確整數。0 可以，負數與小數不行。`comparisons` 為 JSON `[]`、`{}`、`null` 或其他非 array 時是 error，`threshold_result` 為 `fail`，不能當成 pass。視窗用 `(sum - 1) // samples + 1` 表示 `ceil(sum/samples)`，不用 `samples * max`，避免乘法溢位。這些都不是正式對局門檻，文件不寫死任何門檻數字。

多個視窗會加總 samples、count、sum 與 duration，max 取最大，mean 用加總後的整數除法。IPC 速率是 `floor(bytes * 1000000000 / window_duration_ns)`，用有界整數二元除法，中間積不會因為超過 JSON 精確整數範圍就拒絕一個仍然精確的結果。多個 connection id 會排序列出，不當成錯誤。

UE 的秒轉奈秒在寫入 `uint64` 之前拒絕 `Seconds * 1e9 >= 2^64`。`double` 無法精確表示 `MAX_uint64`，那個轉換會進位到 `2^64`，用 `>` 會放行後再越界轉型。Automation 斷言使用 UE 5.8 的雙參數 `TestTrue`／`TestFalse`。本輪沒有編譯這份 C++。

## 本批沒有做

- 沒有把 KCP `wire_bytes` 記進這份 IPC。那是另一條傳輸。
- 沒有改玩法、Bot、schema、ABI、hash、codegen、生成檔、資產、OpenSpec 或 `state.json`。
- 沒有跑 UBT、Editor、PIE、全套測試、100 場或部署。C++ automation `Om.Generated.PresentationPerfWindow` 只覆蓋視窗數學與文案，留待後續限定建置執行。
- 門檻與一場 `run_moba_role_ue.lua` 的實測數字留到最後。headless 100 場沒有 UE frame，不能單獨關閉 6.5。
