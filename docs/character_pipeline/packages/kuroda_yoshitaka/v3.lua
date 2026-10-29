-- A19 黑田孝高（current 版，heromap_skill A19 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A19_2.txt:18-23）。20 版（A19*_20、A19D_20）與 *_old 不屬於 current 版。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A19_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）照 A04 寫成 slow_pct，狀態帶 tag slow；合成與緩速抗性屬規則層。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A19",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A19",
      text = { zh_TW = { name = "黑田孝高" }, en = { name = "Kuroda Yoshitaka" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a19_fire_stratagem", source_id = "A19W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, 8, 7, 6, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 105, 135, 165, 195, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 450, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:26,58 FindUnitsInRadius 450（AOERadius 也是 450）" },
          blast_damage = { 75, 150, 225, 300, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          blast_count = { 2, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A19.lua:15-47 count 1..(0.1 + time)，time = 2，每秒一次（t = 0、1）" },
          blast_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:46 return 1" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/A_Oda/A19.lua:41-43 damage * 0.2" },
          ember_start = { 2, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:49 Timers:CreateTimer(2, ...)" },
          ember_count = { 5, unit = "count", source = "script:vscripts/heroes/A_Oda/A19.lua:50-51 count2 > 5 停止" },
          ember_damage = { 10, 20, 30, 40, unit = "dmg", damage_type = "magical", source = "kv:dmg2" },
          ember_building_damage = { 0.5, 1, 1.5, 2, unit = "dmg", damage_type = "magical", source = "kv:dmg3" },
          burn_time = { 1, unit = "s", hidden = true, source = "kv:modifier_A19W Duration" },
          burn_tick = { 0.99, unit = "s", hidden = true, source = "kv:modifier_A19W ThinkInterval" },
        },
        text = {
          zh_TW = { name = "鬼謀．火攻",
            body = "使目標地點 {radius} 範圍陷入火海：立即與之後每 {blast_interval} 秒各爆炸一次，共 {blast_count} 次，每次對範圍內非魔免的敵人造成 {blast_damage} 魔法傷害（建築只受 {building_pct}）。施放 {ember_start} 秒後開始留下餘火，之後每秒點燃仍在範圍內的敵人，共 {ember_count} 次；每次點燃在稍後造成 {ember_damage} 魔法傷害（建築改為 {ember_building_damage}）。" },
          en = { name = "Stratagem: Fire Attack",
            body = "Sets a {radius} area at the target point ablaze: it explodes immediately and again every {blast_interval} seconds, {blast_count} times in total, each dealing {blast_damage} magic damage to non-spell-immune enemies inside (buildings take only {building_pct}). Starting {ember_start} seconds after the cast, embers ignite enemies still in the area once per second, {ember_count} times; each ignition deals {ember_damage} magic damage shortly after ({ember_building_damage} to buildings)." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a19w_burn", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "餘火", body = "被餘火點燃，稍後受到 {ember_damage} 魔法傷害。" },
              en = { name = "Embers", body = "Ignited by embers; takes {ember_damage} magic damage shortly after." },
            },
            reveal = "public", cue = "a19w_burn_glow",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$ember_damage" },
              } },
            },
            notes = "原作 modifier_A19W：IsDebuff、Duration 1、ThinkInterval 0.99，未寫 IsPurgable。" },
          { id = "a19w_burn_building", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "餘火", body = "建築被餘火點燃，稍後受到 {ember_building_damage} 魔法傷害。" },
              en = { name = "Embers", body = "The building is ignited by embers and takes {ember_building_damage} magic damage shortly after." },
            },
            reveal = "public", cue = "a19w_burn_glow",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$ember_building_damage" },
              } },
            },
            notes = "原作 modifier_A19W2：建築用，傷害改讀 dmg3。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a19w_vo", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a19w_fire_field", at = "at", radius = "$radius" },
            { op = "repeat", interval = "$blast_interval", first = "0", count = "$blast_count", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                actions = {
                  { op = "damage", require_source_alive = false, to = "it", type = "magical",
                    amount = "select(it.is_building, $blast_damage * $building_pct, $blast_damage)" },
                } },
            } },
            { op = "repeat", interval = "$blast_interval", first = "$ember_start", count = "$ember_count", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                actions = {
                  { op = "if", cond = "it.is_building",
                    actions = { { op = "apply_status", status = "a19w_burn_building", to = "it" } },
                    otherwise = { { op = "apply_status", status = "a19w_burn", to = "it" } } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:15-47",
            text = "爆炸計時器 count 從 1 起、超過 0.1 + time = 2.1 才停，所以在施放當下與一秒後各爆一次（說明寫「2 秒內」受到兩倍傷害，相符）。" },
          { policy = "design_intent", ref = "npc/A_Oda/A19.txt:83-116",
            text = "餘火每秒重套一次 1 秒、ThinkInterval 0.99 的狀態，每次套用在 0.99 秒後跳一次；重套時 think 是否重新計時依引擎時序，照設計意圖每次點燃各跳一次、共五次。離開火海後已點燃的那一次仍會結算。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:23-31",
            text = "選取用技能的目標設定：HERO|BASIC|BUILDING、旗標 NONE（不含魔免）；建築的直擊乘 0.2，餘火改用 dmg3（約 5%）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A19.lua:7-13",
            text = "npc_dummy_unit（7 秒、majia）與火焰特效只是呈現，改用 cue。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:74-79", text = "item_soul_adder.lua 的 sound 只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A19.txt:20-118", "vscripts/heroes/A_Oda/A19.lua:3-73" },
      },
      -------------------------------------------------------------------------
      { id = "a19_chain_fire", source_id = "A19E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 170, 190, 210, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1100, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius" },
          radius = { 450, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:85 FindUnitsInRadius 450（AOERadius 也是 450）" },
          duration = { 3, unit = "s", source = "kv:modifier_A19E Duration" },
          tick = { 0.99, unit = "s", hidden = true, source = "kv:modifier_A19E ThinkInterval" },
          damage = { 50, 85, 120, 155, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          armor_cut = { 4, 6, 8, 10, unit = "armor", source = "kv:armor", kv = { -4, -6, -8, -10 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          slow = { 30, 40, 50, 60, unit = "pct", source = "kv:slow", kv = { -30, -40, -50, -60 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "dummy.lua Slow ms_slow，nobu 緩速池" },
        },
        text = {
          zh_TW = { name = "連環火",
            body = "向一名敵人投出流焰，命中後擴散：目標周圍 {radius} 內非魔免的敵人在 {duration} 秒內護甲降低 {armor_cut}、移動速度降低 {slow}，並約每秒受到 {damage} 魔法傷害。" },
          en = { name = "Chain Fire",
            body = "Hurls a stream of fire at an enemy that spreads on impact: non-spell-immune enemies within {radius} of the target lose {armor_cut} armor and {slow} movement speed for {duration} seconds and take {damage} magic damage about every second." },
        },
        target = { type = "unit", range = "$cast_range", aoe_radius = "$radius", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a19e_burn", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "連環火", body = "護甲降低 {armor_cut}、移動速度降低 {slow}，並約每秒受到 {damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Chain Fire", body = "Armor reduced by {armor_cut} and movement speed slowed by {slow}; takes {damage} magic damage about every second for {duration} seconds." },
            },
            reveal = "public", cue = "a19e_burn_glow", tags = { "slow" },
            stats = { armor = "-$armor_cut", slow_pct = "$slow" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A19E：IsDebuff、Duration 3、ThinkInterval 0.99，未寫 IsPurgable；OnCreated 寫進 nobu 緩速池（名稱 A19E）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a19e_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "a19e_cast_sound", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              vision = "$projectile_vision", dodgeable = true, cue = "a19e_fire_bolt", on_hit = {
                { op = "for_each",
                  select = { from = "it", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "apply_status", status = "a19e_burn", to = "it" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A19.txt:201-244",
            text = "Duration 3 配 ThinkInterval 0.99，照設計意圖共三跳。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:134",
            text = "AbilityDamage 200 沒有被使用；腳本讀的 GetCursorPosition() 也沒有使用。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:82-95",
            text = "擴散以命中時的目標位置為圓心，選取用技能的目標設定（HERO|BASIC、不含魔免）；投射物被躲開就沒有效果。" },
        },
        provenance = { "npc/A_Oda/A19.txt:120-246", "vscripts/heroes/A_Oda/A19.lua:75-96" },
      },
      -------------------------------------------------------------------------
      { id = "a19_fire_echo", source_id = "A19R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          int_bonus = { 8, 16, 24, 32, unit = "stat", source = "kv:addint" },
          int_mul = { 0.4, 0.6, 0.8, 1, unit = "ratio", source = "kv:dmg" },
          radius = { 800, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 GetCastRange() 當搜尋半徑" },
          item_pct = { 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A19.lua:104-106 dmg * 0.5" },
          immune_pct = { 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A19.lua:127-128 dmgx * 0.5" },
        },
        text = {
          zh_TW = { name = "火焰返響",
            body = "被動：智力增加 {int_bonus}。每次施放技能或使用道具時，以施放目標（或施放地點）為中心 {radius} 內的敵方英雄與一般單位受到孝高智力 {int_mul} 倍的純粹傷害；魔免單位只受 {immune_pct}，道具觸發時傷害也只有 {item_pct}。切換類技能與部分位移、傳送道具不會觸發。" },
          en = { name = "Fire Echo",
            body = "Passive: +{int_bonus} intelligence. Whenever Yoshitaka casts an ability or uses an item, enemy heroes and units within {radius} of the target (or target point) take pure damage equal to {int_mul} × his intelligence; spell-immune units take only {immune_pct}, and item triggers deal only {item_pct}. Toggles and some movement or teleport items do not trigger it." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public",
          stats = { int = "$int_bonus" },
          triggers = {
            on_ability_executed = { who = "self",
              filter = "not has_tag(event.ability, toggle) and not is_item(event.ability, item_logging) and not is_item(event.ability, item_tpscroll) and not is_item(event.ability, item_flash_shoes) and not is_item(event.ability, item_flash_ring) and not is_item(event.ability, item_samurai_cloth)",
              actions = {
                { op = "let", name = "center", value = "select(exists(event.target), position(event.target), event.point)" },
                { op = "let", name = "amount",
                  value = "self.int * $int_mul * select(event.ability.is_item, $item_pct, 1)" },
                { op = "for_each",
                  select = { from = "center", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune" } },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "pure",
                      amount = "select(it.is_magic_immune, amount * $immune_pct, amount)" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:102",
            text = "中心點是 A19R 自己的 GetCursorPosition()；推定它反映施法者這次施放的游標位置（單位目標技能即目標位置）。無目標技能與道具的游標位置原作沒有定義（可能是上一次的游標或原點），資料以 event.point 表達，需實機確認。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:104-131",
            text = "道具減半與魔免減半相乘（魔免單位受道具觸發只有四分之一）；傷害類型是純粹，魔免單位仍會受傷。智力在觸發當下讀取，含本技能的加成。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:108-114",
            text = "排除切換類技能、attribute_bonusx、伐木、回城卷、閃步鞋、閃戒與武士陣羽織；attribute_bonusx 是屬性加點，v3 不是技能，不必列。OnAbilityExecuted 在技能效果之前觸發。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:277-284",
            text = "被動技能的 OnAbilityPhaseStart 語音永遠不會觸發。" },
        },
        provenance = { "npc/A_Oda/A19.txt:248-307", "vscripts/heroes/A_Oda/A19.lua:99-133" },
      },
      -------------------------------------------------------------------------
      { id = "a19_siege_fire", source_id = "A19T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 135, 110, 95, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          lines = { 3, unit = "count", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:255 for line = 1,3" },
          spread = { 30, unit = "deg", source = "script:vscripts/heroes/A_Oda/A19.lua:170-178 ±30 度旋轉" },
          start_offset = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:188-190 point + 100*vec" },
          step = { 200, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:191 distance = 200" },
          points = { 14, unit = "count", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:200-217 num 1..14" },
          early_points = { 3, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A19.lua:200,254 第一次結算在 0.1 秒，此時每道火牆只鋪了三個點（0.03 秒一個）" },
          length = { expr = "$start_offset + $step * $points", unit = "dist", source = "derived",
            note = "最遠的點離施放者 100 + 200 × 14 = 2900（KV arrow_range 3000 沒有被讀取）" },
          width = { 200, unit = "dist", source = "kv:arrow_width", note = "腳本寫死 200，數字相同" },
          pulse_first = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:254" },
          pulse_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:271-273 return 1" },
          duration = { 13, unit = "s", source = "kv:duration" },
          pulses = { 14, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A19.lua:254-274 先結算再判斷 GameTime < 施放 + 13，在 0.1、1.1……13.1 秒共 14 次" },
          burn_time = { 1.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:263,265 duration = 1.1" },
          burn_tick = { 0.25, unit = "s", source = "kv:modifier_A19T_3 ThinkInterval" },
          damage = { 80, 120, 160, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          building_damage = { 4, 6, 8, unit = "dmg", damage_type = "magical", source = "kv:dmg2" },
          vision_duration = { 12, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:212,230,247 AddFOWViewer 12" },
        },
        text = {
          zh_TW = { name = "籠城之火",
            body = "朝指定方向放出三道火牆（中間一道與左右各偏 {spread} 度），向前延燒約 {length}，持續約 {duration} 秒並提供視野。每 {pulse_interval} 秒，火牆每一段 {width} 範圍內的敵人（包含魔免與建築）被點燃：每次點燃立即造成 {damage} 魔法傷害，燃燒期間每 {burn_tick} 秒再造成 {damage} 魔法傷害；同時靠近多段火牆會被重複點燃。建築不吃點燃的立即傷害，燃燒每跳改為 {building_damage}。" },
          en = { name = "Siege Fire",
            body = "Unleashes three walls of fire in the target direction (one straight ahead and one {spread} degrees to each side) that spread about {length} forward, last about {duration} seconds and grant vision. Every {pulse_interval} seconds, enemies (including spell-immune ones and buildings) within {width} of any segment are ignited: each ignition deals {damage} magic damage immediately and {damage} more every {burn_tick} seconds while burning; standing near several segments ignites them several times. Buildings skip the immediate damage and take {building_damage} from each burn instead." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a19t_burn", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" } },

              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A19T_3：IsHidden、Duration 1.01（套用時覆寫成 1.1）、ThinkInterval 0.25，OnCreated 也造成一次傷害；未寫 IsPurgable。" },
          { id = "a19t_burn_building", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$building_damage" },
              } },
            },
            notes = "原作 modifier_A19T_5：建築用，沒有 OnCreated 傷害。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a19t_cast_sound", at = "caster" },
            { op = "let", name = "origin", value = "position(caster)" },
            { op = "let", name = "dir", value = "direction(caster, point)" },
            { op = "let", name = "base", value = "offset(origin, dir, $start_offset)" },
            { op = "repeat", count = "$points", as = "k", actions = {
              { op = "reveal", at = "offset(base, dir, $step * (k + 1))", radius = "$width", duration = "$vision_duration", teams = "own" },
              { op = "reveal", at = "offset(base, rotate(dir, $spread), $step * (k + 1))", radius = "$width", duration = "$vision_duration", teams = "own" },
              { op = "reveal", at = "offset(base, rotate(dir, 0 - $spread), $step * (k + 1))", radius = "$width", duration = "$vision_duration", teams = "own" },
            } },
            { op = "cue", id = "a19t_fire_walls", at = "base", to = "point", duration = "$duration" },
            { op = "repeat", interval = "$pulse_interval", first = "$pulse_first", count = "$pulses", as = "p", actions = {
              { op = "repeat", count = "$lines", as = "line", actions = {
                { op = "let", name = "ldir",
                  value = "rotate(dir, select(line == 0, 0, select(line == 1, $spread, 0 - $spread)))" },
                { op = "repeat", count = "select(p == 0, $early_points, $points)", as = "j", actions = {
                  { op = "for_each",
                    select = { from = "offset(base, ldir, $step * (j + 1))", shape = "circle", radius = "$width",
                      team = "enemy", types = { "all" }, flags = { "include_magic_immune" } },
                    actions = {
                      { op = "if", cond = "it.is_building",
                        actions = { { op = "apply_status", status = "a19t_burn_building", to = "it" } },
                        otherwise = { { op = "apply_status", status = "a19t_burn", to = "it" } } },
                    } },
                } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:389-397",
            text = "Dota 的 modifier 重套時只刷新：Lua modifier 走 OnRefresh，不會重跑 OnCreated，datadriven 推定相同。所以只有第一次被點燃有立即傷害，之後每秒 4 跳，與原作說明的每秒 4 × dmg 相符；資料刪掉 on_refreshed 的傷害（D13，2026-10-29，confidence medium）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:255-270",
            text = "火牆三道各十四個點是 for line = 1,3 / for i = 1,num 的同步迴圈，以省略 interval 的巢狀 repeat 表達（v3.1：所有次數在同一步內依序跑完）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:188-253",
            text = "三道火牆都從施放者前方 100 的同一點出發，各自每 0.03 秒往前鋪一個點、共 14 個（間距 200），最遠約 2900；靠近起點時三道重疊，敵人會被多個點重複點燃。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:254-274",
            text = "結算從 0.1 秒開始、每秒一次，先結算再判斷是否超過 13 秒，所以共 14 次（到 13.1 秒）；第一次結算時每道只有三個點。特效 13 秒、視野 12 秒，說明寫持續 12 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:258-260",
            text = "選取是 DOTA_UNIT_TARGET_ALL 加 MAGIC_IMMUNE_ENEMIES：建築、信使、守衛都會被點燃；魔免單位的魔法傷害由規則層歸零。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:351-440",
            text = "modifier_A19T_2／A19T_4（無敵的火牆 dummy 版本）沒有任何地方套用；AbilityDamage、spell_duration、arrow_speed、arrow_range 都沒有被讀取。說明的「箭塔距離超過 2500 只受 2.5%」沒有實作（eventfordamage.lua 的 2500 判斷只用在 caster.magic_damage）。" },
          { policy = "as_source",
            text = "孝高死亡後火牆照常點燃，傷害由規則層 require_source_alive 歸零。施放點與施放者重疊時方向為零，所有點都在起點。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:341-347",
            text = "item_soul_adder.lua 的 sound（12 秒迴圈音效）只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A19.txt:309-498", "vscripts/heroes/A_Oda/A19.lua:158-275" },
      },
    },
  }
end
