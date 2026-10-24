# 連敗擊殺金折價錯誤紀錄

- D01：盤點時錯用 `omoba-core/src/runtime_content.rs`，實檔為 `omoba-template-ids/src/runtime_content.rs`。後續先用 `rg --files` 定位。
- D02：Lua經濟設定替換預期八格縮排，實際四格；assert在寫入前中止，未造成半套設定。改以實際來源片段並保留精確次數檢查。

- D03：集中建置發現新模組的父模組匯入僅供測試使用，正式library出現unused import警告。將匯入限制在cfg(test)，保留V1紀錄，最終Rust／建置改以修正後V2重新接受；不以修正前產物配修正後來源封存。

驗收錯誤與解法會追加在此；既有測試失敗不得靠降低斷言解決。
