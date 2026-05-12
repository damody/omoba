## Why

目前 input 指令可能觸發 snapshot，會讓 gameplay input path 產生不必要的狀態抽取與 render-facing state 傳遞。snapshot 的用途應收斂為外部連線進入遊戲時提供新玩家初始化狀態，避免玩家進入遊戲後的操作繼續依賴 snapshot side effects。

## What Changes

- input 指令處理流程不再觸發 snapshot request、snapshot extraction 或 snapshot send。
- snapshot 觸發時機改為只有外部 client/session 連線進來且需要初始化 state 時才執行。
- 已在遊戲中的玩家狀態更新仍依既有 lockstep tick、event/outcome queues 與正常 render state 流程推進，不以 input 指令要求 snapshot。
- 新增測試或 grep guard，避免未來在 input command path 重新加入 snapshot 觸發。

## Capabilities

### New Capabilities

- 無。

### Modified Capabilities

- `sim-snapshot-rendering`: 調整 snapshot lifecycle requirement，明確規範 snapshot 僅能由外部連線初始化觸發，不能由 gameplay input 指令觸發。

## Impact

- 影響 omb transport/session 連線處理、input command handling、snapshot request/extraction routing 與相關測試。
- 可能影響 omfx 初次連線取得權威 state 的流程，但不應改變玩家進入遊戲後的 lockstep input protocol。
- 不新增外部 dependency，不改變 wire schema，除非實作中發現現有 snapshot request message 需要收斂使用範圍。
