# 公開非英雄名牌避讓：最終驗收

完成 complete-nameplate-screen-spacing。沿用每視角最多64候選與選取／游標／建築／中央距離的穩定順序，新增實際 Slate 尺寸與目前 mesh 錨點的有界矩形避讓；重疊的低優先名牌暫時隱藏，移開後可恢復。名牌不移離單位、英雄顯示規則與地面關係標記不變。文字縮放／狀態資訊會自然改變量測足跡。最多64次量測與固定容量幾何工作，不對10000實體建立全體兩兩碰撞。

實際地圖的非英雄名牌堆疊減少；英雄彼此及英雄／非英雄交叉仍可能存在，後續優先評估保留英雄血條的共用障礙保留策略。本批不宣稱所有名牌碰撞均已解決。

初次／隱藏量測與正式 UMG 顯示共用純 Slate layout factory，避免短暫 SObjectWidget 釋放根節點。ReleaseSlateResources 清除量測根，下次需求可重建。UI比例查詢留在負責UMG的OmRuntime模組。保留所有視野、陣營、生命與世代檢查。

## 驗收

- Rust **1966通過／10既有ignored／0失敗**，12工作含內容生成；四項Lua契約、後端及UE5.8正式建置成功。
- World **58成功／0 Failed／0 NotRun**，包含新增NameplateScreenSpacing與既有HealthNameplateBudget；4個既有RuntimePresentationReset場景銷毀warning。
- 密集碰撞、padding邊界、縮放、非法資料、固定64上限、穩定優先順序、首次量測、隱藏／資源釋放後恢復及真Slate繪製均驗證；既有合成地圖保留兩隊full-health可見與關閉偏好時隱藏的斷言。
- **3406來源／資產**集合、SHA、size、mtime前後一致，7 staged DLL與import library SHA通過；先前已通過避讓測試的正式輸入另與最終比較，只允許測試fixture差異。
- 封存 **164份證據**並獨立逐byte／SHA校驗；World退出後目視 **18張PNG**，含密集／恢復與兩地圖。圖像是單機原生fixture，非真網路ACK或實際OS DPI接受；既有first-kill局部HUD裁切與objective窄strip限制仍記錄。
- 健康採樣 **10次，最大171秒**；全部最終與歷史worker／watcher及本批health程序均退休。

## 錯誤與接續

第一輪建置跨模組UMG連結失敗；第一次原生測試暴露臨時SObjectWidget生命期；修正後既有地圖fixture仍需在手動插值後刷新視角判定並使用分離的焦點名牌。未放寬正式避讓規則或原斷言，完整失敗、精確來源檢查取消與後續重跑證據保留。詳見2026-10-23-nameplate-spacing-errors.md，總登錄E1498。

48小時無人值守heartbeat **omoba-48** 仍接續本對話，期限 **2026-10-25 00:33:38台北時間**。後續先讀handoff／盤點／錯誤與程序receipt，避免重複工作。持續引導／區域、定時位移與特殊幾何、召喚整組commit、完整combat replica生命期與更多內容／美術音效仍pending；MVP33/44、11項外部ACK／LAN／實際DPI／固定效能接受不變。未新增真網路對局、quota reset、commit/push或omfx維護。

證據索引：`../acceptance/2026-10-23-nameplate-spacing/index.json`。
