# 滑鼠鎖定在視窗內（Ctrl 單按切換）

狀態：已實作，3-way merge 著陸（未 commit）。這台機器不能編譯 UE，要由主 session 建置，跑 Editor 測試 `Om.Runtime.CursorLock`，並在實機驗證游標確實被限制在視窗內。

## 行為

- 預設鎖定：`AOmPlayerController::bCursorLockedByPlayer = true`，視窗模式也鎖。
- 單按 Ctrl 切換：按下並放開，期間沒有按其他鍵或滑鼠按鍵；左右 Ctrl 都可以。
  - `Ctrl+Q`、`Ctrl+F`、`Ctrl+右鍵`／`Ctrl+左鍵`、autopilot 送出的 `Ctrl+R`，以及先按住其他鍵再按 Ctrl，都不會切換。
- 切換時在玩家動作提示列顯示 3 秒「滑鼠已鎖定（Ctrl 切換）」或「滑鼠已解鎖（Ctrl 切換）」，提示會使用目前綁定的按鍵名稱。
- 操作設定的「單按切換滑鼠鎖定在視窗內」一列會顯示「（目前：鎖定／解鎖）」，也可以改鍵。改鍵時可以只按一個修飾鍵（例如 Right Alt），也可以按一般按鍵（例如 F8）。
- 以下情況會暫時放開游標，但不改變玩家的選擇，回到遊戲後自動恢復：
  - 視窗失焦（Alt+Tab）或出現 modal 視窗；
  - 選角；
  - 開啟操作設定；
  - 開啟聊天或聊天紀錄；
  - 結算或對戰終止面板出現（`ShouldShowMatchResultPanel()`／`IsMatchResultPanelOnScreen()`）。
- 在暫停的情境下按 Ctrl 不會切換。例如聊天輸入中，或在設定裡改鍵時按 Ctrl，都不會改變狀態。

## 實作

- `OmCursorLock.h/.cpp`（新增，純邏輯）：
  - `FOmChordTapDetector`：記錄目前按住的鍵，只有在「按住的其他鍵剛好等於 chord 的修飾鍵」時才進入待觸發；其他鍵或滑鼠按鍵按下就取消。忽略 repeat。`Reset()` 在失焦時使用，因為放開事件可能收不到。
  - `FOmCursorLockContext`／`OmCursorLock::ShouldLock`：決定是否鎖定。
  - `OmCursorLock::Notice`：產生提示文字。
- `AOmPlayerController`：
  - 註冊一個 Slate `IInputProcessor`，在任何 widget 之前看到所有按鍵與滑鼠按鍵事件，包括 autopilot 經 `FSlateApplication::Process*` 送出的合成輸入，而且不攔截任何事件。這樣即使 widget 吃掉了 Q，`Ctrl+Q` 仍然會取消這次單按。
  - 每個 tick 呼叫 `ApplyCursorLock()`，這是遊戲中 lock mode 的唯一來源：狀態改變時立即套用，之後每秒重新套用一次（`UGameViewportClient::SetMouseLockMode(LockAlways/DoNotLock)` 加上 `FViewport::LockMouseToViewport`）。
  - 既有的 `SetInputMode_*(DoNotLock)`（選角、設定、BeginPlay）維持不動，會被這一層覆蓋。
  - EndPlay 時註銷 processor 並放開游標。
  - `OM_CURSOR_FREE` 是 cursor-free 施法，與游標鎖定無關，所以沒有改動。
- `FOmPlayerControlProfile`：
  - 新增 action `CursorLockToggle`（category interface），預設 chord 為單獨的 `LeftControl`。
  - 新增 `FOmControlActionDesc::bModifierTap`：只有這類 action 可以綁定不帶修飾旗標的單一修飾鍵，一般 action 仍然拒絕。
  - 舊的 profile 載入時會自動補上這個預設值，因為單獨的 Ctrl 以前不可能被其他 action 佔用。
- `UOmPlayerActionOwnerComponent::IsImplementedChordAction` 加入這個 action。`UOmControlSettingsWidget` 補上標籤、目前狀態與修飾鍵擷取。`UOmCommandBarWidget::ShowInterfaceNotice` 是新增的公開方法。
- `player-guide.md` 的「介面與溝通」按鍵表已更新。

## 測試（`OmEditor/Private/OmCursorLockTests.cpp`，`Om.Runtime.CursorLock`）

- 預設鎖定。catalog 中有這個 action，預設是單獨的 Left Ctrl，預設 profile 可通過驗證。
- 左右 Ctrl 單按都會切換；按住期間的 repeat 不影響。
- `Ctrl+Q`、`Ctrl+R`（autopilot 的 chord 序列）、`Ctrl+F`、`Ctrl+點擊`、先按住 W 再按 Ctrl，都不會切換。失焦重置後，之前的按下不會完成單按。
- 6 種暫停情境都會放開游標，情境解除後恢復。
- 提示文字。
- 改鍵到 F8 後，F8 單按會切換、Ctrl 不再切換；可以改成單獨的 Right Alt；單一修飾鍵不能帶修飾旗標；一般 action 仍拒絕單一修飾鍵；改鍵結果在 export／import 後保留。

## 風險

- `LockMouseToViewport` 透過 viewport 的 reply state 生效，通常在下一個滑鼠事件就套用。實機上若發現鎖定沒有立即生效，可以改成在套用時呼叫 `FSlateApplication::Get().SetAllUserFocusToGameViewport()`。
