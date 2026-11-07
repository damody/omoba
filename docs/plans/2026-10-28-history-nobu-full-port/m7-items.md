# M7 道具解鎖順序

由 `tools\lua\lua.exe docs\character_pipeline\tools\v3_item_order.lua` 產生，不要手改。掃描規則與 [ir-blockers.md](ir-blockers.md) 相同（ir.rs 的載入規則）；B 每放回一批就和 ir-blockers 一起重跑。

- 道具共 167 件：45 件已由原生道具引擎上架（`templates/item_shop_v3.lua`，價錢、配方與屬性都由 v3 values 產生），11 件沒有 IR（deferred 或沒有行為）。
- 直譯器今天就能完整載入：30 件。只有屬性加成（passive.stats，沒有主動、狀態與觸發）的道具：26 件。

## 1. 實作順序

每一步選「多支援這一項就能完整載入最多道具」的寫法；同分時選讓最多道具只差一項的。

| # | 要支援的寫法 | 這一步解鎖 | 累計 |
|---|---|---|---|
| 1 | `action:repeat` | 5：林之卷、鬼丸國綱．銘刀、大炎上之書、天雷之書、風之卷 | 35 |
| 2 | `passive_trigger:on_created` | 4：般若鬼丸、禦魔鋼、褶裙（已上架）、掛面具足 | 39 |
| 3 | `passive_trigger:on_removed` | 6：鱗片甲、短甲、忍者裝束、蜻蛉切．銘槍、韌性護符、和弓 | 45 |
| 4 | `status.triggers` | 4：火計之書（已上架）、忍者刀、日光助真、闇牙黃泉津 | 49 |
| 5 | `action:projectile` | 4：吹箭（已上架）、統率者鐵扇、桔梗仙冬月、手裏劍（已上架） | 53 |
| 6 | `stat:slow_pct` | 3：凍牙輪、法術書（已上架）、雪走 | 56 |
| 7 | `passive:aura` | 3：制魔腹卷、孫子兵法．風之章、蜘蛛切 | 59 |
| 8 | `passive_trigger:periodic` | 3：狐火大鎧、御魔護符、御守 | 62 |
| 9 | `action:chance` | 2：舞雷刀、長船．太刀 | 64 |
| 10 | `passive_trigger:on_attack_windup` | 2：野太刀、武尊環 | 66 |
| 11 | `event:on_unequip` | 1：怒雷之書 | 67 |
| 12 | `fn:has_status_tag` | 1：山之卷 | 68 |
| 13 | `reveal.teams:own` | 1：落石之書（已上架） | 69 |
| 14 | `selector.flag:visible_only` | 2：大千鳥十文字槍、天變之書 | 71 |
| 15 | `action:create_unit.owner` | 1：鐵碎牙．妖刀 | 72 |
| 16 | `action:delay` | 1：星杵 | 73 |
| 17 | `action:face` | 1：狂風之書 | 74 |
| 18 | `action:knockback` | 1：水計之書 | 75 |
| 19 | `action:restore_mana` | 1：軍糧丸（已上架） | 76 |
| 20 | `fn:is_ally` | 1：戒杖刀 | 77 |
| 21 | `passive_trigger:on_take_damage_pre` | 1：浪人十手 | 78 |
| 22 | `selector.exclude` | 1：凍雲 | 79 |
| 23 | `selector.exclude_types` | 1：爆烈彈（已上架） | 80 |
| 24 | `stat:cast_point_reduction_pct` | 1：無上雷暴之書 | 81 |
| 25 | `stat:heal_reduction_pct` | 1：薩摩銃 | 82 |
| 26 | `stat:outgoing_damage_pct` | 1：正宗．武士刀 | 83 |
| 27 | `status.stat.when` | 1：謙信助宗 | 84 |
| 28 | `ally_selector.effect:apply_status` | 0： | 84 |
| 29 | `ally_selector.types` | 6：火之卷（已上架）、琉璃光姬、療癒圖騰、紅雪左文字、皆朱槍（已上架）、孫子兵法．火之章（已上架） | 90 |
| 30 | `status.aura` | 1：黑漆塗五枚胴 | 91 |
| 31 | `selector.limit` | 0： | 91 |
| 32 | `selector.order` | 2：雷之卷、雷切 | 93 |
| 33 | `state:mute` | 1：赤雷卷 | 94 |
| 34 | `passive_trigger:on_kill` | 1：紫雷卷 | 95 |
| 35 | `status.stacking:independent` | 1：鬼撤．妖刀 | 96 |
| 36 | `action:create_unit.overrides` | 0： | 96 |
| 37 | `token:state` | 1：孫子兵法．林之章 | 97 |
| 38 | `action:set_state` | 0： | 97 |
| 39 | `fn:exists` | 2：下忍、中忍 | 99 |
| 40 | `target.flag:include_magic_immune` | 1：靈都宮燈 | 100 |
| 41 | `fn:has_status` | 1：月輪鏡 | 101 |
| 42 | `attr:base_attack_range` | 1：鬼蜘蛛．夜哭 | 102 |
| 43 | `selector.types:building+hero+tower` | 1：當世具足 | 103 |
| 44 | `fn:is_hero_id` | 0： | 103 |
| 45 | `action:apply_status.source` | 1：梅鶯毒．妖刀 | 104 |
| 46 | `attr:effect_multiplier` | 1：鬼切 | 105 |
| 47 | `stat:all_stats` | 0： | 105 |
| 48 | `fn:has_item` | 1：孫子兵法殘冊 | 106 |
| 49 | `ally_selector.effect:if` | 1：孫子兵法全本 | 107 |
| 50 | `passive.stat.when` | 1：葵紋越前康繼．禦神刀 | 108 |
| 51 | `selector.types:building+creep+hero+neutral+summon` | 1：孫子兵法．山之章 | 109 |
| 52 | `selector.types:building+creep+hero+neutral+summon+tower` | 1：孫子兵法斷簡 | 110 |
| 53 | `event:on_owner_death` | 0： | 110 |
| 54 | `state:revealed` | 2：真視碎片、洞察碎片 | 112 |
| 55 | `selector.types:building+creep+neutral+summon` | 1：真知碎片 | 113 |
| 56 | `passive_trigger:on_attacked_hit` | 0： | 113 |
| 57 | `stat:vision_night` | 3：十字槍、真視赤備挂甲、真知短甲 | 116 |
| 58 | `action:teleport` | 0： | 116 |
| 59 | `action:attack` | 1：長曾彌虎撤．銘刀 | 117 |
| 60 | `action:issue_order` | 1：神直毘御劍 | 118 |
| … | 其餘見 §3 | | |

## 2. 只有屬性加成的道具

這些道具只要 stat 列對應得到 StatKey（rules/vocab.rs `STATS`）就能由資料執行。

| 道具 | 已上架 | 還缺的 stat |
|---|---|---|
| 迅捷鋼（`item_agility_ingot`） | 是 | —（今天就能載入） |
| 攻速鋼（`item_attack_speed_ingot`） | 是 | —（今天就能載入） |
| 胴丸（`item_body_wrap`） | 是 | —（今天就能載入） |
| 妖刀（`item_cursed_katana`） | 是 | —（今天就能載入） |
| 能量鋼（`item_energy_ingot`） | 是 | —（今天就能載入） |
| 陣太刀（`item_gorgeous_tachi`） | 是 | —（今天就能載入） |
| 生命鋼（`item_health_ingot`） | 是 | —（今天就能載入） |
| 技巧鋼（`item_intelligence_ingot`） | 是 | —（今天就能載入） |
| 打刀（`item_katana`） | 是 | —（今天就能載入） |
| 足輕盾（`item_lesser_shield`） | 是 | —（今天就能載入） |
| 法力鋼（`item_mana_ingot`） | 是 | —（今天就能載入） |
| 斬月槍（`item_moon_spear`） | 是 | —（今天就能載入） |
| 薙刀（`item_naginata`） | 是 | —（今天就能載入） |
| 南蠻胴具足（`item_nannbann_armor`） | 否 | —（今天就能載入） |
| 強化鋼（`item_power_ingot`） | 是 | —（今天就能載入） |
| 再生鋼（`item_regeneration_ingot`） | 是 | —（今天就能載入） |
| 武士服（`item_samurai_cloth`） | 否 | —（今天就能載入） |
| 長槍（`item_spear`） | 是 | —（今天就能載入） |
| 疾風衣（`item_speed_clothes`） | 是 | —（今天就能載入） |
| 回魔鋼（`item_spirit_ingot`） | 是 | —（今天就能載入） |
| 番頭盾（`item_stable_shield`） | 是 | —（今天就能載入） |
| 杵杖（`item_staff`） | 是 | —（今天就能載入） |
| 蠻力鋼（`item_strength_ingot`） | 是 | —（今天就能載入） |
| 太刀（`item_tachi`） | 是 | —（今天就能載入） |
| 茶碗（`item_tea_bowl`） | 是 | —（今天就能載入） |
| 鬼頰當（`item_the_hooate_of_ghost`） | 是 | —（今天就能載入） |

## 3. 每件道具的阻擋項

| 道具 | 已上架 | 阻擋項 |
|---|---|---|
| 迅捷鋼（`item_agility_ingot`） | 是 | — |
| 久棄邪物（`item_ancient_doll`） | 否 | 沒有 IR |
| 攻速鋼（`item_attack_speed_ingot`） | 是 | — |
| 野太刀（`item_big_tachi`） | 否 | action:chance、passive_trigger:on_attack_windup |
| 備前兼光（`item_bizennosafunekanemitsu`） | 否 | action:apply_status.stacks、action:set_stacks、action:set_state、attr:is_toggle、fn:has_status、stat:slow_pct、status.stacking:stacks、status.triggers、target.flag:include_magic_immune |
| 吹箭（`item_blowgun`） | 是 | action:projectile |
| 胴丸（`item_body_wrap`） | 是 | — |
| 武尊環（`item_brave_ring`） | 否 | passive_trigger:on_attack_windup |
| 大盜的贈禮（`item_c06e`） | 否 | action:damage.source、action:give_item、action:remove_item、attr:is_magic_immune、status.triggers、token:state |
| 千鳥一文字（`item_chidori`） | 否 | action:add_stacks、action:apply_status.stacks、action:delay、attr:is_illusion、fn:has_status、status.stacking:stacks、status.stat.when、status.triggers |
| 統率者鐵扇（`item_commander_of_fan`） | 否 | action:projectile |
| 當世具足（`item_contemporary_armor`） | 否 | action:restore_mana、ally_selector.effect:apply_status、ally_selector.types、fn:has_status、passive_trigger:on_created、passive_trigger:on_removed、selector.types:building+hero+tower |
| 十字槍（`item_cross_spear`） | 否 | passive:aura、passive_trigger:on_attacked_hit、passive_trigger:on_removed、selector.types:building+creep+hero+neutral+summon+tower、stat:vision_night、state:revealed |
| 妖刀（`item_cursed_katana`） | 是 | — |
| 暗部長靴（`item_dark_boots`） | 是 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 破魔．暗部長靴（`item_dark_boots_1`） | 是 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 逝者之笏（`item_death_of_scepter`） | 否 | action:set_display_stacks、passive_trigger:on_created、passive_trigger:on_death、passive_trigger:on_kill、stat:spell_lifesteal_pct、status.scope、status.stacking:independent、token:state |
| 制魔腹卷（`item_devil_supressor_armor`） | 否 | passive:aura、passive_trigger:on_created |
| 忍法．土遁（`item_earth_wall_book`） | 否 | action:repeat、fn:rotate、selector.team:both |
| 能量鋼（`item_energy_ingot`） | 是 | — |
| 火之卷（`item_fire_scroll`） | 是 | ally_selector.effect:apply_status、ally_selector.types |
| 鱗片甲（`item_flame_armor`） | 否 | passive_trigger:on_created、passive_trigger:on_removed |
| 閃光輪（`item_flash_ring`） | 否 | action:dodge_projectiles、action:modify_cooldown、action:teleport |
| 百里長靴（`item_flash_shoes`） | 否 | action:dodge_projectiles、action:modify_cooldown、action:teleport |
| 水計之書（`item_flood_book`） | 否 | action:knockback、action:repeat、stat:slow_pct |
| 林之卷（`item_forest_scroll`） | 否 | action:repeat |
| 狐火大鎧（`item_foxfire_armor`） | 否 | passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:periodic |
| 冰爆之書（`item_frozen_book`） | 否 | action:remove_status.dispel、action:remove_status.exclude_source_tags、action:remove_status.min_duration、action:remove_status.team_of_source |
| 凍牙輪（`item_frozen_ring`） | 否 | stat:slow_pct |
| 吹雪護肩（`item_fubuki_shoulders`） | 否 | action:reveal.ignore_terrain、passive_trigger:on_take_damage、passive_trigger:periodic、reveal.teams:own、stat:slow_pct、status.stat.when、status.triggers、status.vars |
| 鬼之籠手（`item_ghost_gloves`） | 否 | 沒有 IR |
| 鬼面（`item_ghost_mask`） | 否 | attr:bonus_armor、attr:damage、fn:armor_multiplier、fn:has_status_tag |
| 琉璃光姬（`item_glass_mitsutomo`） | 否 | action:create_unit.owner、ally_selector.effect:apply_status、ally_selector.types、passive_trigger:on_created |
| 兵力（`item_gold_to_prestige`） | 否 | 沒有 IR |
| 陣太刀（`item_gorgeous_tachi`） | 是 | — |
| 巨龍（`item_great_dragon`） | 否 | action:dash、action:delay、action:projectile、attr:forward、fn:exists、fn:is_ally、passive_trigger:on_created、state:command_restricted、target.type:unit_or_point |
| 八幡鈴（`item_hachiman_bell`） | 否 | 沒有 IR |
| 破魔寶珠（`item_hama_orb`） | 是 | 沒有 IR |
| 般若鬼丸（`item_hannya_onimaru`） | 否 | passive_trigger:on_created |
| 療癒圖騰（`item_healing_totem`） | 否 | action:create_unit.owner、ally_selector.effect:apply_status、ally_selector.types |
| 生命鋼（`item_health_ingot`） | 是 | — |
| 冰計之書（`item_ice_book`） | 否 | 沒有 IR |
| 冰風暴之書（`item_ice_storm_book`） | 否 | action:remove_status.dispel、action:remove_status.exclude_source_tags、action:remove_status.min_duration、action:remove_status.source_is、action:remove_status.team_of_source、action:repeat、attr:forward |
| 火計之書（`item_ignite_book`） | 是 | status.triggers |
| 真視碎片（`item_insight_gem_hero`） | 否 | action:repeat、event:on_owner_death、passive:aura、selector.types:building+creep+hero+neutral+summon、state:revealed |
| 真知碎片（`item_insight_gem_ninja`） | 否 | action:repeat、event:on_owner_death、passive:aura、selector.types:building+creep+neutral+summon、state:revealed |
| 洞察碎片（`item_insight_gem_shard`） | 否 | action:repeat、event:on_owner_death、passive:aura、selector.types:building+creep+hero+neutral+summon、state:revealed |
| 技巧鋼（`item_intelligence_ingot`） | 是 | — |
| 和泉守兼定（`item_izuminokami_kanesada`） | 是 | — |
| 吉鋼鐵（`item_kat_steel`） | 否 | 沒有 IR |
| 打刀（`item_katana`） | 是 | — |
| 桔梗仙冬月（`item_kisyosennfuyuzuki`） | 否 | action:projectile |
| 凍雲（`item_kokumo`） | 否 | passive_trigger:on_created、selector.exclude、stat:slow_pct |
| 平蜘蛛釜（`item_kotennmyohiragumo`） | 否 | action:remove_status.dispel、action:remove_status.team_of_source、passive:aura、state:magic_immune、status.reapply_if_removed |
| 紅雪左文字（`item_kousetsusamonnji`） | 否 | action:chance、ally_selector.effect:apply_status、ally_selector.types |
| 地藏王菩薩木雕‧古物（`item_ksitigarbha`） | 否 | 沒有 IR |
| 短甲（`item_lesser_armor`） | 否 | passive_trigger:on_created、passive_trigger:on_removed |
| 足輕盾（`item_lesser_shield`） | 是 | — |
| 舞雷刀（`item_lightning_katana`） | 否 | action:chance |
| 雷之卷（`item_lightning_scroll`） | 否 | selector.exclude、selector.flag:visible_only、selector.limit、selector.order |
| 燒酒（`item_liquor`） | 是 | — |
| 戒杖刀（`item_logs_big_tachi`） | 否 | fn:is_ally、stat:slow_pct、status.triggers |
| 禦魔長踏（`item_magic_defence_boots`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 破魔．禦魔長踏（`item_magic_defence_boots_1`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 禦魔鋼（`item_magic_resistance_ingot`） | 否 | passive_trigger:on_created |
| 法術戒指（`item_magic_ring`） | 否 | 沒有 IR |
| 御魔護符（`item_magic_talisman`） | 否 | passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:periodic |
| 仕手能靴（`item_mana_boots`） | 是 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 破魔．仕手能靴（`item_mana_boots_1`） | 是 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 法力鋼（`item_mana_ingot`） | 是 | — |
| 大俱利伽羅（`item_mana_thief`） | 否 | action:burn_mana、attr:is_magic_immune、attr:mana、passive_trigger:on_created、stat:all_stats |
| 月輪鏡（`item_moon_mirror`） | 否 | action:repeat、action:set_state、fn:has_status、passive:aura、target.flag:include_magic_immune |
| 斬月槍（`item_moon_spear`） | 是 | — |
| 山之卷（`item_mountain_scroll`） | 否 | fn:has_status_tag |
| 妖刀．村正（`item_muramasa_katana`） | 否 | action:add_stacks、action:apply_status.stacks、action:set_display_stacks、fn:has_status、passive_trigger:on_created、passive_trigger:on_death、passive_trigger:on_kill、passive_trigger:on_removed、status.stacking:stacks、status.stat.per_stack |
| 長捲（`item_nagamaki`） | 否 | action:refund、action:set_health |
| 薙刀（`item_naginata`） | 是 | — |
| 南蠻胴具足（`item_nannbann_armor`） | 否 | — |
| 爆烈彈（`item_napalm_bomb`） | 是 | selector.exclude_types |
| 下忍（`item_ninja1`） | 否 | action:create_unit.owner、action:set_state、fn:exists、status.triggers、token:state |
| 中忍（`item_ninja2`） | 否 | action:create_unit.owner、action:set_state、fn:exists、status.triggers、token:state |
| 忍者裝束（`item_ninja_cloth`） | 否 | passive_trigger:on_created、passive_trigger:on_removed |
| 忍者刀（`item_ninja_sword`） | 否 | status.triggers |
| 大千鳥十文字槍（`item_oochidorijuumonnjiyari`） | 否 | action:chance、action:projectile、selector.flag:visible_only |
| 般若湯（`item_perceive_wine`） | 是 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、state:magic_immune、status.reapply_if_removed |
| 褶裙（`item_pleated_skirt`） | 是 | passive_trigger:on_created |
| 毒之戒指（`item_poisonous_ring`） | 否 | 沒有 IR |
| 強化鋼（`item_power_ingot`） | 是 | — |
| 真視赤備挂甲（`item_prophet_akazonae_armor`） | 否 | passive:aura、passive_trigger:on_attacked_hit、passive_trigger:on_created、passive_trigger:on_removed、selector.types:building+creep+hero+neutral+summon+tower、stat:vision_night、state:revealed |
| 真知短甲（`item_prophet_armor`） | 否 | passive:aura、passive_trigger:on_attacked_hit、passive_trigger:on_removed、selector.types:building+creep+hero+neutral+summon+tower、stat:vision_night、state:revealed |
| 御守（`item_protection_amulet`） | 否 | passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:periodic |
| 雷切（`item_raikiri`） | 否 | action:chance、selector.exclude、selector.flag:visible_only、selector.limit、selector.order |
| 軍糧丸（`item_rations`） | 是 | action:restore_mana |
| 皆朱槍（`item_red_spear`） | 是 | ally_selector.effect:apply_status、ally_selector.types |
| 再生鋼（`item_regeneration_ingot`） | 是 | — |
| 靈都宮燈（`item_reito_lantern`） | 否 | action:repeat、action:set_state、fn:has_status_tag、passive:aura、passive_trigger:on_created、passive_trigger:periodic、target.flag:include_magic_immune |
| 落石之書（`item_rockfall_book`） | 是 | fn:has_status_tag、reveal.teams:own |
| 浪人十手（`item_ronin_jitte`） | 否 | passive_trigger:on_take_damage_pre、status.triggers |
| 琉球浪人木屐（`item_ryukyu_ronin_clogs`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_attacked_hit、passive_trigger:on_created、passive_trigger:on_removed、state:magic_immune、status.reapply_if_removed |
| 破魔．琉球浪人木屐（`item_ryukyu_ronin_clogs_1`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_attacked_hit、passive_trigger:on_created、passive_trigger:on_removed、state:magic_immune、status.reapply_if_removed |
| 逆刃刀（`item_sakabatou`） | 否 | action:chance、attr:is_illusion、fn:has_status、status.stacking:keep |
| 清酒（`item_sake`） | 是 | — |
| 武士服（`item_samurai_cloth`） | 否 | — |
| 薩摩銃（`item_satsuma_gun`） | 否 | stat:heal_reduction_pct |
| 新藤五國光（`item_shintogo_kunimitsu`） | 否 | 沒有 IR |
| 掛面具足（`item_shun_armor`） | 否 | passive_trigger:on_created |
| 手裏劍（`item_shuriken`） | 是 | action:projectile |
| 御魂（`item_soul_adder`） | 否 | action:refund、attr:is_magic_immune、event:on_phase_start、state:invulnerable、status.stat.when |
| 長槍（`item_spear`） | 是 | — |
| 疾風衣（`item_speed_clothes`） | 是 | — |
| 法術書（`item_spell_book`） | 是 | stat:slow_pct |
| 回魔鋼（`item_spirit_ingot`） | 是 | — |
| 番頭盾（`item_stable_shield`） | 是 | — |
| 杵杖（`item_staff`） | 是 | — |
| 狂風之書（`item_storm_book`） | 否 | action:face、action:repeat |
| 蠻力鋼（`item_strength_ingot`） | 是 | — |
| 謙信助宗（`item_sukemune`） | 否 | status.stat.when |
| 日光助真（`item_sukezane`） | 否 | status.triggers |
| 流風捷履（`item_swiftness_boots`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:on_take_damage、passive_trigger:periodic、stat:all_stats、state:magic_immune、status.reapply_if_removed |
| 破魔．流風捷履（`item_swiftness_boots_1`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:on_take_damage、passive_trigger:periodic、stat:all_stats、state:magic_immune、status.reapply_if_removed |
| 太刀（`item_tachi`） | 是 | — |
| 茶碗（`item_tea_bowl`） | 是 | — |
| 孫子兵法全本（`item_the_art_of_war`） | 否 | action:create_unit.overrides、action:create_unit.owner、action:repeat、ally_selector.effect:apply_status、ally_selector.effect:if、ally_selector.types、fn:has_item、passive:aura、stat:all_stats |
| 孫子兵法．火之章（`item_the_art_of_war_flame_chapter`） | 是 | ally_selector.effect:apply_status、ally_selector.types、passive:aura |
| 孫子兵法．林之章（`item_the_art_of_war_forest_chapter`） | 否 | action:create_unit.overrides、action:create_unit.owner、passive:aura、token:state |
| 孫子兵法．山之章（`item_the_art_of_war_mountain_chapter`） | 否 | ally_selector.effect:if、ally_selector.types、fn:has_status_tag、passive:aura、selector.types:building+creep+hero+neutral+summon |
| 孫子兵法．風之章（`item_the_art_of_war_tornado_chapter`） | 否 | action:repeat、passive:aura |
| 嗜血妖刀（`item_the_blade_of_blood_devourer`） | 否 | attr:bonus_armor、attr:damage、fn:armor_multiplier、fn:has_status_tag、passive_trigger:on_kill、passive_trigger:periodic、stat:all_stats |
| 星杵（`item_the_club_of_nebula`） | 否 | action:delay、action:projectile、passive_trigger:on_created |
| 黑漆塗五枚胴（`item_the_dark_armor`） | 否 | ally_selector.effect:apply_status、ally_selector.types、passive:aura、passive_trigger:on_created、status.aura |
| 闇牙黃泉津（`item_the_dark_yellow_teeth_and_zimbabwe`） | 否 | status.triggers |
| 銀伊予札白糸威胴丸具足（`item_the_great_silver_armor`） | 否 | action:remove_status.dispel、action:remove_status.exclude_tags、action:remove_status.team_of_source、passive_trigger:on_created、state:magic_immune、status.reapply_if_removed |
| 蜻蛉切．銘槍（`item_the_great_spear_of_dragonfly`） | 否 | passive_trigger:on_created、passive_trigger:on_removed |
| 鬼撤．妖刀（`item_the_great_sword_of_anger`） | 否 | passive_trigger:on_kill、status.stacking:independent |
| 池田鬼神丸國重．銘刀（`item_the_great_sword_of_disease`） | 否 | ally_selector.effect:apply_status、ally_selector.effect:remove_status、ally_selector.types、passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:periodic、selector.types:creep+neutral+summon |
| 吉光骨食．銘刀（`item_the_great_sword_of_drain`） | 否 | action:remove_status.dispel、action:remove_status.exclude_source_tags、action:remove_status.min_duration、action:remove_status.team_of_source、passive_trigger:on_created、passive_trigger:on_removed、passive_trigger:periodic、target.flag:include_magic_immune |
| 鐵碎牙．妖刀（`item_the_great_sword_of_hurricane`） | 否 | action:create_unit.owner、action:repeat、passive_trigger:on_created |
| 葵紋越前康繼．禦神刀（`item_the_great_sword_of_invincible`） | 否 | passive.stat.when、passive_trigger:on_removed、passive_trigger:periodic、stat:all_stats |
| 長船．太刀（`item_the_great_sword_of_precision`） | 否 | action:chance |
| 菊一文字則宗．太刀（`item_the_great_sword_of_soul_hyper`） | 否 | passive:aura、stat:move_speed、stat:status_resistance_pct、target.flag:include_magic_immune、target.flag:pierce_magic_immune |
| 鬼丸國綱．銘刀（`item_the_great_sword_of_spike`） | 否 | action:repeat |
| 陸奧守吉行．銘刀（`item_the_great_sword_of_teleport`） | 否 | action:teleport、event:on_phase_start、passive:aura、passive_trigger:on_attacked_hit、passive_trigger:on_removed、target.types:creep+neutral+summon+courier+building |
| 長曾彌虎撤．銘刀（`item_the_great_sword_of_tiger`） | 否 | action:attack、action:teleport、passive_trigger:on_attacked_hit、passive_trigger:on_created、passive_trigger:on_removed、target.flag:include_magic_immune |
| 梅鶯毒．妖刀（`item_the_great_sword_of_toxic`） | 否 | action:apply_status.source、action:projectile、attr:base_attack_range、fn:is_hero_id、stat:heal_reduction_pct、stat:slow_pct、status.triggers |
| 三日月宗近．銘刀（`item_the_great_sword_of_xnew_moon`） | 否 | action:repeat、passive_trigger:on_created、passive_trigger:on_removed、selector.flag:include_illusions、selector.flag:visible_only、selector.limit、selector.order、state:provides_vision |
| 鬼頰當（`item_the_hooate_of_ghost`） | 是 | — |
| 孫子兵法殘冊（`item_the_lost_art_of_war_1`） | 否 | action:create_unit.overrides、action:create_unit.owner、ally_selector.effect:apply_status、ally_selector.types、fn:has_item、passive:aura、stat:all_stats |
| 孫子兵法斷簡（`item_the_lost_art_of_war_2`） | 否 | action:repeat、ally_selector.effect:if、ally_selector.types、passive:aura、selector.types:building+creep+hero+neutral+summon+tower、stat:all_stats |
| 無上雷暴之書（`item_the_over_lightning_art_of_war`） | 否 | event:on_unequip、passive_trigger:on_created、passive_trigger:on_removed、stat:cast_point_reduction_pct、status.triggers |
| 大炎上之書（`item_the_overflame_art_of_war`） | 否 | action:repeat |
| 靜心兜（`item_the_poketto_of_meiso`） | 是 | — |
| 赤雷卷（`item_the_red_lightning_chapter`） | 否 | selector.exclude、selector.flag:visible_only、selector.limit、selector.order、state:mute |
| 紫雷卷（`item_the_red_lightning_chapter_hyper`） | 否 | passive_trigger:on_kill、selector.exclude、selector.flag:visible_only、selector.limit、selector.order、state:mute |
| 鬼蜘蛛．夜哭（`item_the_scream_of_spiders`） | 否 | action:set_state、attr:base_attack_range、passive:aura、token:state |
| 鬼切（`item_the_spear_of_ghost`） | 否 | action:set_state、attr:base_attack_range、attr:effect_multiplier、fn:is_hero_id、token:state |
| 鞍切．一階（`item_the_spear_of_saddle`） | 否 | action:attack、action:projectile、action:set_display_stacks、attr:armor、attr:is_magic_immune、fn:has_status、passive:orb_slot、stat:all_stats、target.flag:include_magic_immune |
| 蜘蛛切（`item_the_spear_of_spiders`） | 否 | passive:aura |
| 天雷之書（`item_the_storm_call_book`） | 否 | action:repeat |
| 神直毘御劍（`item_the_sword_of_kaminaobi`） | 否 | action:issue_order、action:teleport、target.flag:include_magic_immune |
| 怒雷之書（`item_thunderbolt_book`） | 否 | event:on_unequip、passive_trigger:on_created、status.triggers |
| 天變之書（`item_thunderstorms`） | 否 | action:repeat、reveal.teams:own、selector.flag:visible_only |
| 韌性護符（`item_toughness_ingot`） | 否 | passive_trigger:on_created、passive_trigger:on_removed |
| 正宗．武士刀（`item_true_katana`） | 否 | passive_trigger:on_attack_windup、stat:outgoing_damage_pct |
| 和弓（`item_wayumi`） | 否 | passive_trigger:on_created、passive_trigger:on_removed |
| 改良和弓（`item_wayumi2`） | 否 | action:apply_status.stacks、action:issue_order、action:knockback、passive_trigger:on_created、passive_trigger:on_removed、status.stacking:stacks、status.triggers |
| 風之卷（`item_wind_scroll`） | 否 | action:repeat |
| 雪走（`item_yubashiri`） | 否 | stat:slow_pct |

## 4. 重跑

```
tools\lua\lua.exe scripts\gen_v3_content.lua
tools\lua\lua.exe docs\character_pipeline\tools\v3_item_order.lua
```
