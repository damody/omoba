## Context

現有 debug runtime Lua content mode 透過 `runtime-lua-content` feature 與 `OMB_LUA_CONTENT=1`，讓 `omb`、`omfx` 與 `scripts/base_content` 在啟動時從 `scripts/lua_data` 載入 template/story data。核心載入點在 `omoba-template-ids`，目前以 process lifetime `OnceLock` 保存 `RuntimeContent`，`active_*` lookup 會在第一次初始化後持續回傳同一份 snapshot。

遊戲執行後，多數數值會被 copy 到 ECS components 或 registries：hero/creep/tower stats 進 `CProperty`、`TAttack`、`Hero`、`CreepEmiter`、`TowerTemplateRegistry`、`TowerUpgradeRegistry` 與 `AbilityRegistry`；omfx `sim_runner` 與 UI 也會建立一次性的 tower template、ability、upgrade 與 asset caches。因此「重新讀 Lua」不足以讓遊戲中數值即時改變，還必須同步 generation、重建 registries、刷新 live entities 與清除 frontend caches。

此 change 只針對 DEV 模式。Release/stress/default path 必須保持 generated Rust data 與目前 deterministic 行為，不因 hot reload 多出 runtime dependency 或 watcher。

## Goals / Non-Goals

**Goals:**

- 在 DEV 模式偵測 `scripts/lua_data` 變更，debounce 後重新載入並驗證 Lua content。
- 讓 backend、script DLL 與 omfx local replica 在同一 content generation/hash 下套用 reload，避免 lockstep 分叉。
- 將既有 template id 的數值、顯示文字、render metadata、ability constants 與 tower upgrade definitions 套用到 future spawns、registries、snapshot metadata 與前端 UI cache。
- 對已存在的 hero、tower、creep 套用保守 live refresh：保留 runtime 狀態，更新基礎 stats，並以 HP ratio 保留目前血量比例。
- reload 失敗或不相容時保留上一個有效 generation，遊戲不中斷，並輸出清楚 log。

**Non-Goals:**

- 不支援 Rust `base_content.dll` hot reload；script Rust 程式碼變更仍需 rebuild/restart。
- 不支援 production/release/stress 預設啟用 hot reload。
- 不支援 live 新增、刪除、重排 template/story id，或在執行中改 map topology、path、wave 結構後保證完整套用。
- 不重播或回滾已發生的攻擊、projectile、cooldown、buff duration、lockstep input。

## Decisions

### DEV hot reload 以 `OMB_LUA_HOT_RELOAD=1` 顯式啟用

`OMB_LUA_CONTENT=1` 只代表 runtime 可從 Lua 初始化；新增 `OMB_LUA_HOT_RELOAD=1` 才啟動 watcher/polling 與 live apply。`run.bat` 設定此 env；stress launcher 清除此 env 並維持 release generated-data path。

選擇理由：runtime Lua content 也可能被 release opt-in 使用，但 hot reload 會改變 determinism 與資源生命週期，必須更明確地限制在 DEV。

替代方案：直接把 hot reload 綁在 `OMB_LUA_CONTENT=1`。這較省設定，但會讓所有 runtime Lua content opt-in 都承擔 live reload 行為，不適合 release debugging 或 smoke/stress。

### 使用 debounced polling，不先引入 filesystem watcher crate

DEV reload worker 以固定 interval 掃描 `OMB_LUA_CONTENT_ROOT` 下 relevant files 的 path、mtime、size，發現變更後等待短暫 debounce，再呼叫既有 Lua loader 重建 snapshot。Reload 成功後用 canonical manifest 計算 content hash。

選擇理由：Windows 開發環境下 polling 較可預期，不需要新增跨平台 watcher dependency，也能涵蓋 include/read helper 讀到的檔案。`scripts/lua_data` 規模小，DEV-only polling 成本可控。

替代方案：使用 `notify`。反應較即時，但會新增 dependency、平台差異與事件抖動處理；初版沒有必要。

### Runtime content store 改為 generation-aware 可替換 snapshot

`omoba-template-ids` 將一次性 `OnceLock<Result<Option<RuntimeContent>, String>>` 調整為 process 內可替換的 runtime store，提供：

- `ensure_runtime_lua_content()`：維持初始化語意。
- `reload_runtime_lua_content_dev(expected_hash?)`：DEV-only reload，成功時提升 generation。
- `runtime_lua_content_generation()` / `runtime_lua_content_hash()`：查詢目前 generation/hash。

既有 `active_*` API 初期保留回傳 `'static` references；每個 accepted DEV generation 可以 leak snapshot，確保舊 references 在 tick 或 script 呼叫期間仍有效。

選擇理由：大幅改成 `Arc`/borrowed lifetime 會牽動 `omb`、`omoba-core`、`omfx`、`scripts/base_content` 多處 API。DEV-only bounded leak 是較小、較安全的架構變更；reload 次數通常有限，且只在開發執行中存在。

替代方案：全面改 `active_*` 回傳 owned/Arc data。長期較乾淨，但實作面積大，會把 hot reload change 擴張成資料模型遷移。

### Authoritative backend 排程 reload，omfx replica 以同 tick 同 hash 套用

backend 偵測並驗證新的 Lua snapshot 後，不立即讓 local replica 自行猜測 reload 時機，而是排程一個 DEV reload control event，帶 generation/hash 與 apply tick。omfx sim_runner 收到事件後，在相同 tick 前從相同 `OMB_LUA_CONTENT_ROOT` reload，驗證 hash 相符後才套用；hash mismatch 時暫停 local sim 並要求重啟或重新同步。

選擇理由：backend 與 omfx local replica 都會跑 simulation 與 script dispatch。如果各自 watch filesystem，debounce 時機不同就可能造成 lockstep 分叉。

替代方案：backend/omfx 各自 polling 並期待同時 reload。實作較少，但 determinism 風險高，不採用。

### Script DLL 需要顯式 reload hook

`base_content.dll` 作為 cdylib 可能持有自己的 `omoba-template-ids` runtime store。host reload 成功後，`ScriptRegistry` 需透過 script ABI 呼叫 DEV-only optional hook，要求每個 module 用同一 content root reload 並回報 hash；任一 module hash 不符則整次 reload 失敗，host 保留舊 generation。

選擇理由：tower/ability metadata 與 script tick 可能在 DLL 內呼叫 `active_*`，只更新 host store 會造成 host 與 DLL 數值漂移。

替代方案：要求 script DLL 每次呼叫都向 host 查數值。這會改變 script ABI 資料流且成本較高，不適合初版。

### Structural compatibility gate 保護 live reload 範圍

每次 reload 都和目前 generation 建立 `ContentShape` diff。允許既有 ids 的 numeric/text/render/metadata 欄位變更；拒絕下列變更並輸出原因：

- template、ability、projectile、story id 新增、刪除或重排。
- active story 的 map path、spawn wave topology、entity topology、blocking/pathing topology 變更。
- 無法對應現有 hero/tower/creep instance 的 unit id 或 ability id 變更。

選擇理由：使用者需求是調整「數值」，不是在跑局中重建整個關卡。結構變更牽涉 entity lifecycle、pathfinding、lockstep hash 與 frontend scene graph，應要求重啟。

替代方案：任何 Lua 變更都 full world rebuild。這能支援更多結構變更，但會中斷遊戲狀態，不符合「即時更新數值到遊戲中」。

### Apply 分成 content reload、registry refresh、live entity refresh、frontend cache invalidation

Reload apply pipeline：

1. 載入並驗證新 Lua content snapshot。
2. Reload script DLL runtime store 並驗證 hash。
3. 檢查 structural compatibility。
4. 重建 `CreepEmiter`、`TowerTemplateRegistry`、`TowerUpgradeRegistry`、`AbilityRegistry`。
5. Refresh live heroes/towers/creeps 的 base stats；`CProperty.hp` 依舊 HP ratio 映射到新 `mhp`，buffs、cooldowns、projectiles 與 current orders 保留。
6. 提升 `ContentGeneration` resource，讓 snapshots 攜帶最新 generation。
7. omfx sim_runner 重建 snapshot metadata arcs；frontend UI 看到 generation 改變後清除 tower/ability/upgrade/asset caches 並重新 seed。

選擇理由：各層 cache 生命週期不同，必須以 generation 串起來；同時保留 live runtime 狀態可避免 hot reload 變成隱性重啟。

替代方案：只更新 lookup store，不刷新 ECS/Frontend。這無法讓已存在 entity 或 UI 立即看到新數值，不符合需求。

## Risks / Trade-offs

- Runtime snapshot leak → 限定在 `OMB_LUA_HOT_RELOAD=1`，並在 log 顯示 generation 次數；release/stress 不啟用。
- Reload 時機造成 lockstep 分叉 → 由 backend authoritative 排程 apply tick，omfx 以 hash 驗證後同 tick 套用。
- Structural diff 過嚴導致部分 Lua 修改被拒絕 → 初版優先安全與可預測；log 必須列出 rejected paths/ids 與「需要重啟」提示。
- Live entity refresh 與 buffs/upgrades 疊加出現混合狀態 → base stats 與 upgrade definitions 以同 generation 重建，buff runtime state 保留；若某 buff payload 無法安全重算，保留現況並在 log 標示。
- File polling 掃描成本 → 只在 DEV hot reload env 啟用，interval/debounce 可調，`scripts/lua_data` 規模小。
- Frontend asset cache invalidation 造成短暫貼圖重載抖動 → 只在 generation 改變時清除受影響 caches，既有 scene nodes 在下一次 snapshot/update 修正。

## Migration Plan

1. 在 `omoba-template-ids` 新增 generation-aware runtime store 與 DEV reload API，保持 existing `active_*` lookup 相容。
2. 在 debug launcher 加入 `OMB_LUA_HOT_RELOAD=1`，stress/default path 明確清除或不設定。
3. 在 backend 加入 DEV polling/reload scheduler，並將 reload result、generation/hash 暴露到 ECS resource 或 transport metadata。
4. 擴充 script ABI/registry 的 DEV-only runtime Lua reload hook，讓 host 與 DLL 驗證同 hash。
5. 在 `omoba-core`/`omb` 實作 registry rebuild 與 live entity refresh helpers。
6. 在 omfx sim_runner 實作同 tick reload apply、metadata arc rebuild 與 mismatch handling；在 frontend UI 依 generation 清 cache。
7. 補測試與 smoke 驗證：successful numeric reload、incompatible structural reload rejection、hash mismatch、release/stress 未啟用。

Rollback 策略：移除或清除 `OMB_LUA_HOT_RELOAD` 即可回到啟動時 runtime Lua content 行為；若 reload API 存在但 env 未啟用，不應改變既有 gameplay。

## Open Questions

- `run_smoke.bat` / `run_smoke_long.bat` 是否也要預設啟用 hot reload，或只讓 `run.bat` 啟用以避免 smoke 測試受 watcher 影響？
- Live tower upgrades 是否要完整 clear/reapply script-owned upgrade buffs，或初版只更新 base template 與 future upgrade definitions？
- Ability cooldown/damage 若由 script tick 動態讀 `active_ability_const()`，reload 後是否需要額外廣播 ability metadata generation 給 UI？
