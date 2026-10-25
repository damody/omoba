# 自有召喚物移動權威層：完整驗收

新增獨立 OwnedSummonOrder 正式輸入，指定主人英雄與召喚物的精確 entity/generation，封閉為 MoveTo（必須有點）與 ReleaseToAutonomy（禁止帶點）。每玩家每tick最多32項，使用既有順序輸入邊界；不借用英雄MoveTo，也不清除主人的指令、施法或引導。

接收時檢查正式Playing階段、player roster、當前英雄生命／所有權、召喚關係與壽命、召喚物生命、可選PlayerOwner一致性、同隊同陣營、座標界限。移動另外檢查控制、移速與既有地形導航；拒絕前不改命令。解除可在定身時接受，移除目的地與導航快取，由原有跟隨／戰鬥恢復。重用既有SummonTick、MoveTarget、navigation與攻擊中斷，不新增另一套移動時計或名稱特判。

本批是權威層基礎。UE選取仍供檢視，renderer converter、secure coordinate ingress、canonical network acceptance 明確拒絕新命令；尚未實作玩家可用的召喚物命令UI與scoped actor transport。沒有把canonical entity ID暴露為前端能力，沒有聲稱玩家已可操作召喚物。

## 驗收

- Core2：protobuf往返、封閉操作／點存在性／世代／座標；合法兩種命令都不能誤轉為英雄renderer intent。
- Backend1：合法MoveTo與Release在未協商scoped transport的入口均被拒絕。
- Formal2：實際Mago W召喚物、正式60Hz移動、定身拒絕但解除接受、解除後原有跟隨恢復。混合fixture將主人slot3指定既有apprentice_mend，確認召喚物命令不干擾真實引導的MP、位置、queue與四次治療；這是權威隔離fixture，不是修改Mago作者R。paired replay相符。另含錯誤owner／世代／非召喚物／陣營／召喚關係／死亡／過期／暫停／Finished／格式／障礙目的地與33項洪水界限。
- 八個正式維護Rust套件 fresh1990 passed、0 failed、9個既有ignored；focused5另列，不重複加入套件總數。
- 正式backend/moba-config、UE5.8、DLL與import library staging SHA、既有optional channel配裝離線plan-lock與catalog一致性通過；沒有新增真網路對局。
- 全量3481來源／資產建置後fingerprint一致；相對sealed bot-channel-intent，3470 unchanged、10 existing changed、1新權威module，無刪除。V1編譯失敗原始證據、V2失敗、V3與最後驗收固定輸入、health／原始API reap／owner-waiter退役／cleanup均封存，sealer後另獨立rehash。
- 沒有新的前端呈現變更或fresh native畫面宣稱；既有畫面證據沿父index保留。

錯誤與防重犯：docs/plans/2026-10-24-summon-orders-errors.md。證據：docs/acceptance/2026-10-24-summon-orders/index.json。下一來源基準 omfue/Saved/Automation/SummonOrdersV1/source-before.json，固定inputs.json與summon-orders-parent-v2.json。

接續具體缺口是UE自有召喚物選取／命令提示與精確scoped actor transport，之後再處理移動投射物／多段技能／內容／美術／平衡。MVP33/44與11外部接受仍pending。沒有quota重置、commit/push、omfx維護或終止無關程序。無人值守期限2026-10-26 23:59:59 Asia/Taipei。

獨立封存檔案數 179。
