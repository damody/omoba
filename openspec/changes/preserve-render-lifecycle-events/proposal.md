## Why

玩家視角 replica 已正確產生 `Hide` 與 `Forget`，但這些一次性事件目前會隨可合併的 presentation snapshot 被覆蓋，造成敵方單位反覆進出視野後在陰影邊界留下多個舊 render identity。必須讓生命週期事件可靠到達 renderer，同時保留連續狀態只取最新值的低延遲特性。

## What Changes

- 新增保序且不可丟棄的 renderer lifecycle IPC 資料契約。
- 將 `Hide`、`Forget` 與 view reset 從 latest snapshot lane 分離到 critical FIFO lane。
- 保留每個事件的 `disclosure_epoch`，避免 IPC 轉換後降為 `0`。
- omfx 使用獨立 lifecycle queue，滿載時施加 backpressure，不覆蓋舊事件。
- renderer 先套用 lifecycle，再套用最新連續狀態；生命週期事件須可安全重複執行。
- 新增慢速消費者與反覆跨越視野邊界的回歸測試。

## Capabilities

### New Capabilities

- `reliable-render-lifecycle`: 定義 client runtime 到 renderer 的可靠生命週期事件、epoch、排序、backpressure 與重連 reset 行為。

### Modified Capabilities

無。

## Impact

影響 `proto/game.proto`、`omoba-core` generated schema、`omoba-client-runtime` presentation bridge 與主迴圈，以及 `omfx/game` presentation client、filtered render bridge 與 native 消費流程。網路端的 server authoritative、team replica 投影和 canonical identity 隔離規則不變。
