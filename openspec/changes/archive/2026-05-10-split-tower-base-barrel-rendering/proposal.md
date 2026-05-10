## Why

目前 TD 塔在戰鬥畫面中多半是一張整體圖或簡化節點，無法讓砲口獨立轉向目標，也缺少開火瞬間的視覺回饋。把塔拆成底座圖與砲口圖，並讓砲口追蹤目標、開火時整組往後震，可以讓塔的攻擊行為更容易被玩家讀懂，也讓後續美術替換與不同塔型表現更有彈性。

## What Changes

- 新增戰鬥畫面用的 composite tower rendering：一般 TD 塔由 base sprite 與 barrel sprite 組成；特殊範圍傷害塔可由沒有 barrel 的 body animation frames 組成。
- 砲口 sprite SHALL 支援依塔種設定旋轉模式：一般砲塔在有有效攻擊目標或最近一次攻擊方向時朝向該方向；`tower_tack` 這類針塔 SHALL 可設定為不跟目標旋轉的固定/放射型顯示。
- 所有 barrel SHALL 支援由多張連續 PNG 組成 animation sequence；沒有宣告 frames 時才 fallback 到單張 barrel 圖。
- `tower_tack` 這類放射針塔 SHALL 支援依升級狀態動態改變砲管/針孔數量，至少涵蓋 8、12、16 根同時發射針的視覺狀態。
- 新增一種沒有砲管的範圍傷害塔 archetype：塔本體由多張連續 frame PNG 組成動畫，攻擊時播放或加速播放範圍傷害動畫，不需要 barrel sprite 或目標朝向。
- 所有單位（英雄、召喚物、creep、tower）攻擊 SHALL 有三個權威階段：攻擊前搖、攻擊瞬間事件、攻擊後搖；攻擊瞬間不是 duration，攻速變快時前搖與後搖 SHALL 依整數權重縮短且總和維持完整攻擊間隔。
- 前端 SHALL 在攻擊前搖開始時收到 render cue 並立即播放攻擊動畫；攻擊瞬間才對應 projectile spawn、damage apply 或命中事件。
- 發射瞬間 base 與 barrel SHALL 同步播放短暫 recoil 表演；可依塔種設定為沿砲口反方向後震，或整座塔先縮小再回彈放大的 scale pulse。
- 每種塔的 base 圖、barrel 圖或 animation frames、rotation mode、barrel layout、animation 參數、pivot/offset 與 recoil 參數 SHALL 由 scripts content mod 設定，預設路徑位於 `scripts/base_content/assets/towers/` 與 `scripts/lua_data/templates/towers.lua`。
- 目前所有塔都 SHALL 提供可替換的甜點戰爭 placeholder 圖片，並在 `asset-prompts.md` 為每張圖片提供完整 ChatGPT 生圖提示詞。
- 缺少任何 tower combat 圖片或 metadata 時，前端 SHALL fallback 到既有塔圖或 placeholder，且不得 panic 或影響 gameplay。
- 發射表演只影響前端視覺；不得改變 lockstep simulation、命中判定、攻速、彈道或傷害結果。

## Capabilities

### New Capabilities
- `tower-composite-combat-rendering`: 定義 TD 戰鬥畫面中塔的 base/barrel 雙 sprite 組合、所有 barrel frame animation、一般砲塔砲口朝向目標、針塔固定/放射型不旋轉特例、針塔依升級動態改變砲管數、無砲管範圍傷害動畫塔、開火 recoil 表演，以及 scripts content mod 擁有的可替換資源、甜點戰爭 placeholder 與完整生圖提示詞。
- `unit-attack-phase-timing`: 定義所有單位共用的 attack windup、impact、backswing 三階段時序、攻速縮放規則，以及前後端 attack animation cue contract。

### Modified Capabilities
- `sim-snapshot-rendering`: 擴充 render-facing tower snapshot/event contract，讓 omfx 能取得塔砲口朝向所需的目標或攻擊方向，以及開火瞬間觸發 recoil 的 render-only 事件資料。

## Impact

- Affected content: `scripts/lua_data/templates/towers.lua` 需要新增 tower render metadata；`scripts/base_content/assets/towers/` 需要放置 base/barrel PNG、barrel animation frame PNG、tack 多砲管 variant PNG、無砲管範圍塔 animation frame PNG、README 與 prompt 文件。
- Affected codegen/API: `omoba-template-ids`、`scripts/script-abi`、`scripts/base_content` 與 `omb` 的 `TowerTemplateRegistry` 需要傳遞 tower render metadata，但要保持 ABI 型別可跨 DLL 邊界安全。
- Affected simulation snapshot: `omfx/game/src/sim_runner.rs::SimWorldSnapshot` 與 `EntityRenderData` 需要暴露 tower aim/recoil 與 unit attack phase cues 所需的 render-only data，且不得讓 snapshot extraction mutation 影響 determinism。
- Affected frontend: `omfx/game/src/lib.rs` 或 render bridge 需要把 tower node 拆成 base/barrel child nodes 或 animated-area frame node，處理 barrel rotation、barrel/body frame animation、texture fallback、recoil lifetime 與 per-frame transform 更新。
- Affected tooling/docs: script asset README、gen-docs 或 unit catalog 若顯示 tower metadata，應能描述 base/barrel 圖與 recoil 參數。
