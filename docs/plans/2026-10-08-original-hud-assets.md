# Original HUD artwork provenance — 2026-10-08

Built-in image_gen; model ID was not exposed. Original fantasy artwork, no franchise/external reference, no pixel editing. All assets were visually reviewed and copied byte-for-byte into project Content/Omoba/UI with fixed Lua binary IO. Current source integration is pending compiled/rendered acceptance.

| Content | File | Dimensions | Bytes | SHA-256 |
|---|---|---|---:|---|
| training_vanguard | `training-vanguard-v1.png` | 1254×1254 | 2426095 | `88fe5dd55d74ae2ffe686d1f62bce99e4348f12e68b0ebe244cde8c81dc757ba` |
| training_ranger | `training-ranger-v1.png` | 1254×1254 | 2550240 | `c6bd07f19510a59da7f5a1347db9c8eea0e5ae97f8a4ba2cd5fd5f2d0a67d001` |
| saika_magoichi | `saika-magoichi-v1.png` | 1254×1254 | 2647955 | `ae4f61d31942d355ca10e73f6538d880b1e1d6a660348afc3f248c30a5167068` |
| date_masamune | `date-masamune-v1.png` | 1254×1254 | 2794642 | `11a9d95b16ddd4a996ecfdfeab08527c881b051618912d7aee2b985a53151d39` |
| training_luminary | `training-luminary-v1.png` | 1254×1254 | 2584538 | `788751212b7c9197febb457d967069e64cb9ca4df6ddaf68af0b6eb461a4c5ff` |
| training abilities / item art | `training-item-atlas-v1.png` | 1254×1254 | 2866615 | `93016026f4844cdb6dfdf94b8ed57afd681baf5cefd7706314b4d299d5bece8d` |

## Generation sources and prompt summaries

- training-vanguard-v1.png: C:\Users\User\.codex\generated_images\01a11288-76ef-7302-a8dc-c1a689c648c5\exec-a721ef19-53ce-48d8-9433-ed08d2891948.png
Original stone frontline warrior, cropped dark hair, slate armor and amber seams; large readable face, dark navy background, no text/logo/frame.

- training-ranger-v1.png: C:\Users\User\.codex\generated_images\01a11288-76ef-7302-a8dc-c1a689c648c5\exec-f6a409d3-470f-46cb-a0e8-28ff735afd9c.png
Original woodland archer, adult woman, auburn hair, teal scarf and forest green leather armor; large readable face, dark navy background, no text/logo/frame.

- saika-magoichi-v1.png: C:\Users\User\.codex\generated_images\01a11288-76ef-7302-a8dc-c1a689c648c5\exec-9f195578-061e-4805-a525-82fcc915602d.png
Original game interpretation of Japanese matchlock gunslinger, mature face and short beard, worn headband, lacquered armor, red scarf; large readable face, dark navy background, no existing franchise costume, no text/logo/frame.

- date-masamune-v1.png: C:\Users\User\.codex\generated_images\01a11288-76ef-7302-a8dc-c1a689c648c5\exec-993d1d63-817e-422b-a995-4d6d8a236fad.png
Original one-eyed warrior in indigo and bronze armor, readable face on navy background, no franchise costume, text or frame.

- training-luminary-v1.png: C:\Users\User\.codex\generated_images\01a11288-76ef-7302-a8dc-c1a689c648c5\exec-bc786519-045e-45eb-8692-aa6400d08446.png
Original elder woman with silver hair, warm brown skin, ivory azure gold clothing and sun brooch, readable face on navy background.

- training-item-atlas-v1.png: C:\Users\User\.codex\generated_images\01a11288-76ef-7302-a8dc-c1a689c648c5\exec-e5b13c10-cba5-451a-b865-584eb0347171.png
Original four by four tactical fantasy icon grid; eight actual vanguard/ranger skills, five self-item effects, three future tiles explicitly unbound. No text, gap, frame or logo.

The runtime manifest has 29 stable-ID entries: five portraits, twelve first-atlas abilities, four matching apprentice aliases and eight second-atlas training abilities. Unknown identities clear stale artwork. Ward/blink/heal artwork does not create gameplay actions; D/F, ward and targeted item authority remain separately required.

2026-10-08 Primary inspected the actual D3D12 v3 production HUD and settings captures. Portraits and ability resources loaded, but portrait alignment stretched the square image and ability names remained internal IDs. The next source revision centers the portrait in its horizontal slot, validates actual arranged image aspect in the render fixture, supplies bounded human display names through the same stable-ID manifest, and reads ability/upgrade hints from the actual control profile. Unknown future IDs get readable spaced words without borrowing artwork. `tools/lua/lua.exe .ai-collab/verify-hud-manifest-assets.lua` passed all29 IDs/seven distinct square PNG resources/labels/tile coordinates. This source/asset check is not a new native acceptance; the v3 whole batch remains31 Success/1 Fail because of E390 pointer handling. New source awaits a frozen build after the Recall/item worker finishes.
