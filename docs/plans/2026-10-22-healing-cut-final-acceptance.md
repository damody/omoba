# 2026-10-22 抑療與共用治療結算最終驗收

OpenSpec complete-moba-healing-reduction **5/5完成**。接續17系統盤點，補玩家可實際使用的抑療與原有HealReceivedMultiplier接線。

## 實作

UnitStats共用received_healing，native Heal、ScriptHeal、正常正值HP regen採同一受療倍率／DisableHealing政策。raw與legacy因子限0..4，依source／family canonical排序、家族強者聚合、i128中間值與checked轉換；非法／溢位政策不產生治療，總增益封頂4倍。正負治療分明，寬位元HP加總與上限，不允許一般治療把HP0復活；Unit fallback也不再經render float轉換。

召喚師治療將當前權威倍率放入SummonerCastContext與不可變plan identity，在missing-HP cap前套用；提交重驗倍率，變更舊計畫拒絕，不在sink重複套用。完全禁療仍可按正常規則提交0治療與冷卻。泉水與營地完成hard reset為還原，泉水維持原規則，camp reset明確還原HP，避免抑療使重設營地殘血。

UnitHex新增optional healing_reduction={multiplier,duration}，內容檢查倍率0..小於1並拒絕量化為1、期限1/1024..60秒／非法型別／未知欄位，規則進content hash。舊內容省略維持None。native執行前重驗，沿既有敵隊／可見／距離／generation／owner與cooldown交易守門；拒絕不扣冷卻、不寫Buff，傷害後只對仍存活target加抑療。

咒擊符原ID11、價格450、55傷害、450距離／8秒冷卻保留，新增50%受療倍率持續4秒；此為原創訓練平衡。公共healing_reduction native Buff保留最強倍率與最長有效剩餘期限，同類來源不重複相乘；到期／死亡清除，沿既有公開Buff snapshot／ABI33與owned list顯示。generated Unreal metadata加入獨立倍率／期限，原生tooltip依本地化數字格式顯示50%抑療與4秒期限，非法資訊不啟用hotbar。沒有CABI layout變更。

## 集中驗收

V1因新增camp測試缺BuffStore import而core編譯失敗；全批退休後修正，並補發現的召喚師直接HP入口。V2 core897通過、舊assert期望致死後治療復活而1失敗；base新召喚師fixture缺protobuf型別匯入。全批退休後將舊assert明確改為0並保留存活target治療對照，protobuf改完整型別，完成V3。原始失敗與先前成功紀錄全封存，不加總多批數字。V3 Rust全通過後，WorldV1新tooltip斷言誤用整數文字，依既有本地化一位小數格式修正C++測試；Rust來源未再修改，保留V3 Rust／後端，完成UE V4／WorldV2及新來源比對。

最終RustV3 **1782通過／10既有ignored／0失敗**：template74、core898、codegen58、base265、backend190／1、bridge132／1、client150／8、ABI15；含generation九工作exit0且退休。

新增內容測試驗證optional／界線／量化／型別／hash／legacy，codegen驗證獨立typed metadata。新增core涵蓋native／script／legacy Unit、raw倍率／家族／canonical／溢位／regen恰一次／強弱refresh／到期／負治療／滿血／死亡。擴充既有camp reset及召喚師模型／實際ECS案例，驗證倍率在cap前、倍率變更舊plan拒絕、0治療冷卻。

新增正式60Hz案例實際買咒擊符、超距拒絕且CD0、合法ItemUse與generated70治療變35、native與script20各變10、公開Buff raw512／有限期限、pause凍結、實際SummonerCast200先減半再補50缺血到550、到期70治療恢復、死亡清Buff。既有整場雙隊filtered lifecycle、兵線／塔／野怪／技能等一併回歸。

UE5.8 build-v4成功（UBT **5.38秒**），正式後端v3與Lua三契約成功。WorldV2 **40成功**（39 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有測試警告保留。新增HealingReductionItemMetadata確認generated ID11道具仍可unit targeting、55傷害、50%／4秒說明與非法metadata拒絕。兩map各74影格／6片段，兩map與既有首殺／塔皮／護盾五張原生PNG均目視，屬回歸，不宣稱新增抑療專用實際連線畫面。

**3364來源／資產**接受前後集合、SHA／size／mtime一致；七項staging及import library另驗。**105份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **17次／最大181秒**。7個最終worker／watcher、10個先前worker／watcher與全程monitor均退出。

## 剩餘與界線

H01..H11與修正收於2026-10-22-healing-cut-errors.md，總登錄E1477。仍需全隊經濟補償、更多目標公告、持續引導／特殊技能幾何與動畫、召喚複本生命期與建築／場景美術。原MVP33/44與11項外部ACK／LAN／OS DPI／效能缺證維持；沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-healing-cut/index.json。
