-- C10 香宗我部親泰（current 版，heromap_skill C10 ["16"] = "WERDT"）— package schema v3.1。
-- 原作槽位 W/E/R/D/T → Q/W/E/D/R（npc/C_Neutral/C10_2.txt:31-36）。current 版用 C10W/E/R/D/T，*_old 是 11 版。
-- heromap_autoskill C10 ["16"] = ""：C10D 是 NOT_LEARNABLE，由 C10T_OnUpgrade 每次 +1 級（level_link）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C10_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）與 datadriven MOVESPEED_BONUS_PERCENTAGE 負值都照 A14／A22 寫成 slow_pct 正值，狀態帶 tag slow。
local BASIC = { "hero", "creep", "neutral", "summon" }

return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C10",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C10",
      text = { zh_TW = { name = "香宗我部親泰" }, en = { name = "Kosokabe Chikayasu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c10_black_dragon_wave", source_id = "C10W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 110, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          distance = { 800, unit = "dist", source = "kv:distance" },
          start_radius = { 165, unit = "dist", hidden = true, source = "kv:start_radius" },
          end_radius = { 245, unit = "dist", hidden = true, source = "kv:end_radius" },
          speed = { 1320, unit = "speed", hidden = true, kv = { 1380 },
            source = "script:vscripts/heroes/C_Neutral/C10.lua:144-155 movespeed 被第二次賦值成 start_radius，速度 = 165 × 8",
            source_differs = "KV speed 1380 被同一函式第 148 行覆寫成 start_radius（165），再乘 8 得 1320" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          slow = { 15, 20, 25, 30, unit = "pct", kv = { -15, -20, -25, -30 }, source = "kv:slow",
            source_differs = "KV 以負值表示減速（MOVESPEED_BONUS_PERCENTAGE）；slow_pct 以正值表示" },
          slow_duration = { 6, unit = "s", source = "kv:reduction_duration" },
        },
        text = {
          zh_TW = { name = "黑龍波",
            body = "朝面向射出一道長 {distance} 的黑龍波，貫穿路徑上的敵方英雄與部隊（包含魔免），各造成 {damage} 魔法傷害並緩速 {slow}，持續 {slow_duration} 秒。" },
          en = { name = "Black Dragon Wave",
            body = "Fires a black dragon wave {distance} long in the facing direction that passes through enemy heroes and units (spell-immune ones included), dealing {damage} magic damage to each and slowing them by {slow} for {slow_duration} seconds." },
        },
        target = { type = "unit_or_point", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c10w_slow", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "黑龍波緩速", body = "移動速度降低 {slow}。" },
              en = { name = "Black Dragon Wave Slow", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "c10w_wave_hit",
            notes = "原作 modifier_C10W：IsPurgable 1、IsDebuff 1，datadriven MOVESPEED_BONUS_PERCENTAGE（dummy.lua Slow 那段已註解，不走 nobu 緩速池）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c10w_cast_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "caster.forward",
              speed = "$speed", distance = "$distance", radius = "$start_radius", end_radius = "$end_radius",
              pierce = true, team = "enemy", types = BASIC, flags = { "include_magic_immune" },
              vision = "$end_radius", cue = "c10w_wave",
              on_hit = {
                { op = "apply_status", status = "c10w_slow", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:144-155",
            text = "腳本把 movespeed 重新賦值成 start_radius，投射物速度實際是 165 × 8 = 1320，不是 KV speed 1380。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:131-139,155",
            text = "方向用前搖開始時的面向（ability.forwardVec），不用游標位置；施法者轉身面向目標後才開始前搖，所以結果與朝目標方向相同（v3 用 caster.forward）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:101-115",
            text = "OnProjectileHitUnit 先套緩速再造成傷害；datadriven Damage 不檢查施法者存活，施法者死亡後投射物照樣造成傷害。" },
          { policy = "as_source", text = "原作說明寫緩速 15/25/35/45%、持續 3.5 秒；KV 實際是 15/20/25/30%、6 秒。說明改由數值產生。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C10.lua:160-161",
            text = "bReplaceExisting = true 會取代同一來源上一道線性投射物；冷卻 9 秒遠長於飛行時間，實際沒有影響。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:39-193", "vscripts/heroes/C_Neutral/C10.lua:131-169" },
      },
      -------------------------------------------------------------------------
      { id = "c10_energy_drain", source_id = "C10E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, 7, 6, 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          drain = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:Heal" },
          stun = { 0.8, unit = "s", source = "kv:stun_duration" },
        },
        text = {
          zh_TW = { name = "能量奪取",
            body = "對目標敵人造成 {drain} 魔法傷害並暈眩 {stun} 秒，自身回復 {drain} 生命。" },
          en = { name = "Energy Drain",
            body = "Deals {drain} magic damage to the target enemy and stuns it for {stun} seconds; Chikayasu heals for {drain}." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c10e_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "能量奪取暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Energy Drain Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 datadriven Stun（引擎 modifier_stunned）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c10e_manavoid_cast", at = "target" } },
          on_cast = {
            { op = "damage", to = "target", type = "magical", amount = "$drain", require_source_alive = false },
            { op = "heal", to = "caster", source = "caster", amount = "$drain" },
            { op = "apply_status", status = "c10e_stun", to = "target" },
            { op = "cue", id = "c10e_manavoid", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:215-254",
            text = "回復量是固定的 Heal 值，不是實際造成的傷害；目標魔法抗性高或傷害被擋下，施法者仍回復全額。順序是傷害、回復、暈眩。" },
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:203",
            text = "SPELL_IMMUNITY_ENEMIES_NO 且沒有 MAGIC_IMMUNE 旗標：不能以魔免敵人為目標。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:194-276" },
      },
      -------------------------------------------------------------------------
      { id = "c10_echo_sword", source_id = "C10R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          attack_bonus = { 25, 35, 45, 55, unit = "dmg", source = "kv:attack_bonus" },
          cleave_pct = { 15, 30, 45, 60, unit = "pct", source = "kv:CleavePercent" },
          cleave_radius = { 485, unit = "dist", source = "kv:CleaveRadius" },
        },
        text = {
          zh_TW = { name = "迴響劍",
            body = "被動：基礎攻擊力提高 {attack_bonus}。普攻命中非建築的敵人時，對目標周圍 {cleave_radius} 內的其他敵方英雄與部隊造成該次攻擊傷害 {cleave_pct} 的物理濺射傷害。" },
          en = { name = "Echo Blade",
            body = "Passive: +{attack_bonus} base attack damage. When an attack lands on a non-building enemy, other enemy heroes and units within {cleave_radius} of the target take {cleave_pct} of the attack's damage as physical cleave damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { base_attack_damage = "$attack_bonus" },
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building",
              actions = {
                { op = "for_each",
                  select = { from = "event.target", shape = "circle", radius = "$cleave_radius", team = "enemy",
                    types = BASIC, flags = { "include_magic_immune" }, exclude = "event.target" },
                  actions = {
                    { op = "damage", to = "it", type = "physical", amount = "event.attack.damage * $cleave_pct",
                      flags = { "ignore_armor" } },
                    { op = "cue", id = "c10r_cleave", at = "it" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C10.txt:293-310",
            text = "濺射是引擎 datadriven CleaveAttack；推定依 Dota 引擎規則：以本次攻擊傷害（護甲前）乘比例、不受被濺射者護甲減免、不濺射建築、攻擊建築時不觸發、魔免單位照樣受傷。形狀推定為以目標為圓心的圓。M2 規則層實測後修正。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C10.lua:172-192",
            text = "腳本 C10R（半徑 385、手算護甲回推的濺射）沒有任何 KV 呼叫，是死碼。" },
          { policy = "note", ref = "npc/C_Neutral/C10.txt:309",
            text = "modifier_C10R 寫 IsHidden 0，會顯示在狀態列（passive.visibility = buff）。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:277-330", "vscripts/heroes/C_Neutral/C10.lua:172-192" },
      },
      -------------------------------------------------------------------------
      { id = "c10_day_night_shift", source_id = "C10D", slot = "D", kind = "active", max_level = 3, tags = { "basic" },
        learnable = false, level_link = "c10_formless_heaven",
        values = {
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          switch_time = { 10, unit = "s", source = "script:vscripts/heroes/C_Neutral/C10.lua:6,11 Timers:CreateTimer(10)" },
        },
        text = {
          zh_TW = { name = "晝夜轉換",
            body = "把全圖目前的白天或黑夜轉換成另一種，持續 {switch_time} 秒。隨{@c10_formless_heaven}一起升級。" },
          en = { name = "Day-Night Shift",
            body = "Switches the whole map from day to night or from night to day for {switch_time} seconds. Levels up together with {@c10_formless_heaven}." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            -- 原作改寫全域時鐘到固定點（不是暫時切換再回到原本的時鐘），所以用不帶 duration 的 set_time_of_day 加延遲
            { op = "if", cond = "not game.is_night",
              actions = {
                { op = "set_time_of_day", mode = "night" },
                { op = "delay", time = "$switch_time", actions = { { op = "set_time_of_day", mode = "day" } } },
              },
              otherwise = {
                { op = "set_time_of_day", mode = "day" },
                { op = "delay", time = "$switch_time", actions = { { op = "set_time_of_day", mode = "night" } } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:1-15",
            text = "白天時 SetTimeOfDay(1)（午夜），10 秒後 SetTimeOfDay(0.3)（清晨）；夜晚時 SetTimeOfDay(0.3)，10 秒後 SetTimeOfDay(1)。每次施放都把全域日夜循環的時鐘改到固定點，之後的白天或夜晚長度跟著改變，不會回到原本的時鐘；10 秒後的改寫是全域 Timer，施法者死亡也照樣執行。v3.2 的 set_time_of_day 只指定日夜，時鐘落點由規則層決定（原作夜晚落在午夜、白天落在 0.3，M2 照這兩個值實作）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:97-106",
            text = "C10D 是 NOT_LEARNABLE，只有 C10T 每次升級時 SetLevel(+1)，所以等級等於無相天圓的等級；學會無相天圓前不能使用。" },
          { policy = "as_source", text = "原作說明的「增加夜間視野 150/300/450」實際由 C10T_OnUpgrade 加上，寫在無相天圓的被動。" },
          { policy = "note", ref = "npc/C_Neutral/C10.txt:24",
            text = "KV AbilityManaCost 0；MaxLevel 3，冷卻不隨等級變化。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:19-38", "vscripts/heroes/C_Neutral/C10.lua:1-15", "vscripts/heroes/C_Neutral/C10.lua:97-106" },
      },
      -------------------------------------------------------------------------
      { id = "c10_formless_heaven", source_id = "C10T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 100, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 180, 210, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          radius = { 600, unit = "dist", source = "kv:radius" },
          delay = { 3, unit = "s", source = "kv:delay" },
          slow_duration = { 6, 7.5, 9, unit = "s", source = "kv:slow_duration" },
          slow = { 45, unit = "pct", kv = { -45 }, source = "kv:slow_speed",
            source_differs = "KV 以負值表示減速（dummy.lua Slow 的 ms_slow）；slow_pct 以正值表示" },
          attack_speed = { 100, unit = "atk_speed", source = "kv:modifier_C10T_self MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          regather_slow = { 3, unit = "s", source = "script:vscripts/heroes/C_Neutral/C10.lua:85-87 {duration=3}" },
          max_pull_range = { 5500, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C10.lua:78 Length2D() < 5500" },
          day_armor = { 10, 20, 30, unit = "armor", source = "kv:armor" },
          night_speed = { 20, 25, 30, unit = "pct", source = "kv:speed" },
          night_vision = { 150, 300, 450, unit = "dist",
            source = "script:vscripts/heroes/C_Neutral/C10.lua:100,107 每次升級 SetNightTimeVisionRange(+150)" },
        },
        text = {
          zh_TW = { name = "無相天圓",
            body = "被動：白天護甲 +{day_armor}，夜晚移動速度 +{night_speed}；夜間視野 +{night_vision}。啟動：使自身周圍 {radius} 內的敵方英雄與部隊（包含魔免）緩速 {slow}，持續 {slow_duration} 秒，自身攻擊速度 +{attack_speed}，持續相同時間。{delay} 秒後，若親泰仍存活，把這些敵人全部拉到自己身上；當時已沒有緩速的敵人再緩速 {regather_slow} 秒。" },
          en = { name = "Animitta",
            body = "Passive: +{day_armor} armor during the day and +{night_speed} movement speed at night; +{night_vision} night vision. Active: slows enemy heroes and units within {radius} (spell-immune ones included) by {slow} for {slow_duration} seconds and grants Chikayasu +{attack_speed} attack speed for the same time. After {delay} seconds, if Chikayasu is still alive, all of those enemies are pulled onto him; any of them no longer slowed is slowed again for {regather_slow} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        passive = {
          reveal = "public",
          stats = {
            armor = { op = "add", value = "$day_armor", when = "not game.is_night" },
            move_speed_pct = { op = "add", value = "$night_speed", when = "game.is_night" },
            vision_night = "$night_vision",
          },
        },
        statuses = {
          { id = "c10t_slow", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "無相天圓緩速", body = "移動速度降低 {slow}。" },
              en = { name = "Animitta Slow", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "c10t_reflection_slow",
            notes = "原作 modifier_C10T：OnCreated／OnDestroy 呼叫 dummy.lua Slow／ReturnSpeed（name C10T）；沒寫 IsPurgable，暫定 basic。" },
          { id = "c10t_mark", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 caster.C10T_T：施放當下記下的單位清單（Lua 表，不能驅散）；v3 以隱藏標記表達，收回時移除。" },
          { id = "c10t_haste", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "無相天圓", body = "攻擊速度提高 {attack_speed}。" },
              en = { name = "Animitta", body = "Attack speed increased by {attack_speed}." },
            },
            reveal = "public", stats = { attack_speed = "$attack_speed" },
            notes = "原作 modifier_C10T_self：Duration %slow_duration，ATTACKSPEED_BONUS_CONSTANT 100；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "spell_hint", at = "caster", radius = "$radius" } },
          on_cast = {
            { op = "cue", id = "c10t_cast", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "cue", id = "c10t_shake", at = "it" },
                { op = "apply_status", status = "c10t_mark", to = "it" },
                { op = "apply_status", status = "c10t_slow", to = "it" },
              } },
            { op = "apply_status", status = "c10t_haste", to = "caster" },
            { op = "delay", time = "$delay", actions = {
              { op = "cue", id = "c10t_end", at = "caster" },
              { op = "for_each",
                select = { from = "caster", shape = "global", team = "enemy", types = { "all" },
                  flags = { "include_magic_immune", "include_invulnerable", "include_untargetable", "include_hidden" },
                  where = "has_status(it, c10t_mark)" },
                actions = {
                  { op = "remove_status", from = "it", status = "c10t_mark" },
                  { op = "if", cond = "alive(caster) and distance(it, caster) < $max_pull_range", actions = {
                    { op = "teleport", who = "it", to = "position(caster)", find_clear_space = false },
                    { op = "if", cond = "not has_status(it, c10t_slow)", actions = {
                      { op = "apply_status", status = "c10t_slow", to = "it", duration = "$regather_slow" },
                    } },
                  } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C10.lua:46,83",
            text = "收回時 `not string.match(v:GetUnitName(), \"MINE\")` 跳過地雷，但名稱含 MINE 的單位只有 B13_MINE_hero、A26_MINE_hero、A26_MINE_school_hero，三者都在 _G.EXCLUDE_TARGET_NAME（vscripts/library/common/dummy.lua:354-356），施放時第 46 行就不會被記進清單；這個判斷是死碼。v3 施放時的選取預設排除 untargetable_by_spells，收回的 _G.EXCLUDE_TARGET_NAME 判斷同理不必另寫。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C10.txt:464-477",
            text = "日夜被動是 modifier_C10T_x 每 1 秒檢查 IsDaytime 再換 modifier_C10T_1（護甲）或 _2（移速）：日夜交替後最多 1 秒才換；死亡會移除 _1／_2，復活後最多 1 秒補回。照設計意圖寫成依 game.is_night 即時生效的被動屬性。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:97-116",
            text = "每次升級 SetNightTimeVisionRange(目前 + 150)，三級共 +450，跨死亡保留；同時把 C10D 升一級並立刻重套日夜加成。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:42-53",
            text = "施放時選自身周圍 600 的敵方 HERO|BASIC（含魔免，排除 _G.EXCLUDE_TARGET_NAME），之後離開範圍、變魔免仍會被拉回；隱形與迷霧中的敵人也選得到。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:55-67",
            text = "延遲期間每 0.1 秒檢查施法者是否死亡，死亡就清空清單（之後即使復活也不拉）；v3 以收回當下 alive(caster) 判斷，差異只在「延遲中死亡又在三秒內復活」。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C10.lua:74-91",
            text = "以 SetAbsOrigin 把所有目標疊在施法者的同一點（不找空位），雙方都只給 0.1 秒 modifier_phased 讓碰撞自然推開；只檢查 IsValidEntity，已死亡的單位也在清單內（移動屍體沒有效果，v3 選取預設排除死者）。" },
          { policy = "as_source", text = "原作說明寫被動護甲 10/15/20；KV 實際 10/20/30。說明改由數值產生。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C10.lua:18-30,93-95",
            text = "前搖的 spell hint（hide_unit dummy）、C10T_SE 的施法動作、粒子與音效屬呈現，改用 cue。KV AbilityCastRange 600 只是指示，等於 radius。" },
        },
        provenance = { "npc/C_Neutral/C10.txt:331-538", "vscripts/heroes/C_Neutral/C10.lua:18-129",
          "vscripts/library/common/dummy.lua:816-864" },
      },
    },
  }
end
