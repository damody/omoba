# Bot recovery channel wait — final acceptance

共用 role bot 現在會等待自身已開始的恢復引導，不再以新的購買、移動、攻擊或施法命令打斷它。規則同時支援既有道具與已實作的引導技能；自然完成或接受中斷後，下一個正常 think 即恢復規劃。

新增 recovery_channel::is_active_for_owner 讀取私有 Channels 的 exact Entity generation、owner、remaining，沿用 live owner／生命／phase／silence 與原點位置條件。bot 只查自己當前 roster slot 的 owner，不查敵人的私有通道。公開 recovery_channel duration buff 只供呈現，不能偽造有效引導來凍結 bot。共用決策入口在既有 recall 旁邊處理等待，沒有另設 timer、發出 Stop/Hold 或修改原本視野、節流與導航。

## 驗收

- ProbeV2 的兩項正式60Hz案例：實際作者技能與實際基地購買的道具，都完成全部治療脈衝；定身與暫停不讓 bot 自行取消，完成後恢復決策；每個技能／道具情境都做 paired replay。另一案例驗證假公開狀態不會凍結 bot、另一 owner 照常決策、正式 Stop 中斷後恢復。
- 集中八個維護 Rust 套件 1982 passed、0 failed、9 個既有 ignored；上述 focused2 不再加到總數。
- 正式後端／moba-config、UE5.8 建置與 DLL/import library staging SHA 一致。
- 3479 個正式來源與資產在建置後完整 fingerprint 一致；相對已封存 recovery-skill 父基準為3476 unchanged、3個明列變更，沒有新增／刪除來源。三個原始檔、每版固定輸入、raw logs、owner/waiter、health、exit receipts、retirement、cleanup 和原始 API reap 都保存。
- 沒有前端／作者資料改動，沒有宣稱本批新跑 native 畫面或 Lua 作者套件；相關舊證據依父 index 與完整來源差異證明保留，這批新功能由正式 bot→PlayerInput→authority tick 驗證。

失敗 probeV1 完整保留：隔離測試位置不在商店，ItemBuy 被正常拒絕。V2 只修測試的前置購買位置並檢查庫存，沒有放寬正式商店／通道規則；runtime兩檔在兩個 probe 間 byte-identical。錯誤與預防見 docs/plans/2026-10-24-bot-channel-wait-errors.md。

證據索引 docs/acceptance/2026-10-24-bot-channel-wait/index.json，含 sealer實際API／事後獨立重算。下一批來源基準 omfue/Saved/Automation/BotChannelWaitV1/source-before.json，凍結 inputs.json，父綁定 bot-channel-wait-parent-v1.json。

下一個具體缺口：bot 尚未有自動施放 authored channel 的 typed intent；目前只保護已開始的技能與道具引導。接續新增封閉 SelfChannelRecovery 意圖，校驗實際 host execution 類別、等級與公開安全條件，透過正式輸入驗證開始／等待／中斷／恢復。不要全域放寬任何 channeled cast。其他 projectile/multistage、召喚控制、美術／內容／平衡與11個外部接受仍pending；MVP33/44不變。

沒有新增真網路對局、重置quota、commit/push、維護omfx或終止無關程序。無人值守期限2026-10-26 23:59:59 Asia/Taipei。

獨立封存檔案數 107。
