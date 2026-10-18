# 2026-10-18 可攜交接證據

請先讀專案根目錄 `handoff.md`。這裡保存舊電腦已完成驗收的摘要與少量原圖，不是假設新電腦已通過。

- `index.json`：27/44進度、測試數、資源與限制。
- `accepted-local-summary.json`／`full-local-acceptance.json`／`focused-editor-acceptance.json`：原始成功報告，未改寫測試結果。
- `rust-reuse-evidence.json`：本輪沿用1168 Rust／五checks／九ignored的明確依據；441個已擷取輸入未變，沒有宣稱fresh Rust。
- `original-source-sha256.json`：1197個當時實際輸入的原始SHA與相對路徑；一般文字的Git checkout行尾可能不同，不能把這份歷史指紋當新機測試。
- `input-window/`：八張直接審查過、由AVI直接取出的JPEG body；黑底輸入fixture、tooltip與400×300視窗都原樣保留，沒有補畫／裁切／縮放。
- `world/`：兩張較早已接受的正式compiled-map fixture出生視點PNG，非此次新開對局。

原始報告中的絕對路徑、PID、時間與run-id代表舊電腦歷史，不應在新機執行、刪除或沿用其程序身分。約138MiB的AVI、PCM、log、cache與建置產物沒有提交，請用正式Lua入口產生全新證據。
