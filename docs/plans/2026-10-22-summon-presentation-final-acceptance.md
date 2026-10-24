# 2026-10-22 召喚物公開身分與原生呈現最終驗收

OpenSpec complete-moba-summon-presentation **5/5完成**。接續全遊戲盤點，完成單機可驗證的召喚物種類、typed catalog、公開呈現與原生動畫生命期。

## 實作

full／fast snapshot 明確辨識 UnitType::Summon；公開WaveB用render.kind=4、filtered只重建公開UnitType與內容身分，不加入私有SummonedUnit／AI。Bridge沿既有ABI34 EntitySummon／CatalogSummon，解決同名小兵／召喚物目錄混用，未知內容仍保留召喚物種類並走fallback。25-byte render、protobuf及CABI結構大小不變。無Unit storage的精簡World仍可擷取快照。

召喚物加入通用移動、腳本attack_phase_fx、生命值與動畫投影。saika_gunner使用既有ranger rig／材質／動畫，隔離視覺設定、高度65世界單位，生成原生summon class；保留原ID／400HP／技能與傷害。這是可見原生角色接線，專屬槍械與角色美術仍待製作。

## 集中驗收

RustV5 **1807通過／10既有ignored／0失敗**：template76、core910、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15；含generation九工作全部成功退出。

新增核心測試驗full／fast與WaveB公開身分一致、不洩漏私有腳本／AI、同ID一般單位不誤分類。Bridge真正公開snapshot經通用projection到CABI，驗typed catalog、移動、隱藏／重現、退休／新generation與未知內容。codegen驗實際內容rig、身高、idle／move／attack／death與正確class。正式Production60Hz技能生成兩個owned summon、提交視野後實際開火與公開attack cue、pause及到期退休；其餘塔／兵／野怪／雙map生命周期一起回歸。

UE5.8 build-v2成功（UBT **5.55秒**），正式後端build-v1及Lua三契約成功。WorldV2 **45成功**、0 Failed／NotRun，4既有清理warnings保留。新增測試由真Bridge生成實際summon actor，驗名牌、已載入rig與精確動畫clip、站立／移動／攻擊／pause／隱藏／重現／死亡／移除／新generation。既有44項技能提示、角色、HUD與地圖一起回歸。兩map各74影格／6片段；7張實際PNG已目視，包含新召喚物原生GPU截圖。未新增真網路對局，也未冒稱完整可見戰鬥複本接受。

RustV1漏顯式import，V2精簡World儲存區／fixture資源與公開視野更新，V3新測試誤讀MobaMatch專用動畫欄；WorldV1抽象基底生成造成空指標；V4修復工具防覆寫保護導致未建立runner。皆記錄根因、等待消費者退出再修復，保留失敗receipt與原始logs，不加總失敗與成功數字。最終完整RustV5與WorldV2重新接受。

**3373來源／資產**最終接受前後集合、SHA／size／mtime一致；七項DLL與import library另驗。**159份證據**逐byte／SHA封存並獨立複核。健康監測 **14次**、最大間隔 **171秒**，7最終與7先前worker／watcher、全程monitor全部退出。

## 剩餘

完整combat replica生命周期、退休來源on-hit callback、競爭落點整組原子回滾、更多召喚控制、持續引導／脈衝與特殊技能幾何仍待實作。更多英雄／道具／召喚師、目標專屬名稱音效、建築／場景美術與平衡內容保留。MVP仍33/44，11個外部ACK／LAN／實際OS DPI／固定效能接受不以本機World代替。

錯誤詳見2026-10-22-summon-presentation-errors.md，總登錄E1482。零新增真網路對局／quota reset／commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-summon-presentation/index.json。
