# Unreal frame 套用與 ACK 分離（2026-10-06）

## 計畫與問題

接續OpenSpec安全呈現／6.4：Tick在MarkFrameConsumed失敗後仍推進LastFrameSequence，下次取得同frame便跳過，因此不再補ACK。直接把游標延後則會重複套用HUD／一次性效果，不是通用修正。

## 決定與實作

- 新共用FOmFrameConsumption將已套用sequence與ACK成功旗標分離；apply成功即記錄已套用，ACK失敗保留待ACK狀態，同sequence下次只補ACK。
- ACK成功後，同frame不再apply／ACK；新frame可以繼續，舊frame不建立無界重試佇列。使用bridge既有有效lease的MarkFrameConsumed，沒有自行偽造或延後持有lease。
- 每次Consume（包括已套用的duplicate）先通過E285完整shape gate，再核對frame與lease sequence，不相符不apply／ACK。初始狀態使用具名旗標，不把sequence0誤當已套用。
- 正式Tick使用同一helper，成功與失敗皆release。stats的LastFrameSequence維持「已套用」語意，不冒充ACK成功。StopRuntime重設兩個狀態與診斷節流；不重啟後端，不恢復runtime Lua。
- 新native FrameConsumption矩陣涵蓋lease錯配、ACK多次失敗／恢復且apply僅一次、成功duplicate不重ACK、duplicate仍驗shape、apply失敗不ACK、不推進、新frame、reset同sequence與sequence0。加入既有Lua scoped runner。

## 局部確認

- UE限定模組8 actions成功，exit0；Saved/Logs/frame-consumption-retry.log。
- 固定Lua runner loadfile語法確認通過，兩repo whitespace通過。
- native矩陣已編譯但尚未執行；遵守E285，不反覆啟動已失敗的獨立Cmd測試入口。沒有PIE／stage／完整驗收，不宣稱斷言通過。
- ABI16／wire6／IPC5及生成hash不變，整體21/31、完整6.4保持未勾。見E286。
