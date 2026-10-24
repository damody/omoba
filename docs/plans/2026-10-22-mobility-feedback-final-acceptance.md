# 2026-10-22 位移施放提示最終驗收

OpenSpec complete-mobility-cast-feedback 5/5完成。新增通用位移技能的定身／狀態未知／原地落點提示，普通技能保持定身下可施放。

## 實作

omoba-content-model的ability_requires_movement由typed SweptPath或executable DashToPoint導出位移需求；不看英雄名、技能ID、動畫或dash_width鍵名。codegen產生bRequiresMovement，bridge只在有效公開rank／point／正值range時送入既有HUD payload，owner在已學習point保留policy，替換／未知形狀／ClearHeroSnapshot／ResetRuntimeInputScope清除。無ABI／protobuf／CABI變更。

bridge對完整公開owner source buff snapshot建立獨立root channel：1 clear、2 rooted、0 unknown，永久時間-1；使用精確entity generation、owner、披露、存活與source relation守門，保留4096列預算，不受UI64展示截斷。root取同類最晚解除時間；非法duration／instance／重複root保持未知，未知buff metadata保守撤銷兩種channel。普通stun／silence channel不因root-only壞資料而宣稱不能施法。snapshot／來源撤銷／退休同步清兩種channel。

owner兩種查詢共用嚴格reflection簽章與回傳校驗；不掃描actors或讀取隱藏資料。預覽保留unlearned／cooldown／mana／stun／silence優先，再呈現定身、永久定身、未知位移控制與零地面距離。原地檢查使用公開位置XY與引擎epsilon，不把Z差當可位移，也不宣稱預測固定點量化邊界。普通point技能可選自身位置。tooltip補位移技能root／不同落點需求。

提示保持唯讀、瞄準可留待控制解除，不自行取消／拒絕／發送命令，不分配input id、不推定ACK或自行倒數。後端既有固定點碰撞、root拒絕及mana／CD回滾維持；位移召喚師／道具尚未因本批自動接入這套ability policy。

## 集中驗收

RustV1 **1940通過／10既有ignored／0失敗**；content34、sim72、template76、core919、codegen62、base277、backend190／1、bridge135／1、client151／8、ABI15、restart9，含generation共12工作全部成功。新增model來源分類與arbitrary-ID generator測試；正式60Hz衝刺、薄牆、root／zero／封閉回滾、原技能與地圖生命期全量回歸。四Lua契約成功。

正式backend-v1與UE5.8 build-v2成功，UBT **7.13秒**；七DLL與import library SHA另驗。WorldV2 **53成功／0 Failed／0 NotRun**，warnings共4個，分類在本批errors及engine log。新增MobilityCastFeedback以真bridge／owner reflection驗證公開root／最晚時間／永久／UI64外／非法值／duplicate／未知metadata／世代／死亡／撤銷／移除／退休、普通cast隔離、feedback優先序／原地XY、生成policy與tooltip替換／退休。

新圖擷取真SOmHudToolTip的定身剩餘時間與實際技能操作需求；其餘11圖作既有地圖／HUD／角色／沿路技能說明回歸。共12張PNG在World worker／watcher退休後目視；不當作真對局capsule錄影或ACK。兩張地圖各74影格／6片段。**3394來源／資產**前後集合、SHA／size／mtime一致；封存**105份證據**並獨立逐byte／SHA校驗。健康採樣**7次**、最大間隔**171秒**；7組最終worker／watcher、4組V1過渡工作及monitor已全部退出。

## 錯誤與剩餘

WorldV1為52成功／1失敗，tooltip fixture缺MaxLevel被正式HUD校驗拒絕；已從真metadata補齊，不放寬校驗，保留engine／report／PNG／receipt與來源快照，重建後WorldV2全量接受。另於執行前自查修正批次名替換誤改舊測試ID，未漏跑或放寬測試；詳見2026-10-22-mobility-feedback-errors.md，總登錄E1493。正式驗收實際失敗／警告皆以保留log為準。

持續引導、定時衝刺／殘留、更多特殊幾何、位移召喚師／道具提示、完整combat replica接受、召喚commit整組原子回滾、更多英雄／道具／召喚師、美術與音效仍待完善。MVP33/44及11項外部ACK／LAN／實際OS DPI／固定效能接受不變。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-mobility-feedback/index.json。
