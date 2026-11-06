-- A17 佐佐成政（舊版，11 版 heromap_skill A17 ["11"] = "WERT"，沒有 D）— package schema v3。
-- 版本：選擇 11 版時 tofar_goback.lua choose_11 拔掉全部技能，依 _G.heromap_skill[A17]["11"] = "WERT"
-- 依序 AddAbility("A17W_old"、"A17E_old"、"A17R_old"、"A17T_old")（vscripts/items/Addon_Items/tofar_goback.lua:96-140；
-- vscripts/addon_game_mode.lua:259,362），自動學習清單為空。槽位 W/E/R/T → Q/W/E/R。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A17_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"。
return function(ctx)
  local UNITS = { "hero", "creep", "neutral", "summon" }
  local UNITS_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A17_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A17", text = { zh_TW = { name = "佐佐成政（舊版）" }, en = { name = "Sassa Narimasa (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a17_old_sulfur_shot", source_id = "A17W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 60, 70, 80, 90, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          duration = { 6, unit = "s", source = "kv:debuff_duration" },
          armor_cut = { 3, 6, 9, 12, unit = "armor", source = "kv:debuff_physical_armor", kv = { -3, -6, -9, -12 },
            source_differs = "KV 以負值記減益；行為裡取負號" },
          tick = { 1, unit = "s", source = "kv:debuff_damage_delay" },
          projectile_speed = { 1500, unit = "speed", hidden = true, source = "kv:projectile_speed" },
        },
        text = {
          zh_TW = { name = "硫磺砲",
            body = "朝目標敵人射出可被閃避的硫磺彈。命中後目標燃燒 {duration} 秒：護甲降低 {armor_cut}、位置被看見，每 {tick} 秒受到 {damage} 魔法傷害。" },
          en = { name = "Sulfur Shot",
            body = "Fires a dodgeable sulfur shell at the target enemy. On hit, the target burns for {duration} seconds: {armor_cut} less armor, its position is revealed, and it takes {damage} magic damage every {tick} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = UNITS },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a17w_old_burn", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "硫磺燃燒", body = "護甲降低 {armor_cut}、位置被看見，每 {tick} 秒受到 {damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Sulfur Burn", body = "{armor_cut} less armor and revealed, taking {damage} magic damage every {tick} seconds for {duration} seconds." },
            },
            reveal = "public", cue = "a17w_old_burn", states = { "provides_vision" }, stats = { armor = "-$armor_cut" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A17W_old_debuff（未寫 IsPurgable）：PHYSICAL_ARMOR_BONUS、PROVIDES_VISION、ThinkInterval 1。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a17w_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed", dodgeable = true,
              cue = "a17w_old_assassinate",
              on_hit = { { op = "apply_status", status = "a17w_old_burn", to = "it" } } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A17.txt:857-858",
            text = "Duration 6、ThinkInterval 1：第六跳與到期同時，依引擎順序可能是五或六跳；照 KV 意圖寫成每秒一跳。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:839-848",
            text = "命中只套減益、沒有立即傷害；目標在投射物飛行中變魔免時，datadriven ApplyModifier 依引擎規則套不上（規則層）。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:834", text = "ProvidesVision 0：投射物本身不給視野。" },
        },
        provenance = { "npc/A_Oda/A17.txt:804-906" },
      },
      -------------------------------------------------------------------------
      { id = "a17_old_rapid_cannon", source_id = "A17E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 60, 80, 100, 120, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 10, unit = "s", source = "kv:duration" },
          attack_speed = { 20, 40, 60, 80, unit = "atk_speed", source = "kv:buff_atk_spd" },
          chance = { 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A17.lua:287-290 RandomInt(1,100) <= chance（kv:chance 50，恰好 50%）" },
          damage = { 80, 100, 120, 140, unit = "dmg", damage_type = "physical", source = "kv:AbilityDamage" },
        },
        text = {
          zh_TW = { name = "連擊砲",
            body = "{duration} 秒內攻擊速度提高 {attack_speed}；期間每次攻擊命中有 {chance} 機率額外造成 {damage} 物理傷害。" },
          en = { name = "Rapid Cannon",
            body = "For {duration} seconds, gains +{attack_speed} attack speed; each attack that lands meanwhile has a {chance} chance to deal {damage} bonus physical damage." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a17e_old_buff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "連擊砲", body = "攻擊速度 +{attack_speed}；攻擊命中有 {chance} 機率額外造成 {damage} 物理傷害。持續 {duration} 秒。" },
              en = { name = "Rapid Cannon", body = "+{attack_speed} attack speed; landed attacks have a {chance} chance to deal {damage} bonus physical damage. Lasts {duration} seconds." },
            },
            reveal = "public", cue = "a17e_old_buff", stats = { attack_speed = "$attack_speed" },
            triggers = {
              on_attack_hit = {
                { op = "chance", p = "$chance", actions = {
                  { op = "damage", to = "event.target", source = "self", type = "physical", amount = "$damage" },
                } },
              },
            },
            notes = "原作 modifier_A17E_old_buff（未寫 IsPurgable）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a17e_old_vo", at = "caster" },
            { op = "cue", id = "a17e_old_flare_sound", at = "caster" },
            { op = "apply_status", status = "a17e_old_buff", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:282-293",
            text = "額外傷害是 ApplyDamage 物理（GetAbilityDamage），吃護甲，不是攻擊；攻擊建築命中也會觸發。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A17.lua:295-309", text = "OnCreated／OnDestroy 只建立與銷毀粒子，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A17.txt:908-991", "vscripts/heroes/A_Oda/A17.lua:282-309" },
      },
      -------------------------------------------------------------------------
      { id = "a17_old_rain_guns", source_id = "A17R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          hero_pct = { 5, 10, 15, 20, unit = "pct", source = "kv:add_for_hero" },
          creep_pct = { 25, 35, 45, 55, unit = "pct", source = "kv:add_for_creep" },
          building_pct = { 5, 10, 15, 20, unit = "pct", source = "kv:add_for_building" },
        },
        text = {
          zh_TW = { name = "雨鐵砲",
            body = "被動：每次攻擊命中，額外造成該次攻擊傷害一定比例的物理傷害：英雄 {hero_pct}、一般單位 {creep_pct}、建築 {building_pct}。" },
          en = { name = "Rain of Guns",
            body = "Passive: each attack that lands deals extra physical damage equal to a share of that attack's damage: {hero_pct} against heroes, {creep_pct} against units and {building_pct} against buildings." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              { op = "damage", to = "event.target", source = "self", type = "physical",
                amount = "event.attack.damage * select(event.target.is_hero, $hero_pct, select(event.target.is_creep, $creep_pct, select(event.target.is_building, $building_pct, 0)))" },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:1040-1054",
            text = "add_for_hero／creep／building 三個都放在 \"02\" 鍵下；推定三者都會載入、各自讀得到。若引擎只留最後一個，英雄與一般單位會讀到 0。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:318-330",
            text = "add_chance（100%）的判斷已註解，每次命中必定觸發；判斷順序 IsHero → IsCreep → IsBuilding，都不是時（例如守衛）倍率 0。額外傷害是 ApplyDamage 物理，吃護甲。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:1014",
            text = "%attack_damage 是該次攻擊的傷害（護甲前）；幻象攻擊也會觸發（被動會複製到幻象）。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:1017-1029", text = "OnCreated 的槍口粒子屬呈現。" },
        },
        provenance = { "npc/A_Oda/A17.txt:993-1056", "vscripts/heroes/A_Oda/A17.lua:311-331" },
      },
      -------------------------------------------------------------------------
      { id = "a17_old_great_cannon", source_id = "A17T_old", slot = "R", kind = "toggle", max_level = 3, tags = { "ultimate", "toggle" },
        values = {
          cooldown = { 5, 7, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 110, 160, 210, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, 1150, 1300, unit = "dist", source = "kv:AbilityCastRange", note = "只用於指示器" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel", note = "RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          lock = { 4, 6, 8, unit = "s", source = "kv:lock" },
          night_vision = { 500, unit = "dist", source = "kv:night_sight", note = "只有 BONUS_NIGHT_VISION" },
          attack_range_bonus = { 300, 450, 600, unit = "dist", source = "kv:attack_range_bonus" },
          attack_bonus = { 200, 300, 400, unit = "dmg", source = "kv:attack_bonus", note = "MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE" },
          base_attack_time = { 2.8, unit = "s", source = "kv:base_atk_spd", note = "MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT" },
          move_slow = { 200, unit = "pct", source = "kv:move_speed_bonus", kv = { -200 },
            source_differs = "KV 以負值記減益；行為裡取負號" },
          splash_radius = { 350, 375, 400, unit = "dist", source = "kv:attacked_range" },
          splash = { 60, unit = "pct", source = "script:vscripts/heroes/A_Oda/A17.lua:257 dmg*0.6" },
        },
        text = {
          zh_TW = { name = "大筒",
            body = "開啟後換上大筒，{lock} 秒內無法關閉：夜間視野增加 {night_vision}、攻擊距離增加 {attack_range_bonus}、基礎攻擊力增加 {attack_bonus}，基礎攻擊間隔變為 {base_attack_time} 秒，移動速度降低 {move_slow}。普攻命中時，對目標周圍 {splash_radius} 內其他非建築敵人（包含魔免）造成該次攻擊傷害 {splash} 的物理傷害。再次施放關閉。" },
          en = { name = "Great Cannon",
            body = "When toggled on, Narimasa switches to the great cannon and cannot toggle it off for {lock} seconds: +{night_vision} night vision, +{attack_range_bonus} attack range and +{attack_bonus} base attack damage, a base attack time of {base_attack_time} seconds, and {move_slow} less movement speed. Attacks that land deal {splash} of the attack's damage as physical damage to other non-building enemies within {splash_radius} of the target (spell-immune ones included). Toggle again to turn it off." },
        },
        target = { type = "none", range = "$cast_range" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a17t_old_cannon", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "大筒", body = "夜間視野 +{night_vision}、攻擊距離 +{attack_range_bonus}、基礎攻擊力 +{attack_bonus}，基礎攻擊間隔 {base_attack_time} 秒，移動速度降低 {move_slow}；普攻命中時對目標周圍 {splash_radius} 內其他非建築敵人造成攻擊傷害 {splash} 的物理傷害。持續到關閉為止。" },
              en = { name = "Great Cannon", body = "+{night_vision} night vision, +{attack_range_bonus} attack range and +{attack_bonus} base attack damage, a base attack time of {base_attack_time} seconds and {move_slow} less movement speed; landed attacks deal {splash} of their damage as physical damage to other non-building enemies within {splash_radius} of the target. Lasts until toggled off." },
            },
            reveal = "public", cue = "a17t_old_buff",
            stats = { vision_night = "$night_vision", attack_range = "$attack_range_bonus", base_attack_damage = "$attack_bonus",
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" }, move_speed_pct = "-$move_slow" },
            triggers = {
              on_attack_hit = {
                { op = "for_each",
                  select = { from = "event.target", shape = "circle", radius = "$splash_radius", team = "enemy",
                    types = UNITS_BUILDING, flags = { "include_magic_immune" }, exclude = "event.target",
                    where = "not it.is_building" },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "physical", amount = "event.attack.damage * $splash" },
                  } },
                { op = "cue", id = "a17t_explosion", at = "event.target" },
              },
            },
            notes = "原作 modifier_A17T_old_buff：未寫 Duration 與 IsPurgable；Orb 區塊只換投射物外觀。" },
          { id = "a17t_old_locker", duration = "$lock", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "set_ability_active", of = "self", ability = "a17_old_great_cannon", active = false } },
              on_removed = { { op = "set_ability_active", of = "self", ability = "a17_old_great_cannon", active = true } },
            },
            notes = "原作 modifier_A17T_locker：OnCreated SetActivated(false)、OnDestroy SetActivated(true)。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "a17t_old_cannon", to = "caster" },
            { op = "apply_status", status = "a17t_old_locker", to = "caster" },
          },
          on_toggle_off = {
            { op = "remove_status", from = "caster", status = "a17t_old_cannon" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:1146,1191",
            text = "移速是 MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE −200（不經 nobu 緩速池），合成後遠超 100%，實際由引擎壓到最低移速。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:1148,1186",
            text = "BASE_ATTACK_TIME_CONSTANT 2.8 直接設定基礎攻擊間隔（不是減攻速）；本體改成攻速減益。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:238-260",
            text = "擴散傷害是每個周圍非建築單位各受 %attack_damage × 0.6 的 AMHC 物理傷害，各自再吃護甲；攻擊建築命中也會擴散。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:1069",
            text = "開關型技能每次切換都進冷卻 5/7/9 秒；加上鎖定 4/6/8 秒，關閉前至少要等鎖定結束。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:1144",
            text = "只加夜間視野（BONUS_NIGHT_VISION），白天不變；本體日夜都加。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:1083-1100",
            text = "ApplyModifier 的 Target 只有 Center = CASTER、沒有 Radius，只套到施法者本人。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:1133-1159",
            text = "modifier_A17T_old_buff 未寫 IsPurgable，依本檔慣例寫 dispel = basic（被驅散時技能仍顯示開啟）。" },
        },
        provenance = { "npc/A_Oda/A17.txt:1058-1199", "vscripts/heroes/A_Oda/A17.lua:206-260" },
      },
    },
  }
end
