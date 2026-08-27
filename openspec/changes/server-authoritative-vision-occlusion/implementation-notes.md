# 實作盤點

- Map descriptor 與 Lua 反序列化：`omoba-core/src/runtime/native/scene/import_map.rs::CreepWaveData`、`omoba-core/src/runtime/native/scene/import_campaign.rs::normalize_map_value`。
- `BlockedRegions` 參考流程：`StateInitializer::setup_blocked_regions` → `BlockedRegions` resource → snapshot/render bridge。
- Demo content：`scripts/lua_data/FOG_2TEAM_DEMO/map.lua`；建立 100 個普通單位與兩位額外英雄的位置為 `StateInitializer::create_fog_demo_scene`。
- Wave B committed view：`omoba-core/src/runtime/visibility.rs::build_wave_b_read_view`。
- Team projection 與送出資料：`run_committed_visibility_wave_b`、`omoba-core/src/runtime/team_projector.rs`。
- Replica observer：既有 `TeamProjectionRuntime`/transport observer queue 驗算路徑；本 change 不建立旁路 canonical read。
- omfx snapshot 與 Forget：`omfx/game/src/sim_runner.rs`、`omfx/game/src/native.rs` 的 team-filtered replica/render mirror 清除路徑。
- 公開遮蔽呈現：`SimWorldSnapshot.vision_occluders` → `omfx/game/src/render_bridge.rs`；只作 debug presentation。

## 實作決策

- 遮蔽物是獨立 `VisionOccluderSet` resource，不建立 ECS entity，也不加入 `BlockedRegions`。
- Wave B 使用 stable-sorted immutable occluder slice；LOS 使用 fixed raw `i128` checked arithmetic。
- 樹木及地形輪廓沿用 `BlockedRegionSnapshot` 的公開幾何形狀，但放在獨立 `vision_occluders` 欄位。
