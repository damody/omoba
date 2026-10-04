# 權威迷霧發布與 runtime 保留

## 本批計畫與結果

- [x] 從編譯後 Lua 地圖的路線、野區與地形座標產生固定 Q10 網格；每軸最多 64 格，總容量最多 4096，不以玩家位置或隱藏單位推導範圍。
- [x] Wave B 保存共用 immutable read view；projector 在同一 committed barrier 取樣，不另建玩法 world 或另算一份來源。
- [x] 每隊 projector 獨立保存 explored；預設每 6 ticks 取樣一次，可透過 TeamProjectorConfig 調整。60Hz 時為 10Hz 呈現取樣，不宣稱效能門檻已驗收。
- [x] 呈現專用 event kind 0x46470001、subject=None，FG01 結果在 frame 編碼／padding 前加入。沒有 canonical source ID，也不加入 gameplay FactKind 或 deterministic world/hash。
- [x] TeamGameStart 與內層 snapshot 的私人 metadata 使用相同最新資料；不得放到 public metadata。
- [x] runtime bootstrap 驗證後保留；正常更新僅從成功套用 frame 的 presentation source 接收，缺更新仍輸出同一最新合法事件。非法資料清空呈現快取並記錄診斷，不更改 gameplay/hash。
- [x] 本功能 6 個指定測試、另 1 個追趕留存測試與 client runtime cargo check 成功。

## 問題與決定

FG01 含 team／view epoch／sample tick／grid geometry／三態格子。保留器拒絕未來 tick、倒退、同 tick 衝突、同 epoch 幾何變動、重複事件、entity subject、錯隊／epoch、未知 metadata schema，以及外內 bootstrap 互相矛盾。接收失敗不原子覆寫合法值；production adapter 另採 fail-closed 清空策略。

取樣事件必須在 pad_frame 前生成；不能修改 PaddedTeamFrame.frame 後繼續送舊 wire_bytes。網格在固定節奏發布，即使所有格子未變，也不依隱藏活動決定消息節奏。

追趕會跳過 intermediate presentation extraction，因此 runtime 必須在每個成功 Applied 後立即從 applied_public_events 保留 grid，不能等到繪圖時才收。新增唯讀借用 API，不額外 decode 整個 frame、不 drain 原 HUD／gameplay 事件；Duplicate／Rejected 不更新 cache。

編譯地圖範圍是公開呈現包圍盒，不是可走邊界或 stealth／target 能力判定。無 compiled map_id 的舊直線單路不猜 geometry，因此仍沒有正式 grid。這不是使用固定 demo tile size 的替代解。

runtime 經 verified rebase 後清空 grid；下一份資料必須符合新的 view epoch。rebase manifest 本身未新增 grid，projector 既有 epoch 推進與完整 rebase 呈現契約仍須後續串接，不宣稱本批完成 rebase 迷霧恢復。

## 當前確認

`cargo test --manifest-path omoba-core/Cargo.toml authority_fog_ --lib`：6 passed，0 failed，343 filtered out。包含真正 projector frame 的 wire decode／padding、雙隊私人 bootstrap、節流、explored、資料衝突與前批 LOS／邊界驗證。

新增追趕修正後只跑 `cargo test --manifest-path omoba-core/Cargo.toml authority_fog_catchup --lib`：1 passed，349 filtered out。真正 selective replica 連續套用 8 個 frame、不做 intermediate extraction；最後無 fog event，仍保留 tick16 的 explored。共 7 個相關測試成功，非全套回歸。

`cargo check --manifest-path omoba-client-runtime/Cargo.toml`：成功；只有既有 template build script dead-code warnings。本批未修改 ABI／Unreal、未重建 UE、未執行 MCP／雙 UE／完整 60Hz 對局。Unreal 仍顯示 VISION N/A；後續接 typed IPC、bridge 與原生 minimap 三態繪製。20/30 項維持不變。

## 操作錯誤

E139 記錄 PowerShell 不支援 shell brace expansion、錯誤的 rg 路徑 glob，以及過寬搜尋造成截斷；後续使用已知明確路徑、目錄搭配 --glob 與較小程式區段。沒有因此宣稱讀取成功，沒有增加 fallback 工作流。
