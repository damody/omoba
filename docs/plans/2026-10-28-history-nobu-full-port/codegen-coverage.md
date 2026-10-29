# 產生器還沒產生的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。

- 英雄技能：447 招中 116 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：3 / 97 位。
- 道具（items.dll，M7）：168 件中 66 件產生成 Rust。
- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。
- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。
- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。

## 1. 實作順序

### M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）（10 位）

開始時已全部產生：3 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 1 | `status_trigger:on_ability_executed` | ability_kit | 共用 | A34 | 14 招、3 件道具 | 4 / 10 |
| 2 | `action:leap` | ability_kit | 共用 | B29 | 3 招、0 件道具 | 4 / 10 |
| 3 | `action:reveal.ignore_terrain` | ability_kit | 共用 | B29 | 5 招、1 件道具 | 4 / 10 |
| 4 | `passive:aura` | ability_kit | 共用 | B29 | 11 招、22 件道具 | 5 / 10 |
| 5 | `action:burn_mana.source` | ability_kit | 共用 | B08 | 5 招、1 件道具 | 5 / 10 |
| 6 | `action:dodge_projectiles.from_team:any` | ability_kit | 共用 | B08 | 11 招、1 件道具 | 5 / 10 |
| 7 | `action:projectile.from:spot` | ability_kit | hook 候選 | B08 | 1 招、0 件道具 | 6 / 10 |
| 8 | `state:ranged_attack` | ability_kit | 共用 | B04 | 4 招、0 件道具 | 6 / 10 |
| 9 | `status.aura` | ability_kit | 共用 | B04 | 15 招、0 件道具 | 6 / 10 |
| 10 | `status_trigger:on_attack_launch` | ability_kit | 共用 | B04 | 9 招、1 件道具 | 7 / 10 |
| 11 | `action:projectile in a routine` | ability_kit | 共用 | B06 | 2 招、0 件道具 | 7 / 10 |
| 12 | `passive_trigger:on_order_targeted` | ability_kit | hook 候選 | B06 | 1 招、0 件道具 | 7 / 10 |
| 13 | `selector.team:ally (types creep+hero+neutral+summon)` | 產生器 | 共用 | B06 | 8 招、0 件道具 | 8 / 10 |
| 14 | `action:create_unit.control:none` | ability_kit | 共用 | C15 | 11 招、0 件道具 | 8 / 10 |
| 15 | `action:create_unit.facing` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 8 / 10 |
| 16 | `action:issue_order` | ability_kit | 共用 | C15 | 31 招、2 件道具 | 8 / 10 |
| 17 | `action:remove_unit` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 8 / 10 |
| 18 | `fn:offset types` | 產生器 | 共用 | C15 | 5 招、0 件道具 | 8 / 10 |
| 19 | `status.status_resistance` | ability_kit | 共用 | C15 | 16 招、0 件道具 | 9 / 10 |
| 20 | `action:delay in a routine` | ability_kit | hook 候選 | A07 | 1 招、0 件道具 | 9 / 10 |
| 21 | `action:set_ability_active` | ability_kit | 共用 | A07 | 13 招、0 件道具 | 9 / 10 |
| 22 | `action:swap_abilities` | ability_kit | 共用 | A07 | 6 招、0 件道具 | 9 / 10 |
| 23 | `attr:damage` | 產生器 | 共用 | A07 | 12 招、2 件道具 | 9 / 10 |
| 24 | `attr:level` | 產生器 | 共用 | A07 | 20 招、1 件道具 | 9 / 10 |
| 25 | `event:attack` | ability_kit | 共用 | A07 | 12 招、9 件道具 | 9 / 10 |
| 26 | `passive_trigger:on_attack_windup` | ability_kit | 共用 | A07 | 12 招、3 件道具 | 9 / 10 |
| 27 | `stat:slow_resistance_pct` | ability_kit | hook 候選 | A07 | 1 招、0 件道具 | 9 / 10 |
| 28 | `stat:status_resistance_pct` | ability_kit | 共用 | A07 | 2 招、1 件道具 | 9 / 10 |
| 29 | `status.stacking:independent` | ability_kit | 共用 | A07 | 18 招、3 件道具 | 10 / 10 |

### M6 第 1 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 30 | `passive_trigger:periodic.interval` | ability_kit | 共用 | A01 | 13 招、12 件道具 | 1 / 9 |
| 31 | `state:magic_immune` | ability_kit | 共用 | B27 | 20 招、1 件道具 | 2 / 9 |
| 32 | `event.damage.lethal` | ability_kit | hook 候選 | B26 | 1 招、0 件道具 | 3 / 9 |
| 33 | `action:set_mana` | ability_kit | 共用 | B20 | 7 招、1 件道具 | 3 / 9 |
| 34 | `passive:visibility` | ability_kit | 共用 | B20 | 20 招、0 件道具 | 4 / 9 |
| 35 | `selector.types:building+creep+hero+neutral+summon` | 產生器 | 共用 | A18 | 17 招、0 件道具 | 4 / 9 |
| 36 | `status.absorb` | ability_kit | 共用 | A18 | 6 招、1 件道具 | 5 / 9 |
| 37 | `action:knockback` | ability_kit | 共用 | A14 | 10 招、0 件道具 | 5 / 9 |
| 38 | `stat:move_speed` | ability_kit | 共用 | A14 | 6 招、0 件道具 | 6 / 9 |
| 39 | `state:no_healthbar` | ability_kit | 共用 | C22 | 8 招、0 件道具 | 6 / 9 |
| 40 | `state:unselectable` | ability_kit | 共用 | C22 | 5 招、0 件道具 | 6 / 9 |
| 41 | `status_trigger:on_attack_windup` | ability_kit | 共用 | C22 | 9 招、4 件道具 | 7 / 9 |
| 42 | `action:projectile.on_end` | ability_kit | 共用 | A26 | 2 招、0 件道具 | 7 / 9 |
| 43 | `event:on_phase_start` | ability_kit | 共用 | A26 | 9 招、0 件道具 | 7 / 9 |
| 44 | `expr: wanted a unit, got a point` | 產生器 | hook 候選 | A26 | 1 招、0 件道具 | 8 / 9 |
| 45 | `action:set_display_stacks` | ability_kit | 共用 | A12 | 6 招、2 件道具 | 8 / 9 |
| 46 | `attr:attack_interval` | 產生器 | 共用 | A12 | 1 招、3 件道具 | 8 / 9 |
| 47 | `attr:max_mana` | 產生器 | 共用 | A12 | 2 招、1 件道具 | 8 / 9 |
| 48 | `fn:ability_of` | 產生器 | hook 候選 | A12 | 1 招、0 件道具 | 9 / 9 |

### M6 第 2 批（10 位）

開始時已全部產生：1 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 49 | `attr:remaining` | 產生器 | hook 候選 | C09 | 1 招、0 件道具 | 2 / 10 |
| 50 | `attr:is_creep` | 產生器 | 共用 | C05 | 2 招、0 件道具 | 2 / 10 |
| 51 | `event:on_channel_end` | ability_kit | 共用 | C05 | 12 招、0 件道具 | 3 / 10 |
| 52 | `action:damage.flags:reflection` | ability_kit | 共用 | A16 | 5 招、1 件道具 | 3 / 10 |
| 53 | `event.damage.is_reflection` | ability_kit | 共用 | A16 | 3 招、1 件道具 | 3 / 10 |
| 54 | `selector.types:courier+creep+hero+neutral+summon+ward` | 產生器 | hook 候選 | A16 | 1 招、0 件道具 | 4 / 10 |
| 55 | `attr:int` | 產生器 | 共用 | A23 | 7 招、2 件道具 | 4 / 10 |
| 56 | `state:break` | ability_kit | 共用 | A23 | 2 招、0 件道具 | 4 / 10 |
| 57 | `state:mute` | ability_kit | 共用 | A23 | 6 招、0 件道具 | 5 / 10 |
| 58 | `attr:cast_range` | 產生器 | 共用 | A35 | 5 招、0 件道具 | 6 / 10 |
| 59 | `event:point` | ability_kit | 共用 | A19 | 2 招、0 件道具 | 6 / 10 |
| 60 | `fn:rotate` | 產生器 | 共用 | A19 | 9 招、0 件道具 | 6 / 10 |
| 61 | `fn:select types` | 產生器 | 共用 | A19 | 3 招、0 件道具 | 7 / 10 |
| 62 | `action:cue.at of a sym:landing` | ability_kit | hook 候選 | C01 | 1 招、0 件道具 | 7 / 10 |
| 63 | `action:random_pick` | ability_kit | 共用 | C01 | 2 招、0 件道具 | 7 / 10 |
| 64 | `attr:base_magic_resist` | 產生器 | 共用 | C01 | 6 招、0 件道具 | 7 / 10 |
| 65 | `expr: a position from a sym:landing` | 產生器 | hook 候選 | C01 | 1 招、0 件道具 | 8 / 10 |
| 66 | `action:zone` | ability_kit | 共用 | A30 | 3 招、0 件道具 | 8 / 10 |
| 67 | `attr:owner` | 產生器 | 共用 | A30 | 8 招、1 件道具 | 8 / 10 |
| 68 | `fn:has_tag of a non-event ability` | 產生器 | 共用 | A30 | 5 招、1 件道具 | 8 / 10 |
| 69 | `state:provides_vision` | ability_kit | 共用 | A30 | 4 招、0 件道具 | 9 / 10 |
| 70 | `attr:is_illusion` | 產生器 | 共用 | B21 | 10 招、10 件道具 | 9 / 10 |
| 71 | `attr:is_invulnerable` | 產生器 | hook 候選 | B21 | 1 招、0 件道具 | 9 / 10 |
| 72 | `event.damage.raw` | ability_kit | 共用 | B21 | 1 招、1 件道具 | 9 / 10 |
| 73 | `passive_trigger:on_heal_received` | ability_kit | hook 候選 | B21 | 1 招、0 件道具 | 10 / 10 |

### M6 第 3 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 74 | `state:attack_slow_immune` | ability_kit | hook 候選 | C14 | 1 招、0 件道具 | 1 / 9 |
| 75 | `status_trigger:periodic.filter` | ability_kit | 共用 | C14 | 3 招、0 件道具 | 2 / 9 |
| 76 | `status.reapply_if_removed` | ability_kit | hook 候選 | A22 | 1 招、0 件道具 | 2 / 9 |
| 77 | `status.unique_group` | ability_kit | 共用 | A22 | 2 招、1 件道具 | 3 / 9 |
| 78 | `fn:armor_multiplier` | 產生器 | 共用 | C20 | 5 招、2 件道具 | 3 / 9 |
| 79 | `status.scope` | ability_kit | 共用 | C20 | 6 招、3 件道具 | 4 / 9 |
| 80 | `attr:is_ready` | 產生器 | 共用 | A05 | 1 招、3 件道具 | 5 / 9 |
| 81 | `attr:stacks` | 產生器 | 共用 | C25 | 3 招、0 件道具 | 5 / 9 |
| 82 | `fn:ability_in_slot` | 產生器 | 共用 | C25 | 2 招、0 件道具 | 6 / 9 |
| 83 | `action:projectile.from:spawn` | ability_kit | hook 候選 | B31 | 1 招、0 件道具 | 6 / 9 |
| 84 | `fn:is_visible_to` | 產生器 | 共用 | B31 | 3 招、3 件道具 | 6 / 9 |
| 85 | `fn:random_int` | 產生器 | 共用 | B31 | 4 招、0 件道具 | 6 / 9 |
| 86 | `selector.team:ally (apply_status)` | 產生器 | hook 候選 | B31 | 1 招、0 件道具 | 7 / 9 |
| 87 | `attr:mana_pct` | 產生器 | 共用 | C11 | 2 招、1 件道具 | 7 / 9 |
| 88 | `expr of type nil` | 產生器 | 共用 | C11 | 15 招、0 件道具 | 7 / 9 |
| 89 | `passive_trigger:on_kill` | ability_kit | 共用 | C11 | 8 招、5 件道具 | 7 / 9 |
| 90 | `selector.flag:include_hidden` | 產生器 | 共用 | C11 | 3 招、0 件道具 | 7 / 9 |
| 91 | `selector.shape:global` | 產生器 | 共用 | C11 | 13 招、0 件道具 | 8 / 9 |
| 92 | `action:spend_mana` | ability_kit | 共用 | C04 | 4 招、0 件道具 | 8 / 9 |
| 93 | `action:toggle_ability` | ability_kit | 共用 | C04 | 5 招、0 件道具 | 8 / 9 |
| 94 | `fn:min of non-numbers` | 產生器 | 共用 | C04 | 4 招、5 件道具 | 8 / 9 |
| 95 | `selector.team:ally` | 產生器 | 共用 | C04 | 8 招、0 件道具 | 8 / 9 |
| 96 | `state:flying_for_pathing` | ability_kit | 共用 | C04 | 2 招、0 件道具 | 9 / 9 |

### M6 第 4 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 97 | `action:damage.flags:hp_removal` | ability_kit | hook 候選 | A24 | 1 招、0 件道具 | 0 / 9 |
| 98 | `action:face` | ability_kit | 共用 | A24 | 12 招、0 件道具 | 0 / 9 |
| 99 | `passive_trigger:on_created` | ability_kit | 共用 | A24 | 2 招、44 件道具 | 1 / 9 |
| 100 | `action:kill` | ability_kit | 共用 | B34 | 11 招、2 件道具 | 1 / 9 |
| 101 | `action:set_var` | ability_kit | 共用 | B34 | 11 招、0 件道具 | 1 / 9 |
| 102 | `status.vars` | ability_kit | 共用 | B34 | 19 招、0 件道具 | 1 / 9 |
| 103 | `token:vars` | 產生器 | 共用 | B34 | 13 招、2 件道具 | 2 / 9 |
| 104 | `attr:str` | 產生器 | 共用 | B03 | 2 招、0 件道具 | 2 / 9 |
| 105 | `selector.flag:exclude_invisible` | 產生器 | hook 候選 | B03 | 1 招、0 件道具 | 3 / 9 |
| 106 | `action:repeat in a routine` | ability_kit | 共用 | B30 | 2 招、0 件道具 | 3 / 9 |
| 107 | `attr:agi` | 產生器 | 共用 | B30 | 2 招、0 件道具 | 4 / 9 |
| 108 | `passive_trigger:on_attack_launch` | ability_kit | 共用 | A06 | 4 招、0 件道具 | 4 / 9 |
| 109 | `status_trigger:on_attack_miss` | ability_kit | hook 候選 | A06 | 1 招、0 件道具 | 4 / 9 |
| 110 | `target.type:unit_or_point` | 產生器 | 共用 | A06 | 3 招、0 件道具 | 5 / 9 |
| 111 | `action:create_illusion` | ability_kit | 共用 | B02 | 9 招、0 件道具 | 5 / 9 |
| 112 | `action:projectile.from:polar(tgt, $orbit_step * (k + 1), $orbit_radius)` | ability_kit | hook 候選 | B02 | 1 招、0 件道具 | 5 / 9 |
| 113 | `state type:point` | ability_kit | 共用 | B02 | 2 招、0 件道具 | 5 / 9 |
| 114 | `state:low_attack_priority` | ability_kit | 共用 | B02 | 7 招、0 件道具 | 6 / 9 |
| 115 | `action:remove_status.tags` | ability_kit | 共用 | C02 | 4 招、3 件道具 | 6 / 9 |
| 116 | `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | C02 | 3 招、0 件道具 | 6 / 9 |
| 117 | `fn:has_state:untargetable_by_spells` | 產生器 | 共用 | C02 | 6 招、3 件道具 | 7 / 9 |
| 118 | `action:create_unit.control:owner` | ability_kit | 共用 | B17 | 15 招、0 件道具 | 7 / 9 |
| 119 | `action:create_unit.overrides` | ability_kit | 共用 | B17 | 11 招、0 件道具 | 7 / 9 |
| 120 | `event.damage.is_attack` | ability_kit | 共用 | B17 | 2 招、0 件道具 | 7 / 9 |
| 121 | `state:disarm` | ability_kit | 共用 | B17 | 12 招、1 件道具 | 8 / 9 |
| 122 | `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | A36 | 3 招、0 件道具 | 8 / 9 |
| 123 | `expr: wanted a unit, got a sym:caster_hero` | 產生器 | 共用 | A36 | 2 招、0 件道具 | 8 / 9 |
| 124 | `fn:alive of a sym:caster_hero` | 產生器 | hook 候選 | A36 | 1 招、0 件道具 | 9 / 9 |

### M6 第 5 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 125 | `state:not_on_minimap` | ability_kit | 共用 | A20 | 4 招、0 件道具 | 2 / 9 |
| 126 | `action:set_time_of_day` | ability_kit | 共用 | B14 | 2 招、0 件道具 | 2 / 9 |
| 127 | `attr:is_night` | 產生器 | 共用 | B14 | 4 招、0 件道具 | 3 / 9 |
| 128 | `action:attack` | ability_kit | 共用 | B35 | 5 招、0 件道具 | 3 / 9 |
| 129 | `fn:has_status_tag` | 產生器 | 共用 | B35 | 5 招、5 件道具 | 4 / 9 |
| 130 | `fn:avg_attack_against` | 產生器 | 共用 | A25 | 3 招、0 件道具 | 4 / 9 |
| 131 | `op:%` | 產生器 | 共用 | A25 | 2 招、0 件道具 | 5 / 9 |
| 132 | `attr:is_ranged` | 產生器 | 共用 | A17 | 1 招、2 件道具 | 5 / 9 |
| 133 | `stat:vision_day` | ability_kit | 共用 | A17 | 4 招、0 件道具 | 5 / 9 |
| 134 | `stat:vision_night` | ability_kit | 共用 | A17 | 6 招、3 件道具 | 6 / 9 |
| 135 | `action:damage.flags:ignore_armor` | ability_kit | hook 候選 | C10 | 1 招、0 件道具 | 6 / 9 |
| 136 | `stat.when` | ability_kit | 共用 | C10 | 1 招、4 件道具 | 7 / 9 |
| 137 | `action:apply_status.vars` | ability_kit | 共用 | B05 | 8 招、0 件道具 | 7 / 9 |
| 138 | `action:cue.at of a err` | ability_kit | 共用 | B05 | 4 招、0 件道具 | 7 / 9 |
| 139 | `state:revealed` | ability_kit | 共用 | B05 | 4 招、0 件道具 | 8 / 9 |
| 140 | `selector.order:random` | 產生器 | 共用 | B18 | 6 招、1 件道具 | 8 / 9 |
| 141 | `selector.types:building+creep+neutral+summon` | 產生器 | hook 候選 | B18 | 1 招、0 件道具 | 9 / 9 |

### M6 第 6 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 142 | `action:refund` | ability_kit | 共用 | A33 | 2 招、0 件道具 | 2 / 9 |
| 143 | `state:physical_immune` | ability_kit | hook 候選 | A32 | 1 招、0 件道具 | 3 / 9 |
| 144 | `status.stacking:replace` | ability_kit | 共用 | B10 | 9 招、0 件道具 | 4 / 9 |
| 145 | `action:impulse` | ability_kit | hook 候選 | B25 | 1 招、0 件道具 | 4 / 9 |
| 146 | `attr:cooldown_remaining` | 產生器 | hook 候選 | B25 | 1 招、0 件道具 | 4 / 9 |
| 147 | `fn:max of non-numbers` | 產生器 | 共用 | B25 | 3 招、0 件道具 | 5 / 9 |
| 148 | `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 5 / 9 |
| 149 | `state:evade_disabled` | ability_kit | 共用 | C13 | 4 招、0 件道具 | 5 / 9 |
| 150 | `status_trigger:on_kill` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 6 / 9 |
| 151 | `action:cancel_delay` | ability_kit | 共用 | A10 | 2 招、0 件道具 | 6 / 9 |
| 152 | `action:delay.id` | ability_kit | 共用 | A10 | 2 招、0 件道具 | 6 / 9 |
| 153 | `action:swap_positions` | ability_kit | 共用 | A10 | 4 招、0 件道具 | 7 / 9 |
| 154 | `action:heal.source:owner_of(self)` | ability_kit | 共用 | C16 | 2 招、0 件道具 | 7 / 9 |
| 155 | `attr:dt` | 產生器 | 共用 | C16 | 2 招、0 件道具 | 7 / 9 |
| 156 | `fn:owner_of` | 產生器 | 共用 | C16 | 2 招、1 件道具 | 7 / 9 |
| 157 | `state:flying` | ability_kit | 共用 | C16 | 3 招、0 件道具 | 8 / 9 |
| 158 | `passive_trigger:periodic.filter` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 8 / 9 |
| 159 | `status_trigger:on_mana_spent` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 9 / 9 |

### M6 第 7 批（9 位）

開始時已全部產生：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 160 | `status_trigger:on_moved` | ability_kit | 共用 | C12 | 3 招、0 件道具 | 3 / 9 |
| 161 | `state:melee_attack` | ability_kit | hook 候選 | A03 | 1 招、0 件道具 | 4 / 9 |
| 162 | `stat:outgoing_damage_pct` | ability_kit | 共用 | B32 | 4 招、1 件道具 | 5 / 9 |
| 163 | `requires.condition` | ability_kit | 共用 | A31 | 5 招、0 件道具 | 6 / 9 |
| 164 | `selector.team:both` | 產生器 | 共用 | C23 | 6 招、0 件道具 | 7 / 9 |
| 165 | `fn:status_var` | 產生器 | 共用 | B36 | 3 招、0 件道具 | 7 / 9 |
| 166 | `selector.flag:exclude_illusions` | 產生器 | hook 候選 | B36 | 1 招、0 件道具 | 7 / 9 |
| 167 | `selector.team:ally_not_self` | 產生器 | 共用 | B36 | 2 招、0 件道具 | 8 / 9 |
| 168 | `action:chance.counter_var` | ability_kit | 共用 | C21 | 2 招、6 件道具 | 8 / 9 |
| 169 | `action:chance.pity_after` | ability_kit | 共用 | C21 | 2 招、7 件道具 | 8 / 9 |
| 170 | `attr:attack_speed_mult` | 產生器 | 共用 | C21 | 1 招、1 件道具 | 8 / 9 |
| 171 | `fn:count_in_radius` | 產生器 | 共用 | C21 | 1 招、1 件道具 | 9 / 9 |

### M6 第 8 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 172 | `state:untargetable_by_spells` | ability_kit | hook 候選 | C19 | 1 招、0 件道具 | 2 / 9 |
| 173 | `event:on_channel_tick` | ability_kit | 共用 | B09 | 3 招、0 件道具 | 3 / 9 |
| 174 | `action:end_channel` | ability_kit | 共用 | B11 | 2 招、0 件道具 | 4 / 9 |
| 175 | `state:block_disabled` | ability_kit | hook 候選 | A09 | 1 招、0 件道具 | 4 / 9 |
| 176 | `state:hex` | ability_kit | hook 候選 | A09 | 1 招、0 件道具 | 5 / 9 |
| 177 | `fn:enemy_team` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 5 / 9 |
| 178 | `fn:is_ally` | 產生器 | 共用 | B12 | 3 招、0 件道具 | 5 / 9 |
| 179 | `fn:team_of` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 6 / 9 |
| 180 | `fn:floor of a err` | 產生器 | hook 候選 | A04 | 1 招、0 件道具 | 6 / 9 |
| 181 | `state type:vec` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 6 / 9 |
| 182 | `status_trigger:on_refreshed` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 183 | `attr:attack_target` | 產生器 | hook 候選 | A28 | 1 招、0 件道具 | 7 / 9 |
| 184 | `expr: wanted a bool, got a sym:heal_chain` | 產生器 | hook 候選 | A28 | 1 招、0 件道具 | 7 / 9 |
| 185 | `stat:physical_block_chance` | ability_kit | 共用 | A28 | 1 招、2 件道具 | 8 / 9 |
| 186 | `action:projectile.from:g` | ability_kit | hook 候選 | B23 | 1 招、0 件道具 | 8 / 9 |
| 187 | `action:reveal.teams:enemy` | ability_kit | 共用 | B23 | 2 招、0 件道具 | 8 / 9 |
| 188 | `fn:is_template` | 產生器 | 共用 | B23 | 6 招、0 件道具 | 8 / 9 |
| 189 | `passive_trigger:on_moved` | ability_kit | hook 候選 | B23 | 1 招、0 件道具 | 8 / 9 |
| 190 | `selector.flag:include_dead` | 產生器 | 共用 | B23 | 2 招、0 件道具 | 9 / 9 |

### M6 第 9 批（13 位）

開始時已全部產生：0 位；做完本批的項目後：13 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 191 | `selector.relative_to` | 產生器 | 共用 | C07 | 2 招、0 件道具 | 0 / 13 |
| 192 | `stat:heal_reduction_pct` | ability_kit | 共用 | C07 | 2 招、2 件道具 | 1 / 13 |
| 193 | `fn:alive of a err` | 產生器 | hook 候選 | B33 | 1 招、0 件道具 | 1 / 13 |
| 194 | `fn:edge_distance` | 產生器 | 共用 | B33 | 4 招、0 件道具 | 2 / 13 |
| 195 | `fn:orb_slot_free` | 產生器 | 共用 | A21 | 2 招、0 件道具 | 2 / 13 |
| 196 | `status.forced_move` | ability_kit | 共用 | A21 | 2 招、0 件道具 | 3 / 13 |
| 197 | `passive:orb_slot` | ability_kit | 共用 | B15 | 1 招、1 件道具 | 4 / 13 |
| 198 | `action:damage.credit` | ability_kit | hook 候選 | A11 | 1 招、0 件道具 | 4 / 13 |
| 199 | `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | A11 | 2 招、1 件道具 | 5 / 13 |
| 200 | `action:chain` | ability_kit | hook 候選 | B28 | 1 招、0 件道具 | 5 / 13 |
| 201 | `attr:is_invisible` | 產生器 | hook 候選 | B28 | 1 招、0 件道具 | 5 / 13 |
| 202 | `fn:has_state:revealed` | 產生器 | hook 候選 | B28 | 1 招、0 件道具 | 6 / 13 |
| 203 | `action:give_item` | ability_kit | 共用 | B07 | 2 招、0 件道具 | 6 / 13 |
| 204 | `action:pick_item` | ability_kit | 共用 | B07 | 3 招、0 件道具 | 6 / 13 |
| 205 | `fn:exists of a sym:item_a` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 6 / 13 |
| 206 | `fn:exists of a sym:item_b` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 7 / 13 |
| 207 | `action:modify_cooldown.item` | ability_kit | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 208 | `fn:ability_ready` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 209 | `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 210 | `fn:status_source` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 211 | `selector.flag:include_illusions` | 產生器 | 共用 | C18 | 5 招、0 件道具 | 8 / 13 |
| 212 | `action:create_pickup` | ability_kit | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 213 | `action:remove_item` | ability_kit | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 214 | `attr:gold` | 產生器 | 共用 | C06 | 2 招、0 件道具 | 8 / 13 |
| 215 | `attr:name` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 216 | `fn:exists of a sym:stolen` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 9 / 13 |
| 217 | `action:add_ability` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 9 / 13 |
| 218 | `action:cue.at of a sym:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 219 | `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 220 | `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 9 / 13 |
| 221 | `action:set_base_max_hp` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 222 | `token:state` | 產生器 | hook 候選 | B16 | 1 招、0 件道具 | 10 / 13 |
| 223 | `action:destroy_tree` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 10 / 13 |
| 224 | `action:regrow_trees` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 10 / 13 |
| 225 | `action:remove_status.exclude_tags` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 10 / 13 |
| 226 | `fn:circle_point` | 產生器 | 共用 | B22 | 2 招、0 件道具 | 10 / 13 |
| 227 | `fn:tree_alive` | 產生器 | hook 候選 | B22 | 1 招、0 件道具 | 10 / 13 |
| 228 | `status.immune_to` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 10 / 13 |
| 229 | `target.type:tree` | 產生器 | 共用 | B22 | 2 招、0 件道具 | 11 / 13 |
| 230 | `action:projectile.team:any` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 231 | `action:projectile.where` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 232 | `selector.team:ally (types building)` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 233 | `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 234 | `selector.types:building` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 235 | `state:cannot_miss` | ability_kit | 共用 | B24 | 1 招、3 件道具 | 11 / 13 |
| 236 | `state:taunt` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 12 / 13 |
| 237 | `action:remove_ability` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 238 | `expr: wanted a vec, got a sym:dir` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 239 | `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 240 | `selector.shape:segment` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 241 | `selector.to` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 242 | `selector.width` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 243 | `state:attack_immune` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 244 | `state:blind` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 13 / 13 |

## 2. 每位英雄、每招還缺什麼

### A02 `inahime`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a02_venom_arrow` | ✓ 已產生 |
| `a02_wind_prayer` | ✓ 已產生 |
| `a02_wind_drift` | ✓ 已產生 |
| `a02_shadow_meld` | ✓ 已產生 |

### A07 `honda_tadakatsu`（1 / 6 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a07_unscathed` | ✓ 已產生 |
| `a07_red_thunder` | `routine a07e_cast: action:delay in a routine` |
| `a07_shattering_spear` | `action:set_ability_active`、`attr:level`、`passive_trigger:on_attack_windup` |
| `a07_strongest` | `action:swap_abilities`、`attr:damage`、`event:attack`、`stat:slow_resistance_pct`、`stat:status_resistance_pct` |
| `a07_rouse` | `routine a07d_cast: status.stacking:independent` |
| `a07_rouse_free` | `routine a07d_cast: status.stacking:independent` |

### A34 `omatsu`（3 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a34_frost_bolt` | ✓ 已產生 |
| `a34_iceberg_strike` | ✓ 已產生 |
| `a34_frost_breath` | ✓ 已產生 |
| `a34_rapid_cooling` | `status_trigger:on_ability_executed` |

### B01 `saika_magoichi`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b01_sniper_mode` | ✓ 已產生 |
| `b01_saika_corps` | ✓ 已產生 |
| `b01_rain_iron_cannon` | ✓ 已產生 |
| `b01_three_stage` | ✓ 已產生 |

### B04 `date_masamune`（2 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b04_flame_trail_dash` | `action:dodge_projectiles.from_team:any`、`status.aura` |
| `b04_fire_blast` | ✓ 已產生 |
| `b04_flame_swiftness` | ✓ 已產生 |
| `b04_tanegashima` | `state:ranged_attack`、`status_trigger:on_attack_launch` |

### B06 `sanada_yukimura`（0 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b06_brave_warriors` | `selector.team:ally (types creep+hero+neutral+summon)` |
| `b06_flame_fangs` | `routine b06e_bolt: action:projectile in a routine` |
| `b06_mind_eye` | `passive_trigger:on_order_targeted` |
| `b06_first_warrior` | `status.aura` |

### B08 `asai_nagamasa`（3 / 5 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b08_fang_slash` | ✓ 已產生 |
| `b08_flash_cut` | `action:burn_mana.source`、`action:dodge_projectiles.from_team:any` |
| `b08_living_soul` | ✓ 已產生 |
| `b08_sky_fang` | ✓ 已產生 |
| `b08_endless_fang` | `action:dodge_projectiles.from_team:any`、`action:projectile.from:spot`、`action:reveal.ignore_terrain` |

### B29 `murakami_yoshikiyo`（1 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b29_ground_slam` | ✓ 已產生 |
| `b29_quake_axe` | `action:leap` |
| `b29_morale_boost` | `passive:aura` |
| `b29_earth_tremor` | `action:reveal.ignore_terrain` |

### C15 `tamako`（2 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `c15_scatter_shot` | ✓ 已產生 |
| `c15_burst_arrow` | `action:create_unit.control:none`、`fn:offset types` |
| `c15_precise_light` | ✓ 已產生 |
| `c15_hundred_arrows` | `action:create_unit.facing`、`action:issue_order`、`action:remove_unit`、`status.status_resistance`、`status_trigger:on_attack_launch` |

### C17 `oichi`（5 / 5 招）　已上架

| 招式 | 還缺 |
|---|---|
| `c17_light_chain` | ✓ 已產生 |
| `c17_victory_blessing` | ✓ 已產生 |
| `c17_divine_strike` | ✓ 已產生 |
| `c17_aurora_guard` | ✓ 已產生 |
| `c17_devotion` | ✓ 已產生 |

### A01 `tokugawa_ieyasu`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a01_realm_momentum` | ✓ 已產生 |
| `a01_tosho_gongen` | `selector.team:ally (types creep+hero+neutral+summon)` |
| `a01_heaven_net` | ✓ 已產生 |
| `a01_thunder_blade` | ✓ 已產生 |
| `a01_mikawa_valor` | `passive_trigger:periodic.interval`、`selector.team:ally (types creep+hero+neutral+summon)` |

### A03 `ishikawa_kazumasa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a03_pacing_gu` | `status.vars`、`status_trigger:on_moved` |
| `a03_spider_form` | `action:toggle_ability`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`state:melee_attack`、`status.scope` |
| `a03_enchanting_venom` | ✓ 已產生 |
| `a03_insect_swarm` | ✓ 已產生 |

### A04 `takenaka_shigeharu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a04_water_shikigami` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:issue_order`、`action:set_display_stacks`、`status_trigger:periodic.filter` |
| `a04_ice_path` | `attr:cast_range`、`event:on_phase_start`、`fn:floor of a err`、`state type:vec` |
| `a04_frost_shelter` | `status.absorb`、`status.vars`、`status_trigger:on_refreshed`、`token:vars` |
| `a04_frost_sparrow` | ✓ 已產生 |
| `a04_blizzard` | ✓ 已產生 |

### A05 `sakai_tadatsugu`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a05_valiant_power` | `status.stacking:independent` |
| `a05_mikawa_courage` | ✓ 已產生 |
| `a05_counter_stance` | `action:set_ability_active` |
| `a05_loyal_secret` | `action:set_mana`、`attr:is_illusion`、`attr:is_ready`、`state:no_healthbar` |
| `a05_crusade_affidavit` | `status.scope` |

### A06 `ii_naomasa`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a06_red_power` | `status.stacking:independent` |
| `a06_red_meteor` | ✓ 已產生 |
| `a06_red_flame_flow` | `action:set_ability_active`、`passive:aura`、`passive_trigger:on_attack_launch` |
| `a06_flame_of_anger` | `action:set_ability_active`、`target.type:unit_or_point` |
| `a06_red_fury` | `action:set_var`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`status.stacking:independent`、`status.vars`、`status_trigger:on_attack_miss`、`token:vars` |

### A08 `sakakibara_yasumasa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a08_skull_crusher` | ✓ 已產生 |
| `a08_war_cry` | ✓ 已產生 |
| `a08_indomitable` | `attr:base_magic_resist` |
| `a08_steadfast_general` | `passive:visibility`、`status.stacking:replace` |

### A09 `hachisuka_masakatsu`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a09_treant_summon` | `action:create_unit.control:none`、`action:create_unit.control:owner`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.stacking:independent` |
| `a09_lignify` | `state:block_disabled`、`state:disarm`、`state:evade_disabled`、`state:hex`、`state:mute` |
| `a09_forest_pulse` | `action:spend_mana` |
| `a09_deep_tentacles` | `action:create_unit.control:owner`、`attr:stacks`、`fn:max of non-numbers`、`passive:visibility`、`state:magic_immune` |

### A10 `hirate_masahide`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a10_blink_return` | `action:cancel_delay`、`action:delay.id` |
| `a10_phase_shift` | `action:cancel_delay`、`action:delay.id` |
| `a10_slaughter_array` | `event:point`、`fn:select types`、`passive:visibility` |
| `a10_swap` | `action:swap_positions` |
| `a10_shockwave` | `action:create_unit.control:none`、`attr:level`、`event:on_channel_end`、`expr: wanted a unit, got a sym:caster_hero` |

### A11 `saito_dosan`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a11_vortex` | `action:damage.credit`、`action:damage.flags:no_source_bonuses`、`selector.types:building+creep+hero+neutral+summon` |
| `a11_evil_gaze` | `action:apply_status.vars`、`event:on_channel_end`、`stat:move_speed`、`status.forced_move`、`status.status_resistance`、`status.vars` |
| `a11_old_fox` | `passive:aura`、`status_trigger:on_ability_executed` |
| `a11_myriad_forms` | `attr:int`、`status.aura`、`status.status_resistance` |

### A12 `otani_yoshitsugu`（2 / 9 招）

| 招式 | 還缺 |
|---|---|
| `a12_insight_flash` | `action:knockback` |
| `a12_scheme` | `action:set_display_stacks` |
| `a12_mystic_pierce` | `attr:max_mana` |
| `a12_awe` | ✓ 已產生 |
| `a12_mystic_arrow` | `action:projectile.on_end`、`action:set_display_stacks`、`attr:attack_interval`、`status.status_resistance` |
| `a12_mystic_possession` | `action:swap_abilities`、`fn:ability_of` |
| `a12_insight_flash_t` | `action:knockback` |
| `a12_mystic_field` | `selector.team:ally (types creep+hero+neutral+summon)` |
| `a12_awe_t` | ✓ 已產生 |

### A13 `hattori_hanzo`（1 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a13_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_launch` |
| `a13_shadow_illusion` | `action:create_illusion`、`action:cue.at of a sym:il`、`action:dodge_projectiles.of:il`、`action:face`、`action:swap_abilities`、`attr:level`、`expr: * of num and sym:me_slot`、`expr: + of num and sym:rot`、`expr: wanted a unit, got a sym:il`、`expr: ~= of num and sym:me_slot`、`fn:exists of a sym:il`、`fn:random_int` |
| `a13_substitution` | `action:swap_positions` |
| `a13_chain_sickle` | ✓ 已產生 |
| `a13_iga_ninjutsu` | `attr:level` |
| `a13_shadow_slash` | `action:attack`、`action:damage.flags:reflection`、`fn:avg_attack_against`、`selector.order:random`、`stat:outgoing_damage_pct`、`stat:vision_day`、`stat:vision_night`、`state:disarm`、`state:magic_immune`、`state:mute`、`status.status_resistance` |

### A14 `maeda_toshiie`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a14_impact_spear` | `action:issue_order`、`action:knockback` |
| `a14_divine_strike` | ✓ 已產生 |
| `a14_stacked_spear` | ✓ 已產生 |
| `a14_spear_fury` | `stat:move_speed` |

### A15 `nene`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a15_shadow_step` | `state:low_attack_priority`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_launch` |
| `a15_exhaust_seal` | `attr:level` |
| `a15_moon_strike` | `attr:level` |
| `a15_shadow_windmill` | `attr:damage`、`attr:level`、`event:attack` |
| `a15_shadow_moon_assault` | `attr:damage`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`status_trigger:on_attack_launch` |

### A16 `torii_mototada`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a16_bloody_grasp` | ✓ 已產生 |
| `a16_blood_link` | ✓ 已產生 |
| `a16_blood_aura` | `action:damage.flags:reflection`、`event.damage.is_reflection`、`passive_trigger:periodic.interval` |
| `a16_dark_blood_decree` | `fn:offset types`、`selector.types:courier+creep+hero+neutral+summon+ward` |

### A17 `sassa_narimasa`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a17_shrapnel` | ✓ 已產生 |
| `a17_rapid_fire` | `attr:damage`、`attr:is_ranged`、`event:attack`、`fn:min of non-numbers` |
| `a17_vital_shot` | `passive_trigger:on_attack_launch` |
| `a17_great_cannon` | `action:set_ability_active`、`attr:damage`、`event:attack`、`selector.types:building+creep+hero+neutral+summon`、`stat:vision_day`、`stat:vision_night` |

### A18 `inaba_ittetsu`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a18_obsessive_pressure` | `selector.types:building+creep+hero+neutral+summon` |
| `a18_suppress` | ✓ 已產生 |
| `a18_stubborn_tactics` | `passive_trigger:on_attack_windup`、`passive_trigger:periodic.interval`、`status.absorb` |
| `a18_by_force` | ✓ 已產生 |

### A19 `kuroda_yoshitaka`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a19_fire_stratagem` | `selector.types:building+creep+hero+neutral+summon` |
| `a19_chain_fire` | ✓ 已產生 |
| `a19_fire_echo` | `attr:int`、`event:point`、`fn:select types` |
| `a19_siege_fire` | `fn:rotate` |

### A20 `kuki_yoshitaka`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_pirate_domain` | `action:create_unit.limit_per_owner (no host cap for it)`、`action:remove_status.tags`、`attr:owner`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.aura`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_windup` |
| `a20_water_burst` | `status.stacking:independent`、`status.status_resistance` |
| `a20_pirate_boots` | `routine a20r_wave: action:repeat in a routine`、`routine a20r_wave: fn:rotate` |
| `a20_pirate_daimyo` | `stat:status_resistance_pct`、`status.unique_group` |

### A21 `zuiryuin_nisshu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_mana_steal` | ✓ 已產生 |
| `a21_half_blade` | `attr:level`、`fn:orb_slot_free`、`selector.types:building+creep+hero+neutral+summon`、`status_trigger:on_attack_windup` |
| `a21_energy_press` | ✓ 已產生 |
| `a21_pearl_disc` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:reveal.teams:enemy`、`selector.order:random`、`state:magic_immune`、`status.aura` |
| `a21_false_intel` | `action:issue_order`、`status.forced_move`、`status.status_resistance` |

### A22 `sengoku_hidehisa`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a22_flame_whirl` | `state:magic_immune` |
| `a22_flame_heart` | ✓ 已產生 |
| `a22_boiling_blood` | `passive:aura`、`passive_trigger:periodic.interval`、`status.unique_group` |
| `a22_torment` | `status.reapply_if_removed` |

### A23 `ukita_hideie`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a23_cursed_blood` | ✓ 已產生 |
| `a23_soul_bind` | `attr:int` |
| `a23_falling_fire` | `selector.types:building+creep+hero+neutral+summon`、`status.stacking:independent` |
| `a23_yomi_flame` | `state:break`、`state:mute` |

### A24 `sakuma_morimasa`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a24_blood_shield` | `passive:visibility`、`passive_trigger:on_created`、`passive_trigger:periodic.interval` |
| `a24_slaughter_shot` | `action:face` |
| `a24_blood_charge` | `routine a24_blood_price: action:damage.flags:hp_removal`、`routine a24d_sync: action:set_display_stacks`、`routine a24d_sync: status.absorb` |
| `a24_blood_resonance` | `action:reveal.ignore_terrain`、`passive:visibility`、`passive_trigger:periodic.interval` |
| `a24_bloodbath` | `attr:level` |

### A25 `oda_nobunaga`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a25_lightning_lure` | `fn:avg_attack_against` |
| `a25_calamity_thorns` | `status.stacking:independent` |
| `a25_heshikiri` | `passive_trigger:on_attack_windup` |
| `a25_crimson_barrier` | `fn:offset types`、`fn:rotate`、`op:%`、`selector.types:building+creep+hero+neutral+summon`、`state:magic_immune` |
| `a25_tenka_fubu` | `passive_trigger:on_kill` |

### A26 `nohime`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a26_volley` | ✓ 已產生 |
| `a26_powder_kick` | `action:knockback` |
| `a26_twin_guns` | `passive_trigger:on_attack_windup` |
| `a26_scorching_bomb` | `action:projectile.on_end`、`event:on_phase_start`、`expr: wanted a unit, got a point` |

### A27 `shibata_katsuie`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a27_unstoppable_advance` | `action:create_illusion`、`action:issue_order` |
| `a27_war_banner` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`selector.team:ally (types creep+hero+neutral+summon)` |
| `a27_way_of_warrior` | `passive_trigger:on_attack_windup` |
| `a27_tyrant_hammer` | ✓ 已產生 |
| `a27_oni_shibata` | `state:ranged_attack` |

### A28 `hashiba_hideyoshi`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_evil_art` | `stat:physical_block_chance`、`state:disarm`、`state:evade_disabled`、`state:mute` |
| `a28_creation` | `expr: wanted a bool, got a sym:heal_chain`、`fn:is_ally`、`selector.team:ally (types creep+hero+neutral+summon)` |
| `a28_binding_swamp` | ✓ 已產生 |
| `a28_phoenix_summon` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill` |
| `a28_heaven_fire` | ✓ 已產生 |
| `a28_spreading_fire` | `attr:attack_target`、`fn:has_status_tag`、`passive_trigger:periodic.interval` |

### A29 `ishida_mitsunari`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_sleeve_poison_arrow` | ✓ 已產生 |
| `a29_abyss_grip` | `state:disarm` |
| `a29_withering_aura` | `passive:aura`、`passive_trigger:periodic.filter`、`passive_trigger:periodic.interval`、`status_trigger:on_mana_spent` |
| `a29_vulture_swarm` | `action:cue.at of a err`、`action:heal.source:owner_of(self)`、`attr:dt`、`fn:owner_of`、`status.stacking:replace` |

### A30 `niwa_nagahide`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_wolf_mark` | `action:burn_mana.source`、`action:create_unit.control:none`、`action:issue_order`、`attr:owner`、`state:provides_vision` |
| `a30_earth_fang` | ✓ 已產生 |
| `a30_fire_sea` | `action:zone` |
| `a30_red_fuji` | `fn:has_tag of a non-event ability` |

### A31 `mori_ranmaru`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a31_echoing_rain` | ✓ 已產生 |
| `a31_shunpo` | `action:dodge_projectiles.from_team:any`、`requires.condition` |
| `a31_storm_arrow` | ✓ 已產生 |
| `a31_samidare` | `action:attack`、`event:on_channel_end`、`passive_trigger:on_attack_launch`、`selector.order:random`、`stat:outgoing_damage_pct` |

### A32 `takigawa_kazumasu`（0 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a32_flying_slash` | `action:leap`、`action:swap_abilities`、`fn:has_tag of a non-event ability` |
| `a32_second_slash` | `action:leap` |
| `a32_soul_drain` | `op:%` |
| `a32_phantom_blade` | `passive_trigger:on_attack_windup`、`state:disarm`、`state:physical_immune` |
| `a32_soul_blade` | `action:toggle_ability`、`fn:avg_attack_against` |
| `a32_asura_blade` | `action:knockback`、`target.type:unit_or_point` |

### A33 `honda_masanobu`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_dark_summoning` | `action:create_unit.control:owner` |
| `a33_ghost_step` | `action:set_display_stacks` |
| `a33_corrupting_aura` | `passive:aura` |
| `a33_deepen_wounds` | `action:issue_order`、`action:refund`、`fn:has_state:untargetable_by_spells`、`state:provides_vision`、`state:revealed` |

### A35 `shima_sakon`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_flame_raid` | ✓ 已產生 |
| `a35_sealing_blaze` | `attr:cast_range`、`state:break`、`state:mute` |
| `a35_flame_swiftness` | ✓ 已產生 |
| `a35_scorching_fire` | `action:burn_mana.source`、`action:issue_order`、`attr:cast_range` |

### A36 `maeno_nagayasu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a36_shadow_step` | `status_trigger:on_ability_executed` |
| `a36_spike_trap` | `action:create_unit.control:owner`、`action:create_unit.limit_per_owner (no host cap for it)`、`action:create_unit.overrides`、`state:magic_immune`、`state:no_healthbar`、`status.aura` |
| `a36_detonate` | `routine a36e_explode: action:kill`、`routine a36e_explode: expr: wanted a unit, got a sym:caster_hero`、`routine a36e_explode: fn:alive of a sym:caster_hero`、`routine a36e_explode: fn:select types` |
| `a36_bitter_brew` | ✓ 已產生 |
| `a36_deadly_strike` | ✓ 已產生 |

### B02 `fuma_kotaro`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b02_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_launch` |
| `b02_hazy_shadow` | `action:create_illusion`、`action:face`、`attr:level` |
| `b02_blazing_path` | `action:zone`、`state type:point` |
| `b02_eighteen_shuriken` | `action:projectile.from:polar(tgt, $orbit_step * (k + 1), $orbit_radius)` |
| `b02_four_thunders` | ✓ 已產生 |

### B03 `saito_tomonobu`（3 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b03_zhongkui_strike` | ✓ 已產生 |
| `b03_zhongkui_strike_mid` | ✓ 已產生 |
| `b03_zhongkui_strike_far` | ✓ 已產生 |
| `b03_echigo_vajra` | `action:spend_mana`、`attr:mana_pct`、`passive_trigger:on_kill` |
| `b03_heavy_blow` | `attr:str` |
| `b03_zhongkui_judgment` | `action:set_var`、`selector.flag:exclude_invisible`、`status.vars`、`token:vars` |

### B05 `mori_motonari`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_chain_lightning` | `action:apply_status.vars`、`action:cue.at of a err`、`action:set_var`、`status.stacking:independent`、`status.vars`、`token:vars` |
| `b05_heavenly_thunder` | ✓ 已產生 |
| `b05_three_arrows` | `attr:int`、`passive:aura`、`state:revealed` |
| `b05_holy_light_burst` | `event:on_phase_start`、`status.status_resistance` |

### B07 `ashikaga_yoshiaki`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b07_starfall` | `selector.types:building+creep+hero+neutral+summon` |
| `b07_battle_line` | `action:create_unit.control:owner`、`action:create_unit.limit_per_owner (no host cap for it)`、`action:create_unit.overrides`、`action:give_item`、`action:pick_item`、`action:set_mana`、`fn:exists of a sym:item_a`、`fn:exists of a sym:item_b`、`selector.types:building+creep+hero+neutral+summon`、`state:revealed` |
| `b07e_camp_regen` | `passive:aura` |
| `b07_glorious_momentum` | `passive:aura` |
| `b07_reign_over_all` | `selector.types:building+creep+hero+neutral+summon` |

### B09 `sanada_masayuki`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b09_backlash_rift` | `fn:status_var`、`status.vars`、`status_trigger:on_ability_executed` |
| `b09_mind_burst` | `event:on_channel_end`、`event:on_channel_tick` |
| `b09_will_shield` | `action:spend_mana` |
| `b09_mind_suppression` | `action:set_mana`、`selector.team:ally (types creep+hero+neutral+summon)`、`status.status_resistance` |

### B10 `miyoshi_nagayoshi`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b10_wrath_of_the_dead` | `status.stacking:replace` |
| `b10_corpse_assault` | ✓ 已產生 |
| `b10_demonic_war_spirit` | `passive:aura` |
| `b10_power_of_the_abyss` | `passive_trigger:periodic.interval` |

### B11 `kikuhime`（0 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b11_cherry_rain` | `attr:agi`、`attr:cast_range` |
| `b11_breath_control` | `action:end_channel`、`event:on_channel_end` |
| `b11_interception` | `routine b11r_proc: action:chance.counter_var`、`routine b11r_proc: action:chance.pity_after`、`routine b11r_spawn: action:create_illusion`、`routine b11r_spawn: expr: wanted a unit, got a sym:ill`、`routine b11r_spawn: stat:move_speed` |
| `b11_crow_raid` | `action:dodge_projectiles.from_team:any`、`action:issue_order`、`action:set_ability_active` |
| `b11_crow_strike` | `action:set_ability_active`、`attr:level`、`passive_trigger:on_attack_windup` |
| `b11_inherited_blood` | `passive_trigger:on_kill` |

### B12 `yamamoto_kansuke`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b12_reality_slide` | `fn:enemy_team`、`fn:is_ally`、`fn:team_of`、`selector.team:both` |
| `b12_floating_life` | `selector.team:ally (types creep+hero+neutral+summon)` |
| `b12_scheming_offense` | `attr:base_magic_resist`、`attr:int`、`passive:aura` |
| `b12_mad_heart` | ✓ 已產生 |

### B13 `momochi_sandayu`（1 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b13_sense_break` | ✓ 已產生 |
| `b13_hypnosis` | `state:low_attack_priority` |
| `b13_blood_disguise` | `passive_trigger:on_kill` |
| `b13_burrow` | `action:set_ability_active`、`stat:vision_night`、`state:disarm` |
| `b13_shukuchi` | `action:set_ability_active`、`requires.condition` |
| `b13_storm` | `selector.types:building+creep+hero+neutral+summon`、`status.aura` |

### B14 `katakura_kagetsuna`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_temporary_night` | `action:set_time_of_day`、`event:on_phase_start` |
| `b14_iai_slash` | `action:issue_order`、`attr:is_night`、`attr:str` |
| `b14_shadow_tactics` | `action:issue_order`、`attr:is_night`、`state:magic_immune` |
| `b14_attendant_insight` | `passive_trigger:on_attack_windup` |
| `b14_shadow_cut` | `attr:is_night`、`attr:level`、`passive_trigger:on_attack_windup`、`passive_trigger:periodic.interval`、`state:low_attack_priority`、`status_trigger:on_ability_executed` |

### B15 `imagawa_yoshimoto`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_tokai_giant` | `state:magic_immune` |
| `b15_flame_arrow` | `action:toggle_ability`、`attr:owner`、`fn:orb_slot_free`、`passive:orb_slot`、`status_trigger:on_attack_windup` |
| `b15_giant_hide` | ✓ 已產生 |
| `b15_multishot` | `passive_trigger:on_attack_launch` |
| `b15_bluff` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill`、`status.stacking:independent` |

### B16 `mochizuki_chiyome`（2 / 9 招）

| 招式 | 還缺 |
|---|---|
| `b16_flying_raid` | `action:dodge_projectiles.of:state.b16_moon`、`action:face`、`action:issue_order`、`attr:level` |
| `b16_falling_leaves` | ✓ 已產生 |
| `b16_swift_thunder` | `passive:visibility` |
| `b16_summon_hound` | `action:add_ability`、`action:create_unit.control:owner`、`action:create_unit.facing`、`action:kill`、`action:remove_unit`、`action:set_ability_active`、`fn:edge_distance` |
| `b16_phase_shift` | `action:dodge_projectiles.of:state.b16_moon`、`action:swap_positions`、`requires.condition` |
| `b16_beast_union` | `action:refund`、`action:set_base_max_hp`、`fn:edge_distance`、`passive_trigger:periodic.interval`、`token:state` |
| `b16_mm_phase_shift` | `action:cue.at of a sym:caster_hero`、`action:dodge_projectiles.of:caster_hero`、`action:swap_positions`、`requires.condition` |
| `b16_mm_sweep` | ✓ 已產生 |
| `b16_mm_stealth` | `passive_trigger:on_created`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_launch` |

### B17 `kirigakure_saizo`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_death_ward` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:issue_order`、`action:kill`、`event.damage.is_attack`、`state:disarm`、`state:magic_immune`、`state:no_healthbar`、`status.aura` |
| `b17_tooth_poison` | ✓ 已產生 |
| `b17_shadow_leap` | ✓ 已產生 |
| `b17_heartseeker` | `state:low_attack_priority`、`status.vars`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_launch` |

### B18 `koihime`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_earth_raid` | ✓ 已產生 |
| `b18_echo` | `action:set_var`、`selector.order:random`、`status.vars`、`token:vars` |
| `b18_earth_shelter` | ✓ 已產生 |
| `b18_earth_claw` | `action:apply_status.vars`、`action:create_unit.control:none`、`action:create_unit.overrides`、`action:issue_order`、`action:set_var`、`attr:is_illusion`、`fn:is_visible_to`、`selector.types:building+creep+hero+neutral+summon`、`selector.types:building+creep+neutral+summon`、`status.aura`、`status.vars`、`token:vars` |

### B19 `usami_sadamitsu`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b19_water_dragon` | `action:cue.at of a err`、`state type:point` |
| `b19_ice_fence` | ✓ 已產生 |
| `b19_echigo_tactics` | `action:set_mana`、`expr of type nil`、`selector.shape:global`、`selector.team:ally` |
| `b19_kuruma_gakari` | `event:on_channel_end` |

### B20 `yamagata_masakage`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b20_rock_rush` | ✓ 已產生 |
| `b20_ruin_wave` | ✓ 已產生 |
| `b20_tenacity` | `action:set_mana`、`passive:visibility` |
| `b20_red_charge` | ✓ 已產生 |

### B21 `baba_nobufusa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b21_sky_wave` | ✓ 已產生 |
| `b21_killing_intent` | ✓ 已產生 |
| `b21_rebel_armor` | `action:damage.flags:reflection`、`attr:is_invulnerable`、`event.damage.is_reflection`、`event.damage.raw`、`passive:visibility` |
| `b21_undying_demon` | `attr:is_illusion`、`passive:visibility`、`passive_trigger:on_heal_received`、`status.status_resistance` |

### B22 `taigen_sessai`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b22_natural_barrier` | `action:create_unit.control:none`、`fn:circle_point`、`selector.team:both`、`state:magic_immune` |
| `b22_force_of_nature` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:destroy_tree`、`state:flying`、`target.type:tree` |
| `b22_purify` | `action:kill`、`action:remove_status.tags`、`attr:is_illusion`、`fn:is_ally`、`status.immune_to` |
| `b22_wood_spirit` | `action:create_unit.control:none`、`action:remove_unit`、`action:reveal.ignore_terrain`、`fn:tree_alive`、`selector.types:building+creep+hero+neutral+summon`、`state:revealed`、`target.type:tree` |
| `b22_wrath_of_nature` | `action:create_unit.control:none`、`action:regrow_trees`、`action:remove_status.exclude_tags`、`action:remove_unit`、`action:reveal.ignore_terrain` |

### B23 `ise_hime`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_undead_pact` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill`、`action:set_display_stacks`、`action:set_mana`、`attr:owner`、`event:on_phase_start`、`expr of type nil`、`fn:is_template`、`passive:visibility`、`passive_trigger:on_moved`、`selector.flag:include_dead`、`selector.shape:global`、`selector.team:ally`、`selector.team:both`、`status_trigger:on_moved` |
| `b23_spirit_gathering` | `attr:owner`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally` |
| `b23_living_soul_shock` | `action:projectile.from:g`、`attr:owner`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally` |
| `b23_soul_feast` | `attr:owner`、`event:on_channel_end`、`event:on_channel_tick`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally`、`state:magic_immune` |
| `b23_heroic_spirit_blessing` | `action:reveal.teams:enemy`、`attr:level`、`attr:owner`、`event:on_channel_end`、`event:on_channel_tick`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally`、`status.aura` |

### B24 `akiyama_nobutomo`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b24_earth_stele` | `action:create_unit.overrides`、`action:kill`、`action:knockback`、`action:set_var`、`selector.flag:include_illusions`、`selector.order:random`、`selector.team:ally (types building)`、`selector.team:both`、`selector.types:boss+creep+neutral+summon`、`status.vars`、`token:vars` |
| `b24_taunt` | `action:issue_order`、`selector.flag:include_illusions`、`state:taunt` |
| `b24_stone_resonance` | `action:projectile.team:any`、`action:projectile.where`、`expr of type nil`、`selector.flag:include_illusions` |
| `b24_earth_prison` | `action:swap_abilities`、`fn:circle_point`、`selector.flag:include_illusions`、`selector.team:both`、`state:cannot_miss`、`state:mute` |
| `b24_shatter` | `action:kill`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally`、`selector.types:building` |

### B25 `honganji_kennyo`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b25_exorcism_strike` | `action:burn_mana.source` |
| `b25_exorcism_sutra` | `attr:cooldown_remaining`、`fn:has_status_tag`、`fn:max of non-numbers` |
| `b25_finger_of_death` | ✓ 已產生 |
| `b25_asura_hell` | `action:impulse` |

### B26 `okuma_tomohide`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b26_charging_wave` | ✓ 已產生 |
| `b26_blood_fury` | `state:magic_immune` |
| `b26_kozuke_valor` | `event.damage.lethal` |
| `b26_unstoppable` | ✓ 已產生 |
| `b26_release_wave` | ✓ 已產生 |

### B27 `kojima_sadaoki`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b27_horsemanship` | ✓ 已產生 |
| `b27_mighty_strike` | `action:issue_order` |
| `b27_fearless_charge` | `state:magic_immune` |
| `b27_rending_blade` | ✓ 已產生 |

### B28 `matsunaga_hisahide`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_phoenix_flame` | ✓ 已產生 |
| `b28_flame_shackle` | `action:apply_status.vars`、`action:end_channel`、`event:on_channel_end`、`fn:edge_distance`、`fn:max of non-numbers`、`status.vars`、`status_trigger:on_ability_executed`、`token:vars` |
| `b28_fire_stream` | `selector.types:building+creep+hero+neutral+summon` |
| `b28_serpent_fire` | `action:chain`、`attr:is_invisible`、`fn:has_state:revealed` |

### B30 `kakizaki_kageie`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_reverse_shield` | `action:set_var`、`fn:has_tag of a non-event ability`、`fn:min of non-numbers`、`status.vars`、`token:vars` |
| `b30_soul_breaker` | ✓ 已產生 |
| `b30_flash_strike` | `action:dodge_projectiles.from_team:any`、`action:issue_order`、`attr:agi`、`attr:max_mana` |
| `b30_purgatory_slash` | `routine b30t_ignite: action:repeat in a routine` |

### B31 `takeda_shingen`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_swift_wind` | `selector.team:ally (apply_status)`、`status.aura` |
| `b31_still_forest` | `action:projectile.from:spawn`、`attr:base_magic_resist`、`fn:random_int`、`fn:rotate`、`status.aura` |
| `b31_raging_fire` | `fn:is_visible_to`、`status.aura`、`status_trigger:periodic.filter` |
| `b31_immovable_mountain` | `state:magic_immune` |

### B32 `uesugi_kenshin`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_divine_slash` | `action:dodge_projectiles.from_team:any`、`action:issue_order` |
| `b32_divine_flash` | `action:dodge_projectiles.from_team:any` |
| `b32_divine_formation` | `action:damage.flags:reflection`、`status.scope` |
| `b32_war_god` | `passive:visibility`、`passive_trigger:on_kill`、`stat:outgoing_damage_pct`、`state:provides_vision` |
| `b32_bishamonten` | `attr:level` |

### B33 `mogami_yoshihime`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_corrosive_arrow` | ✓ 已產生 |
| `b33_spider_web` | `state:disarm` |
| `b33_voodoo_power` | ✓ 已產生 |
| `b33_spider_attack` | `action:apply_status.vars`、`action:create_unit.control:none`、`action:create_unit.facing`、`action:face`、`action:issue_order`、`action:kill`、`action:set_var`、`fn:alive of a err`、`fn:edge_distance`、`stat:heal_reduction_pct`、`state:flying`、`state:flying_for_pathing`、`status.stacking:replace`、`status.vars`、`token:vars` |

### B34 `takeda_katsuyori`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b34_wrath_axe` | ✓ 已產生 |
| `b34_spike_wall` | `action:create_unit.control:none`、`action:kill`、`fn:rotate`、`status.aura` |
| `b34_frenzy` | `action:set_var`、`status.vars`、`status_trigger:on_attack_windup`、`token:vars` |
| `b34_tenmoku_howl` | `status.stacking:independent` |

### B35 `kamiizumi_nobutsuna`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b35_sword_essence` | `status.stacking:independent` |
| `b35_isshun` | `action:face` |
| `b35_muto_dori` | `action:attack`、`attr:damage`、`attr:is_illusion`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`fn:random_int` |
| `b35_hiten` | `action:issue_order`、`status_trigger:on_attack_windup` |

### B36 `naoe_kanetsugu`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_naoe_seal` | `action:apply_status.vars`、`status.scope`、`status.stacking:replace`、`status.vars` |
| `b36_footsteps` | `fn:status_var` |
| `b36_heart_fence` | `selector.flag:exclude_illusions`、`selector.team:ally_not_self`、`status.absorb` |
| `b36_seven_spears` | `action:knockback` |
| `b36_aizen_myoo` | `attr:base_magic_resist` |

### C01 `akechi_mitsuhide`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c01_blood_blade` | `attr:base_magic_resist`、`status_trigger:on_attack_windup` |
| `c01_ground_flame` | `routine c01e_wave: status.stacking:independent` |
| `c01_blood_flame` | `passive_trigger:on_attack_windup`、`passive_trigger:periodic.interval` |
| `c01_honnoji` | `action:cue.at of a sym:landing`、`action:random_pick`、`expr: a position from a sym:landing`、`status.stacking:independent`、`status.status_resistance` |

### C02 `akechi_hidemitsu`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c02_mirage_strike` | `action:create_illusion`、`action:face`、`action:issue_order`、`attr:cast_range`、`attr:is_illusion` |
| `c02_phantom_move` | `action:create_illusion`、`action:dodge_projectiles.from_team:any`、`action:issue_order`、`expr: wanted a unit, got a sym:ill` |
| `c02_kyoka_suigetsu` | `action:create_illusion`、`action:issue_order`、`attr:is_creep`、`attr:is_illusion`、`fn:has_state:untargetable_by_spells` |
| `c02_culling_slash` | `action:remove_status.tags`、`attr:is_illusion` |

### C03 `shimazu_yoshihiro`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_iron_shirt` | `state:magic_immune` |
| `c03_quake_axe` | ✓ 已產生 |
| `c03_thunder_wrath` | `passive_trigger:on_attack_windup` |
| `c03_iron_hammer` | `fn:offset types` |

### C04 `izumo_no_okuni`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_swift_dance` | `expr of type nil`、`selector.shape:global`、`selector.team:ally` |
| `c04_withering_step` | `action:set_ability_active`、`action:spend_mana`、`action:toggle_ability`、`attr:int`、`fn:min of non-numbers` |
| `c04_gathering_dance` | `state:flying_for_pathing` |
| `c04_witch_dance` | `event:on_channel_end`、`status.status_resistance` |

### C05 `maeda_keiji`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c05_matsukaze` | ✓ 已產生 |
| `c05_thunder_dance` | ✓ 已產生 |
| `c05_thunder_spear` | `attr:is_creep` |
| `c05_flash_thunder` | `event:on_channel_end`、`event:on_phase_start` |

### C06 `ishikawa_goemon`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c06_quick_hands` | ✓ 已產生 |
| `c06_sleight_of_hand` | `action:give_item`、`action:pick_item`、`action:remove_item`、`attr:name`、`fn:exists of a sym:stolen` |
| `c06_lavish_strike` | `attr:gold`、`attr:is_illusion`、`fn:has_state:untargetable_by_spells`、`fn:min of non-numbers`、`passive:visibility`、`status.absorb` |
| `c06_murder_and_loot` | `passive:visibility`、`passive_trigger:on_kill` |
| `c06_grand_heist` | `action:create_pickup`、`action:issue_order`、`attr:gold` |
| `item_c06e` | ✓ 已產生 |

### C07 `tachibana_dosetsu`（0 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c07_heaven_thunder` | `action:zone` |
| `c07_thunder_shackle` | `action:apply_status.vars`、`event:on_channel_end`、`status.stacking:replace`、`status.status_resistance`、`status.vars`、`status_trigger:on_ability_executed`、`token:vars` |
| `c07_thunderbolt` | `selector.relative_to` |
| `c07_raijin_wrath` | `action:create_unit.control:none`、`attr:level`、`fn:has_tag of a non-event ability`、`fn:is_visible_to`、`stat:heal_reduction_pct`、`status.status_resistance` |
| `c07_storm_move` | `action:issue_order` |
| `c07_magic_shield` | `action:toggle_ability`、`stat:move_speed`、`state:magic_immune`、`status_trigger:on_attack_windup` |

### C08 `kato_danzo`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_ability_executed`、`status_trigger:on_attack_launch` |
| `c08_blood_hunt` | `action:damage.flags:no_source_bonuses`、`stat:move_speed`、`status.stacking:independent` |
| `c08_scatter_escape` | `action:dodge_projectiles.from_team:any`、`attr:level` |
| `c08_kite_chain` | `action:face`、`action:remove_status.tags`、`expr of type nil`、`expr: wanted a vec, got a sym:dir`、`fn:has_state:untargetable_by_spells`、`fn:has_tag of a non-event ability`、`fn:offset types`、`selector.relative_to`、`selector.shape:segment`、`selector.team:ally_not_self`、`selector.to`、`selector.width` |
| `c08_devour` | `action:add_ability`、`action:remove_ability`、`action:set_ability_active`、`action:set_var`、`attr:level`、`fn:ability_in_slot`、`fn:is_ability of a non-event ability`、`fn:random_int`、`fn:status_var`、`stat:vision_day`、`stat:vision_night`、`state:attack_immune`、`state:blind`、`state:magic_immune`、`status.status_resistance`、`status.vars`、`token:vars` |

### C09 `tachibana_ginchiyo`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c09_frost_thorn` | `status_trigger:on_attack_windup` |
| `c09_white_heat` | ✓ 已產生 |
| `c09_ice_soul_dance` | `action:dodge_projectiles.from_team:any`、`attr:remaining` |
| `c09_armor_break` | `passive_trigger:periodic.interval`、`status.status_resistance` |

### C10 `kosokabe_chikayasu`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c10_black_dragon_wave` | `target.type:unit_or_point` |
| `c10_energy_drain` | ✓ 已產生 |
| `c10_echo_sword` | `action:damage.flags:ignore_armor`、`attr:damage`、`event:attack`、`passive:visibility` |
| `c10_day_night_shift` | `action:set_time_of_day`、`attr:is_night` |
| `c10_formless_heaven` | `expr of type nil`、`selector.flag:include_hidden`、`selector.shape:global`、`stat.when`、`stat:vision_night` |

### C11 `tachibana_muneshige`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c11_raikiri` | `expr of type nil`、`selector.flag:include_hidden`、`selector.shape:global` |
| `c11_thunder_speed` | ✓ 已產生 |
| `c11_thunder_eye` | ✓ 已產生 |
| `c11_unrivaled` | `action:burn_mana.source`、`attr:mana_pct`、`expr of type nil`、`passive:visibility`、`passive_trigger:on_kill`、`selector.flag:include_hidden`、`selector.shape:global` |

### C12 `otomo_sorin`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c12_homing_lock` | `action:apply_status.vars`、`state:provides_vision`、`status.stacking:replace`、`status.vars`、`status_trigger:on_moved` |
| `c12_missile_launch` | `action:set_var` |
| `c12_space_rift` | `passive:visibility`、`status.absorb` |
| `c12_optical_mask` | ✓ 已產生 |
| `c12_annihilation_beam` | `fn:rotate` |

### C13 `ryuzoji_takanobu`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c13_siphon` | ✓ 已產生 |
| `c13_tornado_wave` | ✓ 已產生 |
| `c13_gale_grip` | `action:issue_order`、`event.damage.is_attack`、`passive:visibility`、`passive_trigger:on_kill`、`selector.types:building+creep+hero+neutral+summon`、`status_trigger:on_kill` |
| `c13_tornado_vortex` | `action:modify_cooldown.item_tag`、`attr:is_illusion`、`fn:has_state:untargetable_by_spells`、`fn:rotate`、`state:disarm`、`state:evade_disabled` |

### C14 `saito_yoshitatsu`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c14_fire_curse` | `action:issue_order` |
| `c14_fire_breath` | `status_trigger:periodic.filter` |
| `c14_fire_breath_dragon` | `routine c14e_breath: action:projectile in a routine` |
| `c14_dragon_vein` | ✓ 已產生 |
| `c14_dragon_form` | `action:swap_abilities`、`attr:damage`、`event:attack`、`selector.types:building+creep+hero+neutral+summon`、`state:attack_slow_immune`、`state:ranged_attack` |

### C16 `kashin_koji`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_soul_sever` | ✓ 已產生 |
| `c16_curse_shadow` | `action:create_unit.control:owner`、`stat:vision_day`、`stat:vision_night`、`state:magic_immune` |
| `c16_wandering_souls` | `action:cue.at of a err`、`action:heal.source:owner_of(self)`、`attr:dt`、`fn:owner_of`、`state:disarm`、`state:flying`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.stacking:replace` |
| `c16_phantom_sight` | `action:create_illusion`、`action:face`、`fn:has_state:untargetable_by_spells` |
| `c16_eight_gates` | `action:create_unit.control:owner`、`attr:base_magic_resist`、`attr:level`、`selector.types:building+creep+hero+neutral+summon`、`state:magic_immune` |

### C18 `kobayakawa_takakage`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c18_lone_rider` | `action:modify_cooldown.item`、`action:pick_item`、`expr of type nil`、`fn:exists of a sym:blink_item`、`fn:status_source`、`selector.flag:include_dead`、`selector.shape:global`、`state:disarm`、`state:evade_disabled` |
| `c18_twin_chariot` | `fn:ability_ready`、`status.scope` |
| `c18_three_blades` | ✓ 已產生 |
| `c18_chain_stratagem` | `action:issue_order`、`attr:int`、`attr:stacks`、`passive:visibility` |
| `c18_four_cannons` | `expr of type nil`、`selector.flag:include_illusions`、`selector.shape:global` |

### C19 `matsuhime`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c19_alluring_gaze` | `action:set_mana` |
| `c19_gentle_vow` | ✓ 已產生 |
| `c19_paralyzing_poison` | `passive:visibility` |
| `c19_surprise_raid` | `action:issue_order`、`event:on_phase_start`、`requires.condition` |
| `c19_inviolable` | `action:knockback`、`passive:visibility`、`state:untargetable_by_spells` |

### C20 `chosokabe_motochika`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c20_flame_aegis` | `action:damage.flags:reflection`、`event.damage.is_reflection`、`status.scope` |
| `c20_demon_blood` | ✓ 已產生 |
| `c20_flame_dance` | ✓ 已產生 |
| `c20_demon_child` | `attr:damage`、`event:attack`、`fn:armor_multiplier`、`state:ranged_attack` |

### C21 `miyamoto_musashi`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c21_crescent_slash` | `state:magic_immune` |
| `c21_chain_breaker` | `action:attack`、`action:face`、`action:set_ability_active`、`fn:rotate`、`stat:outgoing_damage_pct`、`state:disarm` |
| `c21_two_heavens` | `action:chance.counter_var`、`action:chance.pity_after`、`attr:attack_speed_mult`、`passive_trigger:on_attack_windup`、`status.stacking:replace` |
| `c21_soaring_quake` | `action:attack`、`action:face`、`fn:count_in_radius`、`fn:rotate`、`selector.order:random` |

### C22 `sasaki_kojiro`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c22_ganryu` | `action:issue_order`、`status_trigger:on_attack_windup` |
| `c22_swallow_nest` | `state:no_healthbar`、`state:unselectable` |
| `c22_flying_swallow` | `status.stacking:independent` |
| `c22_swallow_flash` | ✓ 已產生 |
| `c22_dream_mist` | `attr:level` |

### C23 `kato_kiyomasa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c23_tainted_frenzy` | ✓ 已產生 |
| `c23_malice_wall` | `action:create_unit.facing`、`action:face`、`selector.team:both`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.aura` |
| `c23_blood_for_blood` | ✓ 已產生 |
| `c23_gift_of_death` | `action:issue_order`、`action:kill`、`event:on_phase_start` |

### C24 `yagyu_muneyoshi`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c24_kamitori` | `stat:move_speed` |
| `c24_ukifune` | `action:knockback` |
| `c24_zenken` | `attr:damage`、`event:attack` |
| `c24_last_life` | `status.stacking:independent` |

### C25 `chacha`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c25_raid` | `action:knockback` |
| `c25_blood_blade` | `attr:stacks` |
| `c25_grievous_wound` | `action:random_pick`、`fn:ability_in_slot` |
| `c25_falling_sakura` | `event:on_phase_start` |

## 3. 已產生的招式在執行期回 RErr 的情況

產生的程式碼遇到資料沒預期的世界狀態時，施放回 `RErr`（host 回滾成本）。下表是已產生的招式裡各種情況出現的招數；未產生的招式是 stub，一律回 `RErr` 並列出 §2 的缺項。

| 情況 | 招數 |
|---|---|
| 施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR） | 97 |
| 召喚：附近放不下、lifetime 不在 0～600 秒 | 1 |
| 召喚：count 不是 1 | 1 |
| 視野：半徑或位置不合法 | 5 |
| 光鍊：host tether 規格不合法 | 1 |
| 友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離 | 1 |
| 運算式除以零 | 2 |
| 讀位置的單位已不在世界中 | 37 |
| 需要地點，但施放沒有地點也沒有單位目標 | 28 |

### 已產生但 host 沒有做到的地方

產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。

| 招式 | 沒有做到 |
|---|---|
| （無） | |

## 4. 每項寫法擋住多少

| 寫法 | 主要負責 | 分類 | 招式 | 道具 | 英雄 |
|---|---|---|---|---|---|
| `action:issue_order` | ability_kit | 共用 | 31 | 2 | 28 |
| `state:magic_immune` | ability_kit | 共用 | 20 | 1 | 19 |
| `status.vars` | ability_kit | 共用 | 19 | 0 | 18 |
| `attr:level` | 產生器 | 共用 | 20 | 1 | 16 |
| `passive:visibility` | ability_kit | 共用 | 20 | 0 | 16 |
| `status.stacking:independent` | ability_kit | 共用 | 18 | 3 | 15 |
| `selector.types:building+creep+hero+neutral+summon` | 產生器 | 共用 | 17 | 0 | 15 |
| `status.status_resistance` | ability_kit | 共用 | 16 | 0 | 14 |
| `status_trigger:on_ability_executed` | ability_kit | 共用 | 14 | 3 | 14 |
| `action:create_unit.control:owner` | ability_kit | 共用 | 15 | 0 | 13 |
| `status.aura` | ability_kit | 共用 | 15 | 0 | 13 |
| `passive_trigger:periodic.interval` | ability_kit | 共用 | 13 | 12 | 12 |
| `token:vars` | 產生器 | 共用 | 13 | 2 | 12 |
| `state:disarm` | ability_kit | 共用 | 12 | 1 | 12 |
| `action:face` | ability_kit | 共用 | 12 | 0 | 11 |
| `event:on_channel_end` | ability_kit | 共用 | 12 | 0 | 11 |
| `passive_trigger:on_attack_windup` | ability_kit | 共用 | 12 | 3 | 11 |
| `action:create_unit.overrides` | ability_kit | 共用 | 11 | 0 | 11 |
| `action:set_ability_active` | ability_kit | 共用 | 13 | 0 | 10 |
| `attr:damage` | 產生器 | 共用 | 12 | 2 | 10 |
| `event:attack` | ability_kit | 共用 | 12 | 9 | 10 |
| `action:kill` | ability_kit | 共用 | 11 | 2 | 10 |
| `action:set_var` | ability_kit | 共用 | 11 | 0 | 10 |
| `passive:aura` | ability_kit | 共用 | 11 | 22 | 10 |
| `action:create_unit.control:none` | ability_kit | 共用 | 11 | 0 | 9 |
| `action:dodge_projectiles.from_team:any` | ability_kit | 共用 | 11 | 1 | 9 |
| `action:knockback` | ability_kit | 共用 | 10 | 0 | 9 |
| `event:on_phase_start` | ability_kit | 共用 | 9 | 0 | 9 |
| `status.stacking:replace` | ability_kit | 共用 | 9 | 0 | 9 |
| `status_trigger:on_attack_windup` | ability_kit | 共用 | 9 | 4 | 9 |
| `expr of type nil` | 產生器 | 共用 | 15 | 0 | 8 |
| `attr:is_illusion` | 產生器 | 共用 | 10 | 10 | 8 |
| `fn:rotate` | 產生器 | 共用 | 9 | 0 | 8 |
| `status_trigger:on_attack_launch` | ability_kit | 共用 | 9 | 1 | 8 |
| `action:apply_status.vars` | ability_kit | 共用 | 8 | 0 | 8 |
| `passive_trigger:on_kill` | ability_kit | 共用 | 8 | 5 | 8 |
| `state:no_healthbar` | ability_kit | 共用 | 8 | 0 | 8 |
| `selector.shape:global` | 產生器 | 共用 | 13 | 0 | 7 |
| `action:create_illusion` | ability_kit | 共用 | 9 | 0 | 7 |
| `selector.team:ally (types creep+hero+neutral+summon)` | 產生器 | 共用 | 8 | 0 | 7 |
| `action:set_mana` | ability_kit | 共用 | 7 | 1 | 7 |
| `attr:int` | 產生器 | 共用 | 7 | 2 | 7 |
| `state:low_attack_priority` | ability_kit | 共用 | 7 | 0 | 7 |
| `action:swap_abilities` | ability_kit | 共用 | 6 | 0 | 6 |
| `attr:base_magic_resist` | 產生器 | 共用 | 6 | 0 | 6 |
| `fn:has_state:untargetable_by_spells` | 產生器 | 共用 | 6 | 3 | 6 |
| `selector.order:random` | 產生器 | 共用 | 6 | 1 | 6 |
| `stat:move_speed` | ability_kit | 共用 | 6 | 0 | 6 |
| `stat:vision_night` | ability_kit | 共用 | 6 | 3 | 6 |
| `state:mute` | ability_kit | 共用 | 6 | 0 | 6 |
| `status.absorb` | ability_kit | 共用 | 6 | 1 | 6 |
| `status.scope` | ability_kit | 共用 | 6 | 3 | 6 |
| `action:set_display_stacks` | ability_kit | 共用 | 6 | 2 | 5 |
| `selector.team:both` | 產生器 | 共用 | 6 | 0 | 5 |
| `action:burn_mana.source` | ability_kit | 共用 | 5 | 1 | 5 |
| `action:damage.flags:reflection` | ability_kit | 共用 | 5 | 1 | 5 |
| `action:toggle_ability` | ability_kit | 共用 | 5 | 0 | 5 |
| `fn:armor_multiplier` | 產生器 | 共用 | 5 | 2 | 5 |
| `fn:has_status_tag` | 產生器 | 共用 | 5 | 5 | 5 |
| `fn:has_tag of a non-event ability` | 產生器 | 共用 | 5 | 1 | 5 |
| `fn:offset types` | 產生器 | 共用 | 5 | 0 | 5 |
| `state:unselectable` | ability_kit | 共用 | 5 | 0 | 5 |
| `attr:owner` | 產生器 | 共用 | 8 | 1 | 4 |
| `selector.team:ally` | 產生器 | 共用 | 8 | 0 | 4 |
| `action:attack` | ability_kit | 共用 | 5 | 0 | 4 |
| `action:reveal.ignore_terrain` | ability_kit | 共用 | 5 | 1 | 4 |
| `attr:cast_range` | 產生器 | 共用 | 5 | 0 | 4 |
| `requires.condition` | ability_kit | 共用 | 5 | 0 | 4 |
| `action:create_unit.facing` | ability_kit | 共用 | 4 | 0 | 4 |
| `action:cue.at of a err` | ability_kit | 共用 | 4 | 0 | 4 |
| `action:remove_status.tags` | ability_kit | 共用 | 4 | 3 | 4 |
| `action:spend_mana` | ability_kit | 共用 | 4 | 0 | 4 |
| `fn:min of non-numbers` | 產生器 | 共用 | 4 | 5 | 4 |
| `fn:random_int` | 產生器 | 共用 | 4 | 0 | 4 |
| `passive_trigger:on_attack_launch` | ability_kit | 共用 | 4 | 0 | 4 |
| `stat:outgoing_damage_pct` | ability_kit | 共用 | 4 | 1 | 4 |
| `stat:vision_day` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:evade_disabled` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:not_on_minimap` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:provides_vision` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:ranged_attack` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:revealed` | ability_kit | 共用 | 4 | 0 | 4 |
| `action:remove_unit` | ability_kit | 共用 | 4 | 0 | 3 |
| `action:swap_positions` | ability_kit | 共用 | 4 | 0 | 3 |
| `fn:edge_distance` | 產生器 | 共用 | 4 | 0 | 3 |
| `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | 3 | 0 | 3 |
| `action:pick_item` | ability_kit | 共用 | 3 | 0 | 3 |
| `action:zone` | ability_kit | 共用 | 3 | 0 | 3 |
| `attr:stacks` | 產生器 | 共用 | 3 | 0 | 3 |
| `event.damage.is_reflection` | ability_kit | 共用 | 3 | 1 | 3 |
| `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:avg_attack_against` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:is_ally` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:is_visible_to` | 產生器 | 共用 | 3 | 3 | 3 |
| `fn:max of non-numbers` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:select types` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:status_var` | 產生器 | 共用 | 3 | 0 | 3 |
| `state:flying` | ability_kit | 共用 | 3 | 0 | 3 |
| `status_trigger:on_moved` | ability_kit | 共用 | 3 | 0 | 3 |
| `status_trigger:periodic.filter` | ability_kit | 共用 | 3 | 0 | 3 |
| `target.type:unit_or_point` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:is_template` | 產生器 | 共用 | 6 | 0 | 2 |
| `selector.flag:include_illusions` | 產生器 | 共用 | 5 | 0 | 2 |
| `attr:is_night` | 產生器 | 共用 | 4 | 0 | 2 |
| `action:leap` | ability_kit | 共用 | 3 | 0 | 2 |
| `event:on_channel_tick` | ability_kit | 共用 | 3 | 0 | 2 |
| `selector.flag:include_hidden` | 產生器 | 共用 | 3 | 0 | 2 |
| `action:add_ability` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:chance.counter_var` | ability_kit | 共用 | 2 | 6 | 2 |
| `action:chance.pity_after` | ability_kit | 共用 | 2 | 7 | 2 |
| `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | 2 | 1 | 2 |
| `action:end_channel` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:give_item` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:heal.source:owner_of(self)` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:projectile in a routine` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:projectile.on_end` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:random_pick` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:refund` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:repeat in a routine` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:reveal.teams:enemy` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:set_time_of_day` | ability_kit | 共用 | 2 | 0 | 2 |
| `attr:agi` | 產生器 | 共用 | 2 | 0 | 2 |
| `attr:dt` | 產生器 | 共用 | 2 | 0 | 2 |
| `attr:is_creep` | 產生器 | 共用 | 2 | 0 | 2 |
| `attr:mana_pct` | 產生器 | 共用 | 2 | 1 | 2 |
| `attr:max_mana` | 產生器 | 共用 | 2 | 1 | 2 |
| `attr:str` | 產生器 | 共用 | 2 | 0 | 2 |
| `event.damage.is_attack` | ability_kit | 共用 | 2 | 0 | 2 |
| `event:point` | ability_kit | 共用 | 2 | 0 | 2 |
| `expr: wanted a unit, got a sym:caster_hero` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:ability_in_slot` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:circle_point` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:orb_slot_free` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:owner_of` | 產生器 | 共用 | 2 | 1 | 2 |
| `op:%` | 產生器 | 共用 | 2 | 0 | 2 |
| `passive_trigger:on_created` | ability_kit | 共用 | 2 | 44 | 2 |
| `selector.flag:include_dead` | 產生器 | 共用 | 2 | 0 | 2 |
| `selector.relative_to` | 產生器 | 共用 | 2 | 0 | 2 |
| `selector.team:ally_not_self` | 產生器 | 共用 | 2 | 0 | 2 |
| `stat:heal_reduction_pct` | ability_kit | 共用 | 2 | 2 | 2 |
| `stat:status_resistance_pct` | ability_kit | 共用 | 2 | 1 | 2 |
| `state type:point` | ability_kit | 共用 | 2 | 0 | 2 |
| `state:break` | ability_kit | 共用 | 2 | 0 | 2 |
| `state:flying_for_pathing` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.forced_move` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.unique_group` | ability_kit | 共用 | 2 | 1 | 2 |
| `action:cancel_delay` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:delay.id` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | 2 | 0 | 1 |
| `attr:gold` | 產生器 | 共用 | 2 | 0 | 1 |
| `target.type:tree` | 產生器 | 共用 | 2 | 0 | 1 |
| `action:chain` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:create_pickup` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:caster_hero` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:landing` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:damage.credit` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:damage.flags:hp_removal` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:damage.flags:ignore_armor` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:delay in a routine` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:destroy_tree` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:dodge_projectiles.of:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:impulse` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.item` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.from:g` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.from:polar(tgt, $orbit_step * (k + 1), $orbit_radius)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.from:spawn` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.from:spot` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.team:any` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.where` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:regrow_trees` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_ability` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_item` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_status.exclude_tags` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:set_base_max_hp` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `attr:attack_interval` | 產生器 | 共用 | 1 | 3 | 1 |
| `attr:attack_speed_mult` | 產生器 | 共用 | 1 | 1 | 1 |
| `attr:attack_target` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:cooldown_remaining` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:is_invisible` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:is_invulnerable` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:is_ranged` | 產生器 | 共用 | 1 | 2 | 1 |
| `attr:is_ready` | 產生器 | 共用 | 1 | 3 | 1 |
| `attr:name` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:remaining` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `event.damage.lethal` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `event.damage.raw` | ability_kit | 共用 | 1 | 1 | 1 |
| `expr: * of num and sym:me_slot` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: + of num and sym:rot` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: a position from a sym:landing` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: wanted a bool, got a sym:heal_chain` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: wanted a unit, got a point` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: wanted a unit, got a sym:il` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: wanted a vec, got a sym:dir` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `expr: ~= of num and sym:me_slot` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:ability_of` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:ability_ready` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:alive of a err` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:alive of a sym:caster_hero` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:count_in_radius` | 產生器 | 共用 | 1 | 1 | 1 |
| `fn:enemy_team` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:il` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:item_a` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:item_b` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:stolen` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:floor of a err` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:has_state:revealed` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:status_source` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:team_of` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:tree_alive` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `passive:orb_slot` | ability_kit | 共用 | 1 | 1 | 1 |
| `passive_trigger:on_heal_received` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `passive_trigger:on_moved` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `passive_trigger:on_order_targeted` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `passive_trigger:periodic.filter` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `selector.flag:exclude_illusions` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.flag:exclude_invisible` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.shape:segment` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.team:ally (apply_status)` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.team:ally (types building)` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.to` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:building` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:building+creep+neutral+summon` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:courier+creep+hero+neutral+summon+ward` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.width` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `stat.when` | ability_kit | 共用 | 1 | 4 | 1 |
| `stat:physical_block_chance` | ability_kit | 共用 | 1 | 2 | 1 |
| `stat:slow_resistance_pct` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state type:vec` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:attack_immune` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:attack_slow_immune` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:blind` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:block_disabled` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:cannot_miss` | ability_kit | 共用 | 1 | 3 | 1 |
| `state:hex` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:melee_attack` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:physical_immune` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:taunt` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:untargetable_by_spells` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.immune_to` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.reapply_if_removed` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_attack_miss` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_kill` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_mana_spent` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_refreshed` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `token:state` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `action:chance.counter` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:damage.type:event` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:projectile.lifetime` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:remove_status.status with limit` | ability_kit | 共用 | 0 | 3 | 0 |
| `attr:base_attack_range` | 產生器 | 共用 | 0 | 8 | 0 |
| `attr:effect_multiplier` | 產生器 | 共用 | 0 | 7 | 0 |
| `attr:illusion_damage` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `attr:projectile_speed` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:angle_between` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:is_hero_id` | 產生器 | 共用 | 0 | 7 | 0 |
| `passive_trigger:on_death` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_removed` | ability_kit | 共用 | 0 | 25 | 0 |
| `passive_trigger:periodic.first` | ability_kit | 共用 | 0 | 4 | 0 |
| `rust_handler without behavior` | ability_kit | 共用 | 0 | 9 | 0 |
| `selector.team:ally (types creep+neutral+summon)` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `stat:all_stats` | ability_kit | 共用 | 0 | 9 | 0 |
| `stat:cast_point_reduction_pct` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `stat:spell_lifesteal_pct` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.unique_rule` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status_trigger:on_order` | ability_kit | 共用 | 0 | 2 | 0 |
| `status_trigger:on_order_targeted` | ability_kit | 共用 | 0 | 2 | 0 |

## 5. hook 候選

只有 1 招（英雄技能與道具合計）用到的寫法。這些招式在 v3 標 `hook = "<方法名>"`、寫明行為描述與預期結果，由產生的 trait 手寫實作（`src/hooks/<招式>.rs`）；第 2 招也用到時改成共用、刪掉 hook。

| 寫法 | 招式 | 所屬 |
|---|---|---|
| `state:melee_attack` | `a03_spider_form` | A03 ishikawa_kazumasa |
| `status_trigger:on_refreshed` | `a04_frost_shelter` | A04 takenaka_shigeharu |
| `fn:floor of a err` | `a04_ice_path` | A04 takenaka_shigeharu |
| `state type:vec` | `a04_ice_path` | A04 takenaka_shigeharu |
| `status_trigger:on_attack_miss` | `a06_red_fury` | A06 ii_naomasa |
| `action:delay in a routine` | `a07_red_thunder` | A07 honda_tadakatsu |
| `stat:slow_resistance_pct` | `a07_strongest` | A07 honda_tadakatsu |
| `state:block_disabled` | `a09_lignify` | A09 hachisuka_masakatsu |
| `state:hex` | `a09_lignify` | A09 hachisuka_masakatsu |
| `action:damage.credit` | `a11_vortex` | A11 saito_dosan |
| `fn:ability_of` | `a12_mystic_possession` | A12 otani_yoshitsugu |
| `action:cue.at of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `action:dodge_projectiles.of:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: * of num and sym:me_slot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: + of num and sym:rot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: wanted a unit, got a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: ~= of num and sym:me_slot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `fn:exists of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `selector.types:courier+creep+hero+neutral+summon+ward` | `a16_dark_blood_decree` | A16 torii_mototada |
| `status.reapply_if_removed` | `a22_torment` | A22 sengoku_hidehisa |
| `action:damage.flags:hp_removal` | `a24_blood_charge` | A24 sakuma_morimasa |
| `expr: wanted a unit, got a point` | `a26_scorching_bomb` | A26 nohime |
| `expr: wanted a bool, got a sym:heal_chain` | `a28_creation` | A28 hashiba_hideyoshi |
| `attr:attack_target` | `a28_spreading_fire` | A28 hashiba_hideyoshi |
| `passive_trigger:periodic.filter` | `a29_withering_aura` | A29 ishida_mitsunari |
| `status_trigger:on_mana_spent` | `a29_withering_aura` | A29 ishida_mitsunari |
| `state:physical_immune` | `a32_phantom_blade` | A32 takigawa_kazumasu |
| `fn:alive of a sym:caster_hero` | `a36_detonate` | A36 maeno_nagayasu |
| `action:projectile.from:polar(tgt, $orbit_step * (k + 1), $orbit_radius)` | `b02_eighteen_shuriken` | B02 fuma_kotaro |
| `selector.flag:exclude_invisible` | `b03_zhongkui_judgment` | B03 saito_tomonobu |
| `passive_trigger:on_order_targeted` | `b06_mind_eye` | B06 sanada_yukimura |
| `fn:exists of a sym:item_a` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `fn:exists of a sym:item_b` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `action:projectile.from:spot` | `b08_endless_fang` | B08 asai_nagamasa |
| `fn:enemy_team` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `fn:team_of` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `action:set_base_max_hp` | `b16_beast_union` | B16 mochizuki_chiyome |
| `token:state` | `b16_beast_union` | B16 mochizuki_chiyome |
| `action:cue.at of a sym:caster_hero` | `b16_mm_phase_shift` | B16 mochizuki_chiyome |
| `action:dodge_projectiles.of:caster_hero` | `b16_mm_phase_shift` | B16 mochizuki_chiyome |
| `selector.types:building+creep+neutral+summon` | `b18_earth_claw` | B18 koihime |
| `attr:is_invulnerable` | `b21_rebel_armor` | B21 baba_nobufusa |
| `passive_trigger:on_heal_received` | `b21_undying_demon` | B21 baba_nobufusa |
| `action:destroy_tree` | `b22_force_of_nature` | B22 taigen_sessai |
| `status.immune_to` | `b22_purify` | B22 taigen_sessai |
| `fn:tree_alive` | `b22_wood_spirit` | B22 taigen_sessai |
| `action:regrow_trees` | `b22_wrath_of_nature` | B22 taigen_sessai |
| `action:remove_status.exclude_tags` | `b22_wrath_of_nature` | B22 taigen_sessai |
| `action:projectile.from:g` | `b23_living_soul_shock` | B23 ise_hime |
| `passive_trigger:on_moved` | `b23_undead_pact` | B23 ise_hime |
| `selector.team:ally (types building)` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:boss+creep+neutral+summon` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:building` | `b24_shatter` | B24 akiyama_nobutomo |
| `action:projectile.team:any` | `b24_stone_resonance` | B24 akiyama_nobutomo |
| `action:projectile.where` | `b24_stone_resonance` | B24 akiyama_nobutomo |
| `state:taunt` | `b24_taunt` | B24 akiyama_nobutomo |
| `action:impulse` | `b25_asura_hell` | B25 honganji_kennyo |
| `attr:cooldown_remaining` | `b25_exorcism_sutra` | B25 honganji_kennyo |
| `event.damage.lethal` | `b26_kozuke_valor` | B26 okuma_tomohide |
| `action:chain` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `attr:is_invisible` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `fn:has_state:revealed` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `action:projectile.from:spawn` | `b31_still_forest` | B31 takeda_shingen |
| `selector.team:ally (apply_status)` | `b31_swift_wind` | B31 takeda_shingen |
| `fn:alive of a err` | `b33_spider_attack` | B33 mogami_yoshihime |
| `selector.flag:exclude_illusions` | `b36_heart_fence` | B36 naoe_kanetsugu |
| `action:cue.at of a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `expr: a position from a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `action:create_pickup` | `c06_grand_heist` | C06 ishikawa_goemon |
| `action:remove_item` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `attr:name` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `fn:exists of a sym:stolen` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `action:remove_ability` | `c08_devour` | C08 kato_danzo |
| `fn:is_ability of a non-event ability` | `c08_devour` | C08 kato_danzo |
| `state:attack_immune` | `c08_devour` | C08 kato_danzo |
| `state:blind` | `c08_devour` | C08 kato_danzo |
| `expr: wanted a vec, got a sym:dir` | `c08_kite_chain` | C08 kato_danzo |
| `selector.shape:segment` | `c08_kite_chain` | C08 kato_danzo |
| `selector.to` | `c08_kite_chain` | C08 kato_danzo |
| `selector.width` | `c08_kite_chain` | C08 kato_danzo |
| `attr:remaining` | `c09_ice_soul_dance` | C09 tachibana_ginchiyo |
| `action:damage.flags:ignore_armor` | `c10_echo_sword` | C10 kosokabe_chikayasu |
| `status_trigger:on_kill` | `c13_gale_grip` | C13 ryuzoji_takanobu |
| `action:modify_cooldown.item_tag` | `c13_tornado_vortex` | C13 ryuzoji_takanobu |
| `state:attack_slow_immune` | `c14_dragon_form` | C14 saito_yoshitatsu |
| `action:modify_cooldown.item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:exists of a sym:blink_item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:status_source` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:ability_ready` | `c18_twin_chariot` | C18 kobayakawa_takakage |
| `state:untargetable_by_spells` | `c19_inviolable` | C19 matsuhime |
| `action:chance.counter` | `item_big_tachi` | 道具 |
| `stat:spell_lifesteal_pct` | `item_death_of_scepter` | 道具 |
| `status.unique_rule` | `item_foxfire_armor` | 道具 |
| `fn:angle_between` | `item_hannya_onimaru` | 道具 |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `action:projectile.lifetime` | `item_ronin_jitte` | 道具 |
| `action:damage.type:event` | `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 |
| `selector.team:ally (types creep+neutral+summon)` | `item_the_great_sword_of_disease` | 道具 |
| `stat:cast_point_reduction_pct` | `item_the_over_lightning_art_of_war` | 道具 |
| `attr:projectile_speed` | `item_wayumi2` | 道具 |

共 100 種寫法、71 招。

## 重跑

```
tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua
```

