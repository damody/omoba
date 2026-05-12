# omfx Perfetto Profiling

## 目的

omfx Perfetto profiling 是 opt-in 診斷工具，用來把 frontend main thread、`omfx-sim-runner` thread，以及 omfx 會走到的 `omoba-core` runtime/KCP hot paths 輸出成 Perfetto trace。一般 `run.bat` / `run_stress.bat` 不會預設產生 trace。

## 啟用方式

從 repo 根目錄或 `omfx/` workspace 啟動 executor 前設定：

```powershell
$env:OMFX_PERFETTO_TRACE = "1"
$env:OMFX_PERFETTO_DETAIL = "frame"
$env:OMFX_PERFETTO_MAX_SECONDS = "20"
cargo run --manifest-path omfx/Cargo.toml -p executor --features runtime-lua-content
```

TD_STRESS release run 可直接使用：

```bat
run_stress.bat --trace
```

預設輸出到 `omfx\target\profiles\stress.perfetto-trace`。若要指定輸出檔，先設定 `OMFX_PERFETTO_PATH` 再執行 `run_stress.bat --trace`。

可選環境變數：

- `OMFX_PERFETTO_TRACE=1`: 啟用 trace。可用值包含 `1`、`true`、`yes`、`on`。
- `OMFX_PERFETTO_PATH=<path>`: 指定輸出檔。未設定時輸出到 `omfx/target/profiles/omfx-<timestamp>-<pid>.perfetto-trace`。
- `OMFX_PERFETTO_DETAIL=frame|deep`: `frame` 是預設低成本模式；`deep` 保留給 selected inner-loop diagnostics。
- `OMFX_PERFETTO_MAX_SECONDS=<seconds>`: 到時後自動結束 process，適合短時間錄製。

啟用成功時，`omfx_app.log` 與 terminal log 會顯示 trace path，並提示用 `https://ui.perfetto.dev` 開啟。

## Perfetto UI

1. 開啟 `https://ui.perfetto.dev`。
2. 選擇 `Open trace file`。
3. 載入 `.perfetto-trace`。
4. 在 tracks 中查看 frontend main thread、`omfx-sim-runner`、`omfx-lockstep-client` 或 tokio task thread。

## 主要 Spans

- `omfx::Plugin::update`: frontend frame root span，含 tick、entity/projectile count、draw calls、triangles。
- `omfx::frame::lockstep_event_drain`: main thread drain `LockstepEvent` 並轉送 `TickBatch`。
- `omfx::frame::snapshot_consumption`: 讀取 sim snapshot、更新 `render_bridge`、HUD state 與 caches。
- `omfx::frame::entity_interpolation_and_batches`: entity interpolation、body/HP/facing batch writes 與 batch flush。
- `omfx::frame::projectiles_and_vfx`: local projectile 與 explosion VFX update。
- `omfx::frame::camera`: camera update。
- `omfx::frame::ui`: UI labels、HUD、shop/control panel、ability UI。
- `omfx::frame::statistics`: `FrameProfile` 與 renderer stats 更新。
- `omfx::sim_runner::tick`: sim tick root span，含 tick、queue length、input count、runtime publish flag。
- `omfx::sim_runner::*`: receive、input apply、dispatcher、pending drains、script dispatch、metadata、snapshot extraction、FX retention、publish。
- `omoba_core::runtime::*`: shared runtime drains、`process_outcomes`、script dispatch、snapshot/metadata/world init。
- `omoba_core::kcp::*`: KCP connect/join/input submit、frame receive、seq-gap `StateReq`。

## omoba-core 限制

`omoba-core` 只輸出 lightweight `tracing` spans/events。它不解析 `OMFX_PERFETTO_*`，不建立 trace 檔，也不安裝 Perfetto layer。是否寫入 Perfetto 由 `omfx/executor` 在 process startup 決定。

## Overhead 注意

預設 `OMFX_PERFETTO_DETAIL=frame` 不產生 per-entity spans。`deep` 只適合短時間診斷，可能改變效能瓶頸並快速產生大型 trace。TD_STRESS 建議搭配 `OMFX_PERFETTO_MAX_SECONDS=10` 到 `30`。

## 驗證

- Profiling disabled：不設定 `OMFX_PERFETTO_TRACE`，啟動後不應產生 `.perfetto-trace`。
- Profiling enabled：設定 `OMFX_PERFETTO_TRACE=1`，短 run 後應產生 trace，且 log 顯示輸出 path。
- Trace content：Perfetto UI 中應可看到 frontend main thread、`omfx-sim-runner` 與 `omoba-core` spans。
