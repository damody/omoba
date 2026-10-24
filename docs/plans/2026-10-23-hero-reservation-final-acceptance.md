# 英雄名牌保留區：最終驗收

完成 complete-hero-nameplate-reservation。共用固定容量矩形機制保留當前可見英雄的真Slate足跡，讓一般小兵／野怪／建築名牌避開英雄。選取與指向的非英雄可跨英雄保留區，仍服從非英雄間優先避讓。英雄彼此不隱藏、不移位；焦點例外與超過64英雄的保留上限仍明確存在。

英雄候選採自身／選取／指向／中央距離／實體世代的穩定順序，最多64英雄與64非英雄量測；驗證目前渲染可見性、生命、披露與世代，使用目前mesh錨點。每視角重建保留區，隱藏、退役、移動與換世代後無殘留障礙。

## 驗收

- Rust **1966通過／10既有ignored／0失敗**，12組含內容生成；四項Lua契約、正式後端與UE5.8建置成功。
- World **59成功／0 Failed／0 NotRun**；新HeroNameplateReservation與既有NameplateScreenSpacing、HealthNameplateBudget成功。既有RuntimePresentationReset場景銷毀warning保留。
- 幾何邊界、非法資料、獨立64容量、普通／焦點例外、真英雄隱藏／重現／重設／換世代及Slate截圖驗證。
- **3407來源／資產**集合、SHA、size、mtime前後一致；7個staged DLL與import library一致。
- 封存 **108份證據**並獨立逐byte／SHA校驗；程序退出後目視 **20張PNG**，包含普通避讓、焦點例外及兩地圖。單機fixture不替代外部ACK、實際OS DPI或固定效能接受。
- 健康採樣 **6次、最大間距171秒**；7組worker／watcher及health程序均退休。

## 錯誤與接續

本批沒有建置／測試失敗。一次診斷讀取誤猜health.log副檔名，實際為health.jsonl；已記錄原因並要求先查實際檔案名稱，詳見errors.md與總登錄E1499。

48小時無人值守omoba-48繼續至2026-10-25 00:33:38 Taipei。持續引導／區域、定時位移與特殊幾何、召喚整組commit、combat replica生命期與內容美術音效仍pending；MVP33/44、11項外部接受不變。未新增真網路對局、quota reset、commit/push或omfx維護。

證據索引：`../acceptance/2026-10-23-hero-reservation/index.json`。
