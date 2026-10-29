# 產生器還沒產生的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。

- 英雄技能：447 招中 138 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：3 / 97 位。
- 道具（items.dll，M7）：168 件中 53 件產生成 Rust。
- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。
- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。
- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。

## 1. 實作順序

### M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）（10 位）

開始時已全部產生：3 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 1 | `action:set_mana` | ability_kit | 共用 | A34 | 8 招、1 件道具 | 4 / 10 |
| 2 | `action:dodge_projectiles.from_team:any` | ability_kit | 共用 | B04 | 11 招、3 件道具 | 4 / 10 |
| 3 | `status.aura` | ability_kit | 共用 | B04 | 15 招、1 件道具 | 4 / 10 |
| 4 | `status_trigger:on_attack_launch` | ability_kit | 共用 | B04 | 9 招、1 件道具 | 5 / 10 |
| 5 | `action:burn_mana.source` | ability_kit | 共用 | B08 | 5 招、1 件道具 | 5 / 10 |
| 6 | `action:reveal.ignore_terrain` | ability_kit | 共用 | B08 | 5 招、2 件道具 | 6 / 10 |
| 7 | `action:leap` | ability_kit | 共用 | B29 | 3 招、0 件道具 | 6 / 10 |
| 8 | `passive:aura` | ability_kit | 共用 | B29 | 11 招、22 件道具 | 7 / 10 |
| 9 | `action:projectile in a routine` | ability_kit | 共用 | B06 | 2 招、0 件道具 | 7 / 10 |
| 10 | `passive_trigger:on_order_targeted` | ability_kit | hook 候選 | B06 | 1 招、0 件道具 | 7 / 10 |
| 11 | `selector.team:ally (types creep+hero+neutral+summon)` | 產生器 | 共用 | B06 | 8 招、13 件道具 | 8 / 10 |
| 12 | `action:create_unit.control:none` | ability_kit | 共用 | C15 | 11 招、4 件道具 | 8 / 10 |
| 13 | `action:create_unit.facing` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 8 / 10 |
| 14 | `action:issue_order.queue` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 8 / 10 |
| 15 | `action:remove_unit` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 8 / 10 |
| 16 | `fn:offset types` | 產生器 | 共用 | C15 | 5 招、3 件道具 | 8 / 10 |
| 17 | `status.status_resistance` | ability_kit | 共用 | C15 | 16 招、0 件道具 | 9 / 10 |
| 18 | `action:delay in a routine` | ability_kit | hook 候選 | A07 | 1 招、0 件道具 | 9 / 10 |
| 19 | `action:set_ability_active` | ability_kit | 共用 | A07 | 13 招、0 件道具 | 9 / 10 |
| 20 | `action:swap_abilities` | ability_kit | 共用 | A07 | 6 招、0 件道具 | 9 / 10 |
| 21 | `attr:damage` | 產生器 | 共用 | A07 | 12 招、2 件道具 | 9 / 10 |
| 22 | `event:attack` | ability_kit | 共用 | A07 | 12 招、9 件道具 | 9 / 10 |
| 23 | `passive_trigger:on_attack_windup` | ability_kit | 共用 | A07 | 12 招、3 件道具 | 9 / 10 |
| 24 | `stat:status_resistance_pct` | ability_kit | 共用 | A07 | 2 招、2 件道具 | 9 / 10 |
| 25 | `status.stacking:independent` | ability_kit | 共用 | A07 | 18 招、5 件道具 | 10 / 10 |

### M6 第 1 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 26 | `passive:visibility` | ability_kit | 共用 | B20 | 20 招、0 件道具 | 1 / 9 |
| 27 | `passive_trigger:periodic.interval` | ability_kit | 共用 | A01 | 13 招、12 件道具 | 2 / 9 |
| 28 | `state:magic_immune` | ability_kit | 共用 | B27 | 20 招、14 件道具 | 3 / 9 |
| 29 | `event.damage.lethal` | ability_kit | hook 候選 | B26 | 1 招、0 件道具 | 4 / 9 |
| 30 | `selector.types:building+creep+hero+neutral+summon` | 產生器 | 共用 | A18 | 17 招、2 件道具 | 4 / 9 |
| 31 | `status.absorb` | ability_kit | 共用 | A18 | 7 招、1 件道具 | 5 / 9 |
| 32 | `action:knockback` | ability_kit | 共用 | A14 | 10 招、3 件道具 | 5 / 9 |
| 33 | `stat:move_speed` | ability_kit | 共用 | A14 | 6 招、1 件道具 | 6 / 9 |
| 34 | `state:no_healthbar` | ability_kit | 共用 | C22 | 8 招、0 件道具 | 6 / 9 |
| 35 | `state:unselectable` | ability_kit | 共用 | C22 | 5 招、0 件道具 | 6 / 9 |
| 36 | `status_trigger:on_attack_windup` | ability_kit | 共用 | C22 | 9 招、4 件道具 | 7 / 9 |
| 37 | `action:projectile.on_end` | ability_kit | 共用 | A26 | 2 招、0 件道具 | 7 / 9 |
| 38 | `event:on_phase_start` | ability_kit | 共用 | A26 | 9 招、2 件道具 | 7 / 9 |
| 39 | `expr: wanted a unit, got a point` | 產生器 | hook 候選 | A26 | 1 招、0 件道具 | 8 / 9 |
| 40 | `action:set_display_stacks` | ability_kit | 共用 | A12 | 8 招、2 件道具 | 8 / 9 |
| 41 | `attr:attack_interval` | 產生器 | 共用 | A12 | 1 招、3 件道具 | 8 / 9 |
| 42 | `fn:ability_of` | 產生器 | hook 候選 | A12 | 1 招、0 件道具 | 9 / 9 |

### M6 第 2 批（10 位）

開始時已全部產生：1 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 43 | `event:on_channel_end` | ability_kit | 共用 | C05 | 12 招、0 件道具 | 2 / 10 |
| 44 | `attr:remaining` | 產生器 | hook 候選 | C09 | 1 招、0 件道具 | 3 / 10 |
| 45 | `state:break` | ability_kit | 共用 | A23 | 2 招、0 件道具 | 4 / 10 |
| 46 | `attr:cast_range` | 產生器 | 共用 | A35 | 5 招、0 件道具 | 5 / 10 |
| 47 | `action:damage.flags:reflection` | ability_kit | 共用 | A16 | 5 招、1 件道具 | 5 / 10 |
| 48 | `event.damage.is_reflection` | ability_kit | 共用 | A16 | 3 招、1 件道具 | 5 / 10 |
| 49 | `selector.types:courier+creep+hero+neutral+summon+ward` | 產生器 | hook 候選 | A16 | 1 招、0 件道具 | 6 / 10 |
| 50 | `event.damage.raw` | ability_kit | 共用 | B21 | 1 招、1 件道具 | 6 / 10 |
| 51 | `passive_trigger:on_heal_received` | ability_kit | hook 候選 | B21 | 1 招、0 件道具 | 7 / 10 |
| 52 | `action:cue.at of a sym:landing` | ability_kit | hook 候選 | C01 | 1 招、0 件道具 | 7 / 10 |
| 53 | `action:random_pick` | ability_kit | 共用 | C01 | 2 招、0 件道具 | 7 / 10 |
| 54 | `expr: a position from a sym:landing` | 產生器 | hook 候選 | C01 | 1 招、0 件道具 | 8 / 10 |
| 55 | `event:point` | ability_kit | 共用 | A19 | 2 招、0 件道具 | 8 / 10 |
| 56 | `fn:rotate` | 產生器 | 共用 | A19 | 9 招、3 件道具 | 8 / 10 |
| 57 | `fn:select types` | 產生器 | 共用 | A19 | 3 招、0 件道具 | 9 / 10 |
| 58 | `action:issue_order.force` | ability_kit | 共用 | A30 | 5 招、0 件道具 | 9 / 10 |
| 59 | `action:zone` | ability_kit | 共用 | A30 | 3 招、0 件道具 | 9 / 10 |
| 60 | `fn:has_tag of a non-event ability` | 產生器 | 共用 | A30 | 5 招、4 件道具 | 9 / 10 |
| 61 | `state:provides_vision` | ability_kit | 共用 | A30 | 4 招、1 件道具 | 10 / 10 |

### M6 第 3 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 62 | `status_trigger:periodic.filter` | ability_kit | 共用 | C14 | 3 招、0 件道具 | 2 / 9 |
| 63 | `status.reapply_if_removed` | ability_kit | 共用 | A22 | 1 招、13 件道具 | 2 / 9 |
| 64 | `status.unique_group` | ability_kit | 共用 | A22 | 2 招、1 件道具 | 3 / 9 |
| 65 | `fn:armor_multiplier` | 產生器 | 共用 | C20 | 5 招、2 件道具 | 3 / 9 |
| 66 | `status.scope` | ability_kit | 共用 | C20 | 6 招、3 件道具 | 4 / 9 |
| 67 | `attr:is_ready` | 產生器 | 共用 | A05 | 1 招、3 件道具 | 5 / 9 |
| 68 | `attr:stacks` | 產生器 | 共用 | C25 | 3 招、0 件道具 | 5 / 9 |
| 69 | `fn:ability_in_slot` | 產生器 | 共用 | C25 | 2 招、0 件道具 | 6 / 9 |
| 70 | `action:projectile.from:spawn` | ability_kit | hook 候選 | B31 | 1 招、0 件道具 | 6 / 9 |
| 71 | `fn:is_visible_to` | 產生器 | 共用 | B31 | 3 招、3 件道具 | 6 / 9 |
| 72 | `fn:random_int` | 產生器 | 共用 | B31 | 4 招、2 件道具 | 6 / 9 |
| 73 | `selector.team:ally (apply_status)` | 產生器 | 共用 | B31 | 1 招、4 件道具 | 7 / 9 |
| 74 | `action:spend_mana` | ability_kit | 共用 | C04 | 4 招、2 件道具 | 7 / 9 |
| 75 | `action:toggle_ability` | ability_kit | 共用 | C04 | 5 招、0 件道具 | 7 / 9 |
| 76 | `expr of type nil` | 產生器 | 共用 | C04 | 15 招、0 件道具 | 7 / 9 |
| 77 | `selector.shape:global` | 產生器 | 共用 | C04 | 13 招、0 件道具 | 7 / 9 |
| 78 | `selector.team:ally` | 產生器 | 共用 | C04 | 8 招、0 件道具 | 8 / 9 |
| 79 | `attr:mana_pct` | 產生器 | 共用 | C11 | 2 招、1 件道具 | 8 / 9 |
| 80 | `passive_trigger:on_kill` | ability_kit | 共用 | C11 | 8 招、5 件道具 | 8 / 9 |
| 81 | `selector.flag:include_hidden` | 產生器 | 共用 | C11 | 3 招、0 件道具 | 9 / 9 |

### M6 第 4 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 82 | `action:damage.flags:hp_removal` | ability_kit | 共用 | A24 | 2 招、0 件道具 | 0 / 9 |
| 83 | `action:face` | ability_kit | 共用 | A24 | 12 招、1 件道具 | 0 / 9 |
| 84 | `passive_trigger:on_created` | ability_kit | 共用 | A24 | 2 招、44 件道具 | 1 / 9 |
| 85 | `action:create_illusion` | ability_kit | 共用 | C02 | 9 招、0 件道具 | 1 / 9 |
| 86 | `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | C02 | 5 招、0 件道具 | 2 / 9 |
| 87 | `state type:point` | ability_kit | 共用 | B02 | 2 招、0 件道具 | 2 / 9 |
| 88 | `state:low_attack_priority` | ability_kit | 共用 | B02 | 7 招、0 件道具 | 3 / 9 |
| 89 | `action:create_unit.control:owner` | ability_kit | 共用 | B17 | 15 招、6 件道具 | 3 / 9 |
| 90 | `action:create_unit.overrides` | ability_kit | 共用 | B17 | 11 招、4 件道具 | 3 / 9 |
| 91 | `action:kill` | ability_kit | 共用 | B17 | 11 招、2 件道具 | 3 / 9 |
| 92 | `action:set_var` | ability_kit | 共用 | B17 | 13 招、0 件道具 | 3 / 9 |
| 93 | `event.damage.is_attack` | ability_kit | 共用 | B17 | 2 招、0 件道具 | 3 / 9 |
| 94 | `status.vars` | ability_kit | 共用 | B17 | 19 招、0 件道具 | 3 / 9 |
| 95 | `token:vars` | 產生器 | 共用 | B17 | 15 招、2 件道具 | 5 / 9 |
| 96 | `selector.flag:exclude_invisible` | 產生器 | hook 候選 | B03 | 1 招、0 件道具 | 6 / 9 |
| 97 | `action:repeat in a routine` | ability_kit | 共用 | B30 | 2 招、0 件道具 | 6 / 9 |
| 98 | `fn:min of non-numbers` | 產生器 | 共用 | B30 | 2 招、2 件道具 | 7 / 9 |
| 99 | `passive_trigger:on_attack_launch` | ability_kit | 共用 | A06 | 4 招、0 件道具 | 7 / 9 |
| 100 | `status_trigger:on_attack_miss` | ability_kit | hook 候選 | A06 | 1 招、0 件道具 | 7 / 9 |
| 101 | `target.type:unit_or_point` | 產生器 | 共用 | A06 | 3 招、1 件道具 | 8 / 9 |
| 102 | `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | A36 | 3 招、0 件道具 | 8 / 9 |
| 103 | `expr: wanted a unit, got a sym:caster_hero` | 產生器 | 共用 | A36 | 2 招、0 件道具 | 8 / 9 |
| 104 | `fn:alive of a sym:caster_hero` | 產生器 | hook 候選 | A36 | 1 招、0 件道具 | 9 / 9 |

### M6 第 5 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 105 | `action:set_time_of_day` | ability_kit | 共用 | B14 | 2 招、0 件道具 | 2 / 9 |
| 106 | `attr:owner` | 產生器 | hook 候選 | A20 | 1 招、0 件道具 | 2 / 9 |
| 107 | `state:not_on_minimap` | ability_kit | 共用 | A20 | 4 招、0 件道具 | 3 / 9 |
| 108 | `action:attack` | ability_kit | 共用 | B35 | 5 招、2 件道具 | 3 / 9 |
| 109 | `fn:has_status_tag` | 產生器 | 共用 | B35 | 5 招、11 件道具 | 4 / 9 |
| 110 | `stat:vision_day` | ability_kit | 共用 | A17 | 4 招、0 件道具 | 4 / 9 |
| 111 | `stat:vision_night` | ability_kit | 共用 | A17 | 6 招、3 件道具 | 5 / 9 |
| 112 | `action:damage.flags:ignore_armor` | ability_kit | hook 候選 | C10 | 1 招、0 件道具 | 5 / 9 |
| 113 | `stat.when` | ability_kit | 共用 | C10 | 1 招、4 件道具 | 6 / 9 |
| 114 | `fn:avg_attack_against` | 產生器 | 共用 | A25 | 3 招、0 件道具 | 6 / 9 |
| 115 | `op:%` | 產生器 | 共用 | A25 | 2 招、0 件道具 | 7 / 9 |
| 116 | `action:apply_status.vars` | ability_kit | 共用 | B05 | 8 招、0 件道具 | 7 / 9 |
| 117 | `action:cue.at of a err` | ability_kit | 共用 | B05 | 4 招、0 件道具 | 7 / 9 |
| 118 | `state:revealed` | ability_kit | 共用 | B05 | 4 招、6 件道具 | 8 / 9 |
| 119 | `selector.order:random` | 產生器 | 共用 | B18 | 6 招、1 件道具 | 8 / 9 |
| 120 | `selector.types:building+creep+neutral+summon` | 產生器 | 共用 | B18 | 1 招、1 件道具 | 9 / 9 |

### M6 第 6 批（9 位）

開始時已全部產生：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 121 | `action:refund` | ability_kit | 共用 | A33 | 2 招、2 件道具 | 3 / 9 |
| 122 | `status.stacking:replace` | ability_kit | 共用 | B10 | 9 招、0 件道具 | 4 / 9 |
| 123 | `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 4 / 9 |
| 124 | `status_trigger:on_kill` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 5 / 9 |
| 125 | `action:impulse` | ability_kit | hook 候選 | B25 | 1 招、0 件道具 | 5 / 9 |
| 126 | `attr:cooldown_remaining` | 產生器 | hook 候選 | B25 | 1 招、0 件道具 | 5 / 9 |
| 127 | `fn:max of non-numbers` | 產生器 | 共用 | B25 | 3 招、0 件道具 | 6 / 9 |
| 128 | `action:cancel_delay` | ability_kit | 共用 | A10 | 2 招、0 件道具 | 6 / 9 |
| 129 | `action:delay.id` | ability_kit | 共用 | A10 | 2 招、0 件道具 | 6 / 9 |
| 130 | `action:swap_positions` | ability_kit | 共用 | A10 | 4 招、0 件道具 | 7 / 9 |
| 131 | `action:heal.source:owner_of(self)` | ability_kit | 共用 | C16 | 2 招、0 件道具 | 7 / 9 |
| 132 | `attr:game.dt` | 產生器 | 共用 | C16 | 2 招、0 件道具 | 7 / 9 |
| 133 | `fn:owner_of` | 產生器 | 共用 | C16 | 2 招、1 件道具 | 7 / 9 |
| 134 | `state:flying` | ability_kit | 共用 | C16 | 3 招、0 件道具 | 8 / 9 |
| 135 | `passive_trigger:periodic.filter` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 8 / 9 |
| 136 | `status_trigger:on_mana_spent` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 9 / 9 |

### M6 第 7 批（9 位）

開始時已全部產生：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 137 | `status_trigger:on_moved` | ability_kit | 共用 | A03 | 3 招、0 件道具 | 4 / 9 |
| 138 | `stat:outgoing_damage_pct` | ability_kit | 共用 | B32 | 4 招、1 件道具 | 5 / 9 |
| 139 | `requires.condition` | ability_kit | 共用 | A31 | 5 招、0 件道具 | 6 / 9 |
| 140 | `selector.team:both` | 產生器 | 共用 | C23 | 6 招、1 件道具 | 7 / 9 |
| 141 | `fn:status_var` | 產生器 | 共用 | B36 | 3 招、0 件道具 | 7 / 9 |
| 142 | `selector.flag:exclude_illusions` | 產生器 | hook 候選 | B36 | 1 招、0 件道具 | 7 / 9 |
| 143 | `selector.team:ally_not_self` | 產生器 | 共用 | B36 | 2 招、0 件道具 | 8 / 9 |
| 144 | `action:chance.counter_var` | ability_kit | 共用 | C21 | 2 招、6 件道具 | 8 / 9 |
| 145 | `action:chance.pity_after` | ability_kit | 共用 | C21 | 2 招、7 件道具 | 8 / 9 |
| 146 | `fn:count_in_radius` | 產生器 | 共用 | C21 | 1 招、1 件道具 | 9 / 9 |

### M6 第 8 批（9 位）

開始時已全部產生：3 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 147 | `action:end_channel` | ability_kit | 共用 | B11 | 2 招、0 件道具 | 4 / 9 |
| 148 | `event:on_channel_tick` | ability_kit | 共用 | B09 | 3 招、0 件道具 | 4 / 9 |
| 149 | `name:event outside an event` | 產生器 | 共用 | B09 | 4 招、2 件道具 | 5 / 9 |
| 150 | `fn:enemy_team` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 5 / 9 |
| 151 | `fn:is_ally` | 產生器 | 共用 | B12 | 3 招、2 件道具 | 5 / 9 |
| 152 | `fn:team_of` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 6 / 9 |
| 153 | `expr: wanted a bool, got a sym:heal_chain` | 產生器 | hook 候選 | A28 | 1 招、0 件道具 | 6 / 9 |
| 154 | `stat:physical_block_chance` | ability_kit | 共用 | A28 | 1 招、2 件道具 | 7 / 9 |
| 155 | `action:issue_order.order:follow` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 156 | `fn:floor of a err` | 產生器 | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 157 | `state type:vec` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 158 | `status_trigger:on_refreshed` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 8 / 9 |
| 159 | `action:reveal.teams:enemy` | ability_kit | 共用 | B23 | 2 招、0 件道具 | 8 / 9 |
| 160 | `fn:is_template` | 產生器 | 共用 | B23 | 6 招、0 件道具 | 8 / 9 |
| 161 | `passive_trigger:on_moved` | ability_kit | hook 候選 | B23 | 1 招、0 件道具 | 8 / 9 |
| 162 | `selector.flag:include_dead` | 產生器 | 共用 | B23 | 2 招、0 件道具 | 9 / 9 |

### M6 第 9 批（13 位）

開始時已全部產生：0 位；做完本批的項目後：13 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 163 | `selector.relative_to` | 產生器 | 共用 | C07 | 2 招、0 件道具 | 0 / 13 |
| 164 | `stat:heal_reduction_pct` | ability_kit | 共用 | C07 | 2 招、2 件道具 | 1 / 13 |
| 165 | `fn:alive of a err` | 產生器 | hook 候選 | B33 | 1 招、0 件道具 | 1 / 13 |
| 166 | `fn:edge_distance` | 產生器 | 共用 | B33 | 4 招、0 件道具 | 2 / 13 |
| 167 | `action:chain` | ability_kit | hook 候選 | B28 | 1 招、0 件道具 | 2 / 13 |
| 168 | `fn:has_state:revealed` | 產生器 | hook 候選 | B28 | 1 招、0 件道具 | 3 / 13 |
| 169 | `fn:orb_slot_free` | 產生器 | 共用 | B15 | 2 招、0 件道具 | 3 / 13 |
| 170 | `passive:orb_slot` | ability_kit | 共用 | B15 | 1 招、1 件道具 | 4 / 13 |
| 171 | `fn:landmark` | 產生器 | hook 候選 | A21 | 1 招、0 件道具 | 4 / 13 |
| 172 | `status.forced_move` | ability_kit | 共用 | A21 | 2 招、0 件道具 | 5 / 13 |
| 173 | `action:damage.credit` | ability_kit | hook 候選 | A11 | 1 招、0 件道具 | 5 / 13 |
| 174 | `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | A11 | 2 招、2 件道具 | 5 / 13 |
| 175 | `attr:cast_point of an ability` | 產生器 | hook 候選 | A11 | 1 招、0 件道具 | 6 / 13 |
| 176 | `action:give_item` | ability_kit | 共用 | B07 | 3 招、0 件道具 | 6 / 13 |
| 177 | `action:pick_item` | ability_kit | 共用 | B07 | 3 招、0 件道具 | 6 / 13 |
| 178 | `fn:exists of a sym:item_a` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 6 / 13 |
| 179 | `fn:exists of a sym:item_b` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 7 / 13 |
| 180 | `action:remove_item` | ability_kit | 共用 | C06 | 2 招、0 件道具 | 7 / 13 |
| 181 | `attr:name` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 7 / 13 |
| 182 | `fn:exists of a sym:stolen` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 7 / 13 |
| 183 | `state type:item_name` | ability_kit | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 184 | `action:modify_cooldown.item` | ability_kit | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 185 | `fn:ability_ready` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 186 | `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 187 | `fn:status_source` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 188 | `selector.flag:include_illusions` | 產生器 | 共用 | C18 | 5 招、1 件道具 | 9 / 13 |
| 189 | `action:add_ability` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 9 / 13 |
| 190 | `action:cue.at of a sym:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 191 | `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 192 | `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 9 / 13 |
| 193 | `action:set_base_max_hp` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 194 | `token:state` | 產生器 | hook 候選 | B16 | 1 招、0 件道具 | 10 / 13 |
| 195 | `action:projectile.team:any` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 196 | `action:projectile.where` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 197 | `fn:circle_point` | 產生器 | 共用 | B24 | 2 招、0 件道具 | 10 / 13 |
| 198 | `selector.team:ally (types building)` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 199 | `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 200 | `selector.types:building` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 201 | `action:destroy_tree` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 202 | `action:regrow_trees` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 203 | `fn:tree_alive` | 產生器 | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 204 | `status.immune_to` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 205 | `target.type:tree` | 產生器 | 共用 | B22 | 2 招、0 件道具 | 12 / 13 |
| 206 | `action:remove_ability` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 207 | `attr:level` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 208 | `expr: wanted a vec, got a sym:dir` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 209 | `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 210 | `selector.shape:segment` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 211 | `selector.to` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 212 | `selector.width` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 213 | `state:blind` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 13 / 13 |

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
| `a07_shattering_spear` | `action:set_ability_active`、`passive_trigger:on_attack_windup` |
| `a07_strongest` | `action:swap_abilities`、`attr:damage`、`event:attack`、`stat:status_resistance_pct` |
| `a07_rouse` | `routine a07d_cast: status.stacking:independent` |
| `a07_rouse_free` | `routine a07d_cast: status.stacking:independent` |

### A34 `omatsu`（3 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a34_frost_bolt` | ✓ 已產生 |
| `a34_iceberg_strike` | ✓ 已產生 |
| `a34_frost_breath` | ✓ 已產生 |
| `a34_rapid_cooling` | `action:set_mana` |

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
| `b04_tanegashima` | `status_trigger:on_attack_launch` |

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
| `b08_endless_fang` | `action:dodge_projectiles.from_team:any`、`action:reveal.ignore_terrain` |

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
| `c15_hundred_arrows` | `action:create_unit.facing`、`action:issue_order.queue`、`action:remove_unit`、`status.status_resistance`、`status_trigger:on_attack_launch` |

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
| `a03_spider_form` | `action:toggle_ability`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`status.scope` |
| `a03_enchanting_venom` | ✓ 已產生 |
| `a03_insect_swarm` | ✓ 已產生 |

### A04 `takenaka_shigeharu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a04_water_shikigami` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:issue_order.order:follow`、`action:set_display_stacks`、`status_trigger:periodic.filter` |
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
| `a05_loyal_secret` | `action:set_mana`、`attr:is_ready`、`state:no_healthbar` |
| `a05_crusade_affidavit` | `status.scope` |

### A06 `ii_naomasa`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a06_red_power` | `status.stacking:independent` |
| `a06_red_meteor` | ✓ 已產生 |
| `a06_red_flame_flow` | `action:set_ability_active`、`passive:aura`、`passive_trigger:on_attack_launch` |
| `a06_flame_of_anger` | `action:set_ability_active`、`target.type:unit_or_point` |
| `a06_red_fury` | `action:set_var`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`status.stacking:independent`、`status.vars`、`status_trigger:on_attack_miss`、`token:vars` |

### A08 `sakakibara_yasumasa`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a08_skull_crusher` | ✓ 已產生 |
| `a08_war_cry` | ✓ 已產生 |
| `a08_indomitable` | ✓ 已產生 |
| `a08_steadfast_general` | `passive:visibility`、`status.stacking:replace` |

### A09 `hachisuka_masakatsu`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a09_treant_summon` | `action:create_unit.control:none`、`action:create_unit.control:owner`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.stacking:independent` |
| `a09_lignify` | ✓ 已產生 |
| `a09_forest_pulse` | `action:spend_mana` |
| `a09_deep_tentacles` | `action:create_unit.control:owner`、`attr:stacks`、`fn:max of non-numbers`、`passive:visibility`、`state:magic_immune` |

### A10 `hirate_masahide`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a10_blink_return` | `action:cancel_delay`、`action:delay.id` |
| `a10_phase_shift` | `action:cancel_delay`、`action:delay.id` |
| `a10_slaughter_array` | `event:point`、`fn:select types`、`passive:visibility` |
| `a10_swap` | `action:swap_positions` |
| `a10_shockwave` | `action:create_unit.control:none`、`event:on_channel_end`、`expr: wanted a unit, got a sym:caster_hero` |

### A11 `saito_dosan`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a11_vortex` | `action:damage.credit`、`action:damage.flags:no_source_bonuses`、`selector.types:building+creep+hero+neutral+summon` |
| `a11_evil_gaze` | `action:apply_status.vars`、`event:on_channel_end`、`stat:move_speed`、`status.forced_move`、`status.status_resistance`、`status.vars` |
| `a11_old_fox` | `attr:cast_point of an ability`、`name:event outside an event`、`passive:aura` |
| `a11_myriad_forms` | `status.aura`、`status.status_resistance` |

### A12 `otani_yoshitsugu`（3 / 9 招）

| 招式 | 還缺 |
|---|---|
| `a12_insight_flash` | `action:knockback` |
| `a12_scheme` | `action:set_display_stacks` |
| `a12_mystic_pierce` | ✓ 已產生 |
| `a12_awe` | ✓ 已產生 |
| `a12_mystic_arrow` | `action:projectile.on_end`、`action:set_display_stacks`、`attr:attack_interval`、`status.status_resistance` |
| `a12_mystic_possession` | `action:swap_abilities`、`fn:ability_of` |
| `a12_insight_flash_t` | `action:knockback` |
| `a12_mystic_field` | `selector.team:ally (types creep+hero+neutral+summon)` |
| `a12_awe_t` | ✓ 已產生 |

### A13 `hattori_hanzo`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a13_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |
| `a13_shadow_illusion` | `action:create_illusion`、`action:cue.at of a sym:il`、`action:dodge_projectiles.of:il`、`action:face`、`action:swap_abilities`、`expr: * of num and sym:me_slot`、`expr: + of num and sym:rot`、`expr: wanted a unit, got a sym:il`、`expr: ~= of num and sym:me_slot`、`fn:exists of a sym:il`、`fn:random_int` |
| `a13_substitution` | `action:swap_positions` |
| `a13_chain_sickle` | ✓ 已產生 |
| `a13_iga_ninjutsu` | ✓ 已產生 |
| `a13_shadow_slash` | `action:attack`、`action:damage.flags:reflection`、`fn:avg_attack_against`、`selector.order:random`、`stat:outgoing_damage_pct`、`stat:vision_day`、`stat:vision_night`、`state:magic_immune`、`status.status_resistance` |

### A14 `maeda_toshiie`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a14_impact_spear` | `action:knockback` |
| `a14_divine_strike` | ✓ 已產生 |
| `a14_stacked_spear` | ✓ 已產生 |
| `a14_spear_fury` | `stat:move_speed` |

### A15 `nene`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a15_shadow_step` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |
| `a15_exhaust_seal` | ✓ 已產生 |
| `a15_moon_strike` | ✓ 已產生 |
| `a15_shadow_windmill` | `attr:damage`、`event:attack` |
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
| `a17_rapid_fire` | `attr:damage`、`event:attack`、`fn:min of non-numbers` |
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
| `a19_fire_echo` | `event:point`、`fn:select types` |
| `a19_siege_fire` | `fn:rotate` |

### A20 `kuki_yoshitaka`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_pirate_domain` | `action:create_unit.limit_per_owner (no host cap for it)`、`attr:owner`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.aura`、`status_trigger:on_attack_windup` |
| `a20_water_burst` | `status.stacking:independent`、`status.status_resistance` |
| `a20_pirate_boots` | `routine a20r_wave: action:repeat in a routine`、`routine a20r_wave: fn:rotate` |
| `a20_pirate_daimyo` | `stat:status_resistance_pct`、`status.unique_group` |

### A21 `zuiryuin_nisshu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_mana_steal` | ✓ 已產生 |
| `a21_half_blade` | `fn:orb_slot_free`、`selector.types:building+creep+hero+neutral+summon`、`status_trigger:on_attack_windup` |
| `a21_energy_press` | ✓ 已產生 |
| `a21_pearl_disc` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:reveal.teams:enemy`、`selector.order:random`、`state:magic_immune`、`status.aura` |
| `a21_false_intel` | `fn:landmark`、`status.forced_move`、`status.status_resistance` |

### A22 `sengoku_hidehisa`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a22_flame_whirl` | `state:magic_immune` |
| `a22_flame_heart` | ✓ 已產生 |
| `a22_boiling_blood` | `passive:aura`、`passive_trigger:periodic.interval`、`status.unique_group` |
| `a22_torment` | `status.reapply_if_removed` |

### A23 `ukita_hideie`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a23_cursed_blood` | ✓ 已產生 |
| `a23_soul_bind` | ✓ 已產生 |
| `a23_falling_fire` | `selector.types:building+creep+hero+neutral+summon`、`status.stacking:independent` |
| `a23_yomi_flame` | `state:break` |

### A24 `sakuma_morimasa`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a24_blood_shield` | `passive:visibility`、`passive_trigger:on_created`、`passive_trigger:periodic.interval` |
| `a24_slaughter_shot` | `action:face` |
| `a24_blood_charge` | `routine a24_blood_price: action:damage.flags:hp_removal`、`routine a24d_sync: action:set_display_stacks`、`routine a24d_sync: status.absorb` |
| `a24_blood_resonance` | `action:reveal.ignore_terrain`、`passive:visibility`、`passive_trigger:periodic.interval` |
| `a24_bloodbath` | `routine a24_blood_price: action:damage.flags:hp_removal`、`routine a24d_sync: action:set_display_stacks`、`routine a24d_sync: status.absorb` |

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

### A27 `shibata_katsuie`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a27_unstoppable_advance` | `action:create_illusion` |
| `a27_war_banner` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`selector.team:ally (types creep+hero+neutral+summon)` |
| `a27_way_of_warrior` | `passive_trigger:on_attack_windup` |
| `a27_tyrant_hammer` | ✓ 已產生 |
| `a27_oni_shibata` | ✓ 已產生 |

### A28 `hashiba_hideyoshi`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_evil_art` | `stat:physical_block_chance` |
| `a28_creation` | `expr: wanted a bool, got a sym:heal_chain`、`fn:is_ally`、`selector.team:ally (types creep+hero+neutral+summon)` |
| `a28_binding_swamp` | ✓ 已產生 |
| `a28_phoenix_summon` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill` |
| `a28_heaven_fire` | ✓ 已產生 |
| `a28_spreading_fire` | `fn:has_status_tag`、`passive_trigger:periodic.interval` |

### A29 `ishida_mitsunari`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_sleeve_poison_arrow` | ✓ 已產生 |
| `a29_abyss_grip` | ✓ 已產生 |
| `a29_withering_aura` | `passive:aura`、`passive_trigger:periodic.filter`、`passive_trigger:periodic.interval`、`status_trigger:on_mana_spent` |
| `a29_vulture_swarm` | `action:cue.at of a err`、`action:heal.source:owner_of(self)`、`attr:game.dt`、`fn:owner_of`、`status.stacking:replace` |

### A30 `niwa_nagahide`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_wolf_mark` | `action:burn_mana.source`、`action:create_unit.control:none`、`action:issue_order.force`、`state:provides_vision` |
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
| `a32_phantom_blade` | `passive_trigger:on_attack_windup` |
| `a32_soul_blade` | `action:toggle_ability`、`fn:avg_attack_against` |
| `a32_asura_blade` | `action:knockback`、`target.type:unit_or_point` |

### A33 `honda_masanobu`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_dark_summoning` | `action:create_unit.control:owner` |
| `a33_ghost_step` | `action:set_display_stacks` |
| `a33_corrupting_aura` | `passive:aura` |
| `a33_deepen_wounds` | `action:refund`、`state:provides_vision`、`state:revealed` |

### A35 `shima_sakon`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_flame_raid` | ✓ 已產生 |
| `a35_sealing_blaze` | `attr:cast_range`、`state:break` |
| `a35_flame_swiftness` | ✓ 已產生 |
| `a35_scorching_fire` | `action:burn_mana.source`、`attr:cast_range` |

### A36 `maeno_nagayasu`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a36_shadow_step` | ✓ 已產生 |
| `a36_spike_trap` | `action:create_unit.control:owner`、`action:create_unit.limit_per_owner (no host cap for it)`、`action:create_unit.overrides`、`state:magic_immune`、`state:no_healthbar`、`status.aura` |
| `a36_detonate` | `routine a36e_explode: action:kill`、`routine a36e_explode: expr: wanted a unit, got a sym:caster_hero`、`routine a36e_explode: fn:alive of a sym:caster_hero`、`routine a36e_explode: fn:select types` |
| `a36_bitter_brew` | ✓ 已產生 |
| `a36_deadly_strike` | ✓ 已產生 |

### B02 `fuma_kotaro`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b02_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |
| `b02_hazy_shadow` | `action:create_illusion`、`action:face` |
| `b02_blazing_path` | `action:zone`、`state type:point` |
| `b02_eighteen_shuriken` | ✓ 已產生 |
| `b02_four_thunders` | ✓ 已產生 |

### B03 `saito_tomonobu`（4 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b03_zhongkui_strike` | ✓ 已產生 |
| `b03_zhongkui_strike_mid` | ✓ 已產生 |
| `b03_zhongkui_strike_far` | ✓ 已產生 |
| `b03_echigo_vajra` | `action:spend_mana`、`attr:mana_pct`、`passive_trigger:on_kill` |
| `b03_heavy_blow` | ✓ 已產生 |
| `b03_zhongkui_judgment` | `action:set_var`、`selector.flag:exclude_invisible`、`status.vars`、`token:vars` |

### B05 `mori_motonari`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_chain_lightning` | `action:apply_status.vars`、`action:cue.at of a err`、`action:set_var`、`status.stacking:independent`、`status.vars`、`token:vars` |
| `b05_heavenly_thunder` | ✓ 已產生 |
| `b05_three_arrows` | `passive:aura`、`state:revealed` |
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
| `b09_backlash_rift` | `action:set_display_stacks`、`action:set_var`、`fn:status_var`、`name:event outside an event`、`status.vars`、`token:vars` |
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
| `b11_cherry_rain` | `attr:cast_range` |
| `b11_breath_control` | `action:end_channel`、`event:on_channel_end` |
| `b11_interception` | `routine b11r_proc: action:chance.counter_var`、`routine b11r_proc: action:chance.pity_after`、`routine b11r_spawn: action:create_illusion`、`routine b11r_spawn: expr: wanted a unit, got a sym:ill`、`routine b11r_spawn: stat:move_speed` |
| `b11_crow_raid` | `action:dodge_projectiles.from_team:any`、`action:set_ability_active` |
| `b11_crow_strike` | `action:set_ability_active`、`passive_trigger:on_attack_windup` |
| `b11_inherited_blood` | `passive_trigger:on_kill` |

### B12 `yamamoto_kansuke`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b12_reality_slide` | `fn:enemy_team`、`fn:is_ally`、`fn:team_of`、`selector.team:both` |
| `b12_floating_life` | `selector.team:ally (types creep+hero+neutral+summon)` |
| `b12_scheming_offense` | `passive:aura` |
| `b12_mad_heart` | ✓ 已產生 |

### B13 `momochi_sandayu`（1 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b13_sense_break` | ✓ 已產生 |
| `b13_hypnosis` | `state:low_attack_priority` |
| `b13_blood_disguise` | `passive_trigger:on_kill` |
| `b13_burrow` | `action:set_ability_active`、`stat:vision_night` |
| `b13_shukuchi` | `action:set_ability_active`、`requires.condition` |
| `b13_storm` | `selector.types:building+creep+hero+neutral+summon`、`status.aura` |

### B14 `katakura_kagetsuna`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_temporary_night` | `action:set_time_of_day`、`event:on_phase_start` |
| `b14_iai_slash` | ✓ 已產生 |
| `b14_shadow_tactics` | `state:magic_immune` |
| `b14_attendant_insight` | `passive_trigger:on_attack_windup` |
| `b14_shadow_cut` | `passive_trigger:on_attack_windup`、`passive_trigger:periodic.interval`、`state:low_attack_priority` |

### B15 `imagawa_yoshimoto`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_tokai_giant` | `state:magic_immune` |
| `b15_flame_arrow` | `action:toggle_ability`、`fn:orb_slot_free`、`passive:orb_slot`、`status_trigger:on_attack_windup` |
| `b15_giant_hide` | ✓ 已產生 |
| `b15_multishot` | `passive_trigger:on_attack_launch` |
| `b15_bluff` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill`、`status.stacking:independent` |

### B16 `mochizuki_chiyome`（2 / 9 招）

| 招式 | 還缺 |
|---|---|
| `b16_flying_raid` | `action:dodge_projectiles.of:state.b16_moon`、`action:face` |
| `b16_falling_leaves` | ✓ 已產生 |
| `b16_swift_thunder` | `passive:visibility` |
| `b16_summon_hound` | `action:add_ability`、`action:create_unit.control:owner`、`action:create_unit.facing`、`action:kill`、`action:remove_unit`、`action:set_ability_active`、`fn:edge_distance` |
| `b16_phase_shift` | `action:dodge_projectiles.of:state.b16_moon`、`action:swap_positions`、`requires.condition` |
| `b16_beast_union` | `action:refund`、`action:set_base_max_hp`、`fn:edge_distance`、`passive_trigger:periodic.interval`、`token:state` |
| `b16_mm_phase_shift` | `action:cue.at of a sym:caster_hero`、`action:dodge_projectiles.of:caster_hero`、`action:swap_positions`、`requires.condition` |
| `b16_mm_sweep` | ✓ 已產生 |
| `b16_mm_stealth` | `passive_trigger:on_created`、`status_trigger:on_attack_launch` |

### B17 `kirigakure_saizo`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_death_ward` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill`、`event.damage.is_attack`、`state:magic_immune`、`state:no_healthbar`、`status.aura` |
| `b17_tooth_poison` | ✓ 已產生 |
| `b17_shadow_leap` | ✓ 已產生 |
| `b17_heartseeker` | `action:set_var`、`state:low_attack_priority`、`status.vars`、`status_trigger:on_attack_launch`、`token:vars` |

### B18 `koihime`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_earth_raid` | ✓ 已產生 |
| `b18_echo` | `action:set_var`、`selector.order:random`、`status.vars`、`token:vars` |
| `b18_earth_shelter` | ✓ 已產生 |
| `b18_earth_claw` | `action:apply_status.vars`、`action:create_unit.control:none`、`action:create_unit.overrides`、`action:issue_order.force`、`action:set_var`、`fn:is_visible_to`、`selector.types:building+creep+hero+neutral+summon`、`selector.types:building+creep+neutral+summon`、`status.aura`、`status.vars`、`token:vars` |

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
| `b21_rebel_armor` | `action:damage.flags:reflection`、`event.damage.is_reflection`、`event.damage.raw`、`passive:visibility` |
| `b21_undying_demon` | `passive:visibility`、`passive_trigger:on_heal_received`、`status.status_resistance` |

### B22 `taigen_sessai`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b22_natural_barrier` | `action:create_unit.control:none`、`fn:circle_point`、`selector.team:both`、`state:magic_immune` |
| `b22_force_of_nature` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:destroy_tree`、`state:flying`、`target.type:tree` |
| `b22_purify` | `action:kill`、`fn:is_ally`、`status.immune_to` |
| `b22_wood_spirit` | `action:create_unit.control:none`、`action:remove_unit`、`action:reveal.ignore_terrain`、`fn:tree_alive`、`selector.types:building+creep+hero+neutral+summon`、`state:revealed`、`target.type:tree` |
| `b22_wrath_of_nature` | `action:create_unit.control:none`、`action:regrow_trees`、`action:remove_unit`、`action:reveal.ignore_terrain` |

### B23 `ise_hime`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_undead_pact` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill`、`action:set_display_stacks`、`action:set_mana`、`event:on_phase_start`、`expr of type nil`、`fn:is_template`、`passive:visibility`、`passive_trigger:on_moved`、`selector.flag:include_dead`、`selector.shape:global`、`selector.team:ally`、`selector.team:both`、`status_trigger:on_moved` |
| `b23_spirit_gathering` | `expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally` |
| `b23_living_soul_shock` | `expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally` |
| `b23_soul_feast` | `event:on_channel_end`、`event:on_channel_tick`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally`、`state:magic_immune` |
| `b23_heroic_spirit_blessing` | `action:reveal.teams:enemy`、`event:on_channel_end`、`event:on_channel_tick`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.team:ally`、`status.aura` |

### B24 `akiyama_nobutomo`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b24_earth_stele` | `action:create_unit.overrides`、`action:kill`、`action:knockback`、`action:set_var`、`selector.flag:include_illusions`、`selector.order:random`、`selector.team:ally (types building)`、`selector.team:both`、`selector.types:boss+creep+neutral+summon`、`status.vars`、`token:vars` |
| `b24_taunt` | `action:issue_order.force`、`selector.flag:include_illusions` |
| `b24_stone_resonance` | `action:projectile.team:any`、`action:projectile.where`、`expr of type nil`、`selector.flag:include_illusions` |
| `b24_earth_prison` | `action:swap_abilities`、`fn:circle_point`、`selector.flag:include_illusions`、`selector.team:both` |
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

### B27 `kojima_sadaoki`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b27_horsemanship` | ✓ 已產生 |
| `b27_mighty_strike` | ✓ 已產生 |
| `b27_fearless_charge` | `state:magic_immune` |
| `b27_rending_blade` | ✓ 已產生 |

### B28 `matsunaga_hisahide`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_phoenix_flame` | ✓ 已產生 |
| `b28_flame_shackle` | `action:apply_status.vars`、`action:end_channel`、`event:on_channel_end`、`fn:edge_distance`、`fn:max of non-numbers`、`name:event outside an event`、`status.vars`、`token:vars` |
| `b28_fire_stream` | `selector.types:building+creep+hero+neutral+summon` |
| `b28_serpent_fire` | `action:chain`、`fn:has_state:revealed` |

### B30 `kakizaki_kageie`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_reverse_shield` | `action:set_var`、`fn:has_tag of a non-event ability`、`fn:min of non-numbers`、`status.vars`、`token:vars` |
| `b30_soul_breaker` | ✓ 已產生 |
| `b30_flash_strike` | `action:dodge_projectiles.from_team:any` |
| `b30_purgatory_slash` | `routine b30t_ignite: action:repeat in a routine` |

### B31 `takeda_shingen`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_swift_wind` | `selector.team:ally (apply_status)`、`status.aura` |
| `b31_still_forest` | `action:projectile.from:spawn`、`fn:random_int`、`fn:rotate`、`status.aura` |
| `b31_raging_fire` | `fn:is_visible_to`、`status.aura`、`status_trigger:periodic.filter` |
| `b31_immovable_mountain` | `state:magic_immune` |

### B32 `uesugi_kenshin`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_divine_slash` | `action:dodge_projectiles.from_team:any` |
| `b32_divine_flash` | `action:dodge_projectiles.from_team:any` |
| `b32_divine_formation` | `action:damage.flags:reflection`、`status.scope` |
| `b32_war_god` | `passive:visibility`、`passive_trigger:on_kill`、`stat:outgoing_damage_pct`、`state:provides_vision` |
| `b32_bishamonten` | ✓ 已產生 |

### B33 `mogami_yoshihime`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_corrosive_arrow` | ✓ 已產生 |
| `b33_spider_web` | ✓ 已產生 |
| `b33_voodoo_power` | ✓ 已產生 |
| `b33_spider_attack` | `action:apply_status.vars`、`action:create_unit.control:none`、`action:create_unit.facing`、`action:face`、`action:kill`、`action:set_var`、`fn:alive of a err`、`fn:edge_distance`、`stat:heal_reduction_pct`、`state:flying`、`status.stacking:replace`、`status.vars`、`token:vars` |

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
| `b35_muto_dori` | `action:attack`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`fn:random_int` |
| `b35_hiten` | `status_trigger:on_attack_windup` |

### B36 `naoe_kanetsugu`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_naoe_seal` | `action:apply_status.vars`、`status.scope`、`status.stacking:replace`、`status.vars` |
| `b36_footsteps` | `fn:status_var` |
| `b36_heart_fence` | `selector.flag:exclude_illusions`、`selector.team:ally_not_self`、`status.absorb` |
| `b36_seven_spears` | `action:knockback` |
| `b36_aizen_myoo` | ✓ 已產生 |

### C01 `akechi_mitsuhide`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c01_blood_blade` | `status_trigger:on_attack_windup` |
| `c01_ground_flame` | `routine c01e_wave: status.stacking:independent` |
| `c01_blood_flame` | `passive_trigger:on_attack_windup`、`passive_trigger:periodic.interval` |
| `c01_honnoji` | `action:cue.at of a sym:landing`、`action:random_pick`、`expr: a position from a sym:landing`、`status.stacking:independent`、`status.status_resistance` |

### C02 `akechi_hidemitsu`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c02_mirage_strike` | `action:create_illusion`、`action:face`、`attr:cast_range`、`expr: wanted a unit, got a sym:ill` |
| `c02_phantom_move` | `action:create_illusion`、`action:dodge_projectiles.from_team:any`、`expr: wanted a unit, got a sym:ill` |
| `c02_kyoka_suigetsu` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill` |
| `c02_culling_slash` | ✓ 已產生 |

### C03 `shimazu_yoshihiro`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_iron_shirt` | `state:magic_immune` |
| `c03_quake_axe` | ✓ 已產生 |
| `c03_thunder_wrath` | `passive_trigger:on_attack_windup` |
| `c03_iron_hammer` | `fn:offset types` |

### C04 `izumo_no_okuni`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_swift_dance` | `expr of type nil`、`selector.shape:global`、`selector.team:ally` |
| `c04_withering_step` | `action:set_ability_active`、`action:spend_mana`、`action:toggle_ability` |
| `c04_gathering_dance` | ✓ 已產生 |
| `c04_witch_dance` | `event:on_channel_end`、`status.status_resistance` |

### C05 `maeda_keiji`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c05_matsukaze` | ✓ 已產生 |
| `c05_thunder_dance` | ✓ 已產生 |
| `c05_thunder_spear` | ✓ 已產生 |
| `c05_flash_thunder` | `event:on_channel_end`、`event:on_phase_start` |

### C06 `ishikawa_goemon`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c06_quick_hands` | ✓ 已產生 |
| `c06_sleight_of_hand` | `action:give_item`、`action:pick_item`、`action:remove_item`、`attr:name`、`fn:exists of a sym:stolen` |
| `c06_lavish_strike` | `passive:visibility`、`status.absorb` |
| `c06_murder_and_loot` | `passive:visibility`、`passive_trigger:on_kill` |
| `c06_grand_heist` | ✓ 已產生 |
| `item_c06e` | `action:give_item`、`action:remove_item`、`state type:item_name` |

### C07 `tachibana_dosetsu`（0 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c07_heaven_thunder` | `action:zone` |
| `c07_thunder_shackle` | `action:apply_status.vars`、`event:on_channel_end`、`name:event outside an event`、`status.stacking:replace`、`status.status_resistance`、`status.vars`、`token:vars` |
| `c07_thunderbolt` | `selector.relative_to` |
| `c07_raijin_wrath` | `action:create_unit.control:none`、`fn:has_tag of a non-event ability`、`fn:is_visible_to`、`stat:heal_reduction_pct`、`status.status_resistance` |
| `c07_storm_move` | `action:issue_order.force` |
| `c07_magic_shield` | `action:toggle_ability`、`stat:move_speed`、`state:magic_immune`、`status_trigger:on_attack_windup` |

### C08 `kato_danzo`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |
| `c08_blood_hunt` | `action:damage.flags:no_source_bonuses`、`stat:move_speed`、`status.stacking:independent` |
| `c08_scatter_escape` | `action:dodge_projectiles.from_team:any` |
| `c08_kite_chain` | `action:face`、`expr of type nil`、`expr: wanted a vec, got a sym:dir`、`fn:has_tag of a non-event ability`、`fn:offset types`、`selector.relative_to`、`selector.shape:segment`、`selector.team:ally_not_self`、`selector.to`、`selector.width` |
| `c08_devour` | `action:add_ability`、`action:remove_ability`、`action:set_ability_active`、`action:set_var`、`attr:level`、`fn:ability_in_slot`、`fn:is_ability of a non-event ability`、`fn:random_int`、`fn:status_var`、`stat:vision_day`、`stat:vision_night`、`state:blind`、`state:magic_immune`、`status.status_resistance`、`status.vars`、`token:vars` |

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
| `c10_day_night_shift` | `action:set_time_of_day` |
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
| `c13_gale_grip` | `action:issue_order.force`、`event.damage.is_attack`、`passive:visibility`、`passive_trigger:on_kill`、`selector.types:building+creep+hero+neutral+summon`、`status_trigger:on_kill` |
| `c13_tornado_vortex` | `action:modify_cooldown.item_tag`、`fn:rotate` |

### C14 `saito_yoshitatsu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c14_fire_curse` | ✓ 已產生 |
| `c14_fire_breath` | `status_trigger:periodic.filter` |
| `c14_fire_breath_dragon` | `routine c14e_breath: action:projectile in a routine` |
| `c14_dragon_vein` | ✓ 已產生 |
| `c14_dragon_form` | `action:swap_abilities`、`attr:damage`、`event:attack`、`selector.types:building+creep+hero+neutral+summon` |

### C16 `kashin_koji`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_soul_sever` | ✓ 已產生 |
| `c16_curse_shadow` | `action:create_unit.control:owner`、`stat:vision_day`、`stat:vision_night`、`state:magic_immune` |
| `c16_wandering_souls` | `action:cue.at of a err`、`action:heal.source:owner_of(self)`、`attr:game.dt`、`fn:owner_of`、`state:flying`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.stacking:replace` |
| `c16_phantom_sight` | `action:create_illusion`、`action:face` |
| `c16_eight_gates` | `action:create_unit.control:owner`、`selector.types:building+creep+hero+neutral+summon`、`state:magic_immune` |

### C18 `kobayakawa_takakage`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c18_lone_rider` | `action:modify_cooldown.item`、`action:pick_item`、`expr of type nil`、`fn:exists of a sym:blink_item`、`fn:status_source`、`selector.flag:include_dead`、`selector.shape:global` |
| `c18_twin_chariot` | `fn:ability_ready`、`status.scope` |
| `c18_three_blades` | ✓ 已產生 |
| `c18_chain_stratagem` | `attr:stacks`、`passive:visibility` |
| `c18_four_cannons` | `expr of type nil`、`selector.flag:include_illusions`、`selector.shape:global` |

### C19 `matsuhime`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c19_alluring_gaze` | `action:set_mana` |
| `c19_gentle_vow` | ✓ 已產生 |
| `c19_paralyzing_poison` | `passive:visibility` |
| `c19_surprise_raid` | `event:on_phase_start`、`requires.condition` |
| `c19_inviolable` | `action:knockback`、`passive:visibility` |

### C20 `chosokabe_motochika`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c20_flame_aegis` | `action:damage.flags:reflection`、`event.damage.is_reflection`、`status.scope` |
| `c20_demon_blood` | ✓ 已產生 |
| `c20_flame_dance` | ✓ 已產生 |
| `c20_demon_child` | `attr:damage`、`event:attack`、`fn:armor_multiplier` |

### C21 `miyamoto_musashi`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c21_crescent_slash` | `state:magic_immune` |
| `c21_chain_breaker` | `action:attack`、`action:face`、`action:set_ability_active`、`fn:rotate`、`stat:outgoing_damage_pct` |
| `c21_two_heavens` | `action:chance.counter_var`、`action:chance.pity_after`、`passive_trigger:on_attack_windup`、`status.stacking:replace` |
| `c21_soaring_quake` | `action:attack`、`action:face`、`fn:count_in_radius`、`fn:rotate`、`selector.order:random` |

### C22 `sasaki_kojiro`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c22_ganryu` | `status_trigger:on_attack_windup` |
| `c22_swallow_nest` | `state:no_healthbar`、`state:unselectable` |
| `c22_flying_swallow` | `status.stacking:independent` |
| `c22_swallow_flash` | ✓ 已產生 |
| `c22_dream_mist` | ✓ 已產生 |

### C23 `kato_kiyomasa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c23_tainted_frenzy` | ✓ 已產生 |
| `c23_malice_wall` | `action:create_unit.facing`、`action:face`、`selector.team:both`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.aura` |
| `c23_blood_for_blood` | ✓ 已產生 |
| `c23_gift_of_death` | `action:kill`、`event:on_phase_start` |

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
| 施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR） | 116 |
| 召喚：附近放不下、lifetime 不在 0～600 秒 | 1 |
| 召喚：count 不是 1 | 1 |
| 視野：半徑或位置不合法 | 6 |
| 光鍊：host tether 規格不合法 | 1 |
| 友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離 | 1 |
| 運算式除以零 | 5 |
| 讀位置的單位已不在世界中 | 47 |
| 需要地點，但施放沒有地點也沒有單位目標 | 30 |

### 已產生但 host 沒有做到的地方

產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。

| 招式 | 沒有做到 |
|---|---|
| `c06_grand_heist` | create_pickup.cue: the host pickup draws its own bag |
| `b14_iai_slash` | issue_order: only a hero takes the order |
| `b27_mighty_strike` | issue_order: only a hero takes the order |
| `c14_fire_curse` | issue_order: only a hero takes the order |
| `item_kisyosennfuyuzuki` | item cast_point: the host uses items without a windup |
| `item_the_red_lightning_chapter` | item cast_point: the host uses items without a windup |
| `item_the_sword_of_kaminaobi` | issue_order: only a hero takes the order |
| `item_yubashiri` | item cast_point: the host uses items without a windup |

## 4. 每項寫法擋住多少

| 寫法 | 主要負責 | 分類 | 招式 | 道具 | 英雄 |
|---|---|---|---|---|---|
| `state:magic_immune` | ability_kit | 共用 | 20 | 14 | 19 |
| `status.vars` | ability_kit | 共用 | 19 | 0 | 18 |
| `passive:visibility` | ability_kit | 共用 | 20 | 0 | 16 |
| `status.stacking:independent` | ability_kit | 共用 | 18 | 5 | 15 |
| `selector.types:building+creep+hero+neutral+summon` | 產生器 | 共用 | 17 | 2 | 15 |
| `status.status_resistance` | ability_kit | 共用 | 16 | 0 | 14 |
| `token:vars` | 產生器 | 共用 | 15 | 2 | 14 |
| `action:create_unit.control:owner` | ability_kit | 共用 | 15 | 6 | 13 |
| `status.aura` | ability_kit | 共用 | 15 | 1 | 13 |
| `action:set_var` | ability_kit | 共用 | 13 | 0 | 12 |
| `passive_trigger:periodic.interval` | ability_kit | 共用 | 13 | 12 | 12 |
| `action:face` | ability_kit | 共用 | 12 | 1 | 11 |
| `event:on_channel_end` | ability_kit | 共用 | 12 | 0 | 11 |
| `passive_trigger:on_attack_windup` | ability_kit | 共用 | 12 | 3 | 11 |
| `action:create_unit.overrides` | ability_kit | 共用 | 11 | 4 | 11 |
| `action:set_ability_active` | ability_kit | 共用 | 13 | 0 | 10 |
| `attr:damage` | 產生器 | 共用 | 12 | 2 | 10 |
| `event:attack` | ability_kit | 共用 | 12 | 9 | 10 |
| `action:kill` | ability_kit | 共用 | 11 | 2 | 10 |
| `passive:aura` | ability_kit | 共用 | 11 | 22 | 10 |
| `action:create_unit.control:none` | ability_kit | 共用 | 11 | 4 | 9 |
| `action:dodge_projectiles.from_team:any` | ability_kit | 共用 | 11 | 3 | 9 |
| `action:knockback` | ability_kit | 共用 | 10 | 3 | 9 |
| `event:on_phase_start` | ability_kit | 共用 | 9 | 2 | 9 |
| `status.stacking:replace` | ability_kit | 共用 | 9 | 0 | 9 |
| `status_trigger:on_attack_windup` | ability_kit | 共用 | 9 | 4 | 9 |
| `expr of type nil` | 產生器 | 共用 | 15 | 0 | 8 |
| `fn:rotate` | 產生器 | 共用 | 9 | 3 | 8 |
| `status_trigger:on_attack_launch` | ability_kit | 共用 | 9 | 1 | 8 |
| `action:apply_status.vars` | ability_kit | 共用 | 8 | 0 | 8 |
| `action:set_mana` | ability_kit | 共用 | 8 | 1 | 8 |
| `passive_trigger:on_kill` | ability_kit | 共用 | 8 | 5 | 8 |
| `state:no_healthbar` | ability_kit | 共用 | 8 | 0 | 8 |
| `selector.shape:global` | 產生器 | 共用 | 13 | 0 | 7 |
| `action:create_illusion` | ability_kit | 共用 | 9 | 0 | 7 |
| `selector.team:ally (types creep+hero+neutral+summon)` | 產生器 | 共用 | 8 | 13 | 7 |
| `state:low_attack_priority` | ability_kit | 共用 | 7 | 0 | 7 |
| `action:set_display_stacks` | ability_kit | 共用 | 8 | 2 | 6 |
| `status.absorb` | ability_kit | 共用 | 7 | 1 | 6 |
| `action:swap_abilities` | ability_kit | 共用 | 6 | 0 | 6 |
| `selector.order:random` | 產生器 | 共用 | 6 | 1 | 6 |
| `stat:move_speed` | ability_kit | 共用 | 6 | 1 | 6 |
| `stat:vision_night` | ability_kit | 共用 | 6 | 3 | 6 |
| `status.scope` | ability_kit | 共用 | 6 | 3 | 6 |
| `selector.team:both` | 產生器 | 共用 | 6 | 1 | 5 |
| `action:burn_mana.source` | ability_kit | 共用 | 5 | 1 | 5 |
| `action:damage.flags:reflection` | ability_kit | 共用 | 5 | 1 | 5 |
| `action:issue_order.force` | ability_kit | 共用 | 5 | 0 | 5 |
| `action:toggle_ability` | ability_kit | 共用 | 5 | 0 | 5 |
| `fn:armor_multiplier` | 產生器 | 共用 | 5 | 2 | 5 |
| `fn:has_status_tag` | 產生器 | 共用 | 5 | 11 | 5 |
| `fn:has_tag of a non-event ability` | 產生器 | 共用 | 5 | 4 | 5 |
| `fn:offset types` | 產生器 | 共用 | 5 | 3 | 5 |
| `state:unselectable` | ability_kit | 共用 | 5 | 0 | 5 |
| `selector.team:ally` | 產生器 | 共用 | 8 | 0 | 4 |
| `action:attack` | ability_kit | 共用 | 5 | 2 | 4 |
| `action:reveal.ignore_terrain` | ability_kit | 共用 | 5 | 2 | 4 |
| `attr:cast_range` | 產生器 | 共用 | 5 | 0 | 4 |
| `requires.condition` | ability_kit | 共用 | 5 | 0 | 4 |
| `action:create_unit.facing` | ability_kit | 共用 | 4 | 0 | 4 |
| `action:cue.at of a err` | ability_kit | 共用 | 4 | 0 | 4 |
| `action:spend_mana` | ability_kit | 共用 | 4 | 2 | 4 |
| `fn:random_int` | 產生器 | 共用 | 4 | 2 | 4 |
| `name:event outside an event` | 產生器 | 共用 | 4 | 2 | 4 |
| `passive_trigger:on_attack_launch` | ability_kit | 共用 | 4 | 0 | 4 |
| `stat:outgoing_damage_pct` | ability_kit | 共用 | 4 | 1 | 4 |
| `stat:vision_day` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:not_on_minimap` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:provides_vision` | ability_kit | 共用 | 4 | 1 | 4 |
| `state:revealed` | ability_kit | 共用 | 4 | 6 | 4 |
| `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | 5 | 0 | 3 |
| `action:remove_unit` | ability_kit | 共用 | 4 | 0 | 3 |
| `action:swap_positions` | ability_kit | 共用 | 4 | 0 | 3 |
| `fn:edge_distance` | 產生器 | 共用 | 4 | 0 | 3 |
| `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | 3 | 0 | 3 |
| `action:pick_item` | ability_kit | 共用 | 3 | 0 | 3 |
| `action:zone` | ability_kit | 共用 | 3 | 0 | 3 |
| `attr:stacks` | 產生器 | 共用 | 3 | 0 | 3 |
| `event.damage.is_reflection` | ability_kit | 共用 | 3 | 1 | 3 |
| `fn:avg_attack_against` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:is_ally` | 產生器 | 共用 | 3 | 2 | 3 |
| `fn:is_visible_to` | 產生器 | 共用 | 3 | 3 | 3 |
| `fn:max of non-numbers` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:select types` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:status_var` | 產生器 | 共用 | 3 | 0 | 3 |
| `state:flying` | ability_kit | 共用 | 3 | 0 | 3 |
| `status_trigger:on_moved` | ability_kit | 共用 | 3 | 0 | 3 |
| `status_trigger:periodic.filter` | ability_kit | 共用 | 3 | 0 | 3 |
| `target.type:unit_or_point` | 產生器 | 共用 | 3 | 1 | 3 |
| `fn:is_template` | 產生器 | 共用 | 6 | 0 | 2 |
| `selector.flag:include_illusions` | 產生器 | 共用 | 5 | 1 | 2 |
| `action:give_item` | ability_kit | 共用 | 3 | 0 | 2 |
| `action:leap` | ability_kit | 共用 | 3 | 0 | 2 |
| `event:on_channel_tick` | ability_kit | 共用 | 3 | 0 | 2 |
| `selector.flag:include_hidden` | 產生器 | 共用 | 3 | 0 | 2 |
| `action:add_ability` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:chance.counter_var` | ability_kit | 共用 | 2 | 6 | 2 |
| `action:chance.pity_after` | ability_kit | 共用 | 2 | 7 | 2 |
| `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | 2 | 2 | 2 |
| `action:end_channel` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:heal.source:owner_of(self)` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:projectile in a routine` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:projectile.on_end` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:random_pick` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:refund` | ability_kit | 共用 | 2 | 2 | 2 |
| `action:repeat in a routine` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:reveal.teams:enemy` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:set_time_of_day` | ability_kit | 共用 | 2 | 0 | 2 |
| `attr:game.dt` | 產生器 | 共用 | 2 | 0 | 2 |
| `attr:mana_pct` | 產生器 | 共用 | 2 | 1 | 2 |
| `event.damage.is_attack` | ability_kit | 共用 | 2 | 0 | 2 |
| `event:point` | ability_kit | 共用 | 2 | 0 | 2 |
| `expr: wanted a unit, got a sym:caster_hero` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:ability_in_slot` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:circle_point` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:min of non-numbers` | 產生器 | 共用 | 2 | 2 | 2 |
| `fn:orb_slot_free` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:owner_of` | 產生器 | 共用 | 2 | 1 | 2 |
| `op:%` | 產生器 | 共用 | 2 | 0 | 2 |
| `passive_trigger:on_created` | ability_kit | 共用 | 2 | 44 | 2 |
| `selector.flag:include_dead` | 產生器 | 共用 | 2 | 0 | 2 |
| `selector.relative_to` | 產生器 | 共用 | 2 | 0 | 2 |
| `selector.team:ally_not_self` | 產生器 | 共用 | 2 | 0 | 2 |
| `stat:heal_reduction_pct` | ability_kit | 共用 | 2 | 2 | 2 |
| `stat:status_resistance_pct` | ability_kit | 共用 | 2 | 2 | 2 |
| `state type:point` | ability_kit | 共用 | 2 | 0 | 2 |
| `state:break` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.forced_move` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.unique_group` | ability_kit | 共用 | 2 | 1 | 2 |
| `action:cancel_delay` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:damage.flags:hp_removal` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:delay.id` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:remove_item` | ability_kit | 共用 | 2 | 0 | 1 |
| `target.type:tree` | 產生器 | 共用 | 2 | 0 | 1 |
| `action:chain` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:caster_hero` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:landing` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:damage.credit` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:damage.flags:ignore_armor` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:delay in a routine` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:destroy_tree` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:dodge_projectiles.of:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:impulse` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:issue_order.order:follow` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:issue_order.queue` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.item` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.from:spawn` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.team:any` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.where` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:regrow_trees` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_ability` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:set_base_max_hp` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `attr:attack_interval` | 產生器 | 共用 | 1 | 3 | 1 |
| `attr:cast_point of an ability` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:cooldown_remaining` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:is_ready` | 產生器 | 共用 | 1 | 3 | 1 |
| `attr:level` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:name` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:owner` | 產生器 | hook 候選 | 1 | 0 | 1 |
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
| `fn:landmark` | 產生器 | hook 候選 | 1 | 0 | 1 |
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
| `selector.team:ally (apply_status)` | 產生器 | 共用 | 1 | 4 | 1 |
| `selector.team:ally (types building)` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.to` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:building` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:building+creep+neutral+summon` | 產生器 | 共用 | 1 | 1 | 1 |
| `selector.types:courier+creep+hero+neutral+summon+ward` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.width` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `stat.when` | ability_kit | 共用 | 1 | 4 | 1 |
| `stat:physical_block_chance` | ability_kit | 共用 | 1 | 2 | 1 |
| `state type:item_name` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state type:vec` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:blind` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.immune_to` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.reapply_if_removed` | ability_kit | 共用 | 1 | 13 | 1 |
| `status_trigger:on_attack_miss` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_kill` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_mana_spent` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_refreshed` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `token:state` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `action:chance.counter` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:damage.type:event` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:issue_order.ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:issue_order.order:cast` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:projectile.from:offset(base, rotate(fwd, $perp_angle), random_int(-$storm_width, $storm_width))` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:projectile.lifetime` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:remove_status.exclude_source_tags` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.min_duration` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.source_is` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:remove_status.status with limit` | ability_kit | 共用 | 0 | 3 | 0 |
| `attr:effect_multiplier` | 產生器 | 共用 | 0 | 7 | 0 |
| `attr:illusion_damage` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `attr:is_summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `event:on_owner_death` | ability_kit | 共用 | 0 | 3 | 0 |
| `event:on_unequip` | ability_kit | 共用 | 0 | 2 | 0 |
| `expr: wanted a vec, got a sym:fwd` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:angle_between` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:has_item` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:has_tag:no_cast_trigger` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:is_hero_id` | 產生器 | 共用 | 0 | 7 | 0 |
| `item.active.cast.interruptible` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.cost.charges` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.kind:passive` | ability_kit | 共用 | 0 | 3 | 0 |
| `item.active.kind:toggle` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_death` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_removed` | ability_kit | 共用 | 0 | 25 | 0 |
| `passive_trigger:periodic.first` | ability_kit | 共用 | 0 | 4 | 0 |
| `rust_handler without behavior` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `selector.exclude_types` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.team:ally (types building+creep+hero+neutral+summon)` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.team:ally (types building+creep+hero+neutral+summon+tower)` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.team:ally (types building+hero+tower)` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.team:ally (types creep+neutral+summon)` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:building+creep+hero+neutral+summon+tower` | 產生器 | 共用 | 0 | 4 | 0 |
| `stat:all_stats` | ability_kit | 共用 | 0 | 9 | 0 |
| `status.unique_rule` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status_trigger:on_order` | ability_kit | 共用 | 0 | 2 | 0 |
| `status_trigger:on_order_targeted` | ability_kit | 共用 | 0 | 2 | 0 |
| `target.types:building+courier+creep+neutral+summon` | 產生器 | hook 候選 | 0 | 1 | 0 |

## 5. hook 候選

只有 1 招（英雄技能與道具合計）用到的寫法。這些招式在 v3 標 `hook = "<方法名>"`、寫明行為描述與預期結果，由產生的 trait 手寫實作（`src/hooks/<招式>.rs`）；第 2 招也用到時改成共用、刪掉 hook。

| 寫法 | 招式 | 所屬 |
|---|---|---|
| `status_trigger:on_refreshed` | `a04_frost_shelter` | A04 takenaka_shigeharu |
| `fn:floor of a err` | `a04_ice_path` | A04 takenaka_shigeharu |
| `state type:vec` | `a04_ice_path` | A04 takenaka_shigeharu |
| `action:issue_order.order:follow` | `a04_water_shikigami` | A04 takenaka_shigeharu |
| `status_trigger:on_attack_miss` | `a06_red_fury` | A06 ii_naomasa |
| `action:delay in a routine` | `a07_red_thunder` | A07 honda_tadakatsu |
| `attr:cast_point of an ability` | `a11_old_fox` | A11 saito_dosan |
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
| `attr:owner` | `a20_pirate_domain` | A20 kuki_yoshitaka |
| `fn:landmark` | `a21_false_intel` | A21 zuiryuin_nisshu |
| `expr: wanted a unit, got a point` | `a26_scorching_bomb` | A26 nohime |
| `expr: wanted a bool, got a sym:heal_chain` | `a28_creation` | A28 hashiba_hideyoshi |
| `passive_trigger:periodic.filter` | `a29_withering_aura` | A29 ishida_mitsunari |
| `status_trigger:on_mana_spent` | `a29_withering_aura` | A29 ishida_mitsunari |
| `fn:alive of a sym:caster_hero` | `a36_detonate` | A36 maeno_nagayasu |
| `selector.flag:exclude_invisible` | `b03_zhongkui_judgment` | B03 saito_tomonobu |
| `passive_trigger:on_order_targeted` | `b06_mind_eye` | B06 sanada_yukimura |
| `fn:exists of a sym:item_a` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `fn:exists of a sym:item_b` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `fn:enemy_team` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `fn:team_of` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `action:set_base_max_hp` | `b16_beast_union` | B16 mochizuki_chiyome |
| `token:state` | `b16_beast_union` | B16 mochizuki_chiyome |
| `action:cue.at of a sym:caster_hero` | `b16_mm_phase_shift` | B16 mochizuki_chiyome |
| `action:dodge_projectiles.of:caster_hero` | `b16_mm_phase_shift` | B16 mochizuki_chiyome |
| `passive_trigger:on_heal_received` | `b21_undying_demon` | B21 baba_nobufusa |
| `action:destroy_tree` | `b22_force_of_nature` | B22 taigen_sessai |
| `status.immune_to` | `b22_purify` | B22 taigen_sessai |
| `fn:tree_alive` | `b22_wood_spirit` | B22 taigen_sessai |
| `action:regrow_trees` | `b22_wrath_of_nature` | B22 taigen_sessai |
| `passive_trigger:on_moved` | `b23_undead_pact` | B23 ise_hime |
| `selector.team:ally (types building)` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:boss+creep+neutral+summon` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:building` | `b24_shatter` | B24 akiyama_nobutomo |
| `action:projectile.team:any` | `b24_stone_resonance` | B24 akiyama_nobutomo |
| `action:projectile.where` | `b24_stone_resonance` | B24 akiyama_nobutomo |
| `action:impulse` | `b25_asura_hell` | B25 honganji_kennyo |
| `attr:cooldown_remaining` | `b25_exorcism_sutra` | B25 honganji_kennyo |
| `event.damage.lethal` | `b26_kozuke_valor` | B26 okuma_tomohide |
| `action:chain` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `fn:has_state:revealed` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `action:projectile.from:spawn` | `b31_still_forest` | B31 takeda_shingen |
| `fn:alive of a err` | `b33_spider_attack` | B33 mogami_yoshihime |
| `selector.flag:exclude_illusions` | `b36_heart_fence` | B36 naoe_kanetsugu |
| `action:cue.at of a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `expr: a position from a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `attr:name` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `fn:exists of a sym:stolen` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `state type:item_name` | `item_c06e` | C06 ishikawa_goemon |
| `action:remove_ability` | `c08_devour` | C08 kato_danzo |
| `attr:level` | `c08_devour` | C08 kato_danzo |
| `fn:is_ability of a non-event ability` | `c08_devour` | C08 kato_danzo |
| `state:blind` | `c08_devour` | C08 kato_danzo |
| `expr: wanted a vec, got a sym:dir` | `c08_kite_chain` | C08 kato_danzo |
| `selector.shape:segment` | `c08_kite_chain` | C08 kato_danzo |
| `selector.to` | `c08_kite_chain` | C08 kato_danzo |
| `selector.width` | `c08_kite_chain` | C08 kato_danzo |
| `attr:remaining` | `c09_ice_soul_dance` | C09 tachibana_ginchiyo |
| `action:damage.flags:ignore_armor` | `c10_echo_sword` | C10 kosokabe_chikayasu |
| `status_trigger:on_kill` | `c13_gale_grip` | C13 ryuzoji_takanobu |
| `action:modify_cooldown.item_tag` | `c13_tornado_vortex` | C13 ryuzoji_takanobu |
| `action:issue_order.queue` | `c15_hundred_arrows` | C15 tamako |
| `action:modify_cooldown.item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:exists of a sym:blink_item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:status_source` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:ability_ready` | `c18_twin_chariot` | C18 kobayakawa_takakage |
| `action:chance.counter` | `item_big_tachi` | 道具 |
| `attr:is_summon` | `item_commander_of_fan` | 道具 |
| `selector.team:ally (types building+hero+tower)` | `item_contemporary_armor` | 道具 |
| `item.active.cost.charges` | `item_earth_wall_book` | 道具 |
| `status.unique_rule` | `item_foxfire_armor` | 道具 |
| `rust_handler without behavior` | `item_hama_orb` | 道具 |
| `fn:angle_between` | `item_hannya_onimaru` | 道具 |
| `action:remove_status.source_is` | `item_ice_storm_book` | 道具 |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `selector.exclude_types` | `item_napalm_bomb` | 道具 |
| `action:projectile.lifetime` | `item_ronin_jitte` | 道具 |
| `selector.team:ally (types building+creep+hero+neutral+summon)` | `item_the_art_of_war_mountain_chapter` | 道具 |
| `action:damage.type:event` | `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 |
| `selector.team:ally (types creep+neutral+summon)` | `item_the_great_sword_of_disease` | 道具 |
| `item.active.cast.interruptible` | `item_the_great_sword_of_teleport` | 道具 |
| `target.types:building+courier+creep+neutral+summon` | `item_the_great_sword_of_teleport` | 道具 |
| `selector.team:ally (types building+creep+hero+neutral+summon+tower)` | `item_the_lost_art_of_war_2` | 道具 |

共 92 種寫法、66 招。

## 重跑

```
tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua
```

