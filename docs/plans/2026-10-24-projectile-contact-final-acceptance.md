# 2026-10-24 彈體最早接觸驗收

OpenSpec complete-projectile-first-contact 3/3。沿現有 creep 空間範圍查詢選取最早 Q32 接觸，取代「距掃掠中點最近16筆」再取第一個命中。這修正前方單位被跳過、離軸中點人群讓真正路徑目標消失，以及缺少當前位置卻以原點代替的幽靈命中。

## 共用機制

CollisionIndex::select_in_range_by_key 對既有同一空間查詢返回的所有候選套用 caller key，只保留一個最佳結果。舊 search_nn 本來也先取得全部空間候選；沒有新增全世界實體掃描，也沒有額外 nearest 結果陣列。所有四種索引與反向重建均驗證。當前 Pos 與 Creep 必須存在，索引中的舊列不構成命中授權。

swept_circle_contact_q32 先以既有封閉整數 capsule 判定整段，再對單調的 prefix intersection 搜尋第一個 Q32 接觸。支援範圍內的 scaled raw squared distance 使用 i128，不使用浮點根或浮點排序；最多33次二分。接觸向上量化，同一量化內以實體id/generation穩定排序。這是明確的離散時間規則，不宣稱比較小於 Q32 量化的解析實數根。現有 f32 查詢僅為保守 broadphase，5單位padding保留。

## 實際驗證

- 真正 Projectile Sys 七案：前後目標、32離軸人群、Pos缺失、Creep類別缺失、反向重建同時接觸、半徑入口早於中心順序、相切與1raw外側。
- selector 一案涵蓋 hash_grid／sap／quadtree／bvh、40候選、反向重建、相同key與空／非法查詢；shared sim三案涵蓋Q32軸向預期、端點圓盤、零半徑／靜止、large translation的1raw邊界及raw grid一致性。定向11案通過。
- 集中九套 Rust 2093通過／0失敗／9既有ignored；正式內容含既有60Hz對局生命期、雙複本與重播。後端／UE5.8、bridge與base_content DLL/import staging、權威plan-lock與generated catalog一致。
- native ProjectileCueHistory／ProjectileCueStyle兩案、0 test warnings；既有UE呈現未改，本批沒有新截圖或新技能。
- 3487來源／資產在驗收前後hash、檔案集與mtime一致；相對前批3484未變／3變更／0新增。原始3失敗/1控制案例、後續成功凍結來源、四個工作的實際exit／退休／cleanup與健康監測保留，封存後獨立SHA重核。

## 接受界線

維持既有creep-only碰撞候選、傷害／Camo等結算與命中效果中心政策。不是新增MOBA skillshot對所有英雄／小兵的碰撞，也沒有增加地形遮擋或新飛行技能。舊中點nearest截取缺口已關閉，更多正式飛行技能與多段技能仍可擴充。MVP維持33/44、11外部待驗；未新增真網路對局、重設quota、commit/push或維護omfx。

錯誤／防重犯：docs/plans/2026-10-24-projectile-contact-errors.md。索引：docs/acceptance/2026-10-24-projectile-contact/index.json。

獨立封存檔案：151。原始API、sealer與closure均已實際回收。
