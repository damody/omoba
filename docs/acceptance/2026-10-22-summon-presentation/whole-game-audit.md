# 2026-10-22 召喚物公開身分與原生呈現盤點

本輪接續 whole-game completeness inventory，以單機可驗證的跨層缺口為範圍。地圖左下／右上、兵線護主、野怪回頭、塔與技能提示既有接受繼續回歸；沒有新增真實連線對局。

## 本輪根因與通用實作

原生 snapshot 未將 UnitType::Summon 分類為獨立 Summon；公開 WaveB 又把召喚物歸入一般 creep，因此相同 ID 的 summon／creep catalog 無法可靠區分。這會讓正式召喚技能產生有戰鬥能力但前端外觀與動畫錯誤的單位。

現在 full／fast snapshot 與公開 render.kind=4 使用明確召喚物身分。filtered shadow 只重建 UnitType 與公開呈現，不加入私有 SummonedUnit／AI／導航。Bridge 使用既有 ABI34 EntitySummon／CatalogSummon，納入通用角色動作投影；未知內容維持召喚類別與 generic fallback，不靠名字或 owner 猜身分。既有 25-byte render 與 protobuf／CABI 結構大小保持。

saika_gunner 的原生內容使用既有 ranger rig／材質／動畫，獨立複製視覺設定並指定65世界單位高度。Generated class 仍是 summon，自有 ID／400HP／傷害與技能規則保留；不是新專屬槍械或角色美術。

## 接受設計

- 核心：完整與 fast snapshot、公開 WaveB 身分一致；無私有行為洩漏；相同ID一般單位不可變召喚物；無 Unit storage 的精簡世界繼續可用。
- Bridge：真正公開 snapshot 經 projection 到 CABI；同名不同 catalog；移動、隱藏／重現、退休／generation 更新、未知內容。
- 正式60Hz：技能建立兩個 owned summon、提交公開可見性後實際開火、攻擊呈現、pause、到期退休及 generation 精確匹配。
- UE：實際 Bridge 建立 generated summon、名稱與原生 rig、站立／行走／攻擊／死亡、pause、隱藏／重現、移除／新 generation；實際 GPU PNG 捕捉。
- 集中回歸包含 map、塔、野怪、小兵、技能提示與HUD既有案例；來源指紋、DLL／import library SHA與owned worker退休一併檢查。

## 未完成範圍

召喚物公開呈現生命期並不等於完整 combat replica 接受；退休來源 on-hit callback、競爭落點整組原子回滾、更多召喚控制、持續引導／特殊技能幾何仍需後續實作。更多英雄／道具／場景美術與目標音效也保留。MVP仍33/44，11項外部ACK／LAN／實際DPI／固定效能證據不能用 synthetic World替代。

驗收最終結果與證據以同日 summon-presentation-final-acceptance.md 為準。失敗與修復保存在 summon-presentation-errors.md 及原始失敗批次，不能只留成功重跑。
