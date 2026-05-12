## 1. Snapshot Trigger Audit

- [x] 1.1 檢查 `omb/src/transport/` 中所有會讀取 `SnapshotStore`、建構 snapshot response 或處理 snapshot request 的 code path。
- [x] 1.2 檢查 `PlayerCommand`、lockstep `InputSubmit` 與 `player_input_tick`，確認目前哪些 input path 可能間接觸發 snapshot。
- [x] 1.3 檢查 omfx client 端是否在 input submit 後主動要求 snapshot，列出需要改成後續權威流程觀察的呼叫點。

## 2. Core Implementation

- [x] 2.1 將 snapshot network delivery 集中到 external connection/subscription bootstrap helper，並從該 helper 讀取 `SnapshotStore`。
- [x] 2.2 移除或拒絕 gameplay input path 中的 snapshot request handling，確保 `InputSubmit` 與 `PlayerCommand` 不會送 snapshot response。
- [x] 2.3 保留新外部連線進入遊戲時取得 latest snapshot 的流程，並處理 no-snapshot-yet 的安全 fallback。
- [x] 2.4 更新 omfx input forwarding 或 diagnostics，避免 input 後主動依賴 snapshot response。

## 3. Tests And Guards

- [x] 3.1 新增 transport 行為測試，驗證新 session bootstrap 會收到初始化 snapshot。
- [x] 3.2 新增 transport/input 行為測試，驗證 lockstep `InputSubmit` 不會產生 snapshot response。
- [x] 3.3 新增 grep guard 或單元測試，防止 input command branches 直接讀 `SnapshotStore` 或建構 snapshot response。

## 4. Verification

- [x] 4.1 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib`。
- [x] 4.2 視實作影響執行相關 omfx 測試或 build，確認 input flow 不再依賴 input-triggered snapshot。
- [x] 4.3 手動或 smoke 驗證新玩家連線仍能取得初始 state，進遊戲後 input 不觸發 snapshot traffic。
