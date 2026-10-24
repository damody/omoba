# 2026-10-22 共用召喚落點預約最終驗收

OpenSpec complete-moba-summon-reservation-ledger **5/5完成**。接續全遊戲盤點，補上單機權威可驗證的跨event／on_tick召喚位置承諾。

## 實作

原先每個adapter只檢查ECS與自己的pending outcomes，不同技能事件或平行hook canonical重放仍可能向腳本承諾重疊位置，commit才移開。新增World範圍PlacementReservations，以完整Entity含generation記錄team、位置與半徑，Arc／Mutex與BTreeMap保持受控共享與固定排序。只在既有canonical serial allocation建立預約，speculative平行hook不分配；同team共享，敵隊預約不影響己方。每World4096、每invocation128上限。

finish將預約交給outcomes；discard_cast／Drop立即撤銷自己的預約並刪除保留handle。正式summon生成時退休對應預約，沿既有owner／地形／友軍體積檢查提交；新cache清除已退休generation。StateInitializer正式World安裝共享資源，精簡測試World保留cache內fallback。無新增協定／ABI／Lua內容格式變更。

## 集中驗收

RustV1 **1809通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15；含內容generation九工作全部成功退出。

新增跨cache同team位置分離、取消後可重用位置、Drop撤銷、敵隊隔離、退休generation清除及4096容量拒絕。既有8單位canonical平行hook測試新增兩兩至少40世界單位間距，真的process_outcomes後每個實際Pos與腳本取得的位置完全一致，ledger清空，並保留returned handle後續facing命令。正式60Hz召喚生成／戰鬥／退休、雙map生命周期及塔／兵／野怪一起回歸。

UE5.8 build-v1成功（UBT **3.56秒**）、正式後端build-v1與Lua三契約成功。既有World **45成功**、0 Failed／NotRun，4既有清理warnings保留。地圖、召喚物動畫、技能提示、HUD與資源生命期回歸；兩map各74影格／6片段，7張PNG已目視。本輪新增功能的直接證據是權威ECS實際settlement測試，World與PNG是消費端回歸，沒有冒稱新增專用畫面或真網路對局。

**3373來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**72份證據**逐byte／SHA封存並獨立複核。健康監測 **6次**、最大間隔 **171秒**；7組worker／watcher與全程monitor全部退出。編譯／功能接受沒有失敗重跑；路徑盤點及Saved helper識別字問題已於啟動World前修正並記錄。

## 剩餘

任意中間outcome改變世界／退休owner後的整組施法commit原子回滾仍需額外交易設計，本次共用預約不冒稱該項完成。退休來源callback需完整發射時上下文，不以script ID猜測；完整combat replica、持續引導／特殊技能幾何、更多原創英雄道具召喚師、美術與音效仍待擴充。MVP33/44、11項外部ACK／LAN／實際DPI／固定效能條件維持pending。

錯誤與決策見2026-10-22-summon-ledger-errors.md，總登錄E1483。零新增真網路對局／quota reset／commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-summon-ledger/index.json。
