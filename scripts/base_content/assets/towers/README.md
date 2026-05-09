# Tower Combat Assets - Dessert War

This directory is the canonical source for TD combat tower sprites owned by `scripts/base_content`. Content authors can replace any PNG here without changing Rust or omfx code, as long as the filename and alpha channel contract are preserved.

## Replacement Rules

- Keep the exact filename and PNG format.
- Preserve alpha transparency. The area outside the sprite must be transparent.
- Keep each tower sequence the same canvas size, center point, and viewing angle.
- Do not include text, numbers, logos, watermarks, or copied game assets inside the image.
- Use the dessert-war theme: candy, cookie, cake, frosting, chocolate, jam, ice cream, toy-like weapons, thick readable outlines.
- Base and barrel images should normally use the same canvas size so local offsets and pivots stay stable after replacement.

## Naming Contract

| Pattern | Use |
|---|---|
| `tower_<id>_base.png` | Non-rotating base/body support for `render_mode = "base_barrel"`. |
| `tower_<id>_barrel.png` | Single target-facing barrel/top image, default pointing up. |
| `tower_<id>_barrel_frame_01.png` | Optional ordered barrel animation frames. Frame numbers must be zero-padded and contiguous. |
| `tower_tack_barrel_<count>.png` | Fixed radial tack barrel variant for a specific simultaneous needle/barrel count. |
| `tower_tack_barrel_<count>_frame_01.png` | Optional ordered tack variant animation frames. |
| `tower_<id>_frame_01.png` | Body frames for `render_mode = "animated_area"` towers that do not use a barrel node. |
| `tower_fallback_base.png` | Optional fallback base used by loaders when specific assets are missing. |
| `tower_fallback_barrel.png` | Optional fallback barrel used by loaders when specific assets are missing. |

Current shipped placeholders include dart, bomb, ice, tack 8/12/16 radial variants, and cake splash body frames.

## Render Metadata

Tower render metadata is declared in `scripts/lua_data/templates/towers.lua` and passed through generated metadata to omfx.

### `render_mode`

- `base_barrel`: create a base sprite plus a barrel sprite or barrel animation sequence.
- `animated_area`: create one body animation node from ordered frames; no barrel node and no target-facing rotation is required.

Default: `base_barrel`.

### `rotation_mode`

- `targeted`: barrel rotation follows authoritative snapshot facing/aim data. Target-facing barrel images should point upward at zero/default angle.
- `fixed`: barrel visual stays at metadata/default rotation and does not turn toward one target. Use this for radial towers such as `tower_tack`.

Default: `targeted`.

### `barrel_layout`

- `single`: one barrel image or one ordered barrel frame sequence.
- `radial_count_variants`: select a variant image/frame sequence based on snapshot-backed upgrade levels. Tack uses 8, 12, and 16 variants.

Default: `single`.

## Coordinates

All render offsets are visual-only and do not affect gameplay position, range, projectile spawn, hit detection, or cooldown.

- Tower world position is the shared anchor for base and barrel.
- `barrel_offset = { x, y }` is a local pixel/render-unit offset from the tower anchor to the barrel anchor. Positive `x` moves right; positive `y` moves down on the sprite canvas convention used by top-down art.
- `barrel_pivot = { x, y }` is normalized texture space. `{ x = 0.5, y = 0.5 }` is center; `{ x = 0.5, y = 0.65 }` pivots around a point below center.
- `muzzle_offset = { x, y }` is a local offset from the barrel pivot toward the muzzle. It is reserved for recoil alignment and future muzzle flash placement.
- `default_angle_deg` is the fixed/default visual angle when no authoritative aim exists or when `rotation_mode = "fixed"`.

## Recoil

Recoil is render-only.

- `recoil.mode = "directional"`: move the base/barrel group backward opposite the firing direction, then return.
- `recoil.mode = "scale_pulse"`: scale the whole tower group down to `recoil.scale`, then pop back to normal. Use this for tack and no-barrel area towers.
- `recoil.distance`: maximum visual backward distance for directional recoil.
- `recoil.scale`: smallest visual scale during scale pulse.
- `recoil.duration_ms`: time to reach the maximum recoil pose.
- `recoil.return_ms`: time to return to the idle pose.

## Attack Phase Timing

Attack timing metadata belongs to content and is authoritative on the backend.

- `attack_timing.windup + attack_timing.backswing` must equal `1000`.
- `windup` starts the frontend attack animation cue.
- `impact` is the instant at the boundary after windup; it is not a duration.
- Projectile spawn, damage, hit, fire frame, and recoil should align to impact.
- `backswing` is the recovery after impact.

Example:

```lua
attack_timing = {
  windup = 350,
  backswing = 650,
}
```

## Authoring Flow

1. Generate or draw the replacement PNG using `openspec/changes/split-tower-base-barrel-rendering/asset-prompts.md`.
2. Save it over the same filename in this directory.
3. Keep alpha transparency and the same canvas alignment for all frames in the same sequence.
4. If adding a new file or sequence, add matching metadata in `scripts/lua_data/templates/towers.lua`.
5. Run the normal two-workspace build flow so generated tower metadata, script DLL, backend, and omfx agree.
