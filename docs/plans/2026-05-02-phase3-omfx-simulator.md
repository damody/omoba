# Phase 3 — omfx becomes Lockstep Simulator + Renderer

> **For Claude:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development。

**Goal:** omfx 從 純 renderer 升級為 simulator + renderer。引入 omb-as-lib，spawn worker thread 跑完整 omb ECS dispatcher driven by TickBatch input。Render 從 sim World 讀 entity state（取代 NetworkBridge GameEvent path）。TickBroadcaster placeholder state hash 換成真 omoba_sim::state_hash over authoritative ECS。

**Architecture:**
- omfx/game 加 path-dep `omobab = { path = "../../omb" }`，import omb's components / systems / dispatcher / scripting loader
- 新 `omfx/game/src/sim_runner.rs` — worker thread spawns specs World，loads base_content.dll，runs dispatcher per TickBatch
- LockstepClient (Phase 2) 升級從 log-only 變 input-feeder：UI events → InputSubmit → server → TickBatch → sim_runner pushes inputs into ECS → dispatcher tick
- omfx render 從 sim ECS world 讀 (Pos / Facing / Sprite / Hp 等 components) 渲染；舊 NetworkBridge 並行保留待 Phase 4
- TickBroadcaster 在 omb side 用 `omoba_sim::state_hash::hash_sorted_by_id` over ECS Pos/Hp/Tick 取代 placeholder

**Tech Stack:** specs 0.20 single-thread (Phase 1e 已 audit) / omoba-sim primitives / abi_stable base_content.dll / Fyrox 1.0.1 / 8 pin hashes locked from Phase 0/1。

---

## Context（核心設計決策）

### 為何 omfx 直接 dep omb，不 refactor 到 omoba-sim crate
原 design doc 的 Phase 1 outline 假設 ECS components / systems 搬到 `omoba-sim` crate。實際 Phase 1a-1d 切了 omb internal types 但 **沒搬 crate** — components / systems 仍住 omb。

兩個方案：
- **A (採用)**: omfx path-dep omb-as-lib，import omb's specs World / components / dispatcher / scripting。低 refactor、馬上可動。代價：omfx 拉 omb 整 dependency tree (kcp / log4rs / etc.)。
- **B (deferred)**: 把 ECS 抽到新 omoba-sim-ecs crate，omb / omfx 都引用。big refactor (~90 specs Component imports), 推遲。

選 A。Phase 4+ 視需要再做 B（如果 omfx bloat 影響 build time / binary size）。

### omfx Sim Loop 設計
- Worker thread (`std::thread::spawn`) 跑 specs ECS dispatcher
- `crossbeam_channel` 從 LockstepClient 接 TickBatch
- 每收到 TickBatch[T]：
  1. 把 TickBatch.inputs 塞進 ECS resources（PendingPlayerInputs<T>）
  2. 跑 dispatcher.dispatch_seq(&world)
  3. 更新 sim tick counter
  4. publish 「sim state at tick T」 給 render 端讀
- Render thread (Fyrox main) 每 frame：
  - try_recv 最新 sim snapshot pointer
  - read entity Pos / Facing / Sprite / Hp 等
  - 更新 Fyrox scene graph

### Sim → Render 同步
**Lock-free double buffer**：
- sim worker 每 tick 完成寫入 `Arc<Mutex<SimWorldSnapshot>>`（簡單 mutex 即可，因 ECS 一個 tick ~16ms 跨 thread sync 開銷小）
- render thread 用 `try_lock()` 讀取，失敗就用上次的（避免阻塞 frame）
- Phase 3 用 simple mutex；Phase 4 視 perf 改 RwLock 或 lock-free snapshot

### MasterSeed sync
omfx LockstepClient 收 GameStart 時 cache master_seed。傳給 sim_runner，set 到 omb 的 `MasterSeed` resource。確保 omfx side 的 SimRng 跟 omb authoritative side 同 seed。

### State hash hookup
omb side `TickBroadcaster::placeholder_state_hash` (Phase 2 用 `tick * 0x9E3779B97F4A7C15`) 換成真 hash：
```rust
fn real_state_hash(&self, tick: u32, world: &specs::World) -> u64 {
    use omoba_sim::state_hash::hash_sorted_by_id;
    // Hash all entities' authoritative state: id + Pos + Hp + Facing
    let snapshot: Vec<EntityHashItem> = world.create_entity()...collect();
    hash_sorted_by_id(&snapshot, |e| e.id)
}
```

omfx side 同樣每 600 tick 算 local sim hash 與 server StateHash 比對。Mismatch → log error + (Phase 5+) kick / resync。

### NetworkBridge legacy 保留
Phase 3 內 omfx 仍保留 NetworkBridge GameEvent 並行（safety net）。Render 可以從 sim World 跟 NetworkBridge 兩個 source 並行讀（先 sim，fallback 老 path 如 sim 還沒 spawn entity）。Phase 4 統一砍 NetworkBridge。

---

## Tasks

### Task 3.1: omfx 加 omb dep + smoke test

**Files**:
- Modify: `omfx/game/Cargo.toml` — 加 `omobab = { path = "../../omb" }` 跟 `specs = "0.20"`（與 omb 同版）+ `omoba-sim` (with abi-stable feature)
- Create: `omfx/game/src/sim_runner.rs` — empty stub `pub fn smoke() {}`
- Modify: `omfx/game/src/lib.rs` — `mod sim_runner;` declaration

**Verify**: 
- `cargo check --manifest-path /d/omoba/omfx/Cargo.toml` clean
- `cargo build --manifest-path /d/omoba/omfx/Cargo.toml` clean (可能很久, ~3-5 min first compile of omb deps)

**Commit**: omfx submodule + parent bump

### Task 3.2: omfx sim_runner — worker thread + ECS init

**Files**:
- Modify: `omfx/game/src/sim_runner.rs` — full impl
- Modify: `omfx/game/src/lib.rs` — Game::init spawn sim_runner alongside LockstepClient

**SimRunner 結構**:
```rust
pub struct SimRunnerHandle {
    pub state_rx: Arc<Mutex<SimWorldSnapshot>>,  // render reads
    pub tick_input_tx: Sender<(u32 /* tick */, Vec<(u32 /* player_id */, PlayerInput)>)>,
    pub master_seed_tx: Sender<u64>,  // GameStart received → set MasterSeed
    _thread: thread::JoinHandle<()>,
}

pub struct SimWorldSnapshot {
    pub tick: u32,
    pub entities: Vec<EntityRenderData>,  // pos / facing / sprite / hp / etc.
}

pub fn spawn_sim_runner() -> SimRunnerHandle {
    // ... thread::spawn ...
    //   1. init specs World (use omobab::state::initialization helpers)
    //   2. load base_content.dll (use omobab::scripting::loader)
    //   3. build dispatcher (use omobab::state::system_dispatcher)
    //   4. loop:
    //      - master_seed_rx.try_recv → set MasterSeed resource
    //      - tick_input_rx.recv() → push inputs to PendingInputs resource
    //      - dispatcher.dispatch_seq(&mut world)
    //      - extract render data from world → update state_rx snapshot
}
```

關鍵 omb-side helpers 必須是 `pub`：
- `omobab::state::initialization::create_world(scene_path: &Path) -> World`（如果 currently 是 `pub(crate)`，改 `pub`）
- `omobab::scripting::loader::load_scripts(path: &Path) -> ScriptRegistry`
- `omobab::state::system_dispatcher::SystemDispatcher::new() / .dispatch(&mut world)`

如果這些 currently 不 public，omb-side commit 把必要 export 加 `pub` (Phase 3 sub-step)。

**Verify**: omfx cargo check + sim_runner spawn 不 panic（log: "sim_runner started, world has N initial entities"）。

**Commit**: omfx + omb (if any pub exposure) + parent bump

### Task 3.3: LockstepClient → SimRunner input feeder

**Files**:
- Modify: `omfx/game/src/lockstep_client.rs` — receive TickBatch, forward inputs to sim_runner
- Modify: `omfx/game/src/lib.rs` — wire LockstepClient ↔ SimRunner channels

新 flow：
```
LockstepClient receives TickBatch[T] 
  → forward (tick=T, inputs) to sim_runner.tick_input_tx
  → sim_runner thread: push inputs to ECS, run dispatcher
  → sim_runner publishes new SimWorldSnapshot (tick=T)
```

GameStart received → forward master_seed to sim_runner.master_seed_tx。

**Phase 3 範圍**：UI 仍不送 InputSubmit（Phase 4 接 Fyrox UI events）。sim_runner 仍能跑（每 tick empty input batch，creep wave / tower attack 仍動 — 因為 omb sim 內含 wave spawn 邏輯）。

**Verify**: 8-client mock test (Task 3.5) — but minimal verify here: omfx 啟動，sim_runner tick 跑、log 印 "sim tick advance: T=10, entities=N"。

**Commit**: omfx + parent bump

### Task 3.4: omfx render bridge + state hash hookup

**Files**:
- Modify: `omfx/game/src/lib.rs` Game::update — try_lock SimWorldSnapshot, update Fyrox scene
- Create: `omfx/game/src/render_bridge.rs` — convert SimWorldSnapshot.entities → Fyrox sprite mutations
- Modify: `omb/src/lockstep/tick_broadcaster.rs` — replace placeholder_state_hash with real omoba_sim hash over ECS

**Render bridge 任務**:
- 對每個 entity in SimWorldSnapshot.entities:
  - Lookup or spawn Fyrox sprite (cache by entity_id)
  - Update sprite position from Pos.to_f32_for_render() (sim→render boundary)
  - Update facing rotation
  - Update HP bar / icon
- Cleanup sprites for entities not in snapshot (despawned)

**State hash hookup 在 omb**:
```rust
// omb/src/lockstep/tick_broadcaster.rs
fn real_state_hash(&self, tick: u32) -> u64 {
    use omoba_sim::state_hash::hash_sorted_by_id;
    // Read from `Arc<Mutex<SimWorldHandle>>` injected at startup
    let world = self.sim_world.lock().unwrap();
    let mut items: Vec<HashItem> = world.entities()
        .join()
        .map(|e| HashItem { id: e.id(), pos: pos_storage.get(e), hp: cprop_storage.get(e) })
        .collect();
    hash_sorted_by_id(&items, |i| i.id)
}
```

omb-side 需 thread `Arc<Mutex<World>>` to TickBroadcaster — 但 specs World 不是 Send/Sync 友善的。實際做法：在 dispatcher tick 結束時 compute hash + pass via channel。

**Verify**: omfx 開啟看到 Fyrox window 渲染 entities (從 sim World，不是 NetworkBridge)。State hash 變成 real，每 600 tick log。

**Commit**: omfx + omb + parent bump

### Task 3.5: 8-client integration test + Phase 3 close

**Files**:
- Create: `omb/tests/lockstep_8client_desync.rs` — spawn omb server, 8 KCP mock clients each with own sim_runner-equivalent
- Each client tracks local sim state hash
- Run 30s (1800 ticks @ 60Hz)
- Assert: every StateHash from server matches all 8 clients' local hashes

**Manual verify**:
- 跑 omb (server)
- 跑 2 個 omfx instances 並行
- 兩 omfx 互相看到對方 hero 移動同步（透過 sim 跑出相同 state）
- 30 秒 zero desync

**Final**:
- 整 chain cargo build clean
- omoba-sim 65 tests + 8 pin hashes locked
- omb tests + omfx tests 全綠
- gen-docs 渲染 9 units / 8 abilities
- 8-client integration test PASS

**Phase 3 close commit**:
```
chore(phase3): Phase 3 (omfx simulator + renderer) complete

omfx now runs the full omb ECS dispatcher in a worker thread, driven by
TickBatch input from omb's lockstep wire. Render reads from the sim
World (取代 NetworkBridge GameEvent path).

Architecture:
- omfx/game path-deps omobab as lib (specs World, components, systems,
  scripting loader, dispatcher).
- sim_runner thread: load base_content.dll, init World with MasterSeed
  from GameStart, run dispatcher per TickBatch input.
- LockstepClient feeds TickBatch into sim_runner.
- render_bridge converts SimWorldSnapshot → Fyrox scene mutations.
- TickBroadcaster real state_hash via omoba_sim::state_hash over
  authoritative ECS Pos/Hp/Tick.

Verified:
- 8-client integration test: 30s lockstep zero desync
- StateHash from server matches all clients @ every 600-tick checkpoint
- omoba-sim 65 tests + 8 cross-OS pin hashes locked

Legacy NetworkBridge GameEvent path retained but unused for Phase 3
sim consumers — Phase 4 deletes it + the 76 PHASE 2 wire markers.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

After close commit, dispatch superpowers:code-reviewer for entire Phase 3. Approve → fast-forward merge to master.

---

## 開放問題

- **specs World thread safety**: World 不是 Send/Sync (含 `Rc<>` etc.)。Worker thread 跑 specs OK 因為 World 一直 in same thread。Sim → render boundary 需要拷 entity data 出 thread 不直接 expose World。
- **omfx executor 不 spawn omobab.exe**: Phase 3 後 omfx 從 omb-as-lib import，不再 spawn 子行程。`run.bat` 要重寫（先跑 omobab.exe server，後跑 executor.exe client）。
- **base_content.dll path**: omfx 跑時 working dir 在 `omfx/executor/...`，base_content.dll 要 deploy 到 `omfx/scripts/` 或 absolute path 拿。Phase 3 dev mode 用 absolute path，prod deployment 後續處理。
- **Two omfx instances same machine**: 用 OMB_PLAYER_NAME env 區分。OMB_KCP_ADDR 都連同個 server。
