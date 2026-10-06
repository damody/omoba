# 兵線 Bot 通用導航（2026-10-06）

## 問題與決策

接續OpenSpec 5.5。Jungle巡邏已檢查完整路徑，但其他位置的Advance直接提交或保留AttackMove；公開waypoint被地形封住時，Bot可能卡在失敗命令。

- 抽出reachable_travel_destination共用有界候選迭代，跳過50單位抵達範圍、最多8次共用正式static_next_waypoint查詢，使用本人半徑及公開BlockedRegions。
- reachable_lane_destination從preferred開始依前進順序選第一個完整路徑，不環狀回頭；reachable_patrol_destination維持公開營地ring，兩者共用查詢預算與抵達規則。
- 全失敗提交正式HoldPosition清舊命令，已Hold且無queued不重送；地形恢復下次think重新判斷，不使用永久黑名單、私有敵方狀態或傳送。
- 查詢只在最終Advance分支，不搶既有戰鬥、塔前Hold、護送、自保優先；兵線可達時保留原bend及已接受目的地，不每think重送命令。
- 不新增Lua runtime、角色C++、Blueprint或omfx維護；ABI及生成hash不變。

## 局部確認

- 新正式60Hz案例1通過，內含Top／Mid／Carry／Support四組：第一個bend、跳封住點、保留可達命令、全失敗Hold、Hold去重、地形恢復及真正下一tick正常移動而非傳送。
- core `lane_navigation`純函式2通過，涵蓋前進順序／不回頭、恢復重選、8次budget、抵達跳過、空／非法route。
- 相鄰正式60Hz `role_bot_lane_route_60hz`與`jungle_patrol_navigation_60hz`各1通過；本增量共core2＋base3 tests，不將filtered out計為已跑。
- 完整100場、LAN、PIE與release stage留最後，本輪debug test build不代表DLL已部署。

## 防錯

- 本輪未重複E294的私有routes／缺HeroCommandQueue假設，fixture讀公開編譯地圖waypoints，命令由正式PlayerInput產生。
- 既有template build script三個dead_code warnings保持，不改無關TD程式；測試只報實際執行數，不計filtered out。
- 見E295；整體21/31，完整5.5未勾。
