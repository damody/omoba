# M6 第 1 批解鎖清單（工作線 A，2026-10-30）

由 `target/lineA/wave1.lua` 依 codegen-coverage 產生。每招落在三者之一：**已產生**、**共用待補**（列出缺的寫法與負責：A＝產生器、B＝ability_kit／host 或規則層）、**hook**（v3 標了 `hook`）。另檢查每招都有行為描述（events／passive／statuses）。分類依 D7：英雄技能＋道具合計 ≥2 招用到的寫法是共用；hook-candidates.md 複核後，第 1 批沒有 hook 候選。

## A01 `tokugawa_ieyasu`（5 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a01_realm_momentum` | 有 | 已產生 | |
| `a01_tosho_gongen` | 有 | 已產生 | |
| `a01_heaven_net` | 有 | 已產生 | |
| `a01_thunder_blade` | 有 | 已產生 | |
| `a01_mikawa_valor` | 有 | 已產生 | |

## B20 `yamagata_masakage`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b20_rock_rush` | 有 | 已產生 | |
| `b20_ruin_wave` | 有 | 已產生 | |
| `b20_tenacity` | 有 | 共用待補 | `passive:visibility`（B（ability_kit／host），20 招） |
| `b20_red_charge` | 有 | 已產生 | |

## B26 `okuma_tomohide`（3 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b26_charging_wave` | 有 | 已產生 | |
| `b26_blood_fury` | 有 | 共用待補 | `state:magic_immune`（B（規則層），35 招） |
| `b26_kozuke_valor` | 有 | 共用待補 | `event.damage.lethal`（B（ability_kit／host），1 招） |
| `b26_unstoppable` | 有 | 已產生 | |
| `b26_release_wave` | 有 | 已產生 | |

## C22 `sasaki_kojiro`（2 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c22_ganryu` | 有 | 共用待補 | `action:modify_attack`（B（ability_kit／host），21 招）、`fn:has_status of an independent status (line B)`（A（產生器），2 招） |
| `c22_swallow_nest` | 有 | 共用待補 | `state:no_healthbar`（B（規則層），8 招）、`state:unselectable`（B（規則層），5 招） |
| `c22_flying_swallow` | 有 | 共用待補 | `status.stacking:independent with states`（B（ability_kit／host），5 招） |
| `c22_swallow_flash` | 有 | 已產生 | |
| `c22_dream_mist` | 有 | 已產生 | |

## A12 `otani_yoshitsugu`（5 / 9 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a12_insight_flash` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），13 招） |
| `a12_scheme` | 有 | 共用待補 | `action:set_display_stacks`（B（ability_kit／host），10 招） |
| `a12_mystic_pierce` | 有 | 已產生 | |
| `a12_awe` | 有 | 已產生 | |
| `a12_mystic_arrow` | 有 | 共用待補 | `action:set_display_stacks`（B（ability_kit／host），10 招）、`status.status_resistance`（B（ability_kit／host），16 招） |
| `a12_mystic_possession` | 有 | 已產生 | |
| `a12_insight_flash_t` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），13 招） |
| `a12_mystic_field` | 有 | 已產生 | |
| `a12_awe_t` | 有 | 已產生 | |

## A14 `maeda_toshiie`（2 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a14_impact_spear` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），13 招） |
| `a14_divine_strike` | 有 | 已產生 | |
| `a14_stacked_spear` | 有 | 已產生 | |
| `a14_spear_fury` | 有 | 共用待補 | `stat:move_speed`（B（規則層），8 招） |

## A26 `nohime`（1 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a26_volley` | 有 | 已產生 | |
| `a26_powder_kick` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），13 招） |
| `a26_twin_guns` | 有 | 共用待補 | `action:modify_attack`（B（ability_kit／host），21 招） |
| `a26_scorching_bomb` | 有 | 共用待補 | `event:on_phase_start`（B（ability_kit／host），11 招）、`expr: wanted a unit, got a point`（A（產生器），1 招） |

## B27 `kojima_sadaoki`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b27_horsemanship` | 有 | 已產生 | |
| `b27_mighty_strike` | 有 | 已產生 | |
| `b27_fearless_charge` | 有 | 共用待補 | `state:magic_immune`（B（規則層），35 招） |
| `b27_rending_blade` | 有 | 已產生 | |

## C03 `shimazu_yoshihiro`（2 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c03_iron_shirt` | 有 | 共用待補 | `state:magic_immune`（B（規則層），35 招） |
| `c03_quake_axe` | 有 | 已產生 | |
| `c03_thunder_wrath` | 有 | 共用待補 | `action:modify_attack`（B（ability_kit／host），21 招） |
| `c03_iron_hammer` | 有 | 已產生 | |

## 彙總

- 已產生 26 招、共用待補 18 招、hook 0 招；缺行為描述 0 招。

### 依負責整理的待補寫法（解鎖第 1 批要做的事）

| 寫法 | 負責 | 第 1 批用到的英雄 | 全量招數 |
|---|---|---|---|
| `expr: wanted a unit, got a point` | A（產生器） | A26 | 1 |
| `fn:has_status of an independent status (line B)` | A（產生器） | C22 | 2 |
| `action:knockback` | B（ability_kit／host） | A12、A14、A26 | 13 |
| `action:modify_attack` | B（ability_kit／host） | A26、C03、C22 | 21 |
| `action:set_display_stacks` | B（ability_kit／host） | A12 | 10 |
| `event.damage.lethal` | B（ability_kit／host） | B26 | 1 |
| `event:on_phase_start` | B（ability_kit／host） | A26 | 11 |
| `passive:visibility` | B（ability_kit／host） | B20 | 20 |
| `status.stacking:independent with states` | B（ability_kit／host） | C22 | 5 |
| `status.status_resistance` | B（ability_kit／host） | A12 | 16 |
| `stat:move_speed` | B（規則層） | A14 | 8 |
| `state:magic_immune` | B（規則層） | B26、B27、C03 | 35 |
| `state:no_healthbar` | B（規則層） | C22 | 8 |
| `state:unselectable` | B（規則層） | C22 | 5 |

