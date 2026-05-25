---
name: omoba-character-pipeline
description: Build an omoba character-generation package from a character description and skill text. Use when creating AI-assisted hero concept packages, prompts, toolchain diagnostics, and future omb/omfx import stubs.
---

# omoba Character Pipeline

Create a reviewable omoba character package from a character description and skill text. This skill is repo-local because it depends on omoba's Lua content conventions, hero/ability id style, omfx asset slots, and OpenSpec workflow.

The first version produces package files and diagnostics only. It MUST NOT directly modify runtime/gameplay files under `omb/`, `omfx/`, or `scripts/base_content/`.

## Outputs

Character packages live under:

`docs/character_pipeline/packages/<hero_id>/`

Each package uses Lua as source of truth:

- `character.lua`
- `prompts.lua`
- `manifest.lua`
- `toolchain.lua`
- `omoba_stub.lua`
- `review.md`

## Shared AI Roots

Default shared asset root:

`/media/damody/新增磁碟區/AI_Pic`

Known tool roots:

- `/media/damody/新增磁碟區/AI_Pic/ComfyUI/ComfyUI`
- `/media/damody/新增磁碟區/AI_Pic/stable-diffusion-webui`

Treat this location as a shared Windows/Linux asset root. Models, workflows, references, and outputs may be shared. Python virtual environments and portable Python runtimes are platform-specific and MUST NOT be shared or overwritten.

Linux venvs default to:

`~/.cache/omoba-character-pipeline/venvs/`

## Commands

### `bootstrap`

Run the safe toolchain diagnostic before generating packages.

Use:

```bash
python3 docs/character_pipeline/tools/bootstrap.py --diagnose
```

The diagnostic checks:

- `nvidia-smi`
- Python version
- Linux venv root
- shared AI root readability and write probe
- ComfyUI checkout
- stable-diffusion-webui checkout
- Windows-shaped venv/portable Python markers
- Blender availability
- `lua5.4` or compatible Lua interpreter

The diagnostic may report missing tools. It MUST NOT delete, rebuild, or overwrite Windows `venv/Scripts`, `python_embeded`, models, workflows, or outputs.

### `new <description>`

Use the character description and skill text to derive:

- `hero.id`: ASCII snake_case
- display name and title
- gender/body/personality cues
- role and combat read
- art direction
- Q/W/E/R ability list
- visual motifs for each ability

Default to `auto_decide` when the user does not provide a field. Keep assumptions visible in the eventual `review.md`.

### `draft-package <hero_id>`

Create a package under `docs/character_pipeline/packages/<hero_id>/`.

Follow the schema in `docs/character_pipeline/schema/README.md`. Use `hero_example` as a fixture reference, but generate a new coherent package for the requested hero.

Rules:

- Use Lua files, not YAML.
- Keep paths platform-neutral.
- Use fixed seeds unless the user requests otherwise.
- Prefer ComfyUI provider entries. Keep stable-diffusion-webui as fallback.
- Include Hunyuan3D-style 3D provider slots, but do not require a real 3D run in version one.

### `review <hero_id>`

Validate and review the package:

```bash
lua5.4 docs/character_pipeline/tools/validate_package.lua docs/character_pipeline/packages/<hero_id>
```

If `lua5.4` is not installed system-wide, use the user-local bootstrap fallback:

```bash
~/.cache/omoba-character-pipeline/bin/lua5.4 docs/character_pipeline/tools/validate_package.lua docs/character_pipeline/packages/<hero_id>
```

Then read all package files and update `review.md` with:

- AI-made assumptions
- character identity and role
- skill kit coherence
- MOBA/TD isometric readability
- icon readability at small sizes
- model/rig risks
- animation coverage
- omoba import readiness
- `GameContractWarning` notes

### `revise <hero_id>`

Apply requested edits to package Lua files and `review.md`, then run the validator again.

## State Machine

- `missing_toolchain`: bootstrap has not passed, or required diagnostics failed.
- `ready`: bootstrap has enough passing checks to draft packages.
- `drafted`: package exists but has not passed review.
- `approved`: package has passed validation and human batch review.
- `blocked`: toolchain, shared root, or schema validation cannot proceed without intervention.

## Package Generation Rules

When generating from a free-form character description:

1. Normalize `hero.id` to ASCII snake_case.
2. Create exactly four primary abilities for Q/W/E/R unless the user specifies otherwise.
3. Keep ability ids unique and derived from the hero id.
4. Make visual motifs consistent across portrait, skill icons, turnaround, model, rig, and animation prompts.
5. Record every AI-filled assumption in `review.md`.
6. Put future game integration data in `omoba_stub.lua`, not in runtime files.

## Hard Boundaries

First-version package generation MUST NOT:

- edit `omb/`
- edit `omfx/`
- edit `scripts/base_content/`
- copy assets into runtime asset folders
- modify Windows `venv/Scripts`
- modify ComfyUI `python_embeded`
- run long GPU generation jobs as validation
