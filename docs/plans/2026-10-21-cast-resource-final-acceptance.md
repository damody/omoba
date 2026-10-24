# 技能魔力提示與野怪牆體出手：最終驗收

OpenSpec `add-moba-cast-resource-guidance` **5/5完成**；全程未新增權威對局。

## 實作

技能目前已學rank的generated ManaCosts接到公開HUD。合法零與未知分開；tooltip顯示小數cost，瞄準時依目前owned alive hero的有效mana／maxmana提示「魔力不足」。無魔力英雄、未學習／非法／未知數值不推論不足；角色替換、source移除及runtime退休清除舊cost。優先顯示未學習／冷卻狀態。提示只改文字／顏色，不攔截submit，也不是權威ACK；此版適用QWER，不猜測道具與召喚師消耗。

野怪主體與附屬怪入射程後共用正式VisionOccluderSet的精確LOS，被牆擋住不產生damage／攻擊sequence／草叢揭露、不消耗攻擊週期，繼續既有static navigation追擊。視線與距離允許後立即出手，成功攻擊仍有原冷卻。回頭、patience與獎勵ledger不改。正式map boot使用compiled occluders；舊fixture無牆resource仍保留無牆行為。

## 完整驗收

| 項目 | 結果 |
|---|---|
| RustV2 | **1654通過／10既有ignored**：template59、core813、codegen57、base246、backend190/1、bridge127/1、client147/8、ABI15；9個工作含生成均exit0並退休 |
| Lua | local69／owned140與map ID契約成功 |
| UE5.8／backend | 凍結後build-v2成功，UBT6.70秒；backend-v1成功；接受前staging與import library SHA核對 |
| WorldV1 | 30項：29 Success＋1 SuccessWithWarnings／4 warnings，0 Failed／NotRun；兩map各74影格／6片段 |
| 完整local-v1 | **209 native**：201 Success＋8 SuccessWithWarnings／12 warnings，0 Failed／NotRun |
| Owned輸入 | **458案例／222 intents／157 stages／13 cursor**完整phased coverage成功 |
| Blueprint／音訊 | **69 Blueprint**、0 compiler errors／0 warnings／0 failed loads；**五段實際submix PCM**成功 |
| 來源／封存 | **3340來源／資產**接受前後集合、SHA、size、mtime一致；derived DLL與import library另依current build／接受前staging核對；**20份原始JSON**逐byte封存並獨立複核 |
| 三分鐘監測 | 完整流程8次健康取樣，180秒排程UTC最大180秒，另持續讀取最新log；**11個owned工作與11個watcher全部退休** |

## 畫面與測試界線

已目視兩map的overview／HUD、camp與fog原圖，左下／右上基地、三路、牆、河流與連通草叢仍正常。已目視 `resource/ability-resource-tooltip.png`：實際SOmHudToolTip元件以正式BuildAbilityTooltip結果繪製，魔力消耗37.5、距離600、落點半徑220與一般施法說明可讀。這是owned測試視窗的實際元件render、使用fixture公開數值，不冒稱真對局hover截圖。魔力不足與rank／free／invalid／retirement由native測試驗證；野怪隔牆／附屬怪／預算／恢復出手由正式AI ECS測試驗證。

phased接受不等於單session效能或真對局ACK。原MVP仍**33/44，11項外部ACK／LAN／實際OS DPI／效能接受待驗**；未新增對局、重設配額、commit／push或維護omfx。仍可續做特殊script技能形狀／metadata、史詩怪增益／分裂、任意多邊形authoring與專用美術。

RustV1 fixture失敗、build-v1 C4456與舊source-before-v1均保留，最終使用RustV2／build-v2／WorldV1／local-v1／source-v2。錯誤見[同日紀錄](2026-10-21-cast-resource-errors.md) M01–M05；證據見[封存index](../acceptance/2026-10-21-cast-resource/index.json)。OpenSpec CLI不可用，未宣稱CLI validate。
