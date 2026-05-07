## 背景

omoba 採 server-paced lockstep（`docs/plans/2026-05-02-server-paced-lockstep-design.md`）：omfx 跑跟 omb 同步的 sim ECS，input 經 `InputSubmit{target_tick, input}` 上行，omb scheduler 排到 `target_tick` 後在 `TickBatch{tick, [InputForPlayer]}` broadcast 回所有 client。omfx sim_runner 拿 TickBatch 餵進 dispatch，跑到該 tick 後 `extract_snapshot` 推 `SimWorldSnapshot` 給 render thread，render thread 才畫出反映該 input 的畫面。

「input 到看見」總延遲分四段：
1. **client→server 上行**（半個 RTT — 但 KCP 重傳 / window 阻塞下不一定對稱）
2. **server scheduler buffer**（input 等到 `target_tick` — 通常 `now_tick + LOOKAHEAD`，原 plan 寫 `+3`，目前實作 `+3` 已退）
3. **server→client 下行**（半個 RTT 含 TickBatch payload size 可能超過單 KCP segment）
4. **client sim 推進 + render frame**（worker thread `extract_snapshot` 排程 + render thread 拉 mutex + Fyrox draw）

目前 codebase 只量到 `Ping: ... ms`（純 RTT，omfx-side `latest_rtt_us`），**沒任何端到端量測**。stress 場景（1000 塔 × 1000 creep）下，輸入「感覺卡」時無法分辨是 RTT 大、scheduler buffer 大、還是 render frame 慢 — 這是本 change 要解的觀測缺口。

audit 出來既有相關基礎設施：
- `proto/game.proto:438-450` `PlayerInput { oneof action }` — 9 個 action variant
- `proto/game.proto:455-459` `InputSubmit { player_id, target_tick, input }`
- `proto/game.proto:476-479` `InputForPlayer { player_id, input }`
- `omfx/game/src/lib.rs:826-829` `latest_rtt_us` + `LockstepEvent::Latency { rtt_us }` event
- `omfx/game/src/lib.rs:2585-2611` HUD render — `Ping: ... | Tick: ... | Net: ... wire / ... logical | fps: ... draws: ... tris: ...` 一整行
- `omfx/game/src/lib.rs:1842-1843` 1Hz 滾動視窗（`net_wire_bytes_last_sec` 抽樣模式）
- `omfx/game/src/sim_runner.rs::SimWorldSnapshot` 欄位多但**沒** `applied_input_ids`

## 目標 / 非目標

**Goals:**

- 量測端到端 input-to-render 延遲，含全四段（client→server / scheduler buffer / server→client / sim+render frame）
- 每個 action variant 各自有獨立的延遲樣本（區分 TowerPlace / TowerSell / ItemUse 等 — 不同 path 可能延遲分布不同）
- HUD 顯示最近 ~2 秒 rolling window 的 p50 / p99 / latest，跟既有 `Ping` 同列
- 對每筆樣本輸出 `omfx_app.log` debug log，TD_STRESS 60s smoke 後可 grep 出 N=幾百筆的分布
- determinism 不受影響（omoba-sim 69 pin tests / omb 145 lib tests 全綠）

**Non-Goals:**

- **不**做四段個別的 breakdown（client→server / scheduler / server→client / sim+render 各自多少）— 那需要 server 配合在 input 處理路徑插樁回送 timestamps，本 change 控制範圍只在 omfx + 最小 wire schema 改動。Phase 5 可加 `InputAck { input_id, server_recv_us, server_dispatch_us }` 做 breakdown
- **不**做歷史持久化（寫檔 / Prometheus exporter）— 純記憶體 rolling window + log
- **不**做 multi-client aggregation（看自己的就好；observer 模式進來後再考慮）
- **不**對 server-injected ticks 做量測（waveStart / GameEnd 等 ServerEvent 沒有「玩家輸入」起點，量測無意義）
- **不**改 sim 確定性的任何路徑（input_id 嚴禁進 ECS / Outcome / 任何 hash 過的 byte）

## 決策

### 決策 1: `input_id` 放在 `InputSubmit` / `InputForPlayer` wire 層，不放 `PlayerInput`

**選擇：** proto schema：
```
message InputSubmit {
  uint32 player_id = 1;
  uint32 target_tick = 2;
  PlayerInput input = 3;
  uint32 input_id = 4;     // NEW: omfx 指派，server echo back
}

message InputForPlayer {
  uint32 player_id = 1;
  PlayerInput input = 2;
  uint32 input_id = 3;      // NEW: copied from InputSubmit
}
```

`PlayerInput` 本身**不動** — 不污染 domain message。

**替代方案：**
- (A) 加在 `PlayerInput` oneof 外層欄位（每 variant 都帶）：domain message 沾上 metric 概念，違反 schema 純粹性
- (B) 開新 wire tag 0x19 `InputAck { input_id, server_recv_us }`：要 server 多送一條訊息，wire 流量增加；本 change 是「被動觀察」不是「server 配合 breakdown」

**理由：** wrapper-level 欄位是 metadata 自然位置；server 純 echo 不解析；`PlayerInput` 保持乾淨。

### 決策 2: `input_id` 編號用 omfx-side `AtomicU32` counter，不參與 determinism

**選擇：** `omfx/game/src/lockstep_client.rs` 加 `input_id_counter: AtomicU32`，每次 submit 時 `fetch_add(1)` 取下一個 id（從 1 起，0 保留為 "no id"）。range 4 billion 對單局遊戲足夠（一局最多 ~1 萬輸入）。

**替代方案：**
- (A) 拿 wall-clock micros 直接當 id：碰撞風險低但不單調易讀，log 分析麻煩
- (B) `(player_id, monotonic_counter)` 複合 id：multi-player 才需要，本 change non-goal

**理由：** AtomicU32 counter 是 lock-free、O(1)、單調遞增、log 易讀；不進 sim 路徑所以不需 deterministic。

### 決策 3: omfx-side `PendingInputBook: HashMap<u32, PendingInput>` 跟 sim 完全隔離

**選擇：**
```rust
struct PendingInput {
    submit_wall_clock_us: u64,    // Instant::now() at submit
    target_tick: u32,             // expected tick where this input applies
    action_kind: InputActionKind, // for stratified p50/p99 by variant
}

struct PendingInputBook {
    pending: HashMap<u32, PendingInput>, // input_id → PendingInput
    max_age_ms: u64,                     // evict if > 5000ms (sample dropped, never paired)
}
```

submit 時 insert，pair（snapshot.applied_input_ids 含此 id）後 remove + 餵進 `InputLatencyMeter`，evict 過期項避免記憶體洩漏（玩家斷線重連等場景）。

**替代方案：**
- (A) 把 `submit_wall_clock_us` 放進 wire payload 上 server 再 echo back：avoid omfx-side state map，但 wire bytes 增加（每 input 8 bytes × N input/秒），不如 client local
- (B) `BTreeMap` 取代 HashMap：可 O(log N) 找最舊項做 eviction，但本場景 N < 100 同時 pending，HashMap + 每秒 eviction 掃整表更簡單

**理由：** 純 client-side state 最便宜；HashMap O(1) lookup；evict 簡單線性掃 N < 100 完全可接受。

### 決策 4: `SimWorldSnapshot.applied_input_ids: Vec<u32>` 走非 sim metadata channel

**選擇：** `extract_snapshot` 在生 snapshot 時，從**這個 tick 接收到的 TickBatch.inputs[] 的 input_id 列表**生成 `applied_input_ids: Vec<u32>`，塞進 snapshot。重要：

- 此欄位**只**供 render thread 配對 latency 用，sim ECS / Outcome / determinism hash 完全不讀取
- omoba-sim 69 pin tests 不受影響（pin 對象是 Fixed64 / trig / RNG / bincode wire byte，跟 SimWorldSnapshot 無關）
- 即使 omfx 把 sim crate 接到 server-paced replica，server 跟 client 各自的 `applied_input_ids` 內容**會**不一樣（client 從自己上行的 input 來，但 server 收到的是所有 client 的 input）— 這 OK，因為這個欄位純觀察用，不參與任何驗證

**替代方案：**
- (A) render 端自己 grep `TickBatch.inputs[]`：omfx 收到 TickBatch 後就交給 sim_runner consume，render 看不到原始 batch — 要重複保存 N tick 的 batch，浪費
- (B) sim_runner 用 `Outcome::InputApplied { input_id }` queue：把 metric 進 sim outcome 路徑 — **錯誤**，違反 determinism 隔離

**理由：** sim_runner 已經拿到 TickBatch；順手把 input_ids 列表塞進 snapshot 是 O(input_count_per_tick) 動作，免費；render 端按 snapshot.tick 就知道這些 id 該配對。

### 決策 5: `InputLatencyMeter` 用 ringbuffer + sorted Vec 算 p50/p99，每秒重算 1 次

**選擇：**
```rust
struct InputLatencyMeter {
    samples: VecDeque<LatencySample>, // ringbuffer, max N=120 (~2 sec @ 60Hz)
    last_compute_at: Instant,         // throttle p50/p99 recompute to 1Hz
    cached_p50_ms: u32,
    cached_p99_ms: u32,
    cached_max_ms: u32,
    cached_latest_ms: u32,
}

struct LatencySample {
    input_id: u32,
    action_kind: InputActionKind,
    total_ms: u32,
    submitted_at: Instant,  // for window eviction by age
}
```

每 sample 進來：push_back，超過 N pop_front。每秒一次：clone samples → sort → 取 idx N/2 跟 N×0.99 → 寫 cached_p50_ms / cached_p99_ms。HUD 讀 cached_*。stratified-by-action_kind 留給 future（先全部混算）。

**替代方案：**
- (A) 用 `histogram` crate / hdrhistogram：精度高但套件 overkill，N=120 不需要
- (B) 每 sample 重排：浪費 — 60 Hz × O(N log N) = 60 × 120 × log(120) ≈ 50K 比較/秒，能跑但不必要
- (C) 不快取，HUD 每 frame 直接算：HUD 60 fps × 排序 = 同 (B) 浪費

**理由：** 1Hz 重算 + 快取讀取對 HUD 是 zero overhead；sorted Vec O(N log N) 一次成本可忽略；標準庫工具夠用。

### 決策 6: HUD 顯示格式整合進既有 `Ping` 行

**選擇：** 既有 line：
```
Connected | Ping: 12.3 ms | Tick: 1234 | Time: 20.6 | Entities: 87 | Heroes: 1 | Creeps: 6 | Net: 4.2 KB/s wire / 8.1 KB/s logical
```

改成：
```
Connected | Ping: 12 ms | Lag: p50 65 / p99 120 ms | Tick: 1234 | ...
```

`Lag` 欄位前面 `Ping` 是純 RTT 容易誤解為「延遲」，加 `Lag` 段一併顯示讓玩家清楚兩個概念分開（網路 RTT vs 端到端體感）。`Lag:` 段在沒任何樣本時顯 `—`。

**替代方案：**
- (A) 開新一行：HUD 已經塞滿，screen real estate 寶貴
- (B) 用 graph widget：超出本 change 範圍

**理由：** 同行擴展是最低 UI 成本；用 `Lag` 標籤跟 `Ping` 區隔語意；symbol `—` 跟既有 `Ping: —` pattern 一致。

### 決策 7: stress log 結構 — 每樣本一行 debug log

**選擇：** pair 成功時：
```
log::debug!("input_render_latency: id={} kind={:?} target_tick={} submit_us={} render_us={} total_ms={}",
    input_id, kind, target_tick, submit_wall_clock_us, render_wall_clock_us, total_ms);
```

TD_STRESS 60s smoke 跑完後可 grep `input_render_latency:` 拿到所有樣本，pipe `awk` / `python` 算分布。debug level 預設不開，stress 跑 `RUST_LOG=omfx::lib=debug` 啟用。

**替代方案：**
- (A) 寫 CSV 檔：`omfx` 不做檔案寫入是慣例（log4rs 會處理 rotation 等）；用 log 統一
- (B) JSON line format：可以但增加 log size；本 change 簡單 key=value 易 grep

**理由：** log 是現成基建；debug level 不影響 release；key=value 適合 grep + awk pipeline。

## 風險 / 取捨

| Risk | Mitigation |
|---|---|
| `input_id` 進到 sim ECS / Outcome → 破壞 determinism | (1) Decision 1-4 嚴格規範 `input_id` 只走 wire wrapper / omfx PendingInputBook / snapshot.applied_input_ids 三條 omfx-side metadata channel；(2) lib test：跑 sim 一段時間後 `cargo test --no-default-features` 確認 omoba-sim 69 pin tests 全綠；(3) code review：禁止 `input_id` 出現在任何 specs Component / Resource / Outcome variant |
| `PendingInputBook` 記憶體洩漏（pair 失敗 / sample lost） | (1) `max_age_ms = 5000` 過期 evict（每秒掃一次）；(2) 加 metric `pending_count` 進 HUD `Lag:` 段（如 `Lag: p50 65 / p99 120 ms (p:3)`），高於 10 視為 abnormal |
| TD_STRESS 1000 entity tick 時 `applied_input_ids` 數量大 | TD_STRESS 1 player 不會超過 ~5 input/秒；`Vec<u32>` clone 成本可忽略；如果 multi-player 觀察值大可加 `applied_input_ids: SmallVec<[u32; 4]>` 優化 |
| `Instant::now()` 在 Windows 跨核心 jitter（NUMA-skewed timestamp） | 已 audit：omfx 已用 `Instant::now()` 做 RTT 量測（`latest_rtt_us`）跑得 OK；本 change 沿用同樣假設，jitter 影響量級遠小於 lockstep buffer |
| Wire schema 變動 BREAKING 與同期進行的 `lockstep-cleanup-and-hud` 衝突 | 兩個 change 編寫獨立 — `lockstep-cleanup-and-hud` 只改 outcome enum / state/resource_management.rs / sim_runner extract，不改 proto；本 change 只加 proto 欄位 + omfx 邏輯，不改 outcome / emit。Merge 順序無關 |
| HUD 加 `Lag:` 欄破壞既有 ASCII 寬度假設 | 既有 status string format!() 沒有寬度約束 / 沒 column 對齊；新增段不影響 layout |
| `input_id_counter` overflow（u32 4 billion） | 一局 4B input 不可能；overflow 後 wrap-around 可能撞到舊 pending entry 但跟 `Instant` 對比會自然失配 evict — 不寫額外 saturation 檢查 |

## 遷移計畫

無 schema migration（無 DB / 序列化檔需 migrate）。Wire 協定 BREAKING — `InputSubmit` / `InputForPlayer` 增 `input_id` 欄位，但 protobuf field 規則：新增 optional / scalar 欄位不破壞舊 client 解碼（會被忽略）；新 server 收舊 client 的 `InputSubmit{input_id=0}` 也能跑（id=0 被當 sentinel 視為「無 metric」）。

但因為 client/server 同步發行慣例，本 change 直接視為 BREAKING，不寫向下相容 fallback。

部署順序：
1. proto schema 改完跑 `cargo build` 確認 omoba-core / omb / omfx 三邊 prost 重 codegen 都成功
2. omb 端先 wire 完 echo back（最小改動）
3. omfx 端 PendingInputBook + InputLatencyMeter + HUD 加上去
4. smoke：TD_1 點任一塔，HUD `Lag:` 段顯示有限數字，log 出現 `input_render_latency:` debug 行
5. TD_STRESS 60s smoke：確認 p99 < 200ms、無 panic、`pending_count` 穩定 < 10
6. determinism gate：`cargo test -p omoba-sim` 69 全綠

Rollback：本 change 純加法（沒砍既有 emit 跟 logic），revert commit 即可；無 wire 協定不可逆變動（proto 加欄位反向解碼仍可跑）。

## 未決問題

- 既有 `LockstepEvent` enum 是否已有「input echoed back」事件？若沒有，本 change 要加 `LockstepEvent::InputApplied { input_id, target_tick, render_wall_clock_us }`；若已有類似事件可直接擴展（待 audit `lockstep_client.rs` 看 enum 定義）
- HUD `Lag:` 段顯示 `pending_count` 是否進入第一版？或先不顯示，等 stress 跑出問題再加？— 建議**進**，因為觀察到「p99 突然飆 + pending_count 飆」就能直接定位是 sample lost 還是真延遲
- p50 / p99 是否要按 `action_kind` 分組顯示？— 建議**第一版混算**，stress log 已有 `kind=...` 欄位後處理時可 stratified 算；HUD 顯示分組會佔太多 column
- target_tick == sim_runner 跑到的 tick 後，input 在那個 tick 內哪個系統處理（player_input_tick）發生在 dispatch 的哪個 stage？這影響 `extract_snapshot` 看到的 `applied_input_ids` 是否真的包含「該 tick 處理過的所有 id」（待 audit `omb/src/tick/player_input_tick.rs` 跟 `sim_runner.rs::dispatch_loop`）
- TickBatch 的 `inputs[]` 在 omfx side 是被 `lockstep_client` 餵進 `sim_runner` 的 channel 還是 sim_runner 自己 poll 的？這影響 `applied_input_ids` 該在哪一步收集（待 audit `lockstep_client.rs` → `sim_runner.rs` channel handoff）
