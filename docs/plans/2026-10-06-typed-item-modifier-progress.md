# 主動物品限時效果的型別化邊界

## 計畫與決定

1. 上輪 Bot 使用 item_sprint／item_damage_reduce 名稱前綴辨識狀態。這會讓 planner 相依於 executor 私有來源命名，不能作為長期共用 API。
2. 新增 host-local `ItemTimedModifier::{Sprint, DamageReduction}` 與 BuffStore grant／has 方法。效果種類統一定義 family、stat、有效量與方向，writer／reader 都沿此邊界；名稱格式只留在 BuffStore 內。
3. grant 在變更前檢查正量、Sprint≤10000、減傷≤1、時間0–60秒且不含零。保持原來源刷新、strongest-family、raw Fixed64、反向 stat index 與事件行為。
4. has 依有效 family 與對應有界 stat 判斷，不看來源名稱或其他 Buff 混合後的淨值。沿共用 raw／legacy numeric reader，避免查詢與真正聚合使用兩套解析語意。
5. entry 的有效期限由既有 tick 移除管理，查詢不自行提前移除 pending-zero entry，保持同一 lifecycle；對 owner 查詢，不新增前端披露或 ABI。
6. 正式物品執行器與 Bot 已接新 API；未啟用 active_use 的 Bot 仍不建立新增觀測。

## 當前確認

- core 一項矩陣測試通過：兩種類型、無變更拒絕、owner隔離、易傷抵消淨減傷仍可辨識、到期／移除、名字相似但無效果、任意來源及舊數值格式。
- 正式生成主動商品60Hz買用一項／五種商品通過，不跑整場／全套。

```text
cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only item_timed_modifier_identity -- --nocapture
cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only generated_active_shop_items -- --nocapture
```

## 防錯紀錄 E269

這輪沒有新編譯失敗。修正的是架構問題：Bot不能把內部名稱前綴當作效果存在契約；查詢不能因另一效果抵消總值就誤判失效，也不能忽略原聚合支援的舊浮點格式。不要另建第二份狀態快取或重複計時器。

沒有變更 Lua runtime、生成商品資料、Unreal C++、ABI／wire／IPC；不需重跑 UE 建置。完整5.5與21/31保持，最後統一部署與完整驗收。
