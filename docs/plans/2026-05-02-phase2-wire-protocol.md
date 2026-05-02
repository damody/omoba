# Phase 2 — Lockstep Wire Protocol Scaffolding

> **For Claude:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development。

**Goal:** 建 8 人 Server-Paced Lockstep 的 wire layer scaffolding — 新 KCP tags 0x10-0x16 + proto messages + omb `lockstep/` module + omfx 最小 lockstep client。**舊 GameEvent broadcast (tags 0x01-0x07) 並行保留**直到 Phase 3 omfx 變 simulator 才砍。

**Architecture:** 新 `lockstep::InputBuffer` collect 玩家 input targeted at tick T+3 (50ms input delay)。`lockstep::TickBroadcaster` 60Hz pace broadcast TickBatch（即使 empty payload）。Server 同時維持舊 30Hz GameEvent broadcast for omfx render（並行 thread）。omfx 加 `LockstepClient` send InputSubmit + receive TickBatch (logging only, no sim consume yet — Phase 3 才實作 sim)。

**Tech Stack:** prost 0.12 / tokio_kcp / 8 cross-OS pin hashes from Phase 0/1。`omoba_sim::SimRng / Fixed32 / Vec2 / Angle / state_hash::hash_sorted_by_id` 全 Phase 1 已 ship。

---

## Context（核心設計決策）

### Phase 2 = scaffolding, not cutover
舊 KCP tags 0x01-0x07 (PlayerCommand / GameEvent / CommandAck / Subscribe / GameStateRequest / GameStateResponse / ViewportUpdate) **完全保留**。omfx 仍純 renderer，繼續走舊 GameEvent broadcast。Phase 2 只是**並行**建 lockstep pipe。

**為何不一次切**：
1. omfx 砍 GameEvent receive → 變 silent screen 直到 Phase 3 實作 sim
2. omb-mcp 用 0x05/0x06 query — 不能斷
3. base_content scripts 仍透過 dispatch 跑 — 不影響

**何時 cutover**：Phase 3 omfx 變 simulator + renderer 後，omb 砍 GameEvent broadcast，clean up 76 PHASE 2 markers。

### 新 protocol layout

| Tag | Direction | Message | Payload |
|---|---|---|---|
| 0x10 | C→S | InputSubmit | `{ player_id: u32, target_tick: u32, input: PlayerInput }` |
| 0x11 | S→C | TickBatch | `{ tick: u32, inputs: Vec<(u32, PlayerInput)>, server_events: Vec<ServerEvent> }` |
| 0x12 | S→C | StateHash | `{ tick: u32, hash: u64 }` (every 600 ticks = 10s @ 60Hz) |
| 0x13 | C→S | JoinRequest | `{ player_name: string, role: Player\|Observer }` |
| 0x14 | S→C | GameStart | `{ start_tick: u32, master_seed: u64, initial_state: SimSnapshot }` |
| 0x15 | C→S | SnapshotReq | `{ from_tick: u32 }` (observer only) |
| 0x16 | S→C | SnapshotResp | `{ tick: u32, state: SimSnapshot }` |

### `PlayerInput` oneof 設計

```proto
message PlayerInput {
  oneof action {
    NoOp no_op = 1;
    MoveTo move_to = 2;
    AttackTarget attack_target = 3;
    CastAbility cast_ability = 4;
    TowerPlace tower_place = 5;
    TowerUpgrade tower_upgrade = 6;
    TowerSell tower_sell = 7;
    ItemUse item_use = 8;
  }
}

message NoOp {}
message MoveTo { Position16 target = 1; }
message AttackTarget { uint32 target_id = 1; }
message CastAbility { uint32 ability_index = 1; Vec2I target = 2; }  // Vec2I uses Fixed32 raw (sint32 each)
// ... etc
```

### `Vec2I` 取代 `Vec2f`

新 quantized vec for lockstep wire = Fixed32 raw int32 pair。已有 Position16 (0.25 unit) 但 Fixed32 raw 是 1024-scale。為 lockstep 加：
```proto
message Vec2I {
  sint32 x = 1;  // Fixed32::raw()
  sint32 y = 2;
}
message FixedI {
  sint32 raw = 1;  // Fixed32::raw()
}
```

### Server tick / pacer

omb 既有 30Hz dispatcher loop（specs dispatcher）。Phase 2 加 **獨立 60Hz tick scheduler**：
- 收 InputSubmit、buffer for tick T+3
- 每 16.66ms broadcast TickBatch (即使 empty)
- 不影響 既有 30Hz simulation tick — Phase 2 不砍 simulation

實際上 60Hz pacer 跟 30Hz sim 共存：
- sim tick: 30Hz 跑 ECS dispatch + GameEvent broadcast (legacy)
- lockstep tick: 60Hz pace TickBatch 廣播 (new path)
- 兩條 broadcast pipe 各自 thread

**Phase 3 後**：30Hz sim → 砍；60Hz pacer 跑 server-side sim + broadcast TickBatch。

### MasterSeed source

Phase 1c.3 已加 `MasterSeed(u64)` resource (默認 `0xDEAD_BEEF_CAFE_BABE`)。Phase 2 GameStart message 帶 master_seed，server 啟動時用 `MasterSeed(rand_u64())` 隨機初始化（單機 dev 用固定 seed 也 OK）。Client JoinRequest 收到 GameStart 後 cache server's master_seed (但 Phase 2 client 不用，Phase 3 client sim 才用)。

---

## Tasks

### Task 2.1: Proto schema 擴充

**Files**:
- Modify: `D:\omoba\proto\game.proto`
- 新增 messages: `Vec2I`, `FixedI`, `PlayerInput` (oneof + variants), `InputSubmit`, `TickBatch`, `ServerEvent` (oneof, e.g., PlayerJoin / PlayerLeave / WaveStart / etc.), `StateHash`, `JoinRequest` (with role enum), `GameStart`, `SnapshotReq`, `SnapshotResp`, `SimSnapshot`
- 不動既有 messages（GameEvent / PlayerCommand / Subscribe / 等保留）

**Verify**: `cargo build --manifest-path /d/omoba/omb/Cargo.toml` 重編 prost — proto files compile clean，新 types 出現在 `OUT_DIR/game.rs`。`cargo build --manifest-path /d/omoba/omoba-core/Cargo.toml` 同樣（client side prost）。

**Commit**: parent repo commit `feat(proto): Phase 2 lockstep messages — InputSubmit / TickBatch / StateHash / GameStart / Snapshot*`

### Task 2.2: omb lockstep module

**Files**:
- Create: `omb/src/lockstep/mod.rs` — module entry
- Create: `omb/src/lockstep/input_buffer.rs` — `pub struct InputBuffer` collecting per-tick player inputs
- Create: `omb/src/lockstep/tick_broadcaster.rs` — 60Hz tokio interval pacer
- Create: `omb/src/lockstep/state.rs` — `LockstepState` (current_tick, players, master_seed, last_state_hash_tick)
- Modify: `omb/src/main.rs` — spawn lockstep tick thread alongside existing sim dispatcher

**InputBuffer 結構**:
```rust
pub struct InputBuffer {
    /// player_id → tick → Vec<PlayerInput>
    buffer: BTreeMap<u32, BTreeMap<u32, PlayerInput>>,
}

impl InputBuffer {
    pub fn submit(&mut self, player_id: u32, target_tick: u32, input: PlayerInput) { ... }
    pub fn drain_for_tick(&mut self, tick: u32) -> Vec<(u32, PlayerInput)> {
        // collect all (player_id, input) where input.target_tick == tick
        // remove from buffer
    }
}
```

**TickBroadcaster 結構**:
```rust
pub struct TickBroadcaster {
    tick: u32,
    interval: tokio::time::Interval,  // 16_666 microsecs
    input_buffer: Arc<Mutex<InputBuffer>>,
    state_hash: Arc<dyn Fn(u32) -> u64 + Send + Sync>,  // Phase 1 omoba_sim::state_hash hookup
    out_tx: mpsc::UnboundedSender<TickBatch>,
}
```

每 tick:
1. drain InputBuffer for current tick
2. compose TickBatch { tick, inputs, server_events: vec![] }
3. broadcast via out_tx
4. if tick % 600 == 0: compose StateHash and broadcast
5. tick += 1

**Phase 2 限制**：state_hash 暫返 0（因 omoba-sim 還沒跑 server-side sim — 那是 Phase 3）。或者 hash 個 dummy `u64 = tick * 0x9E3779B97F4A7C15`，先建 pipe，Phase 3 換真 hash。

**Verify**: 單元測試 InputBuffer drain ordering / TickBroadcaster fires every 16.66ms。

**Commit**: omb submodule commit + parent bump

### Task 2.3: KCP transport 新 tag dispatch

**Files**:
- Modify: `omb/src/transport/kcp_transport.rs` — 加新 tag handlers (0x10-0x16) + frame dispatch table
- Modify: `omb/src/transport/mod.rs` — `OutboundMsg::Lockstep(...)` variant
- Modify: `omoba-core/src/kcp/{client,framing}.rs` — client side new tag enum + parse
- Modify: `omoba-core/src/kcp/client.rs` — add `send_input(player_id, target_tick, input)` + `subscribe_lockstep()` API for omfx (Phase 2.4)

**Tag constants**:
```rust
pub const TAG_INPUT_SUBMIT: u8 = 0x10;
pub const TAG_TICK_BATCH: u8 = 0x11;
pub const TAG_STATE_HASH: u8 = 0x12;
pub const TAG_JOIN_REQUEST: u8 = 0x13;
pub const TAG_GAME_START: u8 = 0x14;
pub const TAG_SNAPSHOT_REQ: u8 = 0x15;
pub const TAG_SNAPSHOT_RESP: u8 = 0x16;
```

**Server side handler logic**:
- 0x10 InputSubmit: decode prost, push to InputBuffer
- 0x13 JoinRequest: decode, register player session, send 0x14 GameStart back
- 0x15 SnapshotReq: decode (observer flow), send 0x16 SnapshotResp (Phase 5 才實作完整 — Phase 2 只 stub 回 placeholder)
- 0x11 / 0x12 / 0x14 / 0x16: server **send** only

**Client side (omoba-core/src/kcp/client.rs)**:
- 0x14 GameStart: cache master_seed + start_tick
- 0x11 TickBatch: forward to game-side via channel (omfx Phase 2.4 listens)
- 0x12 StateHash: stash latest hash, expose for diagnostics

**LZ4 compression**: 維持既有 logic — 大 payload (TickBatch with many inputs / SimSnapshot) 自動壓縮。tag 0x80 flag 同樣應用。

**Subscribe coexistence**: 既有 0x04 SubscribeRequest (legacy GameEvent stream) + 新 0x13 JoinRequest (lockstep pipe)。一個 client 可同時走兩條（Phase 2 dev 用，Phase 3 後 omfx 只走 lockstep）。

**Verify**: 單元測試 frame encode/decode for new tags。Integration: server start, dummy client connect, send 0x13 → receive 0x14, send 0x10 → server log "received input for tick T+3"。

**Commit**: omoba-core + omb + omb submodule bump

### Task 2.4: omfx 最小 lockstep client

**Files**:
- Create: `omfx/game/src/lockstep_client.rs` — minimal stub
- Modify: `omfx/game/src/lib.rs` — spawn lockstep client thread alongside existing NetworkBridge

**LockstepClient 結構**:
```rust
pub struct LockstepClient {
    kcp_client: omoba_core::kcp::client::GameClient,
    master_seed: Option<u64>,
    last_received_tick: u32,
    last_state_hash: Option<(u32, u64)>,
}

impl LockstepClient {
    pub async fn connect(addr: &str) -> Result<Self> { ... }
    pub async fn join(&mut self, player_name: String) -> Result<u64 /* master_seed */> {
        // send 0x13 JoinRequest
        // wait for 0x14 GameStart
    }
    pub async fn submit_input(&mut self, target_tick: u32, input: PlayerInput) { ... }
    pub async fn poll_tick_batches(&mut self) -> Vec<TickBatch> { ... }
    pub async fn poll_state_hashes(&mut self) -> Vec<StateHash> { ... }
}
```

**Phase 2 範圍**：omfx 連 server、收 TickBatch、log "received tick T with N inputs"。**不執行 sim** — Phase 3。

**UI 暫不 send InputSubmit**（沒 sim 不知 input semantics）— Phase 2 dev mode 可 hardcode dummy NoOp input each frame for testing the pipe。

**Verify**: omfx 啟動後 console 印「lockstep connected, master_seed=X, receiving TickBatch...」一行 per 16.66ms。

**Commit**: omfx submodule commit + parent bump

### Task 2.5: Integration test + verify + close

**Files**: 不新增；補 integration test in omb

**Integration test** (`omb/tests/lockstep_integration.rs`):
- 啟動 server (in-test mock)
- 連 2 個 mock client via tokio_kcp
- Both send JoinRequest → receive GameStart with same master_seed
- Both send InputSubmit (target_tick=10) with NoOp action
- Server tick advances; at tick 10, broadcast TickBatch with both inputs
- Each client receives TickBatch, verifies `tick==10 && inputs.len()==2`
- After 600 ticks, each client receives StateHash matching server's

**Manual verify**:
```
# Terminal 1
cargo run --manifest-path /d/omoba/omb/Cargo.toml -p omobab

# Terminal 2-3
cargo run --manifest-path /d/omoba/omfx/Cargo.toml -p executor
cargo run --manifest-path /d/omoba/omfx/Cargo.toml -p executor

# Both omfx should log lockstep tick batches every ~16.66ms
# omb log should show "tick T broadcast: N inputs from M players"
```

**Final**: 
- whole chain cargo build clean
- omoba-sim 65 tests + 8 pin hashes locked  
- omb 整 build clean
- gen-docs 仍 OK
- legacy GameEvent path 也仍 work (omfx 仍能 render units 透過舊 pipe)

**Phase 2 close commit**:
```
chore(phase2): Phase 2 (lockstep wire protocol scaffolding) complete

New KCP tags 0x10-0x16 (InputSubmit / TickBatch / StateHash / JoinReq /
GameStart / SnapshotReq / SnapshotResp). New proto messages: PlayerInput
oneof + Vec2I / FixedI quantized for Fixed32 raw. omb lockstep module
(InputBuffer + TickBroadcaster) runs alongside legacy 30Hz dispatcher
at 60Hz pace.

omfx LockstepClient (Phase 2 minimal): connect, JoinRequest, receive
TickBatch / StateHash. Does NOT consume — Phase 3 implements omfx
simulator that turns TickBatch input into local sim state.

Legacy GameEvent broadcast (tags 0x01-0x07) preserved unchanged for
omfx render in Phase 2 dev. Phase 3 will switch omfx to consume
TickBatch and Phase 4 cuts the legacy path.

Verified:
- 2-client integration test: both receive synchronized TickBatch
  every 16.66ms; StateHash broadcast every 600 ticks (10s)
- whole chain cargo build clean
- omoba-sim 65 tests + 8 cross-OS pin hashes still locked
- legacy omfx render path unaffected

Next: Phase 3 — omfx becomes simulator (load base_content.dll, run
omoba-sim per TickBatch, kill 30Hz GameEvent dependency).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

After close commit, dispatch superpowers:code-reviewer for entire Phase 2 range. Approve → fast-forward merge to master.

---

## 開放問題

- **GameStart timing**: Phase 2 dev mode server 啟動就有固定 master_seed；多人 lobby Phase 5 (matchmaking) 才動態
- **TickBatch encoding size**: empty batch ~6 bytes header；含 8 player NoOp ~50 bytes; 含 active inputs ~150 bytes — 60Hz × 150 = 9 KB/s peak per client，相對 Phase 1 的 64-206 KB/s 已大幅縮減
- **StateHash 內容**: Phase 2 用 placeholder (tick * golden ratio const)；Phase 3 omfx sim 上線後改真 sim hash via omoba_sim::state_hash::hash_sorted_by_id
- **JoinRequest authentication**: 暫無 — dev mode by player_name string only。Production 需要 Phase 5+ 加 token/auth (out of scope)
