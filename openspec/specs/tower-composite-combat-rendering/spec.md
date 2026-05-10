## Purpose

定義 TD tower 戰鬥畫面中的 composite base/barrel/body rendering、tower attack animation、recoil、asset ownership 與 fallback 行為。

## Requirements

### Requirement: Tower combat visuals use base and barrel sprites

TD 戰鬥畫面中的每座 tower SHALL 以至少兩個可替換 sprite 組成：base sprite 與 barrel sprite。base sprite SHALL 表示塔底座或塔身主體；barrel sprite SHALL 表示可旋轉的砲口、發射器、弓臂或等效攻擊方向部件。兩個 sprite SHALL 使用同一個 tower entity 的世界座標作為共同 anchor，並可套用各自的 local offset、pivot 與 z-order。

#### Scenario: tower creates composite render nodes

- **WHEN** TD_1 中放置一座 `tower_dart`
- **THEN** omfx 為該 tower entity 建立 base render node
- **AND** omfx 為同一個 tower entity 建立 barrel render node
- **AND** 兩個 node 都跟隨該 tower 的 snapshot-backed position

#### Scenario: tower removal releases composite nodes

- **WHEN** snapshot 的 `removed_entity_ids` 包含某座 tower entity id
- **THEN** omfx 移除或回收該 tower 的 base render node
- **AND** omfx 移除或回收該 tower 的 barrel render node
- **AND** 該 tower 不會留下 stale base/barrel sprite

### Requirement: Tower barrels support frame animation sequences

Every tower barrel SHALL support an optional ordered frame sequence. When barrel frame paths are configured, omfx SHALL animate the barrel using those frames. When no frame sequence is configured, omfx SHALL fall back to the single barrel image. Barrel animation SHALL be able to start at attack windup and align its fire/recoil frame with attack impact timing.

#### Scenario: target-facing barrel plays animation frames

- **WHEN** `tower_dart` has `barrel_frames` configured and receives an attack windup cue
- **THEN** omfx starts playing the configured barrel frames during windup
- **AND** the frame sequence remains rotated toward the authoritative target-facing direction
- **AND** the fire frame or recoil key moment aligns with attack impact timing

#### Scenario: missing barrel frames fall back to single image

- **WHEN** a tower declares a barrel image but no barrel frame sequence
- **THEN** omfx renders the single barrel image
- **AND** attack recoil and rotation still work

### Requirement: Tower barrel rotation follows tower rotation mode

barrel sprite SHALL 依 scripts metadata 的 rotation mode 決定是否旋轉。一般 target-facing tower SHALL 使用 authoritative render data 旋轉，使砲口朝向目前攻擊目標或最近一次有效攻擊方向。`tower_tack` 這類放射針塔 SHALL 可設定為 fixed rotation mode，使 barrel visual 不跟單一目標旋轉。omfx SHALL NOT 自行掃描 creep 並重新選擇 tower target。當 target-facing tower 暫時沒有有效目標時，barrel SHALL 保持最近一次有效方向或使用 snapshot/default facing。

#### Scenario: barrel faces current target

- **WHEN** tower 正在攻擊一個位於其右上方的 creep，且 snapshot 回報 tower aim/facing 指向右上方
- **THEN** 該 tower 的 barrel sprite 旋轉到右上方
- **AND** base sprite 不因 barrel aiming 而旋轉到同一方向，除非 metadata 明確要求 base 跟隨旋轉

#### Scenario: tack shooter barrel does not rotate toward one target

- **WHEN** `tower_tack` 的 render metadata 設定 `rotation_mode = "fixed"`，且它正在對多方向發射針
- **THEN** `tower_tack` 的 barrel sprite 維持 metadata default angle 或 asset default orientation
- **AND** barrel sprite 不會因 primary target 位於右上方就旋轉到右上方
- **AND** projectile 或 hit logic 仍依 sim 規則往正確方向發射或命中

### Requirement: Radial tack tower barrel count follows upgrade state

`tower_tack` SHALL support radial barrel count variants that change according to snapshot-backed tower upgrade levels. The visual barrel or needle-hole count SHALL match the simultaneous needle count represented by the tower upgrade state, with at least 8, 12, and 16 count states. This SHALL be driven by scripts metadata and snapshot `upgrade_levels`; omfx SHALL NOT hard-code `tower_tack` upgrade names or behavior flags in frontend-only logic.

#### Scenario: unupgraded tack shooter shows 8 barrels

- **WHEN** `tower_tack` has upgrade levels `[0, 0, 0]` or an equivalent state before the more-needles upgrade
- **THEN** omfx renders the tack barrel visual with 8 radial needle holes or 8 radial barrel instances
- **AND** the barrel group remains fixed rather than rotating toward a single target

#### Scenario: upgraded tack shooter shows 12 barrels

- **WHEN** `tower_tack` upgrade levels indicate the 12-needle state, such as path 3 level 2 in the shipped content
- **THEN** omfx renders the tack barrel visual with 12 radial needle holes or 12 radial barrel instances
- **AND** the texture or child-node count changes without changing gameplay attack logic

#### Scenario: upgraded tack shooter shows 16 barrels

- **WHEN** `tower_tack` upgrade levels indicate the 16-needle state, such as path 3 level 3 in the shipped content
- **THEN** omfx renders the tack barrel visual with 16 radial needle holes or 16 radial barrel instances
- **AND** recoil still uses `scale_pulse` instead of selecting a single target direction

#### Scenario: tack count variants can animate

- **WHEN** `tower_tack` uses a 12-count barrel variant with configured frames
- **THEN** omfx plays the 12-count frame sequence during attack windup
- **AND** the selected frame sequence still represents 12 radial barrels or needle holes

#### Scenario: frontend does not choose a different target

- **WHEN** tower 射程內有多個 creep，且 sim 的 target priority 選中其中一個 creep
- **THEN** barrel sprite 使用 snapshot/render cue 對應的 sim 目標方向
- **AND** omfx 不使用最近距離或螢幕位置自行改選另一個 target

### Requirement: Tower fire triggers configurable recoil animation

當 tower 實際開火時，base sprite 與 barrel sprite SHALL 同步播放 recoil animation。target-facing tower 的 recoil SHALL 支援沿 barrel 目前方向的反方向位移，先後震再回到原位。tower recoil SHALL 也支援 `scale_pulse` 模式，使整座塔的 base/barrel 組合先縮小再回彈放大。recoil distance、scale、attack duration、return duration 與 recoil mode SHALL 可由 scripts metadata 自定義；缺值時 SHALL 使用安全 default。recoil SHALL 是 render-only 表演，MUST NOT 改變 tower gameplay position、attack range、projectile spawn position、命中判定或 cooldown。

#### Scenario: firing tower recoils backward

- **WHEN** `tower_bomb` 朝右方開火，且收到 fire render cue
- **THEN** base sprite 與 barrel sprite 在發射瞬間沿左方短暫位移
- **AND** recoil animation 結束後兩個 sprite 回到原本 local offset

#### Scenario: recoil parameters come from scripts metadata

- **WHEN** `tower_bomb` 的 render metadata 設定 `recoil.distance = 12.0` 且 `tower_dart` 設定 `recoil.distance = 6.0`
- **THEN** `tower_bomb` 的後震距離大於 `tower_dart`
- **AND** 兩者都不影響各自的 gameplay world position

#### Scenario: multiple projectiles in the same tick use one recoil pulse

- **WHEN** 同一座 tower 在同一個 sim tick 產生多個 projectile outcome
- **THEN** omfx 對該 tower 啟動一次 recoil pulse
- **AND** recoil 不會因同 tick 多發 projectile 疊加到超出 metadata 設定的最大後震距離

#### Scenario: tack shooter uses scale pulse recoil

- **WHEN** `tower_tack` 設定 `recoil.mode = "scale_pulse"` 並在同一 tick 往多方向發射針
- **THEN** base sprite 與 barrel sprite 作為同一組合先縮小再回彈放大
- **AND** recoil 不會選擇其中一個 target direction 造成整座針塔往單一反方向偏移
- **AND** recoil 結束後整座塔回到原本大小與 local offset

### Requirement: No-barrel area damage towers use body frame animation

Tower render metadata SHALL support a no-barrel animated-area archetype for area damage towers. Such towers SHALL render from an ordered body frame sequence and SHALL NOT require a barrel sprite, target-facing rotation, or barrel recoil. Their attack animation SHALL start during attack windup and use the configured frame sequence to express the area damage burst.

#### Scenario: animated area tower renders without barrel

- **WHEN** a tower template has `render_mode = "animated_area"`
- **THEN** omfx renders the tower using its ordered body animation frames
- **AND** omfx does not require a barrel node for that tower
- **AND** omfx does not rotate that tower toward a single target

#### Scenario: animated area attack starts in windup

- **WHEN** an animated-area tower receives an attack windup cue
- **THEN** omfx starts or restarts the body frame animation during windup
- **AND** the strongest burst frame aligns with attack impact timing

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

### Requirement: Every combat tower asset has a complete dessert-war prompt

The change SHALL include a human-readable prompt catalog for generating every combat tower PNG in the dessert-war theme. The prompt catalog SHALL include complete prompts for all current tower base/barrel assets and tack count variants, and SHALL tell content authors to replace files in `scripts/base_content/assets/towers/`.

#### Scenario: asset prompts cover current combat tower images

- **WHEN** 企劃打開 `openspec/changes/split-tower-base-barrel-rendering/asset-prompts.md`
- **THEN** 文件包含 `tower_dart_base.png` 與 `tower_dart_barrel.png` 的完整提示詞
- **AND** 文件包含 `tower_bomb_base.png` 與 `tower_bomb_barrel.png` 的完整提示詞
- **AND** 文件包含 `tower_ice_base.png` 與 `tower_ice_barrel.png` 的完整提示詞
- **AND** 文件包含 `tower_tack_base.png`、`tower_tack_barrel_8.png`、`tower_tack_barrel_12.png` 與 `tower_tack_barrel_16.png` 的完整提示詞
- **AND** 文件包含 barrel animation frame 與 `tower_cake_splash_frame_*.png` 的完整提示詞或 frame-by-frame prompt pattern

#### Scenario: prompt catalog uses dessert-war constraints

- **WHEN** 任一 combat tower 圖片提示詞被讀取
- **THEN** 提示詞包含甜點戰爭主題要求
- **AND** 提示詞要求 transparent PNG with alpha
- **AND** 提示詞禁止文字、浮水印、商標與既有遊戲素材複製

#### Scenario: asset README documents replacement contract

- **WHEN** 企劃打開 `scripts/base_content/assets/towers/README.md`
- **THEN** README 說明 base/barrel PNG 的命名規則與用途
- **AND** README 說明 `rotation_mode`、`barrel_pivot`、`barrel_offset`、`muzzle_offset` 與 recoil 參數的座標系
- **AND** README 說明替換圖片需保留 PNG alpha 與檔名

### Requirement: Missing composite assets fall back without gameplay impact

若 tower composite render metadata 缺漏、圖片不存在或圖片解碼失敗，omfx SHALL 使用 fallback texture 或既有 tower visual，並保留 tower 可見、可選取與可攻擊。缺圖 SHALL log 可診斷訊息，但 SHALL NOT panic，也 SHALL NOT 阻止 tower placement、selection、upgrade、sell 或 combat。

#### Scenario: missing barrel image falls back

- **WHEN** `tower_ice_barrel.png` 不存在或載入失敗
- **THEN** omfx 使用 barrel fallback texture 或單張 tower fallback visual
- **AND** `tower_ice` 仍在戰鬥畫面可見
- **AND** 選塔、升級、出售與攻擊流程仍可用

#### Scenario: missing recoil metadata uses defaults

- **WHEN** 某座 tower 沒有宣告 recoil metadata
- **THEN** omfx 使用 default recoil distance 與 duration
- **AND** 開火 recoil 表演仍可播放或安全停用
- **AND** gameplay 結果不受影響

### Requirement: Composite tower rendering avoids per-frame allocation churn

omfx SHALL 為每個 tower entity 重用 base/barrel render handles 與 texture cache。穩定 frame 中，omfx SHALL NOT 每 frame 建立或刪除 tower composite nodes，也 SHALL NOT 每 frame 從磁碟重新載入相同 tower texture。

#### Scenario: stable tower count reuses nodes

- **WHEN** TD_STRESS 類場景中 1000 座 tower 已存在且沒有 tower 新增或移除
- **THEN** omfx 不會每 frame 建立新的 base/barrel nodes
- **AND** 既有 tower nodes 只更新 position、rotation、local offset 或 recoil transform

#### Scenario: texture cache reuses loaded assets

- **WHEN** 多座 `tower_dart` 同時存在
- **THEN** omfx 對 `tower_dart_base.png` 使用已快取 texture
- **AND** omfx 對 `tower_dart_barrel.png` 使用已快取 texture
- **AND** 不會為每座同種 tower 重複讀檔與解碼相同 PNG
