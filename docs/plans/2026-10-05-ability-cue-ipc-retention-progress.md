# 施法 cue 接入可靠 IPC 保留（2026-10-05）

## 問題與決策

- 最新 snapshot 不是一次性事件日誌。60Hz replica 在畫面降頻或 watch 覆寫期間發生的施法，必須在每個成功套用 frame 先擷取，不能等到呈現 snapshot 才取。
- 原本 DMG1 傷害專用保留器改名／移至 `cue_retention.rs` 的 `CueRetention`，由共用 `PresentationCue` 型別辨識 DMG1／ABY1，管理相同容量、時間窗、live replica 依賴、sent／consumed ACK 與重連基線。不新增獨立施法日誌，不複製另一套可靠傳送邏輯。
- 一個 cue 目前只依賴一個 live entity：傷害為 target，施法為 caster。Hide／Forget 精確匹配 replica ID＋disclosure epoch；epoch 變更或 entity 不再 live 即移除，ResetView 清空全部。只有同 connection 的已成功傳送 snapshot ACK 才能退休，準備／發布不是已消費。
- 舊 renderer reconnect 基線與已消費 prepared snapshot 過濾只辨識 DMG1；改辨識共用 typed cue，ABY1 不會繞過基線或在 ACK 後重播。
- 原 external effects 在權威端排序後配置獨立序號，public events 卻仍用 producer-local ordinal。共用事件 ID 使用 tick＋ordinal，因此混合技能／傷害或多 producer 可能撞 ID。現於排序後先配置 external，再配置 public 的連續序號；不把 canonical producer 身分塞進安全事件 ID。

## 已實作

1. 共用 `PresentationCue` 的嚴格格式辨識、effect ID／tick 驗證、tick 與 live entity dependency API。
2. 權威投影排序後跨 external／public 列表的序號配置，保留既有 external 排序及序號起點。
3. `ReplicaHost::ability_presentation` 從成功套用的安全 public events 擷取 ABY1，依目前 live world 的 disclosure epoch 編碼；main 在每個成功 frame、畫面降頻前合併傷害與施法後 capture。
4. 安全 snapshot converter 使用同一 ABY1 轉換 API，只接受 live caster 與精確 Ability payload。共用保留器與傳送時 ACK 過濾接入原 localhost IPC 路徑。

## 局部確認

- `cargo test --manifest-path omoba-client-runtime/Cargo.toml --features compiled-content-only --lib cue -- --nocapture`：7 passed／0 failed／75 filtered out。
- 包含實際 localhost TCP 傳送測試：DMG1、ABY1 分別走 RendererReady／baseline／watch 覆寫／RendererConsumed／ACK 後 prepared snapshot 過濾／RendererShutdown。是 IPC 測試，不是 Unreal、KCP、權威完整對局驗收。
- 混合日誌確認共享去重、connection 限定 ACK、Hide 的 stale epoch 不誤刪、disclosure 變更、重連基線及總容量 1024，不是每種 cue 各 1024。
- 安全 converter 確認隱藏 caster／非法長度／非 Ability event 不生成施法 cue，施法與傷害 ID 不相同。
- core 指定 ability_cue：4 passed／0 failed／428 filtered out，包含跨 producer／跨列表序號與前批安全投影／格式測試。
- client runtime 的正式 `compiled-content-only` binary check 通過，不只編譯 lib tests。既有模板 build script 的三個 dead-code warnings 未隱藏。

## 明確尚未完成

- ABY1 現在能經正式 client runtime 的保留／傳送路徑進入 IPC effects；Unreal bridge 目前仍只解 DMG1，所以還不能聲稱 Unreal 已播放施法效果。
- 後續需接 bridge 安全 admission、穩定技能 ID→compiled catalog lookup、frame coalescing／lease 重試與原生 AbilityCast event、Unreal 消費去重。不得把未知 stable ID 當 UE catalog index，也不得利用 renderer ACK 冒充已播放。
- ABY1 沒有目標／位置／技能等級／切換狀態／陣形欄位，這些不能憑空補造；必要時擴充安全權威契約再生成。
- 本批未修改 Lua 來源或 catalog hash、IPC 4／C ABI 13；未重新 stage DLL、啟动 Unreal 或重跑完整對局。全案仍 21/31，4.3／6.1 維持未勾選。

## 防錯

- 兩次多檔 patch verification failed：presentation_bridge 的後段 test hunk 放在前面，再回頭修改較早的 converter hunk，搜尋不會倒退。確認沒有套用後，依檔內位置遞增重排整個 patch，才成功。不能重送同一順序或把失敗 patch 當已完成 rename。
- status／歷史輸出過大時截斷，改僅輸出 apply 的進度與 contextFiles、小段來源及精確搜尋。根 diff stat 含既有修改，不能把所有 dirty diff 當本批成果；cue_retention.rs 是新檔，普通 diff stat 未列入，已另讀回及編譯確認。

錯誤與決策編號：E219。
