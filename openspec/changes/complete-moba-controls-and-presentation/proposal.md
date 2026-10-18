## Why

目前本機 native／Blueprint 驗收通過，但玩家實際操作仍有明顯落差：小地圖左鍵不移動鏡頭、滾輪沒有縮放反應，畫面也缺乏成熟 MOBA 的辨識度與回饋。需要以玩家可完成的操作建立規格，補齊輸入接線、鏡頭、HUD 與場景呈現，避免再次以 helper 測試成功代表真實操作成功。

## What Changes

- 完成小地圖左鍵定位／拖曳、視窗範圍框、滾輪縮放、邊緣捲動、Space 回英雄與 Y 鎖定；定義 UI 焦點、游標捕捉與鏡頭模式優先序。
- 建立可重新綁定且可儲存的點擊式 MOBA 操作設定：移動／攻擊／停止、QWER 施法模式、升級、自我施法、回城、道具、商店與計分板；既有 Shift 佇列作為本專案擴充，不宣稱是 LoL 預設。
- 完成技能／道具／選取／商店的可視狀態與拒絕原因；規劃召喚師技能、飾品／插眼及隊伍標記的權威與視野資料契約，禁止只有本機假的按鈕。
- 改善可辨識的英雄／小兵／塔／基地、地形／路線、光照／材質、動畫／技能特效／音效、游標與 HUD 版面；以固定場景截圖與實際操作證據驗收。
- 建立實際輸入事件→有效 camera／原權威 ACK／可見 UI 的端到端驗收，涵蓋 DPI、焦點、重連及生命週期，不把直接函式呼叫當滑鼠操作成功。

## Capabilities

### New Capabilities

- `moba-camera-interaction`：鏡頭縮放、移動、鎖定／回英雄、小地圖定位與輸入仲裁。
- `moba-player-control-profile`：可設定的點擊式控制與權威 gameplay 操作，含尚缺動作的明確後端依賴。
- `moba-player-hud-feedback`：技能／道具／商店／計分板／生命週期狀態、操作回饋與可讀資訊。
- `moba-readable-presentation`：場景、角色、動畫／特效／音效、版面與畫面品質基準。
- `moba-team-communication`：隊伍限定 ping／標記、聊天輸入焦點與隱藏資訊保護。

### Modified Capabilities

無。本次新增玩家操作與呈現契約，與尚未封存的 `build-unreal-rust-moba-framework` 對齊；不將其未完成自然對局／LAN／效能項目視為已驗收，也不變更已固定的效能門檻。

## Impact

主要影響 `omfue/Plugins/OmRuntime` 的 camera pawn、player controller、Slate／UMG HUD、minimap model、runtime settings、OmEditor 測試與 `omfue/Config`；gameplay 新動作若確實缺少契約，再同步 `omoba-core`／`omoba-client-runtime`／`omb`／共用內容。鏡頭操作保持本機呈現，不新增玩家端模擬、不修改 Engine／授權、不維護 omfx。設定與測試流程沿固定 Lua；本提案階段不啟動新對局，既有 10/10 批次不重設。參照 Riot 官方說明與基本 MOBA 慣例，鍵位表是本專案決策，不複製 Riot 的資產或聲稱逐版完全一致。
