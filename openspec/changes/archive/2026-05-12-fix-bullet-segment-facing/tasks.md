## 1. Projectile 視覺狀態

- [x] 1.1 檢查 `omfx/game/src/native.rs` 中 projectile 渲染資料流，確認 `projectile_spawn_pos` 與 `ClientProjectile` 是否需要新增固定方向欄位。
- [x] 1.2 在第一次看到 projectile 時記錄穩定的初始視覺方向，優先使用發射點到目標或 projectile 初始有效位置的向量。
- [x] 1.3 為初始方向長度過短的情境加入安全 fallback，避免 NaN rotation 或無效 transform。

## 2. 線段幾何更新

- [x] 2.1 將 projectile 拖尾線段的 `rotation` 改為使用已鎖定的初始方向，而不是每幀用 `pos - spawn_pos` 重算。
- [x] 2.2 保留既有拖尾長度上限、最小長度、顏色、z ordering 與 batch slot 更新行為。
- [x] 2.3 確認 projectile 清理時同步移除新增的方向狀態，避免 stale state 影響重用 entity id。

## 3. 驗證

- [x] 3.1 新增或調整純計算測試，覆蓋水平、垂直、斜向與初始零長度 fallback 的 rotation 行為。
- [x] 3.2 新增或調整測試，確認 projectile 位置更新後線段 `rotation` 維持初始值。
- [x] 3.3 執行相關 `omfx` 或 workspace 測試；若無法執行完整測試，記錄實際可執行的驗證指令與結果。

## 4. 手動確認

- [x] 4.1 使用 `run.bat` 或可行的前端啟動流程，在一般塔射擊案例確認子彈線段從發射點面向敵人。
- [x] 4.2 在目標移動或多 projectile 場景確認線段飛行期間不會旋轉。
