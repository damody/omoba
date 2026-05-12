## Why

目前 debug launcher 已可在啟動時用 runtime Lua content 載入 `scripts/lua_data`，但執行中的 backend、omfx replica 與 script DLL 都只讀一次 Lua snapshot。開發者修改 Lua 數值後仍需要重啟遊戲才能看到效果，拖慢調整塔、英雄、怪物與 ability 數值的迭代速度。

## What Changes

- 新增 DEV-only Lua content hot reload：`run.bat` 類 debug 模式啟用時，偵測 `scripts/lua_data` 變更並在執行中重新載入、驗證、套用新的 content generation。
- 更新 runtime Lua content store，從 process lifetime 一次性 `OnceLock` 改為可替換且有 generation/hash 的 snapshot，讓 backend、omfx replica 與 `base_content.dll` 取用一致的 active content。
- 將 hot reload 限制在既有 id 的數值、顯示文字與 render metadata 更新；template/story id 新增、刪除、重排、map topology 或 wave 結構變更在 DEV reload 時清楚拒絕並提示需要重啟。
- 套用 reload 後刷新 future spawn 來源、tower/ability/upgrades registries、frontend UI/template/asset caches，並以保守策略更新既有 live entities 的基礎數值。
- 保持 release/stress/default runtime 行為不變；不支援 Rust script DLL hot reload。

## Capabilities

### New Capabilities
- `dev-lua-hot-reload`: 定義 DEV 模式 Lua content 變更在執行中重新載入、同步到 backend/omfx replica、刷新 runtime caches，並套用到遊戲中數值的行為契約。

### Modified Capabilities

## Impact

- Affected code: `omoba-template-ids` runtime Lua content loader/store、`omb` game loop/content resources、`omoba-core` native runtime registries/snapshots、`scripts/base_content` active lookup usage、`omfx` sim_runner 與 frontend cache invalidation、debug launchers。
- Affected APIs: 新增 DEV-only reload API、content generation/hash 查詢、reload scheduling/result reporting；可能需要在 lockstep/tick metadata 或 snapshot metadata 攜帶 content generation。
- Affected systems: backend authoritative simulation、omfx local replica determinism、tower/hero/creep/ability template lookup、TD UI/template/asset cache。
- Dependencies: 可能新增 file watching 或 polling 機制；若新增 crate 必須只用於 DEV/runtime Lua content feature，且不得影響 release/stress default path。
