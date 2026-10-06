# 共用導航的同格薄牆繞行

## 計畫與決定

1. 查既有三路Bot與正式導航；已存在公開地形有界BFS，不另建卡住計時器或英雄專屬恢復策略。
2. 發現static_next_waypoint將同格start／goal直接視為直線候選；受薄牆阻擋即None，即使旁邊有路。
3. clear direct保留原快路徑；target本身有碰撞則拒絕。其餘同格受阻沿原margin／span與deterministic neighbors搜尋。
4. 因start key已visited，精確target作獨立terminal edge；找到可安全連到target的非start節點才沿parent回傳第一步。
5. 每段連線仍用共享swept-circle公開地形；不放寬半徑、不傳送、不讀hidden dynamic entity、不增加持久cache或協定欄位。

## 當前功能確認

三項指定core測試各1 passed：

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only same_cell_public_terrain_commands -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only planner_checks_edges_not_only_grid_destinations -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only public_terrain_move_routes_around_thin_wall_at_60hz -- --nocapture
```

- 同格場景：0→30，中間x20–21薄牆，radius1，正式PendingMoveQueue到hero command／movement。
- MoveTo與AttackMove各兩次240ticks／60Hz，逐段sweep零穿牆、確有離開直線繞行、最終距離小於1，每次重播完整位置軌跡一致。
- 原edge測試改為同格可繞行，仍確認格間薄牆、實際target占據拒絕及超跨度受阻拒絕。
- 原格間60Hz實際繞牆測試仍通過。

## 防錯 E273 與限制

沒有新編譯失敗。舊assert把同格拒絕當作防穿牆條件，現在改查合法繞行的每段sweep，不移除地形安全檢查。檔案查詢曾猜錯hero_tick.rs，實際tick路徑由rg --files找出，已記error register。

這是共用導航行為修正，不宣稱任意地圖都有可達路徑、全域navmesh或所有卡住問題已解決。搜尋範圍及既有fallback保持。無Lua內容／ABI／wire／IPC／Unreal C++變更，但deterministic執行語意不同，最後需一致重建Rust權威、replica與bridge，不能拿舊binary混跑。

未做完整三路批次、filtered replay、PIE或效能驗收；依使用者要求留最後。5.4與總進度21/31保持未完成。
