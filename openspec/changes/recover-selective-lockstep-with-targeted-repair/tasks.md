## 1. 根因與恢復路徑

- [x] 1.1 找出 client 將 frame apply error 轉成 `UnsafeSession` 的入口
- [x] 1.2 找出 server 既有 `ComponentRepair`、`EntityReplace` 與 filtered rebase 路徑
- [x] 1.3 找出 sequence 5946 類型的 `UnknownEntity` 發生位置
- [x] 1.4 確認 launcher 是在 runtime 退出後才清理整場

## 2. Entity lifecycle 修正

- [x] 2.1 在 frame 套用前驗證 reveal dependency 引用
- [x] 2.2 在 frame 套用前驗證 Hide／Forget 引用
- [x] 2.3 在 frame 套用前驗證 accepted input 引用
- [x] 2.4 在 frame 套用前驗證 external effect 引用
- [x] 2.5 在 frame 套用前驗證 random tape 引用
- [x] 2.6 在 frame 套用前驗證 component repair 引用
- [x] 2.7 在 frame 套用前驗證 entity replace 引用
- [x] 2.8 記錄 apply phase、operation、replica ID 與 disclosure epoch
- [x] 2.9 Hide／Forget 同幀移除已失效的 pending component repair
- [x] 2.10 Hide／Forget 同幀移除已失效的 pending entity replace
- [x] 2.11 Reveal baseline 同 tick 不再重複套用 Movement public event

## 3. Client 原地恢復

- [x] 3.1 可恢復 frame error 不再呼叫 `shutdown.cancel`
- [x] 3.2 Client 進入 `awaiting_authoritative_rebase`
- [x] 3.3 等待期間保持 KCP transport 與 renderer IPC 存活
- [x] 3.4 等待期間保留最後一個已發布的安全畫面
- [x] 3.5 等待期間不套用後續增量 frame
- [x] 3.6 使用既有 `ClientTeamHashMismatch` 要求 server authoritative recovery
- [x] 3.7 收到 verified filtered rebase 後原子替換 replica
- [x] 3.8 Rebase 後清除過期 pending frame
- [x] 3.9 從 manifest 指定的 resume sequence 繼續
- [x] 3.10 跨 team frame 與 rebase 驗證失敗仍然 fail-closed

## 4. 最後集中測試與檢查

- [x] 4.1 新增 Hide 與 pending repair regression test
- [x] 4.2 新增 stale post-step repair preflight regression test
- [x] 4.3 新增 Reveal baseline 與 Movement 同 tick regression test
- [x] 4.4 執行 `cargo test --manifest-path omoba-core/Cargo.toml`
- [x] 4.5 執行 `cargo test --manifest-path omoba-client-runtime/Cargo.toml`
- [x] 4.6 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab`
- [x] 4.7 執行 55 秒 release 三程序視野邊界 soak
- [x] 4.8 驗證兩隊 sequence 連續且 `unsafe=0`
- [x] 4.9 驗證 Reveal／Hide／Forget 與兩隊 sentinel 隔離
- [x] 4.10 執行 Team 1 fault injection 並驗證 Team 2 不受影響
- [x] 4.11 確認正常 soak 沒有非預期 repair 或 filtered rebase
