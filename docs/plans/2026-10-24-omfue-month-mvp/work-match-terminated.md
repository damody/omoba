# work-match-terminated：玩家看得到「比賽已終止」（E050 的 UE 待辦）

```text
日期：2026-10-25
目前工作 ID／狀態：E050 UE 待辦／bridge 已著陸並通過測試；UE 原始碼已著陸，尚未編譯（需主 session 建置並跑 Editor 測試）
玩家或作者新增能做到的事：server 終止比賽時，畫面中央出現「比賽已終止：<原因>」面板，可以按「離開對局」；終止後不再送出任何遊戲輸入，bridge 也不會重連同一場。
repo HEAD／本次修改：未 commit；2026-10-25 17:22 著陸
已執行命令、exit code、build identity：見「測試」
L1／L2／L3／L4 證據：L1 fresh（bridge）；UE Editor 測試已撰寫但未執行；L3 未執行
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward、target/session-ward/shadow-target
```

## 資料流

client runtime `RenderIpc SessionState`（`RuntimeSessionState`，proto/game.proto:1503）
→ bridge `driver::apply_session_state`

| state | connection_state | 其他 |
|---|---|---|
| STARTING／STALLED | Joining | 保存原因 |
| READY | Connected | 清除原因 |
| SERVER_DISCONNECTED | Error | 不是終態，仍允許重連 |
| TERMINATED | Error | 終態：結束 presentation client，不再重連；latch 住，之後的 READY 也無法讓比賽恢復 |

- `safe_reason` 經過清理：只保留 ASCII 英數字與 `_ : - .`，最長 96 字元（`SESSION_REASON_MAX`）。runtime 端本來就只送 `match_terminated:<class>`，這一層是防禦。
- 終止後 `om_submit_input` 回 `NotStarted`，last_error 是 `match terminated`。

→ UE `UOmRuntimeBridgeSubsystem::PollSessionStatus()`（`AOmWorldBridgeActor::Tick` 每個 tick 都會呼叫，因為終止後不會再有新 frame）→ `FOmMatchTerminationModel`
→ `UOmCommandBarWidget::SetSessionTermination` 把終止內容放進既有的對局結果面板（`SessionTerminationView` 優先於結果 payload）。面板就是勝負面板，附有「離開對局」按鈕（`UOmMatchSessionSubsystem::RequestLeaveCompletedMatch` → QuitGame）。終止時隱藏「下一場」按鈕。即使 HUD 因非 Connected 而被 invalidate，面板仍會顯示。
→ 輸入：subsystem 的 `SubmitGameplayInputEvent` 與 `SubmitInputCommandInternal` 在 `!AllowsGameplayInput()` 時直接拒絕（Error = `Match terminated`）；SERVER_DISCONNECTED 期間也拒絕，恢復後解除。bridge 端另外也會拒絕，形成兩道防線。
→ autopilot：`FOmAutoPilot::Tick` 開頭一偵測到終止，只寫一次 `{"event":"session_terminated","tick":N,"reason":"<class>"}`，接著截圖、寫 coverage，然後停止所有操作。

## CABI（沒有升版）

既有的 struct 和函式都沒改，只新增：

- `OmSessionStatus { struct_size, abi_version, session_state: u32, terminated: bool, safe_reason: OmStringView }`（大小 32，`terminated` 的 offset 是 12，`safe_reason` 的 offset 是 16，smoke 有斷言）
- `OmStatus om_poll_session_status(OmRuntime*, OmSessionStatus*)`：`safe_reason` 指向 runtime 擁有的副本，下一次呼叫之前都有效。
- `OM_SESSION_STARTING..OM_SESSION_TERMINATED`（0..4，和 proto enum 相同）

UE 動態載入 `om_poll_session_status`，並把它列入 `GBridge.IsValid()` 的必要 export，所以 DLL 和 UE 一定要一起換。staged header 已用 cbindgen 重新產生。

## 原因文字（`FOmMatchTerminationModel::ReasonText`，LOCTEXT namespace `OmMatchTermination`）

| server reason_class | 顯示 |
|---|---|
| observer-mismatch-after-rebase | 伺服器偵測到狀態不同步 |
| authority-recovery-exhausted | 狀態同步無法恢復 |
| outbound_queue_watchdog | 伺服器傳送逾時 |
| 其他、空字串 | 伺服器已結束這場比賽 |

面板顯示「比賽已終止」，下方一行是「比賽已終止：<原因>」。斷線時顯示「與伺服器的連線中斷」，下方一行是「暫停操作，等待連線恢復」。

## 修改檔案

- bridge：`src/driver.rs`（DriverStats session 欄位、`sanitize_session_reason`、`apply_session_state`、讀取迴圈、2 個測試）、`src/lib.rs`（`OmSessionStatus`、`om_poll_session_status`、submit 拒絕、`session_reason` 儲存、1 個測試）、`cbindgen.toml`、`smoke/om_bridge_header_smoke.cpp`
- UE：新增 `OmRuntime/Public/OmMatchTermination.h`、`OmRuntime/Private/OmMatchTermination.cpp`、`OmEditor/Private/OmMatchTerminationTests.cpp`；修改 `OmRuntimeBridgeSubsystem.h/.cpp`、`OmCommandBarWidget.h/.cpp`、`OmGenerated/Private/OmWorldBridgeActor.cpp`、`OmAutoPilot.h/.cpp`（改之前已讀過目前版本，只在 Tick 開頭加了一段，header 加一個 bool）、`ThirdParty/OmBridge/include/om_bridge.h`

## 測試

| 命令 | exit | 結果 |
|---|---|---|
| 著陸後的實際樹：`cargo test --manifest-path omfue/bridge/Cargo.toml` | 0 | lib 149 passed、1 ignored；integration 2 passed。新增 `session_states_map_to_connection_states_and_terminated_latches`（逐一驗證狀態映射，惡意 reason 會被清理並截斷，TERMINATED 之後的 READY 無效）、`terminated_session_ends_the_presentation_client_without_reconnecting`（假 presentation server 送出 TERMINATED 後，900ms 內沒有任何重連，一般斷線則會在 250ms 內重連）、`terminated_session_is_polled_once_and_refuses_every_later_input`（經 C API：poll 拿到 terminated 與 reason，之後 submit 回 NotStarted、last_error 為 "match terminated"） |
| UE Editor：`Om.Runtime.MatchTerminationText`、`Om.Runtime.MatchTerminationRefusesInput` | 未執行 | 依指示不跑 UE |

## 主 session 需要做的

- 重跑 `build_ue_bridge.lua`，連同新的 DLL 和 header；UE 建置後執行上面兩個 Editor 測試，以及既有的 MatchResult、AutoPilot、OwnedViewportIpc 測試，確認結果面板沒有退化。
- 實機：讓 server 觸發 `match_terminated`（例如 desync 測試路徑），確認畫面中央出現面板、輸入無效、「離開對局」可以結束，autopilot trace 只出現一次 `session_terminated`。

## 限制

- 面板沿用勝負面板的離開流程。`CanLeaveCompletedMatch` 要求 HUD 的 session source 仍是 current；如果 HUD 在終止後被整個 retire，按鈕會停用，這時只能關閉視窗。
- SERVER_DISCONNECTED 會擋住輸入並顯示斷線面板，但仍允許重連；client runtime 之後是否會改送 TERMINATED，取決於 runtime 端。

## 後續修正（build-21 回報，2026-10-25 19:11 與 split16 desync 一起著陸，UE 尚未編譯）

1. **原因 class 一律區分大小寫。** `FString ==` 不區分大小寫，所以 `OBSERVER-MISMATCH-AFTER-REBASE` 也對到了「不同步」。`ReasonText` 改用 `Equals(..., ESearchCase::CaseSensitive)`，`Observe` 判斷 class 是否改變時也一樣；`ExtractReasonClass` 原本就區分大小寫。規則：class 依協定原樣比對，測試 `Om.Runtime.MatchTerminationText` 不用改。
2. **連線列不再顯示「連線失敗，請檢查網路」。** `UOmPlayerHudOwnerComponent` 在 session 處於阻擋狀態時，改顯示 model 的標題：終止時顯示「比賽已終止」，server 斷線時顯示「與伺服器的連線中斷」。其他狀態不變。
3. **autopilot 終止截圖等面板出現才拍。** 原本在 `session_terminated` 的同一個 frame 就截圖，面板要等 bridge actor 自己的 tick 發布，之後的 frame 才畫得出來。現在 `session_terminated` 只記一次；之後每個 tick 檢查 `IsTerminationPanelVisible()`（控制器的 HUD owner → command bar `IsInViewport() && IsMatchResultPanelVisible()`），面板連續可見 0.25 秒後才截圖，最多等 3 秒。並寫入 `{"event":"termination_panel","tick":N,"visible":true|false,"wait_ms":M}`。如果 `visible=false`，代表 3 秒內面板都沒有出現，就不是時序問題，要查 `TryPrepareHudPublish` 為什麼失敗。
   - 修改：`OmAutoPilot.cpp/.h`（改之前已重新複製目前的版本，只改了 Tick 開頭的終止段落，並新增 `IsTerminationPanelVisible` 與 3 個欄位）、`OmPlayerHudOwnerComponent.cpp`、`OmMatchTermination.cpp`。

## 第二次修正（split20 回報：trace 寫 visible=true，畫面上卻沒有面板；2026-10-25 19:32 著陸，UE 尚未編譯）

- **根因**：結果面板包在 command bar 根 overlay 的一個置中 `SBox` 裡，這個 SBox 的 `Visibility_Lambda` 只在 `IsMobaPresentation()` 時才顯示，沒有像面板本身那樣加上 `|| SessionTerminationView.bVisible`。終止後 bridge actor 會呼叫 `InvalidateOwnedHud`，把 `AuthoritativeGameMode` 重設為 0；command bar 收到 presentation 0 後，`IsMobaPresentation()` 變成 false，父層 SBox 收合，面板跟著看不到。`IsMatchResultPanelVisible` 只檢查面板自己的旗標，所以 trace 誤報為 true。HUD 本身沒有被 retire（頂部狀態列仍會更新「比賽已終止」，就是證據）。
- **修正**（`OmCommandBarWidget`）：
  - 新增 `ShouldShowMatchResultPanel()`（未 retire、有 view、且是 MOBA presentation 或終止中），四個顯示判斷點（父層 SBox、面板建構時的初始值、`SetMatchResultState`、presentation 更新）全部改用它，不會再出現父子條件不一致。
  - `IsMatchResultPanelVisible()` 改為沿 Slate parent 鏈逐層檢查面板和每個祖先是否 visible。這個函式也是離開按鈕的條件，所以不加 viewport 檢查，以免既有 Editor 測試在沒有 viewport 的情況下失敗。
  - 新增 `IsMatchResultPanelOnScreen()`，額外要求在 viewport 中，且最後一次排版的幾何尺寸不為 0。autopilot 的 `IsTerminationPanelVisible` 改用這個函式，`termination_panel visible` 從此代表真的顯示在畫面上。
- **新測試**：`Om.Runtime.MatchTerminationPanelSurvivesHudReset`。presentation 設為 0 後終止，面板和所有祖先都必須可見；不在 viewport 時 `OnScreen` 必須是 false；清除 session 後面板要隱藏。用舊程式跑，這個測試會在「Panel and every ancestor visible」這一步失敗。
- **未處理**：終止後單位與名牌消失、只剩地形，這是另一條 presentation 清除路徑（連線狀態轉為 Error 之後），和面板無關。如果希望保留最後一個畫面，要另外處理。
