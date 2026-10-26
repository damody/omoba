# H01 雜賀孫市：驗收

status: pending

kit 狀態是 `supported`：G-2 `charges` 與 G-3 `summon_immobile` 已由其他 session 完成（work-G2.md，L1）。handler 與實機驗收完成前 `catalog_ready = false`，不進正式選角目錄。

| 技能 | 狀態 | L1 | L3 | 備註 |
|---|---|---|---|---|
| `b01_sniper_mode` | supported | 未執行 | 未執行 | E3 薄切片的第一招候選 |
| `b01_saika_corps` | supported | 未執行 | 未執行 | 需 handler 使用 host charges |
| `b01_rain_iron_cannon` | supported | 未執行 | 未執行 | |
| `b01_three_stage` | supported | 未執行 | 未執行 | |

每招的情境列在 `abilities.lua` 的 `verification.scenarios`。全部通過後，把本檔第一行改成 `status: accepted`，並把 `capabilities.lua` 的 `catalog_ready` 改為 `true`。validator 會要求兩者一致。
