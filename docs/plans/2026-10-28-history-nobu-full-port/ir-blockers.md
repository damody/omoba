# 直譯器尚未支援的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_ir_blockers.lua` 產生，不要手改。每次工作線 B 放回一批，就照 §4 重跑一次。

- 全量內容中帶 IR 的條目：604 個。依 ir.rs 的載入規則掃描，有 534 個會被擋下。
- 直譯器實際載入（`tools/ir_probe`）：70 個成功、534 個失敗。掃描與實際結果不一致：0 個（§3）。
- 一位英雄要「全部技能都能載入」才算解鎖。順序依序是 M5（其餘 8 位已移植英雄），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每一批內，先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。

## 1. 實作順序

### M5（已上架、仍由 Rust handler 執行的英雄）（8 位）

開始時已能全部載入：1 位；做完本批的項目後：8 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 1 | `action:dash` | B04 | 1 招、1 件道具 | 1 / 8 |
| 2 | `action:dodge_projectiles` | B04 | 21 招、2 件道具 | 1 / 8 |
| 3 | `action:projectile` | B04 | 52 招、9 件道具 | 1 / 8 |
| 4 | `state:ranged_attack` | B04 | 4 招、0 件道具 | 1 / 8 |
| 5 | `status.aura` | B04 | 8 招、1 件道具 | 1 / 8 |
| 6 | `status.triggers` | B04 | 116 招、16 件道具 | 1 / 8 |
| 7 | `target.flag:pierce_magic_immune` | B04 | 62 招、1 件道具 | 2 / 8 |
| 8 | `action:set_state` | B06 | 45 招、7 件道具 | 2 / 8 |
| 9 | `ally_selector.effect:apply_status` | B06 | 6 招、11 件道具 | 2 / 8 |
| 10 | `ally_selector.types` | B06 | 9 招、13 件道具 | 2 / 8 |
| 11 | `passive_trigger:on_order_targeted` | B06 | 1 招、0 件道具 | 3 / 8 |
| 12 | `action:delay` | B29 | 51 招、3 件道具 | 3 / 8 |
| 13 | `action:leap` | B29 | 3 招、0 件道具 | 3 / 8 |
| 14 | `action:repeat` | B29 | 64 招、20 件道具 | 3 / 8 |
| 15 | `action:reveal.ignore_terrain` | B29 | 3 招、1 件道具 | 3 / 8 |
| 16 | `passive:aura` | B29 | 11 招、22 件道具 | 4 / 8 |
| 17 | `action:apply_status.source` | C15 | 14 招、1 件道具 | 4 / 8 |
| 18 | `action:create_unit.facing` | C15 | 3 招、0 件道具 | 4 / 8 |
| 19 | `action:create_unit.owner` | C15 | 15 招、8 件道具 | 4 / 8 |
| 20 | `action:issue_order` | C15 | 17 招、2 件道具 | 4 / 8 |
| 21 | `state:invulnerable` | C15 | 13 招、1 件道具 | 4 / 8 |
| 22 | `status.status_resistance` | C15 | 9 招、0 件道具 | 4 / 8 |
| 23 | `summon template not registered` | C15 | 21 招、8 件道具 | 4 / 8 |
| 24 | `token:state` | C15 | 30 招、7 件道具 | 5 / 8 |
| 25 | `action:remove_status.states` | A07 | 1 招、0 件道具 | 5 / 8 |
| 26 | `action:remove_status.team_of_source` | A07 | 12 招、16 件道具 | 5 / 8 |
| 27 | `action:set_health` | A07 | 10 招、1 件道具 | 5 / 8 |
| 28 | `event:on_upgrade` | A07 | 28 招、0 件道具 | 5 / 8 |
| 29 | `fn:ability_level` | A07 | 16 招、0 件道具 | 5 / 8 |
| 30 | `passive_trigger:on_attack_windup` | A07 | 12 招、3 件道具 | 5 / 8 |
| 31 | `stat:slow_pct` | A07 | 23 招、9 件道具 | 5 / 8 |
| 32 | `stat:slow_resistance_pct` | A07 | 1 招、0 件道具 | 5 / 8 |
| 33 | `stat:status_resistance_pct` | A07 | 2 招、1 件道具 | 5 / 8 |
| 34 | `status.stacking:independent` | A07 | 13 招、2 件道具 | 6 / 8 |
| 35 | `action:add_stacks` | A34 | 10 招、2 件道具 | 6 / 8 |
| 36 | `action:apply_status.stacks` | A34 | 8 招、4 件道具 | 6 / 8 |
| 37 | `action:modify_cooldown` | A34 | 9 招、2 件道具 | 6 / 8 |
| 38 | `attr:cooldown` | A34 | 1 招、0 件道具 | 6 / 8 |
| 39 | `attr:cooldown_remaining` | A34 | 1 招、0 件道具 | 6 / 8 |
| 40 | `fn:ability_of` | A34 | 1 招、0 件道具 | 6 / 8 |
| 41 | `fn:has_status` | A34 | 36 招、7 件道具 | 6 / 8 |
| 42 | `fn:is_ability` | A34 | 6 招、0 件道具 | 6 / 8 |
| 43 | `fn:stacks` | A34 | 8 招、0 件道具 | 7 / 8 |
| 44 | `action:burn_mana` | B08 | 6 招、1 件道具 | 7 / 8 |
| 45 | `action:teleport` | B08 | 31 招、5 件道具 | 7 / 8 |
| 46 | `attr:is_magic_immune` | B08 | 17 招、4 件道具 | 7 / 8 |
| 47 | `attr:mana` | B08 | 5 招、1 件道具 | 7 / 8 |
| 48 | `state:command_restricted` | B08 | 7 招、1 件道具 | 7 / 8 |
| 49 | `status.stacking:stacks` | B08 | 8 招、4 件道具 | 7 / 8 |
| 50 | `target.flag:visible_only` | B08 | 5 招、0 件道具 | 8 / 8 |

### M6 第 1 批（9 位）

開始時已能全部載入：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 51 | `action:remove_status.dispel` | C03 | 11 招、16 件道具 | 0 / 9 |
| 52 | `attr:forward` | C03 | 10 招、2 件道具 | 0 / 9 |
| 53 | `state:magic_immune` | C03 | 16 招、13 件道具 | 2 / 9 |
| 54 | `action:set_ability_level` | B26 | 14 招、0 件道具 | 2 / 9 |
| 55 | `passive_trigger:on_take_damage` | B26 | 10 招、3 件道具 | 3 / 9 |
| 56 | `passive_trigger:periodic` | A01 | 13 招、11 件道具 | 3 / 9 |
| 57 | `selector.flag:visible_only` | A01 | 14 招、7 件道具 | 4 / 9 |
| 58 | `action:knockback` | A26 | 8 招、2 件道具 | 4 / 9 |
| 59 | `event:on_phase_start` | A26 | 10 招、2 件道具 | 5 / 9 |
| 60 | `selector.limit` | C22 | 12 招、5 件道具 | 5 / 9 |
| 61 | `state:no_healthbar` | C22 | 2 招、0 件道具 | 5 / 9 |
| 62 | `state:unselectable` | C22 | 3 招、0 件道具 | 6 / 9 |
| 63 | `action:set_display_stacks` | A12 | 12 招、3 件道具 | 6 / 9 |
| 64 | `attr:is_item` | A12 | 3 招、0 件道具 | 6 / 9 |
| 65 | `attr:is_toggle` | A12 | 5 招、1 件道具 | 6 / 9 |
| 66 | `status.stacking:keep` | A12 | 7 招、1 件道具 | 7 / 9 |
| 67 | `fn:exists` | A14 | 13 招、3 件道具 | 7 / 9 |
| 68 | `stat:move_speed` | A14 | 5 招、1 件道具 | 8 / 9 |
| 69 | `action:damage.source` | B20 | 17 招、1 件道具 | 8 / 9 |
| 70 | `action:heal.source` | B20 | 3 招、0 件道具 | 8 / 9 |
| 71 | `action:set_mana` | B20 | 3 招、0 件道具 | 8 / 9 |
| 72 | `attr:hp_pct` | B20 | 10 招、0 件道具 | 8 / 9 |
| 73 | `passive:visibility` | B20 | 20 招、0 件道具 | 9 / 9 |

### M6 第 2 批（9 位）

開始時已能全部載入：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 74 | `selector.types:building+creep+hero+neutral+summon` | A18 | 4 招、3 件道具 | 3 / 9 |
| 75 | `reveal.teams:own` | C01 | 11 招、3 件道具 | 4 / 9 |
| 76 | `attr:is_creep` | C05 | 2 招、0 件道具 | 4 / 9 |
| 77 | `event:on_channel_end` | C05 | 12 招、0 件道具 | 5 / 9 |
| 78 | `state:break` | A23 | 2 招、0 件道具 | 5 / 9 |
| 79 | `state:mute` | A23 | 5 招、2 件道具 | 6 / 9 |
| 80 | `action:restore_mana` | B30 | 2 招、2 件道具 | 6 / 9 |
| 81 | `attr:agi` | B30 | 2 招、0 件道具 | 6 / 9 |
| 82 | `status.vars` | B30 | 17 招、1 件道具 | 7 / 9 |
| 83 | `action:face` | A24 | 9 招、1 件道具 | 7 / 9 |
| 84 | `passive_trigger:on_created` | A24 | 2 招、44 件道具 | 7 / 9 |
| 85 | `target.flag:include_magic_immune` | A24 | 10 招、8 件道具 | 8 / 9 |
| 86 | `attr:cast_range` | A35 | 3 招、0 件道具 | 8 / 9 |
| 87 | `attr:position` | A35 | 9 招、0 件道具 | 8 / 9 |
| 88 | `fn:floor` | A35 | 5 招、0 件道具 | 8 / 9 |
| 89 | `name:ability` | A35 | 4 招、0 件道具 | 9 / 9 |

### M6 第 3 批（9 位）

開始時已能全部載入：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 90 | `status.reapply_if_removed` | A22 | 1 招、13 件道具 | 2 / 9 |
| 91 | `action:swap_abilities` | C14 | 3 招、0 件道具 | 2 / 9 |
| 92 | `state:attack_slow_immune` | C14 | 1 招、0 件道具 | 3 / 9 |
| 93 | `attr:int` | A19 | 4 招、0 件道具 | 3 / 9 |
| 94 | `attr:point` | A19 | 2 招、0 件道具 | 4 / 9 |
| 95 | `action:zone` | A30 | 2 招、0 件道具 | 4 / 9 |
| 96 | `state:provides_vision` | A30 | 4 招、1 件道具 | 5 / 9 |
| 97 | `action:create_illusion` | B02 | 7 招、0 件道具 | 5 / 9 |
| 98 | `state:low_attack_priority` | B02 | 6 招、0 件道具 | 6 / 9 |
| 99 | `action:apply_status.vars` | A11 | 8 招、0 件道具 | 6 / 9 |
| 100 | `status.forced_move` | A11 | 2 招、0 件道具 | 6 / 9 |
| 101 | `token:var` | A11 | 1 招、0 件道具 | 7 / 9 |
| 102 | `action:create_unit.team` | B34 | 13 招、0 件道具 | 7 / 9 |
| 103 | `action:set_var` | B34 | 2 招、0 件道具 | 7 / 9 |
| 104 | `fn:rotate` | B34 | 4 招、1 件道具 | 8 / 9 |
| 105 | `action:chance` | A36 | 7 招、7 件道具 | 8 / 9 |
| 106 | `action:create_unit.overrides` | A36 | 10 招、3 件道具 | 8 / 9 |
| 107 | `action:kill` | A36 | 3 招、0 件道具 | 9 / 9 |

### M6 第 4 批（9 位）

開始時已能全部載入：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 108 | `state:revealed` | B05 | 2 招、6 件道具 | 2 / 9 |
| 109 | `fn:vec` | B18 | 2 招、0 件道具 | 3 / 9 |
| 110 | `action:refund` | A33 | 2 招、2 件道具 | 3 / 9 |
| 111 | `fn:has_state` | A33 | 5 招、0 件道具 | 4 / 9 |
| 112 | `action:remove_status.tags` | C02 | 2 招、0 件道具 | 4 / 9 |
| 113 | `attr:is_illusion` | C02 | 3 招、2 件道具 | 5 / 9 |
| 114 | `attr:str` | B03 | 2 招、0 件道具 | 5 / 9 |
| 115 | `passive_trigger:on_kill` | B03 | 8 招、5 件道具 | 6 / 9 |
| 116 | `action:set_ability_active` | B11 | 9 招、0 件道具 | 7 / 9 |
| 117 | `action:end_channel` | B09 | 1 招、0 件道具 | 7 / 9 |
| 118 | `ally_selector.effect:set_mana` | B09 | 2 招、0 件道具 | 7 / 9 |
| 119 | `event:on_channel_tick` | B09 | 3 招、0 件道具 | 8 / 9 |
| 120 | `action:cancel_delay` | A20 | 2 招、0 件道具 | 8 / 9 |
| 121 | `passive_trigger:on_attacked_hit` | A20 | 3 招、7 件道具 | 8 / 9 |
| 122 | `state:not_on_minimap` | A20 | 1 招、0 件道具 | 8 / 9 |
| 123 | `status.unique_group` | A20 | 1 招、0 件道具 | 9 / 9 |

### M6 第 5 批（13 位）

開始時已能全部載入：3 位；做完本批的項目後：13 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 124 | `selector.shape:global` | B19 | 12 招、0 件道具 | 4 / 13 |
| 125 | `state:melee_attack` | A03 | 1 招、0 件道具 | 4 / 13 |
| 126 | `status.scope` | A03 | 4 招、1 件道具 | 5 / 13 |
| 127 | `fn:has_status_tag` | B25 | 2 招、6 件道具 | 5 / 13 |
| 128 | `selector.order` | B25 | 6 招、5 件道具 | 6 / 13 |
| 129 | `passive_trigger:on_death` | B21 | 1 招、2 件道具 | 6 / 13 |
| 130 | `passive_trigger:on_heal_received` | B21 | 1 招、0 件道具 | 7 / 13 |
| 131 | `fn:status_var` | C12 | 2 招、0 件道具 | 7 / 13 |
| 132 | `op:%` | C12 | 1 招、0 件道具 | 7 / 13 |
| 133 | `status.stacking:replace` | C12 | 8 招、0 件道具 | 8 / 13 |
| 134 | `selector.flag:exclude_illusions` | B36 | 1 招、0 件道具 | 8 / 13 |
| 135 | `selector.team:ally_not_self` | B36 | 1 招、0 件道具 | 9 / 13 |
| 136 | `passive_trigger:on_attack_launch` | A17 | 4 招、0 件道具 | 9 / 13 |
| 137 | `stat:vision_day` | A17 | 3 招、0 件道具 | 9 / 13 |
| 138 | `stat:vision_night` | A17 | 5 招、3 件道具 | 10 / 13 |
| 139 | `ally_selector.effect:remove_unit` | B10 | 3 招、0 件道具 | 10 / 13 |
| 140 | `fn:is_template` | B10 | 8 招、0 件道具 | 10 / 13 |
| 141 | `fn:owner_of` | B10 | 3 招、0 件道具 | 10 / 13 |
| 142 | `selector.flag:include_hidden` | B10 | 5 招、0 件道具 | 11 / 13 |
| 143 | `status.per_source without a {caster} runtime id` | B10 | 1 招、0 件道具 | 11 / 13 |
| 144 | `status.stat.per_stack` | B10 | 1 招、1 件道具 | 12 / 13 |
| 145 | `action:attack` | C11 | 4 招、2 件道具 | 12 / 13 |
| 146 | `attr:mana_pct` | C11 | 1 招、0 件道具 | 12 / 13 |
| 147 | `more than 16 locals` | C11 | 5 招、0 件道具 | 12 / 13 |
| 148 | `selector.exclude` | C11 | 6 招、5 件道具 | 13 / 13 |

### M6 第 6 批（9 位）

開始時已能全部載入：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 149 | `state:disarm` | A29 | 8 招、0 件道具 | 4 / 9 |
| 150 | `fn:random` | A27 | 1 招、0 件道具 | 5 / 9 |
| 151 | `stat:outgoing_damage_pct` | B32 | 3 招、1 件道具 | 6 / 9 |
| 152 | `state:flying_for_pathing` | C04 | 2 招、0 件道具 | 7 / 9 |
| 153 | `fn:avg_attack_against` | A31 | 2 招、0 件道具 | 7 / 9 |
| 154 | `requires` | A31 | 5 招、0 件道具 | 8 / 9 |
| 155 | `passive_trigger:on_deal_damage_pre` | A32 | 1 招、0 件道具 | 8 / 9 |
| 156 | `state:physical_immune` | A32 | 1 招、0 件道具 | 8 / 9 |
| 157 | `target.type:unit_or_point` | A32 | 3 招、1 件道具 | 9 / 9 |

### M6 第 7 批（9 位）

開始時已能全部載入：3 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 158 | `name:caster_hero` | C13 | 1 招、0 件道具 | 4 / 9 |
| 159 | `fn:random_int` | C21 | 3 招、0 件道具 | 5 / 9 |
| 160 | `action:set_stacks` | A06 | 4 招、1 件道具 | 6 / 9 |
| 161 | `attr:base_magic_resist` | B12 | 2 招、0 件道具 | 7 / 9 |
| 162 | `attr:bonus_armor` | A15 | 2 招、2 件道具 | 7 / 9 |
| 163 | `attr:damage` | A15 | 3 招、2 件道具 | 7 / 9 |
| 164 | `fn:armor_multiplier` | A15 | 2 招、2 件道具 | 8 / 9 |
| 165 | `action:modify_damage` | A08 | 1 招、0 件道具 | 8 / 9 |
| 166 | `fn:angle_between` | A08 | 1 招、0 件道具 | 8 / 9 |
| 167 | `fn:count_in_radius` | A08 | 1 招、0 件道具 | 8 / 9 |
| 168 | `fn:enemy_team` | A08 | 1 招、0 件道具 | 8 / 9 |
| 169 | `passive_trigger:on_take_damage_pre` | A08 | 1 招、1 件道具 | 9 / 9 |

### M6 第 8 批（9 位）

開始時已能全部載入：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 170 | `state:flying` | B33 | 2 招、0 件道具 | 2 / 9 |
| 171 | `reveal.teams:enemy` | A21 | 2 招、0 件道具 | 3 / 9 |
| 172 | `action:set_time_of_day` | C10 | 2 招、0 件道具 | 3 / 9 |
| 173 | `attr:is_night` | C10 | 4 招、0 件道具 | 3 / 9 |
| 174 | `damage.flag:ignore_armor` | C10 | 1 招、0 件道具 | 3 / 9 |
| 175 | `name:game` | C10 | 4 招、0 件道具 | 4 / 9 |
| 176 | `passive.stat.when` | C10 | 1 招、1 件道具 | 5 / 9 |
| 177 | `fn:is_ally` | A28 | 2 招、2 件道具 | 5 / 9 |
| 178 | `stat:physical_block_chance` | A28 | 1 招、0 件道具 | 5 / 9 |
| 179 | `state:evade_disabled` | A28 | 2 招、0 件道具 | 6 / 9 |
| 180 | `action:spend_mana` | A09 | 1 招、0 件道具 | 6 / 9 |
| 181 | `state:block_disabled` | A09 | 1 招、0 件道具 | 6 / 9 |
| 182 | `state:hex` | A09 | 1 招、0 件道具 | 7 / 9 |
| 183 | `action:random_pick` | C25 | 1 招、0 件道具 | 7 / 9 |
| 184 | `attr:is_passive` | C25 | 1 招、0 件道具 | 7 / 9 |
| 185 | `attr:is_ready` | C25 | 1 招、0 件道具 | 7 / 9 |
| 186 | `attr:level` | C25 | 2 招、0 件道具 | 7 / 9 |
| 187 | `fn:ability_in_slot` | C25 | 2 招、0 件道具 | 8 / 9 |
| 188 | `ally_selector.effect:for_each` | B23 | 1 招、0 件道具 | 8 / 9 |
| 189 | `ally_selector.effect:projectile` | B23 | 1 招、0 件道具 | 8 / 9 |
| 190 | `ally_selector.effect:remove_status` | B23 | 1 招、1 件道具 | 8 / 9 |
| 191 | `ally_selector.effect:teleport` | B23 | 1 招、0 件道具 | 8 / 9 |
| 192 | `attr:owner` | B23 | 4 招、0 件道具 | 8 / 9 |
| 193 | `passive_trigger:on_moved` | B23 | 1 招、0 件道具 | 8 / 9 |
| 194 | `selector.flag:include_dead` | B23 | 1 招、0 件道具 | 8 / 9 |
| 195 | `selector.team:both` | B23 | 4 招、1 件道具 | 9 / 9 |

### M6 第 9 批（11 位）

開始時已能全部載入：1 位；做完本批的項目後：11 位。

| # | 要支援的寫法 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|
| 196 | `passive:orb_slot` | B15 | 1 招、1 件道具 | 2 / 11 |
| 197 | `selector.relative_to` | C07 | 1 招、0 件道具 | 3 / 11 |
| 198 | `action:chain` | B28 | 1 招、0 件道具 | 3 / 11 |
| 199 | `attr:is_invisible` | B28 | 1 招、0 件道具 | 4 / 11 |
| 200 | `action:give_item` | B07 | 2 招、1 件道具 | 4 / 11 |
| 201 | `action:pick_item` | B07 | 2 招、0 件道具 | 5 / 11 |
| 202 | `action:swap_positions` | A13 | 3 招、0 件道具 | 5 / 11 |
| 203 | `fn:random_point` | A13 | 1 招、0 件道具 | 5 / 11 |
| 204 | `name:vector_end` | A13 | 1 招、0 件道具 | 5 / 11 |
| 205 | `target.type:vector` | A13 | 1 招、0 件道具 | 6 / 11 |
| 206 | `action:add_ability` | B16 | 2 招、0 件道具 | 6 / 11 |
| 207 | `action:remove_unit` | B16 | 1 招、0 件道具 | 6 / 11 |
| 208 | `action:set_base_max_hp` | B16 | 4 招、0 件道具 | 7 / 11 |
| 209 | `action:remove_ability` | C08 | 1 招、0 件道具 | 7 / 11 |
| 210 | `state:attack_immune` | C08 | 1 招、0 件道具 | 7 / 11 |
| 211 | `state:blind` | C08 | 1 招、0 件道具 | 8 / 11 |
| 212 | `ally_selector.effect:kill` | B24 | 1 招、0 件道具 | 8 / 11 |
| 213 | `selector.flag:include_illusions` | B24 | 3 招、1 件道具 | 8 / 11 |
| 214 | `selector.types:building` | B24 | 1 招、0 件道具 | 8 / 11 |
| 215 | `state:cannot_miss` | B24 | 1 招、0 件道具 | 9 / 11 |
| 216 | `action:destroy_tree` | B22 | 1 招、0 件道具 | 9 / 11 |
| 217 | `action:regrow_trees` | B22 | 1 招、0 件道具 | 9 / 11 |
| 218 | `fn:circle_point` | B22 | 1 招、0 件道具 | 9 / 11 |
| 219 | `status.immune_to` | B22 | 1 招、0 件道具 | 9 / 11 |
| 220 | `target.type:tree` | B22 | 2 招、0 件道具 | 10 / 11 |
| 221 | `action:give_gold` | C06 | 1 招、0 件道具 | 10 / 11 |
| 222 | `action:remove_item` | C06 | 1 招、1 件道具 | 10 / 11 |
| 223 | `action:take_gold` | C06 | 1 招、0 件道具 | 10 / 11 |
| 224 | `attr:gold` | C06 | 1 招、0 件道具 | 10 / 11 |
| 225 | `attr:name` | C06 | 1 招、0 件道具 | 11 / 11 |

## 2. 每項寫法擋住多少

「第一個錯誤」是直譯器實際載入時，這一項是第一個報錯原因的條目數。一招常常有好幾項阻擋，所以兩欄不會相等。

| 寫法 | 招式 | 道具 | 英雄 | 第一個錯誤 |
|---|---|---|---|---|
| `status.triggers` | 116 | 16 | 73 | 12 |
| `target.flag:pierce_magic_immune` | 62 | 1 | 55 | 2 |
| `action:repeat` | 64 | 20 | 47 | 20 |
| `action:projectile` | 52 | 9 | 42 | 32 |
| `action:delay` | 51 | 3 | 37 | 9 |
| `action:set_state` | 45 | 7 | 29 | 19 |
| `action:teleport` | 31 | 5 | 28 | 7 |
| `fn:has_status` | 36 | 7 | 22 | 0 |
| `stat:slow_pct` | 23 | 9 | 22 | 7 |
| `event:on_upgrade` | 28 | 0 | 21 | 0 |
| `summon template not registered` | 21 | 8 | 19 | 3 |
| `token:state` | 30 | 7 | 17 | 0 |
| `passive:visibility` | 20 | 0 | 16 | 0 |
| `status.vars` | 17 | 1 | 16 | 1 |
| `action:dodge_projectiles` | 21 | 2 | 15 | 11 |
| `attr:is_magic_immune` | 17 | 4 | 15 | 0 |
| `state:magic_immune` | 16 | 13 | 15 | 0 |
| `action:create_unit.owner` | 15 | 8 | 15 | 12 |
| `action:issue_order` | 17 | 2 | 14 | 3 |
| `action:damage.source` | 17 | 1 | 12 | 46 |
| `action:apply_status.source` | 14 | 1 | 12 | 18 |
| `passive_trigger:periodic` | 13 | 11 | 12 | 0 |
| `state:invulnerable` | 13 | 1 | 12 | 0 |
| `status.stacking:independent` | 13 | 2 | 12 | 0 |
| `action:remove_status.team_of_source` | 12 | 16 | 12 | 0 |
| `selector.flag:visible_only` | 14 | 7 | 11 | 10 |
| `action:create_unit.team` | 13 | 0 | 11 | 3 |
| `event:on_channel_end` | 12 | 0 | 11 | 2 |
| `passive_trigger:on_attack_windup` | 12 | 3 | 11 | 0 |
| `action:remove_status.dispel` | 11 | 16 | 11 | 29 |
| `passive:aura` | 11 | 22 | 10 | 9 |
| `action:create_unit.overrides` | 10 | 3 | 10 | 13 |
| `event:on_phase_start` | 10 | 2 | 10 | 0 |
| `target.flag:include_magic_immune` | 10 | 8 | 10 | 0 |
| `fn:exists` | 13 | 3 | 9 | 0 |
| `selector.limit` | 12 | 5 | 9 | 0 |
| `action:set_health` | 10 | 1 | 9 | 3 |
| `action:face` | 9 | 1 | 9 | 2 |
| `fn:ability_level` | 16 | 0 | 8 | 0 |
| `action:set_ability_level` | 14 | 0 | 8 | 4 |
| `selector.shape:global` | 12 | 0 | 8 | 0 |
| `reveal.teams:own` | 11 | 3 | 8 | 0 |
| `action:add_stacks` | 10 | 2 | 8 | 1 |
| `attr:forward` | 10 | 2 | 8 | 1 |
| `passive_trigger:on_take_damage` | 10 | 3 | 8 | 0 |
| `action:modify_cooldown` | 9 | 2 | 8 | 5 |
| `ally_selector.types` | 9 | 13 | 8 | 11 |
| `attr:position` | 9 | 0 | 8 | 0 |
| `status.status_resistance` | 9 | 0 | 8 | 0 |
| `action:apply_status.vars` | 8 | 0 | 8 | 7 |
| `passive_trigger:on_kill` | 8 | 5 | 8 | 0 |
| `status.aura` | 8 | 1 | 8 | 0 |
| `status.stacking:replace` | 8 | 0 | 8 | 0 |
| `status.stacking:stacks` | 8 | 4 | 8 | 0 |
| `action:set_ability_active` | 9 | 0 | 7 | 3 |
| `action:knockback` | 8 | 2 | 7 | 4 |
| `state:disarm` | 8 | 0 | 7 | 2 |
| `action:set_display_stacks` | 12 | 3 | 6 | 3 |
| `attr:hp_pct` | 10 | 0 | 6 | 2 |
| `action:apply_status.stacks` | 8 | 4 | 6 | 14 |
| `fn:stacks` | 8 | 0 | 6 | 0 |
| `action:chance` | 7 | 7 | 6 | 14 |
| `state:command_restricted` | 7 | 1 | 6 | 0 |
| `status.stacking:keep` | 7 | 1 | 6 | 0 |
| `fn:is_ability` | 6 | 0 | 6 | 0 |
| `state:low_attack_priority` | 6 | 0 | 6 | 0 |
| `fn:is_template` | 8 | 0 | 5 | 0 |
| `action:create_illusion` | 7 | 0 | 5 | 2 |
| `ally_selector.effect:apply_status` | 6 | 11 | 5 | 0 |
| `attr:is_toggle` | 5 | 1 | 5 | 0 |
| `fn:floor` | 5 | 0 | 5 | 0 |
| `fn:has_state` | 5 | 0 | 5 | 0 |
| `stat:move_speed` | 5 | 1 | 5 | 2 |
| `stat:vision_night` | 5 | 3 | 5 | 0 |
| `state:mute` | 5 | 2 | 5 | 0 |
| `action:burn_mana` | 6 | 1 | 4 | 4 |
| `selector.exclude` | 6 | 5 | 4 | 0 |
| `selector.order` | 6 | 5 | 4 | 0 |
| `attr:mana` | 5 | 1 | 4 | 0 |
| `requires` | 5 | 0 | 4 | 0 |
| `selector.flag:include_hidden` | 5 | 0 | 4 | 0 |
| `target.flag:visible_only` | 5 | 0 | 4 | 0 |
| `action:set_stacks` | 4 | 1 | 4 | 1 |
| `attr:int` | 4 | 0 | 4 | 0 |
| `fn:rotate` | 4 | 1 | 4 | 0 |
| `name:ability` | 4 | 0 | 4 | 0 |
| `passive_trigger:on_attack_launch` | 4 | 0 | 4 | 0 |
| `state:provides_vision` | 4 | 1 | 4 | 0 |
| `state:ranged_attack` | 4 | 0 | 4 | 0 |
| `status.scope` | 4 | 1 | 4 | 0 |
| `more than 16 locals` | 5 | 0 | 3 | 0 |
| `action:attack` | 4 | 2 | 3 | 1 |
| `selector.team:both` | 4 | 1 | 3 | 0 |
| `selector.types:building+creep+hero+neutral+summon` | 4 | 3 | 3 | 12 |
| `action:create_unit.facing` | 3 | 0 | 3 | 3 |
| `action:heal.source` | 3 | 0 | 3 | 6 |
| `action:reveal.ignore_terrain` | 3 | 1 | 3 | 3 |
| `action:set_mana` | 3 | 0 | 3 | 1 |
| `action:swap_abilities` | 3 | 0 | 3 | 0 |
| `ally_selector.effect:remove_unit` | 3 | 0 | 3 | 0 |
| `attr:cast_range` | 3 | 0 | 3 | 0 |
| `attr:is_illusion` | 3 | 2 | 3 | 0 |
| `attr:is_item` | 3 | 0 | 3 | 0 |
| `fn:owner_of` | 3 | 0 | 3 | 0 |
| `fn:random_int` | 3 | 0 | 3 | 0 |
| `passive_trigger:on_attacked_hit` | 3 | 7 | 3 | 0 |
| `stat:outgoing_damage_pct` | 3 | 1 | 3 | 0 |
| `stat:vision_day` | 3 | 0 | 3 | 0 |
| `state:unselectable` | 3 | 0 | 3 | 0 |
| `target.type:unit_or_point` | 3 | 1 | 3 | 0 |
| `attr:is_night` | 4 | 0 | 2 | 0 |
| `name:game` | 4 | 0 | 2 | 0 |
| `action:kill` | 3 | 0 | 2 | 0 |
| `action:leap` | 3 | 0 | 2 | 2 |
| `action:swap_positions` | 3 | 0 | 2 | 0 |
| `attr:damage` | 3 | 2 | 2 | 1 |
| `event:on_channel_tick` | 3 | 0 | 2 | 0 |
| `action:add_ability` | 2 | 0 | 2 | 0 |
| `action:cancel_delay` | 2 | 0 | 2 | 0 |
| `action:give_item` | 2 | 1 | 2 | 0 |
| `action:pick_item` | 2 | 0 | 2 | 0 |
| `action:refund` | 2 | 2 | 2 | 1 |
| `action:remove_status.tags` | 2 | 0 | 2 | 5 |
| `action:restore_mana` | 2 | 2 | 2 | 2 |
| `action:set_time_of_day` | 2 | 0 | 2 | 2 |
| `action:set_var` | 2 | 0 | 2 | 1 |
| `action:zone` | 2 | 0 | 2 | 1 |
| `ally_selector.effect:set_mana` | 2 | 0 | 2 | 0 |
| `attr:agi` | 2 | 0 | 2 | 0 |
| `attr:base_magic_resist` | 2 | 0 | 2 | 0 |
| `attr:is_creep` | 2 | 0 | 2 | 1 |
| `attr:level` | 2 | 0 | 2 | 0 |
| `attr:point` | 2 | 0 | 2 | 0 |
| `attr:str` | 2 | 0 | 2 | 0 |
| `fn:ability_in_slot` | 2 | 0 | 2 | 0 |
| `fn:avg_attack_against` | 2 | 0 | 2 | 0 |
| `fn:has_status_tag` | 2 | 6 | 2 | 1 |
| `fn:is_ally` | 2 | 2 | 2 | 0 |
| `fn:status_var` | 2 | 0 | 2 | 0 |
| `fn:vec` | 2 | 0 | 2 | 0 |
| `passive_trigger:on_created` | 2 | 44 | 2 | 8 |
| `reveal.teams:enemy` | 2 | 0 | 2 | 0 |
| `stat:status_resistance_pct` | 2 | 1 | 2 | 0 |
| `state:break` | 2 | 0 | 2 | 1 |
| `state:evade_disabled` | 2 | 0 | 2 | 0 |
| `state:flying` | 2 | 0 | 2 | 0 |
| `state:flying_for_pathing` | 2 | 0 | 2 | 0 |
| `state:no_healthbar` | 2 | 0 | 2 | 0 |
| `state:revealed` | 2 | 6 | 2 | 0 |
| `status.forced_move` | 2 | 0 | 2 | 1 |
| `action:set_base_max_hp` | 4 | 0 | 1 | 0 |
| `attr:owner` | 4 | 0 | 1 | 0 |
| `selector.flag:include_illusions` | 3 | 1 | 1 | 4 |
| `attr:bonus_armor` | 2 | 2 | 1 | 0 |
| `fn:armor_multiplier` | 2 | 2 | 1 | 0 |
| `target.type:tree` | 2 | 0 | 1 | 0 |
| `action:chain` | 1 | 0 | 1 | 0 |
| `action:dash` | 1 | 1 | 1 | 0 |
| `action:destroy_tree` | 1 | 0 | 1 | 0 |
| `action:end_channel` | 1 | 0 | 1 | 1 |
| `action:give_gold` | 1 | 0 | 1 | 1 |
| `action:modify_damage` | 1 | 0 | 1 | 1 |
| `action:random_pick` | 1 | 0 | 1 | 1 |
| `action:regrow_trees` | 1 | 0 | 1 | 0 |
| `action:remove_ability` | 1 | 0 | 1 | 0 |
| `action:remove_item` | 1 | 1 | 1 | 0 |
| `action:remove_status.states` | 1 | 0 | 1 | 1 |
| `action:remove_unit` | 1 | 0 | 1 | 0 |
| `action:spend_mana` | 1 | 0 | 1 | 0 |
| `action:take_gold` | 1 | 0 | 1 | 0 |
| `ally_selector.effect:for_each` | 1 | 0 | 1 | 0 |
| `ally_selector.effect:kill` | 1 | 0 | 1 | 0 |
| `ally_selector.effect:projectile` | 1 | 0 | 1 | 0 |
| `ally_selector.effect:remove_status` | 1 | 1 | 1 | 0 |
| `ally_selector.effect:teleport` | 1 | 0 | 1 | 0 |
| `attr:cooldown` | 1 | 0 | 1 | 0 |
| `attr:cooldown_remaining` | 1 | 0 | 1 | 0 |
| `attr:gold` | 1 | 0 | 1 | 0 |
| `attr:is_invisible` | 1 | 0 | 1 | 0 |
| `attr:is_passive` | 1 | 0 | 1 | 0 |
| `attr:is_ready` | 1 | 0 | 1 | 0 |
| `attr:mana_pct` | 1 | 0 | 1 | 0 |
| `attr:name` | 1 | 0 | 1 | 0 |
| `damage.flag:ignore_armor` | 1 | 0 | 1 | 0 |
| `fn:ability_of` | 1 | 0 | 1 | 0 |
| `fn:angle_between` | 1 | 0 | 1 | 0 |
| `fn:circle_point` | 1 | 0 | 1 | 0 |
| `fn:count_in_radius` | 1 | 0 | 1 | 0 |
| `fn:enemy_team` | 1 | 0 | 1 | 0 |
| `fn:random` | 1 | 0 | 1 | 0 |
| `fn:random_point` | 1 | 0 | 1 | 0 |
| `name:caster_hero` | 1 | 0 | 1 | 0 |
| `name:vector_end` | 1 | 0 | 1 | 0 |
| `op:%` | 1 | 0 | 1 | 0 |
| `passive.stat.when` | 1 | 1 | 1 | 0 |
| `passive:orb_slot` | 1 | 1 | 1 | 0 |
| `passive_trigger:on_deal_damage_pre` | 1 | 0 | 1 | 0 |
| `passive_trigger:on_death` | 1 | 2 | 1 | 0 |
| `passive_trigger:on_heal_received` | 1 | 0 | 1 | 0 |
| `passive_trigger:on_moved` | 1 | 0 | 1 | 0 |
| `passive_trigger:on_order_targeted` | 1 | 0 | 1 | 0 |
| `passive_trigger:on_take_damage_pre` | 1 | 1 | 1 | 0 |
| `selector.flag:exclude_illusions` | 1 | 0 | 1 | 1 |
| `selector.flag:include_dead` | 1 | 0 | 1 | 0 |
| `selector.relative_to` | 1 | 0 | 1 | 0 |
| `selector.team:ally_not_self` | 1 | 0 | 1 | 0 |
| `selector.types:building` | 1 | 0 | 1 | 0 |
| `stat:physical_block_chance` | 1 | 0 | 1 | 1 |
| `stat:slow_resistance_pct` | 1 | 0 | 1 | 0 |
| `state:attack_immune` | 1 | 0 | 1 | 0 |
| `state:attack_slow_immune` | 1 | 0 | 1 | 0 |
| `state:blind` | 1 | 0 | 1 | 0 |
| `state:block_disabled` | 1 | 0 | 1 | 0 |
| `state:cannot_miss` | 1 | 0 | 1 | 0 |
| `state:hex` | 1 | 0 | 1 | 0 |
| `state:melee_attack` | 1 | 0 | 1 | 0 |
| `state:not_on_minimap` | 1 | 0 | 1 | 0 |
| `state:physical_immune` | 1 | 0 | 1 | 0 |
| `status.immune_to` | 1 | 0 | 1 | 0 |
| `status.per_source without a {caster} runtime id` | 1 | 0 | 1 | 0 |
| `status.reapply_if_removed` | 1 | 13 | 1 | 1 |
| `status.stat.per_stack` | 1 | 1 | 1 | 0 |
| `status.unique_group` | 1 | 0 | 1 | 1 |
| `target.type:vector` | 1 | 0 | 1 | 0 |
| `token:var` | 1 | 0 | 1 | 0 |
| `action:modify_base_stat` | 0 | 0 | 0 | 1 |
| `action:remove_status.exclude_source_tags` | 0 | 3 | 0 | 0 |
| `action:remove_status.exclude_tags` | 0 | 12 | 0 | 0 |
| `action:remove_status.limit` | 0 | 0 | 0 | 4 |
| `action:remove_status.min_duration` | 0 | 3 | 0 | 0 |
| `action:remove_status.source_is` | 0 | 1 | 0 | 0 |
| `ally restore: `apply_status` is not a share-of-max restore` | 0 | 0 | 0 | 3 |
| `ally_selector.effect:if` | 0 | 3 | 0 | 0 |
| `attr:armor` | 0 | 1 | 0 | 0 |
| `attr:base_attack_range` | 0 | 3 | 0 | 0 |
| `attr:effect_multiplier` | 0 | 1 | 0 | 0 |
| `call needs args` | 0 | 0 | 0 | 9 |
| `damage.flag:no_source_bonuses` | 0 | 0 | 0 | 1 |
| `event:on_owner_death` | 0 | 3 | 0 | 0 |
| `event:on_unequip` | 0 | 2 | 0 | 1 |
| `fn:has_item` | 0 | 2 | 0 | 0 |
| `fn:is_hero_id` | 0 | 2 | 0 | 0 |
| `passive_trigger:on_removed` | 0 | 26 | 0 | 0 |
| `reveal teams `own` is not run yet` | 0 | 0 | 0 | 14 |
| `selector team Some("both") is not run yet` | 0 | 0 | 0 | 3 |
| `selector.exclude_types` | 0 | 1 | 0 | 0 |
| `selector.shape` | 0 | 0 | 0 | 17 |
| `selector.types:building+creep+hero+neutral+summon+tower` | 0 | 4 | 0 | 5 |
| `selector.types:building+creep+neutral+summon` | 0 | 1 | 0 | 1 |
| `selector.types:building+hero+tower` | 0 | 1 | 0 | 1 |
| `selector.types:courier+creep+hero+neutral+summon+ward` | 0 | 0 | 0 | 1 |
| `selector.types:creep+neutral+summon` | 0 | 1 | 0 | 1 |
| `selector: `exclude_types` is not run yet` | 0 | 0 | 0 | 1 |
| `selector: `exclude` is not run yet` | 0 | 0 | 0 | 14 |
| `selector: `limit` is not run yet` | 0 | 0 | 0 | 18 |
| `stat:all_stats` | 0 | 9 | 0 | 0 |
| `stat:cast_point_reduction_pct` | 0 | 1 | 0 | 0 |
| `stat:heal_reduction_pct` | 0 | 2 | 0 | 1 |
| `stat:spell_lifesteal_pct` | 0 | 1 | 0 | 0 |
| `status `a36e_slow` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `b03w_stun` is not this ability's` | 0 | 0 | 0 | 2 |
| `status `b11r_phased` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `b16mmf_invis` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `b26w_charge` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `b36w_mark` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `c14e_lock` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `c21e_combo` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `c23r_on_spell` is not this ability's` | 0 | 0 | 0 | 3 |
| `status `c25e_dot` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `ninja1_invisible` is not this ability's` | 0 | 0 | 0 | 1 |
| `status `sukemune_resist_cut` stat: `when` is not run yet` | 0 | 0 | 0 | 1 |
| `status.stacking` | 0 | 0 | 0 | 4 |
| `status.stat.when` | 0 | 4 | 0 | 0 |
| `target.types:creep+neutral+summon+courier+building` | 0 | 1 | 0 | 0 |

## 3. 掃描與直譯器實際結果的差異

沒有差異：掃描判定可載入的條目都能載入，載入失敗的條目都至少找到一項阻擋。

## 4. 重跑

```
tools\lua\lua.exe scripts\gen_v3_content.lua
cargo run --release --manifest-path tools\ir_probe\Cargo.toml --target-dir target\ir_probe -- target\v3-content\v3-content.json > target\v3-content\ir-probe.jsonl
tools\lua\lua.exe docs\character_pipeline\tools\v3_ir_blockers.lua --probe target\v3-content\ir-probe.jsonl
```

§3 有差異時，先更新 `v3_ir_blockers.lua` 開頭的支援清單（照 omoba-core/src/runtime/native/ir.rs），再重跑。

