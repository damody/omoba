-- A22_old 仙石秀久（舊版，11 版技能組 *_old，KV 標「仙石秀久 11.2B」）— package schema v3。
-- 版本：原作 11 版（tofar_goback.lua choose_11 拔掉全部技能後，依 heromap_skill A22 ["11"] = "WERDT" 逐一 AddAbility("A22" .. 字母 .. "_old")，
--   heromap_autoskill A22 ["11"] = "D" 自動學 A22D_old；vscripts/addon_game_mode.lua:162,264,367、vscripts/items/Addon_Items/tofar_goback.lua:95-139）。
-- 槽位：A22W_old/E_old/R_old/T_old → Q/W/E/R，A22D_old 照慣例放 D（NOT_LEARNABLE、MaxLevel 1，寫在 hero.auto_learn）。
-- magical_resistance、A22_precache、attribute_bonusx 屬規則層或資產，不列入。
-- 慣例照本體 sengoku_hidehisa：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"。
-- 原作沒有任何 _old 招式的說明文字，說明依 KV 與腳本還原（assumptions.md）。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  -- A22T_old 每次結算的五個圓（中央、±200 半徑 250，±400 半徑 200），去重後各自處理
  local function circles(pos)
    local out = { { op = "let", name = pos, value = "offset(origin, dir, $speed * $tick * k + $lead)" } }
    local cs = { { "0", "$inner_radius" }, { "$inner_offset", "$inner_radius" }, { "0 - $inner_offset", "$inner_radius" },
        { "$outer_offset", "$outer_radius" }, { "0 - $outer_offset", "$outer_radius" } }
    for i = 1, #cs do
      local c = cs[i]
      out[#out + 1] = { op = "for_each",
        select = { from = "offset(" .. pos .. ", side, " .. c[1] .. ")", shape = "circle", radius = c[2], team = "enemy", types = BASIC,
          flags = { "include_magic_immune" },
          where = "not has_status(it, a22t_old_tick_mark) and stacks(it, a22t_old_hits) < $max_hits" },
        actions = {
          { op = "apply_status", status = "a22t_old_tick_mark", to = "it" },
          { op = "if", cond = "has_status(it, a22t_old_hits)",
            actions = { { op = "add_stacks", to = "it", status = "a22t_old_hits", count = 1 } },
            otherwise = { { op = "apply_status", status = "a22t_old_hits", to = "it", stacks = "1" } } },
          { op = "damage", to = "it", type = "magical", amount = "$damage" },
          { op = "knockback", who = "it", from = "away", distance = "$push", duration = "$push_time" },
        } }
    end
    return out
  end
  return {
    schema_version = 3, kind = "hero", id = "A22_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A22", text = { zh_TW = { name = "仙石秀久（舊版）" }, en = { name = "Sengoku Hidehisa (Old)" } },
      auto_learn = { "a22_old_taunt" } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a22_old_taunt", source_id = "A22D_old", slot = "D", kind = "active", max_level = 1, tags = { "basic" },
        learnable = false,
        values = {
          cooldown = { 70, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 145, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 600, unit = "dist", source = "kv:radius" },
          duration = { 3, unit = "s", source = "kv:duration" },
          ping_damage = { 1, unit = "dmg", damage_type = "magical", hidden = true, source = "kv:modifier_A22D_old OnCreated Damage Damage" },
          regen = { 24, unit = "per_s", source = "kv:regen" },
          regen_time = { 20, unit = "s", source = "kv:modifier_A22D_old2 Duration" },
        },
        text = {
          zh_TW = { name = "嘲弄",
            body = "嘲弄周圍 {radius} 內的敵方英雄與單位（包括魔法免疫者），強制他們攻擊仙石秀久，持續 {duration} 秒；同時自身每秒回復 {regen} 生命，持續 {regen_time} 秒。開場自動學會，無法升級。" },
          en = { name = "Mockery",
            body = "Mocks enemy heroes and units within {radius} (spell-immune ones included), forcing them to attack Hidehisa for {duration} seconds; Hidehisa also regenerates {regen} health per second for {regen_time} seconds. Learned automatically at the start and cannot be upgraded." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a22d_old_mark", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "嘲弄", body = "被仙石秀久嘲弄，強制攻擊他 {duration} 秒。" },
              en = { name = "Mocked", body = "Mocked by Hidehisa and forced to attack him for {duration} seconds." },
            },
            reveal = "public", cue = "a22d_old_target_overhead",
            triggers = {
              on_created = { { op = "damage", to = "self", type = "magical", amount = "$ping_damage", source = "caster", require_source_alive = false } },
            },
            notes = "原作 modifier_A22D_old：IsDebuff、IsPurgable 1、ThinkInterval 0.1 但沒有 OnIntervalThink；OnCreated 造成 1 點魔法傷害。" },
          { id = "a22d_old_taunt", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "taunt" },
            notes = "原作 SetForceAttackTarget(caster) 加攻擊命令，duration 秒後清除；不是 modifier，驅散不會解除。" },
          { id = "a22d_old_regen", duration = "$regen_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "嘲弄", body = "每秒回復 {regen} 生命，持續 {regen_time} 秒。" },
              en = { name = "Mockery", body = "Regenerates {regen} health per second for {regen_time} seconds." },
            },
            reveal = "public", cue = "a22d_old_regen_glow", stats = { hp_regen = "$regen" },
            notes = "原作 modifier_A22D_old2：IsBuff、IsPurgable 1、HEALTH_REGEN_CONSTANT regen。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a22d_old_vo", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune", "include_illusions" } },
              actions = {
                { op = "issue_order", who = "it", order = "attack", target = "caster", force = true },
                { op = "apply_status", status = "a22d_old_taunt", to = "it" },
                { op = "apply_status", status = "a22d_old_mark", to = "it" },
              } },
            { op = "apply_status", status = "a22d_old_regen", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:93-134",
            text = "嘲弄是 SetForceAttackTarget(caster) 加一次 ATTACK_TARGET 指令，duration 秒後以 Timer 清除，與 modifier_A22D_old 無關：驅散標記不解除嘲弄，秀久死亡也不提前解除。選取含魔免敵人。" },
          { policy = "as_source", ref = "npc/A_Oda/A22.txt:461-470",
            text = "標記的 1 點魔法傷害是 KV Damage，不檢查施法者存活。" },
          { policy = "note", ref = "npc/A_Oda/A22.txt:425-436",
            text = "MaxLevel 1、NOT_LEARNABLE；AbilityCastRange／AOERadius 600 只是指示器。B24E.vo1 與 EarthSplitter 音效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A22.txt:420-509", "vscripts/heroes/A_Oda/A22.lua:93-134" },
      },
      -------------------------------------------------------------------------
      { id = "a22_old_tiger_swing", source_id = "A22W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 140, 180, 220, 260, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          spread = { 30, unit = "deg", source = "script:vscripts/heroes/A_Oda/A22.lua:146-149 ±30 度" },
          wave_speed = { 2000, unit = "speed", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:164 vec * 2000" },
          wave_distance = { 800, unit = "dist", source = "script:vscripts/heroes/A_Oda/A22.lua:165 fDistance" },
          wave_count = { 3, unit = "count", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:181-183 三次 CreateLinearProjectile" },
          wave_radius = { 175, unit = "dist", source = "script:vscripts/heroes/A_Oda/A22.lua:166-167 fStartRadius／fEndRadius" },
        },
        text = {
          zh_TW = { name = "虎振",
            body = "朝目標方向揮出三道劍氣（正前方與左右各偏 {spread} 度），各飛行 {wave_distance}、寬 {wave_radius}，對碰到的每個非魔免敵方英雄與單位造成 {damage} 魔法傷害；同時被多道劍氣碰到會受到多次傷害。" },
          en = { name = "Tiger Swing",
            body = "Swings three blade waves in the target direction (one straight ahead and one {spread} degrees to each side), each travelling {wave_distance} with a {wave_radius} radius and dealing {damage} magic damage to every non-spell-immune enemy hero and unit it touches; units touched by several waves take damage several times." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "a22w_old_vo", at = "caster" },
            { op = "let", name = "dir", value = "direction(caster, point)" },
            { op = "repeat", count = "$wave_count", as = "w", actions = {
              { op = "projectile", kind = "linear", from = "caster", cue = "a22w_old_wave",
                direction = "rotate(dir, select(w == 0, 0, select(w == 1, $spread, 0 - $spread)))",
                speed = "$wave_speed", distance = "$wave_distance", radius = "$wave_radius", pierce = true,
                team = "enemy", types = BASIC,
                on_hit = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:137-184",
            text = "三道線性投射物共用同一份設定（HERO|BASIC、旗標 NONE），從秀久位置出發；KV 的 BUILDING 目標類型沒有用到。OnProjectileHitUnit 以 KV Damage 結算（不檢查施法者存活），DeleteOnHit 0，每道穿透。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A22.lua:176",
            text = "iVisionRadius = vision_radius 是未定義的全域變數（nil），投射物不提供視野。KV AOERadius 400 沒有作用。" },
          { policy = "note", ref = "npc/A_Oda/A22.txt:531-547", text = "語音、item_soul_adder.lua 的 sound 與命中音效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A22.txt:510-566", "vscripts/heroes/A_Oda/A22.lua:137-184" },
      },
      -------------------------------------------------------------------------
      { id = "a22_old_counterattack", source_id = "A22E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 40, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 70, 80, 90, 100, unit = "mana", source = "kv:AbilityManaCost" },
          chance = { 50, unit = "pct", source = "kv:chance" },
          damage = { 50, 75, 100, 125, unit = "dmg", source = "kv:AbilityDamage", note = "腳本讀 GetAbilityDamage()；AbilitySpecial damage 數字相同" },
          attack_speed = { 15, 30, 45, 60, unit = "atk_speed", source = "kv:atkspd" },
          radius = { 350, unit = "dist", source = "kv:radius" },
          active_time = { 15, unit = "s", source = "kv:modifier_A22E_old2 Duration" },
          immune_pct = { 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A22.lua:236-237 GetAbilityDamage()*0.5" },
        },
        text = {
          zh_TW = { name = "反擊",
            body = "被動：攻擊速度提高 {attack_speed}；每次被攻擊命中時有 {chance} 機率反擊，對周圍 {radius} 內非魔免的敵方英雄與單位造成 {damage} 魔法傷害。主動：{active_time} 秒內，被攻擊命中時另有 {chance} 機率再反擊一次，對周圍 {radius} 內的敵方英雄與單位造成 {damage} 純粹傷害（魔免單位只受 {immune_pct}）。" },
          en = { name = "Counterattack",
            body = "Passive: +{attack_speed} attack speed; each time Hidehisa is hit by an attack there is a {chance} chance to strike back, dealing {damage} magic damage to non-spell-immune enemy heroes and units within {radius}. Active: for {active_time} seconds, each hit also has a separate {chance} chance to strike back again, dealing {damage} pure damage to enemy heroes and units within {radius} (spell-immune units take only {immune_pct})." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed" },
          triggers = {
            on_attacked_hit = {
              { op = "chance", p = "$chance", actions = {
                { op = "cue", id = "a22e_old_counter", at = "self" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "damage", to = "it", type = "magical", amount = "$damage" },
                  } },
              } },
            },
          },
        },
        statuses = {
          { id = "a22e_old_fury", duration = "$active_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a22e_old_fury_glow",
            text = {
              zh_TW = { name = "反擊", body = "被攻擊命中時有 {chance} 機率對周圍 {radius} 內的敵人造成 {damage} 純粹傷害，持續 {active_time} 秒。" },
              en = { name = "Counterattack", body = "When hit by an attack, {chance} chance to deal {damage} pure damage to enemies within {radius}, for {active_time} seconds." },
            },
            triggers = {
              on_attacked_hit = {
                { op = "chance", p = "$chance", actions = {
                  { op = "cue", id = "a22e_old_counter", at = "self" },
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                      flags = { "include_magic_immune" } },
                    actions = {
                      { op = "damage", to = "it", type = "pure",
                        amount = "select(it.is_magic_immune, $damage * $immune_pct, $damage)" },
                    } },
                } },
              },
            },
            notes = "原作 modifier_A22E_old2：IsHidden 0、Duration 15、OnAttacked Random %chance → A22E_old_OnAttacked2；未寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "a22e_old_fury", to = "caster" },
            { op = "cue", id = "a22e_old_cast_sound", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A22.txt:611-653",
            text = "被動 modifier_A22E_old 與主動 modifier_A22E_old2 各自擲一次 50%：主動期間一次被攻擊最多觸發兩次（一次魔法、一次純粹）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:187-240",
            text = "被動反擊選取旗標 0（不含魔免）、魔法傷害；主動反擊含魔免、純粹傷害，魔免單位乘 0.5。兩者都走 AMHC:Damage（秀久存活才有傷害）。" },
          { policy = "as_source", text = "datadriven OnAttacked 依原作實際行為推定為「被攻擊命中時」（on_attacked_hit，同本體九鬼海賊長靴的判斷）；Random 的 Chance 推定為獨立機率。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A22.lua:203,229", text = "ACT_DOTA_SPAWN 動作與 Hero_Tiny.Pick 音效屬呈現。" },
        },
        provenance = { "npc/A_Oda/A22.txt:567-685", "vscripts/heroes/A_Oda/A22.lua:187-240" },
      },
      -------------------------------------------------------------------------
      { id = "a22_old_swordsmanship", source_id = "A22R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_multiplier = { 200, 250, 300, 350, unit = "pct", source = "kv:crit_bonus" },
          crit_chance = { 20, unit = "pct", source = "script:vscripts/heroes/A_Oda/A22.lua:248,254 RandomInt(1,100) <= 20" },
          pity_after = { 5, unit = "count", source = "script:vscripts/heroes/A_Oda/A22.lua:249-254 caster.A22R_old 先加一，> 5 必中" },
        },
        text = {
          zh_TW = { name = "刀術",
            body = "被動：普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害；連續 {pity_after} 次沒有暴擊，下一次必定暴擊。" },
          en = { name = "Swordsmanship",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage; after {pity_after} attacks in a row without one, the next attack always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero",
              actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                { op = "cue", id = "a22r_old_slam_gesture", at = "self" },
              } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A22.lua:243-264",
            text = "原作在 OnAttackStart 先移除再依機率套上 attack speed + 0.1 秒的 PREATTACK_CRITICALSTRIKE modifier；v3 以 modify_attack 只影響這一次攻擊。攻擊被取消時原作的暴擊 modifier 會留到下一次出手前被移除。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:249-254",
            text = "計數器每次出手先加一，大於 5 時必定暴擊並歸零：連續五次沒暴擊後第六次必中；普通暴擊也會歸零。" },
          { policy = "no_effect", ref = "npc/A_Oda/A22.txt:695-703",
            text = "OnUpgrade 在 A22.lua 呼叫 A25R_Levelup，該函式定義在 A25_Oda_Nobunaga.lua，A22.lua 沒有，推定執行期報錯沒有效果（同本體 A22R 的判斷）。" },
        },
        provenance = { "npc/A_Oda/A22.txt:686-740", "vscripts/heroes/A_Oda/A22.lua:243-264" },
      },
      -------------------------------------------------------------------------
      { id = "a22_old_cyclone_slash", source_id = "A22T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 90, 50, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, 300, 350, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 5000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          duration = { 10, 15, 20, unit = "s", source = "kv:duration" },
          speed = { 500, unit = "speed", source = "script:vscripts/heroes/A_Oda/A22.lua:359,362 speed = 500，distance = duration*500" },
          length = { expr = "$speed * $duration", unit = "dist", source = "derived", note = "前進距離 500 × duration" },
          damage = { 200, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          tick = { 0.5, unit = "s", source = "script:vscripts/heroes/A_Oda/A22.lua:449 return 0.5" },
          lead = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:386 pos + dir * 100" },
          inner_offset = { 200, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:395,400 ± vec1 * 200" },
          outer_offset = { 400, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:406,412 ± vec1 * 400" },
          inner_radius = { 250, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:388,395,400 半徑 250" },
          outer_radius = { 200, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:406,412 半徑 200" },
          width = { expr = "$outer_offset + $outer_radius", unit = "dist", source = "derived", note = "中線兩側各 400 + 200" },
          right_angle = { 90, unit = "deg", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:301-309 ±90 度垂直向量" },
          push = { 250, unit = "dist", source = "script:vscripts/heroes/A_Oda/A22.lua:377 distance = 250" },
          push_time = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A22.lua:379 250 / 500" },
          max_hits = { 8, unit = "count", source = "script:vscripts/heroes/A_Oda/A22.lua:272 unit.a22t < 8" },
          hit_reset = { 12, unit = "s", source = "script:vscripts/heroes/A_Oda/A22.lua:280-282 Timers 12 秒後 unit.a22t = nil" },
          tick_mark = { 0.1, unit = "s", hidden = true, source = "design:同一次結算內的去重標記（原作 unit.a22told 在同一 tick 結束時清除），短於 tick" },
        },
        text = {
          zh_TW = { name = "氣旋斬",
            body = "朝目標方向放出氣旋，以每秒 {speed} 的速度前進 {duration} 秒（約 {length}）。每 {tick} 秒，氣旋前方左右各 {width} 寬的範圍內的敵方英雄與單位（包含魔免）受到 {damage} 魔法傷害，並被推離秀久施放的位置 {push}。每個單位在 {hit_reset} 秒內最多被擊中 {max_hits} 次。" },
          en = { name = "Cyclone Slash",
            body = "Unleashes a cyclone in the target direction that advances at {speed} per second for {duration} seconds (about {length}). Every {tick} seconds, enemy heroes and units (spell-immune ones included) within {width} to either side ahead of the cyclone take {damage} magic damage and are pushed {push} away from where Hidehisa cast it. Each unit can be hit at most {max_hits} times within {hit_reset} seconds." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a22t_old_hits", duration = "$hit_reset", stacking = { mode = "no_refresh" }, dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 unit.a22t：每次命中加一，到 8 就不再受影響；第一次命中後 12 秒的 Timer 把它清除（以不刷新時間的層數狀態表達）。" },
          { id = "a22t_old_tick_mark", duration = "$tick_mark", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 unit.a22told：同一次結算中五個圓重疊時只處理一次。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a22t_old_vo", at = "caster" },
            { op = "let", name = "origin", value = "position(caster)" },
            { op = "let", name = "dir", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "let", name = "side", value = "rotate(dir, $right_angle)" },
            { op = "let", name = "away", value = "offset(origin, dir, -1)" },
            { op = "cue", id = "a22t_old_cyclone", at = "origin", to = "offset(origin, dir, $length)", duration = "$duration" },
            { op = "repeat", interval = "$tick", first = "0", count = "$duration / $tick", as = "k", actions = circles("pos") },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:316-347",
            text = "五道線性投射物（中線與左右 200、400）只是特效：OnProjectileHitUnit 沒有任何動作（DeleteOnHit 0）。真正的結算是隱形 dummy A22T_old_UNIT 以 modifier_knockback 從秀久位置往前滑 duration × 500，每 0.5 秒在它前方 100 處搜尋五個圓（中央與 ±200 半徑 250，±400 半徑 200）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:267-285",
            text = "每個單位每次命中受 AbilityDamage 200 魔法傷害（AMHC：秀久存活才有傷害），並被 modifier_knockback 以 fake_center = 施放位置 − dir 為中心推開 250（0.5 秒、不暈眩）；推開方向是離開施放位置，不一定沿氣旋前進方向。knockback 與氣旋同速，被推的單位會一路被追著打，到 8 次為止。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A22.lua:276-282",
            text = "每次命中都排一個 12 秒後把計數清掉的 Timer，實際在第一次命中 12 秒後歸零；以不刷新時間的 12 秒層數狀態表達。大招最長 20 秒，同一單位理論上可在歸零後再被打 8 次。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A22.lua:451-455",
            text = "dummy 在 duration 秒後 ForceKill；結算計時器在 dummy 死亡後停止，所以在 0、0.5……duration − 0.5 秒共 duration / 0.5 次。" },
          { policy = "as_source", ref = "npc/npc_units_custom_load.txt:5864-5880",
            text = "dummy 帶 majia_vison（無血條、不可選取、魔免、攻擊免疫）並 AddNoDraw；它是否提供視野依 npc_dota_creature 預設視野（KV 沒寫），資料不另外表達視野（confidence medium）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A22.lua:312-315",
            text = "施放點與秀久重疊時 dir 改用面向，但 point2 = center + dir 的 center 是未定義的全域變數，會在這裡報錯中斷，整招沒有效果；資料改用面向（推定極少發生）。" },
          { policy = "note", text = "語音與 batrider_firefly_loop 音效屬呈現。caster.center／dir／point 寫入後沒有被讀取。" },
        },
        provenance = { "npc/A_Oda/A22.txt:741-788", "vscripts/heroes/A_Oda/A22.lua:267-456", "npc/npc_units_custom_load.txt:5864-5880" },
      },
    },
  }
end
