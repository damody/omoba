## MODIFIED Requirements

### Requirement: Tower combat assets and render parameters are owned by scripts content mod

每座 tower 的 render mode、base image、barrel image、`render.visual_size`、`placement_radius`、barrel frames、body animation frames、rotation mode、barrel layout、barrel count variants、barrel offset、barrel pivot、muzzle offset 與 recoil 參數 SHALL 由 scripts content mod 明確提供。預設圖片 SHALL 位於 `scripts/base_content/assets/towers/`，預設 metadata SHALL 由 `scripts/lua_data/templates/towers.lua` 的 tower entry 宣告。企劃替換 tower 戰鬥圖片、調整戰鬥 sprite 基準大小或調整放置阻擋半徑時 SHALL 替換 scripts asset 目錄中的 PNG 或更新 scripts Lua metadata，而不是修改 omfx 或 omb source。

長期基準大小 SHALL NOT 由 Rust 程式使用 `footprint`、global scale、clamp、multiplier、`render.visual_size / 2` 或其他推導規則產生。缺少 `render.visual_size` 或 `placement_radius` 的 tower metadata SHALL 在 content/codegen validation 階段失敗，而不是 runtime fallback。

#### Scenario: base and barrel assets exist for shipped towers

- **WHEN** 檢查 `scripts/base_content/assets/towers/`
- **THEN** 目錄包含 `tower_dart_base.png` 與 `tower_dart_barrel.png`
- **AND** 目錄包含 `tower_tack_base.png`，且 tack barrel 使用 count variant 圖而不是單一固定檔名
- **AND** 目錄包含 `tower_bomb_base.png` 與 `tower_bomb_barrel.png`
- **AND** 目錄包含 `tower_ice_base.png` 與 `tower_ice_barrel.png`

#### Scenario: animated area tower frames exist

- **WHEN** 檢查 `scripts/base_content/assets/towers/`
- **THEN** 目錄包含至少 6 張 `tower_cake_splash_frame_*.png` body animation frames
- **AND** 每張 frame 都是非空 PNG with alpha

#### Scenario: barrel animation frames can exist for shipped towers

- **WHEN** 檢查 `scripts/base_content/assets/towers/`
- **THEN** shipped tower barrel animation frames can be represented by ordered files such as `tower_dart_barrel_frame_01.png`
- **AND** 如果 metadata 宣告這些 frames，每個檔案都 SHALL 存在且可載入

#### Scenario: tack barrel variants exist

- **WHEN** 檢查 `scripts/base_content/assets/towers/`
- **THEN** 目錄包含 `tower_tack_barrel_8.png`
- **AND** 目錄包含 `tower_tack_barrel_12.png`
- **AND** 目錄包含 `tower_tack_barrel_16.png`
- **AND** 每張 variant 圖都清楚呈現對應數量的 radial needle holes 或 barrels

#### Scenario: tower template declares render metadata and sizing

- **WHEN** `scripts/lua_data/templates/towers.lua` 中的 `tower_dart` 被 codegen 讀取
- **THEN** generated tower metadata 包含 base image path
- **AND** generated tower metadata 包含 barrel image path
- **AND** generated tower metadata 包含 `render.visual_size > 0`
- **AND** generated tower metadata 包含 `placement_radius > 0`
- **AND** generated tower metadata 包含 rotation mode、barrel layout、barrel offset、barrel pivot 與 recoil parameters

#### Scenario: tack shooter declares fixed rotation mode

- **WHEN** `scripts/lua_data/templates/towers.lua` 中的 `tower_tack` 被 codegen 讀取
- **THEN** generated tower metadata contains `rotation_mode = "fixed"`
- **AND** generated tower metadata contains `barrel_layout = "radial_count_variants"`
- **AND** generated tower metadata contains 8、12、16 count variants with image paths
- **AND** generated tower metadata contains `scale_pulse` recoil settings suitable for a non-target-facing needle tower

#### Scenario: missing sizing metadata fails validation

- **WHEN** content declares a tower without `render.visual_size` or without `placement_radius`
- **THEN** codegen or content validation fails
- **AND** omfx and omb do not receive a generated tower template with inferred sizing values

### Requirement: Tower visual size uses script-owned base size only

omfx SHALL use the script-owned `render.visual_size` as the long-lived base size for tower base, barrel, and animated-area body nodes. omfx SHALL only multiply that value by `WORLD_SCALE` to convert backend world units to render units. Temporary visual effects such as recoil, attack animation, hover effects, or buff visuals MAY apply short-lived transform scale on top of that base size, but SHALL NOT replace the script-owned base size with persistent frontend constants or formulas.

#### Scenario: omfx renders tower using script visual size

- **WHEN** `tower_dart` metadata contains `render.visual_size = 180.0`
- **THEN** omfx renders the tower composite base size from `180.0 * WORLD_SCALE`
- **AND** omfx does not apply a persistent `TD_TOWER_VISUAL_SCALE`, footprint multiplier, or clamp to compute the base size

#### Scenario: recoil scale is temporary

- **WHEN** tower recoil starts and metadata contains `recoil.scale = 0.9`
- **THEN** omfx temporarily scales the already-sized tower visual during recoil
- **AND** after recoil finishes, the tower returns to its script-owned base size

### Requirement: Tower placement radius uses script-owned placement radius only

Tower placement validation SHALL use the script-owned `placement_radius` metadata as the authoritative placement blocker radius. Both omb final placement validation and omfx local placement preview SHALL use this value. Placement validation SHALL NOT infer radius from `render.visual_size / 2`, `footprint`, runtime `CollisionRadius`, attack range, image dimensions, or frontend render transforms.

#### Scenario: backend placement uses explicit placement radius

- **WHEN** `tower_bomb` metadata contains `placement_radius = 96.0`
- **THEN** omb validates path, blocked region, and tower-overlap placement using radius `96.0`
- **AND** omb does not derive placement radius from `render.visual_size / 2` or `footprint`

#### Scenario: frontend preview matches backend placement radius

- **WHEN** omfx previews placement for a selected tower
- **THEN** the preview footprint circle uses the same `placement_radius` value exposed through tower template snapshot metadata
- **AND** omfx local can-place checks agree with backend placement validation for path, region, and existing tower overlap under the same metadata

#### Scenario: runtime gameplay collision remains unchanged

- **WHEN** a tower is successfully placed
- **THEN** changing `placement_radius` does not change the tower runtime `CollisionRadius`, attack range, projectile spawn position, damage, cooldown, or lockstep combat hash
