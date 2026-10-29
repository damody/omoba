# M6 第 1 批解鎖清單（工作線 A，2026-10-30）

由 `target/lineA/wave1.lua` 依 codegen-coverage 產生。每招落在三者之一：**已產生**、**共用待補**（列出缺的寫法與負責：A＝產生器、B＝ability_kit／host 或規則層）、**hook**（v3 標了 `hook`）。另檢查每招都有行為描述（events／passive／statuses）。分類依 D7：英雄技能＋道具合計 ≥2 招用到的寫法是共用；hook-candidates.md 複核後，第 1 批沒有 hook 候選。

## A01 `tokugawa_ieyasu`（3 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a01_realm_momentum` | 有 | 已產生 | |
| `a01_tosho_gongen` | 有 | 共用待補 | `selector.team:ally (types creep+hero+neutral+summon)`（A（產生器），8 招） |
| `a01_heaven_net` | 有 | 已產生 | |
| `a01_thunder_blade` | 有 | 已產生 | |
| `a01_mikawa_valor` | 有 | 共用待補 | `passive_trigger:periodic.interval`（B（ability_kit／host），25 招）、`selector.team:ally (types creep+hero+neutral+summon)`（A（產生器），8 招） |

## B20 `yamagata_masakage`（3 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b20_rock_rush` | 有 | 已產生 | |
| `b20_ruin_wave` | 有 | 已產生 | |
| `b20_tenacity` | 有 | 共用待補 | `action:set_mana`（B（ability_kit／host），8 招）、`passive:visibility`（B（ability_kit／host），20 招） |
| `b20_red_charge` | 有 | 已產生 | |

## B26 `okuma_tomohide`（2 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b26_charging_wave` | 有 | 共用待補 | `fn:ability_level`（A（產生器），14 招） |
| `b26_blood_fury` | 有 | 共用待補 | `action:remove_status without a declared status`（B（ability_kit／host），21 招）、`action:remove_status.dispel`（B（ability_kit／host），16 招）、`action:remove_status.team_of_source`（B（ability_kit／host），13 招）、`state:magic_immune`（B（規則層），21 招） |
| `b26_kozuke_valor` | 有 | 共用待補 | `event.damage.lethal`（B（ability_kit／host），1 招） |
| `b26_unstoppable` | 有 | 已產生 | |
| `b26_release_wave` | 有 | 已產生 | |

## C22 `sasaki_kojiro`（1 / 5 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c22_ganryu` | 有 | 共用待補 | `action:issue_order`（B（ability_kit／host），33 招）、`status_trigger:on_attack_windup`（B（ability_kit／host），13 招） |
| `c22_swallow_nest` | 有 | 共用待補 | `state:no_healthbar`（B（規則層），8 招）、`state:unselectable`（B（規則層），5 招） |
| `c22_flying_swallow` | 有 | 共用待補 | `status.stacking:independent`（B（ability_kit／host），21 招） |
| `c22_swallow_flash` | 有 | 已產生 | |
| `c22_dream_mist` | 有 | 共用待補 | `attr:level`（A（產生器），21 招） |

## A12 `otani_yoshitsugu`（2 / 9 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a12_insight_flash` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），10 招） |
| `a12_scheme` | 有 | 共用待補 | `action:set_display_stacks`（B（ability_kit／host），8 招） |
| `a12_mystic_pierce` | 有 | 共用待補 | `attr:max_mana`（A（產生器），3 招） |
| `a12_awe` | 有 | 已產生 | |
| `a12_mystic_arrow` | 有 | 共用待補 | `action:projectile.on_end`（B（ability_kit／host），2 招）、`action:set_display_stacks`（B（ability_kit／host），8 招）、`attr:attack_interval`（A（產生器），4 招）、`status.status_resistance`（B（ability_kit／host），16 招） |
| `a12_mystic_possession` | 有 | 共用待補 | `action:swap_abilities`（B（ability_kit／host），6 招）、`fn:ability_level`（A（產生器），14 招）、`fn:ability_of`（A（產生器），1 招） |
| `a12_insight_flash_t` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），10 招） |
| `a12_mystic_field` | 有 | 共用待補 | `fn:ability_level`（A（產生器），14 招）、`selector.team:ally (types creep+hero+neutral+summon)`（A（產生器），8 招） |
| `a12_awe_t` | 有 | 已產生 | |

## A14 `maeda_toshiie`（2 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a14_impact_spear` | 有 | 共用待補 | `action:issue_order`（B（ability_kit／host），33 招）、`action:knockback`（B（ability_kit／host），10 招） |
| `a14_divine_strike` | 有 | 已產生 | |
| `a14_stacked_spear` | 有 | 已產生 | |
| `a14_spear_fury` | 有 | 共用待補 | `stat:move_speed`（B（規則層），6 招） |

## A26 `nohime`（1 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `a26_volley` | 有 | 已產生 | |
| `a26_powder_kick` | 有 | 共用待補 | `action:knockback`（B（ability_kit／host），10 招） |
| `a26_twin_guns` | 有 | 共用待補 | `passive_trigger:on_attack_windup`（B（ability_kit／host），15 招） |
| `a26_scorching_bomb` | 有 | 共用待補 | `action:projectile.on_end`（B（ability_kit／host），2 招）、`event:on_phase_start`（B（ability_kit／host），9 招）、`expr: wanted a unit, got a point`（A（產生器），1 招） |

## B27 `kojima_sadaoki`（2 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `b27_horsemanship` | 有 | 已產生 | |
| `b27_mighty_strike` | 有 | 共用待補 | `action:issue_order`（B（ability_kit／host），33 招） |
| `b27_fearless_charge` | 有 | 共用待補 | `action:remove_status without a declared status`（B（ability_kit／host），21 招）、`action:remove_status.dispel`（B（ability_kit／host），16 招）、`action:remove_status.team_of_source`（B（ability_kit／host），13 招）、`state:magic_immune`（B（規則層），21 招） |
| `b27_rending_blade` | 有 | 已產生 | |

## C03 `shimazu_yoshihiro`（1 / 4 招已產生）

| 招式 | 行為描述 | 分類 | 缺的寫法（負責） |
|---|---|---|---|
| `c03_iron_shirt` | 有 | 共用待補 | `action:remove_status without a declared status`（B（ability_kit／host），21 招）、`action:remove_status.dispel`（B（ability_kit／host），16 招）、`action:remove_status.team_of_source`（B（ability_kit／host），13 招）、`state:magic_immune`（B（規則層），21 招） |
| `c03_quake_axe` | 有 | 已產生 | |
| `c03_thunder_wrath` | 有 | 共用待補 | `passive_trigger:on_attack_windup`（B（ability_kit／host），15 招） |
| `c03_iron_hammer` | 有 | 共用待補 | `fn:offset types`（A（產生器），5 招） |

## 彙總

- 已產生 17 招、共用待補 27 招、hook 0 招；缺行為描述 0 招。

### 依負責整理的待補寫法（解鎖第 1 批要做的事）

| 寫法 | 負責 | 第 1 批用到的英雄 | 全量招數 |
|---|---|---|---|
| `attr:attack_interval` | A（產生器） | A12 | 4 |
| `attr:level` | A（產生器） | C22 | 21 |
| `attr:max_mana` | A（產生器） | A12 | 3 |
| `expr: wanted a unit, got a point` | A（產生器） | A26 | 1 |
| `fn:ability_level` | A（產生器） | A12、B26 | 14 |
| `fn:ability_of` | A（產生器） | A12 | 1 |
| `fn:offset types` | A（產生器） | C03 | 5 |
| `selector.team:ally (types creep+hero+neutral+summon)` | A（產生器） | A01、A12 | 8 |
| `action:issue_order` | B（ability_kit／host） | A14、B27、C22 | 33 |
| `action:knockback` | B（ability_kit／host） | A12、A14、A26 | 10 |
| `action:projectile.on_end` | B（ability_kit／host） | A12、A26 | 2 |
| `action:remove_status without a declared status` | B（ability_kit／host） | B26、B27、C03 | 21 |
| `action:remove_status.dispel` | B（ability_kit／host） | B26、B27、C03 | 16 |
| `action:remove_status.team_of_source` | B（ability_kit／host） | B26、B27、C03 | 13 |
| `action:set_display_stacks` | B（ability_kit／host） | A12 | 8 |
| `action:set_mana` | B（ability_kit／host） | B20 | 8 |
| `action:swap_abilities` | B（ability_kit／host） | A12 | 6 |
| `event.damage.lethal` | B（ability_kit／host） | B26 | 1 |
| `event:on_phase_start` | B（ability_kit／host） | A26 | 9 |
| `passive:visibility` | B（ability_kit／host） | B20 | 20 |
| `passive_trigger:on_attack_windup` | B（ability_kit／host） | A26、C03 | 15 |
| `passive_trigger:periodic.interval` | B（ability_kit／host） | A01 | 25 |
| `status.stacking:independent` | B（ability_kit／host） | C22 | 21 |
| `status.status_resistance` | B（ability_kit／host） | A12 | 16 |
| `status_trigger:on_attack_windup` | B（ability_kit／host） | C22 | 13 |
| `stat:move_speed` | B（規則層） | A14 | 6 |
| `state:magic_immune` | B（規則層） | B26、B27、C03 | 21 |
| `state:no_healthbar` | B（規則層） | C22 | 8 |
| `state:unselectable` | B（規則層） | C22 | 5 |

