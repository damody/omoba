# 召喚師資源提示：最終前端驗收

OpenSpec `complete-summoner-resource-feedback` 3/3完成。

共用內容operation產生bRequiresManaPool，OmGenerated既有技能身分消費端將其連同名稱／說明複製進typed HUD row；OmRuntime只依該資料與目前owned mana做共用判斷，沒有反向模組依賴、英雄／技能名稱特判、新FFI欄位或權威結算。卡片disable／tooltip、self preview與owner refusal共用政策；送出瞬間再次檢查目前資源。缺池、零容量、非有限／負值／超出容量明確顯示「需要有效的魔力池，目前無法施放」，不誤稱魔力不足。空魔及滿魔皆可使用回魔，冷卻優先，自身技能不套blink控制／落點限制。

完整codegen **81通過／0失敗**，arbitrary catalog53由operation決定true／false；UE5.8建置／staging／DLL-import／離線catalog一致性成功。原生 **5成功／0測試warnings**：SummonerCastFeedback（含新增生成metadata→actual bridge→typed payload、兩個slot八種資源狀態、owner refusal／initialized widget／tooltip／self preview／cooldown／scope clear）、SummonerOwnerCards、SummonerSelfGuard、SummonerManaIdentity、SummonerPayload。保留既有card拒絕與input-id守門回歸；沒有新增真網路對局、實體鍵盤／DPI或畫面截圖宣稱。

這是前端範圍驗收；後端／core／script Rust來源未變，沿用前批sealed九套2100通過／9ignored，**不宣稱本批重跑九套**。既有moba-config binary SHA與catalog data hash和前批完全相同；ABI36／presentation12／selective14維持。未知synthetic catalog row維持原本非資源政策，實際未知身分仍由既有Rust/bridge catalog admission拒絕；此旗標不是授權。

**3489來源／資產一致**：相對前批3488中3477未變／11變更／1新共用policy header。集中驗收前後hash／set／size／mtime一致，372份證據封存並獨立重核：`docs/acceptance/2026-10-25-summoner-resource/index.json`；ledger `omfue/Saved/BuildTemp/summoner-resource-final-ledger-v1.json`。

V1原始API48711 actual1／編譯C1083失敗與frozen inputs保留，V2 API32696 actual1（native fixture1failed／4passed原始report保留）、V3 API68975 actual0；原始worker與cleanup退出，health≤180秒。錯誤SR01–SR04、根因與防重犯方法見 `docs/plans/2026-10-25-summoner-resource-errors.md`；沒有循環依賴／include-path暫解、沒有commit／push／quota reset／omfx或無關程序終止。

整體MVP33/44與11外部待驗維持；後續仍需更多玩法內容、目標型技能、呈現／美術及外部接受。

補充消費端驗收：生成class surface signature由前批9660373696db6cb4變為本批8d15c6b63cb800d2，因此在V3完整退休後，另跑fresh bridge/client兩套Rust，共301通過／9既有ignored／0失敗；codegen81另計。消費端測試後再次對同一3489來源baseline做完整hash／set／size／mtime複核。這是本批三套前端Rust範圍，不宣稱新跑後端／core／script等九套；後端moba-config二進位與catalog data hash維持前批值。
