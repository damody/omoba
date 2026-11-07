-- A19 黑田孝高（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 kuroda_yoshitaka_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill A19 ["20"] = "WERDT" 逐一 AddAbility("A19" .. 字母 .. "_20")，
--   第五個字母前插入 attribute_bonusx；heromap_autoskill A19 ["20"] = "D" 自動學 A19D_20
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:261,364）。
-- W/E/R/T → Q/W/E/R；A19D_20（PASSIVE | NOT_LEARNABLE、Maxlevel 4）放 D，寫在 hero.auto_learn，之後等級 = A19T_20 等級 + 1（A19.lua:542-549）。
-- KV 在 npc/A_Oda/A19.txt:935-1448；腳本 vscripts/heroes/A_Oda/A19.lua。原作沒有 *_20 招式的說明文字（各語系 addon_*.txt 都沒有），
--   招式名稱取自 KV 檔的註解，說明依 KV 與腳本還原（assumptions.md）。英雄單位數值沿用本體 A19。
-- magical_resistance、attribute_bonusx、A19_precache 屬規則層或資產，不列入。
-- 慣例照本體：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven Damage 與 ApplyDamage 不檢查施法者存活，
--   寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A19_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "A19", text = { zh_TW = { name = "黑田孝高（2.0）" }, en = { name = "Kuroda Yoshitaka (2.0)" } },
      auto_learn = { "a19_v20_clinging_flame" } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a19_v20_fire_stratagem", source_id = "A19W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, 12, 10, 8, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A19.txt:948" },
          mana_cost = { 105, 135, 165, 195, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A19.txt:949" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A19.txt:952" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A19.txt:954" },
          radius = { 450, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:26,58 FindUnitsInRadius 450（AOERadius 也是 450）" },
          blast_damage = { 100, 150, 200, 250, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/A_Oda/A19.txt:962" },
          blast_count = { 2, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A19.lua:15-47 count 1..(0.1 + time)，time = 2（kv:time@npc/A_Oda/A19.txt:967），每秒一次（t = 0、1）" },
          blast_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:46 return 1" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/A_Oda/A19.lua:41-43 damage * 0.2" },
          ember_start = { 2, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:49 Timers:CreateTimer(2, ...)" },
          ember_count = { 5, unit = "count", source = "script:vscripts/heroes/A_Oda/A19.lua:50-51 count2 > 5 停止" },
          ember_damage = { 10, 20, 30, 40, unit = "dmg", damage_type = "magical", source = "kv:dmg2@npc/A_Oda/A19.txt:972" },
          burn_time = { 5, unit = "s", source = "kv:modifier_A19W Duration@npc/A_Oda/A19.txt:994" },
          burn_tick = { 0.99, unit = "s", hidden = true, source = "kv:modifier_A19W ThinkInterval@npc/A_Oda/A19.txt:997" },
        },
        text = {
          zh_TW = { name = "鬼謀．火攻",
            body = "使目標地點 {radius} 範圍陷入火海：立即與之後每 {blast_interval} 秒各爆炸一次，共 {blast_count} 次，每次對範圍內非魔免的敵人造成 {blast_damage} 魔法傷害（建築只受 {building_pct}）。施放 {ember_start} 秒後開始，每秒點燃仍在範圍內的非建築敵人，共 {ember_count} 次；被點燃的敵人約每秒受到 {ember_damage} 魔法傷害，最後一次點燃後再燒 {burn_time} 秒。" },
          en = { name = "Stratagem: Fire Attack",
            body = "Sets a {radius} area at the target point ablaze: it explodes immediately and again every {blast_interval} seconds, {blast_count} times in total, each dealing {blast_damage} magic damage to non-spell-immune enemies inside (buildings take only {building_pct}). Starting {ember_start} seconds after the cast, embers ignite non-building enemies still in the area once per second, {ember_count} times; ignited enemies take {ember_damage} magic damage about every second, burning for {burn_time} seconds after the last ignition." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a19w_20_burn", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "餘火", body = "被餘火點燃，約每秒受到 {ember_damage} 魔法傷害。" },
              en = { name = "Embers", body = "Ignited by embers; takes {ember_damage} magic damage about every second." },
            },
            reveal = "public", cue = "a19w_20_burn_glow",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$ember_damage" },
              } },
            },
            notes = "原作 modifier_A19W（2.0 版）：IsDebuff、Duration 5、ThinkInterval 0.99，未寫 IsPurgable。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a19w_20_vo", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a19w_20_fire_field", at = "at", radius = "$radius" },
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
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING,
                  where = "not it.is_building" },
                actions = {
                  { op = "apply_status", status = "a19w_20_burn", to = "it" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:15-47",
            text = "爆炸計時器 count 從 1 起、超過 0.1 + time = 2.1 才停，所以在施放當下與一秒後各爆一次。AbilityDamage 200～500 沒有被讀取，傷害讀 dmg。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:63-67",
            text = "共用本體的 A19W_OnSpellStart：建築改套 modifier_A19W2，但 A19W_20 沒有宣告這個 modifier（也沒有本體的 dmg3），ApplyDataDrivenModifier 找不到 modifier 什麼都不做，建築沒有餘火。2.0 KV 同時拿掉了 modifier 與數值，推定是刻意不燒建築，照原作。" },
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:988-1006",
            text = "餘火 Duration 5 秒、每秒重套一次（刷新持續時間），所以留在火海的敵人從第 2 秒起一直燒到最後一次點燃後 5 秒；重套時 think 計時是否重設依引擎，照設計意圖約每秒一跳。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A19.lua:7-13",
            text = "npc_dummy_unit（7 秒、majia）與火焰特效只是呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A19.txt:935-1008", "vscripts/heroes/A_Oda/A19.lua:3-73" },
      },
      -------------------------------------------------------------------------
      { id = "a19_v20_chain_fire", source_id = "A19E_20", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A19.txt:1023" },
          mana_cost = { 150, 170, 190, 210, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A19.txt:1024" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A19.txt:1027" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A19.txt:1029" },
          projectile_speed = { 1100, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/A_Oda/A19.txt:1069" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius@npc/A_Oda/A19.txt:1070" },
          radius = { 450, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:85 FindUnitsInRadius 450（AOERadius 也是 450）" },
          duration = { 3, unit = "s", source = "kv:modifier_A19E Duration@npc/A_Oda/A19.txt:1095" },
          tick = { 0.99, unit = "s", hidden = true, source = "kv:modifier_A19E ThinkInterval@npc/A_Oda/A19.txt:1103" },
          damage = { 50, 85, 120, 155, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/A_Oda/A19.txt:1037" },
          armor_cut = { 8, 12, 16, 20, unit = "armor", source = "kv:armor@npc/A_Oda/A19.txt:1042", kv = { -8, -12, -16, -20 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          slow = { 30, 40, 50, 60, unit = "pct", source = "kv:slow@npc/A_Oda/A19.txt:1047", kv = { -30, -40, -50, -60 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
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
          { id = "a19e_20_burn", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "連環火", body = "護甲降低 {armor_cut}、移動速度降低 {slow}，並約每秒受到 {damage} 魔法傷害。" },
              en = { name = "Chain Fire", body = "Armor reduced by {armor_cut} and movement speed slowed by {slow}; takes {damage} magic damage about every second." },
            },
            reveal = "public", cue = "a19e_20_burn_glow",
            stats = { armor = "-$armor_cut", move_speed_pct = "-$slow" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A19E（2.0 版）：IsDebuff、Duration 3、ThinkInterval 0.99，Properties MOVESPEED_BONUS_PERCENTAGE %slow、PHYSICAL_ARMOR_BONUS %armor（引擎屬性，不走 dummy.lua 緩速池），未寫 IsPurgable。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a19e_20_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "a19e_20_cast_sound", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              vision = "$projectile_vision", dodgeable = true, cue = "a19e_20_fire_bolt", on_hit = {
                { op = "for_each",
                  select = { from = "it", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "apply_status", status = "a19e_20_burn", to = "it" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A19.txt:1091-1112",
            text = "Duration 3 配 ThinkInterval 0.99，照設計意圖共三跳。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:1025",
            text = "AbilityDamage 200 沒有被使用；腳本讀的 GetCursorPosition() 也沒有使用。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:75-96",
            text = "共用本體的 A19E_OnProjectileHitUnit：擴散以命中時的目標位置為圓心，選取用技能的目標設定（HERO|BASIC、不含魔免）；投射物被躲開就沒有效果。" },
        },
        provenance = { "npc/A_Oda/A19.txt:1010-1115", "vscripts/heroes/A_Oda/A19.lua:75-96" },
      },
      -------------------------------------------------------------------------
      { id = "a19_v20_fire_echo", source_id = "A19R_20", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          int_bonus = { 8, 16, 24, 32, unit = "stat", source = "kv:addint@npc/A_Oda/A19.txt:1137" },
          int_mul = { 1, unit = "ratio", source = "kv:dmg@npc/A_Oda/A19.txt:1142" },
          radius = { 600, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A19.txt:1130", note = "腳本以 GetCastRange() 當搜尋半徑" },
        },
        text = {
          zh_TW = { name = "火焰返響",
            body = "被動：智力增加 {int_bonus}。每次施放技能或使用道具時，孝高周圍 {radius} 內的敵方英雄與一般單位（包含魔免）受到孝高智力 {int_mul} 倍的純粹傷害。切換類技能不會觸發。" },
          en = { name = "Fire Echo",
            body = "Passive: +{int_bonus} intelligence. Whenever Yoshitaka casts an ability or uses an item, enemy heroes and units within {radius} of him (spell-immune ones included) take pure damage equal to {int_mul} × his intelligence. Toggles do not trigger it." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public",
          stats = { int = "$int_bonus" },
          triggers = {
            on_ability_executed = { who = "self",
              filter = "not has_tag(event.ability, toggle)",
              actions = {
                { op = "let", name = "amount", value = "self.int * $int_mul" },
                { op = "cue", id = "a19r_20_echo", at = "self", radius = "$radius" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune" } },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "pure", amount = "amount" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:135-156",
            text = "2.0 版以孝高自己為圓心（本體是施放目標），沒有道具減半、魔免減半與道具排除名單；只排除切換類技能與 attribute_bonusx（屬性加點，v3 不是技能，不必列）。智力在觸發當下讀取，含本技能的加成。傷害走 AMHC。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:1145-1152",
            text = "被動技能的 OnAbilityPhaseStart 語音永遠不會觸發。" },
        },
        provenance = { "npc/A_Oda/A19.txt:1117-1176", "vscripts/heroes/A_Oda/A19.lua:135-156" },
      },
      -------------------------------------------------------------------------
      { id = "a19_v20_siege_fire", source_id = "A19T_20", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 135, 110, 95, unit = "s", source = "kv:AbilityCooldown@npc/A_Oda/A19.txt:1190" },
          mana_cost = { 300, unit = "mana", source = "kv:AbilityManaCost@npc/A_Oda/A19.txt:1191" },
          cast_range = { 1250, unit = "dist", source = "kv:AbilityCastRange@npc/A_Oda/A19.txt:1194" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/A_Oda/A19.txt:1195" },
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
          width = { 200, unit = "dist", source = "kv:arrow_width@npc/A_Oda/A19.txt:1352", note = "腳本寫死 200，數字相同" },
          pulse_first = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:254" },
          pulse_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:271-273 return 1" },
          duration = { 13, unit = "s", source = "kv:duration@npc/A_Oda/A19.txt:1367" },
          pulses = { 14, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A19.lua:254-274 先結算再判斷 GameTime < 施放 + 13，在 0.1、1.1……13.1 秒共 14 次" },
          burn_time = { 1.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:263,265 duration = 1.1" },
          burn_tick = { 0.25, unit = "s", source = "kv:modifier_A19T_3 ThinkInterval@npc/A_Oda/A19.txt:1263" },
          damage = { 80, 120, 160, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/A_Oda/A19.txt:1362" },
          building_damage = { 16, 24, 32, unit = "dmg", damage_type = "magical", source = "kv:dmg2@npc/A_Oda/A19.txt:1372" },
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
          { id = "a19t_20_burn", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" } },
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A19T_3（A19T_20 自己宣告）：IsHidden、Duration 1.01（套用時覆寫成 1.1）、ThinkInterval 0.25，OnCreated 也造成一次傷害；未寫 IsPurgable。" },
          { id = "a19t_20_burn_building", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
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
            { op = "cue", id = "a19t_20_cast_sound", at = "caster" },
            { op = "let", name = "origin", value = "position(caster)" },
            { op = "let", name = "dir", value = "direction(caster, point)" },
            { op = "let", name = "base", value = "offset(origin, dir, $start_offset)" },
            { op = "repeat", count = "$points", as = "k", actions = {
              { op = "reveal", at = "offset(base, dir, $step * (k + 1))", radius = "$width", duration = "$vision_duration", teams = "own" },
              { op = "reveal", at = "offset(base, rotate(dir, $spread), $step * (k + 1))", radius = "$width", duration = "$vision_duration", teams = "own" },
              { op = "reveal", at = "offset(base, rotate(dir, 0 - $spread), $step * (k + 1))", radius = "$width", duration = "$vision_duration", teams = "own" },
            } },
            { op = "cue", id = "a19t_20_fire_walls", at = "base", to = "point", duration = "$duration" },
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
                        actions = { { op = "apply_status", status = "a19t_20_burn_building", to = "it" } },
                        otherwise = { { op = "apply_status", status = "a19t_20_burn", to = "it" } } },
                    } },
                } },
              } },
            } },
          },
          on_upgrade = {
            { op = "set_ability_level", of = "caster", ability = "a19_v20_clinging_flame", level = "ability.level + 1" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:1258-1283",
            text = "Dota 的 modifier 重套時只刷新，不重跑 OnCreated：只有第一次被點燃有立即傷害，之後每秒 4 跳（同本體的判斷，confidence medium）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:158-275",
            text = "共用本體的 A19T_OnSpellStart：三道火牆都從施放者前方 100 的同一點出發，各自每 0.03 秒往前鋪一個點、共 14 個（間距 200），最遠約 2900；結算從 0.1 秒開始、每秒一次，共 14 次，第一次結算時每道只有三個點。選取 DOTA_UNIT_TARGET_ALL 加 MAGIC_IMMUNE_ENEMIES。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:542-549",
            text = "OnUpgrade A19T_20_OnUpgrade 把焰之附著設成本招等級 + 1（焰之附著出生一級，大招三級時為四級）。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:1225-1330",
            text = "modifier_A19T_2／A19T_4（無敵的火牆 dummy 版本）沒有任何地方套用；AbilityDamage 320～640、spell_duration、arrow_speed、arrow_range 都沒有被讀取。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:1205-1215",
            text = "item_soul_adder.lua 的 sound（12 秒迴圈音效）只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A19.txt:1177-1375", "vscripts/heroes/A_Oda/A19.lua:158-275", "vscripts/heroes/A_Oda/A19.lua:542-549" },
      },
      -------------------------------------------------------------------------
      { id = "a19_v20_clinging_flame", source_id = "A19D_20", slot = "D", kind = "passive", max_level = 4, learnable = false,
        tags = { "basic" },
        values = {
          required_level = { 1, unit = "level", source = "design:D 欄一級自動學會（heromap_autoskill A19 [\"20\"] = \"D\"）" },
          mark_time = { 8, unit = "s", source = "kv:modifier_A19D_20 Duration@npc/A_Oda/A19.txt:1434" },
          radius = { 400, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:441-445 FindUnitsInRadius 400" },
          death_damage = { 50, 200, 350, 500, unit = "dmg", damage_type = "magical", source = "kv:dmg2@npc/A_Oda/A19.txt:1401" },
        },
        text = {
          zh_TW = { name = "焰之附著",
            body = "被動：普攻命中非建築目標時，讓它附著火焰 {mark_time} 秒（已附著時不刷新）。附著期間目標死亡，會爆炸對其周圍 {radius} 內的敵方英雄與一般單位造成 {death_damage} 魔法傷害。本技能無法手動學習：出生即有一級，之後每次{@a19_v20_siege_fire}升級，提升為該技能等級加一。" },
          en = { name = "Clinging Flame",
            body = "Passive: attacks that hit a non-building target set it alight for {mark_time} seconds (not refreshed while it lasts). If the target dies while alight, it explodes, dealing {death_damage} magic damage to enemy heroes and units within {radius}. Cannot be learned manually: it starts at level one and becomes {@a19_v20_siege_fire}'s level plus one each time that ability levels up." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a19d_20_flame", duration = "$mark_time", stacking = "keep", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a19d_20_flame_glow",
            text = { zh_TW = { name = "焰之附著", body = "身上附著火焰；在附著期間死亡會爆炸，傷害周圍的友軍。" },
              en = { name = "Clinging Flame", body = "Wreathed in flame; dying while it lasts makes it explode, damaging nearby allies." } },
            triggers = {
              on_death = {
                { op = "let", name = "at", value = "position(self)" },
                { op = "cue", id = "a19d_20_explode", at = "at", radius = "$radius" },
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "damage", to = "it", source = "caster", type = "magical", amount = "$death_damage" },
                  } },
              },
            },
            notes = "原作 modifier_A19D_20：Passive 0、IsHidden 0、Duration 8，OnDeath 呼叫 A19R_old_OnDeath（讀 A19D_20 的 dmg2）。未寫 IsPurgable，暫定 basic。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building and not self.is_illusion and not has_status(event.target, a19d_20_flame)",
              actions = {
                { op = "apply_status", status = "a19d_20_flame", to = "event.target" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:425-435",
            text = "A19D_20_OnAttackLanded 只在目標沒有 modifier_A19D_20 時套上，已附著時不刷新；孝高的幻象不會附著。友軍目標（反補）也會被附著。KV dmg 10～40 沒有被讀取。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:437-456",
            text = "死亡爆炸共用舊版的 A19R_old_OnDeath：以死亡單位為圓心 400，選取用 A19D_20 的目標設定（孝高的敵方、HERO|BASIC、旗標 0，不含魔免），AMHC 魔法傷害、傷害讀爆炸當下的等級。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:1404-1411",
            text = "被動技能的 OnAbilityPhaseStart 語音永遠不會觸發。" },
        },
        provenance = { "npc/A_Oda/A19.txt:1376-1448", "vscripts/heroes/A_Oda/A19.lua:425-456", "vscripts/addon_game_mode.lua:364" },
      },
    },
  }
end
