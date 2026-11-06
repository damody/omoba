-- B23 伊勢姬（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B23"]["11"] = "WERDT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:302）；W/E/R/D/T → Q/W/E/D/R。
-- B23D_old 是 NOT_LEARNABLE，heromap_autoskill["B23"]["11"] = "D"（addon_game_mode.lua:405）出生時自動學一級，
-- 之後每次學 B23T_old 時設為 B23T_old 等級 + 1（B23.lua:391-393；hero.auto_learn + on_upgrade）。
-- 原作舊版有繁中說明（resource/addon_tchinese.txt:3230-3239），與 KV、腳本的差異記在 quirks。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B23_precache 屬規則層或資產，不列入。
-- W／E／R 與本體共用同一批腳本函式（B23W、B23E_*、B23R），寫法照本體 B23，只有 E 的緩速改成 KV 的 Properties。
-- 原作以 caster.B23D_ghostTable（Lua 表）記錄亡魂；v3 以 unit_state 計數加「主人是伊勢姬的 b23d_old_ghost」選取表達。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local function ghosts()
    return { from = "caster", shape = "global", team = "ally", types = { "all" },
      flags = { "include_magic_immune", "include_invulnerable" },
      where = "is_template(it, b23d_old_ghost) and it.owner == caster" }
  end
  return {
    schema_version = 3, kind = "hero", id = "B23_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "B23",
      text = { zh_TW = { name = "伊勢姬（舊版）" }, en = { name = "Ise-hime (Old)" } },
      auto_learn = { "b23_old_undead_pact" },
    },
    unit_state = {
      b23_old_ghost_count = { type = "int", init = "0", reveal = "public", scope = "hero",
        note = "原作 #caster.B23D_ghostTable；也顯示在減傷狀態的層數上，所以公開。" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b23_old_undead_pact", source_id = "B23D_old", slot = "D", kind = "active", max_level = 4, learnable = false,
        tags = { "basic", "summon" },
        values = {
          cooldown = { 20, 18, 16, 14, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B23.txt:589" },
          mana_cost = { 110, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B23.txt:590" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B23.txt:588" },
          count = { 4, 5, 6, 7, unit = "count", source = "kv:B23D_count@npc/B_Unified/B23.txt:695" },
          hp_base = { 700, unit = "hp", source = "script:vscripts/heroes/B_Unified/B23.lua:53 SetBaseMaxHealth(700 + 50 * level)" },
          hp_per_level = { 50, unit = "hp", source = "script:vscripts/heroes/B_Unified/B23.lua:53" },
          dmg_min_base = { 52, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B23.lua:51 SetBaseDamageMin(52 + 6 * level)" },
          dmg_max_base = { 58, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B23.lua:52 SetBaseDamageMax(58 + 6 * level)" },
          dmg_per_level = { 6, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B23.lua:51-52" },
          damage_reduction = { 10, unit = "pct", kv = { -10 },
            source = "kv:modifier_B23D_damageReduction INCOMING_DAMAGE_PERCENTAGE@npc/B_Unified/B23.txt:660",
            source_differs = "KV 以負值表示減傷，v3 以正值表示；原作說明寫每隻亡魂 14%，實際固定 10%" },
          mana_loss = { 15, unit = "pct", source = "script:vscripts/heroes/B_Unified/B23.lua:113 GetMaxMana() * 0.15" },
          leash = { 1600, unit = "dist", source = "script:vscripts/heroes/B_Unified/B23.lua:140,158 Length() >= 1600" },
          leash_min = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B23.lua:141,159 RandomInt(100,200)" },
          leash_max = { 200, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B23.lua:141,159 RandomInt(100,200)" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B23.lua:55,145,163 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "亡靈之契",
            body = "在伊勢姬身旁召喚一隻受她控制的{@b23d_old_ghost}，最多同時 {count} 隻。亡魂生命為 {hp_base} 加每英雄等級 {hp_per_level}，攻擊力為 {dmg_min_base}～{dmg_max_base} 加每英雄等級 {dmg_per_level}（召喚當下決定）。擁有亡魂時伊勢姬受到的傷害降低 {damage_reduction}。每隻亡魂死亡時，伊勢姬失去最大法力 {mana_loss} 的法力；伊勢姬死亡時所有亡魂一起死亡。亡魂與伊勢姬相距 {leash} 以上時會被傳回她身邊。天生技能，等級跟著{@b23_old_mind_control}提升。" },
          en = { name = "Pact of the Dead",
            body = "Summons a controllable {@b23d_old_ghost} beside Ise-hime, up to {count} at once. Ghosts have {hp_base} health plus {hp_per_level} per hero level and {dmg_min_base}–{dmg_max_base} attack damage plus {dmg_per_level} per hero level, fixed when summoned. While she has ghosts, Ise-hime takes {damage_reduction} less damage. Whenever a ghost dies she loses mana equal to {mana_loss} of her maximum mana; when she dies, all ghosts die with her. Ghosts {leash} or farther from her are teleported back to her side. Innate; its level rises with {@b23_old_mind_control}." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        passive = {
          reveal = "public", visibility = "hidden",
          triggers = {
            on_moved = { {
              op = "for_each", as = "g",
              select = { from = "caster", shape = "global", team = "ally", types = { "all" },
                flags = { "include_magic_immune", "include_invulnerable" },
                where = "is_template(it, b23d_old_ghost) and it.owner == self and distance(it, self) >= $leash" },
              actions = {
                { op = "teleport", who = "g", to = "random_point(self, $leash_min, $leash_max)" },
                { op = "cue", id = "b23d_old_ghost_blink", at = "g" },
                { op = "apply_status", status = "b23d_old_phase", to = "g", source = "g" },
                { op = "issue_order", who = "g", order = "stop" },
              } } },
          },
        },
        statuses = {
          { id = "b23d_old_damage_reduction", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff", -- 同 FC-107（D13）
            reveal = "public",
            stats = { incoming_damage_pct = "-$damage_reduction" },
            triggers = {
              on_death = { {
                op = "for_each", select = ghosts(), actions = { { op = "kill", to = "it" } } } },
            },
            text = {
              zh_TW = { name = "亡靈之契", body = "擁有{@b23d_old_ghost}：受到的傷害降低 {damage_reduction}；層數是亡魂數，只是顯示。伊勢姬死亡時所有亡魂一起死亡。" },
              en = { name = "Pact of the Dead", body = "Has {@b23d_old_ghost}: takes {damage_reduction} less damage; the stack count shows the number of ghosts and is display only. All ghosts die when Ise-hime dies." },
            },
            notes = "原作 modifier_B23D_damageReduction（B23D_old 的 Modifiers 區塊）：層數 = 亡魂數，但 datadriven 屬性不乘層數，減傷固定。沒寫 IsPurgable，照本體 FC-107 修正為不可驅散。" },
          { id = "b23d_old_ghost_link", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_death = {
                { op = "set_state", var = "b23_old_ghost_count", on = "caster", value = "state.b23_old_ghost_count - 1" },
                { op = "if", cond = "has_status(caster, b23d_old_damage_reduction)", actions = {
                  { op = "set_display_stacks", status = "b23d_old_damage_reduction", to = "caster", count = "state.b23_old_ghost_count" },
                  { op = "if", cond = "state.b23_old_ghost_count <= 0", actions = {
                    { op = "remove_status", from = "caster", status = "b23d_old_damage_reduction" },
                  } },
                  { op = "set_mana", to = "caster", value = "max(0, caster.mana - caster.max_mana * $mana_loss)" },
                } },
                { op = "cue", id = "b23d_old_ghost_death", at = "self" },
              },
              on_moved = { {
                op = "if", cond = "distance(self, caster) >= $leash", actions = {
                  { op = "teleport", who = "self", to = "random_point(caster, $leash_min, $leash_max)" },
                  { op = "cue", id = "b23d_old_ghost_blink", at = "self" },
                  { op = "apply_status", status = "b23d_old_phase", to = "self", source = "self" },
                  { op = "issue_order", who = "self", order = "stop" },
                } } },
            },
            notes = "原作 modifier_B23D_diedTrigger 與 modifier_B23D_distanceDetector_ghost（IGNORE_INVULNERABLE、隱藏），施加者是伊勢姬。" },
          { id = "b23d_old_phase", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "原作 modifier_phased 0.1 秒，避免出生或傳送時卡位。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b23d_old_cast_sound", at = "caster" },
            { op = "if", cond = "state.b23_old_ghost_count < $count", actions = {
              { op = "create_unit", template = "b23d_old_ghost", at = "position(caster)", owner = "caster", team = "caster",
                control = "owner", as = "g",
                overrides = {
                  hp = "$hp_base + $hp_per_level * caster.level",
                  attack_damage = "$dmg_min_base + $dmg_per_level * caster.level",
                  attack_damage_max = "$dmg_max_base + $dmg_per_level * caster.level",
                } },
              { op = "apply_status", status = "b23d_old_phase", to = "g", source = "caster" },
              { op = "set_state", var = "b23_old_ghost_count", value = "state.b23_old_ghost_count + 1" },
              { op = "apply_status", status = "b23d_old_damage_reduction", to = "caster" },
              { op = "set_display_stacks", status = "b23d_old_damage_reduction", to = "caster", count = "state.b23_old_ghost_count" },
              { op = "apply_status", status = "b23d_old_ghost_link", to = "g" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:40-65",
            text = "和本體不同：舊版不需要屍體，亡魂直接在伊勢姬所在位置（FindClearSpace）召喚；OnSpellStart 的 Target 區塊（600 內含屍體的單位）沒有被使用，也沒有前搖的屍體檢查。冷卻 20/18/16/14（本體 3 秒）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:47",
            text = "亡魂已滿時照常施放、扣魔、進冷卻，只是不召喚。" },
          { policy = "as_source", ref = "resource/addon_tchinese.txt:3231",
            text = "原作說明寫「每存在一個亡魂伊勢姬所受到的傷害減少 14%」，實際是 INCOMING_DAMAGE_PERCENTAGE -10，datadriven 屬性不乘層數，一隻與七隻亡魂都是固定 10%。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B23.lua:106-118",
            text = "同本體 FC-107（D13 修正：減傷狀態改成不可驅散，亡魂連動不會被破壞），舊版用的是同一段腳本。原作：減傷狀態可被一般驅散（未寫 IsPurgable），被驅散後亡魂死亡時 FindModifierByNameAndCaster 回傳 nil 報錯：不扣法力；伊勢姬死亡時也不再帶走亡魂。是否沿用請工作線 D 確認。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:113-118",
            text = "伊勢姬死亡帶走亡魂時，每隻亡魂的死亡照樣扣她的法力（她已死，影響只在復活前的法力值）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:131-167",
            text = "傳回判定在伊勢姬或亡魂移動時觸發（OnUnitMoved），傳送後亡魂停止目前命令。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:391-393",
            text = "等級跟著{@b23_old_mind_control}：學大招時設為大招等級 + 1。原作說明寫「提升英靈庇佑技能等級可強化此技能」，舊版的大招其實是心靈控制。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B23.lua:67-75",
            text = "被動 OnCreated 給自己 1600 範圍的 spell hint（永久顯示傳回距離），屬呈現。" },
        },
        provenance = { "npc/B_Unified/B23.txt:580-698", "vscripts/heroes/B_Unified/B23.lua:40-167",
          "vscripts/addon_game_mode.lua:302", "vscripts/addon_game_mode.lua:405", "resource/addon_tchinese.txt:3230-3231",
          "npc/npc_units_custom_load.txt:2071-2136" },
      },
      -------------------------------------------------------------------------
      { id = "b23_old_spirit_gathering", source_id = "B23W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B23.txt:537" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B23.txt:538" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B23.txt:535" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B23.txt:534" },
          radius = { 450, unit = "dist", source = "kv:B23W_radius@npc/B_Unified/B23.txt:555" },
          duration = { 30, unit = "s", source = "kv:B23W_duration@npc/B_Unified/B23.txt:550" },
          fire_damage = { 80, 110, 140, 170, unit = "dmg", damage_type = "magical", source = "kv:B23W_fireDamage@npc/B_Unified/B23.txt:560" },
          fire_interval = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B23.lua:198-215 return 1" },
          fire_ticks = { expr = "$duration / $fire_interval", unit = "count", hidden = true, source = "derived",
            note = "script:vscripts/heroes/B_Unified/B23.lua:195-202：count 從 1 起，count > duration + 0.1 停止，共 30 跳（第 0～29 秒）" },
          shock_damage = { 55, 80, 105, 130, unit = "dmg", damage_type = "magical", source = "kv:B23W_shockDamage@npc/B_Unified/B23.txt:545" },
          ghost_min = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B23.lua:219 RandomInt(100,200)" },
          ghost_max = { 200, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B23.lua:219 RandomInt(100,200)" },
        },
        text = {
          zh_TW = { name = "聚靈咒",
            body = "在目標地點留下靈火 {duration} 秒，每 {fire_interval} 秒對 {radius} 範圍內的敵方英雄與一般單位造成 {fire_damage} 魔法傷害。施放時所有{@b23d_old_ghost}聚集到目標地點附近，每隻亡魂各對範圍內的敵人造成一次 {shock_damage} 魔法傷害。" },
          en = { name = "Spirit Gathering",
            body = "Leaves spirit fire at the target point for {duration} seconds, dealing {fire_damage} magic damage to enemy heroes and units within {radius} every {fire_interval} seconds. On cast, every {@b23d_old_ghost} gathers next to the target point and each deals {shock_damage} magic damage once to enemies in the area." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "b23w_old_cast_sound", at = "at" },
            { op = "cue", id = "b23w_old_dark_rift", at = "at", duration = "$duration" },
            { op = "repeat", count = "$fire_ticks", interval = "$fire_interval", first = "0", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                actions = { { op = "damage", to = "it", type = "magical", amount = "$fire_damage", require_source_alive = false } } },
            } },
            { op = "for_each", as = "g", select = ghosts(), actions = {
              { op = "teleport", who = "g", to = "random_point(at, $ghost_min, $ghost_max)" },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                actions = { { op = "damage", to = "it", type = "magical", amount = "$shock_damage", require_source_alive = false } } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B23.txt:558-571",
            text = "舊版 KV 的 RunScript 呼叫本體同一個函式 B23W，數值也與本體相同；行為照本體。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:195-215",
            text = "靈火在施放當下（第 0 秒）就跳第一次，之後每秒一次共 30 跳；ApplyDamage 帶 ability，不經 AMHC。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:217-232",
            text = "亡魂傷害由伊勢姬造成、以目標地點為圓心，與亡魂傳送後的位置無關；N 隻亡魂就是 N 次傷害。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B23.lua:234-248",
            text = "「亡魂攻擊最近英雄」只建了 order 表，沒有呼叫 ExecuteOrderFromTable，沒有效果。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B23.lua:175-178",
            text = "npc_dummy_unit 視野 0、30 秒後 modifier_kill，只承載特效，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B23.txt:519-571", "vscripts/heroes/B_Unified/B23.lua:169-249", "resource/addon_tchinese.txt:3232-3233" },
      },
      -------------------------------------------------------------------------
      { id = "b23_old_living_soul_shock", source_id = "B23E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B23.txt:711" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B23.txt:712" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B23.txt:713" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B23.txt:714" },
          damage = { 35, 50, 65, 80, unit = "dmg", damage_type = "magical", source = "kv:B23E_damage@npc/B_Unified/B23.txt:760" },
          duration = { 3, 5, 7, 9, unit = "s", source = "kv:B23E_duration@npc/B_Unified/B23.txt:765" },
          slow = { 35, unit = "pct", kv = { -35 },
            source = "kv:modifier_B23E_shock MOVESPEED_BONUS_PERCENTAGE@npc/B_Unified/B23.txt:778",
            source_differs = "KV 以負值表示減速，v3 以正值表示" },
          projectile_speed = { 1300, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/B_Unified/B23.txt:724" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius@npc/B_Unified/B23.txt:725" },
        },
        text = {
          zh_TW = { name = "生靈衝擊",
            body = "向目標敵人射出一道靈魂飛彈，每隻{@b23d_old_ghost}也各射出一道。每道命中造成 {damage} 魔法傷害，並各自降低 {slow} 移動速度 {duration} 秒，可以疊加。" },
          en = { name = "Living Soul Shock",
            body = "Fires a soul bolt at the target enemy, and every {@b23d_old_ghost} fires one too. Each bolt deals {damage} magic damage and separately reduces movement speed by {slow} for {duration} seconds; the slows stack." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b23e_old_shock", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "生靈衝擊", body = "移動速度降低 {slow}；每道命中各自計算。" },
              en = { name = "Living Soul Shock", body = "Movement speed reduced by {slow}; each hit counts separately." },
            },
            reveal = "public", stats = { move_speed_pct = "-$slow" },
            notes = "原作 modifier_B23E_shock（B23E_old 的 Modifiers 區塊）：IsDebuff、MODIFIER_ATTRIBUTE_MULTIPLE、Duration %B23E_duration、MOVESPEED_BONUS_PERCENTAGE -35；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b23e_old_cast_sound", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, vision = "$projectile_vision", cue = "b23e_old_bolt",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "apply_status", status = "b23e_old_shock", to = "it" },
              } },
            { op = "for_each", as = "g", select = ghosts(), actions = {
              { op = "projectile", kind = "tracking", from = "g", to = "target", speed = "$projectile_speed",
                dodgeable = true, vision = "$projectile_vision", cue = "b23e_old_bolt",
                on_hit = {
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                  { op = "apply_status", status = "b23e_old_shock", to = "it" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B23.txt:769-781",
            text = "和本體不同：舊版的 modifier_B23E_shock 直接用 Properties MOVESPEED_BONUS_PERCENTAGE -35（不經 nobu 緩速池），又是 MULTIPLE：每道命中各自一個實例、各自計時，減速相加（四道就是 -140%，由引擎的最低移速擋住）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:252-291",
            text = "亡魂的飛彈從亡魂位置出發、追蹤同一目標，傷害來源仍是伊勢姬；可被閃避。命中是 ApplyDamage（帶 ability，不經 AMHC）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B23.lua:270",
            text = "亡魂飛彈的 iUnitTargetType 誤填成 GetAbilityDamageType()；追蹤投射物不依此篩選，無實際影響。DeleteOnHit 0 對追蹤投射物沒有差別。" },
        },
        provenance = { "npc/B_Unified/B23.txt:700-783", "vscripts/heroes/B_Unified/B23.lua:251-291", "resource/addon_tchinese.txt:3234-3235" },
      },
      -------------------------------------------------------------------------
      { id = "b23_old_soul_feast", source_id = "B23R_old", slot = "E", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 22, 20, 18, 16, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B23.txt:796" },
          mana_cost = { 130, 155, 180, 205, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B23.txt:797" },
          channel_time = { 4, unit = "s", source = "kv:AbilityChannelTime@npc/B_Unified/B23.txt:799" },
          radius = { 800, unit = "dist", source = "kv:AOERadius@npc/B_Unified/B23.txt:798", note = "vscripts/heroes/B_Unified/B23.lua:306 FindUnitsInRadius 也是 800" },
          heal = { 15, 21, 27, 33, unit = "heal", source = "kv:B23R_heal@npc/B_Unified/B23.txt:859" },
          tick = { 0.2, unit = "s", source = "script:vscripts/heroes/B_Unified/B23.lua:316 return 0.2" },
          immune_duration = { 4, unit = "s", source = "kv:modifier_B23R_magicImmune Duration@npc/B_Unified/B23.txt:836" },
        },
        text = {
          zh_TW = { name = "魂饗",
            body = "引導：伊勢姬與所有{@b23d_old_ghost}獲得魔法免疫 {immune_duration} 秒；引導期間每 {tick} 秒為 {radius} 範圍內的友方英雄與單位（含自己與亡魂）恢復 {heal} 生命。引導結束時伊勢姬與亡魂都失去魔法免疫。" },
          en = { name = "Soul Feast",
            body = "Channel: Ise-hime and every {@b23d_old_ghost} become spell immune for {immune_duration} seconds; every {tick} seconds while channeling, allied heroes and units within {radius} (including herself and her ghosts) are healed for {heal}. When the channel ends, Ise-hime and her ghosts lose spell immunity." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b23r_old_magic_immune", duration = "$immune_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "魂饗", body = "魔法免疫。" },
              en = { name = "Soul Feast", body = "Spell immune." },
            },
            reveal = "public", states = { "magic_immune" },
            notes = "原作 modifier_B23R_magicImmune（B23R_old 的 Modifiers 區塊）：datadriven，沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b23r_old_heal_sound", at = "caster" },
            { op = "cue", id = "b23r_old_victory_aura", attach = "caster", duration = "$channel_time" },
            { op = "apply_status", status = "b23r_old_magic_immune", to = "caster" },
            { op = "for_each", select = ghosts(), actions = {
              { op = "apply_status", status = "b23r_old_magic_immune", to = "it" },
            } },
            { op = "call", routine = "b23r_old_heal_tick", args = {} },
          },
          on_channel_tick = { interval = "$tick", actions = {
            { op = "call", routine = "b23r_old_heal_tick", args = {} },
          } },
          on_channel_end = {
            { op = "call", routine = "b23r_old_heal_tick", args = {} },
            { op = "cue", id = "b23r_old_victory_aura", attach = "caster", stop = true },
            { op = "for_each", select = ghosts(), actions = {
              { op = "remove_status", from = "it", status = "b23r_old_magic_immune" },
            } },
            -- 同本體 FC-087（D13）：引導結束時也移除伊勢姬自己的魔免
            { op = "remove_status", from = "caster", status = "b23r_old_magic_immune" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B23.lua:321-327",
            text = "同本體 FC-087（D13 修正：引導結束時也移除伊勢姬自己的魔免），舊版用的是同一段腳本。原作：中斷處理只移除亡魂的魔法免疫，沒有移除伊勢姬自己的：施放後立刻取消引導，仍以耗魔換得完整的魔法免疫時間。是否沿用請工作線 D 確認。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B23.lua:305-318",
            text = "原作 Timer 先治療再檢查是否仍在引導，所以施放當下一跳、之後每 0.2 秒一跳，引導結束（或被打斷）後的那一跳仍會治療。照設計意圖寫成：施放一跳、引導每跳一次、結束時一跳。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:306-307",
            text = "選取友方時 Dota 不排除魔免友軍，所以魔免的自己與亡魂也被治療；選取寫 include_magic_immune。原作說明「每 0.2 秒回復」與腳本相符。" },
          { policy = "note", ref = "npc/B_Unified/B23.txt:843-850",
            text = "modifier_B23R_castAnimation 只有特效與 ACT_DOTA_VICTORY 動作，改用 cue。OnChannelInterrupted 在 KV 已註解，結束處理由腳本 Timer 呼叫。" },
        },
        provenance = { "npc/B_Unified/B23.txt:784-862", "vscripts/heroes/B_Unified/B23.lua:293-327", "resource/addon_tchinese.txt:3236-3237" },
      },
      -------------------------------------------------------------------------
      { id = "b23_old_mind_control", source_id = "B23T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 150, 140, 130, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B23.txt:876" },
          mana_cost = { 300, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B23.txt:877" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B23.txt:878" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；DOTA_ABILITY_TYPE_ULTIMATE（npc/B_Unified/B23.txt:872）預設 6/12/18" },
          duration = { 5, 7, 9, unit = "s", kv = { 7, 10, 13 },
            source = "kv:duration@npc/B_Unified/B23.txt:922",
            source_differs = "原作說明（resource/addon_tchinese.txt:3239）寫 7/10/13 秒；腳本讀 KV duration 5/7/9" },
          immune_ratio = { 0.5, unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B23.lua:402-404 魔免目標 duration * 0.5" },
          check_interval = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B23.lua:413-425 Timers 每 0.1 秒檢查" },
          rescue_invuln = { 0.15, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B23.lua:439-441 modifier_invulnerable 0.15" },
          finish_damage = { 100, unit = "dmg", damage_type = "pure",
            source = "script:vscripts/heroes/B_Unified/B23.lua:442 AMHC:Damage 100 純粹" },
        },
        text = {
          zh_TW = { name = "心靈控制",
            body = "控制一個敵方單位 {duration} 秒，期間它屬於伊勢姬的隊伍並由她操作；對魔免單位時間減半。伊勢姬死亡時控制解除。被控制的單位受到致命傷害時，這一擊不生效、生命保留一點、控制解除並短暫無敵，隨後受到伊勢姬 {finish_damage} 純粹傷害。" },
          en = { name = "Mind Control",
            body = "Takes control of an enemy unit for {duration} seconds: it joins Ise-hime's team and she commands it; the duration is halved against spell-immune units. The control ends if Ise-hime dies. If the controlled unit takes a lethal hit, that hit is negated and it is left at one health, the control ends, it becomes briefly invulnerable, and then takes {finish_damage} pure damage from Ise-hime." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, flags = { "include_magic_immune" } },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b23t_old_control", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "心靈控制", body = "被伊勢姬控制，暫時屬於她的隊伍。" },
              en = { name = "Mind Control", body = "Controlled by Ise-hime and temporarily on her team." },
            },
            reveal = "public", cue = "b23t_old_control",
            triggers = {
              periodic = { interval = "$check_interval", actions = {
                { op = "if", cond = "not alive(caster)", actions = {
                  { op = "remove_status", from = "self", status = "b23t_old_control" },
                } },
              } },
              -- FC-024（D13 比照）：致死傷害在結算前判定並取消，先解除控制再由伊勢姬擊殺（原作在扣血後比較，救不到致死一擊）
              on_take_damage_pre = {
                filter = "alive(self) and event.damage.lethal",
                actions = {
                  { op = "modify_damage", cancel = true },
                  { op = "set_health", to = "self", value = "1" },
                  { op = "remove_status", from = "self", status = "b23t_old_control" },
                  { op = "apply_status", status = "b23t_old_rescue", to = "self", source = "self" },
                  { op = "let", name = "victim", value = "self" },
                  { op = "let", name = "controller", value = "caster" },
                  { op = "delay", time = "$rescue_invuln", capture = { "victim", "controller" }, actions = {
                    { op = "remove_status", from = "victim", status = "b23t_old_rescue" },
                    { op = "damage", to = "victim", source = "controller", type = "pure", amount = "$finish_damage" },
                  } },
                } },
            },
            notes = "原作 modifier_B23T_old：Passive 0、IsHidden 0、IsDebuff 0；OnTakeDamage 執行 B23T_old_OnTakeDamage（依 FC-024 改成結算前判定）。沒寫 IsPurgable，暫定 basic；驅散只移除 modifier，換隊由 Timer 檢查 target.B23T_old 決定（見 quirks）。" },
          { id = "b23t_old_rescue", duration = "$rescue_invuln", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "無敵", body = "心靈控制解除時短暫無敵。" },
              en = { name = "Invulnerable", body = "Briefly invulnerable as the mind control breaks." },
            },
            reveal = "public", states = { "invulnerable" },
            notes = "原作引擎內建 modifier_invulnerable 0.15 秒（施加者是單位自己）。" },
        },
        events = {
          on_upgrade = {
            { op = "set_ability_level", of = "caster", ability = "b23_old_undead_pact", level = "ability.level + 1" },
          },
          on_cast = {
            { op = "apply_status", status = "b23t_old_control", to = "target",
              duration = "select(target.is_magic_immune, $duration * $immune_ratio, $duration)" },
          },
        },
        coverage = "pending", confidence = "low",
        pending = {
          { what = "把目標暫時換到施法者隊伍並交給施法者操作，狀態結束時換回原隊伍與原控制者",
            why = "vscripts/heroes/B_Unified/B23.lua:405-425：SetControllableByPlayer(施法者玩家) 與 SetTeam(施法者隊伍)，0.1 秒 Timer 在施法者死亡、時間到或 target.B23T_old 被清除時 SetTeam(原隊伍)、SetControllableByPlayer(原玩家)",
            gap = "新增 action 或狀態欄位（例如狀態 `control = { team = \"caster\", controller = \"caster\" }`，結束時還原），選取、仇恨與擊殺歸屬依新隊伍計算" },
        },
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B23.lua:428-447",
            text = "FC-024（比照本體 D13 修正：致死傷害在結算前判定，取消這一筆、生命設為 1、解除控制、0.15 秒無敵，之後由伊勢姬造成 100 純粹傷害擊殺）。原作：OnTakeDamage 在扣血之後以 damage > GetHealth() 比較：致死的一擊已經殺死單位（仍在伊勢姬隊伍時死亡），救不到；反而是大於剩餘生命的非致死重擊（約超過受擊前生命一半）才觸發：生命設為 1、解除控制、0.15 秒無敵，之後受到伊勢姬 100 純粹傷害（AMHC，伊勢姬已死則不傷害），通常因此死亡、擊殺算伊勢姬。設計意圖推定是「致命傷害時先還給原隊伍再由伊勢姬擊殺」，改用 on_take_damage_pre 以 event.damage.lethal 判定並 modify_damage cancel。換隊與交還控制仍待詞彙（見 pending），所以「還給原隊伍」目前只以移除控制狀態表達。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:413-425",
            text = "控制結束的條件是 Timer 每 0.1 秒檢查：伊勢姬死亡、累計時間到、或 target.B23T_old 被清除（只有上面的重擊處理會清）。modifier 被驅散不會結束控制（換隊照舊，直到時間到）；資料以狀態表達控制期間，驅散差異記在這裡。" },
          { policy = "as_source", ref = "resource/addon_tchinese.txt:3239",
            text = "原作說明寫 7/10/13 秒，實際 KV duration 5/7/9；說明寫「無法操作中立野怪」，但 KV 目標是 ENEMY、HERO | BASIC，腳本也沒有排除中立，推定中立野怪照樣可以被控制（未實測）。可以指定魔免敵人（MAGIC_IMMUNE_ENEMIES），時間減半。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:391-393",
            text = "學大招時把{@b23_old_undead_pact}設為大招等級 + 1。" },
          { policy = "note", text = "控制特效（b23t_old 粒子，CP2 = 時間）只是呈現，改用狀態 cue。KV 沒寫 AbilityCastPoint 以外的前搖（0）。" },
        },
        provenance = { "npc/B_Unified/B23.txt:863-925", "vscripts/heroes/B_Unified/B23.lua:391-447", "resource/addon_tchinese.txt:3238-3239" },
      },
    },
    routines = {
      b23r_old_heal_tick = {
        params = {},
        actions = {
          { op = "for_each",
            select = { from = "caster", shape = "circle", radius = "$b23_old_soul_feast.radius", team = "ally",
              types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
            actions = { { op = "heal", to = "it", amount = "$b23_old_soul_feast.heal", source = "caster" } } },
        },
      },
    },
    unit_templates = {
      { id = "b23d_old_ghost", source_id = "B23D_ghost",
        values = {
          hp = { 700, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:2103" },
          hp_regen = { 10, unit = "per_s", source = "kv:StatusHealthRegen@npc/npc_units_custom_load.txt:2104" },
          armor = { 0, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:2089" },
          move_speed = { 300, unit = "move_speed", source = "kv:MovementSpeed@npc/npc_units_custom_load.txt:2100" },
          turn_rate = { 1.5, unit = "ratio", source = "kv:MovementTurnRate@npc/npc_units_custom_load.txt:2101" },
          attack_min = { 55, unit = "dmg", source = "kv:AttackDamageMin@npc/npc_units_custom_load.txt:2081" },
          attack_max = { 61, unit = "dmg", source = "kv:AttackDamageMax@npc/npc_units_custom_load.txt:2082" },
          attack_interval = { 1.9, unit = "s", source = "kv:AttackRate@npc/npc_units_custom_load.txt:2084" },
          attack_point = { 0.1, unit = "s", source = "kv:AttackAnimationPoint@npc/npc_units_custom_load.txt:2085" },
          attack_range = { 520, unit = "dist", source = "kv:AttackRange@npc/npc_units_custom_load.txt:2087" },
          acquisition_range = { 1200, unit = "dist", source = "kv:AttackAcquisitionRange@npc/npc_units_custom_load.txt:2086" },
          projectile_speed = { 1400, unit = "speed", source = "kv:ProjectileSpeed@npc/npc_units_custom_load.txt:2093" },
          vision_day = { 1800, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:2111" },
          vision_night = { 1800, unit = "dist", source = "kv:VisionNighttimeRange@npc/npc_units_custom_load.txt:2112" },
          bounty_gold = { 10, unit = "gold", source = "kv:BountyGoldMin/Max@npc/npc_units_custom_load.txt:2095-2096" },
          bounty_xp = { 5, unit = "xp", source = "kv:BountyXP@npc/npc_units_custom_load.txt:2097" },
        },
        text = { zh_TW = { name = "亡魂" }, en = { name = "Ghost" } },
        stats = { hp = "$hp", hp_regen = "$hp_regen", armor = "$armor", move_speed = "$move_speed" },
        attack = { kind = "ranged", range = "$attack_range", interval = "$attack_interval", damage = "$attack_min",
          damage_max = "$attack_max", damage_type = "physical", attack_point = "$attack_point",
          projectile_speed = "$projectile_speed" },
        acquisition_range = "$acquisition_range", turn_rate = "$turn_rate",
        vision = { day = "$vision_day", night = "$vision_night" },
        bounty = { gold = "$bounty_gold", xp = "$bounty_xp" },
        ai = "owner_controlled", tags = { "summon" },
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B23.lua:48-53",
            text = "召喚時以伊勢姬等級覆寫生命與攻擊力（KV 的 700／55～61 只是基底）；召喚後伊勢姬升級不會更新既有亡魂。與本體共用同一個單位 B23D_ghost。" },
          { policy = "note", ref = "npc/npc_units_custom_load.txt:2071-2136",
            text = "模型縮放 0.8、DOTA_HULL_SIZE_SMALL 屬呈現與碰撞設定。" },
        },
        provenance = { "npc/npc_units_custom_load.txt:2071-2136", "vscripts/heroes/B_Unified/B23.lua:47-55" } },
    },
  }
end
