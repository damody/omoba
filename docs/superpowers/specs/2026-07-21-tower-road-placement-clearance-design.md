# Tower Road Placement Clearance Design

## Goal

Make tower placement near TD roads match the visible road boundary more closely without weakening tower-to-tower spacing.

## Current behavior

The rendered road has a backend half-width of `64`. Both the frontend preview and backend authority reject a tower when its center is closer than:

```text
road half-width + tower placement_radius
```

Tower `placement_radius` values are currently `80` to `96`, so the effective road exclusion half-width is `144` to `160`. This creates a large invisible margin beyond the visible road.

## Corrected design after runtime coordinate diagnostics

Use `placement_radius` for road clearance, tower-to-tower spacing, and blocked-region checks. Runtime diagnostics showed that `footprint` is only the small combat collision radius and does not represent the rendered tower body.

```text
road clearance = road half-width 64 + tower placement_radius
tower overlap = new tower placement_radius + existing tower placement_radius
blocked region = tower placement_radius intersects polygon
```

The apparent excessive clearance was caused by frontend path collision coordinates being mirrored on X twice. Mouse picking had already converted scene coordinates back to logical coordinates, but cached path and blocked-region points were still scene-mirrored. Keeping collision data in backend logical coordinates makes frontend and backend distances match.

## Scope

- Apply the rule globally to all TD maps.
- Keep both omfx local placement preview and omoba-core authoritative placement validation on `placement_radius`.
- Store frontend path and blocked-region collision points in logical, non-scene-mirrored coordinates.
- Keep the road rendering width unchanged.
- Keep Lua tower metadata unchanged.
- Keep tower-to-tower spacing and blocked-region behavior unchanged.

## Consistency requirement

Frontend and backend must calculate road clearance from the same semantic inputs:

- frontend: `tpl.placement_radius_backend + TD_PATH_HALF_WIDTH_BACKEND`
- backend: `tpl.placement_radius + PATH_HALF_WIDTH`

The backend remains authoritative if a client submits an invalid placement.

## Tests

- A position outside the visible road plus tower placement radius is accepted.
- A position whose rendered tower body overlaps the road is rejected as too close to the road.
- A reported Twin Gate point produces the same nearest-road distance in frontend and backend coordinates.
- Tower-to-tower overlap still uses `placement_radius`.
- Frontend and backend tests use the same representative tower metadata and boundary distances.
- Existing Twin Gate green-gap regression remains passing.

## Non-goals

- Changing road geometry or visual width.
- Changing tower sprites, attack range, combat collision, or Lua metadata.
- Adding per-map road widths or placement overrides.
