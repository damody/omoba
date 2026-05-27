# omoba Character Pipeline Schema

Character packages live under `docs/character_pipeline/packages/<hero_id>/`.

Each package uses Lua as the source of truth. Every Lua source file returns a function:

```lua
return function(ctx)
  return {
    schema_version = 1,
  }
end
```

The `ctx` argument is reserved for future tooling. Version one packages should not require it.

## File Layout

Required files:

- `character.lua`: hero identity, gameplay draft, ability kit, and automation settings.
- `prompts.lua`: provider-ready prompt bundles for 2D, 3D, rig, and animation stages.
- `manifest.lua`: planned/generated asset paths, formats, statuses, and shared output roots.
- `toolchain.lua`: shared asset roots, provider roots, Linux venv paths, selected models, and runner defaults.
- `omoba_stub.lua`: future omb/omfx/template/script import metadata. It is a draft only.
- `review.md`: human-readable batch review summary.

## Identifier Rules

`hero.id` must be ASCII snake_case:

- allowed: `hero_example`, `saika_alt_marksman`
- rejected: `HeroExample`, `hero example`, `英雄`, `hero-example`

Ability ids must be unique. Q/W/E/R slots must not repeat.

## `character.lua`

Required top-level fields:

- `schema_version`
- `hero`
- `gameplay`
- `abilities`
- `automation`

Required `hero` fields:

- `id`
- `display_name`
- `title`
- `gender`
- `personality`
- `role`
- `combat_read`
- `art_direction`

Required ability fields:

- `id`
- `slot`
- `name`
- `gameplay_intent`
- `visual_motif`

## `prompts.lua`

Prompt bundles are keyed by generation stage. Common keys:

- `portrait`
- `turnaround`
- `skill_icons`
- `model_3d`
- `rig`
- `animations`

Each prompt bundle should define:

- `provider`
- `positive_prompt`
- `negative_prompt`
- `seed`
- `size` or `target_format`
- `acceptance_criteria`
- `retry_policy`

`provider` must exist in `toolchain.lua`.

## `manifest.lua`

Asset entries track expected files without requiring the files to exist in version one.

Allowed statuses:

- `planned`
- `generated`
- `approved`
- `rejected`

Paths must be platform-neutral. Use `/`, avoid Windows backslashes, and never use `..` traversal.

## `toolchain.lua`

Records tool roots and defaults:

- `shared_asset_root`
- `linux_venv_root`
- provider definitions
- selected model names or paths
- workflow files
- runner options

The shared asset root may point at `/media/damody/新增磁碟區/AI_Pic`. Linux venvs must live outside the shared tool checkout.

## `omoba_stub.lua`

This file records future import data but is not applied automatically.

Expected sections:

- `hero`
- `abilities`
- `assets`
- `animations`
- `script_hints`
- `template_draft`

The model slot should use a future import path such as:

`omfx/data/heroes/<hero_id>/<hero_id>.glb`

## `review.md`

The review document summarizes:

- AI-made assumptions
- character identity and role
- skill kit coherence
- isometric MOBA/TD readability
- icon readability
- model and rigging risks
- animation coverage
- omoba import readiness
- `GameContractWarning` notes

