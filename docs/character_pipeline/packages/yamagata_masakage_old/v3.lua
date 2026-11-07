-- B20 山縣昌景（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["B20"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:299）；自動學習 heromap_autoskill["11"] = ""
-- （addon_game_mode.lua:402），沒有 D。W/E/R/T → Q/W/E/R。
-- B20W_old／B20E_old／B20T_old 的 RunScript 呼叫的是和本體相同的 B20W_OnSpellStart／B20E_OnSpellStart／B20T_OnSpellStart
-- （vscripts/heroes/B_Unified/B20.lua），腳本以 GetSpecialValueFor 讀同名鍵，所以行為與本體相同、數值取舊版 KV。
-- B20R_old 是舊版獨有的暴擊被動（B20R_old_OnSuccess／B20R_old_OnAttackLanded，B20.lua:172-194）。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 B20*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、B20_precache 屬規則層或資產，不列入。
-- 慣例照本體 B20：引擎內建 modifier_stunned 寫 dispel = "strong"；KV 未寫 IsPurgable 的 datadriven modifier 寫 "basic"；
-- HERO | BASIC 寫成 { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B20_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B20", text = { zh_TW = { name = "山縣昌景（舊版）" }, en = { name = "Yamagata Masakage (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b20_old_rock_rush", source_id = "B20W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B20.txt:293" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B20.txt:294" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B20.txt:292" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B20.txt:291" },
          radius = { 400, unit = "dist", source = "kv:B20W_radius@npc/B_Unified/B20.txt:321" },
          damage = { 280, 360, 440, 520, unit = "dmg", damage_type = "magical", source = "kv:B20W_damage@npc/B_Unified/B20.txt:316" },
          stun = { 1, unit = "s", source = "kv:B20W_stunDuration@npc/B_Unified/B20.txt:311" },
          waves = { 10, unit = "count", source = "script:vscripts/heroes/B_Unified/B20.lua:14 count >= 10" },
          wave_interval = { "1/15", unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B20.lua:37 return 1/15" },
          flight_time = { 0.3, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B20.lua:24-34 count2 > 0.3，每 0.01 秒加 0.01" },
          hit_damage = { expr = "$damage / $waves", unit = "dmg", damage_type = "magical" },
        },
        text = {
          zh_TW = { name = "岩衝",
            body = "朝目標地點連續投出 {waves} 顆岩石，每顆落地時對 {radius} 範圍內非魔免的敵方英雄與部隊造成 {hit_damage} 魔法傷害並暈眩 {stun} 秒（全部命中共 {damage}）。" },
          en = { name = "Rock Rush",
            body = "Hurls {waves} rocks in quick succession at the target point; each one that lands deals {hit_damage} magic damage to non-spell-immune enemy heroes and units within {radius} and stuns them for {stun} seconds ({damage} in total if all hit)." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b20w_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b20_old_rock_rush}的岩石暈眩 {stun} 秒；每顆落地的岩石都會重新計時。" },
              en = { name = "Stunned", body = "Stunned by {@b20_old_rock_rush} rocks for {stun} seconds; each rock that lands restarts the timer." },
            },
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned（AddNewModifier）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b20w_old_pick_sound", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "repeat", interval = "$wave_interval", first = "0", count = "$waves", actions = {
              { op = "cue", id = "b20w_old_rock", at = "caster", to = "at", duration = "$flight_time" },
              { op = "delay", time = "$flight_time", capture = { "at" }, actions = {
                { op = "cue", id = "b20w_old_rock_land", at = "at", radius = "$radius" },
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "apply_status", status = "b20w_old_stun", to = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$hit_damage", require_source_alive = false },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B20.txt:280-324",
            text = "和本體不同的只有數值：冷卻固定 18 秒、耗魔 100/120/140/160、總傷害 280/360/440/520；腳本與本體共用 B20W_OnSpellStart。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B20.lua:22-35",
            text = "每顆岩石的飛行用 0.01 秒的 Timer 累加 count2 到超過 0.3 才落地；Timer 實際每個 tick 才跑一次，30Hz 下飛行約一秒。照設計意圖寫成 0.3 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:47-67",
            text = "傷害拆成十次 damage/10，每次都重新套一秒暈眩；落點固定是施放目標點，與昌景之後的移動無關。選取旗標 NONE 且腳本另排除魔免。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B20.lua:17-34",
            text = "岩石的拋物線粒子從昌景每一波當下的位置出發，只是呈現。" },
        },
        provenance = { "npc/B_Unified/B20.txt:279-324", "vscripts/heroes/B_Unified/B20.lua:4-68" },
      },
      -------------------------------------------------------------------------
      { id = "b20_old_ruin_wave", source_id = "B20E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 25, 22, 19, 16, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B20.txt:338" },
          mana_cost = { 130, 160, 190, 220, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B20.txt:340" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B20.txt:339" },
          radius = { 600, unit = "dist", source = "kv:B20E_radius@npc/B_Unified/B20.txt:372" },
          damage = { 150, 250, 300, 450, unit = "dmg", damage_type = "magical", source = "kv:B20E_damage@npc/B_Unified/B20.txt:367" },
          stun = { 0.2, 0.3, 0.4, 0.5, unit = "s", source = "kv:B20E_stunDuration@npc/B_Unified/B20.txt:362" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B20.lua:90 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "壞毀波",
            body = "衝擊地面，把周圍 {radius} 內的敵方英雄與部隊（含魔免）全部拉到身旁；非魔免的敵人受到 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Ruin Wave",
            body = "Slams the ground, pulling every enemy hero and unit (spell-immune ones included) within {radius} to Masakage's side; non-spell-immune enemies take {damage} magic damage and are stunned for {stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b20e_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "原作 Dota 內建 modifier_phased。" },
          { id = "b20e_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b20_old_ruin_wave}暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by {@b20_old_ruin_wave} for {stun} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned（AddNewModifier）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b20e_old_burrow", at = "caster" },
            { op = "cue", id = "b20e_old_thunderclap", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "teleport", who = "it", to = "caster" },
                { op = "apply_status", status = "b20e_old_phased", to = "it" },
                { op = "if", cond = "not it.is_magic_immune", actions = {
                  { op = "apply_status", status = "b20e_old_stun", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B20.txt:326-375",
            text = "和本體不同的只有數值：冷卻 25/22/19/16、耗魔 130/160/190/220、傷害 150/250/300/450（第三級只比第二級多 50，照 KV）、暈眩 0.2/0.3/0.4/0.5；腳本與本體共用 B20E_OnSpellStart。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:85-104",
            text = "選取帶 MAGIC_IMMUNE_ENEMIES：魔免敵人也會被瞬間拉到昌景身邊（FindClearSpaceForUnit），只是不受傷害與暈眩。_G.EXCLUDE_TARGET_NAME 名單內的單位不被拉，由規則層的 untargetable_by_spells 排除。" },
          { policy = "note", ref = "npc/B_Unified/B20.txt:341",
            text = "AbilityCastRange 600 等於 radius，只是指示。" },
        },
        provenance = { "npc/B_Unified/B20.txt:326-375", "vscripts/heroes/B_Unified/B20.lua:71-105" },
      },
      -------------------------------------------------------------------------
      { id = "b20_old_tenacity", source_id = "B20R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_chance = { 36, unit = "pct", source = "kv:modifier_B20R_old_Crit OnAttackStart Random Chance@npc/B_Unified/B20.txt:404",
            note = "datadriven Random Chance，直接百分比" },
          crit_multiplier = { 220, 240, 260, 280, unit = "pct", source = "kv:B20R_old_CritScale@npc/B_Unified/B20.txt:446" },
          bonus_damage = { 5, 10, 15, 20, unit = "dmg", damage_type = "pure", source = "kv:B20R_old_damageBonus@npc/B_Unified/B20.txt:451" },
        },
        text = {
          zh_TW = { name = "強韌",
            body = "被動：每次普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害；暴擊命中非建築目標時另外造成 {bonus_damage} 純粹傷害。" },
          en = { name = "Tenacity",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage; a critical strike that hits a non-building target also deals {bonus_damage} pure damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { actions = {
              { op = "remove_status", from = "self", status = "b20r_old_crit" },
              { op = "chance", p = "$crit_chance", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                { op = "apply_status", status = "b20r_old_crit", to = "self" },
              } },
            } },
            on_attack_hit = { filter = "has_status(self, b20r_old_crit)", actions = {
              { op = "remove_status", from = "self", status = "b20r_old_crit" },
              { op = "if", cond = "not event.target.is_building", actions = {
                { op = "damage", to = "event.target", type = "pure", amount = "$bonus_damage" },
              } },
            } },
          },
        },
        statuses = {
          { id = "b20r_old_crit", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 modifier_B20R_old_Crit2：IsHidden，PREATTACK_CRITICALSTRIKE；OnAttackStart 擲中時套上、OnAttackLanded 或下一次 OnAttackStart 移除。暴擊倍率改由 modify_attack 記在這次攻擊，狀態只標記「這次攻擊是暴擊」供命中時加純粹傷害。datadriven 未寫 IsPurgable，但只存在一次出手到命中之間，寫 none。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B20.txt:378-455",
            text = "舊版強韌和本體完全不同：沒有魔抗與施法回魔回血，而是暴擊被動。datadriven Random Chance 36 推定是獨立機率（沒有寫 PseudoRandom）。" },
          { policy = "design_intent", ref = "npc/B_Unified/B20.txt:391-438",
            text = "原作在 OnAttackStart 才掛暴擊 modifier，能否套到本次攻擊依引擎時序（datadriven 常見寫法，實際會套到）；照設計意圖，暴擊直接記在這一次攻擊（modify_attack）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:178-194",
            text = "純粹傷害只在暴擊命中時、且目標不是建築時造成（ApplyDamage PURE，經 AMHC 以外的路徑；昌景死亡時不可能命中，不寫 require_source_alive）。暴擊本身對建築照常生效。" },
          { policy = "as_source", ref = "npc/B_Unified/B20.txt:391-398",
            text = "每次出手先移除上一筆暴擊標記：暴擊那次出手被取消（沒有命中）時，標記在下一次出手被清掉，不會延到下一次。" },
        },
        provenance = { "npc/B_Unified/B20.txt:377-456", "vscripts/heroes/B_Unified/B20.lua:172-194" },
      },
      -------------------------------------------------------------------------
      { id = "b20_old_red_charge", source_id = "B20T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B20.txt:468" },
          mana_cost = { 230, 280, 330, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B20.txt:469" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          duration = { 14, unit = "s", source = "kv:B20T_duration@npc/B_Unified/B20.txt:518" },
          attack_speed = { 35, 45, 55, unit = "atk_speed", source = "kv:B20T_asBonus@npc/B_Unified/B20.txt:533" },
          move_speed = { 20, 30, 40, unit = "pct", source = "kv:B20T_msBonus@npc/B_Unified/B20.txt:548" },
          pulse_tick = { 0.5, unit = "s", source = "kv:modifier_B20T ThinkInterval@npc/B_Unified/B20.txt:490" },
          radius = { 600, unit = "dist", source = "kv:B20T_radius@npc/B_Unified/B20.txt:523" },
          damage = { 90, 130, 170, unit = "dmg", damage_type = "magical", source = "kv:B20T_damage@npc/B_Unified/B20.txt:528" },
          slow = { 30, 40, 50, unit = "pct", kv = { -30, -40, -50 }, source = "kv:B20T_msasDebuff@npc/B_Unified/B20.txt:538",
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          attack_slow = { 30, 40, 50, unit = "atk_speed", kv = { -30, -40, -50 }, source = "kv:B20T_msasDebuff@npc/B_Unified/B20.txt:538",
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號；同一個鍵同時給移速百分比與攻速常數" },
          slow_time = { 1, unit = "s", source = "kv:B20T_debuffDuration@npc/B_Unified/B20.txt:543" },
        },
        text = {
          zh_TW = { name = "赤備猛攻",
            body = "{duration} 秒內攻擊速度 +{attack_speed}、移動速度 +{move_speed}，並立即與之後每 {pulse_tick} 秒敲擊地面，使周圍 {radius} 內的敵方英雄與部隊（含魔免）移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {slow_time} 秒；非魔免的敵人另外受到 {damage} 魔法傷害。" },
          en = { name = "Red Regiment Onslaught",
            body = "For {duration} seconds gains +{attack_speed} attack speed and +{move_speed} movement speed, and strikes the ground immediately and every {pulse_tick} seconds, reducing the movement speed of enemy heroes and units (spell-immune ones included) within {radius} by {slow} and their attack speed by {attack_slow} for {slow_time} seconds; non-spell-immune enemies also take {damage} magic damage." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b20t_old_onslaught", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "赤備猛攻", body = "攻擊速度 +{attack_speed}、移動速度 +{move_speed}；每 {pulse_tick} 秒敲擊地面，使周圍 {radius} 內的敵人移動速度降低 {slow}、攻擊速度降低 {attack_slow}，非魔免的敵人另外受到 {damage} 魔法傷害。" },
              en = { name = "Red Regiment Onslaught", body = "+{attack_speed} attack speed and +{move_speed} movement speed; strikes the ground every {pulse_tick} seconds, reducing enemies' movement speed within {radius} by {slow} and attack speed by {attack_slow}, and dealing {damage} magic damage to non-spell-immune ones." },
            },
            reveal = "public", stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed" },
            triggers = {
              periodic = { interval = "$pulse_tick", actions = { { op = "call", routine = "b20t_old_pulse", args = { center = "self" } } } },
            },
            notes = "原作 modifier_B20T（B20T_old 自己的同名 modifier）：IsHidden 0、ThinkInterval 0.5，時間由腳本給 B20T_duration；沒寫 IsPurgable，暫定 basic。" },
          { id = "b20t_old_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "緩速", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {slow_time} 秒。" },
              en = { name = "Slowed", body = "Movement speed reduced by {slow} and attack speed by {attack_slow} for {slow_time} seconds." },
            },
            reveal = "public", stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" }, cue = "move_slow",
            notes = "原作 B20T_old 的 modifier_B28T_debuff（名稱抄自 B28）：IsDebuff，以 Properties 直接降移速百分比與攻速，不走 dummy.lua 緩速池；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b20t_old_onslaught", to = "caster" },
            { op = "call", routine = "b20t_old_pulse", args = { center = "caster" } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B20.txt:457-551",
            text = "和本體不同：持續 14 秒、冷卻 130/120/110、耗魔 230/280/330、攻速 +35/45/55，另外加移速 20/30/40%；敵人的減益同時降移速與攻速（B20T_msasDebuff），以 datadriven Properties 表達，不經緩速池。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:134-170",
            text = "施放時立刻敲一次，之後每 0.5 秒一次。減益以 ApplyDataDrivenModifier 套用，魔免敵人也會被緩速，只是不受傷害。腳本和本體共用 B20T_OnSpellStart／B20T_OnIntervalThink。" },
          { policy = "design_intent", ref = "npc/B_Unified/B20.txt:490",
            text = "ThinkInterval 0.5、持續 14 秒：最後一跳是否落在第十四秒依引擎時序；照 KV 寫成 periodic。" },
          { policy = "note", ref = "npc/B_Unified/B20.txt:500",
            text = "減益 modifier 名為 modifier_B28T_debuff，是從 B28 複製來的命名錯誤，不影響結算。KV 沒寫 RequiredLevel，照引擎大招預設 6/12/18。" },
        },
        provenance = { "npc/B_Unified/B20.txt:457-551", "vscripts/heroes/B_Unified/B20.lua:134-170" },
      },
    },
    routines = {
      b20t_old_pulse = {
        params = { center = "unit" },
        actions = {
          { op = "cue", id = "b20t_old_epicenter", at = "center", radius = "$b20_old_red_charge.radius" },
          { op = "for_each",
            select = { from = "center", shape = "circle", radius = "$b20_old_red_charge.radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
            actions = {
              { op = "apply_status", status = "b20t_old_slow", to = "it", source = "center" },
              { op = "if", cond = "not it.is_magic_immune", actions = {
                { op = "damage", to = "it", source = "center", type = "magical", amount = "$b20_old_red_charge.damage", require_source_alive = false },
              } },
            } },
        },
      },
    },
  }
end
