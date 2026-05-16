## Context

`omb` authoritative loop 與 `omfx` local sim replica 已透過 `omoba_core::lockstep_timing` 共用 120Hz lockstep cadence。`omfx/game/src/native.rs` 的 `Plugin::update` 目前每次 engine update 都會 drain lockstep events、讀取最新 `SimWorldSnapshot`、更新 render bridge、HUD、本地 VFX 與 UI 倒數；如果 render loop 沒有限速，它會重複 render 同一個 sim snapshot，導致畫面更新遠快於 sim 前進速度。

這個 change 的約束是：render pacing 只影響 frontend frame production，不改 backend tick rate、不改 `sim_runner` tick execution、不改 KCP/lockstep protocol，也不能讓 input submit 或 lockstep event drain 被長時間阻塞。

## Goals / Non-Goals

**Goals:**
- 讓 omfx 預設 render cadence 與 shared sim cadence 對齊，避免 renderer 在同一個 sim tick 上 busy-render。
- 在 snapshot tick 沒有前進時，降低重複的 render bridge/UI/VFX per-frame work。
- 保留 input、network event drain、auto smoke hooks 與 sim_runner worker 的反應性。
- 讓 frame diagnostics 可以看出 render 已被 cap，以及 cap 後的實際 fps。

**Non-Goals:**
- 不調整 `LOCKSTEP_TPS` 或 backend authoritative `State::tick()` cadence。
- 不改變 simulation dt、投射物速度、creep 速度、buff duration 或 ability cooldown 的權威計算。
- 不新增 wire protocol 欄位，也不讓 backend 決定 frontend FPS。
- 不把本地動畫完全改成 tick-based animation；本 change 只處理 render pacing。

## Decisions

1. 使用 shared lockstep cadence 作為預設 render cap。

   render cap SHALL 由 `LOCKSTEP_TPS` 或等價 helper 推導，不在 `omfx` 另寫固定 `60`、`120` 或 millisecond magic number。這讓未來 lockstep cadence 調整時，render pacing 自動跟隨 sim contract。

   Alternative considered: 固定 cap 到 60 FPS。這能降低負載，但在目前 120Hz lockstep 下不是「跟 sim 同步」，也會讓 UI/HUD smoothing 與每 tick snapshot consumption 變成另一組獨立 cadence。

2. 將 pacing 放在 frontend render/update 邊界，不放進 `sim_runner`。

   `sim_runner` 應繼續盡快消化收到的 `TickBatch` 並發布最新 snapshot；render 限速只決定 omfx 何時重做 render-facing work。這避免 pacing 影響 deterministic local replica 與 authoritative state hash 對齊。

   Alternative considered: 讓 `sim_runner` sleep 到 120Hz。這會把網路 jitter 轉成 local replica backlog，並可能延遲 snapshot 發布。

3. 在沒有新 snapshot tick 且 frame budget 未到時跳過昂貴 render-facing work。

   `Game` 可以追蹤 `last_render_frame_at` 與 `last_rendered_snapshot_tick`。每次 `update` 仍先執行必要的 input/network drain；若目前 snapshot tick 沒變、距離上一個 render frame 未達 `1 / LOCKSTEP_TPS`，則 early-return 或只做最小維護，避免重新跑 batched sprite/UI/VFX 更新。

   Alternative considered: 單純依賴 Fyrox renderer stats 的 `capped_frame_time` 或 OS vsync。這通常受 monitor refresh、driver 設定或 engine 預設影響，不能保證跟 sim cadence 一致，也不一定能避免 `Plugin::update` 的 CPU work。

4. diagnostics 要明確呈現 cap 狀態。

   現有 `FrameProfile` 已記錄 `pure_render_ms_total`、`capped_render_ms_total` 與 `last_fps`。實作時應補上 render pacing target、skipped/limited frame count 或等價 log 欄位，讓 `omfx_render` log 可驗證實際 FPS 接近 sim cadence，且不是靠偶然的 GPU bottleneck。

## Risks / Trade-offs

- [Risk] early-return 放太前面會漏掉 input 或 lockstep events → Mitigation: pacing 判斷必須在 auto hooks、input submission、lockstep event drain 與 sim snapshot forwarding 之後，且不得阻塞 `events_rx` drain。
- [Risk] `std::thread::sleep` 精度在 Windows 上可能造成 frame jitter → Mitigation: 優先使用 engine/native frame limiter；若只能在 update path sleep，sleep duration 要保守並允許下一幀補回，不影響 sim worker。
- [Risk] 本地 VFX/buff countdown 目前以 frame `dt` 遞減，cap 後視覺會更貼近 sim cadence但較不平滑 → Mitigation: 這是本 change 的目標行為；權威數值仍由 snapshot 重設，且未來可另行設計 tick-aware interpolation。
- [Risk] render fps 低於 sim cadence 時無法補幀 → Mitigation: pacing 只設定上限，不要求補足；慢機器仍依實際 performance render 最新 snapshot。

## Migration Plan

1. 在 `Game` 加入最小 render pacing state 與 frame profile counters。
2. 從 `omoba_core::lockstep_timing` 推導 render target interval。
3. 將 `Plugin::update` 拆出「必跑的 input/network/snapshot forwarding」與「可限速的 render-facing update」。
4. 更新 `omfx_render` diagnostics，記錄 target fps 與 pacing skip/cap 行為。
5. 驗證 `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor`，必要時執行一般 dev run 觀察 `omfx_render` log。
