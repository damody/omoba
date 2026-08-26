# Selective Lockstep Projection Policy Matrix

## 欄位

- `source-visible`：產生 action 的 entity／owner 是否已對目標 team disclosure。
- `target-visible`：action 直接依賴的 target／collision dependency 是否已對目標 team disclosure。
- `local disposition`：team replica 是否能以已 disclosure state deterministic local step。
- `server projection`：server 必須送出的最小合法資訊。

## Movement

Policy ID：`movement.v1`

Producer 範圍：

- `omoba-core/src/runtime/native/tick/player_input_tick.rs`：`MoveTo`、`AttackMove`、`AttackTarget` command admission。
- `omoba-core/src/runtime/native/tick/hero_command_tick.rs`：hero command resolution。
- `omoba-core/src/runtime/native/tick/hero_move_tick.rs`：hero movement step。

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | Replica 可套用已接受的 command，並以 disclosed collision／movement state local step。 | 傳送 accepted input 與必要的 public movement fact；canonical entity identity 以 team replica ID 表示。 |
| hidden | visible | Replica 不建立 hidden mover，也不 local step hidden movement。 | 只有 hidden mover 對 visible state 造成合法因果結果時，傳送不含 hidden source identity／position 的 sanitized external effect。 |
| visible | hidden | Replica 只能推進不依賴 hidden target 的 ground movement；不得解析或追蹤 hidden target。 | Entity-target command 在 input boundary generalized reject；若 authoritative collision／阻擋來自 hidden dependency，以 visible mover correction 或 sanitized stall/result 表示。 |
| hidden | hidden | Replica 不建立任一 hidden entity，也不接收 movement fact。 | 維持 fixed-cadence empty/padded frame；在產生規格允許的 public causal effect 前不得洩漏 action、identity、position或 payload-size 差異。 |

### Movement invariants

- Viewport、camera 與 client-local AOI 不得改變上述 audience。
- `AttackTarget` 必須通過 input tick 的 replica ID、view epoch、disclosure epoch、team binding 與 visibility history 驗證。
- Movement fact 的 stable order 由 `(tick, phase, canonical_source_order, local_ordinal, fact_kind)` 決定。
- Hidden dependency 無法安全 disclosure 時，server authority correction 勝過 client local result。
