# KCP 多人化流量優化 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 omb↔omfx 的 KCP 傳輸從 single-player JSON 改造成 schema 化 binary 協定，在 50ms render_delay 容忍下削減 75~85% 流量，使 10~30 player、每人 AOI 內 500~1000 entity 的 MOBA 共享戰場穩定運作。

**Architecture:** 6 個 phase 依序推進：(P1) 不動 schema 的第一梯隊優化（facing 閾值、死欄位、LZ4、batch dedupe、heartbeat per-player）；(P2) `proto/game.proto` 重寫為統一 `oneof` + Quantization，一次 same-commit 升級前後端（C1 策略）；(P3) `hero.stats` 冷熱拆；(P4) `creep/M` velocity 外推；(P5) per-player AOI + fan-out `Arc<[u8]>` 共用；(P6) 事件驅動 batch + SequenceGap resync。每個 phase 獨立 git worktree，P1 結束先合回 master benchmark。

**Tech Stack:** Rust 1.91.0 / specs 0.20 ECS / tokio / tokio_kcp 0.9 / prost + tonic-build / lz4_flex / Fyrox (omfx) / abi_stable (script ABI 不變動)。

**Upstream design:** `C:\Users\damod\.claude\plans\omb-omfx-valiant-treasure.md`（approved plan）。

---

## Global conventions

- **Rust edition**: 2021；整專案固定 Rust 1.91.0（`rust-toolchain.toml`）。每個 phase 實作期間不得改 toolchain。
- **Feature gating**：本計畫只修 `kcp` feature path。`mqtt` / `grpc` path 保留原 JSON pipeline，build 必須仍然過 —— 每個 phase 的 verification 都要跑三套 feature 的 `cargo check`。
- **Commit message 語系**：延續 repo 風格，`chore:` / `feat:` / `fix:` / `perf:` 前綴，中文描述可接受（參考 recent commit `5e4c4c9 chore: 更新 omb + omfx 子模組參考`）。
- **Metrics counter 命名**：`kcp_tx_bytes_total{event=..}`、`kcp_msg_total{event=..}`。統計物件放 `omb/src/transport/kcp_transport.rs` 的 `KcpTransport` struct 裡；provided getter 讓 stress test 查。
- **Graphify update**：每 phase 收尾 `graphify update .`，保 knowledge graph 同步。
- **.bat 行尾**：本計畫不新建 .bat 檔。如需調用既有：`run.bat`、`run_stress.bat`、`gen_docs.bat`。
- **Submodule bump 時機**：P2 的 proto schema 切換是 hard cutover —— `omb` / `omfx` / `omoba-core` 的新 commit 必須在同一個 master commit bump 三個指標，避免中間有 broken state。
- **Worktree**：每個 phase 起手先 `git worktree add ../omoba-kcp-pN -b kcp-opt/pN`，phase 結束合回 master 再啟下一個。

---

## Phase 0: Metrics 基建（0.5 day，獨立 worktree `kcp-opt/p0`）

目的：先把「量測工具」建好，否則每個 phase 的 gate criteria 無從 assert。

### Task 0.1: 加 bytes/msg counter 到 KcpTransport

**Files:**
- Modify: `omb/src/transport/kcp_transport.rs`（加 counter + getter）
- Modify: `omb/src/transport/mod.rs`（re-export counter type 如果需要）

**Step 1: 讀 kcp_transport.rs 當前結構**

Run: Read `omb/src/transport/kcp_transport.rs`，找 `flush_batch` 函式與 `KcpTransport` struct 定義。

**Step 2: 加 counter 欄位**

在 `KcpTransport` struct 加：

```rust
pub struct KcpBytesCounter {
    /// key = event kind string（例如 "hero.stats", "creep.M"）
    per_event: parking_lot::Mutex<hashbrown::HashMap<&'static str, (u64, u64)>>, // (bytes, msgs)
    total_bytes: std::sync::atomic::AtomicU64,
    total_msgs: std::sync::atomic::AtomicU64,
}

impl KcpBytesCounter {
    pub fn record(&self, kind: &'static str, bytes: usize) {
        self.total_bytes.fetch_add(bytes as u64, std::sync::atomic::Ordering::Relaxed);
        self.total_msgs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut m = self.per_event.lock();
        let e = m.entry(kind).or_insert((0, 0));
        e.0 += bytes as u64;
        e.1 += 1;
    }
    pub fn snapshot(&self) -> KcpCounterSnapshot { /* clone out Map + atomics */ }
    pub fn reset(&self) { /* clear */ }
}
```

於 `KcpTransport` 內持 `Arc<KcpBytesCounter>`，`flush_batch` 送出每則訊息前 `counter.record(event_kind, payload_bytes.len())`。

**Step 3: 暴露給外部 query**

加 `KcpTransport::bytes_counter() -> Arc<KcpBytesCounter>`。

**Step 4: Build check**

Run: `cargo build --manifest-path omb/Cargo.toml -p omobab`
Expected: 無 warning 新增、無 error。

**Step 5: Commit**

```bash
git add omb/src/transport/kcp_transport.rs omb/src/transport/mod.rs
git commit -m "perf(kcp): add per-event bytes/msg counters on transport"
```

---

### Task 0.2: 加 `network_bytes` 整合測試 scaffold（先空跑）

**Files:**
- Create: `omb/tests/network_bytes.rs`

**Step 1: 寫 placeholder test**

```rust
//! 整合測試：跑 TD_STRESS 30s，assert kcp bytes/sec 低於 budget。
//! 目前只 scaffold，Phase 之後每個階段更新 budget。

#[test]
#[ignore]
fn kcp_bytes_budget_td_stress() {
    // TODO: Phase 1 之後填實作
    // 1. 用 game.toml.stress variant 啟 server in-process
    // 2. mock kcp client subscribe all topics
    // 3. sleep 30s，讀 KcpTransport::bytes_counter().snapshot()
    // 4. assert total_bytes / 30 < CURRENT_BUDGET
    panic!("not yet implemented");
}
```

**Step 2: 跑 test 確認 skipped**

Run: `cargo test -p omobab --test network_bytes`
Expected: `0 passed; 0 failed; 1 ignored`（因為 `#[ignore]`）

**Step 3: 跑 `-- --ignored` 確認 panic 如預期**

Run: `cargo test -p omobab --test network_bytes -- --ignored`
Expected: `FAILED - panic "not yet implemented"`

**Step 4: Commit**

```bash
git add omb/tests/network_bytes.rs
git commit -m "test(kcp): add network-bytes stress test scaffold"
```

---

### Task 0.3: 先量 baseline

**Step 1: 合 p0 worktree 回 master**

**Step 2: 手動跑 `run_stress.bat`，30s 後用 debug logging 或 tmp print dump counter snapshot**

**Step 3: 記錄 baseline 數字到 `docs/plans/2026-04-24-kcp-multiplayer-traffic-optimization.md` 最底 `## Baseline` 區段（by 本 plan 執行者自己補）**

預期 baseline：TD_STRESS 1000×1000 下 `total_bytes/sec` ~500~1500 KB/sec（視 creep 狀態 / creep movement churn 而定）。

---

## Phase 1: 第一梯隊優化（1 週，worktree `kcp-opt/p1`）

**Accumulated target**：流量 -50%。

### Task 1.1: Creep facing 閾值 3° → 15°

**Files:**
- Modify: `omb/src/tick/creep_tick.rs:134`

**Step 1: 定位現況**

Run: Grep `0.05` in `omb/src/tick/creep_tick.rs`
Expected: 找到 `if (facing.0 - old_facing).abs() > 0.05`

**Step 2: 改閾值 + 抽常數**

加到檔頭：

```rust
/// MOBA 鏡頭下肉眼無感的 facing 變化量（~15°）。
const FACING_BROADCAST_THRESHOLD_RAD: f32 = 0.26;
```

把 `0.05` 替換成 `FACING_BROADCAST_THRESHOLD_RAD`。

**Step 3: 跑 creep_tick 相關 unit test（若有）**

Run: `cargo test --manifest-path omb/Cargo.toml -p omobab tick::creep_tick`
Expected: 過（沒 test 也 OK，不 block）。

**Step 4: 手動 smoke `run.bat`，開一場，目測 creep 轉身仍正常**

**Step 5: Commit**

```bash
git add omb/src/tick/creep_tick.rs
git commit -m "perf(creep): raise facing broadcast threshold 3° → 15°"
```

---

### Task 1.2: 刪死欄位 from JSON payload

**Files:**
- Modify: `omb/src/comp/game_processor.rs:618`（projectile spawn：刪 `damage`、`source_id`）
- Modify: `omb/src/tick/creep_tick.rs:91,118`（creep M：刪 payload 中 `hp`/`max_hp`）
- Modify: `omb/src/state/resource_management.rs`（tower/upgrade：保留 `levels[3]`，刪 `path`/`level`/`name`）
- Modify: `omb/src/comp/game_processor.rs:466-479`（creep/C payload：刪 `name`，改由 tower_templates-style table 查）

**Step 1: 搜尋 projectile C event 建構點**

Run: Grep `"projectile"` in `omb/src/comp/game_processor.rs`
找到 `serde_json::json!({ ... })` 建構的位置。

**Step 2: 改 JSON builder**

```rust
// Before
serde_json::json!({
    "id": proj_id,
    "source_id": src_id,   // 刪
    "target_id": tgt_id,
    "damage": dmg,          // 刪
    "start_pos": [sx, sy],
    "flight_time_ms": ft,
    "directional": dir,
    "end_pos": [ex, ey],
})

// After
serde_json::json!({
    "id": proj_id,
    "target_id": tgt_id,
    "start_pos": [sx, sy],
    "flight_time_ms": ft,
    "directional": dir,
    "end_pos": [ex, ey],
})
```

**Step 3: 同樣處理 creep/M、tower/upgrade、creep/C 的死欄位**

每個檔案一次，改完再跑 `cargo check`。

**Step 4: 前端 parser 是 schema-permissive（serde_json::Value）不會因缺欄位炸，確認方式**

Run: Grep `damage` in `omfx/game/src/lib.rs`（確認 projectile apply 路徑沒有要求必須有 damage）
Expected: 沒有 `["damage"]` 強制讀取，或讀了也有 unwrap_or default。

**Step 5: 跑 `run.bat` 手動 smoke：開一場，確認 projectile 飛行、creep 移動、tower upgrade 都正常**

**Step 6: Commit**

```bash
git add omb/src/comp/game_processor.rs omb/src/tick/creep_tick.rs omb/src/state/resource_management.rs
git commit -m "perf(kcp): drop unused fields from JSON payloads (projectile.damage/source_id, creep.M.hp, tower.upgrade.path/level/name, creep.C.name)"
```

---

### Task 1.3: LZ4 壓縮整合到 `flush_batch`

**Files:**
- Modify: `omb/Cargo.toml`（加 `lz4_flex = "0.11"`）
- Modify: `omoba-core/Cargo.toml`（同）
- Modify: `omb/src/transport/kcp_transport.rs`（`flush_batch` 後壓縮）
- Modify: `omoba-core/src/kcp/framing.rs`（client decompress）

**Step 1: 加依賴**

```toml
# omb/Cargo.toml + omoba-core/Cargo.toml
lz4_flex = { version = "0.11", default-features = false, features = ["frame", "safe-encode", "safe-decode"] }
```

**Step 2: 設計 frame layout**

現有 frame: `[1B tag][4B len BE][payload]`
新 frame: `[1B tag][1B flags][4B len BE][payload]`，`flags` bit0 = lz4 壓縮標誌；其他位元保留。

⚠ **Breaking**：加 1 byte flags 改了 frame 結構，必須 server/client 同步升版。因 P1 還沒到 schema cutover，這個可以用 alternate approach：**在 tag byte 新增 0x81~0x86 對應壓縮版本**（high bit 1 = compressed），原 0x01~0x06 照常用。這樣同一通道可以混發，easier rollout。

```rust
const TAG_COMPRESSED_BIT: u8 = 0x80;
// 送出時若 payload > 128 bytes，壓 lz4，tag |= 0x80
// 收到時若 tag & 0x80，先 lz4 decompress 再按 (tag & 0x7F) dispatch
```

**Step 3: Encoder 端**

在 `flush_batch` 攢完 payload 後：

```rust
const LZ4_THRESHOLD: usize = 128; // 小 payload 壓了反而變大

let (final_tag, final_payload) = if raw_payload.len() >= LZ4_THRESHOLD {
    let compressed = lz4_flex::block::compress_prepend_size(&raw_payload);
    if compressed.len() < raw_payload.len() {
        (tag | TAG_COMPRESSED_BIT, compressed)
    } else {
        (tag, raw_payload)
    }
} else {
    (tag, raw_payload)
};
// write final_tag, final_payload.len() as u32 BE, final_payload
```

**Step 4: Decoder 端 (`omoba-core/src/kcp/framing.rs`)**

```rust
let compressed = (tag & 0x80) != 0;
let base_tag = tag & 0x7F;
let payload = if compressed {
    lz4_flex::block::decompress_size_prepended(&frame_body)?
} else {
    frame_body
};
// continue with base_tag dispatch
```

**Step 5: 加 unit test for round-trip compression**

```rust
// omoba-core/src/kcp/framing.rs 底下加 mod tests
#[test]
fn lz4_frame_roundtrip() {
    let original = vec![0xAB; 1000];  // 高冗餘 → 壓縮率高
    let compressed = lz4_flex::block::compress_prepend_size(&original);
    assert!(compressed.len() < original.len());
    let back = lz4_flex::block::decompress_size_prepended(&compressed).unwrap();
    assert_eq!(back, original);
}
```

Run: `cargo test -p omoba-core --features kcp kcp::framing::tests::lz4_frame_roundtrip`
Expected: PASS

**Step 6: Smoke `run.bat`，目測 frontend 能連上、event 流正常**

**Step 7: 讀 bytes counter snapshot 對比 baseline**

Expected: `total_bytes/sec` 比 P0 baseline 降 30~50%（JSON 壓縮率高）。

**Step 8: Commit**

```bash
git add omb/Cargo.toml omoba-core/Cargo.toml omb/src/transport/kcp_transport.rs omoba-core/src/kcp/framing.rs
git commit -m "perf(kcp): lz4 compression at flush_batch (tag high-bit = compressed)"
```

---

### Task 1.4: Batch window dedupe by (entity_id, event_variant)

**Files:**
- Modify: `omb/src/transport/kcp_transport.rs:92-115`

**Step 1: 找 batch buffer 結構**

目前推測是 `Vec<PendingMsg>`。改成 upsert map：

```rust
struct DedupeKey {
    tag: u8,
    /// event kind hash (穩定 string hash)
    kind_hash: u32,
    /// entity id if event has one, else MSG_COUNTER (no dedupe)
    entity_id: u64,
}

struct BatchBuffer {
    // 順序保持 insertion 順序；replace 保留原 index
    order: Vec<DedupeKey>,
    map: hashbrown::HashMap<DedupeKey, PendingMsg>,
}

impl BatchBuffer {
    fn upsert(&mut self, key: DedupeKey, msg: PendingMsg) {
        if self.map.insert(key, msg).is_none() {
            self.order.push(key);
        }
    }
    fn drain(&mut self) -> Vec<PendingMsg> {
        let out: Vec<_> = self.order.drain(..)
            .map(|k| self.map.remove(&k).unwrap())
            .collect();
        out
    }
}
```

**Step 2: Dedupe-eligible event 清單**

- `creep.M`, `creep.F`, `creep.H`, `creep.S`, `hero.stats` → dedupe
- `entity.death`, `creep.C`, `projectile.C`, `projectile.D`, `game.explosion`, `game.round`, `tower.create`, `tower.upgrade`, `buff.add`, `buff.remove` → **不 dedupe**（語意上每則都要送到）

用 `entity_id = u64::MAX` 代表 "不 dedupe"，每則都 push 獨立 key（用 global counter 的 msg_seq 當 entity_id）。

**Step 3: 手動 smoke `run.bat`**

**Step 4: 讀 counter 確認 creep.M / creep.F 的 msg 數量掉**

Expected: 相對 P1.3 再降 20~30%。

**Step 5: Commit**

```bash
git add omb/src/transport/kcp_transport.rs
git commit -m "perf(kcp): coalesce creep.M/F/H/S and hero.stats within batch window"
```

---

### Task 1.5: Heartbeat 2s → per-player 500ms

**Files:**
- Modify: `omb/src/state/core.rs:118,525-630`（`heartbeat_interval` + push loop）
- Modify: `omb/src/state/resource_management.rs`（heartbeat builder，若有抽出）

**Step 1: 定位 heartbeat 廣播點**

Run: Grep `heartbeat_interval` in `omb/src/state/core.rs`

**Step 2: 改頻率 + per-player 過濾**

```rust
// Before: const HEARTBEAT_INTERVAL: f32 = 2.0;
const HEARTBEAT_INTERVAL: f32 = 0.5;
```

Heartbeat builder 接受 `player_name: &str`，只包含該 player AOI 內的 entity。P5 的 AOI broadphase 還沒做，這裡先**用 Player 的 current camera pos + radius 1200 world unit** 做 naive linear scan（O(entity 數) per player per heartbeat，30 player × 500ms = 60 scan/sec × 1000 entity = 60K entity-check/sec，可接受）。

**Step 3: Heartbeat payload 壓縮**

```rust
// Before (JSON): [{"id": "123", "hp": 450.0, "max_hp": 1000.0}, ...]  ≈ 60 B/entity
// After (JSON 暫時，P2 再轉 binary):
// [{"i":123,"h":450,"m":1000}, ...]  ≈ 22 B/entity
// 更進一步：只送 hp 不送 max_hp，max_hp 改 HeroStatic/CreepCreate 時送一次
// [{"i":123,"h":450}, ...]  ≈ 15 B/entity
```

**Step 4: 手動 smoke**

**Step 5: 讀 counter 對比**

Expected: heartbeat 的 bytes 差不多（頻率 4x 但 payload 被 AOI filter 掉 70% entity + 每 entity 小 4x），或略降。真正放大收益在多人環境。

**Step 6: Commit**

```bash
git add omb/src/state/core.rs omb/src/state/resource_management.rs
git commit -m "perf(heartbeat): 500ms per-player with AOI filter (linear scan placeholder)"
```

---

### Task 1.6: P1 benchmark + 合回 master

**Step 1: 把 `omb/tests/network_bytes.rs` 的 scaffold 填實作**

```rust
#[test]
#[ignore]
fn kcp_bytes_budget_td_stress() {
    // 啟 server in-process with TD_STRESS story
    // 讓它跑 30s
    // 讀 counter snapshot
    let snap = transport.bytes_counter().snapshot();
    let bps = snap.total_bytes as f64 / 30.0;
    println!("P1 bytes/sec = {:.0}", bps);
    // P1 budget：不超過 baseline 的 60%
    assert!(bps < BASELINE_BPS * 0.60, "P1 bytes/sec regression: {}", bps);
}
```

`BASELINE_BPS` 從 Task 0.3 量到的數字填進來。

**Step 2: 跑 test**

Run: `cargo test -p omobab --test network_bytes -- --ignored`
Expected: PASS，bytes/sec 顯示 <60% baseline。

**Step 3: 合回 master**

```bash
cd D:/omoba
git merge --no-ff kcp-opt/p1
git worktree remove ../omoba-kcp-p1
```

**Step 4: `graphify update .`**

---

## Phase 2: Proto schema 重寫 + binary（1~2 週，worktree `kcp-opt/p2`）

**Accumulated target**：-70%（相對 baseline）。

這是整個計畫最大、最 breaking 的一步。拆成多個 step，中間保持 compile。

### Task 2.1: 設計新 `proto/game.proto`

**Files:**
- Modify: `proto/game.proto`

**Step 1: 讀現行 schema**

Run: Read `proto/game.proto`

**Step 2: 備份舊 schema**

```bash
cp proto/game.proto proto/game.proto.v1.bak
```

**Step 3: 寫新 schema**

```proto
syntax = "proto3";
package omoba;

// ===== Primitive helpers =====

message Position16 {
  // scale = 0.25；範圍 ±8191.75；精度 0.25 world unit
  sint32 x_q = 1;  // 實際用 int16 範圍，proto3 用 sint32 encode 省 varint
  sint32 y_q = 2;
}

message Fixed16 {
  // scale = 0.1；範圍 ±3276.7
  sint32 v_q = 1;
}

message AbilityLevelTriple {
  uint32 cur = 1;
  uint32 max = 2;
}

message BuffSnapshot {
  string buff_id = 1;
  // u16 毫秒，0~65535 ms。toggle 型送 0xFFFF。
  uint32 remaining_ms = 2;
  // 任意 JSON 字串；payload 通常小 (<100B)。
  string payload_json = 3;
}

message AbilityMeta {
  string id = 1;
  string name = 2;
  string description = 3;
  repeated Fixed16 cooldown = 4;
  repeated Fixed16 mana_cost = 5;
  repeated Fixed16 cast_range = 6;
  // 其餘 effects key/value 扁平化
  map<string, string> effects = 7;
}

// ===== Events =====

message HeroStatic {
  uint64 id = 1;
  string name = 2;
  string title = 3;
  uint32 base_str = 4;
  uint32 base_agi = 5;
  uint32 base_int = 6;
  repeated AbilityMeta abilities = 7;
}

message HeroHot {
  uint64 id = 1;
  Fixed16 hp = 2;
  Fixed16 max_hp = 3;
  Fixed16 mana = 4;
  Fixed16 max_mana = 5;
  uint32 level = 6;
  uint32 xp = 7;
  uint32 xp_next = 8;
  uint32 gold = 9;
  Fixed16 attack_damage = 10;
  Fixed16 armor = 11;
  Fixed16 magic_resist = 12;
  Fixed16 move_speed = 13;
  uint32 skill_points = 14;
  repeated AbilityLevelTriple ability_levels = 15;
  repeated BuffSnapshot buffs = 16;
}

message CreepCreate {
  uint64 id = 1;
  Position16 pos = 2;
  Fixed16 hp = 3;
  Fixed16 max_hp = 4;
  Fixed16 move_speed = 5;
  string template_key = 6;  // 查本地 CreepTemplate 表拿 name/label
}

message CreepMove {
  uint64 id = 1;
  Position16 target = 2;
  Fixed16 velocity = 3;
  uint64 arrival_tick = 4;
  uint32 facing_q = 5;  // Facing8: 0~255
}

message CreepHp {
  uint64 id = 1;
  Fixed16 hp = 2;
}

message CreepSlow {
  uint64 id = 1;
  Fixed16 move_speed = 2;
}

message CreepFacing {
  uint64 id = 1;
  uint32 facing_q = 2;
}

message EntityDeath {
  uint64 id = 1;
  // 不需 entity_kind：client 本地已知 entity 類型
}

message ProjectileC {
  uint64 id = 1;
  uint64 target_id = 2;
  Position16 start_pos = 3;
  Position16 end_pos = 4;
  uint32 flight_time_ms = 5;
  bool directional = 6;
}

message ProjectileD {
  uint64 id = 1;
}

message TowerCreate {
  uint64 id = 1;
  Position16 pos = 2;
  string template_key = 3;
}

message TowerUpgrade {
  uint64 tower_id = 1;
  repeated uint32 levels = 2;  // [path0, path1, path2]
}

message HeartbeatHp {
  uint64 server_tick = 1;
  uint64 server_wall_ms = 2;
  // 緊湊 HP snapshot：(id, hp) pair list
  repeated uint64 entity_ids = 3;     // id list
  repeated sint32 hps_q = 4;          // Fixed16 quant，與 entity_ids 同長
  // drift-warning 用
  repeated uint64 pos_snap_ids = 5;
  repeated Position16 pos_snap = 6;
}

message GameMeta {
  oneof kind {
    GameRound round = 1;
    GameLives lives = 2;
    GameEnd end = 3;
    GameExplosion explosion = 4;
    TowerTemplates templates = 5;
    MapRegions regions = 6;
    MapPaths paths = 7;
    // ... 其他低頻 meta
  }
}

message GameRound { uint32 round = 1; uint32 total = 2; bool is_running = 3; }
message GameLives { uint32 lives = 1; }
message GameEnd { string winner = 1; }
message GameExplosion { Position16 pos = 1; Fixed16 radius = 2; uint32 duration_ms = 3; }
message TowerTemplates { bytes json_blob = 1; } // 非熱路徑，先留 JSON
message MapRegions { bytes json_blob = 1; }
message MapPaths { bytes json_blob = 1; }

message MapDataChunk { bytes blob = 1; }  // 一次性大 payload，保留

// ===== Top-level =====

message GameEvent {
  uint64 tick = 1;
  uint32 sequence = 2;
  oneof payload {
    HeroStatic hero_static = 10;
    HeroHot hero_hot = 11;
    CreepCreate creep_c = 12;
    CreepMove creep_m = 13;
    CreepHp creep_h = 14;
    CreepSlow creep_s = 15;
    CreepFacing creep_f = 16;
    EntityDeath death = 17;
    ProjectileC proj_c = 18;
    ProjectileD proj_d = 19;
    TowerCreate tower_c = 20;
    TowerUpgrade tower_u = 21;
    GameMeta meta = 22;
    HeartbeatHp hb = 23;
    MapDataChunk map_data = 24;
    // MOBA 預留 30-49
  }
}

// PlayerCommand / Subscribe / State* 沿用現況，不動。
```

**Step 4: `cargo build -p omoba-core --features kcp`**

prost 應該能成功生成 stub。若有 error 改 schema。

**Step 5: Commit schema（單獨一個 commit，方便 review）**

```bash
git add proto/game.proto
git commit -m "feat(proto): redesign GameEvent.oneof for binary + quantization"
```

---

### Task 2.2: 加 quantization helper module

**Files:**
- Create: `omoba-core/src/quant.rs`
- Modify: `omoba-core/src/lib.rs`（`pub mod quant;`）

**Step 1: 寫 quant module**

```rust
//! Quantization helpers for KCP binary protocol.
//! Position16: scale=0.25，int16 範圍；Fixed16: scale=0.1，int16 範圍。

pub const POSITION_SCALE: f32 = 0.25;
pub const FIXED_SCALE: f32 = 0.1;

pub fn pos_quant(v: f32) -> i32 {
    (v / POSITION_SCALE).round().clamp(i16::MIN as f32, i16::MAX as f32) as i32
}
pub fn pos_dequant(q: i32) -> f32 {
    q as f32 * POSITION_SCALE
}
pub fn fixed_quant(v: f32) -> i32 {
    (v / FIXED_SCALE).round().clamp(i16::MIN as f32, i16::MAX as f32) as i32
}
pub fn fixed_dequant(q: i32) -> f32 {
    q as f32 * FIXED_SCALE
}

/// radian → u8 Facing8（0..256 對應 0..2π）
pub fn facing_quant(rad: f32) -> u32 {
    let norm = rad.rem_euclid(std::f32::consts::TAU);
    let q = (norm / std::f32::consts::TAU * 256.0).round() as u32;
    q & 0xFF
}
pub fn facing_dequant(q: u32) -> f32 {
    (q & 0xFF) as f32 / 256.0 * std::f32::consts::TAU
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn pos_roundtrip_precision() {
        for v in [-4000.0, -0.1, 0.0, 0.25, 1234.5, 8000.0] {
            let q = pos_quant(v);
            let back = pos_dequant(q);
            assert!((back - v).abs() < 0.125, "v={} back={}", v, back);
        }
    }
    #[test] fn facing_wraps() {
        assert_eq!(facing_quant(0.0), facing_quant(std::f32::consts::TAU));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p omoba-core quant`
Expected: PASS

**Step 3: Commit**

```bash
git add omoba-core/src/quant.rs omoba-core/src/lib.rs
git commit -m "feat(core): quantization helpers for kcp binary protocol"
```

---

### Task 2.3: Server encode path：JSON builder → prost

**Files:**
- Modify: `omb/src/state/core.rs`（hero.stats / heartbeat push）
- Modify: `omb/src/state/resource_management.rs`（`build_hero_stats_payload` → `build_hero_hot` + `build_hero_static`）
- Modify: `omb/src/tick/creep_tick.rs`（creep M/F/S event emit）
- Modify: `omb/src/tick/buff_tick.rs:75,107`（buff add/remove，用 Buff*Event 包在 HeroHot 或獨立 event）
- Modify: `omb/src/comp/game_processor.rs:127-627`（explosion/death/spawn outcome）
- Modify: `omb/src/transport/kcp_transport.rs`（flush_batch encode 改用 prost）

本 task 大，按 event 類拆 sub-step。

**Step 1: `build_hero_hot()` / `build_hero_static()` 實作**

把 `build_hero_stats_payload` 拆成兩個 builder，分別產 `HeroHot` / `HeroStatic` prost message。`resource_management.rs` 底 `pub use` 出來。

```rust
pub fn build_hero_hot(world: &World, hero_entity: Entity) -> HeroHot {
    // 從 BuffStore 聚合出衍生屬性
    // ...
    HeroHot {
        id: hero_entity_to_u64(hero_entity),
        hp: Some(Fixed16 { v_q: fixed_quant(hp) }),
        max_hp: Some(Fixed16 { v_q: fixed_quant(max_hp) }),
        // ...
        buffs: buff_list.into_iter().map(|b| BuffSnapshot {
            buff_id: b.id,
            remaining_ms: if b.infinite { 0xFFFF } else { (b.remaining * 1000.0).min(65535.0) as u32 },
            payload_json: b.payload_json,
        }).collect(),
    }
}

pub fn build_hero_static(world: &World, hero_entity: Entity) -> HeroStatic { /* ... */ }
```

`hero_entity_to_u64` helper：`(entity.id() as u64) << 32 | entity.gen().id() as u64`。

**Step 2: Creep events encoder**

```rust
// creep_tick.rs
let ev = GameEvent {
    tick: current_tick,
    sequence: next_seq(),
    payload: Some(game_event::Payload::CreepM(CreepMove {
        id: creep_entity_to_u64(ent),
        target: Some(Position16 { x_q: pos_quant(target.x), y_q: pos_quant(target.y) }),
        velocity: Some(Fixed16 { v_q: fixed_quant(move_speed) }),
        arrival_tick: arrival_tick,
        facing_q: facing_quant(facing.0),
    })),
};
transport.enqueue(ev, EventKind::CreepM, Some(creep_entity_to_u64(ent)));
```

逐一處理 `creep.M`、`creep.F`、`creep.H`、`creep.S`、`creep.C`。

**Step 3: Projectile / Death / GameMeta 改 prost**

同上模式。

**Step 4: Transport encode**

```rust
// kcp_transport.rs
fn encode_event(ev: &GameEvent) -> Vec<u8> {
    let mut buf = Vec::with_capacity(ev.encoded_len());
    ev.encode(&mut buf).expect("prost encode should never fail");
    buf
}
```

tag 保持 0x02（GameEvent），compressed bit 維持 P1 設計。

**Step 5: Build check**

```bash
cargo build --manifest-path omb/Cargo.toml -p omobab
```

**Step 6: Smoke：此時 client 還沒切換，會解碼錯誤 → 預期**。下 task 換 client。

**Step 7: Commit（還沒 runnable，但 server 端完成）**

```bash
git add omb/src/
git commit -m "feat(omb): encode GameEvent as prost oneof (client-side switch pending)"
```

---

### Task 2.4: Client decode path：omoba-core + omfx

**Files:**
- Modify: `omoba-core/src/grpc/client.rs`（GameEventData 改承 prost oneof）
- Modify: `omoba-core/src/kcp/client.rs`（tag 0x02 dispatch → prost decode）
- Modify: `omfx/game/src/lib.rs:2509-2540`（`apply_event` 從 match `(msg_type, action)` string 改 match oneof variant）

**Step 1: `GameEventData` 改型**

```rust
// omoba-core/src/grpc/client.rs
// Before: pub struct GameEventData { pub topic: String, pub msg_type: String, pub action: String, pub payload_json: String, ... }
// After: 直接用 proto 生成的 GameEvent（re-export）。
pub use crate::proto::GameEvent;
pub use crate::proto::game_event::Payload as GameEventPayload;
```

原 `payload_bytes` / `payload_json` 欄位全刪。

**Step 2: omfx NetworkBridge**

```rust
// omfx/game/src/lib.rs (around line 464-579)
fn on_game_event(&mut self, ev: GameEvent) {
    let Some(payload) = ev.payload else { return };
    use omoba_core::proto::game_event::Payload::*;
    match payload {
        HeroStatic(s) => self.apply_hero_static(s),
        HeroHot(h) => self.apply_hero_hot(h),
        CreepC(c) => self.apply_creep_create(c),
        CreepM(m) => self.apply_creep_move(m),
        CreepH(h) => self.apply_creep_hp(h),
        CreepS(s) => self.apply_creep_slow(s),
        CreepF(f) => self.apply_creep_facing(f),
        Death(d) => self.apply_death(d),
        ProjC(p) => self.apply_projectile_create(p),
        ProjD(p) => self.apply_projectile_destroy(p),
        TowerC(t) => self.apply_tower_create(t),
        TowerU(t) => self.apply_tower_upgrade(t),
        Meta(m) => self.apply_meta(m),
        Hb(h) => self.apply_heartbeat(h),
        MapData(m) => self.apply_map_data(m),
    }
}
```

每個 `apply_*` 把舊 `apply_event` 裡對應分支的邏輯搬過來，並用 `pos_dequant`、`fixed_dequant`、`facing_dequant` 還原 float。

**Step 3: HeroStatic client cache**

```rust
// omfx Game struct 加：
hero_static_cache: hashbrown::HashMap<u64, HeroStatic>,

// apply_hero_static：update cache
// apply_hero_hot：render 時查 cache 拿 name/title/abilities 描述
```

**Step 4: EventBuffer 排序改用 `ev.tick`**

`omfx/game/src/lib.rs:131-186` 的 BinaryHeap<Reverse<(tick, seq, GameEvent)>>。

**Step 5: Build check**

```bash
cargo build --manifest-path omfx/Cargo.toml
```

**Step 6: 端到端 smoke**

Run: `run.bat`
Expected: 前端正常顯示 creep / hero / projectile；無 prost decode 錯誤。

**Step 7: 檢查 bytes counter**

Expected: 相對 P1 再降 30~40%（JSON → binary + quantization）。

**Step 8: Commit**

```bash
git add omoba-core/src/ omfx/game/src/lib.rs
git commit -m "feat(client): consume GameEvent oneof directly; drop payload_bytes JSON"
```

---

### Task 2.5: P2 benchmark + submodule bump

**Step 1: 更新 `network_bytes.rs` P2 budget**

```rust
// 改 assert：bps < BASELINE_BPS * 0.30（-70% accumulated）
```

**Step 2: 跑 stress test**

**Step 3: `omb` / `omoba-core` / `omfx` submodule 各自 commit 完 bump 主 repo 指標 in a single master commit**

```bash
cd D:/omoba
git add omb omoba-core omfx proto
git commit -m "chore: bump omb+omfx+omoba-core for prost binary GameEvent cutover"
```

**Step 4: 合 p2 worktree 回 master**

---

## Phase 3: hero.stats 冷熱拆完整化（0.5 週，worktree `kcp-opt/p3`）

**Accumulated target**：-78%。

P2 已把 `HeroHot` / `HeroStatic` 拆出 type。P3 確保**推送時機正確**。

### Task 3.1: 定位 HeroStatic 變化觸發點

**Files:**
- Modify: `omb/src/state/core.rs`（hero create 初始化階段 push HeroStatic）
- Modify: `omb/src/comp/game_processor.rs`（level up、ability learn → push HeroStatic）
- Modify: 其他 item / title 變更路徑（grep `title`, `learn`, `upgrade_ability`）

**Step 1: 檢索現行 push hero.stats 的 call site**

Run: Grep `hero.stats` in `omb/src/`
共 4 處（per plan memory）：`core.rs:574`、`core.rs:750,756`、`game_processor.rs:299-319`。

**Step 2: 分類每個 site 應該推哪種**

| Site | Static? | Hot? |
|---|---|---|
| `core.rs:574` 每 0.3s broadcast | ❌ | ✅ |
| `core.rs:750,756` 初始化 | ✅（first time）| ✅ |
| `game_processor.rs:299-319` death/heal outcome | ❌ | ✅ |
| Level up / ability learn（P3 新增）| ✅ | ✅ |

**Step 3: Level up / ability learn hook**

Grep 找 level up 路徑：`fn level_up`, `xp_next`, `add_ability_point` 等。加一行 push HeroStatic。

**Step 4: 驗證**：開場時收到 HeroStatic 一次，後續 0.3s 只見 HeroHot；level up 時看到 HeroStatic + HeroHot 各一次。

透過 client 加暫時 debug log 在 `apply_hero_static` / `apply_hero_hot` 計次。

**Step 5: Commit**

```bash
git add omb/src/
git commit -m "perf(hero): push HeroStatic on create/level-up/ability-learn only"
```

---

### Task 3.2: HeroHot 欄位再瘦身

**Step 1: audit 當前 HeroHot 欄位**

檢查每個欄位是否真正 0.3s 頻率下會變。`xp_next`、`skill_points` 變化頻率低，改放 HeroStatic？

**決策**：
- `level`、`xp`、`xp_next`、`skill_points`、`ability_levels` → 搬到 HeroStatic（level up 時才推）
- `gold` → HeroHot 保留（TD/MOBA 都常變）
- `mana` / `max_mana` → 若 MVP 無 mana 概念，暫可移除

**Step 2: 改 proto schema**

回頭改 `proto/game.proto`，把欄位搬家。
⚠ 這是 breaking schema change —— 要 same-commit 升版 3 個 submodule。

**Step 3: 相應修改 builder 與 apply\*()**

**Step 4: Smoke + commit**

```bash
git add proto/ omb/ omfx/ omoba-core/
git commit -m "refactor(proto): move level/xp/skill_points from HeroHot to HeroStatic"
```

---

### Task 3.3: P3 benchmark + 合回

**Step 1: 更新 `network_bytes.rs` P3 budget（bps < BASELINE_BPS * 0.22）**

**Step 2: 跑 stress test**

Expected: PASS，accumulated -78%。

**Step 3: 合回 master + submodule bump**

---

## Phase 4: creep/M velocity 外推（1~1.5 週，worktree `kcp-opt/p4`）

**Accumulated target**：-85%。

### Task 4.1: Server 改為 waypoint-切換 event-驅動

**Files:**
- Modify: `omb/src/tick/creep_tick.rs:91,118`

**Step 1: 理解現況**

每 tick creep_tick 都會更新 `target_pos`，現在改成「只在 waypoint 切換 / path 重算 / slow 狀態變化」才 emit CreepMove。

**Step 2: 加 CreepMoveState 記憶**

在 creep 的 component 內：

```rust
pub struct CreepMoveBroadcast {
    last_broadcast_target: Option<Vec2>,
    last_broadcast_velocity: f32,
    last_broadcast_tick: u64,
}
```

每 tick 檢查：
- 若 `current_target != last_broadcast_target` → emit
- 若 `current_velocity / last_broadcast_velocity` 差距 > 5% → emit（slow / haste）
- 其他情況 → 不 emit

**Step 3: `arrival_tick` 計算**

```rust
let dist = (current_pos - target).magnitude();
let travel_ticks = (dist / velocity / tick_dt).ceil() as u64;
let arrival_tick = current_tick + travel_ticks;
```

**Step 4: 驗證**：1000 creep 下 CreepMove msg/sec 應從千級降至百級。

**Step 5: Commit**

```bash
git add omb/src/tick/creep_tick.rs omb/src/comp/
git commit -m "perf(creep): emit CreepMove only on waypoint/velocity change"
```

---

### Task 4.2: Client velocity 外推 render

**Files:**
- Modify: `omfx/game/src/lib.rs:1321-1323`

**Step 1: 每 frame 算外推 position**

```rust
// 現況：pos = lerp(prev, target, elapsed / lerp_duration)
// 新：pos = start_pos + (cur_tick - start_tick).as_secs_f32() * velocity * dir_to_target
// 若 cur_tick >= arrival_tick → pos = target（鎖定）
fn render_creep_pos(&self, c: &CreepMoveState, cur_tick: u64, tick_dt: f32) -> Vec2 {
    if cur_tick >= c.arrival_tick {
        return c.target;
    }
    let dir = (c.target - c.start_pos).normalized();
    let elapsed = (cur_tick - c.start_tick) as f32 * tick_dt;
    let traveled = (c.velocity * elapsed).min((c.target - c.start_pos).magnitude());
    c.start_pos + dir * traveled
}
```

**Step 2: 驗證視覺**

Run: `run.bat`
Expected: creep 移動平滑，無 stutter。

**Step 3: Commit**

```bash
git add omfx/game/src/lib.rs
git commit -m "feat(client): velocity-based creep extrapolation"
```

---

### Task 4.3: Heartbeat 加 position_snapshot drift-warning

**Files:**
- Modify: `omb/src/state/core.rs`（heartbeat builder）
- Modify: `omfx/game/src/lib.rs`（apply_heartbeat → snap drift entity）

**Step 1: Server 端：每 heartbeat 抽樣 AOI 內 10% creep 放 `pos_snap`**

```rust
let mut pos_snap = vec![];
for (i, c) in visible_creeps.iter().enumerate() {
    if i % 10 == 0 {  // 1/10 sample
        pos_snap.push((c.id, c.current_pos));
    }
}
```

**Step 2: Client 端：比對 `pos_snap` 與本地外推位置**

```rust
for (id, server_pos) in hb.pos_snap_ids.iter().zip(hb.pos_snap.iter()) {
    let local_pos = self.render_creep_pos(/* ... */);
    let drift = (server_pos - local_pos).magnitude();
    if drift > 2.0 {  // >2 world unit
        // 硬 snap 回 server 位置，並 reset start_pos/start_tick
        self.creep_states[id].snap_to(server_pos, current_tick);
    }
}
```

**Step 3: Smoke + commit**

```bash
git add omb/src/state/core.rs omfx/game/src/lib.rs
git commit -m "feat(heartbeat): position drift-correction snapshot"
```

---

### Task 4.4: P4 benchmark

Budget: bps < BASELINE * 0.15（-85%）。

---

## Phase 5: Per-player AOI broadphase（1 週，worktree `kcp-opt/p5`）

**Accumulated target**：-88%。

### Task 5.1: 建立 `systems/aoi.rs` spatial hash

**Files:**
- Create: `omb/src/systems/aoi.rs`
- Modify: `omb/src/systems/mod.rs`（`pub mod aoi;`）

**Step 1: 設計 cell grid**

```rust
pub const AOI_CELL_SIZE: f32 = 256.0;

pub struct AoiGrid {
    cells: hashbrown::HashMap<(i32, i32), Vec<AoiEntry>>,
}
pub struct AoiEntry { pub entity_id: u64, pub pos: Vec2 }

impl AoiGrid {
    pub fn rebuild<'a>(&mut self, entries: impl Iterator<Item = AoiEntry>) {
        self.cells.clear();
        for e in entries {
            let key = (
                (e.pos.x / AOI_CELL_SIZE).floor() as i32,
                (e.pos.y / AOI_CELL_SIZE).floor() as i32,
            );
            self.cells.entry(key).or_default().push(e);
        }
    }
    pub fn query<F: FnMut(u64)>(&self, center: Vec2, radius: f32, mut cb: F) {
        let r_cells = (radius / AOI_CELL_SIZE).ceil() as i32;
        let cx = (center.x / AOI_CELL_SIZE).floor() as i32;
        let cy = (center.y / AOI_CELL_SIZE).floor() as i32;
        for dx in -r_cells..=r_cells {
            for dy in -r_cells..=r_cells {
                if let Some(v) = self.cells.get(&(cx+dx, cy+dy)) {
                    for e in v {
                        if (e.pos - center).magnitude_squared() < radius*radius {
                            cb(e.entity_id);
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: 加 `AoiGrid` 為 specs Resource**

每 tick 開頭重建（`systems/aoi.rs::AoiRebuildSystem`）。

**Step 3: Unit test**

```rust
#[test]
fn aoi_query_finds_nearby() {
    let mut g = AoiGrid::default();
    g.rebuild([
        AoiEntry { entity_id: 1, pos: Vec2::new(100.0, 100.0) },
        AoiEntry { entity_id: 2, pos: Vec2::new(2000.0, 2000.0) },
    ].into_iter());
    let mut hits = vec![];
    g.query(Vec2::new(0.0, 0.0), 500.0, |id| hits.push(id));
    assert_eq!(hits, vec![1]);
}
```

**Step 4: Commit**

```bash
git add omb/src/systems/aoi.rs omb/src/systems/mod.rs
git commit -m "feat(aoi): spatial hash broadphase resource"
```

---

### Task 5.2: Transport 層 AOI filter 所有全廣播訊息

**Files:**
- Modify: `omb/src/transport/kcp_transport.rs:156-183`

**Step 1: 在 enqueue 介面帶 `broadcast_policy`**

```rust
pub enum BroadcastPolicy {
    All,                    // GameMeta: round/lives/end
    AoiEntity(u64),         // 事件綁定一個 entity，用該 entity 的 pos 查 AOI
    AoiPoint(Vec2),         // 例如 explosion，用事件本身的 pos
    PlayerOnly(String),     // 命中特定 player 的 inventory/mana 類
}
```

**Step 2: `flush_batch` 時按 policy dispatch**

```rust
fn targets_for(policy: &BroadcastPolicy, aoi: &AoiGrid, players: &[PlayerSession]) -> Vec<usize> {
    match policy {
        BroadcastPolicy::All => (0..players.len()).collect(),
        BroadcastPolicy::AoiEntity(id) => {
            let pos = world.get_entity_pos(*id)?;
            players.iter().enumerate()
                .filter(|(_, p)| (p.camera - pos).magnitude() < 1200.0)
                .map(|(i, _)| i).collect()
        }
        // ...
    }
}
```

**Step 3: 各 event call site 補 policy**

- `creep.M/F/H/S/C`、`HeroHot`、`TowerCreate/Upgrade`、`Projectile*`、`EntityDeath` → `AoiEntity`
- `HeartbeatHp` → per-player（每 player 已各自生成）
- `GameMeta.round/lives/end` → `All`
- `GameMeta.explosion` → `AoiPoint`

**Step 4: Smoke**

**Step 5: Commit**

```bash
git add omb/src/
git commit -m "feat(kcp): per-player AOI filter at transport layer"
```

---

### Task 5.3: Fan-out `Arc<[u8]>` 共用避免重複 encode

**Files:**
- Modify: `omb/src/transport/kcp_transport.rs`

**Step 1: encode 一次，N session 共用**

```rust
let encoded: Arc<[u8]> = Arc::from(encode_event(&ev).into_boxed_slice());
for &target_idx in &target_players {
    players[target_idx].outbox.push(encoded.clone());  // Arc::clone 不複製 bytes
}
```

**Step 2: Unit test**

```rust
#[test]
fn fanout_shares_bytes() {
    let arc1 = Arc::from([1u8, 2, 3, 4].as_slice());
    let arc2: Arc<[u8]> = Arc::clone(&arc1);
    assert_eq!(Arc::as_ptr(&arc1), Arc::as_ptr(&arc2));
}
```

**Step 3: Commit**

```bash
git add omb/src/transport/kcp_transport.rs
git commit -m "perf(kcp): Arc<[u8]> fanout reuse across player sessions"
```

---

### Task 5.4: stress_aoi.rs 30 virtual player 測試

**Files:**
- Create: `omb/tests/stress_aoi.rs`

**Step 1: 寫 test**

```rust
#[test]
#[ignore]
fn aoi_filtered_bytes_30_players() {
    // 啟 server with TD_STRESS
    // 手動插入 30 個 fake PlayerSession，camera 分布於 map 均勻
    // server run 30s
    // 讀 per-player counter，assert mean bytes/sec/player < 150_000
    // assert peak 不超過 mean × 3
}
```

**Step 2: 跑**

Run: `cargo test -p omobab --test stress_aoi -- --ignored`
Expected: PASS。

**Step 3: Commit**

```bash
git add omb/tests/stress_aoi.rs
git commit -m "test(kcp): 30 virtual player AOI stress"
```

---

## Phase 6: 事件驅動 batch + SequenceGap resync（0.5 週，worktree `kcp-opt/p6`）

流量削減同 P5，但改善延遲 UX。

### Task 6.1: Batch window 兩段 threshold

**Files:**
- Modify: `omb/src/transport/kcp_transport.rs:92`

**Step 1: 引入 urgency level**

```rust
pub enum EventUrgency { Urgent, Normal }

// enqueue 接 urgency；event 分類：
// Urgent: EntityDeath, CreepCreate, ProjectileC, ProjectileD, GameMeta::Explosion/End
// Normal: CreepMove/Facing/Hp/Slow, HeroHot, HeartbeatHp
```

**Step 2: 兩 threshold**

```rust
const MIN_BATCH_MS: u64 = 10;
const MAX_BATCH_MS: u64 = 33;

// 進 Urgent：立即 flush（跳過 batch window）
// 進 Normal：
//   - 若 batch 空 → 啟計時 MIN_BATCH_MS
//   - 若 batch 已有東西且距今 < MIN_BATCH_MS → 等到 MIN_BATCH_MS flush
//   - 硬上限 MAX_BATCH_MS flush
```

**Step 3: Commit**

```bash
git add omb/src/transport/kcp_transport.rs
git commit -m "perf(kcp): two-tier batch window (urgent=flush now, normal=10~33ms)"
```

---

### Task 6.2: SequenceGap resync

**Files:**
- Modify: `omoba-core/src/kcp/client.rs`（client 端偵測 gap）
- Modify: `omb/src/transport/kcp_transport.rs`（server 處理 StateReq）

**Step 1: client 記 last_seq per session**

```rust
if ev.sequence != self.last_seq.wrapping_add(1) {
    log::warn!("seq gap: expected {}, got {}", self.last_seq.wrapping_add(1), ev.sequence);
    self.send_state_req();  // tag 0x05
}
self.last_seq = ev.sequence;
```

**Step 2: server StateReq handler**

收到 tag 0x05 後，送該 player 的 full state snapshot（現況 `push_hero_stats_if_needed` + 全 AOI entity create events + 當前 round/lives meta）。

**Step 3: Commit**

```bash
git add omoba-core/src/kcp/client.rs omb/src/transport/kcp_transport.rs
git commit -m "feat(kcp): sequence-gap detection + StateReq full resync"
```

---

### Task 6.3: P6 final benchmark + 整合回 master

**Step 1: 跑整套 stress test**

- `network_bytes` single-player
- `stress_aoi` 30 virtual players
- `run.bat` UX smoke：projectile / death / skill cast 反應應比 P0 baseline 更即時（因 urgent 立即 flush）

**Step 2: 所有 submodule bump，final commit 主 repo**

```bash
cd D:/omoba
git add omb omfx omoba-core proto docs/plans/
git commit -m "chore: kcp multiplayer traffic optimization P1-P6 complete (-85~88% bytes)"
```

**Step 3: 收尾 graphify**

Run: `graphify update .`

---

## Global Verification Matrix

每個 phase 結束必須全通過：

| 檢查 | 指令 | 預期 |
|---|---|---|
| omb build (kcp default) | `cargo build --manifest-path omb/Cargo.toml -p omobab` | PASS |
| omb build (mqtt) | `cargo build --manifest-path omb/Cargo.toml -p omobab --no-default-features --features mqtt` | PASS |
| omb build (grpc) | `cargo build --manifest-path omb/Cargo.toml -p omobab --no-default-features --features grpc` | PASS |
| omoba-core build | `cargo build -p omoba-core --features kcp` | PASS |
| omfx build | `cargo build --manifest-path omfx/Cargo.toml` | PASS |
| Unit tests | `cargo test --manifest-path omb/Cargo.toml -p omobab` | PASS |
| network_bytes budget | `cargo test -p omobab --test network_bytes -- --ignored` | PASS（budget per phase） |
| UX smoke | `run.bat` → 玩 3 分鐘 | 塔/兵/技能/死亡/爆炸正常，render 無 stutter |

Phase 2 之後額外：
- `cargo test -p omoba-core quant` PASS
- `cargo test -p omobab --test stress_aoi -- --ignored`（P5 起）PASS

---

## Baseline（2026-04-24 實測）

**環境**：`run_stress.bat` on Windows 11, TD_STRESS 場景 1000 tower + 1000 creep，single omfx_player 視角（viewport 3235×1820 padded 1.3x，看得到全圖大部分）；release build；non-deterministic —— 每次跑因戰鬥展開略有差異。

**Stress 場景流量動態**：不是穩態。creep 從 waypoint 起點慢慢流入視野；流量隨可見 entity 線性上升。採「late-game」作為 budget reference。

### 5-second window 觀察

| 時間窗 | visible creep | bytes/s | msgs/s |
|---|---|---|---|
| 早期 (spawn 剛結束) | ~180 | 59 KB/s | 334 |
| 中期 | ~320 | 118 KB/s | 889 |
| **Late (reference)** | ~490 | **206 KB/s** | **1631** |
| 預估 peak (visible ~700+) | ~700 | ~280 KB/s | ~2200 |

### 累積 70s breakdown

| Topic | Total bytes | Msgs | B/msg | % of total | 主要 phase 對治 |
|---|---|---|---|---|---|
| `heartbeat.tick` | 2,600,306 | 60 | 43,338 | **36.2%** | P1 (per-player 500ms + HP-only compact) |
| `projectile.C` | 1,659,413 | 7127 | 233 | 23.1% | P1 (drop damage/source_id) + P2 (Position16 binary) |
| `entity.F` | 1,007,013 | 13295 | 76 | 14.0% | P1 (3°→15° threshold; ≈-70%) |
| `creep.H` | 539,459 | 6992 | 77 | 7.5% | P1 heartbeat-吸收 |
| `tower.create` | 385,900 | 2000 | 193 | 5.4% | 一次性 spawn，忽略 steady-state |
| `projectile.D` | 369,252 | 7101 | 52 | 5.1% | P2 binary |
| `hero.stats` | 248,553 | 376 | 661 | 3.5% | P3 冷熱拆 (≈-80%) |
| `creep.create` | 208,040 | 485 | 429 | 2.9% | P2 (template_key 取代 name) |
| `creep.C` | 82,134 | 486 | 169 | 1.1% | P2 |
| `creep.M` | 70,327 | 831 | 85 | 1.0% | P4 (velocity 外推) |

**70s cumulative: 7,174,298 B / 70s ≈ 102 KB/s avg**

### 用作 budget 的基準

```
BASELINE_BPS_STEADY     = 206_000  // late-game 5s window (tick=3378 前後)
BASELINE_BPS_AVG_70S    = 102_000  // 整段 stress run 平均
BASELINE_MSG_SEC_STEADY = 1631     // late-game
```

`omb/tests/network_bytes.rs` 實作時以 **BASELINE_BPS_STEADY = 206 KB/s** 當 baseline（取最壞情況；P1~P6 的 budget 則乘 0.5 / 0.3 / 0.22 / 0.15 / 0.12 / 0.12）。

### 關鍵觀察

1. **heartbeat 一則 43 KB** —— 因為 JSON 包了 514 entity × ~80 B。P1 per-player 500ms snapshot 只含可見 entity ID + HP（6 B/entity）可把 heartbeat 流量降 80%+。
2. **projectile 三項合計 28%** —— P2 的 `Position16` + 刪 `damage`/`source_id` 預估可把 proj_c 230B → 40B（-80%）。
3. **entity.F 每則 76 B 但量大（13K msgs/70s）** —— 閾值 3°→15° 直接打掉 70% 量，累積 -10% total bytes。
4. **tower.create 一次性** —— 385 KB 都在 stress 啟動的 2s 內，不影響 steady-state。後續 P5 把它移到 session-snapshot（StateResp）也可再優化，但 OOS for now。
5. **現況單 player 就已 200 KB/s late-game** —— 30 player × 共享戰場無 AOI 若照現況全廣播，流量會線性放大到 6 MB/s，完全不能接受。**AOI (P5) + binary (P2) + heartbeat compaction (P1) 必做**。

---

## Out of scope（定義清楚）

- Approach C 的 Snapshot diff protocol（Quake 式）
- MQTT / gRPC pipeline 改造（保留 JSON）
- zstd 壓縮
- Interest tier (near/mid/far)
- PvP 防作弊專用 AOI 信任邊界

---

## Execution notes

- **每 phase 獨立 worktree**；`superpowers:using-git-worktrees` 開。
- 本 plan 每個 Task 都 bite-sized（2~15 分鐘），按 TDD 或 minimum-change 交付。
- 遇到真實 code 與 plan 假設不符（例如現行 `payload_bytes` 實際 layout 略不同）→ 先用 Grep/Read 釐清，必要時更新本 plan 文件再走。
- Commit 頻率：每個 Step 結尾有 commit 就 commit，不要攢多 task 才 commit。
- 若某 phase 的 bytes 削減未達預期 ± 10%，**暫停**、回查假設，更新本 plan，經使用者 review 再繼續。
