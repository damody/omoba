# omoba Character Pipeline Skill Design

Date: 2026-05-25

## Goal

Create a repo-local Codex skill that guides a mostly automated character generation workflow for omoba. The first deliverable is not direct game integration. It is a reviewable, rerunnable character package that captures the character concept, ability text, AI generation prompts, toolchain settings, expected asset outputs, and future omb/omfx import metadata.

The workflow must run on Ubuntu, reuse the existing shared AI asset folder used by both Windows and Linux, and prefer AI/default decisions unless the user asks for manual control.

## Scope

In scope for the first version:

- Add a repo-local skill at `.codex/skills/omoba-character-pipeline/`.
- Make toolchain bootstrap and verification the first executable phase.
- Use existing AI tool roots when available:
  - `/media/damody/新增磁碟區/AI_Pic/stable-diffusion-webui`
  - `/media/damody/新增磁碟區/AI_Pic/ComfyUI`
- Generate character packages under `docs/character_pipeline/packages/<hero_id>/`.
- Use Lua files as the package source of truth.
- Produce omoba-facing stub metadata for future import into templates, scripts, and omfx assets.
- Use a batch review gate after generating the full package.

Out of scope for the first version:

- Directly modifying omb, omfx, templates, or base_content scripts.
- Running long GPU generation jobs as part of validation.
- Requiring NVIDIA Kimodo, Omniverse, or any single 3D/animation service as a hard dependency.
- Sharing Python virtual environments between Windows and Linux.

## Recommended Approach

Use a repo-local wizard skill with a fixed, practical toolchain target and an explicit bootstrap phase.

The skill starts by checking and preparing the Ubuntu execution environment. It then turns a role/character description plus skill text into a Lua character package. It does not directly import the result into the game. Instead, it creates structured package files that can later drive image generation, 3D generation, rigging, animation, and omoba import.

This balances automation with safety: the toolchain can be installed and verified early, while generated character data remains isolated until reviewed.

## Existing Environment Findings

The current machine already has a usable NVIDIA GPU:

- GPU: NVIDIA GeForce RTX 5090
- Driver: 595.71.05
- VRAM: 32607 MiB

The shared AI folder is mounted from NTFS:

- Mount: `/media/damody/新增磁碟區`
- Mode: read/write via `ntfs3`

The stable-diffusion-webui checkout exists and contains many checkpoints, but its `venv` is Windows-shaped (`venv/Scripts/python.exe`) and cannot be used as a Linux venv.

The ComfyUI folder exists as a Windows portable-style layout. The actual ComfyUI checkout is:

`/media/damody/新增磁碟區/AI_Pic/ComfyUI/ComfyUI`

System Python is currently 3.12.3 and does not have torch installed. Blender is not currently available on PATH.

These findings drive the bootstrap design: share large assets and outputs, but keep executable environments per platform.

## Cross-Platform Shared Asset Strategy

The directory `/media/damody/新增磁碟區/AI_Pic` is treated as a shared asset root, not as a shared execution environment.

Shared across Windows and Linux:

- model checkpoints
- LoRA and ControlNet assets when present
- ComfyUI workflow JSON files
- generated outputs
- reference images
- package output mirrors

Platform-specific and not shared:

- Python virtual environments
- portable Python executables
- shell scripts generated for one OS
- compiled native extensions
- CUDA/PyTorch installs

Linux bootstrap must not overwrite or delete the existing Windows portable Python or `venv/Scripts` trees. Linux environments should be created outside the shared tool checkouts, for example under:

- `.codex/tools/omoba-character-pipeline/venvs/`
- `~/.cache/omoba-character-pipeline/venvs/`

The shared root is referenced from Linux runner configs. Large model files stay in the shared root.

## Toolchain Target

First-version provider targets:

- 2D image provider: ComfyUI first, stable-diffusion-webui API as fallback.
- Prompt and package generation: Codex writes Lua and Markdown package files.
- 3D provider: reserve runner slots for Hunyuan3D or TripoSR-style tools.
- Rig/check/export: Blender Python.
- Animation target: generate Kimodo/text-to-motion prompts and skeleton target settings, but do not require Kimodo in the first version.

ComfyUI is preferred for automation because workflow JSON, fixed seeds, and batch execution are easier to make repeatable than UI-driven web generation.

## Skill Commands

The skill should document and guide these phases:

### `bootstrap`

Check and prepare the Ubuntu toolchain.

Required checks:

- `nvidia-smi` can see the GPU.
- Linux Python environment can import torch.
- `torch.cuda.is_available()` is true.
- ComfyUI or stable-diffusion-webui can be launched or smoke-tested from Linux.
- Blender CLI can run a background Python expression.
- The shared asset root is readable and writable.

If a tool is missing, bootstrap should install or prepare it before package generation. It must avoid modifying Windows venvs or portable Python folders.

### `new <description>`

Parse a character description and skill text. AI may fill missing fields by default. The output is a candidate hero id, role, visual direction, personality, and ability set.

### `draft-package <hero_id>`

Write the Lua package files:

- `character.lua`
- `prompts.lua`
- `manifest.lua`
- `toolchain.lua`
- `omoba_stub.lua`
- `review.md`

### `review <hero_id>`

Perform a batch review. The user reviews the whole package after the AI has made default decisions.

### `revise <hero_id>`

Update the package based on user feedback.

Reserved for future versions:

- `run-assets`
- `generate-animation`
- `import-omoba`

## State Machine

The skill tracks the workflow conceptually with these states:

- `missing_toolchain`: bootstrap has not passed.
- `ready`: enough toolchain checks pass to draft a package.
- `drafted`: package exists but has not been approved.
- `approved`: package passed batch review.
- `blocked`: validation or toolchain setup failed and needs intervention.

Default behavior is batch review. The skill should not stop for every field unless the user disables automatic decisions.

## Lua Package Schema

The package uses Lua as the source of truth because the project already uses Lua data under `scripts/lua_data/*`.

Each Lua file follows this shape:

```lua
return function(ctx)
  return {
    schema_version = 1,
  }
end
```

### `character.lua`

Defines the hero concept, gameplay draft, abilities, and automation mode.

```lua
return function(ctx)
  return {
    schema_version = 1,
    hero = {
      id = "hero_example",
      display_name = "範例英雄",
      title = "範例稱號",
      gender = "female",
      personality = { "calm", "ruthless" },
      role = "marksman",
      combat_read = "遠程單點輸出，技能以狙擊與標記為主",
      art_direction = {
        style = "stylized moba readable from isometric camera",
        silhouette = "long coat, rifle, high collar",
        palette = { "deep red", "gunmetal", "ivory" },
        materials = { "cloth", "brushed metal" },
        avoid = { "text", "logos", "copyrighted character likeness" },
      },
    },
    gameplay = {
      faction = "player",
      base_stats_draft = {
        hp = 0,
        mana = 0,
        attack_damage = 0,
        attack_range = 0,
        move_speed = 0,
      },
    },
    abilities = {
      {
        id = "hero_example_q",
        slot = "Q",
        name = "穿甲射擊",
        gameplay_intent = "line projectile damage",
        visual_motif = "thin red ballistic trail",
      },
    },
    automation = {
      mode = "auto_decide",
      batch_review = true,
      seed_policy = "fixed",
    },
  }
end
```

### `prompts.lua`

Defines provider-ready prompt bundles for portrait, skill icons, turnaround, 3D model, rig, and animation.

Each prompt bundle should include:

- provider
- positive prompt
- negative prompt
- style lock
- seed
- size or target format
- reference inputs
- acceptance criteria
- retry policy

### `manifest.lua`

Defines planned and generated files. It records status for each asset:

- `planned`
- `generated`
- `approved`
- `rejected`

Manifest paths must be relative or explicitly rooted in the shared asset root. They must not contain unsafe `..` traversal or Windows-only separators.

### `toolchain.lua`

Records provider roots, Linux venv paths, shared asset roots, selected model files, workflow files, seeds, and runner options.

### `omoba_stub.lua`

Defines future game integration metadata:

- hero id, display name, title
- ability ids and Q/W/E/R slots
- expected portrait and icon paths
- model slot, for example `omfx/data/heroes/<hero_id>/<hero_id>.glb`
- animation clips: `idle`, `run`, `attack`, `cast_q`, `cast_w`, `cast_e`, `cast_r`, `death`
- script hints: projectile, buff, summon, aoe, toggle
- draft template stats marked as draft

This file is not applied automatically in the first version.

## Review Document

`review.md` is generated for human review. It should summarize:

- AI-made assumptions.
- Character identity and role.
- Skill kit coherence.
- Visual readability from an isometric MOBA/TD camera.
- Icon readability at small sizes.
- Model and rigging risks.
- Animation coverage.
- omoba import readiness.
- Any game contract warnings.

## Error Handling

Errors are grouped by type:

- `ToolchainError`: GPU, Python, torch, ComfyUI, webui, or Blender is unavailable.
- `SharedRootError`: shared root is missing, unwritable, or would require modifying a Windows environment.
- `SchemaError`: Lua package fields are missing or invalid.
- `GameContractWarning`: package is valid but future omoba integration has risks.

`ToolchainError`, `SharedRootError`, and `SchemaError` block progress. `GameContractWarning` does not block drafting, but must appear in `review.md`.

## Validation

Bootstrap validation:

- Read GPU info with `nvidia-smi`.
- Import torch in the Linux venv.
- Confirm CUDA availability.
- Smoke-test ComfyUI or stable-diffusion-webui.
- Run Blender in background mode.
- Probe shared root read/write with a temporary file and delete it.

Package validation:

- Load all package Lua files with a Lua interpreter.
- Verify required fields.
- Verify hero id is ASCII snake_case.
- Verify ability ids are unique.
- Verify Q/W/E/R slots do not repeat.
- Verify prompt providers exist in `toolchain.lua`.
- Verify manifest paths are safe and platform-neutral.

Game readiness review:

- Static check only.
- Do not build omb or omfx.
- Compare `omoba_stub.lua` against existing conventions for hero ids, ability ids, asset slots, and script hints.

## Testing Strategy

First-version tests should avoid long AI generation jobs.

Use a small fixture character package to test:

- Lua package loading.
- schema validation.
- path safety checks.
- bootstrap diagnosis output when optional tools are missing.

The implementation plan should decide whether validator scripts live under the skill directory or `docs/character_pipeline/tools/`.

## Implementation Defaults

The implementation plan should use these defaults unless a concrete blocker appears:

- Linux venvs live under `~/.cache/omoba-character-pipeline/venvs/`.
- ComfyUI is launched from Linux with a dedicated venv and exercised through its HTTP API.
- stable-diffusion-webui is fallback-only and is exercised through its HTTP API with API mode enabled.
- Package validation uses `lua5.4` from the OS package manager first. A Rust/Lua validator can be added later if integration with existing Rust tooling becomes useful.
- Blender is discovered from PATH first. If missing, bootstrap installs a Linux Blender build without touching the shared AI root.
- The first scaffolded 3D provider slot is Hunyuan3D-style image-to-3D. TripoSR remains a documented fallback slot.
