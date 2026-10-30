# 產生器還沒產生的寫法與實作優先序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua` 產生，不要手改。取代 [ir-blockers.md](ir-blockers.md)（M3' ③，[m3-rust-codegen-design.md](m3-rust-codegen-design.md) §2.3）。產生器（`v3_rust.lua`）或 `ability_kit` 每補一批寫法就重跑。

- 英雄技能：447 招中 310 招產生成 Rust。還不能產生的招式不輸出程式碼，列在這份報告；該英雄不進 `hero_dlls`。全部技能都產生的英雄：26 / 97 位。
- 道具（items.dll，M7）：168 件中 36 件產生成 Rust。
- 一位英雄要全部技能都產生才算解鎖。順序是 M5（已上架的 10 位），然後 M6 第 1～9 批（[m6-waves.md](m6-waves.md)）；每批內先做「每多支援一項、能解鎖最多英雄」的那位英雄缺的項目。
- 「主要負責」是起點：`產生器` 是運算式、選取、目標這類產生器自己能降低的寫法；`ability_kit` 是要先有 kit 呼叫或 host 元件的寫法（工作線 B）。
- 「分類」（D7，m3-rust-codegen-design.md §2.4）：英雄技能與道具合計 2 招以上用到的寫法是 `共用`，做成 v3 詞彙＋產生器＋`ability_kit`；只有 1 招用到的是 `hook 候選`，在 v3 標 `hook` 並手寫產生的 trait（§5 的清單）。

## 1. 實作順序

### M5（已上架的 10 位英雄：B01、C17 換掉 IR，其餘換掉 Rust handler）（10 位）

開始時已全部產生：7 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 1 | `stat:status_resistance_pct` | ability_kit | 共用 | A07 | 2 招、5 件道具 | 8 / 10 |
| 2 | `passive_trigger:on_order_targeted` | ability_kit | hook 候選 | B06 | 1 招、0 件道具 | 9 / 10 |
| 3 | `action:issue_order.queue` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 4 | `unit_template acquisition_range is `$acquisition` has no value` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 5 | `unit_template attack range or interval missing` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 6 | `unit_template attack.interval is `$attack_interval` has no value` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 7 | `unit_template flying` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 9 / 10 |
| 8 | `unit_template passive.states` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 9 | `unit_template stats.hp_regen` | ability_kit | 共用 | C15 | 12 招、0 件道具 | 9 / 10 |
| 10 | `unit_template stats.move_speed is `$move_speed` has no value` | ability_kit | hook 候選 | C15 | 1 招、0 件道具 | 9 / 10 |
| 11 | `unit_template stats.move_speed missing` | ability_kit | 共用 | C15 | 4 招、0 件道具 | 9 / 10 |
| 12 | `unit_template turn_rate` | ability_kit | 共用 | C15 | 7 招、0 件道具 | 10 / 10 |

### M6 第 1 批（9 位）

開始時已全部產生：6 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 13 | `status.stacking:independent with states` | ability_kit | 共用 | C22 | 4 招、1 件道具 | 7 / 9 |
| 14 | `event.damage.lethal` | ability_kit | hook 候選 | B26 | 1 招、0 件道具 | 8 / 9 |
| 15 | `action:knockback.contact_radius` | ability_kit | hook 候選 | A14 | 1 招、0 件道具 | 8 / 9 |
| 16 | `action:knockback.on_end` | ability_kit | hook 候選 | A14 | 1 招、0 件道具 | 8 / 9 |
| 17 | `action:knockback.stop_on_contact` | ability_kit | hook 候選 | A14 | 1 招、0 件道具 | 9 / 9 |

### M6 第 2 批（10 位）

開始時已全部產生：5 位；做完本批的項目後：10 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 18 | `event:point` | ability_kit | 共用 | A19 | 2 招、0 件道具 | 6 / 10 |
| 19 | `action:modify_attack without crit_multiplier` | ability_kit | 共用 | C09 | 2 招、0 件道具 | 6 / 10 |
| 20 | `action:modify_attack.is_orb` | ability_kit | hook 候選 | C09 | 1 招、0 件道具 | 7 / 10 |
| 21 | `action:modify_attack.cannot_miss` | ability_kit | 共用 | C03 | 1 招、2 件道具 | 8 / 10 |
| 22 | `event.damage.raw` | ability_kit | 共用 | B21 | 1 招、1 件道具 | 8 / 10 |
| 23 | `passive_trigger:on_heal_received` | ability_kit | hook 候選 | B21 | 1 招、0 件道具 | 9 / 10 |
| 24 | `state:provides_vision` | ability_kit | 共用 | A30 | 4 招、2 件道具 | 9 / 10 |
| 25 | `unit_template acquisition_range 300 differs from attack.range` | ability_kit | hook 候選 | A30 | 1 招、0 件道具 | 9 / 10 |
| 26 | `unit_template ai:idle` | ability_kit | 共用 | A30 | 2 招、0 件道具 | 9 / 10 |
| 27 | `unit_template stats.mana` | ability_kit | 共用 | A30 | 10 招、0 件道具 | 9 / 10 |
| 28 | `unit_template vision.day differs from vision.night` | ability_kit | 共用 | A30 | 11 招、0 件道具 | 10 / 10 |

### M6 第 3 批（9 位）

開始時已全部產生：5 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 29 | `status.reapply_if_removed` | ability_kit | 共用 | A22 | 1 招、14 件道具 | 6 / 9 |
| 30 | `status.per_source without {caster}` | ability_kit | hook 候選 | B31 | 1 招、0 件道具 | 7 / 9 |
| 31 | `action:modify_cooldown.ability:se` | ability_kit | hook 候選 | C25 | 1 招、0 件道具 | 7 / 9 |
| 32 | `action:modify_cooldown.ability:sq` | ability_kit | hook 候選 | C25 | 1 招、0 件道具 | 7 / 9 |
| 33 | `action:modify_cooldown.ability:sw` | ability_kit | hook 候選 | C25 | 1 招、0 件道具 | 7 / 9 |
| 34 | `fn:ability_in_slot` | 產生器 | 共用 | C25 | 2 招、0 件道具 | 8 / 9 |
| 35 | `action:remove_status of its own independent status (this instance waits)` | ability_kit | hook 候選 | A06 | 1 招、0 件道具 | 8 / 9 |
| 36 | `aura.flag:include_illusions` | ability_kit | 共用 | A06 | 2 招、0 件道具 | 8 / 9 |
| 37 | `status.vars of an independent status` | ability_kit | 共用 | A06 | 2 招、0 件道具 | 8 / 9 |
| 38 | `status_trigger:on_attack_miss` | ability_kit | hook 候選 | A06 | 1 招、0 件道具 | 9 / 9 |

### M6 第 4 批（9 位）

開始時已全部產生：1 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 39 | `action:create_illusion` | ability_kit | 共用 | B02 | 10 招、0 件道具 | 1 / 9 |
| 40 | `expr: a position from a sym:ill` | 產生器 | 共用 | B02 | 3 招、0 件道具 | 1 / 9 |
| 41 | `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | B02 | 8 招、0 件道具 | 3 / 9 |
| 42 | `status.absorb.on_absorb` | ability_kit | 共用 | A24 | 6 招、1 件道具 | 3 / 9 |
| 43 | `status.absorb.on_broken` | ability_kit | 共用 | A24 | 7 招、0 件道具 | 4 / 9 |
| 44 | `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | A36 | 3 招、0 件道具 | 4 / 9 |
| 45 | `unit_template abilities on a dummy` | ability_kit | hook 候選 | A36 | 1 招、0 件道具 | 4 / 9 |
| 46 | `unit_template stats on a dummy` | ability_kit | 共用 | A36 | 10 招、0 件道具 | 5 / 9 |
| 47 | `status.stacking:independent with slow_pct` | ability_kit | 共用 | B34 | 2 招、0 件道具 | 5 / 9 |
| 48 | `unit_template statuses on a dummy` | ability_kit | 共用 | B34 | 7 招、0 件道具 | 6 / 9 |
| 49 | `action:cancel_delay` | ability_kit | 共用 | B14 | 4 招、0 件道具 | 6 / 9 |
| 50 | `action:delay.id` | ability_kit | 共用 | B14 | 4 招、0 件道具 | 6 / 9 |
| 51 | `action:set_time_of_day` | ability_kit | 共用 | B14 | 2 招、0 件道具 | 7 / 9 |
| 52 | `attr:owner` | 產生器 | hook 候選 | A20 | 1 招、0 件道具 | 7 / 9 |
| 53 | `state:not_on_minimap` | ability_kit | 共用 | A20 | 4 招、0 件道具 | 8 / 9 |
| 54 | `aura.flag:exclude_invisible` | ability_kit | hook 候選 | B17 | 1 招、0 件道具 | 8 / 9 |
| 55 | `aura.status:independent` | ability_kit | hook 候選 | B17 | 1 招、0 件道具 | 8 / 9 |
| 56 | `event.damage.is_attack` | ability_kit | 共用 | B17 | 2 招、0 件道具 | 8 / 9 |
| 57 | `unit_template acquisition_range 1200 differs from attack.range` | ability_kit | 共用 | B17 | 2 招、0 件道具 | 8 / 9 |
| 58 | `unit_template ai:owner_controlled` | ability_kit | 共用 | B17 | 13 招、0 件道具 | 8 / 9 |
| 59 | `unit_template attack.projectile_speed 1500 (the host fires at 900)` | ability_kit | 共用 | B17 | 2 招、0 件道具 | 8 / 9 |
| 60 | `unit_template bounty.gold (summons give none)` | ability_kit | 共用 | B17 | 15 招、0 件道具 | 8 / 9 |
| 61 | `unit_template tag:ward` | ability_kit | hook 候選 | B17 | 1 招、0 件道具 | 9 / 9 |

### M6 第 5 批（9 位）

開始時已全部產生：0 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 62 | `action:chance.mode` | ability_kit | hook 候選 | A17 | 1 招、0 件道具 | 1 / 9 |
| 63 | `unit_template bounty on a dummy` | ability_kit | 共用 | A27 | 5 招、0 件道具 | 2 / 9 |
| 64 | `selector.flag:exclude_invisible` | 產生器 | hook 候選 | B03 | 1 招、0 件道具 | 3 / 9 |
| 65 | `fn:avg_attack_against` | 產生器 | 共用 | A25 | 6 招、0 件道具 | 4 / 9 |
| 66 | `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | C13 | 1 招、0 件道具 | 5 / 9 |
| 67 | `action:cue.at of a err` | ability_kit | hook 候選 | B05 | 1 招、0 件道具 | 5 / 9 |
| 68 | `state:revealed` | ability_kit | 共用 | B05 | 5 招、8 件道具 | 6 / 9 |
| 69 | `action:refund` | ability_kit | 共用 | A33 | 2 招、2 件道具 | 6 / 9 |
| 70 | `unit_template acquisition_range 600 differs from attack.range` | ability_kit | 共用 | A33 | 2 招、0 件道具 | 7 / 9 |
| 71 | `unit_template bounty.xp (summons give none)` | ability_kit | 共用 | B19 | 5 招、0 件道具 | 8 / 9 |
| 72 | `selector.types:building+creep+neutral+summon` | 產生器 | 共用 | B18 | 1 招、1 件道具 | 8 / 9 |
| 73 | `unit_template acquisition_range 1000 differs from attack.range` | ability_kit | 共用 | B18 | 7 招、0 件道具 | 9 / 9 |

### M6 第 6 批（9 位）

開始時已全部產生：3 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 74 | `unit_template passive.aura on a dummy` | ability_kit | hook 候選 | A10 | 1 招、0 件道具 | 4 / 9 |
| 75 | `stat.when` | ability_kit | 共用 | C10 | 1 招、6 件道具 | 5 / 9 |
| 76 | `action:impulse` | ability_kit | hook 候選 | B25 | 1 招、0 件道具 | 5 / 9 |
| 77 | `attr:cooldown_remaining` | 產生器 | hook 候選 | B25 | 1 招、0 件道具 | 5 / 9 |
| 78 | `fn:max of non-numbers` | 產生器 | hook 候選 | B25 | 1 招、0 件道具 | 6 / 9 |
| 79 | `action:heal.source:owner_of(self)` | ability_kit | 共用 | A29 | 2 招、0 件道具 | 6 / 9 |
| 80 | `status_trigger:on_mana_spent` | ability_kit | hook 候選 | A29 | 1 招、0 件道具 | 6 / 9 |
| 81 | `unit_template ai:chase_target` | ability_kit | 共用 | A29 | 3 招、0 件道具 | 6 / 9 |
| 82 | `unit_template chase` | ability_kit | 共用 | A29 | 3 招、0 件道具 | 6 / 9 |
| 83 | `unit_template no attack (the dummy unit type, line B)` | ability_kit | 共用 | A29 | 4 招、0 件道具 | 6 / 9 |
| 84 | `unit_template stats.hp missing` | ability_kit | 共用 | A29 | 4 招、0 件道具 | 6 / 9 |
| 85 | `unit_template statuses` | ability_kit | 共用 | A29 | 2 招、0 件道具 | 8 / 9 |
| 86 | `state:flying` | ability_kit | 共用 | C16 | 4 招、0 件道具 | 8 / 9 |
| 87 | `unit_template acquisition_range 1100 differs from attack.range` | ability_kit | hook 候選 | C16 | 1 招、0 件道具 | 8 / 9 |
| 88 | `unit_template attack.damage_type:pure` | ability_kit | hook 候選 | C16 | 1 招、0 件道具 | 8 / 9 |
| 89 | `unit_template attack.projectile_speed 2000 (the host fires at 900)` | ability_kit | hook 候選 | C16 | 1 招、0 件道具 | 9 / 9 |

### M6 第 7 批（9 位）

開始時已全部產生：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 90 | `attr:bonus_armor` | 產生器 | 共用 | A03 | 3 招、2 件道具 | 3 / 9 |
| 91 | `status_trigger:on_moved` | ability_kit | 共用 | A03 | 3 招、0 件道具 | 5 / 9 |
| 92 | `selector.flag:exclude_illusions` | 產生器 | hook 候選 | B36 | 1 招、0 件道具 | 6 / 9 |
| 93 | `action:attack without instant (a projectile attack)` | ability_kit | 共用 | A31 | 1 招、1 件道具 | 7 / 9 |
| 94 | `fn:count_in_radius` | 產生器 | 共用 | C21 | 3 招、1 件道具 | 8 / 9 |
| 95 | `action:modify_damage outside a damage-pre trigger` | ability_kit | hook 候選 | A08 | 1 招、0 件道具 | 9 / 9 |

### M6 第 8 批（9 位）

開始時已全部產生：2 位；做完本批的項目後：9 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 96 | `unit_template passive.triggers` | ability_kit | hook 候選 | A09 | 1 招、0 件道具 | 3 / 9 |
| 97 | `fn:enemy_team` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 3 / 9 |
| 98 | `fn:team_of` | 產生器 | hook 候選 | B12 | 1 招、0 件道具 | 4 / 9 |
| 99 | `action:end_channel` | ability_kit | 共用 | B09 | 3 招、0 件道具 | 4 / 9 |
| 100 | `name:event outside an event` | 產生器 | 共用 | B09 | 4 招、2 件道具 | 5 / 9 |
| 101 | `action:modify_base_stat` | ability_kit | hook 候選 | B11 | 1 招、0 件道具 | 5 / 9 |
| 102 | `attr:is_silenced` | 產生器 | hook 候選 | B11 | 1 招、0 件道具 | 6 / 9 |
| 103 | `action:issue_order.order:follow` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 6 / 9 |
| 104 | `status_trigger:on_refreshed` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 6 / 9 |
| 105 | `unit_template attack.projectile_speed 1300 (the host fires at 900)` | ability_kit | hook 候選 | A04 | 1 招、0 件道具 | 7 / 9 |
| 106 | `action:create_unit.overrides.ability_level` | ability_kit | hook 候選 | A28 | 1 招、0 件道具 | 7 / 9 |
| 107 | `stat:physical_block_chance` | ability_kit | 共用 | A28 | 1 招、2 件道具 | 7 / 9 |
| 108 | `unit_template abilities` | ability_kit | 共用 | A28 | 2 招、0 件道具 | 7 / 9 |
| 109 | `unit_template stats.mana_regen` | ability_kit | 共用 | A28 | 2 招、0 件道具 | 8 / 9 |
| 110 | `action:reveal.teams:enemy` | ability_kit | 共用 | B23 | 2 招、0 件道具 | 8 / 9 |
| 111 | `passive_trigger:on_moved` | ability_kit | hook 候選 | B23 | 1 招、0 件道具 | 8 / 9 |
| 112 | `selector.flag:include_dead` | 產生器 | 共用 | B23 | 2 招、0 件道具 | 8 / 9 |
| 113 | `unit_template attack.projectile_speed 1400 (the host fires at 900)` | ability_kit | 共用 | B23 | 2 招、0 件道具 | 9 / 9 |

### M6 第 9 批（13 位）

開始時已全部產生：0 位；做完本批的項目後：13 位。

| # | 要支援的寫法 | 主要負責 | 分類 | 為了 | 用到的招式數（全量） | 本批累計解鎖 |
|---|---|---|---|---|---|---|
| 114 | `action:chain` | ability_kit | hook 候選 | B28 | 1 招、0 件道具 | 0 / 13 |
| 115 | `fn:has_state:revealed` | 產生器 | hook 候選 | B28 | 1 招、0 件道具 | 1 / 13 |
| 116 | `stat:heal_reduction_pct` | ability_kit | 共用 | B33 | 2 招、2 件道具 | 1 / 13 |
| 117 | `token:vars` | 產生器 | 共用 | B33 | 1 招、2 件道具 | 2 / 13 |
| 118 | `selector.relative_to` | 產生器 | 共用 | C07 | 2 招、0 件道具 | 3 / 13 |
| 119 | `fn:orb_slot_free` | 產生器 | 共用 | B15 | 2 招、0 件道具 | 3 / 13 |
| 120 | `passive:orb_slot` | ability_kit | 共用 | B15 | 1 招、1 件道具 | 4 / 13 |
| 121 | `fn:landmark` | 產生器 | hook 候選 | A21 | 1 招、0 件道具 | 4 / 13 |
| 122 | `status.forced_move` | ability_kit | 共用 | A21 | 2 招、0 件道具 | 5 / 13 |
| 123 | `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | A11 | 2 招、2 件道具 | 5 / 13 |
| 124 | `attr:cast_point of an ability` | 產生器 | hook 候選 | A11 | 1 招、0 件道具 | 6 / 13 |
| 125 | `action:add_ability` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 6 / 13 |
| 126 | `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | B16 | 1 招、0 件道具 | 6 / 13 |
| 127 | `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 6 / 13 |
| 128 | `action:set_base_max_hp` | ability_kit | 共用 | B16 | 2 招、0 件道具 | 6 / 13 |
| 129 | `token:state` | 產生器 | hook 候選 | B16 | 1 招、0 件道具 | 7 / 13 |
| 130 | `action:modify_cooldown.item` | ability_kit | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 131 | `action:pick_item` | ability_kit | 共用 | C18 | 3 招、0 件道具 | 7 / 13 |
| 132 | `fn:ability_ready` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 133 | `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 134 | `fn:status_source` | 產生器 | hook 候選 | C18 | 1 招、0 件道具 | 7 / 13 |
| 135 | `selector.flag:include_illusions` | 產生器 | 共用 | C18 | 5 招、1 件道具 | 8 / 13 |
| 136 | `action:give_item` | ability_kit | 共用 | C06 | 3 招、0 件道具 | 8 / 13 |
| 137 | `action:remove_item` | ability_kit | 共用 | C06 | 2 招、0 件道具 | 8 / 13 |
| 138 | `attr:hero` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 139 | `attr:name` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 140 | `fn:exists of a sym:stolen` | 產生器 | hook 候選 | C06 | 1 招、0 件道具 | 8 / 13 |
| 141 | `state type:item_name` | ability_kit | hook 候選 | C06 | 1 招、0 件道具 | 9 / 13 |
| 142 | `action:create_unit.overrides.armor` | ability_kit | 共用 | B07 | 1 招、4 件道具 | 9 / 13 |
| 143 | `fn:exists of a sym:item_a` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 9 / 13 |
| 144 | `fn:exists of a sym:item_b` | 產生器 | hook 候選 | B07 | 1 招、0 件道具 | 9 / 13 |
| 145 | `unit_template acquisition_range 800 differs from attack.range` | ability_kit | hook 候選 | B07 | 1 招、0 件道具 | 9 / 13 |
| 146 | `unit_template inventory_locked` | ability_kit | hook 候選 | B07 | 1 招、0 件道具 | 9 / 13 |
| 147 | `unit_template item_cast_blacklist` | ability_kit | hook 候選 | B07 | 1 招、0 件道具 | 10 / 13 |
| 148 | `action:remove_ability` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 10 / 13 |
| 149 | `action:set_ability_active (an ability id and a fixed active)` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 10 / 13 |
| 150 | `expr of type nil` | 產生器 | 共用 | C08 | 2 招、0 件道具 | 10 / 13 |
| 151 | `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 10 / 13 |
| 152 | `selector.shape:segment` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 10 / 13 |
| 153 | `selector.to` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 10 / 13 |
| 154 | `selector.width` | 產生器 | hook 候選 | C08 | 1 招、0 件道具 | 10 / 13 |
| 155 | `state:blind` | ability_kit | hook 候選 | C08 | 1 招、0 件道具 | 11 / 13 |
| 156 | `action:projectile.team:any` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 157 | `action:projectile.where` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 158 | `action:redirect_motion` | ability_kit | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 159 | `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | B24 | 1 招、0 件道具 | 11 / 13 |
| 160 | `selector.types:building` | 產生器 | 共用 | B24 | 2 招、0 件道具 | 11 / 13 |
| 161 | `unit_template blocks_pathing on a dummy` | ability_kit | 共用 | B24 | 2 招、0 件道具 | 11 / 13 |
| 162 | `unit_template collision_radius on a dummy` | ability_kit | 共用 | B24 | 3 招、0 件道具 | 11 / 13 |
| 163 | `unit_template tag:building` | ability_kit | 共用 | B24 | 2 招、0 件道具 | 12 / 13 |
| 164 | `action:destroy_tree` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 12 / 13 |
| 165 | `action:regrow_trees` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 12 / 13 |
| 166 | `fn:tree_alive` | 產生器 | hook 候選 | B22 | 1 招、0 件道具 | 12 / 13 |
| 167 | `status.immune_to` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 12 / 13 |
| 168 | `target.type:tree` | 產生器 | 共用 | B22 | 2 招、0 件道具 | 12 / 13 |
| 169 | `unit_template attack.projectile_speed 600 (the host fires at 900)` | ability_kit | hook 候選 | B22 | 1 招、0 件道具 | 13 / 13 |

## 2. 每位英雄、每招還缺什麼

### A02 `inahime`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a02_venom_arrow` | ✓ 已產生 |
| `a02_wind_prayer` | ✓ 已產生 |
| `a02_wind_drift` | ✓ 已產生 |
| `a02_shadow_meld` | ✓ 已產生 |

### A07 `honda_tadakatsu`（5 / 6 招）　已上架

| 招式 | 還缺 |
|---|---|
| `a07_unscathed` | ✓ 已產生 |
| `a07_red_thunder` | ✓ 已產生 |
| `a07_shattering_spear` | ✓ 已產生 |
| `a07_strongest` | `stat:status_resistance_pct` |
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

### B04 `date_masamune`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b04_flame_trail_dash` | ✓ 已產生 |
| `b04_fire_blast` | ✓ 已產生 |
| `b04_flame_swiftness` | ✓ 已產生 |
| `b04_tanegashima` | ✓ 已產生 |

### B06 `sanada_yukimura`（3 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b06_brave_warriors` | ✓ 已產生 |
| `b06_flame_fangs` | ✓ 已產生 |
| `b06_mind_eye` | `passive_trigger:on_order_targeted` |
| `b06_first_warrior` | ✓ 已產生 |

### B08 `asai_nagamasa`（5 / 5 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b08_fang_slash` | ✓ 已產生 |
| `b08_flash_cut` | ✓ 已產生 |
| `b08_living_soul` | ✓ 已產生 |
| `b08_sky_fang` | ✓ 已產生 |
| `b08_endless_fang` | ✓ 已產生 |

### B29 `murakami_yoshikiyo`（4 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `b29_ground_slam` | ✓ 已產生 |
| `b29_quake_axe` | ✓ 已產生 |
| `b29_morale_boost` | ✓ 已產生 |
| `b29_earth_tremor` | ✓ 已產生 |

### C15 `tamako`（3 / 4 招）　已上架

| 招式 | 還缺 |
|---|---|
| `c15_scatter_shot` | ✓ 已產生 |
| `c15_burst_arrow` | ✓ 已產生 |
| `c15_precise_light` | ✓ 已產生 |
| `c15_hundred_arrows` | `action:issue_order.queue`、`unit_template acquisition_range is `$acquisition` has no value`、`unit_template attack range or interval missing`、`unit_template attack.interval is `$attack_interval` has no value`、`unit_template flying`、`unit_template passive.states`、`unit_template stats.hp_regen`、`unit_template stats.move_speed is `$move_speed` has no value`、`unit_template stats.move_speed missing`、`unit_template turn_rate` |

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
| `a03_spider_form` | `attr:bonus_armor` |
| `a03_enchanting_venom` | ✓ 已產生 |
| `a03_insect_swarm` | ✓ 已產生 |

### A04 `takenaka_shigeharu`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a04_water_shikigami` | `action:issue_order.order:follow`、`unit_template acquisition_range 1000 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.projectile_speed 1300 (the host fires at 900)`、`unit_template bounty.gold (summons give none)`、`unit_template stats.hp_regen`、`unit_template stats.mana`、`unit_template turn_rate`、`unit_template vision.day differs from vision.night` |
| `a04_ice_path` | ✓ 已產生 |
| `a04_frost_shelter` | `status.absorb.on_absorb`、`status.absorb.on_broken`、`status_trigger:on_refreshed` |
| `a04_frost_sparrow` | ✓ 已產生 |
| `a04_blizzard` | ✓ 已產生 |

### A05 `sakai_tadatsugu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a05_valiant_power` | ✓ 已產生 |
| `a05_mikawa_courage` | ✓ 已產生 |
| `a05_counter_stance` | ✓ 已產生 |
| `a05_loyal_secret` | ✓ 已產生 |
| `a05_crusade_affidavit` | ✓ 已產生 |

### A06 `ii_naomasa`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a06_red_power` | ✓ 已產生 |
| `a06_red_meteor` | ✓ 已產生 |
| `a06_red_flame_flow` | `aura.flag:include_illusions` |
| `a06_flame_of_anger` | ✓ 已產生 |
| `a06_red_fury` | `action:remove_status of its own independent status (this instance waits)`、`status.vars of an independent status`、`status_trigger:on_attack_miss` |

### A08 `sakakibara_yasumasa`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a08_skull_crusher` | ✓ 已產生 |
| `a08_war_cry` | ✓ 已產生 |
| `a08_indomitable` | ✓ 已產生 |
| `a08_steadfast_general` | `routine a08t_reduce: action:modify_damage outside a damage-pre trigger`、`routine a08t_reduce: fn:count_in_radius` |

### A09 `hachisuka_masakatsu`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a09_treant_summon` | `state:not_on_minimap`、`unit_template acquisition_range 1000 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.projectile_speed 1500 (the host fires at 900)`、`unit_template bounty.gold (summons give none)` |
| `a09_lignify` | ✓ 已產生 |
| `a09_forest_pulse` | ✓ 已產生 |
| `a09_deep_tentacles` | `unit_template acquisition_range 1000 differs from attack.range`、`unit_template bounty.gold (summons give none)`、`unit_template passive.triggers` |

### A10 `hirate_masahide`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a10_blink_return` | `action:cancel_delay`、`action:delay.id` |
| `a10_phase_shift` | `action:cancel_delay`、`action:delay.id` |
| `a10_slaughter_array` | `event:point` |
| `a10_swap` | ✓ 已產生 |
| `a10_shockwave` | `unit_template passive.aura on a dummy` |

### A11 `saito_dosan`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a11_vortex` | `action:damage.flags:no_source_bonuses` |
| `a11_evil_gaze` | `status.forced_move` |
| `a11_old_fox` | `attr:cast_point of an ability`、`aura.flag:include_illusions`、`name:event outside an event` |
| `a11_myriad_forms` | ✓ 已產生 |

### A12 `otani_yoshitsugu`（9 / 9 招）

| 招式 | 還缺 |
|---|---|
| `a12_insight_flash` | ✓ 已產生 |
| `a12_scheme` | ✓ 已產生 |
| `a12_mystic_pierce` | ✓ 已產生 |
| `a12_awe` | ✓ 已產生 |
| `a12_mystic_arrow` | ✓ 已產生 |
| `a12_mystic_possession` | ✓ 已產生 |
| `a12_insight_flash_t` | ✓ 已產生 |
| `a12_mystic_field` | ✓ 已產生 |
| `a12_awe_t` | ✓ 已產生 |

### A13 `hattori_hanzo`（4 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a13_vanishing_step` | ✓ 已產生 |
| `a13_shadow_illusion` | `action:create_illusion`、`action:cue.at of a sym:il`、`action:dodge_projectiles.of:il`、`expr: wanted a unit, got a sym:il`、`fn:exists of a sym:il` |
| `a13_substitution` | ✓ 已產生 |
| `a13_chain_sickle` | ✓ 已產生 |
| `a13_iga_ninjutsu` | ✓ 已產生 |
| `a13_shadow_slash` | `action:attack.effect_multiplier`、`fn:avg_attack_against` |

### A14 `maeda_toshiie`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a14_impact_spear` | `action:knockback.contact_radius`、`action:knockback.on_end`、`action:knockback.stop_on_contact` |
| `a14_divine_strike` | ✓ 已產生 |
| `a14_stacked_spear` | ✓ 已產生 |
| `a14_spear_fury` | ✓ 已產生 |

### A15 `nene`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a15_shadow_step` | ✓ 已產生 |
| `a15_exhaust_seal` | ✓ 已產生 |
| `a15_moon_strike` | ✓ 已產生 |
| `a15_shadow_windmill` | `attr:bonus_armor` |
| `a15_shadow_moon_assault` | `attr:bonus_armor`、`fn:avg_attack_against` |

### A16 `torii_mototada`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a16_bloody_grasp` | ✓ 已產生 |
| `a16_blood_link` | ✓ 已產生 |
| `a16_blood_aura` | ✓ 已產生 |
| `a16_dark_blood_decree` | ✓ 已產生 |

### A17 `sassa_narimasa`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a17_shrapnel` | ✓ 已產生 |
| `a17_rapid_fire` | ✓ 已產生 |
| `a17_vital_shot` | `action:chance.mode` |
| `a17_great_cannon` | ✓ 已產生 |

### A18 `inaba_ittetsu`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a18_obsessive_pressure` | ✓ 已產生 |
| `a18_suppress` | ✓ 已產生 |
| `a18_stubborn_tactics` | ✓ 已產生 |
| `a18_by_force` | ✓ 已產生 |

### A19 `kuroda_yoshitaka`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a19_fire_stratagem` | ✓ 已產生 |
| `a19_chain_fire` | ✓ 已產生 |
| `a19_fire_echo` | `event:point` |
| `a19_siege_fire` | ✓ 已產生 |

### A20 `kuki_yoshitaka`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a20_pirate_domain` | `action:cancel_delay`、`action:create_unit.limit_per_owner (no host cap for it)`、`action:delay.id`、`attr:owner`、`state:not_on_minimap`、`unit_template stats on a dummy` |
| `a20_water_burst` | `status.stacking:independent with slow_pct` |
| `a20_pirate_boots` | ✓ 已產生 |
| `a20_pirate_daimyo` | `stat:status_resistance_pct` |

### A21 `zuiryuin_nisshu`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a21_mana_steal` | ✓ 已產生 |
| `a21_half_blade` | `fn:orb_slot_free` |
| `a21_energy_press` | ✓ 已產生 |
| `a21_pearl_disc` | `action:reveal.teams:enemy`、`unit_template ai:owner_controlled`、`unit_template bounty.gold (summons give none)`、`unit_template stats.hp_regen`、`unit_template stats.mana`、`unit_template stats.mana_regen` |
| `a21_false_intel` | `fn:landmark`、`status.forced_move` |

### A22 `sengoku_hidehisa`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a22_flame_whirl` | ✓ 已產生 |
| `a22_flame_heart` | ✓ 已產生 |
| `a22_boiling_blood` | ✓ 已產生 |
| `a22_torment` | `status.reapply_if_removed` |

### A23 `ukita_hideie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a23_cursed_blood` | ✓ 已產生 |
| `a23_soul_bind` | ✓ 已產生 |
| `a23_falling_fire` | ✓ 已產生 |
| `a23_yomi_flame` | ✓ 已產生 |

### A24 `sakuma_morimasa`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a24_blood_shield` | `routine a24d_sync: status.absorb.on_absorb`、`routine a24d_sync: status.absorb.on_broken` |
| `a24_slaughter_shot` | `routine a24d_sync: status.absorb.on_absorb`、`routine a24d_sync: status.absorb.on_broken` |
| `a24_blood_charge` | `routine a24d_sync: status.absorb.on_absorb`、`routine a24d_sync: status.absorb.on_broken` |
| `a24_blood_resonance` | `routine a24d_sync: status.absorb.on_absorb`、`routine a24d_sync: status.absorb.on_broken` |
| `a24_bloodbath` | `routine a24d_sync: status.absorb.on_absorb`、`routine a24d_sync: status.absorb.on_broken` |

### A25 `oda_nobunaga`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a25_lightning_lure` | `fn:avg_attack_against` |
| `a25_calamity_thorns` | ✓ 已產生 |
| `a25_heshikiri` | ✓ 已產生 |
| `a25_crimson_barrier` | ✓ 已產生 |
| `a25_tenka_fubu` | ✓ 已產生 |

### A26 `nohime`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a26_volley` | ✓ 已產生 |
| `a26_powder_kick` | ✓ 已產生 |
| `a26_twin_guns` | ✓ 已產生 |
| `a26_scorching_bomb` | ✓ 已產生 |

### A27 `shibata_katsuie`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a27_unstoppable_advance` | `action:create_illusion` |
| `a27_war_banner` | `unit_template bounty on a dummy`、`unit_template stats on a dummy`、`unit_template statuses on a dummy` |
| `a27_way_of_warrior` | ✓ 已產生 |
| `a27_tyrant_hammer` | ✓ 已產生 |
| `a27_oni_shibata` | ✓ 已產生 |

### A28 `hashiba_hideyoshi`（4 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a28_evil_art` | `stat:physical_block_chance` |
| `a28_creation` | ✓ 已產生 |
| `a28_binding_swamp` | ✓ 已產生 |
| `a28_phoenix_summon` | `action:create_unit.overrides.ability_level`、`unit_template abilities`、`unit_template ai:owner_controlled`、`unit_template bounty.gold (summons give none)`、`unit_template stats.hp_regen`、`unit_template stats.mana`、`unit_template stats.mana_regen`、`unit_template turn_rate`、`unit_template vision.day differs from vision.night` |
| `a28_heaven_fire` | ✓ 已產生 |
| `a28_spreading_fire` | ✓ 已產生 |

### A29 `ishida_mitsunari`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a29_sleeve_poison_arrow` | ✓ 已產生 |
| `a29_abyss_grip` | ✓ 已產生 |
| `a29_withering_aura` | `status_trigger:on_mana_spent` |
| `a29_vulture_swarm` | `action:heal.source:owner_of(self)`、`unit_template ai:chase_target`、`unit_template chase`、`unit_template flying`、`unit_template no attack (the dummy unit type, line B)`、`unit_template stats.hp missing`、`unit_template stats.move_speed missing`、`unit_template statuses` |

### A30 `niwa_nagahide`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a30_wolf_mark` | `state:provides_vision`、`unit_template acquisition_range 300 differs from attack.range`、`unit_template ai:idle`、`unit_template stats.hp_regen`、`unit_template stats.mana`、`unit_template vision.day differs from vision.night` |
| `a30_earth_fang` | ✓ 已產生 |
| `a30_fire_sea` | ✓ 已產生 |
| `a30_red_fuji` | ✓ 已產生 |

### A31 `mori_ranmaru`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a31_echoing_rain` | ✓ 已產生 |
| `a31_shunpo` | ✓ 已產生 |
| `a31_storm_arrow` | ✓ 已產生 |
| `a31_samidare` | `action:attack without instant (a projectile attack)` |

### A32 `takigawa_kazumasu`（5 / 6 招）

| 招式 | 還缺 |
|---|---|
| `a32_flying_slash` | ✓ 已產生 |
| `a32_second_slash` | ✓ 已產生 |
| `a32_soul_drain` | ✓ 已產生 |
| `a32_phantom_blade` | ✓ 已產生 |
| `a32_soul_blade` | `fn:avg_attack_against` |
| `a32_asura_blade` | ✓ 已產生 |

### A33 `honda_masanobu`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a33_dark_summoning` | `unit_template acquisition_range 600 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template bounty.gold (summons give none)`、`unit_template stats.mana` |
| `a33_ghost_step` | ✓ 已產生 |
| `a33_corrupting_aura` | ✓ 已產生 |
| `a33_deepen_wounds` | `action:refund`、`state:provides_vision`、`state:revealed` |

### A35 `shima_sakon`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `a35_flame_raid` | ✓ 已產生 |
| `a35_sealing_blaze` | ✓ 已產生 |
| `a35_flame_swiftness` | ✓ 已產生 |
| `a35_scorching_fire` | ✓ 已產生 |

### A36 `maeno_nagayasu`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `a36_shadow_step` | ✓ 已產生 |
| `a36_spike_trap` | `action:create_unit.limit_per_owner (no host cap for it)`、`unit_template abilities on a dummy`、`unit_template stats on a dummy` |
| `a36_detonate` | ✓ 已產生 |
| `a36_bitter_brew` | ✓ 已產生 |
| `a36_deadly_strike` | ✓ 已產生 |

### B02 `fuma_kotaro`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b02_vanishing_step` | ✓ 已產生 |
| `b02_hazy_shadow` | `action:create_illusion`、`expr: a position from a sym:ill`、`expr: wanted a unit, got a sym:ill` |
| `b02_blazing_path` | ✓ 已產生 |
| `b02_eighteen_shuriken` | ✓ 已產生 |
| `b02_four_thunders` | ✓ 已產生 |

### B03 `saito_tomonobu`（5 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b03_zhongkui_strike` | ✓ 已產生 |
| `b03_zhongkui_strike_mid` | ✓ 已產生 |
| `b03_zhongkui_strike_far` | ✓ 已產生 |
| `b03_echigo_vajra` | ✓ 已產生 |
| `b03_heavy_blow` | ✓ 已產生 |
| `b03_zhongkui_judgment` | `selector.flag:exclude_invisible` |

### B05 `mori_motonari`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b05_chain_lightning` | `action:cue.at of a err`、`status.vars of an independent status` |
| `b05_heavenly_thunder` | ✓ 已產生 |
| `b05_three_arrows` | `state:revealed` |
| `b05_holy_light_burst` | ✓ 已產生 |

### B07 `ashikaga_yoshiaki`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b07_starfall` | ✓ 已產生 |
| `b07_battle_line` | `action:create_unit.limit_per_owner (no host cap for it)`、`action:create_unit.overrides.armor`、`action:give_item`、`action:pick_item`、`fn:exists of a sym:item_a`、`fn:exists of a sym:item_b`、`state:revealed`、`unit_template abilities`、`unit_template acquisition_range 800 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.projectile_speed 1400 (the host fires at 900)`、`unit_template bounty.gold (summons give none)`、`unit_template bounty.xp (summons give none)`、`unit_template inventory_locked`、`unit_template item_cast_blacklist`、`unit_template stats.mana`、`unit_template turn_rate`、`unit_template vision.day differs from vision.night` |
| `b07e_camp_regen` | ✓ 已產生 |
| `b07_glorious_momentum` | ✓ 已產生 |
| `b07_reign_over_all` | ✓ 已產生 |

### B09 `sanada_masayuki`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b09_backlash_rift` | `name:event outside an event` |
| `b09_mind_burst` | `routine b09e_pulse: action:end_channel` |
| `b09_will_shield` | ✓ 已產生 |
| `b09_mind_suppression` | ✓ 已產生 |

### B10 `miyoshi_nagayoshi`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b10_wrath_of_the_dead` | `unit_template ai:chase_target`、`unit_template chase`、`unit_template flying`、`unit_template no attack (the dummy unit type, line B)`、`unit_template stats.hp missing`、`unit_template stats.move_speed missing`、`unit_template statuses` |
| `b10_corpse_assault` | ✓ 已產生 |
| `b10_demonic_war_spirit` | ✓ 已產生 |
| `b10_power_of_the_abyss` | ✓ 已產生 |

### B11 `kikuhime`（2 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b11_cherry_rain` | ✓ 已產生 |
| `b11_breath_control` | `action:end_channel` |
| `b11_interception` | `routine b11r_spawn: action:create_illusion`、`routine b11r_spawn: expr: wanted a unit, got a sym:ill` |
| `b11_crow_raid` | `routine b11r_spawn: action:create_illusion`、`routine b11r_spawn: expr: wanted a unit, got a sym:ill` |
| `b11_crow_strike` | ✓ 已產生 |
| `b11_inherited_blood` | `action:modify_base_stat`、`attr:is_silenced` |

### B12 `yamamoto_kansuke`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b12_reality_slide` | `fn:enemy_team`、`fn:team_of` |
| `b12_floating_life` | ✓ 已產生 |
| `b12_scheming_offense` | ✓ 已產生 |
| `b12_mad_heart` | ✓ 已產生 |

### B13 `momochi_sandayu`（6 / 6 招）

| 招式 | 還缺 |
|---|---|
| `b13_sense_break` | ✓ 已產生 |
| `b13_hypnosis` | ✓ 已產生 |
| `b13_blood_disguise` | ✓ 已產生 |
| `b13_burrow` | ✓ 已產生 |
| `b13_shukuchi` | ✓ 已產生 |
| `b13_storm` | ✓ 已產生 |

### B14 `katakura_kagetsuna`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b14_temporary_night` | `action:set_time_of_day` |
| `b14_iai_slash` | ✓ 已產生 |
| `b14_shadow_tactics` | ✓ 已產生 |
| `b14_attendant_insight` | ✓ 已產生 |
| `b14_shadow_cut` | `action:cancel_delay`、`action:delay.id` |

### B15 `imagawa_yoshimoto`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b15_tokai_giant` | ✓ 已產生 |
| `b15_flame_arrow` | `fn:orb_slot_free`、`passive:orb_slot` |
| `b15_giant_hide` | ✓ 已產生 |
| `b15_multishot` | `fn:avg_attack_against` |
| `b15_bluff` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill`、`status.stacking:independent with states` |

### B16 `mochizuki_chiyome`（3 / 9 招）

| 招式 | 還缺 |
|---|---|
| `b16_flying_raid` | `action:dodge_projectiles.of:state.b16_moon` |
| `b16_falling_leaves` | ✓ 已產生 |
| `b16_swift_thunder` | `routine b16_adjust: action:set_base_max_hp` |
| `b16_summon_hound` | `action:add_ability`、`unit_template acquisition_range 1000 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template bounty.gold (summons give none)`、`unit_template bounty.xp (summons give none)`、`unit_template stats.hp_regen`、`unit_template stats.mana`、`unit_template vision.day differs from vision.night` |
| `b16_phase_shift` | `action:dodge_projectiles.of:state.b16_moon` |
| `b16_beast_union` | `action:refund`、`action:set_base_max_hp`、`token:state` |
| `b16_mm_phase_shift` | `action:dodge_projectiles.of:caster_hero` |
| `b16_mm_sweep` | ✓ 已產生 |
| `b16_mm_stealth` | ✓ 已產生 |

### B17 `kirigakure_saizo`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b17_death_ward` | `aura.flag:exclude_invisible`、`aura.status:independent`、`event.damage.is_attack`、`unit_template acquisition_range 1200 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.projectile_speed 1500 (the host fires at 900)`、`unit_template bounty.gold (summons give none)`、`unit_template stats.hp_regen`、`unit_template tag:ward` |
| `b17_tooth_poison` | ✓ 已產生 |
| `b17_shadow_leap` | ✓ 已產生 |
| `b17_heartseeker` | ✓ 已產生 |

### B18 `koihime`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b18_earth_raid` | ✓ 已產生 |
| `b18_echo` | ✓ 已產生 |
| `b18_earth_shelter` | ✓ 已產生 |
| `b18_earth_claw` | `selector.types:building+creep+neutral+summon`、`state:revealed`、`unit_template acquisition_range 1000 differs from attack.range`、`unit_template ai:idle`、`unit_template bounty.gold (summons give none)`、`unit_template stats.hp_regen`、`unit_template stats.mana` |

### B19 `usami_sadamitsu`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b19_water_dragon` | ✓ 已產生 |
| `b19_ice_fence` | ✓ 已產生 |
| `b19_echigo_tactics` | ✓ 已產生 |
| `b19_kuruma_gakari` | `unit_template acquisition_range 600 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template bounty.gold (summons give none)`、`unit_template bounty.xp (summons give none)`、`unit_template stats.hp_regen`、`unit_template stats.mana`、`unit_template vision.day differs from vision.night` |

### B20 `yamagata_masakage`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b20_rock_rush` | ✓ 已產生 |
| `b20_ruin_wave` | ✓ 已產生 |
| `b20_tenacity` | ✓ 已產生 |
| `b20_red_charge` | ✓ 已產生 |

### B21 `baba_nobufusa`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b21_sky_wave` | ✓ 已產生 |
| `b21_killing_intent` | ✓ 已產生 |
| `b21_rebel_armor` | `event.damage.raw` |
| `b21_undying_demon` | `passive_trigger:on_heal_received` |

### B22 `taigen_sessai`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b22_natural_barrier` | `unit_template blocks_pathing on a dummy`、`unit_template bounty on a dummy`、`unit_template collision_radius on a dummy`、`unit_template stats on a dummy`、`unit_template tag:building`、`unit_template vision.day differs from vision.night` |
| `b22_force_of_nature` | `action:destroy_tree`、`state:flying`、`target.type:tree`、`unit_template acquisition_range 1000 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.projectile_speed 600 (the host fires at 900)`、`unit_template bounty.gold (summons give none)`、`unit_template stats.hp missing`、`unit_template stats.hp_regen`、`unit_template turn_rate`、`unit_template vision.day differs from vision.night` |
| `b22_purify` | `status.immune_to` |
| `b22_wood_spirit` | `fn:tree_alive`、`state:revealed`、`target.type:tree`、`unit_template statuses on a dummy` |
| `b22_wrath_of_nature` | `action:regrow_trees`、`unit_template bounty on a dummy`、`unit_template stats on a dummy`、`unit_template statuses on a dummy`、`unit_template vision.day differs from vision.night` |

### B23 `ise_hime`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b23_undead_pact` | `passive_trigger:on_moved`、`selector.flag:include_dead`、`status_trigger:on_moved`、`unit_template acquisition_range 1200 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.projectile_speed 1400 (the host fires at 900)`、`unit_template bounty.gold (summons give none)`、`unit_template bounty.xp (summons give none)`、`unit_template stats.hp_regen`、`unit_template turn_rate` |
| `b23_spirit_gathering` | ✓ 已產生 |
| `b23_living_soul_shock` | ✓ 已產生 |
| `b23_soul_feast` | ✓ 已產生 |
| `b23_heroic_spirit_blessing` | `action:reveal.teams:enemy` |

### B24 `akiyama_nobutomo`（0 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b24_earth_stele` | `action:redirect_motion`、`selector.flag:include_illusions`、`selector.types:boss+creep+neutral+summon`、`selector.types:building`、`unit_template bounty on a dummy`、`unit_template collision_radius on a dummy`、`unit_template stats on a dummy`、`unit_template statuses on a dummy`、`unit_template vision.day differs from vision.night` |
| `b24_taunt` | `selector.flag:include_illusions` |
| `b24_stone_resonance` | `action:projectile.team:any`、`action:projectile.where`、`expr of type nil`、`selector.flag:include_illusions` |
| `b24_earth_prison` | `selector.flag:include_illusions`、`unit_template blocks_pathing on a dummy`、`unit_template bounty on a dummy`、`unit_template collision_radius on a dummy`、`unit_template stats on a dummy`、`unit_template statuses on a dummy`、`unit_template tag:building` |
| `b24_shatter` | `selector.types:building` |

### B25 `honganji_kennyo`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b25_exorcism_strike` | ✓ 已產生 |
| `b25_exorcism_sutra` | `attr:cooldown_remaining`、`fn:max of non-numbers` |
| `b25_finger_of_death` | ✓ 已產生 |
| `b25_asura_hell` | `action:impulse` |

### B26 `okuma_tomohide`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b26_charging_wave` | ✓ 已產生 |
| `b26_blood_fury` | ✓ 已產生 |
| `b26_kozuke_valor` | `event.damage.lethal` |
| `b26_unstoppable` | ✓ 已產生 |
| `b26_release_wave` | ✓ 已產生 |

### B27 `kojima_sadaoki`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b27_horsemanship` | ✓ 已產生 |
| `b27_mighty_strike` | ✓ 已產生 |
| `b27_fearless_charge` | ✓ 已產生 |
| `b27_rending_blade` | ✓ 已產生 |

### B28 `matsunaga_hisahide`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b28_phoenix_flame` | ✓ 已產生 |
| `b28_flame_shackle` | `action:end_channel`、`name:event outside an event` |
| `b28_fire_stream` | ✓ 已產生 |
| `b28_serpent_fire` | `action:chain`、`fn:has_state:revealed` |

### B30 `kakizaki_kageie`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b30_reverse_shield` | ✓ 已產生 |
| `b30_soul_breaker` | ✓ 已產生 |
| `b30_flash_strike` | ✓ 已產生 |
| `b30_purgatory_slash` | ✓ 已產生 |

### B31 `takeda_shingen`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b31_swift_wind` | ✓ 已產生 |
| `b31_still_forest` | ✓ 已產生 |
| `b31_raging_fire` | `status.per_source without {caster}` |
| `b31_immovable_mountain` | ✓ 已產生 |

### B32 `uesugi_kenshin`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b32_divine_slash` | ✓ 已產生 |
| `b32_divine_flash` | ✓ 已產生 |
| `b32_divine_formation` | ✓ 已產生 |
| `b32_war_god` | `state:provides_vision` |
| `b32_bishamonten` | ✓ 已產生 |

### B33 `mogami_yoshihime`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b33_corrosive_arrow` | ✓ 已產生 |
| `b33_spider_web` | ✓ 已產生 |
| `b33_voodoo_power` | ✓ 已產生 |
| `b33_spider_attack` | `stat:heal_reduction_pct`、`state:flying`、`token:vars`、`unit_template stats on a dummy`、`unit_template statuses on a dummy` |

### B34 `takeda_katsuyori`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b34_wrath_axe` | ✓ 已產生 |
| `b34_spike_wall` | `unit_template stats on a dummy`、`unit_template statuses on a dummy` |
| `b34_frenzy` | ✓ 已產生 |
| `b34_tenmoku_howl` | `status.stacking:independent with slow_pct` |

### B35 `kamiizumi_nobutsuna`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `b35_sword_essence` | ✓ 已產生 |
| `b35_isshun` | ✓ 已產生 |
| `b35_muto_dori` | ✓ 已產生 |
| `b35_hiten` | ✓ 已產生 |

### B36 `naoe_kanetsugu`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `b36_naoe_seal` | ✓ 已產生 |
| `b36_footsteps` | ✓ 已產生 |
| `b36_heart_fence` | `selector.flag:exclude_illusions`、`status.absorb.on_broken` |
| `b36_seven_spears` | ✓ 已產生 |
| `b36_aizen_myoo` | ✓ 已產生 |

### C01 `akechi_mitsuhide`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c01_blood_blade` | ✓ 已產生 |
| `c01_ground_flame` | `routine c01e_wave: status.stacking:independent with states` |
| `c01_blood_flame` | ✓ 已產生 |
| `c01_honnoji` | `status.stacking:independent with states` |

### C02 `akechi_hidemitsu`（1 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c02_mirage_strike` | `action:create_illusion`、`expr: a position from a sym:ill`、`expr: wanted a unit, got a sym:ill` |
| `c02_phantom_move` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill` |
| `c02_kyoka_suigetsu` | `action:create_illusion`、`expr: wanted a unit, got a sym:ill` |
| `c02_culling_slash` | ✓ 已產生 |

### C03 `shimazu_yoshihiro`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c03_iron_shirt` | ✓ 已產生 |
| `c03_quake_axe` | ✓ 已產生 |
| `c03_thunder_wrath` | `action:modify_attack without crit_multiplier`、`action:modify_attack.cannot_miss` |
| `c03_iron_hammer` | ✓ 已產生 |

### C04 `izumo_no_okuni`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c04_swift_dance` | ✓ 已產生 |
| `c04_withering_step` | ✓ 已產生 |
| `c04_gathering_dance` | ✓ 已產生 |
| `c04_witch_dance` | ✓ 已產生 |

### C05 `maeda_keiji`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c05_matsukaze` | ✓ 已產生 |
| `c05_thunder_dance` | ✓ 已產生 |
| `c05_thunder_spear` | ✓ 已產生 |
| `c05_flash_thunder` | ✓ 已產生 |

### C06 `ishikawa_goemon`（3 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c06_quick_hands` | ✓ 已產生 |
| `c06_sleight_of_hand` | `action:give_item`、`action:pick_item`、`action:remove_item`、`attr:name`、`fn:exists of a sym:stolen` |
| `c06_lavish_strike` | ✓ 已產生 |
| `c06_murder_and_loot` | `attr:hero` |
| `c06_grand_heist` | ✓ 已產生 |
| `item_c06e` | `action:give_item`、`action:remove_item`、`state type:item_name` |

### C07 `tachibana_dosetsu`（3 / 6 招）

| 招式 | 還缺 |
|---|---|
| `c07_heaven_thunder` | ✓ 已產生 |
| `c07_thunder_shackle` | `name:event outside an event` |
| `c07_thunderbolt` | `selector.relative_to` |
| `c07_raijin_wrath` | `stat:heal_reduction_pct`、`unit_template stats on a dummy` |
| `c07_storm_move` | ✓ 已產生 |
| `c07_magic_shield` | ✓ 已產生 |

### C08 `kato_danzo`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c08_vanishing_step` | ✓ 已產生 |
| `c08_blood_hunt` | `action:damage.flags:no_source_bonuses` |
| `c08_scatter_escape` | ✓ 已產生 |
| `c08_kite_chain` | `expr of type nil`、`selector.relative_to`、`selector.shape:segment`、`selector.to`、`selector.width` |
| `c08_devour` | `action:add_ability`、`action:remove_ability`、`action:set_ability_active (an ability id and a fixed active)`、`fn:ability_in_slot`、`fn:is_ability of a non-event ability`、`state:blind` |

### C09 `tachibana_ginchiyo`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c09_frost_thorn` | `action:modify_attack without crit_multiplier`、`action:modify_attack.is_orb` |
| `c09_white_heat` | ✓ 已產生 |
| `c09_ice_soul_dance` | ✓ 已產生 |
| `c09_armor_break` | ✓ 已產生 |

### C10 `kosokabe_chikayasu`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c10_black_dragon_wave` | ✓ 已產生 |
| `c10_energy_drain` | ✓ 已產生 |
| `c10_echo_sword` | ✓ 已產生 |
| `c10_day_night_shift` | `action:set_time_of_day` |
| `c10_formless_heaven` | `stat.when` |

### C11 `tachibana_muneshige`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c11_raikiri` | ✓ 已產生 |
| `c11_thunder_speed` | ✓ 已產生 |
| `c11_thunder_eye` | ✓ 已產生 |
| `c11_unrivaled` | ✓ 已產生 |

### C12 `otomo_sorin`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c12_homing_lock` | `state:provides_vision`、`status_trigger:on_moved` |
| `c12_missile_launch` | ✓ 已產生 |
| `c12_space_rift` | ✓ 已產生 |
| `c12_optical_mask` | ✓ 已產生 |
| `c12_annihilation_beam` | ✓ 已產生 |

### C13 `ryuzoji_takanobu`（2 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c13_siphon` | ✓ 已產生 |
| `c13_tornado_wave` | ✓ 已產生 |
| `c13_gale_grip` | `event.damage.is_attack` |
| `c13_tornado_vortex` | `action:modify_cooldown.item_tag` |

### C14 `saito_yoshitatsu`（5 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c14_fire_curse` | ✓ 已產生 |
| `c14_fire_breath` | ✓ 已產生 |
| `c14_fire_breath_dragon` | ✓ 已產生 |
| `c14_dragon_vein` | ✓ 已產生 |
| `c14_dragon_form` | ✓ 已產生 |

### C16 `kashin_koji`（1 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c16_soul_sever` | ✓ 已產生 |
| `c16_curse_shadow` | `unit_template ai:owner_controlled`、`unit_template bounty.gold (summons give none)`、`unit_template no attack (the dummy unit type, line B)`、`unit_template stats.hp_regen` |
| `c16_wandering_souls` | `action:heal.source:owner_of(self)`、`state:flying`、`state:not_on_minimap`、`unit_template ai:chase_target`、`unit_template chase`、`unit_template flying`、`unit_template no attack (the dummy unit type, line B)`、`unit_template stats.hp missing`、`unit_template stats.move_speed missing` |
| `c16_phantom_sight` | `action:create_illusion`、`expr: a position from a sym:ill`、`expr: wanted a unit, got a sym:ill` |
| `c16_eight_gates` | `unit_template acquisition_range 1100 differs from attack.range`、`unit_template ai:owner_controlled`、`unit_template attack.damage_type:pure`、`unit_template attack.projectile_speed 2000 (the host fires at 900)`、`unit_template bounty.gold (summons give none)`、`unit_template bounty.xp (summons give none)`、`unit_template vision.day differs from vision.night` |

### C18 `kobayakawa_takakage`（2 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c18_lone_rider` | `action:modify_cooldown.item`、`action:pick_item`、`fn:exists of a sym:blink_item`、`fn:status_source`、`selector.flag:include_dead` |
| `c18_twin_chariot` | `fn:ability_ready` |
| `c18_three_blades` | ✓ 已產生 |
| `c18_chain_stratagem` | ✓ 已產生 |
| `c18_four_cannons` | `selector.flag:include_illusions` |

### C19 `matsuhime`（3 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c19_alluring_gaze` | ✓ 已產生 |
| `c19_gentle_vow` | ✓ 已產生 |
| `c19_paralyzing_poison` | `routine c19r_strike: fn:avg_attack_against` |
| `c19_surprise_raid` | `fn:count_in_radius` |
| `c19_inviolable` | ✓ 已產生 |

### C20 `chosokabe_motochika`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c20_flame_aegis` | ✓ 已產生 |
| `c20_demon_blood` | ✓ 已產生 |
| `c20_flame_dance` | ✓ 已產生 |
| `c20_demon_child` | ✓ 已產生 |

### C21 `miyamoto_musashi`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c21_crescent_slash` | ✓ 已產生 |
| `c21_chain_breaker` | ✓ 已產生 |
| `c21_two_heavens` | ✓ 已產生 |
| `c21_soaring_quake` | `fn:count_in_radius` |

### C22 `sasaki_kojiro`（4 / 5 招）

| 招式 | 還缺 |
|---|---|
| `c22_ganryu` | ✓ 已產生 |
| `c22_swallow_nest` | ✓ 已產生 |
| `c22_flying_swallow` | `status.stacking:independent with states` |
| `c22_swallow_flash` | ✓ 已產生 |
| `c22_dream_mist` | ✓ 已產生 |

### C23 `kato_kiyomasa`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c23_tainted_frenzy` | ✓ 已產生 |
| `c23_malice_wall` | `state:flying`、`state:not_on_minimap`、`unit_template acquisition_range 1000 differs from attack.range`、`unit_template stats.mana`、`unit_template turn_rate` |
| `c23_blood_for_blood` | ✓ 已產生 |
| `c23_gift_of_death` | ✓ 已產生 |

### C24 `yagyu_muneyoshi`（4 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c24_kamitori` | ✓ 已產生 |
| `c24_ukifune` | ✓ 已產生 |
| `c24_zenken` | ✓ 已產生 |
| `c24_last_life` | ✓ 已產生 |

### C25 `chacha`（3 / 4 招）

| 招式 | 還缺 |
|---|---|
| `c25_raid` | ✓ 已產生 |
| `c25_blood_blade` | ✓ 已產生 |
| `c25_grievous_wound` | `action:modify_cooldown.ability:se`、`action:modify_cooldown.ability:sq`、`action:modify_cooldown.ability:sw`、`fn:ability_in_slot` |
| `c25_falling_sakura` | ✓ 已產生 |

## 3. 已產生的招式在執行期回 RErr 的情況

產生的程式碼遇到資料沒預期的世界狀態時，施放回 `RErr`（host 回滾成本）。下表是已產生的招式裡各種情況出現的招數；未產生的招式是 stub，一律回 `RErr` 並列出 §2 的缺項。

| 情況 | 招數 |
|---|---|
| 施放檢查：rank、施法者存活、目標存活／隊伍／類別、施法距離（同 IR） | 257 |
| 召喚：附近放不下、lifetime 不在 0～600 秒 | 2 |
| 召喚：count 不是 1 | 1 |
| 視野：半徑或位置不合法 | 14 |
| 光鍊：host tether 規格不合法 | 1 |
| 友方回復：比例不在 0～1、或不在施法者周圍時沒有施法距離 | 1 |
| 運算式除以零 | 19 |
| 讀位置的單位已不在世界中 | 150 |
| 需要地點，但施放沒有地點也沒有單位目標 | 67 |

### 已產生但 host 沒有做到的地方

產生器不默默略過欄位：不支援的一律擋下（上面各節）；下面這些已產生，但文件寫的某件事 host 目前做不到，照原樣列出。

| 招式 | 沒有做到 |
|---|---|
| `c06_grand_heist` | create_pickup.cue: the host pickup draws its own bag |
| `b30_flash_strike` | issue_order: only a hero takes the order |
| `b35_hiten` | issue_order: only a hero takes the order |
| `b14_iai_slash` | issue_order: only a hero takes the order |
| `b14_shadow_tactics` | issue_order: only a hero takes the order |
| `c23_gift_of_death` | issue_order: only a hero takes the order |
| `c18_chain_stratagem` | issue_order: only a hero takes the order |
| `b27_mighty_strike` | issue_order: only a hero takes the order |
| `c14_fire_curse` | issue_order: only a hero takes the order |
| `c22_ganryu` | issue_order: only a hero takes the order |
| `a35_scorching_fire` | issue_order: only a hero takes the order |
| `c07_storm_move` | issue_order: only a hero takes the order |
| `c11_raikiri` | routine c11_bolt_finish: issue_order: only a hero takes the order |
| `c11_unrivaled` | routine c11_bolt_finish: issue_order: only a hero takes the order |
| `b32_divine_slash` | issue_order: only a hero takes the order |

## 4. 每項寫法擋住多少

| 寫法 | 主要負責 | 分類 | 招式 | 道具 | 英雄 |
|---|---|---|---|---|---|
| `unit_template bounty.gold (summons give none)` | ability_kit | 共用 | 15 | 0 | 13 |
| `unit_template ai:owner_controlled` | ability_kit | 共用 | 13 | 0 | 12 |
| `unit_template stats.hp_regen` | ability_kit | 共用 | 12 | 0 | 12 |
| `unit_template stats.mana` | ability_kit | 共用 | 10 | 0 | 10 |
| `unit_template vision.day differs from vision.night` | ability_kit | 共用 | 11 | 0 | 9 |
| `unit_template stats on a dummy` | ability_kit | 共用 | 10 | 0 | 8 |
| `action:create_illusion` | ability_kit | 共用 | 10 | 0 | 7 |
| `unit_template turn_rate` | ability_kit | 共用 | 7 | 0 | 7 |
| `unit_template acquisition_range 1000 differs from attack.range` | ability_kit | 共用 | 7 | 0 | 6 |
| `fn:avg_attack_against` | 產生器 | 共用 | 6 | 0 | 6 |
| `expr: wanted a unit, got a sym:ill` | 產生器 | 共用 | 8 | 0 | 5 |
| `unit_template statuses on a dummy` | ability_kit | 共用 | 7 | 0 | 5 |
| `state:revealed` | ability_kit | 共用 | 5 | 8 | 5 |
| `unit_template bounty.xp (summons give none)` | ability_kit | 共用 | 5 | 0 | 5 |
| `name:event outside an event` | 產生器 | 共用 | 4 | 2 | 4 |
| `state:flying` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:not_on_minimap` | ability_kit | 共用 | 4 | 0 | 4 |
| `state:provides_vision` | ability_kit | 共用 | 4 | 2 | 4 |
| `unit_template flying` | ability_kit | 共用 | 4 | 0 | 4 |
| `unit_template stats.hp missing` | ability_kit | 共用 | 4 | 0 | 4 |
| `unit_template stats.move_speed missing` | ability_kit | 共用 | 4 | 0 | 4 |
| `status.absorb.on_broken` | ability_kit | 共用 | 7 | 0 | 3 |
| `unit_template bounty on a dummy` | ability_kit | 共用 | 5 | 0 | 3 |
| `action:cancel_delay` | ability_kit | 共用 | 4 | 0 | 3 |
| `action:delay.id` | ability_kit | 共用 | 4 | 0 | 3 |
| `status.stacking:independent with states` | ability_kit | 共用 | 4 | 1 | 3 |
| `unit_template no attack (the dummy unit type, line B)` | ability_kit | 共用 | 4 | 0 | 3 |
| `action:create_unit.limit_per_owner (no host cap for it)` | ability_kit | 共用 | 3 | 0 | 3 |
| `action:end_channel` | ability_kit | 共用 | 3 | 0 | 3 |
| `action:pick_item` | ability_kit | 共用 | 3 | 0 | 3 |
| `expr: a position from a sym:ill` | 產生器 | 共用 | 3 | 0 | 3 |
| `fn:count_in_radius` | 產生器 | 共用 | 3 | 1 | 3 |
| `status_trigger:on_moved` | ability_kit | 共用 | 3 | 0 | 3 |
| `unit_template ai:chase_target` | ability_kit | 共用 | 3 | 0 | 3 |
| `unit_template chase` | ability_kit | 共用 | 3 | 0 | 3 |
| `status.absorb.on_absorb` | ability_kit | 共用 | 6 | 1 | 2 |
| `selector.flag:include_illusions` | 產生器 | 共用 | 5 | 1 | 2 |
| `action:give_item` | ability_kit | 共用 | 3 | 0 | 2 |
| `attr:bonus_armor` | 產生器 | 共用 | 3 | 2 | 2 |
| `unit_template collision_radius on a dummy` | ability_kit | 共用 | 3 | 0 | 2 |
| `action:add_ability` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:damage.flags:no_source_bonuses` | ability_kit | 共用 | 2 | 2 | 2 |
| `action:heal.source:owner_of(self)` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:modify_attack without crit_multiplier` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:refund` | ability_kit | 共用 | 2 | 2 | 2 |
| `action:reveal.teams:enemy` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:set_time_of_day` | ability_kit | 共用 | 2 | 0 | 2 |
| `aura.flag:include_illusions` | ability_kit | 共用 | 2 | 0 | 2 |
| `event.damage.is_attack` | ability_kit | 共用 | 2 | 0 | 2 |
| `event:point` | ability_kit | 共用 | 2 | 0 | 2 |
| `expr of type nil` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:ability_in_slot` | 產生器 | 共用 | 2 | 0 | 2 |
| `fn:orb_slot_free` | 產生器 | 共用 | 2 | 0 | 2 |
| `selector.flag:include_dead` | 產生器 | 共用 | 2 | 0 | 2 |
| `selector.relative_to` | 產生器 | 共用 | 2 | 0 | 2 |
| `stat:heal_reduction_pct` | ability_kit | 共用 | 2 | 2 | 2 |
| `stat:status_resistance_pct` | ability_kit | 共用 | 2 | 5 | 2 |
| `status.forced_move` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.stacking:independent with slow_pct` | ability_kit | 共用 | 2 | 0 | 2 |
| `status.vars of an independent status` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template abilities` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template acquisition_range 1200 differs from attack.range` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template acquisition_range 600 differs from attack.range` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template ai:idle` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template attack.projectile_speed 1400 (the host fires at 900)` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template attack.projectile_speed 1500 (the host fires at 900)` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template blocks_pathing on a dummy` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template stats.mana_regen` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template statuses` | ability_kit | 共用 | 2 | 0 | 2 |
| `unit_template tag:building` | ability_kit | 共用 | 2 | 0 | 2 |
| `action:dodge_projectiles.of:state.b16_moon` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:remove_item` | ability_kit | 共用 | 2 | 0 | 1 |
| `action:set_base_max_hp` | ability_kit | 共用 | 2 | 0 | 1 |
| `selector.types:building` | 產生器 | 共用 | 2 | 0 | 1 |
| `target.type:tree` | 產生器 | 共用 | 2 | 0 | 1 |
| `action:attack without instant (a projectile attack)` | ability_kit | 共用 | 1 | 1 | 1 |
| `action:attack.effect_multiplier` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:chain` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:chance.mode` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:create_unit.overrides.ability_level` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:create_unit.overrides.armor` | ability_kit | 共用 | 1 | 4 | 1 |
| `action:cue.at of a err` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:cue.at of a sym:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:destroy_tree` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:dodge_projectiles.of:caster_hero` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:dodge_projectiles.of:il` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:impulse` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:issue_order.order:follow` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:issue_order.queue` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:knockback.contact_radius` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:knockback.on_end` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:knockback.stop_on_contact` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_attack.cannot_miss` | ability_kit | 共用 | 1 | 2 | 1 |
| `action:modify_attack.is_orb` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_base_stat` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.ability:se` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.ability:sq` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.ability:sw` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.item` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_cooldown.item_tag` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:modify_damage outside a damage-pre trigger` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.team:any` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:projectile.where` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:redirect_motion` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:regrow_trees` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_ability` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:remove_status of its own independent status (this instance waits)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:set_ability_active (an ability id and a fixed active)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `attr:cast_point of an ability` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:cooldown_remaining` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:hero` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:is_silenced` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:name` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `attr:owner` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `aura.flag:exclude_invisible` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `aura.status:independent` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `event.damage.lethal` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `event.damage.raw` | ability_kit | 共用 | 1 | 1 | 1 |
| `expr: wanted a unit, got a sym:il` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:ability_ready` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:enemy_team` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:blink_item` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:il` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:item_a` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:item_b` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:exists of a sym:stolen` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:has_state:revealed` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:is_ability of a non-event ability` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:landmark` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:max of non-numbers` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:status_source` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:team_of` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `fn:tree_alive` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `passive:orb_slot` | ability_kit | 共用 | 1 | 1 | 1 |
| `passive_trigger:on_heal_received` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `passive_trigger:on_moved` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `passive_trigger:on_order_targeted` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `selector.flag:exclude_illusions` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.flag:exclude_invisible` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.shape:segment` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.to` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:boss+creep+neutral+summon` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `selector.types:building+creep+neutral+summon` | 產生器 | 共用 | 1 | 1 | 1 |
| `selector.width` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `stat.when` | ability_kit | 共用 | 1 | 6 | 1 |
| `stat:physical_block_chance` | ability_kit | 共用 | 1 | 2 | 1 |
| `state type:item_name` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `state:blind` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.immune_to` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.per_source without {caster}` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status.reapply_if_removed` | ability_kit | 共用 | 1 | 14 | 1 |
| `status_trigger:on_attack_miss` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_mana_spent` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `status_trigger:on_refreshed` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `token:state` | 產生器 | hook 候選 | 1 | 0 | 1 |
| `token:vars` | 產生器 | 共用 | 1 | 2 | 1 |
| `unit_template abilities on a dummy` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template acquisition_range 1100 differs from attack.range` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template acquisition_range 300 differs from attack.range` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template acquisition_range 800 differs from attack.range` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template acquisition_range is `$acquisition` has no value` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template attack range or interval missing` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template attack.damage_type:pure` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template attack.interval is `$attack_interval` has no value` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template attack.projectile_speed 1300 (the host fires at 900)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template attack.projectile_speed 2000 (the host fires at 900)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template attack.projectile_speed 600 (the host fires at 900)` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template inventory_locked` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template item_cast_blacklist` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template passive.aura on a dummy` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template passive.states` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template passive.triggers` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template stats.move_speed is `$move_speed` has no value` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `unit_template tag:ward` | ability_kit | hook 候選 | 1 | 0 | 1 |
| `action:damage.type:event` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:issue_order.ability` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:issue_order.order:cast` | ability_kit | 共用 | 0 | 2 | 0 |
| `action:projectile.lifetime` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `action:remove_status.exclude_source_tags` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.min_duration` | ability_kit | 共用 | 0 | 3 | 0 |
| `action:remove_status.source_is` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `attr:game.creep_wave` | 產生器 | 共用 | 0 | 4 | 0 |
| `attr:illusion_damage` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `attr:is_ready` | 產生器 | 共用 | 0 | 3 | 0 |
| `attr:is_summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `aura.types:building+creep+neutral+summon` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `aura.types:building+hero` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `aura.unique_group` | ability_kit | 共用 | 0 | 4 | 0 |
| `catalog: ability ninja1_flash is not a content ability (templates/abilities.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: ability ninja2_burrow is not a content ability (templates/abilities.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ancient_doll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_big_tachi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_brave_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_flash_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_flash_shoes is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_flood_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ghost_gloves is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_gold_to_prestige is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_hachiman_bell is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ice_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_kat_steel is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_kisyosennfuyuzuki is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ksitigarbha is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_lesser_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_logs_big_tachi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_magic_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_mountain_scroll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_nannbann_armor is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_ninja_sword is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_poisonous_ring is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_sakabatou is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_samurai_cloth is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_shintogo_kunimitsu is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_storm_book is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_sukezane is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_club_of_nebula is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_great_sword_of_spike is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_red_lightning_chapter is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_red_lightning_chapter_hyper is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_the_sword_of_kaminaobi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_thunderstorms is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_wayumi is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_wayumi2 is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_wind_scroll is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `catalog: item item_yubashiri is not in the shop (templates/moba_items.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `event:on_owner_death` | ability_kit | 共用 | 0 | 3 | 0 |
| `event:on_unequip` | ability_kit | 共用 | 0 | 2 | 0 |
| `fn:has_item` | 產生器 | 共用 | 0 | 3 | 0 |
| `fn:has_tag:courier of a unit` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `fn:has_tag:neutral of a unit` | 產生器 | 共用 | 0 | 3 | 0 |
| `fn:has_tag:no_cast_trigger` | 產生器 | 共用 | 0 | 2 | 0 |
| `fn:is_hero_id` | 產生器 | 共用 | 0 | 7 | 0 |
| `fn:min of non-numbers` | 產生器 | 共用 | 0 | 2 | 0 |
| `item.active.cast.interruptible` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.cost.charges` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `item.active.kind:passive` | ability_kit | 共用 | 0 | 3 | 0 |
| `item.active.kind:toggle` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive:aura of an item` | ability_kit | 共用 | 0 | 22 | 0 |
| `passive_trigger:on_ability_executed of an item (no learn marker)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `passive_trigger:on_death of an item (no learn marker)` | ability_kit | 共用 | 0 | 2 | 0 |
| `passive_trigger:on_respawn of an item (no learn marker)` | ability_kit | 共用 | 0 | 2 | 0 |
| `rust_handler without behavior` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `selector.exclude_types` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:building+hero+tower` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `selector.types:creep+neutral+summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `stat:all_stats` | ability_kit | 共用 | 0 | 9 | 0 |
| `stat:cooldown_reduction_pct` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.absorb on this apply form` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.absorb.phase:pre_mitigation` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `status.scope:item_instance` | ability_kit | 共用 | 0 | 26 | 0 |
| `status.unique_rule:latest` | ability_kit | 共用 | 0 | 9 | 0 |
| `status_trigger:on_order` | ability_kit | 共用 | 0 | 2 | 0 |
| `status_trigger:on_order_targeted` | ability_kit | 共用 | 0 | 2 | 0 |
| `target.types:building+courier+creep+neutral+summon` | 產生器 | hook 候選 | 0 | 1 | 0 |
| `template:art_of_war_archer is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:art_of_war_infantry is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:earth_wall is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:forest_chapter_infantry is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:forest_infantry is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:healing_totem is not a content summon (templates/summons.lua)` | ability_kit | 共用 | 0 | 2 | 0 |
| `template:hurricane_field is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:lost_art_archer is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:lost_art_infantry is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:ninja_unit1 is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |
| `template:ninja_unit2 is not a content summon (templates/summons.lua)` | ability_kit | hook 候選 | 0 | 1 | 0 |

## 5. hook 候選

只有 1 招（英雄技能與道具合計）用到的寫法。這些招式在 v3 標 `hook = "<方法名>"`、寫明行為描述與預期結果，由產生的 trait 手寫實作（`src/hooks/<招式>.rs`）；第 2 招也用到時改成共用、刪掉 hook。

| 寫法 | 招式 | 所屬 |
|---|---|---|
| `status_trigger:on_refreshed` | `a04_frost_shelter` | A04 takenaka_shigeharu |
| `action:issue_order.order:follow` | `a04_water_shikigami` | A04 takenaka_shigeharu |
| `unit_template attack.projectile_speed 1300 (the host fires at 900)` | `a04_water_shikigami` | A04 takenaka_shigeharu |
| `action:remove_status of its own independent status (this instance waits)` | `a06_red_fury` | A06 ii_naomasa |
| `status_trigger:on_attack_miss` | `a06_red_fury` | A06 ii_naomasa |
| `action:modify_damage outside a damage-pre trigger` | `a08_steadfast_general` | A08 sakakibara_yasumasa |
| `unit_template passive.triggers` | `a09_deep_tentacles` | A09 hachisuka_masakatsu |
| `unit_template passive.aura on a dummy` | `a10_shockwave` | A10 hirate_masahide |
| `attr:cast_point of an ability` | `a11_old_fox` | A11 saito_dosan |
| `action:cue.at of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `action:dodge_projectiles.of:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: wanted a unit, got a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `fn:exists of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `action:attack.effect_multiplier` | `a13_shadow_slash` | A13 hattori_hanzo |
| `action:knockback.contact_radius` | `a14_impact_spear` | A14 maeda_toshiie |
| `action:knockback.on_end` | `a14_impact_spear` | A14 maeda_toshiie |
| `action:knockback.stop_on_contact` | `a14_impact_spear` | A14 maeda_toshiie |
| `action:chance.mode` | `a17_vital_shot` | A17 sassa_narimasa |
| `attr:owner` | `a20_pirate_domain` | A20 kuki_yoshitaka |
| `fn:landmark` | `a21_false_intel` | A21 zuiryuin_nisshu |
| `action:create_unit.overrides.ability_level` | `a28_phoenix_summon` | A28 hashiba_hideyoshi |
| `status_trigger:on_mana_spent` | `a29_withering_aura` | A29 ishida_mitsunari |
| `unit_template acquisition_range 300 differs from attack.range` | `a30_wolf_mark` | A30 niwa_nagahide |
| `unit_template abilities on a dummy` | `a36_spike_trap` | A36 maeno_nagayasu |
| `selector.flag:exclude_invisible` | `b03_zhongkui_judgment` | B03 saito_tomonobu |
| `action:cue.at of a err` | `b05_chain_lightning` | B05 mori_motonari |
| `passive_trigger:on_order_targeted` | `b06_mind_eye` | B06 sanada_yukimura |
| `fn:exists of a sym:item_a` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `fn:exists of a sym:item_b` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `unit_template acquisition_range 800 differs from attack.range` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `unit_template inventory_locked` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `unit_template item_cast_blacklist` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `action:modify_base_stat` | `b11_inherited_blood` | B11 kikuhime |
| `attr:is_silenced` | `b11_inherited_blood` | B11 kikuhime |
| `fn:enemy_team` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `fn:team_of` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `token:state` | `b16_beast_union` | B16 mochizuki_chiyome |
| `action:dodge_projectiles.of:caster_hero` | `b16_mm_phase_shift` | B16 mochizuki_chiyome |
| `aura.flag:exclude_invisible` | `b17_death_ward` | B17 kirigakure_saizo |
| `aura.status:independent` | `b17_death_ward` | B17 kirigakure_saizo |
| `unit_template tag:ward` | `b17_death_ward` | B17 kirigakure_saizo |
| `passive_trigger:on_heal_received` | `b21_undying_demon` | B21 baba_nobufusa |
| `action:destroy_tree` | `b22_force_of_nature` | B22 taigen_sessai |
| `unit_template attack.projectile_speed 600 (the host fires at 900)` | `b22_force_of_nature` | B22 taigen_sessai |
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
| `fn:max of non-numbers` | `b25_exorcism_sutra` | B25 honganji_kennyo |
| `event.damage.lethal` | `b26_kozuke_valor` | B26 okuma_tomohide |
| `action:chain` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `fn:has_state:revealed` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `status.per_source without {caster}` | `b31_raging_fire` | B31 takeda_shingen |
| `selector.flag:exclude_illusions` | `b36_heart_fence` | B36 naoe_kanetsugu |
| `attr:hero` | `c06_murder_and_loot` | C06 ishikawa_goemon |
| `attr:name` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `fn:exists of a sym:stolen` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `state type:item_name` | `item_c06e` | C06 ishikawa_goemon |
| `action:remove_ability` | `c08_devour` | C08 kato_danzo |
| `action:set_ability_active (an ability id and a fixed active)` | `c08_devour` | C08 kato_danzo |
| `fn:is_ability of a non-event ability` | `c08_devour` | C08 kato_danzo |
| `state:blind` | `c08_devour` | C08 kato_danzo |
| `selector.shape:segment` | `c08_kite_chain` | C08 kato_danzo |
| `selector.to` | `c08_kite_chain` | C08 kato_danzo |
| `selector.width` | `c08_kite_chain` | C08 kato_danzo |
| `action:modify_attack.is_orb` | `c09_frost_thorn` | C09 tachibana_ginchiyo |
| `action:modify_cooldown.item_tag` | `c13_tornado_vortex` | C13 ryuzoji_takanobu |
| `action:issue_order.queue` | `c15_hundred_arrows` | C15 tamako |
| `unit_template acquisition_range is `$acquisition` has no value` | `c15_hundred_arrows` | C15 tamako |
| `unit_template attack range or interval missing` | `c15_hundred_arrows` | C15 tamako |
| `unit_template attack.interval is `$attack_interval` has no value` | `c15_hundred_arrows` | C15 tamako |
| `unit_template passive.states` | `c15_hundred_arrows` | C15 tamako |
| `unit_template stats.move_speed is `$move_speed` has no value` | `c15_hundred_arrows` | C15 tamako |
| `unit_template acquisition_range 1100 differs from attack.range` | `c16_eight_gates` | C16 kashin_koji |
| `unit_template attack.damage_type:pure` | `c16_eight_gates` | C16 kashin_koji |
| `unit_template attack.projectile_speed 2000 (the host fires at 900)` | `c16_eight_gates` | C16 kashin_koji |
| `action:modify_cooldown.item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:exists of a sym:blink_item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:status_source` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:ability_ready` | `c18_twin_chariot` | C18 kobayakawa_takakage |
| `action:modify_cooldown.ability:se` | `c25_grievous_wound` | C25 chacha |
| `action:modify_cooldown.ability:sq` | `c25_grievous_wound` | C25 chacha |
| `action:modify_cooldown.ability:sw` | `c25_grievous_wound` | C25 chacha |
| `catalog: item item_ancient_doll is not in the shop (templates/moba_items.lua)` | `item_ancient_doll` | 道具 |
| `catalog: item item_big_tachi is not in the shop (templates/moba_items.lua)` | `item_big_tachi` | 道具 |
| `passive_trigger:on_ability_executed of an item (no learn marker)` | `item_bizennosafunekanemitsu` | 道具 |
| `catalog: item item_brave_ring is not in the shop (templates/moba_items.lua)` | `item_brave_ring` | 道具 |
| `attr:is_summon` | `item_commander_of_fan` | 道具 |
| `selector.types:building+hero+tower` | `item_contemporary_armor` | 道具 |
| `status.absorb on this apply form` | `item_devil_supressor_armor` | 道具 |
| `status.absorb.phase:pre_mitigation` | `item_devil_supressor_armor` | 道具 |
| `item.active.cost.charges` | `item_earth_wall_book` | 道具 |
| `template:earth_wall is not a content summon (templates/summons.lua)` | `item_earth_wall_book` | 道具 |
| `catalog: item item_flash_ring is not in the shop (templates/moba_items.lua)` | `item_flash_ring` | 道具 |
| `catalog: item item_flash_shoes is not in the shop (templates/moba_items.lua)` | `item_flash_shoes` | 道具 |
| `catalog: item item_flood_book is not in the shop (templates/moba_items.lua)` | `item_flood_book` | 道具 |
| `template:forest_infantry is not a content summon (templates/summons.lua)` | `item_forest_scroll` | 道具 |
| `catalog: item item_ghost_gloves is not in the shop (templates/moba_items.lua)` | `item_ghost_gloves` | 道具 |
| `catalog: item item_gold_to_prestige is not in the shop (templates/moba_items.lua)` | `item_gold_to_prestige` | 道具 |
| `catalog: item item_hachiman_bell is not in the shop (templates/moba_items.lua)` | `item_hachiman_bell` | 道具 |
| `rust_handler without behavior` | `item_hama_orb` | 道具 |
| `catalog: item item_ice_book is not in the shop (templates/moba_items.lua)` | `item_ice_book` | 道具 |
| `action:remove_status.source_is` | `item_ice_storm_book` | 道具 |
| `aura.types:building+hero` | `item_insight_gem_hero` | 道具 |
| `aura.types:building+creep+neutral+summon` | `item_insight_gem_ninja` | 道具 |
| `catalog: item item_kat_steel is not in the shop (templates/moba_items.lua)` | `item_kat_steel` | 道具 |
| `catalog: item item_kisyosennfuyuzuki is not in the shop (templates/moba_items.lua)` | `item_kisyosennfuyuzuki` | 道具 |
| `catalog: item item_ksitigarbha is not in the shop (templates/moba_items.lua)` | `item_ksitigarbha` | 道具 |
| `catalog: item item_lesser_armor is not in the shop (templates/moba_items.lua)` | `item_lesser_armor` | 道具 |
| `catalog: item item_logs_big_tachi is not in the shop (templates/moba_items.lua)` | `item_logs_big_tachi` | 道具 |
| `catalog: item item_magic_ring is not in the shop (templates/moba_items.lua)` | `item_magic_ring` | 道具 |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `catalog: item item_mountain_scroll is not in the shop (templates/moba_items.lua)` | `item_mountain_scroll` | 道具 |
| `catalog: item item_nannbann_armor is not in the shop (templates/moba_items.lua)` | `item_nannbann_armor` | 道具 |
| `selector.exclude_types` | `item_napalm_bomb` | 道具 |
| `template:ninja_unit1 is not a content summon (templates/summons.lua)` | `item_ninja1` | 道具 |
| `template:ninja_unit2 is not a content summon (templates/summons.lua)` | `item_ninja2` | 道具 |
| `catalog: item item_ninja_sword is not in the shop (templates/moba_items.lua)` | `item_ninja_sword` | 道具 |
| `catalog: item item_poisonous_ring is not in the shop (templates/moba_items.lua)` | `item_poisonous_ring` | 道具 |
| `action:projectile.lifetime` | `item_ronin_jitte` | 道具 |
| `catalog: item item_sakabatou is not in the shop (templates/moba_items.lua)` | `item_sakabatou` | 道具 |
| `catalog: item item_samurai_cloth is not in the shop (templates/moba_items.lua)` | `item_samurai_cloth` | 道具 |
| `catalog: item item_shintogo_kunimitsu is not in the shop (templates/moba_items.lua)` | `item_shintogo_kunimitsu` | 道具 |
| `catalog: item item_storm_book is not in the shop (templates/moba_items.lua)` | `item_storm_book` | 道具 |
| `catalog: item item_sukezane is not in the shop (templates/moba_items.lua)` | `item_sukezane` | 道具 |
| `template:art_of_war_archer is not a content summon (templates/summons.lua)` | `item_the_art_of_war` | 道具 |
| `template:art_of_war_infantry is not a content summon (templates/summons.lua)` | `item_the_art_of_war` | 道具 |
| `template:forest_chapter_infantry is not a content summon (templates/summons.lua)` | `item_the_art_of_war_forest_chapter` | 道具 |
| `catalog: item item_the_club_of_nebula is not in the shop (templates/moba_items.lua)` | `item_the_club_of_nebula` | 道具 |
| `action:damage.type:event` | `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 |
| `selector.types:creep+neutral+summon` | `item_the_great_sword_of_disease` | 道具 |
| `template:hurricane_field is not a content summon (templates/summons.lua)` | `item_the_great_sword_of_hurricane` | 道具 |
| `catalog: item item_the_great_sword_of_spike is not in the shop (templates/moba_items.lua)` | `item_the_great_sword_of_spike` | 道具 |
| `fn:has_tag:courier of a unit` | `item_the_great_sword_of_teleport` | 道具 |
| `item.active.cast.interruptible` | `item_the_great_sword_of_teleport` | 道具 |
| `target.types:building+courier+creep+neutral+summon` | `item_the_great_sword_of_teleport` | 道具 |
| `template:lost_art_archer is not a content summon (templates/summons.lua)` | `item_the_lost_art_of_war_1` | 道具 |
| `template:lost_art_infantry is not a content summon (templates/summons.lua)` | `item_the_lost_art_of_war_1` | 道具 |
| `stat:cooldown_reduction_pct` | `item_the_over_lightning_art_of_war` | 道具 |
| `catalog: item item_the_red_lightning_chapter is not in the shop (templates/moba_items.lua)` | `item_the_red_lightning_chapter` | 道具 |
| `catalog: item item_the_red_lightning_chapter_hyper is not in the shop (templates/moba_items.lua)` | `item_the_red_lightning_chapter_hyper` | 道具 |
| `catalog: item item_the_sword_of_kaminaobi is not in the shop (templates/moba_items.lua)` | `item_the_sword_of_kaminaobi` | 道具 |
| `catalog: item item_thunderstorms is not in the shop (templates/moba_items.lua)` | `item_thunderstorms` | 道具 |
| `catalog: item item_wayumi is not in the shop (templates/moba_items.lua)` | `item_wayumi` | 道具 |
| `catalog: item item_wayumi2 is not in the shop (templates/moba_items.lua)` | `item_wayumi2` | 道具 |
| `catalog: item item_wind_scroll is not in the shop (templates/moba_items.lua)` | `item_wind_scroll` | 道具 |
| `catalog: item item_yubashiri is not in the shop (templates/moba_items.lua)` | `item_yubashiri` | 道具 |
| `catalog: ability ninja1_flash is not a content ability (templates/abilities.lua)` | `ninja1_flash` | 道具 |
| `catalog: ability ninja2_burrow is not a content ability (templates/abilities.lua)` | `ninja2_burrow` | 道具 |

共 155 種寫法、112 招。

## 重跑

```
tools\lua\lua.exe docs\character_pipeline\tools\v3_codegen_coverage.lua
```

