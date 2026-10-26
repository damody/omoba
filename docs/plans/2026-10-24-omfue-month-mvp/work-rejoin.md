# 斷線超過 30 秒後重新加入同一場（rejoin）

狀態：server、client runtime 的狀態機與身分驗證、bridge、UE 顯示與恢復都已實作，3-way merge 著陸（未 commit）。UE 由主 session 建置。實機（netem outage）尚未執行，時段由主 session 安排。

## 行為

1. **斷線**：client runtime 30 秒沒有 replica 進度（`LINK_LOST_AFTER`），或 server 關閉了這個 session 時，runtime 不再結束，改為發出 `RECONNECTING`（`link_lost`／`session_closed`），進入重連。
2. **重連**：每次嘗試前發出 `RECONNECTING`（`rejoin_attempt:N`）。
   - 退避間隔：第一次立即，之後依序 1、2、4、8、10、10… 秒。
   - 每次嘗試最多等 10 秒（連線＋JoinRequest＋TeamGameStart）。
   - 持續到加入成功、使用者放棄（runtime shutdown），或 15 分鐘的重連窗結束（這時比賽多半已經結束、server 已經關閉），結束時發出 `SERVER_DISCONNECTED`（`rejoin_gave_up`）。
3. **server 端**：
   - 舊 session 結束時（client 沉默 30 秒，或連線關閉），如果是 secure 玩家（不是 observer），會送出一筆 server `Stop` 給該英雄（`hold_departed_hero`），然後取消註冊。
   - `Stop` 使用該玩家 input allocator 的下一個 id，走一般的輸入路徑，所以兩隊 replica 都在同一個 tick 套用。英雄會停止移動、攻擊意圖與佇列，仍保留閒置時的自動攻擊，原地待機到玩家回來。
   - server log：`KCP player disconnected; hero held player=<id> tick=… input_id=…`。
   - 選擇「待機」而不是交給 bot：role bot 只為 bot 玩家建立，人類玩家的槽位沒有 bot 的記憶與決策狀態，中途接管會讓 bot 的 plan 從一半開始；待機比較可預期，也不會讓 bot 讀到該玩家的私有狀態。
4. **身分驗證（resume token）**：
   - 每次成功加入時，server 產生新的 32 bytes 隨機 token（`LockstepState::issue_resume_token`），只放在該玩家單播的 `TeamGameStart.resume_token`（field 38）。這個 token 不寫 log，也不進任何 hash。
   - 之後同一個 player_id 再加入時，`JoinRequest.resume_token`（field 22）必須等於最新的 token（`verify_join_identity`，以常數時間比對）。不帶 token、帶舊 token，或用別的玩家的 token，都會被拒絕，連線也會結束。
   - 第一次加入不需要 token，行為不變；仍沿用既有的 `authenticated_team_bindings`（player→team 綁定）與 secure 協商。
   - 每次加入都會換新的 token，舊 token 立刻失效。
   - server log：`KCP player rejoined player=<id> team=… tick=… session=…`。
5. **重建視野**：rejoin 的 `TeamGameStart` 就是 server 每個 tick 重建的該隊 team bootstrap，內容涵蓋 disclosed world、metadata、view epoch、`last_seen_input_id` 與 owner ward。runtime 用它建立新的 `ReplicaHost`，與既有的 bootstrap／rebase 路徑相同（E053／E056）。
   - 之後依 `next_team_sequence` 接收 frame。輸入 allocator 從 `last_seen_input_id`（包含 server 的 Stop）之後繼續編號。
6. **UE**：
   - 重連時顯示結算面板「重新連線中…」，內容是「與伺服器的連線中斷，正在重新加入同一場比賽（第 N 次嘗試）。可以按「放棄」離開。」，按鈕顯示「放棄」（走既有的離開對局流程）。
   - 重連期間輸入暫停、owned HUD 暫停（連線狀態為 Joining）。
   - 加入成功後，runtime 會送出 `READY`（`rejoined`），接著送 ready 與 `ResetView`：bridge 清除所有舊的 entity，UE 移除舊的 actor，再由新的 snapshot 重建。
   - 面板消失後，HUD、輸入恢復；滑鼠鎖定也依玩家的選擇恢復（結算面板出現時會暫時放開）。
   - UE log：`OM_SESSION_RECONNECTING attempt=N`／`OM_SESSION_REJOINED`。

## 修改

| 範圍 | 檔案 |
|---|---|
| proto | `proto/game.proto`：`JoinRequest.resume_token=22`、`TeamGameStart.resume_token=38`、`RuntimeSessionState.RECONNECTING=5`。已同步更新 `omoba-core/src/generated/game.rs` fallback |
| core | `kcp/client.rs`：`join_selective_lockstep_resuming`（舊 API 等同於帶空 token）。`team_projector.rs`：bootstrap 的 token 欄位留空，由 transport 填入 |
| server | `omb/src/lockstep/state.rs`：`verify_join_identity`、`issue_resume_token`、`JoinIdentity`。`omb/src/transport/kcp_transport.rs`：加入時驗證並發 token、`hold_departed_hero`、rejoin log |
| client runtime | `rejoin.rs`（新增，退避狀態機）。`session.rs`：`connect_resuming`。`main.rs`：外層 `RuntimeContext`／`run_session`／`reconnect` 迴圈；renderer 連線、evidence、presentation sequence 跨 session 保留 |
| bridge | `OM_SESSION_RECONNECTING=5`（lib.rs＋om_bridge.h，只新增常數，不升 ABI）。`apply_session_state` 中 Reconnecting 對應 Joining，不是終止狀態 |
| UE | `OmMatchTermination`（SessionReconnecting、IsReconnecting、第 N 次嘗試的文字）。`OmCommandBarWidget::SetSessionTermination(..., bReconnecting)` 與「放棄」按鈕標籤。`OmWorldBridgeActor` 傳入重連狀態。`OmRuntimeBridgeSubsystem` 的 log |
| 驗收工具 | `scripts/check_netem_outage.lua`：每個 link-lost 時間窗都必須通過 `-rejoined` gate（runtime 在 link lost 之後印出 `client-runtime rejoined`、server 有 `hero held`，以及同一位玩家的 `KCP player rejoined`）。原有的 observer mismatch 0 與無終止 gate 照舊 |

## 測試

| 測試 | 內容 |
|---|---|
| `omobab lockstep::state::rejoin_requires_the_latest_resume_token_of_that_player` | 第一次加入不需要 token；不帶、全 0、截短、加長、別人的 token 都被拒絕；最新的 token 可以通過；每次加入都換新、舊的失效。mutation（比對永遠通過）會讓它失敗 |
| `omoba-client-runtime rejoin::*`（3 項） | 退避 0、1、2、4、8、10、10…；重連窗結束時放棄；reason 文字 |
| `base_content rejoin_replica::held_hero_and_mid_match_rejoin_bootstrap_keep_replicas_identical` | 走路中的英雄被 server Stop 後不再移動；在比賽中途用當下的 team bootstrap 建立新的兩隊 replica，300 tick 逐 tick hash 與 membership 都和權威一致；rejoin 後的新指令可以再次移動英雄 |
| `om_bridge driver::session_states_map_to_connection_states_and_terminated_latches` | Reconnecting 對應 Joining，帶 attempt reason，不是終止狀態；READY 後恢復 Connected |
| UE `Om.Runtime.RejoinSessionState`（新檔 `OmRejoinSessionTests.cpp`） | 重連會阻擋輸入、顯示標題與嘗試次數；READY 後恢復；放棄後成為 disconnect；終止狀態無法被重連解除 |

## 實機（待主 session 排時段）

quick map，`--netem-outage 2:120:45`（team 2 在 120 秒時斷線 45 秒）。

- 預期：link lost 後第 30 秒發出 `RECONNECTING`，斷線期間的重試都會失敗（proxy 全部丟包），斷線結束後的下一次重試加入成功。
- 驗收：`tools\lua\lua.exe scripts\check_netem_outage.lua --run <dir>` 的 `outage-1-team-2-rejoined`、`no-observer-mismatch` 與 `no-termination` 都要 PASS。

## 風險與後續

- UE 程序重啟後無法 rejoin：token 只存在 runtime 的記憶體中。目前的 runtime 由啟動流程管理，跨程序保存 token 需要安全的本機儲存位置。
- netem proxy 對新的 KCP 連線（新的 conv／來源 port）是否照常轉送，要靠實機確認。
- 重連期間 UE 的「放棄」會直接離開 UE。runtime 程序仍會重試到重連窗結束，或由啟動流程停止它。

## 著陸後的實際樹檢查（凍結解除後）

`CARGO_TARGET_DIR=target\session-ward`，結果全部通過：

| 命令 | 結果 |
|---|---|
| `cargo check -p omobab --tests --bins` | 通過 |
| `cargo test -p omobab --lib` | 194 passed |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml` | 167 passed（lib 含 3 項 rejoin）＋ main 3 passed |
| `cargo test --manifest-path omfue/bridge/Cargo.toml --lib` | 149 passed |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | 1161 passed |
| `base_content -- rejoin_replica forced_displacement true_sight_area` | 8 passed |
