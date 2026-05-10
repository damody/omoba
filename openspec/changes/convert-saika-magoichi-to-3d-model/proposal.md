## Why

`saika_magoichi` 目前在戰鬥場景中仍以通用 2D batched quad 與 facing bar 呈現，無法使用已放在 `scripts/lua_data/templates/heroes/saika_magoichi/` 的專屬 FBX 模型與貼圖。這次變更讓雜賀孫市改以 content-owned 3D 模型呈現，同時保留既有英雄數值、技能、UI portrait 與 lockstep gameplay 行為。

## What Changes

- 在 hero template metadata 中加入可選的 3D render 設定，讓 `saika_magoichi` 指向 `saika_magoichi.fbx` 與 `saika_magoichi_mat.png`。
- 讓 generated template data 與 `SimWorldSnapshot` 提供 hero render metadata，供 omfx 依 `unit_id` 建立與更新 3D hero visual。
- 在 omfx 新增 hero 3D model 載入、節點生命週期、位置與 facing 同步；有 3D metadata 的 hero 不再顯示通用 2D body/facing quad。
- 保留 2D fallback：缺少模型、貼圖、metadata 或載入失敗時，英雄仍使用現有 2D batched quad 可見且可操作。
- 不變更英雄技能、屬性、碰撞、攻擊、移動、portrait、ability icon 或 backend gameplay protocol。

## Capabilities

### New Capabilities
- `hero-3d-rendering`: 定義 hero 3D asset metadata 的 canonical 來源、snapshot contract、omfx 3D model rendering、fallback 與 gameplay isolation。

### Modified Capabilities
- 無。

## Impact

- `scripts/lua_data/templates/heroes.lua`：新增 `saika_magoichi` 的 3D render metadata。
- `scripts/lua_data/templates/heroes/saika_magoichi/`：使用既有 `saika_magoichi.fbx` 與 `saika_magoichi_mat.png` 作為 content-owned assets。
- `omoba-template-ids/build.rs` 與 generated API：新增 hero render metadata 型別與 lookup。
- `omfx/game/src/sim_runner.rs`：snapshot expose hero render template metadata。
- `omfx/game/src/lib.rs`：新增 hero 3D node cache、asset loader、transform update 與 fallback 行為。
- 測試影響：template codegen tests、snapshot/render metadata tests、omfx build；需要確認 `run.bat` 的 debug build 能載入模型而不影響 TD/MOBA runtime。
