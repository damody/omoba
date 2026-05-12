## Why

TD_STRESS 在怪物數超過約 400 後，玩家移動指令實際要超過 1 秒才被看見，但 HUD 顯示的 `Lag` 只有約 46ms，表示目前 latency metric 沒有涵蓋玩家感知的完整路徑，且高量 creep / entity 事件可能讓 lockstep input 或 snapshot handoff 被反壓拖慢。

這需要同時修正診斷定義與 runtime 行為，避免壓測時用錯誤的低延遲數字掩蓋輸入排隊、transport broadcast 或 sim publish backlog。

## What Changes

- 修正 input latency metric，使 `Lag` 能代表玩家送出 input 到畫面呈現對應權威結果的端到端延遲，而不是只顯示成功 pair 的短路徑樣本。
- 在 phase trace 中明確揭露 TD_STRESS 相關 backlog：client pending age、server input queue、TickBatch receive-to-forward、sim publish、render pair 等階段。
- 調整 TD_STRESS 下 input / TickBatch 與高量 legacy game events 的處理優先權，避免 creep movement / health / entity flood 阻塞玩家 input 的接收、排程或回放。
- 加入 regression 驗證，確保 400+ creeps 的壓測場景中移動指令不會被高量事件延遲到秒級。
- 不改變 lockstep determinism、gameplay ECS state hash、script ABI 或 TD_STRESS 的遊戲規則。

## Capabilities

### New Capabilities


### Modified Capabilities

- `input-latency-metric`: `Lag` 與 phase trace 必須涵蓋並揭露 TD_STRESS 下玩家感知的端到端 input delay，不能只回報成功 paired input 的樂觀短路徑。
- `player-input-routing`: 玩家 input 的接收、排程與應用不得被高量非 input broadcast 事件 starvation；TD_STRESS 下移動指令必須維持可接受的處理延遲。

## Impact

- Affected code: `omb/src/transport/kcp_transport.rs`、`omb/src/lockstep/`、`omfx/game/src/lockstep_client.rs`、`omfx/game/src/sim_runner.rs`、`omfx/game/src/native.rs`。
- Affected specs: `input-latency-metric`、`player-input-routing`。
- Affected diagnostics: `input_render_latency:`、`input_latency_phase:` log、HUD `Lag:` 顯示與 TD_STRESS smoke / stress log analysis。
- No wire-level breaking change is expected unless investigation proves an existing message needs extra optional metadata; any new fields must remain metadata-only and outside deterministic sim state.
