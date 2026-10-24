# 2026-10-22 沿路碰撞衝刺最終驗收

OpenSpec complete-swept-dash-combat 5/5完成。本批修正fire_dash直接傳送、只傷害終點的玩法缺口，增加可重用的固定點沿路衝刺與單次命中機制。

## 實作

omoba-sim提供有界i128精確capsule判定；包含相切、端點、零長度、斜線、大座標及非法值守門。base_content共用SweptDash先校驗內容預算及來源，依rank射程clamp；透過既有advance_with_collision檢查公開地形／root政策，受阻時最多25次查詢找同一直線最遠可行prefix，避免穿薄牆或軸向滑動。只對實際路徑的中心點capsule內活敵人canonical排序去重，各命中一次；所有預檢完成後才提交位置／朝向與傷害。

fire_dash使用range／dash_width／damage_per_tick／dash_duration逐rank內容，範圍900／1000／1100／1200、寬150、傷害250／300／350／400。使用固定點per_tick * duration * 10換算，面板與實傷一致；移除102/1024近似0.1作除數的偏差。非法metadata／rank／target、定身、零位移或完全封閉透過既有RErr施法交易退回，不扣mana／CD，不發布成功施放cue。

metadata改用既有Custom selector表達沿路，Lua與原始碼描述如實說明瞬間、遇地形停止、每目標一次。沒有英雄名稱特判；其他既有純DashToPoint的全段可行政策保留。本批並未實作0.5秒逐tick位移、殘留火焰、持續引導或新的路徑瞄準圖形；不改ABI／protobuf／CABI。

## 集中驗收

最終RustV2 **1904通過／10既有ignored／0失敗**。本輪將sim72全量納入，含新增3個geometry案例；base277含新增7個共用執行器／真handler／正式60Hz案例。其他為template76、core919、codegen60、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation共11工作全部成功。

正式60Hz案例涵蓋四rank射程clamp與魔力／冷卻、實際中段／終點／側外／遠端／友軍、薄牆裁切與碰撞半徑、root／zero／封閉回滾、後續30 tick無重複傷害。非法資料／極值及沿路metadata另驗。測試以既有mana_regen_percentage=-1024 fixture隔離自然回魔，正式回魔規則未修改。

四Lua契約成功；正式backend-v2與UE5.8 build-v2成功，UBT **7.94秒**。WorldV2 **51成功／0 Failed／0 NotRun**，共4個warnings，已逐項分類於錯誤MD及engine log。兩張地圖各74影格／6片段；10張既有功能PNG已在World worker／watcher退休後目視，作為Unreal消費端回歸，並非新增技能路徑預覽或真網路ACK證據。

**3389來源／資產**前後集合、SHA／size／mtime一致；七DLL及import library另驗。**87份證據**逐byte／SHA封存及獨立校驗，含V1失敗資料。健康監測**14次**、最大間隔**171秒**；最終7組worker／watcher、V1過渡工作及全程monitor均退出。

## 錯誤與剩餘

錯誤記錄2026-10-22-dash-path-errors.md，總登錄E1491。V1為276通過／1失敗，唯一失敗是fixture忽略同tick自然回魔85 raw；修正測試隔離後嚴格檢查扣魔一次，未放寬正式玩法或覆寫失敗證據。API／路徑猜測錯誤亦已記錄。

尚缺持續引導、按時間位移／殘留效果、特殊技能幾何完整預覽、完整combat replica接受、召喚commit整組原子回滾、更多英雄／道具／召喚師、美術音效。原MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能接受不變；未宣稱全遊戲等同LOL。零新增真網路對局、quota reset、commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-dash-path/index.json。
