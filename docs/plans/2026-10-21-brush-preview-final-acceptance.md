# 草叢隱蔽與技能落點範圍：驗收

日期：2026-10-21。工作目錄 D:/omoba，Unreal D:/UE5.8。未新增權威／網路對局，未重設既有對局配額；所有實作驗證使用 Rust World 模擬及本機 synthetic renderer／IPC。未提交或推送。

## 實作

- 兩張三路 map 各新增四塊可通行草叢；optional brushes 使用最多 32 個不接觸外接 bounds 的膠囊形、唯一 ID、地圖內閉合邊界。舊地圖空集合保持相容。Lua 由相同 bounds 產生公開地面／邊緣，沒有把整片裝飾叢林誤當隱蔽區。
- 草叢獨立於 terrain collision／opaque LOS。草外來源不能揭露草內目標，同草叢內的友方英雄／眼等來源仍須通過距離、偵測等級、既有 opaque LOS 資源。草叢不遮住後方草外目標。Wave B 單位揭露、迷霧格網及 targeted item 即時驗證共用判定；原本 Public／OwnerTeam／ForceShow／ForceHide 規則保留。權威使用與地表相同的膠囊距離判定，doubled raw 保留半 quantum 中心，外接 bounds 角落不會誤隱蔽。
- 生成器從 executable AreaDamage 的 radius_key 讀各級半徑，point 同心圓聯集取最大值；非 point／無此效果為零。HUD 僅發布已揭露有效等級，依 WorldUnitsToCm 換算。Slate 顯示落點傷害圈與較細的施法距離圈，共用玩家 viewport、DPI 轉換、不可投影分段裁切；未學習、未知、技能替換、非法值及 runtime 退休不殘留。
- 箭雨各級 220／240／260／280 的實際效果資料已接入，不採用 cosmetic cue 半徑，不猜測特殊手寫技能形狀或敵方私有狀態。

## 已完成的本輪證據

- BrushPreviewRustV3：template59、core795、codegen57、base_content246、backend190、bridge127、client147、ABI15，共 **1636通過／10既有ignored**。V1 codegen fixture 失敗保留；V2 是膠囊修正前來源，不採其 aggregate 作最終接受。
- 三種子 5v5 各4500tick、完整三路生命週期、野怪返回、護主、塔與基地保護、兵種／超級兵等既有回歸包含於上述 suite；這些是記憶體模擬，不是新增服務端對局。
- Lua 本機驗收契約69、owned IPC契約140與多 map ID 契約通過。
- brush-preview-build-v2：UE5.8 UBT 成功，3.14秒；backend-v2 建置／staging成功。
- BrushPreviewWorldV2：28項，27 Success＋1 SuccessWithWarnings，4警告，0失敗。PointAreaPreview 在實際UE執行成功，覆蓋每級metadata、HUD快取、未學習／替換／NaN；不是權威技能ACK。
- 已查看兩張 map 的總覽與遊戲視窗畫面；草叢以既有地面／邊緣呈現，不宣稱專用草叢美術完成。
- WorldV2 兩 map 的 overview／spawn 四張 PNG 與已檢查的 WorldV1 逐 byte 相同，另保留各74幀／6片段。膠囊修正只改權威幾何，沒有更換畫面。從最終 local-v2 前半段已關閉的原始 AVI byte-exact 擷取10張 JPEG，frame750 的 Q「請選擇可見單位／左鍵確認／右鍵或Esc取消」可讀；frame450、550是其他狀態，不當作施法提示證據。這是共用提示的畫面回歸，並未宣稱該原始影格展示箭雨 AoE。

完整 LocalAcceptance `brush-preview-20261021-v2` 成功：**207 native（199 Success＋8 SuccessWithWarnings）、458cases／222intents／157stages／13cursor、69Blueprint、五段實際PCM**。native12警告、World4警告、音訊1警告保留；Blueprint0編譯錯誤／警告／載入失敗。音訊裝置為 Steam Streaming Microphone，驗到數位PCM，沒有宣稱實體喇叭聽感。完整操作採 foundation／tail 兩程序的嚴格coverage union，**不是單session效能通過或權威技能ACK**。

local-v1 在膠囊修正後主動退休，部分證據保留，退出1不列為產品自動測試失敗或完整成功。完整local-v2已正常退出0、工作流及UE／publisher後代均已退休。

最終接受封存在 [index.json](../acceptance/2026-10-21-brush-preview/index.json)：**13份原始JSON逐byte複本**，另核對Rust／正式workflow日誌SHA及10張原始JPEG的SHA。8次完整流程監測，最大間隔**180秒**，最後已退休。

來源指紋盤點共3341檔，其中**3339個來源／資產前後SHA、大小、mtime完全一致**。另外兩檔是正式建置產物：`om_bridge.lib`／`scripts/base_content.dll`，於12:02前更新，早於12:04完整接受。已逐檔確認符合 current build 輸出，兩個 staged DLL 也符合 build-v2 在接受前記錄的SHA；不聲稱3341檔都未變。原 broad guard 及提前seal拒絕記錄保留，詳見P08。兩個freeze盤點之間只改brush_vision.rs，Lua／模板契約來源未變；修正版UI仍已另跑完整local-v2。

## 正式驗收入口

在 clone 根目錄執行；重跑接受流程需另取新的 run-id，保留既有證據。

```bat
tools\lua\lua.exe scripts/build_ue_moba.lua --ue-root D:/UE5.8 --build-executor local --disable-action-cache --prefetch-shared-pch --max-parallel-actions 20 --compiler-version Latest
tools\lua\lua.exe scripts/ue_local_acceptance.lua --ue-root D:/UE5.8 --run-id brush-preview-20261021-v2 --owned-execution phased --record-video
```

## 接受界線

本輪不等於所有 LOL 機制完成。攻擊時短暫揭露草叢、連接多邊形草叢、編譯地圖牆體的獨立視野遮蔽接線、特殊手寫技能的精確扇形／路徑、史詩怪增益與分裂，以及專用草叢／水晶美術仍可後續增補。單位指定預覽仍只稱候選目標，不替權威判斷距離／陣營／命中。完整 MVP 仍為33/44，11項外部接受缺口未以本機證據代替。
