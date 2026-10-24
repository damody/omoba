# 英雄升等提示：最終驗收

`complete-hero-level-feedback` 完成4/4 tasks。既有權威英雄HUD payload已提供等級，但玩家主要只能從標題小字與角色資訊查看。新增共用FOmHeroLevelFeedback tracker，並接入真正command bar header「等級提升！目前為 N 級」。初次同步只建立baseline；只有同owner、同entity ID與正世代下觀察到更高等級才顯示，不從XP、技能點或冷卻推算。多次觀察到的提升合併為最新等級，不合成未收到的中間事件。

最高已觀察等級與elapsed單調檢查防止重複／倒退資料重播或延長；四秒呈現時間後收起。owner／世代更換、非法／缺少資料、非playing phase及HUD退休清除基線與提示，退休HUD不能被相同角色資料復活。接受native entity index0，拒絕負數／大於u32的ID及非正世代。呈現計時器與既有UI通知一樣使用本地frame delta，不讀取模擬pause狀態；它不影響升等、資源或遊戲時間。

真HUD使用既有header wrap／可讀字體／文字縮放／showHUD與內容版面失效，沒有新增wire／C ABI、gameplay規則或英雄名稱特判。

## 集中驗收

World70全執行／0failed／0notrun；3432正式輸入一致；101份證據／32PNG；health7 samples／max171秒。

UE5.8正式build與stage/import雜湊一致；World70全執行，0 Failed/NotRun，保留既有RuntimePresentationReset4個警告。新增Om.Runtime.HeroLevelFeedback檢查初次同步、升等、重複不延時、精確到期、到期不復活、倒退時間／等級、合併最新等級、NaN／負delta、owner／generation更換、end／missing、非法ID與退休等邊界。真正初始化UObject與Slate root，檢查widget文字及專用header節點可見，從實際HUD節點擷取PNG，非人工複製tooltip文字。

本批未修改Rust/backend/content；沿用前批1799passed／0failed／9existingignored的完整Rust與backend接受結果，並以雙向完整檔案集合檢查確認924個Rust／Lua／Cargo／proto輸入bytes/SHA未變更。此數字是繼承的既有回歸，未宣稱本批重新跑1799項。正式public ABI compiled-map export另行執行成功。

32PNG：27張與前批完全一致，5張逐張實際view，包含新hero-level/hero-level.png。既有first-kill HUD下／右裁切與部分map hero/focus名牌重疊仍保留限制；本批不把那些視覺問題勾為修復。

## 證據與限制

UE V1編譯成功退休後，程式檢查發現signed identity入口需要拒絕負世代／越界ID。保留5個原來源與SHA manifest後硬化並增加native斷言；UE V2及首次World70驗證最終範圍，沒有捏造未執行的紅測試。Source V1實際指紋檢查成功，但runner遺漏包裝器要求的exit=0標記，監控receipt判失敗；保留原始runner/receipt/logs與成功proof，再以source V2新output與正確標記完整檢查。初次sealer缺scripts Lua搜尋路徑，保留原script後修正；已有封存bytes完全相同才重用，未覆寫任何接受index。所有reader/watchers與單一170秒health timer退休reap，來源exactset/SHA/bytes/mtime與封存bytes/SHA獨立驗證通過。[證據索引](D:/omoba/docs/acceptance/2026-10-23-hero-level/index.json)；[錯誤紀錄](D:/omoba/docs/plans/2026-10-23-hero-level-errors.md)。

升等提示是收到狀態後的觀察性呈現，不保證網路未收到的每個中間升等事件。11外部父項、MVP33/44、完整combat replica與持續引導仍pending。無真網路對局／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。
