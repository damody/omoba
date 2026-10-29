# v3 詞彙使用頻率與執行器實作順序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_usage.lua` 從全部 v3 文件產生（`v3_sources.lua`：97 位英雄、156 件道具；deferred 條目不計），不要手改。

- **使用次數**：文件中出現的次數。
- **英雄／道具數**：用到的英雄與道具數。條目引用的共用 routine、單位模板與狀態也算在條目上。
- 英雄以整份文件為單位，道具以單件為單位。

## 1. 依解鎖英雄數排序的執行器元件清單

從 M3 第一批（雜賀孫市、阿市需要的元件）開始。之後每一步選「每多實作一個元件、能解鎖最多英雄」的那位英雄，補齊他缺的元件，元件依使用英雄數排序。
「累計英雄」與「累計道具」是實作到這一列為止，所有技能（道具則是整件）都能用資料執行的數量。只剩道具使用的元件排在最後。

| 目標英雄數 | 需要實作的元件數 | 同時解鎖的道具 |
|---|---|---|
| 2 | 55 | 6 |
| 10 | 101 | 19 |
| 25 | 141 | 40 |
| 50 | 194 | 46 |
| 75 | 234 | 50 |
| 97 | 284 | 71 |
| 全部道具（156） | 307 | 156 |

| # | 元件 | 為了 | 用到的英雄 | 用到的道具 | 累計英雄 | 累計道具 | 這一步解鎖 |
|---|---|---|---|---|---|---|---|
| 1 | `action:apply_status` | M3 第一批（B01、C17） | 97 | 114 | 0 | 1 |  |
| 2 | `action:cue` | M3 第一批（B01、C17） | 97 | 94 | 0 | 1 |  |
| 3 | `action:damage` | M3 第一批（B01、C17） | 97 | 47 | 0 | 1 |  |
| 4 | `event:on_cast` | M3 第一批（B01、C17） | 97 | 97 | 0 | 1 |  |
| 5 | `status:stacking:refresh` | M3 第一批（B01、C17） | 97 | 112 | 0 | 1 |  |
| 6 | `status:dispel:basic` | M3 第一批（B01、C17） | 96 | 82 | 0 | 1 |  |
| 7 | `passive:triggers` | M3 第一批（B01、C17） | 67 | 75 | 0 | 1 |  |
| 8 | `event:on_attack_hit` | M3 第一批（B01、C17） | 54 | 26 | 0 | 1 |  |
| 9 | `fn:select` | M3 第一批（B01、C17） | 43 | 21 | 0 | 1 |  |
| 10 | `stat:armor` | M3 第一批（B01、C17） | 37 | 31 | 0 | 4 |  |
| 11 | `fn:alive` | M3 第一批（B01、C17） | 35 | 7 | 0 | 4 |  |
| 12 | `status:remove_on` | M3 第一批（B01、C17） | 34 | 28 | 0 | 4 |  |
| 13 | `damage:physical` | M3 第一批（B01、C17） | 32 | 4 | 0 | 4 |  |
| 14 | `stat:move_speed` | M3 第一批（B01、C17） | 32 | 21 | 0 | 4 |  |
| 15 | `action:create_unit` | M3 第一批（B01、C17） | 29 | 10 | 0 | 4 |  |
| 16 | `template:ai` | M3 第一批（B01、C17） | 29 | 10 | 0 | 4 |  |
| 17 | `template:create` | M3 第一批（B01、C17） | 29 | 10 | 0 | 4 |  |
| 18 | `stat:magic_resist` | M3 第一批（B01、C17） | 22 | 4 | 0 | 4 |  |
| 19 | `template:attack` | M3 第一批（B01、C17） | 21 | 9 | 0 | 4 |  |
| 20 | `stat:base_attack_damage` | M3 第一批（B01、C17） | 18 | 0 | 0 | 4 |  |
| 21 | `stat:attack_range` | M3 第一批（B01、C17） | 10 | 3 | 0 | 4 |  |
| 22 | `stat:base_attack_damage_pct` | M3 第一批（B01、C17） | 6 | 7 | 0 | 4 |  |
| 23 | `cost:charges` | M3 第一批（B01、C17） | 3 | 1 | 1 | 4 | B01 |
| 24 | `damage:magical` | M3 第一批（B01、C17） | 94 | 39 | 1 | 4 |  |
| 25 | `action:for_each` | M3 第一批（B01、C17） | 92 | 57 | 1 | 4 |  |
| 26 | `action:if` | M3 第一批（B01、C17） | 92 | 57 | 1 | 4 |  |
| 27 | `select:shape:circle` | M3 第一批（B01、C17） | 92 | 57 | 1 | 4 |  |
| 28 | `action:let` | M3 第一批（B01、C17） | 82 | 23 | 1 | 4 |  |
| 29 | `select:flag:include_magic_immune` | M3 第一批（B01、C17） | 73 | 34 | 1 | 4 |  |
| 30 | `rules:require_source_alive=false` | M3 第一批（B01、C17） | 55 | 3 | 1 | 4 |  |
| 31 | `select:where` | M3 第一批（B01、C17） | 46 | 20 | 1 | 4 |  |
| 32 | `action:call` | M3 第一批（B01、C17） | 36 | 3 | 1 | 4 |  |
| 33 | `event:on_phase_start` | M3 第一批（B01、C17） | 36 | 2 | 1 | 4 |  |
| 34 | `action:heal` | M3 第一批（B01、C17） | 35 | 12 | 1 | 4 |  |
| 35 | `event:on_created` | M3 第一批（B01、C17） | 33 | 46 | 1 | 4 |  |
| 36 | `action:reveal` | M3 第一批（B01、C17） | 26 | 4 | 1 | 4 |  |
| 37 | `fn:min` | M3 第一批（B01、C17） | 26 | 10 | 1 | 4 |  |
| 38 | `action:set_health` | M3 第一批（B01、C17） | 23 | 5 | 1 | 4 |  |
| 39 | `event:on_ability_executed` | M3 第一批（B01、C17） | 23 | 4 | 1 | 4 |  |
| 40 | `stat:hp_regen` | M3 第一批（B01、C17） | 22 | 22 | 1 | 5 |  |
| 41 | `stat:incoming_damage_pct` | M3 第一批（B01、C17） | 14 | 2 | 1 | 5 |  |
| 42 | `state:root` | M3 第一批（B01、C17） | 14 | 5 | 1 | 5 |  |
| 43 | `fn:is_enemy` | M3 第一批（B01、C17） | 12 | 6 | 1 | 5 |  |
| 44 | `select:flag:include_invulnerable` | M3 第一批（B01、C17） | 12 | 5 | 1 | 5 |  |
| 45 | `fn:has_tag` | M3 第一批（B01、C17） | 11 | 9 | 1 | 5 |  |
| 46 | `fn:is_item` | M3 第一批（B01、C17） | 11 | 6 | 1 | 5 |  |
| 47 | `select:flag:include_untargetable` | M3 第一批（B01、C17） | 10 | 2 | 1 | 5 |  |
| 48 | `stat:attack_damage_pct` | M3 第一批（B01、C17） | 9 | 0 | 1 | 5 |  |
| 49 | `action:restore_mana` | M3 第一批（B01、C17） | 7 | 2 | 1 | 6 |  |
| 50 | `flag:self_cast_allowed` | M3 第一批（B01、C17） | 7 | 0 | 1 | 6 |  |
| 51 | `fn:edge_distance` | M3 第一批（B01、C17） | 5 | 0 | 1 | 6 |  |
| 52 | `status:per_source` | M3 第一批（B01、C17） | 4 | 0 | 1 | 6 |  |
| 53 | `flag:can_target_buildings` | M3 第一批（B01、C17） | 3 | 0 | 1 | 6 |  |
| 54 | `flag:no_target_self` | M3 第一批（B01、C17） | 3 | 0 | 1 | 6 |  |
| 55 | `action:tether` | M3 第一批（B01、C17） | 1 | 0 | 2 | 6 | C17 |
| 56 | `stat:attack_speed` | A01（德川家康） | 58 | 32 | 2 | 9 |  |
| 57 | `event:periodic` | A01（德川家康） | 55 | 22 | 2 | 9 |  |
| 58 | `flag:pierce_magic_immune` | A01（德川家康） | 55 | 0 | 2 | 9 |  |
| 59 | `stat:move_speed_pct` | A01（德川家康） | 42 | 11 | 2 | 10 |  |
| 60 | `action:chance` | A01（德川家康） | 26 | 18 | 2 | 10 |  |
| 61 | `event:on_take_damage` | A01（德川家康） | 23 | 6 | 2 | 10 |  |
| 62 | `select:flag:visible_only` | A01（德川家康） | 22 | 9 | 2 | 10 |  |
| 63 | `status:stacking:independent` | A01（德川家康） | 19 | 27 | 3 | 10 | A01 |
| 64 | `status:dispel:none` | B26（大熊朝秀） | 82 | 75 | 3 | 10 |  |
| 65 | `action:remove_status` | B26（大熊朝秀） | 63 | 49 | 3 | 10 |  |
| 66 | `fn:has_status` | B26（大熊朝秀） | 57 | 18 | 3 | 10 |  |
| 67 | `stat:slow_pct` | B26（大熊朝秀） | 41 | 12 | 3 | 11 |  |
| 68 | `event:on_removed` | B26（大熊朝秀） | 31 | 28 | 3 | 11 |  |
| 69 | `state:magic_immune` | B26（大熊朝秀） | 22 | 16 | 3 | 11 |  |
| 70 | `action:set_ability_level` | B26（大熊朝秀） | 21 | 0 | 3 | 11 |  |
| 71 | `fn:ability_level` | B26（大熊朝秀） | 14 | 0 | 4 | 11 | B26 |
| 72 | `state:stun` | A18（稻葉一鐵） | 69 | 10 | 4 | 11 |  |
| 73 | `status:dispel:strong` | A18（稻葉一鐵） | 39 | 9 | 4 | 11 |  |
| 74 | `event:on_attack_windup` | A18（稻葉一鐵） | 20 | 7 | 4 | 11 |  |
| 75 | `status:vars` | A18（稻葉一鐵） | 19 | 3 | 4 | 11 |  |
| 76 | `action:modify_attack` | A18（稻葉一鐵） | 15 | 6 | 4 | 12 |  |
| 77 | `action:set_var` | A18（稻葉一鐵） | 14 | 1 | 4 | 12 |  |
| 78 | `stat:agi` | A18（稻葉一鐵） | 8 | 37 | 5 | 14 | A18 |
| 79 | `action:repeat` | B20（山縣昌景） | 52 | 21 | 5 | 14 |  |
| 80 | `state:no_unit_collision` | B20（山縣昌景） | 51 | 13 | 5 | 14 |  |
| 81 | `action:delay` | B20（山縣昌景） | 47 | 12 | 5 | 14 |  |
| 82 | `action:teleport` | B20（山縣昌景） | 37 | 6 | 5 | 14 |  |
| 83 | `action:set_mana` | B20（山縣昌景） | 9 | 2 | 6 | 14 | B20 |
| 84 | `fn:position` | A16（鳥居元忠） | 38 | 14 | 6 | 14 |  |
| 85 | `fn:direction` | A16（鳥居元忠） | 37 | 5 | 6 | 14 |  |
| 86 | `fn:offset` | A16（鳥居元忠） | 37 | 5 | 6 | 14 |  |
| 87 | `damage:pure` | A16（鳥居元忠） | 35 | 6 | 6 | 14 |  |
| 88 | `status:stacking:keep` | A16（鳥居元忠） | 27 | 5 | 6 | 14 |  |
| 89 | `damage_flag:reflection` | A16（鳥居元忠） | 5 | 1 | 7 | 14 | A16 |
| 90 | `action:projectile` | A19（黑田孝高） | 51 | 13 | 7 | 16 |  |
| 91 | `fn:exists` | A19（黑田孝高） | 26 | 10 | 7 | 16 |  |
| 92 | `fn:rotate` | A19（黑田孝高） | 9 | 3 | 7 | 16 |  |
| 93 | `stat:int` | A19（黑田孝高） | 6 | 54 | 8 | 19 | A19 |
| 94 | `select:limit` | B03（齋藤朝信） | 26 | 10 | 8 | 19 |  |
| 95 | `event:on_kill` | B03（齋藤朝信） | 8 | 5 | 8 | 19 |  |
| 96 | `action:spend_mana` | B03（齋藤朝信） | 7 | 4 | 8 | 19 |  |
| 97 | `select:flag:exclude_invisible` | B03（齋藤朝信） | 1 | 0 | 9 | 19 | B03 |
| 98 | `action:set_state` | B27（小島貞興） | 45 | 21 | 9 | 19 |  |
| 99 | `action:issue_order` | B27（小島貞興） | 32 | 5 | 9 | 19 |  |
| 100 | `event:on_upgrade` | B27（小島貞興） | 21 | 0 | 9 | 19 |  |
| 101 | `unit_state:scope:match` | B27（小島貞興） | 2 | 0 | 10 | 19 | B27 |
| 102 | `action:dodge_projectiles` | B30（柿崎景家） | 15 | 3 | 10 | 19 |  |
| 103 | `fn:max` | B30（柿崎景家） | 10 | 1 | 10 | 19 |  |
| 104 | `state:command_restricted` | B30（柿崎景家） | 9 | 1 | 11 | 19 | B30 |
| 105 | `unit_state:scope:ability` | A26（濃姬） | 11 | 0 | 11 | 19 |  |
| 106 | `action:knockback` | A26（濃姬） | 9 | 3 | 11 | 19 |  |
| 107 | `action:destroy_trees` | A26（濃姬） | 8 | 7 | 12 | 19 | A26 |
| 108 | `cast:channel` | C05（前田慶次） | 24 | 0 | 12 | 19 |  |
| 109 | `action:modify_cooldown` | C05（前田慶次） | 14 | 7 | 12 | 19 |  |
| 110 | `event:on_channel_end` | C05（前田慶次） | 11 | 0 | 13 | 19 | C05 |
| 111 | `fn:distance` | A35（島左近） | 35 | 7 | 13 | 19 |  |
| 112 | `state:silence` | A35（島左近） | 18 | 4 | 13 | 19 |  |
| 113 | `fn:floor` | A35（島左近） | 10 | 2 | 13 | 19 |  |
| 114 | `action:burn_mana` | A35（島左近） | 7 | 2 | 13 | 19 |  |
| 115 | `state:mute` | A35（島左近） | 6 | 2 | 13 | 19 |  |
| 116 | `state:break` | A35（島左近） | 2 | 0 | 15 | 19 | A23 A35 |
| 117 | `unit_state:scope:unit` | C03（島津義弘） | 18 | 1 | 15 | 19 |  |
| 118 | `stat:max_hp` | C03（島津義弘） | 9 | 11 | 16 | 22 | C03 |
| 119 | `unit_state:scope:hero` | C24（柳生宗嚴） | 23 | 20 | 16 | 22 |  |
| 120 | `stat:hp_regen_pct` | C24（柳生宗嚴） | 6 | 4 | 17 | 22 | C24 |
| 121 | `stat:attack_damage` | A24（佐久間盛政） | 13 | 52 | 17 | 31 |  |
| 122 | `action:face` | A24（佐久間盛政） | 11 | 1 | 17 | 31 |  |
| 123 | `action:set_display_stacks` | A24（佐久間盛政） | 9 | 3 | 18 | 31 | A24 |
| 124 | `stat:str` | A14（前田利家） | 11 | 35 | 18 | 40 |  |
| 125 | `stat:evasion_pct` | A14（前田利家） | 7 | 2 | 18 | 40 |  |
| 126 | `event:on_learn` | A14（前田利家） | 5 | 0 | 19 | 40 | A14 |
| 127 | `fn:has_state` | C02（明智秀滿） | 12 | 4 | 19 | 40 |  |
| 128 | `action:create_illusion` | C02（明智秀滿） | 7 | 0 | 20 | 40 | C02 |
| 129 | `state:invulnerable` | A30（丹羽長秀） | 25 | 3 | 20 | 40 |  |
| 130 | `state:provides_vision` | A30（丹羽長秀） | 4 | 2 | 20 | 40 |  |
| 131 | `action:zone` | A30（丹羽長秀） | 3 | 0 | 21 | 40 | A30 |
| 132 | `state:no_healthbar` | C22（佐佐木小次郎） | 15 | 1 | 21 | 40 |  |
| 133 | `state:unselectable` | C22（佐佐木小次郎） | 13 | 1 | 22 | 40 | C22 |
| 134 | `action:set_ability_active` | B11（菊姬） | 10 | 0 | 22 | 40 |  |
| 135 | `action:end_channel` | B11（菊姬） | 3 | 0 | 22 | 40 |  |
| 136 | `action:modify_base_stat` | B11（菊姬） | 1 | 0 | 23 | 40 | B11 |
| 137 | `action:chain` | B28（松永久秀） | 1 | 0 | 23 | 40 |  |
| 138 | `event:periodic_2` | B28（松永久秀） | 1 | 0 | 24 | 40 | B28 |
| 139 | `status:status_resistance` | B21（馬場信房） | 14 | 0 | 24 | 40 |  |
| 140 | `event:on_death` | B21（馬場信房） | 6 | 4 | 24 | 40 |  |
| 141 | `event:on_heal_received` | B21（馬場信房） | 1 | 0 | 25 | 40 | B21 |
| 142 | `fn:vec` | A04（竹中重治） | 5 | 0 | 25 | 40 |  |
| 143 | `action:clear_state` | A04（竹中重治） | 4 | 1 | 25 | 40 |  |
| 144 | `fn:sqrt` | A04（竹中重治） | 2 | 0 | 26 | 40 | A04 |
| 145 | `select:shape:global` | A25（織田信長） | 12 | 1 | 26 | 40 |  |
| 146 | `fn:avg_attack_against` | A25（織田信長） | 7 | 0 | 26 | 40 |  |
| 147 | `fn:random` | A25（織田信長） | 3 | 0 | 27 | 40 | A25 |
| 148 | `select:order:nearest` | B18（湖衣姬） | 11 | 8 | 27 | 40 |  |
| 149 | `select:order:random` | B18（湖衣姬） | 7 | 1 | 27 | 40 |  |
| 150 | `fn:is_visible_to` | B18（湖衣姬） | 3 | 3 | 28 | 40 | B18 |
| 151 | `status:aura` | B31（武田信玄） | 12 | 2 | 28 | 40 |  |
| 152 | `fn:random_int` | B31（武田信玄） | 7 | 2 | 29 | 40 | B31 |
| 153 | `state:invisible` | A36（前野長康） | 13 | 3 | 29 | 40 |  |
| 154 | `action:kill` | A36（前野長康） | 10 | 2 | 29 | 40 |  |
| 155 | `template:abilities` | A36（前野長康） | 3 | 2 | 30 | 40 | A36 |
| 156 | `event:on_attack_launch` | B04（伊達政宗） | 12 | 1 | 30 | 41 |  |
| 157 | `state:ranged_attack` | B04（伊達政宗） | 4 | 0 | 30 | 41 |  |
| 158 | `action:dash` | B04（伊達政宗） | 1 | 1 | 31 | 41 | B04 |
| 159 | `action:swap_abilities` | C14（齋藤義龍） | 6 | 0 | 31 | 41 |  |
| 160 | `state:attack_slow_immune` | C14（齋藤義龍） | 1 | 0 | 32 | 41 | C14 |
| 161 | `fn:stacks` | A12（大谷吉繼） | 13 | 3 | 32 | 41 |  |
| 162 | `fn:is_ability` | A12（大谷吉繼） | 7 | 0 | 32 | 41 |  |
| 163 | `fn:ability_of` | A12（大谷吉繼） | 2 | 0 | 33 | 41 | A12 |
| 164 | `fn:random_point` | B02（風魔小太郎） | 10 | 0 | 33 | 41 |  |
| 165 | `state:low_attack_priority` | B02（風魔小太郎） | 8 | 0 | 33 | 41 |  |
| 166 | `fn:polar` | B02（風魔小太郎） | 4 | 0 | 34 | 41 | B02 |
| 167 | `fn:is_template` | B19（宇佐美定滿） | 7 | 0 | 34 | 41 |  |
| 168 | `fn:owner_of` | B19（宇佐美定滿） | 4 | 1 | 35 | 41 | B19 |
| 169 | `stat:base_attack_time` | A27（柴田勝家） | 9 | 0 | 35 | 41 |  |
| 170 | `event:on_attacked_hit` | A27（柴田勝家） | 5 | 10 | 36 | 41 | A27 |
| 171 | `passive:aura` | B29（村上義清） | 11 | 25 | 36 | 42 |  |
| 172 | `action:leap` | B29（村上義清） | 2 | 0 | 37 | 42 | B29 |
| 173 | `state:revealed` | A33（本多正信） | 4 | 10 | 37 | 42 |  |
| 174 | `action:refund` | A33（本多正信） | 2 | 2 | 38 | 42 | A33 |
| 175 | `stat:miss_chance_pct` | B05（毛利元就） | 2 | 1 | 39 | 42 | B05 |
| 176 | `status:scope:hero` | B32（上杉謙信） | 6 | 1 | 39 | 42 |  |
| 177 | `stat:outgoing_damage_pct` | B32（上杉謙信） | 5 | 2 | 40 | 43 | B32 |
| 178 | `fn:armor_multiplier` | C20（長宗我部元親） | 5 | 2 | 41 | 43 | C20 |
| 179 | `action:attack` | B35（上泉信綱） | 8 | 2 | 41 | 43 |  |
| 180 | `fn:has_status_tag` | B35（上泉信綱） | 7 | 13 | 42 | 44 | B35 |
| 181 | `select:exclude` | C11（立花宗茂） | 11 | 5 | 42 | 44 |  |
| 182 | `select:flag:include_hidden` | C11（立花宗茂） | 5 | 1 | 43 | 44 | C11 |
| 183 | `status:stacks` | A31（森蘭丸） | 16 | 6 | 43 | 44 |  |
| 184 | `status:stacking:refresh_on_add` | A31（森蘭丸） | 3 | 1 | 44 | 44 | A31 |
| 185 | `action:add_stacks` | B25（本願寺顯如） | 13 | 5 | 44 | 44 |  |
| 186 | `status:stacking:stacks` | B25（本願寺顯如） | 13 | 4 | 45 | 44 | B08 |
| 187 | `action:impulse` | B25（本願寺顯如） | 1 | 0 | 46 | 44 | B25 |
| 188 | `action:random_pick` | C01（明智光秀） | 2 | 0 | 47 | 44 | C01 |
| 189 | `action:set_stacks` | A06（井伊直政） | 5 | 2 | 47 | 46 |  |
| 190 | `event:on_attack_miss` | A06（井伊直政） | 1 | 0 | 48 | 46 | A06 |
| 191 | `damage_flag:no_source_bonuses` | A11（齋藤道三） | 2 | 2 | 48 | 46 |  |
| 192 | `status:forced_move` | A11（齋藤道三） | 2 | 0 | 49 | 46 | A11 |
| 193 | `select:order:farthest` | A15（寧寧） | 1 | 0 | 49 | 46 |  |
| 194 | `status:stacking:no_refresh` | A15（寧寧） | 1 | 1 | 50 | 46 | A15 |
| 195 | `stat:magic_resist_pool` | A34（阿松） | 2 | 12 | 50 | 46 |  |
| 196 | `stat:max_mana` | A34（阿松） | 1 | 9 | 51 | 48 | A34 |
| 197 | `action:cancel_cast` | C25（茶茶） | 5 | 3 | 51 | 48 |  |
| 198 | `fn:ability_in_slot` | C25（茶茶） | 2 | 0 | 52 | 48 | C25 |
| 199 | `action:cancel_delay` | B14（片倉景綱） | 3 | 0 | 52 | 48 |  |
| 200 | `action:set_time_of_day` | B14（片倉景綱） | 2 | 0 | 53 | 48 | B14 |
| 201 | `stat:vision_night` | C10（香宗我部親泰） | 6 | 3 | 53 | 48 |  |
| 202 | `damage_flag:ignore_armor` | C10（香宗我部親泰） | 1 | 0 | 54 | 48 | C10 |
| 203 | `fn:count_in_radius` | C19（松姬） | 3 | 0 | 54 | 48 |  |
| 204 | `fn:team_of` | C19（松姬） | 3 | 0 | 55 | 48 | C19 |
| 205 | `fn:is_ally` | B12（山本勘助） | 4 | 3 | 55 | 49 |  |
| 206 | `fn:enemy_team` | B12（山本勘助） | 3 | 0 | 56 | 49 | B12 |
| 207 | `state:disarm` | C21（宮本武藏） | 13 | 3 | 56 | 50 |  |
| 208 | `status:stacking:replace` | C21（宮本武藏） | 9 | 2 | 57 | 50 | C21 |
| 209 | `action:modify_damage` | C13（龍造寺隆信） | 5 | 1 | 57 | 50 |  |
| 210 | `event:on_deal_damage_pre` | C13（龍造寺隆信） | 4 | 0 | 58 | 50 | B17 |
| 211 | `state:evade_disabled` | C13（龍造寺隆信） | 4 | 0 | 59 | 50 | C13 |
| 212 | `event:on_take_damage_pre` | A05（酒井忠次） | 2 | 1 | 59 | 50 |  |
| 213 | `cost:cooldown_start:manual` | A05（酒井忠次） | 1 | 0 | 60 | 50 | A05 |
| 214 | `fn:angle_between` | A08（榊原康政） | 1 | 0 | 61 | 50 | A08 |
| 215 | `stat:base_move_speed` | B13（百地三太夫） | 2 | 0 | 61 | 50 |  |
| 216 | `stat:mana_regen_amp_pct` | B13（百地三太夫） | 1 | 0 | 62 | 50 | B13 |
| 217 | `fn:status_var` | B36（直江兼續） | 4 | 0 | 62 | 50 |  |
| 218 | `select:flag:exclude_illusions` | B36（直江兼續） | 1 | 0 | 63 | 50 | B36 |
| 219 | `event:on_moved` | C12（大友宗麟） | 3 | 0 | 64 | 50 | C12 |
| 220 | `event:on_channel_tick` | B23（伊勢姬） | 2 | 0 | 64 | 50 |  |
| 221 | `select:flag:include_dead` | B23（伊勢姬） | 2 | 1 | 65 | 50 | B23 |
| 222 | `event:on_toggle_off` | C04（阿國） | 10 | 2 | 65 | 50 |  |
| 223 | `event:on_toggle_on` | C04（阿國） | 10 | 2 | 66 | 50 | B09 |
| 224 | `action:toggle_ability` | C04（阿國） | 5 | 0 | 67 | 50 | A02 |
| 225 | `state:flying_for_pathing` | C04（阿國） | 2 | 1 | 68 | 50 | C04 |
| 226 | `state:melee_attack` | A03（石川數正） | 1 | 0 | 69 | 50 | A03 |
| 227 | `stat:vision_day` | A17（佐佐成政） | 4 | 0 | 70 | 50 | A17 |
| 228 | `state:physical_immune` | A32（瀧川一益） | 1 | 0 | 71 | 50 | A32 |
| 229 | `fn:status_remaining` | C09（立花誾千代） | 1 | 0 | 72 | 50 | C09 |
| 230 | `state:not_on_minimap` | B10（三好長慶） | 9 | 0 | 72 | 50 |  |
| 231 | `action:remove_unit` | B10（三好長慶） | 7 | 0 | 72 | 50 |  |
| 232 | `state:flying` | B10（三好長慶） | 6 | 0 | 72 | 50 |  |
| 233 | `template:chase` | B10（三好長慶） | 3 | 0 | 74 | 50 | B10 C16 |
| 234 | `state:attack_immune` | C15（玉子） | 3 | 0 | 75 | 50 | C15 |
| 235 | `state:untargetable_by_spells` | B34（武田勝賴） | 3 | 2 | 76 | 51 | B34 |
| 236 | `action:swap_positions` | A10（平手政秀） | 3 | 0 | 77 | 51 | A10 |
| 237 | `status:unique_group` | C23（加藤清正） | 3 | 41 | 78 | 53 | C23 |
| 238 | `stat:status_resistance_pct` | A20（九鬼嘉隆） | 2 | 5 | 79 | 54 | A20 |
| 239 | `state:block_disabled` | A09（蜂須賀政勝） | 1 | 0 | 79 | 54 |  |
| 240 | `state:hex` | A09（蜂須賀政勝） | 1 | 0 | 80 | 54 | A09 |
| 241 | `stat:mana_regen` | A22（仙石秀久） | 6 | 14 | 80 | 60 |  |
| 242 | `status:reapply_if_removed` | A22（仙石秀久） | 1 | 14 | 81 | 61 | A22 |
| 243 | `event:on_mana_spent` | A29（石田三成） | 1 | 0 | 82 | 61 | A29 |
| 244 | `event:on_expired` | A28（羽柴秀吉） | 4 | 2 | 82 | 62 |  |
| 245 | `stat:physical_block_chance` | A28（羽柴秀吉） | 1 | 0 | 83 | 62 | A28 |
| 246 | `stat:hp_regen_amp_pct` | B33（最上義姬） | 1 | 0 | 84 | 62 | B33 |
| 247 | `fn:orb_slot_free` | A21（瑞龍院日秀） | 2 | 0 | 84 | 62 |  |
| 248 | `fn:landmark` | A21（瑞龍院日秀） | 1 | 0 | 85 | 62 | A21 |
| 249 | `event:on_respawn` | B06（真田幸村） | 2 | 2 | 85 | 62 |  |
| 250 | `event:on_order_targeted` | B06（真田幸村） | 1 | 2 | 86 | 62 | B06 |
| 251 | `passive:orb_slot` | B15（今川義元） | 1 | 1 | 86 | 62 |  |
| 252 | `stat:physical_block` | B15（今川義元） | 1 | 5 | 87 | 65 | B15 |
| 253 | `stat:incoming_physical_pct` | A07（本多忠勝） | 2 | 6 | 87 | 69 |  |
| 254 | `action:clear_slow_pool` | A07（本多忠勝） | 1 | 0 | 87 | 69 |  |
| 255 | `stat:slow_resistance_pct` | A07（本多忠勝） | 1 | 3 | 88 | 70 | A07 |
| 256 | `select:relative_to` | C07（立花道雪） | 2 | 0 | 88 | 70 |  |
| 257 | `stat:heal_reduction_pct` | C07（立花道雪） | 1 | 2 | 89 | 71 | C07 |
| 258 | `action:pull` | A13（服部半藏） | 1 | 0 | 89 | 71 |  |
| 259 | `fn:cos` | A13（服部半藏） | 1 | 0 | 89 | 71 |  |
| 260 | `fn:sin` | A13（服部半藏） | 1 | 0 | 90 | 71 | A13 |
| 261 | `action:pick_item` | B07（足利義昭） | 3 | 3 | 90 | 71 |  |
| 262 | `action:give_item` | B07（足利義昭） | 2 | 0 | 90 | 71 |  |
| 263 | `template:inventory_locked` | B07（足利義昭） | 1 | 0 | 91 | 71 | B07 |
| 264 | `action:add_ability` | B16（望月千代女） | 2 | 0 | 91 | 71 |  |
| 265 | `action:set_base_max_hp` | B16（望月千代女） | 1 | 0 | 91 | 71 |  |
| 266 | `fn:has_item` | B16（望月千代女） | 1 | 4 | 92 | 71 | B16 |
| 267 | `action:remove_ability` | C08（加藤段藏） | 1 | 0 | 92 | 71 |  |
| 268 | `select:shape:segment` | C08（加藤段藏） | 1 | 0 | 92 | 71 |  |
| 269 | `state:blind` | C08（加藤段藏） | 1 | 0 | 93 | 71 | C08 |
| 270 | `select:flag:include_illusions` | C18（小早川隆景） | 2 | 1 | 93 | 71 |  |
| 271 | `fn:ability_ready` | C18（小早川隆景） | 1 | 0 | 93 | 71 |  |
| 272 | `fn:status_source` | C18（小早川隆景） | 1 | 0 | 94 | 71 | C18 |
| 273 | `fn:circle_point` | B24（秋山信友） | 2 | 0 | 94 | 71 |  |
| 274 | `action:redirect_motion` | B24（秋山信友） | 1 | 0 | 94 | 71 |  |
| 275 | `state:cannot_miss` | B24（秋山信友） | 1 | 3 | 94 | 71 |  |
| 276 | `state:taunt` | B24（秋山信友） | 1 | 0 | 95 | 71 | B24 |
| 277 | `action:destroy_tree` | B22（太原雪齋） | 1 | 0 | 95 | 71 |  |
| 278 | `action:regrow_trees` | B22（太原雪齋） | 1 | 0 | 95 | 71 |  |
| 279 | `fn:tree_alive` | B22（太原雪齋） | 1 | 0 | 95 | 71 |  |
| 280 | `status:immune_to` | B22（太原雪齋） | 1 | 0 | 96 | 71 | B22 |
| 281 | `action:create_pickup` | C06（石川五右衛門） | 1 | 0 | 96 | 71 |  |
| 282 | `action:give_gold` | C06（石川五右衛門） | 1 | 0 | 96 | 71 |  |
| 283 | `action:remove_item` | C06（石川五右衛門） | 1 | 3 | 96 | 71 |  |
| 284 | `action:take_gold` | C06（石川五右衛門） | 1 | 0 | 97 | 71 | C06 |
| 285 | `cost:cooldown_groups` | 道具 | 0 | 37 | 97 | 89 |  |
| 286 | `status:scope:item_instance` | 道具 | 0 | 23 | 97 | 102 |  |
| 287 | `stat:spell_amp_pct` | 道具 | 0 | 14 | 97 | 114 |  |
| 288 | `stat:all_stats` | 道具 | 0 | 9 | 97 | 122 |  |
| 289 | `status:unique_rule` | 道具 | 0 | 9 | 97 | 128 |  |
| 290 | `fn:is_hero_id` | 道具 | 0 | 7 | 97 | 135 |  |
| 291 | `event:on_owner_death` | 道具 | 0 | 3 | 97 | 135 |  |
| 292 | `fn:item_count` | 道具 | 0 | 3 | 97 | 138 |  |
| 293 | `fn:primary_is` | 道具 | 0 | 3 | 97 | 141 |  |
| 294 | `stat:cast_point_reduction_pct` | 道具 | 0 | 3 | 97 | 142 |  |
| 295 | `event:on_attacked` | 道具 | 0 | 2 | 97 | 144 |  |
| 296 | `event:on_order` | 道具 | 0 | 2 | 97 | 145 |  |
| 297 | `event:on_unequip` | 道具 | 0 | 2 | 97 | 146 |  |
| 298 | `unit_state:scope:item_instance` | 道具 | 0 | 2 | 97 | 148 |  |
| 299 | `action:stop` | 道具 | 0 | 1 | 97 | 149 |  |
| 300 | `damage:event` | 道具 | 0 | 1 | 97 | 150 |  |
| 301 | `event:on_refreshed` | 道具 | 0 | 1 | 97 | 150 |  |
| 302 | `select:exclude_types` | 道具 | 0 | 1 | 97 | 151 |  |
| 303 | `select:flag:include_out_of_game` | 道具 | 0 | 1 | 97 | 152 |  |
| 304 | `stat:cooldown_reduction_pct` | 道具 | 0 | 1 | 97 | 153 |  |
| 305 | `stat:lifesteal_pct` | 道具 | 0 | 1 | 97 | 154 |  |
| 306 | `stat:spell_lifesteal_pct` | 道具 | 0 | 1 | 97 | 155 |  |
| 307 | `status:absorb` | 道具 | 0 | 1 | 97 | 156 |  |

## 2. 各類使用次數

### 2.1 Action（72 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `apply_status` | 1073 | 97 | 114 |
| `cue` | 1029 | 97 | 94 |
| `if` | 658 | 92 | 57 |
| `for_each` | 404 | 92 | 57 |
| `damage` | 518 | 97 | 47 |
| `remove_status` | 317 | 63 | 49 |
| `let` | 310 | 82 | 23 |
| `repeat` | 121 | 52 | 21 |
| `set_state` | 275 | 45 | 21 |
| `projectile` | 107 | 51 | 13 |
| `delay` | 114 | 47 | 12 |
| `heal` | 62 | 35 | 12 |
| `chance` | 64 | 26 | 18 |
| `teleport` | 78 | 37 | 6 |
| `call` | 217 | 36 | 3 |
| `create_unit` | 62 | 29 | 10 |
| `issue_order` | 70 | 32 | 5 |
| `reveal` | 59 | 26 | 4 |
| `set_health` | 49 | 23 | 5 |
| `modify_cooldown` | 73 | 14 | 7 |
| `set_ability_level` | 56 | 21 | 0 |
| `modify_attack` | 24 | 15 | 6 |
| `dodge_projectiles` | 46 | 15 | 3 |
| `add_stacks` | 29 | 13 | 5 |
| `set_var` | 41 | 14 | 1 |
| `destroy_trees` | 24 | 8 | 7 |
| `set_display_stacks` | 35 | 9 | 3 |
| `face` | 22 | 11 | 1 |
| `kill` | 22 | 10 | 2 |
| `knockback` | 16 | 9 | 3 |
| `set_mana` | 23 | 9 | 2 |
| `spend_mana` | 15 | 7 | 4 |
| `set_ability_active` | 40 | 10 | 0 |
| `attack` | 17 | 8 | 2 |
| `burn_mana` | 15 | 7 | 2 |
| `restore_mana` | 11 | 7 | 2 |
| `cancel_cast` | 13 | 5 | 3 |
| `create_illusion` | 14 | 7 | 0 |
| `set_stacks` | 11 | 5 | 2 |
| `remove_unit` | 10 | 7 | 0 |
| `swap_abilities` | 24 | 6 | 0 |
| `modify_damage` | 9 | 5 | 1 |
| `pick_item` | 7 | 3 | 3 |
| `toggle_ability` | 10 | 5 | 0 |
| `clear_state` | 6 | 4 | 1 |
| `refund` | 4 | 2 | 2 |
| `remove_item` | 4 | 1 | 3 |
| `swap_positions` | 11 | 3 | 0 |
| `cancel_delay` | 5 | 3 | 0 |
| `end_channel` | 3 | 3 | 0 |
| `zone` | 3 | 3 | 0 |
| `add_ability` | 13 | 2 | 0 |
| `leap` | 8 | 2 | 0 |
| `set_time_of_day` | 6 | 2 | 0 |
| `give_item` | 3 | 2 | 0 |
| `dash` | 2 | 1 | 1 |
| `random_pick` | 2 | 2 | 0 |
| `give_gold` | 4 | 1 | 0 |
| `set_base_max_hp` | 4 | 1 | 0 |
| `clear_slow_pool` | 2 | 1 | 0 |
| `take_gold` | 2 | 1 | 0 |
| `chain` | 1 | 1 | 0 |
| `create_pickup` | 1 | 1 | 0 |
| `destroy_tree` | 1 | 1 | 0 |
| `impulse` | 1 | 1 | 0 |
| `modify_base_stat` | 1 | 1 | 0 |
| `pull` | 1 | 1 | 0 |
| `redirect_motion` | 1 | 1 | 0 |
| `regrow_trees` | 1 | 1 | 0 |
| `remove_ability` | 1 | 1 | 0 |
| `stop` | 1 | 0 | 1 |
| `tether` | 1 | 1 | 0 |

### 2.2 運算式函式（53 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `has_status` | 174 | 57 | 18 |
| `select` | 165 | 43 | 21 |
| `position` | 101 | 38 | 14 |
| `offset` | 100 | 37 | 5 |
| `alive` | 99 | 35 | 7 |
| `direction` | 78 | 37 | 5 |
| `distance` | 61 | 35 | 7 |
| `exists` | 77 | 26 | 10 |
| `min` | 60 | 26 | 10 |
| `has_status_tag` | 29 | 7 | 13 |
| `has_tag` | 29 | 11 | 9 |
| `is_enemy` | 22 | 12 | 6 |
| `is_item` | 59 | 11 | 6 |
| `stacks` | 31 | 13 | 3 |
| `has_state` | 24 | 12 | 4 |
| `ability_level` | 69 | 14 | 0 |
| `floor` | 23 | 10 | 2 |
| `rotate` | 22 | 9 | 3 |
| `max` | 19 | 10 | 1 |
| `random_point` | 18 | 10 | 0 |
| `random_int` | 14 | 7 | 2 |
| `is_ability` | 43 | 7 | 0 |
| `is_template` | 20 | 7 | 0 |
| `is_hero_id` | 10 | 0 | 7 |
| `armor_multiplier` | 9 | 5 | 2 |
| `avg_attack_against` | 8 | 7 | 0 |
| `is_ally` | 7 | 4 | 3 |
| `is_visible_to` | 9 | 3 | 3 |
| `owner_of` | 18 | 4 | 1 |
| `edge_distance` | 17 | 5 | 0 |
| `vec` | 8 | 5 | 0 |
| `has_item` | 7 | 1 | 4 |
| `status_var` | 7 | 4 | 0 |
| `polar` | 5 | 4 | 0 |
| `team_of` | 11 | 3 | 0 |
| `count_in_radius` | 10 | 3 | 0 |
| `item_count` | 9 | 0 | 3 |
| `primary_is` | 9 | 0 | 3 |
| `enemy_team` | 5 | 3 | 0 |
| `random` | 3 | 3 | 0 |
| `ability_of` | 28 | 2 | 0 |
| `ability_in_slot` | 12 | 2 | 0 |
| `orb_slot_free` | 6 | 2 | 0 |
| `circle_point` | 5 | 2 | 0 |
| `sqrt` | 2 | 2 | 0 |
| `cos` | 4 | 1 | 0 |
| `sin` | 4 | 1 | 0 |
| `ability_ready` | 2 | 1 | 0 |
| `landmark` | 2 | 1 | 0 |
| `status_source` | 2 | 1 | 0 |
| `angle_between` | 1 | 1 | 0 |
| `status_remaining` | 1 | 1 | 0 |
| `tree_alive` | 1 | 1 | 0 |

### 2.3 事件（技能 events、狀態與被動 triggers）（34 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `on_cast` | 577 | 97 | 97 |
| `on_attack_hit` | 109 | 54 | 26 |
| `on_created` | 95 | 33 | 46 |
| `periodic` | 148 | 55 | 22 |
| `on_removed` | 77 | 31 | 28 |
| `on_phase_start` | 67 | 36 | 2 |
| `on_take_damage` | 40 | 23 | 6 |
| `on_ability_executed` | 53 | 23 | 4 |
| `on_attack_windup` | 35 | 20 | 7 |
| `on_upgrade` | 37 | 21 | 0 |
| `on_attacked_hit` | 15 | 5 | 10 |
| `on_attack_launch` | 30 | 12 | 1 |
| `on_kill` | 18 | 8 | 5 |
| `on_toggle_off` | 18 | 10 | 2 |
| `on_toggle_on` | 18 | 10 | 2 |
| `on_channel_end` | 14 | 11 | 0 |
| `on_death` | 16 | 6 | 4 |
| `on_expired` | 7 | 4 | 2 |
| `on_learn` | 9 | 5 | 0 |
| `on_deal_damage_pre` | 5 | 4 | 0 |
| `on_respawn` | 4 | 2 | 2 |
| `on_moved` | 6 | 3 | 0 |
| `on_take_damage_pre` | 4 | 2 | 1 |
| `on_order_targeted` | 3 | 1 | 2 |
| `on_owner_death` | 3 | 0 | 3 |
| `on_channel_tick` | 3 | 2 | 0 |
| `on_attacked` | 2 | 0 | 2 |
| `on_order` | 2 | 0 | 2 |
| `on_unequip` | 2 | 0 | 2 |
| `on_refreshed` | 2 | 0 | 1 |
| `on_attack_miss` | 1 | 1 | 0 |
| `on_heal_received` | 1 | 1 | 0 |
| `on_mana_spent` | 1 | 1 | 0 |
| `periodic_2` | 1 | 1 | 0 |

### 2.4 狀態旗標（states）（31 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `stun` | 143 | 69 | 10 |
| `no_unit_collision` | 122 | 51 | 13 |
| `magic_immune` | 49 | 22 | 16 |
| `invulnerable` | 38 | 25 | 3 |
| `silence` | 28 | 18 | 4 |
| `root` | 26 | 14 | 5 |
| `invisible` | 35 | 13 | 3 |
| `no_healthbar` | 20 | 15 | 1 |
| `disarm` | 17 | 13 | 3 |
| `revealed` | 20 | 4 | 10 |
| `unselectable` | 16 | 13 | 1 |
| `command_restricted` | 17 | 9 | 1 |
| `not_on_minimap` | 10 | 9 | 0 |
| `low_attack_priority` | 16 | 8 | 0 |
| `mute` | 10 | 6 | 2 |
| `provides_vision` | 8 | 4 | 2 |
| `flying` | 6 | 6 | 0 |
| `untargetable_by_spells` | 5 | 3 | 2 |
| `cannot_miss` | 6 | 1 | 3 |
| `ranged_attack` | 5 | 4 | 0 |
| `evade_disabled` | 4 | 4 | 0 |
| `attack_immune` | 4 | 3 | 0 |
| `flying_for_pathing` | 3 | 2 | 1 |
| `break` | 2 | 2 | 0 |
| `blind` | 2 | 1 | 0 |
| `attack_slow_immune` | 1 | 1 | 0 |
| `block_disabled` | 1 | 1 | 0 |
| `hex` | 1 | 1 | 0 |
| `melee_attack` | 1 | 1 | 0 |
| `physical_immune` | 1 | 1 | 0 |
| `taunt` | 1 | 1 | 0 |

### 2.5 StatKey（stats）（42 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `attack_speed` | 136 | 58 | 32 |
| `armor` | 96 | 37 | 31 |
| `attack_damage` | 72 | 13 | 52 |
| `int` | 65 | 6 | 54 |
| `move_speed_pct` | 82 | 42 | 11 |
| `slow_pct` | 82 | 41 | 12 |
| `move_speed` | 70 | 32 | 21 |
| `str` | 48 | 11 | 35 |
| `agi` | 46 | 8 | 37 |
| `hp_regen` | 60 | 22 | 22 |
| `magic_resist` | 37 | 22 | 4 |
| `mana_regen` | 26 | 6 | 14 |
| `max_hp` | 20 | 9 | 11 |
| `base_attack_damage` | 29 | 18 | 0 |
| `incoming_damage_pct` | 18 | 14 | 2 |
| `magic_resist_pool` | 19 | 2 | 12 |
| `spell_amp_pct` | 15 | 0 | 14 |
| `base_attack_damage_pct` | 19 | 6 | 7 |
| `attack_range` | 18 | 10 | 3 |
| `max_mana` | 14 | 1 | 9 |
| `hp_regen_pct` | 11 | 6 | 4 |
| `base_attack_time` | 12 | 9 | 0 |
| `vision_night` | 12 | 6 | 3 |
| `evasion_pct` | 11 | 7 | 2 |
| `attack_damage_pct` | 10 | 9 | 0 |
| `all_stats` | 9 | 0 | 9 |
| `incoming_physical_pct` | 11 | 2 | 6 |
| `outgoing_damage_pct` | 9 | 5 | 2 |
| `status_resistance_pct` | 9 | 2 | 5 |
| `physical_block` | 6 | 1 | 5 |
| `vision_day` | 6 | 4 | 0 |
| `slow_resistance_pct` | 4 | 1 | 3 |
| `cast_point_reduction_pct` | 3 | 0 | 3 |
| `heal_reduction_pct` | 3 | 1 | 2 |
| `miss_chance_pct` | 3 | 2 | 1 |
| `base_move_speed` | 2 | 2 | 0 |
| `lifesteal_pct` | 3 | 0 | 1 |
| `spell_lifesteal_pct` | 2 | 0 | 1 |
| `cooldown_reduction_pct` | 1 | 0 | 1 |
| `hp_regen_amp_pct` | 1 | 1 | 0 |
| `mana_regen_amp_pct` | 1 | 1 | 0 |
| `physical_block_chance` | 1 | 1 | 0 |

### 2.6 Selector 形狀與過濾（21 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `shape:circle` | 373 | 92 | 57 |
| `flag:include_magic_immune` | 223 | 73 | 34 |
| `where` | 120 | 46 | 20 |
| `limit` | 65 | 26 | 10 |
| `flag:visible_only` | 54 | 22 | 9 |
| `order:nearest` | 33 | 11 | 8 |
| `flag:include_invulnerable` | 33 | 12 | 5 |
| `exclude` | 25 | 11 | 5 |
| `shape:global` | 30 | 12 | 1 |
| `flag:include_untargetable` | 20 | 10 | 2 |
| `order:random` | 9 | 7 | 1 |
| `flag:include_hidden` | 8 | 5 | 1 |
| `flag:include_illusions` | 15 | 2 | 1 |
| `flag:include_dead` | 6 | 2 | 1 |
| `relative_to` | 5 | 2 | 0 |
| `shape:segment` | 4 | 1 | 0 |
| `flag:exclude_invisible` | 2 | 1 | 0 |
| `order:farthest` | 2 | 1 | 0 |
| `exclude_types` | 1 | 0 | 1 |
| `flag:exclude_illusions` | 1 | 1 | 0 |
| `flag:include_out_of_game` | 1 | 0 | 1 |

### 2.7 狀態功能（24 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `stacking:refresh` | 894 | 97 | 112 |
| `dispel:basic` | 588 | 96 | 82 |
| `dispel:none` | 383 | 82 | 75 |
| `remove_on` | 83 | 34 | 28 |
| `dispel:strong` | 83 | 39 | 9 |
| `stacking:independent` | 64 | 19 | 27 |
| `unique_group` | 55 | 3 | 41 |
| `stacking:keep` | 55 | 27 | 5 |
| `scope:item_instance` | 28 | 0 | 23 |
| `stacks` | 30 | 16 | 6 |
| `vars` | 25 | 19 | 3 |
| `stacking:stacks` | 24 | 13 | 4 |
| `reapply_if_removed` | 15 | 1 | 14 |
| `status_resistance` | 23 | 14 | 0 |
| `aura` | 19 | 12 | 2 |
| `stacking:replace` | 11 | 9 | 2 |
| `unique_rule` | 9 | 0 | 9 |
| `scope:hero` | 8 | 6 | 1 |
| `per_source` | 5 | 4 | 0 |
| `stacking:refresh_on_add` | 4 | 3 | 1 |
| `forced_move` | 2 | 2 | 0 |
| `stacking:no_refresh` | 2 | 1 | 1 |
| `absorb` | 1 | 0 | 1 |
| `immune_to` | 1 | 1 | 0 |

### 2.8 被動功能（3 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `triggers` | 184 | 67 | 75 |
| `aura` | 40 | 11 | 25 |
| `orb_slot` | 3 | 1 | 1 |

### 2.9 成本功能（3 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `cooldown_groups` | 37 | 0 | 37 |
| `charges` | 4 | 3 | 1 |
| `cooldown_start:manual` | 2 | 1 | 0 |

### 2.10 施放功能（1 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `channel` | 30 | 24 | 0 |

### 2.11 單位模板（6 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `ai` | 54 | 29 | 10 |
| `create` | 54 | 29 | 10 |
| `attack` | 41 | 21 | 9 |
| `abilities` | 6 | 3 | 2 |
| `chase` | 3 | 3 | 0 |
| `inventory_locked` | 1 | 1 | 0 |

### 2.12 unit_state 範圍（5 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `scope:hero` | 61 | 23 | 20 |
| `scope:unit` | 26 | 18 | 1 |
| `scope:ability` | 17 | 11 | 0 |
| `scope:item_instance` | 4 | 0 | 2 |
| `scope:match` | 2 | 2 | 0 |

### 2.13 傷害類型（4 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `magical` | 400 | 94 | 39 |
| `pure` | 51 | 35 | 6 |
| `physical` | 66 | 32 | 4 |
| `event` | 1 | 0 | 1 |

### 2.14 傷害旗標（3 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `reflection` | 6 | 5 | 1 |
| `no_source_bonuses` | 4 | 2 | 2 |
| `ignore_armor` | 1 | 1 | 0 |

### 2.15 技能旗標（規則層）（4 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `pierce_magic_immune` | 73 | 55 | 0 |
| `self_cast_allowed` | 9 | 7 | 0 |
| `can_target_buildings` | 7 | 3 | 0 |
| `no_target_self` | 3 | 3 | 0 |

### 2.16 規則層數值與規則（1 種）

| 名稱 | 使用次數 | 英雄 | 道具 |
|---|---|---|---|
| `require_source_alive=false` | 171 | 55 | 3 |

