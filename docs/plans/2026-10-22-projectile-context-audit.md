# 全遊戲盤點：彈體來源上下文

本批接續handoff的單機優先玩法缺口，落實退休來源命中回呼，非重複既有HUD複核。範圍只含正式omoba-core runtime與離線authority測試；唯一Unreal前端保留全量建置／單機回歸，omfx不維護。

## 實作

- 每發ProjectileOrigin保存發射時ScriptUnitTag、位置／朝向、攻擊／生命／單位／陣營、塔升級旗標／層級／選敵政策、building及該entity的buff。塔附近單位快取不複製；不複製全世界BuffStore。
- 不可變快照採Arc共享，child／追加命中不深拷貝來源buff；來源當前alive／資源生命期並未偽造。
- 正式projectile tick命中事件攜帶每發origin，不依賴命中時仍存在的tag。dispatch只在精確source生命退休後啟用唯讀快照；存活來源保留即時查詢，is_alive不偽造。
- Script ProjectileQuery及GameWorld的來源相關唯讀屬性使用相同來源邊界；生命／攻擊／護甲／移速／射程／buff／upgrade／身分與陣營可供退休來源回呼查詢。
- 回呼追加Damage與ScriptProjectile帶入內部來源封套；processor只接受來源一致的三種outcome，拒絕巢狀封套與錯誤來源，再走原正式processor／observable facts。追加Damage沿既有ProjectileImpactOrigin歸因，子彈沿既有ProjectileOrigins生命期與generation規則。
- 快照中的buff時間是該發射上下文，並未重新套用或延長全局buff；亦不把退休來源復活。不改ABI／protobuf／CABI／public snapshot。
- 召喚物已承諾彈體的來源死亡／stun不取消後續回呼索敵；普通自動索敵保持source admission，所有目標仍沿原可見性／可攻擊性／LOS守門。

## 驗證範圍

新增核心測試走真生成、飛行、命中、派送及提交；涵蓋退休來源read-only資料、近距離友軍排除、同ID新generation隔離、追加傷害、下一代子彈與清理、存活來源即時修改、錯誤來源／巢狀封套拒絕，以及真MOBA敵方英雄被回呼追加傷害擊殺後的獎勵歸屬及一次性。既有內容腳本、backend、bridge、client、native World與兩張map保留集中回歸。

## 尚待完善

持續引導／特殊技能幾何、完整combat replica接受、召喚落點整組commit原子回滾、更多原創英雄／道具／召喚師、美術音效仍有缺口。原MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能未變，不新增真對局或重設配額。此批不宣稱全遊戲已與LOL一致。

最終接受以2026-10-22-projectile-context-final-acceptance.md及本批封存index.json為準；錯誤保留於2026-10-22-projectile-context-errors.md。
