# ise_hime（B23）UE 呈現檢查

| 招式 | cue 播放（依美術） | 目標 cue | 狀態 cue | 判定 |
|---|---|---|---|---|
| `b23_undead_pact` | style 1 | 0（預期 0 個） | b23d_damage_reduction：ring 1 | 有呈現 |
| `b23_spirit_gathering` | style 1 | 0（預期 0 個） | — | 有呈現 |
| `b23_living_soul_shock` | style 1 | 0（預期 0 個） | b23e_shock：ring 1 | 有呈現 |
| `b23_soul_feast` | style 1 | 0（預期 0 個） | b23r_magic_immune：ring 1 | 有呈現 |
| `b23_heroic_spirit_blessing` | style 1 | 0（預期 0 個） | b23t_invulnerable：ring 1；b23t_attack_speed：0 | 有呈現 |

擊退拋物線（全局）：0 次

- 實機：`target/lineC-pres14/ise_hime-run`（spotlight，rerun20，2026-11-01，props 舞台）。observer 有檢查。build：snapshot commit `a0d43837`（E583，一輪建一次）。
- 亡靈之契（E630）：props 舞台在示範者 200 內放一具屍體，這招第一次有呈現。rerun19 實際用的是 base 舞台（排隊腳本沒有選 props recipe），所以被 CAST_REJECTED。
- 頭像、技能圖示、載入圖由 HUD sync 依 id 接上。
