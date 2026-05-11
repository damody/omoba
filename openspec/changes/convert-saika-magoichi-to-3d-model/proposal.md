## Why

`saika_magoichi` 目前在戰鬥場景中仍以通用 2D batched quad 與 facing bar 呈現，無法使用已放在 `scripts/lua_data/templates/heroes/saika_magoichi/` 的專屬 FBX 模型與貼圖。這次變更讓雜賀孫市改以 content-owned 3D 模型呈現，同時保留既有英雄數值、技能、UI portrait 與 lockstep gameplay 行為。

## What Changes

- 在 scripts-owned hero template metadata 中加入可選的 3D render 設定，讓 `saika_magoichi` 指向 `saika_magoichi.fbx` 與 `saika_magoichi_mat.png`；asset path、texture path、scale、pitch/roll/yaw/z offset、muzzle bone、animation source 與 tick ranges 都 SHALL 存在於 `scripts/lua_data`，不放在 `omfx` source 或 `omfx/data`。
- 依 `assimp info` 檢查結果列出 Saika base FBX：4 meshes、32 bones、1 animation (`Take 001`)、24 animation channels；並使用同目錄 action FBX (`b01_ani_attack/run/stand3...`) 作為 scripts-owned animation sources，將 attack、critical、move、sniper 四個動作綁到各自 source 的明確 tick ranges。
- 讓 generated template data 與 `SimWorldSnapshot` 提供 hero render metadata，供 omfx 依 `unit_id` 建立與更新 3D hero visual。
- 在 omfx 新增 hero 3D model 載入、節點生命週期、位置、facing 與 animation state 同步；有 3D metadata 的 hero 不再顯示通用 2D body/facing quad。
- 讓 omfx 依 snapshot/render cues 切換 Saika 的移動、攻擊、爆擊與狙擊模式動畫，缺少特定 cue 時使用安全 fallback clip。
- 正確處理攻擊動畫與攻擊生命週期的前搖、擊中、後搖：攻擊在 impact 前可被已接受的移動或技能指令取消且不造成傷害；impact 後進入後搖時即使被移動或技能取消，攻擊結果仍保留。
- `omfx` 只提供通用功能：讀取 generated/snapshot metadata、解析 scripts asset path、載入 model/texture、播放 metadata 指定的 animation segment；不得 hard-code Saika 專屬 path、scale、tick range 或 action mapping。
- 保留 2D fallback：缺少模型、貼圖、metadata 或載入失敗時，英雄仍使用現有 2D batched quad 可見且可操作。
- 不變更英雄技能數值、屬性、碰撞、攻擊傷害公式、portrait、ability icon 或 backend gameplay protocol。

## Capabilities

### New Capabilities
- `hero-3d-rendering`: 定義 hero 3D asset metadata 的 canonical 來源、snapshot contract、omfx 3D model rendering、fallback 與 gameplay isolation。

### Modified Capabilities
- `unit-attack-phase-timing`: 補上 attack windup cancel、impact commit point、backswing cancel 與前端動畫對齊規則。

## Impact

- `scripts/lua_data/templates/heroes.lua`：新增 `saika_magoichi` 的 3D render metadata。
- `scripts/lua_data/templates/heroes/saika_magoichi/`：使用既有 `saika_magoichi.fbx` 與 `saika_magoichi_mat.png` 作為 content-owned assets。
- `omoba-template-ids/build.rs` 與 generated API：新增 hero render metadata 型別與 lookup。
- `omfx/game/src/sim_runner.rs`：snapshot expose hero render template metadata 與 render-only hero animation cues。
- `omfx/game/src/lib.rs`：新增 data-driven hero 3D node cache、scripts asset loader、transform update、animation binding/playback 與 fallback 行為；不新增 Saika 專屬資料表。
- `omb` / lockstep attack scheduling：需要讓移動與技能指令依攻擊階段取消或保留攻擊結果，並提供 render-only cancel/phase cue 讓前端動畫同步。
- 測試影響：template codegen tests、snapshot/render metadata tests、omfx build；需要確認 `run.bat` 的 debug build 能載入模型而不影響 TD/MOBA runtime。
