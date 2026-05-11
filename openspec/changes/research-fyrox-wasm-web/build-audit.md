## Baseline WASM Build Audit

### Toolchain

- `rustc --version`: `rustc 1.91.0 (f8297e351 2025-10-28)`
- `wasm-pack --version`: `wasm-pack 0.13.1`
- `wasm32-unknown-unknown`: initially missing; installed with `rustup target add wasm32-unknown-unknown`

If setting up a fresh machine, run:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Initial Build Command

```powershell
wasm-pack build --target web --release
```

Working directory: `D:\omoba\omfx\executor-wasm`

### First Build Blockers

- Dependency / wasm backend: `getrandom v0.3.4` failed for `wasm32-unknown-unknown` because no `wasm_js` backend cfg was enabled.
- Dependency / native toolchain: `ring v0.17.14` failed because `clang` was not installed or not on `PATH` for wasm C compilation.
- Dependency source: `ring` is pulled through `omfx -> omobab -> log4rs -> rumqttc -> tokio-rustls -> rustls -> ring`, which is native backend/server-only for the Web client target.
- Dependency source: `tokio_kcp` is pulled through `omoba-core` and `omobab`, both currently reachable from `omfx/game` during the wasm build.
- Dependency source: `getrandom v0.3.4` is pulled through `specs` / `omobab` / `omoba-core` paths that should not all be present in the first Web client build.

### Blocker Classification

- `dependency`: `omfx/game` currently pulls native server and KCP dependencies into `executor-wasm`.
- `native API`: backend process spawn, Windows Job Object, native script DLL, UDP/KCP socket and server-only crates must be target-gated out.
- `thread/runtime`: current `lockstep_client` and `sim_runner` use native threads and tokio runtime assumptions that need a browser path.
- `asset path`: not reached yet; will be validated after dependency/native gates compile.
- `Fyrox/web renderer`: not reached yet; current blocker happens before renderer validation.
- `transport`: browser cannot use current KCP/UDP client directly; WebSocket bridge remains the first Web transport target.
