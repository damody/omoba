# 實作紀錄與證據索引

## 修改前現況（2026-08-27）

- Production observer 位於 `omoba-core/src/runtime/observer_validation.rs`，單一
  `selective-observer-validation` thread 以 `BTreeMap<team_id,
  SelectiveReplicaRuntime>`保存兩隊狀態，套 frame 時建立
  `NoopDisclosedWorldStepper`。這只能驗證 protocol transition 與 correction，不能證明
  Specs gameplay parity。
- `run_team_projection_after_wave_b()`在每個已存在的 team projector 上、每個 committed
  tick 呼叫 `enqueue_visible_demo_repairs()`。它遍歷當下可見 canonical entity；已有
  replica mapping、存在 demo render component，且值與 `hash_entities`不同時，就排入
  `ComponentRepair`並先更新 hash mirror。這會讓正常移動依靠 authoritative replacement。
- Authoritative `State::tick()` gameplay 順序：更新時間與輸入 → dispatcher → runtime
  events → hero clear → tower spawn → tower sell → tower target priority → item use → ability
  upgrade → ability cast → move → outcomes/maintain → tower upgrade → tower ability cast →
  tower scheduler → tower callback → script dispatch → creep wave → outcomes/maintain → Wave A
  commit → parallel visibility/projector。
- omfx full-world `sim_runner`順序：套輸入與時間 → dispatcher → clear runtime events → hero
  clear → tower spawn → tower sell → tower target priority → item use → ability upgrade → ability
  cast → move → outcomes/maintain → tower upgrade → tower ability cast → tower scheduler → tower
  callback → script dispatch → outcomes/maintain。它缺少 authoritative 的 creep-wave hook，且
  secure selective path另以 Noop stepper 套 filtered frame。

## Phase 差異表

| Phase | Authoritative | omfx（修改前） | 差異 |
|---|---:|---:|---|
| 套用輸入與固定時間 | 是 | 是 | adapter 不同 |
| Specs dispatcher | 是 | 是 | 呼叫入口不同 |
| 清除 runtime events | flush | clear | 收集語意不同 |
| Hero command clear | 是 | 是 | 無 |
| Tower spawn | 是 | 是 | 無 |
| Tower sell | 是 | 是 | 無 |
| Tower target priority | 是 | 是 | 無 |
| Item use | 是 | 是 | 無 |
| Ability upgrade | 是 | 是 | 無 |
| Ability cast | 是 | 是 | 無 |
| Move | 是 | 是 | 無 |
| 第一次 outcomes/maintain | 是 | 是 | sink adapter 不同 |
| Tower upgrade | 是 | 是 | 無 |
| Tower ability cast | 是 | 是 | 無 |
| Tower scheduler | 是 | 是 | 無 |
| Tower callback | 是 | 是 | 無 |
| Script dispatch | 是 | 是 | registry adapter 不同 |
| Creep wave | 是 | 否 | omfx 缺少 |
| 第二次 outcomes/maintain | 是 | 是 | sink adapter 不同 |

## 舊證據狀態

既有 selective-lockstep Phase 6 observer / differential / performance evidence 全部標記為
`superseded`：它們仍保留歷史價值，但因 production observer 使用 Noop stepper且 steady-state
repair會收斂狀態，不得再作為真正 gameplay deterministic parity 的通過證據。

## 最後驗證必備 guard

第 13 章必須加入 source guard：production `observer_validation`與 omfx secure selective
runtime 不得引用或建立 `NoopDisclosedWorldStepper`；Noop 僅能在 `#[cfg(test)]` fixture中存在。

## 既有基準（未啟動新 run）

- Authoritative cadence：設定與既有文件皆以 120 Hz 為 production cadence，tick period約
  8.333 ms。
- Outbound queue：修改前 team frame 使用 `mqtx.try_send`並忽略結果；因此滿載沒有可相信的
  wait duration，基準只能記為「0 ms observed wait、但可能靜默缺幀」。
- Observer lag：既有 metric 為單一 `audit_lag_ticks`，worker queue預設容量 4096；沒有留下
  可歸因到 Team 1/2 的 p50/p95/p99歷史值，故舊基準標記為 unavailable，而不是 0 或 pass。

## 實作決策紀錄

- Wire field `global_seed`使用未占用的 field number 18。
- 固定只接受 Team 1與Team 2；其他 team fail closed。
- Queue watchdog預設 5秒；超時產生 secure safe termination，不降級protocol、不略過frame。

## 最終驗證（2026-08-27）

- `omoba-core`：269 unit tests、3 個 1–100 round autoplay tests、doc tests 通過。
- `omb -p omobab --lib`：130 passed、1 個既有 benchmark ignored。
- `omfx -p omfx -p executor`：128 passed。
- `omb-script-abi`：13 passed；`base_content`：45 passed。
- Team replica 契約：phase、filtered bootstrap、Reveal/Hide/Forget、RNG、external
  effects、三方 hash、repair/rebase、故障注入與 10,000 entities fixtures 全數通過。
- Windows 與 WSL Ubuntu Rust 1.95.0 的 checkpoint hash 相同：
  `869ff527d094f3803a189f06d4293b8afcd6f1e94a6443259d37e12568d6a0e1`。
- 30 分鐘首次 soak 完成 215,869 ticks：p99 cycle 3.440 ms、commit p99 1 us、
  4,820.198 B/s/player、0 deadline miss、0 rebase、0 coverage gap。它揭露舊 RSS
  取樣與三個無界生命週期資料：retired ID set、未消費的 render/transition 暫存，及
  rebootstrap 重複載入 script DLL；因此該次 memory gate 正確失敗，沒有被當成 pass。
- 修正後 retired ID 使用固定 1 MiB fail-closed filter，render/transition 暫存每 frame
  清除，rebootstrap 保留 thread-local ScriptRegistry/dispatcher，只重建 filtered world。
- 修正後 10,000 entities、120 Hz、雙 observer 定向 soak：14,400 ticks、p99 cycle
  2.045 ms、4,384.015 B/s/player、0 deadline miss、0 rebase、0 coverage gap；Windows
  live-resident 穩態由 65,536 bytes 降至 61,440 bytes，斜率 -34.855 B/s。
- `openspec validate fix-server-lockstep-team-replicas --strict`通過；production source
  guard確認沒有 Noop observer、proactive repair或 entity/system/action RNG seed。

## Traceability 與 release verdict

需求到證據的映射：共用 phase runner由 phase trace/source guard覆蓋；filtered world與
資訊隔離由 hidden sentinel、memory exclusion及封包 redaction覆蓋；`global_seed + tick`
由 Windows/Linux RNG與 checkpoint parity覆蓋；server-authoritative recovery由
pre-repair differential及 repair/replace/rebase fixture覆蓋；兩隊平行 worker與可靠 queue
由 completion-order、單隊 failure、backpressure及 watchdog fixture覆蓋；效能與長局生命
週期由 10,000 entities 壓測、30 分鐘 workload及修正後 live-resident soak覆蓋。

**Release verdict：PASS。** 240/240 tasks完成，沒有 blocking failure、未回報 coverage
gap、protocol downgrade或資訊隔離例外。既有 Noop parity evidence維持 superseded。
