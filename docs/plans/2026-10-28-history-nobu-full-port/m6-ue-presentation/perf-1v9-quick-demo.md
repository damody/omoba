# UE frame time：1 人＋9 bot quick demo（cue、狀態 cue、目標 cue 全開）

- 局：`target/lineC-perf/demo8`（2026-10-31，`run_moba_split_local --recipe moba_single_player_quick.lua --autopilot lane_v0 --performance-lock`，1920x1080，600 秒）。B 的 E374 之後，觀察端不一致 0 次，整局跑完。
- 分析：`tools\lua\lua.exe scripts\ue_frame_windows.lua --log target/lineC-perf/demo8/p1/ue-p1.log`（每秒一個 OM_PERF game_frame_interval 視窗，對照同一秒的 OM_CUE／OM_STATUS_CUE／OM_CAST_TARGET_CUES／OM_BUFF 與 OM_PERF_SPIKE）。
- 量測期間機器上還有其他 session 的建置在跑，數字偏保守。

## 結果

- 熱身 30 秒後 274 個視窗：平均 frame interval 中位數 16.9 ms、p95 23.5 ms（約 60 fps）。
- 團戰最忙的一秒（t=246，39 個呈現事件）平均 17.0 ms、最大 20.9 ms：**cue 數量本身沒有造成掉幀**，不需要 cue 上限、池化或 LOD。
- 但有 17 個視窗出現 ≥400 ms 的卡頓（引擎把 delta 截在 0.4 秒），另有 23 個 50～400 ms。卡頓的那一幀 presentation 工作幾乎都集中在兩個階段：
  - `ability_events`：1800 ms、500 ms、460 ms，都是某個 cue **第一次**播放；log 同時出現 `FlushAsyncLoading` 與 `Compiling System NiagaraSystem /Game/Omoba/VFX/CueShapes/NS_shape_burst`。
  - `entities`：開局快照 794 ms、之後 120～235 ms，都是某種單位**第一次**生成（模型、材質、動畫同步載入）。
  - 整局 287 次 `FlushAsyncLoading`。

## 處理（E389）

- 第一階段（omfue 4e995da）：`OmPresentationPreload` 在 bridge 的 BeginPlay 非同步預載所有 cue 可能播放的資產（Editor cue overrides 的效果與音效、產生的技能 VFX 層、隱身與變形外觀），持有到比賽結束。
- 第二階段（omfue c5c7e90）：非英雄單位類別與載入名單上本局英雄的類別，從預設物件以反射收集所有 soft 引用（模型、材質、動畫），一起預載。新英雄不需要改 UE。

## 修正後（demo9，同樣設定，2026-10-31）

`target/lineC-perf/demo9`：預載 242 個資產（cue 8、單位 234），開局 14.8 秒內完成，觀察端不一致 0 次。

| | demo8（修正前） | demo9（修正後） |
|---|---|---|
| 熱身後的視窗 | 274 | 491 |
| 平均 frame interval 中位數／p95 | 16.9／23.5 ms | 16.8／21.3 ms |
| 每秒最大值的 p95 | 400 ms | 71.2 ms |
| ≥400 ms 的秒數 | 17 | 3 |
| 100～400 ms 的秒數 | 15 | 14 |
| presentation 工作 >100 ms 的幀 | 8 | 2（都是開局第一個快照，在載入畫面期間、預載完成之前） |
| 比賽中 FlushAsyncLoading | 287 | 12 |
| Niagara 系統即時編譯 | 3 | 1 |

剩下的 100～400 ms 卡頓，當下 presentation 的工作都只有 0～3 ms，時間不在 omfue 的 actor tick 裡（量測期間機器上同時有其他 session 的建置與實機局）。cue、狀態 cue、目標 cue 的數量與 frame time 沒有關聯，不需要 cue 上限、池化或 LOD。
