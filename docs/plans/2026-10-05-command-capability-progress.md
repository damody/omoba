# Hold網路准入與通用命令能力協商

## 本批計畫與結果

1. 盤點輸入全鏈：發現server secure coordinate白名單漏Hold，補獨立版本契約，不直接無條件開放。
2. Join及bootstrap追加command protocol1與完整content rules hash；legacy保持關閉，無效協商拒絕。server只有bound selective SingleLane／ThreeLane player啟用；fresh與cached／rejoin回覆一致。
3. client InputBridge預設及resume關閉，主程式使用已核對的server回覆開啟；未協商拒絕Hold且不消耗ID，Move仍可送。server保留owner、nonzero ID與gameplay mode限制；shop／Recall不能替代command能力。
4. accepted-input投影追加Hold action_kind20且無target，不改queued payload、owner解析或披露安全。機械同步protobuf fallback，既有Fyrox legacy Join顯式關閉。

## 當前功能確認

- scripts workspace core command契約：1 passed。
- client runtime input_bridge：13 passed，包含新capability／ID測試、queued、owner／epoch與shared codec。
- server library command能力契約：1 passed，包含Join無效組合、兩種protobuf往返、未協商／錯owner／無binding／零ID拒絕。
- client runtime binary cargo check：成功。
- server binary cargo check：成功。

沒有啟動完整驗收、UE、100場或新網路對局；沒有部署DLL／stage。UE仍受E177共享引擎修改限制。OpenSpec20/30保持，不能把此部分實作當完整4.1完成。

## 錯誤與決定

E203記錄client binary錯誤型別與server accepted-input exhaustive分支漏接，修復後局部確認。C ABI13、IPC4、KCP selective2與content hash502a59ee5aa2a677不變；command1是另一能力，不是整體網路版本。以版本契約保留未來命令擴充，不移除既有安全gate。
