# 2026-10-22 全遊戲盤點：選取目標戰鬥資訊

沿全遊戲盤點，已具地圖對角基地、塔／兵／護主、野怪返回／分裂、技能提示、控制名牌等規則與接線。本批補選取目標資訊卡：原先只有名稱、HP、kind／team／relation，無法在固定位置讀護盾、控制、塔皮與返回狀態。

## 通用實作

FOmSelectedUnitStatePayload新增typed Shield／CrowdControl／HealthSegments／JungleReturnMode，僅從同一公開frame既有有界摘要讀取。沿既有source player、target完整generation、living／visibility與runtime scope守門；可選取不可受傷的公開目標，不把inspect當attack admission。不新增私人屬性／敵方技能冷卻查詢，不改後端、內容catalog、CABI34或protobuf。

資訊卡與名牌共用OmShieldLabel、OmCrowdControlLabel、OmHealthSegmentsLabel、OmJungleReturnLabel，維持既有名牌文字與bounded格式。非法子狀態獨立省略，不污染HP主卡。HP不變但任一狀態或顯示秒數改變即發布HUD；同顯示秒數維持節流，但Bridge selected getter保存最新raw公開值，不能讓文字節流造成查詢時間停留舊值。HUD顯示snapshot只需維持相同文字，不宣稱逐fractional秒更新。

既有取消選取、死亡、隱藏、換generation、source conflict、runtime／HUD退休清理整張卡；沒有新增全場cache或per-entity Slate更新。UOmPlayerHudOwnerComponent維持原有typed source／epoch／player驗證並傳遞完整payload。

## 驗收與剩餘

新增真公開frame→Bridge selection→實際native HUD測試，驗四項資料、共用格式、malformed子狀態、HP固定時單獨改盾／控制／塔皮／返回、同秒raw snapshot、清單缺席／取消／世代／source／隱藏／死亡與HUD退休。新增實際OmSelectedUnitCard Slate截圖，保留既有控制／返回名牌、七張消費端圖與兩map manifest。

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際DPI／固定效能接受維持pending。本機synthetic呈現不替代網路ACK。

最終數字見2026-10-22-selected-combat-final-acceptance.md與封存索引。零新增真網路對局、quota reset、commit／push，omfx未維護。
