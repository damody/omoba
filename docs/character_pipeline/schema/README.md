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


## Schema v2：玩法契約（2026-10-25）

`character.lua` 設 `schema_version = 2` 的 package 使用 v2 規則（v1 package 不受影響）。v2 涵蓋計畫 `docs/plans/2026-10-24-omfue-month-mvp/README.md` §4.2，規則實作在 `tools/package_v2.lua`，由 `tools/validate_package.lua` 依版本分派。

必要檔案：`character.lua`、`source.lua`、`abilities.lua`、`capabilities.lua`、`presentation.lua`（都回傳 `function(ctx)`，`schema_version = 2`），以及非空的 `assumptions.md`、`acceptance.md`。v1 的 prompts／manifest／toolchain／omoba_stub 在 v2 為選用。

| 檔案 | 內容 |
|---|---|
| `character.lua` | `hero`（id、display_name、title、role、combat_read）、`identity`（game_hero_id = hero.id、rust_module、source_hero_id、`integration = append｜update_existing`、depends_on）、`attack`（melee／ranged、range）、`base_stats` |
| `source.lua` | `provenance`（repository、commit、source_hero_id、variant、language）、`abilities`（source_ability_id、path、line、64 位 raw_sha256）、`assumptions`（id、kind = assumed／adaptation／kv_text_conflict／not_applicable、text） |
| `abilities.lua` | 每招：id、source_ability_id、action（Q／W／E／R／extra1～3；被動也佔一格）、ability_type、max_level、status、capabilities、targeting（kind、team、range、shape、visibility）、timing（cast_point、cooldown_starts）、resources（cooldown、mana_cost、charges）、required_hero_level、effects（封閉詞彙）、lifecycle（caster_death、target_invalid、vision_lost、cancel、expire）、implementation（generic／rust_handler）、bot_usage、verification.scenarios、assumptions 參照 |
| `capabilities.lua` | 需要的 capability 與狀態（supported／ambiguous／needs_framework／deferred）、`kit_status`、`catalog_ready` |
| `presentation.lua` | animation_states（idle、move、attack、cast、death…）、每招 preview／animation／cues；人工 Editor 設定優先於這裡的生成預設 |

`targeting.shape` 的寫法（只用於 HUD 指示器）：
- `circle` 填 `radius`（落點圓）。
- `line` 填 `width`（全寬）。
- `cone` 填 `half_angle`（角度制，0 < 半角 ≤ 90）。
- `none` 表示沒有範圍。
- 數值可以是常數，也可以是每級一個值的陣列。

產生的內容會帶上 `targeting_shape`（omoba-content-model `AbilityTargetingShape`），以及 `shape_radius`／`shape_width`／`shape_half_angle` extras。只有當 point 目標的技能沒有 gameplay 範圍契約（effects、aim_geometry、projectile）時，HUD 卡片才會公布這個範圍；沒有公布時一律為 0。這個範圍不會改變施放判定或執行結果，tooltip 會標示為「技能自訂判定」。

驗證規則摘要：

- 每個 rank 陣列長度等於 `max_level`。數值使用與 `scripts/gen_hero_registry.lua` 相同的 authored 範圍：0，或絕對值在 [1/1024, 1000000] 的有限 f32。
- `required_hero_level` 是 1..25 的遞增整數。
- action 不得重複。MOBA loadout 上限 4 格且被動也佔一格；用 extra1～3 的技能必須列出 `extra_action_slot` capability。非被動技能必須有 bot_usage。
- 每個 source ability 都要對應到 package ability，反之亦然；assumption 參照必須存在。
- 技能 status 由 capabilities 推導（最差者為準），`kit_status` 由技能推導。`catalog_ready = true` 需要全部 supported，且 `acceptance.md` 含 `status: accepted`。
- `--catalog <source-catalog.lua>`：provenance commit、variant、hero 與每招 raw_sha256 必須和來源索引一致；來源改變時 package 判為 stale。
- `--requirements <candidate-requirements.lua>`：capability 狀態必須和 D3 登錄一致；不一致時要寫 `override_reason`。

範例：`packages/saika_magoichi`（H01，來源 B01 current 版）。
