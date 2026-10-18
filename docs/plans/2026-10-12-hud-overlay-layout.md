2026-10-12 HUD E975 排版範圍接受：`.ai-collab/hud-overlay-accepted-v1/summary.json`。Codex自行實作，無子代理。將狀態／對局摘要／字幕／截圖提示／診斷改為同一垂直flow，依實際設定鈕寬度與文字換行量測HeaderBottom；一般／item／scroll高度預算共用此邊界，保留既有字級與捲動。ordinary build-v1成功，三native Success零warning；16種800／400、text1／2、左右小地圖、有／無選取的實際paint rect無header/combat/minimap/settings交疊。fresh selection owned-v1真12cases／2TCP／1stage及三cleanup成功，800／400真點擊原圖與四native原圖逐張直接檢視。fresh69Blueprint成功且原packages SHA／mtime不變，21來源hash在gates期間不變；Rust/codegen明示沿用neutral範圍、producer hash及stage再驗。400最大字級長摘要仍需CombatScrollBox；此為排版範圍，不宣稱完整409／OS DPI／世界完成。E978–E979已記MD。下一步依compiled-map-scene-presentation計畫，匯出兩張compiled公開地圖並驗完整renderer場景；完整父項驗收最後，12/44與quota10/10保持。

# HUD 固定資訊與操作區共用排版

Codex 自行實作，無子代理。E975 已由原始400×300圖片確認：選取功能可用，但頂部對局文字與英雄操作區重疊。先等 unit-selection-owned-full-v4 結束與選取範圍證據核對，再修改來源；本計畫不算實作完成。

1. 盤點正式 HUD 的頂部狀態、對局摘要、設定鈕、字幕、聊天／輪盤、英雄／目標／技能道具區及左右小地圖的實際幾何；區分目前顯示與Collapsed，避免為不可見資訊預留整塊空間。
2. 把頂部可見資訊納入共用 composition／高度預算。沿目前 TryPlaceMeasuredClusters、CompositionReservations、ApplyMeasuredItemRow 及有界候選量測排版，維持單一widget、字級下限、tooltip與實際hit guard。需要捲動時使用既有CombatScrollBox，不能移走資訊、固定遮蓋或整組縮小字體掩蓋交疊。
3. 對局摘要使用正式公開資料保持清楚易讀，處理長名稱／最大時間／死亡回城／Unavailable；若簡化可見文案，仍保留必要狀態與可讀提示。不要把驗收或實作細節放入玩家正常流程。
4. 驗真800／400、最大文字與左右小地圖、選取有／無、聊天／字幕／設定／商店、death／retire；實際繪製rect交疊斷言須覆蓋頂部摘要與CombatViewport，不以整張viewport fit=true取代。世界空白仍可點，HUD scroll不動鏡頭。
5. 普通建置、受影響 native／owned實際事件、當輪原圖直接檢視與Blueprint相容核對。錯誤寫MD，完整父項驗收最後；quota10/10保持。接續兩張正式公開地圖呈現計畫，避免把黑色操作fixture當完整世界。

已查入口：`OmCommandBarWidget.cpp` 的 SetPresentationMatchLine、TryPlaceMeasuredClusters、RefreshHudLayout／Fits／CombatViewport高度上限；`OmResponsiveHudTests.cpp` 與 `OmOwnedViewportIpcTests.cpp`。目前 Fits 高度只扣Margin與BottomInset；TryPlaceMeasuredClusters 在 CompositionReservations空時提早返回，頂部對局條尚未進入同一預算。設定鈕另有條件式水平避讓，不能只補一個固定40像素常數。
