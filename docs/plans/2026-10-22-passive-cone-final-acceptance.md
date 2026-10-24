# 普攻被動與受擊點扇形最終驗收

2026-10-22 批次，於 2026-10-23 臺北時間跨日完成。OpenSpec `complete-passive-cone-combat`：5/5 完成。

## 實際修正

雨鐵砲原本雖描述受擊點扇形，卻從攻擊者位置測量候選角度；原生普攻投射物命中也沒有接上既有的被動 hook。本批修正兩者，讓真正普攻命中後，以受擊點為起點、沿攻擊者至受擊點方向，在 90°／150 半徑內對每個存活敵人結算一次既有比例真實傷害。

- `omoba-sim::sector` 提供可共用的有界扇形判定。半徑／dot／cross 使用 i128；角度沿用引擎 4096 tick、Q10 sin/cos 邊界，弧度依共用 TAU 常數取最近 tick。包含起點、角度／半徑邊界，支援 ray／半平面／大角度／全圓，非法維度失敗關閉。這不是無量化的實數角度宣稱。
- 共用 PassiveCone resolver 校驗 rank、半徑、角度、比例及攻擊／傷害預算，無 unchecked rank indexing。先完成預檢與精確篩選，再 canonical 排序去重並發傷害；排除友軍、死亡／HP0、來源本身與扇形外候選。零方向不發效果。保留原四 rank 百分比／半徑／角度資料與 Pure／TRUE profile。
- 普攻來源以 authority-only `normal_attack` 明確標記；只有真正且正傷害的那發原生普攻具有資格，純視覺連發／miss 沒有。技能子彈即使使用 NORMAL damage profile，或沿用 credit／source snapshot，也會清除此資格，避免被動連鎖重複觸發。
- Hero 可在沒有 UnitScript tag component 時保存最多 64 筆排序過的有效技能 rank；活來源讀當前值，精確舊世代退休後使用發射時不可變來源資料。被動 type／max rank 由 registry 校驗，依 ID 固定順序派發。保留既有 projectile credit 與死亡後命中語義，不復活來源或讀取替代世代。
- Rain 的主動 execute 明確 RErr；on_learn 校驗 rank 並使用 canonical 永久生命期。Unreal 從明確 metadata 發布 `bPublishedPassive`，被動 tooltip 顯示「無需主動施放」，替換為 active state 時恢復一般提示。沒有新增瞄準扇形 overlay 或假造權威落點／ACK。

無 script ABI、public C ABI 或 protobuf 變更；不依英雄名稱或特效半徑判定命中。

## 集中驗收

最終 Rust V2：**1962 通過／10 既有 ignored／0 失敗**，含內容生成共 12 工作成功。

| 套件 | 通過 | ignored |
|---|---:|---:|
| content | 36 | 0 |
| sim | 75 | 0 |
| template | 76 | 0 |
| core | 921 | 0 |
| codegen | 63 | 0 |
| base_content release | 291 | 0 |
| backend | 190 | 1 |
| bridge | 135 | 1 |
| client | 151 | 8 |
| ABI | 15 | 0 |
| restart | 9 | 0 |

新增 3 個幾何、5 個真 adapter／來源與 2 個正式 60Hz 案例。涵蓋起點、前後方、角度／半徑相切、一 raw 外、旋轉與極端輸入；四 rank 資料、非法 rank／budget／零方向、被動主動拒絕及永久 marker；真普攻三發視覺連發只有一發具資格、退休來源與同 ID 新世代隔離、技能子彈不繼承資格。

60Hz 案例經真正原生普攻投射物生成、飛行、命中、被動派發與傷害結算，驗證四 rank 的主要目標額外傷害、前向／邊界命中、後方／角外不命中、只結算一次、pause 凍結飛行與命中、發射後暈眩保留被動；另驗主動施放拒絕不扣資源。這是離線 authority ECS 的真結算，不是新網路對局或正式輸入 ACK。

四個固定 Lua 契約成功。正式 backend V1 與 UE5.8 build V1 成功，UBT **70.67 秒**；七 staged DLL 與 import library SHA 分別驗證。

World V1：**56 成功／0 Failed／0 NotRun**，其中 55 Success、1 SuccessWithWarnings；4 個既有 RuntimePresentationReset／Untitled DestroyActor warning。新增 PassiveConeMetadata 驗證真正 generated passive type／描述、四 rank 零 point-circle／swept path、非位移、實際 tooltip／active 提示退休與替換。既有地圖、NPC、角色、施放及 HUD 矩陣保留回歸。

World worker／watcher 退出後目視 **16 張 PNG**，新被動 tooltip 可讀且沒有主動按鍵指示。兩地圖各 74 影格／6 片段、authority_ack=false；既有地圖標籤擁擠與局部 fixture 裁切限制詳列 errors.md，未當作全 HUD／真連線／實際 OS DPI 接受。

最終 **3402 來源／資產**集合、SHA、size、mtime 前後一致。另比較第一輪建置／World 與最終來源，只有 Rust 測試佇列清理那個檔案不同，其他正式輸入 SHA／size 一致；seal 與 independent verify 都重新檢查此條件。

**133 份證據**封存並另行逐 byte／SHA 校驗，包含完整最終紀錄、第一輪失敗、focused probe、來源等價證明與執行腳本。健康採樣 **13 次、最大間隔 171 秒**；7 組最終、4 組歷史／probe worker／watcher 與 health monitor 全數退休。

## 錯誤與剩餘

Rust V1 唯一失敗是測試假設閒置 Hero 一定有命令佇列，結果 290 通過／1 失敗；改成有佇列才清理，沒有放寬任何傷害／pause／stun 斷言。Focused probe 2/2、完整 Rust V2 均通過。失敗、自然拒絕的 source V1 與所有原始證據完整保留，見 `2026-10-22-passive-cone-errors.md`，總登錄 E1496。

證據索引：`../acceptance/2026-10-22-passive-cone/index.json`。

仍缺持續引導／持續區域、定時衝刺／殘留、更多主動特殊幾何與完整預覽、變身技能資料的共用嚴格校驗、召喚整組 commit 原子回滾、完整 combat replica 生命期接受及更多英雄／道具／召喚師／美術音效。MVP 33/44 與 11 項外部 ACK／LAN／實際 OS DPI／固定效能接受維持原狀。零新增真網路對局、quota reset、commit／push；omfx 未維護。
