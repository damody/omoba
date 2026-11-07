# M6 第 2 批解鎖清單（工作線 A，2026-10-30）

由 `target/lineA/wave2.lua` 依 codegen-coverage 產生。每招落在三者之一：**已產生**、**共用待補**（列出缺的寫法與負責：A＝產生器、B＝ability_kit／host 或規則層）、**hook**（v3 標了 `hook`）。另檢查每招都有行為描述（events／passive／statuses）。分類依 D7：英雄技能＋道具合計 ≥2 招用到的寫法是共用；hook-candidates.md 複核後，hook 候選依 hook-candidates.md。

## C05 `maeda_keiji`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c05_matsukaze` | 有 | 已產生 | |
| `c05_thunder_dance` | 有 | 已產生 | |
| `c05_thunder_spear` | 有 | 已產生 | |
| `c05_flash_thunder` | 有 | 共用待補 | `event:on_channel_end`（B（ability_kit／host），12 招）、`event:on_phase_start`（B（ability_kit／host），11 招） |

## C09 `tachibana_ginchiyo`（1 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c09_frost_thorn` | 有 | 共用待補 | `action:modify_attack`（B（ability_kit／host），21 招）、`action:spend_mana`（B（ability_kit／host），8 招）、`action:toggle_ability`（B（ability_kit／host），7 招） |
| `c09_white_heat` | 有 | 已產生 | |
| `c09_ice_soul_dance` | 有 | 共用待補 | `attr:remaining`（A（產生器），1 招） |
| `c09_armor_break` | 有 | 共用待補 | `status.status_resistance`（B（ability_kit／host），16 招） |

## A18 `inaba_ittetsu`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a18_obsessive_pressure` | 有 | 已產生 | |
| `a18_suppress` | 有 | 已產生 | |
| `a18_stubborn_tactics` | 有 | 共用待補 | `action:modify_attack`（B（ability_kit／host），21 招）、`status.absorb`（B（ability_kit／host），9 招） |
| `a18_by_force` | 有 | 已產生 | |

## B30 `kakizaki_kageie`（2 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b30_reverse_shield` | 有 | 共用待補 | `fn:has_tag:fountain of a unit`（A（產生器），2 招） |
| `b30_soul_breaker` | 有 | 已產生 | |
| `b30_flash_strike` | 有 | 已產生 | |
| `b30_purgatory_slash` | 有 | 共用待補 | `routine b30t_ignite: action:repeat in a routine`（B（ability_kit／host），2 招） |

## A16 `torii_mototada`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a16_bloody_grasp` | 有 | 已產生 | |
| `a16_blood_link` | 有 | 已產生 | |
| `a16_blood_aura` | 有 | 共用待補 | `action:damage.flags:reflection`（B（規則層），6 招）、`event.damage.is_reflection`（B（ability_kit／host），4 招） |
| `a16_dark_blood_decree` | 有 | 已產生 | |

## A24 `sakuma_morimasa`（0 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a24_blood_shield` | 有 | 共用待補 | `passive:visibility`（B（ability_kit／host），20 招）、`passive_trigger:on_created`（B（ability_kit／host），46 招） |
| `a24_slaughter_shot` | 有 | 共用待補 | `action:face`（B（ability_kit／host），13 招） |
| `a24_blood_charge` | 有 | 共用待補 | `routine a24_blood_price: action:damage.source with flags or credit`（B（ability_kit／host），2 招）、`routine a24d_sync: action:set_display_stacks`（B（ability_kit／host），10 招）、`routine a24d_sync: status.absorb`（B（ability_kit／host），9 招） |
| `a24_blood_resonance` | 有 | 共用待補 | `action:reveal.ignore_terrain`（B（ability_kit／host），7 招）、`passive:visibility`（B（ability_kit／host），20 招） |
| `a24_bloodbath` | 有 | 共用待補 | `routine a24_blood_price: action:damage.source with flags or credit`（B（ability_kit／host），2 招）、`routine a24d_sync: action:set_display_stacks`（B（ability_kit／host），10 招）、`routine a24d_sync: status.absorb`（B（ability_kit／host），9 招） |

## C01 `akechi_mitsuhide`（1 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c01_blood_blade` | 有 | 已產生 | |
| `c01_ground_flame` | 有 | 共用待補 | `routine c01e_wave: status.stacking:independent with states`（B（ability_kit／host），5 招） |
| `c01_blood_flame` | 有 | 共用待補 | `action:modify_attack`（B（ability_kit／host），21 招） |
| `c01_honnoji` | 有 | 共用待補 | `action:cue.at of a sym:landing`（B（ability_kit／host），1 招）、`action:random_pick`（B（ability_kit／host），2 招）、`expr: a position from a sym:landing`（A（產生器），1 招）、`status.stacking:independent with states`（B（ability_kit／host），5 招）、`status.status_resistance`（B（ability_kit／host），16 招） |

## A23 `ukita_hideie`（2 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a23_cursed_blood` | 有 | 已產生 | |
| `a23_soul_bind` | 有 | 已產生 | |
| `a23_falling_fire` | 有 | 共用待補 | `status.stacking:independent with triggers`（B（ability_kit／host），7 招） |
| `a23_yomi_flame` | 有 | 共用待補 | `state:break`（B（規則層），2 招） |

## A35 `shima_sakon`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a35_flame_raid` | 有 | 已產生 | |
| `a35_sealing_blaze` | 有 | 共用待補 | `state:break`（B（規則層），2 招） |
| `a35_flame_swiftness` | 有 | 已產生 | |
| `a35_scorching_fire` | 有 | 已產生 | |

## 彙總

- 已產生 18 招、共用待補 19 招、hook 0 招；缺行為描述 0 招。

### 依負責整理的待補寫法（解鎖第 1 批要做的事）

| 寫法 | 負責 | 第 1 批用到的英雄 | 全量招數 |
|---|---|---|---|
| `attr:remaining` | A（產生器） | C09 | 1 |
| `expr: a position from a sym:landing` | A（產生器） | C01 | 1 |
| `fn:has_tag:fountain of a unit` | A（產生器） | B30 | 2 |
| `action:cue.at of a sym:landing` | B（ability_kit／host） | C01 | 1 |
| `action:damage.source with flags or credit` | B（ability_kit／host） | A24 | 2 |
| `action:face` | B（ability_kit／host） | A24 | 13 |
| `action:modify_attack` | B（ability_kit／host） | A18、C01、C09 | 21 |
| `action:random_pick` | B（ability_kit／host） | C01 | 2 |
| `action:repeat in a routine` | B（ability_kit／host） | B30 | 2 |
| `action:reveal.ignore_terrain` | B（ability_kit／host） | A24 | 7 |
| `action:set_display_stacks` | B（ability_kit／host） | A24 | 10 |
| `action:spend_mana` | B（ability_kit／host） | C09 | 8 |
| `action:toggle_ability` | B（ability_kit／host） | C09 | 7 |
| `event.damage.is_reflection` | B（ability_kit／host） | A16 | 4 |
| `event:on_channel_end` | B（ability_kit／host） | C05 | 12 |
| `event:on_phase_start` | B（ability_kit／host） | C05 | 11 |
| `passive:visibility` | B（ability_kit／host） | A24 | 20 |
| `passive_trigger:on_created` | B（ability_kit／host） | A24 | 46 |
| `status.absorb` | B（ability_kit／host） | A18、A24 | 9 |
| `status.stacking:independent with states` | B（ability_kit／host） | C01 | 5 |
| `status.stacking:independent with triggers` | B（ability_kit／host） | A23 | 7 |
| `status.status_resistance` | B（ability_kit／host） | C01、C09 | 16 |
| `action:damage.flags:reflection` | B（規則層） | A16 | 6 |
| `state:break` | B（規則層） | A23、A35 | 2 |

