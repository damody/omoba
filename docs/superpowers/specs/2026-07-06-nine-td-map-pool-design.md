# Nine TD Map Pool Design

## Goal

Add a 9-map tower-defense map pool for the pregame map selection flow. The pool is organized as three maps per difficulty tier:

- 初級: teaches core tower placement, range coverage, and slow/AOE basics.
- 中級: adds split entrances, constrained tower placement, blocked regions, and wave pressure.
- 高級: adds multi-lane pressure, special enemy rules, and low-error timing while keeping the same playable footprint.

All maps must fit inside the current 綠野路口 footprint.

## Hard Bounds

綠野路口 (`TD_1`) is the size reference. Its current checkpoints use this outer coordinate range:

- `x`: `-1400.0..=1400.0`
- `y`: `-800.0..=800.0`

Every new map variant must keep all route checkpoints, spawn/base points, tower placement markers, decorative gameplay regions, and blocked regions inside that same rectangle. Advanced maps must get harder through route topology, tower scarcity, wave timing, and enemy mix, not through a larger map.

## Map Pool

### 初級

1. **綠野路口** (`td_green_crossroads`)
   - Role: baseline tutorial and sizing reference.
   - Route: existing zigzag route from left-top to left-bottom through wide turns.
   - Mechanics: no special map rule.
   - Player lesson: place towers near turns, understand range overlap, start waves.

2. **河畔小徑** (`td_riverside_path`)
   - Route: long S curve inside the same rectangle.
   - Mechanics: central high-value tower area with clear visual emphasis.
   - Player lesson: long-range towers gain value when they cover multiple route segments.

3. **農莊彎道** (`td_farmstead_bends`)
   - Route: two large bends and a clear final exit.
   - Mechanics: occasional fast creeps in otherwise forgiving waves.
   - Player lesson: slow towers and AOE towers solve different problems.

### 中級

4. **雙門哨站** (`td_twin_gate_outpost`)
   - Route: two entrances merge into one mid-map route.
   - Mechanics: alternating waves pressure different entrances.
   - Player lesson: split early firepower, then concentrate upgrades near the merge.

5. **潮汐港灣** (`td_tidal_harbor`)
   - Route: dock-like bends around water-shaped blocked areas.
   - Mechanics: every tenth wave includes shielded creeps.
   - Player lesson: constrained tower placement rewards pre-planned coverage.

6. **礦坑迴廊** (`td_mine_corridor`)
   - Route: short path that revisits a central fire zone.
   - Mechanics: rock blocked regions restrict some tower lines.
   - Player lesson: burst timing and central upgrades matter more than raw path length.

### 高級

7. **熔火岔道** (`td_molten_fork`)
   - Route: three branches merge late.
   - Mechanics: lava zones periodically speed creeps up.
   - Player lesson: distribute tower types and use hero movement to cover timing gaps.

8. **暮色迷宮** (`td_twilight_maze`)
   - Route: long internal maze with scarce tower positions.
   - Mechanics: stealth creeps require detection from a tower type or hero ability.
   - Player lesson: detection, upgrade order, and scarce build slots define success.

9. **冰封斷橋** (`td_frozen_broken_bridge`)
   - Route: three short lanes crossing the same fixed footprint.
   - Mechanics: ice slows ordinary creeps, but elite creeps ignore it and take shortcuts.
   - Player lesson: low leak tolerance, lane prioritization, and burst upgrades.

## Content Architecture

Each map should be implemented as a separate Lua story/map variant under `scripts/lua_data/`. The first implementation can reuse the existing TD tower, creep, hero, and wave templates, then override only map topology and wave composition.

Recommended story ids:

- `TD_GREEN_CROSSROADS`
- `TD_RIVERSIDE_PATH`
- `TD_FARMSTEAD_BENDS`
- `TD_TWIN_GATE_OUTPOST`
- `TD_TIDAL_HARBOR`
- `TD_MINE_CORRIDOR`
- `TD_MOLTEN_FORK`
- `TD_TWILIGHT_MAZE`
- `TD_FROZEN_BRIDGE`

The pregame catalog should expose exactly these nine maps. Difficulty selection remains separate; the map cards can be grouped or filtered by selected difficulty so each tier shows its three intended maps.

## Bounds Validation

Add a test or catalog validation helper that scans each TD map definition and fails if any gameplay coordinate is outside the 綠野路口 bounds:

- Check `CheckPoint.X/Y`.
- Check route spawn/base points through their checkpoints.
- Check blocked region vertices if polygon/region data is present.
- Check future tower marker or placement zone coordinates if those are added.

The validation error should include the map id, object name, offending coordinate, and allowed range.

## UI Expectations

The map-selection screen should show the selected difficulty tier's three maps. If the current UI still renders up to six map cards, it can show only the three relevant maps plus a back button. Card copy should include:

- Chinese display name.
- One-line route identity.
- One-line mechanic or warning.
- Optional reward text.

No map card should imply a larger battlefield than 綠野路口.

## Testing

Automated checks:

- Pregame catalog loads all nine map entries.
- Each difficulty tier resolves to exactly three playable maps.
- Bounds validation passes for all nine maps.
- Bounds validation fails on a synthetic out-of-range checkpoint.

Manual smoke:

- Start each map from the pregame flow.
- Confirm camera framing and UI do not need larger-than-綠野路口 assumptions.
- Confirm creeps spawn, follow the full route, reach base, and interact with towers.

## Implementation Decisions

- Difficulty selection filters the map list. After choosing 初級, 中級, or 高級, the map-selection screen shows only that tier's three maps.
- The first pass creates all nine base routes, catalog entries, and bounds tests. Special mechanics such as stealth, shield, lava speed, and elite shortcut can be staged after the route pool is playable, but the map descriptions should keep those mechanics as the intended final identity.
