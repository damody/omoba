## Context

TD tower 目前的 runtime metadata 由 `scripts/lua_data/templates/towers.lua` 經 `omoba-template-ids` 產生，再由 `scripts/base_content` 的 tower script 回報 `TowerMetadata`，host 端彙整成 `TowerTemplateRegistry`，最後透過 `SimWorldSnapshot.tower_templates` 提供給 omfx。戰鬥畫面的 entity rendering 已使用 snapshot-backed mirror，tower selection、upgrade UI 與 range display 都從 snapshot/mirror 讀取，不應回到 legacy frontend-only state。

這次需求新增兩類資料：靜態的 tower render metadata（base/barrel 圖、barrel animation frames、沒有砲管的 body animation frames、offset、pivot、recoil 參數）與動態的 tower fire cue（哪座塔在哪個 tick 開火、方向為何）。靜態資料應由 scripts content mod 擁有；動態 cue 應維持 render-only，類似既有 `ExplosionFxQueue`，不得進入 determinism hash 或改動 gameplay 結果。

SVG 示意圖位於 `openspec/changes/split-tower-base-barrel-rendering/combat-tower-layout.svg`，用來說明一般 target-facing tower、barrel frame animation、`tower_tack` radial count variants，以及無砲管 animated-area tower 的視覺意圖。

## Goals / Non-Goals

**Goals:**

- 每座 TD 塔在戰鬥畫面依 metadata render 為 base sprite + barrel sprite，或 render 為沒有 barrel 的 animated-area frame sequence。
- barrel sprite 依 tower render metadata 的 rotation mode 表現：一般砲塔朝向目前目標或最近一次有效攻擊方向，`tower_tack` 這類針塔則固定不旋轉並使用放射型多砲管視覺，砲管/針孔數量可依升級狀態動態切換。
- 所有 barrel 都可選擇由多張連續 PNG 組成 animation frames，開火時可從第一張重播或加速播放。
- tower 開火瞬間，base 與 barrel 依 scripts metadata 播放短暫 recoil，沿 barrel 反方向後震再回彈；無砲管範圍塔則播放 body frame animation 與可選 scale pulse。
- tower render 圖片、barrel animation frames、body animation frames 與 recoil 參數由 scripts content mod 設定，預設資源放在 `scripts/base_content/assets/towers/`。
- 缺圖、缺 metadata 或舊塔資料仍可 fallback，不 panic、不阻塞 gameplay。
- 保持 stress 場景友善：不在每 frame 建立/刪除 UI/scene nodes，不每 frame 重新載入 texture。

**Non-Goals:**

- 不重做 projectile gameplay、命中判定、attack cooldown、damage 或 lockstep protocol。
- 不要求第一版支援骨架動畫、粒子 muzzle flash、sprite atlas packing 或正式美術流程。
- 不讓前端自行選 target 或預測 gameplay 目標；前端只消費 snapshot/render cue。
- 不把 recoil 參數變成 gameplay stat；它只影響 render transform。

## Decisions

### Decision: render metadata 由 tower Lua template 宣告，assets 放在 scripts content mod

在每個 tower template 增加 `render` table，例如：

```lua
render = {
  render_mode = "base_barrel",
  base = "assets/towers/tower_dart_base.png",
  barrel = "assets/towers/tower_dart_barrel.png",
  barrel_frames = {
    "assets/towers/tower_dart_barrel_frame_01.png",
    "assets/towers/tower_dart_barrel_frame_02.png",
    "assets/towers/tower_dart_barrel_frame_03.png",
  },
  barrel_animation = { fps = 12.0, loop = true, fire_fps = 20.0, fire_once = true },
  rotation_mode = "targeted",
  barrel_layout = "single",
  barrel_offset = { x = 0.0, y = -6.0 },
  barrel_pivot = { x = 0.5, y = 0.65 },
  muzzle_offset = { x = 0.0, y = -28.0 },
  recoil = {
    mode = "directional",
    distance = 8.0,
    scale = 0.92,
    duration_ms = 80,
    return_ms = 120,
  },
}
```

`base`、`barrel` 與 frame 路徑以 `scripts/base_content/` 為基準，第一版預設目錄為 `scripts/base_content/assets/towers/`。`render_mode` 預設為 `base_barrel`；沒有砲管的範圍傷害塔使用 `animated_area`，此模式不需要 `barrel`、`rotation_mode` 或 target-facing aim。`barrel_frames` 是所有砲管共用的 optional animation sequence；若缺值或任何 frame 載入失敗，omfx SHALL fallback 到單張 `barrel` 圖與可診斷 log。`barrel_animation` 控制 idle/fire frame timing。`rotation_mode` 預設為 `targeted`，代表 barrel 會跟 snapshot aim/facing 旋轉；`tower_tack` 這類向四周發射的針塔使用 `fixed`，代表 barrel 視覺保持 metadata default angle，不跟單一目標旋轉。`barrel_layout` 預設為 `single`；針塔可用 `radial_count_variants`，讓 omfx 依 tower upgrade levels 選擇 8/12/16 根砲管或針孔的 variant。`barrel_offset` 使用 render-local 單位，`barrel_pivot` 使用 0..1 normalized texture pivot，`muzzle_offset` 供 recoil 或未來 muzzle flash 定位使用。`recoil.mode` 可為 `directional` 或 `scale_pulse`；`directional` 沿 firing direction 反方向後震，`scale_pulse` 讓整座塔的 base/barrel 組合先縮小再回彈放大。`recoil.distance`、`scale`、`duration_ms`、`return_ms` 可自定義，缺值走 default。

`tower_tack` 的建議 metadata：

```lua
render = {
  base = "assets/towers/tower_tack_base.png",
  barrel = "assets/towers/tower_tack_barrel_8.png",
  rotation_mode = "fixed",
  barrel_layout = "radial_count_variants",
  barrel_variants = {
    {
      min_path = 3,
      min_level = 0,
      count = 8,
      image = "assets/towers/tower_tack_barrel_8.png",
      frames = {
        "assets/towers/tower_tack_barrel_8_frame_01.png",
        "assets/towers/tower_tack_barrel_8_frame_02.png",
        "assets/towers/tower_tack_barrel_8_frame_03.png",
      },
    },
    {
      min_path = 3,
      min_level = 2,
      count = 12,
      image = "assets/towers/tower_tack_barrel_12.png",
      frames = {
        "assets/towers/tower_tack_barrel_12_frame_01.png",
        "assets/towers/tower_tack_barrel_12_frame_02.png",
        "assets/towers/tower_tack_barrel_12_frame_03.png",
      },
    },
    {
      min_path = 3,
      min_level = 3,
      count = 16,
      image = "assets/towers/tower_tack_barrel_16.png",
      frames = {
        "assets/towers/tower_tack_barrel_16_frame_01.png",
        "assets/towers/tower_tack_barrel_16_frame_02.png",
        "assets/towers/tower_tack_barrel_16_frame_03.png",
      },
    },
  },
  barrel_animation = { fps = 10.0, loop = true, fire_fps = 20.0, fire_once = true },
  default_angle_deg = 0.0,
  recoil = {
    mode = "scale_pulse",
    scale = 0.9,
    distance = 0.0,
    duration_ms = 55,
    return_ms = 90,
  },
}
```

`recoil.mode = "directional"` 時沿 firing direction 反方向後震，適合 Dart/Bomb/Ice 這類單方向砲塔；`recoil.mode = "scale_pulse"` 時不選單一後退方向，而是把整座塔的 base/barrel 組合先縮小到 `recoil.scale`，再回彈到原本大小，適合 Tack 這類放射針塔，也可作為所有塔的低成本 recoil 實作。

`barrel_variants` 的選擇依 snapshot-backed `upgrade_levels` 進行。以 `tower_tack` 為例，Path 3 等級 0/1 顯示 8 根針，Path 3 等級 2 顯示 12 根針，Path 3 等級 3 以上顯示 16 根針；若之後內容有 32 根針升級，可以再新增 `count = 32` variant，不需要改 omfx 渲染架構。

沒有砲管的範圍傷害塔建議新增 content id `tower_cake_splash`，作為甜點戰爭主題的範圍傷害塔。它不需要 barrel，也不應該朝單一目標旋轉。metadata 範例：

```lua
render = {
  render_mode = "animated_area",
  base = "assets/towers/tower_cake_splash_frame_01.png",
  animation = {
    frames = {
      "assets/towers/tower_cake_splash_frame_01.png",
      "assets/towers/tower_cake_splash_frame_02.png",
      "assets/towers/tower_cake_splash_frame_03.png",
      "assets/towers/tower_cake_splash_frame_04.png",
      "assets/towers/tower_cake_splash_frame_05.png",
      "assets/towers/tower_cake_splash_frame_06.png",
    },
    fps = 10.0,
    loop = true,
    fire_fps = 18.0,
    fire_once = true,
  },
  recoil = {
    mode = "scale_pulse",
    scale = 0.88,
    duration_ms = 70,
    return_ms = 110,
  },
}
```

`animated_area` 模式的 idle animation 可以 loop 播放，攻擊前搖開始時可從 frame 1 重播或加速播放，以表現奶油爆發、果醬濺射或糖霜波動。這類塔的 gameplay AoE 仍由 sim/projectile/outcome 決定，animation 只表達範圍傷害視覺。

Alternatives considered:

- 把圖片路徑硬寫在 omfx：較快但違反「設定在 scripts 資料夾 mod 下」，且新增塔需要改前端。
- 只使用 `td_ui` 塔圖：可重用既有素材，但 UI icon 尺寸與戰鬥 sprite pivot 需求不同，容易讓 UI/戰鬥美術互相牽制。

### Decision: 透過 ABI-safe metadata pipeline 傳遞 render 欄位

新增 ABI-safe 的 tower render metadata 型別，欄位只使用 `abi_stable` 相容型別，例如 `RString`、`Fixed64`、primitive integer。資料流為 `towers.lua` → `omoba-template-ids` generated const → `scripts/base_content` tower metadata → `scripts/script-abi::TowerMetadata` → `omb::TowerTemplateRegistry` → `omfx::TowerTemplateSnapshot`。

為了降低破壞性，所有新增欄位都要有 default/fallback。host 與 script DLL 本來就要求同 rustc/同 ABI crate 重 build，因此這不是外部持久化資料遷移；但仍需避免在 `scripts/script-abi` 引入 `serde_json`、`specs` 或 frontend-only dependency。

Alternatives considered:

- omfx 直接讀 `scripts/lua_data/templates/towers.lua`：會讓 runtime/frontend 執行 Lua 或解析 source data，違反目前 build-time codegen 邊界。
- 用 JSON blob 塞進 ABI：彈性高但增加 ABI crate dependency 與 runtime parsing，不符合 `script-abi` 簡化原則。

### Decision: barrel 朝向使用 snapshot 的 tower aim/facing，不由前端重新找目標

前端 barrel rotation 以 render metadata 的 `rotation_mode` 決定。`targeted` 模式以 render snapshot 的 tower-facing data 為準，優先使用既有 `EntityRenderData.facing_rad`；若目前 tower attack system 沒有在瞄準時更新 tower facing，實作應在 deterministic sim 內更新 tower facing，或新增 tower-only `aim_rad` render 欄位。`fixed` 模式不使用目標方向旋轉 barrel，僅使用 `default_angle_deg` 或 asset 預設朝向。前端不得掃描 creep 並自行選最近目標，避免畫面與權威 target selection 不一致。

當 `targeted` 塔暫時沒有有效目標時，barrel 保持最近一次有效 facing。若 snapshot 沒有任何有效 facing，使用 metadata default angle 或 entity `facing_rad` 的 default。當 `fixed` 塔收到 fire cue 時，仍可用 cue 的 `dir_rad` 表示 projectile/攻擊方向，但 barrel visual SHALL NOT 因此轉向單一目標。當 `fixed` 塔同時宣告 `radial_count_variants`，omfx SHALL 以 upgrade levels 切換 barrel texture 或產生對應數量的 child barrel instances。

Alternatives considered:

- 前端用距離自行選目標：看起來簡單，但 target priority、隱身/陣營、路徑順序與 sim 規則可能不一致。
- 只在開火瞬間旋轉：不符合「砲口會一直對著目標」的需求，平時會看起來卡住。
- 對所有塔強制 target-facing：會讓 `tower_tack` 這種本質是放射針塔的塔看起來錯誤，像是整座針塔只瞄準一個目標。

### Decision: fire recoil 用 render-only queue，與 ExplosionFxQueue 同生命週期

新增 `TowerFireFx` 與 `TowerFireFxQueue`，由 attack/projectile outcome processing 在 tower 實際開火時 push，`extract_snapshot` 每 tick 用 `std::mem::take` drain 到 `SimWorldSnapshot.tower_fire_fx`。欄位最少包含 `entity_id`、`spawn_tick`、`dir_rad`。這個 queue 不進 state hash，render 端只用它啟動短暫 recoil animation。

同一座塔同 tick 產生多發 projectile 時，render 可以合併為一次 recoil，以避免 Tack/多重射擊造成過度震動。若 tower metadata 的 recoil mode 是 `scale_pulse`，omfx 應播放不依賴單一目標方向的整塔縮放 pulse；若是 `directional`，才沿 cue `dir_rad` 的反方向後震。若某些腳本未產生 projectile 但仍算開火，應在 attack hook 或 outcome processing 補一個 fire cue。

Alternatives considered:

- 前端從 projectile visual spawn 推回 tower recoil：會漏掉沒有 projectile visual 的攻擊，也會把 recoil 綁死在前端 projectile 實作。
- 把 recoil 狀態存進 ECS component：可重播但沒有 gameplay 意義，會增加 deterministic state 與 hash 面積。

### Decision: 所有單位共用 attack windup/impact/backswing 三階段

所有會普攻的單位（英雄、召喚物、creep、tower）都使用同一套權威攻擊生命週期：`windup`（攻擊前搖）、`impact`（攻擊瞬間）、`backswing`（攻擊後搖）。後端負責排程這三段時間，`impact` 才是 projectile spawn、damage apply 或命中 outcome 的權威時間點。前端只根據 render cue 播動畫，不提前造成 gameplay 結果。

建議 metadata 使用整數權重，避免浮點誤差：

```lua
attack_timing = {
  windup = 350,
  backswing = 650,
}
```

`windup + backswing` SHALL 等於 `1000`。`impact` 不是一段 duration，而是 `windup` 結束、`backswing` 開始的瞬間事件點。實際攻擊間隔由現有攻速與 buff 聚合得到，例如 effective interval = `asd_interval * AttackSpeedMultiplier`。計算 SHALL 使用整數或 `Fixed64`，不得用浮點直接比較。建議做法是 `windup_duration = effective_interval * windup / 1000`，`backswing_duration = effective_interval - windup_duration`，確保 `windup_duration + backswing_duration == effective_interval`，且兩者都會隨 effective interval 縮短。若某單位沒有宣告 `attack_timing`，使用全域 default 整數權重；content validation SHALL 拒絕權重總和不是 `1000` 的設定。

這要接在目前既有事件語意上：現有 tower/hero attack tick 在 `asd_count >= effective_interval` 時會產生 `Outcome::ProjectileLine2`、`Outcome::ProjectileDirectional`、script `spawn_projectile_ex` 或直接 damage/attack outcomes，並透過 `Outcome::UpdateAttack` / `asd_count` 做冷卻計算。新設計不新增「impact duration」，而是把這些既有 projectile/damage/attack outcome 延後到 authoritative impact event point 執行；`asd_count`/cooldown 仍代表完整攻擊間隔的權威節奏。

後端在 `windup` 開始時推送 render-only `AttackPhaseFx { entity_id, target_entity/target_pos, windup_ms, impact_at_ms, backswing_ms, attack_seq, dir_rad }` 到 snapshot queue，其中 `impact_at_ms == windup_ms`。前端收到 cue 的同一 render 更新就開始播放攻擊動畫，並把 barrel/body animation 的總長度對齊 `windup_ms + backswing_ms`，其中最明顯的 fire/recoil frame 對齊 `impact_at_ms`。`impact` cue 可由同一筆 `AttackPhaseFx` 的 phase offset 表示，不需要額外 optimistic damage。

Alternatives considered:

- 前端看到 projectile 才播攻擊動畫：會太晚，玩家看不到攻擊前搖。
- 前端自行用攻速預測 windup：容易和後端 buff/lockstep tick 不一致。
- 只對 tower 加三段時序：英雄與召喚物仍會缺少攻擊前搖表現，動畫系統無法共用。

### Decision: omfx 使用 per-entity scene node cache 與 texture cache

每個 tower mirror entry 對應一組 render handles：base node、barrel node、radial barrel group 或 animated-area frame node，以及可選 fallback node。建立/刪除只在 tower entity 出現、`removed_entity_ids` 移除、radial barrel count 需要切換，或 animation frame cache 初始化時發生；穩定 frame 只更新 position、rotation、local offset、texture/frame handle 與 recoil transform。texture 透過路徑 cache 載入，animation frames 也應一次載入並快取；缺圖時 fallback 到 `tower_fallback_base.png`、既有 tower UI 圖或純色 placeholder。

Recoil animation 由 render wall-clock 或 snapshot tick aging 驅動皆可，但它不得寫回 sim。建議以 `spawn_tick` 對齊目前 snapshot tick 計算初始時間，再在 render frame 中平滑回彈。`directional` 後震曲線第一版可用簡單 piecewise linear：attack phase 快速退後，return phase 線性或 ease-out 回到 0。`scale_pulse` 第一版可用同一段時間曲線：attack phase 從 `1.0` 縮到 `recoil.scale`，return phase 回到 `1.0`，base 與 barrel 使用同一個 parent/group scale，避免兩張圖分離。

Alternatives considered:

- 每次開火重建 barrel node：會在 1000 塔 stress 場景造成不必要 allocation。
- 把 base/barrel 合成單張 texture：無法獨立旋轉砲口，也讓 recoil 表演受限。

## Risks / Trade-offs

- [Risk] 新增 ABI fields 會要求 scripts workspace 與 host 同步 rebuild → Mitigation：沿用既有雙 cargo build 流程，所有新增欄位提供 default，並更新 gen-docs/catalog 顯示。
- [Risk] barrel pivot/offset 填錯會讓砲口旋轉偏移 → Mitigation：README 明確列出座標系、推薦尺寸與 debug placeholder，第一版資源使用明顯中心線。
- [Risk] tower fire cue 與 projectile spawn timing 不一致 → Mitigation：在 deterministic outcome processing 的同一 tick push `TowerFireFx`，並在測試中檢查 projectile outcome 會產生 fire cue。
- [Risk] 大量 tower 每 frame 更新兩個 sprite 增加 render 成本 → Mitigation：只為 tower 建立兩個長生命節點，texture cache O(1) 重用；保留 collision ring/name label 的節流模式。
- [Risk] 多重射擊塔 recoil 過於頻繁或方向不穩 → Mitigation：同 entity 同 tick 合併 cue，方向使用 primary target/facing，不逐發抖動。
- [Risk] 對針塔套用單一目標旋轉會讓視覺語意錯誤 → Mitigation：新增 `rotation_mode = "fixed"` 與 `recoil.mode = "scale_pulse"`，讓 `tower_tack` 固定不轉向，開火只做整塔縮小再回彈的 pulse。
- [Risk] `tower_tack` 升級後發射針數與視覺砲管數不一致 → Mitigation：新增 `barrel_layout = "radial_count_variants"` 與 upgrade-level-based variant 選擇，至少提供 8/12/16 三種 barrel 圖。
- [Risk] 前端攻擊動畫與後端 impact 不一致 → Mitigation：後端在 windup 開始時推送包含 windup/impact/backswing timing 的 authoritative render cue，前端只對齊 cue 播放，不自行預測攻擊時間。
- [Risk] 攻速極快時前搖/後搖太短看不見 → Mitigation：動畫 frame sampling 允許跳 frame 或播放最接近 impact 的關鍵 frame，但後端不拉長 authoritative interval；`windup + backswing` 永遠等於 effective interval。
- [Risk] animation frame 數量多會增加載入與記憶體成本 → Mitigation：每種 animated tower/barrel 第一版限制少量 PNG，透過 texture cache 一次載入並重用，不在每 frame 讀檔。
- [Risk] 缺少正式美術時戰鬥畫面變空白 → Mitigation：所有塔提供 placeholder base/barrel PNG，loader 缺圖時 log 並 fallback。

## Migration Plan

- 先新增 metadata 型別與 default codegen，不改現有塔行為。
- 在 `scripts/base_content/assets/towers/` 新增預設 base/barrel placeholder 與 README。
- 為現有 `tower_dart`、`tower_tack`、`tower_bomb`、`tower_ice` 補上 `render` metadata，其中 `tower_tack` 使用 `rotation_mode = "fixed"`、`barrel_layout = "radial_count_variants"`、8/12/16 barrel variants 與 `scale_pulse` recoil。
- 新增 barrel animation frames 與無砲管範圍傷害塔 frames，並把 attack animation 起點對齊 windup cue。
- 新增所有單位共用 attack timing metadata 與後端 windup/impact/backswing scheduling。
- 同步新增 `asset-prompts.md`，逐張列出甜點戰爭完整生圖提示詞，讓企劃可用 ChatGPT 重新產生同名 PNG。
- 使用 `combat-tower-layout.svg` 對照實作結果，確認一般砲塔會朝目標旋轉、`tower_tack` 不朝單一目標旋轉且可切換 8/12/16 barrel variants。
- 擴充 `TowerTemplateRegistry` 與 `TowerTemplateSnapshot`，讓 omfx 能取得 render metadata。
- 新增 `TowerFireFxQueue` 並從 projectile/attack outcome 產生 fire cue。
- 在 omfx 實作 composite tower scene cache、barrel rotation、recoil animation 與 fallback。
- 驗證 TD_1 放塔、選塔、升級、賣塔、攻擊、爆炸、stress map 都維持可用。

Rollback 策略：若 composite render 出問題，omfx 可用 feature flag 或 runtime fallback 回單張塔圖/既有 entity render；metadata 與 fire cue 保留不影響 gameplay。

## Open Questions

- barrel sprite 的預設朝向要定義為「圖片向上為 0 rad」或「圖片向右為 0 rad」？建議採用向上，較符合現有 top-down TD 美術。
- `muzzle_offset` 第一版是否要先只保留 metadata，不產生 muzzle flash？建議保留但不強制使用。
- `tower_tack` 的第 4 階若有 32 根針，第一版是否要一併提供 `tower_tack_barrel_32.png`？目前硬性要求 8/12/16，架構保留 32 variant 擴充點。
