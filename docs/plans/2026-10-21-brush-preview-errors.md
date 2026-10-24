# 草叢與技能落點提示錯誤紀錄

## P01：Windows rg 路徑萬用字元

調查時誤將 `OmRuntime*`／`OmCompiledMap*` 當作原生路徑傳給 rg，Windows 不展開，回報 os error 123。改以已存在目錄加 --glob，或 rg --files 先列實際路徑；不得再將星號拼入 path 參數。

## P02：Saved 根目錄辨識

交接摘要以 Saved 簡寫，實際是 omfue/Saved。第一次讀取根目錄 Saved 的 helper 失敗；已以實際 omfue/Saved/BuildTemp 找到來源，後續一律使用完整專案路徑。

OpenSpec CLI 在本機未安裝，本輪按既有 skill 流程手動維護 artifacts；不聲稱 CLI validate 成功。

## P03：新 codegen fixture 必須滿足執行技能契約

BrushPreviewRustV1 的新測試遺漏 ability_type／cast_type，驗證器正確拒絕 effects require bounded instant active/ultimate。補上 active／instant，保留 production validation，不放寬斷言。V1 失敗證據保留，使用新 V2 結果重驗。

## P04：跨介面的草叢可見性

檢查發現 targeted item 使用即時視野來源驗證，不只 Wave B 公開集合；同時接入共用 brush_allows_vision，避免草內目標仍可被單位道具指定。地圖啟動也明確覆寫空草叢，避免切回舊單路時沿用前一張 map 的資源。

## P05：不要由型別名稱推測 component 檔案路徑

調查 item 測試時嘗試讀 native/comp/faction.rs 與 pos.rs，檔案並不存在；該型別從既有模組匯入。改直接依既有 ECS 測試註冊／使用型別，新的真實 World 可見性測試通過。後續先 rg --files／rg 型別定義，避免重複猜路徑。

## 驗收過程

## P06：公開草叢是膠囊形，權威不能使用外接矩形

WorldV1 畫面與 MapSurfaceActor 實作核對發現 Jungle surface 是 plane 加圓端，並未覆蓋外接矩形角落。原始矩形 contains 會在可見綠地外隱藏目標，不能接受。改以相同 bounds 的 capsule 距離測試；全程 doubled raw 固定點，保留半 quantum 中心，不使用浮點或猜測內縮。補橫／直軸、圓端、外角、邊界及極值測試。V2 Rust／build-v1／WorldV1／local-v1保留為前次來源 cohort，最終改用完整新 cohort，不假稱原驗收涵蓋此修正。

local-v1已主動退休當輪擁有的UE，完整工作流依預期退出1並清理後代；superseded-v1.json保留PID／creation token。它是過期來源的部分驗收，既不能當成功，也不能當產品自動測試失敗。

## P07：短建置在掛監測前已退休

backend-v2已在掛watcher前完成，watcher正確拒絕 invalid owned PID；不重試相同PID、不把快取成功誤判卡住。已讀取原工作流exit0與正式staging輸出確認完成。後續先看完成收據／目前存活程序，再決定是否掛watcher；短於180秒且已有退休收據的操作不需要補造週期樣本。

## P08：來源比對誤納正式建置產物

最後 broad guard 正確攔下 Source/ThirdParty/OmBridge/lib/Win64/om_bridge.lib 的建置更新；該 import library 是12:02的正式 build-v2產物，不是程式來源，且早於12:04完整接受開始。第一次 guard 失敗及隨後因 guard 缺檔而拒絕的 seal 均保留，沒有建立成功封存。重新完整雜湊 snapshot 並盤點全部差異，另找到 scripts/base_content.dll，亦為正式stage輸出。最後僅這兩個產物更新，其餘3339來源／資產SHA、大小、mtime完全一致。兩產物逐byte SHA符合current build、時間早於full run，兩個staged DLL亦符合build-v2預先記錄的SHA。最終報告分列3339來源與2個有建置證明的產物，不聲稱3341來源皆未變。最後seal及獨立verify成功：13份byte-exact JSON、Rust1636／10、8次監測／最大180秒。

V1 aggregate 因P03保留失敗；V2為膠囊修正前cohort，local-v1主動退休。最終接受改用V3 Rust1636／10、build-v2、backend-v2、WorldV2及完整local-v2。所有被取代結果保留，不能把過期aggregate或部分流程寫成完整成功。
