# 產生器還沒產生的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。

- 英雄技能：447 招中 170 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：5 / 97 位。
- 道具（items.dll，M7）：168 件中 60 件產生成 Rust。
- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。
- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。
- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。

## 1. 實作順序

### M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）（10 位）

開始時已全部產生：4 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 1 | `action:reveal.ignore_terrain` | ability_kit | 共用 | B08 | 5 招、2 件道具 | 5 / 10 |
| 2 | `status.aura` | ability_kit | 共用 | B04 | 15 招、1 件道具 | 5 / 10 |
| 3 | `status_trigger:on_attack_launch` | ability_kit | 共用 | B04 | 9 招、1 件道具 | 6 / 10 |
| 4 | `action:projectile in a routine` | ability_kit | 共用 | B06 | 2 招、0 件道具 | 6 / 10 |
| 5 | `passive_trigger:on_order_targeted` | ability_kit | hook 候選 | B06 | 1 招、0 件道具 | 7 / 10 |
| 6 | `action:leap` | ability_kit | 共用 | B29 | 3 招、0 件道具 | 7 / 10 |
| 7 | `passive:aura` | ability_kit | 共用 | B29 | 11 招、22 件道具 | 8 / 10 |
| 8 | `action:delay in a routine` | ability_kit | hook 候選 | A07 | 1 招、0 件道具 | 8 / 10 |
| 9 | `action:modify_attack` | ability_kit | 共用 | A07 | 15 招、6 件道具 | 8 / 10 |
| 10 | `attr:damage` | 產生器 | 共用 | A07 | 12 招、2 件道具 | 8 / 10 |
| 11 | `event:attack` | ability_kit | 共用 | A07 | 12 招、9 件道具 | 8 / 10 |
| 12 | `stat:status_resistance_pct` | ability_kit | 共用 | A07 | 2 招、2 件道具 | 9 / 10 |
| 13 | `action:create_unit.control:none` | ability_kit | 共用 | C15 | 11 招、4 件道具 | 9 / 10 |
| 14 | `action:create_unit.facing` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 9 / 10 |
| 15 | `action:issue_order.queue` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 16 | `action:remove_unit` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 9 / 10 |
| 17 | `status.status_resistance` | ability_kit | 共用 | C15 | 16 招、0 件道具 | 10 / 10 |

### M6 第 1 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 18 | `passive:visibility` | ability_kit | 共用 | B20 | 20 招、0 件道具 | 2 / 9 |
| 19 | `status.absorb` | ability_kit | 共用 | A18 | 7 招、2 件道具 | 3 / 9 |
| 20 | `state:magic_immune` | ability_kit | 共用 | B27 | 20 招、15 件道具 | 4 / 9 |
| 21 | `event.damage.lethal` | ability_kit | hook 候選 | B26 | 1 招、0 件道具 | 5 / 9 |
| 22 | `action:knockback` | ability_kit | 共用 | A12 | 10 招、3 件道具 | 5 / 9 |
| 23 | `action:set_display_stacks` | ability_kit | 共用 | A12 | 8 招、2 件道具 | 6 / 9 |
| 24 | `stat:move_speed` | ability_kit | 共用 | A14 | 7 招、1 件道具 | 7 / 9 |
| 25 | `event:on_phase_start` | ability_kit | 共用 | A26 | 9 招、2 件道具 | 7 / 9 |
| 26 | `expr: wanted a unit, got a point` | 產生器 | hook 候選 | A26 | 1 招、0 件道具 | 8 / 9 |
| 27 | `fn:has_status of an independent status (line B)` | 產生器 | 共用 | C22 | 2 招、0 件道具 | 8 / 9 |
| 28 | `state:no_healthbar` | ability_kit | 共用 | C22 | 8 招、0 件道具 | 8 / 9 |
| 29 | `state:unselectable` | ability_kit | 共用 | C22 | 5 招、0 件道具 | 8 / 9 |
| 30 | `status.stacking:independent with states` | ability_kit | 共用 | C22 | 4 招、1 件道具 | 9 / 9 |

### M6 第 2 批（10 位）

開始時已全部產生：1 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 31 | `event:on_channel_end` | ability_kit | 共用 | C05 | 12 招、0 件道具 | 2 / 10 |
| 32 | `state:break` | ability_kit | 共用 | A35 | 2 招、0 件道具 | 3 / 10 |
| 33 | `status.stacking:independent with triggers` | ability_kit | 共用 | A23 | 7 招、0 件道具 | 4 / 10 |
| 34 | `action:damage.flags:reflection` | ability_kit | 共用 | A16 | 5 招、1 件道具 | 4 / 10 |
| 35 | `event.damage.is_reflection` | ability_kit | 共用 | A16 | 3 招、1 件道具 | 5 / 10 |
| 36 | `event.damage.raw` | ability_kit | 共用 | B21 | 1 招、1 件道具 | 5 / 10 |
| 37 | `passive_trigger:on_heal_received` | ability_kit | hook 候選 | B21 | 1 招、0 件道具 | 6 / 10 |
| 38 | `action:spend_mana` | ability_kit | 共用 | C09 | 6 招、2 件道具 | 6 / 10 |
| 39 | `action:toggle_ability` | ability_kit | 共用 | C09 | 7 招、0 件道具 | 6 / 10 |
| 40 | `attr:remaining` | 產生器 | hook 候選 | C09 | 1 招、0 件道具 | 7 / 10 |
| 41 | `action:cue.at of a sym:landing` | ability_kit | hook 候選 | C01 | 1 招、0 件道具 | 7 / 10 |
| 42 | `action:random_pick` | ability_kit | 共用 | C01 | 2 招、0 件道具 | 7 / 10 |
| 43 | `expr: a position from a sym:landing` | 產生器 | hook 候選 | C01 | 1 招、0 件道具 | 8 / 10 |
| 44 | `event:point` | ability_kit | 共用 | A19 | 2 招、0 件道具 | 8 / 10 |
| 45 | `fn:rotate` | 產生器 | 共用 | A19 | 9 招、3 件道具 | 8 / 10 |
| 46 | `fn:select types` | 產生器 | 共用 | A19 | 3 招、0 件道具 | 9 / 10 |
| 47 | `action:issue_order.force` | ability_kit | 共用 | A30 | 5 招、0 件道具 | 9 / 10 |
| 48 | `action:zone` | ability_kit | 共用 | A30 | 3 招、0 件道具 | 9 / 10 |
| 49 | `fn:has_tag:boss of a unit` | 產生器 | 共用 | A30 | 2 招、0 件道具 | 9 / 10 |
| 50 | `state:provides_vision` | ability_kit | 共用 | A30 | 4 招、1 件道具 | 10 / 10 |

### M6 第 3 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 51 | `status_trigger:periodic.filter` | ability_kit | 共用 | C14 | 3 招、0 件道具 | 2 / 9 |
| 52 | `status.reapply_if_removed` | ability_kit | 共用 | A22 | 1 招、14 件道具 | 2 / 9 |
| 53 | `status.unique_group` | ability_kit | 共用 | A22 | 2 招、1 件道具 | 3 / 9 |
| 54 | `expr of type nil` | 產生器 | 共用 | C04 | 15 招、0 件道具 | 3 / 9 |
| 55 | `selector.shape:global` | 產生器 | 共用 | C04 | 13 招、0 件道具 | 4 / 9 |
| 56 | `fn:armor_multiplier` | 產生器 | 共用 | C20 | 5 招、2 件道具 | 4 / 9 |
| 57 | `status.scope` | ability_kit | 共用 | C20 | 6 招、3 件道具 | 5 / 9 |
| 58 | `attr:is_ready` | 產生器 | 共用 | A05 | 1 招、3 件道具 | 6 / 9 |
| 59 | `attr:stacks` | 產生器 | 共用 | C25 | 3 招、0 件道具 | 6 / 9 |
| 60 | `fn:ability_in_slot` | 產生器 | 共用 | C25 | 2 招、0 件道具 | 7 / 9 |
| 61 | `action:projectile.from:spawn` | ability_kit | hook 候選 | B31 | 1 招、0 件道具 | 7 / 9 |
| 62 | `fn:is_visible_to` | 產生器 | 共用 | B31 | 3 招、3 件道具 | 7 / 9 |
| 63 | `fn:random_int` | 產生器 | 共用 | B31 | 4 招、2 件道具 | 8 / 9 |
| 64 | `attr:mana_pct` | 產生器 | 共用 | C11 | 2 招、1 件道具 | 8 / 9 |
| 65 | `passive_trigger:on_kill` | ability_kit | 共用 | C11 | 8 招、5 件道具 | 8 / 9 |
| 66 | `selector.flag:include_hidden` | 產生器 | 共用 | C11 | 3 招、0 件道具 | 9 / 9 |

### M6 第 4 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 67 | `selector.flag:exclude_invisible` | 產生器 | hook 候選 | B03 | 1 招、0 件道具 | 1 / 9 |
| 68 | `action:repeat in a routine` | ability_kit | 共用 | B30 | 2 招、0 件道具 | 1 / 9 |
| 69 | `fn:has_tag:fountain of a unit` | 產生器 | 共用 | B30 | 1 招、1 件道具 | 2 / 9 |
| 70 | `action:damage.source with flags or credit` | ability_kit | 共用 | A24 | 2 招、0 件道具 | 2 / 9 |
| 71 | `action:face` | ability_kit | 共用 | A24 | 12 招、1 件道具 | 2 / 9 |
| 72 | `passive_trigger:on_created` | ability_kit | 共用 | A24 | 2 招、44 件道具 | 3 / 9 |
| 73 | `action:create_illusion` | ability_kit | 共用 | C02 | 10 招、0 件道具 | 3 / 9 |
| 74 | `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | C02 | 6 招、0 件道具 | 4 / 9 |
| 75 | `state type:point` | ability_kit | 共用 | B02 | 2 招、0 件道具 | 4 / 9 |
| 76 | `state:low_attack_priority` | ability_kit | 共用 | B02 | 7 招、0 件道具 | 5 / 9 |
| 77 | `action:kill` | ability_kit | 共用 | B34 | 11 招、2 件道具 | 5 / 9 |
| 78 | `status.stacking:independent with a source` | ability_kit | 共用 | B34 | 2 招、0 件道具 | 5 / 9 |
| 79 | `status.stacking:independent with slow_pct` | ability_kit | 共用 | B34 | 2 招、0 件道具 | 6 / 9 |
| 80 | `action:create_unit.control:owner` | ability_kit | 共用 | B17 | 15 招、6 件道具 | 6 / 9 |
| 81 | `action:create_unit.overrides` | ability_kit | 共用 | B17 | 11 招、4 件道具 | 6 / 9 |
| 82 | `event.damage.is_attack` | ability_kit | 共用 | B17 | 2 招、0 件道具 | 7 / 9 |
| 83 | `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | A36 | 3 招、0 件道具 | 7 / 9 |
| 84 | `expr: wanted a unit, got a sym:caster_hero` | 產生器 | 共用 | A36 | 2 招、0 件道具 | 7 / 9 |
| 85 | `fn:alive of a sym:caster_hero` | 產生器 | hook 候選 | A36 | 1 招、0 件道具 | 8 / 9 |
| 86 | `action:remove_status of an independent status (line B: instance removal)` | ability_kit | 共用 | A06 | 1 招、3 件道具 | 8 / 9 |
| 87 | `passive_trigger:on_attack_launch` | ability_kit | 共用 | A06 | 4 招、0 件道具 | 8 / 9 |
| 88 | `status.vars of an independent status` | ability_kit | 共用 | A06 | 2 招、0 件道具 | 8 / 9 |
| 89 | `status_trigger:on_attack_miss` | ability_kit | hook 候選 | A06 | 1 招、0 件道具 | 8 / 9 |
| 90 | `target.type:unit_or_point` | 產生器 | 共用 | A06 | 3 招、1 件道具 | 9 / 9 |

### M6 第 5 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 91 | `action:set_time_of_day` | ability_kit | 共用 | B14 | 2 招、0 件道具 | 1 / 9 |
| 92 | `fn:is_template` | 產生器 | 共用 | A27 | 8 招、0 件道具 | 2 / 9 |
| 93 | `attr:owner` | 產生器 | hook 候選 | A20 | 1 招、0 件道具 | 2 / 9 |
| 94 | `state:not_on_minimap` | ability_kit | 共用 | A20 | 4 招、0 件道具 | 3 / 9 |
| 95 | `action:attack` | ability_kit | 共用 | B35 | 5 招、2 件道具 | 3 / 9 |
| 96 | `fn:has_status_tag` | 產生器 | 共用 | B35 | 5 招、13 件道具 | 4 / 9 |
| 97 | `stat.when` | ability_kit | 共用 | C10 | 1 招、4 件道具 | 4 / 9 |
| 98 | `stat:vision_night` | ability_kit | 共用 | C10 | 6 招、3 件道具 | 5 / 9 |
| 99 | `fn:min of non-numbers` | 產生器 | 共用 | A17 | 1 招、2 件道具 | 5 / 9 |
| 100 | `stat:vision_day` | ability_kit | 共用 | A17 | 4 招、0 件道具 | 6 / 9 |
| 101 | `action:cue.at of a err` | ability_kit | 共用 | B05 | 4 招、0 件道具 | 6 / 9 |
| 102 | `state:revealed` | ability_kit | 共用 | B05 | 4 招、6 件道具 | 7 / 9 |
| 103 | `selector.order:random` | 產生器 | 共用 | B18 | 6 招、1 件道具 | 7 / 9 |
| 104 | `selector.types:building+creep+neutral+summon` | 產生器 | 共用 | B18 | 1 招、1 件道具 | 8 / 9 |
| 105 | `fn:avg_attack_against` | 產生器 | 共用 | A25 | 3 招、0 件道具 | 8 / 9 |
| 106 | `fn:offset types` | 產生器 | 共用 | A25 | 1 招、1 件道具 | 8 / 9 |
| 107 | `op:%` | 產生器 | 共用 | A25 | 2 招、0 件道具 | 9 / 9 |

### M6 第 6 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 108 | `action:refund` | ability_kit | 共用 | A33 | 2 招、2 件道具 | 2 / 9 |
| 109 | `fn:has_tag:dummy of a unit` | 產生器 | 共用 | A32 | 2 招、0 件道具 | 3 / 9 |
| 110 | `status.stacking:replace` | ability_kit | 共用 | B10 | 9 招、0 件道具 | 4 / 9 |
| 111 | `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 4 / 9 |
| 112 | `status_trigger:on_kill` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 5 / 9 |
| 113 | `action:impulse` | ability_kit | hook 候選 | B25 | 1 招、0 件道具 | 5 / 9 |
| 114 | `attr:cooldown_remaining` | 產生器 | hook 候選 | B25 | 1 招、0 件道具 | 5 / 9 |
| 115 | `fn:max of non-numbers` | 產生器 | 共用 | B25 | 3 招、0 件道具 | 6 / 9 |
| 116 | `action:cancel_delay` | ability_kit | 共用 | A10 | 2 招、0 件道具 | 6 / 9 |
| 117 | `action:delay.id` | ability_kit | 共用 | A10 | 2 招、0 件道具 | 6 / 9 |
| 118 | `action:swap_positions` | ability_kit | 共用 | A10 | 4 招、0 件道具 | 7 / 9 |
| 119 | `action:heal.source:owner_of(self)` | ability_kit | 共用 | C16 | 2 招、0 件道具 | 7 / 9 |
| 120 | `attr:game.dt` | 產生器 | 共用 | C16 | 2 招、0 件道具 | 7 / 9 |
| 121 | `fn:owner_of` | 產生器 | 共用 | C16 | 2 招、1 件道具 | 7 / 9 |
| 122 | `state:flying` | ability_kit | 共用 | C16 | 3 招、0 件道具 | 8 / 9 |
| 123 | `passive_trigger:periodic.filter` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 8 / 9 |
| 124 | `status_trigger:on_mana_spent` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 9 / 9 |

### M6 第 7 批（9 位）

開始時已全部產生：3 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 125 | `status_trigger:on_moved` | ability_kit | 共用 | A03 | 3 招、0 件道具 | 4 / 9 |
| 126 | `fn:status_var` | 產生器 | 共用 | C12 | 4 招、0 件道具 | 5 / 9 |
| 127 | `stat:outgoing_damage_pct` | ability_kit | 共用 | B32 | 4 招、2 件道具 | 6 / 9 |
| 128 | `requires.condition` | ability_kit | 共用 | A31 | 5 招、0 件道具 | 7 / 9 |
| 129 | `fn:count_in_radius` | 產生器 | 共用 | C21 | 1 招、1 件道具 | 8 / 9 |
| 130 | `selector.flag:exclude_illusions` | 產生器 | hook 候選 | B36 | 1 招、0 件道具 | 8 / 9 |
| 131 | `selector.team:ally_not_self` | 產生器 | 共用 | B36 | 2 招、0 件道具 | 9 / 9 |

### M6 第 8 批（9 位）

開始時已全部產生：3 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 132 | `action:end_channel` | ability_kit | 共用 | B11 | 2 招、0 件道具 | 4 / 9 |
| 133 | `event:on_channel_tick` | ability_kit | 共用 | B09 | 3 招、0 件道具 | 4 / 9 |
| 134 | `name:event outside an event` | 產生器 | 共用 | B09 | 4 招、2 件道具 | 5 / 9 |
| 135 | `fn:enemy_team` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 5 / 9 |
| 136 | `fn:is_ally` | 產生器 | 共用 | B12 | 3 招、2 件道具 | 5 / 9 |
| 137 | `fn:team_of` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 6 / 9 |
| 138 | `expr: wanted a bool, got a sym:heal_chain` | 產生器 | hook 候選 | A28 | 1 招、0 件道具 | 6 / 9 |
| 139 | `stat:physical_block_chance` | ability_kit | 共用 | A28 | 1 招、2 件道具 | 7 / 9 |
| 140 | `action:issue_order.order:follow` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 141 | `state type:vec` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 142 | `status_trigger:on_refreshed` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 8 / 9 |
| 143 | `action:reveal.teams:enemy` | ability_kit | 共用 | B23 | 2 招、0 件道具 | 8 / 9 |
| 144 | `passive_trigger:on_moved` | ability_kit | hook 候選 | B23 | 1 招、0 件道具 | 8 / 9 |
| 145 | `selector.flag:include_dead` | 產生器 | 共用 | B23 | 2 招、0 件道具 | 9 / 9 |

### M6 第 9 批（13 位）

開始時已全部產生：0 位；做完本批的項目後：13 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 146 | `selector.relative_to` | 產生器 | 共用 | C07 | 2 招、0 件道具 | 0 / 13 |
| 147 | `stat:heal_reduction_pct` | ability_kit | 共用 | C07 | 2 招、2 件道具 | 1 / 13 |
| 148 | `fn:edge_distance` | 產生器 | 共用 | B33 | 4 招、0 件道具 | 1 / 13 |
| 149 | `token:vars` | 產生器 | 共用 | B33 | 1 招、2 件道具 | 2 / 13 |
| 150 | `action:chain` | ability_kit | hook 候選 | B28 | 1 招、0 件道具 | 2 / 13 |
| 151 | `fn:has_state:revealed` | 產生器 | hook 候選 | B28 | 1 招、0 件道具 | 3 / 13 |
| 152 | `fn:orb_slot_free` | 產生器 | 共用 | B15 | 2 招、0 件道具 | 3 / 13 |
| 153 | `passive:orb_slot` | ability_kit | 共用 | B15 | 1 招、1 件道具 | 4 / 13 |
| 154 | `fn:landmark` | 產生器 | hook 候選 | A21 | 1 招、0 件道具 | 4 / 13 |
| 155 | `status.forced_move` | ability_kit | 共用 | A21 | 2 招、0 件道具 | 5 / 13 |
| 156 | `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | A11 | 2 招、2 件道具 | 5 / 13 |
| 157 | `attr:cast_point of an ability` | 產生器 | hook 候選 | A11 | 1 招、0 件道具 | 6 / 13 |
| 158 | `action:give_item` | ability_kit | 共用 | B07 | 3 招、0 件道具 | 6 / 13 |
| 159 | `action:pick_item` | ability_kit | 共用 | B07 | 3 招、0 件道具 | 6 / 13 |
| 160 | `fn:exists of a sym:item_a` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 6 / 13 |
| 161 | `fn:exists of a sym:item_b` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 7 / 13 |
| 162 | `action:remove_item` | ability_kit | 共用 | C06 | 2 招、0 件道具 | 7 / 13 |
| 163 | `attr:name` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 7 / 13 |
| 164 | `fn:exists of a sym:stolen` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 7 / 13 |
| 165 | `state type:item_name` | ability_kit | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 166 | `action:modify_cooldown.item` | ability_kit | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 167 | `fn:ability_ready` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 168 | `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 169 | `fn:status_source` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 8 / 13 |
| 170 | `selector.flag:include_illusions` | 產生器 | 共用 | C18 | 5 招、1 件道具 | 9 / 13 |
| 171 | `action:add_ability` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 9 / 13 |
| 172 | `action:cue.at of a sym:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 173 | `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 174 | `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 9 / 13 |
| 175 | `action:set_base_max_hp` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 9 / 13 |
| 176 | `token:state` | 產生器 | hook 候選 | B16 | 1 招、0 件道具 | 10 / 13 |
| 177 | `action:projectile.team:any` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 178 | `action:projectile.where` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 179 | `action:redirect_motion` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 180 | `fn:circle_point` | 產生器 | 共用 | B24 | 2 招、0 件道具 | 10 / 13 |
| 181 | `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 10 / 13 |
| 182 | `selector.types:building` | 產生器 | 共用 | B24 | 2 招、0 件道具 | 11 / 13 |
| 183 | `action:destroy_tree` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 184 | `action:regrow_trees` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 185 | `fn:tree_alive` | 產生器 | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 186 | `status.immune_to` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 11 / 13 |
| 187 | `target.type:tree` | 產生器 | 共用 | B22 | 2 招、0 件道具 | 12 / 13 |
| 188 | `action:remove_ability` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 189 | `action:set_ability_active (an ability id and a fixed active)` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 190 | `attr:level` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 191 | `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 192 | `selector.shape:segment` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 193 | `selector.to` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 194 | `selector.width` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 12 / 13 |
| 195 | `state:blind` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 13 / 13 |

## 2. 每位英雄、每招還缺什麼

### A02 `inahime`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a02_venom_arrow` | ✓ 已產生 |
| `a02_wind_prayer` | ✓ 已產生 |
| `a02_wind_drift` | ✓ 已產生 |
| `a02_shadow_meld` | ✓ 已產生 |

### A07 `honda_tadakatsu`（3 / 6 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a07_unscathed` | ✓ 已產生 |
| `a07_red_thunder` | `routine a07e_cast: action:delay in a routine` |
| `a07_shattering_spear` | `action:modify_attack` |
| `a07_strongest` | `attr:damage`、`event:attack`、`stat:status_resistance_pct` |
| `a07_rouse` | ✓ 已產生 |
| `a07_rouse_free` | ✓ 已產生 |

### A34 `omatsu`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a34_frost_bolt` | ✓ 已產生 |
| `a34_iceberg_strike` | ✓ 已產生 |
| `a34_frost_breath` | ✓ 已產生 |
| `a34_rapid_cooling` | ✓ 已產生 |

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
| `b04_flame_trail_dash` | `status.aura` |
| `b04_fire_blast` | ✓ 已產生 |
| `b04_flame_swiftness` | ✓ 已產生 |
| `b04_tanegashima` | `status_trigger:on_attack_launch` |

### B06 `sanada_yukimura`（1 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b06_brave_warriors` | ✓ 已產生 |
| `b06_flame_fangs` | `routine b06e_bolt: action:projectile in a routine` |
| `b06_mind_eye` | `passive_trigger:on_order_targeted` |
| `b06_first_warrior` | `status.aura` |

### B08 `asai_nagamasa`（4 / 5 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b08_fang_slash` | ✓ 已產生 |
| `b08_flash_cut` | ✓ 已產生 |
| `b08_living_soul` | ✓ 已產生 |
| `b08_sky_fang` | ✓ 已產生 |
| `b08_endless_fang` | `action:reveal.ignore_terrain` |

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
| `c15_burst_arrow` | `action:create_unit.control:none` |
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

### A01 `tokugawa_ieyasu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a01_realm_momentum` | ✓ 已產生 |
| `a01_tosho_gongen` | ✓ 已產生 |
| `a01_heaven_net` | ✓ 已產生 |
| `a01_thunder_blade` | ✓ 已產生 |
| `a01_mikawa_valor` | ✓ 已產生 |

### A03 `ishikawa_kazumasa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a03_pacing_gu` | `status_trigger:on_moved` |
| `a03_spider_form` | `action:toggle_ability`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`status.scope` |
| `a03_enchanting_venom` | ✓ 已產生 |
| `a03_insect_swarm` | ✓ 已產生 |

### A04 `takenaka_shigeharu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a04_water_shikigami` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:issue_order.order:follow`、`action:set_display_stacks`、`status_trigger:periodic.filter` |
| `a04_ice_path` | `event:on_phase_start`、`state type:vec` |
| `a04_frost_shelter` | `status.absorb`、`status_trigger:on_refreshed` |
| `a04_frost_sparrow` | ✓ 已產生 |
| `a04_blizzard` | ✓ 已產生 |

### A05 `sakai_tadatsugu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a05_valiant_power` | `status.stacking:independent with triggers` |
| `a05_mikawa_courage` | ✓ 已產生 |
| `a05_counter_stance` | ✓ 已產生 |
| `a05_loyal_secret` | `attr:is_ready`、`state:no_healthbar` |
| `a05_crusade_affidavit` | `status.scope` |

### A06 `ii_naomasa`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a06_red_power` | ✓ 已產生 |
| `a06_red_meteor` | ✓ 已產生 |
| `a06_red_flame_flow` | `passive:aura`、`passive_trigger:on_attack_launch` |
| `a06_flame_of_anger` | `target.type:unit_or_point` |
| `a06_red_fury` | `action:remove_status of an independent status (line B: instance removal)`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`status.stacking:independent with triggers`、`status.vars of an independent status`、`status_trigger:on_attack_miss` |

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
| `a09_treant_summon` | `action:create_unit.control:none`、`action:create_unit.control:owner`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.stacking:independent with a source`、`status.stacking:independent with triggers` |
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
| `a11_vortex` | `action:damage.flags:no_source_bonuses` |
| `a11_evil_gaze` | `event:on_channel_end`、`stat:move_speed`、`status.forced_move`、`status.status_resistance` |
| `a11_old_fox` | `attr:cast_point of an ability`、`name:event outside an event`、`passive:aura` |
| `a11_myriad_forms` | `status.aura`、`status.status_resistance` |

### A12 `otani_yoshitsugu`（5 / 9 招）

| 招式 | 還缺 |
|---|---|
| `a12_insight_flash` | `action:knockback` |
| `a12_scheme` | `action:set_display_stacks` |
| `a12_mystic_pierce` | ✓ 已產生 |
| `a12_awe` | ✓ 已產生 |
| `a12_mystic_arrow` | `action:set_display_stacks`、`status.status_resistance` |
| `a12_mystic_possession` | ✓ 已產生 |
| `a12_insight_flash_t` | `action:knockback` |
| `a12_mystic_field` | ✓ 已產生 |
| `a12_awe_t` | ✓ 已產生 |

### A13 `hattori_hanzo`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a13_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |
| `a13_shadow_illusion` | `action:create_illusion`、`action:cue.at of a sym:il`、`action:dodge_projectiles.of:il`、`action:face`、`expr: * of num and sym:me_slot`、`expr: + of num and sym:rot`、`expr: wanted a unit, got a sym:il`、`expr: ~= of num and sym:me_slot`、`fn:exists of a sym:il`、`fn:random_int` |
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

### A16 `torii_mototada`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a16_bloody_grasp` | ✓ 已產生 |
| `a16_blood_link` | ✓ 已產生 |
| `a16_blood_aura` | `action:damage.flags:reflection`、`event.damage.is_reflection` |
| `a16_dark_blood_decree` | ✓ 已產生 |

### A17 `sassa_narimasa`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a17_shrapnel` | ✓ 已產生 |
| `a17_rapid_fire` | `attr:damage`、`event:attack`、`fn:min of non-numbers` |
| `a17_vital_shot` | `passive_trigger:on_attack_launch` |
| `a17_great_cannon` | `attr:damage`、`event:attack`、`stat:vision_day`、`stat:vision_night` |

### A18 `inaba_ittetsu`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a18_obsessive_pressure` | ✓ 已產生 |
| `a18_suppress` | ✓ 已產生 |
| `a18_stubborn_tactics` | `action:modify_attack`、`status.absorb` |
| `a18_by_force` | ✓ 已產生 |

### A19 `kuroda_yoshitaka`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a19_fire_stratagem` | ✓ 已產生 |
| `a19_chain_fire` | ✓ 已產生 |
| `a19_fire_echo` | `event:point`、`fn:select types` |
| `a19_siege_fire` | `fn:rotate` |

### A20 `kuki_yoshitaka`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_pirate_domain` | `action:create_unit.limit_per_owner (no host cap for it)`、`attr:owner`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.aura` |
| `a20_water_burst` | `fn:has_status of an independent status (line B)`、`status.stacking:independent with slow_pct`、`status.status_resistance` |
| `a20_pirate_boots` | `routine a20r_wave: action:repeat in a routine`、`routine a20r_wave: fn:rotate` |
| `a20_pirate_daimyo` | `stat:status_resistance_pct`、`status.unique_group` |

### A21 `zuiryuin_nisshu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_mana_steal` | ✓ 已產生 |
| `a21_half_blade` | `action:toggle_ability`、`fn:orb_slot_free` |
| `a21_energy_press` | ✓ 已產生 |
| `a21_pearl_disc` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:reveal.teams:enemy`、`selector.order:random`、`state:magic_immune`、`status.aura` |
| `a21_false_intel` | `fn:landmark`、`status.forced_move`、`status.status_resistance` |

### A22 `sengoku_hidehisa`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a22_flame_whirl` | `state:magic_immune` |
| `a22_flame_heart` | ✓ 已產生 |
| `a22_boiling_blood` | `passive:aura`、`status.unique_group` |
| `a22_torment` | `status.reapply_if_removed` |

### A23 `ukita_hideie`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a23_cursed_blood` | ✓ 已產生 |
| `a23_soul_bind` | ✓ 已產生 |
| `a23_falling_fire` | `status.stacking:independent with triggers` |
| `a23_yomi_flame` | `state:break` |

### A24 `sakuma_morimasa`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a24_blood_shield` | `passive:visibility`、`passive_trigger:on_created` |
| `a24_slaughter_shot` | `action:face` |
| `a24_blood_charge` | `routine a24_blood_price: action:damage.source with flags or credit`、`routine a24d_sync: action:set_display_stacks`、`routine a24d_sync: status.absorb` |
| `a24_blood_resonance` | `action:reveal.ignore_terrain`、`passive:visibility` |
| `a24_bloodbath` | `routine a24_blood_price: action:damage.source with flags or credit`、`routine a24d_sync: action:set_display_stacks`、`routine a24d_sync: status.absorb` |

### A25 `oda_nobunaga`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a25_lightning_lure` | `fn:avg_attack_against` |
| `a25_calamity_thorns` | ✓ 已產生 |
| `a25_heshikiri` | `action:modify_attack` |
| `a25_crimson_barrier` | `fn:offset types`、`fn:rotate`、`op:%`、`state:magic_immune` |
| `a25_tenka_fubu` | `passive_trigger:on_kill` |

### A26 `nohime`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a26_volley` | ✓ 已產生 |
| `a26_powder_kick` | `action:knockback` |
| `a26_twin_guns` | `action:modify_attack` |
| `a26_scorching_bomb` | `event:on_phase_start`、`expr: wanted a unit, got a point` |

### A27 `shibata_katsuie`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a27_unstoppable_advance` | `action:create_illusion` |
| `a27_war_banner` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`fn:is_template` |
| `a27_way_of_warrior` | `action:modify_attack` |
| `a27_tyrant_hammer` | ✓ 已產生 |
| `a27_oni_shibata` | ✓ 已產生 |

### A28 `hashiba_hideyoshi`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_evil_art` | `stat:physical_block_chance` |
| `a28_creation` | `expr: wanted a bool, got a sym:heal_chain`、`fn:is_ally` |
| `a28_binding_swamp` | ✓ 已產生 |
| `a28_phoenix_summon` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill` |
| `a28_heaven_fire` | ✓ 已產生 |
| `a28_spreading_fire` | `fn:has_status_tag` |

### A29 `ishida_mitsunari`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_sleeve_poison_arrow` | ✓ 已產生 |
| `a29_abyss_grip` | ✓ 已產生 |
| `a29_withering_aura` | `passive:aura`、`passive_trigger:periodic.filter`、`status_trigger:on_mana_spent` |
| `a29_vulture_swarm` | `action:cue.at of a err`、`action:heal.source:owner_of(self)`、`attr:game.dt`、`fn:owner_of`、`status.stacking:replace` |

### A30 `niwa_nagahide`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_wolf_mark` | `action:create_unit.control:none`、`action:issue_order.force`、`state:provides_vision` |
| `a30_earth_fang` | ✓ 已產生 |
| `a30_fire_sea` | `action:zone` |
| `a30_red_fuji` | `fn:has_tag:boss of a unit` |

### A31 `mori_ranmaru`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a31_echoing_rain` | ✓ 已產生 |
| `a31_shunpo` | `requires.condition` |
| `a31_storm_arrow` | ✓ 已產生 |
| `a31_samidare` | `action:attack`、`event:on_channel_end`、`passive_trigger:on_attack_launch`、`selector.order:random`、`stat:outgoing_damage_pct` |

### A32 `takigawa_kazumasu`（0 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a32_flying_slash` | `action:leap`、`fn:has_tag:dummy of a unit` |
| `a32_second_slash` | `action:leap` |
| `a32_soul_drain` | `op:%` |
| `a32_phantom_blade` | `action:modify_attack` |
| `a32_soul_blade` | `action:toggle_ability`、`fn:avg_attack_against` |
| `a32_asura_blade` | `action:knockback`、`target.type:unit_or_point` |

### A33 `honda_masanobu`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_dark_summoning` | `action:create_unit.control:owner` |
| `a33_ghost_step` | `action:set_display_stacks` |
| `a33_corrupting_aura` | `passive:aura` |
| `a33_deepen_wounds` | `action:refund`、`state:provides_vision`、`state:revealed` |

### A35 `shima_sakon`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_flame_raid` | ✓ 已產生 |
| `a35_sealing_blaze` | `state:break` |
| `a35_flame_swiftness` | ✓ 已產生 |
| `a35_scorching_fire` | ✓ 已產生 |

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
| `b03_zhongkui_judgment` | `selector.flag:exclude_invisible` |

### B05 `mori_motonari`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_chain_lightning` | `action:cue.at of a err`、`status.stacking:independent with triggers`、`status.vars of an independent status` |
| `b05_heavenly_thunder` | ✓ 已產生 |
| `b05_three_arrows` | `passive:aura`、`state:revealed` |
| `b05_holy_light_burst` | `event:on_phase_start`、`status.status_resistance` |

### B07 `ashikaga_yoshiaki`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b07_starfall` | ✓ 已產生 |
| `b07_battle_line` | `action:create_unit.control:owner`、`action:create_unit.limit_per_owner (no host cap for it)`、`action:create_unit.overrides`、`action:give_item`、`action:pick_item`、`fn:exists of a sym:item_a`、`fn:exists of a sym:item_b`、`state:revealed` |
| `b07e_camp_regen` | `passive:aura` |
| `b07_glorious_momentum` | `passive:aura` |
| `b07_reign_over_all` | ✓ 已產生 |

### B09 `sanada_masayuki`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b09_backlash_rift` | `action:set_display_stacks`、`fn:status_var`、`name:event outside an event` |
| `b09_mind_burst` | `event:on_channel_end`、`event:on_channel_tick` |
| `b09_will_shield` | `action:spend_mana` |
| `b09_mind_suppression` | `status.status_resistance` |

### B10 `miyoshi_nagayoshi`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b10_wrath_of_the_dead` | `status.stacking:replace` |
| `b10_corpse_assault` | ✓ 已產生 |
| `b10_demonic_war_spirit` | `passive:aura` |
| `b10_power_of_the_abyss` | ✓ 已產生 |

### B11 `kikuhime`（1 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b11_cherry_rain` | ✓ 已產生 |
| `b11_breath_control` | `action:end_channel`、`event:on_channel_end` |
| `b11_interception` | `routine b11r_spawn: action:create_illusion`、`routine b11r_spawn: expr: wanted a unit, got a sym:ill`、`routine b11r_spawn: stat:move_speed` |
| `b11_crow_raid` | `routine b11r_spawn: action:create_illusion`、`routine b11r_spawn: expr: wanted a unit, got a sym:ill`、`routine b11r_spawn: stat:move_speed` |
| `b11_crow_strike` | `action:modify_attack` |
| `b11_inherited_blood` | `passive_trigger:on_kill` |

### B12 `yamamoto_kansuke`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b12_reality_slide` | `fn:enemy_team`、`fn:is_ally`、`fn:team_of` |
| `b12_floating_life` | ✓ 已產生 |
| `b12_scheming_offense` | `passive:aura` |
| `b12_mad_heart` | ✓ 已產生 |

### B13 `momochi_sandayu`（1 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b13_sense_break` | ✓ 已產生 |
| `b13_hypnosis` | `state:low_attack_priority` |
| `b13_blood_disguise` | `passive_trigger:on_kill` |
| `b13_burrow` | `stat:vision_night` |
| `b13_shukuchi` | `requires.condition` |
| `b13_storm` | `status.aura` |

### B14 `katakura_kagetsuna`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_temporary_night` | `action:set_time_of_day`、`event:on_phase_start` |
| `b14_iai_slash` | ✓ 已產生 |
| `b14_shadow_tactics` | `state:magic_immune` |
| `b14_attendant_insight` | `action:modify_attack` |
| `b14_shadow_cut` | `state:low_attack_priority` |

### B15 `imagawa_yoshimoto`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_tokai_giant` | `state:magic_immune` |
| `b15_flame_arrow` | `action:spend_mana`、`action:toggle_ability`、`fn:orb_slot_free`、`passive:orb_slot` |
| `b15_giant_hide` | ✓ 已產生 |
| `b15_multishot` | `passive_trigger:on_attack_launch` |
| `b15_bluff` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill`、`status.stacking:independent with states`、`status.stacking:independent with triggers` |

### B16 `mochizuki_chiyome`（2 / 9 招）

| 招式 | 還缺 |
|---|---|
| `b16_flying_raid` | `action:dodge_projectiles.of:state.b16_moon`、`action:face` |
| `b16_falling_leaves` | ✓ 已產生 |
| `b16_swift_thunder` | `passive:visibility` |
| `b16_summon_hound` | `action:add_ability`、`action:create_unit.control:owner`、`action:create_unit.facing`、`action:kill`、`action:remove_unit`、`fn:edge_distance` |
| `b16_phase_shift` | `action:dodge_projectiles.of:state.b16_moon`、`action:swap_positions`、`requires.condition` |
| `b16_beast_union` | `action:refund`、`action:set_base_max_hp`、`fn:edge_distance`、`token:state` |
| `b16_mm_phase_shift` | `action:cue.at of a sym:caster_hero`、`action:dodge_projectiles.of:caster_hero`、`action:swap_positions`、`requires.condition` |
| `b16_mm_sweep` | ✓ 已產生 |
| `b16_mm_stealth` | `passive_trigger:on_created`、`status_trigger:on_attack_launch` |

### B17 `kirigakure_saizo`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_death_ward` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill`、`event.damage.is_attack`、`state:magic_immune`、`state:no_healthbar`、`status.aura` |
| `b17_tooth_poison` | ✓ 已產生 |
| `b17_shadow_leap` | ✓ 已產生 |
| `b17_heartseeker` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |

### B18 `koihime`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_earth_raid` | ✓ 已產生 |
| `b18_echo` | `selector.order:random` |
| `b18_earth_shelter` | ✓ 已產生 |
| `b18_earth_claw` | `action:create_unit.control:none`、`action:create_unit.overrides`、`action:issue_order.force`、`fn:is_visible_to`、`selector.types:building+creep+neutral+summon`、`status.aura` |

### B19 `usami_sadamitsu`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b19_water_dragon` | `action:cue.at of a err`、`state type:point` |
| `b19_ice_fence` | ✓ 已產生 |
| `b19_echigo_tactics` | `expr of type nil`、`selector.shape:global` |
| `b19_kuruma_gakari` | `event:on_channel_end` |

### B20 `yamagata_masakage`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b20_rock_rush` | ✓ 已產生 |
| `b20_ruin_wave` | ✓ 已產生 |
| `b20_tenacity` | `passive:visibility` |
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
| `b22_natural_barrier` | `action:create_unit.control:none`、`fn:circle_point`、`state:magic_immune` |
| `b22_force_of_nature` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:destroy_tree`、`state:flying`、`target.type:tree` |
| `b22_purify` | `action:kill`、`fn:is_ally`、`status.immune_to` |
| `b22_wood_spirit` | `action:create_unit.control:none`、`action:remove_unit`、`action:reveal.ignore_terrain`、`fn:tree_alive`、`state:revealed`、`target.type:tree` |
| `b22_wrath_of_nature` | `action:create_unit.control:none`、`action:regrow_trees`、`action:remove_unit`、`action:reveal.ignore_terrain` |

### B23 `ise_hime`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_undead_pact` | `action:create_unit.control:owner`、`action:create_unit.overrides`、`action:kill`、`action:set_display_stacks`、`event:on_phase_start`、`expr of type nil`、`fn:is_template`、`passive:visibility`、`passive_trigger:on_moved`、`selector.flag:include_dead`、`selector.shape:global`、`status_trigger:on_moved` |
| `b23_spirit_gathering` | `expr of type nil`、`fn:is_template`、`selector.shape:global` |
| `b23_living_soul_shock` | `expr of type nil`、`fn:is_template`、`selector.shape:global` |
| `b23_soul_feast` | `event:on_channel_end`、`event:on_channel_tick`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`state:magic_immune` |
| `b23_heroic_spirit_blessing` | `action:reveal.teams:enemy`、`event:on_channel_end`、`event:on_channel_tick`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`status.aura` |

### B24 `akiyama_nobutomo`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b24_earth_stele` | `action:create_unit.overrides`、`action:kill`、`action:knockback`、`action:redirect_motion`、`fn:is_template`、`selector.flag:include_illusions`、`selector.order:random`、`selector.types:boss+creep+neutral+summon`、`selector.types:building` |
| `b24_taunt` | `action:issue_order.force`、`selector.flag:include_illusions` |
| `b24_stone_resonance` | `action:projectile.team:any`、`action:projectile.where`、`expr of type nil`、`selector.flag:include_illusions` |
| `b24_earth_prison` | `fn:circle_point`、`selector.flag:include_illusions` |
| `b24_shatter` | `action:kill`、`expr of type nil`、`fn:is_template`、`selector.shape:global`、`selector.types:building` |

### B25 `honganji_kennyo`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b25_exorcism_strike` | ✓ 已產生 |
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

### B28 `matsunaga_hisahide`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_phoenix_flame` | ✓ 已產生 |
| `b28_flame_shackle` | `action:end_channel`、`event:on_channel_end`、`fn:edge_distance`、`fn:max of non-numbers`、`name:event outside an event` |
| `b28_fire_stream` | ✓ 已產生 |
| `b28_serpent_fire` | `action:chain`、`fn:has_state:revealed` |

### B30 `kakizaki_kageie`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_reverse_shield` | `fn:has_tag:fountain of a unit` |
| `b30_soul_breaker` | ✓ 已產生 |
| `b30_flash_strike` | ✓ 已產生 |
| `b30_purgatory_slash` | `routine b30t_ignite: action:repeat in a routine` |

### B31 `takeda_shingen`（0 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_swift_wind` | `status.aura` |
| `b31_still_forest` | `action:projectile.from:spawn`、`fn:random_int`、`fn:rotate`、`status.aura` |
| `b31_raging_fire` | `fn:is_visible_to`、`status.aura`、`status_trigger:periodic.filter` |
| `b31_immovable_mountain` | `state:magic_immune` |

### B32 `uesugi_kenshin`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_divine_slash` | ✓ 已產生 |
| `b32_divine_flash` | ✓ 已產生 |
| `b32_divine_formation` | `action:damage.flags:reflection`、`status.scope` |
| `b32_war_god` | `passive:visibility`、`passive_trigger:on_kill`、`stat:outgoing_damage_pct`、`state:provides_vision` |
| `b32_bishamonten` | ✓ 已產生 |

### B33 `mogami_yoshihime`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_corrosive_arrow` | ✓ 已產生 |
| `b33_spider_web` | ✓ 已產生 |
| `b33_voodoo_power` | ✓ 已產生 |
| `b33_spider_attack` | `action:create_unit.control:none`、`action:create_unit.facing`、`action:face`、`action:kill`、`fn:edge_distance`、`stat:heal_reduction_pct`、`state:flying`、`status.stacking:replace`、`token:vars` |

### B34 `takeda_katsuyori`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b34_wrath_axe` | ✓ 已產生 |
| `b34_spike_wall` | `action:create_unit.control:none`、`action:kill`、`fn:rotate`、`status.aura` |
| `b34_frenzy` | `action:modify_attack` |
| `b34_tenmoku_howl` | `status.stacking:independent with a source`、`status.stacking:independent with slow_pct` |

### B35 `kamiizumi_nobutsuna`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b35_sword_essence` | ✓ 已產生 |
| `b35_isshun` | `action:face` |
| `b35_muto_dori` | `action:attack`、`attr:damage`、`event:attack`、`fn:armor_multiplier`、`fn:has_status_tag`、`fn:random_int` |
| `b35_hiten` | `action:modify_attack` |

### B36 `naoe_kanetsugu`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_naoe_seal` | `status.scope`、`status.stacking:replace` |
| `b36_footsteps` | `fn:status_var` |
| `b36_heart_fence` | `selector.flag:exclude_illusions`、`selector.team:ally_not_self`、`status.absorb` |
| `b36_seven_spears` | `action:knockback` |
| `b36_aizen_myoo` | ✓ 已產生 |

### C01 `akechi_mitsuhide`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c01_blood_blade` | ✓ 已產生 |
| `c01_ground_flame` | `routine c01e_wave: status.stacking:independent with states` |
| `c01_blood_flame` | `action:modify_attack` |
| `c01_honnoji` | `action:cue.at of a sym:landing`、`action:random_pick`、`expr: a position from a sym:landing`、`status.stacking:independent with states`、`status.status_resistance` |

### C02 `akechi_hidemitsu`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c02_mirage_strike` | `action:create_illusion`、`action:face`、`expr: wanted a unit, got a sym:ill` |
| `c02_phantom_move` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill` |
| `c02_kyoka_suigetsu` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill` |
| `c02_culling_slash` | ✓ 已產生 |

### C03 `shimazu_yoshihiro`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_iron_shirt` | `state:magic_immune` |
| `c03_quake_axe` | ✓ 已產生 |
| `c03_thunder_wrath` | `action:modify_attack` |
| `c03_iron_hammer` | ✓ 已產生 |

### C04 `izumo_no_okuni`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_swift_dance` | `expr of type nil`、`selector.shape:global` |
| `c04_withering_step` | `action:spend_mana`、`action:toggle_ability` |
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
| `c07_thunder_shackle` | `event:on_channel_end`、`name:event outside an event`、`status.stacking:replace`、`status.status_resistance` |
| `c07_thunderbolt` | `selector.relative_to` |
| `c07_raijin_wrath` | `action:create_unit.control:none`、`fn:has_tag:dummy of a unit`、`fn:is_visible_to`、`stat:heal_reduction_pct`、`status.status_resistance` |
| `c07_storm_move` | `action:issue_order.force` |
| `c07_magic_shield` | `action:toggle_ability`、`stat:move_speed`、`state:magic_immune` |

### C08 `kato_danzo`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_vanishing_step` | `state:low_attack_priority`、`status_trigger:on_attack_launch` |
| `c08_blood_hunt` | `action:damage.flags:no_source_bonuses`、`stat:move_speed`、`status.stacking:independent with triggers` |
| `c08_scatter_escape` | ✓ 已產生 |
| `c08_kite_chain` | `action:face`、`expr of type nil`、`fn:has_tag:boss of a unit`、`selector.relative_to`、`selector.shape:segment`、`selector.team:ally_not_self`、`selector.to`、`selector.width` |
| `c08_devour` | `action:add_ability`、`action:remove_ability`、`action:set_ability_active (an ability id and a fixed active)`、`attr:level`、`fn:ability_in_slot`、`fn:is_ability of a non-event ability`、`fn:random_int`、`fn:status_var`、`stat:vision_day`、`stat:vision_night`、`state:blind`、`state:magic_immune`、`status.status_resistance` |

### C09 `tachibana_ginchiyo`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c09_frost_thorn` | `action:modify_attack`、`action:spend_mana`、`action:toggle_ability` |
| `c09_white_heat` | ✓ 已產生 |
| `c09_ice_soul_dance` | `attr:remaining` |
| `c09_armor_break` | `status.status_resistance` |

### C10 `kosokabe_chikayasu`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c10_black_dragon_wave` | `target.type:unit_or_point` |
| `c10_energy_drain` | ✓ 已產生 |
| `c10_echo_sword` | `attr:damage`、`event:attack`、`passive:visibility` |
| `c10_day_night_shift` | `action:set_time_of_day` |
| `c10_formless_heaven` | `expr of type nil`、`selector.flag:include_hidden`、`selector.shape:global`、`stat.when`、`stat:vision_night` |

### C11 `tachibana_muneshige`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c11_raikiri` | `expr of type nil`、`selector.flag:include_hidden`、`selector.shape:global` |
| `c11_thunder_speed` | ✓ 已產生 |
| `c11_thunder_eye` | ✓ 已產生 |
| `c11_unrivaled` | `attr:mana_pct`、`expr of type nil`、`passive:visibility`、`passive_trigger:on_kill`、`selector.flag:include_hidden`、`selector.shape:global` |

### C12 `otomo_sorin`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c12_homing_lock` | `state:provides_vision`、`status.stacking:replace`、`status_trigger:on_moved` |
| `c12_missile_launch` | `fn:status_var` |
| `c12_space_rift` | `passive:visibility`、`status.absorb` |
| `c12_optical_mask` | ✓ 已產生 |
| `c12_annihilation_beam` | `fn:rotate` |

### C13 `ryuzoji_takanobu`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c13_siphon` | ✓ 已產生 |
| `c13_tornado_wave` | ✓ 已產生 |
| `c13_gale_grip` | `action:issue_order.force`、`event.damage.is_attack`、`passive:visibility`、`passive_trigger:on_kill`、`status_trigger:on_kill` |
| `c13_tornado_vortex` | `action:modify_cooldown.item_tag`、`fn:rotate` |

### C14 `saito_yoshitatsu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c14_fire_curse` | ✓ 已產生 |
| `c14_fire_breath` | `status_trigger:periodic.filter` |
| `c14_fire_breath_dragon` | `routine c14e_breath: action:projectile in a routine` |
| `c14_dragon_vein` | ✓ 已產生 |
| `c14_dragon_form` | `attr:damage`、`event:attack` |

### C16 `kashin_koji`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_soul_sever` | ✓ 已產生 |
| `c16_curse_shadow` | `action:create_unit.control:owner`、`stat:vision_day`、`stat:vision_night`、`state:magic_immune` |
| `c16_wandering_souls` | `action:cue.at of a err`、`action:heal.source:owner_of(self)`、`attr:game.dt`、`fn:owner_of`、`state:flying`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.stacking:replace` |
| `c16_phantom_sight` | `action:create_illusion`、`action:face` |
| `c16_eight_gates` | `action:create_unit.control:owner`、`state:magic_immune` |

### C18 `kobayakawa_takakage`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c18_lone_rider` | `action:modify_cooldown.item`、`action:pick_item`、`expr of type nil`、`fn:exists of a sym:blink_item`、`fn:status_source`、`selector.flag:include_dead`、`selector.shape:global` |
| `c18_twin_chariot` | `fn:ability_ready`、`status.scope` |
| `c18_three_blades` | ✓ 已產生 |
| `c18_chain_stratagem` | `attr:stacks`、`passive:visibility` |
| `c18_four_cannons` | `expr of type nil`、`selector.flag:include_illusions`、`selector.shape:global` |

### C19 `matsuhime`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c19_alluring_gaze` | ✓ 已產生 |
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
| `c21_chain_breaker` | `action:attack`、`action:face`、`fn:rotate`、`stat:outgoing_damage_pct` |
| `c21_two_heavens` | `action:modify_attack`、`status.stacking:replace` |
| `c21_soaring_quake` | `action:attack`、`action:face`、`fn:count_in_radius`、`fn:rotate`、`selector.order:random` |

### C22 `sasaki_kojiro`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c22_ganryu` | `action:modify_attack`、`fn:has_status of an independent status (line B)` |
| `c22_swallow_nest` | `state:no_healthbar`、`state:unselectable` |
| `c22_flying_swallow` | `status.stacking:independent with states` |
| `c22_swallow_flash` | ✓ 已產生 |
| `c22_dream_mist` | ✓ 已產生 |

### C23 `kato_kiyomasa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c23_tainted_frenzy` | ✓ 已產生 |
| `c23_malice_wall` | `action:create_unit.facing`、`action:face`、`state:no_healthbar`、`state:not_on_minimap`、`state:unselectable`、`status.aura` |
| `c23_blood_for_blood` | ✓ 已產生 |
| `c23_gift_of_death` | `action:kill`、`event:on_phase_start` |

### C24 `yagyu_muneyoshi`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c24_kamitori` | `stat:move_speed` |
| `c24_ukifune` | `action:knockback` |
| `c24_zenken` | `attr:damage`、`event:attack` |
| `c24_last_life` | ✓ 已產生 |

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
| 施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR） | 148 |
| 召喚：附近放不下、lifetime 不在 0～600 秒 | 1 |
| 召喚：count 不是 1 | 1 |
| 視野：半徑或位置不合法 | 7 |
| 光鍊：host tether 規格不合法 | 1 |
| 友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離 | 1 |
| 運算式除以零 | 8 |
| 讀位置的單位已不在世界中 | 68 |
| 需要地點，但施放沒有地點也沒有單位目標 | 37 |

### 已產生但 host 沒有做到的地方

產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。

| 招式 | 沒有做到 |
|---|---|
| `c06_grand_heist` | create_pickup.cue: the host pickup draws its own bag |
| `b30_flash_strike` | issue_order: only a hero takes the order |
| `b14_iai_slash` | issue_order: only a hero takes the order |
| `b27_mighty_strike` | issue_order: only a hero takes the order |
| `c14_fire_curse` | issue_order: only a hero takes the order |
| `a35_scorching_fire` | issue_order: only a hero takes the order |
| `b32_divine_slash` | issue_order: only a hero takes the order |
| `item_flash_ring` | item cast_point: the host uses items without a windup |
| `item_kisyosennfuyuzuki` | item cast_point: the host uses items without a windup |
| `item_the_red_lightning_chapter` | item cast_point: the host uses items without a windup |
| `item_the_sword_of_kaminaobi` | issue_order: only a hero takes the order |
| `item_yubashiri` | item cast_point: the host uses items without a windup |

## 4. 每項寫法擋住多少

| 寫法 | 主要負責 | 分類 | 招式 | 道具 | 英雄 |
|---|---|---|---|---|---|
| `state:magic_immune` | ability_kit | 共用 | 20 | 15 | 19 |
| `passive:visibility` | ability_kit | 共用 | 20 | 0 | 16 |
| `action:modify_attack` | ability_kit | 共用 | 15 | 6 | 15 |
| `status.status_resistance` | ability_kit | 共用 | 16 | 0 | 14 |
| `action:create_unit.control:owner` | ability_kit | 共用 | 15 | 6 | 13 |
| `status.aura` | ability_kit | 共用 | 15 | 1 | 13 |
| `action:face` | ability_kit | 共用 | 12 | 1 | 11 |
| `event:on_channel_end` | ability_kit | 共用 | 12 | 0 | 11 |
| `action:create_unit.overrides` | ability_kit | 共用 | 11 | 4 | 11 |
| `attr:damage` | 產生器 | 共用 | 12 | 2 | 10 |
| `event:attack` | ability_kit | 共用 | 12 | 9 | 10 |
| `action:kill` | ability_kit | 共用 | 11 | 2 | 10 |
| `passive:aura` | ability_kit | 共用 | 11 | 22 | 10 |
| `action:create_unit.control:none` | ability_kit | 共用 | 11 | 4 | 9 |
| `action:knockback` | ability_kit | 共用 | 10 | 3 | 9 |
| `event:on_phase_start` | ability_kit | 共用 | 9 | 2 | 9 |
| `status.stacking:replace` | ability_kit | 共用 | 9 | 0 | 9 |
| `expr of type nil` | 產生器 | 共用 | 15 | 0 | 8 |
| `fn:rotate` | 產生器 | 共用 | 9 | 3 | 8 |
| `status_trigger:on_attack_launch` | ability_kit | 共用 | 9 | 1 | 8 |
| `passive_trigger:on_kill` | ability_kit | 共用 | 8 | 5 | 8 |
| `state:no_healthbar` | ability_kit | 共用 | 8 | 0 | 8 |
| `selector.shape:global` | 產生器 | 共用 | 13 | 0 | 7 |
| `action:create_illusion` | ability_kit | 共用 | 10 | 0 | 7 |
| `action:toggle_ability` | ability_kit | 共用 | 7 | 0 | 7 |
| `state:low_attack_priority` | ability_kit | 共用 | 7 | 0 | 7 |
| `status.stacking:independent with triggers` | ability_kit | 共用 | 7 | 0 | 7 |
| `action:set_display_stacks` | ability_kit | 共用 | 8 | 2 | 6 |
| `stat:move_speed` | ability_kit | 共用 | 7 | 1 | 6 |
| `status.absorb` | ability_kit | 共用 | 7 | 2 | 6 |
| `action:spend_mana` | ability_kit | 共用 | 6 | 2 | 6 |
| `selector.order:random` | 產生器 | 共用 | 6 | 1 | 6 |
| `stat:vision_night` | ability_kit | 共用 | 6 | 3 | 6 |
| `status.scope` | ability_kit | 共用 | 6 | 3 | 6 |
| `action:damage.flags:reflection` | ability_kit | 共用 | 5 | 1 | 5 |
| `action:issue_order.force` | ability_kit | 共用 | 5 | 0 | 5 |
| `fn:armor_multiplier` | 產生器 | 共用 | 5 | 2 | 5 |
| `fn:has_status_tag` | 產生器 | 共用 | 5 | 13 | 5 |
| `state:unselectable` | ability_kit | 共用 | 5 | 0 | 5 |
| `action:attack` | ability_kit | 共用 | 5 | 2 | 4 |
| `action:reveal.ignore_terrain` | ability_kit | 共用 | 5 | 2 | 4 |
| `requires.condition` | ability_kit | 共用 | 5 | 0 | 4 |
| `action:create_unit.facing` | ability_kit | 共用 | 4 | 0 | 4 |
| `action:cue.at of a err` | ability_kit | 共用 | 4 | 0 | 4 |
| `fn:random_int` | 產生器 | 共用 | 4 | 2 | 4 |
| `fn:status_var` | 產生器 | 共用 | 4 | 0 | 4 |
| `name:event outside an event` | 產生器 | 共用 | 4 | 2 | 4 |
| `passive_trigger:on_attack_launch` | ability_kit | 共用 | 4 | 0 | 4 |
| `stat:outgoing_damage_pct` | ability_kit | 共用 | 4 | 2 | 4 |
| `stat:vision_day` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:not_on_minimap` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:provides_vision` | ability_kit | 共用 | 4 | 1 | 4 |
| `state:revealed` | ability_kit | 共用 | 4 | 6 | 4 |
| `fn:is_template` | 產生器 | 共用 | 8 | 0 | 3 |
| `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | 6 | 0 | 3 |
| `action:remove_unit` | ability_kit | 共用 | 4 | 0 | 3 |
| `action:swap_positions` | ability_kit | 共用 | 4 | 0 | 3 |
| `fn:edge_distance` | 產生器 | 共用 | 4 | 0 | 3 |
| `status.stacking:independent with states` | ability_kit | 共用 | 4 | 1 | 3 |
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
| `state:flying` | ability_kit | 共用 | 3 | 0 | 3 |
| `status_trigger:on_moved` | ability_kit | 共用 | 3 | 0 | 3 |
| `status_trigger:periodic.filter` | ability_kit | 共用 | 3 | 0 | 3 |
| `target.type:unit_or_point` | 產生器 | 共用 | 3 | 1 | 3 |
| `selector.flag:include_illusions` | 產生器 | 共用 | 5 | 1 | 2 |
| `action:give_item` | ability_kit | 共用 | 3 | 0 | 2 |
| `action:leap` | ability_kit | 共用 | 3 | 0 | 2 |
| `event:on_channel_tick` | ability_kit | 共用 | 3 | 0 | 2 |
| `selector.flag:include_hidden` | 產生器 | 共用 | 3 | 0 | 2 |
| `action:add_ability` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | 2 | 2 | 2 |
| `action:end_channel` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:heal.source:owner_of(self)` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:projectile in a routine` | ability_kit | 共用 | 2 | 0 | 2 |
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
| `fn:has_status of an independent status (line B)` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:has_tag:boss of a unit` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:has_tag:dummy of a unit` | 產生器 | 共用 | 2 | 0 | 2 |
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
| `status.stacking:independent with a source` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.stacking:independent with slow_pct` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.unique_group` | ability_kit | 共用 | 2 | 1 | 2 |
| `status.vars of an independent status` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:cancel_delay` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:damage.source with flags or credit` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:delay.id` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:remove_item` | ability_kit | 共用 | 2 | 0 | 1 |
| `selector.types:building` | 產生器 | 共用 | 2 | 0 | 1 |
| `target.type:tree` | 產生器 | 共用 | 2 | 0 | 1 |
| `action:chain` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:caster_hero` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:landing` | ability_kit | hook 候選 | 1 | 0 | 1 |
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
| `action:redirect_motion` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:regrow_trees` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_ability` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_status of an independent status (line B: instance removal)` | ability_kit | 共用 | 1 | 3 | 1 |
| `action:set_ability_active (an ability id and a fixed active)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:set_base_max_hp` | ability_kit | hook 候選 | 1 | 0 | 1 |
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
| `expr: ~= of num and sym:me_slot` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:ability_ready` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:alive of a sym:caster_hero` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:count_in_radius` | 產生器 | 共用 | 1 | 1 | 1 |
| `fn:enemy_team` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:il` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:item_a` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:item_b` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:stolen` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:has_state:revealed` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:has_tag:fountain of a unit` | 產生器 | 共用 | 1 | 1 | 1 |
| `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:landmark` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:min of non-numbers` | 產生器 | 共用 | 1 | 2 | 1 |
| `fn:offset types` | 產生器 | 共用 | 1 | 1 | 1 |
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
| `selector.to` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:building+creep+neutral+summon` | 產生器 | 共用 | 1 | 1 | 1 |
| `selector.width` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `stat.when` | ability_kit | 共用 | 1 | 4 | 1 |
| `stat:physical_block_chance` | ability_kit | 共用 | 1 | 2 | 1 |
| `state type:item_name` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state type:vec` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:blind` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.immune_to` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.reapply_if_removed` | ability_kit | 共用 | 1 | 14 | 1 |
| `status_trigger:on_attack_miss` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_kill` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_mana_spent` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_refreshed` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `token:state` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `token:vars` | 產生器 | 共用 | 1 | 2 | 1 |
| `action:damage.type:event` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:issue_order.ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:issue_order.order:cast` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:projectile.from:offset(base, rotate(fwd, $perp_angle), random_int(-$storm_width, $storm_width))` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:projectile.lifetime` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:remove_status.exclude_source_tags` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.min_duration` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.source_is` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `attr:effect_multiplier` | 產生器 | 共用 | 0 | 7 | 0 |
| `attr:illusion_damage` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `attr:is_summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `event:on_owner_death` | ability_kit | 共用 | 0 | 3 | 0 |
| `event:on_unequip` | ability_kit | 共用 | 0 | 2 | 0 |
| `fn:angle_between` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:has_item` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:has_tag:neutral of a unit` | 產生器 | 共用 | 0 | 3 | 0 |
| `fn:has_tag:no_cast_trigger` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:is_hero_id` | 產生器 | 共用 | 0 | 7 | 0 |
| `item.active.cast.interruptible` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.cost.charges` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.kind:passive` | ability_kit | 共用 | 0 | 3 | 0 |
| `item.active.kind:toggle` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_death` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_removed` | ability_kit | 共用 | 0 | 25 | 0 |
| `rust_handler without behavior` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `selector.exclude_types` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:building+creep+hero+neutral+summon+tower` | 產生器 | 共用 | 0 | 5 | 0 |
| `selector.types:building+hero+tower` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:creep+neutral+summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
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
| `state type:vec` | `a04_ice_path` | A04 takenaka_shigeharu |
| `action:issue_order.order:follow` | `a04_water_shikigami` | A04 takenaka_shigeharu |
| `status_trigger:on_attack_miss` | `a06_red_fury` | A06 ii_naomasa |
| `action:delay in a routine` | `a07_red_thunder` | A07 honda_tadakatsu |
| `attr:cast_point of an ability` | `a11_old_fox` | A11 saito_dosan |
| `action:cue.at of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `action:dodge_projectiles.of:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: * of num and sym:me_slot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: + of num and sym:rot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: wanted a unit, got a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: ~= of num and sym:me_slot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `fn:exists of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
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
| `action:redirect_motion` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:boss+creep+neutral+summon` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `action:projectile.team:any` | `b24_stone_resonance` | B24 akiyama_nobutomo |
| `action:projectile.where` | `b24_stone_resonance` | B24 akiyama_nobutomo |
| `action:impulse` | `b25_asura_hell` | B25 honganji_kennyo |
| `attr:cooldown_remaining` | `b25_exorcism_sutra` | B25 honganji_kennyo |
| `event.damage.lethal` | `b26_kozuke_valor` | B26 okuma_tomohide |
| `action:chain` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `fn:has_state:revealed` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `action:projectile.from:spawn` | `b31_still_forest` | B31 takeda_shingen |
| `selector.flag:exclude_illusions` | `b36_heart_fence` | B36 naoe_kanetsugu |
| `action:cue.at of a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `expr: a position from a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `attr:name` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `fn:exists of a sym:stolen` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `state type:item_name` | `item_c06e` | C06 ishikawa_goemon |
| `action:remove_ability` | `c08_devour` | C08 kato_danzo |
| `action:set_ability_active (an ability id and a fixed active)` | `c08_devour` | C08 kato_danzo |
| `attr:level` | `c08_devour` | C08 kato_danzo |
| `fn:is_ability of a non-event ability` | `c08_devour` | C08 kato_danzo |
| `state:blind` | `c08_devour` | C08 kato_danzo |
| `selector.shape:segment` | `c08_kite_chain` | C08 kato_danzo |
| `selector.to` | `c08_kite_chain` | C08 kato_danzo |
| `selector.width` | `c08_kite_chain` | C08 kato_danzo |
| `attr:remaining` | `c09_ice_soul_dance` | C09 tachibana_ginchiyo |
| `status_trigger:on_kill` | `c13_gale_grip` | C13 ryuzoji_takanobu |
| `action:modify_cooldown.item_tag` | `c13_tornado_vortex` | C13 ryuzoji_takanobu |
| `action:issue_order.queue` | `c15_hundred_arrows` | C15 tamako |
| `action:modify_cooldown.item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:exists of a sym:blink_item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:status_source` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:ability_ready` | `c18_twin_chariot` | C18 kobayakawa_takakage |
| `attr:is_summon` | `item_commander_of_fan` | 道具 |
| `selector.types:building+hero+tower` | `item_contemporary_armor` | 道具 |
| `item.active.cost.charges` | `item_earth_wall_book` | 道具 |
| `status.unique_rule` | `item_foxfire_armor` | 道具 |
| `rust_handler without behavior` | `item_hama_orb` | 道具 |
| `fn:angle_between` | `item_hannya_onimaru` | 道具 |
| `action:remove_status.source_is` | `item_ice_storm_book` | 道具 |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `selector.exclude_types` | `item_napalm_bomb` | 道具 |
| `action:projectile.lifetime` | `item_ronin_jitte` | 道具 |
| `action:damage.type:event` | `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 |
| `selector.types:creep+neutral+summon` | `item_the_great_sword_of_disease` | 道具 |
| `item.active.cast.interruptible` | `item_the_great_sword_of_teleport` | 道具 |
| `target.types:building+courier+creep+neutral+summon` | `item_the_great_sword_of_teleport` | 道具 |

共 82 種寫法、57 招。

## 重跑

```
tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua
```

