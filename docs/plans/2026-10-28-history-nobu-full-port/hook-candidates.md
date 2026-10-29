# hook 候選清單（工作線 A，2026-10-30，依 codegen-coverage.md §5）

門檻（D7，m3 §2.4）：英雄技能與道具合計只有 1 招用到的寫法。分兩類：

- **A. 產生器自己補**：運算式、函式、屬性、選取等，不是機制，A 會直接在產生器補上，不寫 hook。
- **B. 機制候選**：真的只有 1 招有的機制，建議在 v3 標 `hook = "<名稱>"`（hooks 宣告 kind = "ability"、why、test、cases），交給 hook 工作線手寫 `src/hooks/<招式>.rs`。同一招若還卡在其他「共用」寫法上，可以等共用補完再判斷要不要整招 hook。

A 類 57 項、B 類 62 項。

## B. 機制候選（建議 hook）

| 招式 | 英雄 | 獨有的機制 |
|---|---|---|
| `a03_spider_form` | A03 ishikawa_kazumasa | `state:melee_attack` |
| `a04_frost_shelter` | A04 takenaka_shigeharu | `status_trigger:on_refreshed` |
| `a06_red_fury` | A06 ii_naomasa | `status_trigger:on_attack_miss` |
| `a07_red_thunder` | A07 honda_tadakatsu | `action:delay in a routine` |
| `a07_strongest` | A07 honda_tadakatsu | `action:remove_status.states`、`stat:slow_resistance_pct` |
| `a09_lignify` | A09 hachisuka_masakatsu | `state:block_disabled`、`state:hex` |
| `a11_vortex` | A11 saito_dosan | `action:damage.credit` |
| `a13_chain_sickle` | A13 hattori_hanzo | `action:projectile.arm_distance`、`action:projectile.hit_priority`、`action:projectile.kind:path`、`action:projectile.path`、`action:projectile.remove_on`、`action:projectile.team:both`、`action:projectile.where` |
| `a20_pirate_daimyo` | A20 kuki_yoshitaka | `status.unique_group` |
| `a20_pirate_domain` | A20 kuki_yoshitaka | `action:remove_status.limit` |
| `a22_torment` | A22 sengoku_hidehisa | `status.reapply_if_removed` |
| `a24_blood_charge` | A24 sakuma_morimasa | `action:damage.flags:hp_removal` |
| `a29_withering_aura` | A29 ishida_mitsunari | `status_trigger:on_mana_spent` |
| `a32_phantom_blade` | A32 takigawa_kazumasu | `state:physical_immune` |
| `a32_soul_blade` | A32 takigawa_kazumasu | `passive_trigger:on_deal_damage_pre` |
| `b02_eighteen_shuriken` | B02 fuma_kotaro | `action:projectile.from:polar(tgt, $orbit_step * (k + 1), $orbit_radius)` |
| `b04_flame_trail_dash` | B04 date_masamune | `action:dash` |
| `b06_mind_eye` | B06 sanada_yukimura | `passive_trigger:on_order_targeted` |
| `b08_endless_fang` | B08 asai_nagamasa | `action:projectile.from:spot` |
| `b16_beast_union` | B16 mochizuki_chiyome | `action:set_base_max_hp`、`token:state` |
| `b21_undying_demon` | B21 baba_nobufusa | `passive_trigger:on_heal_received` |
| `b22_force_of_nature` | B22 taigen_sessai | `action:destroy_tree` |
| `b22_purify` | B22 taigen_sessai | `status.immune_to` |
| `b22_wrath_of_nature` | B22 taigen_sessai | `action:regrow_trees`、`action:remove_status.except`、`action:remove_status.exclude_tags` |
| `b23_living_soul_shock` | B23 ise_hime | `action:projectile.from:g` |
| `b23_undead_pact` | B23 ise_hime | `passive_trigger:on_moved` |
| `b24_earth_prison` | B24 akiyama_nobutomo | `state:cannot_miss` |
| `b24_taunt` | B24 akiyama_nobutomo | `state:taunt` |
| `b25_asura_hell` | B25 honganji_kennyo | `action:impulse` |
| `b28_flame_shackle` | B28 matsunaga_hisahide | `status_trigger:periodic_2` |
| `b28_serpent_fire` | B28 matsunaga_hisahide | `action:chain` |
| `b31_still_forest` | B31 takeda_shingen | `action:projectile.from:spawn` |
| `c06_grand_heist` | C06 ishikawa_goemon | `action:create_pickup` |
| `c06_sleight_of_hand` | C06 ishikawa_goemon | `action:give_gold`、`action:remove_item` |
| `c08_devour` | C08 kato_danzo | `action:apply_status.stacks on a status without counted stacks`、`action:remove_ability`、`action:set_stacks on a status without counted stacks`、`state:attack_immune`、`state:blind` |
| `c10_echo_sword` | C10 kosokabe_chikayasu | `action:damage.flags:ignore_armor` |
| `c13_gale_grip` | C13 ryuzoji_takanobu | `status_trigger:on_kill` |
| `c13_tornado_vortex` | C13 ryuzoji_takanobu | `action:modify_cooldown.item_tag` |
| `c14_dragon_form` | C14 saito_yoshitatsu | `state:attack_slow_immune` |
| `c18_lone_rider` | C18 kobayakawa_takakage | `action:modify_cooldown.item` |
| `c19_inviolable` | C19 matsuhime | `state:untargetable_by_spells` |
| `item_death_of_scepter` | 道具 | `stat:spell_lifesteal_pct` |
| `item_the_dark_yellow_teeth_and_zimbabwe` | 道具 | `action:damage.type:event`、`event.damage.raw` |
| `item_the_great_sword_of_invincible` | 道具 | `action:modify_cooldown without amount` |
| `item_the_over_lightning_art_of_war` | 道具 | `stat:cast_point_reduction_pct` |

## A. 產生器自己補（不是 hook）

| 寫法 | 招式 | 英雄 |
|---|---|---|
| `state type:vec` | `a04_ice_path` | A04 takenaka_shigeharu |
| `attr:attack_interval` | `a12_mystic_arrow` | A12 otani_yoshitsugu |
| `fn:ability_of` | `a12_mystic_possession` | A12 otani_yoshitsugu |
| `expr: a position from a sym:vector_end` | `a13_chain_sickle` | A13 hattori_hanzo |
| `target.type:vector` | `a13_chain_sickle` | A13 hattori_hanzo |
| `action:cue.at of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: * of num and sym:me_slot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: + of num and sym:rot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: wanted a unit, got a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `expr: ~= of num and sym:me_slot` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `fn:exists of a sym:il` | `a13_shadow_illusion` | A13 hattori_hanzo |
| `selector.types:courier+creep+hero+neutral+summon+ward` | `a16_dark_blood_decree` | A16 torii_mototada |
| `action:for_each.notes` | `a21_pearl_disc` | A21 zuiryuin_nisshu |
| `expr: wanted a unit, got a point` | `a26_scorching_bomb` | A26 nohime |
| `fn:random` | `a27_unstoppable_advance` | A27 shibata_katsuie |
| `expr: wanted a bool, got a sym:heal_chain` | `a28_creation` | A28 hashiba_hideyoshi |
| `attr:missing_hp` | `a32_soul_drain` | A32 takigawa_kazumasu |
| `fn:alive of a sym:caster_hero` | `a36_detonate` | A36 maeno_nagayasu |
| `selector.flag:exclude_invisible` | `b03_zhongkui_judgment` | B03 saito_tomonobu |
| `fn:exists of a sym:item_a` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `fn:exists of a sym:item_b` | `b07_battle_line` | B07 ashikaga_yoshiaki |
| `action:if.notes` | `b11_interception` | B11 kikuhime |
| `fn:status_remaining` | `b11_interception` | B11 kikuhime |
| `fn:enemy_team` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `fn:team_of` | `b12_reality_slide` | B12 yamamoto_kansuke |
| `action:cue.at of a sym:caster_hero` | `b16_mm_phase_shift` | B16 mochizuki_chiyome |
| `selector.types:building+creep+neutral+summon` | `b18_earth_claw` | B18 koihime |
| `fn:tree_alive` | `b22_wood_spirit` | B22 taigen_sessai |
| `selector.team:ally (types building)` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:boss+creep+neutral+summon` | `b24_earth_stele` | B24 akiyama_nobutomo |
| `selector.types:building` | `b24_shatter` | B24 akiyama_nobutomo |
| `attr:cooldown_remaining` | `b25_exorcism_sutra` | B25 honganji_kennyo |
| `attr:is_invisible` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `fn:has_state:revealed` | `b28_serpent_fire` | B28 matsunaga_hisahide |
| `attr:max_mana` | `b30_flash_strike` | B30 kakizaki_kageie |
| `selector.team:ally (apply_status)` | `b31_swift_wind` | B31 takeda_shingen |
| `fn:alive of a err` | `b33_spider_attack` | B33 mogami_yoshihime |
| `selector.flag:exclude_illusions` | `b36_heart_fence` | B36 naoe_kanetsugu |
| `action:cue.at of a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `expr: a position from a sym:landing` | `c01_honnoji` | C01 akechi_mitsuhide |
| `attr:name` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `fn:exists of a sym:stolen` | `c06_sleight_of_hand` | C06 ishikawa_goemon |
| `fn:is_ability of a non-event ability` | `c08_devour` | C08 kato_danzo |
| `expr: wanted a vec, got a sym:dir` | `c08_kite_chain` | C08 kato_danzo |
| `selector.shape:segment` | `c08_kite_chain` | C08 kato_danzo |
| `selector.to` | `c08_kite_chain` | C08 kato_danzo |
| `selector.width` | `c08_kite_chain` | C08 kato_danzo |
| `attr:remaining` | `c09_ice_soul_dance` | C09 tachibana_ginchiyo |
| `fn:exists of a sym:blink_item` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:status_source` | `c18_lone_rider` | C18 kobayakawa_takakage |
| `fn:ability_ready` | `c18_twin_chariot` | C18 kobayakawa_takakage |
| `fn:count_in_radius` | `c21_soaring_quake` | C21 miyamoto_musashi |
| `attr:illusion_damage` | `item_mana_thief` | 道具 |
| `action:remove_status.notes` | `item_the_great_sword_of_invincible` | 道具 |
| `attr:armor` | `item_the_spear_of_saddle` | 道具 |
| `attr:attack_speed_mult` | `item_wayumi2` | 道具 |
| `attr:projectile_speed` | `item_wayumi2` | 道具 |

## 複核（2026-10-30，「類似就共用」）

依主 session 2026-10-30 指示，B 類 45 招以機制判斷重新分類：規則層共通概念歸規則層（B）；和其他招類似的併成共用寫法；只剩真正獨有的標 hook。

| 招式 | 最終分類 | 負責 | 理由 |
|---|---|---|---|
| `a03_spider_form` | 規則層 | B | `state:melee_attack` 與 B04T 的 `state:ranged_attack` 同一種「切換攻擊方式」，做成規則層狀態 |
| `a04_frost_shelter` | 共用（觸發事件） | B＋A | `on_refreshed`：狀態被重新套上的事件，和其他 status 觸發同一套；B 在 BuffStore 重套時發事件，A 接產生器 |
| `a06_red_fury` | 共用（觸發事件） | B＋A | `on_attack_miss`：普攻事件家族（hit／miss／launch／windup）的一員 |
| `a07_red_thunder` | 共用（產生器） | A | `delay in a routine`：routine 帶 owner 時排程記在 owner 招式的步驟表，產生器補 |
| `a07_strongest` | 規則層＋共用 | B | `stat:slow_resistance_pct` 是規則層屬性；`remove_status.states` 與 dispel／except／exclude_tags 同一組「移除狀態」選項 |
| `a09_lignify` | 規則層 | B | `state:hex`、`state:block_disabled` 是規則層控制狀態（變形、無法格擋） |
| `a11_vortex` | 規則層 | B | `damage.credit`：擊殺歸屬，傷害管線的封包欄位 |
| `a13_chain_sickle` | hook | hook 工作線 | 鎖鏈鐮的路徑投射物（arm_distance、hit_priority、沿路徑、命中後拉回、敵我皆可）只有這招，組不出來 |
| `a20_pirate_daimyo` | 規則層 | B | `status.unique_group`：同組狀態只取一個，BuffStore 規則 |
| `a20_pirate_domain` | 共用 | B | `remove_status.limit`：移除狀態選項，和 dispel／except 同一組 |
| `a22_torment` | 規則層 | B | `status.reapply_if_removed`：被驅散後重新套上，BuffStore 規則 |
| `a24_blood_charge` | 規則層 | B | `damage.flags:hp_removal`：生命移除（不觸發吸收、不算傷害事件），傷害管線旗標 |
| `a29_withering_aura` | 共用（觸發事件） | B＋A | `on_mana_spent`：耗魔事件，與 on_ability_executed 同一族 |
| `a32_phantom_blade` | 規則層 | B | `state:physical_immune`：物理免疫，與魔免同一層 |
| `a32_soul_blade` | 共用（產生器） | A | 被動的 `on_deal_damage_pre`：與狀態的 pre 觸發同一個 host 掛點，產生器讓被動也能用（以 learn marker 當標記） |
| `b02_eighteen_shuriken` | 共用 | B＋A | `projectile.from` 任意起點：B02、B08、B23、B31 都要從非施法者位置發射；host 投射物加起點，A 接產生器 |
| `b04_flame_trail_dash` | 共用 | B＋A | `dash` 與 B29E `leap`、B25 `impulse` 都是位移；併成一種位移元件 |
| `b06_mind_eye` | 共用（觸發事件） | B＋A | `on_order_targeted`：被指定為技能目標的事件，屬指令事件家族 |
| `b08_endless_fang` | 共用 | B＋A | `projectile.from:spot`，同 B02 |
| `b16_beast_union` | 共用（產生器） | A | `set_base_max_hp` 併進 hero_command（與 set_health 同族）；`token:state` 是連帶 |
| `b21_undying_demon` | 共用（觸發事件） | B＋A | `on_heal_received`：受治療事件，與受傷事件同族 |
| `b22_force_of_nature` | 共用 | line-gamemode＋A | `destroy_tree` 與 `destroy_trees`、`regrow_trees` 同一組樹的操作 |
| `b22_purify` | 規則層 | B | `status.immune_to`：狀態免疫標籤，BuffStore 規則 |
| `b22_wrath_of_nature` | 共用 | line-gamemode＋B | `regrow_trees` 同 B22 樹；`remove_status.except`／`exclude_tags` 同 A07T 移除選項 |
| `b23_living_soul_shock` | 共用 | B＋A | `projectile.from`，同 B02 |
| `b23_undead_pact` | 共用（觸發事件） | B＋A | `on_moved`：移動事件；A05、C12 的距離斷開也要 |
| `b24_earth_prison` | 規則層 | B | `state:cannot_miss`：必中，攻擊結算規則 |
| `b24_taunt` | 規則層 | B | `state:taunt`：嘲諷，強制攻擊目標，控制狀態 |
| `b25_asura_hell` | 共用 | B＋A | `impulse` 是位移，併入 dash／leap 元件 |
| `b28_flame_shackle` | 共用（產生器） | A | `periodic_2`：同一狀態第二個週期觸發，產生器支援多個 periodic |
| `b28_serpent_fire` | 共用 | B＋A | `chain`：連鎖彈跳，與第 9 批的連鎖、host 投射物 bounce 同一種 |
| `b31_still_forest` | 共用 | B＋A | `projectile.from:spawn`，同 B02 |
| `c06_grand_heist` | hook | hook 工作線 | `create_pickup`（地上可撿的金錢）只有這招 |
| `c06_sleight_of_hand` | 規則層 | B | `give_gold`／`remove_item`：經濟與背包操作，和 FC-129～131 同一處 |
| `c08_devour` | 共用（資料）＋規則層 | A＋B | 層數寫法：v3 補宣告 stacking 表（A）；`remove_ability` 與 swap_abilities 同族；`attack_immune`、`blind` 是規則層狀態（B） |
| `c10_echo_sword` | 規則層 | B | `damage.flags:ignore_armor`：傷害管線旗標 |
| `c13_gale_grip` | 共用（觸發事件） | B＋A | `on_kill`：擊殺事件，道具也有 |
| `c13_tornado_vortex` | 共用 | B＋A | `modify_cooldown.item_tag` 與 C18 `modify_cooldown.item` 同一種：道具冷卻；host 道具冷卻（B），產生器（A） |
| `c14_dragon_form` | 規則層 | B | `state:attack_slow_immune`：攻速緩速免疫 |
| `c18_lone_rider` | 共用 | B＋A | `modify_cooldown.item`，同 C13 |
| `c19_inviolable` | 規則層 | B | `state:untargetable_by_spells`：不能被技能指定，施放准入 |
| `item_death_of_scepter` | 規則層 | B | `stat:spell_lifesteal_pct`：技能吸血屬性 |
| `item_the_dark_yellow_teeth_and_zimbabwe` | 共用 | B＋A | 傷害事件帶原始量與類型（`event.damage.raw`、`damage.type:event`）：反傷類共用，host 事件補欄位 |
| `item_the_great_sword_of_invincible` | 共用（產生器） | A | `modify_cooldown` 沒寫 amount：預設用這件道具自己的冷卻值 |
| `item_the_over_lightning_art_of_war` | 規則層 | B | `stat:cast_point_reduction_pct`：前搖縮短屬性 |

合計：hook 2 招（a13_chain_sickle、c06_grand_heist），其餘併入規則層或共用。

### 給 B 的規則層清單

- 狀態：`hex`、`block_disabled`、`physical_immune`、`cannot_miss`、`taunt`、`attack_immune`、`blind`、`attack_slow_immune`、`untargetable_by_spells`、`melee_attack`（與 `ranged_attack` 同一組）。
- 屬性：`slow_resistance_pct`、`spell_lifesteal_pct`、`cast_point_reduction_pct`。
- 傷害管線：`damage.flags:hp_removal`、`damage.flags:ignore_armor`、`damage.credit`；傷害事件帶原始量與類型（反傷類）。
- 狀態語意：`status.unique_group`、`status.immune_to`、`status.reapply_if_removed`；`remove_status` 的 `states`／`limit`／`except`／`exclude_tags`。
- 事件來源（A 接產生器）：`on_refreshed`、`on_attack_miss`、`on_mana_spent`、`on_order_targeted`、`on_heal_received`、`on_moved`、`on_kill`。
- 元件（A 接產生器）：位移（dash／leap／impulse 一種）、投射物任意起點、連鎖彈跳、道具冷卻（`modify_cooldown.item`／`item_tag`）、`remove_ability`（與 swap_abilities 同族）、經濟與背包（give_gold／remove_item，與 FC-129～131 同處）。
- 樹（line-gamemode）：`destroy_tree`、`destroy_trees`、`regrow_trees`。

### A 直接補的產生器寫法

routine 裡的 `delay`、多個 `periodic`、被動的 `on_deal_damage_pre`、`set_base_max_hp`（hero_command）、`modify_cooldown` 沒寫 amount 時用自己的冷卻、C08 的層數宣告（v3 資料）。
