# H02 伊達政宗：驗收

status: pending

kit 狀態：`supported`。四招的必要能力都已存在：
- 追蹤投射物：G-1。
- 衝刺：`swept_dash`。
- 圓形暈眩：`point_blast`。
- 自身變身：`self_transform`。

`catalog_ready` 維持 `false`，等實機驗收通過後才改。

| 技能 | L1（Rust handler 測試） | L3（實機） | 備註 |
|---|---|---|---|
| `b04_flame_trail_dash` | 見 work-H.md | 未執行 | 每個 rank 的距離、觸發範圍、火焰傷害 |
| `b04_fire_blast` | 見 work-H.md | 未執行 | 每個 rank 的傷害與暈眩、半徑邊界 |
| `b04_flame_swiftness` | 見 work-H.md | 未執行 | 每個 rank 的攻速與護甲 |
| `b04_tanegashima` | 見 work-H.md | 未執行 | 每個 rank 的持續時間、射程、攻擊與暈眩機率 |

實機驗收由主 session 執行：`build --execute` 加一局對局。通過後：
1. 把第一行改成 `status: accepted`。
2. 把 `capabilities.lua` 的 `catalog_ready` 改為 `true`。
