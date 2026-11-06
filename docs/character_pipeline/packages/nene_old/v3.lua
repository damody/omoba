-- A15 寧寧（舊版，11 版 heromap_skill A15 ["11"] = "WERT"，沒有 D）— package schema v3。
-- 版本：選擇 11 版時 tofar_goback.lua choose_11 拔掉全部技能，依 _G.heromap_skill[A15]["11"] = "WERT"
-- 依序 AddAbility("A15W_old"、"A15E_old"、"A15R_old"、"A15T_old")（vscripts/items/Addon_Items/tofar_goback.lua:96-140；
-- vscripts/addon_game_mode.lua:257,360），自動學習清單為空（本體 A15D 的自動學習只在 16 版）。槽位 W/E/R/T → Q/W/E/R。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A15_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"。
return function(ctx)
  local UNITS = { "hero", "creep", "neutral", "summon" }
  local UNITS_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A15_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A15", text = { zh_TW = { name = "寧寧（舊版）" }, en = { name = "Nene (Old)" } } },
    unit_state = {
      a15_old_bounce_found = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "影風車選下一跳時「有沒被標記的單位」的暫存旗標，只在同一步內使用（同本體 a15r_found）" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a15_old_vanishing_step", source_id = "A15W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 110, 120, 130, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 10, 15, 20, 25, unit = "s", source = "kv:A15W_old_duration" },
          bonus_damage = { 100, 160, 220, 280, unit = "dmg", damage_type = "physical", source = "kv:A15W_old_damage" },
          move_speed_pct = { 10, 15, 20, 25, unit = "pct", source = "kv:A15W_old_movespd" },
        },
        text = {
          zh_TW = { name = "忍法．迷蹤步",
            body = "立即進入隱形 {duration} 秒（不打斷目前動作），移動速度提高 {move_speed_pct}、無單位碰撞、攻擊優先度降低。隱形中攻擊出手的瞬間，對非建築目標額外造成 {bonus_damage} 物理傷害並現身；使用任何技能或道具也會現身。" },
          en = { name = "Ninja Art: Vanishing Step",
            body = "Instantly turns invisible for {duration} seconds without interrupting the current action, gaining +{move_speed_pct} movement speed, no unit collision and low attack priority. When an attack is launched while invisible, it deals {bonus_damage} bonus physical damage to a non-building target and reveals Nene; using any ability or item also reveals her." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a15w_old_stealth", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "迷蹤步", body = "移動速度提高 {move_speed_pct}、無單位碰撞、攻擊優先度降低；攻擊出手時對非建築目標額外造成 {bonus_damage} 物理傷害並現身，使用技能或道具也會現身。持續 {duration} 秒。" },
              en = { name = "Vanishing Step", body = "+{move_speed_pct} movement speed, no unit collision and low attack priority; launching an attack deals {bonus_damage} bonus physical damage to a non-building target and reveals Nene, as does using an ability or item. Lasts {duration} seconds." },
            },
            reveal = "public", cue = "a15w_old_vendetta_start",
            states = { "no_unit_collision", "low_attack_priority" },
            stats = { move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_attack_launch = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "cue", id = "a15w_old_vendetta_crit", at = "self", to = "event.target" },
                  { op = "damage", to = "event.target", source = "self", type = "physical", amount = "$bonus_damage" },
                } },
                { op = "remove_status", from = "self", status = "a15w_old_stealth" },
                { op = "remove_status", from = "self", status = "a15w_old_invisible" },
              },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "a15w_old_stealth" },
                { op = "remove_status", from = "self", status = "a15w_old_invisible" },
              } },
            },
            notes = "原作 modifier_A15W_old：加速、無碰撞、低攻擊優先度與破隱邏輯（INVISIBILITY_LEVEL 0）。" },
          { id = "a15w_old_invisible", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "隱形", body = "隱形，持續 {duration} 秒；攻擊或使用技能、道具時現身。" },
              en = { name = "Invisible", body = "Invisible for {duration} seconds; attacking or using an ability or item reveals her." },
            },
            reveal = "public", states = { "invisible" }, tags = { "invisible", "engine_invisible" },
            notes = "原作同時套上的引擎 modifier_invisible。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a15w_old_stealth", to = "caster" },
            { op = "apply_status", status = "a15w_old_invisible", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:316-336",
            text = "A15W_old_End 掛在 OnAttack（出手瞬間）直接 AMHC 物理傷害；之後攻擊即使 miss，額外傷害照樣打出。`if not X or X` 恆真。" },
          { policy = "as_source",
            text = "on_ability_executed 在施法者自己的 on_cast 之前觸發，這次施放本身不會拆掉剛套上的隱形；道具使用也會現身。" },
          { policy = "note", ref = "npc/A_Oda/A15.txt:549-592", text = "modifier_A15W_old 未寫 IsPurgable，依本檔慣例寫 dispel = basic。" },
        },
        provenance = { "npc/A_Oda/A15.txt:515-613", "vscripts/heroes/A_Oda/A15.lua:316-336" },
      },
      -------------------------------------------------------------------------
      { id = "a15_old_moon_watch", source_id = "A15E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 36, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 220, 240, 260, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 13, 15, 17, 19, unit = "s", source = "kv:A15E_duration" },
          radius = { 1300, 1600, 1900, 2200, unit = "dist", source = "kv:A15E_radius" },
          vision = { 1000, 1300, 1600, 1900, unit = "dist", source = "kv:A15E_vision" },
          sight_tick = { 0.2, unit = "s", hidden = true, source = "kv:modifier_A15E_old_show_invi_aura ThinkInterval" },
          sight_time = { 1, unit = "s", hidden = true, source = "kv:modifier_truesight Duration" },
        },
        text = {
          zh_TW = { name = "忍法．觀月",
            body = "在任意地點升起明月，持續 {duration} 秒：提供半徑 {vision} 的飛行視野，並讓半徑 {radius} 內的敵方單位與建築（包含魔免與無敵者）現形。" },
          en = { name = "Ninja Art: Moon Watch",
            body = "Raises a moon at any point for {duration} seconds: it grants flying vision in a radius of {vision} and reveals enemy units and buildings (spell-immune and invulnerable ones included) within {radius}." },
        },
        target = { type = "point", aoe_radius = "$vision" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a15e_old_truesight", duration = "$sight_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "revealed" }, tags = { "true_sight" },
            notes = "引擎 modifier_truesight（每 0.2 秒對範圍內敵人套 1 秒）。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a15e_old_moonfall", at = "at", duration = "$duration" },
            { op = "reveal", at = "at", radius = "$vision", duration = "$duration", teams = "own", ignore_terrain = true },
            { op = "repeat", interval = "$sight_tick", count = "$duration / $sight_tick", first = 0, actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon", "building" }, flags = { "include_magic_immune", "include_invulnerable" } },
                actions = { { op = "apply_status", status = "a15e_old_truesight", to = "it" } } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:344-349",
            text = "原作在目標點放 npc_dummy_unit（基礎視野 0，majia_vison，modifier_kill duration），身上的 modifier_A15E_old_show_invi_aura 給 BONUS_DAY／NIGHT_VISION = A15E_vision 與 FLYING；資料以 reveal（ignore_terrain）表達。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:648-663",
            text = "每 0.2 秒對 A15E_radius 內敵人（HERO|CREEP|BASIC|BUILDING，含魔免與無敵）套 1 秒 modifier_truesight；現形半徑比視野半徑大 300，外圈的單位要有其他視野才看得到。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:629",
            text = "AbilityCastRange 0，推定為不限距離（資料不寫 target.range）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A15.lua:352-365", text = "每秒重建的月落粒子屬呈現。" },
        },
        provenance = { "npc/A_Oda/A15.txt:615-695", "vscripts/heroes/A_Oda/A15.lua:338-366",
          "npc/npc_units_custom_load.txt:5111-5145", "npc/npc_abilities_custom_load.txt:5628-5651" },
      },
      -------------------------------------------------------------------------
      { id = "a15_old_shadow_windmill", source_id = "A15R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          jump_max = { 2, 4, 6, 8, unit = "count", hidden = true, source = "kv:jump_max" },
          bounces = { expr = "$jump_max - 1", unit = "count", source = "derived",
            note = "同本體：第一個目標也消耗一次 jump_count，實際彈跳 jump_max - 1 次" },
          falloff = { -30, unit = "pct", source = "kv:bonus_damage（-0.30，每跳乘上 1 + bonus_damage）" },
          jump_radius = { 700, unit = "dist", source = "kv:jump_radius",
            note = "本體腳本加 (A15T 等級 - 1) × 50；舊版沒有 A15T，照 KV 固定 700" },
          building_pct = { 30, unit = "pct", source = "script:vscripts/heroes/A_Oda/A15.lua:214-216" },
          general_pct = { 50, unit = "pct", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:217-219 侍大將分支的 damage * 0.5",
            note = "FC-050（本體 D13 修法）：原作寫錯變數名稱而報錯，照原意對侍大將造成一半傷害" },
          bounce_speed = { 1300, unit = "speed", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:262" },
          bounce_vision = { 225, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:263" },
          reveal_radius = { 300, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:251 AddFOWViewer" },
          reveal_duration = { 2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:251 AddFOWViewer" },
          hit_mark = { 0.15, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:190" },
          hit_mark_building = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:191-193" },
        },
        text = {
          zh_TW = { name = "忍法．影風車",
            body = "被動：普攻命中後，飛標從目標彈向 {jump_radius} 內最近的其他敵人（包含建築與魔免），共彈跳 {bounces} 次；每跳的傷害是普攻傷害乘上（一加 {falloff}）的跳數次方，造成物理傷害，對建築只有 {building_pct}。" },
          en = { name = "Ninja Art: Shadow Windmill",
            body = "Passive: after an attack lands, a blade bounces from the target to the nearest other enemy within {jump_radius} (buildings and spell-immune units included), {bounces} times in total; each bounce deals the attack's damage times (one plus {falloff}) to the power of the bounce number as physical damage, and only {building_pct} of that to buildings." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              { op = "call", routine = "a15r_old_start", args = { victim = "event.target", base = "event.attack.damage" } },
            },
          },
        },
        statuses = {
          { id = "a15r_old_hit", duration = "$hit_mark", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 target.hit[instance]：0.15 秒（建築 0.1 秒）後清除；v3 不分實例（同本體）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/A_Oda/A15.txt:698-770",
            text = "FC-174 D22 舊版，比照 D13 修：原作 OnAttackLanded 呼叫的 A15R_OnAttackLanded／A15R_bounce_init（A15.lua:114-136）以 FindAbilityByName(\"A15R\") 取技能，舊版沒有 A15R 而對 nil 呼叫方法報錯，整個被動沒有效果；A15R_bounceAttack（:162）讀 A15T 等級也會因舊版沒有 A15T 報錯。這是把本體被動改名時漏改的 bug。修法：照本體 A15R 的彈跳做，數值換成 A15R_old 的 KV（bonus_damage −0.30、jump_max 2/4/6/8、jump_radius 700）。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A15.lua:148-151,162-168,220-223",
            text = "FC-175 D22 舊版，隨上一條一起修：本體與 A15T 連動的部分（彈跳半徑 + (A15T 等級 − 1) × 50、modifier_A15T2 的 jump_max 翻倍、modifier_A15T 的彈跳吸血）在舊版沒有對應技能（舊版大招是 A15T_old 夢魘，沒有這些 modifier），推定都不生效：半徑固定 700、不翻倍、不吸血。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A15.lua:217-219",
            text = "FC-050 D22 舊版，比照本體 FC-050 修：彈跳到侍大將（EXCLUDE_TARGET_NAME2）時原作寫錯變數名稱（damage_table）報錯中斷；照原意只受一半傷害（general_pct）、照樣繼續彈跳。侍大將以單位 tag general 辨識。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A15.lua:205-207",
            text = "同本體：「傷害低於兩成就以兩成計」寫成改 base_damage 而不是 new_damage，沒有效果；彈跳傷害沒有下限。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:185-199,236-248",
            text = "同本體：命中標記只保留 0.15 秒（建築 0.1 秒），只會避開剛打到的單位，可以在兩個單位之間來回彈；所有附近單位都帶標記時，改選範圍內最遠、不是目前目標的單位。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:704,726-733",
            text = "基礎傷害是 OnAttackLanded 的 %attack_damage；第一個目標只吃普攻本身，彈跳以 ApplyDamage 造成物理傷害（不算普攻）。選取帶 FOW_VISIBLE 與 MAGIC_IMMUNE_ENEMIES：只跳向看得到的單位，建築也會被彈到。" },
        },
        provenance = { "npc/A_Oda/A15.txt:697-770", "vscripts/heroes/A_Oda/A15.lua:114-279" },
      },
      -------------------------------------------------------------------------
      { id = "a15_old_nightmare", source_id = "A15T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          heal = { 500, 700, 900, unit = "heal", source = "kv:A15T_old_heal" },
          duration = { 40, unit = "s", source = "kv:A15T_old_duration" },
          damage = { 150, 210, 270, unit = "dmg", damage_type = "magical", source = "kv:A15T_old_damage" },
          radius = { 1000, unit = "dist", source = "kv:A15T_old_radius" },
          move_speed_pct = { 20, unit = "pct", source = "kv:modifier_A15T_old MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE" },
          tick = { 0.5, unit = "s", source = "kv:modifier_A15T_old ThinkInterval" },
          missile_speed = { 400, unit = "speed", hidden = true, source = "script:vscripts/heroes/A_Oda/A15.lua:394 iMoveSpeed = 400" },
        },
        text = {
          zh_TW = { name = "忍法密傳．夢魘",
            body = "立即回復 {heal} 生命，並進入夢魘狀態 {duration} 秒：移動速度提高 {move_speed_pct}，每 {tick} 秒朝範圍 {radius} 內看得見的一名敵方英雄或一般單位射出可被閃避的夢魘彈，命中造成 {damage} 魔法傷害。" },
          en = { name = "Secret Ninja Art: Nightmare",
            body = "Instantly restores {heal} health and enters a nightmare for {duration} seconds: +{move_speed_pct} movement speed, and every {tick} seconds fires a dodgeable nightmare bolt at one visible enemy hero or unit within {radius}, dealing {damage} magic damage on hit." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a15t_old_nightmare", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "夢魘", body = "移動速度提高 {move_speed_pct}；每 {tick} 秒朝範圍 {radius} 內一名敵人射出夢魘彈，造成 {damage} 魔法傷害。" },
              en = { name = "Nightmare", body = "+{move_speed_pct} movement speed; every {tick} seconds fires a nightmare bolt at an enemy within {radius} for {damage} magic damage." },
            },
            reveal = "public", cue = "a15t_old_aura", stats = { move_speed_pct = "$move_speed_pct" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = UNITS,
                    flags = { "visible_only" }, limit = "1" },
                  actions = {
                    { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$missile_speed", dodgeable = true,
                      cue = "a15t_old_bolt",
                      on_hit = {
                        { op = "if", cond = "alive(self)", actions = {
                          { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage" },
                        } },
                      } },
                  } },
              } },
            },
            notes = "原作 modifier_A15T_old（IsPurgable 0、ThinkInterval 0.5）。" },
        },
        events = {
          on_cast = {
            { op = "heal", to = "caster", source = "caster", amount = "$heal" },
            { op = "cue", id = "a15t_old_ghost_sound", at = "caster" },
            { op = "apply_status", status = "a15t_old_nightmare", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:850-859",
            text = "AbilitySpecial 有兩個 \"03\"（A15T_old_damage 與 A15T_old_radius）；推定兩者都會載入、各自讀得到（同一層子鍵逐一讀取），若引擎只留一個，另一個會讀到 0。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:384-386",
            text = "目標是 FIND_ANY_ORDER 的第一個（FOW_VISIBLE），不是隨機也不是最近；資料以 limit = 1 依選取預設順序。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A15.lua:406-412",
            text = "命中時 caster 必須有效（IsValidEntity）；傷害走 AMHC，攻擊者已死不傷害屬規則層。投射物 iUnitTargetType 誤填成傷害類型，對追蹤投射物沒有影響。" },
          { policy = "as_source", ref = "npc/A_Oda/A15.txt:816-835",
            text = "datadriven modifier 在死亡時移除；dispel none 依 IsPurgable 0。" },
          { policy = "note", ref = "npc/A_Oda/A15.txt:795-800", text = "item_ghost_sceptre 音效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A15.txt:772-861", "vscripts/heroes/A_Oda/A15.lua:368-413" },
      },
    },
    routines = {
      -- 影風車（D22 修正後照本體 A15R）：起點。第一個目標只標記、不吃彈跳傷害。
      a15r_old_start = {
        params = { victim = "unit", base = "num" },
        actions = {
          { op = "apply_status", status = "a15r_old_hit", to = "victim",
            duration = "select(victim.is_building, $a15_old_shadow_windmill.hit_mark_building, $a15_old_shadow_windmill.hit_mark)" },
          { op = "if", cond = "$a15_old_shadow_windmill.jump_max - 1 > 0", actions = {
            { op = "call", routine = "a15r_old_jump",
              args = { from = "victim", base = "base", scale = "1 + $a15_old_shadow_windmill.falloff", left = "$a15_old_shadow_windmill.jump_max - 1" } },
          } },
        },
      },
      -- 選下一跳：最近、沒有命中標記；全部都有標記時取最遠的其他單位。
      a15r_old_jump = {
        params = { from = "unit", base = "num", scale = "num", left = "num" },
        actions = {
          { op = "set_state", var = "a15_old_bounce_found", value = "false" },
          { op = "for_each",
            select = { from = "from", shape = "circle", radius = "$a15_old_shadow_windmill.jump_radius", team = "enemy", types = UNITS_BUILDING,
              flags = { "include_magic_immune", "visible_only" }, where = "not has_status(it, a15r_old_hit)",
              order = "nearest", limit = "1" },
            actions = {
              { op = "set_state", var = "a15_old_bounce_found", value = "true" },
              { op = "call", routine = "a15r_old_fire", args = { from = "from", to = "it", base = "base", scale = "scale", left = "left" } },
            } },
          { op = "if", cond = "not state.a15_old_bounce_found", actions = {
            { op = "for_each",
              select = { from = "from", shape = "circle", radius = "$a15_old_shadow_windmill.jump_radius", team = "enemy", types = UNITS_BUILDING,
                flags = { "include_magic_immune", "visible_only" }, exclude = "from", order = "farthest", limit = "1" },
              actions = {
                { op = "call", routine = "a15r_old_fire", args = { from = "from", to = "it", base = "base", scale = "scale", left = "left" } },
              } },
          } },
        },
      },
      a15r_old_fire = {
        params = { from = "unit", to = "unit", base = "num", scale = "num", left = "num" },
        actions = {
          { op = "reveal", at = "position(to)", radius = "$a15_old_shadow_windmill.reveal_radius",
            duration = "$a15_old_shadow_windmill.reveal_duration", teams = "own" },
          { op = "projectile", kind = "tracking", from = "from", to = "to", speed = "$a15_old_shadow_windmill.bounce_speed",
            vision = "$a15_old_shadow_windmill.bounce_vision", dodgeable = true, cue = "a15r_old_glaive", on_hit = {
              { op = "call", routine = "a15r_old_land", args = { victim = "it", base = "base", scale = "scale", left = "left" } },
            } },
        },
      },
      a15r_old_land = {
        params = { victim = "unit", base = "num", scale = "num", left = "num" },
        actions = {
          { op = "apply_status", status = "a15r_old_hit", to = "victim",
            duration = "select(victim.is_building, $a15_old_shadow_windmill.hit_mark_building, $a15_old_shadow_windmill.hit_mark)" },
          -- FC-050（本體 D13 修法）：侍大將只受一半傷害、照樣彈跳
          { op = "let", name = "g", value = "select(has_tag(victim, general), $a15_old_shadow_windmill.general_pct, 1)" },
          { op = "damage", to = "victim", type = "physical",
            amount = "select(victim.is_building, base * scale * $a15_old_shadow_windmill.building_pct, base * scale * g)" },
          { op = "if", cond = "left - 1 > 0", actions = {
            { op = "call", routine = "a15r_old_jump",
              args = { from = "victim", base = "base", scale = "scale * (1 + $a15_old_shadow_windmill.falloff)", left = "left - 1" } },
          } },
        },
      },
    },
  }
end
