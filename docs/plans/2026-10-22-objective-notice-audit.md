# 2026-10-22 權威目標公告盤點

前輪經濟補償已完成，這輪接續17系統盤點的「拆塔／史詩目標公告」缺口。範圍限單機可驗證的正式遊戲接線，沒有新增真網路對局或以視野消失推測目標死亡。

權威種類含lane tower／inhibitor／nexus tower／具objective rewards的epic根怪，沿正向committed致死與既有生命期去重。tick結束依tick／完整entity key排序，現播公告不被打斷，最多七個最近待播；每則四秒scaled時間、pause凍結、到期換下一則、終局清除。重生後的新目標生命可再公告，普通野怪與匿名／友軍／強制退休不製造敵方成功。佇列與序號進replay digest。

公開HUD四元組team／kind／sequence／remaining_raw跨filtered public events、protobuf optional ObjectiveNoticePresentation、CABI34 OmObjectiveNoticeHud與Unreal typed HUD。維持既有首殺欄位／schema4相容性，CABI struct明確升版並更新Rust／C++ header smoke的尺寸與alignment，舊bridge被既有版本守門拒絕。checked-in proto fallback由既有OMOBA_UPDATE_PROTO_FALLBACK建置入口更新。

client拒絕缺項／重複（含同id但截短payload）／非public team／非法kind／序號／剩時，Unreal再次拒絕非法Blueprint payload。非法objective notice只清該公告，不清整個HUD。原生文字顯示藍／紅隊摧毀防禦塔、兵營、基地防禦塔或奪取史詩目標；沿權威狀態清除，沒有一次性音效重播。公告目前揭露通用目標類別，具體營地名稱／專屬音效仍可另擴充。

集中驗收完成：RustV2 1794通過／10既有ignored／0失敗、UE5.8 build-v2（UBT3.14秒）與正式後端成功、Lua三契約、WorldV2 42成功／0 Failed／NotRun／4既有警告。3369來源／資產集合、SHA／size／mtime一致，七項staging與import library另驗；六張圖片目視，專用公告列fixture也驗證死亡owner可看公告（顯示等待重生），不宣稱新真連線畫面。所有consumer／watcher與全程monitor均退出。封存細節以同名final-acceptance MD為準。來源與產物在驗收批次凍結；失敗全批退休後才修復，所有原始失敗紀錄保留，不合併不同批次的通過數。原MVP33/44與11外部ACK／LAN／OS DPI／效能證據仍待外部接受；無quota reset、commit／push或omfx維護。
