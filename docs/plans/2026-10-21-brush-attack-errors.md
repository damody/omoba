# 草叢攻擊揭露：錯誤與防重複紀錄

沿用前輪原則：來源與derived outputs分開核對，新的失敗cohort不可覆寫；音訊runner保留Engine與Editor兩層process-local背景音訊設定；長程序啟動即掛owned watcher。OpenSpec CLI未安裝，手動維護artifact。

## R01：私有兵線索引與公開canonical ID不可混用

RustV1 core三個新測試失敗，測試helper把entity_key（id高位）拿來查Wave B（generation高位）的canonical_id，並非正式揭露失效。helper改呼叫既有canonical_entity_id；保留V1 log，另重跑新cohort。正式grant查詢仍一致使用私有state索引，公開stable_rule_id使用公開canonical ID。不放寬可見性斷言。

## R02：新路徑不可假設簡化測試world已註冊所有storage

實作檢視時將即時targeting的CProperty查詢放在MobaMatch及有效grant檢查之後，保留無match的legacy／簡化world行為；不是透過補註冊讓新程式隱性依賴更多component。調查有猜測不存在faction.rs的讀取，後續依rg實際路徑查找。


## R03：真實World驗收要區分受測entity與其他有效行為

RustV2 core 805通過／1失敗，指定攻擊者到期後已不可見，但測試要求整個grant map為空，忽略同時AI出手的小兵。改驗該entity的所有key已清除，保留其他合法grant與原定到期／暫停斷言。另加入有視野來源的第三陣營，證明grant不會對其他觀察者公開。V1／V2只保留失敗cohort，不採為成功；退休仍在跑的superseded工作後完整跑V3。


## R04：rg不是PowerShell cmdlet

診斷時把-ErrorAction傳給rg，造成參數解析錯誤。改用Test-Path後再執行rg；不可把檢查工具錯誤或無輸出當成產品建置成功。正式結果仍以workflow exit／receipt及原始build log核對。


R04補充：一次診斷命令的if區塊缺少右大括號，PowerShell在解析階段拒絕執行，未修改來源或驗收結果；改成直接讀取已存在目錄，避免不必要的巢狀條件。


## R05：普攻入口必須包含獨立野怪AI

V3 Rust及UE WorldV1通過後，覆核發現野怪tick直接產生DamageInstance，不經英雄／script／lane cue。不能只因現有測試通過就忽略此正式入口。抽出接受state參數的共用內部函式，野怪含附屬怪在真正出手後使用同一grant；中立攻擊者可對非中立受擊team揭露，同隊／中立受擊方仍不授權。新增真實camp AI在brush內出手的World測試；主動退休尚未完成的local-v1，保留其結果，用新RustV4、build-v2／WorldV2與完整local-v2重新驗收。兩秒期限改具名常數，仍用模擬時間。


## R06：野怪fixture應由正式受傷／仇恨入口建立狀態

RustV4 core因E0616編譯失敗，新測試直接寫入兄弟module的private camp.target。改呼叫record_moba_jungle_damage建立仇恨，沒有擴大欄位可見性；正式production code未變。保留V4編譯證據，另完整跑RustV5。此fixture調整只在cfg(test)來源，UE build-v2及backend-v2的正式程式碼保持不變，後續WorldV2／local-v2使用current staging。
