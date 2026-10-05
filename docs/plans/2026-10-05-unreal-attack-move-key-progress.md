# 原生游標攻擊移動進度（2026-10-05）

## 本輪計畫與決定

1. 補齊現有 AttackMove API 的玩家原生入口，不重做 Rust 命令格式。
2. A 直接在游標處攻擊移動，Shift+A 排隊；不增加 A 後左鍵的模態狀態，避免干擾既有滑鼠 graph。
3. 共用 owner／HUD／游標 guard；只做模組編譯確認，完整遊戲驗收留最後。

## 實作

- OmPlayerController 的 SetupInputComponent 各綁定唯一普通A／Shift+A，反射呼叫 SubmitOwnedAttackMoveFromCursor，保持 OmRuntime 不依賴下游 OmGenerated。
- WorldBridge 的通用 owned API 檢查原生controller、HUD／選角攔截、有限且視窗內游標、有效地面投影，再從 runtime configured player ID 提交既有 AttackMove 與 queued。
- Rust繼續決定命令合法性；前端成功送出不等同權威已接受。無英雄專用程式／新增Blueprint graph、runtime Lua 或第二份玩法World。
- NativeAbilityUpgradeBinding 測試擴充普通A／Shift+A唯一binding、HUD攔截、owned反射API、缺controller／runtime拒絕案例。新增非Shipping dispatch helper 僅沿正式delegate，不注入玩法輸入。

## 當前功能確認

`Build.bat OmGameEditor Win64 Development -Project=D:\code\omoba\omfue\om.uproject -Module=OmRuntime+OmGenerated+OmEditor -WaitMutex -NoHotReloadFromIDE -NoEngineChanges`

- 13 actions，Result: Succeeded，9.56秒。
- 日誌：`omfue/Saved/Logs/native-attack-move-key-modules-20261005.log`。
- 新增斷言只編譯，未執行Editor automation、PIE、真實鍵盤或有效游標送出測試；不宣稱完整操作已驗收。
- Lua資料、內容hash、ABI14／wire5／IPC4不變；不stage二進位、不修改engine BuildId、不維護omfx。
- 工具輸出截斷與入口缺口記E255。完整6.2仍未完成，OpenSpec維持21/31。
