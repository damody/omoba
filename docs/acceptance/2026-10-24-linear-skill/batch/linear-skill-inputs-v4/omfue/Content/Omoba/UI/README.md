# Original omoba HUD icon atlas

## Complete hero portraits, 2026-10-18

`training-apprentice-v1.png` (晨光學徒) and `training-support-v1.png` (晨光援護者) are original opaque 1254×1254 artwork generated with the built-in imagegen tool. Both outputs were directly reviewed and copied byte-for-byte; the originals remain intact. Full prompts, original output paths, dimensions and SHA-256 are in `hero-portrait-artwork-provenance-v1.json`. The tool did not expose a model identifier. No reference image, downloaded/Riot artwork or pixel post-processing was used.

The manifest now contains portraits for all seven authoring hero IDs. Build and verify-only preflight check complete catalog registration and source PNG structure/square tiles; native tests decode every generated hero portrait and can capture the actual production Slate widget with `-om-hero-portrait-capture=<fresh absolute directory>`. Unknown content keeps its explicit fallback and retirement clears old resources. This is current integration status; older "pending" notes below are historical.

## Standalone summoner artwork, 2026-10-08

`training-point-blink-v1.png` and `training-self-heal-v1.png` are original opaque 1254×1254 PNGs generated with the built-in image_gen tool, visually reviewed and copied byte-for-byte into this directory. Their exact prompts and original source paths are recorded in `.ai-collab/moba-summoner-artwork.md`. No reference image or post-generation pixel edit was used. These are prepared D/F assets; registry support and gameplay HUD consumption are still pending.

`omoba-hud-icons-v1.png`: 1254 × 1254 pixels, opaque PNG, 4 × 4 normalized UV tiles. SHA-256: `7fd6180e7591ee6f8ec9c9eab46e2c815ec443325a1dadcdc809b236ff1a796a`.

Created on 2026-10-07 with the built-in imagegen tool for this project. The tool did not expose a model identifier. Original fantasy artwork; no Riot asset or screenshot was used. Source output: `C:/Users/User/.codex/generated_images/01a11288-76ef-7302-a8dc-c1a689c648c5/exec-5be7f7e4-67ba-4840-9514-472396a07237.png`. The project PNG is a byte-identical copy, not a dependency on that user directory.

Tile order: sniper_mode, saika_reinforcements, rain_iron_cannon, three_stage_technique; flame_blade, fire_dash, flame_assault, matchlock_gun; lumen_bolt, lumen_touch, lumen_lance, lumen_mend; move, attack, recall, ward. Apprentice abilities reuse their matching lumen tile. Unknown abilities retain readable text without borrowing unrelated artwork. Consumers use normalized quarter UVs with a 2% tile inset; integer pixel divisibility is not required. Raw PNG is staged as UFS via project packaging settings.

Full generation prompt:

> Use-case: stylized-concept. Create one production bitmap sprite atlas for the original omoba MOBA game's HUD, 4 columns by 4 rows, perfectly square image, sixteen equally sized square tiles with exact edge-to-edge quarter boundaries and no gutters. Crisp painterly tactical fantasy icons, readable at 64 pixels, strong single focal silhouette, luminous color accents over deep charcoal navy tile backgrounds, restrained metallic detail, coherent polished game art. No letters, numbers, labels, captions, logos or copied game assets. No external frame surrounding the atlas. Each icon artwork stays within central 75% of its tile so quarter cropping is safe. The sixteen tiles in exact left-to-right, top-to-bottom order: Row 1: cyan sniper scope with a bright precise aiming lens; blue reinforcements banner and three original matchlock soldier silhouettes; orange-red rain of iron projectiles; magenta three simultaneous matchlock firing streaks. Row 2: orange flaming sword blade; red fire dash silhouette leaving a sweeping flame trail; a golden-orange massive flame assault impact; a detailed original wooden matchlock gun discharging a single glowing bullet. Row 3: a gold-white luminous bolt aimed diagonally; a teal glowing restorative hand; a white-gold lance of light; a gold and emerald regenerating blossom. Row 4: an emerald ground movement arrow along a short path; a crimson attack target with crossed weapon shapes; a blue recall/home spiral surrounding an original small tower; a cyan vision eye and small ward beacon. These are gameplay icons for original assets, not a screenshot or UI mockup. Produce a clean uniform 4x4 grid for software UV cropping.

Generation used `transparent_background=false`. No post-generation pixel edits.

## 2026-10-08 original portraits and training icons

`hud-assets-v1.json` is the versioned runtime artwork registry. It selects files and normalized tile UVs by the actual hero/ability content ID. Five original 1254×1254 portraits cover training_vanguard, training_ranger, saika_magoichi, date_masamune and training_luminary. `training-item-atlas-v1.png` adds the eight actual vanguard/ranger abilities. Its item and future ward/blink/heal tiles are artwork only and are not bound as gameplay abilities.

These images were generated with the built-in image_gen tool on 2026-10-08, visually reviewed, and copied byte-for-byte with fixed Lua binary IO. No existing franchise image, external reference, or pixel edit was used. Each portrait has a large readable face on navy, no lettering/frame/logo, and an original costume. Source paths, dimensions, byte counts, SHA-256 and prompt summaries are recorded in `.ai-collab/candidates/hero-portraits/manifest.json` and `training-item-atlas-manifest.json`, and the durable repository report `docs/plans/2026-10-08-original-hud-assets.md`.

Registry parsing rejects unknown versions, duplicate IDs, invalid tile grids, fractional indices and paths outside this UI directory. Unknown content displays a neutral background and readable text. The consuming widget retains textures through a transient UObject map and does not import them every frame. Source integration still requires current compiled/rendered acceptance.


## 2026-10-24 linear skill loadout

training-line-mage-v1.png and lumen-ray-v1.png are original opaque1254x1254 portrait/ability artwork generated with built-in imagegen, directly reviewed and copied byte-for-byte without pixel edits. Full exact prompts, original paths, dimensions, workspace destinations and SHA-256 are in linear-skill-artwork-provenance-v1.json. HUD manifest coverage now includes all9 authoring hero loadouts and the new lumen_ray skill; no portrait aliases or runtime identity special cases were added. Native decode and production Slate capture remain required in the final batch acceptance.
