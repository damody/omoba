# 護障召喚師技能：最終驗收

完成add-summoner-self-shield。正式Lua增加catalog3 `training_self_shield_v1`護障：160護盾、2秒、180秒冷卻，舊blink1／heal2及預設D/F不變。共用作者enum／嚴格驗證、generated fixed-point operation、immutable plan與ECS sink接入現有D/F冷卻交易。None target、正有限可量化amount、duration<=60校驗；拒絕不寫Buff／冷卻，成功一次後重送拒絕。Missing BuffStore拒絕，其他既有spell仍可使用其原路徑。

使用既有召喚師配裝／選角流程，將D或F其中一格設為`training_self_shield_v1`；不自動取代原配裝。此目錄為build-time生成，修改作者數值後須重建；未宣稱新增召喚師runtime hot reload支援。

護盾沿共用damage_shield吸收／公開／到期生命期，不另造局部護盾。不累加，保留較高護盾量與較長剩餘時間；可在控制狀態使用，不取消移動，禁療不縮減護盾。正式Unreal新增共用generated OmSummonerCatalog，依catalog與spell identity相符才投影DisplayName／Description，實際D/F tooltip顯示作者說明，沒有技能名稱特判或C ABI變更。

## 集中驗收

- 本批Rust **1572成功／零失敗／8既有ignored**：content40、template79、codegen68、core930、base_content300、bridge-summoner4、正式client-runtime151。實際generation成功；不是所有workspace全面重跑。
- 新驗證包含作者非法／缺字段／unknown字段與target shape、generated identity與escaping、計畫sink失敗及重送、禁療獨立、真正ECS控制／冷卻／生命／owner／pause／phase／missing resource／wrong target拒絕、較高shield與較長lifetime重授、吸收及到期。
- 正式60Hz PlayerInput測試確認新spell接受、public buff payload163840raw、pause凍結、冷卻重送不刷新、170點直接傷害由160護盾吸收後HP只扣10，以及後續到期。全base_content既有bounded三seed×24000tick三路filtered lifecycle與其他離線回放完成；無新網路對局。
- 六項Lua契約／選角套件成功，包含新spell ID handoff；正式backend及UE5.8建置成功。**World65成功／零Failed／NotRun**，新增SummonerSelfShield實際tooltip與既有召喚師／護盾／技能／地圖回歸通過；4個既有RuntimePresentationReset warning保留。
- **3420來源／資產**集合、SHA、size、mtime一致；7 staged DLL及import library一致。**168份證據／25PNG**獨立逐byte／SHA校驗，20張與前批已審閱像素精確相同，五張差異／新圖已目視。新圖是實際共用tooltip，不冒稱新spell硬體按鍵或真網路接受。
- 9個最終worker、3個歷史worker及watcher退休，兩段health timer均退休；合計採樣10次、跨階段最大間隔171秒。V1 core929成功／1失敗是重授時限誤認2而非既有5；保留原始world／content／client-hud來源和V1全部日誌，reader退休後修測試／作者說明並補client enum consumer。V2全core930及新60Hz測試成功，但全base_content誤用debug；對齊前批release配置前，以ready Cargo與parent chain／精確native creation token只退休本輪debug測試，保留所有不完整日誌，不當作通過。V3使用release重跑全base_content並完成整個Rust範圍；sourceV1因前置cohort未完成而退休，sourceV2在同一凍結3420輸入接受，正式來源未再改動。監看V2動態列出本批各版本實際日誌。

## 範圍與接續

專用護障圖示／美術仍待補；當前使用既有文字／卡片及通用護盾呈現。fixture無player controller的主要／次要未綁定行，不代表實際遊戲預設D鍵缺失。既有first-kill局部下方HUD裁切、objective窄strip、英雄／焦點名牌重疊仍有限制。一般技能channels／zones、整組召喚資源原子回滾、完整combat replica生命期、更多原創內容及11項外部接受仍pending，MVP33/44不變。

錯誤／預防詳見2026-10-23-summoner-shield-errors.md與E1507。48h omoba-48沿原2026-10-25 00:33:38 Taipei期限持續。未新增真網路對局、quota reset、commit/push或omfx工作。證據：`../acceptance/2026-10-23-summoner-shield/index.json`。
