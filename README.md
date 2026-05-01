# omoba

Rust MOBA / TD 雙模遊戲。Server (`omb`) + Frontend renderer (`omfx`) submodule。詳細架構與常用指令見 [`CLAUDE.md`](CLAUDE.md)。

---

## Transient Fyrox patches

omfx 跑在 Fyrox 1.0.1 上，有兩個小 bug / 限制必須直接編輯 cargo registry cache 的 Fyrox source 才能修。**這些 patch 不會 commit 到任何 repo**，每次 `cargo clean`、`cargo update`、換 machine 都會被沖掉，需要重新 patch + 重 build。

### Patch 1：強制 vsync OFF（解 Windows DWM 鎖 60 fps）

upstream Fyrox 1.0.1 的 `vsync: false` 在 Windows 是 no-op — `fyrox-graphics-gl/src/server.rs` 只在 `vsync=true` 時 `set_swap_interval(Wait(1))`，`false` 完全不 set，OS 默認 vsync on（DWM compositor 鎖 60 Hz）。

**檔案**：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-graphics-gl-1.0.1/src/server.rs` 約第 664 行

```rust
                if vsync {
                    Log::verify(gl_surface.set_swap_interval(
                        &gl_context,
                        SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
                    ));
                } else {
                    // Force vsync OFF (override OS/driver default which is usually
                    // vsync on for Windows DWM windowed apps).
                    Log::verify(gl_surface.set_swap_interval(
                        &gl_context,
                        SwapInterval::DontWait,
                    ));
                }
```

加上 `else` branch 主動 set `SwapInterval::DontWait`。

### Patch 2：每 frame sleep 1 ms（避免 CPU 100%）

vsync OFF 後 stress 場景能跑 280+ fps，但 CPU 會被佔滿。在 Fyrox 主 event loop 的 `Event::AboutToWait` 結尾加 `thread::sleep(1ms)` 把 CPU 讓出來。

**檔案**：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/engine/executor.rs` 約第 406 行（`run_normal` 函式內）

```rust
            Event::AboutToWait => {
                game_loop_iteration(
                    &mut engine,
                    ApplicationLoopController::ActiveEventLoop(active_event_loop),
                    &mut previous,
                    &mut lag,
                    fixed_time_step,
                    throttle_threshold,
                    throttle_frame_interval,
                    frame_counter,
                    &mut last_throttle_frame_number,
                );
                // omfx-local patch：每 frame 強制 sleep 1ms 把 CPU 從 ~100% 降下來。
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
```

實測：280 fps → 183 fps（render 3.15 ms + sleep 1 ms + Windows scheduler overshoot ~1.5 ms = ~5.5 ms/frame）。

要更穩的 fps cap（例如 144 fps）把 `from_millis(1)` 改大即可（每加 1 ms 大約再降 ~50 fps，因 Windows `thread::sleep` 通常 overshoot 1-2 ms）。

#### 配套：`timeBeginPeriod(1)`（已 commit 在 `executor/src/main.rs`）

Windows 預設 timer granularity 15.6 ms，`thread::sleep(1ms)` 在 idle Windows
上會變成 sleep ~15 ms 把 fps 鎖到 60。`executor/src/main.rs` 開頭呼叫
`timeBeginPeriod(1)` 強制把 system-wide timer resolution 降到 1 ms。

實際在 desktop Windows 通常已經有別的 process（Chrome / 遊戲）request 1 ms
timer，`timeBeginPeriod` 多半是 no-op 但保險著。在 server / clean Windows
環境（沒 Chrome 等）上一定要有它，否則 sleep(1) 會塞 15 ms。

注意這只是 timer granularity 上限，**不是真正 1 ms sleep**。Windows
scheduler context switch latency 大約 1-2 ms，sleep(1) 在最佳情況也是 ~2 ms。
要真 1 ms 級的精度需要 `CreateWaitableTimerEx(STATE_HIGH_RESOLUTION)` 或
spin-wait，目前不需要。

### 重 patch 流程

```bash
# 1. 編輯 ~/.cargo/registry/.../fyrox-graphics-gl-1.0.1/src/server.rs (Patch 1)
# 2. 編輯 ~/.cargo/registry/.../fyrox-impl-1.0.1/src/engine/executor.rs (Patch 2)
# 3. 強制重編 affected crates
cargo clean --release -p fyrox-graphics-gl --manifest-path omfx/Cargo.toml
cargo clean --release -p fyrox-impl --manifest-path omfx/Cargo.toml
# 4. 重 build
cargo build --release --manifest-path omfx/Cargo.toml -p executor
```

### 持久化選項（沒做）

- Fork `fyrox-graphics-gl` + `fyrox-impl`，用 `[patch.crates-io]` 指向 forked 路徑。維護成本：每次 Fyrox 升版都要重 rebase。
- 直接接受 transient — patch 在當前 cache 內 build 都正常，只有清 cache / 升版才需要重來。

目前選 transient（成本最低）。
