# 全遊戲盤點：沿實際路徑的碰撞衝刺

接續最新彈體來源上下文交接。本輪盤點發現fire_dash直接set_pos、不受內容射程／碰撞限制且只查終點圓形；與內容的沿路傷害不符。正式地圖左下／右上、兵線／野怪／塔既有接線保留回歸；本批增加可單機驗證的實際技能玩法。

## 實作

- omoba-sim terrain新增有界i128精確point-in-capsule，包含相切與起終端；零長度與非法座標／半徑安全處理。
- base_content共用SweptDash以內容range／width／damage／profile執行，不依英雄名稱判斷幾何。range先clamp，沿既有advance_with_collision查詢公開地形／root政策；受阻時二分最遠合法直線prefix，禁止擅自軸向滑動或穿薄牆。
- 實際路徑包圍圓取得候選，再以中心點capsule過濾、按entity id／generation排序去重；活著且HP正值的敵人各命中一次。在查詢／驗證全完成前不產生任何位置／朝向／傷害outcome。
- fire_dash依逐rank內容接線；嚴格拒絕非法JSON、rank、數值與target。完全無法移動時透過既有RErr取消mana／CD／成功cue。四rank範圍900／1000／1100／1200，寬150，傷害250／300／350／400；不再用量化0.1秒的102/1024作除數。
- metadata使用既有Custom selector表達沿路命中；Lua描述如實說明瞬間、地形停止與每目標一次，不再稱終點圓形／持續傷害。不改ABI／protobuf／CABI，也不改其他已有DashToPoint的全有或全無政策。

## 測試與範圍

新增3個精確geometry案例（相切、端點、單raw外側、斜線／平移、零長度、極值）及7個base案例；含真handler／正式60Hz input→dispatch→outcome、四rank射程clamp／魔力／冷卻、薄牆裁切、友軍／側外／遠牆後排除、非法資料與root／zero／封閉回滾、後續tick無重複傷害、沿路metadata與面板實傷一致。

此批明確為瞬間位移及單次路徑傷害，尚未實作按0.5秒持續移動／火焰殘留；持續引導、特殊技能幾何完整預覽、完整combat replica接受、召喚commit整組原子回滾、更多英雄／道具／召喚師與美術音效仍有缺口。原MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能接受不變；不新增真對局、不重設配額、不維護omfx。

集中結果以本批final-acceptance及封存index為準。錯誤記錄於2026-10-22-dash-path-errors.md。
