# 路線可達性與 autopilot 卡住處理（E070 後續，E072）

## 地圖驗證
- `omoba-template-ids/src/lua_content.rs`：lane 路段與 terrain 的 swept-circle（半徑 100）碰撞時，build 會失敗。錯誤訊息包含 map id、lane、segment index 與兩端座標。
- `omoba-core/src/runtime/moba_map_layout.rs::every_compiled_lane_route_is_reachable_by_a_hero`：
  - 對所有編譯後的 MOBA 地圖（three_lane_training、layered、quick）的每個路線點做 `path_hits_regions` 檢查，每個相鄰路段做 `static_next_waypoint` 檢查，英雄半徑取 30。
  - 包含負向測試：刻意在路線點上放牆，必須偵測得到。
- 結果：三張地圖全部通過，地圖資料不需要修改。

## 根因
E070 被拒的 (845.7,3351.1) 不是路線點，而是 autopilot `FarthestVisibleToward` 沿英雄到 waypoint 直線取出的畫面可見截點。這條直線穿過了 quick map 的 `jungle_wall_3_0`。

## autopilot（OmAutoPilot.h/.cpp）
- `ClearOfWalls`：點擊點落在牆內（含半徑 40 的膨脹）時，往英雄方向每次退 10 單位，直到離開牆。牆的資料快取自 frame 的 `terrain_rects`。
- `FProgressWatch`：以下任一情況判定為卡住：收到 `move_route` 被拒的 ack；或 3 秒內送出 2 次以上 move、但位移小於 40。
- 卡住時：
  - 若不是撤退，就跳到下一個路線點。
  - 用 `DetourPoint` 取 350 單位、±45/90/135° 的繞行點，每次左右交替。
  - 寫入 trace `{"event":"stuck_recover","reason":"rejected|no_progress",...}`，並計入 coverage `stuck_recover`。
- 測試：`Om.Runtime.AutoPilotStuckRecover`，涵蓋判定時機、拒絕信號、split38 實際座標的牆外修正，以及繞行點位於牆外且左右交替。這部分還沒有在 UE 編譯，等下一次 build 驗證。
- 沒有修改 `hero_command_tick.rs`，只以唯讀方式使用 `static_next_waypoint`。
