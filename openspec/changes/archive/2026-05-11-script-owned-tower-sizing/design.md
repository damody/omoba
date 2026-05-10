## Context

Tower combat visuals now use script-provided render metadata, but sizing still risks leaking back into code as formulas such as `visual_size = footprint * multiplier` or `placement_radius = render.size / 2`. That makes content authoring fragile because a tower's intended visual footprint and placement blocker cannot be read directly from `scripts/lua_data/templates/towers.lua`.

The sizing contract crosses the full metadata path: Lua templates → `omoba-template-ids` generated consts → `scripts/script-abi` → `scripts/base_content` → `omb` runtime registry → `omfx` snapshot/render cache. The design must keep all long-lived tower size values script-owned while preserving existing gameplay semantics for runtime collision, attack range, projectile spawn, damage, cooldown, and lockstep hash.

## Goals / Non-Goals

**Goals:**

- Make tower combat visual size an explicit script metadata field, not a Rust-side formula.
- Make tower placement radius an explicit script metadata field, not `visual_size / 2`, `footprint`, clamp, or any other inferred value.
- Keep `WORLD_SCALE` as a pure backend-world-unit to render-unit conversion factor.
- Keep recoil, attack animation, buff visual effects, and hover effects as temporary transforms layered on top of script-owned base size.
- Validate missing or invalid sizing metadata during content/codegen tests instead of silently falling back at runtime.
- Keep existing runtime collision, pathing collision, attack range, projectile spawn, damage, and cooldown unchanged unless a future change explicitly moves them to script-owned sizing too.

**Non-Goals:**

- This change does not rebalance tower range, damage, cooldown, projectile speed, or hit radius.
- This change does not replace `footprint` everywhere; it only removes its use as an implicit visual or placement sizing fallback.
- This change does not introduce editor UI for tuning tower sizes.
- This change does not require new art assets.

## Decisions

### Decision: Use explicit script fields for each long-lived size

Use two separate content fields:

- `render.visual_size`: square combat sprite diameter in backend world units.
- `placement_radius`: authoritative tower placement blocker radius in backend world units.

Rationale: visual diameter and placement blocker are related design concepts but not always the same. A cake splash tower may look large while still allowing placement closer to paths; a thin tall tower may need a smaller blocker than its sprite canvas. Separate fields avoid hidden assumptions like `/ 2`.

Alternative considered: keep only `render.size` and derive placement radius as half. Rejected because it is exactly the hidden hard rule this change is meant to remove.

Alternative considered: reuse `footprint`. Rejected because `footprint` already participates in existing runtime collision/template data and would couple visual authoring to gameplay behavior.

### Decision: Keep `WORLD_SCALE` as conversion only

`omfx` SHALL compute render size as `render.visual_size * WORLD_SCALE`. It SHALL NOT multiply by additional global tower visual scale constants, clamps, footprint-derived multipliers, or per-kind hardcoded tables.

Rationale: `WORLD_SCALE` already maps backend coordinates to render coordinates. Keeping it as the only unit conversion makes the script value directly inspectable and predictable.

Alternative considered: keep a frontend `TD_TOWER_VISUAL_SCALE` for broad tuning. Rejected because it encourages content authors to compensate by shrinking script values and makes long-term authored sizes ambiguous.

### Decision: Validate explicit values in content/codegen

`omoba-template-ids` SHALL reject tower metadata where `render.visual_size <= 0` or `placement_radius <= 0`. Shipped towers SHALL explicitly declare both fields in `scripts/lua_data/templates/towers.lua`.

Rationale: failing early keeps invalid content from becoming a runtime frontend/backend mismatch. It also prevents Rust-side default rules from becoming an invisible content contract.

Alternative considered: provide Rust defaults from `footprint`. Rejected because it recreates hidden sizing behavior outside scripts.

### Decision: Preserve temporary visual scale for effects

Recoil, buff visuals, attack windup animation, and similar short-lived effects may still apply transform scale in omfx. Those effects are multiplicative and temporary over the script-owned base visual size.

Rationale: this preserves existing animation behavior while keeping long-lived size authored in content.

Alternative considered: store every animation scale keyframe in Lua now. Rejected as unnecessary scope expansion; existing recoil metadata already covers the current short-lived effects.

### Decision: Snapshot carries both values

`SimWorldSnapshot.tower_templates` SHALL carry `render.visual_size` and `placement_radius` so omfx can render sprites and preview placement without independently loading Lua or reconstructing backend rules.

Rationale: omfx already consumes tower template snapshots for render metadata. Adding explicit fields keeps data flow deterministic and avoids frontend-only content parsing.

Alternative considered: frontend reads `scripts/lua_data/templates/towers.lua` directly. Rejected because the authoritative loaded script metadata is already available through omb and snapshots.

## Risks / Trade-offs

- [Risk] ABI-safe metadata layout changes can break host/script DLL compatibility if only one workspace is rebuilt → Mitigation: keep the existing two-step build workflow and run scripts workspace tests plus omb/omfx checks.
- [Risk] Existing code may still use `footprint` for placement preview in one path → Mitigation: add focused tests or grep-based assertions for no placement sizing derivation from `footprint` or `visual_size / 2`.
- [Risk] Content values may make towers impossible to place in narrow TD maps → Mitigation: tune shipped `placement_radius` values explicitly in Lua and include manual TD_1 placement verification.
- [Risk] `render.visual_size` and `placement_radius` may drift semantically in docs → Mitigation: update tower asset README, gen-docs, and OpenSpec specs with the exact units and ownership contract.

## Migration Plan

- Add `render.visual_size` and `placement_radius` to shipped tower Lua entries.
- Extend codegen and ABI metadata in the same branch so script DLL and host stay in sync.
- Update backend placement validation and frontend placement preview to consume `placement_radius`.
- Update omfx composite rendering to consume `render.visual_size * WORLD_SCALE` for base/barrel/body node size.
- Remove Rust constants and helper formulas that derive long-lived tower size from footprint, clamps, global scale, or `/ 2`.
- Run focused tests/checks across `omoba-template-ids`, `scripts`, `omb`, and `omfx`.

## Open Questions

- 是否要把既有 `footprint` 在未來 change 中重新命名成 runtime collision radius，以免繼續和 placement radius 混淆？
