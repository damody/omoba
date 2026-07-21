# Lua Spawn Path Selector Design

## Goal

Allow a tower-defense map to choose the entrance for every generated balloon with a Lua function. Fix Twin Gate Outpost so it has three left-side entrances, one shared right-side base, and an even per-wave distribution authored in Lua. Also make rejected tower placement explain its reason and preserve the buildable green space shown in the bug report.

## Current Problems

TD difficulty rounds are generated in Rust by `btd_round_waves`. That function receives only the first map path and assigns every balloon to it. Consequently:

- map-authored multi-path wave details are ignored in TD mode;
- every generated round uses one entrance;
- a map cannot select an entrance from the round number or balloon properties.

Twin Gate Outpost also describes only two spawn checkpoints. Its third left endpoint is classified as the base, producing a route that travels to the right and then returns to the upper-left endpoint. This does not match the intended three-entrance layout.

Tower placement currently returns a boolean. Insufficient gold, road clearance, blocked regions, and tower overlap all produce the same red preview, so a player cannot tell why an apparently clear position was rejected.

## Lua API

A map may define `SelectSpawnPath` in the table returned by `map.lua`:

```lua
return function(ctx)
  return {
    GameMode = "TowerDefense",

    Path = {
      -- Path index 1, 2, 3, ...
    },

    SelectSpawnPath = function(round_index, balloon_index, balloon)
      return ((balloon_index - 1) % 3) + 1
    end,
  }
end
```

The callback contract is:

- `round_index` is the one-based effective TD round number.
- `balloon_index` is the one-based index in that round after expanding grouped round descriptions.
- `balloon` is a new table with `id`, `label`, `base`, `camo`, `regrow`, and `fortified` fields.
- The return value is an integer in `1..#Path`, using the declaration order of the map's `Path` array.
- The callback is an authoring-time pure function. It must not depend on random numbers, clocks, I/O, mutable external state, or call order beyond the three arguments.

An invalid return value fails content loading. The error names the story, round, balloon index, returned value, and valid path range. A callback error retains its Lua source location and adds the same story/round/balloon context.

`SelectSpawnPath` is optional for compatibility. Without it, all balloons use path index 1, which preserves current single-path maps and the existing behavior of unconverted multi-path maps.

## Compilation Boundary

Lua functions cannot be represented by the existing `StoryValue`/JSON map model. The loader will therefore treat `SelectSpawnPath` as a map compilation hook rather than serialized runtime data:

1. Load the map builder output as a Lua table.
2. Remove and retain the optional `SelectSpawnPath` function before JSON conversion.
3. Expand the canonical BTD round catalog into individual balloon descriptors.
4. Call the selector once for every balloon and validate its result.
5. Store the selected one-based path indices in ordinary generated map data, grouped by round.
6. Convert the remaining map table to JSON and then to `CreepWaveData` as today.

The canonical BTD round descriptions and parser must have one owner accessible to `omoba-template-ids`, because content compilation now needs the same expanded balloons that runtime wave construction consumes. `omoba-core` will consume the resulting canonical round data rather than keep a second copy.

Shipped content computes selections in `omoba-template-ids/build.rs` and embeds them in generated story data. Runtime Lua-content loading and development reload use the same loader path and recompute the same generated selections. The selector is never stored in ECS state and is never invoked from a simulation tick.

This boundary keeps lockstep, snapshots, and replays independent of a Lua VM. The generated path-selection arrays also participate in the existing generated/runtime content shape and hash checks.

## Wave Construction

`CreepWaveData` gains optional generated path selections grouped by round. TD wave construction will:

1. Expand the canonical round into its original globally ordered balloon sequence.
2. Read the selected path index for each balloon.
3. Append the balloon to the corresponding `PathCreeps` entry.
4. Preserve the balloon's original global spawn time.

Preserving global time is important: three entrances do not emit three balloons simultaneously unless the round data itself schedules that. For ten balloons and the modulo example, the path counts are `4, 3, 3`, while the overall cadence remains one balloon per normal spawn interval.

Runtime validates that the selection count matches the expanded balloon count and that every selected path still exists. A mismatch is a content error rather than a silent fallback.

## Twin Gate Outpost

Twin Gate Outpost will define three paths in stable A/B/C order:

- three distinct spawn checkpoints on the upper-left, middle-left, and lower-left arms;
- a shared merge checkpoint on the right side;
- one base checkpoint to the right of the merge, within the established `-1400..=1400` by `-800..=800` bounds.

Every path starts at its own `Spawn`, reaches the shared merge, and ends at the same `Base`. The map will use:

```lua
SelectSpawnPath = function(_, balloon_index, _)
  return ((balloon_index - 1) % 3) + 1
end
```

Thus every round distributes balloons A, B, C, A, B, C, with counts differing by at most one.

## Tower Placement Feedback

Placement validation will return a structured result instead of a boolean. The initial rejection reasons are:

- insufficient gold, including required and available gold;
- too close to a road;
- inside or intersecting a blocked region;
- overlapping an existing tower;
- missing tower placement metadata.

The preview remains green when valid and red when invalid, but an invalid preview also shows a short localized reason. Click-time validation uses the same result as preview rendering. Backend validation remains authoritative and logs the corresponding reason.

Path geometry used by validation must come from the same path snapshot used to render roads. The Twin Gate topology change removes the incorrect return route to the upper-left endpoint. A geometry regression test will use the reported green-space position near backend coordinate `(-420, -250)` with sufficient gold and no tower overlap; it must be accepted. Positions whose placement circle intersects a rendered road must remain rejected.

## Error Handling and Compatibility

- A missing selector defaults to path 1.
- A selector on a map with no paths is a load error.
- A non-integer, NaN, infinite, zero, negative, or out-of-range result is a load error.
- An error in any round rejects the map as a whole; partially generated selection data is never installed.
- Single-path maps remain behaviorally unchanged.
- Existing map-authored non-TD `CreepWave.Detail` processing remains unchanged.
- Development reload affects newly created worlds. It does not rewrite waves in an already running match.

## Testing

Automated tests will cover:

- the three callback arguments are one-based and contain the documented balloon fields;
- the modulo selector returns `1, 2, 3, 1, 2, 3`;
- ten balloons distribute as `4, 3, 3` without changing global spawn times;
- selectors can branch on round number and balloon flags such as `camo`;
- missing selectors default to path 1;
- invalid return values and callback errors include full map/round/balloon context;
- shipped generation and runtime Lua-content loading produce identical selections;
- single-path TD maps retain their existing wave output;
- Twin Gate has exactly three spawn checkpoints and one shared base, and all three paths terminate at that base;
- the reported green-space placement coordinate is accepted with sufficient gold;
- road, region, overlap, metadata, and gold placement failures return the correct reason.

Manual smoke testing will start several rounds on Twin Gate and confirm A/B/C ordering visually, verify that uneven round sizes differ by no more than one between entrances, place a tower in the reported green gap, and verify each displayed placement rejection reason.

## Out of Scope

- Running arbitrary Lua during simulation ticks.
- Random entrance selection.
- Changing balloon combat stats from the selector.
- Reassigning already spawned balloons or rewriting an in-progress match during hot reload.
