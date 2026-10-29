-- A15 寧寧（current 版，heromap_skill A15 ["16"] = "WERT"，heromap_autoskill ["16"] = "D"）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R，D 維持 D（npc/A_Oda/A15_2.txt:15-21）。
-- 開場自動學 A15D（vscripts/addon_game_mode.lua:360；eventforpichero.lua:59-62），寫在 hero.auto_learn。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A15_precache 只是資產，都不列入；*_old 技能不屬於 current 版。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；引擎 modifier_invisible 另加 tag engine_invisible（同 A13）。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A15",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A15",
      text = { zh_TW = { name = "寧寧" }, en = { name = "Nene" } },
      auto_learn = { "a15_shadow_step" },
    },
    unit_state = {
      a15e_found = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "月擊選落點時「範圍內有沒被打過的敵人」的暫存旗標，只在同一步內使用" },
      a15r_found = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "影風車選下一跳時「有沒被標記的單位」的暫存旗標，只在同一步內使用" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a15_shadow_step", source_id = "A15D", slot = "D", kind = "active", max_level = 5, learnable = false,
        tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          buff_duration = { 20, unit = "s", source = "kv:Duration@npc/A_Oda/A15.txt:63",
            note = "OnSpellStart ApplyModifier 的 Duration 20.0；modifier_A15D 自己的 Duration %duration 被 ApplyModifier 的 20 覆寫" },
          invis_duration = { 10, 15, 20, 25, 25, unit = "s", kv = { 10, 15, 20, 25 },
            source = "kv:duration", note = "ApplyModifier modifier_invisible Duration %duration",
            source_differs = "KV 只有四個值而技能有五級；第五級推定 Dota 取最後一個值" },
          bonus_damage = { 100, 160, 220, 280, 340, unit = "dmg", damage_type = "physical", source = "kv:A15D_Damage" },
          move_speed_pct = { 5, 10, 15, 20, 25, unit = "pct", source = "kv:A15D_movement_speed" },
        },
        text = {
          zh_TW = { name = "忍法．迷蹤步",
            body = "立即進入隱形 {invis_duration} 秒（不打斷目前動作），並在 {buff_duration} 秒內移動速度提高 {move_speed_pct}、無單位碰撞、攻擊優先度降低。這段期間攻擊出手的瞬間，對非建築目標額外造成 {bonus_damage} 物理傷害並解除隱形與加速；使用任何技能或道具也會解除。本技能無法手動學習：出生即有一級，之後{@a15_exhaust_seal}、{@a15_moon_strike}或{@a15_shadow_windmill}升級時，提升為該技能等級加一。" },
          en = { name = "Ninja Art: Vanishing Step",
            body = "Instantly turns invisible for {invis_duration} seconds without interrupting the current action, and for {buff_duration} seconds gains {move_speed_pct} movement speed, no unit collision and low attack priority. When an attack is launched during that time, it deals {bonus_damage} bonus physical damage to a non-building target and ends both the invisibility and the speed bonus; using any ability or item also ends them. Cannot be learned manually: starts at level one, and whenever {@a15_exhaust_seal}, {@a15_moon_strike} or {@a15_shadow_windmill} levels up it rises to that ability's level plus one." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a15d_stealth", duration = "$buff_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "迷蹤步", body = "移動速度提高 {move_speed_pct}、無單位碰撞、攻擊優先度降低；攻擊出手時對非建築目標額外造成 {bonus_damage} 物理傷害並結束，使用技能或道具也會結束。持續 {buff_duration} 秒。" },
              en = { name = "Vanishing Step", body = "+{move_speed_pct} movement speed, no unit collision and low attack priority; launching an attack deals {bonus_damage} bonus physical damage to a non-building target and ends it, as does using an ability or item. Lasts {buff_duration} seconds." },
            },
            reveal = "public", cue = "a15d_vendetta_start",
            states = { "no_unit_collision", "low_attack_priority" },
            stats = { move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_attack_launch = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "cue", id = "a15d_vendetta_crit", at = "self", to = "event.target" },
                  { op = "damage", to = "event.target", source = "self", type = "physical", amount = "$bonus_damage" },
                } },
                { op = "remove_status", from = "self", status = "a15d_stealth" },
                { op = "remove_status", from = "self", status = "a15d_invisible" },
              },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "a15d_stealth" },
                { op = "remove_status", from = "self", status = "a15d_invisible" },
              } },
            },
            notes = "原作 modifier_A15D：加速、無碰撞、低攻擊優先度與破隱加傷（INVISIBILITY_LEVEL 設 0）。" },
          { id = "a15d_invisible", duration = "$invis_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "隱形", body = "隱形，持續 {invis_duration} 秒；攻擊或使用技能、道具時現身。" },
              en = { name = "Invisible", body = "Invisible for {invis_duration} seconds; attacking or using an ability or item reveals her." },
            },
            reveal = "public", states = { "invisible" }, tags = { "invisible", "engine_invisible" },
            notes = "原作同時套上的引擎 modifier_invisible（隱形本體）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a15d_vo", at = "caster" },
            { op = "apply_status", status = "a15d_stealth", to = "caster" },
            { op = "apply_status", status = "a15d_invisible", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:55-79",
            text = "加速狀態固定 20 秒，隱形是 10/15/20/25 秒：低等級時隱形先結束、加速與破隱加傷仍在；高等級時加速先結束，之後的隱形不再有加傷。說明只寫一個持續時間。" },
          { policy = "no_effect", ref = "npc/A_Oda/A15.txt:65-69",
            text = "OnSpellStart 的 RunScript 呼叫 A15D，但 A15.lua 沒有這個函式；datadriven 記錄錯誤後繼續執行，下一個 ApplyModifier（隱形）照常套上。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:15-36",
            text = "加傷掛在 OnAttack（攻擊出手瞬間）直接以 AMHC:Damage 造成物理傷害，之後攻擊即使 miss、被閃避或投射物被躲，額外傷害照樣打出。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A15.lua:16",
            text = "A15D_End 的 `if not X or X` 判斷恆真，等於沒有條件。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:4-12",
            text = "W／E／R 的 OnUpgrade 都呼叫 A15_Levelup：D 等級低於該技能等級 + 1 時提升；T 沒有這段，所以 D 最高五級。" },
          { policy = "as_source",
            text = "on_ability_executed 在施法者自己的 on_cast 之前觸發（§12.5），所以這次施放 A15D 本身不會拆掉剛套上的隱形；對道具使用也會觸發現身。" },
          { policy = "note", ref = "npc/A_Oda/A15.txt:55-79",
            text = "ApplyModifier 的 Target 只有 Center = CASTER、沒有 Radius，依引擎行為只套到施法者本人。" },
          { policy = "note", ref = "npc/A_Oda/A15.txt:83-126",
            text = "modifier_A15D 未寫 IsPurgable，依本檔慣例寫 dispel = basic；引擎 modifier_invisible 的驅散屬性未確認。" },
        },
        provenance = { "npc/A_Oda/A15.txt:22-146", "vscripts/heroes/A_Oda/A15.lua:4-36", "vscripts/addon_game_mode.lua:360" },
      },
      -------------------------------------------------------------------------
      { id = "a15_exhaust_seal", source_id = "A15W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 26, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, 135, 145, 155, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 4, unit = "s", source = "kv:A15W_duration" },
          dot_damage = { 30, 40, 50, 60, unit = "dmg", damage_type = "magical", source = "kv:A15W_dotDamage" },
          tick = { 1, unit = "s", source = "kv:modifier_A15W_debuff ThinkInterval" },
          damage_reduction = { 40, 55, 70, 85, unit = "pct",
            source = "kv:A15W_outgoingDamage", kv = { -40, -55, -70, -85 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE" },
        },
        text = {
          zh_TW = { name = "忍法．竭印",
            body = "對一名敵人施加竭印 {duration} 秒：目標被沉默，基礎攻擊力降低 {damage_reduction}，並每 {tick} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Ninja Art: Exhaustion Seal",
            body = "Seals an enemy for {duration} seconds: the target is silenced, its base attack damage is reduced by {damage_reduction}, and it takes {dot_damage} magic damage every {tick} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a15w_seal", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "竭印", body = "沉默，基礎攻擊力降低 {damage_reduction}，並每 {tick} 秒受到 {dot_damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Exhaustion Seal", body = "Silenced, base attack damage reduced by {damage_reduction}, and takes {dot_damage} magic damage every {tick} seconds for {duration} seconds." },
            },
            reveal = "public", cue = "a15w_seal", states = { "silence" },
            stats = { base_attack_damage_pct = "-$damage_reduction" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage" },
              } },
            },
            notes = "原作 modifier_A15W_debuff：IsDebuff、未寫 IsPurgable。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a15w_seal", to = "target" },
            { op = "cue", id = "a15w_cast_sound", at = "caster" },
          },
          on_upgrade = { { op = "call", routine = "a15d_level_sync", args = { lvl = "ability.level" } } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A15.txt:202-211",
            text = "ThinkInterval 1.0 配持續 4 秒，最後一跳是否落在到期當下依引擎時序；照設計意圖每秒一跳共四跳。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:196",
            text = "降低的是基礎攻擊力（BASEDAMAGEOUTGOING_PERCENTAGE），綠字攻擊力不受影響；原作說明寫「攻擊傷害」。" },
          { policy = "note", ref = "npc/A_Oda/A15.txt:178-183",
            text = "item_soul_adder.lua 的 sound 只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A15.txt:147-233", "vscripts/heroes/A_Oda/A15.lua:38-44" },
      },
      -------------------------------------------------------------------------
      { id = "a15_moon_strike", source_id = "A15E", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 16, 14, 12, 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, 145, 165, 185, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 0.4, 0.5, 0.6, 0.7, unit = "s", source = "kv:A15E_duration" },
          radius = { 400, unit = "dist", source = "kv:A15E_radius" },
          moon_radius = { 180, unit = "dist", source = "kv:A15E_moonRadius" },
          damage = { 145, unit = "dmg", damage_type = "magical", source = "kv:A15E_damage" },
          stun = { 1, unit = "s", source = "kv:A15E_stunDuration" },
          max_moon = { 4, 5, 6, 7, unit = "count", source = "kv:A15E_maxMoon" },
          max_hit = { 4, unit = "count", hidden = true, source = "kv:A15E_maxHit" },
          hits_allowed = { expr = "$max_hit + 1", unit = "count", source = "derived",
            note = "腳本以「層數 > maxHit 才跳過」判斷，所以實際可以被打 maxHit + 1 次" },
          moon_interval = { 0.1, unit = "s", source = "script:vscripts/heroes/A_Oda/A15.lua:110 return 0.1" },
          hit_mark = { 0.8, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:103 duration = 0.8" },
        },
        text = {
          zh_TW = { name = "忍法．月擊",
            body = "對目標地點持續施法，每 {moon_interval} 秒在半徑 {radius} 內落下一道月束，最多 {max_moon} 道。月束優先落在範圍內這次還沒被打中的敵人頭上，沒有這種敵人時落在隨機位置；落點 {moon_radius} 內的敵人受到 {damage} 魔法傷害並暈眩 {stun} 秒。每個單位最多被打中 {hits_allowed} 次。" },
          en = { name = "Ninja Art: Moon Strike",
            body = "Channels on the target point, calling down a moonbeam within {radius} every {moon_interval} seconds, up to {max_moon}. Each beam lands on an enemy in the area not yet hit by this cast if there is one, otherwise at a random spot; enemies within {moon_radius} of it take {damage} magic damage and are stunned for {stun} seconds. Each unit can be hit at most {hits_allowed} times." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a15e_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被月束擊中，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Struck by a moonbeam and stunned for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 AddNewModifier 引擎內建 modifier_stunned；受狀態抗性縮短。" },
          { id = "a15e_hit", duration = "$hit_mark", stacking = { mode = "no_refresh" }, dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 modifier_A15E_hit（IsHidden、IsPurgable 0）：第一次命中時套 0.8 秒、層數 1，之後只 SetStackCount 不刷新時間。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "repeat", interval = "$moon_interval", first = "0", count = "$max_moon", ["while"] = "caster.is_channeling",
              actions = {
                { op = "set_state", var = "a15e_found", value = "false" },
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    where = "not has_status(it, a15e_hit)", limit = "1" },
                  actions = {
                    { op = "set_state", var = "a15e_found", value = "true" },
                    { op = "call", routine = "a15e_moon", args = { center = "position(it)" } },
                  } },
                { op = "if", cond = "not state.a15e_found", actions = {
                  { op = "call", routine = "a15e_moon", args = { center = "random_point(at, 0, $radius)" } },
                } },
              } },
          },
          on_upgrade = { { op = "call", routine = "a15d_level_sync", args = { lvl = "ability.level" } } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A15.lua:94",
            text = "判斷寫成「層數 > maxHit 才跳過」，第一次命中設一層、之後每次加一，所以同一單位實際可被打中 maxHit + 1 = 5 次（暈眩可疊到五次）；說明與 KV 名稱都是最多 4 次。資料照原作。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:66-76",
            text = "優先目標取 FindUnitsInRadius（FIND_ANY_ORDER）裡第一個沒有命中標記的單位，v3 以 entity id 排序取第一個；沒有時落在 RandomVector(RandomInt(0, radius)) 的隨機點，分布與 random_point 略有不同（原作半徑均勻、不是面積均勻）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:59-111",
            text = "第一道在施法開始的下一步落下，之後每 0.1 秒一道，引導結束或被打斷就停；落點選取與傷害都用技能的目標設定（HERO|BASIC、不含魔免）。" },
          { policy = "design_intent", ref = "npc/A_Oda/A15.txt:252",
            text = "引導時間 0.4~0.7 秒與最多 4~7 道、每 0.1 秒一道相符；最後一道是否趕在引導結束前由引擎時序決定，照設計意圖落滿 maxMoon 道。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:98-104",
            text = "命中標記 0.8 秒、加層不刷新；引導最長 0.7 秒，所以標記在整次施放內有效。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A15.lua:102", text = "print(unit) 是除錯輸出。" },
        },
        provenance = { "npc/A_Oda/A15.txt:235-317", "vscripts/heroes/A_Oda/A15.lua:46-112" },
      },
      -------------------------------------------------------------------------
      { id = "a15_shadow_windmill", source_id = "A15R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          jump_max = { 2, 3, 4, 5, unit = "count", hidden = true, source = "kv:jump_max" },
          bounces = { expr = "$jump_max - 1", unit = "count", source = "derived",
            note = "第一個目標也消耗一次 jump_count，實際彈跳 jump_max - 1 次（與原作說明 1/2/3/4 相符）" },
          falloff = { -20, unit = "pct", source = "kv:bonus_damage（-0.2，每跳乘上 1 + bonus_damage）" },
          jump_radius = { 450, unit = "dist", source = "kv:jump_radius" },
          jump_radius_per_ult = { 50, unit = "dist",
            source = "script:vscripts/heroes/A_Oda/A15.lua:162-163 jump_radius + (A15T 等級 - 1) * 50" },
          building_pct = { 30, unit = "pct", source = "script:vscripts/heroes/A_Oda/A15.lua:214-216" },
          bounce_speed = { 1300, unit = "speed", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:262" },
          bounce_vision = { 225, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:263" },
          reveal_radius = { 300, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:251 AddFOWViewer" },
          reveal_duration = { 2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:251 AddFOWViewer" },
          hit_mark = { 0.15, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:190" },
          hit_mark_building = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:191-193" },
        },
        text = {
          zh_TW = { name = "忍法．影風車",
            body = "被動：普攻命中後，飛標從目標彈向 {jump_radius} 內最近的其他敵人（包含建築與魔免），共彈跳 {bounces} 次；每跳的傷害是普攻傷害乘上（一加 {falloff}）的跳數次方，造成物理傷害，對建築只有 {building_pct}。{@a15_shadow_moon_assault}每高一級，彈跳半徑再加 {jump_radius_per_ult}；未學時半徑少 {jump_radius_per_ult}。" },
          en = { name = "Ninja Art: Shadow Pinwheel",
            body = "Passive: after an attack lands, a blade bounces from the target to the nearest other enemy within {jump_radius} (buildings and spell-immune units included), {bounces} times in total; each bounce deals the attack's damage times (one plus {falloff}) to the power of the bounce number as physical damage, and only {building_pct} of that to buildings. Each level of {@a15_shadow_moon_assault} beyond the first adds {jump_radius_per_ult} to the bounce radius; without it the radius is {jump_radius_per_ult} smaller." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              { op = "call", routine = "a15r_start", args = { victim = "event.target", base = "event.attack.damage" } },
            },
          },
        },
        statuses = {
          { id = "a15r_hit", duration = "$hit_mark", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 target.hit[instance]：0.15 秒（建築 0.1 秒）後清除；v3 不分實例。" },
        },
        events = {
          on_upgrade = { { op = "call", routine = "a15d_level_sync", args = { lvl = "ability.level" } } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A15.lua:217-219",
            text = "EXCLUDE_TARGET_NAME2 分支寫錯變數名稱（damage_table，正確是 damageTable），彈跳到侍大將（com_general_oda／com_general_unified，npc/npc_units_custom_load.txt:6178-6180）時執行期報錯：該單位不受傷、不吸血，彈跳在這裡中斷；原意是對侍大將只造成一半傷害。資料照原作，以單位 tag general（v3.1）辨識侍大將。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A15.lua:205-207",
            text = "「傷害低於兩成就以兩成計」寫成改 base_damage 而不是 new_damage，沒有效果；彈跳傷害沒有下限。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:162-163",
            text = "彈跳半徑 = 450 + (A15T 等級 − 1) × 50；未學 A15T 時是 400，A15T 一到三級是 450／500／550。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:185-199,236-248",
            text = "命中標記只保留 0.15 秒（建築 0.1 秒），而每跳飛行通常更久，所以只會避開剛打到的單位，可以在兩個單位之間來回彈；所有附近單位都帶標記時，改選範圍內最後一個（最遠）不是目前目標的單位。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A15.lua:173-180",
            text = "原作以「目前目標相同」找彈跳實例，多條彈跳同時打到同一單位時計數會互相干擾；v3 各條彈跳獨立、命中標記不分實例，差異只在極端情況。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:148-151,165-168",
            text = "A15T 生效時 jump_max 翻倍，所以彈跳 2 × jump_max − 1 次（3/5/7/9），不是說明的兩倍（2/4/6/8）；翻倍在彈跳開始時決定。原作每跳重新讀 jump_max，A15T 在彈跳途中結束時，剩下的跳數比較與次方會錯亂（可能不造成傷害），v3 以開始時的值為準。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:356-364",
            text = "基礎傷害是 OnAttackLanded 的 %attack_damage（護甲前的攻擊傷害）；第一個目標只吃普攻本身，彈跳以 ApplyDamage 造成物理傷害（不算普攻、不觸發攻擊特效）。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:324-326",
            text = "選取帶 FOW_VISIBLE 與 MAGIC_IMMUNE_ENEMIES：只跳向看得到的單位，魔免單位照樣受物理傷害；建築也會被彈到。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:220-223",
            text = "學會 A15T 後彈跳傷害也吸血，量是未乘建築倍率的傷害 × 只算額外護甲的減傷倍率；建築與魔免目標不吸血。" },
        },
        provenance = { "npc/A_Oda/A15.txt:319-401", "vscripts/heroes/A_Oda/A15.lua:114-279", "vscripts/library/common/dummy.lua:327-330" },
      },
      -------------------------------------------------------------------------
      { id = "a15_shadow_moon_assault", source_id = "A15T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 155, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          duration = { 10, 15, 18, unit = "s", source = "kv:A15T_duration" },
          lifesteal = { 10, 15, 20, unit = "pct", source = "kv:A15T_lifeSteal" },
          arrow_count = { 2, 3, 4, unit = "count", source = "kv:arrow_count" },
          split_range = { 800, unit = "dist", source = "kv:range", note = "腳本 FindUnitsInRadius 寫死 800，數字相同" },
          arrow_speed = { 1200, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          split_ratio = { 45, unit = "pct", kv = { 50 },
            source = "script:vscripts/heroes/A_Oda/A15.lua:471 GetAverageTrueAttackDamage(target) * 0.9 * 0.5",
            source_differs = "原作說明寫 50% 傷害，腳本是平均攻擊 × 0.9 × 0.5" },
          jump_mult = { 2, unit = "ratio", source = "script:vscripts/heroes/A_Oda/A15.lua:148-150 jump_max*2" },
        },
        text = {
          zh_TW = { name = "忍法密傳．影月攻勢",
            body = "被動：普攻命中非建築、非魔免的敵人時，回復攻擊傷害（只以目標額外護甲減免後）{lifesteal} 的生命；{@a15_shadow_windmill}的彈跳傷害也能吸血。主動：{duration} 秒內，{@a15_shadow_windmill}的彈跳次數改以 {jump_mult} 倍計算，且每次攻擊出手時，另外向 {split_range} 內最近、看得到的其他敵人各射出一支飛標，最多 {arrow_count} 支；飛標命中時以平均攻擊力 {split_ratio} 為基礎觸發一次{@a15_shadow_windmill}的彈跳，飛標本身不造成傷害。" },
          en = { name = "Secret Ninja Art: Shadow Moon Assault",
            body = "Passive: attacks that land on non-building, non-spell-immune enemies heal Nene for {lifesteal} of the attack damage (reduced only by the target's bonus armor); {@a15_shadow_windmill} bounce damage also heals her. Active: for {duration} seconds, {@a15_shadow_windmill} bounces {jump_mult} times as many times, and each time Nene launches an attack she also throws a blade at each of up to {arrow_count} other nearest visible enemies within {split_range}; a blade that hits triggers a {@a15_shadow_windmill} bounce based on {split_ratio} of her average attack damage, and the blade itself deals no damage." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building and not event.target.is_magic_immune and not has_status_tag(event.target, isvoid)",
              actions = {
                { op = "heal", to = "self", source = "self",
                  amount = "event.attack.damage * armor_multiplier(event.target.bonus_armor) * $lifesteal" },
                { op = "cue", id = "a15t_lifesteal", at = "self" },
              } },
          },
        },
        statuses = {
          { id = "a15t_volley", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "影月攻勢", body = "{@a15_shadow_windmill}的彈跳次數以 {jump_mult} 倍計算；每次攻擊出手時，另外向 {split_range} 內最近的其他敵人射出最多 {arrow_count} 支飛標，命中時觸發彈跳。持續 {duration} 秒。" },
              en = { name = "Shadow Moon Assault", body = "{@a15_shadow_windmill} bounces {jump_mult} times as many times; each attack launched also throws blades at up to {arrow_count} other nearest enemies within {split_range}, each triggering a bounce on hit. Lasts {duration} seconds." },
            },
            reveal = "public", cue = "a15t_glow",
            triggers = {
              on_attack_launch = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$split_range", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune", "visible_only" }, exclude = "event.target",
                    where = "not has_status_tag(it, engine_invisible)", order = "nearest", limit = "$arrow_count" },
                  actions = {
                    { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$arrow_speed", dodgeable = true,
                      cue = "a15t_split_arrow", on_hit = {
                        { op = "cue", id = "a15t_split_hit_sound", at = "it" },
                        { op = "call", routine = "a15r_start",
                          args = { victim = "it", base = "avg_attack_against(self, it) * $split_ratio" } },
                      } },
                  } },
              },
            },
            notes = "原作 modifier_A15T2（未寫 IsPurgable）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a15t_volley", to = "caster" },
            { op = "cue", id = "a15t_cast_sound", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:461-475",
            text = "依 Dota 的傷害型別語意推定：A15T 沒有 AbilityUnitDamageType，GetAbilityDamageType() 為 DAMAGE_TYPE_NONE，這種 ApplyDamage 不扣血；分裂飛標只觸發影風車的彈跳，自己不造成傷害（D13，2026-10-29，confidence medium）。說明與設計的落差是上面那條 fix_candidate。" },
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A15.lua:461-475",
            text = "A15T 沒有 AbilityUnitDamageType，分裂飛標的 ApplyDamage 類型是 NONE，推定不造成傷害；說明寫「50% 傷害的飛標」。資料照推定的原作行為（不傷害），只保留飛標觸發的影風車彈跳。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:471-473",
            text = "飛標命中呼叫 A15R_OnAttackLanded，以平均攻擊 × 0.45 為基礎開始一條影風車彈跳；第一個目標不吃彈跳傷害，之後每跳照影風車規則衰減。A15R 未學時以等級零讀 jump_max，推定取第一級的值。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:299-314",
            text = "吸血自己算護甲：GetPhysicalArmorValue(true) 只取額外護甲，乘上 1 − 0.06a／(1 + 0.06|a|)，不看最終傷害；基礎量是 %attack_damage（護甲前）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:292",
            text = "目標帶 isvoid（A32R 等技能設定，A32.lua:258 設了之後 current 版不會清除）時普攻不吸血；以 has_status_tag(isvoid) 表達（登錄表 status_tags.isvoid，由 A32 的 a32r_void 狀態宣告）。彈跳吸血不檢查 isvoid。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A15.lua:281-286",
            text = "A15T_OnSpellStart 沒有任何 KV 引用，是死碼；吸血由 Passive 的 modifier_A15T 提供，學會即生效。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:432-458",
            text = "分裂目標：800 內 HERO|BASIC、看得到、沒有 modifier_invisible、不是目前攻擊目標，依距離由近到遠取 arrow_count 個；在攻擊出手（OnAttack）時發射，攻擊建築時也會分裂。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A15.lua:433-437",
            text = "以 pairs 迭代中 table.remove 移除 majia 單位會跳過下一個元素；v3 選取預設不含 dummy，差異只在 majia 單位相鄰時。" },
          { policy = "no_effect", ref = "npc/A_Oda/A15.txt:487-491",
            text = "total_arrow_count 與腳本讀的 damage_modifier 都沒有作用。" },
        },
        provenance = { "npc/A_Oda/A15.txt:403-513", "vscripts/heroes/A_Oda/A15.lua:281-314", "vscripts/heroes/A_Oda/A15.lua:415-475",
          "vscripts/heroes/A_Oda/A32.lua:250-258" },
      },
    },
    routines = {
      a15d_level_sync = {
        params = { lvl = "num" },
        actions = {
          { op = "if", cond = "ability_level(caster, a15_shadow_step) < lvl + 1", actions = {
            { op = "set_ability_level", of = "caster", ability = "a15_shadow_step", level = "lvl + 1" },
          } },
        },
        notes = "A15_Levelup（A15.lua:4-12）：D 等級低於該技能等級 + 1 時提升。",
      },
      a15e_moon = {
        params = { center = "point" },
        actions = {
          { op = "cue", id = "a15e_moonfall", at = "center" },
          { op = "for_each",
            select = { from = "center", shape = "circle", radius = "$a15_moon_strike.moon_radius", team = "enemy", types = BASIC },
            actions = {
              { op = "if", cond = "stacks(it, a15e_hit) <= $a15_moon_strike.max_hit", actions = {
                { op = "apply_status", status = "a15e_stun", to = "it" },
                { op = "if", cond = "has_status(it, a15e_hit)",
                  actions = { { op = "add_stacks", to = "it", status = "a15e_hit", count = 1, refresh = false } },
                  otherwise = { { op = "apply_status", to = "it", status = "a15e_hit", stacks = "1" } } },
                { op = "damage", to = "it", type = "magical", amount = "$a15_moon_strike.damage" },
              } },
            } },
        },
      },
      -- 影風車：起點（普攻命中或分裂飛標命中）。第一個目標只標記、不吃彈跳傷害。
      a15r_start = {
        params = { victim = "unit", base = "num" },
        actions = {
          { op = "apply_status", status = "a15r_hit", to = "victim",
            duration = "select(victim.is_building, $a15_shadow_windmill.hit_mark_building, $a15_shadow_windmill.hit_mark)" },
          { op = "let", name = "jumps",
            value = "select(has_status(caster, a15t_volley), $a15_shadow_windmill.jump_max * $a15_shadow_moon_assault.jump_mult, $a15_shadow_windmill.jump_max)" },
          { op = "if", cond = "jumps - 1 > 0", actions = {
            { op = "call", routine = "a15r_jump",
              args = { from = "victim", base = "base", scale = "1 + $a15_shadow_windmill.falloff", left = "jumps - 1" } },
          } },
        },
      },
      -- 選下一跳：最近、沒有命中標記；全部都有標記時取最遠的其他單位。
      a15r_jump = {
        params = { from = "unit", base = "num", scale = "num", left = "num" },
        actions = {
          { op = "let", name = "jr",
            value = "$a15_shadow_windmill.jump_radius + (ability_level(caster, a15_shadow_moon_assault) - 1) * $a15_shadow_windmill.jump_radius_per_ult" },
          { op = "set_state", var = "a15r_found", value = "false" },
          { op = "for_each",
            select = { from = "from", shape = "circle", radius = "jr", team = "enemy", types = BASIC_BUILDING,
              flags = { "include_magic_immune", "visible_only" }, where = "not has_status(it, a15r_hit)",
              order = "nearest", limit = "1" },
            actions = {
              { op = "set_state", var = "a15r_found", value = "true" },
              { op = "call", routine = "a15r_fire", args = { from = "from", to = "it", base = "base", scale = "scale", left = "left" } },
            } },
          { op = "if", cond = "not state.a15r_found", actions = {
            { op = "for_each",
              select = { from = "from", shape = "circle", radius = "jr", team = "enemy", types = BASIC_BUILDING,
                flags = { "include_magic_immune", "visible_only" }, exclude = "from", order = "farthest", limit = "1" },
              actions = {
                { op = "call", routine = "a15r_fire", args = { from = "from", to = "it", base = "base", scale = "scale", left = "left" } },
              } },
          } },
        },
      },
      a15r_fire = {
        params = { from = "unit", to = "unit", base = "num", scale = "num", left = "num" },
        actions = {
          { op = "reveal", at = "position(to)", radius = "$a15_shadow_windmill.reveal_radius",
            duration = "$a15_shadow_windmill.reveal_duration", teams = "own" },
          { op = "projectile", kind = "tracking", from = "from", to = "to", speed = "$a15_shadow_windmill.bounce_speed",
            vision = "$a15_shadow_windmill.bounce_vision", dodgeable = true, cue = "a15r_glaive", on_hit = {
              { op = "call", routine = "a15r_land", args = { victim = "it", base = "base", scale = "scale", left = "left" } },
            } },
        },
      },
      a15r_land = {
        params = { victim = "unit", base = "num", scale = "num", left = "num" },
        actions = {
          { op = "apply_status", status = "a15r_hit", to = "victim",
            duration = "select(victim.is_building, $a15_shadow_windmill.hit_mark_building, $a15_shadow_windmill.hit_mark)" },
          -- 原作對侍大將（EXCLUDE_TARGET_NAME2）寫錯變數名稱而報錯：不受傷、不吸血，這條彈跳中斷
          { op = "if", cond = "not has_tag(victim, general)", actions = {
            { op = "if",
              cond = "ability_level(caster, a15_shadow_moon_assault) > 0 and not victim.is_building and not victim.is_magic_immune",
              actions = {
                { op = "heal", to = "caster", source = "caster",
                  amount = "base * scale * armor_multiplier(victim.bonus_armor) * $a15_shadow_moon_assault.lifesteal" },
              } },
            { op = "damage", to = "victim", type = "physical",
              amount = "select(victim.is_building, base * scale * $a15_shadow_windmill.building_pct, base * scale)" },
            { op = "if", cond = "left - 1 > 0", actions = {
              { op = "call", routine = "a15r_jump",
                args = { from = "victim", base = "base", scale = "scale * (1 + $a15_shadow_windmill.falloff)", left = "left - 1" } },
            } },
          } },
        },
      },
    },
  }
end
