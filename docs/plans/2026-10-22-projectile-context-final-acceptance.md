# 2026-10-22 彈體來源上下文最終驗收

OpenSpec complete-projectile-source-context 5/5完成。本批增加正式戰鬥完整度：已發射彈體不再因來源entity退休而跳過腳本命中效果；追加傷害與子彈保持原始MOBA來源歸因。

## 實作與邊界

ProjectileOrigin保存不可變、可共享的每發來源快照，包含腳本身分與唯讀戰鬥資料；不複製整個世界或塔的空間快取。dispatch在精確來源生命退休後才回退查詢，存活來源仍用即時數值，is_alive不偽造。影響的是已承諾彈體的命中回呼，不允許死亡來源重新主動施法／行動。

正式tick把命中與來源一併交給串行腳本派送。追加Damage及ScriptProjectile只在精確來源一致時帶入內部封套，拒絕巢狀／來源不一致的封套，再走原有傷害、observable facts及生成流程。子彈保留generation、origin及清理生命期。退休召喚物回呼的目標查詢保留目標可見性／可攻擊性／LOS守門，已承諾彈體不再被來源退休擋掉。

來源快照buff時間只供該發射上下文使用，不重新套用或延長全局buff。未新增公開敵方私有資料、ABI、protobuf或CABI欄位；沒有英雄／塔名稱特判。

## 集中驗收

最終RustV3：**1825通過／10既有ignored／0失敗**，含template76、core919、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9，以及generation十工作全部成功。五個新增核心案例包含真飛行／退休與ID重用／read-only資料／追加傷害／子彈繼承及清理、存活來源即時修改、非法封套拒絕、MOBA擊殺獎勵歸因與一次性、退休召喚物目標守門。四Lua契約成功。

正式backend-v3與UE5.8 build-v3成功，UBT **3.49秒**。WorldV3 **51成功／0 Failed／0 NotRun**，共5個warnings（4個既有測試World清理警告，1個UE自動連線探測HTTP逾時）。兩張地圖各74影格／6片段；十張現有功能PNG均在World worker／watcher退出後目視。本批沒有新增Unreal UI功能；圖片與World用於回歸，不拿synthetic viewport當網路ACK。

**3388來源／資產**接受前後集合、SHA／size／mtime一致；七DLL及import library另驗。封存**114份證據**並獨立逐byte／SHA校驗，含先前失敗preflight與過渡版本receipt。健康監測**15次**、最大間隔**171秒**，最終7組worker／watcher及所有過渡工作均退休，全程monitor退出。

## 錯誤與剩餘

錯誤見2026-10-22-projectile-context-errors.md，總登錄E1490。路徑猜測、測試pattern欄位錯位、fixture缺資源、preflight缺exit標記均保留紀錄；未覆寫失敗證據。召喚物來源退休邊界在最終接受前修正，不放寬自動AI或目標守門。

持續引導／特殊幾何、完整combat replica接受、召喚整組commit原子回滾、特殊英雄命中機制、更多英雄／道具／召喚師、美術音效仍待完善。MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱整個遊戲已等同LOL。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-projectile-context/index.json。
