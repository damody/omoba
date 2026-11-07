-- B28 松永久秀（current 版，heromap_skill B28 ["16"] = "WERT"，沒有 D）— package schema v3。
-- current 版＝英雄實際出生時的技能：npc/B_Unified/B28_2.txt:12-17 的 B28W/B28E/B28R/B28T。原作槽位 W/E/R/T → Q/W/E/R。
-- _old（11 版）不做；B28T 腳本借用 B28T_old_jump_init／B28T_old_Jump 兩個函式，那兩個函式屬 current 版。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B28_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- modifier_B28E、modifier_B28E2 列在 _G.EXCLUDE_MODIFIER_NAME（vscripts/library/common/dummy.lua:376-392），狀態帶 tag cleanse_protected。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "B28",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B28",
      text = { zh_TW = { name = "松永久秀" }, en = { name = "Matsunaga Hisahide" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b28_phoenix_flame", source_id = "B28W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 130, 150, 170, 190, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 175, unit = "dmg", damage_type = "magical", source = "kv:B28W_damage" },
          dot_damage = { 50, 85, 120, 155, unit = "dmg", damage_type = "magical", source = "kv:B28W_dotDamage" },
          dot_duration = { 5, unit = "s", source = "kv:B28W_dotDuration" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_B28W_dot ThinkInterval" },
          speed = { 1100, unit = "speed", hidden = true, source = "kv:shock_speed" },
          width = { 300, unit = "dist", source = "kv:shock_width", note = "StartRadius／EndRadius" },
          distance = { 1500, unit = "dist", source = "kv:shock_distance", note = "FixedDistance" },
          vision = { 200, unit = "dist", hidden = true, source = "kv:shock_vision" },
        },
        text = {
          zh_TW = { name = "飛凰焰",
            body = "朝目標方向放出火鳳，飛行 {distance} 距離、半徑 {width}，貫穿路徑上所有非魔免的敵方英雄與部隊：造成 {damage} 魔法傷害並點燃 {dot_duration} 秒，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Phoenix Flame",
            body = "Releases a fire phoenix in the target direction that travels {distance} with a {width} radius, piercing every non-spell-immune enemy hero and unit in its path: deals {damage} magic damage and ignites them for {dot_duration} seconds, dealing {dot_damage} magic damage every {dot_interval} seconds." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b28w_burn", duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "點燃", body = "每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Ignited", body = "Takes {dot_damage} magic damage every {dot_interval} seconds." },
            },
            reveal = "public",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B28W_dot：IsDebuff、Duration 5、ThinkInterval 1.0、datadriven Damage；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b28w_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", radius = "$width", distance = "$distance", vision = "$vision", pierce = true,
              team = "enemy", types = BASIC, cue = "b28w_phoenix",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "apply_status", status = "b28w_burn", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/B_Unified/B28.txt:92-107",
            text = "ThinkInterval 1.0 配持續 5 秒，第五跳是否落在到期當下依引擎時序；照設計意圖每秒一跳。再次命中只刷新持續時間。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:41-62",
            text = "SPELL_IMMUNITY_ENEMIES_NO、TargetFlags NONE：魔免敵人不會被命中。飛行距離固定 1500，比施法距離 1000 遠；DeleteOnHit 0，貫穿。" },
        },
        provenance = { "npc/B_Unified/B28.txt:31-148" },
      },
      -------------------------------------------------------------------------
      { id = "b28_flame_shackle", source_id = "B28E", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 23, 21, 19, 17, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 145, 170, 195, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 2, 3, 4, 5, unit = "s", source = "kv:B28E_duration", note = "AbilityChannelTime" },
          damage = { 100, unit = "dmg", damage_type = "magical", source = "kv:B28E_damage" },
          tick = { 1.01, unit = "s", source = "kv:modifier_B28E ThinkInterval" },
          share = { 30, unit = "pct",
            source = "script:vscripts/heroes/B_Unified/B28.lua:90 damage*0.3",
            note = "KV B28E_share_percentage 30 被讀進 caster.B28E_share_percentage 但沒有使用，腳本寫死 0.3；數字相同" },
          near_dist = { 200, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:17-21 dis < 200 → 200、r = dis − 200" },
          falloff = { 600, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:22 (600 − r) / 600 * duration" },
          half = { 0.5, unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:23-24 duration/2" },
          check_interval = { 0.2, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:40-58 Timer 0.2" },
        },
        text = {
          zh_TW = { name = "焰光桎梏",
            body = "以火焰鎖鏈連住目標敵人並持續施法：立即造成 {damage} 魔法傷害並使其暈眩，暈眩期間每 {tick} 秒再受 {damage} 魔法傷害。暈眩時間依兩者距離決定，貼近時等於整個施法時間，越遠越短，最短為一半；暈眩結束時施法也跟著結束。暈眩期間，久秀受到的傷害有 {share} 以純粹傷害轉給目標。施法中久秀若施放其他技能，鎖鏈立即解除。" },
          en = { name = "Flame Shackle",
            body = "Chains an enemy with fire and channels: deals {damage} magic damage at once and stuns the target, dealing {damage} magic damage again every {tick} seconds while stunned. The stun lasts the whole channel at close range, shorter the farther away, down to half; the channel ends when the stun does. While the target is stunned, {share} of the damage Hisahide takes is dealt to it as pure damage. If Hisahide casts another ability during the channel, the chain breaks immediately." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b28e_stun", duration = "$channel_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "焰光桎梏", body = "被火焰鎖鏈暈眩，每 {tick} 秒受到 {damage} 魔法傷害；久秀受到的傷害有 {share} 以純粹傷害轉給這個單位。久秀的施法結束時解除。" },
              en = { name = "Flame Shackle", body = "Stunned by a fire chain, taking {damage} magic damage every {tick} seconds; {share} of the damage Hisahide takes is dealt to this unit as pure damage. Ends when Hisahide's channel ends." },
            },
            reveal = "public", states = { "stun" }, tags = { "cleanse_protected" }, cue = "b28e_chain",
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
              } },
              periodic_2 = { interval = "$check_interval", actions = {
                { op = "if", cond = "not has_status(caster, b28e_guard)", actions = {
                  { op = "remove_status", from = "self", status = "b28e_stun" },
                } },
              } },
            },
            notes = "原作 modifier_B28E：IsPurgable 1、IsStunDebuff、STUNNED、ThinkInterval 1.01 的 datadriven Damage。腳本先以距離算出的時間套用，接著 datadriven 再套一次（沒有 Duration，推定變成不限時），實際由施法結束時移除。" },
          { id = "b28e_bond", duration = "$channel_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            vars = { target = { type = "unit", reveal = "public" } },
            triggers = {
              on_take_damage = {
                filter = "self.is_hero and exists(vars.target) and has_status(vars.target, b28e_stun)",
                actions = {
                  { op = "damage", to = "vars.target", source = "self", type = "pure", amount = "event.damage.final * $share" },
                } },
            },
            notes = "原作 modifier_B28E_OnTakeDamage（Lua，IsHidden；沒覆寫 IsPurgable，暫定 none）：持續整個引導時間；對象存在 caster.B28E_target，0.2 秒檢查迴圈在目標失去暈眩時清空。" },
          { id = "b28e_guard", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", tags = { "cleanse_protected" },
            triggers = {
              on_ability_executed = { who = "self",
                filter = "not event.ability.is_toggle and not is_item(event.ability, item_logging) and not is_item(event.ability, item_tpscroll)",
                actions = {
                  { op = "remove_status", from = "self", status = "b28e_guard" },
                } },
            },
            notes = "原作 modifier_B28E2：IsHidden、沒有 Duration（不限時）；OnAbilityExecuted 施放非開關技能（item_logging、item_tpscroll 除外）就移除自己；OnTakeDamage 呼叫的 B28E_OnTakeDamage 是空函式。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "dist", value = "max(edge_distance(caster, target), $near_dist)" },
            { op = "let", name = "stun_time",
              value = "$channel_time * max($half, ($falloff + $near_dist - dist) / $falloff)" },
            { op = "apply_status", status = "b28e_bond", to = "caster", vars = { target = "target" } },
            { op = "apply_status", status = "b28e_stun", to = "target", duration = "stun_time" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
            { op = "cue", id = "b28e_fatal_bonds_sound", at = "target" },
            { op = "apply_status", status = "b28e_guard", to = "caster" },
            { op = "delay", time = "stun_time", actions = {
              { op = "end_channel", interrupted = true },
            } },
          },
          on_channel_end = {
            { op = "remove_status", from = "target", status = "b28e_stun" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:13-34",
            text = "暈眩時間 = 引導時間 × max(0.5, (600 − (max(OBB 距離, 200) − 200)) / 600)：200 以內是完整時間，800 時剩一半。時間到時腳本以 EndChannel(true) 中斷引導（觸發 OnChannelInterrupted），所以引導實際只持續暈眩時間。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:169-198",
            text = "OnSpellStart 依序：腳本（套分傷、以距離時間套暈眩）→ datadriven 再套一次 modifier_B28E（沒有 Duration）→ 100 魔法傷害 → 套 modifier_B28E2。第二次套用推定讓暈眩變成不限時，實際結束由引導結束（OnChannelFinish／OnChannelInterrupted 的 RemoveModifier）決定，結果與以距離時間套用相同。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:12,83-93",
            text = "分傷 modifier 持續整個引導時間（用的是重算前的 duration），但只在 caster.B28E_target 有值時有效；0.2 秒迴圈在目標失去 modifier_B28E 時清空對象，之後 AMHC:Damage 對 nil 目標沒有效果。資料以「目標仍被暈眩」近似，差距最多 0.2 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:83-93",
            text = "MODIFIER_EVENT_ON_TAKEDAMAGE 的 event.damage 是減免後傷害；任何來源（包括自己、中立、建築）造成的傷害都會分 30% 給目標，類型固定純粹，以 AMHC:Damage 造成（久秀已死時不傷害，規則層）。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:243-264",
            text = "暈眩的 ThinkInterval 是 1.01 秒，不是 1 秒：五秒的暈眩只跳四次（不含施放時那一下），兩秒只跳一次。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:40-58,61-67",
            text = "modifier_B28E2 套上後不會自己消失，只有施放其他非開關技能才移除；0.2 秒迴圈發現它不在時解除暈眩，但不中斷引導。施法中施放其他技能通常本來就會打斷引導，這個檢查只對不打斷引導的施放有差別。" },
          { policy = "note", ref = "vscripts/library/common/dummy.lua:384-385",
            text = "modifier_B28E、modifier_B28E2 列在 EXCLUDE_MODIFIER_NAME，不會被 nobu 的淨化類效果移除（tag cleanse_protected）；但 modifier_B28E 仍是 IsPurgable 1。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B28.lua:69-73",
            text = "modifier_B28E2 的 OnTakeDamage 呼叫 B28E_OnTakeDamage，函式只宣告區域變數，沒有效果。" },
          { policy = "note", text = "鎖鏈粒子每 0.2 秒重建一次（舊的沒有銷毀），只是呈現，改用狀態 cue。" },
        },
        provenance = { "npc/B_Unified/B28.txt:150-285", "vscripts/heroes/B_Unified/B28.lua:5-97",
          "vscripts/library/common/dummy.lua:376-392" },
      },
      -------------------------------------------------------------------------
      { id = "b28_fire_stream", source_id = "B28R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 21, 19, 17, 15, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 10, unit = "s", source = "kv:B28R_duration" },
          move_speed_pct = { 0, 8, 16, 24, unit = "pct", source = "kv:B28R_msBonus" },
          radius = { 600, unit = "dist", source = "kv:B28R_radius" },
          interval = { 0.3, unit = "s", source = "kv:modifier_B28R ThinkInterval" },
          damage = { 20, 40, 60, 80, unit = "dmg", damage_type = "magical", source = "kv:B28R_damage" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/B_Unified/B28.lua:104-105 dmg*0.2" },
          dot_damage = { 80, unit = "dmg", damage_type = "magical", source = "kv:B28R_dotDamage" },
          dot_duration = { 3, unit = "s", source = "kv:modifier_B28R_dot Duration" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_B28R_dot ThinkInterval" },
          projectile_speed = { 1300, unit = "speed", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:127 iMoveSpeed" },
          projectile_vision = { 225, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B28.lua:128 iVisionRadius" },
        },
        text = {
          zh_TW = { name = "火流焰",
            body = "身纏火焰 {duration} 秒，移動速度提高 {move_speed_pct}。期間每 {interval} 秒朝周圍 {radius} 內一個看得見的敵人（含建築）射出火球（可被閃避）：造成 {damage} 魔法傷害並點燃 {dot_duration} 秒，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害；建築只受 {building_pct} 傷害且不會被點燃。" },
          en = { name = "Fire Stream",
            body = "Wreathes himself in flame for {duration} seconds, gaining {move_speed_pct} movement speed. Every {interval} seconds he shoots a fireball (dodgeable) at one visible enemy within {radius}, buildings included: it deals {damage} magic damage and ignites the target for {dot_duration} seconds, dealing {dot_damage} magic damage every {dot_interval} seconds; buildings take only {building_pct} damage and are not ignited." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b28r_flame", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "火流焰", body = "移動速度提高 {move_speed_pct}；每 {interval} 秒朝周圍 {radius} 內一個看得見的敵人射出火球，造成 {damage} 魔法傷害並點燃。" },
              en = { name = "Fire Stream", body = "+{move_speed_pct} movement speed; every {interval} seconds shoots a fireball at a visible enemy within {radius}, dealing {damage} magic damage and igniting it." },
            },
            reveal = "public", cue = "b28r_flame_aura",
            stats = { move_speed_pct = "$move_speed_pct" },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "for_each", as = "victim",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING,
                    flags = { "visible_only" }, limit = 1 },
                  actions = {
                    { op = "projectile", kind = "tracking", from = "self", to = "victim", speed = "$projectile_speed",
                      dodgeable = true, vision = "$projectile_vision", cue = "b28r_fireball",
                      on_hit = {
                        { op = "if", cond = "it.is_building",
                          actions = { { op = "damage", to = "it", type = "magical", amount = "$damage * $building_pct" } },
                          otherwise = {
                            { op = "damage", to = "it", type = "magical", amount = "$damage" },
                            { op = "apply_status", status = "b28r_burn", to = "it" },
                          } },
                      } },
                  } },
              } },
            },
            notes = "原作 modifier_B28R：Duration 10、MOVESPEED_BONUS_PERCENTAGE、ThinkInterval 0.3 呼叫 B28R_OnIntervalThink；沒寫 IsPurgable，暫定 basic。" },
          { id = "b28r_burn", duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "點燃", body = "每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Ignited", body = "Takes {dot_damage} magic damage every {dot_interval} seconds." },
            },
            reveal = "public",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B28R_dot：Duration 3、ThinkInterval 1、datadriven Damage；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b28r_flame", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:112-139",
            text = "每 0.3 秒以 FIND_ANY_ORDER 搜尋 600 內的敵方英雄、部隊與建築（不含魔免），取第一個施法者隊伍看得見的單位射一顆火球就 break；資料以 entity id 順序取第一個（limit 1）。建築也會被當成目標，搶走對英雄的火球。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:369",
            text = "B28R_msBonus 一級是 0：一級沒有加速。" },
          { policy = "as_source", ref = "npc/B_Unified/B28.txt:346-361",
            text = "持續傷害每次命中只刷新持續時間；同一目標每 0.3 秒被刷新一次。推定 datadriven 刷新不重設 ThinkInterval（同 A04），仍每秒一跳；需 M2 實測。" },
          { policy = "design_intent", ref = "npc/B_Unified/B28.txt:346-361",
            text = "持續 3 秒、ThinkInterval 1 的第三跳是否落在到期當下依引擎時序；照設計意圖每秒一跳。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B28.lua:132",
            text = "投射物表的 iUnitTargetType 誤填 ability:GetAbilityDamageType()；追蹤投射物不看這個欄位，沒有影響。" },
          { policy = "note", text = "KV 沒寫 AbilityCastPoint（IMMEDIATE）；AbilityCastRange 600 只是指示，等於 B28R_radius。" },
        },
        provenance = { "npc/B_Unified/B28.txt:287-393", "vscripts/heroes/B_Unified/B28.lua:99-139" },
      },
      -------------------------------------------------------------------------
      { id = "b28_serpent_fire", source_id = "B28T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 125, 120, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 300, 400, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          radius = { 800, unit = "dist", source = "kv:B28T_radius" },
          base_damage = { 250, 325, 400, unit = "dmg", damage_type = "magical", source = "kv:base_damage" },
          bonus_damage = { 50, unit = "pct", kv = { 0.5 },
            source = "kv:bonus_damage", note = "每次彈跳增加的傷害比例",
            source_differs = "KV 以比例 0.5 表示；pct 寫成百分比數字" },
          wave_num = { 4, 8, 12, unit = "count", source = "kv:wave_num",
            note = "FC-089（D13）：原作沒有讀取，每個可見敵人都放出一道；照 KV 名稱「閃電數量」限制蛇火道數" },
          max_jumps = { 4, unit = "count", source = "design:FC-089（D13）原作 jump_max 100 等於沒有上限，改為每道蛇火最多跳 4 次",
            note = "原作 jump_count 從 jump_max 起算、每跳先減一再判斷大於零，最多跳 jump_max − 1 = 99 次（vscripts/heroes/B_Unified/B28.lua:225-229）" },
          jump_delay = { 0.3, unit = "s", source = "kv:jump_delay" },
          jump_radius = { 800, unit = "dist", source = "kv:jump_radius" },
          damage_delay = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B28.lua:278 Timers:CreateTimer(1)" },
          reveal_radius = { 300, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:266 AddFOWViewer 300" },
          reveal_duration = { 2, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:266 AddFOWViewer 2.0" },
          hint_duration = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B28.lua:147-152 spell hint 0.5" },
        },
        text = {
          zh_TW = { name = "鬼謀．蛇火",
            body = "對目標地點 {radius} 內最近的至多 {wave_num} 個看得見的敵方英雄與部隊造成 {base_damage} 魔法傷害，並從每個目標各放出一道蛇火：每 {jump_delay} 秒跳向 {jump_radius} 內最近、這道蛇火還沒打過的看得見的敵人，跳到後 {damage_delay} 秒造成傷害；每多跳一次，傷害增加基礎傷害的 {bonus_damage}，最多跳 {max_jumps} 次。沒有新目標時這道蛇火結束。" },
          en = { name = "Serpent Fire Stratagem",
            body = "Deals {base_damage} magic damage to up to {wave_num} visible enemy heroes and units nearest the target point within {radius} and releases a serpent fire from each of them: every {jump_delay} seconds it jumps to the nearest visible enemy within {jump_radius} that this serpent has not hit yet, dealing damage {damage_delay} seconds after landing; each jump adds {bonus_damage} of the base damage, up to {max_jumps} jumps. A serpent ends when it finds no new target." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_phase_start = { { op = "cue", id = "spell_hint", at = "point", radius = "$radius", duration = "$hint_duration" } },
          on_cast = {
            { op = "cue", id = "b28t_flames_sound", at = "caster" },
            { op = "for_each",
              -- FC-089（D13）：最多 wave_num 道蛇火，取離施放點最近的起點
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                where = "not it.is_invisible or has_state(it, revealed)", order = "nearest", limit = "$wave_num" },
              actions = {
                { op = "chain", from = "caster", first = "it", jumps = "$max_jumps", radius = "$jump_radius",
                  interval = "$jump_delay", team = "enemy", types = BASIC, order = "nearest",
                  where = "not it.is_invisible or has_state(it, revealed)", cue = "b28t_bolt",
                  actions = {
                    -- jump：第幾個目標（first 是 1）；原作第 n 次彈跳是 jump = n + 1
                    { op = "if", cond = "jump == 1",
                      actions = { { op = "damage", to = "it", type = "magical", amount = "$base_damage", require_source_alive = false, flags = { "no_amplify" } } },
                      otherwise = {
                        { op = "reveal", at = "it", radius = "$reveal_radius", duration = "$reveal_duration", teams = "own" },
                        { op = "cue", id = "b28t_hit_glow", at = "it" },
                        { op = "delay", time = "$damage_delay", capture = { "jump" }, actions = {
                          { op = "if", cond = "exists(it)", actions = {
                            { op = "damage", to = "it", type = "magical", amount = "$base_damage * (1 + (jump - 1) * $bonus_damage)",
                              require_source_alive = false, flags = { "no_amplify" } },
                          } },
                        } },
                      } },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B28.txt:491-495",
            text = "FC-089（D13 修正：蛇火最多 wave_num 4/8/12 道（取離施放點最近的起點），每道最多跳 4 次；每道各有命中表，不會互相卡住）。原作：KV wave_num（閃電數量）4/8/12 沒有被讀取：範圍內每個看得見的敵人都各自放出一道蛇火，數量沒有上限；jump_max 100 等於沒有上限，每道蛇火會打遍 800 跳距內連得到的所有敵人（包括其他蛇火的起點），傷害每跳增加 50%。五個聚在一起的敵人各會被打五次、倍率一路疊高，遠超說明的設計。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B28.lua:217-225",
            text = "每次彈跳以「ability.target[i] == 這個目標」反查是哪一道蛇火，取最後一個符合的 i：所有蛇火同時開始、每 0.3 秒在同一個 tick 同步跳，兩道蛇火同時跳到同一個單位時，兩個 Timer 都改用同一道（較新的）蛇火的計數與命中表，另一道就此卡住不再前進，而接手的那道之後每一步都被執行兩次（兩倍傷害、計數一次扣二）。這是同一 tick 內查表衝突的副作用；資料以 chain 表達每道蛇火各自的計數與命中表（設計意圖），不重現合併。是否要照原作重現（需要 hook）列入使用者決定。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B28.lua:197-236",
            text = "chain 的 jump 是第幾個目標（first 是 1）：first 在施放當下受到基礎傷害；原作第 n 次彈跳（jump = n + 1）在跳到後 damage_delay 秒造成 base_damage × (1 + n × bonus_damage)，n 是 jump_max − 剩餘次數（vscripts/heroes/B_Unified/B28.lua:225-275）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:161-171,241-246",
            text = "起點與彈跳都只選「沒有隱形，或身上有 modifier_truesight」的敵方英雄與部隊（FLAG 0，不含魔免）；起點的 FindUnitsInRadius 不檢查視野，迷霧中沒有隱形的敵人也會被選到。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B28.lua:249-263,276-282",
            text = "彈跳取 jump_radius 內距離嚴格小於目前最近值的單位（同距離取 FIND_ANY_ORDER 的第一個）；命中表 target.hit[instance] 存在單位上、跨施放保留但以遞增的 instance 區分。第 n 跳的傷害是 base_damage × (1 + n × 0.5)，在跳到後 1 秒以 ApplyDamage 造成（只檢查目標與施法者實體仍有效，不檢查施法者存活）；跳到時給施法者隊伍 300 半徑、2 秒視野。" },
          { policy = "no_effect", ref = "npc/B_Unified/B28.txt:456-500",
            text = "KV B28T_finishTime 1.0、stun_duration 1.0 都沒有被讀取；蛇火沒有暈眩（wave_num 見 FC-089）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B28.lua:141-153",
            text = "前搖開始時建 hide_unit 承載 0.5 秒範圍提示，只是呈現，改用 cue。函式名稱帶 _old（B28T_old_jump_init／B28T_old_Jump）但是 current 版在用。" },
        },
        provenance = { "npc/B_Unified/B28.txt:395-502", "vscripts/heroes/B_Unified/B28.lua:141-292" },
      },
    },
  }
end
