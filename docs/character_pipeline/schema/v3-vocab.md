<!-- 由 `tools\lua\lua.exe docs/character_pipeline/tools/package_v3.lua --vocab-md` 產生；不要手改 -->
# package schema v3.3 詞彙表

## 各層欄位

- 文件：`abilities`、`deferred`、`hero`、`hooks`、`id`、`items`、`kind`、`notes`、`routines`、`schema_version`、`source`、`statuses`、`unit_state`、`unit_templates`、`values`
- hero：`attack`、`auto_learn`、`notes`、`source_id`、`text`
- values.*：`damage_type`、`expr`、`hidden`、`integer`、`kv`、`note`、`per`、`rounding`、`source`、`source_differs`、`unit`、`v2_key`、`v2_only`
- 技能：`bot`、`cast`、`confidence`、`cost`、`coverage`、`deferred_reason`、`events`、`flags`、`handler`、`id`、`kind`、`learnable`、`level_link`、`max_level`、`notes`、`passive`、`pending`、`provenance`、`quirks`、`requires`、`slot`、`source_id`、`statuses`、`tags`、`target`、`text`、`unit_state`、`values`
- target：`aoe_radius`、`flags`、`min_range`、`range`、`team`、`type`、`types`、`vector_length`
- cast：`backswing`、`cast_point`、`channel_time`、`ignore_backswing`、`immediate`、`interruptible`、`move_while_casting`、`turn_while_casting`
- cost：`charges`、`cooldown`、`cooldown_groups`、`cooldown_start`、`gold`、`health`、`mana`、`refund`
- cost.charges：`initial`、`max`、`on_rank_up`、`restore`、`scope`
- requires：`condition`、`has_status`、`hero_level`、`not_status`
- passive：`aura`、`orb_slot`、`reveal`、`states`、`stats`、`triggers`、`unique_group`、`visibility`
- 道具：`active`、`confidence`、`coverage`、`deferred_reason`、`handler`、`id`、`max_level`、`notes`、`passive`、`pending`、`provenance`、`quirks`、`shop`、`source_id`、`statuses`、`tags`、`text`、`unit_state`、`values`
- shop：`category`、`consumable`、`cost`、`droppable`、`purchasable`、`recipe`、`sellable`、`shareable`、`stock`、`unique`
- 道具 active：`bot`、`cast`、`cost`、`events`、`flags`、`kind`、`learnable`、`requires`、`tags`、`target`
- 狀態：`absorb`、`aura`、`block_reapply`、`charges`、`cue`、`dispel`、`duration`、`forced_move`、`id`、`immune_to`、`linger`、`notes`、`per_source`、`reapply_if_removed`、`remove_on`、`reveal`、`runtime_id`、`scope`、`stacking`、`states`、`stats`、`status_resistance`、`tags`、`text`、`triggers`、`unique_group`、`unique_rule`、`values`、`vars`、`visibility`
- stacking（table）：`decay`、`initial`、`max`、`mode`、`per_stack_duration`、`value_stacks`
- absorb：`amount`、`fraction`、`lethal_only`、`on_absorb`、`on_broken`、`phase`、`types`
- aura：`flags`、`include_self`、`linger`、`radius`、`status`、`team`、`types`、`unique_group`、`where`
- triggers.*（table）：`actions`、`filter`、`first`、`interval`、`notes`、`phase`、`priority`、`who`
- selector：`angle`、`exclude`、`exclude_types`、`flags`、`from`、`inner`、`length`、`limit`、`notes`、`order`、`outer`、`post_filter`、`prefer`、`radius`、`relative_to`、`shape`、`take_while`、`team`、`to`、`types`、`visible_to`、`where`、`width`
- 單位模板：`abilities`、`acquisition_range`、`ai`、`attack`、`blocks_pathing`、`bounty`、`chase`、`collision_radius`、`flying`、`id`、`inventory_locked`、`item_cast_blacklist`、`limit_per_owner`、`max_level`、`notes`、`over_limit`、`owner_death`、`passive`、`provenance`、`quirks`、`source_id`、`stats`、`statuses`、`tags`、`text`、`turn_rate`、`values`、`vision`
- 單位模板 attack：`attack_point`、`damage`、`damage_max`、`damage_type`、`interval`、`kind`、`projectile_speed`、`range`
- create_unit.overrides：`ability_level`、`agi`、`armor`、`attack_damage`、`attack_damage_max`、`attack_interval`、`attack_range`、`bounty_gold`、`bounty_xp`、`hp`、`hp_regen`、`int`、`level`、`magic_resist`、`mana`、`mana_regen`、`model_scale`、`move_speed`、`str`、`vision_day`、`vision_night`
- create_illusion.copy：`abilities`、`except_abilities`、`hp`、`items`、`level`、`mana`、`status_caster`、`status_duration`、`status_except`、`status_tags`、`statuses`、`team`、`unit_state`
- create_illusion.outgoing／incoming：`all`、`magical`、`physical`、`pure`
- unit_state.*／vars.*：`init`、`note`、`reveal`、`scope`、`type`
- hooks.*：`kind`、`notes`、`params`、`reads`、`test`、`why`

## 數值單位（values.*.unit）

| unit | 意義 |
|---|---|
| `armor` | 護甲 |
| `atk_speed` | 攻擊速度 |
| `count` | 次數或層數（整數） |
| `deg` | 角度 |
| `dist` | 原作距離 |
| `dmg` | 傷害 |
| `gold` | 金錢 |
| `heal` | 治療量 |
| `hp` | 生命值 |
| `level` | 等級 |
| `mana` | 法力 |
| `move_speed` | 移動速度 |
| `number` | 其他無單位數值 |
| `pct` | 百分比（運算式中為比例） |
| `per_s` | 每秒量 |
| `ratio` | 倍率（不轉換） |
| `s` | 秒 |
| `speed` | 投射物或位移速度（距離／秒） |
| `stat` | 三圍屬性點數 |
| `xp` | 經驗 |

## 出處前綴（values.*.source）

- `assumed:` 原作無法判定，暫定值（assumed:<理由>）
- `design:` 本專案的設計決定或改編（design:<理由>）
- `kv:` 原作 KV 鍵（kv:<Key>，可加 @<path>:<line>）
- `rules:` 全域規則層（rules:<key>）
- `script:` 原作腳本（script:<path>:<lines> 說明）
- `text:` 只出現在原作說明文字（text:<file>:<line>）

## quirk 政策（quirks[].policy）

- `as_source`：照原作實際行為還原（預設）
- `design_intent`：跟 tick 頻率等引擎副作用有關，照設計意圖（KV 數值）
- `fix_candidate`：明顯違反設計意圖且影響可玩性，列入修正候選清單由使用者決定；目前仍照原作
- `no_effect`：原作執行期報錯或死碼，照中斷後的結果（通常沒有效果）
- `note`：只是紀錄（命名錯誤、呈現細節），不影響結算
- `port_adaptation`：現行移植（v2 Rust handler）與原作不同的改編；資料照現行，M1 決定保留或改回原作

## 揭露政策（reveal）

- `derived`：只揭露結算結果（例如屬性總值）
- `public`：隨單位揭露，雙方 replica 都知道
- `team`：只有己方 replica 知道；不可影響對方 replica 的判定

## 說明格式（{name:fmt}）

- `abs`：取絕對值
- `all`：每個 rank 用 / 分隔，目前 rank 加粗（預設）
- `current`：只顯示目前 rank
- `int`：四捨五入成整數
- `max`：只顯示最後一個 rank
- `signed`：帶正負號

## Action

`!` 是必填。參數型別：expr 數值運算式、cond 條件、unit／point／vec／any 該型別的運算式、actions 巢狀 action、selector、status／ability／template／routine／hook 已宣告的 id、cue 呈現 id、name 綁定名稱、bool、enum:a|b、list:型別、table 名稱→運算式。

| op | 參數 |
|---|---|
| `add_ability` | `ability`=ability!，`as`=name，`cast_point`=expr，`level`=expr，`slot`=enum:Q|W|E|R|D|F|sub|hidden，`to`=unit! |
| `add_stacks` | `count`=expr!，`max`=expr，`refresh`=bool，`status`=status!，`to`=unit! |
| `apply_status` | `as`=name，`duration`=expr，`source`=unit，`stacks`=expr，`status`=status!，`to`=unit!，`vars`=table |
| `attack` | `as`=name，`attacker`=unit!，`cannot_miss`=bool，`effect_multiplier`=expr，`fake`=bool，`ignore_range`=bool，`instant`=bool，`target`=unit!，`trigger_effects`=bool，`use_orb`=bool |
| `burn_mana` | `amount`=expr!，`as`=name，`source`=unit，`to`=unit! |
| `call` | `args`=table，`routine`=routine! |
| `cancel_cast` | `reason`=text |
| `cancel_delay` | `id`=name! |
| `chain` | `actions`=actions!，`allow_repeat`=bool，`as`=name，`cue`=cue，`first`=unit!，`flags`=list:flag，`from`=any!，`interval`=expr，`jumps`=expr!，`order`=enum:nearest|random|id，`projectile_speed`=expr，`radius`=expr!，`take_while`=cond，`team`=enum:enemy|ally|both|any!，`types`=list:enum:hero|creep|building|summon|ward|neutral|boss|illusion|all，`where`=cond |
| `chance` | `actions`=actions!，`counter`=enum:ability|hero|item_name|status，`counter_var`=name，`mode`=enum:independent|prd，`otherwise`=actions，`p`=expr!，`pity_after`=expr |
| `clear_slow_pool` | `of`=unit! |
| `clear_state` | `item`=any，`on`=unit，`var`=name! |
| `create_illusion` | `as`=name，`at`=any，`control`=enum:none|owner|ai，`copy`=illusion_copy，`count`=expr，`duration`=expr!，`incoming`=illusion_damage，`of`=unit!，`outgoing`=illusion_damage，`triggers_effects`=bool |
| `create_pickup` | `as`=name，`at`=any!，`cue`=cue，`killable`=bool，`lifetime`=expr，`on_destroyed`=actions，`on_pickup`=actions!，`who`=enum:any|hero|ally|enemy |
| `create_tree` | `as`=name，`at`=any!，`lifetime`=expr |
| `create_unit` | `ai_target`=unit，`as`=name，`at`=any!，`control`=enum:none|owner|ai，`copy_of`=unit，`count`=expr，`facing`=vec，`lifetime`=expr，`limit_per_owner`=expr，`over_limit`=enum:remove_oldest|reject|none|orphan_oldest，`overrides`=unit_overrides，`owner`=unit，`team`=unit，`template`=template! |
| `cue` | `at`=any，`attach`=unit，`duration`=expr，`follow`=bool，`id`=cue!，`radius`=expr，`stop`=bool，`team`=enum:own|enemy|all，`to`=any |
| `damage` | `ability`=ability，`amount`=expr!，`as`=name，`credit`=unit，`flags`=list:enum:no_amplify|no_reflect|hp_removal|non_lethal|reflection|no_spell_lifesteal|no_lifesteal|is_attack|ignore_armor|bypass_block|no_crit|no_source_bonuses，`require_source_alive`=bool，`source`=unit，`to`=unit!，`type`=enum:physical|magical|pure|event! |
| `dash` | `as`=name，`direction`=vec，`distance`=expr，`height`=expr，`hit_radius`=expr，`interruptible`=bool，`on_end`=actions，`on_hit`=actions，`on_tick`=actions，`speed`=expr!，`stop_at_target`=bool，`to`=any，`who`=unit! |
| `delay` | `actions`=actions!，`capture`=list:name，`id`=name，`time`=expr! |
| `destroy_projectile` | `projectile`=any! |
| `destroy_tree` | `tree`=any! |
| `destroy_trees` | `at`=any!，`radius`=expr! |
| `dodge_projectiles` | `from_team`=enum:enemy|ally|any，`kind`=enum:tracking|all，`of`=unit! |
| `drop_item` | `at`=any，`from`=unit!，`item`=any! |
| `end_channel` | `interrupted`=bool |
| `face` | `toward`=any!，`who`=unit! |
| `for_each` | `actions`=actions!，`as`=name，`select`=selector! |
| `give_gold` | `amount`=expr!，`reliable`=bool，`show`=bool，`to`=unit! |
| `give_item` | `as`=name，`charges`=expr，`item`=item_ref!，`to`=unit!，`vars`=table |
| `give_xp` | `amount`=expr!，`to`=unit! |
| `heal` | `amount`=expr!，`as`=name，`flags`=list:enum:ignore_heal_amp|ignore_heal_reduction|no_overheal|silent，`source`=unit，`to`=unit! |
| `hook` | `args`=table，`as`=name，`callbacks`=callbacks，`name`=hook! |
| `if` | `actions`=actions!，`cond`=cond!，`otherwise`=actions |
| `impulse` | `as`=name，`decay`=expr!，`direction`=vec!，`duration`=expr，`speed`=expr!，`who`=unit! |
| `issue_order` | `ability`=ability，`at`=any，`force`=bool，`order`=enum:move|attack|attack_move|stop|hold|cast|follow!，`queue`=bool，`target`=unit，`who`=unit! |
| `kill` | `killer`=unit，`silent`=bool，`to`=unit! |
| `knockback` | `contact_radius`=expr，`distance`=expr!，`duration`=expr!，`from`=any!，`height`=expr，`on_end`=actions，`stop_on_contact`=bool，`stun`=bool，`who`=unit! |
| `leap` | `duration`=expr!，`height`=expr，`on_end`=actions，`to`=any!，`who`=unit! |
| `let` | `name`=name!，`value`=any! |
| `modify_attack` | `bonus_damage`=expr，`bonus_damage_type`=enum:physical|magical|pure|event，`cannot_miss`=bool，`crit_multiplier`=expr，`damage_multiplier`=expr，`is_orb`=bool，`no_effects`=bool，`orb_priority`=expr |
| `modify_base_stat` | `amount`=expr!，`stat`=enum:str|agi|int!，`to`=unit! |
| `modify_charges` | `ability`=ability!，`amount`=expr，`mode`=enum:add|set|refill!，`of`=unit |
| `modify_cooldown` | `ability`=ability，`amount`=expr，`item`=any，`item_tag`=name，`mode`=enum:set|reduce|reset|end|start!，`of`=unit |
| `modify_damage` | `add`=expr，`block`=expr，`cancel`=bool，`multiplier`=expr，`set`=expr |
| `pick_item` | `as`=name!，`from`=unit!，`order`=enum:random|first|last，`slots`=enum:inventory|backpack|all，`where`=cond |
| `projectile` | `arm_distance`=expr，`as`=name，`bind`=name，`cue`=cue，`direction`=vec，`distance`=expr，`dodgeable`=bool，`end_radius`=expr，`flags`=list:flag，`from`=any!，`hit_buildings`=bool，`hit_priority`=list:enum:enemy|ally，`kind`=enum:linear|tracking|path!，`lifetime`=expr，`max_hits`=expr，`on_end`=actions，`on_hit`=actions，`on_tick`=actions，`path`=list:any，`pierce`=bool，`radius`=expr，`remove_on`=list:enum:owner_death|status_end，`speed`=expr!，`team`=enum:enemy|ally|both|any，`tick_interval`=expr，`to`=any，`types`=list:enum:hero|creep|building|summon|ward|neutral|boss|illusion|all，`vision`=expr，`where`=cond |
| `pull` | `along`=any，`max_duration`=expr，`on_end`=actions，`speed`=expr!，`stop_distance`=expr，`toward`=any，`who`=unit! |
| `random_pick` | `as`=name，`options`=options! |
| `redirect_motion` | `direction`=vec，`distance`=expr，`speed`=expr，`toward`=any，`who`=unit! |
| `refund` | `cooldown`=bool，`health`=bool，`mana`=bool |
| `regrow_trees` | `at`=any，`radius`=expr |
| `remove_ability` | `ability`=ability!，`from`=unit! |
| `remove_item` | `from`=unit!，`item`=any! |
| `remove_status` | `dispel`=enum:basic|strong|all_debuffs|all_buffs|all，`except`=list:status，`exclude_source_tags`=list:name，`exclude_tags`=list:name，`from`=unit!，`limit`=expr，`min_duration`=expr，`min_remaining`=expr，`only`=enum:debuffs|buffs，`source_ability`=name，`source_is`=list:enum:has_ability|no_ability|alive|gone，`states`=list:state_flag，`status`=status，`tags`=list:name，`team_of_source`=enum:enemy|ally|any |
| `remove_unit` | `to`=unit! |
| `repeat` | `actions`=actions!，`as`=name，`count`=expr，`first`=expr，`id`=name，`interval`=expr，`stop_when`=cond，`while`=cond |
| `restore_mana` | `amount`=expr!，`as`=name，`source`=unit，`to`=unit! |
| `reveal` | `as`=name，`at`=any!，`duration`=expr!，`follow`=unit，`ignore_terrain`=bool，`radius`=expr!，`teams`=enum:own|enemy|all!，`true_sight`=bool |
| `set_ability_active` | `ability`=ability!，`active`=bool!，`of`=unit! |
| `set_ability_level` | `ability`=ability!，`level`=expr!，`of`=unit! |
| `set_base_max_hp` | `to`=unit!，`value`=expr! |
| `set_display_stacks` | `count`=expr!，`status`=status!，`to`=unit! |
| `set_health` | `to`=unit!，`value`=expr! |
| `set_item_charges` | `count`=expr!，`item`=any! |
| `set_mana` | `to`=unit!，`value`=expr! |
| `set_owner` | `owner`=unit!，`unit`=unit! |
| `set_stacks` | `count`=expr!，`status`=status!，`to`=unit! |
| `set_state` | `item`=any，`on`=unit，`value`=any!，`var`=name! |
| `set_time_of_day` | `duration`=expr，`mode`=enum:day|night! |
| `set_var` | `on`=unit，`status`=status，`value`=any!，`var`=name! |
| `spend_mana` | `ability`=ability，`amount`=expr!，`from`=unit! |
| `stop` | （無） |
| `swap_abilities` | `cooldown_policy`=enum:freeze|continue|reset!，`duration`=expr，`from`=ability!，`on`=unit!，`to`=ability! |
| `swap_item` | `a`=any!，`b`=any! |
| `swap_positions` | `a`=unit!，`b`=unit! |
| `take_gold` | `amount`=expr!，`as`=name，`from`=unit! |
| `teleport` | `find_clear_space`=bool，`keep_orders`=bool，`max_distance`=expr，`to`=any!，`who`=unit! |
| `temporary_night` | `duration`=expr! |
| `tether` | `as`=name，`cue`=cue，`duration`=expr!，`from`=unit!，`interval`=expr，`max_distance`=expr，`on_break`=actions，`on_expire`=actions，`on_tick`=actions，`to`=unit! |
| `toggle_ability` | `ability`=ability!，`of`=unit!，`on`=bool! |
| `zone` | `angle`=expr，`as`=name，`at`=any!，`blocks_pathing`=bool，`cue`=cue，`direction`=vec，`duration`=expr!，`first`=expr，`flags`=list:flag，`follow`=unit，`inner`=expr，`interval`=expr，`length`=expr，`on_end`=actions，`on_enter`=actions，`on_leave`=actions，`on_tick`=actions，`outer`=expr，`radius`=expr，`shape`=enum:circle|ring|cone|line|rect!，`team`=enum:enemy|ally|both|any，`types`=list:enum:hero|creep|building|summon|ward|neutral|boss|illusion|all，`vision`=expr，`width`=expr |

## 技能事件（events）

- `on_cast`
- `on_channel_end`：event.interrupted
- `on_channel_tick`：event.tick
- `on_charge_restored`
- `on_equip`
- `on_learn`
- `on_order_targeted`：event.issuer、event.order
- `on_owner_death`
- `on_owner_respawn`
- `on_phase_cancel`
- `on_phase_start`
- `on_toggle_off`
- `on_toggle_on`
- `on_unequip`
- `on_upgrade`

## 狀態觸發（triggers）

- `on_ability_executed`：event.ability、event.point、event.target、event.unit
- `on_attack_cancel`：event.attack、event.attacker、event.target
- `on_attack_hit`：event.attack、event.attacker、event.target
- `on_attack_launch`：event.attack、event.attacker、event.target
- `on_attack_miss`：event.attack、event.attacker、event.target
- `on_attack_windup`：event.attack、event.attacker、event.target
- `on_attacked`：event.attack、event.attacker、event.target
- `on_attacked_hit`：event.attack、event.attacker、event.target
- `on_aura_enter`：event.unit
- `on_aura_leave`：event.unit
- `on_created`
- `on_deal_damage`：event.ability、event.attacker、event.damage、event.item、event.victim
- `on_deal_damage_pre`：event.ability、event.attacker、event.damage、event.item、event.victim
- `on_death`：event.killer
- `on_expired`
- `on_gold_changed`：event.delta
- `on_heal_received`：event.amount、event.healer
- `on_item_used`：event.item、event.unit
- `on_kill`：event.ability、event.victim
- `on_level_up`：event.level
- `on_mana_spent`：event.ability、event.amount
- `on_moved`：event.distance
- `on_order`：event.order
- `on_order_targeted`：event.issuer、event.order
- `on_owner_death`：event.killer
- `on_refreshed`
- `on_removed`：event.reason
- `on_respawn`
- `on_stack_changed`：event.delta
- `on_take_damage`：event.ability、event.attacker、event.damage、event.item、event.victim
- `on_take_damage_pre`：event.ability、event.attacker、event.damage、event.item、event.victim
- `periodic`

事件紀錄欄位：

- `attack`：`attacker`、`crit`、`damage`、`effect_multiplier`、`from_ability`、`is_orb`、`is_ranged`、`missed`、`target`、`triggers_effects`
- `damage`：`after_armor`、`credit`、`final`、`is_ability`、`is_attack`、`is_magical`、`is_physical`、`is_pure`、`is_reflection`、`lethal`、`raw`、`source`、`source_kind`、`type`
- `order`：`ability`、`by_player`、`kind`、`point`、`target`

## 運算式函式

| 函式 | 參數 | 回傳 |
|---|---|---|
| `ability_in_slot` | unit, slot | ability |
| `ability_level` | unit, ability | num |
| `ability_of` | unit, ability | ability |
| `ability_ready` | unit, ability | bool |
| `abs` | num | num |
| `alive` | any | bool |
| `angle_between` | vec, vec | num |
| `armor_multiplier` | num | num |
| `atan2` | num, num | num |
| `avg_attack_against` | unit, unit | num |
| `ceil` | num | num |
| `circle_point` | any, num, num, num | point |
| `clamp` | num, num, num | num |
| `cos` | num | num |
| `count_in_radius` | any, num, team, tag | num |
| `direction` | any, any | vec |
| `distance` | any, any | num |
| `edge_distance` | unit, unit | num |
| `enemy_team` | unit | team |
| `exists` | any | bool |
| `floor` | num | num |
| `has_item` | unit, item_name | bool |
| `has_state` | unit, state_flag | bool |
| `has_status` | unit, status | bool |
| `has_status_tag` | unit, status_tag | bool |
| `has_tag` | any, tag | bool |
| `hook_value` | name, any, ... | any |
| `in_circle` | any, any, num | bool |
| `in_cone` | any, any, vec, num, num | bool |
| `is_ability` | any, name | bool |
| `is_ally` | unit, unit | bool |
| `is_enemy` | unit, unit | bool |
| `is_hero_id` | unit, name | bool |
| `is_illusion_of` | unit, unit | bool |
| `is_item` | any, item_name | bool |
| `is_template` | unit, template | bool |
| `is_visible_to` | any, unit | bool |
| `item_count` | unit, item_name | num |
| `landmark` | team, landmark | unit |
| `lerp_point` | any, any, num | point |
| `max` | num, num, ... | num |
| `min` | num, num, ... | num |
| `normalize` | vec | vec |
| `offset` | any, vec, num | point |
| `orb_slot_free` | unit, ability | bool |
| `order_is` | any, order_kind | bool |
| `owner_of` | any | unit |
| `polar` | any, num, num | point |
| `position` | any | point |
| `primary_is` | unit, name | bool |
| `random` | num, num | num |
| `random_int` | num, num | num |
| `random_point` | any, num, num | point |
| `random_point_area` | any, num, num | point |
| `rotate` | vec, num | vec |
| `round` | num | num |
| `select` | bool, any, any | same |
| `sin` | num | num |
| `sqrt` | num | num |
| `stacks` | unit, status | num |
| `status_remaining` | unit, status | num |
| `status_source` | unit, status | unit |
| `status_var` | unit, status, name | any |
| `team_of` | unit | team |
| `tree_alive` | any | bool |
| `units_owned` | unit, template | num |
| `vec` | num, num | vec |

## 屬性

- 單位（`caster.x`、`it.x`…）：`agi`、`alive`、`armor`、`attack`、`attack_base`、`attack_bonus`、`attack_interval`、`attack_range`、`attack_speed`、`attack_speed_mult`、`attack_target`、`base_agi`、`base_armor`、`base_attack_range`、`base_int`、`base_magic_resist`、`base_move_speed`、`base_str`、`bonus_armor`、`collision_radius`、`deaths`、`evasion`、`forward`、`gold`、`hero`、`hp`、`hp_pct`、`illusion_damage`、`int`、`is_attacking`、`is_building`、`is_casting`、`is_channeling`、`is_creep`、`is_hero`、`is_illusion`、`is_invisible`、`is_invulnerable`、`is_magic_immune`、`is_moving`、`is_ranged`、`is_silenced`、`is_stunned`、`is_summon`、`kills`、`level`、`magic_resist`、`mana`、`mana_pct`、`max_hp`、`max_mana`、`missing_hp`、`missing_mana`、`move_speed`、`owner`、`position`、`primary`、`projectile_speed`、`spell_amp`、`status_resistance`、`str`、`team`
- 技能（`ability.x`）：`cast_count`、`cast_point`、`cast_range`、`charges`、`cooldown`、`cooldown_remaining`、`is_channeling`、`is_item`、`is_learned`、`is_passive`、`is_ready`、`is_toggle`、`is_toggled`、`is_ultimate`、`level`、`mana_cost`、`max_level`、`owner`、`slot_index`
- 狀態（`status.x`）：`display_stacks`、`duration`、`elapsed`、`holder`、`remaining`、`source`、`stacks`
- 道具：`alive`、`charges`、`cooldown_remaining`、`cost`、`level`、`name`、`owner`、`purchaser`、`sellable`、`slot`
- 對局（`game.x`）：`creep_armor_bonus`、`creep_wave`、`dt`、`is_night`、`score_dire`、`score_radiant`、`tick`、`time`
- 投射物：`alive`、`path_length`、`position`、`source`、`traveled`
- 位置：`x`、`y`

## StatKey（stats）

`agi`、`all_stats`、`armor`、`attack_damage`、`attack_damage_pct`、`attack_range`、`attack_speed`、`base_attack_damage`、`base_attack_damage_pct`、`base_attack_time`、`base_move_speed`、`bash_chance`、`bonus_gold_pct`、`bonus_xp_pct`、`cast_point_reduction_pct`、`cast_range`、`cooldown_reduction_pct`、`detection_level`、`evasion_pct`、`heal_amp_pct`、`heal_reduction_pct`、`hp_regen`、`hp_regen_amp_pct`、`hp_regen_pct`、`incoming_damage_pct`、`incoming_magical_pct`、`incoming_physical_pct`、`int`、`invisibility_level`、`lifesteal_pct`、`magic_resist`、`magic_resist_pool`、`mana_cost_reduction_pct`、`mana_regen`、`mana_regen_amp_pct`、`mana_regen_pct`、`max_hp`、`max_mana`、`miss_chance_pct`、`model_scale`、`move_speed`、`move_speed_min`、`move_speed_pct`、`outgoing_damage_pct`、`physical_block`、`physical_block_chance`、`primary_stat`、`slow_pct`、`slow_resistance_pct`、`spell_amp_pct`、`spell_lifesteal_pct`、`status_resistance_pct`、`str`、`true_strike_chance`、`turn_rate`、`vision_day`、`vision_night`

stat 運算子（`{ op, value, when, per_stack }`）：`add`、`max`、`min`、`mul`、`pct`、`set_absolute`

## 控制與狀態旗標（states）

`attack_immune`、`attack_slow_immune`、`blind`、`block_disabled`、`break`、`cannot_be_healed`、`cannot_be_magic_immune`、`cannot_miss`、`command_restricted`、`disarm`、`evade_disabled`、`fear`、`flying`、`flying_for_pathing`、`frozen`、`hex`、`hidden`、`ignore_collision_trees`、`invisible`、`invulnerable`、`low_attack_priority`、`magic_immune`、`melee_attack`、`mute`、`no_attack`、`no_cast`、`no_healthbar`、`no_unit_collision`、`not_on_minimap`、`out_of_game`、`passives_disabled`、`physical_immune`、`provides_vision`、`ranged_attack`、`revealed`、`root`、`silence`、`sleep`、`stun`、`taunt`、`true_sight`、`unselectable`、`untargetable_by_spells`

## 選取

- 形狀：`circle`(radius)、`cone`(angle, length)、`global`()、`line`(length, width)、`rect`(length, width)、`ring`(inner, outer)、`segment`(to, width)、`unit`()
- team：`ally`、`ally_not_self`、`any`、`both`、`enemy`、`self`
- types：`all`、`base`、`boss`、`building`、`courier`、`creep`、`dummy`、`fountain`、`general`、`hero`、`illusion`、`item_on_ground`、`neutral`、`summon`、`tower`、`tree`、`ward`
- flags：`exclude_illusions`、`exclude_invisible`、`exclude_self`、`heroes_first`、`ignore_fog`、`include_dead`、`include_hidden`、`include_illusions`、`include_invulnerable`、`include_magic_immune`、`include_out_of_game`、`include_untargetable`、`visible_only`
- order：`farthest`、`highest_attack`、`highest_hp`、`highest_hp_pct`、`id`、`lowest_hp`、`lowest_hp_pct`、`most_recent`、`nearest`、`random`
- 地標（landmark）：`barracks`、`base`、`fort`、`fountain`、`shop`、`spawn`、`tower`；forced_move.locks：`all`、`attack`、`cast`、`hold`、`item`、`move`、`stop`

## 其他列舉

- slot：`D`、`E`、`F`、`Q`、`R`、`W`、`hidden`、`sub`；kind：`active`、`autocast`、`channel`、`passive`、`toggle`；tags：`attack_modifier`、`aura`、`basic`、`channel`、`item`、`movement`、`no_cast_trigger`、`no_refresh`、`orb`、`sub`、`sub_ability`、`summon`、`toggle`、`ultimate`
- target.type：`direction`、`item`、`none`、`point`、`tree`、`unit`、`unit_or_point`、`vector`；技能 flags：`break_invisibility`、`can_target_buildings`、`can_target_trees`、`dispellable_cast`、`does_not_break_invisibility`、`hidden_cooldown`、`ignore_cast_range_bonus`、`ignore_invulnerable`、`no_target_self`、`not_on_illusions`、`pierce_magic_immune`、`self_cast_allowed`
- cooldown_start：`manual`、`on_cast`、`on_channel_end`、`on_end`、`on_toggle_off`
- stacking：`extend`、`independent`、`keep`、`refresh`、`replace` 或 `{ max, mode, value_stacks, per_stack_duration, decay, initial }`
- dispel：`basic`、`death_only`、`none`、`strong`；visibility：`buff`、`debuff`、`hidden`；status scope：`hero`、`item_instance`、`permanent`、`unit`
- unit_state／vars type：`ability`、`bool`、`fixed`、`int`、`item`、`item_name`、`point`、`tick`、`unit`、`unit_list`、`vec`；scope：`ability`、`hero`、`item_instance`、`item_name`、`match`、`permanent`、`unit`
- 單位模板 ai：`attack_move`、`chase_target`、`follow_owner`、`guard`、`idle`、`none`、`owner_controlled`、`wander`；over_limit：`none`、`orphan_oldest`、`reject`、`remove_oldest`；stats：`agi`、`armor`、`hp`、`hp_regen`、`int`、`magic_resist`、`mana`、`mana_regen`、`move_speed`、`str`
- coverage：`deferred`、`expressible`、`needs_hook`、`pending`、`rust_handler`；confidence：`high`、`low`、`medium`
- 規則層暫列（M2 會移進 game_rules.lua）：`rules.damage.require_source_alive`
