# 2026-10-24 共用彈體飛行預算驗收

OpenSpec complete-projectile-flight-budget 3/3。正式 Projectile Sys 的移動與掃掠共用 remaining-lifetime 限制的飛行預算及同一整數端點。最後一 tick 不再掃掠到宣告終點之外；剩餘壽命不足 tick 時只前進剩餘時間；不足一世界單位或最小正 raw 距離仍須有足夠步長才命中。

checked_travel_endpoint 從既有 checked_line_endpoint 的整數運算抽出：raw 平方距離、整數平方根上取整、分量朝零取整，避免 Q10 squared length 抹除微小方向。零預算合法靜止、負預算／不支援座標閉合。既有瞄準 wrapper 維持10000 range與零方向拒絕；實際彈體不支援幾何則退出且不傷害。既有命中半徑的端點圓盤、傷害歸因、正常命中與移除順序保留；不把碰撞半徑內的正常端點接觸錯當越界。

## 驗證範圍

- 真正 Projectile Sys 五案：100終點／1000步長不傷200目標；16 raw tick／8 raw壽命只走50，不傷60目標；0.5單位零速不命中；1 raw距離零速拒絕、有1 raw步長接受；正常端點只產生一份傷害與命中。
- 共用數學測試遍歷 -32..32 raw 雙軸與五個預算，驗證保守平方步長、可達端點與非法輸入，並重跑既有直線瞄準契約。
- 集中九套 Rust（含 omoba-sim）、後端與 UE5.8、DLL/import staging、離線 plan-lock 權威↔generated catalog 一致；實際數量見下方。
- 原生 ProjectileCueHistory 與 ProjectileCueStyle 兩案、0 test warnings。這是未改動 UE 呈現的 cue 回歸；本批沒有新增原生截圖，也不宣稱新飛行技能或新網路對局。
- 3487來源／資產在驗收前後 hash／檔案集／mtime一致；對前批3485未變、2精確變更、0新增。原始失敗／成功凍結來源及五個工作原始 exit、退休、cleanup與150秒健康監測均封存，完成後再次獨立核對 SHA。

## 保留界線

現有 straight projectile 沿既有 creep碰撞政策；新增資料驅動飛行技能、通用 MOBA skillshot 單位碰撞與多段技能仍是後續內容。本批修正共用飛行終點與壽命，不宣稱擴張命中對象、地形遮擋或範圍政策。MVP仍33/44、11外部待驗；無新真網路對局／quota重設／commit／push／omfx維護。

錯誤／根因／防重犯：docs/plans/2026-10-24-projectile-flight-errors.md。F03保留3失敗/1正常案例，F04保留raw1追加失敗，沒有以中間成功覆蓋失敗證據。

Fresh Rust 2082通過／0失敗／9既有ignored；native2／0warnings；157份獨立證據。索引：docs/acceptance/2026-10-24-projectile-flight/index.json。
