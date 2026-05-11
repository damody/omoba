## 1. Runtime Content Store

- [x] 1.1 將 `omoba-template-ids::runtime_content` 從一次性 `OnceLock` 調整為可替換的 DEV runtime store，並保留 existing `active_*` lookup API 行為
- [x] 1.2 新增 content generation/hash 查詢 API，例如 `runtime_lua_content_generation()` 與 `runtime_lua_content_hash()`
- [x] 1.3 新增 DEV-only reload API，例如 `reload_runtime_lua_content_dev(...)`，支援 expected hash 驗證與清楚錯誤回傳
- [x] 1.4 為 Lua-loaded manifest/story/template data 建立 deterministic hash，確保 backend、script DLL 與 omfx 可比對同一份 content
- [x] 1.5 實作 `ContentShape` 或等價 structural compatibility check，允許既有 id 數值/metadata 更新並拒絕 id/topology 變更
- [x] 1.6 補 `omoba-template-ids` 測試：successful reload、invalid Lua 保留舊 generation、structural change rejection、hash deterministic

## 2. DEV Gating And Backend Scheduler

- [x] 2.1 新增 `OMB_LUA_HOT_RELOAD` truthy env 判斷，並確保未啟用時沒有 watcher/poller/live apply 行為
- [x] 2.2 更新 `run.bat` 設定 `OMB_LUA_HOT_RELOAD=1`，並確認 `run_stress.bat` 不設定或清除此 env
- [x] 2.3 在 backend DEV path 實作 debounced polling，掃描 `OMB_LUA_CONTENT_ROOT` relevant files 的 path/mtime/size 變更
- [x] 2.4 backend 偵測候選變更後執行 reload validation，成功時建立 pending generation/hash 與 apply tick
- [x] 2.5 新增 ECS resource 或 transport metadata 來攜帶 active/pending content generation/hash 與 reload result
- [x] 2.6 補 backend 測試或 smoke helper：env gating、debounce reload、reload failure 不改 active generation

## 3. Script Module Synchronization

- [x] 3.1 擴充 `scripts/script-abi`，加入 DEV-only optional runtime Lua reload hook 或等價 module API，避免影響 default release path
- [x] 3.2 在 `scripts/base_content` 實作 hook，從相同 content root reload runtime Lua content 並回報 generation/hash
- [x] 3.3 在 `omoba-core` script registry 或 `omb` host reload pipeline 呼叫所有 loaded modules 的 hook，任一失敗或 hash mismatch 時 reject candidate
- [x] 3.4 補測試或 harness 驗證 host 與 script DLL hash 相符才套用 reload，mismatch 時保留舊 generation

## 4. Gameplay Apply Pipeline

- [x] 4.1 重建 future spawn 來源，包含 `CreepEmiter` 或等價 wave spawn cache，讓 reload 後生成的 creeps 使用新 template stats
- [x] 4.2 重建 `TowerTemplateRegistry`、`TowerUpgradeRegistry` 與 `AbilityRegistry`，並將 generation/hash 記錄在 metadata snapshot 來源
- [x] 4.3 實作 hero live refresh：更新 `Hero`、`CProperty`、`TAttack`、`TurnSpeed` 等 base stats，並以 HP ratio 保留目前血量比例
- [x] 4.4 實作 creep live refresh：更新 `CProperty`、bounty、movement/combat base stats，保留 wave progress 與 current orders
- [x] 4.5 實作 tower live refresh：更新 `TProperty`、`CProperty`、`TAttack`、`CircularVision`、render/metadata fields，並保留 tower entity、upgrade levels 與 runtime state
- [x] 4.6 確認 buffs、cooldowns、projectiles、orders、lockstep input history 不會在 stats refresh 時被重置
- [x] 4.7 補 gameplay tests：future spawn uses updated stats、existing entity preserves HP ratio、registry metadata updates、structural rejection leaves world unchanged

## 5. omfx Replica And Frontend Caches

- [ ] 5.1 將 backend scheduled reload generation/hash 傳到 omfx sim_runner，並在 apply tick 前 reload/verify matching hash
- [ ] 5.2 omfx hash mismatch 或 local reload failure 時停止 local sim apply，並回報清楚 DEV reload error state
- [ ] 5.3 在 sim_runner reload 成功後重建 abilities、tower templates、tower upgrades metadata arcs 或等價 caches
- [ ] 5.4 在 snapshot 或 sim metadata 加入 content generation/hash，讓 frontend UI 能偵測新 generation
- [ ] 5.5 frontend 看到 generation 改變時清除並重新 seed tower template、upgrade、ability、icon/texture/model 等 Lua-referenced caches
- [ ] 5.6 補 omfx tests 或 smoke 驗證：tower sidebar values/icons 更新、asset path cache invalidation、mismatch 不 silent diverge

## 6. Verification

- [ ] 6.1 執行 `cargo test --manifest-path scripts/Cargo.toml -p base_content` 驗證 script-side reload hook 與 runtime Lua content feature
- [ ] 6.2 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab` 或相關 package tests 驗證 backend reload scheduler 與 gameplay apply pipeline
- [ ] 6.3 執行 `cargo test --manifest-path omb/Cargo.toml -p omb-script-abi` 驗證 ABI 變更
- [ ] 6.4 用 `run.bat` 手動驗證：修改既有 Lua 數值後，不 rebuild、不重啟即可看到遊戲內數值/UI 更新
- [ ] 6.5 用 `run_stress.bat` 或 release/stress smoke 驗證 hot reload 未啟用且 generated-data 行為不變
