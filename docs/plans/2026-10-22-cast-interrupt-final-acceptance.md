# 2026-10-22 內容驅動施法受傷中斷最終驗收

OpenSpec complete-moba-cast-damage-interruption **5/5完成**。接續全遊戲17系統盤點，本批完成單機可驗證的逐rank受傷中斷施法與正式tooltip。

## 實作

AbilityLevel新增嚴格cast_interrupt_on_damage布林值，缺值false；啟用時要求cast_time在Q10可表示的1/1024..60秒。Rust與固定Lua generator拒絕字串／數字布林及無前搖政策。靜態模板、runtime catalog、script AbilityDef JSON及Unreal generated metadata逐rank保留；content hash包含政策，compiled比較與reload shape要求所有peer重建，未變更ABI34。

admission將政策複製進每次Windup，既有replay digest納入。一般／投射物共用傷害結算與腳本直接傷害在shield之後，只在正HP debit且仍pending／opt-in時沿通用cancel清除，沒有英雄名稱特判。完全護盾吸收、零傷害、治療或未啟用政策不觸發中斷；已release效果不回滾。取消結果沿既有owner進度、公開準備Buff與角色動作退休。

雜賀援軍四rank啟用，其他未設定技能保留預設。Unreal正式Bridge依rank傳typed HUD布林，真Widget tooltip顯示HP傷害取消及完全護盾例外；未知形狀、未學習、零前搖及disabled rank不繼承舊提示。

## 集中驗收

RustV1 **1803通過／10既有ignored／0失敗**：template76、core909、codegen59、base269、backend190／1、bridge134／1、client151／8、ABI15，含generation九工作成功退休。

新增template測試實際呼叫固定Lua，驗證與Rust一致接受／拒絕；省略政策false、逐rankhash、runtime／compiled與reload rebuild守門。core涵蓋預設／opt-in、取消公開marker／結果、release不可回滾與digest。codegen以任意技能驗證true／false rank順序及typed metadata，未加入英雄特判。

正式Production60Hz實際施放generated援軍，比較一般／腳本直接HP傷害、完全shield兩路、部分shield、零傷害及治療七組；中斷不扣魔力或進cooldown，未中斷只產生兩個召喚物並提交cooldown，後續傷害不回滾。既有召喚生成／戰鬥／退休、雙map／雙隊filtered整場生命周期及塔／兵／野怪一起回歸。

UE5.8 build-v2成功（UBT **6.43秒**），正式後端build-v1及Lua三契約成功。WorldV2 **44成功**，0 Failed／NotRun，4既有清理warnings保留；新原生測試驗正式四rank metadata、真Widget提示與未知形狀／未學習／零前搖／disabled負例；既有43項動畫、技能提示、HUD與地圖回歸。兩map各74影格／6片段，六張回歸PNG已目視；未宣稱新增真連線或施法受傷專用截圖。

V1 World43成功／1失敗是新fixture漏設已知Point形狀，受到既有tooltip守門。等待全部consumer／watcher及來源檢查退出後，只修原生測試；V2前後來源基準逐檔比較，唯一變動為OmPointAreaPreviewTests.cpp，保留未受影響的完整RustV1結果。UE／World與來源guard重做V2，原始失敗log／report／receipt封存，不加總兩批數字。

**3372來源／資產**V2接受前後集合、SHA／size／mtime一致；七項DLL與import library另驗。**92份證據**逐byte／SHA封存並獨立複核。全程健康監測 **10次**、最大間隔 **171秒**；7最終及4先前worker／watcher與全程monitor全部退出。

## 剩餘

持續引導／脈衝、特殊線／扇形／多段／導引幾何與提示；召喚完整可見複本生命期、更多控制與競爭落點原子回滾；目標專屬名稱／音效、更多道具／召喚師與平衡資料、英雄／建築／場景美術仍有工作。MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能證據維持待驗，原生44測試成功不代表44個MVP父項完成。

P01..P03／T01見2026-10-22-cast-interrupt-errors.md，總登錄E1481。零新增真網路對局／quota reset／commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-cast-interrupt/index.json。
