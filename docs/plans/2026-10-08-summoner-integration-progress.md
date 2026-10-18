# D/F 召喚師技能整合紀錄

此文件記錄已驗證的增量，不能代替完整操作計畫驗收。最新完整計畫為8／44父項完成；下方6／44與未完成敘述保留各批歷史狀態。

2026-10-09 最新：D/F資料／權威元件／transport／ABI／UE操作整合已完成task3.8。`.ai-collab/owned-summoner-accepted-v1/summary.json` 保存build-v3 nocache PASS20.81s、native-v2 exact63Success（1筆RHICore警告）、真renderer IPC viewport-v2 exact158／69／51與cleanup全過。原小地圖121／58／36前綴實際比對成功。兩槽normal／quick／indicator、自身／地點、取消／焦點／modal／profile與冷卻／未裝備／死亡／來源變更已驗；11圖5新目視6相同沿用。E527／E528失敗及修正保留。這是local admission加既有權威元件證據，仍不是真人對局authority ACK／結果；7.1／7.3／7.4與完整最後驗收保持待辦。

## 已驗證資料邊界

權威槽位使用獨立 SummonerCast／slot 0、1，不借用 QWER 或物品槽。技能、施法形狀、距離和冷卻由可信伺服器配裝與內容目錄決定。死亡時保留裝備和冷卻資料，禁止施放；無配裝保持不可用。

Rust／protobuf 持續呈現增量：85 項測試及三項 all-targets 檢查通過。ABI18／IPC6 增量：89 橋接測試、16 輸入測試、28 呈現測試及三項 all-targets 檢查通過。失敗與修正保留於 E424。產生的 C++ 標頭已由正式同版 MSVC 通過 compile-only smoke，實際驗證 ABI18、兩槽相鄰、summoner_slot 不借 ability／item 欄位及結構偏移；證據 `.ai-collab/summoner-c-header-validation-v1/summary.json`。UE 模組與原生功能測試仍待完成。

## TOML 配裝格式

`ServerSetting.AUTHENTICATED_SUMMONER_LOADOUTS` 使用可省略的具名槽位，對應既有 Rust 兩槽模型；仍接受舊的兩元素陣列。下例是明確伺服器設定範例，不會自動套用到既有對局。

```toml
[server.AUTHENTICATED_SUMMONER_LOADOUTS]
"1" = { slot0 = "training_point_blink_v1", slot1 = "training_self_heal_v1" }
"2" = { slot1 = "training_self_heal_v1" }
"3" = {}
```

不支援的第三槽、錯誤陣列長度、額外欄位、非標準玩家 ID、未知玩家、未知技能及重複配裝均拒絕。TOML 不含 `null`；缺少槽位表示未配備，不能以預設技能填補。實際六項設定／傳輸測試通過，證據為 `.ai-collab/summoner-toml-validation-v2/summary.json`；第一輪 4／6 失敗保留並記錄於 E426。

## 目前正在整合

Unreal 已有兩槽 typed payload、frame lease 期間複製字串、死亡裝備保留，以及專屬提交介面的來源變更，尚未通過編譯及 native 驗證。審查修正 E427：不讓異常冷卻變成 ready，並保留空槽的正確索引。

選角主機、final plan、第二次 `--lock-plan` 與正式 game.toml 的可信配裝交接已完成獨立 v4 驗證：28 項 Rust 測試、51 項 Lua 檢查、兩項編譯檢查及 debug／release 設定工具建置通過。實際編譯工具經兩次 launch.prepare 保留雙槽、單槽及空槽；沒有啟動權威對局。證據：`.ai-collab/summoner-trusted-handoff-validation-v4/summary.json`。D/F 實體按鍵、共用施法狀態機、獨立模式設定及 HUD 卡片仍待編譯與整合，操作手冊不可提前標成可用。

原創位移及治療圖示已保存於 `omfue/Content/Omoba/UI/`，具有來源與雜湊紀錄；registry 已有 31 筆內容與兩張召喚師圖的 source 接線，尚待 native 驗收及卡片整合。沒有新增權威對局，既有 10／10 額度不重設。所有操作整合後才跑完整最終驗收。

### 2026-10-08 召喚師 payload 本批驗證

正式 controls-summoner-payload-build-v2 refresh 成功，且同工作流普通 NoEngineChanges／NoUBA／nocache gate 成功；ABI18 stage SHA 已核對。controls-summoner-payload-native-v1 真 D3D12 exact42 Success，failed／warning／notRun 全0，新權威對局0。三張 production HUD／settings／shop PNG 已逐張開圖；無 server／match，不當正式三路畫面。共用 aim/profile 六檔及響應式 HUD 五檔經獨立審查與 Primary 修正後 promotion，尚待新建置及原生驗證；D/F 實體 owner／settings／卡片由 Grok4.7Fast run-muytxfno-wz1dww 接續。父項仍6/44。
