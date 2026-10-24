# 2026-10-22 史詩兵線光環最終驗收

OpenSpec complete-moba-objective-minion-aura **5/5完成**。接續17系統盤點，補單台電腦可驗證的史詩目標與兵線連動。

## 實作

隊伍獎勵新增optional minion_aura：buff_id、radius1..3000、攻擊／護甲bonus0..1000且非全零。舊內容缺欄None，個人Buff不可帶光環，ID必須註冊、不可與英雄獎勵ID混用，同ID政策一致。Lua manifest、生成與canonical hash沿現有內容管線。北史詩保留180秒／死亡遺失英雄Buff，另給900半徑、+15攻擊／+10護甲的攻城祝福。

權威從現有team lease選有效／存活／eligible英雄，以固定點平方距離影響同side存活LaneRole::Creep，包含邊界、排除英雄／塔／野怪／敵人。同類光環不隨英雄數或獎勵層數疊加。以正常BuffStore單instance提供加成，僅在membership／payload改變或instance缺失時寫入；離開、發射者死亡、資格失去、到期／結束清除，pause凍結。新wave於首次戰鬥決策前同步；state保存自己擁有的target generation／ID／加成並納入digest。沒有emitter時不掃兵線。

同時修正實作檢查發現的實際戰鬥接線：MOBA兵線攻擊發射以既有UnitStats聚合基礎攻擊，DamageInstance的Unit／Hero護甲與魔抗同樣聚合既有BuffStore。原有傷害旗標、暴擊／閃避與固定點減傷公式保留。效果沿公開snapshot／ABI33 buff payload與生成Unreal catalog傳遞，沒有新增專用光環美術或宣稱有新圖示。

## 集中驗收

V1於policy tuple的E0382編譯錯誤失敗，三worker與watcher完全退休後才改成借用Option<&Aura>並跑V2；V1所有原始log與receipt封存。Lua三契約V1成功。

RustV2 **1762通過／10既有ignored／0失敗**：template72、core886、codegen57、base262、backend190／1、bridge130／1、client150／8、ABI15。含generation九工作exit0且退休。

新增core三案例覆蓋含邊界範圍、隊伍／兵種、兩英雄重疊、正常攻擊／護甲聚合、離開／返回、HP0／死亡資格、到期／結束、pause、新wave、移除generation與缺失Buff恢復。新增正式60Hz測試經真實北史詩死亡授獎，驗證實際lane AI DamageInstance攻擊+15、真實100物傷依基礎護甲+10只減傷一次、公開snapshot含siege_aura、敵人不受益、匿名傷害後native DeathTick退休並撤回小兵效果。既有完整map／雙隊digest／終局凍結及其他玩法一起回歸。

UE5.8 build-v2成功（UBT **46.03秒**）、正式後端build-v2與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有preview-world清理警告保留。兩張map各74影格／6片段，兩張map與既有首殺資訊列三張原生PNG已目視，屬既有場景／HUD回歸，不當成專用光環畫面接受。

**3356來源／資產**接受前後集合、SHA／size／mtime一致，七項staging與import library另驗。**68份證據**逐byte／SHA封存後獨立複核；三分鐘監測 **8次／最大181秒**。7個最終worker／watcher及3個失敗V1 worker／watcher與全程monitor均退休。

## 剩餘與界線

錯誤A01..A04與通用修正收於2026-10-22-minion-aura-errors.md，總登錄E1472。完整MVP仍33/44，11項外部ACK／LAN／OS DPI／效能待驗維持；沒有新增網路對局、配額重設、commit／push或omfx維護。沒有引用前輪native／IPC／Blueprint／PCM數字作本輪成功。

仍需擴充野怪分裂、經濟補償、塔皮、更多目標公告、技能引導／特殊幾何與施法動畫、召喚複本生命期及美術；完整度盤點仍是功能入口清單，不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-minion-aura/index.json。
