# work-creep-facing：小兵、野怪在 UE 裡的面向錯誤

```text
日期：2026-10-25
目前工作 ID／狀態：creep facing／已修正並著陸（2026-10-25 21:21 著陸）；Rust 測試通過，UE Editor 測試已撰寫但尚未編譯或執行
玩家或作者新增能做到的事：分路小兵、野怪的正面跟著移動方向，攻擊時轉向目標；兩隊 replica 看到的面向完全相同。所有動畫單位（英雄、小兵、野怪、召喚物）共用同一條規則：可見正面＝移動或攻擊方向。
repo HEAD／本次修改：未 commit；見「修改檔案」
已執行命令、exit code、build identity：見「測試」
L1／L2／L3／L4 證據：L1 fresh（Rust）；UE Editor 測試尚未執行；L3 未執行
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward/shadow-target
```

## 根因（依資料確認）

逐一檢查主 session 列出的可能原因：

| 假設 | 結果 |
|---|---|
| 小兵走另一條 mesh 套用路徑 | 否。lane_melee／caster／siege／super、所有 MOBA 野怪、b01_saika_corps_gunner、saika_gunner、lumen_sentry_turret 的生成 class 都是 `AOmAnimatedUnitActor`，並設定 QuaterniusV2 的 `NativeMeshAsset`（male／female_peasant、male_ranger，和已驗證正確的英雄同一組骨架），都會經過 `InitializeNativePresentation` 的自動對齊。 |
| codegen 給了明確的 yaw 而關掉自動對齊 | 否。`OmContentClasses.cpp` 裡沒有任何 `bAutoAlignMeshForward = false`，所有 `NativeMeshTransform` 的 yaw 都是 0。 |
| actor yaw 沒有跟著移動方向更新 | **是。** UE 的 actor yaw 直接取自 frame 的 `facing_rad`，bridge 從 replica 的 `DISCLOSED_FACING` component 讀取；沒有這個 component 時預設 0。權威端 `MobaMatch::spawn_wave` 建立分路小兵、`jungle::spawn_body` 建立野怪時都沒有加 `Facing` component，移動也只改 `Pos`，movement fact 送的是 `facing_ticks: None`。所以 replica 永遠沒有小兵和野怪的面向，UE 一律把它們轉向世界 +X。 |

英雄正確，是因為英雄有 `Facing`，`hero_move_tick`／`hero_tick` 會更新它，可見英雄的 pose fact 也帶著 facing。召喚物出生時繼承施法者的面向，`summon_tick` 依移動更新，攻擊時由 script 轉向目標（新測試確認雜賀眾槍兵會面向它射擊的敵人），本來就正確。

## 修正（權威端單一規則，所有 replica 經 fact 同步）

- `moba_match.rs` 新增 `direction_angle(from, to)` 與 `face_toward(world, entity, from, to)`：可見正面＝這一步的位移方向，或正在攻擊的目標方向；位移為 0 時保持原面向。只有面向改變時才回傳新的 ticks。
- 分路小兵：出生時就面向下一個 waypoint。每一步移動後 `face_toward(起點, 終點)`；目標在射程內（攻擊中）時 `face_toward(自己, 目標)`。PreStepMovement fact 帶上改變後的 `facing_ticks`，沒有移動、只有轉向時，也會送一筆位置不變但帶 facing 的 fact。
- 野怪：出生時帶 `Facing`。追擊時每一步都套用同一規則；在射程內時面向目標。新增 `emit_camp_pose` 統一送出 pose fact。
- replica 端不用改：`apply_authoritative_movement_outcomes` 本來就會把 payload ≥ 20 bytes 的 facing 寫進 specs `Facing`，再匯出成 `DISCLOSED_FACING`，進入每隊的 hash。所以 replica 和權威只要有任何偏差，都會被 checkpoint 抓到。
- 塔不套用這條規則（塔不是動畫單位，不轉向）。

沒有 proto、CABI、UE 或 codegen 變更。舊的 client 看到的 movement payload 會多 4 bytes 的 facing，這是 bridge／replica 既有的格式。

## 測試

在 shadow tree 執行，`CARGO_TARGET_DIR=target/session-ward/shadow-target`；著陸後在實際樹重跑新測試，並 `cargo check` omobab，皆通過。

| 命令 | exit | 結果 |
|---|---|---|
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | 0 | 1144 passed，含新的 `npc_facing_tests` 3 個 |
| `cargo test -p base_content` | 0 | 442 passed，含新的 `unit_facing_parity` 2 個 |
| `cargo test --locked -p omobab` | 0 | 193 passed、1 ignored |
| `cargo test --manifest-path omfue/bridge/Cargo.toml` | 101 | 誤報：失敗的是 shadow tree 裡過期的 bridge 副本，實際樹的 lib.rs:5325／5389 已有 `tower_hp`／`base_hp`（E055）。與本次無關 |

新測試：
- `lane_creeps_face_their_step_and_their_attack_target`（three_lane_training，3600 tick）：每一隻移動中的小兵都面向這一步的位移；每一次攻擊都面向目標。
- `lane_creeps_are_born_facing_their_next_waypoint`。
- `camp_monsters_face_their_chase_step_and_their_attack_target`：野怪被打之後追擊並攻擊英雄。
- `a_stationary_summon_faces_the_enemy_it_shoots`：雜賀眾槍兵開火時面向目標（確認召喚物本來就正確）。
- `lane_fights_disclose_identical_creep_facing_to_both_teams`：3600 tick 雙英雄推線 soak，兩隊 replica 逐 tick hash 一致。小兵的 `DISCLOSED_FACING` 現在在 hash 裡，所以這個測試也驗證了 replica 的面向。

mutation 驗證：讓 `face_toward` 不做事時，小兵與野怪測試失敗。注意：第一版小兵測試用直線單線地圖，mutation 下仍然通過（固定面向剛好等於直線移動方向），已改用 three_lane_training。

## UE Editor 測試（已寫，需主 session 編譯並執行）

`Om.Generated.EveryUnitClassFacesActorForward`（新檔 `OmEditor/Private/OmUnitFacingTests.cpp`）：
- 列舉 `/Script/OmGenerated` 中所有非 abstract 的 `AOmAnimatedUnitActor` 子類，包括英雄、分路小兵、野怪與召喚物。
- 有 native mesh 的 class 逐一 spawn 並呼叫 `InitializeNativePresentation`。必須載入 mesh 且骨架能算出正面；actor yaw 設為 0、90、-135 時，骨架正面的世界 yaw 都要等於 actor yaw（誤差 0.5°）。
- 關掉自動對齊的 class 則要求保留 authored yaw。
- 沒有 mesh 的 class（TD 與舊版單位，走 fallback 視覺）會列在 log 中。
- 至少要檢查 25 個 class。

## 主 session 需要做的

- 重建 server、client runtime、base_content（三端一起），以及 UE。
- 執行 `Om.Generated.EveryUnitClassFacesActorForward`。
- 實機：在 facing_probe 或截圖中確認小兵、野怪朝移動方向走、面向攻擊目標。

## 限制

- 小兵是瞬間轉向，沒有轉向速度；英雄有 `TurnSpeed`。如果之後要平滑轉向，可以在 UE 端插值，不要改權威端。
- 沒有 mesh、走 fallback 視覺的舊 TD 單位不在 MOBA 實機範圍內，所以只列出、不檢查。
