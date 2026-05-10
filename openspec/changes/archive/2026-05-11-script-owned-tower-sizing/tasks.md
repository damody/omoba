## 1. Content Metadata 與文件

- [x] 1.1 在 `scripts/lua_data/templates/towers.lua` 為每座 shipped tower 明確宣告 `render.visual_size` 與 `placement_radius`，包含 `tower_dart`、`tower_tack`、`tower_bomb`、`tower_ice`、`tower_cake_splash`。
- [x] 1.2 移除 Lua 以外的 sizing fallback 需求；若保留 Lua helper/default，確認值仍由 scripts mod 輸出且不是 Rust runtime 推導。
- [x] 1.3 更新 `scripts/base_content/assets/towers/README.md`，說明 `render.visual_size` 與 `placement_radius` 的單位、用途與和 `WORLD_SCALE`、`footprint`、runtime `CollisionRadius` 的差異。
- [x] 1.4 更新 gen-docs model/render output，顯示 `render.visual_size` 與 `placement_radius`。

## 2. Codegen 與 ABI Metadata

- [x] 2.1 更新 `omoba-template-ids` tower Lua model，讀取 `render.visual_size` 與 top-level `placement_radius`。
- [x] 2.2 更新 `TowerRenderMetadataConst` 或相關 const-facing 型別，加入 `visual_size`，並為 tower stats 或 metadata 加入 `placement_radius`。
- [x] 2.3 在 codegen validation 中拒絕 `render.visual_size <= 0` 或 `placement_radius <= 0`，不得 fallback 到 `footprint`、clamp、multiplier 或 `/ 2`。
- [x] 2.4 更新 generated metadata tests，確認 `tower_dart` 與 `tower_bomb` 的 `render.visual_size`、`placement_radius` 來自 Lua 明確值。
- [x] 2.5 更新 `scripts/script-abi` 的 ABI-safe `TowerMetadata` / `TowerRenderMetadata`，傳遞 `render.visual_size` 與 `placement_radius`。
- [x] 2.6 更新 `scripts/base_content` metadata builder，將 generated sizing fields 填入 ABI metadata。

## 3. Backend Runtime 與 Placement

- [x] 3.1 更新 `omb/src/comp/tower_registry.rs` runtime tower template 型別，保存 `render.visual_size` 與 `placement_radius`。
- [x] 3.2 更新 `omb/src/state/initialization.rs` registry populate path，從 script ABI metadata 投影 sizing fields。
- [x] 3.3 更新 `omb/src/state/core.rs` tower template broadcast JSON，包含 explicit sizing fields。
- [x] 3.4 更新 `omb/src/state/resource_management.rs` tower create placement validation，使用 `placement_radius` 做 path、region 與 tower-overlap 檢查。
- [x] 3.5 移除後端 placement 中所有由 `render.visual_size / 2`、`footprint` 或 Rust 常數推導 placement radius 的邏輯。
- [x] 3.6 確認 tower spawn 後的 runtime `CollisionRadius`、attack range、projectile spawn、damage、cooldown 與 lockstep state hash 不因 `placement_radius` 改變。

## 4. Frontend Snapshot 與 Rendering

- [x] 4.1 更新 `omfx/game/src/sim_runner.rs::TowerTemplateSnapshot`，加入 `render_visual_size` 與 `placement_radius`。
- [x] 4.2 更新 `extract_snapshot` tower template projection，從 runtime registry 複製 explicit sizing fields 並維持 `Arc` static lifecycle。
- [x] 4.3 更新 `omfx/game/src/lib.rs::TdTemplate` 與 template cache，保存 explicit sizing fields。
- [x] 4.4 更新 composite tower render size，使用 `render_visual_size * WORLD_SCALE` 作為 base/barrel/body 長期基準大小。
- [x] 4.5 確認 recoil、attack animation、buff/hover visual scale 只作為短暫 transform 疊加，不覆寫 script-owned base size。
- [x] 4.6 更新 local tower placement preview 與 can-place checks，使用 snapshot/cache 中的 `placement_radius * WORLD_SCALE`。
- [x] 4.7 移除前端所有 persistent tower sizing 常數或公式，例如 `TD_TOWER_VISUAL_SCALE`、footprint multiplier、clamp、`visual_size / 2` placement radius。

## 5. Tests 與驗證

- [x] 5.1 執行 `cargo test --manifest-path omoba-template-ids/Cargo.toml`。
- [x] 5.2 執行 `cargo test --manifest-path scripts/Cargo.toml -p base_content`。
- [x] 5.3 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab` 或至少 placement/snapshot 相關 focused tests。
- [x] 5.4 執行 `cargo check --manifest-path omfx/Cargo.toml`。
- [x] 5.5 執行 `openspec validate --all --strict`。
- [x] 5.6 手動驗證 TD_1 放塔 preview footprint 圈與後端實際可放置結果一致。
- [x] 5.7 手動驗證調整 `scripts/lua_data/templates/towers.lua` 中單座 tower 的 `render.visual_size` 只改變視覺基準大小，不影響 attack range、damage、cooldown。
- [x] 5.8 手動驗證調整單座 tower 的 `placement_radius` 只改變放置碰撞，不影響 runtime `CollisionRadius`、攻擊與彈道。
- [x] 5.9 使用 grep 或 focused test 確認 tower placement sizing 不再包含 `render.visual_size / 2`、`render.size / 2`、`footprint` fallback 或 frontend global visual scale。
