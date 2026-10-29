# M6 批次規劃：尚未上架英雄的開放順序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_waves.lua` 從全部 v3 文件產生，不要手改。依據的元件與計數見 [usage-stats.md](usage-stats.md)。

- **基準**：已上架 10 位英雄需要的 147 個元件。M3 讓 B01、C17 用資料執行，M5 把其餘上架英雄切成資料執行，所以這些元件在 M6 開始前就要完成。
- **排序**：其餘英雄依「每多實作一個元件、能解鎖最多英雄」排序，每 9～10 位一批。B 做完一批的「新元件」，這一批的英雄就能全部用資料執行。
- 同一組元件可能一次解鎖超過 10 位英雄，這種批次會比較大，驗收時可以再拆成兩組。最後一批太小時併入前一批。
- **機制特色**：依英雄用到的元件歸類，例如召喚、分身、位移。驗收重點列出這批第一次出現的機制要特別測的情況；逐招、逐 rank 的情境由 `v3_scenarios.lua` 產生。

| 批次 | 英雄數 | 新元件數 | 首次出現的機制 |
|---|---|---|---|
| 1 | 9 | 4 | 投射物、引導、擊退／拉扯、換技能、全場共用狀態 |
| 2 | 9 | 12 | — |
| 3 | 9 | 15 | 光環、召喚、地面區域、隱形、分身 |
| 4 | 9 | 16 | — |
| 5 | 13 | 21 | 充能、召喚物 AI |
| 6 | 9 | 10 | 位移 |
| 7 | 9 | 13 | — |
| 8 | 9 | 13 | 偷技能 |
| 9 | 11 | 33 | 法球、連鎖彈跳 |

## 第 1 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| A01 德川家康 | 一般技能 |
| B20 山縣昌景 | 一般技能 |
| B26 大熊朝秀 | 一般技能 |
| C22 佐佐木小次郎 | 投射物、引導 |
| A12 大谷吉繼 | 擊退／拉扯、投射物、換技能 |
| A14 前田利家 | 擊退／拉扯 |
| A26 濃姬 | 擊退／拉扯、投射物 |
| B27 小島貞興 | 投射物、全場共用狀態 |
| C03 島津義弘 | 一般技能 |

**執行器要新增的元件**（4 個）：`action:knockback`、`action:set_display_stacks`、`unit_state:scope:match`、`fn:floor`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。
- 投射物（本批首次出現）：投射物命中判定、被閃避、射程盡頭
- 引導（本批首次出現）：引導被打斷、引導中的每跳
- 擊退／拉扯（本批首次出現）：強制位移的距離、地形與碰撞
- 換技能（本批首次出現）：技能替換後的冷卻與等級
- 全場共用狀態（本批首次出現）：全場共用狀態的同步與揭露

## 第 2 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| C05 前田慶次 | 投射物、引導 |
| C09 立花誾千代 | 一般技能 |
| A18 稻葉一鐵 | 一般技能 |
| B30 柿崎景家 | 投射物、全場共用狀態 |
| A16 鳥居元忠 | 一般技能 |
| A24 佐久間盛政 | 投射物 |
| C01 明智光秀 | 引導 |
| A23 宇喜多秀家 | 引導 |
| A35 島左近 | 一般技能 |

**執行器要新增的元件**（12 個）：`event:on_channel_end`、`fn:status_remaining`、`status:vars`、`action:set_var`、`fn:max`、`damage:pure`、`damage_flag:reflection`、`action:face`、`action:random_pick`、`state:silence`、`state:mute`、`state:break`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。

## 第 3 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| C14 齋藤義龍 | 投射物、換技能 |
| C24 柳生宗嚴 | 擊退／拉扯 |
| A19 黑田孝高 | 投射物 |
| A11 齋藤道三 | 光環、引導 |
| A22 仙石秀久 | 投射物、光環 |
| A30 丹羽長秀 | 召喚、地面區域、投射物、引導 |
| A36 前野長康 | 召喚、光環、隱形 |
| B34 武田勝賴 | 召喚、投射物、光環 |
| B02 風魔小太郎 | 分身、地面區域、投射物、隱形 |

**執行器要新增的元件**（15 個）：`state:attack_slow_immune`、`stat:hp_regen_pct`、`fn:rotate`、`stat:int`、`damage_flag:no_source_bonuses`、`status:forced_move`、`status:unique_group`、`status:reapply_if_removed`、`state:provides_vision`、`action:zone`、`action:kill`、`template:abilities`、`state:untargetable_by_spells`、`state:low_attack_priority`、`action:create_illusion`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。
- 光環（本批首次出現）：光環半徑、殘留時間、同名不疊加
- 召喚（本批首次出現）：召喚物的數量上限、時限、控制權與擊殺獎勵
- 地面區域（本批首次出現）：區域的持續時間、每跳間隔、離開後的殘留
- 隱形（本批首次出現）：隱形被看穿、攻擊或施法解除隱形
- 分身（本批首次出現）：分身的輸出與承受傷害比例、道具複製、分身死亡

## 第 4 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| C02 明智秀滿 | 分身、投射物 |
| A33 本多正信 | 召喚、光環、引導 |
| B03 齋藤朝信 | 投射物 |
| B05 毛利元就 | 光環 |
| B11 菊姬 | 分身、引導 |
| B09 真田昌幸 | 引導 |
| B18 湖衣姬 | 召喚 |
| B31 武田信玄 | 投射物、光環、引導 |
| A20 九鬼嘉隆 | 召喚、投射物、光環、隱形 |

**執行器要新增的元件**（16 個）：`fn:has_state`、`state:revealed`、`action:refund`、`event:on_kill`、`select:flag:exclude_invisible`、`select:order:nearest`、`stat:miss_chance_pct`、`action:end_channel`、`action:modify_base_stat`、`fn:status_var`、`event:on_channel_tick`、`fn:vec`、`fn:is_visible_to`、`fn:random_int`、`event:on_attacked_hit`、`action:cancel_delay`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。

## 第 5 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| A03 石川數正 | 投射物 |
| B35 上泉信綱 | 投射物 |
| C20 長宗我部元親 | 一般技能 |
| B25 本願寺顯如 | 擊退／拉扯、投射物、引導 |
| B36 直江兼續 | 擊退／拉扯 |
| C12 大友宗麟 | 投射物、隱形 |
| A04 竹中重治 | 召喚、引導 |
| B21 馬場信房 | 投射物 |
| A17 佐佐成政 | 充能 |
| B10 三好長慶 | 召喚、投射物、光環、召喚物 AI |
| B19 宇佐美定滿 | 召喚、引導 |
| C11 立花宗茂 | 投射物 |
| C16 果心居士 | 召喚、分身、投射物、隱形、召喚物 AI |

**執行器要新增的元件**（21 個）：`fn:has_status_tag`、`status:scope:hero`、`fn:armor_multiplier`、`event:on_moved`、`state:melee_attack`、`action:impulse`、`status:stacking:replace`、`action:clear_state`、`select:flag:exclude_illusions`、`event:on_death`、`fn:sqrt`、`event:on_heal_received`、`state:disarm`、`select:shape:global`、`fn:is_template`、`stat:vision_night`、`state:flying`、`select:flag:include_hidden`、`fn:owner_of`、`stat:vision_day`、`template:chase`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。
- 充能（本批首次出現）：充能上限、回充時間、升級時的充能
- 召喚物 AI（本批首次出現）：召喚物追擊、放棄目標、瞬移回主人

## 第 6 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| A27 柴田勝家 | 召喚、分身、投射物 |
| A10 平手政秀 | 召喚、位移、投射物、光環、引導 |
| A25 織田信長 | 投射物 |
| A29 石田三成 | 召喚、投射物、光環、召喚物 AI |
| B32 上杉謙信 | 投射物 |
| A31 森蘭丸 | 投射物、引導 |
| C04 阿國 | 引導 |
| A32 瀧川一益 | 位移、擊退／拉扯、投射物、換技能 |
| B17 霧隱才藏 | 召喚、投射物、光環、隱形 |

**執行器要新增的元件**（10 個）：`fn:random`、`action:swap_positions`、`fn:avg_attack_against`、`event:on_mana_spent`、`stat:outgoing_damage_pct`、`status:stacking:refresh_on_add`、`state:flying_for_pathing`、`action:modify_damage`、`event:on_deal_damage_pre`、`state:physical_immune`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。
- 位移（本批首次出現）：交換位置的對象與距離

## 第 7 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| C13 龍造寺隆信 | 召喚、投射物、引導 |
| A05 酒井忠次 | 一般技能 |
| A08 榊原康政 | 一般技能 |
| C21 宮本武藏 | 一般技能 |
| A06 井伊直政 | 投射物、光環 |
| A15 寧寧 | 投射物、引導、隱形 |
| B12 山本勘助 | 光環 |
| C19 松姬 | 擊退／拉扯、投射物 |
| C23 加藤清正 | 召喚、光環 |

**執行器要新增的元件**（13 個）：`state:evade_disabled`、`event:on_take_damage_pre`、`cost:cooldown_start:manual`、`fn:count_in_radius`、`fn:enemy_team`、`fn:angle_between`、`action:set_stacks`、`event:on_attack_miss`、`select:order:farthest`、`status:stacking:no_refresh`、`fn:is_ally`、`fn:team_of`、`action:cancel_cast`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。

## 第 8 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| B14 片倉景綱 | 隱形 |
| B23 伊勢姬 | 召喚、投射物、光環、引導 |
| C10 香宗我部親泰 | 投射物 |
| C25 茶茶 | 擊退／拉扯、偷技能、充能 |
| A28 羽柴秀吉 | 召喚、投射物、引導 |
| B33 最上義姬 | 召喚、投射物 |
| A21 瑞龍院日秀 | 召喚、投射物、光環 |
| B13 百地三太夫 | 召喚、光環、隱形 |
| A09 蜂須賀政勝 | 召喚 |

**執行器要新增的元件**（13 個）：`action:set_time_of_day`、`select:flag:include_dead`、`damage_flag:ignore_armor`、`fn:ability_in_slot`、`event:on_expired`、`stat:physical_block_chance`、`stat:hp_regen_amp_pct`、`fn:orb_slot_free`、`fn:landmark`、`stat:base_move_speed`、`stat:mana_regen_amp_pct`、`state:block_disabled`、`state:hex`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。
- 偷技能（本批首次出現）：偷到的技能等級、時限與冷卻

## 第 9 批

**英雄**：

| 英雄 | 機制特色 |
|---|---|
| B15 今川義元 | 分身、投射物、法球 |
| B28 松永久秀 | 投射物、連鎖彈跳、引導 |
| C07 立花道雪 | 召喚、地面區域、引導 |
| A13 服部半藏 | 分身、位移、投射物、換技能、隱形 |
| B07 足利義昭 | 召喚、光環、引導 |
| B16 望月千代女 | 召喚、位移、換技能、隱形 |
| C08 加藤段藏 | 換技能、偷技能、隱形 |
| C18 小早川隆景 | 一般技能 |
| B24 秋山信友 | 召喚、擊退／拉扯、投射物、換技能 |
| B22 太原雪齋 | 召喚 |
| C06 石川五右衛門 | 一般技能 |

**執行器要新增的元件**（33 個）：`passive:orb_slot`、`stat:physical_block`、`action:chain`、`event:periodic_2`、`select:relative_to`、`stat:heal_reduction_pct`、`action:pull`、`fn:cos`、`fn:sin`、`action:pick_item`、`action:give_item`、`template:inventory_locked`、`action:add_ability`、`action:set_base_max_hp`、`fn:has_item`、`action:remove_ability`、`select:shape:segment`、`state:blind`、`select:flag:include_illusions`、`fn:ability_ready`、`fn:status_source`、`fn:circle_point`、`action:redirect_motion`、`state:cannot_miss`、`state:taunt`、`action:destroy_tree`、`action:regrow_trees`、`fn:tree_alive`、`status:immune_to`、`action:create_pickup`、`action:give_gold`、`action:remove_item`、`action:take_gold`

**驗收重點**：

- 每位英雄每招的 `v3_scenarios.lua` 情境，包括正常施放、資源不足、冷卻、目標失效、魔免、驅散、死亡、可見性、quirk；以及實機一局，autopilot 要施放到每一招。
- 法球（本批首次出現）：法球與其他法球、道具法球的互斥
- 連鎖彈跳（本批首次出現）：彈跳次數、不重複目標、彈跳半徑

## 基準元件（M3／M5 完成）

`action:add_stacks`、`action:apply_status`、`action:attack`、`action:burn_mana`、`action:call`、`action:chance`、`action:clear_slow_pool`、`action:create_unit`、`action:cue`、`action:damage`、`action:dash`、`action:delay`、`action:destroy_trees`、`action:dodge_projectiles`、`action:for_each`、`action:heal`、`action:if`、`action:issue_order`、`action:leap`、`action:let`、`action:modify_attack`、`action:modify_cooldown`、`action:projectile`、`action:remove_status`、`action:remove_unit`、`action:repeat`、`action:restore_mana`、`action:reveal`、`action:set_ability_active`、`action:set_ability_level`、`action:set_health`、`action:set_mana`、`action:set_state`、`action:spend_mana`、`action:swap_abilities`、`action:teleport`、`action:tether`、`action:toggle_ability`、`cast:channel`、`cost:charges`、`damage:magical`、`damage:physical`、`event:on_ability_executed`、`event:on_attack_hit`、`event:on_attack_launch`、`event:on_attack_windup`、`event:on_cast`、`event:on_created`、`event:on_learn`、`event:on_order_targeted`、`event:on_phase_start`、`event:on_removed`、`event:on_respawn`、`event:on_take_damage`、`event:on_toggle_off`、`event:on_toggle_on`、`event:on_upgrade`、`event:periodic`、`flag:can_target_buildings`、`flag:no_target_self`、`flag:pierce_magic_immune`、`flag:self_cast_allowed`、`fn:ability_level`、`fn:ability_of`、`fn:alive`、`fn:direction`、`fn:distance`、`fn:edge_distance`、`fn:exists`、`fn:has_status`、`fn:has_tag`、`fn:is_ability`、`fn:is_enemy`、`fn:is_item`、`fn:min`、`fn:offset`、`fn:polar`、`fn:position`、`fn:random_point`、`fn:select`、`fn:stacks`、`passive:aura`、`passive:triggers`、`rules:require_source_alive=false`、`select:exclude`、`select:flag:include_invulnerable`、`select:flag:include_magic_immune`、`select:flag:include_untargetable`、`select:flag:visible_only`、`select:limit`、`select:order:random`、`select:shape:circle`、`select:where`、`stat:agi`、`stat:armor`、`stat:attack_damage`、`stat:attack_damage_pct`、`stat:attack_range`、`stat:attack_speed`、`stat:base_attack_damage`、`stat:base_attack_damage_pct`、`stat:base_attack_time`、`stat:evasion_pct`、`stat:hp_regen`、`stat:incoming_damage_pct`、`stat:incoming_physical_pct`、`stat:magic_resist`、`stat:magic_resist_pool`、`stat:mana_regen`、`stat:max_hp`、`stat:max_mana`、`stat:move_speed`、`stat:move_speed_pct`、`stat:slow_pct`、`stat:slow_resistance_pct`、`stat:status_resistance_pct`、`stat:str`、`state:attack_immune`、`state:command_restricted`、`state:invisible`、`state:invulnerable`、`state:magic_immune`、`state:no_healthbar`、`state:no_unit_collision`、`state:not_on_minimap`、`state:ranged_attack`、`state:root`、`state:stun`、`state:unselectable`、`status:aura`、`status:dispel:basic`、`status:dispel:none`、`status:dispel:strong`、`status:per_source`、`status:remove_on`、`status:stacking:independent`、`status:stacking:keep`、`status:stacking:refresh`、`status:stacking:stacks`、`status:stacks`、`status:status_resistance`、`template:ai`、`template:attack`、`template:create`、`unit_state:scope:ability`、`unit_state:scope:hero`、`unit_state:scope:unit`

