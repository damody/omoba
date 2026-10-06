# 共用導航只回傳完整可達路徑的第一步

## 計畫與決定

1. 檢查搜尋耗盡時的行為；原planner會用距target最近的visited格重建路徑，即使沒有抵達goal。
2. 封閉但未被占據的終點是反例：外側x64可走，目標x128在四面牆內。接近不等於可達。
3. 用reached明確表示已接到goal或合法same-cell precise terminal edge，只有Some時沿parent回傳第一步；移除best／distance-score副本。
4. 所有模式在搜尋前拒絕target footprint碰撞，避免為occupied target搜尋整個範圍；碰撞半徑、sweep、margin／span与鄰居順序不變。
5. 失敗交原英雄queue.advance或NPC停留，不建立卡住計時器、teleport、第二套Bot導航或無界搜尋。搜尋失敗只是本planner沒有完整路徑，不代表全域不存在路徑。

## 當前功能確認

四項指定core測試各1 passed：

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only path_planner_rejects_partial_route -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only unreachable_route_does_not_walk_partial_path -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only same_cell_public_terrain_commands -- --nocapture
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only npc_static_navigation_detours_and_returns_at_60hz -- --nocapture
```

- planner矩陣：封閉目標本身未碰撞但完整路徑不存在，兩次返回None；occupied target亦None，清障礙後正常Some。
- 60Hz正式PendingMoveQueue／command／movement：MoveTo與AttackMove，各有／無下一筆queued合法MoveTo，共4情境各120ticks。无後續時全程原地、不先向死路靠近；有後續時抵達(0,64)，所有段sweep安全，最後queue／MoveTarget清空。
- E273同格繞障碍的兩種正式命令仍抵達且重播一致。
- NPC原正常繞行與回位60Hz測試仍通過。

## 防錯 E274 與驗收界線

這輪沒有新編譯或查詢失敗。記錄的是原有best-distance fallback的架構問題：不能把partial route當完整可達，也不能為了抵達強制穿牆。

無Lua／生成內容／hash／ABI／wire／IPC／Unreal C++變更，不需UE建置；導航deterministic執行语意改變，最後部署須一致重建Rust權威、replica與bridge，不能混用舊binary。未跑完整三路批次、filtered、PIE或效能驗收；5.4與總進度21/31保持未完成。
