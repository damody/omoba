-- B02_old 風魔小太郎（舊版，11 版技能組；KV 註解標「11.2B」）— package schema v3。
-- 版本：原作 heromap_version B02 ["11"] = true，heromap_skill B02 ["11"] = "WERT"（vscripts/addon_game_mode.lua:179,281），
-- heromap_autoskill B02 ["11"] = ""（vscripts/addon_game_mode.lua:384）。
-- 槽位依據：vscripts/items/Addon_Items/tofar_goback.lua:105-133 先移除全部技能，再依 "WERT" 順序
-- AddAbility B02W_old、B02E_old、B02R_old、attribute_bonusx、B02T_old，所以 W/E/R/T → Q/W/E/R；沒有 D，也沒有自動學習。
-- 舊版的迷蹤步是 Q（B02W_old，可升四級），不是本體的 D；舊版沒有幻影（忍法．朧）。
-- attribute_bonusx、B02_precache、halloween_attack、buff_* 屬規則層，都不列入。
-- 原作沒有舊版招式的繁中說明（resource/addon_tchinese.txt 沒有 B02*_old），招式名取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎 modifier_invisible 另加 tag engine_invisible（同本體）。
-- 原作 HERO | BASIC 照本體寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B02_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B02", text = { zh_TW = { name = "風魔小太郎（舊版）" }, en = { name = "Fuma Kotaro (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b02_old_vanishing_step", source_id = "B02W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, 12, 10, 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 110, 120, 130, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 25, 30, 35, 40, unit = "s", source = "kv:duration" },
          bonus_damage = { 90, 175, 260, 345, unit = "dmg", damage_type = "physical", source = "kv:bonus_damage" },
          move_speed_pct = { 10, 15, 20, 25, unit = "pct", source = "kv:speed" },
        },
        text = {
          zh_TW = { name = "忍法．迷蹤步",
            body = "立即進入隱形 {duration} 秒（不打斷目前動作），移動速度提高 {move_speed_pct}、無單位碰撞。隱形中的攻擊命中時，對目標額外造成 {bonus_damage} 物理傷害並現身；使用任何技能或道具也會現身。" },
          en = { name = "Ninja Art: Vanishing Step",
            body = "Instantly turns invisible for {duration} seconds without interrupting the current action, gaining +{move_speed_pct} movement speed and no unit collision. When an attack lands while invisible, it deals {bonus_damage} bonus physical damage to the target and reveals Kotaro; using any ability or item also reveals him." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b02w_old_wind_walk", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", cue = "b02w_old_windwalk_trail",
            states = { "no_unit_collision" },
            stats = { move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_attack_hit = {
                { op = "remove_status", from = "self", status = "b02w_old_wind_walk" },
                { op = "remove_status", from = "self", status = "b02w_old_invisible" },
                { op = "cue", id = "b02w_old_first_hit", at = "self", to = "event.target" },
                { op = "damage", to = "event.target", source = "self", type = "physical", amount = "$bonus_damage" },
              },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "b02w_old_wind_walk" },
                { op = "remove_status", from = "self", status = "b02w_old_invisible" },
              } },
            },
            notes = "原作 modifier_wind_walk_datadriven：IsBuff、IsHidden、NO_UNIT_COLLISION、MOVESPEED_BONUS_PERCENTAGE %speed；OnAttackLanded 移除兩個 modifier 並跑 B02W_old_first_hit，OnAbilityExecuted 移除兩個 modifier。沒寫 IsPurgable，暫定 basic。" },
          { id = "b02w_old_invisible", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "隱形", body = "隱形中；攻擊命中或使用技能、道具時現身。" },
              en = { name = "Invisible", body = "Invisible; landing an attack or using an ability or item reveals him." },
            },
            reveal = "public", states = { "invisible" }, tags = { "invisible", "engine_invisible" },
            notes = "原作同時套上的引擎 modifier_invisible（隱形本體）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b02w_old_vo", at = "caster" },
            { op = "cue", id = "b02w_old_windwalk", at = "caster" },
            { op = "apply_status", status = "b02w_old_wind_walk", to = "caster" },
            { op = "apply_status", status = "b02w_old_invisible", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B02.txt:727-760",
            text = "和本體 B02D 不同：舊版掛在 OnAttackLanded（攻擊命中），miss 或被閃避不會加傷也不會現身；也沒有排除建築，打建築同樣加傷。沒有降低攻擊優先度。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:428-438",
            text = "額外傷害以 AMHC:Damage 物理傷害直接打出，讀 B02W_old 自己的 bonus_damage；先移除隱形再傷害。" },
          { policy = "no_effect", ref = "npc/B_Unified/B02.txt:775-778",
            text = "AbilitySpecial fade_time 0.9 沒有被使用：隱形立即生效，沒有漸隱時間。" },
          { policy = "as_source",
            text = "on_ability_executed 在施法者自己的 on_cast 之前觸發（§12.5），所以施放本技能不會拆掉剛套上的隱形；使用道具也會現身。" },
          { policy = "note", ref = "npc/B_Unified/B02.txt:722-725",
            text = "modifier_wind_walk_datadriven 未寫 IsPurgable，依慣例寫 dispel = basic；引擎 modifier_invisible 的驅散屬性未確認（同本體）。" },
        },
        provenance = { "npc/B_Unified/B02.txt:668-796", "vscripts/heroes/B_Unified/B02.lua:428-438" },
      },
      -------------------------------------------------------------------------
      { id = "b02_old_blazing_path", source_id = "B02E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 11, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 140, 160, 180, 280, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 10, unit = "s", source = "kv:duration" },
          step_distance = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B02.lua:455 nobu_distance(point, ability.pre_position) > 100" },
          flame_lifetime = { 10, unit = "s", source = "kv:fire_duration", note = "腳本以 Timer fire_duration 後 dummy:Destroy()" },
          flame_radius = { 140, unit = "dist", source = "kv:fire_radius（modifier_B02E_2 Aura_Radius）" },
          aura_tick = { 0.1, unit = "s", hidden = true,
            source = "assumed:Dota 光環以固定間隔更新範圍內單位；以 0.1 秒檢查近似" },
          aura_linger = { 0.5, unit = "s", hidden = true,
            source = "assumed:Dota datadriven 光環離開範圍後的預設殘留 0.5 秒" },
          burn_tick = { 0.25, unit = "s", source = "kv:dmg_delay（modifier_B02E_3 ThinkInterval）" },
          damage = { 20, 30, 40, 50, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
        },
        text = {
          zh_TW = { name = "忍法．熾烈之徑",
            body = "{duration} 秒內，小太郎每移動一段距離就在腳下留下一團火焰，存在 {flame_lifetime} 秒。火焰 {flame_radius} 範圍內的敵方英雄與部隊會燃燒，每 {burn_tick} 秒受到 {damage} 魔法傷害，離開火焰後仍短暫燃燒。" },
          en = { name = "Ninja Art: Blazing Path",
            body = "For {duration} seconds, Kotaro leaves a patch of fire behind him every short distance he moves, lasting {flame_lifetime} seconds. Enemy heroes and units within {flame_radius} of a flame burn, taking {damage} magic damage every {burn_tick} seconds, and keep burning briefly after leaving it." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b02e_old_path", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "熾烈之徑", body = "每移動一段距離就在腳下留下一團火焰，存在 {flame_lifetime} 秒，燒傷靠近的敵人。" },
              en = { name = "Blazing Path", body = "Leaves a patch of fire every short distance moved, lasting {flame_lifetime} seconds and burning nearby enemies." },
            },
            reveal = "public", cue = "b02e_old_flame_trail",
            vars = {
              started = { type = "bool", init = "false", reveal = "public" },
              last_pos = { type = "point", init = "position(self)", reveal = "public" },
            },
            triggers = {
              on_moved = {
                { op = "if", cond = "not vars.started or distance(self, vars.last_pos) > $step_distance", actions = {
                  { op = "set_var", var = "started", value = "true" },
                  { op = "set_var", var = "last_pos", value = "position(self)" },
                  { op = "let", name = "at", value = "position(self)" },
                  { op = "zone", shape = "circle", at = "at", radius = "$flame_radius", duration = "$flame_lifetime",
                    interval = "$aura_tick", first = "0", cue = "b02e_old_flame",
                    on_tick = {
                      { op = "for_each",
                        select = { from = "at", shape = "circle", radius = "$flame_radius", team = "enemy",
                          types = { "hero", "creep", "neutral", "summon" } },
                        actions = {
                          { op = "apply_status", status = "b02e_old_burn", to = "it", source = "self" },
                        } },
                    } },
                } },
              },
            },
            notes = "原作 modifier_B02E：Duration %duration；OnCreated 把 ability.pre_position 設成 (0,0,9999)，OnUnitMoved 每次移動檢查與上一團火焰的平面距離超過 100 就在目前位置建 dummy 掛 modifier_B02E_2；沒寫 IsPurgable，暫定 basic。" },
          { id = "b02e_old_burn", duration = "$aura_linger", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "燃燒", body = "每 {burn_tick} 秒受到 {damage} 魔法傷害。" },
              en = { name = "Burning", body = "Takes {damage} magic damage every {burn_tick} seconds." },
            },
            reveal = "public",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_B02E_3（modifier_B02E_2 的光環狀態）：IsDebuff、不隱藏、ThinkInterval %dmg_delay 造成 %dmg 魔法傷害；光環刷新只延長存在時間，計時不重來。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b02e_old_vo", at = "caster" },
            { op = "apply_status", status = "b02e_old_path", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:440-447",
            text = "起點設在 (0,0,9999)，nobu_distance 只算平面距離：施放後第一次移動就會在腳下生火（除非英雄正好站在地圖原點附近）；和本體不同，原地不動不會留火、施放點本身不留火。以 vars.started 表達。" },
          { policy = "as_source", ref = "npc/B_Unified/B02.txt:854-869",
            text = "火焰 dummy 無敵（modifier_B02E_2 INVULNERABLE），無法被摧毀；光環沒有旗標，魔免敵人不會被點燃。" },
          { policy = "as_source", ref = "npc/B_Unified/B02.txt:864-885",
            text = "燃燒是光環狀態，同時站在多團火焰裡推定只有一份（同本體），傷害不疊加；離開火焰後依光環殘留再燒約半秒。光環更新與殘留時間是 Dota 引擎行為，數值為推定。" },
          { policy = "note", ref = "npc/B_Unified/B02.txt:814-824",
            text = "和本體不同：舊版沒有 modifier_phased（施放時不會無碰撞），也沒有 ThinkInterval 取樣，改由 OnUnitMoved 觸發。" },
        },
        provenance = { "npc/B_Unified/B02.txt:796-915", "vscripts/heroes/B_Unified/B02.lua:440-465",
          "vscripts/library/math.lua:10-14" },
      },
      -------------------------------------------------------------------------
      { id = "b02_old_eighteen_shuriken", source_id = "B02R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 170, 200, 230, 260, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 580, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          root_time = { 2.4, 2.7, 3.0, 3.3, unit = "s", source = "kv:time" },
          recheck = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:208-219 每 0.1 秒檢查" },
          shuriken_count = { 18, unit = "count", kv = { 18 },
            source = "kv:技能名（十八方手裏劍）",
            source_differs = "原作腳本 OnCreated 一發 + modifier_B02R_2（Duration 1.91、ThinkInterval 0.1）十九發，實際二十發；比照本體 FC-071 照名稱改成十八發（待工作線 D 確認是否沿用）" },
          shuriken_interval = { 0.1, unit = "s", hidden = true, source = "kv:modifier_B02R_2 ThinkInterval" },
          volley_duration = { 1.91, unit = "s", hidden = true, source = "kv:modifier_B02R_2 Duration" },
          orbit_radius = { 420, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:251 420" },
          orbit_step = { 20, unit = "deg", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:247-251 每發角度 +20" },
          shuriken_speed = { 1000, unit = "speed", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:261" },
          damage = { 5, 10, 15, 20, unit = "dmg", damage_type = "pure", source = "kv:dmg" },
          ministun = { 0.05, unit = "s", hidden = true, source = "kv:modifier_B02R_3 Duration" },
        },
        text = {
          zh_TW = { name = "忍法．十八方手裏劍",
            body = "以陷阱纏住目標敵人 {root_time} 秒（期間被驅散會立刻重新纏住），並在目標四周依序出現 {shuriken_count} 枚手裏劍飛向目標，每枚造成 {damage} 純粹傷害並短暫暈眩；魔免目標照樣受傷與被纏住，但不會暈眩。手裏劍可以被躲開。" },
          en = { name = "Ninja Art: Eighteen-Direction Shuriken",
            body = "Roots the target enemy with a trap for {root_time} seconds (re-applied at once if dispelled) while {shuriken_count} shuriken appear one after another around it and fly at it, each dealing {damage} pure damage and a brief stun; spell-immune targets still take the damage and are rooted but not stunned. The shuriken can be dodged." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b02r_old_root", duration = "$root_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "手裏劍陷阱", body = "被陷阱纏住，無法移動；被驅散會立刻重新纏住。" },
              en = { name = "Shuriken Trap", body = "Rooted by a trap: cannot move; re-applied at once if dispelled." },
            },
            reveal = "public", states = { "root" }, cue = "b02r_old_trap",
            notes = "原作 modifier_B02R（B02R_old 的 Modifiers）：ROOTED、IsPurgable 1。" },
          { id = "b02r_old_volley", duration = "$volley_duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            notes = "原作 modifier_B02R_2：IsHidden、IsDebuff、沒寫 IsPurgable（暫定 basic）；存在期間每 0.1 秒發一枚手裏劍，被驅散或目標死亡就不再發射。" },
          { id = "b02r_old_ministun", duration = "$ministun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" },
              en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_B02R_3：IsStunDebuff、Duration 0.05；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b02r_old_vo", at = "caster" },
            { op = "let", name = "tgt", value = "target" },
            { op = "apply_status", status = "b02r_old_root", to = "tgt" },
            { op = "apply_status", status = "b02r_old_volley", to = "tgt" },
            { op = "repeat", interval = "$recheck", first = "$recheck", count = "$root_time / $recheck - 1", as = "n",
              actions = {
                { op = "if", cond = "not has_status(tgt, b02r_old_root)", actions = {
                  { op = "apply_status", status = "b02r_old_root", to = "tgt", duration = "$root_time - (n + 1) * $recheck" },
                } },
              } },
            { op = "repeat", interval = "$shuriken_interval", first = "0", count = "$shuriken_count", as = "k",
              ["while"] = "has_status(tgt, b02r_old_volley)", actions = {
                { op = "projectile", kind = "tracking", from = "polar(tgt, $orbit_step * (k + 1), $orbit_radius)", to = "tgt",
                  speed = "$shuriken_speed", dodgeable = true, cue = "b02r_old_shuriken",
                  on_hit = {
                    { op = "if", cond = "not it.is_magic_immune", actions = {
                      { op = "apply_status", status = "b02r_old_ministun", to = "it" },
                    } },
                    { op = "damage", to = "it", type = "pure", amount = "$damage" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B02.txt:964-982",
            text = "比照本體 FC-071（使用者 2026-10-29 決定全修：照名稱十八枚）。舊版 OnSpellStart 呼叫與本體同一個 B02R 腳本：OnCreated 一枚加上 Duration 1.91 內十九次 think，實際二十枚（100/200/300/400 總傷害）。舊版沒有原作說明；是否沿用 FC-071 請工作線 D 確認。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:222-234",
            text = "KV 傷害類型是魔法，但腳本以 AMHC:Damage 造成純粹傷害；魔免目標照樣受傷，只是不暈眩。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:208-219",
            text = "纏繞每 0.1 秒檢查一次，被驅散就以剩餘時間重新套上，等於無法被驅散。舊版纏繞時間（2.4～3.3 秒）比手裏劍齊射（1.91 秒）長，齊射結束後仍纏住一段時間。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B02.lua:209-211",
            text = "原作 sumt 以浮點累加 0.1，最後一次檢查可能落在剩餘時間約為零時；照設計意圖只檢查到剩餘一個間隔。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:243-252",
            text = "每枚手裏劍從目標當下位置外 420、角度每次加 20 度的地方出發（第一枚 20 度），是可躲避的追蹤投射物；發射點 dummy 改用 polar 位置。" },
          { policy = "no_effect", ref = "npc/B_Unified/B02.txt:1010-1019",
            text = "AbilitySpecial cast、cast_D 沒有被使用。" },
        },
        provenance = { "npc/B_Unified/B02.txt:915-1031", "vscripts/heroes/B_Unified/B02.lua:148-269" },
      },
      -------------------------------------------------------------------------
      { id = "b02_old_four_thunders", source_id = "B02T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 90, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, 350, 450, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          mark_duration = { 10, unit = "s", source = "kv:view_duration" },
          vision_tick = { 0.5, unit = "s", hidden = true, source = "kv:modifier_B02T_old ThinkInterval" },
          vision_radius = { 600, unit = "dist", source = "script:vscripts/heroes/B_Unified/B02.lua:513 AddFOWViewer 600" },
          vision_time = { 3, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:513 AddFOWViewer 3.0" },
          strike_delay = { 5, unit = "s", source = "kv:first_hit_delay" },
          stun = { 2, unit = "s", source = "kv:stun_duration" },
          bolt_count = { 4, 8, 12, unit = "count", source = "kv:wave_num" },
          bolt_stagger = { 0.07, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:546 Timer 0.07*(i-1)" },
          damage = { 300, unit = "dmg", damage_type = "magical", source = "kv:base_damage" },
          jump_bonus = { 5, unit = "pct", source = "kv:bonus_damage", note = "KV 寫比例 0.05：每跳加基礎傷害的 5%" },
          jump_max = { 100, unit = "count", source = "kv:jump_max" },
          jump_delay = { 0.3, unit = "s", source = "kv:jump_delay" },
          jump_radius = { 1000, unit = "dist", source = "kv:jump_radius" },
          jump_vision_radius = { 300, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:653 AddFOWViewer 300" },
          jump_vision_time = { 2, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B02.lua:653 AddFOWViewer 2.0" },
        },
        text = {
          zh_TW = { name = "忍法密傳．四方導雷",
            body = "在目標敵人四周召出 {bolt_count} 顆雷球並標記它 {mark_duration} 秒，期間持續提供目標周圍 {vision_radius} 的視野。{strike_delay} 秒後目標暈眩 {stun} 秒，每顆雷球依序打出一道閃電，各對目標造成 {damage} 魔法傷害；每道閃電之後每 {jump_delay} 秒跳向 {jump_radius} 內最近、這道閃電還沒打過的敵人（包含魔免），每跳傷害比上一跳多基礎的 {jump_bonus}，最多 {jump_max} 跳。閃電找不到下一個目標時，若至少跳過一次，會折返再打原目標一次。" },
          en = { name = "Secret Ninja Art: Four-Way Thunder",
            body = "Summons {bolt_count} thunder orbs around the target enemy and marks it for {mark_duration} seconds, granting vision of {vision_radius} around it. After {strike_delay} seconds the target is stunned for {stun} seconds and each orb fires a bolt in turn, each dealing {damage} magic damage to it; every {jump_delay} seconds each bolt then jumps to the nearest enemy within {jump_radius} it has not hit yet (spell-immune ones included), each jump dealing {jump_bonus} of the base more than the one before, up to {jump_max} jumps. When a bolt finds no next target after jumping at least once, it returns to strike the original target once more." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b02t_old_mark", duration = "$mark_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "四方導雷", body = "被雷球環繞，敵方持續獲得周圍的視野。" },
              en = { name = "Four-Way Thunder", body = "Surrounded by thunder orbs; enemies keep vision around this unit." },
            },
            reveal = "public", cue = "b02t_old_ball",
            triggers = {
              periodic = { interval = "$vision_tick", actions = {
                { op = "reveal", at = "position(self)", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
              } },
            },
            notes = "原作 modifier_B02T_old：IsDebuff 0、不隱藏、Duration %view_duration、ThinkInterval 0.5 跑 B02T_old_think（AddFOWViewer 600、3 秒）；OnUnitMoved 只移動雷球特效。沒寫 IsPurgable，暫定 basic。" },
          { id = "b02t_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" },
              en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" }, cue = "b02t_old_lightning",
            notes = "原作 modifier_B02T_old_stun：IsStunDebuff、Duration %stun_duration；沒寫 IsPurgable（暫定 basic），不在 ultimate_modifiers。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b02t_old_vo", at = "caster" },
            { op = "let", name = "tgt", value = "target" },
            { op = "apply_status", status = "b02t_old_mark", to = "tgt" },
            { op = "delay", time = "$strike_delay", capture = { "tgt" }, actions = {
              { op = "if", cond = "alive(tgt)", actions = {
                { op = "apply_status", status = "b02t_old_stun", to = "tgt" },
                { op = "repeat", interval = "$bolt_stagger", first = "0", count = "$bolt_count", ["while"] = "alive(tgt)",
                  actions = {
                    { op = "cue", id = "b02t_old_link", at = "tgt" },
                    { op = "damage", to = "tgt", type = "magical", amount = "$damage" },
                    { op = "chain", from = "tgt", first = "tgt", jumps = "$jump_max - 1", radius = "$jump_radius",
                      interval = "$jump_delay", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
                      flags = { "include_magic_immune" }, order = "nearest", cue = "b02t_old_link",
                      actions = {
                        { op = "if", cond = "jump > 1", actions = {
                          { op = "reveal", at = "position(it)", radius = "$jump_vision_radius", duration = "$jump_vision_time", teams = "own" },
                          { op = "damage", to = "it", type = "magical", amount = "$damage * (1 + (jump - 1) * $jump_bonus)" },
                        } },
                      },
                      -- B02.lua:663-695: the bolt that can jump no further strikes the first target once more
                      -- (the no-target branch only after a jump; the used-up branch always has jumped)
                      on_end = {
                        { op = "if", cond = "jump > 1 and alive(tgt)", actions = {
                          { op = "cue", id = "b02t_old_final", at = "tgt" },
                          { op = "damage", to = "tgt", type = "magical", amount = "$damage * (1 + jump * $jump_bonus)" },
                        } },
                      } },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:534-559",
            text = "每顆雷球除了開出一串彈跳閃電，還直接對原目標造成一次基礎傷害：雷球數 4/8/12，原目標單是起手就受 1200/2400/3600 魔法傷害（魔免歸零，由規則層處理）。暈眩只在第一顆前套一次。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:597-616",
            text = "各串以「最後一個 target[i] == 目前目標」找自己的 instance，多串同時從同一目標出發時會互換 instance 編號，但每個編號的命中表與跳數一致，推定結果等同各串獨立（chain 每串有自己的命中表）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:620-660",
            text = "彈跳以當下目標位置找 jump_radius 內距離最近、這串沒打過的單位，旗標沿用技能的 MAGIC_IMMUNE_ENEMIES、沒有 FOW_VISIBLE：看不見的敵人也會被跳到；新目標得到 300 範圍、2 秒的視野。第 k 跳傷害為 基礎 ×（1 + k × 加成）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B02.lua:475-480",
            text = "延遲雷擊檢查的是 IsValidEntity（英雄死亡後實體仍存在）；目標已死時暈眩與傷害都打不到，推定等同不觸發，寫成 alive(tgt)。若目標在五秒內死亡又復活，原作會打在復活後的英雄身上，v3 也一樣（alive 為真）。" },
          { policy = "as_source", ref = "npc/B_Unified/B02.txt:1080-1104",
            text = "標記 modifier_B02T_old 被驅散只會停掉視野；延遲雷擊是 OnCreated 建立的 Timer，照常落下。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B02.lua:487-505",
            text = "雷球在目標上方半徑 300 的環上（ability.radius = 300）、閃電連線與最後折返特效，都只是表現，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B02.txt:1031-1200", "vscripts/heroes/B_Unified/B02.lua:467-706" },
      },
    },
  }
end
