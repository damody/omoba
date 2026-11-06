-- C11 立花宗茂（2.0 版，原作 *_20 技能組）— package schema v3。每個版本是一位獨立英雄（D22）；舊版另成 tachibana_muneshige_old。
-- 槽位依據：選 2.0 版時 choose_20 拔掉全部技能，依 heromap_skill C11 ["20"] = "WERDT" 逐一 AddAbility("C11" .. 字母 .. "_20")
--   （vscripts/items/Addon_Items/tofar_goback.lua:6-48、vscripts/addon_game_mode.lua:327,430）。原作 KV 只有 C11W_20、C11E_20、C11R_20
--   （npc/C_Neutral/C11.txt:879-1239），C11D_20 與 C11T_20 沒有定義（腳本 C11.lua:352-361,560-566 有殘留函式），AddAbility 失敗；
--   接著自動學習 FindAbilityByName("C11D_20"):SetLevel(1) 對 nil 取值報錯，choose_20 在此中止。
--   所以 2.0 版實際出生只有三招：W/E/R → Q/W/E，沒有大招與 D（見 assumptions.md）。
-- 腳本 vscripts/heroes/C_Neutral/C11.lua:37-177,179-248,328-337,414-558。原作沒有 *_20 招式的說明文字，名稱取自 KV 註解，
--   說明依 KV 與腳本還原。英雄單位數值沿用本體 C11。magical_resistance、attribute_bonusx 屬規則層，不列入。
-- 慣例照本體 C11：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；HERO | BASIC 寫成 BASIC 清單。
-- 雷光電閃的彈跳照本體 bolt_routines 的寫法（投射物流水號判斷 on_end 是否未命中），只保留 2.0 版的效果。
local BASIC = { "hero", "creep", "neutral", "summon" }
local AB = "c11_v20_lightning_flash"
local function V(n) return "$" .. AB .. "." .. n end

return function(ctx)
  local routines = {
    c11w_20_fire = {
      params = { from = "point", to = "unit", first = "unit", left = "num" },
      actions = {
        { op = "set_state", var = "c11_v20_hop", value = "state.c11_v20_hop + 1" },
        { op = "let", name = "my", value = "state.c11_v20_hop" },
        { op = "projectile", kind = "tracking", from = "from", to = "to", speed = V("projectile_speed"),
          vision = V("vision"), dodgeable = true, cue = "c11w_20_bolt",
          on_hit = {
            { op = "set_state", var = "c11_v20_hop_done", value = "my" },
            { op = "call", routine = "c11w_20_land", args = { victim = "it", first = "first", left = "left" } },
          },
          -- OnProjectileFinish → C11W_miss_target：在投射物位置當成命中假目標，照樣計一跳並往下彈
          on_end = {
            { op = "if", cond = "state.c11_v20_hop_done ~= my", actions = {
              { op = "call", routine = "c11w_20_next", args = { center = "here", exclude = "caster", first = "first", left = "left" } },
            } },
          } },
      },
    },
    c11w_20_land = {
      params = { victim = "unit", first = "unit", left = "num" },
      actions = {
        { op = "if", cond = "has_status(caster, c11w_20_buff)", actions = {
          { op = "apply_status", status = "c11w_20_buff", to = "caster" },
        } },
        -- 原作先建立下一段投射物，再造成傷害
        { op = "call", routine = "c11w_20_next", args = { center = "position(victim)", exclude = "victim", first = "first", left = "left" } },
        { op = "damage", to = "victim", type = "magical", amount = V("damage"), require_source_alive = false },
        { op = "cue", id = "c11w_20_arc_lightning", at = "victim" },
      },
    },
    c11w_20_next = {
      params = { center = "point", exclude = "unit", first = "unit", left = "num" },
      actions = {
        { op = "set_state", var = "c11_v20_found", value = "false" },
        { op = "if", cond = "left > 0", actions = {
          { op = "for_each",
            select = { from = "center", shape = "circle", radius = V("jump_range"), team = "enemy", types = BASIC,
              flags = { "visible_only", "include_magic_immune" }, exclude = "exclude", order = "random", limit = "1" },
            actions = {
              { op = "set_state", var = "c11_v20_found", value = "true" },
              { op = "call", routine = "c11w_20_fire", args = { from = "center", to = "it", first = "first", left = "left - 1" } },
            } },
        } },
        { op = "if", cond = "not state.c11_v20_found", actions = {
          { op = "call", routine = "c11w_20_return", args = { from = "center", first = "first" } },
        } },
      },
    },
    c11w_20_return = {
      params = { from = "point", first = "unit" },
      actions = {
        { op = "set_state", var = "c11_v20_hop", value = "state.c11_v20_hop + 1" },
        { op = "let", name = "my", value = "state.c11_v20_hop" },
        { op = "projectile", kind = "tracking", from = "from", to = "select(alive(first), first, caster)",
          speed = V("projectile_speed"), vision = V("vision"), dodgeable = true, cue = "c11w_20_bolt",
          on_hit = {
            { op = "set_state", var = "c11_v20_hop_done", value = "my" },
            { op = "call", routine = "c11w_20_finish", args = { first = "first" } },
          },
          on_end = {
            { op = "if", cond = "state.c11_v20_hop_done ~= my", actions = {
              { op = "call", routine = "c11w_20_finish", args = { first = "first" } },
            } },
          } },
      },
    },
    c11w_20_finish = {
      params = { first = "unit" },
      actions = {
        { op = "remove_status", from = "caster", status = "c11w_20_lightning_form" },
        { op = "cue", id = "c11w_20_no_draw", attach = "caster", stop = true },
        { op = "if", cond = "alive(first) and distance(first, caster) < " .. V("return_range"), actions = {
          { op = "teleport", who = "caster", to = "position(first)", find_clear_space = true },
          { op = "issue_order", who = "caster", order = "attack", target = "first" },
        } },
      },
    },
  }

  return {
    schema_version = 3, kind = "hero", id = "C11_v20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "v20" },
    hero = { source_id = "C11", text = { zh_TW = { name = "立花宗茂（2.0）" }, en = { name = "Tachibana Muneshige (2.0)" } } },
    unit_state = {
      c11_v20_found = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "彈跳選下一個目標時是否找到（取代原作 next_target == nil 的判斷）" },
      c11_v20_hop = { type = "int", init = "0", reveal = "public", scope = "hero",
        note = "每段投射物的流水號，用來分辨投射物結束時是否已命中" },
      c11_v20_hop_done = { type = "int", init = "0", reveal = "public", scope = "hero",
        note = "最後一段命中的投射物流水號" },
    },
    routines = routines,
    abilities = {
      -------------------------------------------------------------------------
      { id = AB, source_id = "C11W_20", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 17, 15, 13, 11, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C11.txt:894" },
          mana_cost = { 110, 130, 150, 170, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C11.txt:895" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C11.txt:891" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C11.txt:892" },
          damage = { 100, 150, 200, 250, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C11.txt:896" },
          max_jump = { 5, unit = "count", source = "kv:max_jump@npc/C_Neutral/C11.txt:903" },
          jump_range = { 700, unit = "dist", source = "kv:jump_range@npc/C_Neutral/C11.txt:913" },
          projectile_speed = { 2000, unit = "speed", hidden = true, source = "kv:projectile_speed@npc/C_Neutral/C11.txt:918" },
          vision = { 800, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:435 iVisionRadius 800" },
          return_range = { 2000, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:458-459 Length2D() < 2000" },
          hide_duration = { 3, unit = "s", hidden = true, source = "kv:modifier_C11W Duration@npc/C_Neutral/C11.txt:1002" },
          damage_bonus = { 1, 3, 5, 7, unit = "pct", source = "kv:attack_buff@npc/C_Neutral/C11.txt:908", note = "modifier_C11W_buff DAMAGEOUTGOING_PERCENTAGE" },
          buff_duration = { 4, unit = "s", source = "kv:modifier_C11W_buff Duration@npc/C_Neutral/C11.txt:1014" },
          crit_window = { 10, unit = "s", source = "kv:modifier_C11W_old Duration@npc/C_Neutral/C11.txt:1026" },
          crit_chance = { 20, unit = "pct", source = "kv:crit_chance@npc/C_Neutral/C11.txt:923",
            note = "C11.lua:339-350 RandomInt(1,100) <= 20，恰好 20%" },
          crit_bonus = { 140, 180, 200, 220, unit = "pct", source = "kv:crit_bonus@npc/C_Neutral/C11.txt:928", note = "PREATTACK_CRITICALSTRIKE" },
        },
        text = {
          zh_TW = { name = "雷光電閃",
            body = "化為雷電射向目標敵人，在敵人之間傳導，共擊中 {max_jump} 次，每次對被擊中的敵人造成 {damage} 魔法傷害；下一個目標從被擊中者周圍 {jump_range} 內看得見的其他敵人（包含魔免）隨機挑選。期間宗茂消失且無敵，結束時雷電回到第一個目標，宗茂出現在其身旁並攻擊它。施放後 {buff_duration} 秒內造成的傷害提高 {damage_bonus}（雷電每次擊中都會刷新）；{crit_window} 秒內普攻有 {crit_chance} 機率造成 {crit_bonus} 的爆擊。" },
          en = { name = "Lightning Flash",
            body = "Muneshige turns into lightning and strikes the target enemy, arcing between enemies for {max_jump} hits in total, each dealing {damage} magic damage; the next victim is chosen at random among other visible enemies (spell-immune ones included) within {jump_range} of the last one. He vanishes and is invulnerable meanwhile; at the end the lightning returns to the first target and he reappears beside it and attacks it. For {buff_duration} seconds after casting he deals {damage_bonus} more damage (refreshed by every lightning hit), and for {crit_window} seconds his attacks have a {crit_chance} chance to critically strike for {crit_bonus}." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        flags = { "pierce_magic_immune" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11w_20_lightning_form", duration = "$hide_duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", states = { "invulnerable" },
            notes = "原作 modifier_C11W（C11W_20 版，KV ApplyModifier 只套自己）：Duration 3、INVULNERABLE；沒寫 IsPurgable，暫定 basic。彈跳結束時提早移除。" },
          { id = "c11w_20_buff", duration = "$buff_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "雷光電閃", body = "造成的傷害提高 {damage_bonus}。" },
              en = { name = "Lightning Flash", body = "Deals {damage_bonus} more damage." },
            },
            reveal = "public", stats = { outgoing_damage_pct = "$damage_bonus" },
            notes = "原作 modifier_C11W_buff：DAMAGEOUTGOING_PERCENTAGE %attack_buff、Duration 4；每次命中重套（刷新）並 SetStackCount + 1，datadriven 屬性不乘層數，層數只是顯示，不表達。沒寫 IsPurgable，暫定 basic。" },
          { id = "c11w_20_crit_window", duration = "$crit_window", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "雷光", body = "普攻有 {crit_chance} 機率造成 {crit_bonus} 的爆擊。" },
              en = { name = "Lightning", body = "Attacks have a {crit_chance} chance to critically strike for {crit_bonus}." } },
            reveal = "public", cue = "c11w_20_crit_aura",
            triggers = {
              on_attack_windup = {
                { op = "chance", p = "$crit_chance", actions = {
                  { op = "modify_attack", crit_multiplier = "$crit_bonus" },
                } },
              },
            },
            notes = "原作 modifier_C11W_old（宣告在 C11W_20 的 Modifiers 裡，名稱沿用舊版）：Duration 10，OnAttackStart 呼叫 C11W_old_M_on_attack_start 擲骰，成功時套 modifier_C11W_old_crit（PREATTACK_CRITICALSTRIKE %crit_bonus），下一次 OnAttackStart 先移除；資料以 modify_attack 只影響這一次攻擊表達。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11w_20_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "c11w_20_buff", to = "caster" },
            { op = "apply_status", status = "c11w_20_crit_window", to = "caster" },
            { op = "apply_status", status = "c11w_20_lightning_form", to = "caster" },
            { op = "cue", id = "c11w_20_no_draw", attach = "caster" },
            { op = "call", routine = "c11w_20_fire",
              args = { from = "position(caster)", to = "target", first = "target", left = "$max_jump - 1" } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:449-558",
            text = "C11W_20_hit_unit：命中時先刷新 modifier_C11W_buff，再從被擊中者周圍 jump_range 內隨機挑下一個目標（只排除當下目標，可以彈回打過的人），jump_count 到 max_jump 或找不到時改射回第一個目標（已死就射回宗茂）；先發射下一段再 ApplyDamage。回程那一段命中時結束、不造成傷害。和本體不同：沒有消去法力（remove_mana_percentage 不存在，讀到 0）、沒有附加普攻、施放時沒有 ProjectileDodge。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:887; vscripts/heroes/C_Neutral/C11.lua:503-515",
            text = "目標與彈跳旗標 FOW_VISIBLE | MAGIC_IMMUNE_ENEMIES：可以指定並彈到魔免敵人（魔法傷害依規則層歸零），只彈看得見的敵人。ApplyDamage 不檢查施法者存活。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:973-983; vscripts/heroes/C_Neutral/C11.lua:171-177",
            text = "OnProjectileFinish 呼叫本體的 C11W_miss_target → C11W_hit_unit：在投射物位置建 hide_unit 當成命中，照樣計一跳、往下彈並發射下一段，對假目標的傷害沒有意義；之後讀 caster.isHit（2.0 版沒有 C11W_start，為 nil）報錯中止，所以不會附加攻擊、也不刷新增傷。推定 OnProjectileFinish 只在追蹤投射物沒命中時觸發（同本體）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:418-421,454-468",
            text = "宗茂 AddNoDraw 隱身，3 秒 Timer 或結束時現身；結束時若第一個目標存活且離宗茂原位 2000 內，以 FindClearSpace 傳到它身上並下攻擊命令。增傷 buff 套在施放時並在每次命中刷新；爆擊視窗 10 秒，只看施放時間。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C11.txt:931-934",
            text = "KV duration 9 沒有被讀取。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:879-1068", "vscripts/heroes/C_Neutral/C11.lua:171-177",
          "vscripts/heroes/C_Neutral/C11.lua:339-350", "vscripts/heroes/C_Neutral/C11.lua:414-558" },
      },
      -------------------------------------------------------------------------
      { id = "c11_v20_thunder_fang", source_id = "C11E_20", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C11.txt:1080" },
          mana_cost = { 50, 65, 80, 95, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C11.txt:1081" },
          duration = { 20, unit = "s", source = "kv:AbilityDuration@npc/C_Neutral/C11.txt:1082" },
          attack_speed = { 20, 40, 60, 80, unit = "atk_speed", source = "kv:bonus_attack_speed@npc/C_Neutral/C11.txt:1089" },
          move_speed_pct = { 4, 8, 12, 16, unit = "pct", source = "kv:bonus_movement_speed@npc/C_Neutral/C11.txt:1094" },
        },
        text = {
          zh_TW = { name = "疾雷伏牙",
            body = "{duration} 秒內攻擊速度 +{attack_speed}、移動速度 +{move_speed_pct}。" },
          en = { name = "Swift Thunder Fang",
            body = "For {duration} seconds, gains +{attack_speed} attack speed and +{move_speed_pct} movement speed." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11e_20_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "疾雷伏牙", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。" },
              en = { name = "Swift Thunder Fang", body = "Attack speed increased by {attack_speed} and movement speed by {move_speed_pct}." },
            },
            reveal = "public", cue = "c11e_20_static_link", stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_C11E（C11E_20 版）：ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11e_20_vo", at = "caster" } },
          on_cast = { { op = "apply_status", status = "c11e_20_haste", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:1089-1095",
            text = "和本體雷之疾同構，移動速度改為 4/8/12/16%（本體 7~13%）；攻速是 ATTACKSPEED_BONUS_CONSTANT 點數。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:1070-1156" },
      },
      -------------------------------------------------------------------------
      { id = "c11_v20_thunder_eye", source_id = "C11R_20", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C11.txt:1169" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C11.txt:1170" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C11.txt:1173" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C11.txt:1174" },
          radius = { 450, unit = "dist", source = "kv:radius@npc/C_Neutral/C11.txt:1188" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C11.txt:1171" },
          stun = { 0.5, 0.8, 1.1, 1.4, unit = "s", source = "kv:stun_time@npc/C_Neutral/C11.txt:1183" },
          vision_radius = { 600, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C11.lua:191 AddFOWViewer 600" },
          vision_time = { 2, unit = "s", source = "script:vscripts/heroes/C_Neutral/C11.lua:191 AddFOWViewer 2" },
          bolt_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:243" },
          bolt_vision_time = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:245" },
        },
        text = {
          zh_TW = { name = "雷之眼",
            body = "在目標區域 {radius} 範圍降下落雷，對其中的敵方英雄與部隊造成 {damage} 魔法傷害並暈眩 {stun} 秒；並取得該處 {vision_radius} 範圍的視野 {vision_time} 秒。" },
          en = { name = "The Eye of Thunder",
            body = "Calls down lightning on the target area, dealing {damage} magic damage to enemy heroes and units within {radius} and stunning them for {stun} seconds; also grants vision within {vision_radius} there for {vision_time} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11r_20_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "雷之眼暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Eye of Thunder Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" }, cue = "c11r_20_shock",
            notes = "原作 modifier_C11R_stun（C11R_20 版）：IsStunDebuff、STUNNED；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11r_20_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "reveal", at = "at", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
            { op = "for_each",
              select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "c11r_20_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            { op = "cue", id = "c11r_20_thunder_ring", at = "at" },
            { op = "delay", time = "$bolt_delay", capture = { "at" }, actions = {
              { op = "cue", id = "c11r_20_thunder_sound", at = "at" },
              { op = "reveal", at = "at", radius = "$radius", duration = "$bolt_vision_time", teams = "own" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:179-248",
            text = "與本體共用 C11R_start：沒有 MAGIC_IMMUNE 旗標，魔免敵人不受影響；沒有 FOW 旗標，隱形與迷霧中的敵人照樣受傷。先暈眩再造成傷害。和本體不同：施法距離 700、前搖 0.25。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C11.lua:218-246",
            text = "六道落雷特效與音效 dummy 屬呈現，改用 cue；0.1 秒後另給 450 範圍 0.5 秒的視野。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:1158-1237", "vscripts/heroes/C_Neutral/C11.lua:179-248" },
      },
    },
  }
end
