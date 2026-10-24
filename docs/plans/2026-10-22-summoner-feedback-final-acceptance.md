# 2026-10-22 召喚師施放提示最終驗收

OpenSpec `complete-summoner-cast-feedback`：5/5 完成。本批增加 D／F 地點召喚師技能的控制、原地位移、冷卻與落點提示，並修正權威原地閃躍消耗冷卻。

## 實作

- 真正的 PaintAim 呼叫共用 owner helper，從當前公開 row 校驗 protocol、存活／HP、擁有者、slot／catalog、重複 slot、目標類型、有限正值射程與完整冷卻數值；不合法或退休狀態顯示未知。
- 直接重用現有 bridge reflection 與 CastPreviewFeedback，呈現暈眩、沉默、定身與權威剩餘時間；不另設控制快取、倒數或本地施放守門。D／F 換位及任意 catalog ID 使用同一流程。
- 依目前封閉契約，Point summoner operation 是 PointBlink；未來新增 operation 必須擴充契約。HUD 未公開每個技能的超距離政策，因此提示保留伺服器判定，不假設所有技能必定截短。不預測地形落點，不代表 ACK。
- 真實 tooltip 保留「指定地點」，增加不同地面落點、定身／暈眩／沉默限制、跨牆與可站立落點規則。SelfHeal 不繼承位移限制，既有數值及控制政策保持。
- 權威模型在固定點超距離處理完成後比較 destination／origin；原地回 `NoDisplacement`，真 ECS queue 發 `NO_DISPLACEMENT` SettlementRejected receipt。拒絕發生在 effect／cooldown／命令清除前；一 raw 位移仍合法。

## 集中驗收

Rust V2：**1952 通過／10 既有 ignored／0 失敗**，含 generation 共 12 工作成功。

| 套件 | 通過 | ignored |
|---|---:|---:|
| content | 36 | 0 |
| sim | 72 | 0 |
| template | 76 | 0 |
| core | 921 | 0 |
| codegen | 63 | 0 |
| base_content release | 284 | 0 |
| backend | 190 | 1 |
| bridge | 135 | 1 |
| client | 151 | 8 |
| ABI | 15 | 0 |
| restart | 9 | 0 |

新增模型案例涵蓋雙 slot、Reject／ClampToRange、原地與 clamp 後固定點零位移、合法一 raw。真 ECS queue 案例檢查拒絕 receipt、HP／位置／移動／完整命令 queue／雙冷卻保持，隨後一 raw 合法施放會扣正確 slot 並清移動。

四個固定 Lua 契約成功。正式 backend V1 成功；UE5.8 V1／V2 建置成功，最終 UBT **18.58 秒**。七個 staged DLL 與 import library SHA 分別核對。

World V2：**55 成功／0 Failed／0 NotRun**，其中 54 Success、1 SuccessWithWarnings、4 個既有 RuntimePresentationReset 的 Untitled World DestroyActor warning。新增 SummonerCastFeedback 與既有 ResponsiveHudLayout 均成功；其餘地圖、塔／野怪／小兵與角色呈現、施法、HUD、音效資源等原生矩陣保留回歸。

新原生案例使用真 owner／bridge reflection、公開 root／stun／silence snapshot，驗證控制優先序／時間、D／F 換位、任意 catalog、錯誤 owner／generation、失效／未裝備／重複 slot／NaN／負冷卻／非法 range／protocol、退休與清除；同時擷取真正 SOmHudToolTip。它是合成公開 snapshot 接受，不能代替真網路施放或 ACK。

World worker／watcher 退出後目視 **15 張 PNG**。新提示可讀，使用正式 world scale；兩張地圖各 74 影格／6 片段，authority_ack=false。既有分層地圖標籤擁擠、公告 fixture 裁切等限制詳列 errors.md，未冒稱完整 HUD 或 OS DPI 接受。

**3398 來源／資產**前後集合、SHA／size／mtime 完全一致。**115 份證據**封存並另行逐 byte／SHA 校驗；**7 次健康採樣，最大間隔 170 秒**。7 組最終 worker／watcher，以及保留歷史結果的 5 組 worker／watcher 全部退休，monitor 亦已退出。

## 錯誤、證據與剩餘

Rust V1 的舊獨立冷卻 fixture 使用原地施放；改成實際位移並保留原斷言後，V2 全量通過。World V1 的 tooltip 用語回歸保留原文字後以 V2 全量通過，未放寬版面測試。等待中的 source V1 用已記錄 PID／executable／creation token 精確取消；最終使用修正後 V2 基準。所有失敗、取消原因與證據保留；詳見同目錄 `2026-10-22-summoner-feedback-errors.md`，總登錄 E1495。

證據索引：`../acceptance/2026-10-22-summoner-feedback/index.json`。仍缺持續引導／持續區域、定時衝刺／殘留、更多特殊幾何與召喚師內容、道具位移提示、召喚整組 commit 原子回滾、完整 combat replica 生命期接受及更多美術音效。MVP 33/44 與 11 項外部 ACK／LAN／實際 OS DPI／固定效能接受維持原狀。本批零新增真網路對局、quota reset、commit／push；omfx 未維護。
