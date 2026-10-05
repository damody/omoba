# 點位命令 IPC 排隊語意（2026-10-05）

## 問題與通用決定

既有 Unreal 事件／C command／PlayerInput 已有 queued，但 MoveToIntent 與 AttackMoveIntent 只傳座標；bridge 轉換遺失旗標，client runtime 固定產生 queued=false。因此先前小地圖 Shift 資料接線不等於權威排隊成功，不能沿用舊 ACK／位移證據宣稱完整 Shift 驗收。

- protobuf 兩種 point intent 附加 bool queued=3，保留座標欄位及 PlayerInput 原編號，省略時為 false。
- Unreal bridge／Fyrox presentation adapter 保留正式 PlayerInput 的 queued，client InputBridge 依 intent 原值轉送。沒有新增英雄／地圖特例，不改權威 queue 容量或 phase 順序。
- Presentation IPC version3→4，既有 framing／RendererReady 路徑嚴格拒絕 v3，避免舊 runtime 靜默忽略新欄位。KCP selective protocol2、HUD schema2、C ABI13 與 Lua content hash 不變，各版本不可混為同一版本。
- 既有測試注入／combat smoke 明確 queued=false，維持原本立即指令，不把測試動作改為排隊。
- 透過既有 build.rs 與 OMOBA_UPDATE_PROTO_FALLBACK=1 機械重新生成 tracked game.rs，包含前批 Hold20 與本批 point queued；命令結束還原該環境變數，不手改生成資料。

## 當前功能確認

- client runtime `--lib point_queue`：3 passed（本批新2＋既有 checkpoint_queue1）。新測試涵蓋兩 action × true／false、prost round-trip、owner／epoch拒絕不消耗ID、target與queued不變；raw v3 input envelope拒絕。既有overflow fixture也改為queued=true，保留其拒絕規則。
- bridge `--lib point_queue`：本批新1 passed，兩 action × true／false與正負座標完全保留。
- base_content `--lib point_queue_formal`：本批新1 passed，正常 generated manifest／Production60Hz；Hold保留active，兩種正式排隊命令依序append且無移動；立即Move清佇列並取代Hold，推進20 ticks後真的位移。
- client runtime binary cargo check成功；Fyrox `cargo check --manifest-path omfx/Cargo.toml -p omfx --lib`成功。未執行完整Fyrox遊戲測試、UE、LAN、100場或全套回歸。
- 沿 E194 僅當次測試TEMP／TMP轉D槽並finally還原，沒有清理C槽、改系統設定或部署DLL。

## 真實失敗與修正

首次base測試在立即Move當tick assert_ne(Pos, origin)失敗，前面的active／queued斷言已成功。正式命令與物理位移有既有階段時序，不能要求提交當tick必定移動。保留當tick佇列斷言，結束storage借用後推進20 ticks確認位移；不調換phase、不提高遊戲速度、不移除真正位置確認。第二次測試成功，見E200。

Fyrox相容check首次E0004：前批Hold20新增後，native InputActionKind::from_player_input少明確分支；附加HoldPosition分類與IPC converter正常轉送，沒有把Hold當NoOp或用wildcard掩蓋。再次check成功；其既有Move converter測試增加queued=true斷言，但未執行完整Fyrox測試。

## 剩餘

完整框架仍20/30。本功能修復IPC語意，UE原生Shift鍵與小地圖Shift互動到權威佇列的端到端確認留安全Unreal編譯基線可用後；E177未解除。所有4.1／6.2完整驗收集中最後，不勾選部分完成項。
