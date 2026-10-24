# 持續區域公開狀態：最終驗收

`complete-ground-zone-status` 完成4/4 tasks。在既有buff catalog最後追加「持續區域」，舊ID保持；公開投影只接受已校驗DamageZoneSpec與正值、至多30秒的剩餘生命期。只有名稱、非法資料、到期或超上限資料不能產生有效提示。多個區域合併為一項公開狀態，顯示最晚結束時間，不揭露來源數量、內部key、落點、amount或JSON。

既有角色資訊／狀態清單顯示「持續區域（剩餘N秒）」，倒數、刷新與完整缺列退休沿用既有投影；死亡handler立即清除施法者的區域，不再等待下一buff tick。HUD退休亦清空所有buff字串／來源快取，重新顯示同一角色不會復現舊狀態。技能說明由相同rank extras產生，相同數值只顯示一次，傷害名稱保持通用。沒有新增wire欄位／C ABI layout，沒有英雄／營地名稱特判。

## 集中結果

1795passed/0failed/9existingignored；3428正式輸入一致；117份證據／30PNG；health7 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|115|0|
|codegen|68|0|
|core|958|0|
|base_content|310|0|
|bridge|136|1|
|client|151|8|

新增三個core測試：合法多區域合併／倒數／刷新／到期與public codec無payload，偽造名稱／非法payload／生命期拒絕，以及顯式Death同批清除。新增正式60Hz輸入驗證炎襲後公開狀態、pause、2.5秒倒數與死亡；新增bridge測試驗證視覺instance在倒數／刷新保持、世代替換退休、到期消失與空payload。

正式backend與UE5.8 build／部署成功，staged DLL與import library SHA一致。World67全部執行，0 Failed/NotRun，保留既有4個RuntimePresentationReset warning。新增Om.Runtime.GroundZoneStatus從真正public frame建立owner狀態，初始化UOmCommandBarWidget與Slate root後檢查角色資訊的中文倒數、owner/gen隔離、到期與scope退休，並截取真正共享tooltip像素。compiled-map公共ABI export另行成功。

30PNG：24張與前批完全一致，6張逐張實際view（包含新zone-status/zone-status.png）。原有首殺下／右HUD裁切與部分hero/focus名牌重疊保持限制。本批是角色資訊中的狀態倒數，未宣稱新增常駐進度條、地面貼花或獨立postdeath zone；持續引導仍未完成。

## 證據與範圍

原始錯誤、root cause與防止重犯方法見errors MD。Rust V1成功退休後，保留兩個C++修正前來源，依程式檢查補上HUD快取清理與同角色重現斷言；沒有捏造尚未執行的native失敗。Rust來源未變更，首次UE建置／World67驗證最終C++。所有final/history owner與watcher核對creation token退休，170秒health timer停止reap；來源exact set/SHA/bytes/mtime與封存bytes/SHA獨立驗證成功。[證據索引](D:/omoba/docs/acceptance/2026-10-23-ground-zone-status/index.json)；[錯誤紀錄](D:/omoba/docs/plans/2026-10-23-ground-zone-status-errors.md)。

11外部父項、MVP33/44及完整combat replica保持pending。無真網路對局／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。
