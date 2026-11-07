-- C22 佐佐木小次郎（舊版，11.2B 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C22"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:338）；11 版開放（addon_game_mode.lua:236），
-- 自動學習 heromap_autoskill["11"] = ""（addon_game_mode.lua:441），舊版沒有 D（本體的 C22D 不會加入）。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C22*_old），招式名取自 KV 註解（npc/C_Neutral/C22.txt:543、625、751、860），
-- 說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C22_precache 屬規則層或資產，不列入。
-- 慣例照本體 C22：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- ApplyDamage／datadriven Damage 不檢查施法者存活，寫 require_source_alive = false；HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C22_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C22", text = { zh_TW = { name = "佐佐木小次郎（舊版）" }, en = { name = "Sasaki Kojiro (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c22_old_swallow", source_id = "C22W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C22.txt:564" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C22.txt:567" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C22.txt:558" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C22.txt:559" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C22.txt:623" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun_time@npc/C_Neutral/C22.txt:615" },
          attack_speed = { 50, 100, 150, 200, unit = "atk_speed", source = "kv:atkspd@npc/C_Neutral/C22.txt:620" },
          buff_time = { 3, unit = "s", source = "kv:modifier_C22W_old Duration@npc/C_Neutral/C22.txt:603" },
        },
        text = {
          zh_TW = { name = "秘劍．燕",
            body = "斬擊目標敵人，造成 {damage} 魔法傷害並暈眩 {stun} 秒；之後 {buff_time} 秒內小次郎的攻擊速度提高 {attack_speed}。" },
          en = { name = "Secret Sword: Swallow",
            body = "Slashes the target enemy for {damage} magic damage and stuns it for {stun} seconds; Kojiro then gains +{attack_speed} attack speed for {buff_time} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c22w_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "秘劍．燕暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Swallow Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "datadriven Stun 動作（引擎內建 modifier_stunned）；受狀態抗性縮短。" },
          { id = "c22w_old_haste", duration = "$buff_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "秘劍．燕", body = "攻擊速度提高 {attack_speed}。" },
              en = { name = "Secret Sword: Swallow", body = "+{attack_speed} attack speed." },
            },
            reveal = "public", stats = { attack_speed = "$attack_speed" },
            notes = "原作 modifier_C22W_old：Duration 3、ATTACKSPEED_BONUS_CONSTANT、未寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c22w_old_voice", at = "caster" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
            { op = "apply_status", status = "c22w_old_stun", to = "target" },
            { op = "apply_status", status = "c22w_old_haste", to = "caster" },
            { op = "cue", id = "c22w_old_impact_sound", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C22.txt:570-598",
            text = "全部是 datadriven 動作：先傷害、再引擎暈眩、再給自己加攻速；施法距離只有 200（近身）。" },
          { policy = "note", ref = "npc/C_Neutral/C22.txt:560-561",
            text = "施法動作 ECHO_SLAM、播放速度 2 屬呈現。" },
        },
        provenance = { "npc/C_Neutral/C22.txt:543-624" },
      },
      -------------------------------------------------------------------------
      { id = "c22_old_return", source_id = "C22E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, 21, 20, 19, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C22.txt:636" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C22.txt:637" },
          cast_range = { 900, 1000, 1100, 1200, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C22.txt:638" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C22.txt:635" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C22.txt:749" },
          stun = { 2.5, unit = "s", source = "kv:stun_time@npc/C_Neutral/C22.txt:736" },
          pull_delay = { 0.1, unit = "s", source = "kv:pull_back_delay@npc/C_Neutral/C22.txt:741" },
          pull_max = { 2000, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:284 dis < 2000" },
          pull_offset = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:286 GetForwardVector()*100" },
          projectile_speed = { 1800, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/C_Neutral/C22.txt:661" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius@npc/C_Neutral/C22.txt:662" },
        },
        text = {
          zh_TW = { name = "秘劍．返",
            body = "向目標敵人射出可被閃避的追蹤劍氣，命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒；{pull_delay} 秒後若目標仍被暈眩、且離小次郎不到 {pull_max}，把它拉到小次郎面前並下令攻擊它。" },
          en = { name = "Secret Sword: Return",
            body = "Launches a dodgeable tracking sword wave at the target enemy, dealing {damage} magic damage and stunning it for {stun} seconds on hit; {pull_delay} seconds later, if the target is still stunned and within {pull_max} of Kojiro, it is pulled in front of him and he is ordered to attack it." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c22e_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "秘劍．返暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Return Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_C22E_old_stun：IsStunDebuff、IsDebuff、未寫 IsPurgable（暫定 basic）；不在 ultimate_modifiers。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c22e_old_voice", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, vision = "$projectile_vision", cue = "c22e_old_wave",
              on_hit = {
                { op = "cue", id = "c22e_old_impact_sound", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "apply_status", status = "c22e_old_stun", to = "it" },
                { op = "let", name = "victim", value = "it" },
                { op = "delay", time = "$pull_delay", capture = { "victim" }, actions = {
                  { op = "if", cond = "alive(victim) and has_status(victim, c22e_old_stun) and distance(caster, victim) < $pull_max",
                    actions = {
                      { op = "teleport", who = "victim", to = "offset(caster, caster.forward, $pull_offset)", find_clear_space = false },
                      { op = "issue_order", who = "caster", order = "attack", target = "victim" },
                    } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "no_effect", ref = "npc/C_Neutral/C22.txt:722-729",
            text = "modifier_C22E_old（3 秒攻速 50～200）與 atkspd 沒有任何地方套用，沒有效果。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:280-297",
            text = "拉回前先算距離（目標若已失效會先報錯，推定不會發生，因為 0.1 秒內實體不會被移除）；用 FindClearSpaceForUnit 放到小次郎面前 100（產生器不支援對他人 find_clear_space，資料同本體 C22D 寫 false，見 assumptions.md），並以不排隊的 ATTACK_TARGET 命令取代小次郎目前的指令。" },
          { policy = "as_source", ref = "npc/C_Neutral/C22.txt:671-703",
            text = "傷害與暈眩是 datadriven 動作，命中時才結算；劍氣被閃避（Dodgeable）就什麼都不發生。" },
          { policy = "note", ref = "npc/C_Neutral/C22.txt:630",
            text = "IGNORE_BACKSWING 只影響後搖取消，屬呈現。" },
        },
        provenance = { "npc/C_Neutral/C22.txt:625-750", "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:280-297" },
      },
      -------------------------------------------------------------------------
      { id = "c22_old_ganryu", source_id = "C22R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          evasion = { 10, 15, 20, 25, unit = "pct", source = "kv:evasion@npc/C_Neutral/C22.txt:841" },
          crit_chance = { 30, 35, 40, 45, unit = "pct", source = "kv:crit_chance@npc/C_Neutral/C22.txt:851",
            note = "腳本 RandomInt(1,100) <= crit_chance，機率剛好是 crit_chance%（vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:307）" },
          pity_after = { 3, 2, 2, 2, unit = "count",
            source = "script:vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:307 C22R_old > 100/crit_chance" },
          crit_multiplier = { 130, 160, 190, 220, unit = "pct", source = "kv:crit_bonus@npc/C_Neutral/C22.txt:846" },
          heal = { 40, 60, 80, 100, unit = "heal", source = "kv:heal@npc/C_Neutral/C22.txt:856" },
        },
        text = {
          zh_TW = { name = "巖流",
            body = "被動：閃避 {evasion} 的攻擊。攻擊非建築目標時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害，並在出手時回復 {heal} 生命；連續 {pity_after} 次沒有暴擊時，下一次必定暴擊。" },
          en = { name = "Ganryu",
            body = "Passive: evades {evasion} of attacks. Attacks against non-building targets have a {crit_chance} chance to critically strike for {crit_multiplier} damage, restoring {heal} health as the attack starts; after {pity_after} such attacks in a row without a crit, the next one always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
          triggers = {
            on_attack_windup = { filter = "not event.target.is_building", actions = {
              { op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                { op = "heal", to = "self", source = "self", amount = "$heal" },
                { op = "cue", id = "c22r_old_crit_blur", at = "event.target" },
                { op = "cue", id = "c22r_old_slam_gesture", at = "self" },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "npc/C_Neutral/C22.txt:808-834",
            text = "原作在 OnAttackStart 掛一個沒有時限的暴擊 modifier，下一次 OnAttackLanded 才移除；攻擊被取消或落空時，暴擊會留給之後第一次命中的攻擊。照意圖寫成只影響這一次攻擊的 modify_attack。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:299-312",
            text = "攻擊建築直接跳過（不計數、不擲骰）；保底計數 caster.C22R_old 每次出手 +1、暴擊時歸零，大於 100 / crit_chance 必暴：一到四級分別在連續 3、2、2、2 次沒暴擊後必中。回血在出手時（前搖開始），攻擊被取消照樣回血。" },
          { policy = "note", ref = "npc/C_Neutral/C22.txt:765-774",
            text = "閃避是 Passive 的 modifier_ninja_cloth（EVASION_CONSTANT）。" },
        },
        provenance = { "npc/C_Neutral/C22.txt:751-859", "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:299-318" },
      },
      -------------------------------------------------------------------------
      { id = "c22_old_earth_swallow", source_id = "C22T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C22.txt:876" },
          mana_cost = { 260, 310, 360, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C22.txt:877" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6@npc/C_Neutral/C22.txt:872-873" },
          damage = { 1200, 1400, 1600, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C22.txt:878" },
          radius = { 600, 625, 650, unit = "dist", source = "kv:aoe_radius@npc/C_Neutral/C22.txt:947" },
          start_delay = { 1, unit = "s", source = "kv:start_delay@npc/C_Neutral/C22.txt:942" },
          front_offset = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:331 GetForwardVector()*100" },
          dot_damage = { 40, 60, 80, unit = "dmg", damage_type = "magical", source = "kv:dot_damage@npc/C_Neutral/C22.txt:952" },
          dot_tick = { 1, unit = "s", source = "kv:dot_damage_delay@npc/C_Neutral/C22.txt:957" },
          dot_duration = { 10.1, unit = "s", source = "kv:dot_duration@npc/C_Neutral/C22.txt:962" },
        },
        text = {
          zh_TW = { name = "隱秘劍．地燕斬",
            body = "劈開身前的大地，{start_delay} 秒後對該處 {radius} 內的敵人（含魔免單位）造成 {damage} 魔法傷害，並摧毀範圍內的樹木；被擊中的敵人之後 {dot_duration} 秒內每 {dot_tick} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Hidden Secret Sword: Earth Swallow Slash",
            body = "Splits the ground in front of Kojiro; after {start_delay} seconds, enemies within {radius} of that spot (spell-immune units included) take {damage} magic damage and trees there are destroyed. Enemies hit then take {dot_damage} magic damage every {dot_tick} seconds for {dot_duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c22t_old_bleed", duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "地燕斬", body = "每 {dot_tick} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Earth Swallow Slash", body = "Takes {dot_damage} magic damage every {dot_tick} seconds." },
            },
            reveal = "public",
            triggers = {
              periodic = { interval = "$dot_tick", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_C22T_old_apply_dot_damage：IsDebuff、Duration 10.1、ThinkInterval 1、未寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c22t_old_split_earth_sound", at = "caster" },
            { op = "let", name = "center", value = "offset(caster, caster.forward, $front_offset)" },
            { op = "cue", id = "c22t_old_earth_splitter", at = "center" },
            { op = "delay", time = "$start_delay", capture = { "center" }, actions = {
              { op = "destroy_trees", at = "center", radius = "$radius" },
              { op = "for_each",
                select = { from = "center", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                  flags = { "include_magic_immune" } },
                actions = {
                  { op = "apply_status", status = "c22t_old_bleed", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
              { op = "cue", id = "c22t_old_blast", at = "center" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:323-325",
            text = "FC-181 D22 舊版，比照 D13 修：原作 GetLevelSpecialValueFor(\"aoe_radius\", GetLevel()) 用 1 起算的等級當索引，一、二級讀到 625、650，三級超出 KV 的三個值（推定取最後一個 650）；照 KV 設計意圖改為 600/625/650。start_delay 只有一個值，不受影響。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:331-362",
            text = "中心在施放當下小次郎面前 100，延遲後結算不跟著小次郎移動。技能 TargetFlags 含 MAGIC_IMMUNE_ENEMIES：魔免敵人也會被選到、掛上持續傷害，但魔法傷害由規則層歸零；ApplyDamage 不檢查施法者存活。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C22.txt:903-918",
            text = "modifier_C22T_old_aura（KV 註解「沒用到」）沒有被套用。" },
          { policy = "as_source", ref = "npc/C_Neutral/C22.txt:962",
            text = "持續傷害時間 10.1 秒、每秒一跳，共十跳（KV 註解：多一點時間確保最後一下）。" },
          { policy = "note", ref = "npc/C_Neutral/C22.txt:888-893",
            text = "item_soul_adder.lua 的 sound 只播 split_earth 音效，屬呈現。" },
        },
        provenance = { "npc/C_Neutral/C22.txt:860-965", "vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:320-372" },
      },
    },
  }
end
