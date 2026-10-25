# 自有召喚物安全傳輸：完整驗收

在上一批精確召喚物移動權威層之上，新增 client-only SummonCommandInput（tag30），只攜帶主人與召喚物的 opaque replica ID／view epoch／disclosure epoch，以及封閉的MoveTo或Release與有界點。原有canonical OwnedSummonOrder（tag29）仍不能由公開入口直接提交。

secure edge要求目前協商版本、joined player、authenticated player/team、current view與非零input ID；在指定輸入tick的已公開資料中解析兩個同隊參照，再產生精確entity/generation的正式order。解析失敗沒有部分改寫。正式權威層仍核對真實玩家、roster、主人生命、召喚關係、陣營、壽命、控制與地形，不能把「同隊可見」當成「該玩家擁有」。客戶端host提供從當前own hero與own kind4 summon資料建構命令的API，拒絕別人的召喚物、小兵、不存在的ID及無效點形狀。

Selective protocol提升13→14，舊peer拒絕協商；presentation保持11，因為本批沒有新增renderer intent。Canonical accepted kind29以owner為actor、summon為target，清除payload內全部canonical index/generation。Projector與replica拒絕漏出的身分、錯誤action kind alias、client form冒充ACK與缺失的actor/target。過濾Specs副本不持有私有SummonedUnit；此post-step ACK只作通知，確認owned Hero與owned Summon後直接結束，不重新下達英雄或召喚物移動。

## 集中驗收

- Core4：protobuf與封閉形狀；雙參照的原子解析、錯誤team/view/tick/disclosure／未知ID／非法generation；ACK遮蔽、漏身分與kind alias拒絕；真實TeamViewProjector→SelectiveReplicaRuntime→Specs stepper兩tick套用，明確Applied、拒絕偽造alias、完整canonical hash一致且未重複移動。此公開元件fixture明列materialization會補上的Hero/Facing/TurnSpeed/CollisionRadius基準。
- Client1：從實際projector產生的disclosed snapshot啟動ReplicaHost，成功建構自有召喚物MoveTo／Release，拒絕其他玩家、其他類型、未知ID與壞形狀。
- Backend2：協商commands能力及joined player正確才接受合法opaque形狀；錯誤玩家、未啟用能力、壞形狀拒絕。最後集中驗收另外實際重跑raw canonical MoveTo／Release入口拒絕回歸。
- Formal1：實際Mago W產生的召喚物，opaque refs→shared secure seal→正式60Hz authority移動；未seal的client形狀不能進trusted driver。同隊可見但PlayerOwner改為別人時，sealed Release仍被真正權威拒絕；paired replay一致。這與公開ACK fixture分開驗證，不宣稱建立真網路對局或拍攝玩家操作畫面。
- 八個維護Rust套件 fresh1996 passed、0 failed、9個既有ignored；上述focused8不重複加入總數。正式backend/moba-config、UE5.8、DLL/import library staging SHA、離線optional channel配裝plan-lock／catalog一致性通過。
- 全量3482來源／資產建置後指紋一致；相對sealed summon-orders，3468 unchanged、13 existing changed、1新shared transport module，無刪除。所有失敗probe、最終固定輸入、health、原始API reap、owner/waiter退役與cleanup，以及sealer後獨立rehash封存。

失敗與預防保留於 docs/plans/2026-10-24-summon-wire-errors.md：lower formal exhaustive match明確拒絕client-only action；整合fixture必須核對Applied與精確advertised start tick；公開元件基準必須包含會materialize的元件；跨crate使用既有public encoder。保留嚴格斷言，沒有放寬產品驗證。

證據：docs/acceptance/2026-10-24-summon-wire/index.json。下一來源基準 omfue/Saved/Automation/SummonWireV1/source-before.json、固定inputs.json、parent summon-wire-parent-v2.json。

UE召喚物選取／施令UI及bridge issuing仍pending；renderer converter仍明確不支援新client action。沒有fresh native畫面宣稱。下一批接typed UE命令與可見施放／移動提示，再處理更多projectile／多段技能／內容／美術／平衡。MVP33/44與11外部接受不變。沒有真網路對局、quota重置、commit/push、omfx維護或終止無關程序。期限2026-10-26 23:59:59 Asia/Taipei。

獨立封存檔案數 294。

補充修復範圍驗收：closure中的production-repair-proof.json逐一核對6個probe，shared transport的測試區之前產品程式完全相同，client除該constructor fixture以外的內容完全相同。所有原始API已reap；最後查無本批owned worker。
