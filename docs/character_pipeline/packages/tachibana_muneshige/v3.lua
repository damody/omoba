-- C11 立花宗茂（current 版，heromap_skill C11 ["16"] = "WERT"）— package schema v3.1。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/C_Neutral/C11_2.txt:14-19）。current 版用 C11W/E/R/T；*_old 是 11 版、*_20 是 20 版，都不做。
-- heromap_autoskill C11 ["16"] = ""，沒有出生自動學的技能。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C11_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
--
-- 雷切（C11W）與西國無雙主動（C11T）共用同一套彈跳腳本（C11W_start／C11W_hit_unit／C11W_miss_target），
-- 這裡用 bolt_routines() 在建置期替兩招各產生一組 routine：差別只在數值擁有者、選取旗標與 C11T 每次命中的攻速加成。
-- 共用的英雄層級變數：c11_bolting（原作 caster.orb，期間西國無雙的命中效果只有六成）、c11_hop／c11_hop_done（辨識投射物是否命中）。
local BASIC = { "hero", "creep", "neutral", "summon" }

local function bolt_routines(r, p, ab, sel_flags, ult)
  local function V(n) return "$" .. ab .. "." .. n end
  local haste = {}
  if ult then haste = { { op = "apply_status", status = "c11t_haste", to = "caster" } } end
  local function with_haste(list)
    local out = {}
    for i = 1, #haste do out[#out + 1] = haste[i] end
    for i = 1, #list do out[#out + 1] = list[i] end
    return out
  end
  local land = {
    { op = "burn_mana", to = "victim", source = "caster", amount = "victim.mana * " .. V("mana_pct") },
    -- 原作先建立下一段投射物，再造成傷害與攻擊
    { op = "call", routine = p .. "_next", args = { center = "position(victim)", exclude = "victim", first = "first", left = "left" } },
  }
  if not ult then
    land[#land + 1] = { op = "damage", to = "victim", type = "magical", amount = V("damage"), require_source_alive = false, flags = { "no_amplify" } }
  end
  land[#land + 1] = { op = "if", cond = "not has_status(victim, c11_struck)", actions = {
    { op = "apply_status", status = "c11_struck", to = "victim" },
    { op = "attack", attacker = "caster", target = "victim", cannot_miss = true, ignore_range = true, instant = true,
      trigger_effects = true, use_orb = true },
  } }
  land[#land + 1] = { op = "cue", id = "c11w_arc_lightning", at = "victim" }


  r[p .. "_fire"] = {
    params = { from = "point", to = "unit", first = "unit", left = "num" },
    actions = {
      { op = "set_state", var = "c11_hop", value = "state.c11_hop + 1" },
      { op = "let", name = "my", value = "state.c11_hop" },
      { op = "projectile", kind = "tracking", from = "from", to = "to", speed = V("projectile_speed"),
        vision = "$c11_raikiri.vision", dodgeable = true, cue = "c11w_bolt",
        on_hit = with_haste({
          { op = "set_state", var = "c11_hop_done", value = "my" },
          { op = "call", routine = p .. "_land", args = { victim = "it", first = "first", left = "left" } },
        }),
        -- OnProjectileFinish：目標閃避或消失時，在投射物位置當成命中假目標，照樣計一跳並往下彈
        on_end = {
          { op = "if", cond = "state.c11_hop_done ~= my", actions = {
            { op = "call", routine = p .. "_next", args = { center = "here", exclude = "caster", first = "first", left = "left" } },
          } },
        } },
    },
  }
  r[p .. "_land"] = { params = { victim = "unit", first = "unit", left = "num" }, actions = land }
  r[p .. "_next"] = {
    params = { center = "point", exclude = "unit", first = "unit", left = "num" },
    actions = {
      { op = "set_state", var = "c11_found", value = "false" },
      { op = "if", cond = "left > 0", actions = {
        { op = "for_each",
          select = { from = "center", shape = "circle", radius = V("jump_range"), team = "enemy", types = BASIC,
            flags = sel_flags, exclude = "exclude", order = "random", limit = "1" },
          actions = {
            { op = "set_state", var = "c11_found", value = "true" },
            { op = "call", routine = p .. "_fire", args = { from = "center", to = "it", first = "first", left = "left - 1" } },
          } },
      } },
      { op = "if", cond = "not state.c11_found", actions = {
        { op = "call", routine = p .. "_return", args = { from = "center", first = "first" } },
      } },
    },
  }
  r[p .. "_return"] = {
    params = { from = "point", first = "unit" },
    actions = {
      { op = "set_state", var = "c11_hop", value = "state.c11_hop + 1" },
      { op = "let", name = "my", value = "state.c11_hop" },
      { op = "projectile", kind = "tracking", from = "from", to = "select(alive(first), first, caster)",
        speed = V("projectile_speed"), vision = "$c11_raikiri.vision", dodgeable = true, cue = "c11w_bolt",
        on_hit = with_haste({
          { op = "set_state", var = "c11_hop_done", value = "my" },
          { op = "call", routine = "c11_bolt_finish", args = { first = "first" } },
        }),
        on_end = {
          { op = "if", cond = "state.c11_hop_done ~= my", actions = {
            { op = "call", routine = "c11_bolt_finish", args = { first = "first" } },
          } },
        } },
    },
  }

end

local function bolt_start(p, ab)
  return {
    { op = "dodge_projectiles", of = "caster", kind = "tracking" },
    { op = "for_each",
      select = { from = "caster", shape = "global", team = "enemy", types = { "all" },
        flags = { "include_magic_immune", "include_invulnerable", "include_untargetable", "include_hidden" },
        where = "has_status(it, c11_struck)" },
      actions = { { op = "remove_status", from = "it", status = "c11_struck" } } },
    { op = "set_state", var = "c11_bolting", value = "true" },
    { op = "apply_status", status = "c11_lightning_form", to = "caster" },
    { op = "cue", id = "c11w_no_draw", attach = "caster" },
    { op = "call", routine = p .. "_fire",
      args = { from = "position(caster)", to = "target", first = "target", left = "$" .. ab .. ".max_jump - 1" } },
  }
end

return function(ctx)
  local routines = {
    c11_bolt_finish = {
      params = { first = "unit" },
      actions = {
        { op = "set_state", var = "c11_bolting", value = "false" },
        { op = "remove_status", from = "caster", status = "c11_lightning_form" },
        { op = "cue", id = "c11w_no_draw", attach = "caster", stop = true },
        { op = "if", cond = "alive(first) and distance(first, caster) < $c11_raikiri.return_range", actions = {
          { op = "teleport", who = "caster", to = "position(first)", find_clear_space = true },
          { op = "issue_order", who = "caster", order = "attack", target = "first" },
        } },
      },
    },
  }
  bolt_routines(routines, "c11w", "c11_raikiri", { "visible_only" }, false)
  bolt_routines(routines, "c11t", "c11_unrivaled", { "include_magic_immune" }, true)

  return {
    schema_version = 3, kind = "hero", id = "C11",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C11",
      text = { zh_TW = { name = "立花宗茂" }, en = { name = "Tachibana Muneshige" } },
    },
    unit_state = {
      c11_bolting = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "原作 caster.orb = 0.6：雷切／西國無雙彈跳期間為真，西國無雙被動的命中效果乘上 orb_mult" },
      c11_found = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "彈跳選下一個目標時是否找到（取代原作 next_target == nil 的判斷）" },
      c11_hop = { type = "int", init = "0", reveal = "public", scope = "hero",
        note = "每段投射物的流水號，用來分辨投射物結束時是否已命中" },
      c11_hop_done = { type = "int", init = "0", reveal = "public", scope = "hero",
        note = "最後一段命中的投射物流水號" },
    },
    statuses = {
      { id = "c11_lightning_form", duration = "$hide_duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
        reveal = "public", states = { "invulnerable" },
        values = {
          hide_duration = { 3, unit = "s", hidden = true, source = "kv:modifier_C11W Duration@npc/C_Neutral/C11.txt:139" },
        },
        notes = "原作 modifier_C11W（W 與 T 各有一份同名定義）：Duration 3、INVULNERABLE、IsHidden；沒寫 IsPurgable，暫定 basic。彈跳結束時提早移除。" },
      { id = "c11_struck", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
        reveal = "public",
        notes = "原作 caster.isHit 清單：本次施放已附加過攻擊的單位；每次施放開始時清空。" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c11_raikiri", source_id = "C11W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 13, 11, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 110, 130, 150, 170, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 100, 130, 160, 190, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          max_jump = { 3, 4, 5, 6, unit = "count", source = "kv:max_jump" },
          mana_pct = { 5, 7, 9, 11, unit = "pct", source = "kv:remove_mana_percentage" },
          jump_range = { 700, unit = "dist", source = "kv:jump_range" },
          projectile_speed = { 2000, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          vision = { 800, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:24 iVisionRadius 800" },
          return_range = { 2000, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C11.lua:51-52 Length2D() < 2000" },
        },
        text = {
          zh_TW = { name = "雷切",
            body = "化為雷電射向目標敵人，在敵人之間傳導，共擊中 {max_jump} 次：每次對被擊中的敵人造成 {damage} 魔法傷害、消去其現有法力的 {mana_pct}，並對每個敵人附加一次普攻（必定命中，觸發攻擊效果）；下一個目標從被擊中者周圍 {jump_range} 內看得見的其他敵人隨機挑選。期間宗茂消失且無敵，結束時雷電回到第一個目標，宗茂出現在其身旁並攻擊它。" },
          en = { name = "Raikiri",
            body = "Muneshige turns into lightning and strikes the target enemy, arcing between enemies for {max_jump} hits in total: each hit deals {damage} magic damage, removes {mana_pct} of the victim's current mana, and performs one attack on each enemy (never misses, applies attack effects); the next victim is chosen at random among other visible enemies within {jump_range} of the last one. Muneshige vanishes and is invulnerable meanwhile; at the end the lightning returns to the first target and he reappears beside it and attacks it." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "c11w_vo", at = "caster" } },
          on_cast = bolt_start("c11w", "c11_raikiri"),
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:113-146",
            text = "下一個目標從被擊中者周圍隨機挑（FIND_ANY_ORDER + RandomInt），只排除當下這個目標：可以彈回之前打過的人。第 max_jump 次命中後、或找不到下一個目標時，雷電回到第一個目標（已死就回到宗茂自己）；回程那一段不造成傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:155-167",
            text = "附加攻擊（PerformAttack，useCastAttackOrb、processProcs、neverMiss）每次施放對同一個單位只做一次；傷害與消去法力每次命中都有。順序：消去法力 → 發射下一段 → 傷害 → 攻擊。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:15,47,272-275",
            text = "彈跳期間 caster.orb = 0.6，西國無雙被動的命中效果（消去法力與魔法傷害）只有六成；回程命中時才清除。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:171-177",
            text = "OnProjectileFinish（目標閃避或消失）時在投射物位置建 hide_unit 當成命中：照樣計一跳、從那裡往下找目標，但沒有人受傷。推定 OnProjectileFinish 只在追蹤投射物沒命中時觸發；v3 用投射物流水號判斷 on_end 是否為未命中。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:44-66",
            text = "結束時若第一個目標存活且離宗茂原本的位置 2000 內，以 FindClearSpace 傳到它身上並下攻擊命令；否則原地現身。宗茂消失期間仍在原地，沒有被暈眩，可以接受命令。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:39",
            text = "目標與彈跳都用 FOW_VISIBLE、沒有 MAGIC_IMMUNE 旗標：只彈看得見、非魔免的敵人。ApplyDamage 不檢查施法者存活。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:2,8-14",
            text = "施放時 ProjectileDodge 閃掉追蹤投射物；無敵最多 3 秒（彈跳結束時提早移除），3 秒的 Timer 也會讓宗茂重新顯示。" },
          { policy = "note", text = "原作說明寫每次 100/150/200/250 傷害；KV AbilityDamage 是 100/130/160/190。說明改由數值產生。" },
          { policy = "note", ref = "npc/C_Neutral/C11.txt:98-106",
            text = "KV 的 ApplyModifier Target 區塊只有 Center／Flags、沒有 Radius，原意是只掛在自己身上（與腳本第 12 行重複套用同一個狀態）。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:30-146", "vscripts/heroes/C_Neutral/C11.lua:1-177", "vscripts/heroes/C_Neutral/C11.lua:328-340" },
      },
      -------------------------------------------------------------------------
      { id = "c11_thunder_speed", source_id = "C11E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 50, 65, 80, 95, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 20, unit = "s", source = "kv:AbilityDuration" },
          attack_speed = { 20, 40, 60, 80, unit = "atk_speed", source = "kv:bonus_attack_speed" },
          move_speed_pct = { 7, 9, 11, 13, unit = "pct", source = "kv:bonus_movement_speed" },
        },
        text = {
          zh_TW = { name = "雷之疾",
            body = "{duration} 秒內攻擊速度 +{attack_speed}、移動速度 +{move_speed_pct}。" },
          en = { name = "The Speed of Thunder",
            body = "For {duration} seconds, gains +{attack_speed} attack speed and +{move_speed_pct} movement speed." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c11e_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "雷之疾", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。" },
              en = { name = "The Speed of Thunder", body = "Attack speed increased by {attack_speed} and movement speed by {move_speed_pct}." },
            },
            reveal = "public", cue = "c11e_static_link", stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_C11E：ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11e_vo", at = "caster" } },
          on_cast = { { op = "apply_status", status = "c11e_haste", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", text = "原作說明寫「攻擊速度 20/40/60/80%」；KV 是 ATTACKSPEED_BONUS_CONSTANT（攻速點數）。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:148-232" },
      },
      -------------------------------------------------------------------------
      { id = "c11_thunder_eye", source_id = "C11R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 450, unit = "dist", source = "kv:radius" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 0.5, 0.8, 1.1, 1.4, unit = "s", source = "kv:stun_time" },
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
          { id = "c11r_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "雷之眼暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Eye of Thunder Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" }, cue = "c11r_shock",
            notes = "原作 modifier_C11R_stun：IsStunDebuff、STUNNED；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11r_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "reveal", at = "at", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
            { op = "for_each",
              select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "c11r_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
              } },
            { op = "cue", id = "c11r_thunder_ring", at = "at" },
            { op = "delay", time = "$bolt_delay", capture = { "at" }, actions = {
              { op = "cue", id = "c11r_thunder_sound", at = "at" },
              { op = "reveal", at = "at", radius = "$radius", duration = "$bolt_vision_time", teams = "own" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:192-215",
            text = "沒有 MAGIC_IMMUNE 旗標，魔免敵人不受影響；沒有 FOW 旗標，隱形與迷霧中的敵人照樣受傷。先暈眩再造成傷害。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C11.lua:218-246",
            text = "六道落雷特效與音效 dummy 屬呈現，改用 cue；0.1 秒後另給 450 範圍 0.5 秒的視野。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:234-315", "vscripts/heroes/C_Neutral/C11.lua:179-248" },
      },
      -------------------------------------------------------------------------
      { id = "c11_unrivaled", source_id = "C11T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 120, 100, 80, unit = "s", source = "kv:AbilityCooldown" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          bonus_all = { 6, 9, 12, unit = "stat", source = "kv:bonus_all" },
          mana_pct = { 9, unit = "pct", source = "kv:remove_mana_percentage" },
          damage = { 80, 100, 120, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          low_mana = { 50, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C11.lua:285-287 GetManaPercent() <= 50" },
          low_mana_mult = { 1.5, unit = "ratio", source = "script:vscripts/heroes/C_Neutral/C11.lua:287 damage * 1.5" },
          orb_mult = { 60, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C11.lua:15 caster.orb = 0.6" },
          max_jump = { 2, 4, 6, unit = "count", hidden = true, kv = { 2, 4, 6, 8 }, source = "kv:max_jump", source_differs = "KV 四個值，MaxLevel 3 只用到前三個",
            note = "KV 寫四個值 2/4/6/8，MaxLevel 3 只用到前三個；因為 jump_range 是 0，實際不會彈跳" },
          jump_range = { 0, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C11.lua:73 GetSpecialValueFor(\"jump_range\")；C11T 沒有這個鍵，得 0" },
          projectile_speed = { 4000, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          attack_speed = { 75, 125, 175, unit = "atk_speed", source = "kv:attack_speed_percentage" },
          haste_duration = { 5, unit = "s", source = "kv:duration" },
        },
        text = {
          zh_TW = { name = "西國無雙",
            body = "被動：全屬性 +{bonus_all}。普攻命中非建築的敵人時，消去其現有法力的 {mana_pct} 並造成 {damage} 魔法傷害；消去後目標法力不高於 {low_mana} 時傷害乘以 {low_mana_mult}。在{@c11_raikiri}或本招的雷電期間，這個效果只有 {orb_mult}。主動：化為雷電射向目標敵人（可以是魔免敵人），消去其現有法力的 {mana_pct} 並附加一次普攻，之後雷電立刻折返同一個目標，宗茂出現在其身旁並攻擊它。每段雷電命中時宗茂獲得 {attack_speed} 攻擊速度，持續 {haste_duration} 秒，可以疊加。" },
          en = { name = "Unrivaled in Saigoku",
            body = "Passive: +{bonus_all} to all attributes. Attacks that land on non-building enemies remove {mana_pct} of the target's current mana and deal {damage} magic damage, multiplied by {low_mana_mult} if the target is at or below {low_mana} mana afterwards. During the lightning of {@c11_raikiri} or this ability the effect is only {orb_mult} as strong. Active: turns into lightning that strikes the target enemy (can target spell-immune enemies), removing {mana_pct} of its current mana and performing one attack on it; the lightning then immediately returns to the same target and Muneshige reappears beside it and attacks it. Each lightning hit grants him +{attack_speed} attack speed for {haste_duration} seconds, stacking." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        flags = { "pierce_magic_immune" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { str = "$bonus_all", agi = "$bonus_all", int = "$bonus_all" },
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building",
              actions = {
                { op = "let", name = "m", value = "select(state.c11_bolting, $orb_mult, 1)" },
                { op = "burn_mana", to = "event.target", source = "self", amount = "event.target.mana * $mana_pct * m" },
                { op = "damage", to = "event.target", type = "magical", require_source_alive = false, flags = { "no_amplify" },
                  amount = "$damage * m * select(event.target.mana_pct <= $low_mana, $low_mana_mult, 1)" },
                { op = "cue", id = "c11t_manavoid", at = "event.target" },
              } },
            on_kill = { filter = "event.victim.is_hero", actions = { { op = "cue", id = "c11t_title" } } },
          },
        },
        statuses = {
          { id = "c11t_haste", duration = "$haste_duration", stacking = "independent", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "西國無雙", body = "攻擊速度提高 {attack_speed}，持續 {haste_duration} 秒；每段雷電命中各自計算，可以疊加。" },
              en = { name = "Unrivaled in Saigoku", body = "+{attack_speed} attack speed for {haste_duration} seconds; each lightning hit stacks separately." },
            },
            reveal = "public", stats = { attack_speed = "$attack_speed" }, cue = "c11r_shock",
            notes = "原作 modifier_C07T_attack_speed（C11T 自己的定義，與 C07 同名但屬於不同技能）：MULTIPLE、Duration 5；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c11w_vo", at = "caster" } },
          on_cast = bolt_start("c11t", "c11_unrivaled"),
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/C_Neutral/C11.lua:73,113-146",
            text = "主動沿用 C11W_hit_unit，彈跳半徑讀 jump_range；C11T 沒有這個鍵，得 0，FindUnitsInRadius 找不到其他目標，所以第一下命中後雷電就折返第一個目標，max_jump 2/4/6 沒有作用。說明寫會傳導。資料照原作不彈跳；若修正，jump_range 改用雷切的 700。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C11.lua:68,76-81",
            text = "主動沿用 C11W_hit_unit，傷害是 GetAbilityDamage()；C11T 沒有 AbilityDamage，彈跳本身的傷害是 0。實際傷害來自附加攻擊觸發的被動（六成）。" },
          { policy = "fix_candidate", ref = "vscripts/heroes/C_Neutral/C11.lua:316-325",
            text = "C11T_stun 讀 stun_time（C11T 只有 stun_duration 1），得 0，以 {duration = 0} 套 modifier_C11T_stun：推定沒有暈眩效果。說明寫「電暈 1 秒」，明顯是寫錯鍵名。資料照原作不暈眩；若修正，每段命中暈眩 stun_duration 1 秒（非魔免目標）。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:416-438",
            text = "OnProjectileHitUnit 的攻速加成在每段命中都套用，包括回到第一個目標（或宗茂自己）的回程那段；未命中（OnProjectileFinish）的那段沒有。實際只有兩段（去程與回程），共兩層，各自 5 秒。" },
          { policy = "as_source", ref = "npc/C_Neutral/C11.txt:322-327",
            text = "主動沒有 AbilityManaCost（不耗魔）；目標與彈跳帶 MAGIC_IMMUNE_ENEMIES，沒有 FOW 旗標：迷霧中與魔免的敵人都會被彈到（魔免者的魔法傷害由規則層歸零）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:262-292",
            text = "被動先消去法力再判斷剩餘法力百分比，所以消去本身可能把目標推到一半以下；caster.orb 同時縮減消去量與傷害，1.5 倍乘在縮減之後。沒有法力的單位 GetManaPercent 推定為 0，總是吃 1.5 倍。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C11.lua:257",
            text = "被動攻擊建築時直接離開；ApplyDamage 不檢查施法者存活。命中效果依 PerformAttack 觸發，雷切的附加攻擊也會觸發。" },
          { policy = "note", ref = "npc/C_Neutral/C11.txt:450-525",
            text = "C11T 有兩個 \"Modifiers\" 區塊（第二個只有 modifier_C11T_passive）；原作可玩，假設兩者都保留。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C11.lua:298-313",
            text = "「每兩次攻擊額外攻擊一次」的程式已註解，沒有效果。" },
        },
        provenance = { "npc/C_Neutral/C11.txt:317-526", "vscripts/heroes/C_Neutral/C11.lua:1-177",
          "vscripts/heroes/C_Neutral/C11.lua:250-325" },
      },
    },
    routines = routines,
  }
end
