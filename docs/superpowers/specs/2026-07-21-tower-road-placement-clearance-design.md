# Tower Road Placement Clearance Design

## Goal

Make tower placement near TD roads match the visible road boundary more closely without weakening tower-to-tower spacing.

## Current behavior

The rendered road has a backend half-width of `64`. Both the frontend preview and backend authority reject a tower when its center is closer than:

```text
road half-width + tower placement_radius
```

Tower `placement_radius` values are currently `80` to `96`, so the effective road exclusion half-width is `144` to `160`. This creates a large invisible margin beyond the visible road.

## Selected design

Use the tower runtime `footprint` for road clearance, while preserving `placement_radius` for tower-to-tower and blocked-region checks.

```text
road clearance = road half-width 64 + tower footprint
tower overlap = new tower placement_radius + existing tower placement_radius
blocked region = tower placement_radius intersects polygon
```

Current tower footprints are `10` to `12.5`, producing an effective road exclusion half-width of approximately `74` to `76.5`.

## Scope

- Apply the rule globally to all TD maps.
- Change both the omfx local placement preview and omoba-core authoritative placement validation.
- Keep the road rendering width unchanged.
- Keep Lua tower metadata unchanged.
- Keep tower-to-tower spacing and blocked-region behavior unchanged.

## Consistency requirement

Frontend and backend must calculate road clearance from the same semantic inputs:

- frontend: `tpl.footprint_backend + TD_PATH_HALF_WIDTH_BACKEND`
- backend: `tpl.footprint + PATH_HALF_WIDTH`

The backend remains authoritative if a client submits an invalid placement.

## Tests

- A position outside the visible road plus footprint margin is accepted.
- A position inside the road plus footprint margin is rejected as too close to the road.
- Tower-to-tower overlap still uses `placement_radius`.
- Frontend and backend tests use the same representative tower metadata and boundary distances.
- Existing Twin Gate green-gap regression remains passing.

## Non-goals

- Changing road geometry or visual width.
- Changing tower sprites, attack range, combat collision, or Lua metadata.
- Adding per-map road widths or placement overrides.
