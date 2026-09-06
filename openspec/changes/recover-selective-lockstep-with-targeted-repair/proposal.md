## Why

目前玩家視角 replica 遇到 `UnknownEntity` 等可恢復的增量幀錯誤時會停止 client runtime，進而讓本機啟動器關閉整場。這違反 server authoritative 原則，也讓單一投影差異變成玩家可見的閃退，因此需要以低流量、戰爭迷霧安全的差異修復取代直接終止。

## What Changes

- Client 遇到可恢復的 replica apply error 時保留最後安全畫面、暫停增量套用並送出結構化 repair report，不再立即關閉程序。
- Server 不信任 client 指定的資料範圍，而是依 authoritative team projection 選擇 `ComponentRepair`、`EntityReplace`／dependency bundle，必要時才使用 filtered rebase。
- 修復內容受 team visibility、disclosure epoch 與 component allowlist 約束，不能藉由錯誤回報取得視野外資訊。
- Team 1／Team 2 server observer replica 使用相同修復路徑；server 自產非法幀不得送往玩家。
- 修復完成後重試失敗幀並從既有 sequence 繼續；沒有進展時逐級升級，只有安全錯誤或恢復耗盡才終止該 client。
- 增加可定位 phase、operation、replica ID 與 epoch 的診斷資料，修正視野邊界的 entity lifecycle 排序問題。
- `run_2player.bat` 保留目前程序生命週期，但正常 repair／rebase 不再讓 runtime 退出，因此不會觸發整場清理。

## Capabilities

### New Capabilities

- `targeted-team-replica-recovery`: 定義玩家視角 replica 的最小差異回報、server 安全決策、分級修復、client 原地恢復與 observer 驗算。

### Modified Capabilities

- `lockstep-event-flow`: 增加增量幀暫停、修復控制訊息、失敗幀重試與恢復後有序續播的事件流程要求。

## Impact

- `omoba-core`：team protocol schema、selective replica、authority recovery、team projector、observer validation、KCP client 與有序 team stream。
- `omb`：KCP repair request 接收、team-safe 修復產生、replay／rebase outbound 與結構化診斷。
- `omoba-client-runtime`：`AwaitingRepair` 狀態、有界增量暫存、修復套用、失敗幀重試與 renderer IPC 存活。
- `scripts/run_2player.lua`：只需辨識真正的 process exit；repair 狀態不視為 launcher failure。
- Protocol v2 沿用既有 `ClientTeamHashMismatch`、`ComponentRepair`、`EntityReplace` 與 filtered rebase 訊息，不新增另一套重複的 repair protocol。
